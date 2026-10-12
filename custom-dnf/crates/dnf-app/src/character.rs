//! 角色外观：官方**分层合成**（裸身身体层 + 时装部件层）。
//!
//! # 官方分层规则（本模块实现，证据见 `docs/90` 会话 16）
//!
//! 1. **裸身身体**：`Character/Swordman/sm_body.img` —— 第 90 帧（Stay）实测是**光头裸身**
//!    （只有护臂/护腕），所以头发/上衣/裤子/靴子**全部**是叠在它上面的部件层。
//! 2. **绘制层 = `[LAYER]`**：官方 `.equ` 的每个部件块自带 `[LAYER]`（可以是**负数** =
//!    画在身体之前）。官方把全部部件与身体一起按 `[LAYER]` 升序绘制，身体层 = 0
//!    （区间 `(-50, 99)`：负数是"身体之前"，最小正数部件是 99/100，见 `dnf_data::avatar`）。
//! 3. **部件目录**：动作来自 `equipment/character/<职业>/avatar/<槽>/<槽>_<字母>/<动作>.ani`
//!    （如 `pants/pants_a/Stay.ani`），里面的 `[IMAGE]` 是路径模板 `.../sm_pants%02d%02da.img`，
//!    用装备 `[variation]` 填号。
//! 4. **同一槽的多个部件一起画**：官方 `.equ` 只点名**一个**部件目录，但同一槽的兄弟部件
//!    共享同一层值（实证：`.equ` 点的是 `pants_a`，而 `sm_pants0000a.img` 在 Stay 帧**全空**，
//!    真正的裤子美术在 `pants_b` ⇒ 只画 `_a` 会得到一个没穿裤子的角色）。
//! 5. 每层帧号取**该层自己 `.ani` 的 `[IMAGE]` 索引**（权威值；与身体层编号一致，已核对）。
//!
//! 绘制位置 = 角色脚点 + 身体层 `[IMAGE POS]` + 该帧在画布内的 `(x, y)`。

use crate::assets::ClientAssets;
use dnf_core::actor::ActorClips;
use dnf_core::anim::Clip;
use dnf_data::avatar::{self, AvatarTable};
use std::rc::Rc;

/// 一个已装备的时装槽。
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct OutfitSlot {
    pub slot: Slot,
    /// `[variation]` 的两个编号。
    pub variation: (i32, i32),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Slot {
    Hair,
    Coat,
    Pants,
    Shoes,
    Cap,
    Neck,
    Belt,
    Face,
}

impl Slot {
    pub const ALL: [Slot; 8] = [
        Slot::Coat,
        Slot::Pants,
        Slot::Hair,
        Slot::Shoes,
        Slot::Cap,
        Slot::Neck,
        Slot::Belt,
        Slot::Face,
    ];

    /// 官方资源目录名（`equipment/character/<职业>/avatar/<key>/`）。
    pub const fn key(self) -> &'static str {
        match self {
            Slot::Hair => "hair",
            Slot::Coat => "coat",
            Slot::Pants => "pants",
            Slot::Shoes => "shoes",
            Slot::Cap => "cap",
            Slot::Neck => "neck",
            Slot::Belt => "belt",
            Slot::Face => "face",
        }
    }

    /// 官方 `.equ` 的 `[equipment type]` 原文（8 个时装部位）。
    pub const fn equip_type(self) -> &'static str {
        match self {
            Slot::Hair => "hair avatar",
            Slot::Coat => "coat avatar",
            Slot::Pants => "pants avatar",
            Slot::Shoes => "shoes avatar",
            Slot::Cap => "hat avatar",
            Slot::Neck => "breast avatar",
            Slot::Belt => "waist avatar",
            Slot::Face => "face avatar",
        }
    }
}

/// 一套时装。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Outfit {
    pub slots: Vec<OutfitSlot>,
}

impl Outfit {
    /// 说明：**新号默认时装属于服务端数据**（本机客户端不存这份清单），
    /// 因此这里给出一套全 `0000` 的官方基础设计；可在 `data/outfit.txt` 里改。
    /// （官方基础设计的层值：hair 2000 / coat 1900 / pants 1500 / shoes 1400。）
    pub fn default_basic() -> Self {
        Outfit {
            slots: vec![
                OutfitSlot {
                    slot: Slot::Hair,
                    variation: (0, 0),
                },
                OutfitSlot {
                    slot: Slot::Coat,
                    variation: (0, 0),
                },
                OutfitSlot {
                    slot: Slot::Pants,
                    variation: (0, 0),
                },
                OutfitSlot {
                    slot: Slot::Shoes,
                    variation: (0, 0),
                },
            ],
        }
    }

    /// 从 `key = a b` 文本解析（`data/outfit.txt`）。
    pub fn parse(text: &str) -> Outfit {
        let mut slots = Vec::new();
        for raw in text.lines() {
            let line = raw.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            let Some((k, v)) = line.split_once('=') else {
                continue;
            };
            let key = k.trim().to_ascii_lowercase();
            let nums: Vec<i32> = v
                .split_whitespace()
                .filter_map(|t| t.parse::<i32>().ok())
                .collect();
            if nums.len() < 2 {
                continue;
            }
            if let Some(slot) = Slot::ALL.iter().copied().find(|s| s.key() == key) {
                slots.push(OutfitSlot {
                    slot,
                    variation: (nums[0], nums[1]),
                });
            }
        }
        if slots.is_empty() {
            Outfit::default_basic()
        } else {
            Outfit { slots }
        }
    }

    /// 文本形式（诊断/落盘）。
    pub fn to_text(&self) -> String {
        let mut s = String::from(
            "# slot = variation_a variation_b   （a/b 为官方 [variation] 的两个编号）\n",
        );
        for o in &self.slots {
            s.push_str(&format!(
                "{:<6} = {} {}\n",
                o.slot.key(),
                o.variation.0,
                o.variation.1
            ));
        }
        s
    }
}

/// 合成后的一层：贴图 + 相对角色脚点的偏移 + **官方 `[LAYER]`**。
pub struct SpriteLayer {
    pub image: Rc<dnf_data::img::ImgFile>,
    pub frame: usize,
    /// 相对脚点的像素偏移（已含 `[IMAGE POS]` 与画布内 `(x, y)`）。
    pub offset: (i32, i32),
    /// 官方 `[LAYER]`（身体 = 0）。
    pub layer: i32,
    /// 部件标签（`pants_a` / `body` …）。
    pub label: String,
    /// 图集路径（诊断用）。
    pub image_name: String,
}

/// 某动作某帧的贴图信息（来自官方身体层 `.ani`）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MotionFrame {
    pub damage_type: dnf_data::ani::DamageType,
    /// 在 `sm_body.img` 里的帧号。
    pub image_index: i32,
    /// 该帧的 `[IMAGE POS]`（画布原点相对脚点）。
    pub image_pos: (i32, i32),
}

/// 一个动作的完整帧序（来自身体层 `.ani`）+ 各动作片段。
pub struct CharacterVisual {
    pub clips: ActorClips,
    /// 动作名（身体 `.ani` 名）→ 每帧的贴图信息。
    pub motions: Vec<(String, Vec<MotionFrame>)>,
}

/// 动作名 → 官方 `.chr` 的动作标签。
///
/// ⚠ 动作路径**不再由本工程猜**：官方 `character/swordman/swordman.chr` 自带动作表
/// （`[waiting motion]` / `[move motion]` / `[dash motion]` / `[jump motion]` …），
/// 路径从 `.chr` 读出（见 [`motion_ani`]）。
pub const MOTION_TAGS: [(&str, &str); 6] = [
    ("Stay", "waiting motion"),
    ("Move", "move motion"),
    ("Dash", "dash motion"),
    ("Jump", "jump motion"),
    ("JumpAttack", "jumpattack motion"),
    ("DashAttack", "dashattack motion"),
];

/// 角色资源目录（`.chr` 里动作路径的相对根）。
pub const CHARACTER_DIR: &str = "character/swordman";

/// 官方动作表里的动作路径 → PVF 完整路径。
///
/// `.chr` 写的是 `Animation/Stay.ani`，PVF 里的实际条目是
/// `character/swordman/animation/stay.ani`（大小写由 `assets.ani` 统一处理）。
pub fn motion_ani(chr: &dnf_data::chr::ChrFile, tag: &str) -> Option<String> {
    let rel = chr.motion(tag)?;
    if rel.is_empty() {
        return None;
    }
    Some(format!("{}/{}", CHARACTER_DIR, rel))
}

/// 动作名在**官方动作表**里的序号（= 官方状态机使用的动作 ID）。
pub fn motion_index(chr: &dnf_data::chr::ChrFile, tag: &str) -> Option<usize> {
    chr.motions_ordered
        .iter()
        .position(|(t, _)| t.eq_ignore_ascii_case(tag))
}

/// 身体层贴图（裸身）。
pub const BODY_IMG: &str = "Character/Swordman/sm_body.img";
/// Official level-1 base weapon, equipment/equipment.lst id27000.
pub const DEFAULT_WEAPON: &str = "equipment/character/swordman/weapon/ssword/sswd_damaged.equ";

/// 从官方资源构建角色外观（帧序 + 片段时长）。
///
/// 动作路径取自官方 `.chr` 动作表；`.chr` 里没有该标签时才退回约定路径。
pub fn build_visual(assets: &ClientAssets, chr: &dnf_data::chr::ChrFile) -> CharacterVisual {
    let mut motions = Vec::new();
    let mut clips = ActorClips::swordman_official();
    for (name, tag) in MOTION_TAGS {
        let path = motion_ani(chr, tag)
            .unwrap_or_else(|| format!("{}/Animation/{}.ani", CHARACTER_DIR, name));
        if let Some(a) = assets.ani(&path) {
            let frames: Vec<MotionFrame> = a
                .frames
                .iter()
                .map(|f| MotionFrame {
                    damage_type: dnf_data::ani::DamageType::from_token(&f.damage_type),
                    image_index: f.image_index,
                    image_pos: f.image_pos,
                })
                .collect();
            let delays = a.delays();
            match name {
                "Stay" => clips.stand = Clip::new("Stay", delays, true),
                "Move" => clips.walk = Clip::new("Move", delays, true),
                "Dash" => clips.run = Clip::new("Dash", delays, true),
                "Jump" => clips.jump = Clip::new("Jump", delays, false),
                "JumpAttack" => clips.jump_attack = Clip::new("JumpAttack", delays, false),
                "DashAttack" => clips.dash_attack = Clip::new("DashAttack", delays, false),
                _ => {}
            }
            motions.push((name.to_string(), frames));
        }
    }
    // Prepare body-layer frames from the real multi-line array, not invented IDs.
    let mut attacks = Vec::new();
    for (i, relative) in chr.resource_list("attack motion").iter().enumerate() {
        let path = format!("{}/{}", CHARACTER_DIR, relative);
        if let Some(a) = assets.ani(&path) {
            attacks.push(Clip::new(format!("Attack{}", i + 1), a.delays(), false));
            motions.push((
                format!("Attack{}", i + 1),
                a.frames
                    .iter()
                    .map(|f| MotionFrame {
                        damage_type: dnf_data::ani::DamageType::from_token(&f.damage_type),
                        image_index: f.image_index,
                        image_pos: f.image_pos,
                    })
                    .collect(),
            ));
        }
    }
    if attacks.len() == chr.resource_list("attack motion").len() && !attacks.is_empty() {
        clips.attacks = attacks;
    }
    if let Some(relative) = chr.resource_list("etc motion").get(1) {
        if let Some(a) = assets.ani(&format!("{CHARACTER_DIR}/{relative}")) {
            clips.hard_attack = Clip::new("HardAttack", a.delays(), false);
            motions.push((
                "HardAttack".into(),
                a.frames
                    .iter()
                    .map(|f| MotionFrame {
                        damage_type: dnf_data::ani::DamageType::from_token(&f.damage_type),
                        image_index: f.image_index,
                        image_pos: f.image_pos,
                    })
                    .collect(),
            ));
        }
    }
    let relative = chr
        .resource_list("etc motion")
        .get(7)
        .cloned()
        .unwrap_or_else(|| "Animation/Grab.ani".into());
    if let Some(a) = assets.ani(&format!("{CHARACTER_DIR}/{relative}")) {
        clips.grab = Clip::new("Grab", a.delays(), false);
        motions.push((
            "Grab".into(),
            a.frames
                .iter()
                .map(|f| MotionFrame {
                    damage_type: dnf_data::ani::DamageType::from_token(&f.damage_type),
                    image_index: f.image_index,
                    image_pos: f.image_pos,
                })
                .collect(),
        ));
    }
    CharacterVisual { clips, motions }
}

impl CharacterVisual {
    pub fn protection(&self, motion: &str, frame: usize) -> dnf_core::protection::Protection {
        use dnf_core::protection::Protection;
        use dnf_data::ani::DamageType;
        let kind = self
            .motions
            .iter()
            .find(|(n, _)| n == motion)
            .and_then(|(_, frames)| frames.get(frame))
            .map_or(DamageType::Normal, |f| f.damage_type);
        match kind {
            DamageType::Normal => Protection::Normal,
            DamageType::SuperArmor => Protection::SuperArmor,
            DamageType::Unbreakable => Protection::Invincible,
        }
    }
    /// 某动作第 `frame` 帧的 `[IMAGE POS]`（身体层权威值）。
    pub fn image_pos(&self, motion: &str, frame: usize) -> (i32, i32) {
        self.motions
            .iter()
            .find(|(n, _)| n == motion)
            .and_then(|(_, f)| f.get(frame).map(|m| m.image_pos))
            .unwrap_or((-232, -333))
    }

    /// 某动作第 `frame` 帧在**图集**里的帧号。
    pub fn image_index(&self, motion: &str, frame: usize) -> Option<i32> {
        self.motions
            .iter()
            .find(|(n, _)| n == motion)
            .and_then(|(_, f)| f.get(frame).map(|m| m.image_index))
    }
}

/// 一个已解析好的时装部件。
pub struct PartSource {
    /// 部件目录（`pants_a`）。
    dir: String,
    /// 官方层值。
    layer: i32,
    /// 已填号的图集路径。
    name: String,
    /// 该部件自己 `.ani` 的逐帧图集帧号。
    indices: Vec<i32>,
}

fn suffix_of(template: &str) -> Option<char> {
    template
        .rfind(".img")
        .and_then(|i| i.checked_sub(1))
        .and_then(|i| template.as_bytes().get(i))
        .map(|b| *b as char)
}

/// 取某槽在某动作下的全部部件（含官方层值）。
///
/// * 装备由 `[equipment type]` + `[variation]` 在官方表里精确定位（找不到就整槽不画）。
/// * 槽内**所有**存在的部件目录都会被加载（见模块头注释第 4 条），
///   层值优先取该目录自己的官方层值，取不到才退回装备主部件的层值。
pub fn slot_layers(
    assets: &ClientAssets,
    table: &AvatarTable,
    slot: Slot,
    variation: (i32, i32),
    motion: &str,
) -> Vec<PartSource> {
    let Some(item) = table.find(slot.equip_type(), variation) else {
        return Vec::new();
    };
    let Some(main) = item.main() else {
        return Vec::new();
    };
    let item_layer = main.layer;
    let mut out = Vec::new();
    for dir in table.dirs_of(slot.key()) {
        if !table.has_motion(dir, motion) {
            continue;
        }
        let ani_path = format!(
            "equipment/character/swordman/avatar/{}/{}/{}.ani",
            slot.key(),
            dir,
            motion
        );
        let Some(ani) = assets.ani(&ani_path) else {
            continue;
        };
        let Some(tpl) = ani.frames.first().map(|f| f.image.clone()) else {
            continue;
        };
        if tpl.is_empty() {
            continue;
        }
        let Some(suffix) = suffix_of(&tpl) else {
            continue;
        };
        let name = dnf_data::equ::fill_template(&tpl, variation, suffix);
        if !assets.has_img(&name) {
            continue;
        }
        out.push(PartSource {
            dir: dir.clone(),
            layer: table.dir_layer(dir).unwrap_or(item_layer),
            name,
            indices: ani.frames.iter().map(|f| f.image_index).collect(),
        });
    }
    out.sort_by(|a, b| a.dir.cmp(&b.dir));
    out
}

/// 组装当前动作的全部绘制层（已按官方 `[LAYER]` 升序排好）。
pub fn compose(
    assets: &ClientAssets,
    table: &AvatarTable,
    visual: &CharacterVisual,
    outfit: &Outfit,
    weapon: &dnf_data::avatar::AvatarItem,
    motion: &str,
    frame: usize,
) -> Vec<SpriteLayer> {
    let mut layers: Vec<SpriteLayer> = Vec::new();
    let pos = visual.image_pos(motion, frame);
    if let Some((directory, _)) = weapon.path.rsplit_once('/') {
        for part in &weapon.parts {
            let path = format!("{}/{}/{}.ani", directory, part.dir, motion);
            let Some(ani) = assets.ani(&path) else {
                continue;
            };
            let Some(af) = ani.frames.get(frame) else {
                continue;
            };
            let Some(suffix) = suffix_of(&af.image) else {
                continue;
            };
            let name = dnf_data::equ::fill_template(&af.image, weapon.variation, suffix);
            let Some(img) = assets.img(&name) else {
                continue;
            };
            let idx = af.image_index.max(0) as usize;
            if let Some(f) = img.frames.get(idx) {
                let offset = (af.image_pos.0 + f.x, af.image_pos.1 + f.y);
                layers.push(SpriteLayer {
                    image: img,
                    frame: idx,
                    offset,
                    layer: part.layer,
                    label: format!("weapon_{}", part.dir),
                    image_name: name,
                });
            }
        }
    }

    // ---- 时装部件层 ----
    for o in &outfit.slots {
        for src in slot_layers(assets, table, o.slot, o.variation, motion) {
            let Some(img) = assets.img(&src.name) else {
                continue;
            };
            // 官方该部件自己 `.ani` 的帧号优先（权威），退化时才用身体层帧号
            let idx = src
                .indices
                .get(frame)
                .copied()
                .or_else(|| visual.image_index(motion, frame))
                .unwrap_or(frame as i32)
                .max(0) as usize;
            let Some(f) = img.frames.get(idx) else {
                continue;
            };
            let offset = (pos.0 + f.x, pos.1 + f.y);
            layers.push(SpriteLayer {
                image: img,
                frame: idx,
                offset,
                layer: src.layer,
                label: src.dir.clone(),
                image_name: src.name,
            });
        }
    }

    // ---- 裸身身体层（官方层 0）----
    if let Some(img) = assets.img(BODY_IMG) {
        let idx = visual.image_index(motion, frame).unwrap_or(0).max(0) as usize;
        if let Some(f) = img.frames.get(idx) {
            let offset = (pos.0 + f.x, pos.1 + f.y);
            layers.push(SpriteLayer {
                image: img,
                frame: idx,
                offset,
                layer: avatar::BODY_LAYER,
                label: "body".into(),
                image_name: BODY_IMG.to_string(),
            });
        }
    }

    // 官方顺序：按 [LAYER] 升序；同层按部件目录名稳定排序
    layers.sort_by(|a, b| a.layer.cmp(&b.layer).then_with(|| a.label.cmp(&b.label)));
    layers
}
