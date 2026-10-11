//! 角色状态机：站立 / 步行 / 奔跑 / 跳跃（+ 空中控制）。
//!
//! # 2.5D 坐标系（与官方一致）
//!
//! DNF 是「2.5D 横版」：← → 控制**水平轴 x**，↑ ↓ 控制**纵深轴 y**（地面平面），
//! 跳跃只改变**屏幕竖直偏移 z**（不影响地面平面上的 y）。因此：
//!
//! * 世界位置 = `(x, y)`，`y` 同时决定伪 3D 的画家排序（z-order）与地面线；
//! * 跳跃高度 `z` 只影响绘制时的纵向偏移与影子大小。
//!
//! # 动画选择
//!
//! | 状态 | 官方动画 | 播放速率 |
//! |------|----------|----------|
//! | 站立 | `Animation/Stay.ani`（6 帧 ×120ms） | 100% |
//! | 步行 | `Animation/Move.ani`（8 帧 ×100ms） | 100% |
//! | 奔跑 | `Animation/Dash.ani`（8 帧 ×100ms） | 100% |
//! | 跳跃 | `Animation/Jump.ani`（16 帧，含 2 个保持帧） | 100% |
//!
//! **播放速率 100% 是官方值**：自由态/跳跃态切入动作时调用
//! `vt+888(this, 1, 0|1, 1000, 1000)`（`char_server_info.cpp:4187`/`4279`/`4676`），
//! 其中 `IRDActiveObject_vmt378`（0x495310）的第二参数 `1000` 经 `vt+984`
//! = `IRDActiveObject_vmt3D8`（0x495A10）的 `× 0.1` 得到动画速率 `100.0`（= 100%）。
//! `[move speed]` 的千分值**只缩放位移速率**（`vmt37C` 里的 `this_[1663] * 0.001`），
//! 不缩放动画播放速率 —— 这是本工程此前搞错、现在按官方修正的一点。
//!
//! 帧时长与速率关系见 [`crate::anim`]。

use crate::anim::{AnimPlayer, Clip};
use crate::input::{AxisInput, InputState};
use crate::jump::{Jump, JumpProfile, JumpTuning};
use crate::keymap::{Command, KeyMap};
use crate::keys::Dik;
use crate::movement::{facing_from_dir, MoveMode, MoveTuning, Mover};

/// 角色动作状态。
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Motion {
    Stand,
    Walk,
    Run,
    Jump,
}

impl Motion {
    pub const fn label(self) -> &'static str {
        match self {
            Motion::Stand => "STAND",
            Motion::Walk => "WALK",
            Motion::Run => "RUN",
            Motion::Jump => "JUMP",
        }
    }
}

/// 角色数值（来自官方 `.chr`）。
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ActorSpec {
    /// `[move speed]`（1000 = 100%）—— 官方 `vmt3D0` case 1，只缩放**位移**速率。
    pub move_speed: f32,
    /// `[jump power]`（官方样例注释：1000 = 标准值；鬼剑士 = 430）
    /// —— **跳跃初速度公式的 `a1`**。
    pub jump_power: f32,
    /// `[jump speed]`（鬼剑士 = 95）—— 公式的 `a3`，**同时缩放重力**（`g = 1500 × 值/100`）。
    pub jump_speed: f32,
    /// `[weight]`（官方样例注释：单位 10g；鬼剑士 = 68000）—— 公式的 `a2`（越重跳越低）。
    pub weight: f32,
    /// `[width]` 的 X/Y（角色的 X 轴 / Y 轴宽度，官方注释：碰撞体尺寸）
    pub width: (i32, i32),
}

impl Default for ActorSpec {
    fn default() -> Self {
        // 官方 character/swordman/swordman.chr
        ActorSpec {
            move_speed: 850.0,
            jump_power: 430.0,
            jump_speed: 95.0,
            weight: 68000.0,
            width: (40, 10),
        }
    }
}

/// 角色可用的动画片段。
#[derive(Clone, Debug)]
pub struct ActorClips {
    pub stand: Clip,
    pub walk: Clip,
    pub run: Clip,
    pub jump: Clip,
}

impl ActorClips {
    /// 官方鬼剑士的帧时长（从 `Script.pvf` 解出的真值），供 `dnf-data` 不可用时回退。
    pub fn swordman_official() -> Self {
        ActorClips {
            stand: Clip::new("Stay", vec![120; 6], true),
            walk: Clip::new("Move", vec![100; 8], true),
            run: Clip::new("Dash", vec![100; 8], true),
            jump: Clip::new(
                "Jump",
                vec![
                    100, 100, 80, 80, 80, 80, 80, 10000, 100, 80, 80, 80, 80, 80, 10000, 50,
                ],
                false,
            ),
        }
    }
}

/// 状态机事件。
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ActorEvent {
    None,
    Jumped,
    Landed,
}

/// 角色。
#[derive(Clone, Debug)]
pub struct Actor {
    pub spec: ActorSpec,
    pub clips: ActorClips,
    /// 世界水平坐标（像素）。
    pub x: i32,
    /// 地面平面纵深坐标（像素，↑↓ 控制）。
    pub y: i32,
    /// 朝向：1 = 右，-1 = 左。
    pub facing: i32,
    pub motion: Motion,
    /// 是否允许**空中水平位移**（官方允许：跳跃态把轴 0 设成 286 档，轴 1 设 0）。
    pub air_control: bool,

    /// 每轴方向键解析（后按优先）。
    axis_x: AxisInput,
    axis_y: AxisInput,
    mover: Mover,
    jump: Jump,
    profile: Option<JumpProfile>,
    anim: AnimPlayer,
    land_lock: u32,
    /// 落地帧（官方 `jump.ani` 帧 15，50ms）剩余的逻辑步数。
    land_steps: u32,
    step_index: u64,
    /// 奔跑已「武装」（双击触发后为 true，直到方向键全部松开）。
    ///
    /// 官方奔跑（状态 11）的参数组是 x 286/238、y 136/114（`equ_buff.cpp:7172`/`:7290-7293`）。
    /// 奔跑状态在**服务端**持有，客户端只负责「方向符号 → 速率」（`vmt37C`）
    /// ⇒ **转向后仍是奔跑**（切换方向不需要重新双击）。
    /// 触发方式 = 双击方向键（官方命令表 `(UP)…(BUFF)` 里没有「跑步」命令；
    /// 且官方输入容器的记录模型要求该轴回到中立才算一次点击 ⇒ 按住一键猛按反向键不触发）。
    dash_armed: bool,

    /// 起跳那一刻的移动档位（官方跳跃分支把 `+1719..1722` 的暂存参数写回 x 轴、
    /// **不写 y 轴** ⇒ 空中继承地面档位，两轴都可动）。见 [`MoveMode`]。
    air_mode: MoveMode,
    last_press_step: [u64; 4],
}

/// 每轴方向 `(x, y)`，分量 -1/0/+1。
pub type DirState = (i32, i32);

/// 本帧新按下的方向键：`(键序号, 键码, 按下前该轴是否中立)`。
pub type DirPress = (usize, Dik, bool);

/// 双击方向键的判定窗口（逻辑步）。
///
/// **标定常数**：官方奔跑触发在输入层，已还原源码里只看到状态机消费
/// `vt+184`/`vt+188`（`4` = 无输入），没找到双击判定 ⇒ 窗口取 15 步（60Hz 下 250ms），
/// 与「双击方向键冲刺」这一 DNF 既有操作一致。
pub const DOUBLE_TAP_STEPS: u64 = 15;

/// 官方动作播放速率（100 = 100%）。
///
/// 依据 `vt+888(this, 1, 0|1, 1000, 1000)` → `vmt378` → `vt+984` = `vmt3D8(a2)` 的 `a2 * 0.1`。
pub const OFFICIAL_CLIP_RATE: f32 = 100.0;

impl Actor {
    pub fn new(spec: ActorSpec, clips: ActorClips, x: i32, y: i32) -> Self {
        let rate = spec.move_speed * 0.1;
        Actor {
            spec,
            clips: clips.clone(),
            x,
            y,
            facing: 1,
            motion: Motion::Stand,
            air_control: true,
            axis_x: AxisInput::default(),
            axis_y: AxisInput::default(),
            mover: Mover::new(),
            jump: Jump::idle(),
            profile: None,
            anim: AnimPlayer::new(clips.stand, rate),
            land_lock: 0,
            land_steps: 0,
            step_index: 0,
            dash_armed: false,
            air_mode: MoveMode::Walk,
            last_press_step: [0; 4],
        }
    }

    /// 是否处于「奔跑姿态」（双击触发、松开方向键前一直保持）。
    pub fn is_running_armed(&self) -> bool {
        self.dash_armed
    }

    /// 离地高度（像素，≥ 0）。
    pub fn altitude(&self) -> f32 {
        self.jump.altitude()
    }

    pub fn is_airborne(&self) -> bool {
        self.jump.active
    }

    pub fn anim(&self) -> &AnimPlayer {
        &self.anim
    }

    /// 当前移动方向（x, y），每分量 -1/0/+1。
    ///
    /// 每个轴按**官方语义**解析：后按的键优先、松开后回落到仍按住的键
    /// （官方 `sub_4BF3D0` 从尾向前取最近一条有效记录，见 [`AxisInput`]）。
    pub fn read_direction(&mut self, input: &InputState, keymap: &KeyMap) -> DirState {
        self.poll_direction(input, keymap).0
    }

    /// 更新两轴输入，返回 `(方向, 本帧新按下的键)`。
    /// 键列表元素 = `(键在四个方向键里的序号, 键码, 按下前该轴是否中立)`。
    fn poll_direction(&mut self, input: &InputState, keymap: &KeyMap) -> (DirState, Vec<DirPress>) {
        self.axis_x
            .set_keys(keymap.get(Command::Left), keymap.get(Command::Right));
        self.axis_y
            .set_keys(keymap.get(Command::Up), keymap.get(Command::Down));
        let keys = [
            keymap.get(Command::Left),
            keymap.get(Command::Right),
            keymap.get(Command::Up),
            keymap.get(Command::Down),
        ];
        let mut presses = Vec::new();
        for hit in [self.axis_x.update(input), self.axis_y.update(input)]
            .into_iter()
            .flatten()
        {
            if let Some(i) = keys.iter().position(|d| *d == hit.0) {
                presses.push((i, hit.0, hit.1));
            }
        }
        ((self.axis_x.value(), self.axis_y.value()), presses)
    }

    /// 推进一个逻辑步。
    pub fn step(
        &mut self,
        input: &InputState,
        keymap: &KeyMap,
        mt: &MoveTuning,
        jt: &JumpTuning,
    ) -> ActorEvent {
        self.step_index += 1;
        let mut event = ActorEvent::None;

        // ---- 1. 方向与奔跑判定 ----
        let (dir, presses) = self.poll_direction(input, keymap);
        // 官方双击语义：一次「点击」只在**该轴回到中立**时成立（官方每轴输入容器里
        // 上一个记录必须已失效）。所以「按住 ← 同时猛按 →」永远不构成双击 ⇒ 不会奔跑。
        let _ = keymap;
        for (i, _key, neutral) in presses {
            if neutral {
                let last = self.last_press_step[i];
                if last != 0 && self.step_index - last <= DOUBLE_TAP_STEPS {
                    self.dash_armed = true;
                }
                self.last_press_step[i] = self.step_index;
            }
        }
        // 方向键全部松开 ⇒ 奔跑结束（再按住要重新双击）
        if dir == (0, 0) {
            self.dash_armed = false;
        }
        // 官方奔跑档由**双击方向键**触发；进入奔跑后**切换方向仍然奔跑**
        // （官方奔跑状态在服务端持有，客户端只换方向符号 ⇒ `vmt37C`）。
        let running = dir != (0, 0) && self.dash_armed;

        // ---- 2. 起跳 ----
        if self.land_lock > 0 {
            self.land_lock -= 1;
        }
        let jump_key = keymap.get(Command::Jump);
        if input.just_pressed(jump_key) && !self.jump.active && self.land_lock == 0 {
            let profile = self.jump.start(
                jt,
                self.spec.jump_power,
                self.spec.weight,
                self.spec.jump_speed,
            );
            self.profile = Some(profile);
            // 官方把当下档位暂存到 +1719..1722，跳跃分支再写回 ⇒ 空中继承它
            self.air_mode = if running {
                MoveMode::Run
            } else {
                MoveMode::Walk
            };
            event = ActorEvent::Jumped;
        }

        // ---- 3. 地面平面位移 ----
        // 官方规则（`char_server_info.cpp:4291-4300` 起跳分支 + `case 6`）：
        //   * 0x6988 == 0（正常跳跃）：轴 0 写回**暂存的地面参数**（走 143/119 或跑 286/238），
        //     轴 1 **不写** ⇒ 空中继承地面档位，**两轴都能动**（可调纵深落点），并且**可改朝向**；
        //   * 0x6988 != 0（受控/技能跳跃）：轴 0 = 286/286、轴 1 = 0，且空中不能改朝向。
        //     本工程按正常跳跃实现（官方玩家跳跃即此条）。
        let dt_s = jt.logic_ms / 1000.0;
        let airborne = self.jump.active;
        let mode = if airborne {
            self.air_mode
        } else if running {
            MoveMode::Run
        } else {
            MoveMode::Walk
        };
        if !airborne || self.air_control {
            let (dx, dy) = self.mover.step(mt, self.spec.move_speed, mode, dir, dt_s);
            self.x += dx;
            self.y += dy;
        }
        self.facing = facing_from_dir(dir, self.facing);

        // ---- 4. 竖直弹道 ----
        let was_rising = self.jump.active && self.jump.rising;
        if self.jump.active {
            if let Some(p) = self.profile {
                if self.jump.step(&p, dt_s) {
                    event = ActorEvent::Landed;
                    self.land_lock = jt.landing_lock_steps;
                    // 官方 jump.ani 帧 15（50ms）是落地帧：落地后仍留在跳跃动画里播完它。
                    self.land_steps = self.land_frame_steps(jt);
                }
            }
        }

        // ---- 5. 状态与动画 ----
        self.motion = if self.jump.active || self.land_steps > 0 {
            Motion::Jump
        } else if dir == (0, 0) {
            Motion::Stand
        } else if running {
            Motion::Run
        } else {
            Motion::Walk
        };

        // 官方播放速率统一 100%（见文件头表格）；`[move speed]` 只缩放位移。
        let clip = match self.motion {
            Motion::Stand => self.clips.stand.clone(),
            Motion::Walk => self.clips.walk.clone(),
            Motion::Run => self.clips.run.clone(),
            Motion::Jump => self.clips.jump.clone(),
        };
        let restart = self.motion == Motion::Jump && matches!(event, ActorEvent::Jumped);
        self.anim.set_clip(clip, OFFICIAL_CLIP_RATE, restart);

        self.anim.advance(jt.logic_ms);

        // 官方 `jump.ani` 帧 7（顶点）与帧 14（落地）是 `[DELAY] 10000` 保持帧。
        // `sub_58FC30` case 6（`char_server_info.cpp:4913-4930`）在**物理**到达顶点时用
        // `sub_4AECE0(动画对象, *(this + 1717))` 把动画**直接跳到**顶点帧，落地时切到落地帧
        // ⇒ 段落切换由物理驱动，动画自己不会走完（所以不能只靠 advance）。
        if self.motion == Motion::Jump {
            let (apex, land) = self.jump_hold_frames();
            if was_rising && !self.jump.rising {
                self.anim.set_frame(apex);
                self.anim.release_hold();
            } else if matches!(event, ActorEvent::Landed) {
                self.anim.set_frame(land);
                self.anim.release_hold();
            }
        }
        if !self.jump.active && self.land_steps > 0 {
            self.land_steps -= 1;
        }

        event
    }

    /// 官方 `jump.ani` 的两个保持帧下标：`(顶点帧, 落地帧)`（= 首个/末个 `[DELAY] 10000`）。
    fn jump_hold_frames(&self) -> (usize, usize) {
        let holds: Vec<usize> = self
            .clips
            .jump
            .delays_ms
            .iter()
            .enumerate()
            .filter(|(_, d)| **d >= crate::anim::HOLD_DELAY_MS)
            .map(|(i, _)| i)
            .collect();
        match (holds.first(), holds.last()) {
            (Some(a), Some(b)) => (*a, *b),
            _ => (0, 0),
        }
    }

    /// 落地帧所占的逻辑步数（官方 `jump.ani` 末帧 50ms × `100 / jump speed`）。
    fn land_frame_steps(&self, jt: &JumpTuning) -> u32 {
        let last = self.clips.jump.delays_ms.last().copied().unwrap_or(0);
        let speed_ratio = if self.spec.jump_speed > 0.0 {
            100.0 / self.spec.jump_speed
        } else {
            1.0
        };
        let ms = last as f32 * speed_ratio;
        ((ms / jt.logic_ms).ceil() as u32).max(1)
    }

    /// 传送（换房间）：重置亚像素余量与跳跃，并清掉锁定。
    pub fn teleport(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
        self.mover.reset();
        self.axis_x.reset();
        self.axis_y.reset();
        self.jump = Jump::idle();
        self.profile = None;
        self.land_lock = 0;
        self.land_steps = 0;
    }
}
