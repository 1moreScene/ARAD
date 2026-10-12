//! Official audio catalog, weighted tags, SoundPacks and PCM WAVE decoding.
use crate::npk::NpkSet;
use std::collections::HashMap;
use std::path::Path;

#[derive(Clone, Debug)]
pub struct Effect {
    pub file: String,
    pub volume_adjust: Option<f32>,
    pub priority: Option<i32>,
    pub loop_delay: f32,
}
#[derive(Clone, Debug)]
pub struct GroupItem {
    pub tag: String,
    pub probability: u32,
}
#[derive(Clone, Default)]
pub struct AudioCatalog {
    pub effects: HashMap<String, Effect>,
    pub groups: HashMap<String, Vec<GroupItem>>,
}
fn attr<'a>(tag: &'a str, key: &str) -> Option<&'a str> {
    let start = tag.find(&format!("{key}=\""))? + key.len() + 2;
    let rest = &tag[start..];
    Some(&rest[..rest.find('"')?])
}
impl AudioCatalog {
    pub fn parse(xml: &str) -> Self {
        let mut out = Self::default();
        let mut group = None;
        for segment in xml.split('<').skip(1) {
            let tag = segment.split('>').next().unwrap_or("");
            if tag.starts_with("EFFECT ") {
                if let (Some(id), Some(file)) = (attr(tag, "ID"), attr(tag, "FILE")) {
                    out.effects.insert(
                        id.to_string(),
                        Effect {
                            file: file.replace('\\', "/"),
                            volume_adjust: attr(tag, "VOLUME_ADJUST").and_then(|v| v.parse().ok()),
                            priority: attr(tag, "PRIORITY").and_then(|v| v.parse().ok()),
                            loop_delay: attr(tag, "LOOP_DELAY")
                                .and_then(|v| v.parse::<f32>().ok())
                                .filter(|v| v.is_finite() && *v >= 0.0)
                                .unwrap_or(0.0),
                        },
                    );
                }
            } else if tag.starts_with("RANDOM ") {
                group = attr(tag, "ID").map(str::to_string);
                if let Some(id) = &group {
                    out.groups.entry(id.clone()).or_default();
                }
            } else if tag.starts_with("/RANDOM") {
                group = None;
            } else if tag.starts_with("ITEM ") {
                if let (Some(id), Some(child)) = (&group, attr(tag, "TAG")) {
                    out.groups.entry(id.clone()).or_default().push(GroupItem {
                        tag: child.to_string(),
                        probability: attr(tag, "PROB").and_then(|v| v.parse().ok()).unwrap_or(1),
                    });
                }
            }
        }
        out
    }
    pub fn files_for(&self, tag: &str) -> Vec<String> {
        self.expand(tag, 0)
    }
    /// Each RANDOM node draws independently; zero-weight branches never play.
    pub fn resolve<'a>(
        &'a self,
        tag: &str,
        rng: &mut impl FnMut(u64) -> u64,
    ) -> Option<&'a Effect> {
        let mut current = tag.to_string();
        for _ in 0..=8 {
            if let Some(effect) = self.effects.get(&current) {
                return Some(effect);
            }
            let items = self.groups.get(&current)?;
            let total: u64 = items.iter().map(|i| i.probability as u64).sum();
            if total == 0 {
                return None;
            }
            let mut draw = rng(total) % total;
            let selected = items.iter().find(|i| {
                if draw < i.probability as u64 {
                    true
                } else {
                    draw -= i.probability as u64;
                    false
                }
            })?;
            current.clone_from(&selected.tag);
        }
        None
    }
    fn expand(&self, tag: &str, depth: u8) -> Vec<String> {
        if depth > 8 {
            return Vec::new();
        }
        if let Some(effect) = self.effects.get(tag) {
            return vec![effect.file.clone()];
        }
        self.groups
            .get(tag)
            .into_iter()
            .flatten()
            .flat_map(|item| self.expand(&item.tag, depth + 1))
            .collect()
    }
}

#[derive(Debug)]
pub struct Pcm {
    pub sample_rate: u32,
    pub channels: u16,
    /// Interleaved normalized samples, PCM unsigned 8 or signed 16/24/32 bit.
    pub samples: Vec<f32>,
}

pub fn decode_wave(bytes: &[u8]) -> Result<Pcm, String> {
    if encoding(bytes) != Encoding::Wave {
        return Err("expected RIFF/WAVE".into());
    }
    let u16_at = |s: &[u8], i| u16::from_le_bytes([s[i], s[i + 1]]);
    let u32_at = |s: &[u8], i| u32::from_le_bytes(s[i..i + 4].try_into().unwrap());
    let end = (u32_at(bytes, 4) as usize)
        .checked_add(8)
        .ok_or("RIFF size overflow")?;
    if end > bytes.len() || end < 12 {
        return Err("truncated RIFF".into());
    }
    let mut pos = 12;
    let mut format = None;
    let mut data = None;
    while pos < end {
        if end - pos < 8 {
            return Err("truncated chunk header".into());
        }
        let size = u32_at(bytes, pos + 4) as usize;
        let next = (pos + 8).checked_add(size).ok_or("chunk size overflow")?;
        if next > end {
            return Err("truncated WAVE chunk".into());
        }
        let chunk = &bytes[pos + 8..next];
        match &bytes[pos..pos + 4] {
            b"fmt " => {
                if size < 16 {
                    return Err("short fmt chunk".into());
                }
                format = Some((
                    u16_at(chunk, 0),
                    u16_at(chunk, 2),
                    u32_at(chunk, 4),
                    u32_at(chunk, 8),
                    u16_at(chunk, 12),
                    u16_at(chunk, 14),
                ));
            }
            b"data" => {
                if data.is_none() {
                    data = Some(chunk);
                }
            }
            _ => {}
        }
        // Sound Forge originals omit an odd data pad at EOF or before LIST metadata.
        if size & 1 != 0
            && &bytes[pos..pos + 4] == b"data"
            && (next == end || bytes.get(next..next + 4) == Some(b"LIST"))
        {
            pos = next;
            continue;
        }
        pos = next.checked_add(size & 1).ok_or("chunk padding overflow")?;
        if pos > end {
            return Err("missing chunk padding".into());
        }
    }
    let (codec, channels, sample_rate, byte_rate, align, bits) = format.ok_or("missing fmt")?;
    if codec != 1 {
        return Err(format!("unsupported WAVE codec {codec}; PCM required"));
    }
    if !(1..=2).contains(&channels)
        || !(8000..=192000).contains(&sample_rate)
        || ![8, 16, 24, 32].contains(&bits)
    {
        return Err(format!(
            "unsupported PCM {channels}ch/{sample_rate}Hz/{bits}bit"
        ));
    }
    if align != channels * (bits / 8) || byte_rate != sample_rate * align as u32 {
        return Err("invalid PCM alignment/rate".into());
    }
    let data = data.ok_or("missing data")?;
    if data.is_empty() || data.len() % align as usize != 0 {
        return Err("empty or partial PCM frame".into());
    }
    let samples = data
        .chunks_exact((bits / 8) as usize)
        .map(|s| match bits {
            8 => (s[0] as f32 - 128.0) / 128.0,
            16 => i16::from_le_bytes([s[0], s[1]]) as f32 / 32768.0,
            24 => {
                let v = ((s[0] as i32) | ((s[1] as i32) << 8) | ((s[2] as i32) << 16)) << 8 >> 8;
                v as f32 / 8388608.0
            }
            32 => i32::from_le_bytes(s.try_into().unwrap()) as f32 / 2147483648.0,
            _ => unreachable!(),
        })
        .collect();
    Ok(Pcm {
        sample_rate,
        channels,
        samples,
    })
}
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Encoding {
    Ogg,
    Wave,
    Unknown,
}
pub fn encoding(bytes: &[u8]) -> Encoding {
    if bytes.starts_with(b"OggS") {
        Encoding::Ogg
    } else if bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WAVE") {
        Encoding::Wave
    } else {
        Encoding::Unknown
    }
}
pub struct PreparedAudio {
    pub catalog: AudioCatalog,
    pub packs: NpkSet,
}
pub const INITIAL_TAGS: [&str; 47] = [
    "CONDITION_SLOW",
    "FROZEN",
    "CONDITION_POSION",
    "CONDITION_STUN",
    "CONDITION_CURSE",
    "CONDITION_BLIND",
    "CONDTION_LIGHT",
    "CONDITION_STONE",
    "CONDITION_SLEEP",
    "CONDITION_BLOODING",
    "CONDITION_HASTE",
    "CONDITION_BLESS",
    "CONDITION_CONFUSE",
    "CONDITION_REMOVE",
    "STONE_DIE",
    "STONE_CUT",
    "FROZEN_DIE",
    "CONDITION_POSION_DROP",
    "CONDITION_POISON_POP",
    "CONDITION_LIGTHNING_LOOP",
    "BLOOD_DROP_LOOP",
    "SM_BLOOD_CAST",
    "BLOOD_HOLD",
    "BLOOD_SUCK",
    "BLOOD_SPREAD",
    "FIRE_PILLAR_HIT",
    "ALERT_1",
    "ICON_QUICK_SET",
    "ICON_DROP",
    "R_SM_GUE",
    "R_SM_ATK",
    "R_SM_JUMP_ATK",
    "SM_JUMP",
    "JUMP_START",
    "JUMP_END",
    "R_MINERALSWDA",
    "MINERALSWDB",
    "MINERALSWD_TOUCH",
    "WALK_SOIL_LOOP",
    "RUN_SOIL_LOOP",
    "SM_THRUST",
    "SM_UPPER_SLASH",
    "SM_NOMANA",
    "R_MINERALSWDA_HIT",
    "MINERAL_SWDB_HIT",
    "R_GOBLIN_DMG",
    "R_GOBLIN_DIE",
];
impl PreparedAudio {
    pub fn load(client: &Path) -> Result<Self, String> {
        let xml = std::fs::read(client.join("audio.xml")).map_err(|e| e.to_string())?;
        let catalog = AudioCatalog::parse(&String::from_utf8_lossy(&xml));
        let packs = NpkSet::scan_dir(&client.join("SoundPacks")).map_err(|e| e.to_string())?;
        Ok(Self { catalog, packs })
    }
    pub fn probe(&self, tag: &str) -> Result<Vec<(String, Encoding)>, String> {
        let files = self.catalog.files_for(tag);
        if files.is_empty() {
            return Err(format!("missing audio tag {tag}"));
        }
        files
            .into_iter()
            .map(|file| {
                let bytes = self.packs.read(&file).map_err(|e| e.to_string())?;
                Ok((file, encoding(&bytes)))
            })
            .collect()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn wave(bits: u16, channels: u16, data: &[u8]) -> Vec<u8> {
        let mut bytes = b"RIFF\0\0\0\0WAVEJUNK\x01\0\0\0x\0fmt ".to_vec();
        bytes.extend_from_slice(&16u32.to_le_bytes());
        bytes.extend_from_slice(&1u16.to_le_bytes());
        bytes.extend_from_slice(&channels.to_le_bytes());
        bytes.extend_from_slice(&44100u32.to_le_bytes());
        let align = channels * bits / 8;
        bytes.extend_from_slice(&(44100 * align as u32).to_le_bytes());
        bytes.extend_from_slice(&align.to_le_bytes());
        bytes.extend_from_slice(&bits.to_le_bytes());
        bytes.extend_from_slice(b"data");
        bytes.extend_from_slice(&(data.len() as u32).to_le_bytes());
        bytes.extend_from_slice(data);
        if data.len() % 2 != 0 {
            bytes.push(0);
        }
        let size = bytes.len() as u32 - 8;
        bytes[4..8].copy_from_slice(&size.to_le_bytes());
        bytes
    }
    #[test]
    fn wave_pcm_widths_stereo_and_odd_unknown_chunks() {
        assert_eq!(
            decode_wave(&wave(8, 1, &[0, 128, 255])).unwrap().samples,
            vec![-1.0, 0.0, 127.0 / 128.0]
        );
        assert_eq!(
            decode_wave(&wave(16, 2, &[0, 128, 0, 64])).unwrap().samples,
            vec![-1.0, 0.5]
        );
        assert_eq!(
            decode_wave(&wave(24, 1, &[0, 0, 128, 0, 0, 64]))
                .unwrap()
                .samples,
            vec![-1.0, 0.5]
        );
        assert_eq!(
            decode_wave(&wave(32, 1, &[0, 0, 0, 128, 0, 0, 0, 64]))
                .unwrap()
                .samples,
            vec![-1.0, 0.5]
        );
    }
    #[test]
    fn malformed_wave_is_rejected_without_panics() {
        let valid = wave(16, 1, &[0, 0, 0, 64]);
        for n in 0..valid.len() {
            assert!(decode_wave(&valid[..n]).is_err(), "accepted truncation {n}");
        }
        let mut bad = valid.clone();
        bad[30..32].copy_from_slice(&3u16.to_le_bytes());
        assert!(decode_wave(&bad).is_err()); // IEEE float is not PCM.
        assert!(decode_wave(&wave(16, 2, &[0, 0])).is_err()); // Partial stereo frame.
        let mut bad = valid;
        bad[16..20].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(decode_wave(&bad).is_err());
    }
    #[test]
    fn original_sound_forge_unpadded_data_is_supported() {
        let mut bytes = wave(8, 1, &[0, 128, 255]);
        bytes.pop(); // The original file omits the odd data padding byte.
        let size = bytes.len() as u32 - 8;
        bytes[4..8].copy_from_slice(&size.to_le_bytes());
        assert_eq!(decode_wave(&bytes).unwrap().samples.len(), 3);
        bytes.extend_from_slice(b"LIST\x04\0\0\0INFO");
        let size = bytes.len() as u32 - 8;
        bytes[4..8].copy_from_slice(&size.to_le_bytes());
        assert_eq!(
            decode_wave(&bytes).unwrap().samples,
            vec![-1.0, 0.0, 127.0 / 128.0]
        );
    }
    #[test]
    fn weighted_boundaries_zero_weights_and_cycles() {
        let catalog = AudioCatalog::parse(
            r#"<EFFECT ID="A" FILE="a.wav" VOLUME_ADJUST="-12.5"/>
            <EFFECT ID="B" FILE="b.wav"/><EFFECT ID="C" FILE="c.wav"/>
            <RANDOM ID="R"><ITEM TAG="A" PROB="33"/><ITEM TAG="B" PROB="33"/><ITEM TAG="C" PROB="34"/></RANDOM>
            <RANDOM ID="Z"><ITEM TAG="MISSING" PROB="0"/><ITEM TAG="R" PROB="1"/></RANDOM>
            <RANDOM ID="LOOP"><ITEM TAG="LOOP" PROB="1"/></RANDOM>"#,
        );
        let mut counts = [0; 3];
        for draw in 0..100 {
            let effect = catalog.resolve("R", &mut |_| draw).unwrap();
            counts[effect.file.as_bytes()[0] as usize - b'a' as usize] += 1;
        }
        assert_eq!(counts, [33, 33, 34]);
        assert_eq!(catalog.effects["A"].volume_adjust, Some(-12.5));
        assert_eq!(catalog.resolve("Z", &mut |_| 0).unwrap().file, "a.wav");
        assert!(catalog.resolve("LOOP", &mut |_| 0).is_none());
        assert!(catalog.resolve("MISSING", &mut |_| 0).is_none());
    }
    #[test]
    fn official_tags_and_audio_containers_are_ready() {
        let Some(dir) = crate::locate_client_dir() else {
            return;
        };
        let audio = PreparedAudio::load(&dir).unwrap();
        let group = &audio.catalog.groups["R_SM_ATK"];
        assert_eq!(
            group.iter().map(|i| i.probability).collect::<Vec<_>>(),
            vec![33, 33, 34]
        );
        for tag in INITIAL_TAGS {
            for (_, format) in audio.probe(tag).unwrap() {
                assert_ne!(format, Encoding::Unknown);
            }
            for file in audio.catalog.files_for(tag) {
                let pcm = decode_wave(&audio.packs.read(&file).unwrap()).unwrap();
                assert!((1..=2).contains(&pcm.channels));
                assert!((8000..=192000).contains(&pcm.sample_rate));
                assert!(pcm.samples.iter().any(|&s| s != 0.0));
            }
        }
    }
}
