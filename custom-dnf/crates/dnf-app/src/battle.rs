//! Official goblin script motions and the verified 58E920/58FC30 reaction subset.
//! Stationary target HP/damage and automatic respawn remain testing facilities.
use crate::assets::ClientAssets;
use dnf_core::protection::{DamageOutcome, Protection, ProtectionState};
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
use std::cell::RefCell;
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
    pub status_assets: Rc<crate::active_status::Assets>,
    outlines: RefCell<HashMap<(&'static str, usize), Rc<OutlineMask>>>,
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
        let frame = &motions["Damage2"].frames[0];
        let image = assets
            .img(&frame.image)
            .ok_or("missing status sizing image")?;
        let image = image
            .frames
            .get(frame.image_index.max(0) as usize)
            .ok_or("invalid status sizing frame")?;
        let status_size = (
            image.width as f32 * frame.image_rate.0,
            image.height as f32 * frame.image_rate.1,
        );
        Ok(Self {
            status_assets: Rc::new(crate::active_status::Assets::load(assets, status_size)?),
            outlines: RefCell::new(HashMap::new()),
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
    fn outline_mask(
        &self,
        assets: &ClientAssets,
        motion: &'static str,
        frame: usize,
        alpha: f32,
    ) -> Rc<OutlineMask> {
        if alpha == 1.0 {
            if let Some(mask) = self.outlines.borrow().get(&(motion, frame)) {
                return mask.clone();
            }
        }
        let mut image = Framebuffer::new(256, 256);
        let anchor = (128, 220);
        for layer in self.layers.iter().filter(|l| l.layer < 0) {
            if let Some(ani) = layer.motions.get(motion) {
                draw_frame(ani, frame, assets, &mut image, anchor, -1, alpha);
            }
        }
        draw_frame(
            &self.motions[motion],
            frame,
            assets,
            &mut image,
            anchor,
            -1,
            alpha,
        );
        for layer in self.layers.iter().filter(|l| l.layer >= 0) {
            if let Some(ani) = layer.motions.get(motion) {
                draw_frame(ani, frame, assets, &mut image, anchor, -1, alpha);
            }
        }
        let (mut x0, mut y0, mut x1, mut y1) = (256, 256, 0, 0);
        for (i, p) in image.data.chunks_exact(4).enumerate() {
            if p[3] != 0 {
                x0 = x0.min(i % 256);
                x1 = x1.max(i % 256 + 1);
                y0 = y0.min(i / 256);
                y1 = y1.max(i / 256 + 1);
            }
        }
        let w = x1.saturating_sub(x0);
        let h = y1.saturating_sub(y0);
        let mut pixels = Vec::with_capacity(w * h * 4);
        if w > 0 && h > 0 {
            for y in y0..y1 {
                pixels.extend_from_slice(&image.data[(y * 256 + x0) * 4..(y * 256 + x1) * 4]);
            }
        }
        let mask = Rc::new(OutlineMask {
            width: w,
            height: h,
            offset: (x0 as i32 - 128, y0 as i32 - 220),
            pixels,
        });
        if alpha == 1.0 {
            self.outlines
                .borrow_mut()
                .insert((motion, frame), mask.clone());
        }
        mask
    }
}
struct OutlineMask {
    width: usize,
    height: usize,
    offset: (i32, i32),
    pixels: Vec<u8>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reaction {
    Idle,
    Hurt,
    Air,
    Down,
    GettingUp,
    Grabbed,
    Stunned,
    Sleeping,
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
    pub status_audio: Vec<&'static str>,
    pub status_exits: Vec<(dnf_core::active_status::ActiveStatus, f64, bool)>,
    pub statuses: dnf_core::active_status::ActiveStatuses,
    pub status_damage: i32,
    pub status_splash: i32,
    pub status_feedback: i32,
    status_dot_serial: [u64; 3],
    status_dot_pending: [i32; 3],
    status_dot_next: [f64; 3],
    pub status_resistance: [i32; 17],
    pub elemental_resistance: [i32; 4],
    pub demo_enabled: bool,
    pub demo_target: Option<(f32, f32)>,
    demo_moving: bool,
    demo_direction: i32,
    demo_clock: f64,
    pub grab: dnf_core::grab::GrabState,
    pub defense: i32,
    pub weight: f32,
    pub hit_recovery: i32,
    pub protection: ProtectionState,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub hp: i32,
    pub max_hp: i32,
    pub hits: u64,
    pub reaction: Reaction,
    pub last_damage: i32,
    pub damage_feedback_ms: f64,
    pub stop: HitStop,
    spawn: (f32, f32),
    vx: f32,
    vz: f32,
    ax: f32,
    recovery_left: f64,
    grab_down_ms: f64,
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
            status_exits: Vec::new(),
            status_audio: Vec::new(),
            statuses: Default::default(),
            status_damage: 0,
            status_splash: 0,
            status_feedback: 0,
            status_dot_serial: [0; 3],
            status_dot_pending: [0; 3],
            status_dot_next: [500.0; 3],
            status_resistance: std::array::from_fn(|i| {
                data.definition
                    .at_level(
                        &format!(
                            "{} resistance",
                            dnf_core::active_status::ActiveStatus::ALL[i].token()
                        ),
                        1,
                    )
                    .unwrap_or(0.0) as i32
            }),
            elemental_resistance: [0; 4],
            demo_enabled: false,
            demo_target: None,
            demo_moving: false,
            demo_direction: 1,
            demo_clock: 0.0,
            grab: dnf_core::grab::GrabState::new(data.definition.grabable()),
            defense: 0,
            weight: data.weight,
            hit_recovery: data.hit_recovery,
            protection: ProtectionState::default(),
            x: x as f32,
            y: y as f32,
            z: 0.0,
            hp: Self::MAX_HP,
            max_hp: Self::MAX_HP,
            hits: 0,
            reaction: Reaction::Idle,
            last_damage: 0,
            damage_feedback_ms: 0.0,
            stop: HitStop::default(),
            spawn: (x as f32, y as f32),
            vx: 0.0,
            vz: 0.0,
            ax: 0.0,
            recovery_left: 0.0,
            grab_down_ms: 0.0,
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
        let grabable = self.grab.grabable;
        let (defense, weight, hit_recovery, protection) = (
            self.defense,
            self.weight,
            self.hit_recovery,
            self.protection.clone(),
        );
        *self = Self::new(x, y, data);
        self.grab.grabable = grabable;
        self.max_hp = max_hp;
        self.hp = max_hp;
        self.defense = defense;
        self.weight = weight;
        self.hit_recovery = hit_recovery;
        self.protection = protection;
    }
    pub fn set_hp(&mut self, hp: i32, data: &GoblinAssets) {
        let hp = hp.clamp(0, self.max_hp);
        if hp > 0 && self.hp == 0 {
            self.reset(self.x.round() as i32, self.y.round() as i32, data);
        }
        self.hp = hp;
        if hp == 0 {
            self.clear_active_statuses(true);
            self.reaction = Reaction::Dead;
            self.z = 0.0;
            self.vx = 0.0;
            self.vz = 0.0;
            self.corpse_ms = 0.0;
            self.respawn_ms = 3000.0;
            self.set_motion(data);
            self.anim.set_frame(data.land_frame);
        }
    }
    /// Training damage reduction, intentionally separate from the full official formula.
    pub fn reduced_damage(&self, damage: i32) -> i32 {
        let defense = (self.defense.max(0) as f64 * self.statuses.stat_rate()) as i64
            + self.statuses.stone_vitality() as i64 * 2;
        ((damage.max(1) as i64 * 1000) / (1000 + defense)).max(1) as i32
    }
    pub fn effective_protection(&self, data: &GoblinAssets) -> Protection {
        self.protection_state(data).effective()
    }
    pub fn clear_active_statuses(&mut self, death: bool) {
        if !self.statuses.is_empty() {
            self.status_audio.push("CONDITION_REMOVE");
        }
        for kind in [
            dnf_core::active_status::ActiveStatus::Freeze,
            dnf_core::active_status::ActiveStatus::Stone,
        ] {
            if self.statuses.has(kind) {
                self.status_exits.push((kind, 0.0, death));
            }
        }
        if self
            .statuses
            .has(dnf_core::active_status::ActiveStatus::Stone)
        {
            self.status_audio.push("STONE_DIE");
        }
        if death
            && self
                .statuses
                .has(dnf_core::active_status::ActiveStatus::Freeze)
        {
            self.status_audio.push("FROZEN_DIE");
        }
        self.statuses.clear();
    }
    pub fn apply_attack_statuses(&mut self, data: &GoblinAssets, info: &AttackInfo) {
        use dnf_core::active_status::ActiveStatus;
        if self.hp <= 0 || self.effective_protection(data) == Protection::Invincible {
            return;
        }
        for (token, chance, level) in info.active_statuses() {
            let Some(kind) = ActiveStatus::from_token(token) else {
                continue;
            };
            if kind == ActiveStatus::Sleep && self.reaction == Reaction::Grabbed {
                continue;
            }
            let resistance = (self.status_resistance[kind as usize]
                + self.statuses.resistance_delta(kind))
            .clamp(0, 100);
            let rate = chance * (100 - resistance) as f32 / 100.0;
            if rate * 100.0 > (self.rand() % 10000) as f32 {
                let had = self.statuses.has(kind);
                if self
                    .statuses
                    .apply(kind, level, data.status_assets.config(kind, level), 0)
                    && !had
                {
                    if let Some(tag) = crate::active_status::sound_tag(kind) {
                        self.status_audio.push(tag);
                    }
                }
            }
        }
    }
    pub fn acquire_grab(&mut self, owner: u64, data: &GoblinAssets) -> bool {
        if self.statuses.blocks_grab() {
            return false;
        }
        let state = self.effective_protection(data);
        if !self.grab.acquire(owner, self.hp > 0, state) {
            return false;
        }
        self.reaction = Reaction::Grabbed;
        self.vx = 0.0;
        self.vz = 0.0;
        self.ax = 0.0;
        self.stop = HitStop::default();
        self.set_motion(data);
        true
    }
    /// Absorb/burst on the bound target bypasses ordinary hurt animation and
    /// slash feedback. Invincibility still rejects damage; ownership is separate.
    pub fn grab_damage(&mut self, data: &GoblinAssets, damage: i32) -> bool {
        if self
            .protection_state(data)
            .apply_damage(&mut self.hp, damage)
            == DamageOutcome::Rejected
        {
            return false;
        }
        self.hits += 1;
        let shock = self.statuses.on_hit();
        if shock > 0 && self.hp > 0 {
            let before = self.hp;
            self.hp = (self.hp - shock).max(0);
            self.status_damage += before - self.hp;
            self.status_feedback += before - self.hp;
        }
        self.last_damage = damage;
        self.damage_feedback_ms = 750.0;
        if self.hp == 0 {
            self.respawn_ms = 3000.0;
            self.clear_active_statuses(true);
        }
        true
    }
    pub fn release_grab(
        &mut self,
        owner: u64,
        data: &GoblinAssets,
        blast: Option<(&AttackInfo, i32)>,
        down_ms: f64,
    ) {
        if self.grab.owner() != Some(owner) {
            return;
        }
        self.grab.release(owner);
        if let Some((info, facing)) = blast {
            let push = info.number("push aside").unwrap_or(0);
            let lift = info.number("lift up").unwrap_or(0);
            let horizontal = (launch_velocity(push as f32, self.weight, 100.0) as i32) / 2;
            let vertical = launch_velocity(lift as f32, self.weight, 100.0) as i32;
            self.launch(horizontal, vertical * vertical / 3000, facing);
            self.bounce = false;
            self.grab_down_ms = down_ms;
        } else if self.hp > 0 {
            self.reaction = Reaction::Idle;
            self.z = 0.0;
        } else {
            self.set_hp(0, data);
        }
        self.set_motion(data);
    }
    fn rand(&mut self) -> u32 {
        self.random = self.random.wrapping_mul(214013).wrapping_add(2531011);
        (self.random >> 16) & 0x7fff
    }
    pub fn motion(&self) -> &'static str {
        match self.reaction {
            Reaction::Idle => {
                if self.demo_moving {
                    "Move"
                } else {
                    "Stay"
                }
            }
            Reaction::Stunned => "Damage1",
            Reaction::Sleeping => "Down",
            Reaction::Hurt | Reaction::Grabbed => {
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
    fn protection_state(&self, data: &GoblinAssets) -> ProtectionState {
        let kind = data.motions[self.motion()]
            .frames
            .get(self.anim.frame())
            .map_or(dnf_data::ani::DamageType::Normal, |f| {
                dnf_data::ani::DamageType::from_token(&f.damage_type)
            });
        let mut state = self.protection.clone();
        state.set_frame(match kind {
            dnf_data::ani::DamageType::Normal => Protection::Normal,
            dnf_data::ani::DamageType::SuperArmor => Protection::SuperArmor,
            dnf_data::ani::DamageType::Unbreakable => Protection::Invincible,
        });
        state
    }
    pub fn hurt_boxes(&self, data: &GoblinAssets) -> Vec<Box3> {
        if self.hp <= 0 || self.protection_state(data).effective() == Protection::Invincible {
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
        let outcome = self
            .protection_state(data)
            .apply_damage(&mut self.hp, damage);
        if outcome == DamageOutcome::Rejected {
            return false;
        }
        let shock = self.statuses.on_hit();
        if shock > 0 && self.hp > 0 {
            let before = self.hp;
            self.hp = (self.hp - shock).max(0);
            self.status_damage += before - self.hp;
            self.status_feedback += before - self.hp;
        }
        self.hits += 1;
        self.last_damage = damage;
        self.damage_feedback_ms = 750.0;
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
        if self.hp == 0 {
            self.clear_active_statuses(true);
        } else if self
            .statuses
            .has(dnf_core::active_status::ActiveStatus::Stone)
        {
            self.status_audio.push("STONE_CUT");
        }
        if self.hp > 0
            && (outcome == DamageOutcome::Armored
                || self.reaction == Reaction::Grabbed
                || self.statuses.frozen())
        {
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
            let horizontal = (launch_velocity(push as f32, self.weight, 100.0) as i32) / 2;
            let vertical = launch_velocity(lift as f32, self.weight, 100.0) as i32;
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
                recovery_ms(self.hit_recovery, (rate * 100.0) as i32, special) as f64;
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
        use dnf_core::active_status::ActiveStatus;
        for (_, age, _) in &mut self.status_exits {
            *age += dt_ms;
        }
        self.status_exits.retain(|(_, age, _)| *age < 1500.0);
        self.protection.advance(dt_ms);
        self.damage_feedback_ms = (self.damage_feedback_ms - dt_ms).max(0.0);
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
        if self.hp > 0 {
            let before = self.statuses.mask();
            let dot_kinds = [
                ActiveStatus::Poison,
                ActiveStatus::Burn,
                ActiveStatus::Bleeding,
            ];
            let dot_before = dot_kinds.map(|kind| self.statuses.get(kind).copied());
            for (i, s) in dot_before.iter().enumerate() {
                if let Some(s) = s {
                    if self.status_dot_serial[i] != s.serial {
                        self.status_dot_serial[i] = s.serial;
                        self.status_dot_pending[i] = 0;
                        self.status_dot_next[i] = 500.0;
                    }
                }
            }
            let fire = self.elemental_resistance[0]
                - if self.statuses.has(ActiveStatus::Freeze) {
                    50
                } else {
                    0
                };
            let dots = self.statuses.advance(
                active,
                self.demo_moving || self.vx != 0.0,
                [
                    (1.0 - self.status_resistance[2] as f64 / 100.0).clamp(0.0, 1.0),
                    (1.0 - self.status_resistance[9] as f64 / 100.0).clamp(0.0, 1.0)
                        * (1.0 - fire as f64 / 150.0).max(0.0),
                    1.0,
                ],
            );
            for dot in dots.into_iter().flatten() {
                self.status_splash += dot.splash;
                if dot.amount > 0 {
                    let before = self.hp;
                    self.hp = (self.hp - dot.amount).max(0);
                    self.status_damage += before - self.hp;
                    let channel = match dot.kind {
                        ActiveStatus::Poison => 0,
                        ActiveStatus::Burn => 1,
                        _ => 2,
                    };
                    self.status_dot_pending[channel] += before - self.hp;
                }
            }
            // HP is continuous; official poison/burn number observers report
            // accumulated damage every500ms. Bleed reports its discrete1s tick.
            for (i, s) in dot_before.iter().enumerate() {
                if let Some(s) = s {
                    let elapsed = s.elapsed_ms + active.min(s.remaining_ms);
                    if i == 2
                        || elapsed + 1e-7 >= self.status_dot_next[i]
                        || !self.statuses.has(dot_kinds[i])
                        || self.hp == 0
                    {
                        self.status_feedback += std::mem::take(&mut self.status_dot_pending[i]);
                        while elapsed + 1e-7 >= self.status_dot_next[i] {
                            self.status_dot_next[i] += 500.0;
                        }
                    }
                }
            }
            for kind in [ActiveStatus::Freeze, ActiveStatus::Stone] {
                if before & (1 << kind as usize) != 0 && !self.statuses.has(kind) {
                    self.status_exits.push((kind, 0.0, false));
                }
            }
            if before & !self.statuses.mask() != 0 {
                self.status_audio.push("CONDITION_REMOVE");
            }
            if self.hp == 0 {
                self.set_hp(0, data);
            }
        }
        if active == 0.0 {
            return;
        }
        if self.hp > 0 {
            if self.reaction == Reaction::Grabbed {
                return;
            }
            if self.statuses.frozen() {
                return;
            }
            if self.statuses.has(ActiveStatus::Sleep)
                && matches!(
                    self.reaction,
                    Reaction::Idle | Reaction::Hurt | Reaction::Stunned | Reaction::Sleeping
                )
            {
                if self.reaction != Reaction::Sleeping {
                    self.reaction = Reaction::Sleeping;
                    self.z = 0.0;
                    self.vx = 0.0;
                    self.set_motion(data);
                    self.anim.set_frame(data.land_frame);
                }
                return;
            }
            if self.statuses.has(ActiveStatus::Stun)
                && matches!(
                    self.reaction,
                    Reaction::Idle | Reaction::Hurt | Reaction::Stunned
                )
            {
                if self.reaction != Reaction::Stunned {
                    self.reaction = Reaction::Stunned;
                    self.vx = 0.0;
                    self.set_motion(data);
                }
                return;
            }
            if matches!(self.reaction, Reaction::Stunned | Reaction::Sleeping) {
                self.reaction = Reaction::Idle;
                self.set_motion(data);
            }
            let moving = self.demo_enabled
                && self.reaction == Reaction::Idle
                && !self.statuses.blocks_move()
                && self.demo_target.is_none_or(|(x, y)| {
                    !self.statuses.has(ActiveStatus::Blind)
                        || (self.x - x).abs() <= 250.0 && (self.y - y).abs() <= 250.0
                });
            if self.demo_moving != moving {
                self.demo_moving = moving;
                self.set_motion(data);
            }
            if moving {
                self.demo_clock -= active;
                if self.demo_clock <= 0.0 {
                    self.demo_clock = 1000.0;
                    self.demo_direction = if self.x > self.spawn.0 + 60.0 {
                        -1
                    } else if self.x < self.spawn.0 - 60.0 {
                        1
                    } else {
                        self.demo_direction
                    };
                    if self.statuses.has(ActiveStatus::Confuse) && self.rand() % 100 < 60 {
                        self.demo_direction = -self.demo_direction;
                    }
                }
                let speed = data.definition.at_level("move speed", 1).unwrap_or(400.0) as f32
                    / 10.0
                    * self.statuses.speed_rate() as f32;
                self.x = (self.x + speed * self.demo_direction as f32 * active as f32 / 1000.0)
                    .clamp((walk_x.0 + 20) as f32, (walk_x.1 - 20) as f32);
            }
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
        self.anim.advance(if self.demo_moving {
            active * self.statuses.speed_rate()
        } else {
            active
        });
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
                self.grab_down_ms = (self.grab_down_ms - active).max(0.0);
                if self.grab_down_ms > 0.0 {
                    self.anim.set_frame(data.land_frame);
                    return;
                }
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
            Reaction::Idle | Reaction::Grabbed | Reaction::Stunned | Reaction::Sleeping => {}
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
        let tint = crate::active_status::body_tint(&self.statuses);
        data.status_assets.draw(self, assets, fb, camera, true);
        let alpha = if self.reaction == Reaction::Dead {
            ((self.corpse_duration - self.corpse_ms) / 100.0).clamp(0.0, 1.0) as f32
        } else {
            1.0
        };
        if alpha > 0.0 {
            if let Some((color, offsets)) =
                crate::protection_visual::outline(self.effective_protection(data))
            {
                let mask = data.outline_mask(assets, self.motion(), self.frame(), alpha);
                for &(ox, oy) in offsets {
                    fb.blit_silhouette(
                        &mask.pixels,
                        mask.width,
                        mask.height,
                        x - camera + mask.offset.0 + ox,
                        y - z + mask.offset.1 + oy,
                        false,
                        color,
                    );
                }
            }
            if data.motions[self.motion()].shadow {
                fb.fill_ellipse(x - camera, y, 18, 5, rgba(0, 0, 0, (90.0 * alpha) as u8));
            }
            for layer in data.layers.iter().filter(|l| l.layer < 0) {
                if let Some(ani) = layer.motions.get(self.motion()) {
                    draw_frame_tinted(
                        ani,
                        self.frame(),
                        assets,
                        fb,
                        (x - camera, y - z),
                        -1,
                        alpha,
                        tint,
                    );
                }
            }
            draw_frame_tinted(
                &data.motions[self.motion()],
                self.frame(),
                assets,
                fb,
                (x - camera, y - z),
                -1,
                alpha,
                tint,
            );
            for layer in data.layers.iter().filter(|l| l.layer >= 0) {
                if let Some(ani) = layer.motions.get(self.motion()) {
                    draw_frame_tinted(
                        ani,
                        self.frame(),
                        assets,
                        fb,
                        (x - camera, y - z),
                        -1,
                        alpha,
                        tint,
                    );
                }
            }
        }
        data.status_assets.draw(self, assets, fb, camera, false);
        for kind in [
            dnf_core::active_status::ActiveStatus::Poison,
            dnf_core::active_status::ActiveStatus::Burn,
            dnf_core::active_status::ActiveStatus::Bleeding,
        ] {
            if let Some(s) = self.statuses.get(kind) {
                let a = (50.0 + (s.elapsed_ms * 0.008975979).sin() * 25.0) as u8;
                let color = if kind == dnf_core::active_status::ActiveStatus::Poison {
                    rgba(255, 0, 255, a)
                } else {
                    rgba(255, 0, 0, a)
                };
                let mask = data.outline_mask(assets, self.motion(), self.frame(), alpha);
                fb.blit_silhouette(
                    &mask.pixels,
                    mask.width,
                    mask.height,
                    x - camera + mask.offset.0,
                    y - z + mask.offset.1,
                    false,
                    color,
                );
            }
        }
        data.status_assets.draw_exits(self, fb, camera);
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
pub(crate) fn draw_frame(
    ani: &AnimFile,
    frame: usize,
    assets: &ClientAssets,
    fb: &mut Framebuffer,
    anchor: (i32, i32),
    facing: i32,
    alpha: f32,
) {
    draw_frame_tinted(
        ani,
        frame,
        assets,
        fb,
        anchor,
        facing,
        alpha,
        dnf_render::framebuffer::Color::WHITE,
    );
}
fn draw_frame_tinted(
    ani: &AnimFile,
    frame: usize,
    assets: &ClientAssets,
    fb: &mut Framebuffer,
    anchor: (i32, i32),
    facing: i32,
    alpha: f32,
    tint: dnf_render::framebuffer::Color,
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
                        tint: rgba(
                            (c[0] as u32 * tint.r as u32 / 255) as u8,
                            (c[1] as u32 * tint.g as u32 / 255) as u8,
                            (c[2] as u32 * tint.b as u32 / 255) as u8,
                            (c[3] as f32 * alpha) as u8,
                        ),
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
