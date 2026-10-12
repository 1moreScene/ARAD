//! DNF 同人客户端 —— 用**官方真实素材**驱动：官方地图 + 官方鬼剑士精灵。
//!
//! ```text
//! dnf-app                    # 打开窗口游玩（方向键移动，C 跳跃，Esc 退出）
//! dnf-app --info             # 打印官方资源装载报告（PVF/NPK/精灵/地图/键位取证）
//! dnf-app --headless         # 无头确定性回放，导出 PNG 到 out/
//! dnf-app --shot-after 90    # 跑 90 帧后实拍窗口客户区（验证窗口真有画面）
//! ```

mod active_status;
mod active_status_verify;
mod assets;
mod audio;
mod battle;
mod character;
mod damage;
mod debug_ui;
mod game;
mod grab_skill;
mod hud;
#[cfg(windows)]
mod keyboard;
mod localization;
mod notice;
mod performance;
mod protection_visual;
mod room;
mod script;
mod timing;
mod ui;
#[cfg(windows)]
mod win32;

use character::Outfit;
use dnf_core::keymap::KeyMap;
use dnf_core::{ActorEvent, MoveMode};
use dnf_render::framebuffer::{Framebuffer, HEIGHT, WIDTH};
use std::path::{Path, PathBuf};

struct Args {
    headless: bool,
    info: bool,
    gdi_selftest: bool,
    audio_selftest: bool,
    audio_out: Option<PathBuf>,
    volume: f32,
    mute: bool,
    window_replay: bool,
    panel: bool,
    ui_selftest: bool,
    status_selftest: bool,
    debug_selftest: bool,
    debug_panel: bool,
    performance_benchmark: bool,
    timing_selftest: bool,
    grab_selftest: bool,
    active_status_selftest: bool,
    ui_replay: bool,
    hud_replay: bool,
    feedback_replay: bool,
    show_keys: bool,
    scale: Option<f64>,
    frames: u64,
    script: script::ScriptKind,
    out: PathBuf,
    client: Option<PathBuf>,
    keymap: Option<PathBuf>,
    outfit: Option<PathBuf>,
    map: String,
    png_every: u64,
    trace: bool,
    shot_after: Option<u64>,
    shot_out: PathBuf,
}

impl Default for Args {
    fn default() -> Self {
        Args {
            headless: false,
            info: false,
            gdi_selftest: false,
            audio_selftest: false,
            audio_out: None,
            volume: 1.0,
            mute: false,
            window_replay: false,
            panel: false,
            ui_selftest: false,
            status_selftest: false,
            debug_selftest: false,
            debug_panel: false,
            performance_benchmark: false,
            timing_selftest: false,
            grab_selftest: false,
            active_status_selftest: false,
            ui_replay: false,
            hud_replay: false,
            feedback_replay: false,
            show_keys: false,
            scale: None,
            frames: 660,
            script: script::ScriptKind::All,
            out: PathBuf::from("out"),
            client: None,
            keymap: None,
            outfit: None,
            map: game::DEFAULT_MAP.to_string(),
            png_every: 0,
            trace: false,
            shot_after: None,
            shot_out: PathBuf::from("out/window.png"),
        }
    }
}

fn parse_args() -> Result<Args, String> {
    let mut a = Args::default();
    let mut it = std::env::args().skip(1);
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--headless" => a.headless = true,
            "--info" => a.info = true,
            "--gdi-selftest" => a.gdi_selftest = true,
            "--audio-selftest" => a.audio_selftest = true,
            "--audio-out" => {
                a.audio_out = Some(PathBuf::from(it.next().ok_or("--audio-out需要路径")?))
            }
            "--mute" => a.mute = true,
            "--volume" => {
                let value: f32 = it
                    .next()
                    .ok_or("--volume需要0..100")?
                    .parse()
                    .map_err(|_| "--volume非法")?;
                if !value.is_finite() || !(0.0..=100.0).contains(&value) {
                    return Err("--volume范围0..100".into());
                }
                a.volume = value / 100.0;
            }
            "--window-replay" => a.window_replay = true,
            "--show-keys" => a.show_keys = true,
            "--panel" => a.panel = true,
            "--ui-selftest" => a.ui_selftest = true,
            "--status-selftest" => a.status_selftest = true,
            "--debug-selftest" => a.debug_selftest = true,
            "--debug-panel" => a.debug_panel = true,
            "--performance-benchmark" => a.performance_benchmark = true,
            "--timing-selftest" => a.timing_selftest = true,
            "--grab-selftest" => a.grab_selftest = true,
            "--active-status-selftest" => a.active_status_selftest = true,
            "--ui-replay" => {
                a.ui_replay = true;
                a.window_replay = true;
            }
            "--hud-replay" => {
                a.hud_replay = true;
                a.window_replay = true;
            }
            "--feedback-replay" => {
                a.feedback_replay = true;
                a.window_replay = true;
            }
            "--scale" => {
                let scale: f64 = it
                    .next()
                    .ok_or("--scale需要倍率")?
                    .parse()
                    .map_err(|_| "--scale非法")?;
                if !scale.is_finite() || !(0.5..=4.0).contains(&scale) {
                    return Err("--scale范围0.5..4".into());
                }
                a.scale = Some(scale);
            }
            "--trace" => a.trace = true,
            "--frames" => {
                a.frames = it
                    .next()
                    .ok_or("--frames 需要参数")?
                    .parse()
                    .map_err(|_| "--frames 非法")?
            }
            "--png-every" => {
                a.png_every = it
                    .next()
                    .ok_or("--png-every 需要参数")?
                    .parse()
                    .map_err(|_| "--png-every 非法")?
            }
            "--script" => {
                let v = it.next().ok_or("--script 需要参数")?;
                a.script = script::ScriptKind::parse(&v).ok_or(format!("未知脚本：{}", v))?;
            }
            "--out" => a.out = PathBuf::from(it.next().ok_or("--out 需要参数")?),
            "--client" => a.client = Some(PathBuf::from(it.next().ok_or("--client 需要参数")?)),
            "--keymap" => a.keymap = Some(PathBuf::from(it.next().ok_or("--keymap 需要参数")?)),
            "--outfit" => a.outfit = Some(PathBuf::from(it.next().ok_or("--outfit 需要参数")?)),
            "--map" => a.map = it.next().ok_or("--map 需要参数")?,
            "--shot-after" => {
                a.shot_after = Some(
                    it.next()
                        .ok_or("--shot-after 需要参数")?
                        .parse()
                        .map_err(|_| "--shot-after 非法")?,
                )
            }
            "--shot-out" => a.shot_out = PathBuf::from(it.next().ok_or("--shot-out 需要参数")?),
            "--help" | "-h" => return Err("help".into()),
            other => return Err(format!("未知参数：{}", other)),
        }
    }
    Ok(a)
}

const HELP: &str = "\
DNF 同人客户端（官方真实素材：地图 + 鬼剑士精灵）

用法：dnf-app [选项]

  （无选项）        打开窗口游玩
  --window-replay   在真实窗口回放 --script 指定的输入（验证模式）
  --show-keys       启动时显示控制台内的按键记录（F7开关）
  --debug-panel     启动时打开自绘调试控制台（F6开关）
  --debug-selftest  验证人物/怪物调试与数量控制，并导出截图
  --active-status-selftest 验证17状态编号、叠加与效果截图
  --grab-selftest   验证噬魂之手抓取矩阵与技能截图
  --timing-selftest 验证60/120/300Hz驱动下世界时序一致
  --performance-benchmark 测量1/6/32怪物、霸体与面板的CPU渲染/缩放耗时
  --hud-replay      验证模式：回放技能拖放/快捷键施放/冷却/说明框
  --feedback-replay 验证模式：血蓝条缓动、悬停值及施放失败反馈
  --panel           启动时显示角色属性/装备窗口（M/I开关，F6调试设置）
  --ui-selftest     验证原版UI素材、调试编辑与伤害飘字，导出到 --out
  --status-selftest 验证霸体/无敌/叠加恢复/上挑，导出截图到 --out
  --ui-replay       在真实窗口回放调试编辑与关闭面板后的普攻
  --scale N         显示倍率；默认随桌面DPI（125%时800×600）；1为物理640×480
  --headless        无头确定性回放并输出 PNG
  --info            打印官方资源装载报告 + 键位取证表
  --gdi-selftest    GDI 呈现链自检（内存 DC 读回像素）
  --audio-selftest  播放官方重叠音效并检查声卡缓冲完成（约4秒）
  --audio-out PATH  无头回放同时导出确定性WAVE音轨
  --volume N        音效音量0..100（默认100）；--mute静音
  --shot-after N    窗口跑够 N 帧后实拍客户区到 --shot-out 并退出
  --shot-out PATH   实拍输出路径（默认 out/window.png）
  --frames N        无头模式逻辑步数（默认 660 = 11 秒）
  --script NAME     回放脚本：idle | walk | run | jump | turn | mashtap | airturn | attack | airattack | dashattack | upper | ghost | cancelghost | cancelupper | all
  --png-every N     每 N 步导出一张 PNG
  --trace           打印逐帧轨迹
  --out DIR         PNG 输出目录（默认 out）
  --client DIR      官方客户端目录（含 Script.pvf 与 ImagePacks2/）
  --keymap FILE     键位覆盖文件（默认 custom-dnf/data/keymap.txt）
  --outfit FILE     时装配置（默认 custom-dnf/data/outfit.txt）
  --map PATH        房间（默认 map/lorien/104.map）
  -h, --help        显示本帮助
";

fn normalize(p: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for comp in p.components() {
        match comp {
            std::path::Component::ParentDir => {
                out.pop();
            }
            std::path::Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    out
}

fn data_file(name: &str) -> PathBuf {
    normalize(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../data")
            .join(name),
    )
}

fn load_keymap(args: &Args) -> (KeyMap, Vec<String>) {
    let mut km = KeyMap::official_default();
    let mut notes = Vec::new();
    let path = args
        .keymap
        .clone()
        .unwrap_or_else(|| data_file("keymap.txt"));
    match std::fs::read_to_string(&path) {
        Ok(text) => {
            let warns = km.load_overrides(&text);
            notes.push(format!(
                "键位覆盖：{}（{} 条警告）",
                path.display(),
                warns.len()
            ));
            for w in warns {
                notes.push(format!("  键位警告: {}", w));
            }
        }
        Err(_) => notes.push(format!(
            "未找到键位文件 {}，用内建官方默认键位",
            path.display()
        )),
    }
    (km, notes)
}

/// 读时装配置；文件不存在时**写出一份默认配置**并说明。
fn load_outfit(args: &Args) -> (Outfit, Vec<String>) {
    let path = args
        .outfit
        .clone()
        .unwrap_or_else(|| data_file("outfit.txt"));
    match std::fs::read_to_string(&path) {
        Ok(text) => (
            Outfit::parse(&text),
            vec![format!("时装配置：{}", path.display())],
        ),
        Err(_) => {
            let o = Outfit::default_basic();
            let mut notes = vec![format!(
                "未找到时装配置 {}，用内建默认（官方基础设计 0000）",
                path.display()
            )];
            if std::fs::write(&path, o.to_text()).is_ok() {
                notes.push(format!(
                    "已写出默认配置：{}（改它即可换装）",
                    path.display()
                ));
            }
            (o, notes)
        }
    }
}

/// 打印官方移动/跳跃参数（全部来自官方代码常量，标注单位与出处）。
fn print_movement_info(spec: &dnf_core::ActorSpec) {
    let mt = dnf_core::MoveTuning::default();
    let jt = dnf_core::JumpTuning::default();
    let ratio = mt.speed_ratio(spec.move_speed);
    let (wx, _) = mt.velocity_for(spec.move_speed, MoveMode::Walk, (1, 0));
    let (_, wy_w) = mt.velocity_for(spec.move_speed, MoveMode::Walk, (0, 1));
    let (rx, _) = mt.velocity_for(spec.move_speed, MoveMode::Run, (1, 0));
    let (_, ry) = mt.velocity_for(spec.move_speed, MoveMode::Run, (0, 1));
    let (rx_d, ry_d) = mt.velocity_for(spec.move_speed, MoveMode::Run, (1, 1));
    let (wxd, wyd) = mt.velocity_for(spec.move_speed, MoveMode::Walk, (1, 1));
    let profile = dnf_core::JumpProfile::derive(&jt, spec.jump_power, spec.weight, spec.jump_speed);
    println!();
    println!("=== 移动 / 跳跃参数（全部为官方常量与公式）===");
    println!(
        "  速度基准: [move speed]={} ⇒ 倍率 {:.3}（官方 vmt37C 的千分比 × 0.001）",
        spec.move_speed, ratio
    );
    println!(
        "  走（官方 143/119 · 114/95）: 直 x={:.2} y={:.2} px/s；斜 x={:.2} y={:.2} px/s",
        wx, wy_w, wxd, wyd
    );
    println!(
        "  跑（官方 286/238 · 136/114）: 直 x={:.2} y={:.2} px/s；斜 x={:.2} y={:.2} px/s",
        rx, ry, rx_d, ry_d
    );
    println!("  （触发 = 连续左左/右右，按下到按下 ≤500ms；上下双击不启动奔跑；");
    println!("    反向退出奔跑；按住一键点反向键的命令候选与有效奔跑状态分别处理）");
    println!(
        "  跳跃: 准备100ms → 飞行（x继承地面档位，y参数整数/2；可改向）→ 落地50ms（锁定两轴）；落地回到自由态"
    );
    println!(
        "  跳跃: jump power={} weight={} jump speed={} ⇒ 初速 {:.1} px/s、重力 {:.0} px/s²、最高点 {:.1} px、滞空 {:.0} ms",
        spec.jump_power,
        spec.weight,
        spec.jump_speed,
        profile.v0,
        profile.gravity,
        profile.apex_height,
        profile.total_air_ms()
    );
    println!(
        "  （初速 = sub_495430(jump power, weight, jump speed)；重力 = 1500 × jump speed/100；两者都无标定值）"
    );
    println!(
        "  地面移动 1 px/s 的官方依据: pvp_ui.cpp:6319 位移式 + 同函数自由落体式定出单位（见 docs/02）"
    );
}

fn print_info(data: &game::GameData, km: &KeyMap, notes: &[String]) {
    println!("=== DNF 同人客户端 · 官方资源装载报告 ===");
    println!("客户端目录: {}", data.assets.client_dir.display());
    for line in &data.report {
        println!("  - {}", line);
    }
    print_movement_info(&data.spec);
    if let Some(audio) = &data.audio {
        for tag in dnf_data::audio::INITIAL_TAGS {
            match audio.probe(tag) {
                Ok(files) => println!("  音效 {tag}: {:?}", files),
                Err(error) => println!("  音效 {tag}: {error}"),
            }
        }
    }
    if let Some(combat) = &data.combat {
        for (i, attack) in combat.normal.iter().enumerate() {
            println!(
                "  普攻{}: {}帧 / {}ms；{} → {}",
                i + 1,
                attack.animation.frame_count(),
                attack.animation.total_ms(),
                attack.animation_path,
                attack.info_path
            );
        }
        println!(
            "  空中攻击: {}帧 / {}ms；{} → {}",
            combat.jump_attack.animation.frame_count(),
            combat.jump_attack.animation.total_ms(),
            combat.jump_attack.animation_path,
            combat.jump_attack.info_path
        );
    }
    match room::Room::load(&data.assets, &data.map_path) {
        Ok(r) => {
            println!();
            println!("=== 房间装载诊断（官方素材）===");
            for line in &r.report {
                println!("  - {}", line);
            }
        }
        Err(e) => println!("  ! 房间装载失败: {}", e),
    }
    println!();
    println!("=== 角色分层（官方动作 Stay 第 1 帧，顺序 = 官方 .equ [LAYER] 升序，身体层 0）===");
    for l in character::compose(
        &data.assets,
        &data.avatar,
        &data.visual,
        &data.outfit,
        &data.weapon,
        "Stay",
        0,
    ) {
        // 官方图集里有"什么都不画"的占位帧（1x1 全透明，放在画布 `(500,0)`），单独标出来
        let blank = l
            .image
            .frames
            .get(l.frame)
            .map(|f| !f.rgba.chunks_exact(4).any(|p| p[3] != 0))
            .unwrap_or(true);
        println!(
            "  - [LAYER]{:>5} {:<12} 帧{:>3} 偏移({:>5},{:>5}) {}{}",
            l.layer,
            l.label,
            l.frame,
            l.offset.0,
            l.offset.1,
            l.image_name,
            if blank {
                "  ← 官方空帧（本帧不画）"
            } else {
                ""
            }
        );
    }
    let (imgs, anis, tils) = data.assets.cache_stats();
    println!("  - 运行期缓存: img={} ani={} til={}", imgs, anis, tils);
    println!();
    println!("=== 官方默认键位表（取证等级见 docs/01_键位取证.md）===");
    for b in km.bindings() {
        // All base and shortcut slots were verified against 439AB0/415240.
        println!(
            "  {:<16} {:<5} DIK {:#04X}  {}",
            dnf_core::keymap::command_name(b.command),
            b.dik.to_string(),
            b.dik.code(),
            b.provenance.label()
        );
        if let dnf_core::keymap::Provenance::Confirmed(ev) = b.provenance {
            println!("      └─ 证据: {}", ev);
        }
    }
    for n in notes {
        println!("  {}", n);
    }
    println!("=== 鬼剑士技能命令模板（动作尚未全部实现）===");
    for (id, path, definition) in &data.command_templates {
        println!(
            "  skill{id} {path}: {:?} {:?} required={:?}",
            definition.trigger, definition.stages, definition.required_held
        );
    }
}

fn run_headless(data: &game::GameData, km: KeyMap, args: &Args) -> i32 {
    let mut fb = Framebuffer::new(WIDTH, HEIGHT);
    let out = args.out.clone();
    std::fs::create_dir_all(&out).ok();

    println!("=== 无头确定性回放（官方素材）===");
    println!(
        "房间: {}  脚本: {}  逻辑步: {}",
        data.map_path,
        args.script.name(),
        args.frames
    );
    println!("PNG 输出: {}", out.display());
    println!();

    let mut markers: std::collections::BTreeSet<u64> = std::collections::BTreeSet::new();
    markers.insert(1);
    let mut max_alt = 0.0f32;
    let mut jump_peak_step = 0u64;
    let mut min_x = i32::MAX;
    let mut max_x = i32::MIN;
    let mut motions = std::collections::BTreeMap::<String, u64>::new();
    let mut first_jump: Option<u64> = None;
    let mut first_land: Option<u64> = None;

    let mut game = data.build_game(km.clone()).expect("装载房间");
    game.panel.open = args.panel;
    game.show_keys = args.show_keys;
    game.debug_panel.open = args.debug_panel || args.show_keys;
    game.debug_panel.tab = if args.show_keys { 2 } else { 0 };
    let mut audio_bank = if args.audio_out.is_some() {
        match data
            .audio
            .as_ref()
            .ok_or_else(|| "没有音效索引".to_string())
            .and_then(|a| audio::Bank::load(a, 40))
        {
            Ok(bank) => Some(bank),
            Err(e) => {
                eprintln!("音轨准备失败: {e}");
                return 1;
            }
        }
    } else {
        None
    };
    let mut mixer = audio::Mixer::new(if args.mute { 0.0 } else { args.volume });
    let mut audio_samples = Vec::new();
    for step in 0..args.frames {
        game.begin_input();
        for dik in if args.feedback_replay {
            feedback_replay(&mut game)
        } else if args.hud_replay {
            hud_replay(&mut game)
        } else {
            script::sample(args.script, step, &game.keymap)
        } {
            game.set_key(dik, true);
        }
        let before = game.actor.motion.label();
        game.step();
        if let Some(bank) = &mut audio_bank {
            bank.set_loop(game.loop_sound, &mut mixer);
            bank.set_status_loops(game.status_loop_tags(), &mut mixer);
            for tag in &game.sound_requests {
                bank.play(tag, &mut mixer);
            }
            let mut block = [0i16; (audio::RATE as usize / 60) * 2];
            mixer.render(&mut block);
            audio_samples.extend_from_slice(&block);
        }
        let after = game.actor.motion.label();
        *motions.entry(after.to_string()).or_insert(0) += 1;
        if before != after {
            markers.insert(game.steps);
        }
        let alt = game.actor.altitude();
        if alt > max_alt {
            max_alt = alt;
            jump_peak_step = game.steps;
        }
        min_x = min_x.min(game.actor.x);
        max_x = max_x.max(game.actor.x);
        if first_jump.is_none() && game.last_event == ActorEvent::Jumped {
            first_jump = Some(game.steps);
        }
        if first_jump.is_some() && first_land.is_none() && game.last_event == ActorEvent::Landed {
            first_land = Some(game.steps);
        }
        if args.png_every > 0 && game.steps % args.png_every == 0 {
            markers.insert(game.steps);
        }
        if args.trace {
            println!("{}", game.trace_line());
        }
    }
    if jump_peak_step > 0 {
        markers.insert(jump_peak_step);
    }

    // 第 2 遍：同输入重跑，逐标记导出（同一步号 ⇒ 同一画面）
    let mut game = data.build_game(km).expect("装载房间");
    game.panel.open = args.panel;
    game.show_keys = args.show_keys;
    game.debug_panel.open = args.debug_panel || args.show_keys;
    game.debug_panel.tab = if args.show_keys { 2 } else { 0 };
    let mut exported = 0usize;
    for step in 0..args.frames {
        game.begin_input();
        for dik in if args.feedback_replay {
            feedback_replay(&mut game)
        } else if args.hud_replay {
            hud_replay(&mut game)
        } else {
            script::sample(args.script, step, &game.keymap)
        } {
            game.set_key(dik, true);
        }
        game.step();
        if markers.contains(&game.steps) {
            game.render(data, &mut fb);
            if dnf_render::png::write_file(&out.join(format!("frame_{:04}.png", game.steps)), &fb)
                .is_ok()
            {
                exported += 1;
            }
        }
    }
    game.render(data, &mut fb);
    dnf_render::png::write_file(&out.join("frame_last.png"), &fb).ok();
    exported += 1;

    println!();
    println!("=== 结果 ===");
    println!("导出 PNG: {} 张", exported);
    println!(
        "X 范围: {} .. {}（房间可走 {}..{}，宽 {}px，{} 条瓦片带）",
        min_x,
        max_x,
        game.room.walk_x.0,
        game.room.walk_x.1,
        game.room.room_width(),
        game.room.tiles.len()
    );
    println!("跳跃最高点: {:.1}px（第 {} 步）", max_alt, jump_peak_step);
    if let (Some(j), Some(l)) = (first_jump, first_land) {
        println!(
            "起跳 {:?} → 落地 {:?}：空中 {} 步（{:.0}ms）",
            j,
            l,
            l - j,
            (l - j) as f64 * 1000.0 / 60.0
        );
    }
    println!("动作分布(步数): {:?}", motions);
    if let Some(path) = &args.audio_out {
        if let Err(e) = audio::write_wave(path, &audio_samples) {
            eprintln!("音轨写出失败: {e}");
            return 1;
        }
        println!(
            "音轨: {}，{} samples，peak_voices={}",
            path.display(),
            audio_samples.len(),
            mixer.peak_voices
        );
    }
    0
}

#[cfg(windows)]
fn run_window_mode(data: &game::GameData, km: KeyMap, args: &Args) -> i32 {
    let mut game = match data.build_game(km) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("装载房间失败：{}", e);
            return 1;
        }
    };
    game.jt.logic_ms = dnf_core::clock::WORLD_MS;
    let mut fb = Framebuffer::new(WIDTH, HEIGHT);
    println!("=== 窗口模式（官方素材）===");
    game.panel.open = args.panel;
    game.show_keys = args.show_keys;
    game.debug_panel.open = args.debug_panel || args.show_keys;
    game.debug_panel.tab = if args.show_keys { 2 } else { 0 };
    let skillbar_path = data_file("skillbar.txt");
    if !args.window_replay {
        if let Ok(raw) = std::fs::read_to_string(&skillbar_path) {
            if let Err(error) = game.hud.load_bindings(&raw) {
                eprintln!("技能快捷栏设置：{error}，使用默认布局");
            }
        }
    }
    for line in &data.report {
        println!("  - {}", line);
    }
    println!(
        "  方向键移动（单击走 / 双击冲刺） X普攻/跑动突刺 Z上挑 技能栏默认A鬼斩/S上挑（均默认强制） C跳跃/空中X F5重置哥布林与MP Esc退出"
    );
    println!("  输入法已屏蔽；窗口不置顶、不抢焦点；失焦仅停止接收输入，游戏继续运行。");
    println!("  M / I 角色属性与装备，F6独立自绘控制台，F7控制台运行信息；点击数值或Tab选择，Enter应用，Esc关闭。");
    println!(
        "  F8测试霸体红色轮廓，F9测试无敌白色轮廓；再按解除，F5清除测试状态。白色为本地显示约定。"
    );
    println!(
        "  K或HUD技能按钮打开技能窗；拖放到六格快捷栏，悬停显示说明；布局自动保存，冷却随技能。"
    );
    let shot = args.shot_after.map(|n| (n, args.shot_out.clone()));
    let mut input_was_active = false;
    let mut input_needs_baseline = true;
    let mut sound = if args.mute || args.volume == 0.0 {
        None
    } else {
        match data
            .audio
            .as_ref()
            .ok_or("没有音效索引".to_string())
            .and_then(|a| audio::Engine::start(a, args.volume))
        {
            Ok(engine) => {
                println!(
                    "  音效已启用: {}个预解码声音 / 44100Hz立体声 / 32声部",
                    engine.cached_sounds()
                );
                Some(engine)
            }
            Err(error) => {
                eprintln!("音效启用失败: {error}");
                None
            }
        }
    };

    win32::run_window(
        "DNF 同人客户端 · 洛兰森林 · 鬼剑士",
        (WIDTH as i32, HEIGHT as i32),
        args.scale,
        shot,
        move |presenter, advance, render| {
            if input_was_active != presenter.input_active {
                game.clear_input();
                input_was_active = presenter.input_active;
                input_needs_baseline = true;
            }
            if advance {
                let sample = if !presenter.input_active {
                    None
                } else if args.window_replay {
                    Some(Vec::new())
                } else {
                    presenter.poll_keyboard()
                };
                let sample_valid = sample.is_some();
                let sample = sample.unwrap_or_default();
                if !sample_valid {
                    game.clear_input();
                    input_needs_baseline = true;
                }
                // Regaining focus must not turn a background-held C/X into a
                // newly pressed game command. Establish an edge-free baseline.
                if input_needs_baseline && presenter.input_active && sample_valid {
                    if !args.window_replay {
                        let pointer = win32::poll_pointer(presenter);
                        game.hud.pointer_baseline(pointer);
                        game.debug_panel.pointer_baseline(pointer.2);
                    }
                    game.begin_input();
                    for &(dik, down) in &sample {
                        game.set_key(dnf_core::keys::Dik::new(dik), down);
                    }
                    input_needs_baseline = false;
                }
                game.begin_input();
                for (dik, down) in sample {
                    game.set_key(dnf_core::keys::Dik::new(dik), down);
                }
                if args.window_replay && presenter.input_active {
                    for dik in if args.feedback_replay {
                        feedback_replay(&mut game)
                    } else if args.hud_replay {
                        hud_replay(&mut game)
                    } else if args.ui_replay {
                        ui_replay(&mut game)
                    } else {
                        script::sample(args.script, game.replay_frame(), &game.keymap)
                    } {
                        game.set_key(dik, true);
                    }
                } else if presenter.input_active {
                    game.panel.pointer = win32::poll_pointer(presenter);
                } else {
                    game.panel.pointer = (-1, -1, false);
                }
                game.step();
                if game.hud.dirty && !args.window_replay {
                    match std::fs::write(&skillbar_path, game.hud.bindings_text()) {
                        Ok(()) => game.hud.message = "快捷栏已保存".into(),
                        Err(error) => {
                            game.hud.message = "快捷栏保存失败".into();
                            eprintln!("快捷栏保存：{error}");
                        }
                    }
                    game.hud.dirty = false;
                }
                if let Some(sound) = &mut sound {
                    sound.set_loop(game.loop_sound);
                    sound.set_status_loops(game.status_loop_tags());
                    sound.play(&game.sound_requests);
                }
                if args.trace {
                    println!(
                        "{} input_active={}",
                        game.trace_line(),
                        presenter.input_active
                    );
                }
            }
            if render {
                game.fps = presenter.fps;
                game.performance = presenter.performance;
                let start = std::time::Instant::now();
                game.render(data, &mut fb);
                presenter.last_render_ms = performance::ms(start);
                presenter.present(&fb);
            }
            !game.exit_requested
        },
    )
}

#[cfg(not(windows))]
fn run_window_mode(_data: &game::GameData, _km: KeyMap, _args: &Args) -> i32 {
    eprintln!("窗口模式仅支持 Windows；请使用 --headless");
    1
}

fn feedback_replay(game: &mut game::Game) -> Vec<dnf_core::keys::Dik> {
    use dnf_core::keys::Dik;
    let t = game.replay_frame();
    game.panel.pointer = match t {
        15..=24 => (22, 440, false),
        28..=34 => (617, 440, false),
        _ => (-1, -1, false),
    };
    match t {
        0 => game.apply_debug(ui::Command::Set(2, 0)),
        25 => {
            game.apply_debug(ui::Command::Set(2, 150));
            game.apply_debug(ui::Command::Set(0, 75));
        }
        // These synthetic costs exercise the same public admission as future HP/item skills.
        129 => {
            game.apply_debug(ui::Command::Set(0, 10));
            game.actor.upper_skill.hp = 10;
        }
        133 => {
            game.apply_debug(ui::Command::Set(0, 11));
            game.actor.upper_skill.hp = 0;
            game.actor.upper_skill.items = vec![(3037, 1)];
        }
        137 => {
            game.actor.inventory.insert(3037, 1);
        }
        _ => {}
    }
    match t {
        2 | 4 | 26 | 110 => vec![Dik::A],
        130 | 138 => vec![Dik::Z],
        134 => vec![Dik::S],
        _ => Vec::new(),
    }
}

fn hud_replay(game: &mut game::Game) -> Vec<dnf_core::keys::Dik> {
    use dnf_core::keys::Dik;
    let t = game.replay_frame();
    game.panel.pointer = match t {
        3 => (226, 83, true),
        4 => (330, 265, true),
        5 => (457, 461, true),
        6 => (457, 461, false),
        30..=120 => (457, 461, false),
        _ => (-1, -1, false),
    };
    if t == 14 {
        let v = game.debug_values();
        game.apply_debug(ui::Command::Set(0, v[1] / 2));
    }
    match t {
        0 | 8 => vec![Dik::K],
        10 => vec![Dik::A],
        20 => vec![Dik::D],
        _ => Vec::new(),
    }
}

fn count_non_black(fb: &Framebuffer) -> usize {
    fb.data
        .chunks_exact(4)
        .filter(|p| p[0] != 0 || p[1] != 0 || p[2] != 0)
        .count()
}

#[cfg(windows)]
fn audio_selftest(data: &game::GameData, args: &Args) -> Result<(), String> {
    use std::sync::atomic::Ordering;
    let audio = data.audio.as_ref().ok_or("没有音效索引")?;
    let mut engine = audio::Engine::start(audio, 0.7)?;
    println!("音频设备已打开，缓存{}个声音", engine.cached_sounds());
    let mut seen = std::collections::BTreeSet::new();
    for kind in [script::ScriptKind::Attack, script::ScriptKind::AirAttack] {
        let mut game = data.build_game(KeyMap::official_default())?;
        for step in 0..120 {
            game.begin_input();
            for dik in script::sample(kind, step, &game.keymap) {
                game.set_key(dik, true);
            }
            game.step();
            engine.play(&game.sound_requests);
            seen.extend(game.sound_requests.iter().copied());
            if !game.sound_requests.is_empty() {
                println!("{}", game.trace_line());
            }
            std::thread::sleep(std::time::Duration::from_millis(17));
        }
    }
    for tag in [
        "R_SM_ATK",
        "R_SM_JUMP_ATK",
        "SM_JUMP",
        "JUMP_START",
        "JUMP_END",
        "R_MINERALSWDA",
        "MINERALSWDB",
    ] {
        if !seen.contains(tag) {
            return Err(format!("回放未触发音效 {tag}"));
        }
    }
    if engine.stats.audible_completed.load(Ordering::Relaxed) == 0
        || engine.stats.peak_voices.load(Ordering::Relaxed) < 2
    {
        return Err("没有完成的非静音设备缓冲或未发生重叠混音".into());
    }
    engine.play(&["R_SM_ATK", "R_MINERALSWDA"]);
    std::thread::sleep(std::time::Duration::from_millis(60));
    engine.set_active(false);
    std::thread::sleep(std::time::Duration::from_millis(30));
    let requests = engine.stats.requests.load(Ordering::Relaxed);
    let audible = engine.stats.audible_completed.load(Ordering::Relaxed);
    engine.play(&["SM_JUMP", "JUMP_START"]);
    std::thread::sleep(std::time::Duration::from_millis(40));
    if engine.stats.requests.load(Ordering::Relaxed) != requests
        || engine.stats.audible_completed.load(Ordering::Relaxed) != audible
    {
        return Err("失焦停止后仍有请求或非静音缓冲播放".into());
    }
    engine.set_active(true);
    engine.play(&["SM_JUMP", "JUMP_START"]);
    // Observe a completed audible device block instead of assuming a fixed queue latency.
    let resume_deadline = std::time::Instant::now() + std::time::Duration::from_secs(1);
    while engine.stats.audible_completed.load(Ordering::Relaxed) <= audible
        && std::time::Instant::now() < resume_deadline
    {
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    if engine.stats.requests.load(Ordering::Relaxed) != requests + 2
        || engine.stats.audible_completed.load(Ordering::Relaxed) <= audible
    {
        return Err("恢复焦点后的音效未恢复播放".into());
    }
    println!("audio focus_stop_resume=passed");
    if let Some(path) = &args.audio_out {
        let mut bank = audio::Bank::load(audio, 40)?;
        let mut mixer = audio::Mixer::new(0.7);
        bank.play("R_SM_ATK", &mut mixer);
        bank.play("R_MINERALSWDA", &mut mixer);
        let mut samples = vec![0; audio::RATE as usize * 2];
        mixer.render(&mut samples);
        audio::write_wave(path, &samples)?;
    }
    Ok(())
}

fn ui_replay(game: &mut game::Game) -> Vec<dnf_core::keys::Dik> {
    use dnf_core::keys::Dik;
    let t = game.replay_frame();
    game.panel.pointer = match t {
        4 => (170, 200, true),
        18 => (210, 120, true),
        20 => (170, 144, true),
        _ => (-1, -1, false),
    };
    match t {
        0 => vec![Dik::M],
        2 | 34 => vec![Dik::F6],
        6 => vec![Dik::KEY_1],
        8 => vec![Dik::KEY_2],
        10 => vec![Dik::KEY_3],
        12 | 30 => vec![Dik::RETURN],
        16 | 36 => vec![Dik::X],
        22 => vec![Dik::KEY_1],
        24 => vec![Dik::KEY_2],
        26 | 28 => vec![Dik::KEY_0],
        _ => vec![],
    }
}
fn ui_selftest(data: &game::GameData, km: KeyMap, out: &Path) -> Result<(), String> {
    std::fs::create_dir_all(out).map_err(|e| e.to_string())?;
    let mut game = data.build_game(km)?;
    let mut fb = Framebuffer::default();
    let start = (game.actor.x, game.actor.y);
    let frames = [
        (1, "character-equipment"),
        (3, "debug-basic"),
        (13, "damage-edited"),
        (31, "movement-edited"),
        (50, "damage-hit"),
    ];
    for _ in 0..85 {
        game.begin_input();
        for k in ui_replay(&mut game) {
            game.set_key(k, true);
        }
        game.step();
        if game.steps < 34 && ((game.actor.x, game.actor.y) != start || game.goblin.hits != 0) {
            return Err("UI input leaked into battle".into());
        }
        if game.steps == 13 && game.debug_values()[4] != 123 {
            return Err("damage edit failed".into());
        }
        if game.steps == 31 && game.actor.spec.move_speed != 1200.0 {
            return Err("move speed edit failed".into());
        }
        for &(step, name) in &frames {
            if game.steps == step {
                game.render(data, &mut fb);
                dnf_render::png::write_file(&out.join(format!("{name}.png")), &fb)
                    .map_err(|e| e.to_string())?;
            }
        }
    }
    // Attack1.atk has -15%: int(123 * 85 / 100) = 104.
    if game.goblin.hits != 1 || game.goblin.hp != 196 {
        return Err(format!(
            "edited damage not applied: HP={} hits={}",
            game.goblin.hp, game.goblin.hits
        ));
    }
    game.apply_debug(ui::Command::Set(3, 10));
    game.apply_debug(ui::Command::Set(2, 999));
    if game.actor.mp != 10 {
        return Err("MP clamp failed".into());
    }
    game.apply_debug(ui::Command::Set(18, 1234));
    if game.goblin.hp != 1234 || game.goblin.max_hp != 1234 {
        return Err("target HP edit failed".into());
    }
    game.apply_debug(ui::Command::Reset);
    if game.actor.spec.move_speed != data.spec.move_speed
        || game.actor.mp != 150
        || game.goblin.hp != 300
        || game.debug_values()[4] != 18
    {
        return Err("reset failed".into());
    }
    fb = Framebuffer::new(780, 480);
    fb.clear(dnf_render::framebuffer::rgba(30, 31, 36, 255));
    for (column, name) in [
        "Normal",
        "Critical",
        "Recovery",
        "MP",
        "Player",
        "Additional",
    ]
    .iter()
    .enumerate()
    {
        dnf_render::font::draw_text(
            &mut fb,
            20 + column as i32 * 125,
            12,
            name,
            dnf_render::framebuffer::Color::WHITE,
        );
        for (row, age) in [0.0, 50.0, 100.0, 300.0, 400.0, 450.0].iter().enumerate() {
            damage::DamagePopup {
                damage: 123,
                point: [65.0 + column as f32 * 125.0, 100.0 + row as f32 * 60.0, 0.0],
                age_ms: *age,
                style: column as i32,
            }
            .draw(&data.ui.numbers, &mut fb, 0);
        }
    }
    dnf_render::png::write_file(&out.join("damage-styles-timeline.png"), &fb)
        .map_err(|e| e.to_string())?;
    println!("UI自检通过: 原版面板/插画/武器图标/Combo数字；鼠标+键盘编辑123攻击与1200移速；面板阻断X；原版首段85%倍率命中HP196；MP限值、目标HP与默认恢复。截图: {}",out.display());
    Ok(())
}

fn main() {
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            if e == "help" {
                println!("{}", HELP);
                return;
            }
            eprintln!("参数错误：{}\n\n{}", e, HELP);
            std::process::exit(2);
        }
    };

    let client = args.client.clone().or_else(dnf_data::locate_client_dir);
    let Some(client) = client else {
        eprintln!(
            "找不到官方客户端目录（需含 Script.pvf 与 ImagePacks2/）。\n用 --client 指定，或设环境变量 DNF_CLIENT_DIR。"
        );
        std::process::exit(3);
    };

    let (km, mut notes) = load_keymap(&args);
    let (outfit, outfit_notes) = load_outfit(&args);
    notes.extend(outfit_notes);

    let data = match game::GameData::load(&client, &args.map, outfit) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("装载官方资源失败：{}", e);
            std::process::exit(3);
        }
    };

    if args.info {
        print_info(&data, &km, &notes);
        return;
    }
    if args.performance_benchmark {
        if let Err(error) = performance::benchmark(&data, &args.out) {
            eprintln!("性能基准失败: {error}");
            std::process::exit(1);
        }
        return;
    }
    if args.active_status_selftest {
        if let Err(error) = active_status_verify::verify(&data, &args.out) {
            eprintln!("ActiveStatus selftest failed: {error}");
            std::process::exit(1);
        }
        println!("ActiveStatus selftest passed");
        return;
    }
    if args.grab_selftest {
        if let Err(error) = grab_skill::verify(&data, &args.out) {
            eprintln!("grab selftest failed: {error}");
            std::process::exit(1);
        }
        println!("grab skill31 selftest passed");
        return;
    }
    if args.timing_selftest {
        if let Err(error) = timing::verify(&data, &args.out) {
            eprintln!("时序自检失败: {error}");
            std::process::exit(1);
        }
        println!("时序自检通过：300Hz世界，60/120/300Hz调度的移动/动画/技能/碰撞与音效一致");
        return;
    }
    if args.status_selftest {
        if let Err(error) = protection_visual::selftest(&data, &args.out) {
            eprintln!("状态自检失败: {error}");
            std::process::exit(1);
        }
        println!("状态自检通过：霸体伤害、无敌拒绝命中、叠加恢复、上挑霸体和轮廓截图");
        return;
    }
    if args.debug_selftest {
        if let Err(error) = debug_ui::selftest(&data, &args.out) {
            eprintln!("调试控制台自检失败: {error}");
            std::process::exit(1);
        }
        println!("调试控制台自检通过：人物属性/状态、怪物单只/全部设置、0..32数量、输入隔离与无场景DEBUG文字");
        return;
    }
    if args.ui_selftest {
        if let Err(e) = ui_selftest(&data, km, &args.out) {
            eprintln!("UI自检失败: {e}");
            std::process::exit(1);
        }
        return;
    }
    if args.audio_selftest {
        #[cfg(windows)]
        match audio_selftest(&data, &args) {
            Ok(()) => {
                println!("音频自检通过：动作音效已提交并完成设备播放，重叠混音通过");
                return;
            }
            Err(e) => {
                eprintln!("音频自检失败: {e}");
                std::process::exit(1);
            }
        }
        #[cfg(not(windows))]
        {
            eprintln!("设备播放自检仅支持Windows；无头模式可用--audio-out");
            std::process::exit(1);
        }
    }

    if args.gdi_selftest {
        let mut game = data.build_game(km).expect("装载房间");
        let mut fb = Framebuffer::new(WIDTH, HEIGHT);
        for step in 0..120 {
            game.begin_input();
            for dik in script::sample(script::ScriptKind::All, step, &game.keymap) {
                game.set_key(dik, true);
            }
            game.step();
        }
        game.render(&data, &mut fb);
        println!("=== GDI 呈现链自检（内存 DC + DIB 段）===");
        println!(
            "帧缓冲: {}x{}，非黑像素 {}",
            fb.width,
            fb.height,
            count_non_black(&fb)
        );
        #[cfg(windows)]
        match win32::gdi_blit_self_test(&fb) {
            Ok((total, non_black, corner)) => {
                println!("StretchDIBits 目标位图: {} 像素，非黑 {}", total, non_black);
                println!("左上角像素 BGRA = {:?}", corner);
                if non_black == 0 {
                    eprintln!("失败：GDI 目标位图全黑");
                    std::process::exit(1);
                }
                println!("结果：GDI 呈现链 OK");
                return;
            }
            Err(e) => {
                eprintln!("失败：{}", e);
                std::process::exit(1);
            }
        }
        #[cfg(not(windows))]
        return;
    }

    for n in &notes {
        println!("  {}", n);
    }

    let code = if args.headless {
        run_headless(&data, km, &args)
    } else {
        run_window_mode(&data, km, &args)
    };
    std::process::exit(code);
}
