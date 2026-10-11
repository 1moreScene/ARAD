//! 普通跳跃的垂直弹道（40JP）。
//! 495430 使用整数入参：v0 = 4000*power / ((100*weight/speed-10000)*0.0121+2100)。
//! weight/speed 先执行整数除法；power 来自 58E710 的属性块 +928。
//! 4957F0 模式2将 -1500 与 jump speed(+956) 整数加权；受控模式1不加权。
//! 49FD80 实际控制器积分是平均速度积分，常重力下等价于本模块抛物线解析式。
//! 角色状态机另处理准备(帧0→1)、飞行(顶点设帧8)、落地(设帧15)。
//! 普通跳跃两轴可控，y参数必须整数/2；完整证据与实现边界见 docs/02。

/// 重力基准来自官方；logic_ms 是重制客户端的采样步长。
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct JumpTuning {
    /// 官方重力基准（px/s²，向下为正）。官方 `sub_4957F0(1)` = -1500。
    pub gravity_base: f32,
    /// 逻辑步长（毫秒）。
    pub logic_ms: f64,
}

impl Default for JumpTuning {
    fn default() -> Self {
        JumpTuning {
            gravity_base: 1500.0,
            logic_ms: 1000.0 / 60.0,
        }
    }
}

/// 官方击倒发射常量：水平速度（`char_server_info.cpp:4203`）。
pub const KNOCKBACK_X: f32 = 200.0;
/// 官方击倒发射常量：竖直速度（`char_server_info.cpp:4205`）。
pub const KNOCKBACK_Z: f32 = 400.0;

/// 官方跳跃初速度公式（`sub_495430`，`anim_parser.cpp:4724`）。
///
/// * `jump_power` = `[jump power]`（官方样例注释：`1000` 为标准值）—— 公式的 `a1`；
///   技能则用技能的 `[static data]`（官方 `skill/swordman/backstep.skl` = `200`）。
/// * `weight` = `[weight]`（官方样例注释：单位 10g；属性块 `+984`）—— 公式的 `a2`；
/// * `jump_speed` = `[jump speed]`（属性块 `+956`，同时是重力缩放）—— 公式的 `a3`。
pub fn launch_velocity(jump_power: f32, weight: f32, jump_speed: f32) -> f32 {
    // 495430 uses signed integer arguments and IDIV before its floating point
    // denominator. Do not replace this integer division with float division.
    if !jump_power.is_finite()
        || !weight.is_finite()
        || !jump_speed.is_finite()
        || weight <= 0.0
        || jump_speed < 1.0
        || jump_power <= 0.0
    {
        return 0.0;
    }
    let denom = (((100 * weight as i64) / jump_speed as i64) - 10000) as f64 * 0.0121 + 2100.0;
    if denom <= 0.0 {
        return 0.0;
    }
    (4000.0 * jump_power.trunc() as f64 / denom) as f32
}

/// 官方重力（属性加权模式 = `sub_4957F0(this, 0)`）：`1500 × [jump speed] / 100`。
///
/// `[jump speed]` 越低 → 重力越小 → 滞空越久（鬼剑士 95 ⇒ 1425 px/s²）。
pub fn gravity_for(tuning: &JumpTuning, jump_speed: f32) -> f32 {
    if !jump_speed.is_finite() || jump_speed <= 0.0 {
        return 0.0;
    }
    (tuning.gravity_base as i64 * jump_speed as i64 / 100) as f32
}

/// 官方抛物线最高点 = `v² / (2g)`（`char_server_info.cpp:3950` 的 `v37*v37/(-2*g)`）。
pub fn apex_height(v0: f32, gravity: f32) -> f32 {
    if gravity <= 0.0 {
        return 0.0;
    }
    v0 * v0 / (2.0 * gravity)
}

/// 跳跃瞬时状态（`y` 用本工程的绘制约定：负 = 在脚点之上）。
///
/// 49FD80采用平均速度积分；恒定重力下等价于本模块的解析抛物线。
/// 连续弹道与帧率无关；可观察的顶点/碰地事件仍落在采样步边界。
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Jump {
    pub active: bool,
    /// 起跳以来经过的时间（秒）。
    pub t: f32,
    /// 当前竖直速度（px/s，负 = 上升）。
    pub vy: f32,
    /// 离地高度（px，负 = 在空中）。`0` = 站在地面。
    pub y: f32,
    /// 是否处于上升段。
    pub rising: bool,
}

impl Jump {
    pub const fn idle() -> Self {
        Jump {
            active: false,
            t: 0.0,
            vy: 0.0,
            y: 0.0,
            rising: false,
        }
    }

    /// 起跳。三个入参都来自官方 `.chr`：`[jump power]` / `[weight]` / `[jump speed]`。
    pub fn start(
        &mut self,
        tuning: &JumpTuning,
        jump_power: f32,
        weight: f32,
        jump_speed: f32,
    ) -> JumpProfile {
        let profile = JumpProfile::derive(tuning, jump_power, weight, jump_speed);
        self.active = true;
        self.rising = true;
        self.t = 0.0;
        self.vy = -profile.v0;
        self.y = 0.0;
        profile
    }

    /// 推进一个逻辑步（`dt_s` 秒）；返回是否本步落地。
    pub fn step(&mut self, profile: &JumpProfile, dt_s: f32) -> bool {
        if !self.active {
            return false;
        }
        self.t += dt_s;
        let t = self.t;
        self.vy = -profile.v0 + profile.gravity * t;
        self.y = -(profile.v0 * t - 0.5 * profile.gravity * t * t);
        if self.vy >= 0.0 {
            self.rising = false;
        }
        if self.y >= 0.0 {
            self.y = 0.0;
            self.vy = 0.0;
            self.t = 0.0;
            self.active = false;
            self.rising = false;
            return true;
        }
        false
    }

    /// 离地像素高度（正数；站在地面时为 `0.0`，不会是 `-0.0`）。
    pub fn altitude(&self) -> f32 {
        if self.y >= 0.0 {
            0.0
        } else {
            -self.y
        }
    }
}

/// 一次起跳的派生参数（全部由官方公式解析求出）。
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct JumpProfile {
    /// 初速度（px/s，向上为正）。
    pub v0: f32,
    /// 重力加速度（px/s²）。
    pub gravity: f32,
    /// 官方公式给出的最高点（px）。
    pub apex_height: f32,
    /// 预期上升段时长（毫秒）。
    pub rise_ms: f32,
    /// 预期滞空总时长（毫秒）。
    pub air_ms: f32,
}

impl JumpProfile {
    pub fn derive(tuning: &JumpTuning, jump_power: f32, weight: f32, jump_speed: f32) -> Self {
        let v0 = launch_velocity(jump_power, weight, jump_speed);
        let g = gravity_for(tuning, jump_speed);
        let rise_ms = if g > 0.0 { v0 / g * 1000.0 } else { 0.0 };
        JumpProfile {
            v0,
            gravity: g,
            apex_height: apex_height(v0, g),
            rise_ms,
            air_ms: rise_ms * 2.0,
        }
    }

    /// 总空中时间（毫秒）。
    pub fn total_air_ms(&self) -> f32 {
        self.air_ms
    }
}
