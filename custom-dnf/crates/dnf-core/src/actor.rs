//! 40JP free movement / state 11 / normal state 6 jump.
//! Evidence: 58E920, 58FC30, 4B7430, 4B89D0 and swordman setup 5EEE40.
//! Network requests/acknowledgements are collapsed into local transitions.
use crate::anim::{AnimPlayer, Clip};
use crate::input::InputState;
use crate::jump::{Jump, JumpProfile, JumpTuning};
use crate::key_manager::{self, CommandChecker};
use crate::keymap::{Command, KeyMap};
use crate::movement::{facing_from_dir, MoveMode, MoveTuning, Mover};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Motion {
    Stand,
    Walk,
    Run,
    Jump,
    Attack,
    JumpAttack,
    DashAttack,
    UpperSlash,
    GhostSlash,
}
impl Motion {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Stand => "STAND",
            Self::Walk => "WALK",
            Self::Run => "RUN",
            Self::Jump => "JUMP",
            Self::Attack => "ATTACK",
            Self::JumpAttack => "AIRATK",
            Self::DashAttack => "DASHATK",
            Self::UpperSlash => "UPPER",
            Self::GhostSlash => "GHOST",
        }
    }
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ActorSpec {
    pub move_speed: f32,
    /// Attack animation rate and later combo-stage slide use this attribute.
    pub attack_speed: f32,
    pub jump_power: f32,
    pub jump_speed: f32,
    pub weight: f32,
    pub width: (i32, i32),
}
impl Default for ActorSpec {
    fn default() -> Self {
        Self {
            move_speed: 850.0,
            attack_speed: 850.0,
            jump_power: 430.0,
            jump_speed: 95.0,
            weight: 68000.0,
            width: (40, 10),
        }
    }
}
#[derive(Clone, Debug)]
pub struct ActorClips {
    pub stand: Clip,
    pub walk: Clip,
    pub run: Clip,
    pub jump: Clip,
    pub attacks: Vec<Clip>,
    pub jump_attack: Clip,
    pub dash_attack: Clip,
    pub hard_attack: Clip,
}
impl ActorClips {
    pub fn swordman_official() -> Self {
        Self {
            hard_attack: Clip::new(
                "HardAttack",
                vec![
                    50, 50, 50, 25, 25, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 200,
                ],
                false,
            ),
            stand: Clip::new("Stay", vec![120; 6], true),
            walk: Clip::new("Move", vec![100; 8], true),
            run: Clip::new("Dash", vec![100; 8], true),
            jump: Clip::new(
                "Jump",
                vec![
                    100, 100, 80, 80, 80, 80, 80, 10000, 100, 80, 80, 80, 80, 80, 10000, 50,
                ],
                false,
            ),
            attacks: [10, 11, 9]
                .into_iter()
                .enumerate()
                .map(|(i, n)| {
                    let mut delays = vec![50; n];
                    delays[n - 1] = 150;
                    Clip::new(format!("Attack{}", i + 1), delays, false)
                })
                .collect(),
            jump_attack: Clip::new("JumpAttack", vec![100, 50, 50, 40, 30, 30], false),
            dash_attack: Clip::new(
                "DashAttack",
                vec![100, 80, 50, 100, 60, 60, 60, 60, 30, 30],
                false,
            ),
        }
    }
}
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ActorEvent {
    None,
    Jumped,
    Landed,
    Attacked(usize),
    AirAttacked,
    DashAttacked,
    UpperSlashed,
    GhostSlashed,
    NoMana,
    SkillRejected(SkillFailure),
}
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum SkillFailure {
    Cooldown,
    Hp,
    Mp,
    Item(i32),
}
/// Official this[1723]: 0 preparation, 1 flight, 2 landing.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum JumpPhase {
    Ground,
    Preparing,
    Flying,
    Landing,
}
/// Swordman setup 5EEE40: FRAME INDICES, not hold-frame indices.
pub const SWORDMAN_JUMP_FRAMES: (usize, usize, usize) = (1, 8, 15);
pub const OFFICIAL_CLIP_RATE: f32 = 100.0;
/// 5EEE40 initializes this[1974] with these per-stage continuation frames.
pub const SWORDMAN_COMBO_FRAMES: [usize; 3] = [5, 4, 5];
pub type DirState = (i32, i32);

#[derive(Clone, Debug)]
pub struct ActiveSkill {
    pub level: usize,
    pub mp: i32,
    /// Per-skill HP costs are supplied by skill behavior, not assumed from a global SKL tag.
    pub hp: i32,
    pub items: Vec<(i32, i32)>,
    pub cooldown_ms: f64,
    pub values: Vec<i32>,
    pub cancel_skill: bool,
}
/// The last displayed pose of the interrupted action, captured before replacing its clip.
#[derive(Clone, Debug)]
pub struct CancelPose {
    pub motion: Motion,
    pub stage: usize,
    pub frame: usize,
    pub position: (i32, i32),
    pub facing: i32,
}

#[derive(Clone, Debug)]
struct GroundAttack {
    stage: usize,
    kind: Motion,
    velocity: f32,
    acceleration: f32,
}

#[derive(Clone, Debug)]
pub struct Actor {
    pub spec: ActorSpec,
    pub clips: ActorClips,
    pub x: i32,
    pub y: i32,
    pub facing: i32,
    pub motion: Motion,
    /// Simulation switch; false disables normal jump plane movement and turning.
    /// This is not the distinct official forced/skill jump branch.
    pub air_control: bool,
    pub commands: CommandChecker,
    mover: Mover,
    jump: Jump,
    profile: Option<JumpProfile>,
    anim: AnimPlayer,
    phase: JumpPhase,
    time_ms: f64,
    running: bool,
    air_mode: MoveMode,
    /// Input-layer candidate, independent of effective state 11.
    pub dash_candidate: i32,
    ground_attack: Option<GroundAttack>,
    air_attack: bool,
    air_attacks_used: u32,
    pub mp: i32,
    pub hp: i32,
    pub inventory: std::collections::HashMap<i32, i32>,
    pub skill_failures: Vec<SkillFailure>,
    pub cast_vibration: crate::feedback::CastVibration,
    upper_ready_ms: f64,
    ghost_ready_ms: f64,
    upper_committed_cooldown_ms: f64,
    ghost_committed_cooldown_ms: f64,
    shortcut_skill: Option<i32>,
    pub skill_slots: [Option<i32>; 6],
    pub upper_skill: ActiveSkill,
    pub ghost_skill: ActiveSkill,
    pub cancelled_pose: Option<CancelPose>,
    last_plane_velocity: (f32, f32),
}
impl Actor {
    pub fn new(spec: ActorSpec, clips: ActorClips, x: i32, y: i32) -> Self {
        Self {
            spec,
            clips: clips.clone(),
            x,
            y,
            facing: 1,
            motion: Motion::Stand,
            air_control: true,
            commands: CommandChecker::swordman(),
            mover: Mover::new(),
            jump: Jump::idle(),
            profile: None,
            anim: AnimPlayer::new(clips.stand, OFFICIAL_CLIP_RATE),
            phase: JumpPhase::Ground,
            time_ms: 0.0,
            running: false,
            air_mode: MoveMode::Walk,
            dash_candidate: 0,
            ground_attack: None,
            air_attack: false,
            air_attacks_used: 0,
            mp: 150,
            hp: 150,
            inventory: std::collections::HashMap::new(),
            skill_failures: Vec::new(),
            cast_vibration: crate::feedback::CastVibration::default(),
            upper_ready_ms: 0.0,
            ghost_ready_ms: 0.0,
            upper_committed_cooldown_ms: 0.0,
            ghost_committed_cooldown_ms: 0.0,
            shortcut_skill: None,
            skill_slots: [Some(5), Some(46), None, None, None, None],
            upper_skill: ActiveSkill {
                level: 1,
                mp: 6,
                hp: 0,
                items: Vec::new(),
                cooldown_ms: 2000.0,
                values: vec![10, 300, 100],
                cancel_skill: true,
            },
            ghost_skill: ActiveSkill {
                level: 1,
                mp: 20,
                hp: 0,
                items: Vec::new(),
                cooldown_ms: 8000.0,
                values: vec![40],
                cancel_skill: true,
            },
            cancelled_pose: None,
            last_plane_velocity: (0.0, 0.0),
        }
    }
    pub fn is_running_armed(&self) -> bool {
        self.running
    }
    pub fn altitude(&self) -> f32 {
        self.jump.altitude()
    }
    pub fn is_airborne(&self) -> bool {
        self.jump.active
    }
    pub fn jump_phase(&self) -> JumpPhase {
        self.phase
    }
    pub fn anim(&self) -> &AnimPlayer {
        &self.anim
    }
    pub fn attack_stage(&self) -> Option<usize> {
        self.ground_attack
            .as_ref()
            .filter(|a| a.kind == Motion::Attack)
            .map(|a| a.stage)
    }
    pub fn upper_cooldown_ms(&self) -> f64 {
        (self.upper_ready_ms - self.time_ms).max(0.0)
    }
    pub fn ghost_cooldown_ms(&self) -> f64 {
        (self.ghost_ready_ms - self.time_ms).max(0.0)
    }
    /// None before the first cast; negative time drives the original 450ms completion flash.
    pub fn skill_cooldown_remaining_ms(&self, id: i32) -> Option<f64> {
        let ready = match id {
            5 => self.ghost_ready_ms,
            46 => self.upper_ready_ms,
            _ => return None,
        };
        (ready > 0.0 && self.skill_cooldown_total_ms(id) > 0.0).then_some(ready - self.time_ms)
    }
    pub fn skill_cooldown_total_ms(&self, id: i32) -> f64 {
        match id {
            5 => self.ghost_committed_cooldown_ms,
            46 => self.upper_committed_cooldown_ms,
            _ => 0.0,
        }
    }
    /// Global cooldown time advances while a local impact freezes animation/physics.
    pub fn advance_hit_stop(&mut self, dt_ms: f64) {
        self.time_ms += dt_ms;
        self.cast_vibration.advance(dt_ms);
    }
    pub fn air_attacks_used(&self) -> u32 {
        self.air_attacks_used
    }

    fn start_ground_attack(&mut self, stage: usize, dir: DirState) {
        // 58E920 locks both axes. 4B89D0 gives later stages a timed forward slide.
        let parameter = if stage == 0 {
            0.0
        } else if dir.0 == 0 {
            100.0
        } else if dir.0 == self.facing {
            200.0
        } else {
            0.0
        };
        let ratio = self.spec.attack_speed / 1000.0;
        self.ground_attack = Some(GroundAttack {
            stage,
            kind: Motion::Attack,
            velocity: parameter * ratio * self.facing as f32,
            acceleration: if parameter == 0.0 {
                0.0
            } else {
                -300.0 * ratio * self.facing as f32
            },
        });
        self.running = false;
        self.motion = Motion::Attack;
        self.anim.set_clip(
            self.clips.attacks[stage].clone(),
            self.spec.attack_speed * 0.1,
            true,
        );
    }
    fn start_dash_attack(&mut self) {
        // 4B89D0 state12: abs(vx) integer, decelerate to zero at frame4.
        // 491C70 truncates EACH scaled frame delay before accumulation.
        let rate = self.spec.attack_speed * 0.1;
        let stop_ms: i32 = self
            .clips
            .dash_attack
            .delays_ms
            .iter()
            .take(4)
            .map(|&delay| (delay as f32 * 100.0 / rate) as i32)
            .sum();
        let speed = self.last_plane_velocity.0.abs().trunc();
        let acceleration = if stop_ms > 0 {
            -(1000.0 * speed / stop_ms as f32).trunc()
        } else {
            0.0
        };
        self.ground_attack = Some(GroundAttack {
            stage: 0,
            kind: Motion::DashAttack,
            velocity: speed * self.facing as f32,
            acceleration: acceleration * self.facing as f32,
        });
        self.running = false;
        self.motion = Motion::DashAttack;
        self.anim
            .set_clip(self.clips.dash_attack.clone(), rate, true);
    }
    fn start_upper_slash(&mut self, dir: DirState) {
        // 5F10C0 requests state24 {stage2,skill46}; 5F39F0 maps to state8.
        self.start_ground_attack(2, dir);
        if let Some(attack) = &mut self.ground_attack {
            attack.kind = Motion::UpperSlash;
        }
        self.motion = Motion::UpperSlash;
        self.mp -= self.upper_skill.mp;
        self.upper_committed_cooldown_ms = self.upper_skill.cooldown_ms;
        self.upper_ready_ms = self.time_ms + self.upper_committed_cooldown_ms;
    }
    pub fn can_skill(&self, skill: &ActiveSkill) -> bool {
        skill.level > 0
            && self.phase == JumpPhase::Ground
            && match &self.ground_attack {
                None => true,
                Some(a) => a.kind == Motion::Attack && skill.cancel_skill,
            }
    }
    pub fn resource_failures(&self, skill: &ActiveSkill) -> Vec<SkillFailure> {
        let mut failures = Vec::new();
        if skill.hp > 0 && self.hp <= skill.hp {
            failures.push(SkillFailure::Hp);
        }
        if self.mp < skill.mp {
            failures.push(SkillFailure::Mp);
        }
        for &(id, count) in &skill.items {
            if count > 0 && self.inventory.get(&id).copied().unwrap_or(0) < count {
                failures.push(SkillFailure::Item(id));
            }
        }
        failures
    }
    /// Shared visual rejection, reusable by future skill-specific HP/item admissions.
    pub fn reject_skill(&mut self, failure: SkillFailure) {
        self.skill_failures.push(failure);
        self.cast_vibration.trigger();
    }
    /// Skill admission is independent of animation freeze. Failed casts leave the old action intact.
    pub fn try_skill(&mut self) -> Option<ActorEvent> {
        let shortcut = self.shortcut_skill.take();
        let ghost = shortcut == Some(5);
        let z = self.commands.hit(key_manager::UPPER);
        let upper = z || shortcut == Some(46);
        if z {
            self.commands.consume(key_manager::UPPER);
        }
        let (skill, ready) = if upper {
            (&self.upper_skill, self.upper_cooldown_ms())
        } else if ghost {
            (&self.ghost_skill, self.ghost_cooldown_ms())
        } else {
            return None;
        };
        if !self.can_skill(skill) {
            return None;
        }
        let mut failures = self.resource_failures(skill);
        if ready > 0.0 {
            failures.insert(0, SkillFailure::Cooldown);
        }
        if let Some(&first) = failures.first() {
            for failure in failures {
                self.reject_skill(failure);
            }
            return Some(if first == SkillFailure::Mp {
                ActorEvent::NoMana
            } else {
                ActorEvent::SkillRejected(first)
            });
        }
        // Admission is atomic: rejected casts consume nothing and keep the previous pose.
        let hp = skill.hp;
        let items = skill.items.clone();
        self.hp -= hp;
        for (id, count) in items {
            *self.inventory.entry(id).or_default() -= count;
        }
        if self.ground_attack.is_none() {
            self.facing = facing_from_dir(self.read_direction(), self.facing);
        }
        if let Some(a) = &self.ground_attack {
            self.cancelled_pose = Some(CancelPose {
                motion: a.kind,
                stage: a.stage,
                frame: self.anim.frame(),
                position: (self.x, self.y),
                facing: self.facing,
            });
        }
        if upper {
            self.start_upper_slash(self.read_direction());
            Some(ActorEvent::UpperSlashed)
        } else {
            self.ground_attack = Some(GroundAttack {
                stage: 0,
                kind: Motion::GhostSlash,
                velocity: 0.0,
                acceleration: 0.0,
            });
            self.running = false;
            self.motion = Motion::GhostSlash;
            self.anim.set_clip(
                self.clips.hard_attack.clone(),
                self.spec.attack_speed * 0.1,
                true,
            );
            self.mp -= self.ghost_skill.mp;
            self.ghost_committed_cooldown_ms = self.ghost_skill.cooldown_ms;
            self.ghost_ready_ms = self.time_ms + self.ghost_committed_cooldown_ms;
            Some(ActorEvent::GhostSlashed)
        }
    }
    pub fn read_direction(&self) -> DirState {
        let x = match self.commands.last_held(&[Command::Left, Command::Right]) {
            Some(Command::Left) => -1,
            Some(Command::Right) => 1,
            _ => 0,
        };
        let y = match self.commands.last_held(&[Command::Up, Command::Down]) {
            Some(Command::Up) => -1,
            Some(Command::Down) => 1,
            _ => 0,
        };
        (x, y)
    }
    /// Global checker clocks run before object hit stop, so frozen actors do not buffer presses.
    pub fn update_commands(&mut self, input: &InputState, km: &KeyMap, dt_ms: f64) {
        let free = self.ground_attack.is_none();
        let ground = free && self.phase == JumpPhase::Ground;
        let attack = if let Some(a) = &self.ground_attack {
            a.kind == Motion::Attack
                && a.stage + 1 < self.clips.attacks.len()
                && self.anim.frame() >= SWORDMAN_COMBO_FRAMES[a.stage]
        } else {
            ground
                || self.phase == JumpPhase::Flying && self.air_attacks_used == 0 && !self.air_attack
        };
        self.commands.enable(key_manager::ATTACK, attack);
        self.commands.enable(key_manager::JUMP, ground);
        self.commands
            .enable(key_manager::UPPER, self.can_skill(&self.upper_skill));
        self.shortcut_skill = Command::ALL[9..15]
            .iter()
            .enumerate()
            .find_map(|(i, &cmd)| self.skill_slots[i].filter(|_| input.just_pressed(km.get(cmd))));
        self.commands
            .enable(key_manager::RUN_RIGHT, ground && !self.running);
        self.commands
            .enable(key_manager::RUN_LEFT, ground && !self.running);
        self.commands.update(input, km, dt_ms);
    }
    pub fn step(
        &mut self,
        input: &InputState,
        km: &KeyMap,
        mt: &MoveTuning,
        jt: &JumpTuning,
    ) -> ActorEvent {
        self.update_commands(input, km, jt.logic_ms);
        self.step_prepared(mt, jt)
    }
    pub fn step_prepared(&mut self, mt: &MoveTuning, jt: &JumpTuning) -> ActorEvent {
        self.advance_hit_stop(jt.logic_ms);
        self.step_at_current_time(mt, jt)
    }
    /// World time is already advanced; animation/physics may advance by a shorter hit-stop remainder.
    pub fn step_at_current_time(&mut self, mt: &MoveTuning, jt: &JumpTuning) -> ActorEvent {
        let dir = self.read_direction();
        self.dash_candidate = if self.commands.hit(key_manager::RUN_RIGHT) {
            1
        } else if self.commands.hit(key_manager::RUN_LEFT) {
            -1
        } else {
            0
        };
        let mut event = self.try_skill().unwrap_or(ActorEvent::None);
        let dt = (jt.logic_ms / 1000.0) as f32;
        let attack_pressed = self.commands.hit(key_manager::ATTACK);
        if let Some(attack) = &self.ground_attack {
            let stage = attack.stage;
            if attack.kind == Motion::Attack
                && attack_pressed
                && stage + 1 < self.clips.attacks.len()
                && self.anim.frame()
                    >= SWORDMAN_COMBO_FRAMES
                        .get(stage)
                        .copied()
                        .unwrap_or(usize::MAX)
            {
                self.start_ground_attack(stage + 1, dir);
                event = ActorEvent::Attacked(stage + 1);
            }
        }
        if let Some(attack) = &mut self.ground_attack {
            // 49FD80: average velocity integration, with 4A0600 stopping at zero.
            let duration = if attack.acceleration != 0.0 {
                dt.min((-attack.velocity / attack.acceleration).max(0.0))
            } else {
                dt
            };
            let v1 = attack.velocity + attack.acceleration * duration;
            let displacement = (attack.velocity + v1) * 0.5 * duration;
            (self.x, self.y) =
                self.mover
                    .step_position((self.x, self.y), (displacement / dt, 0.0), dt);
            attack.velocity = if duration < dt { 0.0 } else { v1 };
            self.motion = attack.kind;
            self.anim.advance(jt.logic_ms);
            if self.anim.finished() {
                self.ground_attack = None;
            }
            return event;
        }
        // 4B7430 case 11: current-facing x, or x neutral with nonneutral y.
        let current_mode = if self.running {
            MoveMode::Run
        } else {
            MoveMode::Walk
        };
        let velocity_dir =
            movement_direction(mt.velocity_for(self.spec.move_speed, current_mode, dir));
        if self.running
            && (velocity_dir == (0, 0) || (velocity_dir.0 != 0 && velocity_dir.0 != self.facing))
        {
            self.running = false;
        }
        if self.phase == JumpPhase::Ground {
            self.facing = facing_from_dir(velocity_dir, self.facing);
            // Live trace: candidates still occur with opposing input, but do
            // not sustain state 11. Collapse request/ack/cancel here. This is
            // NOT a condition in 4BF540; see docs/02 for the network boundary.
            let opposing_held =
                self.commands.is_down(Command::Left) && self.commands.is_down(Command::Right);
            if self.dash_candidate != 0 && opposing_held {
                self.commands.consume(if self.dash_candidate > 0 {
                    key_manager::RUN_RIGHT
                } else {
                    key_manager::RUN_LEFT
                });
            }
            if self.dash_candidate != 0 && !opposing_held {
                self.running = true;
            }
            // 4B7430 prioritizes dash over jump. A same-update jump is not queued.
            if self.dash_candidate == 0 && self.commands.hit(key_manager::JUMP) {
                self.air_mode = if self.running {
                    MoveMode::Run
                } else {
                    MoveMode::Walk
                };
                self.running = false;
                self.phase = JumpPhase::Preparing;
                self.anim
                    .set_clip(self.clips.jump.clone(), OFFICIAL_CLIP_RATE, true);
                self.air_attacks_used = 0;
            } else if self.dash_candidate == 0 && attack_pressed && !self.clips.attacks.is_empty() {
                if self.running {
                    self.start_dash_attack();
                    self.anim.advance(jt.logic_ms);
                    return ActorEvent::DashAttacked;
                }
                self.start_ground_attack(0, dir);
                self.anim.advance(jt.logic_ms);
                return ActorEvent::Attacked(0);
            }
        }
        match self.phase {
            JumpPhase::Ground => {
                let mode = if self.running {
                    MoveMode::Run
                } else {
                    MoveMode::Walk
                };
                let velocity = mt.velocity_for(self.spec.move_speed, mode, dir);
                self.last_plane_velocity = velocity;
                (self.x, self.y) = self.mover.step_position((self.x, self.y), velocity, dt);
                self.motion = if movement_direction(velocity) == (0, 0) {
                    Motion::Stand
                } else if self.running {
                    Motion::Run
                } else {
                    Motion::Walk
                };
                let (clip, rate) = match self.motion {
                    Motion::Stand => (self.clips.stand.clone(), OFFICIAL_CLIP_RATE),
                    Motion::Run => (self.clips.run.clone(), self.spec.move_speed * 0.1),
                    _ => (self.clips.walk.clone(), self.spec.move_speed * 0.1),
                };
                self.anim.set_clip(clip, rate, false);
                self.anim.advance(jt.logic_ms);
            }
            JumpPhase::Preparing => {
                self.motion = Motion::Jump;
                self.anim.advance(jt.logic_ms);
                if self.anim.frame() >= SWORDMAN_JUMP_FRAMES.0 {
                    self.profile = Some(self.jump.start(
                        jt,
                        self.spec.jump_power,
                        self.spec.weight,
                        self.spec.jump_speed,
                    ));
                    self.phase = JumpPhase::Flying;
                    self.anim.set_frame(SWORDMAN_JUMP_FRAMES.0);
                    event = ActorEvent::Jumped;
                }
            }
            JumpPhase::Flying => {
                if self.air_control {
                    let velocity = mt.air_velocity_for(self.spec.move_speed, self.air_mode, dir);
                    (self.x, self.y) = self.mover.step_position((self.x, self.y), velocity, dt);
                    if self.air_attacks_used == 0 {
                        self.facing = facing_from_dir(movement_direction(velocity), self.facing);
                    }
                }
                // 4B7430 state6/1 grants one air attack per positive-power launch.
                if !self.air_attack && self.air_attacks_used == 0 && attack_pressed {
                    self.air_attack = true;
                    self.air_attacks_used += 1;
                    self.anim.set_clip(
                        self.clips.jump_attack.clone(),
                        self.spec.attack_speed * 0.1,
                        true,
                    );
                    event = ActorEvent::AirAttacked;
                }
                self.motion = if self.air_attack {
                    Motion::JumpAttack
                } else {
                    Motion::Jump
                };
                self.anim.advance(jt.logic_ms);
                if let Some(profile) = self.profile {
                    if self.jump.step(&profile, dt) {
                        self.phase = JumpPhase::Landing;
                        self.air_attack = false;
                        self.motion = Motion::Jump;
                        self.anim
                            .set_clip(self.clips.jump.clone(), OFFICIAL_CLIP_RATE, false);
                        self.anim.set_frame(SWORDMAN_JUMP_FRAMES.2);
                        event = ActorEvent::Landed;
                    } else if self.air_attack && self.anim.finished() {
                        // Return to state6/1 with launch power0: preserve t/vz,
                        // takeoff mode and spent attack budget, never relaunch.
                        self.air_attack = false;
                        self.motion = Motion::Jump;
                        self.anim
                            .set_clip(self.clips.jump.clone(), OFFICIAL_CLIP_RATE, true);
                        self.anim.set_frame(if self.jump.rising {
                            SWORDMAN_JUMP_FRAMES.1 - 1
                        } else {
                            SWORDMAN_JUMP_FRAMES.2 - 1
                        });
                    } else if !self.air_attack
                        && !self.jump.rising
                        && self.anim.frame() < SWORDMAN_JUMP_FRAMES.1
                    {
                        self.anim.set_frame(SWORDMAN_JUMP_FRAMES.1);
                    }
                }
            }
            JumpPhase::Landing => {
                // 58E920 substate 2: both plane speeds zero, frame=15, rate=100%.
                self.motion = Motion::Jump;
                self.anim.advance(jt.logic_ms);
                if self.anim.finished() {
                    self.phase = JumpPhase::Ground;
                    self.profile = None;
                    self.running = false;
                }
            }
        }
        event
    }
    pub fn teleport(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
        self.mover.reset();
        self.commands.clear();
        self.shortcut_skill = None;
        self.cancelled_pose = None;
        self.jump = Jump::idle();
        self.profile = None;
        self.phase = JumpPhase::Ground;
        self.running = false;
        self.air_mode = MoveMode::Walk;
        self.dash_candidate = 0;
        self.ground_attack = None;
        self.air_attack = false;
        self.air_attacks_used = 0;
        self.last_plane_velocity = (0.0, 0.0);
        self.motion = Motion::Stand;
        self.anim
            .set_clip(self.clips.stand.clone(), OFFICIAL_CLIP_RATE, true);
    }

    /// Losing window focus discards keyboard gestures without cancelling flight.
    pub fn clear_direction_input(&mut self) {
        self.commands.clear();
        self.shortcut_skill = None;
        self.dash_candidate = 0;
        self.running = false;
    }
}

/// 4996F0/499730 classify controller velocity, not the raw key direction.
fn movement_direction(velocity: (f32, f32)) -> DirState {
    fn axis(v: f32) -> i32 {
        if v > 5.0 {
            1
        } else if v < -5.0 {
            -1
        } else {
            0
        }
    }
    (axis(velocity.0), axis(velocity.1))
}
