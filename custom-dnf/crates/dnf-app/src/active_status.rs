//! Official ActiveStatus tuning and ANI/PTL presentation for training monsters.
use crate::{assets::ClientAssets, battle::Goblin, game::Game};
use dnf_core::active_status::{ActiveStatus, ActiveStatuses, StatusConfig};
use dnf_data::{
    ani::AnimFile,
    img::ImgFile,
    particle::{AxisKind, ParticleScript},
};
use dnf_render::framebuffer::{rgba, Color, Framebuffer, SpriteTransform};
use std::{collections::HashMap, rc::Rc};
#[derive(Clone, Copy)]
enum Anchor {
    Feet,
    Body,
    Head,
}
struct Effect {
    ani: Rc<AnimFile>,
    anchor: Anchor,
    offset: (i32, i32),
    behind: bool,
    hold_last: bool,
}
struct Emitter {
    script: ParticleScript,
    objects: Vec<Rc<AnimFile>>,
    z: f32,
}
pub struct Assets {
    target_size: (f32, f32),
    prepared: HashMap<(usize, usize), dnf_render::framebuffer::PreparedSprite>,
    pub table: dnf_data::active_status::StatusTable,
    effects: HashMap<ActiveStatus, Vec<Effect>>,
    emitters: HashMap<ActiveStatus, Vec<Emitter>>,
    images: HashMap<String, Rc<ImgFile>>,
    freeze_exit: [Option<Rc<AnimFile>>; 2],
    stone_exit: Option<Emitter>,
}
impl Assets {
    pub fn load(a: &ClientAssets, target_size: (f32, f32)) -> Result<Self, String> {
        let table = dnf_data::active_status::StatusTable::load(&a.client_dir.join("ARAD.exe"))?;
        let mut out = Self {
            target_size,
            prepared: HashMap::new(),
            table,
            effects: HashMap::new(),
            emitters: HashMap::new(),
            images: HashMap::new(),
            freeze_exit: [None, None],
            stone_exit: None,
        };
        let base = "common/activestatus/animation";
        use ActiveStatus::*;
        for (kind, file, anchor, offset, behind) in [
            (Slow, "Slow", Anchor::Feet, (0, 0), false),
            (Freeze, "Freeze1", Anchor::Body, (0, 0), true),
            (Freeze, "Freeze2", Anchor::Body, (0, 0), false),
            (Freeze, "Freeze3", Anchor::Body, (0, 0), false),
            (Stun, "Stun1", Anchor::Head, (0, -5), true),
            (Stun, "Stun2", Anchor::Head, (0, -5), false),
            (Blind, "Blind1", Anchor::Head, (0, -5), true),
            (Blind, "Blind2", Anchor::Head, (0, -5), false),
            (Lightning, "Lightning", Anchor::Body, (0, 0), false),
            (Stone, "StoneCurse", Anchor::Body, (0, 0), false),
            (Sleep, "Sleep", Anchor::Body, (30, -36), false),
            (
                Burn,
                "ActiveStatus_BurnDamage_Effect",
                Anchor::Body,
                (0, 0),
                false,
            ),
            (Bleeding, "Bleeding", Anchor::Head, (0, 12), false),
            (Haste, "Haste", Anchor::Feet, (0, 0), false),
            (Confuse, "Confuse", Anchor::Head, (0, -50), false),
            (Confuse, "ConfuseGlow", Anchor::Head, (0, -50), false),
            (Hold, "Hold", Anchor::Body, (0, -58), false),
            (Hold, "Hold", Anchor::Body, (0, -38), false),
            (Hold, "Hold", Anchor::Body, (0, -17), false),
        ] {
            let path = format!("{base}/{file}.ani");
            let ani = out.load_ani(a, &path)?;
            out.effects.entry(kind).or_default().push(Effect {
                ani,
                anchor,
                offset,
                behind,
                hold_last: file != "Freeze3",
            });
        }
        for (kind, file, z) in [
            (Poison, "PoisonParticle", 70.0),
            (Curse, "CurseParticle", 70.0),
            (Bless, "BlessParticle1", 50.0),
            (Bless, "BlessParticle2", 0.0),
            (Stone, "StoneCursing", 10.0),
        ] {
            let dir = "common/activestatus/particle";
            let path = format!("{dir}/{file}.ptl");
            let text = a.pvf.read_text(&path).map_err(|e| e.to_string())?;
            let script = dnf_data::particle::parse(&text, dir)?;
            let mut objects = Vec::new();
            for path in &script.objects {
                objects.push(out.load_ani(a, path)?);
            }
            out.emitters
                .entry(kind)
                .or_default()
                .push(Emitter { script, objects, z });
        }
        out.freeze_exit[0] = Some(out.load_ani(a, "common/activestatus/animation/freezedie.ani")?);
        out.freeze_exit[1] = Some(out.load_ani(a, "common/activestatus/animation/freezedie.ani")?);
        let text = a
            .pvf
            .read_text("common/activestatus/particle/stonecursedestroy.ptl")
            .map_err(|e| e.to_string())?;
        let script = dnf_data::particle::parse(&text, "common/activestatus/particle")?;
        let mut objects = Vec::new();
        for path in &script.objects {
            objects.push(out.load_ani(a, path)?);
        }
        out.stone_exit = Some(Emitter {
            script,
            objects,
            z: 30.0,
        });
        Ok(out)
    }
    fn load_ani(&mut self, a: &ClientAssets, path: &str) -> Result<Rc<AnimFile>, String> {
        let original = a
            .ani(path)
            .ok_or_else(|| format!("missing status animation {path}"))?;
        let mut ani = (*original).clone();
        let lower = path.to_ascii_lowercase();
        let scale = if lower.contains("/freeze") {
            (self.target_size.0 / 130.0, self.target_size.1 / 140.0)
        } else if lower.contains("/stone") {
            (self.target_size.0 / 60.0, self.target_size.1 / 60.0)
        } else if lower.ends_with("activestatus_burndamage_effect.ani") {
            (
                (self.target_size.0 * 1.5 / 100.0).min(1.0),
                (self.target_size.1 * 1.5 / 141.0).min(1.0),
            )
        } else {
            (1.0, 1.0)
        };
        for frame in &mut ani.frames {
            frame.image_rate = (frame.image_rate.0 * scale.0, frame.image_rate.1 * scale.1);
        }
        let ani = Rc::new(ani);
        for f in &ani.frames {
            if !f.image.is_empty() && !self.images.contains_key(&f.image) {
                let img = a
                    .img(&f.image)
                    .ok_or_else(|| format!("missing status image {}", f.image))?;
                self.images.insert(f.image.clone(), img);
            }
        }
        for (i, f) in ani.frames.iter().enumerate() {
            if let Some(img) = self.images.get(&f.image) {
                if let Some(frame) = img.frames.get(f.image_index.max(0) as usize) {
                    let sprite = if f.image_rate == (1.0, 1.0) && f.image_rotate == 0.0 {
                        dnf_render::framebuffer::PreparedSprite::from_rgba(
                            frame.rgba.clone(),
                            frame.width as usize,
                            frame.height as usize,
                            (f.image_pos.0 + frame.x, f.image_pos.1 + frame.y),
                        )
                    } else {
                        dnf_render::framebuffer::prepare_sprite(
                            &frame.rgba,
                            (frame.width as usize, frame.height as usize),
                            (f.image_pos.0 + frame.x, f.image_pos.1 + frame.y),
                            f.image_rate,
                            f.image_rotate,
                        )
                    };
                    self.prepared.insert((Rc::as_ptr(&ani) as usize, i), sprite);
                }
            }
        }
        Ok(ani)
    }
    pub fn config(&self, kind: ActiveStatus, level: u8) -> StatusConfig {
        let (duration_ms, power, secondary) = self.table.parameters(kind as usize, level);
        StatusConfig {
            duration_ms,
            power,
            secondary,
        }
    }
    pub fn draw(
        &self,
        m: &Goblin,
        client: &ClientAssets,
        fb: &mut Framebuffer,
        camera: i32,
        behind: bool,
    ) {
        if m.hp <= 0 || m.statuses.is_empty() {
            return;
        }
        for state in m.statuses.iter() {
            if let Some(effects) = self.effects.get(&state.kind) {
                for (i, e) in effects
                    .iter()
                    .enumerate()
                    .filter(|(_, e)| e.behind == behind)
                {
                    let head = match e.anchor {
                        Anchor::Feet => m.y,
                        Anchor::Body => m.y - m.z,
                        Anchor::Head => m.y - m.z - 68.0,
                    };
                    let phase = state.elapsed_ms
                        + if state.kind == ActiveStatus::Hold {
                            i as f64 * 137.0
                        } else {
                            0.0
                        };
                    self.draw_ani(
                        &e.ani,
                        phase,
                        (
                            m.x.round() as i32 - camera + e.offset.0,
                            head.round() as i32 + e.offset.1,
                        ),
                        fb,
                        1.0,
                        e.hold_last,
                    );
                }
            }
            if !behind {
                if let Some(emitters) = self.emitters.get(&state.kind) {
                    for (emitter_index, e) in emitters.iter().enumerate() {
                        self.draw_emitter(
                            e,
                            state.elapsed_ms,
                            state.serial,
                            m,
                            fb,
                            camera,
                            emitter_index,
                        );
                    }
                }
            }
        }
        let _ = client; // sprites are prepared at load, no filesystem/IMG work per frame
    }
    pub fn draw_exits(&self, m: &Goblin, fb: &mut Framebuffer, camera: i32) {
        for &(kind, time, dead) in &m.status_exits {
            if kind == ActiveStatus::Freeze {
                if let Some(ani) = &self.freeze_exit[dead as usize] {
                    self.draw_ani(
                        ani,
                        time,
                        (m.x.round() as i32 - camera, (m.y - m.z).round() as i32),
                        fb,
                        1.0,
                        false,
                    );
                }
            }
            if kind == ActiveStatus::Stone {
                if let Some(e) = &self.stone_exit {
                    self.draw_emitter(e, time, 571, m, fb, camera, 0);
                }
            }
        }
    }
    fn draw_emitter(
        &self,
        e: &Emitter,
        time: f64,
        serial: u64,
        m: &Goblin,
        fb: &mut Framebuffer,
        camera: i32,
        index: usize,
    ) {
        let p = &e.script;
        if p.frequency == 0.0 {
            return;
        }
        let last = if p.frequency < 0.0 {
            p.max_count.saturating_sub(1) as u64
        } else {
            (time * p.frequency / 1000.0).floor() as u64
        };
        let first = if p.frequency < 0.0 {
            0
        } else {
            ((time - p.life_ms).max(0.0) * p.frequency / 1000.0).floor() as u64
        };
        for seq in first..=last {
            if p.max_count > 0 && seq >= p.max_count as u64 {
                break;
            }
            let birth = if p.frequency < 0.0 {
                0.0
            } else {
                seq as f64 * 1000.0 / p.frequency
            };
            if p.max_time_ms > 0.0 && birth > p.max_time_ms {
                break;
            }
            let age = time - birth;
            if !(0.0..p.life_ms).contains(&age) {
                continue;
            }
            let seed = serial.wrapping_mul(37) + seq * 103 + (index as u64) * 591;
            let mut pos = [0.0; 3];
            for axis in 0..3 {
                let noise = random(seed + axis as u64 * 971);
                pos[axis] = (noise - 0.5) * p.area[axis];
                let t = age as f32 / 1000.0;
                let change = (p.change_ms as f32 / 1000.0).max(0.001);
                let axis_script = p.axes[axis];
                let v = axis_script.first;
                pos[axis] += match axis_script.kind {
                    AxisKind::Stop => 0.0,
                    AxisKind::Line => {
                        let a = t.min(change);
                        v[0] * a
                            + 0.5 * (v[1] - v[0]) * a * a / change
                            + v[1] * (t - change).max(0.0)
                    }
                    AxisKind::Sin | AxisKind::Cos => {
                        let amp = v[0] + (v[1] - v[0]) * noise;
                        let period = (axis_script.second[0]
                            + (axis_script.second[1] - axis_script.second[0]) * noise)
                            .max(1.0)
                            / 1000.0;
                        let angle =
                            t / period * std::f32::consts::TAU + noise * std::f32::consts::TAU;
                        amp * if axis_script.kind == AxisKind::Sin {
                            angle.sin()
                        } else {
                            angle.cos()
                        }
                    }
                };
            }
            if p.circle {
                let angle = random(seed + 7621) * std::f32::consts::TAU;
                let xx = pos[0];
                pos[0] = xx * angle.cos();
                pos[1] += xx * angle.sin() * 0.3;
            }
            pos[2] -= 0.5 * p.gravity * (age as f32 / 1000.0).powi(2);
            if p.vanish_on_land && e.z + pos[2] < 0.0 {
                continue;
            }
            let alpha =
                (p.alpha[0] + (p.alpha[1] - p.alpha[0]) * (age as f32 / p.life_ms as f32)) / 255.0;
            let ani = &e.objects
                [(random(seed + 4903) * e.objects.len() as f32) as usize % e.objects.len()];
            self.draw_ani(
                ani,
                age,
                (
                    (m.x + pos[0]).round() as i32 - camera,
                    (m.y - m.z - e.z + pos[1] - pos[2]).round() as i32,
                ),
                fb,
                alpha,
                true,
            );
        }
    }
    fn draw_ani(
        &self,
        ani: &AnimFile,
        time: f64,
        anchor: (i32, i32),
        fb: &mut Framebuffer,
        alpha: f32,
        hold_last: bool,
    ) {
        let total: f64 = ani.frames.iter().map(|f| f.delay_ms.max(1) as f64).sum();
        if total <= 0.0 {
            return;
        }
        let mut phase = if ani.looping { time % total } else { time };
        for (i, f) in ani.frames.iter().enumerate() {
            let delay = f.delay_ms.max(1) as f64;
            if phase < delay || f.delay_ms >= 10000 || hold_last && i + 1 == ani.frames.len() {
                let Some(img) = self.images.get(&f.image) else {
                    return;
                };
                let Some(image) = img.frames.get(f.image_index.max(0) as usize) else {
                    return;
                };
                let tint = rgba(
                    f.rgba[0],
                    f.rgba[1],
                    f.rgba[2],
                    (f.rgba[3] as f32 * alpha.clamp(0.0, 1.0)) as u8,
                );
                let additive = f.events.iter().any(|(tag, val)| {
                    tag == "GRAPHIC EFFECT" && val.trim_matches('`') == "LINEARDODGE"
                });
                if let Some(sprite) = self.prepared.get(&(ani as *const AnimFile as usize, i)) {
                    fb.blit_prepared(sprite, anchor, tint, additive);
                } else if additive && f.image_rate == (1.0, 1.0) && f.image_rotate == 0.0 {
                    fb.blit_additive(
                        &image.rgba,
                        image.width as usize,
                        image.height as usize,
                        anchor.0 + f.image_pos.0 + image.x,
                        anchor.1 + f.image_pos.1 + image.y,
                        false,
                        tint,
                    );
                } else {
                    let transform = SpriteTransform {
                        anchor,
                        offset: (f.image_pos.0 + image.x, f.image_pos.1 + image.y),
                        rate: f.image_rate,
                        angle: f.image_rotate,
                        tint,
                    };
                    if additive {
                        fb.blit_transformed_additive(
                            &image.rgba,
                            (image.width as usize, image.height as usize),
                            transform,
                        );
                    } else {
                        fb.blit_transformed(
                            &image.rgba,
                            (image.width as usize, image.height as usize),
                            SpriteTransform {
                                anchor,
                                offset: (f.image_pos.0 + image.x, f.image_pos.1 + image.y),
                                rate: f.image_rate,
                                angle: f.image_rotate,
                                tint,
                            },
                        );
                    }
                }
                return;
            }
            phase -= delay;
        }
    }
}
fn random(seed: u64) -> f32 {
    let n = seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    ((n >> 32) as u32 % 10000) as f32 / 10000.0
}
pub fn body_tint(states: &ActiveStatuses) -> Color {
    if states.has(ActiveStatus::Stone) {
        let age = states.get(ActiveStatus::Stone).unwrap().elapsed_ms;
        let c = (255.0 - (age / 500.0).clamp(0.0, 1.0) * 155.0) as u8;
        rgba(c, c, c, 255)
    } else if states.has(ActiveStatus::Freeze) {
        rgba(100, 200, 255, 255)
    } else {
        Color::WHITE
    }
}
impl Game {
    pub fn status_loop_tags(&self) -> [Option<&'static str>; 4] {
        let mask = self
            .monsters()
            .filter(|m| m.hp > 0)
            .fold(0, |m, s| m | s.statuses.mask());
        let poison =
            mask & ((1 << ActiveStatus::Poison as u8) | (1 << ActiveStatus::Burn as u8)) != 0;
        [
            poison.then_some("CONDITION_POSION_DROP"),
            poison.then_some("CONDITION_POISON_POP"),
            (mask & (1 << ActiveStatus::Lightning as u8) != 0)
                .then_some("CONDITION_LIGTHNING_LOOP"),
            (mask & (1 << ActiveStatus::Bleeding as u8) != 0).then_some("BLOOD_DROP_LOOP"),
        ]
    }
    pub fn apply_monster_status(
        &mut self,
        target: Option<usize>,
        kind: ActiveStatus,
        level: u8,
    ) -> usize {
        let assets = self.goblin_assets.clone();
        let config = assets.status_assets.config(kind, level);
        let mut applied = 0;
        for i in 0..self.monster_count() {
            if target.is_some() && target != Some(i) {
                continue;
            }
            let m = self.monster_mut(i).unwrap();
            let had = m.statuses.has(kind);
            if m.hp > 0
                && !(kind == ActiveStatus::Sleep && m.reaction == crate::battle::Reaction::Grabbed)
                && m.effective_protection(&assets) != dnf_core::protection::Protection::Invincible
                && m.statuses.apply(kind, level, config, 0xFFFF)
            {
                if !had {
                    if let Some(tag) = sound_tag(kind) {
                        m.status_audio.push(tag);
                    }
                }
                applied += 1;
            }
        }
        applied
    }
}
pub fn sound_tag(kind: ActiveStatus) -> Option<&'static str> {
    use ActiveStatus::*;
    Some(match kind {
        Slow => "CONDITION_SLOW",
        Freeze => "FROZEN",
        Poison | Burn => "CONDITION_POSION",
        Stun => "CONDITION_STUN",
        Curse => "CONDITION_CURSE",
        Blind => "CONDITION_BLIND",
        Lightning => "CONDTION_LIGHT",
        Stone => "CONDITION_STONE",
        Sleep => "CONDITION_SLEEP",
        Bleeding => "CONDITION_BLOODING",
        Haste => "CONDITION_HASTE",
        Bless => "CONDITION_BLESS",
        Confuse => "CONDITION_CONFUSE",
        _ => return None,
    })
}
#[cfg(test)]
mod render_tests {
    use super::*;
    #[test]
    fn actual_status_frames_and_particles_are_identical_with_and_without_prepared_geometry() {
        let Some(dir) = dnf_data::locate_client_dir() else {
            return;
        };
        let data = crate::game::GameData::load(
            &dir,
            crate::game::DEFAULT_MAP,
            crate::character::Outfit::default_basic(),
        )
        .unwrap();
        let mut assets = Assets::load(&data.assets, data.goblin.status_assets.target_size).unwrap();
        for kind in ActiveStatus::ALL.into_iter().filter(|k| k.implemented()) {
            let mut g = data
                .build_game(dnf_core::KeyMap::official_default())
                .unwrap();
            g.goblin.max_hp = 50000;
            g.goblin.reset(410, 329, &data.goblin);
            let config = StatusConfig {
                duration_ms: 6000.0,
                ..assets.config(kind, 1)
            };
            g.goblin.statuses.apply(kind, 1, config, 1);
            for _ in 0..40 {
                g.begin_input();
                g.step();
            }
            let mut cached = Framebuffer::default();
            cached.clear(rgba(30, 60, 90, 255));
            let mut reference = cached.clone();
            assets.draw(&g.goblin, &data.assets, &mut cached, 0, true);
            assets.draw(&g.goblin, &data.assets, &mut cached, 0, false);
            let prepared = std::mem::take(&mut assets.prepared);
            assets.draw(&g.goblin, &data.assets, &mut reference, 0, true);
            assets.draw(&g.goblin, &data.assets, &mut reference, 0, false);
            assets.prepared = prepared;
            assert_eq!(cached.data, reference.data, "{kind:?}");
        }
    }
}
