//! 动画播放器（`.ani` 帧序列 + `[DELAY]` 毫秒时长）。
//!
//! 官方 `.ani` 帧时长以毫秒计；播放速率由「千分值 × 0.1」给出
//! （逆向源码 `IRDActiveObject::vmt3D8` = 0x495A10 → `sub_493D00(anim, 千分值 × 0.1)`，
//! 其中 100 = 100%）。本播放器把速率换算成有效帧时长：`delay / (rate/100)`。
//!
//! `[DELAY] 10000` 是官方的「保持」标记（见 `jump.ani` 帧 7/14）：播放到该帧即停住，
//! 等待逻辑切换动画，因此它等价于「该帧无限长」。

/// 保持标记阈值（官方用 10000ms 表示「停在此帧」）。
pub const HOLD_DELAY_MS: u32 = 10000;

/// 一条动画片段（帧时长序列）。
#[derive(Clone, Debug, PartialEq)]
pub struct Clip {
    pub name: String,
    pub delays_ms: Vec<u32>,
    pub looping: bool,
}

impl Clip {
    pub fn new(name: impl Into<String>, delays_ms: Vec<u32>, looping: bool) -> Self {
        Clip {
            name: name.into(),
            delays_ms,
            looping,
        }
    }

    pub fn frame_count(&self) -> usize {
        self.delays_ms.len()
    }

    /// 是否含有官方「保持」帧。
    pub fn has_hold(&self) -> bool {
        self.delays_ms.iter().any(|d| *d >= HOLD_DELAY_MS)
    }

    /// 单圈总时长（毫秒，忽略保持帧）。
    pub fn cycle_ms(&self) -> u32 {
        self.delays_ms.iter().filter(|d| **d < HOLD_DELAY_MS).sum()
    }
}

/// 播放状态。
#[derive(Clone, Debug)]
pub struct AnimPlayer {
    clip: Clip,
    /// 播放速率百分数（100 = 100%，官方「千分值 × 0.1」）。
    rate_percent: f32,
    /// 当前帧已播放的等效毫秒。
    elapsed_ms: f64,
    frame: usize,
    finished: bool,
}

impl AnimPlayer {
    pub fn new(clip: Clip, rate_percent: f32) -> Self {
        AnimPlayer {
            clip,
            rate_percent,
            elapsed_ms: 0.0,
            frame: 0,
            finished: false,
        }
    }

    pub fn clip(&self) -> &Clip {
        &self.clip
    }

    pub fn rate_percent(&self) -> f32 {
        self.rate_percent
    }

    pub fn frame(&self) -> usize {
        self.frame
    }

    pub fn finished(&self) -> bool {
        self.finished
    }

    /// 当前帧有效时长（毫秒，已含速率换算）。
    pub fn frame_ms(&self) -> f32 {
        let raw = self.clip.delays_ms.get(self.frame).copied().unwrap_or(1);
        let rate = if self.rate_percent > 0.0 {
            self.rate_percent
        } else {
            100.0
        };
        raw as f32 * 100.0 / rate
    }

    /// 切换片段；同名同帧序片段且 `restart == false` 时**不**重置进度。
    pub fn set_clip(&mut self, clip: Clip, rate_percent: f32, restart: bool) {
        let same = self.clip.name == clip.name && self.clip.delays_ms == clip.delays_ms;
        if !same || restart {
            self.clip = clip;
            self.elapsed_ms = 0.0;
            self.frame = 0;
            self.finished = false;
        }
        self.rate_percent = rate_percent;
    }

    /// 直接把当前帧设为 `frame`（官方 `sub_4AECE0(动画对象, 帧)`）。
    ///
    /// 官方在跳跃到达顶点时用它把动画**直接跳到**顶点保持帧：
    /// `char_server_info.cpp:4913-4920` —— `z 速度 <= 0` 且当前帧 < `*(this+1717)`
    /// ⇒ `sub_4AECE0(动画对象, *(this+1717))`。
    pub fn set_frame(&mut self, frame: usize) -> bool {
        if frame >= self.clip.frame_count() || frame == self.frame {
            return false;
        }
        self.frame = frame;
        self.elapsed_ms = 0.0;
        self.finished = self.clip.delays_ms.get(frame).copied().unwrap_or(0) >= HOLD_DELAY_MS;
        true
    }

    /// 释放「保持帧」：若当前停在官方保持帧上，则由逻辑放行到下一帧。
    ///
    /// 官方 `jump.ani` 帧 7（顶点）/ 帧 14（落地前）的 `[DELAY] 10000` 就是这个语义：
    /// 动画自己不会走，必须由状态机在段落切换时推它一把。
    pub fn release_hold(&mut self) -> bool {
        if self.clip.delays_ms.get(self.frame).copied().unwrap_or(0) >= HOLD_DELAY_MS {
            self.elapsed_ms = 0.0;
            self.finished = false;
            if self.frame + 1 < self.clip.frame_count() {
                self.frame += 1;
                return true;
            }
        }
        false
    }

    /// 推进 `dt_ms` 真实毫秒。返回是否发生换帧。
    pub fn advance(&mut self, dt_ms: f64) -> bool {
        if dt_ms <= 0.0 || self.clip.delays_ms.is_empty() {
            return false;
        }
        let before = self.frame;
        self.elapsed_ms += dt_ms;
        let mut guard = 0;
        loop {
            guard += 1;
            if guard > 4096 {
                break;
            }
            // 保持帧：一旦进入就停住，等待逻辑切换动画（官方 `jump.ani` 帧 7/14）。
            let raw = self.clip.delays_ms.get(self.frame).copied().unwrap_or(1);
            if raw >= HOLD_DELAY_MS {
                self.elapsed_ms = 0.0;
                self.finished = true;
                break;
            }
            let dur = self.frame_ms().max(0.001) as f64;
            if self.elapsed_ms + 1e-7 < dur {
                break;
            }
            self.elapsed_ms = (self.elapsed_ms - dur).max(0.0);
            if self.frame + 1 < self.clip.frame_count() {
                self.frame += 1;
            } else if self.clip.looping {
                self.frame = 0;
            } else {
                self.frame = self.clip.frame_count() - 1;
                self.elapsed_ms = 0.0;
                self.finished = true;
                break;
            }
        }
        before != self.frame
    }
}
