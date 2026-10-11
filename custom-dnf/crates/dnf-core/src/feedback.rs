//! 58E920 state3 recovery and independent 49A320/4D9840 freeze clocks.
/// 4B49B0 failure starts actor+8088 for 300ms. 4B88E0 quantizes a 74ms triangle.
#[derive(Clone, Debug, Default)]
pub struct CastVibration {
    age_ms: Option<f64>,
    offset: i32,
}
impl CastVibration {
    pub fn trigger(&mut self) {
        if self.age_ms.is_none() {
            self.age_ms = Some(0.0);
            self.offset = -2;
        }
    }
    pub fn advance(&mut self, dt: f64) {
        let Some(age) = &mut self.age_ms else { return };
        *age += dt.max(0.0);
        if *age >= 300.0 {
            self.age_ms = None;
            self.offset = 0;
            return;
        }
        let phase = *age as i32 % 74;
        let value = if phase < 37 {
            2.0 * phase as f64 / 37.0
        } else {
            2.0 * (74 - phase) as f64 / 37.0
        } as i32;
        // The exact peak (2) leaves the previous offset unchanged in the EXE.
        match value {
            0 => self.offset = -2,
            1 => self.offset = 2,
            _ => {}
        }
    }
    pub fn offset(&self) -> i32 {
        self.offset
    }
    pub fn active(&self) -> bool {
        self.age_ms.is_some()
    }
}
#[derive(Clone, Copy, Debug, Default)]
pub struct HitStop {
    pub remaining_ms: f64,
}
impl HitStop {
    /// Repeated hits extend to the longer remaining duration, rather than adding.
    pub fn apply(&mut self, ms: f64) {
        if ms.is_finite() {
            self.remaining_ms = self.remaining_ms.max(ms.max(0.0));
        }
    }
    pub fn active_ms(&mut self, dt: f64) -> f64 {
        let frozen = dt.min(self.remaining_ms);
        self.remaining_ms = (self.remaining_ms - dt).max(0.0);
        let active = dt - frozen;
        if active < 1e-7 {
            0.0
        } else {
            active
        }
    }
}
pub fn recovery_ms(hit_recovery: i32, rate_percent: i32, knuck_back: bool) -> i32 {
    if knuck_back {
        return 333;
    }
    let base = ((2000 - hit_recovery).max(0) as f64 * 0.25) as i32;
    (base * rate_percent / 100).max(1)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cast_vibration_does_not_move_world_or_extend_on_repeated_rejection() {
        let mut v = CastVibration::default();
        v.trigger();
        assert_eq!(v.offset(), -2);
        v.advance(20.0);
        assert_eq!(v.offset(), 2);
        v.advance(17.0);
        assert_eq!(v.offset(), 2);
        v.advance(23.0);
        assert_eq!(v.offset(), -2);
        v.trigger();
        v.advance(240.0);
        assert!(!v.active());
        assert_eq!(v.offset(), 0);
    }
    #[test]
    fn independent_freeze_clocks_preserve_fraction_and_extend() {
        let mut a = HitStop::default();
        let mut b = HitStop::default();
        a.apply(100.0);
        b.apply(70.0);
        assert_eq!(a.active_ms(60.0), 0.0);
        assert_eq!(b.active_ms(60.0), 0.0);
        a.apply(20.0);
        assert_eq!(a.remaining_ms, 40.0);
        assert_eq!(b.active_ms(20.0), 10.0);
        assert_eq!(a.active_ms(20.0), 0.0);
    }
    #[test]
    fn official_state_three_duration_uses_script_attribute_and_percent() {
        assert_eq!(recovery_ms(500, 100, false), 375);
        assert_eq!(recovery_ms(500, 86, false), 322);
        assert_eq!(recovery_ms(500, 100, true), 333);
    }
}
