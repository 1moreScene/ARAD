use dnf_core::actor::SWORDMAN_COMBO_FRAMES;
use dnf_core::*;

#[test]
fn upper_protection_is_owned_by_action_and_teleport_preserves_independent_buff() {
    use dnf_core::protection::{DamageOutcome, Protection};
    let mut r = Replay::new();
    r.step(&[Command::Skill]);
    assert_eq!(r.actor.motion, Motion::UpperSlash);
    assert_eq!(r.actor.protection.effective(), Protection::SuperArmor);
    let hp = r.actor.hp;
    assert_eq!(r.actor.receive_damage(10), DamageOutcome::Armored);
    assert_eq!(r.actor.hp, hp - 10);
    r.actor.protection.grant(1, Protection::Invincible, None);
    r.actor.teleport(100, 200);
    assert_eq!(r.actor.protection.effective(), Protection::Invincible);
    r.actor.protection.remove(1);
    assert_eq!(r.actor.protection.effective(), Protection::Normal);
}

#[test]
fn upper_protection_exits_and_timers_follow_world_clock_during_hit_stop() {
    use dnf_core::protection::Protection;
    let mut r = Replay::new();
    r.step(&[Command::Skill]);
    for _ in 0..90 {
        r.step(&[]);
    }
    assert_eq!(r.actor.protection.effective(), Protection::Normal);
    r.actor
        .protection
        .grant(2, Protection::Invincible, Some(50.0));
    r.actor.advance_hit_stop(50.0);
    assert_eq!(r.actor.protection.effective(), Protection::Normal);
}

#[test]
fn upper_armor_ends_at_frame_two_at_each_attack_speed_before_action_finishes() {
    use dnf_core::protection::{DamageOutcome, Protection};
    for speed in [425.0, 850.0, 1700.0] {
        let mut r = Replay::new();
        r.actor.spec.attack_speed = speed;
        r.step(&[Command::Skill]);
        let mut saw_frame_one = false;
        while r.actor.anim().frame() < 2 {
            assert_eq!(r.actor.motion, Motion::UpperSlash);
            assert_eq!(r.actor.protection.effective(), Protection::SuperArmor);
            saw_frame_one |= r.actor.anim().frame() == 1;
            r.step(&[]);
        }
        assert!(saw_frame_one, "speed={speed}");
        assert_eq!(r.actor.anim().frame(), 2);
        assert_eq!(r.actor.motion, Motion::UpperSlash);
        assert!(!r.actor.anim().finished());
        assert_eq!(r.actor.protection.effective(), Protection::Normal);
        assert_eq!(r.actor.receive_damage(1), DamageOutcome::React);
        while r.actor.motion == Motion::UpperSlash {
            r.step(&[]);
            assert_eq!(r.actor.protection.effective(), Protection::Normal);
        }
    }
}

#[test]
fn upper_frame_gate_preserves_separate_armor_buff_and_hit_stop_preserves_startup_armor() {
    use dnf_core::protection::Protection;
    let mut r = Replay::new();
    r.step(&[Command::Skill]);
    r.actor.advance_hit_stop(500.0); // world time alone cannot consume animation-frame armor
    assert_eq!(r.actor.protection.effective(), Protection::SuperArmor);
    r.actor.protection.grant(101, Protection::SuperArmor, None);
    while r.actor.anim().frame() < 2 {
        r.step(&[]);
    }
    assert_eq!(r.actor.protection.effective(), Protection::SuperArmor);
    r.actor.protection.remove(101);
    assert_eq!(r.actor.motion, Motion::UpperSlash);
    assert_eq!(r.actor.protection.effective(), Protection::Normal);
}

#[test]
fn resource_rejection_is_atomic_and_shared_vibration_preserves_cancelled_attack() {
    use dnf_core::actor::SkillFailure;
    let mut r = Replay::new();
    r.actor.ghost_skill.hp = 10;
    r.actor.ghost_skill.items = vec![(3037, 1)];
    r.actor.hp = 10;
    r.actor.mp = 19;
    r.step(&[Command::Attack]);
    r.step(&[]);
    assert_eq!(
        r.step(&[Command::SkillSlot1]),
        ActorEvent::SkillRejected(SkillFailure::Hp)
    );
    assert_eq!(
        r.actor.skill_failures,
        vec![SkillFailure::Hp, SkillFailure::Mp, SkillFailure::Item(3037)]
    );
    assert_eq!(
        (r.actor.hp, r.actor.mp, r.actor.x, r.actor.y),
        (10, 19, 300, 200)
    );
    assert_eq!(r.actor.motion, Motion::Attack);
    assert!(r.actor.cancelled_pose.is_none());
    assert!(r.actor.cast_vibration.active());
    assert_eq!(r.actor.ghost_cooldown_ms(), 0.0);
    r.actor.hp = 11;
    r.actor.mp = 20;
    r.actor.inventory.insert(3037, 1);
    r.step(&[]);
    assert_eq!(r.step(&[Command::SkillSlot1]), ActorEvent::GhostSlashed);
    assert_eq!(
        (r.actor.hp, r.actor.mp, r.actor.inventory[&3037]),
        (1, 0, 0)
    );
    for _ in 0..90 {
        r.step(&[]);
    }
    assert!(!r.actor.cast_vibration.active());
    assert_eq!(
        r.step(&[Command::SkillSlot1]),
        ActorEvent::SkillRejected(SkillFailure::Cooldown)
    );
}

struct Replay {
    actor: Actor,
    input: InputState,
    km: KeyMap,
}
impl Replay {
    fn new() -> Self {
        Self {
            actor: Actor::new(
                ActorSpec::default(),
                ActorClips::swordman_official(),
                300,
                200,
            ),
            input: InputState::new(),
            km: KeyMap::default(),
        }
    }
    fn step(&mut self, commands: &[Command]) -> ActorEvent {
        self.input.begin_frame();
        for command in commands {
            self.input.set(self.km.get(*command), true);
        }
        self.actor.step(
            &self.input,
            &self.km,
            &MoveTuning::default(),
            &JumpTuning::default(),
        )
    }
    fn launch(&mut self) {
        self.step(&[Command::Jump]);
        for _ in 0..10 {
            if self.actor.is_airborne() {
                return;
            }
            self.step(&[]);
        }
        panic!("did not launch");
    }
}

#[test]
fn x_starts_attack_and_locks_ground_movement_and_turning() {
    let mut r = Replay::new();
    assert_eq!(r.step(&[Command::Attack]), ActorEvent::Attacked(0));
    assert_eq!(r.actor.motion, Motion::Attack);
    assert_eq!(r.actor.anim().clip().name, "Attack1");
    assert_eq!(r.actor.anim().rate_percent(), 85.0);
    for _ in 0..12 {
        r.step(&[Command::Left, Command::Up, Command::Jump]);
    }
    assert_eq!((r.actor.x, r.actor.y, r.actor.facing), (300, 200, 1));
    assert!(!r.actor.is_airborne());
}

#[test]
fn hold_x_does_not_auto_combo_or_restart_on_completion() {
    let mut r = Replay::new();
    let mut starts = 0;
    for _ in 0..100 {
        if matches!(r.step(&[Command::Attack]), ActorEvent::Attacked(_)) {
            starts += 1;
        }
    }
    assert_eq!(starts, 1);
    assert_eq!(r.actor.motion, Motion::Stand);
}

#[test]
fn early_x_is_not_buffered_and_combo_uses_stage_specific_gates() {
    let mut r = Replay::new();
    r.step(&[Command::Attack]);
    r.step(&[]);
    assert_eq!(r.step(&[Command::Attack]), ActorEvent::None);
    assert_eq!(r.actor.attack_stage(), Some(0));
    while r.actor.anim().frame() < 5 {
        r.step(&[]);
    }
    assert_eq!(r.step(&[Command::Attack]), ActorEvent::Attacked(1));
    while r.actor.anim().frame() < 4 {
        r.step(&[]);
    }
    assert_eq!(r.step(&[Command::Attack]), ActorEvent::Attacked(2));
    while r.actor.anim().frame() < 5 {
        r.step(&[]);
    }
    assert_eq!(r.step(&[Command::Attack]), ActorEvent::None);
    for _ in 0..100 {
        r.step(&[]);
    }
    assert_eq!(r.actor.motion, Motion::Stand);
    assert!(
        r.actor.x > 300,
        "later stages should have official forward slide"
    );
    assert_eq!(r.actor.y, 200);
}

#[test]
fn normal_attack_duration_scales_by_attack_speed() {
    for (speed, min, max) in [(850.0, 42, 44), (1700.0, 21, 23)] {
        let mut r = Replay::new();
        r.actor.spec.attack_speed = speed;
        r.step(&[Command::Attack]);
        let mut steps = 1;
        while r.actor.attack_stage().is_some() {
            r.step(&[]);
            steps += 1;
            assert!(steps < 100);
        }
        assert!((min..=max).contains(&steps), "speed={speed} steps={steps}");
    }
}

#[test]
fn air_attack_once_preserves_the_existing_jump_trajectory() {
    let mut r = Replay::new();
    let mut control = Replay::new();
    r.launch();
    control.launch();
    let mut attacks = 0;
    let mut resumed = false;
    for i in 0..60 {
        let commands = if matches!(i, 2 | 5 | 12 | 20) {
            &[Command::Attack][..]
        } else {
            &[][..]
        };
        if r.step(commands) == ActorEvent::AirAttacked {
            attacks += 1;
        }
        control.step(&[]);
        assert!(
            (r.actor.altitude() - control.actor.altitude()).abs() < 0.001,
            "attack cannot relaunch or alter gravity"
        );
        if i > 25 && r.actor.is_airborne() && r.actor.motion == Motion::Jump {
            resumed = true;
        }
    }
    assert_eq!(attacks, 1);
    assert!(resumed);
    assert!(!r.actor.is_airborne());
}

#[test]
fn landing_interrupts_air_attack_and_resets_budget_only_on_next_jump() {
    let mut r = Replay::new();
    r.launch();
    while r.actor.altitude() < 20.0 {
        r.step(&[]);
    }
    while r.actor.altitude() > 20.0 {
        r.step(&[]);
    }
    assert_eq!(r.step(&[Command::Attack]), ActorEvent::AirAttacked);
    let mut landed = false;
    for _ in 0..10 {
        if r.step(&[]) == ActorEvent::Landed {
            landed = true;
            assert_eq!(r.actor.motion, Motion::Jump);
            assert_eq!(r.actor.anim().clip().name, "Jump");
            assert_eq!(r.actor.anim().frame(), 15);
            break;
        }
    }
    assert!(landed);
    for _ in 0..10 {
        r.step(&[]);
    }
    r.launch();
    assert_eq!(r.step(&[Command::Attack]), ActorEvent::AirAttacked);
}

#[test]
fn air_attack_locks_facing_for_the_rest_of_that_jump() {
    let mut r = Replay::new();
    r.launch();
    r.step(&[Command::Attack]);
    for _ in 0..30 {
        r.step(&[Command::Left]);
    }
    assert_eq!(r.actor.facing, 1);
    assert!(
        r.actor.x < 300,
        "plane controls can move backward while facing remains locked"
    );
}

#[test]
fn rebind_and_teleport_keep_attack_state_consistent() {
    let mut r = Replay::new();
    r.km.load_overrides("Attack=0x30"); // B: A now occupies skill shortcut 1.
    assert_eq!(r.step(&[Command::Attack]), ActorEvent::Attacked(0));
    r.actor.teleport(500, 200);
    assert_eq!(r.actor.attack_stage(), None);
    assert_eq!(r.actor.motion, Motion::Stand);
    r.step(&[]);
    assert_eq!(r.step(&[Command::Attack]), ActorEvent::Attacked(0));
}

#[test]
fn running_x_uses_dash_animation_and_stops_at_frame_four() {
    let mut r = Replay::new();
    r.step(&[Command::Right]);
    r.step(&[]);
    r.step(&[Command::Right]);
    for _ in 0..8 {
        r.step(&[Command::Right]);
    }
    assert_eq!(r.actor.motion, Motion::Run);
    let x = r.actor.x;
    let y = r.actor.y;
    assert_eq!(
        r.step(&[Command::Right, Command::Attack]),
        ActorEvent::DashAttacked
    );
    assert_eq!(r.actor.anim().clip().name, "DashAttack");
    for _ in 0..40 {
        r.step(&[Command::Left, Command::Up, Command::Jump, Command::Attack]);
    }
    assert_eq!(r.actor.y, y);
    assert_eq!(r.actor.facing, 1);
    assert!(
        (44..=50).contains(&(r.actor.x - x)),
        "slide {}",
        r.actor.x - x
    );
    let stopped = r.actor.x;
    for _ in 0..4 {
        r.step(&[]);
    }
    assert_eq!(r.actor.x, stopped);
    assert_eq!(r.actor.attack_stage(), None);
}

#[test]
fn upper_uses_third_animation_mana_cooldown_and_no_hold_repeat() {
    let mut r = Replay::new();
    assert_eq!(r.step(&[Command::Skill]), ActorEvent::UpperSlashed);
    assert_eq!(r.actor.motion, Motion::UpperSlash);
    assert_eq!(r.actor.anim().clip().name, "Attack3");
    assert_eq!(r.actor.mp, 144);
    for _ in 0..160 {
        assert_ne!(r.step(&[Command::Skill]), ActorEvent::UpperSlashed);
    }
    r.step(&[]);
    assert_eq!(r.step(&[Command::Skill]), ActorEvent::UpperSlashed);
    assert_eq!(r.actor.mp, 138);
}

#[test]
fn upper_rejects_airborne_cooldown_and_insufficient_mana() {
    let mut r = Replay::new();
    r.launch();
    assert_ne!(r.step(&[Command::Skill]), ActorEvent::UpperSlashed);
    assert_eq!(r.actor.mp, 150);
    r.actor.teleport(300, 200);
    r.actor.mp = 5;
    r.step(&[]);
    assert_eq!(r.step(&[Command::Skill]), ActorEvent::NoMana);
    assert_eq!(r.actor.mp, 5);
    assert_eq!(r.actor.upper_cooldown_ms(), 0.0);
    r.actor.mp = 6;
    r.step(&[]);
    assert_eq!(r.step(&[Command::Skill]), ActorEvent::UpperSlashed);
    assert_eq!(r.actor.mp, 0);
    for _ in 0..60 {
        r.step(&[]);
    }
    assert_ne!(r.step(&[Command::Skill]), ActorEvent::UpperSlashed);
}

#[test]
fn both_skills_cancel_every_normal_combo_stage_and_capture_the_interrupted_pose() {
    for command in [Command::Skill, Command::SkillSlot1] {
        for stage in 0..3 {
            let mut r = Replay::new();
            r.step(&[Command::Attack]);
            for (next, &gate) in SWORDMAN_COMBO_FRAMES.iter().enumerate().take(stage) {
                r.step(&[]);
                while r.actor.anim().frame() < gate {
                    r.step(&[]);
                }
                assert_eq!(r.step(&[Command::Attack]), ActorEvent::Attacked(next + 1));
            }
            for _ in 0..5 {
                r.step(&[]);
            }
            let old = r.actor.anim().frame();
            let e = r.step(&[command]);
            assert_eq!(
                e,
                if command == Command::Skill {
                    ActorEvent::UpperSlashed
                } else {
                    ActorEvent::GhostSlashed
                }
            );
            let pose = r.actor.cancelled_pose.as_ref().unwrap();
            assert_eq!((pose.stage, pose.frame), (stage, old));
            assert_eq!(
                r.actor.mp,
                if command == Command::Skill { 144 } else { 130 }
            );
            assert_eq!(r.actor.attack_stage(), None);
        }
    }
}

#[test]
fn holding_x_does_not_block_upper_or_ghost_forced_cast() {
    for key in [Command::Skill, Command::SkillSlot1] {
        let mut r = Replay::new();
        r.step(&[Command::Attack]);
        r.step(&[Command::Attack]);
        assert_eq!(
            r.step(&[Command::Attack, key]),
            if key == Command::Skill {
                ActorEvent::UpperSlashed
            } else {
                ActorEvent::GhostSlashed
            }
        );
        assert!(r.actor.cancelled_pose.is_some());
    }
}

#[test]
fn ghost_no_hold_repeat_no_air_cast_no_skill_chain_and_failed_cancel_preserves_action() {
    let mut r = Replay::new();
    assert_eq!(r.step(&[Command::SkillSlot1]), ActorEvent::GhostSlashed);
    assert_eq!(r.actor.anim().clip().name, "HardAttack");
    assert!(r.actor.ghost_cooldown_ms() > 7900.0);
    for _ in 0..520 {
        assert_ne!(r.step(&[Command::SkillSlot1]), ActorEvent::GhostSlashed);
    }
    assert_eq!(r.actor.mp, 130);
    r.step(&[]);
    assert_eq!(r.step(&[Command::SkillSlot1]), ActorEvent::GhostSlashed);
    r.step(&[]);
    assert_ne!(r.step(&[Command::Skill]), ActorEvent::UpperSlashed);
    assert!(r.actor.cancelled_pose.is_none());
    let mut r = Replay::new();
    r.launch();
    assert_ne!(r.step(&[Command::SkillSlot1]), ActorEvent::GhostSlashed);
    r.actor.teleport(300, 200);
    r.step(&[]);
    r.step(&[Command::Attack]);
    r.step(&[]);
    r.actor.mp = 19;
    assert_eq!(r.step(&[Command::SkillSlot1]), ActorEvent::NoMana);
    assert_eq!(r.actor.motion, Motion::Attack);
    assert!(r.actor.cancelled_pose.is_none());
    assert_eq!(r.actor.ghost_cooldown_ms(), 0.0);
    r.actor.mp = 150;
    r.actor.ghost_skill.cancel_skill = false;
    r.step(&[]);
    assert_ne!(r.step(&[Command::SkillSlot1]), ActorEvent::GhostSlashed);
}
