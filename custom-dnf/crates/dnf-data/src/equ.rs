//! `.equ` 装备/时装定义（只取渲染需要的字段）。
//!
//! 官方 `equipment/character/swordman/avatar/coat/acoat_*.equ` 实测结构：
//!
//! ```text
//! [name2] `Noble Tattoo of The Wind Devas`
//! [equipment type] `[coat avatar]` 0
//! [variation] 19 3                      ← 两个编号，用于拼精灵路径（%02d%02d）
//! [LAYER] 500                           ← 绘制层序
//! [move motion] `coat_d/Move.ani`       ← 动作 → 该件自己的动画
//! ```
//!
//! 时装精灵路径是**模板**，例如 coat 的 `move.ani` 里写：
//! `Character/Swordman/Equipment/Avatar/coat/sm_coat%02d%02db.img`
//! —— 两个 `%02d` 由 `[variation]` 填，末位 `a`/`b` 是同一设计的两张分层图
//! （实测：variation `0000` 的 hair/coat 只有 `a`，pants/shoes 只有 `b`）。

use std::collections::HashMap;

#[derive(Clone, Debug, Default)]
pub struct EquFile {
    pub name: String,
    pub name2: String,
    /// `[equipment type]`，如 `coat avatar`。
    pub equipment_type: String,
    /// `[variation] a b`。
    pub variation: (i32, i32),
    /// `[LAYER]`。
    pub layer: i32,
    /// 动作标签（`move motion` 等）→ 动画路径。
    pub motions: HashMap<String, String>,
}

impl EquFile {
    /// 是否属于某个时装槽（`coat` / `pants` / `hair` / `shoes` / `cap` / `neck` / `belt` / `face`）。
    pub fn slot(&self) -> Option<&'static str> {
        const SLOTS: [&str; 8] = [
            "coat", "pants", "hair", "shoes", "cap", "neck", "belt", "face",
        ];
        let t = self.equipment_type.to_ascii_lowercase();
        let head = t.split_whitespace().next().unwrap_or("");
        SLOTS.iter().copied().find(|s| *s == head)
    }
}

fn split_tag(line: &str) -> Option<(String, &str)> {
    let line = line.trim();
    if !line.starts_with('[') {
        return None;
    }
    let end = line.find(']')?;
    Some((
        line[1..end].trim().to_ascii_lowercase(),
        line[end + 1..].trim(),
    ))
}

fn dequote(s: &str) -> String {
    let raw = if let Some(a) = s.find('`') {
        let r = &s[a + 1..];
        match r.find('`') {
            Some(b) => r[..b].to_string(),
            None => r.trim().to_string(),
        }
    } else {
        s.trim().to_string()
    };
    // 官方把枚举值也放在方括号里，如 `[coat avatar]` / `[normal]`
    raw.trim_matches(|c| c == '[' || c == ']' || c == ' ')
        .to_string()
}

/// 解析 `.equ` 文本。
pub fn parse(text: &str) -> EquFile {
    let mut out = EquFile::default();
    for raw in text.lines() {
        let line = match raw.find("//") {
            Some(i) => &raw[..i],
            None => raw,
        };
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let Some((tag, rest)) = split_tag(trimmed) else {
            continue;
        };
        match tag.as_str() {
            "name" => out.name = dequote(rest),
            "name2" => out.name2 = dequote(rest),
            "equipment type" => out.equipment_type = dequote(rest),
            "variation" => {
                let v: Vec<i32> = rest
                    .split_whitespace()
                    .filter_map(|t| t.parse::<i32>().ok())
                    .collect();
                if v.len() >= 2 {
                    out.variation = (v[0], v[1]);
                }
            }
            "layer" => {
                out.layer = rest
                    .split_whitespace()
                    .next()
                    .and_then(|t| t.parse().ok())
                    .unwrap_or(0)
            }
            _ => {
                if tag.ends_with(" motion") || tag.ends_with(" attack motion") {
                    out.motions.insert(tag, dequote(rest));
                }
            }
        }
    }
    out
}

/// 解析 `.ani` 里的贴图路径模板（含 `%02d`）。
pub fn img_template(ani_text: &str) -> Option<String> {
    for raw in ani_text.lines() {
        let t = raw.trim();
        if let Some(rest) = t.strip_prefix("[IMAGE]") {
            let s = rest.trim();
            if let Some(a) = s.find('`') {
                let r = &s[a + 1..];
                if let Some(b) = r.find('`') {
                    return Some(r[..b].to_string());
                }
            }
        }
    }
    None
}

/// 用 `[variation]` 填模板里的两个 `%02d`，并把末位分层字母替换成 `suffix`。
///
/// 输入模板形如 `Character/Swordman/Equipment/Avatar/coat/sm_coat%02d%02db.img`，
/// `variation = (0, 0)`、`suffix = 'a'` → `.../sm_coat0000a.img`。
pub fn fill_template(template: &str, variation: (i32, i32), suffix: char) -> String {
    let filled = match (template.find("%02d"), template.rfind("%02d")) {
        (Some(a), Some(b)) if a != b => {
            let mut out = String::with_capacity(template.len() + 2);
            out.push_str(&template[..a]);
            out.push_str(&format!("{:02}", variation.0));
            out.push_str(&template[a + 4..b]);
            out.push_str(&format!("{:02}", variation.1));
            out.push_str(&template[b + 4..]);
            out
        }
        _ => template.to_string(),
    };
    // 末位分层字母：`...b.img` → `...<suffix>.img`
    if let Some(idx) = filled.rfind(".img") {
        if idx >= 1 {
            let mut bytes = filled.into_bytes();
            bytes[idx - 1] = suffix as u8;
            return String::from_utf8_lossy(&bytes).into_owned();
        }
    }
    filled
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "[name] `x`\n[name2] `Noble Tattoo`\n[equipment type] `[coat avatar]` 0\n[variation] 19 3\n[LAYER] 500\n[move motion] `coat_d/Move.ani`\n";

    #[test]
    fn 解析equ() {
        let e = parse(SAMPLE);
        assert_eq!(e.name2, "Noble Tattoo");
        assert_eq!(e.equipment_type, "coat avatar");
        assert_eq!(e.variation, (19, 3));
        assert_eq!(e.layer, 500);
        assert_eq!(
            e.motions.get("move motion").map(|s| s.as_str()),
            Some("coat_d/Move.ani")
        );
        assert_eq!(e.slot(), Some("coat"));
    }

    #[test]
    fn 模板填充() {
        let t = "Character/Swordman/Equipment/Avatar/coat/sm_coat%02d%02db.img";
        assert_eq!(
            fill_template(t, (19, 3), 'a'),
            "Character/Swordman/Equipment/Avatar/coat/sm_coat1903a.img"
        );
        assert_eq!(
            fill_template(t, (0, 0), 'b'),
            "Character/Swordman/Equipment/Avatar/coat/sm_coat0000b.img"
        );
    }

    #[test]
    fn 官方equ真值() {
        let Some(pvf) = crate::test_support::pvf() else {
            return;
        };
        let text = pvf
            .read_text("equipment/character/swordman/avatar/coat/nbtattoo_winddeva.equ")
            .unwrap();
        let e = parse(&text);
        assert_eq!(e.equipment_type, "coat avatar");
        assert_eq!(e.variation, (19, 3));
        assert_eq!(e.layer, 500);
        assert_eq!(e.slot(), Some("coat"));
        // 动作模板来自官方 .ani
        let ani = pvf
            .read_text("equipment/character/swordman/avatar/coat/coat_b/move.ani")
            .unwrap();
        let tpl = img_template(&ani).unwrap();
        assert_eq!(
            tpl,
            "Character/Swordman/Equipment/Avatar/coat/sm_coat%02d%02db.img"
        );
    }
}
