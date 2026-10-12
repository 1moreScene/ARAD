use crate::game::{Game, GameData};
use dnf_core::active_status::ActiveStatus;
use dnf_render::framebuffer::Framebuffer;
fn fixture_game(data: &GameData) -> Game {
    let mut g = data
        .build_game(dnf_core::KeyMap::official_default())
        .unwrap();
    g.actor.teleport(320, 329);
    g.actor.facing = 1;
    g.goblin.max_hp = 50000;
    g.goblin.reset(410, 329, &data.goblin);
    g.actor
        .protection
        .grant(919, dnf_core::protection::Protection::Invincible, None);
    g
}
fn ticks(g: &mut Game, n: usize) {
    for _ in 0..n {
        g.begin_input();
        g.step();
    }
}
fn shot(g: &Game, data: &GameData, out: &std::path::Path, name: &str) -> Result<(), String> {
    let mut fb = Framebuffer::default();
    g.render(data, &mut fb);
    dnf_render::png::write_file(&out.join(format!("{name}.png")), &fb).map_err(|e| e.to_string())
}
pub fn verify(data: &GameData, out: &std::path::Path) -> Result<(), String> {
    std::fs::create_dir_all(out).map_err(|e| e.to_string())?;
    let mut csv = String::from(
        "id,token,implemented,level,duration_ms,power,secondary,applied,damage_after_700ms\n",
    );
    for kind in ActiveStatus::ALL {
        let mut g = fixture_game(data);
        let config = data.goblin.status_assets.config(kind, 1);
        let applied = g.apply_monster_status(Some(0), kind, 1);
        if (applied == 1) != kind.implemented() {
            return Err(format!("status admission {kind:?} mismatch"));
        }
        ticks(&mut g, 42);
        csv.push_str(&format!(
            "{},{},{},1,{:.0},{:.4},{:.4},{},{}\n",
            kind as u8,
            kind.token(),
            kind.implemented(),
            config.duration_ms,
            config.power,
            config.secondary,
            applied,
            50000 - g.goblin.hp
        ));
        if kind.implemented() {
            shot(
                &g,
                data,
                out,
                &format!("{:02}_{}", kind as u8, kind.token()),
            )?;
        }
    }
    let mut g = fixture_game(data);
    for kind in ActiveStatus::ALL
        .into_iter()
        .filter(|s| s.implemented() && *s != ActiveStatus::Sleep)
    {
        g.apply_monster_status(None, kind, 8);
    }
    ticks(&mut g, 42);
    if g.goblin.statuses.iter().count() != 14 {
        return Err("coexisting fourteen states lost a slot".into());
    }
    shot(&g, data, out, "20_coexisting")?;
    g.debug_panel.open = true;
    g.debug_panel.tab = 1;
    g.debug_panel.monster_page = 1;
    shot(&g, data, out, "21_status_panel")?;
    g.debug_panel.status_dropdown = true;
    shot(&g, data, out, "22_status_dropdown")?;
    let mut g = fixture_game(data);
    g.apply_monster_status(None, ActiveStatus::Freeze, 1);
    ticks(&mut g, 125);
    shot(&g, data, out, "23_thaw")?;
    let mut g = fixture_game(data);
    g.apply_monster_status(None, ActiveStatus::Stone, 1);
    ticks(&mut g, 30);
    g.apply_control(crate::debug_ui::Control::MonsterStatusRemove {
        target: None,
        kind: ActiveStatus::Stone,
    });
    ticks(&mut g, 10);
    shot(&g, data, out, "24_stone_release")?;
    std::fs::write(out.join("active_status.csv"), csv).map_err(|e| e.to_string())?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn continuous_hp_damage_uses_five_hundred_ms_visual_feedback_and_survives_grab() {
        let Some(data) = data() else { return };
        let mut g = fixture_game(&data);
        g.jt.logic_ms = dnf_core::clock::WORLD_MS;
        g.apply_monster_status(None, ActiveStatus::Poison, 40);
        ticks(&mut g, 147);
        assert!(g.goblin.hp < 50000);
        assert!(g.popups.is_empty());
        ticks(&mut g, 3);
        assert_eq!(g.popups.len(), 1);
        assert_eq!(g.popups[0].damage, 50000 - g.goblin.hp);
        let hp = g.goblin.hp;
        assert!(g.goblin.acquire_grab(31, &data.goblin));
        g.goblin.step(&data.goblin, g.room.walk_x, 100.0);
        assert!(g.goblin.hp < hp);
        assert!(g.goblin.statuses.has(ActiveStatus::Poison));
    }
    fn data() -> Option<GameData> {
        Some(
            GameData::load(
                &dnf_data::locate_client_dir()?,
                crate::game::DEFAULT_MAP,
                crate::character::Outfit::default_basic(),
            )
            .unwrap(),
        )
    }
    #[test]
    fn all_official_active_statuses_have_assets_and_can_coexist() {
        let Some(data) = data() else { return };
        let out = std::env::temp_dir().join(format!("dnf-active-status-{}", std::process::id()));
        verify(&data, &out).unwrap();
    }
    #[test]
    fn multi_dot_timers_and_physics_match_sixty_one_twenty_three_hundred_hz() {
        let Some(data) = data() else { return };
        let replay = |hz: u32| {
            let mut g = fixture_game(&data);
            g.jt.logic_ms = 1000.0 / hz as f64;
            g.goblin.demo_enabled = true;
            for kind in [
                ActiveStatus::Slow,
                ActiveStatus::Poison,
                ActiveStatus::Burn,
                ActiveStatus::Bleeding,
                ActiveStatus::Curse,
                ActiveStatus::Bless,
            ] {
                g.apply_monster_status(None, kind, 1);
            }
            let mut result = Vec::new();
            for t in 0..hz * 8 {
                g.begin_input();
                g.step();
                if (t + 1) % (hz / 20) == 0 {
                    result.push((
                        g.goblin.hp,
                        g.goblin.x.to_bits(),
                        g.goblin.frame(),
                        g.goblin.statuses.mask(),
                        g.goblin.status_damage,
                    ));
                }
            }
            result
        };
        let expected = replay(300);
        assert_eq!(replay(60), expected);
        assert_eq!(replay(120), expected);
    }
    #[test]
    fn invulnerability_admission_hit_shock_sleep_and_dot_death_cleanup_work() {
        let Some(data) = data() else { return };
        let mut g = fixture_game(&data);
        g.goblin
            .protection
            .grant(51, dnf_core::protection::Protection::Invincible, None);
        assert_eq!(g.apply_monster_status(None, ActiveStatus::Poison, 1), 0);
        g.goblin.protection.remove(51);
        g.apply_monster_status(None, ActiveStatus::Lightning, 1);
        g.apply_monster_status(None, ActiveStatus::Sleep, 1);
        ticks(&mut g, 20);
        let info=dnf_data::combat::parse_attack_info("[damage reaction] `[none]`\n[active status] `[poison]` 100 1\n[active status] `[curse]` 100 1");
        let feedback = dnf_data::combat::HitInfo {
            kind: "etc".into(),
            blood: false,
            stop_ms: 0,
            recovery_rate: 1.0,
        };
        let hp = g.goblin.hp;
        assert!(g.goblin.hit(
            &data.goblin,
            &info,
            &feedback,
            crate::battle::Hit {
                damage: 10,
                facing: 1,
                upper: false,
                point: [410.0, 329.0, 0.0]
            }
        ));
        assert_eq!(hp - g.goblin.hp, 28);
        assert!(!g.goblin.statuses.has(ActiveStatus::Sleep));
        g.goblin.apply_attack_statuses(&data.goblin, &info);
        assert!(
            g.goblin.statuses.has(ActiveStatus::Poison)
                && g.goblin.statuses.has(ActiveStatus::Curse)
        );
        g.goblin.set_hp(1, &data.goblin);
        ticks(&mut g, 20);
        assert_eq!(g.goblin.hp, 0);
        assert!(g.goblin.statuses.is_empty());
        ticks(&mut g, 190);
        assert!(g.goblin.hp > 0);
        assert!(g.goblin.statuses.is_empty());
    }
    #[test]
    fn freeze_and_stone_block_grabs_without_disabling_unrelated_statuses() {
        let Some(data) = data() else { return };
        let mut g = fixture_game(&data);
        g.apply_monster_status(None, ActiveStatus::Poison, 1);
        g.apply_monster_status(None, ActiveStatus::Freeze, 1);
        g.apply_monster_status(None, ActiveStatus::Stone, 1);
        let x = g.goblin.x;
        let frame = g.goblin.frame();
        g.goblin.demo_enabled = true;
        ticks(&mut g, 30);
        assert_eq!(g.goblin.x, x);
        assert_eq!(g.goblin.frame(), frame);
        assert!(g.goblin.hp < 50000);
        assert!(!g.goblin.acquire_grab(31, &data.goblin));
        g.apply_control(crate::debug_ui::Control::MonsterStatusRemove {
            target: None,
            kind: ActiveStatus::Freeze,
        });
        assert!(
            g.goblin.statuses.has(ActiveStatus::Poison)
                && g.goblin.statuses.has(ActiveStatus::Stone)
        );
        g.apply_control(crate::debug_ui::Control::MonsterStatusRemove {
            target: None,
            kind: ActiveStatus::Stone,
        });
        assert!(g.goblin.acquire_grab(31, &data.goblin));
    }
    #[test]
    fn panel_pauses_status_timers_and_scopes_apply_only_selected_monster() {
        let Some(data) = data() else { return };
        let mut g = fixture_game(&data);
        g.set_monster_count(3);
        g.apply_monster_status(Some(1), ActiveStatus::Poison, 1);
        g.apply_monster_status(None, ActiveStatus::Slow, 1);
        assert!(!g.monster(0).unwrap().statuses.has(ActiveStatus::Poison));
        assert!(g.monster(1).unwrap().statuses.has(ActiveStatus::Poison));
        let state = g.monster(1).unwrap().statuses.clone();
        g.debug_panel.open = true;
        g.debug_panel.tab = 1;
        g.debug_panel.monster_page = 1;
        ticks(&mut g, 60);
        assert_eq!(
            g.monster(1).unwrap().statuses.get(ActiveStatus::Poison),
            state.get(ActiveStatus::Poison)
        );
        g.apply_control(crate::debug_ui::Control::MonsterStatusRemove {
            target: Some(1),
            kind: ActiveStatus::Poison,
        });
        assert!(g.monsters().all(|m| m.statuses.has(ActiveStatus::Slow)));
    }
    #[test]
    fn dropdown_selection_does_not_apply_or_replace_existing_states() {
        let Some(data) = data() else { return };
        let mut g = fixture_game(&data);
        g.apply_monster_status(None, ActiveStatus::Poison, 1);
        g.debug_panel.open = true;
        g.debug_panel.tab = 1;
        g.debug_panel.monster_page = 1;
        let view = crate::debug_ui::View::from_game(&g);
        let keys = dnf_core::input::InputState::new();
        assert!(g
            .debug_panel
            .input(&keys, (150, 196, true), &view)
            .is_none());
        assert!(g.debug_panel.status_dropdown);
        g.debug_panel.input(&keys, (150, 196, false), &view);
        g.debug_panel.input(&keys, (80, 217, true), &view);
        assert!(!g.debug_panel.status_dropdown);
        assert_eq!(g.debug_panel.status_choice, 0);
        g.debug_panel.input(&keys, (80, 217, false), &view);
        let action = g.debug_panel.input(&keys, (80, 232, true), &view).unwrap();
        g.apply_control(action);
        assert!(
            g.goblin.statuses.has(ActiveStatus::Poison)
                && g.goblin.statuses.has(ActiveStatus::Slow)
        );
    }
}
