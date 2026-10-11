//! 端到端校验：直接用**本机官方客户端**的 `Script.pvf` 跑一遍解析链，并把
//! 解析结果与人工从官方文件读出的真值逐项比对。
//!
//! 找不到客户端目录时自动跳过（不失败），因此 CI 上也不会红。

use dnf_data::{ani, chr, locate_client_dir, PvfReader};

fn open() -> Option<PvfReader> {
    let dir = locate_client_dir()?;
    match PvfReader::open(&dir.join("Script.pvf")) {
        Ok(r) => Some(r),
        // 官方客户端（ARAD.exe）正开着时会独占 Script.pvf ⇒ 跳过，不判失败
        Err(e) if dnf_data::client_busy(&e.to_string()) => {
            eprintln!("跳过：官方客户端正在运行，Script.pvf 被占用");
            None
        }
        Err(e) => panic!("Script.pvf 应能打开且目录 CRC 通过: {}", e),
    }
}

#[test]
fn 官方pvf目录自校验() {
    let Some(r) = open() else {
        eprintln!("跳过：未找到官方客户端目录");
        return;
    };
    assert!(r.entry_count > 20_000, "条目数异常: {}", r.entry_count);
    assert!(r.contains("character/swordman/swordman.chr"));
    assert!(r.contains("skill/rdarkeyindex.dat"));
}

#[test]
fn 官方鬼剑士chr真值() {
    let Some(r) = open() else { return };
    let text = r
        .read_text("character/swordman/swordman.chr")
        .expect("读 .chr");
    let c = chr::parse(&text);
    assert_eq!(c.job, "swordman");
    assert_eq!(c.width, (40, 10));
    assert_eq!(c.move_speed(), 850.0);
    assert_eq!(c.jump_power(), 430.0);
    assert_eq!(c.jump_speed(), 95.0);
    assert_eq!(c.attr("hit recovery"), Some(600.0));
    // 转职增量不得污染基础属性（官方 `[growtype N]` 会重复顶层标签）
    assert!(c.growtypes.contains_key(&1));
    // 动作映射
    assert_eq!(c.motion("move motion"), Some("Animation/Move.ani"));
    assert_eq!(c.motion("dash motion"), Some("Animation/Dash.ani"));
    assert_eq!(c.motion("jump motion"), Some("Animation/Jump.ani"));
    assert_eq!(c.motion("waiting motion"), Some("Animation/Stay.ani"));
}

#[test]
fn 官方鬼剑士动画真值() {
    let Some(r) = open() else { return };

    let stay = ani::parse(
        &r.read_text("character/swordman/animation/stay.ani")
            .unwrap(),
    );
    assert_eq!(stay.delays(), vec![120; 6]);
    assert!(stay.looping);

    let mv = ani::parse(
        &r.read_text("character/swordman/animation/move.ani")
            .unwrap(),
    );
    assert_eq!(mv.delays(), vec![100; 8]);
    assert!(mv.looping);
    assert_eq!(mv.cycle_ms(), 800);
    assert_eq!(mv.frames[0].image, "Character/Swordman/sm_body.img");
    assert_eq!(mv.frames[0].image_index, 180);
    assert_eq!(mv.frames[7].image_index, 187);

    let dash = ani::parse(
        &r.read_text("character/swordman/animation/dash.ani")
            .unwrap(),
    );
    assert_eq!(dash.delays(), vec![100; 8]);
    assert_eq!(dash.frames[0].image_index, 105);
    assert_eq!(dash.frames[7].image_index, 112);

    let jump = ani::parse(
        &r.read_text("character/swordman/animation/jump.ani")
            .unwrap(),
    );
    assert_eq!(
        jump.delays(),
        vec![100, 100, 80, 80, 80, 80, 80, 10000, 100, 80, 80, 80, 80, 80, 10000, 50]
    );
    assert!(!jump.looping);
    // 官方数据：上升段（帧 0..6）600ms、下降段（帧 8..13）500ms、落地帧 50ms
    assert_eq!(
        jump.delays()[0..7].iter().sum::<u32>(),
        600,
        "上升段应为 600ms"
    );
    assert_eq!(
        jump.delays()[8..14].iter().sum::<u32>(),
        500,
        "下降段应为 500ms"
    );
    assert_eq!(jump.delays()[15], 50, "落地帧应为 50ms");
    assert_eq!(jump.frames[15].image_index, 132);
}

#[test]
fn 官方命令词表真值() {
    let Some(r) = open() else { return };
    let idx = r
        .read_text("skill/rdarkeyindex.dat")
        .expect("读 RDARKeyIndex.dat");
    // 官方原文：0 (UP) 1 (DOWN) 2 (LEFT) 3 (RIGHT) 4 (SKILL) 5 (ATTACK) 6 (JUMP) 7 (CREATURE) 8 (BUFF)
    assert!(idx.contains("0 (UP)"), "{}", idx);
    assert!(idx.contains("3 (RIGHT)"), "{}", idx);
    assert!(idx.contains("5 (ATTACK)"), "{}", idx);
    assert!(idx.contains("6 (JUMP)"), "{}", idx);
    assert!(idx.contains("8 (BUFF)"), "{}", idx);
}

#[test]
fn 官方地图数据在包内() {
    let Some(r) = open() else { return };
    let names = r.names_with_prefix("map/");
    assert!(names.len() > 100, "map/ 条目数异常: {}", names.len());
    let tile = names.iter().any(|n| n.ends_with(".til"));
    assert!(tile, "应存在官方 .til 瓦片文件");
}
