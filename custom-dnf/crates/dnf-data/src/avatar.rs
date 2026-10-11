//! 官方「外观部件表」：`equipment/character/<职业>/avatar/<槽>/<物品>.equ`。
//!
//! # 官方结构（逐字取自真实文件 + `src/game/parsers/item_parser.cpp` 的解析器）
//!
//! 一个 `.equ` **可以包含多个部件块**，每块自带一个 `[LAYER]` 与整套动作路径：
//!
//! ```text
//! [equipment type] `[coat avatar]` 0
//! [variation] 0 0
//! [LAYER] 1900                       ← 部件块 1 的绘制层
//! [waiting motion] `coat_a/Stay.ani`
//! [move motion]    `coat_a/Move.ani`
//! ...
//! [LAYER] 50                         ← 部件块 2（同一件装备的第二部件）
//! [waiting motion] `weaponmaster/Stay.ani`
//! ```
//!
//! 实证（本机 5,175 个 `.equ` 全量统计）：
//!
//! * `[LAYER]` 就是**绘制层排序键**，可以是负数（`-1000/-500/-200/-100/-50` = 画在身体**之前**）。
//! * 剑士时装的层值：`shoes_a 1400` / `pants_a 1500` / `waist_a 1700` / `breast_a 1800` /
//!   `coat_a 1900` / `hair_a 2000` / `hat_a 2100` / `breast_c 2200` / `hat_c 2500`；
//!   反面部件：`face_b 100` / `coat_d 500` / `breast_d 600` / `waist_b 1100`。
//! * **裸身身体层 = 0**：`sm_body.img` 第 90 帧（Stay）就是**光头裸身**（只有护臂，见
//!   `docs/90` 会话 16 的像素证据）⇒ 头发/衣服/裤子/靴子全都在身体**之后**画，层值必须 > 0；
//!   而"画在身体之前"的部件官方用的是**负数**层 ⇒ 身体层在 `(-50, 99)` 这个区间内，取 **0**。
//!
//! 槽位名与 `[equipment type]` 的对应（官方 8 个时装部位）：
//!
//! | 本工程 `Slot` | 官方 `[equipment type]` |
//! |---|---|
//! | hair / coat / pants / shoes | `hair avatar` / `coat avatar` / `pants avatar` / `shoes avatar` |
//! | cap / neck / belt / face | `hat avatar` / `breast avatar` / `waist avatar` / `face avatar` |

use crate::pvf::PvfReader;
use std::collections::HashMap;

/// 裸身身体的绘制层（区间 `(-50, 99)` 内，取 0；见模块头注释的实证）。
pub const BODY_LAYER: i32 = 0;

/// 一个部件块。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AvatarPart {
    /// 官方 `[LAYER]`。
    pub layer: i32,
    /// 动作目录名（`[waiting motion]` 的目录部分，如 `coat_a`）。
    pub dir: String,
}

/// 一件装备（含它的全部部件块）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AvatarItem {
    /// `[equipment type]` 去掉方括号后的原文（如 `coat avatar`）。
    pub equip_type: String,
    /// `[variation]` 两个编号。
    pub variation: (i32, i32),
    pub parts: Vec<AvatarPart>,
    /// 来源路径（诊断用）。
    pub path: String,
    /// `[name2]`（英文名，诊断用）。
    pub name2: String,
}

impl AvatarItem {
    /// 主部件（第一个 `[LAYER]` 块）。
    pub fn main(&self) -> Option<&AvatarPart> {
        self.parts.first()
    }
}

/// 官方的 8 个时装部位（`[equipment type]` 原文）。
pub const AVATAR_TYPES: [&str; 8] = [
    "hat avatar",
    "hair avatar",
    "face avatar",
    "coat avatar",
    "pants avatar",
    "shoes avatar",
    "breast avatar",
    "waist avatar",
];

/// 解析一个 `.equ` 文本（支持多部件块）。
pub fn parse(text: &str) -> AvatarItem {
    let mut item = AvatarItem {
        equip_type: String::new(),
        variation: (0, 0),
        parts: Vec::new(),
        path: String::new(),
        name2: String::new(),
    };
    let mut cur: Option<AvatarPart> = None;
    for raw in text.lines() {
        let line = match raw.find("//") {
            Some(i) => &raw[..i],
            None => raw,
        };
        let line = line.trim();
        if line.is_empty() || !line.starts_with('[') {
            continue;
        }
        let Some(end) = line.find(']') else { continue };
        let tag = line[1..end].trim().to_ascii_uppercase();
        let rest = line[end + 1..].trim();
        match tag.as_str() {
            "EQUIPMENT TYPE" => item.equip_type = dequote(rest),
            "NAME2" => item.name2 = dequote(rest),
            "VARIATION" => {
                let v: Vec<i32> = rest
                    .split_whitespace()
                    .filter_map(|t| t.parse().ok())
                    .collect();
                if v.len() >= 2 {
                    item.variation = (v[0], v[1]);
                }
            }
            "LAYER" => {
                if let Some(p) = cur.take() {
                    item.parts.push(p);
                }
                let layer = rest
                    .split_whitespace()
                    .next()
                    .and_then(|t| t.parse::<i32>().ok())
                    .unwrap_or(0);
                cur = Some(AvatarPart {
                    layer,
                    dir: String::new(),
                });
            }
            _ => {
                // 动作标签族：`[waiting motion]` 等 —— 取目录名（部件块的身份）
                if tag.ends_with("MOTION") || tag == "WAITING MOTION" {
                    if let Some(p) = cur.as_mut() {
                        if p.dir.is_empty() {
                            let path = dequote(rest);
                            if let Some((d, _)) = path.rsplit_once('/') {
                                p.dir = d.rsplit('/').next().unwrap_or(d).to_string();
                            }
                        }
                    }
                }
            }
        }
    }
    if let Some(p) = cur.take() {
        item.parts.push(p);
    }
    item
}

fn dequote(s: &str) -> String {
    let s = s.trim();
    if let Some(a) = s.find('`') {
        let r = &s[a + 1..];
        if let Some(b) = r.find('`') {
            return r[..b]
                .trim_matches(|c| c == '[' || c == ']' || c == ' ')
                .to_string();
        }
    }
    s.trim_matches(|c| c == '[' || c == ']' || c == ' ')
        .to_string()
}

/// 官方时装部件总表（按职业扫描）。
#[derive(Default)]
pub struct AvatarTable {
    pub items: Vec<AvatarItem>,
    /// 动作目录 → 官方层值（同一目录可能被多件装备引用，取众数/首次值）。
    dir_layers: HashMap<String, i32>,
    /// 槽位目录 → 该槽全部动作目录（`pants_a` / `pants_b` / …）。
    slot_dirs: HashMap<String, Vec<String>>,
    /// 槽位目录 → 存在的 `<Motion>.ani` 名集合（用于判断某个部件有没有这个动作）。
    dir_motions: HashMap<String, Vec<String>>,
}

impl AvatarTable {
    /// 扫描 `equipment/character/<job>/avatar/**` 下的全部 `.equ`。
    ///
    /// `job` = `swordman` / `fighter` / `gunner` / `mage`。
    pub fn load(pvf: &PvfReader, job: &str) -> AvatarTable {
        let prefix = format!("equipment/character/{}/avatar/", job);
        let mut table = AvatarTable::default();
        let mut counts: HashMap<String, HashMap<i32, usize>> = HashMap::new();

        for name in pvf.names_with_prefix(&prefix) {
            let low = name.to_ascii_lowercase();
            // 槽位目录 + 动作目录 + 动作名：.../avatar/<slot>/<slot>_<letter>/<motion>.ani
            let rest = &low[prefix.len()..];
            let segs: Vec<&str> = rest.split('/').collect();
            if segs.len() == 3 {
                let (slot, dir, file) = (segs[0], segs[1], segs[2]);
                let _ = slot;
                let motions = table.dir_motions.entry(dir.to_string()).or_default();
                if let Some(m) = file.strip_suffix(".ani") {
                    motions.push(m.to_string());
                }
                table
                    .slot_dirs
                    .entry(slot.to_string())
                    .or_default()
                    .push(dir.to_string());
            }
            if !low.ends_with(".equ") {
                continue;
            }
            let Ok(text) = pvf.read_text(&low) else {
                continue;
            };
            let mut item = parse(&text);
            item.path = low.clone();
            for p in &item.parts {
                if p.dir.is_empty() {
                    continue;
                }
                *counts
                    .entry(p.dir.clone())
                    .or_default()
                    .entry(p.layer)
                    .or_insert(0) += 1;
            }
            table.items.push(item);
        }
        for dirs in table.slot_dirs.values_mut() {
            dirs.sort();
            dirs.dedup();
        }
        for motions in table.dir_motions.values_mut() {
            motions.sort();
            motions.dedup();
        }
        for (dir, by_layer) in counts {
            // 同一动作目录被多件装备引用时，取出现次数最多的层值
            if let Some((layer, _)) = by_layer.iter().max_by_key(|(_, c)| **c) {
                table.dir_layers.insert(dir, *layer);
            }
        }
        table
    }

    /// 按 `[equipment type]` + `[variation]` 精确查找一件装备。
    pub fn find(&self, equip_type: &str, variation: (i32, i32)) -> Option<&AvatarItem> {
        self.items
            .iter()
            .find(|i| i.equip_type.eq_ignore_ascii_case(equip_type) && i.variation == variation)
    }

    /// 动作目录（`pants_a`）的官方层值。
    pub fn dir_layer(&self, dir: &str) -> Option<i32> {
        self.dir_layers.get(dir).copied()
    }

    /// 某槽位（目录名，如 `pants`）的全部动作目录。
    pub fn dirs_of(&self, slot_dir: &str) -> &[String] {
        self.slot_dirs
            .get(slot_dir)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// 某动作目录里是否存在该动作（`stay` / `move` / `dash` / `jump` …）。
    pub fn has_motion(&self, dir: &str, motion: &str) -> bool {
        self.dir_motions
            .get(dir)
            .map(|v| v.iter().any(|m| m.eq_ignore_ascii_case(motion)))
            .unwrap_or(false)
    }

    /// 全部动作目录的官方层值（诊断用）。
    pub fn dir_layer_map(&self) -> &HashMap<String, i32> {
        &self.dir_layers
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const COAT: &str = "[name] `x`\n[name2] `Tight Sleeveless Shirts`\n[equipment type] `[coat avatar]` 0\n[variation] 0 0\n[LAYER] 1900\n[waiting motion] `coat_a/Stay.ani`\n[move motion] `coat_a/Move.ani`\n";

    /// 官方 `berserker.equ` 有两个部件块（层 2700 与 50）—— 多块解析的真值。
    const TWO_BLOCKS: &str = "[name2] `y`\n[equipment type] `[face avatar]` 0\n[LAYER] 2700\n[waiting motion] `berserker/Stay.ani`\n[move motion] `berserker/Move.ani`\n[LAYER] 50\n[waiting motion] `berserker_eye/Stay.ani`\n[move motion] `berserker_eye/Move.ani`\n";

    #[test]
    fn 解析单部件块() {
        let i = parse(COAT);
        assert_eq!(i.equip_type, "coat avatar");
        assert_eq!(i.variation, (0, 0));
        assert_eq!(i.name2, "Tight Sleeveless Shirts");
        assert_eq!(i.parts.len(), 1);
        assert_eq!(i.parts[0].layer, 1900);
        assert_eq!(i.parts[0].dir, "coat_a");
    }

    #[test]
    fn 解析多部件块() {
        let i = parse(TWO_BLOCKS);
        assert_eq!(i.parts.len(), 2, "一个 .equ 可以有多段 [LAYER]");
        assert_eq!(
            (i.parts[0].layer, i.parts[0].dir.as_str()),
            (2700, "berserker")
        );
        assert_eq!(
            (i.parts[1].layer, i.parts[1].dir.as_str()),
            (50, "berserker_eye")
        );
    }

    #[test]
    fn 官方剑士时装层值真值() {
        let Some(pvf) = crate::test_support::pvf() else {
            return;
        };
        let t = AvatarTable::load(pvf, "swordman");
        // 四个基础设计（variation 0 0）的官方层值
        let cases = [
            ("hair avatar", "hair_a", 2000),
            ("coat avatar", "coat_a", 1900),
            ("pants avatar", "pants_a", 1500),
            ("shoes avatar", "shoes_a", 1400),
        ];
        for (et, dir, layer) in cases {
            let item = t
                .find(et, (0, 0))
                .unwrap_or_else(|| panic!("找不到 {} (0,0)", et));
            let main = item.main().expect("至少一个部件块");
            assert_eq!(main.layer, layer, "{} 的官方 [LAYER]", et);
            assert_eq!(main.dir, dir, "{} 的官方动作目录", et);
        }
        // 反面部件：coat_d 在下层、belt_b 在下层、cap_c 在最上层
        assert_eq!(t.dir_layer("coat_d"), Some(500));
        assert_eq!(t.dir_layer("belt_b"), Some(1100));
        assert_eq!(t.dir_layer("belt_a"), Some(1700));
        assert_eq!(t.dir_layer("coat_a"), Some(1900));
        assert_eq!(t.dir_layer("cap_c"), Some(2500));
        // 槽位目录清单
        assert!(t.dirs_of("pants").contains(&"pants_a".to_string()));
        assert!(t.dirs_of("pants").contains(&"pants_b".to_string()));
        assert!(t.has_motion("pants_b", "stay"));
        assert!(t.has_motion("pants_a", "dash"));
    }

    /// 多部件块的真值：官方 `equipment/character/swordman/growtype/berserker.equ`。
    #[test]
    fn 官方转职部件真值() {
        let Some(pvf) = crate::test_support::pvf() else {
            return;
        };
        let text = pvf
            .read_text("equipment/character/swordman/growtype/berserker.equ")
            .expect("读 berserker.equ");
        let i = parse(&text);
        assert_eq!(i.parts.len(), 2, "官方文件里有两段 [LAYER]");
        assert_eq!(i.parts[0].layer, 2700);
        assert_eq!(i.parts[0].dir, "berserker");
        assert_eq!(i.parts[1].layer, 50);
        assert_eq!(i.parts[1].dir, "berserker_eye");
    }
}
