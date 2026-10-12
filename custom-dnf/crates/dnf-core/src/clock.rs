//! 固定步长逻辑时钟。
//!
//! 官方动画与引擎时钟按经过毫秒推进；未证明原版为固定60Hz。
//! 重制世界采用300Hz固定步长，60Hz保留为旧回放的参考时间单位。
//! 真实耗时的余数与追帧积压留到下次，避免提高频率后丢时间。

/// 逻辑步长（毫秒/步）。
pub const LOGIC_MS: f64 = 1000.0 / 60.0;
/// Remake world simulation quantum. 60Hz remains the legacy replay timebase;
/// official timing evidence is elapsed milliseconds, not a proven fixed60Hz.
pub const WORLD_HZ: u32 = 300;
pub const WORLD_MS: f64 = 1000.0 / WORLD_HZ as f64;

#[derive(Clone, Debug)]
pub struct FixedClock {
    accumulator_ms: f64,
    step_ms: f64,
    /// 单帧最多推进多少逻辑步（防止卡顿后爆炸）。
    pub max_steps_per_frame: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn higher_rate_catch_up_retains_elapsed_time_instead_of_slowing_the_world() {
        for hz in [60, 120, 300] {
            let dt = 1000.0 / hz as f64;
            let mut clock = FixedClock::new(dt);
            let mut total = clock.advance(1000.0);
            while clock.pending_ms() + 1e-7 >= dt {
                total += clock.advance(0.0);
            }
            assert_eq!(total, hz);
            assert!(clock.pending_ms() < 1e-7);
        }
    }
    #[test]
    fn three_hundred_updates_cover_one_second_despite_wall_clock_jitter() {
        let mut clock = FixedClock::new(WORLD_MS);
        let mut total = 0;
        for _ in 0..100 {
            total += clock.advance(1.0);
            total += clock.advance(2.0);
            total += clock.advance(7.0);
        }
        assert_eq!(total, WORLD_HZ);
    }
}

impl Default for FixedClock {
    fn default() -> Self {
        Self::new(LOGIC_MS)
    }
}

impl FixedClock {
    pub fn new(step_ms: f64) -> Self {
        assert!(step_ms.is_finite() && step_ms > 0.0);
        FixedClock {
            accumulator_ms: 0.0,
            step_ms,
            max_steps_per_frame: (8.0 * LOGIC_MS / step_ms).ceil().max(1.0) as u32,
        }
    }

    pub fn step_ms(&self) -> f64 {
        self.step_ms
    }

    /// 投入 `dt_ms`，返回本次应推进的逻辑步数。
    pub fn advance(&mut self, dt_ms: f64) -> u32 {
        if dt_ms.is_finite() {
            self.accumulator_ms += dt_ms.max(0.0);
        }
        let mut steps = 0;
        while self.accumulator_ms + 1e-7 >= self.step_ms && steps < self.max_steps_per_frame {
            self.accumulator_ms -= self.step_ms;
            self.accumulator_ms = self.accumulator_ms.max(0.0);
            steps += 1;
        }
        // Keep pending wall time. Raising the rate must not cause catch-up
        // caps to discard more simulation time and slow movement/cooldowns.
        steps
    }
    pub fn pending_ms(&self) -> f64 {
        self.accumulator_ms
    }

    /// 逻辑步 → 毫秒（用于动画推进）。`sub_493D00` 的动画时钟同样按毫秒积分。
    pub fn steps_to_ms(steps: u32) -> f64 {
        steps as f64 * LOGIC_MS
    }
}
