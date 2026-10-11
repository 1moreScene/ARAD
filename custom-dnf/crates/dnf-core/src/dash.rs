//! Compatibility API; direction commands now use the shared CNCommandChecker matcher.
use crate::{
    key_manager::{CommandChecker, Definition, Stage, Trigger, RUN_LEFT, RUN_RIGHT},
    Command, InputState, KeyMap,
};
pub const DOUBLE_TAP_MS: u64 = 500;
#[derive(Clone, Debug)]
pub struct DashRecognizer {
    checker: CommandChecker,
    last_ms: f64,
}
impl Default for DashRecognizer {
    fn default() -> Self {
        let mut checker = CommandChecker::default();
        for (id, key) in [(RUN_RIGHT, Command::Right), (RUN_LEFT, Command::Left)] {
            checker.register(Definition::new(
                id,
                vec![
                    Stage::key(key, DOUBLE_TAP_MS),
                    Stage::key(key, DOUBLE_TAP_MS),
                ],
                Trigger::DownOnce,
            ));
        }
        Self {
            checker,
            last_ms: 0.0,
        }
    }
}
impl DashRecognizer {
    pub fn reset(&mut self) {
        *self = Self::default();
    }
    pub fn update(&mut self, input: &InputState, km: &KeyMap, now_ms: f64) -> i32 {
        self.checker
            .update(input, km, (now_ms - self.last_ms).max(0.0));
        self.last_ms = now_ms;
        if self.checker.hit(RUN_RIGHT) {
            1
        } else if self.checker.hit(RUN_LEFT) {
            -1
        } else {
            0
        }
    }
}
