//! 官方默认键位映射。
//!
//! # 取证等级
//!
//! 每一个绑定都带 [`Provenance`]，说明它的证据强度：
//!
//! * [`Provenance::Confirmed`] —— 从官方客户端二进制**逐字节确证**。
//! * [`Provenance::CommunityStandard`] —— 官方客户端的既有公开约定（业界通用默认值），
//!   本项目**未能**从本机客户端二进制逐字节确证（见 `docs/01_键位取证.md` 的取证过程与失败路径）。
//! * [`Provenance::Unverified`] —— 占位默认值，需要用户按官方客户端「键位设置」界面校正。
//!
//! # 已确证的部分
//! 2026-10-11补齐439AB0的游戏层绑定，21默认值均与415240槽表交叉验证；Buff实际为Space。
//!
//! `ARAD.exe` 的 `sub_415240`（引擎输入对象初始化，VA `0x415240`）把键槽表整表写入
//! 输入对象 `+0x310`；`sub_415200(this, slot)` 用该表查 `slot` 的 DIK 扫描码再去读
//! `+269`（当前按下）/`+525`（上一帧按下）两个 256 字节状态数组。
//!
//! `sub_415240` 的前四个槽（槽 0..3）写入的正是：
//!
//! | 槽 | DIK | 键 |
//! |----|-----|----|
//! | 0 | `0xCB` (203) | ← |
//! | 1 | `0xCD` (205) | → |
//! | 2 | `0xC8` (200) | ↑ |
//! | 3 | `0xD0` (208) | ↓ |
//!
//! 与 `docs/90_发现日志.md` 会话 12/13 的记载一致（DIK 203/205/200/208 = VK 37/39/38/40）。
//!
//! # 命令词表
//!
//! 命令词表来自官方 PVF 文件 `Skill/RDARKeyIndex.dat`（本项目已从 `Script.pvf` 取出，
//! 原文见 `docs/01_键位取证.md`）：
//!
//! ```text
//! 0 (UP)  1 (DOWN)  2 (LEFT)  3 (RIGHT)  4 (SKILL)  5 (ATTACK)  6 (JUMP)  7 (CREATURE)  8 (BUFF)
//! ```

use crate::keys::Dik;

/// 证据强度。
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Provenance {
    /// 从官方客户端二进制逐字节确证（附证据地址/文件）。
    Confirmed(&'static str),
    /// 官方客户端的公开既有约定；本项目未能逐字节确证。
    CommunityStandard,
    /// 未经证据支持的占位值，需按官方键位设置校正。
    Unverified,
}

impl Provenance {
    pub fn is_confirmed(self) -> bool {
        matches!(self, Provenance::Confirmed(_))
    }
    pub fn label(self) -> &'static str {
        match self {
            Provenance::Confirmed(_) => "确证",
            Provenance::CommunityStandard => "行业约定(未逐字节确证)",
            Provenance::Unverified => "未验证(占位)",
        }
    }
    pub fn evidence(self) -> &'static str {
        match self {
            Provenance::Confirmed(e) => e,
            Provenance::CommunityStandard => "-",
            Provenance::Unverified => "-",
        }
    }
}

/// 官方命令词表（`Skill/RDARKeyIndex.dat` 的 9 条 + 技能/道具快捷栏）。
///
/// 前 9 个变体的顺序**必须**与官方 `.dat` 的索引 0..8 一致，方便与 `.skl` 的
/// `[command]` 语义对齐（`(UP)(DOWN)(LEFT)(RIGHT)(SKILL)(ATTACK)(JUMP)(CREATURE)(BUFF)`）。
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Command {
    Up = 0,
    Down = 1,
    Left = 2,
    Right = 3,
    Skill = 4,
    Attack = 5,
    Jump = 6,
    Creature = 7,
    Buff = 8,
    /// 技能快捷栏 1..6
    SkillSlot1 = 9,
    SkillSlot2 = 10,
    SkillSlot3 = 11,
    SkillSlot4 = 12,
    SkillSlot5 = 13,
    SkillSlot6 = 14,
    /// 道具快捷栏 1..6
    ItemSlot1 = 15,
    ItemSlot2 = 16,
    ItemSlot3 = 17,
    ItemSlot4 = 18,
    ItemSlot5 = 19,
    ItemSlot6 = 20,
}

/// 键位表条目数。
pub const COMMAND_COUNT: usize = 21;

impl Command {
    pub const ALL: [Command; COMMAND_COUNT] = [
        Command::Up,
        Command::Down,
        Command::Left,
        Command::Right,
        Command::Skill,
        Command::Attack,
        Command::Jump,
        Command::Creature,
        Command::Buff,
        Command::SkillSlot1,
        Command::SkillSlot2,
        Command::SkillSlot3,
        Command::SkillSlot4,
        Command::SkillSlot5,
        Command::SkillSlot6,
        Command::ItemSlot1,
        Command::ItemSlot2,
        Command::ItemSlot3,
        Command::ItemSlot4,
        Command::ItemSlot5,
        Command::ItemSlot6,
    ];

    pub const fn index(self) -> usize {
        self as usize
    }

    /// 官方 `.skl` 命令词（`(UP)` 等）；快捷栏不是命令词，返回 `None`。
    pub const fn token(self) -> Option<&'static str> {
        Some(match self {
            Command::Up => "(UP)",
            Command::Down => "(DOWN)",
            Command::Left => "(LEFT)",
            Command::Right => "(RIGHT)",
            Command::Skill => "(SKILL)",
            Command::Attack => "(ATTACK)",
            Command::Jump => "(JUMP)",
            Command::Creature => "(CREATURE)",
            Command::Buff => "(BUFF)",
            _ => return None,
        })
    }
}

/// 一条键位绑定。
#[derive(Copy, Clone, Debug)]
pub struct Binding {
    pub command: Command,
    pub dik: Dik,
    pub provenance: Provenance,
}

impl Binding {
    pub const fn new(command: Command, dik: Dik, provenance: Provenance) -> Self {
        Binding {
            command,
            dik,
            provenance,
        }
    }
}

/// 官方默认键位表。
#[derive(Clone, Debug)]
pub struct KeyMap {
    bindings: [Binding; COMMAND_COUNT],
}

impl Default for KeyMap {
    fn default() -> Self {
        Self::official_default()
    }
}

impl KeyMap {
    /// 构造键位表：以**方向键（已逐字节确证）** + 其余命令的默认值。
    pub const fn official_default() -> Self {
        use Command::*;
        KeyMap {
            bindings: [
                // ---- 方向键：逐字节确证（ARAD.exe sub_415240 槽 0..3）----
                Binding::new(
                    Up,
                    Dik::UP,
                    Provenance::Confirmed("ARAD.exe sub_415240 槽2 = DIK 0xC8 (VK 38)"),
                ),
                Binding::new(
                    Down,
                    Dik::DOWN,
                    Provenance::Confirmed("ARAD.exe sub_415240 槽3 = DIK 0xD0 (VK 40)"),
                ),
                Binding::new(
                    Left,
                    Dik::LEFT,
                    Provenance::Confirmed("ARAD.exe sub_415240 槽0 = DIK 0xCB (VK 37)"),
                ),
                Binding::new(
                    Right,
                    Dik::RIGHT,
                    Provenance::Confirmed("ARAD.exe sub_415240 槽1 = DIK 0xCD (VK 39)"),
                ),
                Binding::new(
                    Skill,
                    crate::device_slots::dik(80),
                    Provenance::Confirmed("439AB0 logical4 -> slot80; 415240 -> Z"),
                ),
                Binding::new(
                    Attack,
                    crate::device_slots::dik(78),
                    Provenance::Confirmed("439AB0 logical5 -> slot78; 415240 -> X"),
                ),
                Binding::new(
                    Jump,
                    crate::device_slots::dik(57),
                    Provenance::Confirmed("439AB0 logical6 -> slot57; 415240 -> C"),
                ),
                Binding::new(
                    Creature,
                    crate::device_slots::dik(76),
                    Provenance::Confirmed("439AB0 logical7 -> slot76; 415240 -> V"),
                ),
                Binding::new(
                    Buff,
                    crate::device_slots::dik(13),
                    Provenance::Confirmed("439AB0 logical8 -> slot13; 415240 -> Space"),
                ),
                Binding::new(
                    SkillSlot1,
                    crate::device_slots::dik(55),
                    Provenance::Confirmed(
                        "439AB0 skill slots55/73/58/60/61/62; 415240 -> A/S/D/F/G/H",
                    ),
                ),
                Binding::new(
                    SkillSlot2,
                    crate::device_slots::dik(73),
                    Provenance::Confirmed(
                        "439AB0 skill slots55/73/58/60/61/62; 415240 -> A/S/D/F/G/H",
                    ),
                ),
                Binding::new(
                    SkillSlot3,
                    crate::device_slots::dik(58),
                    Provenance::Confirmed(
                        "439AB0 skill slots55/73/58/60/61/62; 415240 -> A/S/D/F/G/H",
                    ),
                ),
                Binding::new(
                    SkillSlot4,
                    crate::device_slots::dik(60),
                    Provenance::Confirmed(
                        "439AB0 skill slots55/73/58/60/61/62; 415240 -> A/S/D/F/G/H",
                    ),
                ),
                Binding::new(
                    SkillSlot5,
                    crate::device_slots::dik(61),
                    Provenance::Confirmed(
                        "439AB0 skill slots55/73/58/60/61/62; 415240 -> A/S/D/F/G/H",
                    ),
                ),
                Binding::new(
                    SkillSlot6,
                    crate::device_slots::dik(62),
                    Provenance::Confirmed(
                        "439AB0 skill slots55/73/58/60/61/62; 415240 -> A/S/D/F/G/H",
                    ),
                ),
                Binding::new(
                    ItemSlot1,
                    crate::device_slots::dik(46),
                    Provenance::Confirmed("439AB0 item slots46..51; 415240 -> 1..6"),
                ),
                Binding::new(
                    ItemSlot2,
                    crate::device_slots::dik(47),
                    Provenance::Confirmed("439AB0 item slots46..51; 415240 -> 1..6"),
                ),
                Binding::new(
                    ItemSlot3,
                    crate::device_slots::dik(48),
                    Provenance::Confirmed("439AB0 item slots46..51; 415240 -> 1..6"),
                ),
                Binding::new(
                    ItemSlot4,
                    crate::device_slots::dik(49),
                    Provenance::Confirmed("439AB0 item slots46..51; 415240 -> 1..6"),
                ),
                Binding::new(
                    ItemSlot5,
                    crate::device_slots::dik(50),
                    Provenance::Confirmed("439AB0 item slots46..51; 415240 -> 1..6"),
                ),
                Binding::new(
                    ItemSlot6,
                    crate::device_slots::dik(51),
                    Provenance::Confirmed("439AB0 item slots46..51; 415240 -> 1..6"),
                ),
            ],
        }
    }

    pub fn get(&self, command: Command) -> Dik {
        self.bindings[command.index()].dik
    }

    pub fn provenance(&self, command: Command) -> Provenance {
        self.bindings[command.index()].provenance
    }

    /// 覆盖一条绑定。若新值与当前值相同，则**保留**原有取证等级
    /// （这样把官方默认值原样写进 `data/keymap.txt` 不会把「已确证」降级成「未验证」）。
    pub fn set(&mut self, command: Command, dik: Dik) {
        let b = &mut self.bindings[command.index()];
        if b.dik != dik {
            b.dik = dik;
            b.provenance = Provenance::Unverified;
        }
    }

    pub fn bindings(&self) -> &[Binding; COMMAND_COUNT] {
        &self.bindings
    }

    /// 方向键位（8 方向）→ 该方向是否按下。
    ///
    /// ⚠ **已不再用于方向解析**：两键相抵会得到 `(0, 0)`，而官方语义是
    /// **后按的键优先**（`sub_4BF3D0` 从尾向前取最近一条有效记录 ⇒ 见
    /// [`crate::key_manager::CommandChecker`]）。角色方向走统一记录管理器；本方法只留作
    /// 「两个方向键各自是否按下」的查询。
    pub fn direction_keys_down(&self, is_down: impl Fn(Dik) -> bool) -> (bool, bool, bool, bool) {
        (
            is_down(self.get(Command::Left)),
            is_down(self.get(Command::Right)),
            is_down(self.get(Command::Up)),
            is_down(self.get(Command::Down)),
        )
    }

    /// 从 `Key = Value` 文本加载（`data/keymap.txt` 格式）。
    ///
    /// 行格式：`<命令名>=<扫描码十进制或 0x 十六进制>`，`#` 开头为注释。
    /// 未知命令名会被忽略（返回列表中报告）。
    pub fn load_overrides(&mut self, text: &str) -> Vec<String> {
        let mut warnings = Vec::new();
        for (lineno, raw) in text.lines().enumerate() {
            let line = raw.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            let Some((k, v)) = line.split_once('=') else {
                warnings.push(format!("第 {} 行缺少 '='：{}", lineno + 1, raw));
                continue;
            };
            let k = k.trim();
            let v = v.trim();
            let code = if let Some(hex) = v.strip_prefix("0x").or_else(|| v.strip_prefix("0X")) {
                u8::from_str_radix(hex, 16)
            } else {
                v.parse::<u8>()
            };
            let Ok(code) = code else {
                warnings.push(format!("第 {} 行扫描码非法：{}", lineno + 1, raw));
                continue;
            };
            match Command::ALL
                .iter()
                .copied()
                .find(|c| command_name(*c).eq_ignore_ascii_case(k))
            {
                Some(cmd) => self.set(cmd, Dik::new(code)),
                None => warnings.push(format!("第 {} 行未知命令名：{}", lineno + 1, k)),
            }
        }
        warnings
    }
}

/// 规范命令名（`data/keymap.txt` 的键名）。
pub const fn command_name(c: Command) -> &'static str {
    match c {
        Command::Up => "Up",
        Command::Down => "Down",
        Command::Left => "Left",
        Command::Right => "Right",
        Command::Skill => "Skill",
        Command::Attack => "Attack",
        Command::Jump => "Jump",
        Command::Creature => "Creature",
        Command::Buff => "Buff",
        Command::SkillSlot1 => "SkillSlot1",
        Command::SkillSlot2 => "SkillSlot2",
        Command::SkillSlot3 => "SkillSlot3",
        Command::SkillSlot4 => "SkillSlot4",
        Command::SkillSlot5 => "SkillSlot5",
        Command::SkillSlot6 => "SkillSlot6",
        Command::ItemSlot1 => "ItemSlot1",
        Command::ItemSlot2 => "ItemSlot2",
        Command::ItemSlot3 => "ItemSlot3",
        Command::ItemSlot4 => "ItemSlot4",
        Command::ItemSlot5 => "ItemSlot5",
        Command::ItemSlot6 => "ItemSlot6",
    }
}
