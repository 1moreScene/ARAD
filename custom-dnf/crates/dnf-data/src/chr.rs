//! `.chr` 角色定义解析（官方纯文本标签格式）。
//!
//! 官方 `character/swordman/swordman.chr` 结构：
//!
//! ```text
//! [job]  `[swordman]`
//! [width]  40 10
//! [initial value]
//! [HP MAX]  150.0
//! [move speed]  850.0
//! [jump power]  430.0
//! [jump speed]  95
//! [/...]
//! [growtype 1]  // 转职增量（会**重复**顶层属性标签）
//! [move speed]  1.0
//! ...
//! [waiting motion]  `Animation/Stay.ani`
//! ```
//!
//! ⚠️ 官方解析陷阱（docs/90 会话 11）：`[growtype N]` 段会重复顶层属性标签，
//! 必须**首次出现优先**，否则鬼剑士基础移速会被转职增量覆盖。

use std::collections::HashMap;

/// `[width]` 的官方注释：角色的 X 轴 / Y 轴宽度（碰撞体尺寸）。
pub const WIDTH_KEY: &str = "width";

/// 一个 `.chr` 文件。
#[derive(Clone, Debug, Default)]
pub struct ChrFile {
    /// `[job]`（去掉反引号与方括号，如 `swordman`）。
    pub job: String,
    /// `[width]` X/Y。
    pub width: (i32, i32),
    /// 基础属性（`[initial value]` 段，首次出现优先）。
    pub attributes: HashMap<String, f64>,
    /// `[growtype N]` 的转职增量。
    pub growtypes: HashMap<u32, HashMap<String, f64>>,
    /// `* motion` 标签 → 动画路径。
    pub motions: HashMap<String, String>,
    /// `* motion` 标签按**官方文件出现顺序**排列。
    ///
    /// 这个顺序就是官方给动作分配的**动作 ID**（官方状态机按 ID 切动作：
    /// `IRDActiveObject` 的 `+1540`/`+1541` 等字段存的就是这里的序号，
    /// 见 `src/game/ui/char_server_info.cpp` 的 `sub_58FC30` 状态分支）。
    pub motions_ordered: Vec<(String, String)>,
    /// Multi-line resource blocks, e.g. attack motion/info and etc motion/info.
    /// These are arrays, not one additional scalar motion ID.
    pub resource_lists: HashMap<String, Vec<String>>,
    /// Scalar .atk references (jumpattack info, dashattack info).
    pub attack_infos: HashMap<String, String>,
    /// `[skill]` 段的 (技能索引, 等级)。
    pub skills: Vec<(i32, i32)>,
}

impl ChrFile {
    pub fn attr(&self, key: &str) -> Option<f64> {
        self.attributes.get(&key.to_ascii_lowercase()).copied()
    }

    /// `[move speed]`（1000 = 100%）。
    pub fn move_speed(&self) -> f64 {
        self.attr("move speed").unwrap_or(1000.0)
    }

    /// `[jump power]`（1000 = 100%）。
    pub fn jump_power(&self) -> f64 {
        self.attr("jump power").unwrap_or(1000.0)
    }

    pub fn attack_speed(&self) -> f64 {
        self.attr("attack speed").unwrap_or(1000.0)
    }

    /// `[jump speed]`（100 = 100%；官方鬼剑士 = 95）—— 跳跃公式的 `a3`。
    pub fn jump_speed(&self) -> f64 {
        self.attr("jump speed").unwrap_or(100.0)
    }

    /// `[weight]`（官方鬼剑士 = 68000）—— 跳跃公式的 `a2`。
    pub fn weight(&self) -> f64 {
        self.attr("weight").unwrap_or(0.0)
    }

    pub fn motion(&self, tag: &str) -> Option<&str> {
        self.motions
            .get(&tag.to_ascii_lowercase())
            .map(|s| s.as_str())
    }

    pub fn resource_list(&self, tag: &str) -> &[String] {
        self.resource_lists
            .get(&tag.to_ascii_lowercase())
            .map(Vec::as_slice)
            .unwrap_or(&[])
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

fn dequote(s: &str) -> String {
    let t = s.trim();
    let t = if let Some(a) = t.find('`') {
        let rest = &t[a + 1..];
        match rest.find('`') {
            Some(b) => &rest[..b],
            None => rest,
        }
    } else {
        t
    };
    t.trim_matches(|c| c == '[' || c == ']' || c == '`' || c == ' ')
        .to_string()
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

fn float(s: &str) -> Option<f64> {
    let cleaned: String = s
        .chars()
        .map(|c| {
            if c.is_ascii_digit() || c == '-' || c == '+' || c == '.' {
                c
            } else {
                ' '
            }
        })
        .collect();
    cleaned
        .split_whitespace()
        .next()
        .and_then(|t| t.parse::<f64>().ok())
}

/// 解析 `.chr` 文本。
pub fn parse(text: &str) -> ChrFile {
    let mut out = ChrFile::default();
    // `[growtype N]` 的当前编号；None = 还在基础段。
    let mut growtype: Option<u32> = None;
    let mut in_skill = false;
    let mut resource_block: Option<String> = None;

    for raw in text.lines() {
        let line = strip_comment(raw);
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let Some((tag, rest)) = split_tag(trimmed) else {
            if let Some(tag) = &resource_block {
                if trimmed.contains('`') {
                    out.resource_lists
                        .entry(tag.clone())
                        .or_default()
                        .push(dequote(trimmed));
                }
                continue;
            }
            // 非标签行：只在 `[skill]` 段内被消费（`技能索引 等级`）
            if in_skill {
                let v = ints(trimmed);
                if v.len() >= 2 {
                    out.skills.push((v[0], v[1]));
                }
            }
            continue;
        };
        let lower = tag.to_ascii_lowercase();

        if lower.starts_with('/') {
            // 结束标签（`[/attack motion]`）：不参与编号
            in_skill = false;
            resource_block = None;
            continue;
        }
        resource_block = None;
        if matches!(
            lower.as_str(),
            "attack motion" | "etc motion" | "attack info" | "etc attack info"
        ) {
            let paths = out.resource_lists.entry(lower.clone()).or_default();
            if rest.contains('`') {
                paths.push(dequote(rest));
            }
            resource_block = Some(lower);
            continue;
        }
        if lower.ends_with("attack info") && rest.contains('`') {
            out.attack_infos.insert(lower, dequote(rest));
            continue;
        }
        if lower == "/skill" {
            in_skill = false;
            continue;
        }
        if lower == "skill" {
            in_skill = true;
            continue;
        }
        if let Some(num) = lower.strip_prefix("growtype") {
            let num = num.trim();
            if let Ok(n) = num.parse::<u32>() {
                growtype = Some(n);
                continue;
            }
            // `[growtype name]` / `[growtype maximum level]` 等：按普通属性处理
        }

        match lower.as_str() {
            "job" => out.job = dequote(rest),
            "width" => {
                let v = ints(rest);
                if v.len() >= 2 {
                    out.width = (v[0], v[1]);
                }
            }
            _ => {
                if lower.contains(" motion") {
                    // ⚠ 官方动作 ID = 本文件里的**出现顺序**（0 = waiting motion，1 = move motion；
                    //   官方状态机 `sub_58FC30` 的 case 0 正是用 `+1540`=0 / `+1541`=1 切站立/移动）。
                    let path = dequote(rest);
                    if !out.motions.contains_key(&lower) {
                        out.motions_ordered.push((lower.clone(), path.clone()));
                    }
                    out.motions.insert(lower, path);
                    continue;
                }
                if let Some(v) = float(rest) {
                    let table = match growtype {
                        Some(n) => out.growtypes.entry(n).or_default(),
                        None => &mut out.attributes,
                    };
                    // 首次出现优先（官方 `[growtype N]` 会重复顶层标签）
                    table.entry(lower).or_insert(v);
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 官方 `character/swordman/swordman.chr` 的关键片段（真值）。
    pub const SWORDMAN_CHR: &str = r#"[job]  `[swordman]`
[width]  40 10

[initial value] 
[HP MAX]  150.0
[MP MAX]  150.0
[physical attack]  5.0
[move speed]  850.0
[attack speed]  850.0
[cast speed]  700.0
[hit recovery]  600.0
[jump power]  430.0
[weight]  68000.0
[jump speed]  95
[skill] 
181 2 // 上挑
[/skill]
[growtype 1]  // 基础
[HP MAX]  52.5
[move speed]  1.0

[growtype 4]  // 狂战士
[move speed]  1.0

[waiting motion]  `Animation/Stay.ani`
[move motion]  `Animation/Move.ani`
[dash motion]  `Animation/Dash.ani`
[jump motion]  `Animation/Jump.ani`
"#;

    #[test]
    fn 解析swordman_chr真值() {
        let c = parse(SWORDMAN_CHR);
        assert_eq!(c.job, "swordman");
        assert_eq!(c.width, (40, 10));
        assert_eq!(c.move_speed(), 850.0);
        assert_eq!(c.jump_power(), 430.0);
        assert_eq!(c.jump_speed(), 95.0);
        assert_eq!(c.attr("hit recovery"), Some(600.0));
        assert_eq!(c.weight(), 68000.0);
        assert_eq!(c.attr("weight"), Some(68000.0));
    }

    #[test]
    fn growtype不覆盖基础属性() {
        let c = parse(SWORDMAN_CHR);
        // `[growtype 1] [move speed] 1.0` 绝不能覆盖基础的 850
        assert_eq!(c.move_speed(), 850.0);
        assert_eq!(
            c.growtypes
                .get(&1)
                .and_then(|m| m.get("move speed"))
                .copied(),
            Some(1.0)
        );
        assert_eq!(
            c.growtypes
                .get(&4)
                .and_then(|m| m.get("move speed"))
                .copied(),
            Some(1.0)
        );
        // growtype 段里的 HP MAX 也不得污染基础 HP MAX
        assert_eq!(c.attr("hp max"), Some(150.0));
    }

    #[test]
    fn 动作标签映射与技能段() {
        let c = parse(SWORDMAN_CHR);
        assert_eq!(c.motion("waiting motion"), Some("Animation/Stay.ani"));
        assert_eq!(c.motion("move motion"), Some("Animation/Move.ani"));
        assert_eq!(c.motion("dash motion"), Some("Animation/Dash.ani"));
        assert_eq!(c.motion("jump motion"), Some("Animation/Jump.ani"));
        assert_eq!(c.skills, vec![(181, 2)]);
    }

    /// 官方 `swordman.chr`：动作表必须按**文件出现顺序**给出官方动作 ID，
    /// 且结束标签（`[/attack motion]`）不参与编号。
    #[test]
    fn 官方动作表顺序真值() {
        let Some(pvf) = crate::test_support::pvf() else {
            return;
        };
        let text = pvf
            .read_text("character/swordman/swordman.chr")
            .expect("读 swordman.chr");
        let c = parse(&text);
        let ids: Vec<&str> = c.motions_ordered.iter().map(|(t, _)| t.as_str()).collect();
        // 官方状态机 case 0 用 +1540 / +1541 切「站立 / 移动」⇒ 这两项必须是 0 / 1
        assert_eq!(ids[0], "waiting motion");
        assert_eq!(ids[1], "move motion");
        assert!(
            !ids.iter().any(|t| t.starts_with('/')),
            "结束标签不得编号: {:?}",
            ids
        );
        // 关键动作的官方序号与路径
        let expect = [
            ("waiting motion", "Animation/Stay.ani"),
            ("move motion", "Animation/Move.ani"),
            ("jump motion", "Animation/Jump.ani"),
            ("dash motion", "Animation/Dash.ani"),
        ];
        for (tag, path) in expect {
            assert_eq!(c.motion(tag), Some(path), "{}", tag);
        }
        assert_eq!(c.motions_ordered.len(), ids.len());
        assert_eq!(
            c.resource_list("attack motion"),
            &[
                "Animation/Attack1.ani",
                "Animation/Attack2.ani",
                "Animation/Attack3.ani"
            ]
        );
        assert_eq!(c.resource_list("attack info").len(), 3);
        eprintln!("官方动作表: {:?}", c.motions_ordered);
    }
}
