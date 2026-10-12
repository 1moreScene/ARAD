//! 键盘状态容器（当前帧 / 上一帧），与官方输入对象同构。
//!
//! 官方 `sub_415190` 清出两张 256 字节表（当前帧 `+269`、上一帧 `+525`），
//! `sub_415770` 每帧把「当前 → 上一帧」再采样一次键盘设备状态（415840是手柄）；据此可以区分
//! 「按下」「按住」「抬起」「本帧刚按下」四种边沿。本模块是它的等价实现。

use crate::keys::Dik;

/// 键盘状态：两张 256 字节表。
#[derive(Clone)]
pub struct InputState {
    down: [bool; 256],
    prev: [bool; 256],
}

impl Default for InputState {
    fn default() -> Self {
        Self::new()
    }
}

impl InputState {
    pub const fn new() -> Self {
        InputState {
            down: [false; 256],
            prev: [false; 256],
        }
    }

    /// 每帧开头调用：把当前帧快照为上一帧，并清空当前帧。
    pub fn begin_frame(&mut self) {
        self.prev = self.down;
        self.down = [false; 256];
    }

    /// 采样一个扫描码（等价 `sub_415770` 的设备状态写入）。
    pub fn set(&mut self, dik: Dik, pressed: bool) {
        self.down[dik.code() as usize] = pressed;
    }
    /// Internal substeps retain held keys, but a physical edge is delivered once.
    pub fn settle_edges(&mut self) {
        self.prev = self.down;
    }

    pub fn is_down(&self, dik: Dik) -> bool {
        self.down[dik.code() as usize]
    }

    pub fn was_down(&self, dik: Dik) -> bool {
        self.prev[dik.code() as usize]
    }

    /// 本帧刚按下（上升沿）。
    pub fn just_pressed(&self, dik: Dik) -> bool {
        self.down[dik.code() as usize] && !self.prev[dik.code() as usize]
    }

    /// 本帧刚抬起（下降沿）。
    pub fn just_released(&self, dik: Dik) -> bool {
        !self.down[dik.code() as usize] && self.prev[dik.code() as usize]
    }
}

/// 一个轴上的方向键解析（官方语义：**后按的键优先**，不是两键相抵）。
///
/// # 官方证据
///
/// * 键盘设备每帧维护「当前帧 / 上一帧」按键表并对外提供**上升沿查询**
///   （`sub_415200(this, slot)`，`src/nengine/gfx_texture.cpp:5013`：
///   `v2[269] != 0 && v2[525] == 0`），说明「哪个键是刚按下的」在官方是被追踪的状态；
/// * 每个轴各有一个输入容器（`unk_9A0310` = x 轴、`unk_9A0320` = y 轴，
///   `src/game/equ_buff.cpp:331-333`），取方向用 `sub_4BF3D0`
///   （`src/game/equ_buff.cpp:13056`）：
///
///   ```c
///   for (i = 最后一条记录; ; i -= 16) {                 // ★ 从**尾**向前扫
///       if (*(uint8_t *)(i + 12) != 0                       // 该记录仍「有效」（键还按着）
///        && 记录 id ∈ 允许集) { 命中 = i; break; }
///       if (--v4 < 0) return -1;                            // 全部无效 → 无输入
///   }
///   return *(uint32_t *)命中;                               // 返回**最近**那条
///   ```
///
///   ⇒ **反着扫、取最近一条仍有效的记录** = 「后按覆盖先按」；
///   松开某键时4C0140更新held/changed，4BF450仅查询，于是**自动回落到仍按住的另一个键** ——
///   按住 ← 再按 → ⇒ 向右；松开 → ⇒ 又向左。这正与官方「方向值 4 = 无输入」互补：
///   只有**两键都松开**才应该出现「无方向」。
/// * `sub_4B04B0`（`src/game/equ_buff.cpp:325`）再把该值映射成 0..3 / 4（无输入），
///   供状态机与 `vmt37C` 使用。
///
/// 两轴各自独立 ⇒ 同时按 ←↑ 得到斜向（官方参数表里另给斜向档 119/95 也印证了这一点）。
#[derive(Clone, Debug)]
pub struct AxisInput {
    neg: Dik,
    pos: Dik,
    /// 按下顺序（后按的在后）；只可能含这两个键。
    stack: Vec<Dik>,
}

impl Default for AxisInput {
    fn default() -> Self {
        AxisInput {
            neg: Dik::new(0),
            pos: Dik::new(0),
            stack: Vec::new(),
        }
    }
}

impl AxisInput {
    /// `neg` = 负向键（← / ↑），`pos` = 正向键（→ / ↓）。
    pub fn new(neg: Dik, pos: Dik) -> Self {
        AxisInput {
            neg,
            pos,
            stack: Vec::new(),
        }
    }

    /// 从键位表同步两个键（键位可被 `data/keymap.txt` 覆盖，所以每帧取一次）。
    pub fn set_keys(&mut self, neg: Dik, pos: Dik) {
        if self.neg != neg || self.pos != pos {
            self.neg = neg;
            self.pos = pos;
            self.stack.clear();
        }
    }

    /// 按输入边沿更新栈（上升沿 push，下降沿 remove）。
    ///
    /// 返回**本次刚按下的键**及「按下这一刻该轴是否为中立（没有任何键按着）」——
    /// 这只是轴的输入状态，不代表官方命令识别；命令识别见 `dash` 模块。
    pub fn update(&mut self, input: &InputState) -> Option<(Dik, bool)> {
        let mut hit = None;
        // 同一快照先移除所有已释放的键，避免左→右与右→左出现不对称。
        self.stack.retain(|k| input.is_down(*k));
        for k in [self.neg, self.pos] {
            if input.just_pressed(k) {
                let neutral_before = self.stack.is_empty();
                self.stack.retain(|d| *d != k);
                self.stack.push(k);
                hit = Some((k, neutral_before));
            } else if !input.is_down(k) {
                self.stack.retain(|d| *d != k);
            } else if !self.stack.contains(&k) {
                // Rebinding/teleport can clear the axis while a physical key
                // remains held. Restore movement without inventing a press.
                self.stack.push(k);
            }
        }
        hit
    }

    /// 该轴当前方向：`-1` / `+1` / `0`（= 官方「方向值 4」，无输入）。
    pub fn value(&self) -> i32 {
        match self.stack.last() {
            Some(k) if *k == self.pos => 1,
            Some(_) => -1,
            None => 0,
        }
    }

    /// 传送 / 状态重置时清空（对应官方「记录全部失效」）。
    pub fn reset(&mut self) {
        self.stack.clear();
    }
}
