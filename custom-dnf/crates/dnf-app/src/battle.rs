//! Official goblin script motions and the verified 58E920/58FC30 reaction subset.
//! Stationary target HP/damage and automatic respawn remain testing facilities.
use crate::assets::ClientAssets;
use dnf_core::{
    anim::{AnimPlayer, Clip, HOLD_DELAY_MS},
    feedback::{recovery_ms, HitStop},
    jump::launch_velocity,
};
use dnf_data::{
    ani::AnimFile,
    combat::{AttackInfo, HitInfo},
    mob::MobFile,
};
use dnf_render::framebuffer::{rgba, Framebuffer, SpriteTransform};
use std::{collections::HashMap, rc::Rc};

#[derive(Clone, Copy, Debug)]
pub struct Box3 {
    pub min: [f32; 3],
    pub max: [f32; 3],
}
impl Box3 {
    pub fn translated(b: [i32; 6], origin: [f32; 3], facing: i32) -> Self {
        let x = if facing < 0 { -(b[0] + b[3]) } else { b[0] };
        let min = [
            origin[0] + x as f32,
            origin[1] + b[1] as f32,
            origin[2] + b[2] as f32,
        ];
        Self {
            min,
            max: [
                min[0] + b[3] as f32,
                min[1] + b[4] as f32,
                min[2] + b[5] as f32,
            ],
        }
    }
    pub fn intersects(self, b: Self) -> bool {
        (0..3).all(|i| self.min[i] < b.max[i] && b.min[i] < self.max[i])
    }
    pub fn intersection(self, b: Self) -> Option<Self> {
        self.intersects(b).then(|| Self {
            min: std::array::from_fn(|i| self.min[i].max(b.min[i])),
            max: std::array::from_fn(|i| self.max[i].min(b.max[i])),
        })
    }
    pub fn center(self) -> [f32; 3] {
        std::array::from_fn(|i| (self.min[i] + self.max[i]) * 0.5)
    }
}
const MOTIONS: [(&str, &str); 9] = [
    ("Stay", "waiting motion"),
    ("Move", "move motion"),
    ("Sit", "sit motion"),
    ("Damage1", "damage motion 1"),
    ("Damage2", "damage motion 2"),
    ("Down", "down motion"),
    ("Overturn", "overturn motion"),
    ("Attack1", "attack motion 0"),
    ("Attack2", "attack motion 1"),
];
pub struct MonsterLayer {
    pub path: String,
    pub layer: i32,
    pub motions: HashMap<&'static str, Rc<AnimFile>>,
}
pub struct GoblinAssets {
    pub definition: MobFile,
    pub motions: HashMap<&'static str, Rc<AnimFile>>,
    pub layers: Vec<MonsterLayer>,
    pub effects: HashMap<String, Rc<AnimFile>>,
    pub weight: f32,
    pub hit_recovery: i32,
    pub rise_hold: usize,
    pub fall_hold: usize,
    pub land_frame: usize,
}
fn motion_set(
    assets: &ClientAssets,
    script: &MobFile,
    directory: &str,
) -> Result<HashMap<&'static str, Rc<AnimFile>>, String> {
    let mut out = HashMap::new();
    for (name, tag) in MOTIONS {
        let Some(relative) = script.motion(tag) else {
            continue;
        };
        let path = format!("{directory}/{relative}");
        let ani = assets
            .ani(&path)
            .ok_or_else(|| format!("missing scripted motion {path}"))?;
        for f in &ani.frames {
            if assets.img(&f.image).is_none() {
                return Err(format!("missing {}", f.image));
            }
        }
        out.insert(name, ani);
    }
    Ok(out)
}
impl GoblinAssets {
    pub fn load(assets: &ClientAssets) -> Result<Self, String> {
        let text = assets
            .pvf
            .read_text("monster/goblin/goblin.mob")
            .map_err(|e| e.to_string())?;
        let definition = dnf_data::mob::parse(&text);
        let motions = motion_set(assets, &definition, "monster/goblin")?;
        for name in ["Stay", "Damage1", "Damage2", "Down", "Sit"] {
            if !motions.contains_key(name) {
                return Err(format!("goblin missing {name}"));
            }
        }
        let holds = motions["Down"]
            .frames
            .iter()
            .enumerate()
            .filter(|(_, f)| f.delay_ms >= HOLD_DELAY_MS)
            .map(|(i, _)| i)
            .collect::<Vec<_>>();
        if holds.len() != 2 || holds[1] + 1 >= motions["Down"].frames.len() {
            return Err("goblin Down hold gates mismatch".into());
        }
        let (rise_hold, fall_hold, land_frame) = (holds[0], holds[1], holds[1] + 1);
        let mut layers = Vec::new();
        // CNGoblinGoblin ctor4A5F10 -> 59A990; .equ supplies each actual path/layer.
        for path in [
            "equipment/monster/goblin/gcc1.equ",
            "equipment/monster/goblin/weapon01.equ",
        ] {
            let text = assets.pvf.read_text(path).map_err(|e| e.to_string())?;
            let script = dnf_data::mob::parse(&text);
            let layer = script
                .at_level("layer", 1)
                .ok_or("monster equipment missing LAYER")? as i32;
            layers.push(MonsterLayer {
                path: path.into(),
                layer,
                motions: motion_set(assets, &script, "equipment/monster/goblin")?,
            });
        }
        layers.sort_by_key(|l| l.layer);
        let mut effects = HashMap::new();
        for size in ["small", "large"] {
            for direction in 1..=3 {
                let name = format!("slash{size}{direction}");
                let path = format!("common/hiteffect/animation/{name}.ani");
                let ani = assets.ani(&path).ok_or_else(|| format!("missing {path}"))?;
                for f in &ani.frames {
                    if assets.img(&f.image).is_none() {
                        return Err(format!("missing {}", f.image));
                    }
                }
                effects.insert(name, ani);
            }
        }
        let weight = definition
            .at_level("weight", 1)
            .ok_or("goblin missing weight")? as f32;
        let hit_recovery = definition
            .at_level("hit recovery", 1)
            .ok_or("goblin missing hit recovery")? as i32;
        Ok(Self {
            definition,
            motions,
            layers,
            effects,
            weight,
            hit_recovery,
            rise_hold,
            fall_hold,
            land_frame,
        })
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reaction {
    Idle,
    Hurt,
    Air,
    Down,
    GettingUp,
    Dead,
}
struct Impact {
    anim: AnimPlayer,
    name: String,
    position: [f32; 3],
    facing: i32,
}
pub struct Hit {
    pub damage: i32,
    pub facing: i32,
    pub upper: bool,
    pub point: [f32; 3],
}
pub struct Goblin {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub hp: i32,
    pub max_hp: i32,
    pub hits: u64,
    pub reaction: Reaction,
    pub last_damage: i32,
    pub damage_ticks: u32,
    pub stop: HitStop,
    spawn: (f32, f32),
    vx: f32,
    vz: f32,
    ax: f32,
    recovery_left: f64,
    anim: AnimPlayer,
    impact: Option<Impact>,
    variant: usize,
    bounce: bool,
    peak: f32,
    random: u32,
    corpse_ms: f64,
    corpse_duration: f64,
    respawn_ms: f64,
}
impl Goblin {
    pub const MAX_HP: i32 = 300;
    pub fn new(x: i32, y: i32, data: &GoblinAssets) -> Self {
        Self {
            x: x as f32,
            y: y as f32,
            z: 0.0,
            hp: Self::MAX_HP,
            max_hp: Self::MAX_HP,
            hits: 0,
            reaction: Reaction::Idle,
            last_damage: 0,
            damage_ticks: 0,
            stop: HitStop::default(),
            spawn: (x as f32, y as f32),
            vx: 0.0,
            vz: 0.0,
            ax: 0.0,
            recovery_left: 0.0,
            anim: AnimPlayer::new(clip(data, "Stay"), 100.0),
            impact: None,
            variant: 0,
            bounce: false,
            peak: 0.0,
            random: 1,
            corpse_ms: 0.0,
            corpse_duration: 200.0,
            respawn_ms: 0.0,
        }
    }
    pub fn reset(&mut self, x: i32, y: i32, data: &GoblinAssets) {
        let max_hp = self.max_hp;
        *self = Self::new(x, y, data);
        self.max_hp = max_hp;
        self.hp = max_hp;
    }
    fn rand(&mut self) -> u32 {
        self.random = self.random.wrapping_mul(214013).wrapping_add(2531011);
        (self.random >> 16) & 0x7fff
    }
    pub fn motion(&self) -> &'static str {
        match self.reaction {
            Reaction::Idle => "Stay",
            Reaction::Hurt => {
                if self.variant == 0 {
                    "Damage1"
                } else {
                    "Damage2"
                }
            }
            Reaction::Air | Reaction::Down | Reaction::Dead => "Down",
            Reaction::GettingUp => "Sit",
        }
    }
    pub fn frame(&self) -> usize {
        self.anim.frame()
    }
    pub fn is_bouncing(&self) -> bool {
        self.bounce && self.reaction == Reaction::Air
    }
    fn set_motion(&mut self, data: &GoblinAssets) {
        self.anim.set_clip(clip(data, self.motion()), 100.0, true);
    }
    fn damage_type<'a>(&self, data: &'a GoblinAssets) -> &'a str {
        data.motions[self.motion()]
            .frames
            .get(self.anim.frame())
            .map_or("NORMAL", |f| f.damage_type.as_str())
    }
    pub fn hurt_boxes(&self, data: &GoblinAssets) -> Vec<Box3> {
        if self.hp <= 0 || self.damage_type(data) == "UNBREAKABLE" {
            return Vec::new();
        }
        data.motions[self.motion()]
            .frames
            .get(self.anim.frame())
            .into_iter()
            .flat_map(|f| &f.damage_boxes)
            .map(|&b| Box3::translated(b, [self.x, self.y, self.z], -1))
            .collect()
    }
    fn launch(&mut self, speed: i32, height: i32, facing: i32) {
        let speed = self.range((speed as f64 * 0.8) as i32, (speed as f64 * 1.2) as i32);
        let height = self.range((height as f64 * 0.9) as i32, (height as f64 * 1.1) as i32);
        self.vx = speed as f32 * facing as f32;
        self.ax = 0.0;
        self.vz = (2.0 * 1500.0 * height.max(0) as f32).sqrt();
        self.peak = self.z;
        self.reaction = Reaction::Air;
    }
    fn range(&mut self, lo: i32, hi: i32) -> i32 {
        lo + (hi - lo) * (self.rand() % 1000) as i32 / 999
    }
    pub fn hit(
        &mut self,
        data: &GoblinAssets,
        info: &AttackInfo,
        feedback: &HitInfo,
        hit: Hit,
    ) -> bool {
        let Hit {
            damage,
            facing,
            upper,
            point,
        } = hit;
        if self.hp <= 0 || self.damage_type(data) == "UNBREAKABLE" {
            return false;
        }
        let armor = self.damage_type(data) == "SUPERARMOR";
        self.hp = (self.hp - damage).max(0);
        self.hits += 1;
        self.last_damage = damage;
        self.damage_ticks = 45;
        self.stop.apply(feedback.stop_ms.max(0) as f64);
        let big = self.rand() % 2 == 0;
        let direction = match info.token("attack direction") {
            Some("hit lift up") => 3,
            Some("hit horizon") => 2,
            _ => 1,
        };
        let name = format!("slash{}{direction}", if big { "large" } else { "small" });
        self.impact = Some(Impact {
            anim: AnimPlayer::new(Clip::new(&name, data.effects[&name].delays(), false), 100.0),
            name,
            position: point,
            facing,
        });
        if self.hp == 0 {
            self.respawn_ms = 3000.0;
        }
        if armor && self.hp > 0 {
            return true;
        }
        if info.token("damage reaction") == Some("none") && self.hp > 0 {
            return true;
        }
        let down = upper
            || info.token("damage reaction") == Some("down")
            || self.hp == 0
            || matches!(self.reaction, Reaction::Air | Reaction::Down);
        if down {
            let was_down = matches!(self.reaction, Reaction::Air | Reaction::Down);
            let was_bouncing = self.bounce;
            let lift = info.number("lift up").unwrap_or(0);
            let push = info.number("push aside").unwrap_or(0);
            let horizontal = (launch_velocity(push as f32, data.weight, 100.0) as i32) / 2;
            let vertical = launch_velocity(lift as f32, data.weight, 100.0) as i32;
            let mut height = vertical * vertical / 3000;
            if was_down && was_bouncing {
                height /= 4;
            }
            if was_down && height == 0 && lift > 0 {
                height = 1;
            }
            self.bounce = was_down;
            self.launch(horizontal, height, facing);
            self.set_motion(data);
            if was_down {
                self.anim.set_frame(data.rise_hold);
            }
        } else if info.token("damage reaction") != Some("none") {
            self.variant = if self.reaction == Reaction::Hurt {
                1 - self.variant
            } else {
                (self.rand() % 2) as usize
            };
            self.reaction = Reaction::Hurt;
            self.z = 0.0;
            self.vz = 0.0;
            let mut rate = feedback.recovery_rate;
            if info.token("attack type") == Some("magic") {
                rate *= 0.7;
            }
            let special = info.number("knuck back") == Some(1);
            let burn = feedback.kind == "burn";
            self.recovery_left =
                recovery_ms(data.hit_recovery, (rate * 100.0) as i32, special) as f64;
            let duration = if special {
                333
            } else if burn {
                1
            } else {
                150
            }
            .min(self.recovery_left as i32);
            let speed = if special {
                800
            } else if burn {
                1
            } else {
                400
            };
            self.vx = speed as f32 * facing as f32;
            self.ax = (-1000 * speed / duration.max(1)) as f32 * facing as f32;
            self.set_motion(data);
        }
        true
    }
    pub fn step(&mut self, data: &GoblinAssets, walk_x: (i32, i32), dt_ms: f64) {
        self.damage_ticks = self.damage_ticks.saturating_sub(1);
        if let Some(impact) = &mut self.impact {
            impact.anim.advance(dt_ms);
            if impact.anim.finished() {
                self.impact = None;
            }
        }
        if self.hp == 0 {
            self.respawn_ms -= dt_ms;
            if self.respawn_ms <= 0.0 {
                let hits = self.hits;
                self.reset(self.spawn.0 as i32, self.spawn.1 as i32, data);
                self.hits = hits;
                return;
            }
        }
        let active = self.stop.active_ms(dt_ms);
        if active == 0.0 {
            return;
        }
        let dt = active as f32 / 1000.0;
        let duration = if self.ax != 0.0 {
            dt.min((-self.vx / self.ax).max(0.0))
        } else {
            dt
        };
        let end = self.vx + self.ax * duration;
        self.x = (self.x + (self.vx + end) * 0.5 * duration)
            .clamp((walk_x.0 + 20) as f32, (walk_x.1 - 20) as f32);
        self.vx = if duration < dt { 0.0 } else { end };
        self.anim.advance(active);
        match self.reaction {
            Reaction::Hurt => {
                self.recovery_left -= active;
                if self.recovery_left <= 0.0 {
                    self.reaction = Reaction::Idle;
                    self.vx = 0.0;
                    self.ax = 0.0;
                    self.set_motion(data);
                }
            }
            Reaction::Air => {
                self.z += self.vz * dt - 750.0 * dt * dt;
                self.vz -= 1500.0 * dt;
                self.peak = self.peak.max(self.z.round());
                // Initial knockdown pins frame2 throughout flight; ONLY bounce has2->3 apex gate.
                if self.bounce && self.vz <= 0.0 && self.anim.frame() == data.rise_hold {
                    self.anim.set_frame(data.fall_hold);
                }
                if self.z <= 0.0 && self.vz < 0.0 {
                    self.z = 0.0;
                    if self.hp == 0 {
                        self.reaction = Reaction::Dead;
                        self.vx = 0.0;
                        self.vz = 0.0;
                        self.corpse_ms = 0.0;
                        self.random = Self::MAX_HP as u32;
                        self.corpse_duration = (100 + 150 * (self.rand() % 101) / 100) as f64;
                        self.anim.set_frame(data.land_frame);
                    } else if !self.bounce {
                        self.bounce = true;
                        let speed = self.vx.abs() as i32;
                        let direction = if self.vx < 0.0 { -1 } else { 1 };
                        self.launch(speed, self.peak as i32 / 5, direction);
                        self.anim.set_frame(data.rise_hold);
                    } else {
                        self.reaction = Reaction::Down;
                        self.vx = 0.0;
                        self.vz = 0.0;
                        self.anim.set_frame(data.land_frame);
                    }
                }
            }
            Reaction::Down => {
                if self.anim.finished() {
                    self.reaction = Reaction::GettingUp;
                    self.set_motion(data);
                }
            }
            Reaction::GettingUp => {
                if self.anim.finished() {
                    self.reaction = Reaction::Idle;
                    self.set_motion(data);
                }
            }
            Reaction::Dead => {
                self.corpse_ms += active;
            }
            Reaction::Idle => {}
        }
    }
    pub fn draw(
        &self,
        data: &GoblinAssets,
        assets: &ClientAssets,
        fb: &mut Framebuffer,
        camera: i32,
    ) {
        let x = self.x.round() as i32;
        let y = self.y.round() as i32;
        let z = self.z.round() as i32;
        let alpha = if self.reaction == Reaction::Dead {
            ((self.corpse_duration - self.corpse_ms) / 100.0).clamp(0.0, 1.0) as f32
        } else {
            1.0
        };
        if alpha > 0.0 {
            if data.motions[self.motion()].shadow {
                fb.fill_ellipse(x - camera, y, 18, 5, rgba(0, 0, 0, (90.0 * alpha) as u8));
            }
            for layer in data.layers.iter().filter(|l| l.layer < 0) {
                if let Some(ani) = layer.motions.get(self.motion()) {
                    draw_frame(
                        ani,
                        self.frame(),
                        assets,
                        fb,
                        (x - camera, y - z),
                        -1,
                        alpha,
                    );
                }
            }
            draw_frame(
                &data.motions[self.motion()],
                self.frame(),
                assets,
                fb,
                (x - camera, y - z),
                -1,
                alpha,
            );
            for layer in data.layers.iter().filter(|l| l.layer >= 0) {
                if let Some(ani) = layer.motions.get(self.motion()) {
                    draw_frame(
                        ani,
                        self.frame(),
                        assets,
                        fb,
                        (x - camera, y - z),
                        -1,
                        alpha,
                    );
                }
            }
        }
        if let Some(effect) = &self.impact {
            let p = effect.position;
            draw_frame(
                &data.effects[&effect.name],
                effect.anim.frame(),
                assets,
                fb,
                (p[0].round() as i32 - camera, (p[1] - p[2]).round() as i32),
                effect.facing,
                1.0,
            );
        }
        if self.hp > 0 {
            let top = y - z - 86;
            fb.fill_rect(x - camera - 25, top, 50, 4, rgba(15, 15, 15, 220));
            fb.fill_rect(
                x - camera - 25,
                top,
                50 * self.hp / self.max_hp.max(1),
                4,
                rgba(230, 75, 75, 255),
            );
        }
    }
}
fn clip(data: &GoblinAssets, name: &str) -> Clip {
    let ani = &data.motions[name];
    Clip::new(name, ani.delays(), ani.looping)
}
fn draw_frame(
    ani: &AnimFile,
    frame: usize,
    assets: &ClientAssets,
    fb: &mut Framebuffer,
    anchor: (i32, i32),
    facing: i32,
    alpha: f32,
) {
    if let Some(f) = ani.frames.get(frame) {
        if let Some(img) = assets.img(&f.image) {
            if let Some(image) = img.frames.get(f.image_index.max(0) as usize) {
                let c = f.rgba;
                fb.blit_transformed(
                    &image.rgba,
                    (image.width as usize, image.height as usize),
                    SpriteTransform {
                        anchor,
                        offset: (f.image_pos.0 + image.x, f.image_pos.1 + image.y),
                        rate: (f.image_rate.0 * facing as f32, f.image_rate.1),
                        angle: f.image_rotate,
                        tint: rgba(c[0], c[1], c[2], (c[3] as f32 * alpha) as u8),
                    },
                );
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn assets() -> Option<(ClientAssets, GoblinAssets)> {
        let dir = dnf_data::locate_client_dir()?;
        let assets = ClientAssets::load(&dir).unwrap();
        let goblin = GoblinAssets::load(&assets).unwrap();
        Some((assets, goblin))
    }
    fn info() -> HitInfo {
        HitInfo {
            kind: "cut".into(),
            blood: true,
            stop_ms: 100,
            recovery_rate: 1.0,
        }
    }
    fn hit(g: &mut Goblin, data: &GoblinAssets, raw: &str, upper: bool) {
        assert!(g.hit(
            data,
            &dnf_data::combat::parse_attack_info(raw),
            &info(),
            Hit {
                damage: 18,
                facing: 1,
                upper,
                point: [g.x, g.y, 30.0]
            }
        ));
    }
    #[test]
    fn goblin_body_and_both_equipment_motion_tables_are_script_driven() {
        let Some((_, data)) = assets() else {
            return;
        };
        assert_eq!((data.weight, data.hit_recovery), (45000.0, 500));
        assert_eq!(data.layers.len(), 2);
        assert_eq!((data.rise_hold, data.fall_hold, data.land_frame), (2, 3, 4));
        for layer in &data.layers {
            assert_eq!(layer.layer, 100);
            for name in [
                "Stay", "Move", "Damage1", "Damage2", "Down", "Sit", "Overturn", "Attack1",
            ] {
                assert!(layer.motions.contains_key(name), "{} {name}", layer.path);
            }
        }
        assert!((data.motions["Overturn"].frames[0].image_rotate - 3.140022).abs() < 1e-5);
        assert_eq!(data.motions["Down"].frames[4].delay_ms, 500);
        assert_eq!(data.motions["Sit"].frames[0].delay_ms, 150);
    }
    #[test]
    fn ordinary_ground_hit_has_independent_stop_and_375ms_stun_and_30px_recoil() {
        let Some((_, data)) = assets() else {
            return;
        };
        let mut g = Goblin::new(300, 329, &data);
        hit(
            &mut g,
            &data,
            "[damage reaction] `[damage]`\n[lift up] 30\n[push aside] 30",
            false,
        );
        let variant = g.motion();
        assert!(matches!(variant, "Damage1" | "Damage2"));
        assert_eq!(g.z, 0.0);
        for _ in 0..20 {
            g.step(&data, (16, 1104), 5.0);
        }
        assert_eq!(g.x, 300.0);
        assert_eq!(g.recovery_left, 375.0);
        for _ in 0..31 {
            g.step(&data, (16, 1104), 5.0);
        }
        assert!((g.x - 330.01).abs() < 0.05);
        assert_eq!(g.z, 0.0);
        hit(&mut g, &data, "[damage reaction] `[damage]`", false);
        assert_ne!(g.motion(), variant);
        for _ in 0..95 {
            g.step(&data, (16, 1104), 5.0);
        }
        assert_eq!(g.reaction, Reaction::Idle);
    }
    #[test]
    fn knockdown_runs_script_holds_bounce_landing_and_sit_recovery() {
        let Some((_, data)) = assets() else {
            return;
        };
        let mut g = Goblin::new(300, 329, &data);
        hit(
            &mut g,
            &data,
            "[damage reaction] `[down]`\n[lift up] 300\n[push aside] 100",
            true,
        );
        assert_eq!(g.motion(), "Down");
        assert_eq!(g.frame(), 0);
        let mut stages = std::collections::BTreeSet::new();
        let mut apex = 0.0f32;
        for _ in 0..260 {
            g.step(&data, (16, 1104), 5.0);
            apex = apex.max(g.z);
            if !g.bounce && g.vz < 0.0 && g.z > 0.0 {
                assert_eq!(g.frame(), data.rise_hold);
                stages.insert("first_descent");
            }
            if g.bounce && g.vz <= 0.0 && g.z > 0.0 {
                assert_eq!(g.frame(), data.fall_hold);
                stages.insert("bounce_descent");
            }
            if g.reaction == Reaction::Down {
                assert_eq!(g.frame(), data.land_frame);
                stages.insert("land");
            }
            if g.reaction == Reaction::GettingUp {
                assert_eq!(g.motion(), "Sit");
                stages.insert("sit");
            }
        }
        assert!((67.0..=82.0).contains(&apex));
        for _ in 0..100 {
            g.step(&data, (16, 1104), 5.0);
            if g.reaction == Reaction::GettingUp {
                stages.insert("sit");
            }
        }
        assert_eq!(g.reaction, Reaction::Idle);
        assert_eq!(
            stages,
            ["first_descent", "bounce_descent", "land", "sit"]
                .into_iter()
                .collect()
        );
    }
    #[test]
    fn boxes_mirror_at_anchor_and_require_three_axis_overlap() {
        let a = Box3::translated([10, -5, 20, 30, 10, 50], [100.0, 300.0, 0.0], -1);
        assert_eq!(a.min, [60.0, 295.0, 20.0]);
        assert_eq!(a.max, [90.0, 305.0, 70.0]);
        assert!(a.intersects(Box3::translated([0, 0, 0, 5, 5, 5], [80.0, 300.0, 30.0], 1)));
        for p in [
            [90.0, 300.0, 30.0],
            [80.0, 305.0, 30.0],
            [80.0, 300.0, 70.0],
        ] {
            assert!(!a.intersects(Box3::translated([0, 0, 0, 5, 5, 5], p, 1)));
        }
    }
}
