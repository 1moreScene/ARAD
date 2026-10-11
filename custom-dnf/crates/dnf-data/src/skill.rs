//! 502F00 -> 482A90 -> CNRDSkill: script definitions and per-level runtime tables.
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct SkillDefinition {
    pub id: i32,
    pub path: String,
    pub fields: HashMap<String, String>,
    pub maximum_level: usize,
    pub level_info: Vec<Vec<i32>>,
    pub mp: Vec<i32>,
    pub cooldown: Vec<i32>,
    pub casting: Vec<i32>,
    pub icons: Vec<(String, usize)>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SkillLevel {
    pub level: usize,
    pub mp: i32,
    pub cooldown_ms: i32,
    pub casting_ms: i32,
    pub values: Vec<i32>,
    pub items: Vec<(i32, i32)>,
}
fn numbers(s: &str) -> Vec<f32> {
    s.split_whitespace()
        .filter_map(|v| v.parse().ok())
        .collect()
}
fn table(fields: &HashMap<String, String>, tag: &str, max: usize, factor: f32) -> Vec<i32> {
    let values = numbers(fields.get(tag).map(String::as_str).unwrap_or(""));
    let lo = values.first().copied().unwrap_or(0.0) * factor;
    let hi = values
        .get(1)
        .copied()
        .unwrap_or_else(|| values.first().copied().unwrap_or(0.0))
        * factor;
    (0..max)
        .map(|i| {
            // The client truncates lerp to clong, it does not round costs.
            (lo as f64 + (hi as f64 - lo as f64) * i as f64 / max.saturating_sub(1).max(1) as f64)
                as i32
        })
        .collect()
}
impl SkillDefinition {
    pub fn parse(id: i32, path: &str, text: &str) -> Result<Self, String> {
        let mut fields = HashMap::<String, String>::new();
        let mut current = String::new();
        for raw in text.lines() {
            let line = raw.split("//").next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            let value = if line.starts_with('[') {
                let end = line.find(']').ok_or("unterminated skill tag")?;
                let tag = line[1..end].trim().to_ascii_lowercase();
                if tag.starts_with('/') {
                    current.clear();
                    continue;
                }
                current = tag;
                line[end + 1..].trim()
            } else {
                line
            };
            if !current.is_empty() {
                let dst = fields.entry(current.clone()).or_default();
                if !dst.is_empty() && !value.is_empty() {
                    dst.push('\n');
                }
                dst.push_str(value);
            }
        }
        let maximum_level = fields
            .get("maximum level")
            .and_then(|v| v.parse::<usize>().ok())
            .filter(|&n| n > 0 && n <= 1000)
            .ok_or("invalid skill maximum level")?;
        let flat = numbers(fields.get("level info").map(String::as_str).unwrap_or(""));
        let width = flat.first().copied().unwrap_or(0.0) as usize;
        if width > 128 || width > 0 && (flat.len() - 1) % width != 0 {
            return Err(format!("{path}: malformed level info"));
        }
        let level_info = if width == 0 {
            Vec::new()
        } else {
            flat[1..]
                .chunks(width)
                .map(|row| row.iter().map(|&v| v as i32).collect())
                .collect()
        };
        let mut icons = Vec::new();
        if let Some(icon) = fields.get("icon") {
            let words: Vec<_> = icon.split_whitespace().collect();
            for pair in words.chunks_exact(2) {
                if let Ok(index) = pair[1].parse() {
                    icons.push((pair[0].trim_matches('`').to_string(), index));
                }
            }
        }
        Ok(Self {
            id,
            path: path.into(),
            maximum_level,
            level_info,
            mp: table(
                &fields,
                "consume mp",
                maximum_level,
                f32::from_bits(0x3F666666),
            ),
            cooldown: table(&fields, "cool time", maximum_level, 1.0),
            casting: table(&fields, "casting time", maximum_level, 1.0),
            icons,
            fields,
        })
    }
    pub fn at_level(&self, level: usize) -> Option<SkillLevel> {
        let index = level.checked_sub(1)?;
        Some(SkillLevel {
            level,
            mp: *self.mp.get(index)?,
            cooldown_ms: *self.cooldown.get(index)?,
            casting_ms: *self.casting.get(index)?,
            values: self.level_info.get(index).cloned().unwrap_or_default(),
            items: numbers(
                self.fields
                    .get("consume item")
                    .map(String::as_str)
                    .unwrap_or(""),
            )
            .chunks_exact(3)
            .filter_map(|v| {
                let count = v[1] as i32
                    + index as i32 * (v[2] as i32 - v[1] as i32)
                        / self.maximum_level.saturating_sub(1).max(1) as i32;
                (count > 0).then_some((v[0] as i32, count))
            })
            .collect(),
        })
    }
    pub fn text(&self, tag: &str, strings: &crate::strings::StringTable) -> String {
        strings.resolve(self.fields.get(tag).map(String::as_str).unwrap_or(""))
    }
    /// 47F310: substitute level/static values and formatting multipliers.
    pub fn level_description(
        &self,
        level: usize,
        strings: &crate::strings::StringTable,
    ) -> Option<String> {
        let raw = self.fields.get("level property")?;
        let lines: Vec<_> = raw
            .lines()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect();
        let bounds: Vec<usize> = lines
            .first()?
            .split_whitespace()
            .filter_map(|s| s.parse().ok())
            .collect();
        if bounds.len() != 2 || level < bounds[0] || level > bounds[1] {
            return None;
        }
        let mut template = strings.resolve(lines.get(1)?);
        let statics = numbers(
            self.fields
                .get("static data")
                .map(String::as_str)
                .unwrap_or(""),
        );
        for line in &lines[2..] {
            let v: Vec<f64> = line
                .split_whitespace()
                .filter_map(|s| s.parse().ok())
                .collect();
            if v.len() != 3 {
                continue;
            }
            let a = v[0] as i32;
            let b = v[1] as usize;
            let value = if a == -1 {
                *self.level_info.get(level.checked_sub(1)?)?.get(b)? as f64
            } else {
                let lo = *statics.get(a as usize)? as f64;
                let hi = *statics.get(b)? as f64;
                lo + (hi - lo) * level.saturating_sub(1) as f64
                    / self.maximum_level.saturating_sub(1).max(1) as f64
            } * v[2];
            let begin = template.find('<')?;
            let end = template[begin..].find('>')? + begin;
            let formatted = match &template[begin + 1..end] {
                "int" => format!("{}", value.round() as i32),
                "float1" => format!("{value:.1}"),
                "float2" => format!("{value:.2}"),
                _ => return None,
            };
            template.replace_range(begin..=end, &formatted);
        }
        Some(template.replace("%%", "%"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn consume_item_interpolates_each_id_and_requires_no_items_when_cost_is_zero() {
        let s = SkillDefinition::parse(
            99,
            "cost",
            "[maximum level] 3\n[consume item]\n3037 0 2\n3033 2 4\n[/consume item]",
        )
        .unwrap();
        assert_eq!(s.at_level(1).unwrap().items, vec![(3033, 2)]);
        assert_eq!(s.at_level(2).unwrap().items, vec![(3037, 1), (3033, 3)]);
        assert_eq!(s.at_level(3).unwrap().items, vec![(3037, 2), (3033, 4)]);
    }
    #[test]
    #[cfg(windows)]
    fn official_skill_tooltips_resolve_reference_and_format_current_level_columns() {
        let Some(pvf) = crate::test_support::pvf() else {
            return;
        };
        let strings = crate::strings::StringTable::load(pvf, 12).unwrap();
        let s = SkillDefinition::parse(
            46,
            "upperslash.skl",
            &pvf.read_text("skill/swordman/upperslash.skl").unwrap(),
        )
        .unwrap();
        let one = s.level_description(1, &strings).unwrap();
        assert!(one.contains("+10") && one.contains("100.0%") && !one.contains('<'));
        let two = s.level_description(2, &strings).unwrap();
        assert!(two.contains("+20") && two.contains("113.7%"));
        assert!(s.level_description(0, &strings).is_none());
    }
    #[test]
    fn script_tables_keep_columns_and_truncate_scaled_mp() {
        let s = SkillDefinition::parse(5,"test.skl","[maximum level] 3\n[consume MP] 23 73\n[cool time] 8000 8000\n[level info]\n2\n40 300\n89 341\n138 383\n[/level info]\n[command]\n(SKILL)\n[/command]").unwrap();
        assert_eq!(s.mp, vec![20, 43, 65]);
        assert_eq!(s.at_level(2).unwrap().values, vec![89, 341]);
        assert_eq!(s.fields["command"], "(SKILL)");
        assert!(s.at_level(0).is_none());
        assert!(s.at_level(4).is_none());
        assert!(
            SkillDefinition::parse(5, "bad", "[maximum level] 2\n[level info]\n2\n40").is_err()
        );
    }
}
