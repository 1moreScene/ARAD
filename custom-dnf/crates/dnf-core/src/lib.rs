//! # dnf-core —— DNF 同人客户端逻辑核心
//!
//! 无第三方依赖。包含：
//!
//! * [`keys`] —— 官方 DirectInput 扫描码（DIK）常量表
//! * [`keymap`] —— 官方默认键位映射（带取证等级）
//! * [`input`] —— 每帧键盘状态（当前/上一帧边沿）
//! * [`movement`] —— 官方速度参数组（143/119 · 114/95 · 286）+ 朝向符号 + 亚像素余量
//! * [`jump`] —— 官方跳跃公式（整数 weight/jump_speed）、属性加权重力
//! * [`anim`] —— `.ani` 帧序列播放（含官方「保持帧」语义）
//! * [`actor`] —— 站立 / 步行 / 奔跑 / 跳跃状态机
//! * [`clock`] —— 固定步长逻辑时钟

pub mod actor;
pub mod anim;
pub mod clock;
pub mod dash;
pub mod device_slots;
pub mod feedback;
pub mod input;
pub mod jump;
pub mod key_manager;
pub mod keymap;
pub mod keys;
pub mod movement;

pub use actor::{Actor, ActorClips, ActorEvent, ActorSpec, JumpPhase, Motion};
pub use anim::{AnimPlayer, Clip, HOLD_DELAY_MS};
pub use clock::{FixedClock, LOGIC_MS};
pub use input::InputState;
pub use jump::{
    apex_height, launch_velocity, Jump, JumpProfile, JumpTuning, KNOCKBACK_X, KNOCKBACK_Z,
};
pub use keymap::{command_name, Binding, Command, KeyMap, Provenance, COMMAND_COUNT};
pub use keys::{Dik, DIRECTIONS};
pub use movement::{
    select_axis, AxisSpeed, MoveMode, MoveTuning, Mover, AIR_LOCK_X, FREE_X, FREE_Y, RUN_X, RUN_Y,
};

#[cfg(test)]
mod tests {
    use super::*;

    fn keymap() -> KeyMap {
        KeyMap::official_default()
    }

    #[test]
    fn 方向键与官方二进制一致() {
        let m = keymap();
        // ARAD.exe sub_415240 槽 0..3
        assert_eq!(m.get(Command::Left).code(), 0xCB);
        assert_eq!(m.get(Command::Right).code(), 0xCD);
        assert_eq!(m.get(Command::Up).code(), 0xC8);
        assert_eq!(m.get(Command::Down).code(), 0xD0);
        for c in [Command::Up, Command::Down, Command::Left, Command::Right] {
            assert!(m.provenance(c).is_confirmed(), "{:?} 应为已确证", c);
        }
    }

    #[test]
    fn 命令词表_顺序与官方_dat_索引一致() {
        // Skill/RDARKeyIndex.dat: 0 (UP) 1 (DOWN) 2 (LEFT) 3 (RIGHT) 4 (SKILL) 5 (ATTACK) 6 (JUMP) 7 (CREATURE) 8 (BUFF)
        assert_eq!(Command::Up.index(), 0);
        assert_eq!(Command::Down.index(), 1);
        assert_eq!(Command::Left.index(), 2);
        assert_eq!(Command::Right.index(), 3);
        assert_eq!(Command::Skill.index(), 4);
        assert_eq!(Command::Attack.index(), 5);
        assert_eq!(Command::Jump.index(), 6);
        assert_eq!(Command::Creature.index(), 7);
        assert_eq!(Command::Buff.index(), 8);
        assert_eq!(Command::Skill.token(), Some("(SKILL)"));
        assert_eq!(Command::Buff.token(), Some("(BUFF)"));
        assert_eq!(Command::SkillSlot1.token(), None);
    }

    #[test]
    fn 键位表可覆盖() {
        let mut m = keymap();
        let warn = m.load_overrides("# 注释\nJump=0x39\nAttack = 0x2D\nBogus=5\n");
        assert_eq!(m.get(Command::Jump), Dik::SPACE);
        assert_eq!(m.get(Command::Attack), Dik::X);
        assert_eq!(warn.len(), 1);
        // 改动过的键降级为未验证
        assert!(!m.provenance(Command::Jump).is_confirmed());
        // 原样写回官方默认值 → 取证等级保留
        let mut m2 = keymap();
        m2.load_overrides("Left=0xCB\nUp=0xC8\nDown=0xD0\nRight=0xCD\n");
        for c in [Command::Up, Command::Down, Command::Left, Command::Right] {
            assert!(m2.provenance(c).is_confirmed(), "{:?} 取证等级被误降级", c);
        }
    }

    #[test]
    fn 输入边沿检测() {
        let mut st = InputState::new();
        st.begin_frame();
        st.set(Dik::X, true);
        assert!(st.is_down(Dik::X));
        assert!(st.just_pressed(Dik::X));
        st.begin_frame();
        st.set(Dik::X, true);
        assert!(st.is_down(Dik::X));
        assert!(!st.just_pressed(Dik::X));
        st.begin_frame();
        assert!(st.just_released(Dik::X));
    }

    #[test]
    fn 亚像素余量长期精确() {
        let mut mv = Mover::new();
        let t = MoveTuning::default();
        // 官方：走 x = 143 px/s × 850/1000 = 121.55 px/s；60Hz 下 600 步 = 10 秒 = 1215.5 px
        let mut total = 0i64;
        for _ in 0..600 {
            total += mv.step(&t, 850.0, MoveMode::Walk, (1, 0), 1.0 / 60.0).0 as i64;
        }
        assert!((total - 1215).abs() <= 1, "total={}", total);
    }

    #[test]
    fn 官方速度参数表() {
        let t = MoveTuning::default();
        // 直向（另一轴无输入）：官方 sub_58E920 case 0 的 143 / 114
        assert_eq!(t.velocity_for(1000.0, MoveMode::Walk, (1, 0)), (143.0, 0.0));
        assert_eq!(t.velocity_for(1000.0, MoveMode::Walk, (0, 1)), (0.0, 114.0));
        // 斜向（两轴都有输入）：官方 119 / 95 —— 注意**不是** 707 归一化
        assert_eq!(
            t.velocity_for(1000.0, MoveMode::Walk, (1, 1)),
            (119.0, 95.0)
        );
        // 左朝向取负（官方 vmt37C：朝向 0/2 → -1）
        assert_eq!(
            t.velocity_for(1000.0, MoveMode::Walk, (-1, -1)),
            (-119.0, -95.0)
        );
        // 无输入 → 速率 0（官方 v12 初值 0.0）
        assert_eq!(t.velocity_for(1000.0, MoveMode::Walk, (0, 0)), (0.0, 0.0));
    }

    #[test]
    fn 官方奔跑参数组() {
        let t = MoveTuning::default();
        // 官方 equ_buff.cpp:7172 / :7290-7293：x 直 286 / 斜 238，y 直 136 / 斜 114
        let (walk, _) = t.velocity_for(1000.0, MoveMode::Walk, (1, 0));
        assert_eq!(t.velocity_for(1000.0, MoveMode::Run, (1, 0)), (286.0, 0.0));
        assert_eq!(t.velocity_for(1000.0, MoveMode::Run, (0, 1)), (0.0, 136.0));
        assert_eq!(
            t.velocity_for(1000.0, MoveMode::Run, (1, 1)),
            (238.0, 114.0)
        );
        assert_eq!(
            t.velocity_for(1000.0, MoveMode::Run, (-1, -1)),
            (-238.0, -114.0)
        );
        // 奔跑直向 = 2 × 行走直向
        assert!((286.0 - walk * 2.0).abs() < 1e-6);
        // 奔跑纵深比（0.476）与行走纵深比（0.797）**不同** ⇒ 必须各用各的官方数
        let (_, walk_y) = t.velocity_for(1000.0, MoveMode::Walk, (0, 1));
        assert!((walk_y / walk - 0.797f32).abs() < 0.002);
        assert!((136.0f32 / 286.0 - 0.4755).abs() < 0.001);
    }

    #[test]
    fn 移动速度属性按千分基准缩放() {
        let t = MoveTuning::default();
        let (v850, _) = t.velocity_for(850.0, MoveMode::Walk, (1, 0));
        let (v1000, _) = t.velocity_for(1000.0, MoveMode::Walk, (1, 0));
        assert!((v850 - v1000 * 0.85).abs() < 1e-6);
        // 鬼剑士实测：143 × 0.85 = 121.55 px/s
        assert!((v850 - 121.55).abs() < 1e-4);
    }

    #[test]
    fn 跳跃_官方公式与重力() {
        let jt = JumpTuning::default();
        let mut j = Jump::idle();
        let p = j.start(&jt, 430.0, 68000.0, 95.0);
        // 官方 sub_495430：4000·a1 / ((100·a2/a3 − 10000)·0.0121 + 2100)
        //   a1 = [jump power] 430、a2 = [weight] 68000、a3 = [jump speed] 95
        let expect_v0 = 4000.0 * 430.0 / (((100 * 68000 / 95 - 10000) as f64) * 0.0121 + 2100.0);
        assert!((p.v0 as f64 - expect_v0).abs() < 1e-3, "v0={}", p.v0);
        assert!((p.v0 - 604.6).abs() < 0.5, "v0={}", p.v0);
        // 官方重力 = 1500 × [jump speed]/100（属性加权模式，sub_4957F0(0)）
        assert!((p.gravity - 1425.0).abs() < 1e-4, "g={}", p.gravity);
        // 最高点 = v²/(2g)（官方 case 4 的 v37*v37/(-2*g)）
        assert!((p.apex_height - p.v0 * p.v0 / (2.0 * 1425.0)).abs() < 1e-4);
        assert!(
            (p.apex_height - 128.3).abs() < 1.0,
            "apex={}",
            p.apex_height
        );
        // 上升 424ms、滞空 849ms
        assert!((p.rise_ms - 424.3).abs() < 1.5, "rise={}", p.rise_ms);
        assert!(
            (p.total_air_ms() - 848.6).abs() < 3.0,
            "air={}",
            p.total_air_ms()
        );
    }

    #[test]
    fn 跳跃_离地并落地() {
        let jt = JumpTuning::default();
        let mut j = Jump::idle();
        let p = j.start(&jt, 430.0, 68000.0, 95.0);
        assert!(j.active);
        let mut max_alt = 0.0f32;
        let mut steps = 0;
        let mut landed = false;
        for _ in 0..600 {
            if j.step(&p, (jt.logic_ms / 1000.0) as f32) {
                landed = true;
                break;
            }
            steps += 1;
            max_alt = max_alt.max(j.altitude());
        }
        assert!(landed, "应当落地");
        // 官方解析积分：最高点 = v²/(2g) = 128px（与帧率无关，采样点略有偏差）
        assert!((max_alt - p.apex_height).abs() < 0.2, "max={}", max_alt);
        // 滞空 = 2v/g = 849ms ⇒ 60Hz 下第 51 步落地
        assert!((steps as f32 - 50.0).abs() <= 2.0, "steps={}", steps);
        assert_eq!(j.altitude(), 0.0);
    }

    #[test]
    fn 跳跃公式_三个入参都是官方字段() {
        // a1 = [jump power]：线性
        let base = launch_velocity(430.0, 68000.0, 95.0);
        assert!((launch_velocity(860.0, 68000.0, 95.0) - 2.0 * base).abs() < 1e-3);
        // a2 = [weight]：越重跳越低
        assert!(launch_velocity(430.0, 136000.0, 95.0) < base);
        // a3 = [jump speed]：越大分母越小 → 初速越大（同时重力也更大，见 gravity_for）
        assert!(launch_velocity(430.0, 68000.0, 190.0) > base);
        // 官方击飞常量：水平 200、竖直 400（char_server_info.cpp:4203-4205）
        assert_eq!((KNOCKBACK_X, KNOCKBACK_Z), (200.0, 400.0));
        assert!((apex_height(KNOCKBACK_Z, 1500.0) - 53.33).abs() < 0.05);
        // 官方后跳技能 [static data] 200 ⇒ 一次小跳（约 28px / 395ms）
        let back = JumpProfile::derive(&JumpTuning::default(), 200.0, 68000.0, 95.0);
        assert!(
            (back.apex_height - 27.7).abs() < 0.5,
            "back={}",
            back.apex_height
        );
        assert!((back.total_air_ms() - 394.6).abs() < 2.0);
    }

    #[test]
    fn 方向_后按的键优先_松开后回落() {
        let km = keymap();
        let mut a = Actor::new(
            ActorSpec::default(),
            ActorClips::swordman_official(),
            300,
            200,
        );
        let mut input = InputState::new();

        // 按住 ←
        input.begin_frame();
        input.set(km.get(Command::Left), true);
        idle_step(&mut a, &input, &km);
        assert_eq!(a.facing, -1, "按住左应向朝左");

        // 再按住 →（官方：后按覆盖，不停住）
        input.begin_frame();
        input.set(km.get(Command::Left), true);
        input.set(km.get(Command::Right), true);
        let x0 = a.x;
        idle_step(&mut a, &input, &km);
        assert_eq!(a.facing, 1, "后按 → 应转向右，而不是停住");
        assert_eq!(a.motion, Motion::Walk);
        assert!(a.x > x0, "后按 → 应向右移动（官方 sub_4BF3D0 取最近记录）");

        // 松开 →（← 仍按住）→ 自动回落到左
        input.begin_frame();
        input.set(km.get(Command::Left), true);
        let x1 = a.x;
        idle_step(&mut a, &input, &km);
        assert_eq!(a.facing, -1, "松开后应回落到仍按住的左");
        assert!(a.x < x1);

        // 两键都松开 → 无输入（官方方向值 4）
        input.begin_frame();
        idle_step(&mut a, &input, &km);
        assert_eq!(a.motion, Motion::Stand);
    }

    #[test]
    fn 跳跃_正常跳跃继承地面档位且两轴可动() {
        let km = keymap();
        let mut a = Actor::new(
            ActorSpec::default(),
            ActorClips::swordman_official(),
            300,
            200,
        );
        let mut input = InputState::new();

        // ① 原地起跳（走档）→ 空中按右，位移 = 走速 143×0.85 = 121.55 px/s
        input.begin_frame();
        input.set(km.get(Command::Jump), true);
        idle_step(&mut a, &input, &km);
        assert_eq!(a.motion, Motion::Jump);
        for _ in 0..5 {
            input.begin_frame();
            idle_step(&mut a, &input, &km);
        }
        assert_eq!(a.x, 300, "原地起跳不应偏移");
        // Preparation finishes at the 100ms frame-1 boundary. Launch has no
        // plane displacement; subsequent steps use the air parameters.
        while !a.is_airborne() {
            input.begin_frame();
            idle_step(&mut a, &input, &km);
        }
        let x0 = a.x;
        for _ in 0..10 {
            input.begin_frame();
            input.set(km.get(Command::Right), true);
            idle_step(&mut a, &input, &km);
        }
        let dx = (a.x - x0) as f32;
        // 走档 143×0.85 = 121.55 px/s ⇒ 10 步 ≈ 20.3px（±2 取整）
        assert!((dx - 20.3).abs() <= 2.0, "空中走档位移: dx={}", dx);
        assert_eq!(a.facing, 1);

        // ② 空中改按上：纵深也要能动（官方普通跳跃y参数114/95整数除以2）
        let y0 = a.y;
        for _ in 0..10 {
            input.begin_frame();
            input.set(km.get(Command::Up), true);
            idle_step(&mut a, &input, &km);
        }
        let dy = (y0 - a.y) as f32;
        // Official y parameter 114/2 = 57; 57×0.85 = 48.45 px/s.
        assert!((dy - 8.075).abs() <= 1.0, "空中纵深应减半: dy={}", dy);

        // ③ 空中改朝向（官方 case 6 的 vt+148/vt+144）
        input.begin_frame();
        input.set(km.get(Command::Left), true);
        idle_step(&mut a, &input, &km);
        assert_eq!(a.facing, -1, "空中应能改朝向");
        // ④ 落地后继续正常走
        for _ in 0..120 {
            input.begin_frame();
            input.set(km.get(Command::Left), true);
            idle_step(&mut a, &input, &km);
            if !a.is_airborne() && a.motion == Motion::Walk {
                break;
            }
        }
        assert!(!a.is_airborne() || a.motion == Motion::Walk);
    }

    #[test]
    fn 跳跃中不能改变纵深_仅限受控跳跃分支() {
        // 官方 `0x6988 != 0` 的受控/技能跳跃分支把轴 1 写 0；本工程按正常跳跃（继承档位）实现，
        // 因此这里只锁住参数常量本身（AIR_LOCK_X）与它代表的语义。
        assert_eq!(AIR_LOCK_X.straight, 286.0);
        assert_eq!(AIR_LOCK_X.diagonal, 286.0);
    }

    #[test]
    fn 按住一个方向时点另一个方向不触发奔跑() {
        let km = keymap();
        let mut a = Actor::new(
            ActorSpec::default(),
            ActorClips::swordman_official(),
            300,
            200,
        );
        let mut input = InputState::new();

        // 从未双击 ⇒ 按住右并**连续快速点按左**；输入可以产生双击候选，但不能建立持续奔跑
        input.begin_frame();
        input.set(km.get(Command::Right), true);
        idle_step(&mut a, &input, &km);
        for _ in 0..40 {
            for down in [false, true] {
                input.begin_frame();
                input.set(km.get(Command::Right), true);
                if down {
                    input.set(km.get(Command::Left), true);
                }
                idle_step(&mut a, &input, &km);
            }
            assert_ne!(
                a.motion,
                Motion::Run,
                "按住一键点反向键不应建立持续奔跑状态"
            );
        }
    }
    #[test]
    fn 双击同一方向键触发奔跑() {
        let km = keymap();
        let mut a = Actor::new(
            ActorSpec::default(),
            ActorClips::swordman_official(),
            300,
            200,
        );
        let mut input = InputState::new();
        // 单击 → 走
        input.begin_frame();
        input.set(km.get(Command::Right), true);
        idle_step(&mut a, &input, &km);
        assert_eq!(a.motion, Motion::Walk);
        // 松开后快速再按（双击）→ 跑
        input.begin_frame();
        idle_step(&mut a, &input, &km);
        input.begin_frame();
        input.set(km.get(Command::Right), true);
        idle_step(&mut a, &input, &km);
        assert_eq!(a.motion, Motion::Run, "双击同一方向键应触发奔跑");
        // 按住保持奔跑
        for _ in 0..20 {
            input.begin_frame();
            input.set(km.get(Command::Right), true);
            idle_step(&mut a, &input, &km);
        }
        assert_eq!(a.motion, Motion::Run);
        // 奔跑中改按左 ⇒ 退出状态11，回自由移动
        input.begin_frame();
        input.set(km.get(Command::Left), true);
        idle_step(&mut a, &input, &km);
        assert_eq!(a.motion, Motion::Walk, "奔跑反向应退出状态11");
    }

    #[test]
    fn 奔跑起跳后空中可反向() {
        let km = keymap();
        let mut a = Actor::new(
            ActorSpec::default(),
            ActorClips::swordman_official(),
            300,
            200,
        );
        let mut input = InputState::new();
        // 双击右 → 奔跑
        for down in [false, true, false, true] {
            input.begin_frame();
            if down {
                input.set(km.get(Command::Right), true);
            }
            idle_step(&mut a, &input, &km);
        }
        input.begin_frame();
        input.set(km.get(Command::Right), true);
        idle_step(&mut a, &input, &km);
        assert_eq!(a.motion, Motion::Run);
        // 奔跑中起跳（官方：空中 x 用 286 档，与奔跑直向同值）
        input.begin_frame();
        input.set(km.get(Command::Right), true);
        input.set(km.get(Command::Jump), true);
        idle_step(&mut a, &input, &km);
        assert_eq!(a.motion, Motion::Jump);
        while !a.is_airborne() {
            input.begin_frame();
            input.set(km.get(Command::Right), true);
            idle_step(&mut a, &input, &km);
        }
        let x0 = a.x;
        for _ in 0..5 {
            input.begin_frame();
            input.set(km.get(Command::Right), true);
            idle_step(&mut a, &input, &km);
        }
        assert!(a.x > x0, "空中应保持水平移动");
        // 空中改按左 ⇒ 立刻反向（官方 case 6 允许改朝向）
        let x1 = a.x;
        for i in 0..6 {
            input.begin_frame();
            input.set(km.get(Command::Left), true);
            idle_step(&mut a, &input, &km);
            if i == 0 {
                assert_eq!(a.facing, -1, "空中应立刻转向");
            }
        }
        let dx = (x1 - a.x) as f32;
        assert!(
            (dx - 24.0).abs() <= 1.0,
            "空中水平速度应恒为 286×0.85: dx={}",
            dx
        );
    }

    #[test]
    fn 奔跑反向退回步行且不恢复奔跑() {
        let km = keymap();
        let mut a = Actor::new(
            ActorSpec::default(),
            ActorClips::swordman_official(),
            300,
            200,
        );
        let mut input = InputState::new();

        // 双击右 → 奔跑
        for down in [false, true, false, true] {
            input.begin_frame();
            if down {
                input.set(km.get(Command::Right), true);
            }
            idle_step(&mut a, &input, &km);
        }
        input.begin_frame();
        input.set(km.get(Command::Right), true);
        idle_step(&mut a, &input, &km);
        assert_eq!(a.motion, Motion::Run);
        assert!(a.is_running_armed(), "双击后应处于奔跑姿态");

        // 保持奔跑，直接改按左（官方：状态11实际移动方向与朝向不一致时请求状态0）
        input.begin_frame();
        input.set(km.get(Command::Left), true);
        idle_step(&mut a, &input, &km);
        assert_eq!(a.facing, -1, "应立刻转向");
        assert_eq!(a.motion, Motion::Walk, "反向退出奔跑");
        assert!(!a.is_running_armed());
        let x0 = a.x;
        for _ in 0..20 {
            input.begin_frame();
            input.set(km.get(Command::Left), true);
            idle_step(&mut a, &input, &km);
        }
        // Walk 143×0.85 = 121.55 px/s ⇒ 20 steps ≈ 40.5px.
        let dx = (x0 - a.x) as f32;
        assert!((dx - 40.5).abs() <= 2.0, "反向步行位移: dx={}", dx);

        // 全部松开 ⇒ 奔跑结束，再按住要重新双击
        input.begin_frame();
        idle_step(&mut a, &input, &km);
        assert!(!a.is_running_armed(), "松开方向键后奔跑结束");
        input.begin_frame();
        input.set(km.get(Command::Right), true);
        idle_step(&mut a, &input, &km);
        assert_eq!(a.motion, Motion::Walk, "松开后重新按住应回到走");
    }

    #[test]
    fn 长按一个方向_各种频率点按反向键都不奔跑() {
        let km = keymap();
        for pattern in [1usize, 2, 3, 5, 8] {
            for hold in [Command::Right, Command::Left] {
                let tap = if hold == Command::Right {
                    Command::Left
                } else {
                    Command::Right
                };
                let mut a = Actor::new(
                    ActorSpec::default(),
                    ActorClips::swordman_official(),
                    300,
                    200,
                );
                let mut input = InputState::new();
                for step in 0..200u32 {
                    input.begin_frame();
                    input.set(km.get(hold), true);
                    // 每 pattern 帧点按一次反向键（按下 pattern/2 帧、松开 pattern/2 帧）
                    let phase = (step as usize) % (pattern * 2);
                    if phase < pattern {
                        input.set(km.get(tap), true);
                    }
                    idle_step(&mut a, &input, &km);
                    assert_ne!(
                        a.motion,
                        Motion::Run,
                        "pattern={} hold={:?} step={} 不该奔跑",
                        pattern,
                        hold,
                        step
                    );
                }
            }
        }
    }

    #[test]
    fn 反向点按后松开_回落到仍按住的方向且不奔跑() {
        let km = keymap();
        let mut a = Actor::new(
            ActorSpec::default(),
            ActorClips::swordman_official(),
            300,
            200,
        );
        let mut input = InputState::new();
        // 按住左 10 帧
        for _ in 0..10 {
            input.begin_frame();
            input.set(km.get(Command::Left), true);
            idle_step(&mut a, &input, &km);
        }
        assert_eq!(a.facing, -1);
        assert_eq!(a.motion, Motion::Walk);
        // 点按右 3 帧（后按优先 → 朝右），期间不得奔跑
        for _ in 0..3 {
            input.begin_frame();
            input.set(km.get(Command::Left), true);
            input.set(km.get(Command::Right), true);
            idle_step(&mut a, &input, &km);
            assert_eq!(a.motion, Motion::Walk, "点按反向键不得奔跑");
        }
        assert_eq!(a.facing, 1, "后按的右键应优先");
        // 松开右 ⇒ 回落到左
        for _ in 0..10 {
            input.begin_frame();
            input.set(km.get(Command::Left), true);
            idle_step(&mut a, &input, &km);
            assert_eq!(a.motion, Motion::Walk);
        }
        assert_eq!(a.facing, -1, "松开后应回落");
    }

    #[test]
    fn 动画_保持帧会停住() {
        let mut p = AnimPlayer::new(Clip::new("Jump", vec![100, 100, 10000, 50], false), 100.0);
        for _ in 0..50 {
            p.advance(100.0);
        }
        assert_eq!(p.frame(), 2);
        assert!(p.finished());
    }

    #[test]
    fn 动画_循环回绕() {
        let mut p = AnimPlayer::new(Clip::new("Move", vec![100; 8], true), 100.0);
        let mut seen = [false; 8];
        for _ in 0..800 {
            p.advance(10.0);
            seen[p.frame()] = true;
        }
        assert!(seen.iter().all(|s| *s));
    }

    #[test]
    fn 动画_速率按千分值0_1换算() {
        let mut p = AnimPlayer::new(Clip::new("Move", vec![100; 8], true), 85.0);
        // 100ms 的帧在 85% 速率下需 117.6ms
        assert!((p.frame_ms() - 100.0 * 100.0 / 85.0).abs() < 1e-3);
        p.advance(117.7);
        assert_eq!(p.frame(), 1);
    }

    #[test]
    fn 逻辑时钟_整数步与余量() {
        let mut c = FixedClock::default();
        assert_eq!(c.advance(10.0), 0);
        assert_eq!(c.advance(10.0), 1);
        assert_eq!(c.advance(100.0), 6);
    }

    fn idle_step(actor: &mut Actor, input: &InputState, km: &KeyMap) -> ActorEvent {
        actor.step(input, km, &MoveTuning::default(), &JumpTuning::default())
    }

    #[test]
    fn 状态机_站立步行奔跑跳跃() {
        let km = keymap();
        let mut a = Actor::new(
            ActorSpec::default(),
            ActorClips::swordman_official(),
            300,
            200,
        );
        let mut input = InputState::new();

        input.begin_frame();
        idle_step(&mut a, &input, &km);
        assert_eq!(a.motion, Motion::Stand);

        // 按住 → 步行（未达奔跑阈值）
        input.begin_frame();
        input.set(km.get(Command::Right), true);
        idle_step(&mut a, &input, &km);
        assert_eq!(a.motion, Motion::Walk);
        assert_eq!(a.facing, 1);

        // 按住不放**不会**自动奔跑（按住切入官方未确证，本工程只认双击）⇒ 仍是步行
        for _ in 0..40 {
            input.begin_frame();
            input.set(km.get(Command::Right), true);
            idle_step(&mut a, &input, &km);
        }
        assert_eq!(a.motion, Motion::Walk, "按住只走（官方 143 档）");
        let x_after_walk = a.x;
        assert!(x_after_walk > 300);

        // 双击方向键 → 奔跑（官方 286 档）
        for down in [false, true, false, true] {
            input.begin_frame();
            if down {
                input.set(km.get(Command::Right), true);
            }
            idle_step(&mut a, &input, &km);
        }
        assert_eq!(a.motion, Motion::Run, "双击方向键应触发奔跑");
        let x_after_run = a.x;
        // 冲刺档 = 官方 286 ⇒ 243.1 px/s
        for _ in 0..40 {
            input.begin_frame();
            input.set(km.get(Command::Right), true);
            idle_step(&mut a, &input, &km);
        }
        let run_dx = a.x - x_after_run;
        // 40 步 @243.1 px/s ≈ 162px（±4px 取整误差）
        assert!((run_dx - 162).abs() <= 4, "run_dx={}", run_dx);

        // 跳跃
        input.begin_frame();
        input.set(km.get(Command::Right), true);
        input.set(km.get(Command::Jump), true);
        let ev = idle_step(&mut a, &input, &km);
        assert_eq!(ev, ActorEvent::None);
        assert_eq!(a.motion, Motion::Jump);
        assert!(!a.is_airborne(), "起跳准备阶段仍在地面");
        let mut launched = false;
        for _ in 0..10 {
            input.begin_frame();
            if idle_step(&mut a, &input, &km) == ActorEvent::Jumped {
                launched = true;
                break;
            }
        }
        assert!(launched);

        // 落地
        let mut landed = false;
        for _ in 0..600 {
            input.begin_frame();
            if idle_step(&mut a, &input, &km) == ActorEvent::Landed {
                landed = true;
                break;
            }
        }
        assert!(landed, "应当落地");
        assert_eq!(a.altitude(), 0.0);
        // 官方 jump.ani 帧 15（落地帧，50ms）仍在跳跃动画内播完，之后才回到站立
        assert_eq!(a.motion, Motion::Jump, "落地帧内应仍处于跳跃动作");
        assert_eq!(a.anim().frame(), 15, "落地时应播到官方落地帧（下标 15）");

        let mut stand_steps = 0;
        for _ in 0..30 {
            input.begin_frame();
            idle_step(&mut a, &input, &km);
            stand_steps += 1;
            if a.motion == Motion::Stand {
                break;
            }
        }
        assert_eq!(a.motion, Motion::Stand, "落地帧播完后应回到站立");
        assert!(
            stand_steps <= 8,
            "落地帧不应超过 8 步（官方 50ms ≈ 3~4 步）: {}",
            stand_steps
        );
    }

    #[test]
    fn 跳跃动画_两个保持帧都被逻辑放行() {
        let km = keymap();
        let mut a = Actor::new(ActorSpec::default(), ActorClips::swordman_official(), 0, 0);
        let mut input = InputState::new();

        input.begin_frame();
        input.set(km.get(Command::Jump), true);
        idle_step(&mut a, &input, &km);

        let mut seq: Vec<usize> = vec![a.anim().frame()];
        let mut landed_at = None;
        for i in 0..600 {
            input.begin_frame();
            if idle_step(&mut a, &input, &km) == ActorEvent::Landed {
                seq.push(a.anim().frame());
                landed_at = Some(i);
                break;
            }
            seq.push(a.anim().frame());
        }
        assert!(landed_at.is_some(), "应当落地");
        // 帧序必须单调不回退（官方 jump.ani 是一条单向时间线）
        assert!(
            seq.windows(2).all(|w| w[1] >= w[0]),
            "帧序不应回退: {:?}",
            seq
        );
        // 官方在**物理**到顶点时把动画直接跳到下落起始帧8（5EEE40配置，sub_4AECE0设置），
        // 顶点由速度符号决定，不靠动画上升段的固定总时长。
        assert!(seq.contains(&8), "顶点后必须进入下落段（帧 8）: {:?}", seq);
        // 普通分支直接设8，无须先设7再release_hold。
        assert!(seq.contains(&7) || seq.contains(&8));
        assert_eq!(seq.last().copied(), Some(15), "应播到官方落地帧 15");
        // 落地帧之后回到站立
        for _ in 0..10 {
            input.begin_frame();
            idle_step(&mut a, &input, &km);
            if a.motion == Motion::Stand {
                break;
            }
        }
        assert_eq!(a.motion, Motion::Stand, "落地帧播完后应回到站立");
    }

    #[test]
    fn 朝向_左右切换() {
        let km = keymap();
        let mut a = Actor::new(ActorSpec::default(), ActorClips::swordman_official(), 0, 0);
        let mut input = InputState::new();
        input.begin_frame();
        input.set(km.get(Command::Left), true);
        idle_step(&mut a, &input, &km);
        assert_eq!(a.facing, -1);
        assert!(a.x < 0);
    }
}
