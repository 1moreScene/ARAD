//! 确定性输入脚本（无头回放 / 冒烟验证用）。
//!
//! 用一条固定时间线驱动角色，覆盖 **站立 → 步行 → 奔跑 → 跳跃 → 空中 → 落地 → 反向**，
//! 每一步的输入都是纯函数，保证逐帧可复现（同一步号 ⇒ 同一画面）。

use dnf_core::keymap::{Command, KeyMap};
use dnf_core::keys::Dik;

/// 脚本种类。
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ScriptKind {
    Idle,
    Walk,
    Run,
    Jump,
    /// 奔跑中当场反向（验证反向退出奔跑）。
    Turn,
    /// 按住一个方向、同时不断点按反方向键（验证**不会**因此触发奔跑）。
    MashTap,
    /// 空中改方向（验证跳跃中可调整落地点）。
    AirTurn,
    Attack,
    AirAttack,
    DashAttack,
    UpperSlash,
    GhostSlash,
    CancelGhost,
    CancelUpper,
    /// 全部动作串联。
    All,
}

impl ScriptKind {
    pub fn parse(s: &str) -> Option<Self> {
        Some(match s.to_ascii_lowercase().as_str() {
            "idle" => ScriptKind::Idle,
            "walk" => ScriptKind::Walk,
            "run" => ScriptKind::Run,
            "jump" => ScriptKind::Jump,
            "turn" => ScriptKind::Turn,
            "mashtap" => ScriptKind::MashTap,
            "airturn" => ScriptKind::AirTurn,
            "attack" => ScriptKind::Attack,
            "airattack" => ScriptKind::AirAttack,
            "dashattack" => ScriptKind::DashAttack,
            "upper" => ScriptKind::UpperSlash,
            "ghost" => ScriptKind::GhostSlash,
            "cancelghost" => ScriptKind::CancelGhost,
            "cancelupper" => ScriptKind::CancelUpper,
            "all" => ScriptKind::All,
            _ => return None,
        })
    }

    pub fn name(self) -> &'static str {
        match self {
            ScriptKind::Idle => "idle",
            ScriptKind::Walk => "walk",
            ScriptKind::Run => "run",
            ScriptKind::Jump => "jump",
            ScriptKind::Turn => "turn",
            ScriptKind::MashTap => "mashtap",
            ScriptKind::AirTurn => "airturn",
            ScriptKind::Attack => "attack",
            ScriptKind::AirAttack => "airattack",
            ScriptKind::DashAttack => "dashattack",
            ScriptKind::UpperSlash => "upper",
            ScriptKind::GhostSlash => "ghost",
            ScriptKind::CancelGhost => "cancelghost",
            ScriptKind::CancelUpper => "cancelupper",
            ScriptKind::All => "all",
        }
    }
}

/// 采样第 `step` 个逻辑步（0 基）应当按下的键。
pub fn sample(kind: ScriptKind, step: u64, km: &KeyMap) -> Vec<Dik> {
    let mut keys = Vec::new();
    let right = km.get(Command::Right);
    let left = km.get(Command::Left);
    let up = km.get(Command::Up);
    let down = km.get(Command::Down);
    let jump = km.get(Command::Jump);
    let attack = km.get(Command::Attack);

    match kind {
        ScriptKind::Idle => {}
        ScriptKind::Walk => {
            // 步行：按住右（按住只走 143 档，不会自动变跑）
            keys.push(right);
        }
        ScriptKind::Run => {
            // 奔跑：双击方向键触发官方 286 档（第 3 步松手一次），之后按住保持
            if step != 3 {
                keys.push(right);
            }
        }
        ScriptKind::Jump => {
            // 站立起跳到落地，循环
            let phase = step % 90;
            if phase < 2 {
                keys.push(jump);
            }
        }
        ScriptKind::Turn => {
            // 奔跑中反向：先原地双击起步（第 2 帧松一次构成双击），
            // 第 40 帧起改按左，官方状态11退出到自由态，随后步行。
            match step {
                0 | 1 | 3 => {}
                40.. => keys.push(left),
                _ => keys.push(right),
            }
        }
        ScriptKind::MashTap => {
            // 全程按住右；每隔 4 帧点按一次左（按 2 帧、松 2 帧）——官方不会因此奔跑
            keys.push(right);
            if step % 4 < 2 {
                keys.push(left);
            }
        }
        ScriptKind::AirTurn => {
            // 站立起跳 → 空中先向右 → 再向左（落地点可调）
            match step {
                2 => keys.push(jump),
                8..=18 => keys.push(right),
                19..=32 => keys.push(left),
                _ => {}
            }
        }
        ScriptKind::Attack => {
            if matches!(step, 0 | 28 | 55) {
                keys.push(attack);
            }
        }
        ScriptKind::AirAttack => {
            if step == 0 {
                keys.push(jump);
            }
            if matches!(step, 12 | 24 | 39) {
                keys.push(attack);
            }
        }
        ScriptKind::DashAttack => {
            if step == 0 || (2..25).contains(&step) {
                keys.push(right);
            }
            if matches!(step, 6 | 18) {
                keys.push(attack);
            }
        }
        ScriptKind::UpperSlash => {
            if matches!(step, 0 | 20 | 125) {
                keys.push(km.get(Command::Skill));
            }
        }
        ScriptKind::GhostSlash => {
            if matches!(step, 0 | 20 | 490) {
                keys.push(km.get(Command::SkillSlot1));
            }
        }
        ScriptKind::CancelGhost | ScriptKind::CancelUpper => {
            if step == 0 {
                keys.push(attack);
            }
            if step == 8 {
                keys.push(km.get(if kind == ScriptKind::CancelGhost {
                    Command::SkillSlot1
                } else {
                    Command::Skill
                }));
            }
        }
        ScriptKind::All => {
            // 整条时间线（共 660 步 = 11 秒）：
            //   0..40   站立
            //   40..100 步行（按住，143 档）
            //   100..270 奔跑向右（先松 1 帧回中立，再双击触发 286 档；202 起跳继续跑）
            //   200..270 奔跑中起跳 → 空中 → 落地（继续向右）
            //   270..340 站立
            //   340..480 奔跑向左（让相机横移、角色转身）
            //   480..520 站立
            //   520..560 向纵深内（↑）
            //   560..600 向纵深外（↓）
            //   600..660 原地反复起跳
            if (40..100).contains(&step) {
                keys.push(right);
            } else if (100..270).contains(&step) {
                // 向右奔跑：先松一帧，再按两次产生连续逻辑命令后缀
                if (101..=270).contains(&step) && step != 102 {
                    keys.push(right);
                }
                if step == 202 {
                    keys.push(jump);
                }
            } else if (340..480).contains(&step) {
                // 向左奔跑：同上（341 松一帧后 342.. 按住）
                if (342..480).contains(&step) && step != 343 {
                    keys.push(left);
                }
            } else if (520..560).contains(&step) {
                keys.push(up);
            } else if (560..600).contains(&step) {
                keys.push(down);
            } else if step >= 600 {
                let phase = step % 90;
                if phase < 2 {
                    keys.push(jump);
                }
            }
        }
    }
    keys
}
