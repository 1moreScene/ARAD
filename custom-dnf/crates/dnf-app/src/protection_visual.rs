//! Software equivalent of colored offset sprite copies (499C70 / 4917A0).
use dnf_core::protection::Protection;
use dnf_render::framebuffer::{rgba, Color};

const NEIGHBORS: [(i32, i32); 8] = [
    (-2, -2),
    (-2, 0),
    (-2, 2),
    (0, -2),
    (0, 2),
    (2, -2),
    (2, 0),
    (2, 2),
];

pub fn outline(protection: Protection) -> Option<(Color, &'static [(i32, i32)])> {
    match protection {
        Protection::Normal => None,
        // Local continuous contour using the verified red ABGR FF0000FF/radius2.
        // 499C70 also supports eight fixed copies; the CNSuperArmor appendage
        // selects its random-offset branch. Do not generalize that branch to all
        // skill armor, or turn it into a periodically flashing contour.
        Protection::SuperArmor => Some((rgba(255, 0, 0, 150), &NEIGHBORS)),
        // White is a local visual convention. No universal 40JP invincibility
        // outline/color has been proved; gameplay protection does not depend on it.
        Protection::Invincible => Some((rgba(255, 255, 255, 150), &NEIGHBORS)),
    }
}

pub fn selftest(data: &crate::game::GameData, out: &std::path::Path) -> Result<(), String> {
    use dnf_core::{keys::Dik, protection::DamageOutcome, KeyMap};
    use dnf_render::{framebuffer::Framebuffer, png};
    std::fs::create_dir_all(out).map_err(|e| e.to_string())?;
    let mut game = data.build_game(KeyMap::official_default())?;
    game.actor.teleport(320, 329);
    game.camera_x = 0;
    let shot = |game: &crate::game::Game, name: &str| -> Result<(), String> {
        let mut fb = Framebuffer::default();
        game.render(data, &mut fb);
        png::write_file(&out.join(format!("{name}.png")), &fb).map_err(|e| e.to_string())
    };
    let key = |game: &mut crate::game::Game, key: Dik| {
        game.begin_input();
        game.set_key(key, true);
        game.step();
        game.begin_input();
        game.set_key(key, false);
        game.step();
    };
    let check = |ok: bool, message: &str| if ok { Ok(()) } else { Err(message.to_string()) };
    shot(&game, "01_normal")?;
    key(&mut game, Dik::F8);
    check(
        game.actor.protection.effective() == Protection::SuperArmor,
        "F8 armor",
    )?;
    shot(&game, "02_superarmor")?;
    // Same pose/camera/background remains identical over repeated renders and
    // arbitrary simulation step counters: the contour has no temporal flashing.
    let mut first = Framebuffer::default();
    game.draw_actor(data, &mut first);
    let step = game.steps;
    for tick in 0..60 {
        game.steps = tick;
        let mut repeated = Framebuffer::default();
        game.draw_actor(data, &mut repeated);
        check(
            first.data == repeated.data,
            "armor outline must not flicker",
        )?;
    }
    game.steps = step;
    let hp = game.actor.hp;
    let motion = game.actor.motion;
    check(
        game.actor.receive_damage(7) == DamageOutcome::Armored,
        "armor damage outcome",
    )?;
    check(
        game.actor.hp == hp - 7 && game.actor.motion == motion,
        "armor HP/motion",
    )?;
    key(&mut game, Dik::F9);
    check(
        game.actor.protection.effective() == Protection::Invincible,
        "invincible priority",
    )?;
    shot(&game, "03_invincible")?;
    let hp = game.actor.hp;
    check(
        game.actor.receive_damage(7) == DamageOutcome::Rejected && game.actor.hp == hp,
        "invincible must reject damage",
    )?;
    key(&mut game, Dik::F9);
    check(
        game.actor.protection.effective() == Protection::SuperArmor,
        "underlying armor restored",
    )?;
    shot(&game, "04_restored_armor")?;
    game.actor
        .protection
        .grant(77, Protection::Invincible, Some(50.0));
    for _ in 0..3 {
        game.begin_input();
        game.step();
    }
    check(
        game.actor.protection.effective() == Protection::SuperArmor,
        "timed invincible expired",
    )?;
    key(&mut game, Dik::F5);
    check(
        game.actor.protection.effective() == Protection::Normal,
        "F5 reset",
    )?;
    shot(&game, "05_normal_again")?;
    let frames = &data
        .visual
        .motions
        .iter()
        .find(|(n, _)| n == "Attack3")
        .ok_or("missing Attack3")?
        .1;
    check(
        frames
            .iter()
            .all(|f| f.damage_type == dnf_data::ani::DamageType::Normal),
        "official Attack3 changed",
    )?;
    key(&mut game, Dik::Z);
    check(
        game.actor.motion == dnf_core::Motion::UpperSlash
            && game.actor.protection.effective() == Protection::SuperArmor,
        "upper skill armor",
    )?;
    shot(&game, "06_upper_armor")?;
    while game.actor.anim().frame() < dnf_core::actor::UPPER_ARMOR_END_FRAME {
        game.begin_input();
        game.step();
    }
    check(
        game.actor.motion == dnf_core::Motion::UpperSlash
            && game.actor.protection.effective() == Protection::Normal,
        "upper armor must end at zero-based frame2 before action ends",
    )?;
    shot(&game, "07_upper_no_armor")?;
    for _ in 0..90 {
        game.begin_input();
        game.step();
    }
    check(
        game.actor.protection.effective() == Protection::Normal,
        "upper armor must end with action",
    )?;
    let ani = data
        .assets
        .ani("monster/spider/spiderbossani/burrowstart.ani")
        .ok_or("missing official UNBREAKABLE animation")?;
    check(
        ani.frames.iter().any(|f| {
            dnf_data::ani::DamageType::from_token(&f.damage_type)
                == dnf_data::ani::DamageType::Unbreakable
        }),
        "official invincible token",
    )?;
    game.goblin
        .protection
        .grant(9, Protection::Invincible, None);
    check(
        game.goblin.hurt_boxes(&data.goblin).is_empty(),
        "invincible hurt boxes",
    )?;
    let hp = game.goblin.hp;
    let hits = game.goblin.hits;
    let accepted = game.goblin.hit(
        &data.goblin,
        &dnf_data::combat::AttackInfo::default(),
        &data.weapon_feedback,
        crate::battle::Hit {
            damage: 10,
            facing: 1,
            upper: true,
            point: [280.0, 329.0, 30.0],
        },
    );
    check(
        !accepted
            && game.goblin.hp == hp
            && game.goblin.hits == hits
            && game.goblin.stop.remaining_ms == 0.0,
        "invincible hit side effects",
    )?;
    std::fs::write(out.join("status_report.txt"),
        "PASS: armor damage / invincibility rejection / independent source removal / timer expiry / reset / official UpperSlash first-two-frame gate / real PVF UNBREAKABLE / hurt boxes and hit feedback gate / identical contour for 60 same-pose renders\nArmor: verified red ABGR FF0000FF/radius2; continuous eight-direction contour at alpha150 is local visual policy\nUpperSlash: only frame indices0/1 protected; frame2 and recovery unprotected, duration follows attack speed\nInvincible white outline: local convention, official universal color unconfirmed\n")
        .map_err(|e| e.to_string())?;
    Ok(())
}
