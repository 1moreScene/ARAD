//! `.ani` 动画定义解析（官方纯文本标签格式）。
//!
//! 官方 `character/swordman/animation/move.ani` 形如：
//!
//! ```text
//! [LOOP] 1
//! [SHADOW] 1
//! [FRAME MAX] 8
//!
//! [FRAME000]
//! [IMAGE] `Character/Swordman/sm_body.img` 180
//! [IMAGE POS] -232 -333
//! [DELAY] 100
//! ```
//!
//! 官方标签不区分大小写，值可带反引号；`//` 之后为注释。

use std::collections::HashMap;

/// 40JP parser maps these exact tokens to frame+52: 0 / 1 / 2.
/// UNBREAKABLE excludes the frame from receiving hits; it is not merely armor.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DamageType {
    #[default]
    Normal,
    SuperArmor,
    Unbreakable,
}
impl DamageType {
    pub fn from_token(token: &str) -> Self {
        match token.trim().trim_matches('`').to_ascii_uppercase().as_str() {
            "SUPERARMOR" => Self::SuperArmor,
            "UNBREAKABLE" => Self::Unbreakable,
            _ => Self::Normal, // Official parser only recognizes three tokens.
        }
    }
}

/// 一帧。
#[derive(Clone, Debug, PartialEq)]
pub struct AnimFrame {
    /// 贴图文件（如 `Character/Swordman/sm_body.img`）。
    pub image: String,
    /// 贴图内帧号。
    pub image_index: i32,
    /// 该帧时长（毫秒）。
    pub delay_ms: u32,
    /// `[IMAGE POS]`。
    pub image_pos: (i32, i32),
    pub image_rate: (f32, f32),
    pub image_rotate: f32,
    pub rgba: [u8; 4],
    pub damage_type: String,
    /// 该帧内的判定盒（`[DAMAGE BOX]`，每行 6 个整数）。
    pub damage_boxes: Vec<[i32; 6]>,
    /// Offensive boxes, kept distinct from DAMAGE BOX hurt boxes.
    pub attack_boxes: Vec<[i32; 6]>,
    /// Ordered per-frame tags for future hit, sound and effect callbacks.
    pub events: Vec<(String, String)>,
}
impl Default for AnimFrame {
    fn default() -> Self {
        Self {
            image: String::new(),
            image_index: 0,
            delay_ms: 0,
            image_pos: (0, 0),
            image_rate: (1.0, 1.0),
            image_rotate: 0.0,
            rgba: [255; 4],
            damage_type: "NORMAL".into(),
            damage_boxes: Vec::new(),
            attack_boxes: Vec::new(),
            events: Vec::new(),
        }
    }
}

/// 一个 `.ani` 文件。
#[derive(Clone, Debug, Default)]
pub struct AnimFile {
    pub looping: bool,
    pub shadow: bool,
    pub frame_max: usize,
    pub frames: Vec<AnimFrame>,
    /// 其它未消费的顶层标签原文（保留以便后续扩展）。
    pub raw_top: HashMap<String, String>,
}

impl AnimFile {
    /// 帧时长序列（供 `dnf-core::Clip`）。
    pub fn delays(&self) -> Vec<u32> {
        self.frames.iter().map(|f| f.delay_ms).collect()
    }

    /// 单圈总时长（毫秒；含官方「保持帧」时该值会很大 —— 那是刻意的）。
    pub fn total_ms(&self) -> u64 {
        self.frames.iter().map(|f| f.delay_ms as u64).sum()
    }

    /// 忽略保持帧的单圈时长。
    pub fn cycle_ms(&self) -> u64 {
        self.frames
            .iter()
            .filter(|f| f.delay_ms < 10_000)
            .map(|f| f.delay_ms as u64)
            .sum()
    }

    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }
}

fn strip_comment(line: &str) -> &str {
    match line.find("//") {
        Some(i) => &line[..i],
        None => line,
    }
}

fn split_tag(line: &str) -> Option<(String, &str)> {
    let line = line.trim();
    if !line.starts_with('[') {
        return None;
    }
    let end = line.find(']')?;
    let tag = line[1..end].trim().to_string();
    Some((tag, line[end + 1..].trim()))
}

fn backticked(s: &str) -> Option<&str> {
    let a = s.find('`')?;
    let rest = &s[a + 1..];
    let b = rest.find('`')?;
    Some(&rest[..b])
}

/// 最后一个反引号之后的内容（行尾的裸数值都在这儿）。
fn tail_after_backticks(s: &str) -> &str {
    match s.rfind('`') {
        Some(i) => &s[i + 1..],
        None => "",
    }
}

fn ints(s: &str) -> Vec<i32> {
    let cleaned: String = s
        .chars()
        .map(|c| {
            if c.is_ascii_digit() || c == '-' || c == '+' {
                c
            } else {
                ' '
            }
        })
        .collect();
    cleaned
        .split_whitespace()
        .filter_map(|t| t.parse::<i32>().ok())
        .collect()
}

fn first_int(s: &str) -> Option<i32> {
    ints(s).first().copied()
}

/// 解析 `.ani` 文本。
pub fn parse(text: &str) -> AnimFile {
    let mut file = AnimFile::default();
    let mut current: Option<AnimFrame> = None;

    for raw in text.lines() {
        let line = strip_comment(raw);
        let Some((tag, rest)) = split_tag(line) else {
            continue;
        };
        let upper = tag.to_ascii_uppercase();

        // `[FRAME000]`
        if let Some(num) = upper.strip_prefix("FRAME") {
            if !num.is_empty() && num.chars().all(|c| c.is_ascii_digit()) {
                if let Some(f) = current.take() {
                    file.frames.push(f);
                }
                current = Some(AnimFrame::default());
                continue;
            }
        }

        match upper.as_str() {
            "LOOP" => file.looping = first_int(rest).unwrap_or(0) != 0,
            "SHADOW" => file.shadow = first_int(rest).unwrap_or(0) != 0,
            "FRAME MAX" => {
                let n = first_int(rest).unwrap_or(0).max(0) as usize;
                file.frame_max = n;
            }
            "IMAGE" => {
                if let Some(f) = current.as_mut() {
                    if let Some(p) = backticked(rest) {
                        f.image = p.to_string();
                    }
                    // 帧号必须取**最后一个反引号之后**的数字：路径本身也含数字
                    // （如 `Map\01far1.img`、`sm_coat0000a.img`），直接从整行取会取错。
                    f.image_index = first_int(tail_after_backticks(rest)).unwrap_or(0);
                }
            }
            "IMAGE POS" => {
                if let Some(f) = current.as_mut() {
                    let v = ints(rest);
                    if v.len() >= 2 {
                        f.image_pos = (v[0], v[1]);
                    }
                }
            }
            "DELAY" => {
                if let Some(f) = current.as_mut() {
                    f.delay_ms = first_int(rest).unwrap_or(0).max(0) as u32;
                }
            }
            "IMAGE RATE" => {
                if let Some(f) = current.as_mut() {
                    let v = rest
                        .split_whitespace()
                        .filter_map(|s| s.parse::<f32>().ok())
                        .filter(|v| v.is_finite())
                        .collect::<Vec<_>>();
                    if v.len() >= 2 {
                        f.image_rate = (v[0], v[1]);
                    }
                }
            }
            "IMAGE ROTATE" => {
                if let Some(f) = current.as_mut() {
                    f.image_rotate = rest
                        .trim()
                        .parse::<f32>()
                        .ok()
                        .filter(|v| v.is_finite())
                        .unwrap_or(0.0);
                }
            }
            "RGBA" => {
                if let Some(f) = current.as_mut() {
                    let v = ints(rest);
                    if v.len() >= 4 {
                        f.rgba = std::array::from_fn(|i| v[i].clamp(0, 255) as u8);
                    }
                }
            }
            "DAMAGE TYPE" => {
                if let Some(f) = current.as_mut() {
                    f.damage_type = rest.trim_matches('`').to_ascii_uppercase();
                }
            }
            "DAMAGE BOX" | "ATTACK BOX" => {
                if let Some(f) = current.as_mut() {
                    let v = ints(rest);
                    if v.len() >= 6 {
                        let bounds = [v[0], v[1], v[2], v[3], v[4], v[5]];
                        if upper == "ATTACK BOX" {
                            f.attack_boxes.push(bounds);
                        } else {
                            f.damage_boxes.push(bounds);
                        }
                    }
                }
            }
            _ => {
                if current.is_none() {
                    file.raw_top.insert(upper.clone(), rest.to_string());
                }
            }
        }
        if !upper.starts_with("FRAME") {
            if let Some(f) = current.as_mut() {
                f.events.push((upper, rest.to_string()));
            }
        }
    }
    if let Some(f) = current.take() {
        file.frames.push(f);
    }
    if file.frame_max == 0 {
        file.frame_max = file.frames.len();
    }
    file
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_official_damage_tokens_activate_protection() {
        assert_eq!(
            DamageType::from_token(" `superarmor` "),
            DamageType::SuperArmor
        );
        assert_eq!(
            DamageType::from_token("UNBREAKABLE"),
            DamageType::Unbreakable
        );
        for token in [
            "NORMAL",
            "SUPERARMO",
            "Invisible",
            "Invincible",
            "NONE",
            "LINEARDODGE",
        ] {
            assert_eq!(DamageType::from_token(token), DamageType::Normal, "{token}");
        }
    }
    #[test]
    fn scripted_transform_and_damage_type_are_preserved() {
        let a=parse("[FRAME MAX] 1\n[FRAME000]\n[IMAGE] `x.img` 0\n[IMAGE RATE] 1.25 -0.75\n[IMAGE ROTATE] 3.140022\n[RGBA] 120 200 255 80\n[DAMAGE TYPE] `SUPERARMOR`\n[DELAY] 10000");
        let f = &a.frames[0];
        assert_eq!(f.image_rate, (1.25, -0.75));
        assert_eq!(f.rgba, [120, 200, 255, 80]);
        assert!((f.image_rotate - 3.140022).abs() < 1e-5);
        assert_eq!(f.damage_type, "SUPERARMOR");
    }

    /// 官方 `character/swordman/animation/move.ani` 的真值（从 `Script.pvf` 解出）。
    pub const MOVE_ANI: &str = r#"[LOOP] 1
[SHADOW] 1
[FRAME MAX] 8

[FRAME000]
[IMAGE] `Character/Swordman/sm_body.img` 180
[IMAGE POS] -232 -333
[IMAGE RATE] 1.000000 1.000000
[IMAGE ROTATE] 0.000000
[RGBA] 255 255 255 255
[INTERPOLATION] 0
[GRAPHIC EFFECT] `NONE`
[DELAY] 100
[DAMAGE TYPE] `NORMAL`
[DAMAGE BOX] -14 -5 0 27 10 107

[FRAME001]
[IMAGE] `Character/Swordman/sm_body.img` 181
[IMAGE POS] -232 -333
[DELAY] 100
"#;

    #[test]
    fn 解析move_ani真值() {
        let a = parse(MOVE_ANI);
        assert!(a.looping);
        assert!(a.shadow);
        assert_eq!(a.frame_max, 8);
        assert_eq!(a.frame_count(), 2);
        assert_eq!(a.frames[0].image, "Character/Swordman/sm_body.img");
        assert_eq!(a.frames[0].image_index, 180);
        assert_eq!(a.frames[0].image_pos, (-232, -333));
        assert_eq!(a.frames[0].delay_ms, 100);
        assert_eq!(a.frames[0].damage_boxes, vec![[-14, -5, 0, 27, 10, 107]]);
        assert_eq!(a.frames[1].image_index, 181);
    }

    #[test]
    fn 解析jump_ani的保持帧() {
        // 官方 jump.ani 的 DELAY 序列（真值）
        let delays = [
            100u32, 100, 80, 80, 80, 80, 80, 10000, 100, 80, 80, 80, 80, 80, 10000, 50,
        ];
        let mut text = String::from("[LOOP] 0\n[SHADOW] 1\n[FRAME MAX] 16\n");
        for (i, d) in delays.iter().enumerate() {
            text.push_str(&format!(
                "\n[FRAME{:03}]\n[IMAGE] `Character/Swordman/sm_body.img` {}\n[IMAGE POS] -232 -333\n[DELAY] {}\n",
                125 + i,
                125 + i,
                d
            ));
        }
        let a = parse(&text);
        assert_eq!(a.delays(), delays.to_vec());
        assert!(!a.looping);
        // 官方数据：上升段（帧 0..6）600ms、下降段（帧 8..13）500ms、落地帧 50ms
        assert_eq!(a.delays()[0..7].iter().sum::<u32>(), 600);
        assert_eq!(a.delays()[8..14].iter().sum::<u32>(), 500);
        assert_eq!(a.delays()[15], 50);
    }

    #[test]
    fn 注释与反引号处理() {
        let a = parse("[FRAME000]\n[IMAGE] `a/b.img` 7 // 尾巴\n[DELAY] 33 // 33ms\n");
        assert_eq!(a.frames[0].image, "a/b.img");
        assert_eq!(a.frames[0].image_index, 7);
        assert_eq!(a.frames[0].delay_ms, 33);
    }

    #[test]
    fn 路径含数字时帧号仍取反引号之后() {
        // 官方地图动画：`` [IMAGE] `Map\01far1.img` 0 ``
        // 回归：曾从整行取数字（"01far1" → 01 = 1），导致帧号 1 越界、背景整层消失。
        let a = parse("[FRAME000]\n[IMAGE] `Map\\01far1.img` 0\n[IMAGE POS] 0 0\n");
        assert_eq!(a.frames[0].image, "Map\\01far1.img");
        assert_eq!(a.frames[0].image_index, 0);
        let b = parse("[FRAME000]\n[IMAGE] `Character/Swordman/sm_body.img` 180\n");
        assert_eq!(b.frames[0].image_index, 180);
    }

    #[test]
    fn 官方地图背景动画真值() {
        let Some(pvf) = crate::test_support::pvf() else {
            return;
        };
        let a = parse(&pvf.read_text("map/lorien/animation/far1.ani").unwrap());
        assert_eq!(a.frame_count(), 1);
        assert_eq!(a.frames[0].image, "Map\\01far1.img");
        assert_eq!(a.frames[0].image_index, 0);
        assert_eq!(a.frames[0].image_pos, (0, 0));
        let b = parse(&pvf.read_text("map/lorien/animation/mid1.ani").unwrap());
        assert_eq!(b.frames[0].image, "Map\\01mid1.img");
        assert_eq!(b.frames[0].image_index, 0);
    }
}
