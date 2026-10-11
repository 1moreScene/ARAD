use dnf_core::*;

struct Replay {
    actor: Actor,
    input: InputState,
    keys: KeyMap,
    tuning: JumpTuning,
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
            keys: KeyMap::official_default(),
            tuning: JumpTuning::default(),
        }
    }
    fn step(&mut self, keys: &[Command]) -> ActorEvent {
        self.input.begin_frame();
        for key in keys {
            self.input.set(self.keys.get(*key), true);
        }
        self.actor.step(
            &self.input,
            &self.keys,
            &MoveTuning::default(),
            &self.tuning,
        )
    }
    fn run(&mut self) {
        self.step(&[Command::Right]);
        self.step(&[]);
        self.step(&[Command::Right]);
        assert_eq!(self.actor.motion, Motion::Run);
    }
    fn launch(&mut self, keys: &[Command]) {
        let mut press = keys.to_vec();
        press.push(Command::Jump);
        self.step(&press);
        for _ in 0..10 {
            if self.step(keys) == ActorEvent::Jumped {
                return;
            }
        }
        panic!("normal jump must launch at the preparation animation boundary");
    }
}

#[test]
fn official_500ms_boundary_uses_press_time_not_release_time() {
    for (elapsed, runs) in [(499, true), (500, true), (501, false), (800, false)] {
        let mut r = Replay::new();
        r.tuning.logic_ms = 1.0;
        r.step(&[Command::Right]);
        for _ in 0..elapsed - 1 {
            r.step(&[]);
        }
        r.step(&[Command::Right]);
        assert_eq!(r.actor.motion == Motion::Run, runs, "interval {elapsed}ms");
    }
    let mut r = Replay::new();
    r.tuning.logic_ms = 10.0;
    for _ in 0..60 {
        r.step(&[Command::Right]);
    }
    r.step(&[]);
    r.step(&[Command::Right]);
    assert_eq!(
        r.actor.motion,
        Motion::Walk,
        "long first hold exceeds press-to-press window"
    );
}

#[test]
fn only_horizontal_consecutive_logical_commands_can_dash() {
    for key in [Command::Up, Command::Down] {
        let mut r = Replay::new();
        r.step(&[key]);
        r.step(&[]);
        r.step(&[key]);
        assert_eq!(r.actor.motion, Motion::Walk);
    }
    for middle in [
        Command::Up,
        Command::Down,
        Command::Left,
        Command::Attack,
        Command::Skill,
    ] {
        let mut r = Replay::new();
        r.step(&[Command::Right]);
        r.step(&[middle]);
        r.step(&[Command::Right]);
        assert_eq!(
            r.actor.dash_candidate, 0,
            "intervening {middle:?} breaks command suffix"
        );
        assert_eq!(
            r.actor.motion,
            if middle == Command::Attack {
                Motion::Attack
            } else if middle == Command::Skill {
                Motion::UpperSlash
            } else {
                Motion::Walk
            }
        );
    }
}

#[test]
fn opposing_mash_candidates_are_not_effective_run_state() {
    for (hold, tap) in [
        (Command::Right, Command::Left),
        (Command::Left, Command::Right),
    ] {
        let mut r = Replay::new();
        r.step(&[hold]);
        let mut saw_candidate = false;
        for _ in 0..40 {
            r.step(&[hold, tap]);
            saw_candidate |= r.actor.dash_candidate != 0;
            assert_eq!(r.actor.motion, Motion::Walk);
            r.step(&[hold]);
            assert_eq!(r.actor.motion, Motion::Walk);
        }
        assert!(
            saw_candidate,
            "official input records can match even when state does not sustain"
        );
    }
}

#[test]
fn reverse_cancels_run_and_releasing_override_does_not_restore_it() {
    let mut r = Replay::new();
    r.run();
    r.step(&[Command::Right, Command::Left]);
    assert_eq!(r.actor.motion, Motion::Walk);
    assert_eq!(r.actor.facing, -1);
    r.step(&[Command::Right]);
    assert_eq!(r.actor.motion, Motion::Walk);
    assert_eq!(r.actor.facing, 1);
}

#[test]
fn vertical_only_input_keeps_an_existing_run_and_keeps_facing() {
    let mut r = Replay::new();
    r.run();
    r.step(&[Command::Up]);
    assert_eq!(r.actor.motion, Motion::Run);
    assert_eq!(r.actor.facing, 1);
    r.step(&[]);
    assert_eq!(r.actor.motion, Motion::Stand);
}

#[test]
fn preparing_and_landing_lock_both_plane_axes_and_turning() {
    let mut r = Replay::new();
    r.step(&[Command::Right, Command::Jump]);
    let pos = (r.actor.x, r.actor.y);
    assert_eq!(r.actor.jump_phase(), JumpPhase::Preparing);
    for _ in 0..3 {
        r.step(&[Command::Left, Command::Up]);
        assert_eq!((r.actor.x, r.actor.y), pos);
        assert_eq!(r.actor.facing, 1);
    }
    while r.actor.jump_phase() != JumpPhase::Landing {
        r.step(&[Command::Left, Command::Up]);
    }
    let pos = (r.actor.x, r.actor.y);
    let facing = r.actor.facing;
    assert_eq!(r.actor.anim().frame(), 15);
    for _ in 0..2 {
        r.step(&[Command::Right, Command::Down, Command::Jump]);
        assert_eq!((r.actor.x, r.actor.y), pos);
        assert_eq!(r.actor.facing, facing);
        assert!(!r.actor.is_airborne());
    }
}

#[test]
fn normal_air_axis_y_integer_halves_before_scaling() {
    let t = MoveTuning::default();
    assert_eq!(
        t.air_velocity_for(1000.0, MoveMode::Walk, (0, 1)),
        (0.0, 57.0)
    );
    assert_eq!(
        t.air_velocity_for(1000.0, MoveMode::Walk, (1, 1)),
        (119.0, 47.0)
    );
    assert_eq!(
        t.air_velocity_for(1000.0, MoveMode::Run, (0, 1)),
        (0.0, 68.0)
    );
    assert_eq!(
        t.air_velocity_for(1000.0, MoveMode::Run, (1, 1)),
        (238.0, 57.0)
    );
}

#[test]
fn jump_uses_takeoff_mode_and_returns_to_walk_after_landing() {
    let mut r = Replay::new();
    r.run();
    r.launch(&[Command::Right]);
    let pos = (r.actor.x, r.actor.y);
    for _ in 0..10 {
        r.step(&[Command::Right, Command::Down]);
    }
    assert!((r.actor.x - pos.0 - 34).abs() <= 1);
    assert!((r.actor.y - pos.1 - 8).abs() <= 1);
    for _ in 0..100 {
        r.step(&[Command::Right]);
    }
    assert_eq!(r.actor.motion, Motion::Walk);
}

#[test]
fn physical_apex_selects_frame_eight_and_ground_selects_fifteen() {
    let mut r = Replay::new();
    r.launch(&[]);
    let mut frame_eight = false;
    for _ in 0..100 {
        let event = r.step(&[]);
        frame_eight |= r.actor.anim().frame() == 8;
        if event == ActorEvent::Landed {
            assert!(frame_eight);
            assert_eq!(r.actor.anim().frame(), 15);
            return;
        }
    }
    panic!("jump never landed");
}

#[test]
fn move_run_animation_rate_uses_movement_attribute_jump_and_idle_use_100() {
    let mut r = Replay::new();
    assert_eq!(r.actor.anim().rate_percent(), 100.0);
    r.step(&[Command::Right]);
    assert_eq!(r.actor.anim().rate_percent(), 85.0);
    r.step(&[]);
    r.step(&[Command::Right]);
    assert_eq!(r.actor.anim().rate_percent(), 85.0);
    r.step(&[Command::Right, Command::Jump]);
    assert_eq!(r.actor.anim().rate_percent(), 100.0);
}

#[test]
fn teleport_clears_dash_history_and_phase_even_if_key_stays_held() {
    let mut r = Replay::new();
    r.run();
    r.actor.teleport(400, 200);
    r.step(&[Command::Right]);
    assert_eq!(r.actor.motion, Motion::Walk);
    assert!(r.actor.x > 400);
    assert!(!r.actor.is_running_armed());
    r.actor.teleport(400, 200);
    r.step(&[]);
    r.step(&[Command::Right]);
    assert_eq!(r.actor.motion, Motion::Walk);
}

#[test]
fn float_position_rounding_preserves_fraction_when_stopped_and_reversed() {
    let mut m = Mover::new();
    let p = m.step_position((10, 10), (4.0, 0.0), 0.1);
    assert_eq!(p, (10, 10));
    assert_eq!(m.step_position(p, (0.0, 0.0), 1.0), p);
    let p = m.step_position(p, (4.0, 0.0), 0.1);
    assert_eq!(p, (11, 10));
    assert_eq!(m.step_position(p, (-4.0, 0.0), 0.1), (10, 10));
    assert_eq!(m.step_position((50, 10), (0.0, 0.0), 0.1), (50, 10));
}

#[test]
fn jump_integer_formula_handles_invalid_inputs_without_nan() {
    assert_eq!(launch_velocity(430.0, 68000.0, 0.0), 0.0);
    assert_eq!(launch_velocity(430.0, f32::NAN, 95.0), 0.0);
    assert_eq!(launch_velocity(-1.0, 68000.0, 95.0), 0.0);
    assert_eq!(
        launch_velocity(430.9, 68000.9, 95.9),
        launch_velocity(430.0, 68000.0, 95.0)
    );
}

#[test]
fn direction_slots_classify_velocity_with_five_pixel_deadzone() {
    let mut r = Replay::new();
    r.actor.spec.move_speed = 20.0;
    for _ in 0..20 {
        r.step(&[Command::Left]);
    }
    assert_eq!(r.actor.motion, Motion::Stand);
    assert_eq!(r.actor.facing, 1);
    assert!(
        r.actor.x < 300,
        "small nonzero speed still integrates position"
    );
}

#[test]
fn thirty_sixty_hz_intervals_are_inside_the_500ms_window() {
    let mut r = Replay::new();
    r.step(&[Command::Right]);
    for _ in 0..29 {
        r.step(&[]);
    }
    r.step(&[Command::Right]);
    assert_eq!(r.actor.motion, Motion::Run);
}

#[test]
fn focus_loss_clears_dash_gestures_and_preserves_jump_physics() {
    let mut r = Replay::new();
    r.run();
    r.actor.clear_direction_input();
    r.step(&[Command::Right]);
    assert_eq!(r.actor.motion, Motion::Walk);
    r.launch(&[]);
    r.step(&[]);
    let altitude = r.actor.altitude();
    r.actor.clear_direction_input();
    assert!(r.actor.is_airborne());
    assert_eq!(r.actor.altitude(), altitude);
    r.step(&[]);
    assert!(r.actor.altitude() > altitude);
}
