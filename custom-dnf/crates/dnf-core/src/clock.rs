//! 固定步长逻辑时钟。
//!
//! DNF 的动画 `[DELAY]` 以毫秒为单位（`move.ani` = 8 帧 × 100ms），逻辑按固定
//! 步长推进。本模块把真实耗时（毫秒，可为小数）切成整数个 1/60 s 的步长，
//! 余数留到下一帧，保证逻辑帧数与真实时间长期一致。

/// 逻辑步长（毫秒/步）。
pub const LOGIC_MS: f64 = 1000.0 / 60.0;

#[derive(Clone, Debug)]
pub struct FixedClock {
    accumulator_ms: f64,
    step_ms: f64,
    /// 单帧最多推进多少逻辑步（防止卡顿后爆炸）。
    pub max_steps_per_frame: u32,
}

impl Default for FixedClock {
    fn default() -> Self {
        Self::new(LOGIC_MS)
    }
}

impl FixedClock {
    pub fn new(step_ms: f64) -> Self {
        FixedClock {
            accumulator_ms: 0.0,
            step_ms,
            max_steps_per_frame: 8,
        }
    }

    pub fn step_ms(&self) -> f64 {
        self.step_ms
    }

    /// 投入 `dt_ms`，返回本次应推进的逻辑步数。
    pub fn advance(&mut self, dt_ms: f64) -> u32 {
        self.accumulator_ms += dt_ms.max(0.0);
        let mut steps = 0;
        while self.accumulator_ms >= self.step_ms && steps < self.max_steps_per_frame {
            self.accumulator_ms -= self.step_ms;
            steps += 1;
        }
        if steps == self.max_steps_per_frame {
            // 丢弃积压，避免追帧雪崩。
            self.accumulator_ms = self.accumulator_ms.min(self.step_ms);
        }
        steps
    }

    /// 逻辑步 → 毫秒（用于动画推进）。`sub_493D00` 的动画时钟同样按毫秒积分。
    pub fn steps_to_ms(steps: u32) -> f64 {
        steps as f64 * LOGIC_MS
    }
}
