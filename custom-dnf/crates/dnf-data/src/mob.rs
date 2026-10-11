//! Monster and monster-equipment script motions/level ranges. Never infer ANI filenames.
use std::collections::HashMap;

#[derive(Clone, Debug, Default)]
pub struct MobFile {
    pub motions: HashMap<String, String>,
    pub values: HashMap<String, Vec<f64>>,
}
pub fn parse(text: &str) -> MobFile {
    let mut out = MobFile::default();
    let mut array = None;
    let mut index = 0;
    for raw in text.lines() {
        let line = raw.split("//").next().unwrap_or("").trim();
        if line.starts_with('[') {
            let Some(end) = line.find(']') else {
                continue;
            };
            let tag = line[1..end].to_ascii_lowercase();
            let rest = line[end + 1..].trim();
            array = None;
            if tag.starts_with('/') {
                continue;
            }
            if tag == "attack motion" || tag == "etc motion" {
                array = Some(tag);
                index = 0;
                continue;
            }
            if tag.contains(" motion") {
                if let Some(path) = quoted(rest) {
                    out.motions
                        .entry(tag)
                        .or_insert_with(|| path.replace('\\', "/"));
                }
            } else {
                let values = rest
                    .split_whitespace()
                    .filter_map(|s| s.parse::<f64>().ok())
                    .collect::<Vec<_>>();
                if !values.is_empty() {
                    out.values.entry(tag).or_insert(values);
                }
            }
        } else if let (Some(block), Some(path)) = (&array, quoted(line)) {
            out.motions
                .insert(format!("{block} {index}"), path.replace('\\', "/"));
            index += 1;
        }
    }
    out
}
pub fn quoted(s: &str) -> Option<&str> {
    let begin = s.find('`')? + 1;
    let end = s[begin..].find('`')? + begin;
    Some(&s[begin..end])
}
impl MobFile {
    pub fn motion(&self, tag: &str) -> Option<&str> {
        self.motions.get(tag).map(String::as_str)
    }
    pub fn at_level(&self, tag: &str, level: i32) -> Option<f64> {
        let values = self.values.get(tag)?;
        let &min = values.first()?;
        let max = values.get(1).copied().unwrap_or(min);
        let levels = self.values.get("level");
        let lo = levels.and_then(|v| v.first()).copied().unwrap_or(1.0);
        let hi = levels.and_then(|v| v.get(1)).copied().unwrap_or(lo);
        let t = if hi > lo {
            ((level as f64 - lo) / (hi - lo)).clamp(0.0, 1.0)
        } else {
            0.0
        };
        Some(min + (max - min) * t)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn script_motions_and_range_are_data_driven() {
        let m=parse("[level] 1 6\n[hit recovery] 500 700\n[damage motion 2] `Custom/Hurt.ani`\n[attack motion]\n`Swing.ani`\n`Idle.ani`\n[/attack motion]");
        assert_eq!(m.motion("damage motion 2"), Some("Custom/Hurt.ani"));
        assert_eq!(m.motion("attack motion 1"), Some("Idle.ani"));
        assert_eq!(m.at_level("hit recovery", 6), Some(700.0));
    }
}
