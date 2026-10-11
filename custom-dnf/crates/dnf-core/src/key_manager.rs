//! CNCommandChecker: 4C0140 records, 4BF540 matching, 4BF380 priority and 4BFBC0 queries.
use crate::{keys::Dik, Command, InputState, KeyMap, COMMAND_COUNT};

pub const ATTACK: u32 = 0;
pub const JUMP: u32 = 1;
pub const RUN_RIGHT: u32 = 2;
pub const RUN_LEFT: u32 = 3;
pub const UPPER: u32 = 46;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyRecord {
    pub command: Command,
    pub press_ms: u64,
    pub hold_ms: u64,
    pub held: bool,
    pub changed: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Trigger {
    Down = 0,
    DownOnce = 1,
    Up = 2,
    UpOnce = 3,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum MatchState {
    #[default]
    Idle = 0,
    Hit = 1,
    Held = 2,
    Suppressed = 3,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stage {
    pub keys: Vec<Command>,
    pub hold_ms: u64,
    pub window_ms: u64,
}
impl Stage {
    pub fn key(key: Command, window_ms: u64) -> Self {
        Self {
            keys: vec![key],
            hold_ms: 0,
            window_ms,
        }
    }
}
#[derive(Clone, Debug)]
pub struct Definition {
    pub id: u32,
    pub stages: Vec<Stage>,
    pub required_held: Vec<Command>,
    pub trigger: Trigger,
    pub enabled: bool,
    pub state: MatchState,
}
impl Definition {
    pub fn new(id: u32, stages: Vec<Stage>, trigger: Trigger) -> Self {
        Self {
            id,
            stages,
            required_held: Vec::new(),
            trigger,
            enabled: true,
            state: MatchState::Idle,
        }
    }
    fn higher_priority_than(&self, other: &Self) -> bool {
        if self.stages.len() != other.stages.len() {
            return self.stages.len() > other.stages.len();
        }
        let mut hold_order = None;
        let mut window_order = None;
        for (a, b) in self.stages.iter().rev().zip(other.stages.iter().rev()) {
            if a.keys.len() != b.keys.len() {
                return a.keys.len() > b.keys.len()
                    && b.keys.iter().all(|key| a.keys.contains(key));
            }
            if !a.keys.iter().all(|key| b.keys.contains(key)) {
                return false;
            }
            if hold_order.is_none() && a.hold_ms != b.hold_ms {
                hold_order = Some(a.hold_ms > b.hold_ms);
            }
            if window_order.is_none() && a.window_ms != b.window_ms {
                window_order = Some(a.window_ms < b.window_ms);
            }
        }
        if self.required_held.len() != other.required_held.len() {
            return self.required_held.len() > other.required_held.len();
        }
        hold_order.or(window_order).unwrap_or(false)
    }
    /// 502F00 grammar. A missing block is distinct from an explicitly empty command.
    pub fn parse_skill(id: u32, text: &str) -> Result<Option<Self>, String> {
        let lower = text.to_ascii_lowercase();
        let Some(at) = lower.find("[command]") else {
            return Ok(None);
        };
        let end = lower[at + 9..]
            .find("[/command]")
            .ok_or("unterminated command")?
            + at
            + 9;
        let body = text[at + 9..end]
            .lines()
            .map(|line| line.split("//").next().unwrap_or(""))
            .collect::<Vec<_>>()
            .join(" ");
        let mut chars = body.chars().peekable();
        let mut keys = Vec::new();
        let mut held = Vec::new();
        let mut stages = Vec::new();
        let mut hold_ms = 0;
        let mut trigger = Trigger::DownOnce;
        while let Some(c) = chars.next() {
            match c {
                c if c.is_whitespace() => {}
                '(' => {
                    let mut key = String::new();
                    let mut closed = false;
                    for c in chars.by_ref() {
                        if c == ')' {
                            closed = true;
                            break;
                        }
                        key.push(c);
                    }
                    if !closed {
                        return Err("unterminated command key".into());
                    }
                    let token = format!("({})", key.trim().to_ascii_uppercase());
                    keys.push(
                        Command::ALL[..9]
                            .iter()
                            .copied()
                            .find(|c| c.token() == Some(token.as_str()))
                            .ok_or_else(|| format!("unknown command {token}"))?,
                    );
                }
                '[' => {
                    let mut mode = String::new();
                    let mut closed = false;
                    for c in chars.by_ref() {
                        if c == ']' {
                            closed = true;
                            break;
                        }
                        mode.push(c);
                    }
                    if !closed {
                        return Err("unterminated trigger".into());
                    }
                    trigger = match mode.trim().to_ascii_uppercase().as_str() {
                        "DOWN" => Trigger::Down,
                        "DOWN ONCE" => Trigger::DownOnce,
                        "UP" => Trigger::Up,
                        "UP ONCE" => Trigger::UpOnce,
                        _ => return Err(format!("unknown trigger {mode}")),
                    };
                }
                '&' => {
                    held.append(&mut keys);
                    hold_ms = 0;
                }
                ',' | '!' | '~' => {
                    if !keys.is_empty() {
                        stages.push(Stage {
                            keys: std::mem::take(&mut keys),
                            hold_ms,
                            window_ms: match c {
                                '!' => 150,
                                '~' => 1000,
                                _ => 300,
                            },
                        });
                    }
                    hold_ms = 0;
                }
                c if c.is_ascii_digit() => {
                    let mut number = String::from(c);
                    while chars.peek().is_some_and(|c| c.is_ascii_digit()) {
                        number.push(chars.next().unwrap());
                    }
                    hold_ms = number
                        .parse()
                        .map_err(|_| "invalid command hold duration")?;
                }
                _ => return Err(format!("unexpected command character {c}")),
            }
        }
        if !keys.is_empty() {
            stages.push(Stage {
                keys,
                hold_ms,
                window_ms: 300,
            });
        }
        let mut definition = Self::new(id, stages, trigger);
        definition.required_held = held;
        Ok(Some(definition))
    }
}

#[derive(Clone, Debug)]
pub struct CommandChecker {
    pub records: Vec<KeyRecord>,
    pub definitions: Vec<Definition>,
    pub retention_ms: u64,
    elapsed_ms: f64,
    pub now_ms: u64,
    bindings: Option<[Dik; COMMAND_COUNT]>,
    down: [bool; COMMAND_COUNT],
    pressed: [bool; COMMAND_COUNT],
    released: [bool; COMMAND_COUNT],
    channels: [bool; 4],
    forced: [bool; 9],
}
impl Default for CommandChecker {
    fn default() -> Self {
        Self {
            records: Vec::new(),
            definitions: Vec::new(),
            retention_ms: 10000,
            elapsed_ms: 0.0,
            now_ms: 0,
            bindings: None,
            down: [false; COMMAND_COUNT],
            pressed: [false; COMMAND_COUNT],
            released: [false; COMMAND_COUNT],
            channels: [true; 4],
            forced: [false; 9],
        }
    }
}
impl CommandChecker {
    pub fn swordman() -> Self {
        let mut out = Self::default();
        for (id, key) in [
            (ATTACK, Command::Attack),
            (JUMP, Command::Jump),
            (UPPER, Command::Skill),
        ] {
            out.register(Definition::new(
                id,
                vec![Stage::key(key, 300)],
                Trigger::DownOnce,
            ));
        }
        for (id, key) in [(RUN_RIGHT, Command::Right), (RUN_LEFT, Command::Left)] {
            out.register(Definition::new(
                id,
                vec![Stage::key(key, 500), Stage::key(key, 500)],
                Trigger::DownOnce,
            ));
        }
        out
    }
    pub fn register(&mut self, mut definition: Definition) {
        self.definitions.retain(|d| d.id != definition.id);
        definition.state = MatchState::Idle;
        self.retention_ms = self.retention_ms.max(
            definition
                .stages
                .iter()
                .map(|s| s.window_ms.saturating_add(s.hold_ms.saturating_mul(2)))
                .fold(0, u64::saturating_add),
        );
        let index = self
            .definitions
            .iter()
            .position(|d| definition.higher_priority_than(d))
            .unwrap_or(self.definitions.len());
        self.definitions.insert(index, definition);
    }
    pub fn enable(&mut self, id: u32, value: bool) {
        if let Some(d) = self.definitions.iter_mut().find(|d| d.id == id) {
            d.enabled = value;
        }
    }
    pub fn consume(&mut self, id: u32) {
        if let Some(d) = self.definitions.iter_mut().find(|d| d.id == id) {
            d.state = MatchState::Suppressed;
        }
    }
    pub fn hit(&self, id: u32) -> bool {
        self.definitions
            .iter()
            .any(|d| d.id == id && d.enabled && d.state == MatchState::Hit)
    }
    pub fn is_down(&self, key: Command) -> bool {
        self.down[key.index()]
    }
    pub fn just_pressed(&self, key: Command) -> bool {
        self.pressed[key.index()]
    }
    pub fn just_released(&self, key: Command) -> bool {
        self.released[key.index()]
    }
    pub fn last_held(&self, allowed: &[Command]) -> Option<Command> {
        self.records
            .iter()
            .rev()
            .find(|r| r.held && allowed.contains(&r.command))
            .map(|r| r.command)
    }
    pub fn clear(&mut self) {
        self.records.clear();
        self.down.fill(false);
        self.pressed.fill(false);
        self.released.fill(false);
        self.forced.fill(false);
        for d in &mut self.definitions {
            d.state = MatchState::Idle;
        }
    }
    pub fn set_channel(&mut self, channel: usize, enabled: bool) {
        if let Some(c) = self.channels.get_mut(channel) {
            *c = enabled;
        }
    }
    pub fn force(&mut self, key: Command, held: bool) {
        if key.index() < 9 {
            self.forced[key.index()] = held;
        }
    }
    pub fn update(&mut self, input: &InputState, km: &KeyMap, dt_ms: f64) {
        self.elapsed_ms += dt_ms.max(0.0);
        self.now_ms = self.elapsed_ms.round() as u64;
        let bindings = Command::ALL.map(|key| km.get(key));
        if self.bindings != Some(bindings) {
            let forced = self.forced;
            self.clear();
            self.forced = forced;
            self.bindings = Some(bindings);
        }
        let stale = self
            .records
            .iter()
            .take_while(|r| !r.held && r.press_ms.saturating_add(self.retention_ms) < self.now_ms)
            .count();
        self.records.drain(..stale);
        for r in &mut self.records {
            r.changed = false;
            if r.held {
                r.hold_ms = self.now_ms.saturating_sub(r.press_ms);
            }
        }
        self.pressed.fill(false);
        self.released.fill(false);
        for key in Command::ALL {
            let index = key.index();
            let value = self.channels[0] && input.is_down(bindings[index])
                || index < 9 && self.channels[2] && self.forced[index];
            if index < 9 {
                let previous = self
                    .records
                    .iter()
                    .rposition(|r| r.command == key && r.held);
                match (value, previous) {
                    (true, None) => {
                        // Edge-free focus/rebind baseline preserves held direction without new action presses.
                        let changed = input.just_pressed(bindings[index]) || self.forced[index];
                        self.records.push(KeyRecord {
                            command: key,
                            press_ms: self.now_ms,
                            hold_ms: 0,
                            held: true,
                            changed,
                        });
                        self.pressed[index] = changed;
                    }
                    (false, Some(at)) => {
                        let record = &mut self.records[at];
                        record.held = false;
                        record.changed = true;
                        self.released[index] = true;
                    }
                    _ => {}
                }
            } else {
                self.pressed[index] = value && input.just_pressed(bindings[index]);
                self.released[index] = !value && self.down[index];
            }
            self.down[index] = value;
        }
        let mut blocked = false;
        for definition in &mut self.definitions {
            if blocked {
                definition.state = MatchState::Idle;
                continue;
            }
            match_definition(definition, &self.records, self.now_ms, &self.down);
            blocked = matches!(definition.state, MatchState::Hit | MatchState::Held);
        }
    }
}
fn match_definition(
    d: &mut Definition,
    records: &[KeyRecord],
    now: u64,
    down: &[bool; COMMAND_COUNT],
) {
    if !d.enabled && d.state == MatchState::Idle {
        return;
    }
    let Some(stage) = d.stages.last() else {
        d.state = MatchState::Idle;
        return;
    };
    let Some(last) = records.last() else {
        d.state = MatchState::Idle;
        return;
    };
    let all_held = stage.keys.iter().all(|k| down[k.index()]);
    match d.state {
        MatchState::Hit => {
            match d.trigger {
                Trigger::Down => {
                    if !all_held {
                        d.state = MatchState::Idle;
                    }
                }
                Trigger::DownOnce => {
                    d.state = if !all_held {
                        MatchState::Idle
                    } else if d.enabled {
                        MatchState::Held
                    } else {
                        MatchState::Suppressed
                    };
                }
                Trigger::Up => {
                    if last.held {
                        d.state = MatchState::Idle;
                    }
                }
                Trigger::UpOnce => d.state = MatchState::Idle,
            }
            return;
        }
        MatchState::Held | MatchState::Suppressed => {
            if d.state == MatchState::Held && !d.enabled {
                d.state = MatchState::Suppressed;
            }
            if d.trigger == Trigger::DownOnce && !all_held {
                d.state = MatchState::Idle;
            }
            return;
        }
        MatchState::Idle => {}
    }
    if !d.required_held.iter().all(|key| down[key.index()]) {
        return;
    }
    let valid_trigger = match d.trigger {
        Trigger::Down => last.held,
        Trigger::DownOnce => last.held && (stage.hold_ms > 0 || last.changed),
        Trigger::Up => !last.held,
        Trigger::UpOnce => !last.held && last.changed,
    };
    if !valid_trigger {
        return;
    }
    let mut end = records.len();
    let mut following_press = 0;
    for stage in d.stages.iter().rev() {
        let n = stage.keys.len();
        if n == 0 || end < n {
            return;
        }
        let block = &records[end - n..end];
        if !stage
            .keys
            .iter()
            .all(|key| block.iter().any(|r| r.command == *key))
        {
            return;
        }
        let latest = block.last().unwrap().press_ms;
        if stage.hold_ms == 0 && latest.saturating_sub(block[0].press_ms) > 100 {
            return;
        }
        let overlap_end = block
            .iter()
            .map(|r| r.press_ms.saturating_add(r.hold_ms))
            .min()
            .unwrap()
            .min(now);
        let window_start = if stage.hold_ms > 0 {
            overlap_end
        } else {
            latest
        };
        if window_start.saturating_add(stage.window_ms) < following_press
            || overlap_end < latest
            || overlap_end - latest < stage.hold_ms
        {
            return;
        }
        following_press = latest;
        end -= n;
    }
    d.state = MatchState::Hit;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn minimum_hold_required_keys_forced_channel_and_shortcuts() {
        let mut m = CommandChecker::default();
        let mut stage = Stage::key(Command::Skill, 300);
        stage.hold_ms = 150;
        let mut definition = Definition::new(99, vec![stage], Trigger::DownOnce);
        definition.required_held = vec![Command::Up];
        m.register(definition);
        let mut i = InputState::new();
        sample(&mut m, &mut i, &[Command::Skill, Command::Up], 1.0);
        sample(&mut m, &mut i, &[Command::Skill, Command::Up], 149.0);
        assert!(!m.hit(99));
        sample(&mut m, &mut i, &[Command::Skill, Command::Up], 1.0);
        assert!(m.hit(99));
        let mut m = CommandChecker::swordman();
        m.set_channel(0, false);
        m.force(Command::Attack, true);
        sample(&mut m, &mut i, &[], 10.0);
        assert!(m.hit(ATTACK));
        m.set_channel(2, false);
        sample(&mut m, &mut i, &[], 10.0);
        assert!(m.just_released(Command::Attack));
        let mut m = CommandChecker::swordman();
        let mut i = InputState::new();
        sample(&mut m, &mut i, &[Command::Right], 1.0);
        sample(&mut m, &mut i, &[Command::SkillSlot1], 10.0);
        assert!(m.just_pressed(Command::SkillSlot1));
        assert_eq!(m.records.len(), 1);
        sample(&mut m, &mut i, &[Command::Right], 10.0);
        assert!(m.hit(RUN_RIGHT));
    }
    fn sample(m: &mut CommandChecker, input: &mut InputState, keys: &[Command], dt: f64) {
        let km = KeyMap::official_default();
        input.begin_frame();
        for &key in keys {
            input.set(km.get(key), true);
        }
        m.update(input, &km, dt);
    }
    #[test]
    fn records_capture_hold_release_changes_and_direction_from_one_history() {
        let mut m = CommandChecker::swordman();
        let mut i = InputState::new();
        sample(&mut m, &mut i, &[Command::Right], 10.0);
        sample(&mut m, &mut i, &[Command::Right, Command::Left], 25.0);
        assert_eq!(
            m.last_held(&[Command::Left, Command::Right]),
            Some(Command::Left)
        );
        sample(&mut m, &mut i, &[Command::Right], 15.0);
        assert!(m.just_released(Command::Left));
        assert_eq!(m.records[1].hold_ms, 15);
        assert_eq!(
            m.last_held(&[Command::Left, Command::Right]),
            Some(Command::Right)
        );
        sample(&mut m, &mut i, &[Command::Right], 5.0);
        assert!(!m.records[1].changed);
        assert_eq!(m.records[0].hold_ms, 45);
    }
    #[test]
    fn four_trigger_modes_and_disable_wait_for_release() {
        for mode in [
            Trigger::Down,
            Trigger::DownOnce,
            Trigger::Up,
            Trigger::UpOnce,
        ] {
            let mut m = CommandChecker::default();
            m.register(Definition::new(
                10,
                vec![Stage::key(Command::Attack, 300)],
                mode,
            ));
            let mut i = InputState::new();
            sample(&mut m, &mut i, &[Command::Attack], 10.0);
            assert_eq!(m.hit(10), matches!(mode, Trigger::Down | Trigger::DownOnce));
            sample(&mut m, &mut i, &[Command::Attack], 10.0);
            assert_eq!(m.hit(10), mode == Trigger::Down);
            sample(&mut m, &mut i, &[], 10.0);
            assert_eq!(m.hit(10), matches!(mode, Trigger::Up | Trigger::UpOnce));
            sample(&mut m, &mut i, &[], 10.0);
            assert_eq!(m.hit(10), mode == Trigger::Up);
        }
        let mut m = CommandChecker::swordman();
        let mut i = InputState::new();
        sample(&mut m, &mut i, &[Command::Attack], 1.0);
        m.enable(ATTACK, false);
        sample(&mut m, &mut i, &[Command::Attack], 1.0);
        m.enable(ATTACK, true);
        sample(&mut m, &mut i, &[Command::Attack], 1.0);
        assert!(!m.hit(ATTACK));
        sample(&mut m, &mut i, &[], 1.0);
        sample(&mut m, &mut i, &[Command::Attack], 1.0);
        assert!(m.hit(ATTACK));
    }
    #[test]
    fn suffix_priority_and_100ms_chord_window() {
        for (gap, expected) in [(100, true), (101, false)] {
            let mut m = CommandChecker::swordman();
            m.register(Definition::new(
                50,
                vec![Stage {
                    keys: vec![Command::Up, Command::Attack],
                    hold_ms: 0,
                    window_ms: 300,
                }],
                Trigger::DownOnce,
            ));
            let mut i = InputState::new();
            sample(&mut m, &mut i, &[Command::Up], 1.0);
            sample(&mut m, &mut i, &[Command::Up, Command::Attack], gap as f64);
            assert_eq!(m.hit(50), expected);
            assert_eq!(m.hit(ATTACK), !expected);
        }
    }
    #[test]
    fn held_history_blocks_prefix_pruning_and_registration_extends_retention() {
        let mut m = CommandChecker::default();
        let mut i = InputState::new();
        sample(&mut m, &mut i, &[Command::Up], 1.0);
        sample(&mut m, &mut i, &[Command::Up, Command::Attack], 1.0);
        sample(&mut m, &mut i, &[Command::Up], 11000.0);
        assert_eq!(m.records.len(), 2);
        sample(&mut m, &mut i, &[], 1.0);
        sample(&mut m, &mut i, &[], 1.0);
        assert!(m.records.is_empty());
        let mut s = Stage::key(Command::Skill, 1000);
        s.hold_ms = 6000;
        m.register(Definition::new(9, vec![s], Trigger::DownOnce));
        assert_eq!(m.retention_ms, 13000);
    }
    #[test]
    fn parse_skill_command_and_required_keys_hold_and_release() {
        let d = Definition::parse_skill(
            99,
            "[command]\n(UP)&(RIGHT)!, (SKILL) 150 [UP ONCE]\n[/command]",
        )
        .unwrap()
        .unwrap();
        assert_eq!(d.required_held, vec![Command::Up]);
        assert_eq!(d.stages[0].window_ms, 150);
        assert_eq!(d.stages[1].hold_ms, 150);
        assert_eq!(d.trigger, Trigger::UpOnce);
        assert!(Definition::parse_skill(0, "[command] (BOGUS) [/command]").is_err());
        assert!(Definition::parse_skill(0, "[command][/command]")
            .unwrap()
            .unwrap()
            .stages
            .is_empty());
        assert!(Definition::parse_skill(0, "[name2] `Upper Slash`")
            .unwrap()
            .is_none());
    }
    #[test]
    fn baseline_and_hot_rebinding_do_not_create_action_presses() {
        let km = KeyMap::official_default();
        let mut input = InputState::new();
        let mut m = CommandChecker::swordman();
        input.set(km.get(Command::Attack), true);
        input.begin_frame();
        input.set(km.get(Command::Attack), true);
        m.update(&input, &km, 10.0);
        assert!(m.is_down(Command::Attack));
        assert!(!m.hit(ATTACK));
        let mut remapped = km.clone();
        remapped.set(Command::Attack, km.get(Command::Skill));
        input.begin_frame();
        input.set(km.get(Command::Skill), true);
        m.update(&input, &remapped, 10.0);
        assert!(m.hit(ATTACK));
        remapped.set(Command::Jump, km.get(Command::Skill));
        input.begin_frame();
        input.set(km.get(Command::Skill), true);
        m.update(&input, &remapped, 10.0);
        assert!(!m.hit(JUMP));
    }
}
