use crate::game::GameData;
use dnf_core::{keys::Dik, protection::Protection, KeyMap, Motion};
#[derive(Debug, PartialEq)]
pub(crate) struct FrequencyPose {
    x: i32,
    y: i32,
    z: u32,
    motion: Motion,
    frame: usize,
    hp: i32,
    mp: i32,
    upper_cd: u64,
    ghost_cd: u64,
    state: Protection,
    pub(crate) targets: Vec<(u32, u32, i32, u64, crate::battle::Reaction, usize, u64)>,
}
pub(crate) fn frequency_replay(
    data: &GameData,
    hz: u32,
    combat: bool,
) -> (Vec<FrequencyPose>, Vec<&'static str>) {
    let mut g = data.build_game(KeyMap::official_default()).unwrap();
    g.actor.teleport(300, 329);
    g.goblin.reset(350, 329, &data.goblin);
    g.set_monster_count(if combat { 3 } else { 1 });
    for i in 0..g.monster_count() {
        g.monster_mut(i).unwrap().reset(350, 329, &data.goblin);
    }
    g.jt.logic_ms = 1000.0 / hz as f64;
    let mut poses = Vec::new();
    let mut sounds = Vec::new();
    for tick in 0..hz * 4 {
        let t = tick as u64 * 1000 / hz as u64;
        g.begin_input();
        if combat {
            if (0..50).contains(&t) || (300..350).contains(&t) || (550..600).contains(&t) {
                g.set_key(Dik::X, true);
            }
            if (650..700).contains(&t) || (3300..3350).contains(&t) {
                g.set_key(Dik::Z, true);
            }
            if (1400..1450).contains(&t) {
                g.set_key(Dik::A, true);
            }
        } else {
            if t < 50 || (100..550).contains(&t) {
                g.set_key(Dik::RIGHT, true);
            }
            if (300..350).contains(&t) {
                g.set_key(Dik::C, true);
            }
            if (550..1200).contains(&t) {
                g.set_key(Dik::LEFT, true);
            }
            if (800..850).contains(&t) {
                g.set_key(Dik::X, true);
            }
            if (1300..1350).contains(&t) {
                g.set_key(Dik::Z, true);
            }
            if (2200..2250).contains(&t) {
                g.set_key(Dik::A, true);
            }
        }
        if t == 2500 {
            g.actor
                .protection
                .grant(77, Protection::Invincible, Some(1000.0));
        }
        g.step();
        sounds.extend_from_slice(&g.sound_requests);
        if (tick + 1) % (hz / 20) == 0 {
            poses.push(FrequencyPose {
                x: g.actor.x,
                y: g.actor.y,
                z: g.actor.altitude().to_bits(),
                motion: g.actor.motion,
                frame: g.actor.anim().frame(),
                hp: g.actor.hp,
                mp: g.actor.mp,
                upper_cd: g.actor.upper_cooldown_ms().to_bits(),
                ghost_cd: g.actor.ghost_cooldown_ms().to_bits(),
                state: g.actor.protection.effective(),
                targets: g
                    .monsters()
                    .map(|m| {
                        (
                            m.x.to_bits(),
                            m.z.to_bits(),
                            m.hp,
                            m.hits,
                            m.reaction,
                            m.frame(),
                            m.stop.remaining_ms.to_bits(),
                        )
                    })
                    .collect(),
            });
        }
    }
    assert_eq!(g.world_updates, 1200);
    assert_eq!(g.replay_frame(), 240);
    (poses, sounds)
}

pub fn verify(data: &GameData, out: &std::path::Path) -> Result<(), String> {
    std::fs::create_dir_all(out).map_err(|e| e.to_string())?;
    let mut csv =
        String::from("case,driver_hz,world_hz,world_updates,matching_snapshots,sound_events\n");
    for combat in [false, true] {
        let reference = frequency_replay(data, 300, combat);
        for hz in [60, 120, 300] {
            let actual = frequency_replay(data, hz, combat);
            if actual != reference {
                return Err(format!("timeline mismatch driver={hz} combat={combat}"));
            }
            csv.push_str(&format!(
                "{},{hz},300,1200,{},{}\n",
                if combat { "combat" } else { "movement" },
                actual.0.len(),
                actual.1.len()
            ));
        }
    }
    std::fs::write(out.join("timing.csv"), csv).map_err(|e| e.to_string())?;
    Ok(())
}
