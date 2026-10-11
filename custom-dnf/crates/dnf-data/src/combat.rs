//! Prepared normal/air attack resources. No hit timing or damage is guessed.
use crate::{ani, chr::ChrFile, pvf::PvfReader, AnimFile};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub struct HitInfo {
    pub kind: String,
    pub blood: bool,
    pub stop_ms: i32,
    pub recovery_rate: f32,
}
fn words(raw: &str) -> Vec<String> {
    raw.split_whitespace()
        .map(|s| {
            s.trim_matches('`')
                .trim_matches(|c| c == '[' || c == ']')
                .to_string()
        })
        .collect()
}
pub fn parse_hit_info(raw: &str) -> Option<HitInfo> {
    let tokens = words(raw);
    // [no blood] contains a space, unlike [blood]. Preserve that token separately.
    let no_blood = raw.contains("[no blood]");
    let offset = if no_blood { 1 } else { 0 };
    Some(HitInfo {
        kind: tokens.first()?.clone(),
        blood: !no_blood && tokens.get(1)? == "blood",
        stop_ms: tokens.get(2 + offset)?.parse().ok()?,
        recovery_rate: tokens.get(3 + offset)?.parse().ok()?,
    })
}
pub fn weapon_hit_info(chr_text: &str, index: usize) -> Option<HitInfo> {
    let mut inside = false;
    let mut rows = Vec::new();
    for line in chr_text.lines() {
        let line = line.split("//").next().unwrap_or("").trim();
        if line == "[weapon hit info]" {
            inside = true;
            continue;
        }
        if line.starts_with("[/weapon hit info]") {
            break;
        }
        if inside {
            if let Some(info) = parse_hit_info(line) {
                rows.push(info);
            }
        }
    }
    rows.get(index).cloned()
}

#[derive(Clone, Debug, Default)]
pub struct AttackInfo {
    /// Keep every tag for later interpretation; omitted fields stay unknown.
    pub fields: HashMap<String, Vec<String>>,
    pub raw_text: String,
}
impl AttackInfo {
    pub fn feedback(&self, weapon: &HitInfo) -> HitInfo {
        let mut info = self
            .fields
            .get("hit info")
            .and_then(|v| v.last())
            .and_then(|s| parse_hit_info(s))
            .unwrap_or_else(|| weapon.clone());
        if info.stop_ms < 0 {
            info.stop_ms = weapon.stop_ms;
        }
        if info.recovery_rate < 0.0 {
            info.recovery_rate = weapon.recovery_rate;
        }
        info
    }
    pub fn number(&self, tag: &str) -> Option<i32> {
        self.fields
            .get(&tag.to_ascii_lowercase())?
            .last()?
            .split_whitespace()
            .next()?
            .parse()
            .ok()
    }
    pub fn token(&self, tag: &str) -> Option<&str> {
        let raw = self.fields.get(&tag.to_ascii_lowercase())?.last()?.trim();
        Some(raw.trim_matches('`').trim_matches(|c| c == '[' || c == ']'))
    }
}
pub fn parse_attack_info(text: &str) -> AttackInfo {
    let mut out = AttackInfo {
        raw_text: text.to_string(),
        ..AttackInfo::default()
    };
    for line in text.lines() {
        let line = line.split("//").next().unwrap_or("").trim();
        if !line.starts_with('[') {
            continue;
        }
        if let Some(end) = line.find(']') {
            let tag = line[1..end].trim().to_ascii_lowercase();
            if !tag.starts_with('/') {
                out.fields
                    .entry(tag)
                    .or_default()
                    .push(line[end + 1..].trim().to_string());
            }
        }
    }
    out
}

#[derive(Clone, Debug)]
pub struct AttackAsset {
    pub animation_path: String,
    pub animation: AnimFile,
    pub info_path: String,
    pub info: AttackInfo,
}
#[derive(Clone, Debug)]
pub struct PreparedCombat {
    pub normal: Vec<AttackAsset>,
    pub jump_attack: AttackAsset,
    pub dash_attack: AttackAsset,
    pub upper_slash: AttackInfo,
    pub hard_attack: AttackAsset,
}

fn load_asset(
    pvf: &PvfReader,
    directory: &str,
    animation: &str,
    info: &str,
) -> Result<AttackAsset, String> {
    let path = |relative: &str| {
        format!(
            "{}/{}",
            directory.trim_end_matches('/'),
            relative.replace('\\', "/")
        )
        .to_ascii_lowercase()
    };
    let animation_path = path(animation);
    let info_path = path(info);
    let animation = ani::parse(
        &pvf.read_text(&animation_path)
            .map_err(|e| format!("{animation_path}: {e}"))?,
    );
    if animation.frames.is_empty() {
        return Err(format!("{animation_path}: empty animation"));
    }
    let info = parse_attack_info(
        &pvf.read_text(&info_path)
            .map_err(|e| format!("{info_path}: {e}"))?,
    );
    Ok(AttackAsset {
        animation_path,
        animation,
        info_path,
        info,
    })
}

impl PreparedCombat {
    pub fn load(pvf: &PvfReader, chr: &ChrFile, directory: &str) -> Result<Self, String> {
        let motions = chr.resource_list("attack motion");
        let infos = chr.resource_list("attack info");
        if motions.is_empty() || motions.len() != infos.len() {
            return Err(format!(
                "attack motion/info arrays mismatch: {}/{}",
                motions.len(),
                infos.len()
            ));
        }
        let normal = motions
            .iter()
            .zip(infos)
            .map(|(m, i)| load_asset(pvf, directory, m, i))
            .collect::<Result<Vec<_>, _>>()?;
        let jump_motion = chr
            .motion("jumpattack motion")
            .ok_or("missing jumpattack motion")?;
        let jump_info = chr
            .attack_infos
            .get("jumpattack info")
            .ok_or("missing jumpattack info")?;
        let mut upper_slash = parse_attack_info(
            &pvf.read_text(&format!("{directory}/attackinfo/upperslash.atk"))
                .map_err(|e| e.to_string())?,
        );
        let skill = pvf
            .read_text("skill/swordman/upperslash.skl")
            .map_err(|e| e.to_string())?;
        let definition =
            crate::skill::SkillDefinition::parse(46, "skill/swordman/upperslash.skl", &skill)?;
        let first = definition.at_level(1).ok_or("missing upper level1")?.values;
        for (tag, value) in ["skill additional damage", "lift up", "push aside"]
            .into_iter()
            .zip(first)
        {
            upper_slash
                .fields
                .insert(tag.into(), vec![value.to_string()]);
        }
        Ok(Self {
            hard_attack: load_asset(
                pvf,
                directory,
                chr.resource_list("etc motion")
                    .get(1)
                    .ok_or("missing HardAttack motion")?,
                chr.resource_list("etc attack info")
                    .first()
                    .ok_or("missing HardAttack info")?,
            )?,
            normal,
            jump_attack: load_asset(pvf, directory, jump_motion, jump_info)?,
            dash_attack: load_asset(
                pvf,
                directory,
                chr.motion("dashattack motion")
                    .ok_or("missing dashattack motion")?,
                chr.attack_infos
                    .get("dashattack info")
                    .ok_or("missing dashattack info")?,
            )?,
            upper_slash,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hit_info_inherits_and_overrides_weapon_script_values() {
        let weapon = parse_hit_info("`[cut]` `[blood]` 100 1.00").unwrap();
        assert_eq!(
            parse_hit_info("`[blow]` `[no blood]` 110 1.05")
                .unwrap()
                .stop_ms,
            110
        );
        assert_eq!(
            parse_attack_info("[damage bonus] 10").feedback(&weapon),
            weapon
        );
        let override_info =
            parse_attack_info("[hit info] `[cut]` `[blood]` 70 0.86").feedback(&weapon);
        assert_eq!(override_info.stop_ms, 70);
        assert_eq!(override_info.recovery_rate, 0.86);
        assert_eq!(
            parse_attack_info("[hit info] `[cut]` `[blood]` -1 -1").feedback(&weapon),
            weapon
        );
    }
    #[test]
    fn official_swordman_attack_assets_are_complete() {
        let Some(pvf) = crate::test_support::pvf() else {
            return;
        };
        let chr = crate::chr::parse(&pvf.read_text("character/swordman/swordman.chr").unwrap());
        let combat = PreparedCombat::load(pvf, &chr, "character/swordman").unwrap();
        assert_eq!(combat.upper_slash.number("lift up"), Some(300));
        assert_eq!(combat.upper_slash.number("push aside"), Some(100));
        assert_eq!(combat.normal.len(), 3);
        assert_eq!(combat.normal[0].animation.frame_count(), 10);
        assert_eq!(combat.normal[0].animation.total_ms(), 600);
        assert_eq!(combat.normal[0].info.number("damage bonus"), Some(-15));
        assert_eq!(combat.normal[2].info.number("lift up"), Some(300));
        assert_eq!(
            combat.jump_attack.animation.delays(),
            vec![100, 50, 50, 40, 30, 30]
        );
        assert_eq!(combat.jump_attack.info.number("damage bonus"), Some(10));
        assert_eq!(combat.jump_attack.info.number("push aside"), Some(270));
        assert_eq!(combat.jump_attack.info.number("lift up"), Some(180));
        assert_eq!(combat.jump_attack.info.token("attack type"), Some("physic"));
        assert_eq!(combat.jump_attack.info.number("not present"), None);
        assert!(
            combat.normal[0].animation.frames[0].attack_boxes.is_empty(),
            "body DAMAGE BOX cannot become attack boxes"
        );
    }
}
