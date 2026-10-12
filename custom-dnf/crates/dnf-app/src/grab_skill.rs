//! 40JP skill31: state22, 5EF0A0 admission, 5F10C0 100/750ms gates.
use crate::{
    assets::ClientAssets,
    battle::{self, Box3, Hit, Reaction},
    game::{Game, GameData},
};
use dnf_core::{
    anim::{AnimPlayer, Clip},
    protection::Protection,
    ActorEvent, Motion,
};
use dnf_data::{ani::AnimFile, combat::AttackInfo, skill::SkillDefinition};
use dnf_render::framebuffer::Framebuffer;
use std::rc::Rc;

const OWNER: u64 = 31;
const INVINCIBLE_SOURCE: u32 = 31;
pub struct Assets {
    pub body: Rc<AnimFile>,
    hold: Rc<AnimFile>,
    blast: Rc<AnimFile>,
    absorb: AttackInfo,
    burst: AttackInfo,
    down_ms: f64,
}
impl Assets {
    pub fn load(a: &ClientAssets, skills: &[SkillDefinition]) -> Result<Self, String> {
        let ani = |p: &str| -> Result<Rc<AnimFile>, String> {
            let v = a.ani(p).ok_or_else(|| format!("missing {p}"))?;
            for f in &v.frames {
                if !f.image.is_empty() && a.img(&f.image).is_none() {
                    return Err(format!("missing {}", f.image));
                }
            }
            Ok(v)
        };
        let atk = |p: &str| {
            a.pvf
                .read_text(p)
                .map(|t| dnf_data::combat::parse_attack_info(&t))
                .map_err(|e| e.to_string())
        };
        let obj = a
            .pvf
            .read_text("passiveobject/character/swordman/grabblastblood.obj")
            .map_err(|e| e.to_string())?;
        let obj = dnf_data::mob::parse(&obj);
        let blast_path = obj
            .motion("basic motion")
            .ok_or("grab PO basic motion missing")?;
        let skill = skills
            .iter()
            .find(|s| s.id == 31)
            .ok_or("skill31 missing")?;
        let down_ms = skill
            .fields
            .get("static data")
            .and_then(|v| v.split_whitespace().next())
            .and_then(|v| v.parse().ok())
            .ok_or("grab down duration missing")?;
        Ok(Self {
            body: ani("character/swordman/animation/grab.ani")?,
            hold: ani("character/swordman/effect/animation/grabblastblood.ani")?,
            blast: ani(&format!("passiveobject/character/swordman/{blast_path}"))?,
            absorb: atk("character/swordman/attackinfo/grabblastblood.atk")?,
            burst: atk("passiveobject/character/swordman/attackinfo/grabblastblood.atk")?,
            down_ms,
        })
    }
}
struct Hold {
    target: usize,
    elapsed: f64,
    sucked: bool,
}
struct Burst {
    anim: AnimPlayer,
    origin: [f32; 3],
    facing: i32,
    hit: std::collections::HashSet<usize>,
    damage: i32,
}
#[derive(Default)]
pub struct Runtime {
    hold: Option<Hold>,
    burst: Option<Burst>,
    hold_fx: Option<AnimPlayer>,
    strength_ms: f64,
    strength: i32,
    attempted: bool,
}
impl Runtime {
    pub fn strength_bonus(&self) -> i32 {
        if self.strength_ms > 0.0 {
            self.strength
        } else {
            0
        }
    }
    // Training damage model: strength ratio. Full official equipment/STR formula is separate.
    pub fn scale_physical(&self, damage: i32, base_strength: i32) -> i32 {
        (damage as i64 * (250 + base_strength.max(0) + self.strength_bonus()) as i64
            / (250 + base_strength.max(0)) as i64)
            .clamp(1, i32::MAX as i64) as i32
    }
    pub fn target(&self) -> Option<usize> {
        self.hold.as_ref().map(|h| h.target)
    }
    pub fn cancel(&mut self, g: &mut Game) {
        if let Some(h) = self.hold.take() {
            let assets = g.goblin_assets.clone();
            if let Some(m) = g.monster_mut(h.target) {
                m.release_grab(OWNER, &assets, None, 0.0);
            }
        }
        self.hold_fx = None;
        self.burst = None;
        g.actor.protection.remove(INVINCIBLE_SOURCE);
        g.actor.release_grab();
    }
    fn damage(g: &mut Game, index: usize, info: &AttackInfo, damage: i32) -> bool {
        let assets = g.goblin_assets.clone();
        let feedback = info.feedback(&dnf_data::combat::HitInfo {
            kind: "etc".into(),
            blood: false,
            stop_ms: 0,
            recovery_rate: 1.0,
        });
        let facing = g.actor.facing;
        let Some(m) = g.monster_mut(index) else {
            return false;
        };
        let point = [m.x, m.y, m.z + 45.0];
        let damage = m.reduced_damage(damage);
        let hit = Hit {
            damage,
            facing,
            upper: false,
            point,
        };
        let applied = if m.reaction == Reaction::Grabbed {
            m.grab_damage(&assets, damage)
        } else {
            m.hit(&assets, info, &feedback, hit)
        };
        if !applied {
            return false;
        }
        let dead = m.hp == 0;
        g.popups.push(crate::damage::DamagePopup {
            damage,
            point,
            age_ms: 0.0,
            style: g.debug_values()[19],
        });
        g.sound_requests
            .push(if dead { "R_GOBLIN_DIE" } else { "R_GOBLIN_DMG" });
        true
    }
    pub fn step(&mut self, g: &mut Game) {
        let dt = g.jt.logic_ms;
        self.strength_ms = (self.strength_ms - dt).max(0.0);
        if self.strength_ms < 1e-7 {
            self.strength_ms = 0.0;
        }
        let assets = g.grab_assets.clone();
        if g.last_event == ActorEvent::BloodlustCast {
            self.attempted = false;
        }
        if let Some(mut hold) = self.hold.take() {
            let valid = g.actor.motion == Motion::Bloodlust
                && g.actor.hp > 0
                && g.monster(hold.target).is_some_and(|m| {
                    m.hp > 0
                        && m.reaction == Reaction::Grabbed
                        && m.grab.owner() == Some(OWNER)
                        && m.grab.grabable
                        && !m.statuses.blocks_grab()
                        && m.effective_protection(&g.goblin_assets) != Protection::Invincible
                });
            if !valid {
                self.hold = Some(hold);
                self.cancel(g);
            } else {
                hold.elapsed += dt;
                let x = (g.actor.x + 43 * g.actor.facing)
                    .clamp(g.room.walk_x.0 + 20, g.room.walk_x.1 - 20)
                    as f32;
                let y = g.actor.y as f32 - 1.0;
                let height = g
                    .monster(hold.target)
                    .unwrap()
                    .hurt_boxes(&g.goblin_assets)
                    .iter()
                    .map(|b| b.max[2] - g.monster(hold.target).unwrap().z)
                    .fold(0.0, f32::max);
                let m = g.monster_mut(hold.target).unwrap();
                m.x = x;
                m.y = y;
                m.z = (110.0 - height).max(10.0); // 650E80, height from current body boxes
                if let Some(fx) = &mut self.hold_fx {
                    fx.advance(dt);
                }
                if !hold.sucked && hold.elapsed + 1e-7 >= 100.0 {
                    Self::damage(
                        g,
                        hold.target,
                        &assets.absorb,
                        g.actor.blood_skill.values[0],
                    );
                    g.sound_requests.push("BLOOD_SUCK");
                    hold.sucked = true;
                }
                if g.monster(hold.target).is_some_and(|m| m.hp == 0) {
                    self.hold = Some(hold);
                    self.cancel(g);
                } else if hold.elapsed + 1e-7 >= 750.0 {
                    let damage = g.actor.blood_skill.values[1];
                    let origin = [
                        g.actor.x as f32 + 43.0 * g.actor.facing as f32,
                        g.actor.y as f32,
                        90.0,
                    ];
                    let facing = g.actor.facing;
                    Self::damage(g, hold.target, &assets.burst, damage);
                    let monster_assets = g.goblin_assets.clone();
                    if let Some(m) = g.monster_mut(hold.target) {
                        m.release_grab(
                            OWNER,
                            &monster_assets,
                            Some((&assets.burst, facing)),
                            assets.down_ms,
                        );
                    }
                    self.burst = Some(Burst {
                        anim: AnimPlayer::new(
                            Clip::new("GrabBlastBlood2", assets.blast.delays(), false),
                            100.0,
                        ),
                        origin,
                        facing,
                        hit: std::iter::once(hold.target).collect(),
                        damage,
                    });
                    self.strength_ms = g.actor.blood_skill.values[2] as f64;
                    self.strength = g.actor.blood_skill.values[3];
                    self.hold_fx = None;
                    g.actor.protection.remove(INVINCIBLE_SOURCE);
                    g.actor.release_grab();
                    g.sound_requests.extend(["BLOOD_SPREAD", "FIRE_PILLAR_HIT"]);
                } else {
                    self.hold = Some(hold);
                }
            }
        }
        if self.hold.is_none()
            && !self.attempted
            && g.actor.motion == Motion::Bloodlust
            && !g.actor.anim().finished()
        {
            let frame = g.actor.anim().frame();
            if let Some(body) = assets.body.frames.get(frame) {
                let origin = [g.actor.x as f32, g.actor.y as f32, g.actor.altitude()];
                // First valid contact wins; invincible/non-grabable targets don't block others.
                let target = (0..g.monster_count()).find(|&index| {
                    let m = g.monster(index).unwrap();
                    if m.statuses.blocks_grab()
                        || !m
                            .grab
                            .can_grab(m.hp > 0, m.effective_protection(&g.goblin_assets))
                    {
                        return false;
                    }
                    let hurt = m.hurt_boxes(&g.goblin_assets);
                    body.attack_boxes.iter().any(|&b| {
                        hurt.iter()
                            .any(|&h| Box3::translated(b, origin, g.actor.facing).intersects(h))
                    })
                });
                if let Some(index) = target {
                    let monster_assets = g.goblin_assets.clone();
                    if g.monster_mut(index)
                        .unwrap()
                        .acquire_grab(OWNER, &monster_assets)
                    {
                        self.hold = Some(Hold {
                            target: index,
                            elapsed: 0.0,
                            sucked: false,
                        });
                        self.hold_fx = Some(AnimPlayer::new(
                            Clip::new("BloodHold", assets.hold.delays(), true),
                            100.0,
                        ));
                        self.attempted = true;
                        g.actor.hold_grab();
                        g.actor
                            .protection
                            .grant(INVINCIBLE_SOURCE, Protection::Invincible, None);
                        g.sound_requests.push("BLOOD_HOLD");
                    }
                }
            }
        }
        if let Some(mut burst) = self.burst.take() {
            if let Some(frame) = assets.blast.frames.get(burst.anim.frame()) {
                for i in 0..g.monster_count() {
                    if burst.hit.contains(&i) {
                        continue;
                    }
                    let hurt = g.monster(i).unwrap().hurt_boxes(&g.goblin_assets);
                    if frame.attack_boxes.iter().any(|&b| {
                        hurt.iter()
                            .any(|&h| Box3::translated(b, burst.origin, burst.facing).intersects(h))
                    }) && Self::damage(g, i, &assets.burst, burst.damage)
                    {
                        burst.hit.insert(i);
                    }
                }
            }
            burst.anim.advance(dt);
            if !burst.anim.finished() {
                self.burst = Some(burst);
            }
        }
    }
    pub fn draw(&self, g: &Game, data: &GameData, fb: &mut Framebuffer) {
        if let Some(fx) = &self.hold_fx {
            battle::draw_frame(
                &g.grab_assets.hold,
                fx.frame(),
                &data.assets,
                fb,
                (g.actor.x - g.camera_x + 43 * g.actor.facing, g.actor.y - 90),
                g.actor.facing,
                1.0,
            );
        }
        if let Some(b) = &self.burst {
            battle::draw_frame(
                &g.grab_assets.blast,
                b.anim.frame(),
                &data.assets,
                fb,
                (
                    b.origin[0].round() as i32 - g.camera_x,
                    (b.origin[1] - b.origin[2]).round() as i32,
                ),
                b.facing,
                1.0,
            );
        }
    }
}
impl Game {
    pub(crate) fn cancel_grab(&mut self) {
        let mut grab = std::mem::take(&mut self.grab_skill);
        grab.cancel(self);
        self.grab_skill = grab;
    }
}

fn scenario(data: &GameData, hz: u32, state: Protection, grabable: bool) -> Game {
    let mut g = data
        .build_game(dnf_core::KeyMap::official_default())
        .unwrap();
    g.jt.logic_ms = 1000.0 / hz as f64;
    g.actor.teleport(300, 329);
    g.actor.facing = 1;
    g.goblin.max_hp = 5000;
    g.goblin.reset(350, 329, &data.goblin);
    g.goblin.grab.grabable = grabable;
    g.goblin.protection.grant(987, state, None);
    g
}
fn tick(g: &mut Game, pressed: bool) {
    let key = g.keymap.get(dnf_core::keymap::Command::SkillSlot3);
    g.begin_input();
    g.set_key(key, pressed);
    g.step();
}
fn shot(g: &Game, data: &GameData, out: &std::path::Path, name: &str) -> Result<(), String> {
    let mut fb = Framebuffer::default();
    g.render(data, &mut fb);
    dnf_render::png::write_file(&out.join(format!("{name}.png")), &fb).map_err(|e| e.to_string())
}
pub fn verify(data: &GameData, out: &std::path::Path) -> Result<(), String> {
    std::fs::create_dir_all(out).map_err(|e| e.to_string())?;
    let mut csv = String::from("state,grabable,driver_hz,captured,hits,damage,strength_bonus\n");
    for state in [
        Protection::Normal,
        Protection::SuperArmor,
        Protection::Invincible,
    ] {
        for grabable in [true, false] {
            for hz in [60, 120, 300] {
                let mut g = scenario(data, hz, state, grabable);
                let mut captured = false;
                let mut sounds = Vec::new();
                for t in 0..hz * 2 {
                    tick(&mut g, t == 0);
                    sounds.extend_from_slice(&g.sound_requests);
                    captured |= g.grab_skill.target() == Some(0);
                    if hz == 300
                        && state == Protection::SuperArmor
                        && grabable
                        && [75, 105, 290].contains(&t)
                    {
                        shot(&g, data, out, &format!("bloodlust_{t}"))?;
                    }
                }
                let expected = grabable && state != Protection::Invincible;
                let damage = 5000 - g.goblin.hp;
                if captured != expected
                    || damage != if expected { 492 } else { 0 }
                    || g.goblin.hits != if expected { 2 } else { 0 }
                    || g.grab_skill.target().is_some()
                    || g.goblin.grab.owner().is_some()
                    || g.actor.protection.effective() != Protection::Normal
                {
                    return Err(format!("grab matrix {state:?}/{grabable}/{hz}: captured={captured},damage={damage},hits={},actor={:?}",g.goblin.hits,g.actor.protection.effective()));
                }
                if expected
                    && (["BLOOD_HOLD", "BLOOD_SUCK", "BLOOD_SPREAD"]
                        .iter()
                        .any(|tag| sounds.iter().filter(|s| *s == tag).count() != 1)
                        || g.grab_skill.strength_bonus() != 20)
                {
                    return Err("grab sound/buff sequence mismatch".into());
                }
                csv.push_str(&format!(
                    "{state:?},{grabable},{hz},{captured},{},{damage},{}\n",
                    g.goblin.hits,
                    g.grab_skill.strength_bonus()
                ));
            }
        }
    }
    let mut g = scenario(data, 300, Protection::SuperArmor, true);
    g.debug_panel.open = true;
    g.debug_panel.tab = 1;
    shot(&g, data, out, "grab_controls")?;
    g.debug_panel.open = false;
    g.hud.library_open = true;
    let (x, y, _, _) = crate::hud::learned_rect(2);
    g.hud.pointer = (x + 10, y + 10, false);
    shot(&g, data, out, "bloodlust_chinese_tooltip")?;
    std::fs::write(out.join("grab_matrix.csv"), csv).map_err(|e| e.to_string())?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> Option<GameData> {
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
    fn real_grab_does_not_damage_rejected_targets_and_timeline_is_driver_independent() {
        let Some(data) = fixture() else { return };
        let out = std::env::temp_dir().join(format!("dnf-grab-test-{}", std::process::id()));
        verify(&data, &out).unwrap();
    }
    #[test]
    fn capture_has_100ms_absorb_750ms_release_and_five_second_buff() {
        let Some(data) = fixture() else { return };
        let mut g = scenario(&data, 300, Protection::SuperArmor, true);
        tick(&mut g, true);
        while g.grab_skill.target().is_none() {
            tick(&mut g, false);
            assert!(g.world_updates < 100);
        }
        assert_eq!(g.goblin.hits, 0);
        assert_eq!(g.actor.protection.effective(), Protection::Invincible);
        for _ in 0..29 {
            tick(&mut g, false);
        }
        assert_eq!(g.goblin.hits, 0);
        tick(&mut g, false);
        assert_eq!(g.goblin.hp, 4836);
        for _ in 0..194 {
            tick(&mut g, false);
        }
        assert_eq!(g.grab_skill.target(), Some(0));
        tick(&mut g, false);
        assert_eq!(g.grab_skill.target(), None);
        assert_eq!(g.goblin.hp, 4508);
        assert_eq!(g.grab_skill.strength_bonus(), 20);
        for _ in 0..1499 {
            tick(&mut g, false);
        }
        assert_eq!(g.grab_skill.strength_bonus(), 20);
        tick(&mut g, false);
        assert_eq!(g.grab_skill.strength_bonus(), 0);
    }
    #[test]
    fn target_removal_death_reset_and_invincibility_cancel_without_stale_ownership() {
        let Some(data) = fixture() else { return };
        for change in 0..5 {
            let mut g = scenario(&data, 300, Protection::Normal, true);
            tick(&mut g, true);
            while g.grab_skill.target().is_none() {
                tick(&mut g, false);
                assert!(g.world_updates < 100);
            }
            match change {
                0 => g.set_monster_count(0),
                1 => g.goblin.set_hp(0, &data.goblin),
                2 => g.apply_control(crate::debug_ui::Control::Respawn(Some(0))),
                3 => g.goblin.protection.grant(876, Protection::Invincible, None),
                _ => g.actor.teleport(500, 329),
            }
            tick(&mut g, false);
            assert!(g.grab_skill.target().is_none());
            assert!(g.goblin.grab.owner().is_none());
            assert_eq!(g.actor.protection.effective(), Protection::Normal);
            assert_eq!(g.grab_skill.strength_bonus(), 0);
        }
    }
    #[test]
    fn burst_hits_nearby_targets_once_and_preserves_other_invincible_targets() {
        let Some(data) = fixture() else { return };
        let mut g = scenario(&data, 300, Protection::SuperArmor, true);
        g.set_monster_count(3);
        for i in 1..3 {
            let m = g.monster_mut(i).unwrap();
            m.reset(400, 329, &data.goblin);
        }
        g.monster_mut(2)
            .unwrap()
            .protection
            .grant(800, Protection::Invincible, None);
        tick(&mut g, true);
        for _ in 0..600 {
            tick(&mut g, false);
        }
        assert_eq!(g.monster(0).unwrap().hits, 2);
        assert_eq!(g.monster(1).unwrap().hits, 1);
        assert_eq!(g.monster(2).unwrap().hits, 0);
    }
    #[test]
    fn translated_skl_descriptions_preserve_official_runtime_tables() {
        let Some(data) = fixture() else { return };
        for id in crate::hud::SKILLS {
            let translated = data.skills.iter().find(|s| s.id == id).unwrap();
            let raw = data.assets.pvf.read_text(&translated.path).unwrap();
            let original = SkillDefinition::parse(id, &translated.path, &raw).unwrap();
            assert_eq!(translated.mp, original.mp);
            assert_eq!(translated.cooldown, original.cooldown);
            assert_eq!(translated.level_info, original.level_info);
            assert_eq!(
                translated.text("name", &data.skill_strings),
                crate::hud::skill_name(id)
            );
            assert!(!translated
                .text("explain", &data.skill_strings)
                .contains('?'));
            assert!(translated
                .level_description(1, &data.skill_strings)
                .is_some());
        }
    }
    #[test]
    fn grabbed_positions_animation_cooldowns_and_buff_match_at_all_driver_rates() {
        let Some(data) = fixture() else { return };
        let replay = |hz: u32| {
            let mut g = scenario(&data, hz, Protection::SuperArmor, true);
            let mut samples = Vec::new();
            let mut sounds = Vec::new();
            for t in 0..hz * 7 {
                tick(&mut g, t == 0);
                sounds.extend_from_slice(&g.sound_requests);
                if (t + 1) % (hz / 20) == 0 {
                    samples.push((
                        g.actor.x,
                        g.actor.motion,
                        g.actor.anim().frame(),
                        g.actor.skill_cooldown_remaining_ms(31).map(f64::to_bits),
                        g.actor.mp,
                        g.actor.protection.effective(),
                        g.grab_skill.target(),
                        g.grab_skill.strength_bonus(),
                        g.goblin.hp,
                        g.goblin.x.to_bits(),
                        g.goblin.z.to_bits(),
                        g.goblin.reaction,
                    ));
                }
            }
            (samples, sounds)
        };
        let reference = replay(300);
        assert_eq!(replay(60), reference);
        assert_eq!(replay(120), reference);
    }
}
