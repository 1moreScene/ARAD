//! 官方每轴速度参数、档位选择与浮点坐标积分。
//! 58E920 自由态：x143/119、y114/95；4B89D0 奔跑：x286/238、y136/114。
//! 普通跳跃 x 继承起跳档位，y参数整数/2；受控跳跃是另一分支(x286/286,y0)。
//! 495600 合成 v=参数*方向符号*move_speed/1000；任一轴无输入使用直向档。
//! 不作cos45°归一化。4996F0/499730按实际速度±5阈值生成动作方向槽。
//! 49FD80积分f32坐标，432F40四舍五入后查询碰撞/输出整数坐标。
//! Actor使用 Mover::step_position；step/step_velocity仅保留为整数位移辅助接口。
//! 左左/右右命令窗口500ms来自4B5B80，命令识别与有效奔跑分别见dash/actor。

/// 一个轴上的官方速度参数（单位 px/s）：直向档 / 斜向档。
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct AxisSpeed {
    /// 另一轴「无输入」时用（官方 +6608 族）。
    pub straight: f32,
    /// 两轴都有输入时用（官方 +6616 族）。
    pub diagonal: f32,
}

/// 官方自由行动状态（`sub_58FC30` case 0 / `sub_58E920` case 0）的 x 轴参数。
pub const FREE_X: AxisSpeed = AxisSpeed {
    straight: 143.0,
    diagonal: 119.0,
};

/// 官方自由行动状态的 y 轴（纵深轴）参数。
pub const FREE_Y: AxisSpeed = AxisSpeed {
    straight: 114.0,
    diagonal: 95.0,
};

/// 官方**奔跑（冲刺）**的 x 轴参数（`src/game/equ_buff.cpp:7172`：
/// `vt+876(this, 0, 286, 238, 1, 0, 0)`；`:7290-7293` 把同一组存进 `+1719..1722`）。
pub const RUN_X: AxisSpeed = AxisSpeed {
    straight: 286.0,
    diagonal: 238.0,
};

/// 官方**奔跑（冲刺）**的 y 轴（纵深）参数（同处 `equ_buff.cpp:7291-7293`）。
///
/// ⚠ 注意：奔跑的纵深比（`136/286 = 114/238 ≈ 0.476`）与**步行**的 0.797 **不同**
/// ⇒ 不能用比值推，官方就是这组数。
pub const RUN_Y: AxisSpeed = AxisSpeed {
    straight: 136.0,
    diagonal: 114.0,
};

/// 官方**无地面控制型跳跃**的参数（`char_server_info.cpp:4270`：轴 0 = `286, 286`、
/// 轴 1 = `0, 0`）——空中只能横向移动、**不能**调纵深、也**不能**改朝向
/// （`case 6` 的改向条件要求 `*(this+6988) == 0`）。该分支由**包载荷**第 4 字节决定
/// （`:4222` 的 `*(this+6988) = 包[4] != 0`），是技能类/受控跳跃用的。
pub const AIR_LOCK_X: AxisSpeed = AxisSpeed {
    straight: 286.0,
    diagonal: 286.0,
};

/// 地面档位。普通跳跃保留x参数，y参数在 air_velocity_for 中整数除以2。
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum MoveMode {
    Walk,
    Run,
}

/// 移动参数（**全部是官方数**）。
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct MoveTuning {
    /// 速度基准：`[move speed]` 的 100% 值（官方 = 1000）。
    pub speed_basis: f32,
    /// 自由行动 x 轴官方参数。
    pub walk_x: AxisSpeed,
    /// 自由行动 y 轴官方参数。
    pub walk_y: AxisSpeed,
    /// 奔跑（冲刺）x 轴官方参数。
    pub run_x: AxisSpeed,
    /// 奔跑（冲刺）y 轴（纵深）官方参数。
    pub run_y: AxisSpeed,
}

impl Default for MoveTuning {
    fn default() -> Self {
        MoveTuning {
            speed_basis: 1000.0,
            walk_x: FREE_X,
            walk_y: FREE_Y,
            run_x: RUN_X,
            run_y: RUN_Y,
        }
    }
}

impl MoveTuning {
    /// 由 `[move speed]` 属性值得到速度倍率（`850/1000 = 0.85`）。
    ///
    /// 官方依据：`IRDActiveObject_vmt37C` 的 `x this_[1663] x 0.001`（千分比），
    /// 以及 `sample/charactersample.chr` 的官方注释 `[move speed] 1000  // 1000이 표시상 100%`。
    pub fn speed_ratio(&self, move_speed: f32) -> f32 {
        if self.speed_basis <= 0.0 {
            return 1.0;
        }
        move_speed / self.speed_basis
    }

    /// 该档位的两轴官方参数表。
    pub fn params(&self, mode: MoveMode) -> (AxisSpeed, AxisSpeed) {
        match mode {
            MoveMode::Walk => (self.walk_x, self.walk_y),
            MoveMode::Run => (self.run_x, self.run_y),
        }
    }

    /// 官方档位选择：任一轴无输入（方向值 4）→ 直向档，否则斜向档。
    pub fn select(&self, mode: MoveMode, dir: (i32, i32)) -> (f32, f32) {
        let (x, y) = self.params(mode);
        select_axis(x, y, dir)
    }

    /// 目标速度（px/s，未取整）。
    ///
    /// 朝向符号已含「无输入 → 0」的官方语义（`vmt37C` 里 `v12` 初值 0.0）。
    pub fn velocity_for(&self, move_speed: f32, mode: MoveMode, dir: (i32, i32)) -> (f32, f32) {
        let (dx, dy) = dir;
        if dx == 0 && dy == 0 {
            return (0.0, 0.0);
        }
        let ratio = self.speed_ratio(move_speed);
        let (px, py) = self.select(mode, dir);
        (
            px * ratio * dx.signum() as f32,
            py * ratio * dy.signum() as f32,
        )
    }

    /// Normal jump: x retains the takeoff parameters; y uses signed integer /2
    /// BEFORE scaling (58F5BC..58F5DF). Walking y diagonal is 95/2 = 47, not 47.5.
    pub fn air_velocity_for(&self, move_speed: f32, mode: MoveMode, dir: (i32, i32)) -> (f32, f32) {
        let (x, y) = self.params(mode);
        let y = AxisSpeed {
            straight: (y.straight as i32 / 2) as f32,
            diagonal: (y.diagonal as i32 / 2) as f32,
        };
        let (vx, vy) = select_axis(x, y, dir);
        let ratio = self.speed_ratio(move_speed);
        (
            vx * ratio * dir.0.signum() as f32,
            vy * ratio * dir.1.signum() as f32,
        )
    }
}

/// 官方档位选择（`IRDActiveObject_vmt36C` 末尾的 `flag`）：
/// **任一轴方向值 == 4（无输入）→ 直向档，否则斜向档**。
pub fn select_axis(x: AxisSpeed, y: AxisSpeed, dir: (i32, i32)) -> (f32, f32) {
    let straight = dir.0 == 0 || dir.1 == 0;
    let xv = if straight { x.straight } else { x.diagonal };
    let yv = if straight { y.straight } else { y.diagonal };
    (xv, yv)
}

/// 亚像素余量累积器（每轴一个）。
#[derive(Copy, Clone, Debug, Default)]
pub struct Mover {
    frac_x: f32,
    frac_y: f32,
    position: Option<(f32, f32)>,
}

impl Mover {
    pub const fn new() -> Self {
        Mover {
            frac_x: 0.0,
            frac_y: 0.0,
            position: None,
        }
    }

    /// 推进一个逻辑步（`dt_s` = 该步秒数），返回**整数像素位移**。
    ///
    /// 整数位移辅助接口；角色实际更新使用 step_position 的浮点坐标。
    pub fn step(
        &mut self,
        tuning: &MoveTuning,
        move_speed: f32,
        mode: MoveMode,
        dir: (i32, i32),
        dt_s: f32,
    ) -> (i32, i32) {
        let (vx, vy) = tuning.velocity_for(move_speed, mode, dir);
        self.step_velocity((vx, vy), dt_s)
    }

    pub fn step_velocity(&mut self, velocity: (f32, f32), dt_s: f32) -> (i32, i32) {
        let (vx, vy) = velocity;
        if vx == 0.0 && vy == 0.0 {
            // Zero speed does not erase the floating position's fractional part.
            return (0, 0);
        }
        self.frac_x += vx * dt_s;
        self.frac_y += vy * dt_s;
        let ix = self.frac_x.trunc();
        let iy = self.frac_y.trunc();
        self.frac_x -= ix;
        self.frac_y -= iy;
        (ix as i32, iy as i32)
    }

    /// Official controller 49FD80 integrates floating positions. 4A0650 reads
    /// them through 432F40 (round to nearest, halves away from zero).
    /// Public integer coordinates may be clamped by the room; synchronize that
    /// axis if the caller changed the last rounded coordinate.
    pub fn step_position(
        &mut self,
        position: (i32, i32),
        velocity: (f32, f32),
        dt_s: f32,
    ) -> (i32, i32) {
        let (mut x, mut y) = self
            .position
            .unwrap_or((position.0 as f32, position.1 as f32));
        if x.round() as i32 != position.0 {
            x = position.0 as f32;
        }
        if y.round() as i32 != position.1 {
            y = position.1 as f32;
        }
        x += velocity.0 * dt_s;
        y += velocity.1 * dt_s;
        self.position = Some((x, y));
        (x.round() as i32, y.round() as i32)
    }

    pub fn position(&self) -> Option<(f32, f32)> {
        self.position
    }

    /// 传送（换房间 / 传送门）时重置余量。
    pub fn reset(&mut self) {
        self.frac_x = 0.0;
        self.frac_y = 0.0;
        self.position = None;
    }

    pub fn frac(&self) -> (f32, f32) {
        (self.frac_x, self.frac_y)
    }
}

/// 8 方向 → 朝向（1 = 右，-1 = 左）。水平分量为 0 时保持原朝向。
///
/// 官方是 **4 档朝向槽**（`vmt37C`：0/2 = 左、1/3 = 右），并且镜像靠
/// **动画速率取负号**实现；本工程把它折叠成 ±1 + `flip`，绘制结果等价。
pub fn facing_from_dir(dir: (i32, i32), facing: i32) -> i32 {
    if dir.0 > 0 {
        1
    } else if dir.0 < 0 {
        -1
    } else {
        facing
    }
}
