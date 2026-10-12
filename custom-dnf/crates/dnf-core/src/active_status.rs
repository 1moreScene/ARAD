//! CNRDActiveStatus: seventeen independent slots; id17 is the none sentinel.
//! Tuning is supplied by the official PE tables, not invented enum defaults.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ActiveStatus {
    Slow,
    Freeze,
    Poison,
    Stun,
    Curse,
    Blind,
    Lightning,
    Stone,
    Sleep,
    Burn,
    DeadlyStrike,
    Bleeding,
    Haste,
    Bless,
    Element,
    Confuse,
    Hold,
}
impl ActiveStatus {
    pub const ALL: [Self; 17] = [
        Self::Slow,
        Self::Freeze,
        Self::Poison,
        Self::Stun,
        Self::Curse,
        Self::Blind,
        Self::Lightning,
        Self::Stone,
        Self::Sleep,
        Self::Burn,
        Self::DeadlyStrike,
        Self::Bleeding,
        Self::Haste,
        Self::Bless,
        Self::Element,
        Self::Confuse,
        Self::Hold,
    ];
    pub fn from_id(id: usize) -> Option<Self> {
        Self::ALL.get(id).copied()
    }
    pub fn from_token(token: &str) -> Option<Self> {
        let token = token
            .trim_matches('`')
            .trim_matches(|c| c == '[' || c == ']');
        Self::ALL.into_iter().find(|s| s.token() == token)
    }
    pub const fn token(self) -> &'static str {
        match self {
            Self::Slow => "slow",
            Self::Freeze => "freeze",
            Self::Poison => "poison",
            Self::Stun => "stun",
            Self::Curse => "curse",
            Self::Blind => "blind",
            Self::Lightning => "lightning",
            Self::Stone => "stone",
            Self::Sleep => "sleep",
            Self::Burn => "burn",
            Self::DeadlyStrike => "deadlystrike",
            Self::Bleeding => "bleeding",
            Self::Haste => "haste",
            Self::Bless => "bless",
            Self::Element => "element",
            Self::Confuse => "confuse",
            Self::Hold => "hold",
        }
    }
    pub const fn name(self) -> &'static str {
        match self {
            Self::Slow => "减速",
            Self::Freeze => "冰冻",
            Self::Poison => "中毒",
            Self::Stun => "眩晕",
            Self::Curse => "诅咒",
            Self::Blind => "失明",
            Self::Lightning => "感电",
            Self::Stone => "石化",
            Self::Sleep => "睡眠",
            Self::Burn => "灼烧",
            Self::DeadlyStrike => "致命攻击（保留）",
            Self::Bleeding => "出血",
            Self::Haste => "加速",
            Self::Bless => "祝福",
            Self::Element => "元素（保留）",
            Self::Confuse => "混乱",
            Self::Hold => "束缚",
        }
    }
    pub const fn implemented(self) -> bool {
        !matches!(self, Self::DeadlyStrike | Self::Element)
    }
    pub const fn description(self) -> &'static str {
        match self {
            Self::Slow => "降低移动与攻击速度",
            Self::Freeze => "冻结动画与行动；不能被抓取",
            Self::Poison => "持续扣血，独立累计小数伤害",
            Self::Stun => "无法主动行动；与睡眠互斥",
            Self::Curse => "降低力量、智力、体力与精神",
            Self::Blind => "限制视野；不会遮挡旁观者画面",
            Self::Lightning => "受到有效攻击时追加感电伤害",
            Self::Stone => "石化定身；增加体力，提前解除有碎裂伤害",
            Self::Sleep => "倒地睡眠；200ms后受击唤醒",
            Self::Burn => "持续扣血，并伤害周围其他目标",
            Self::Bleeding => "每秒扣血，行动中伤害增为1.5倍",
            Self::Haste => "提高移动与攻击速度",
            Self::Bless => "提高力量、智力、体力与精神",
            Self::Confuse => "行动方向有60%概率反转",
            Self::Hold => "限制主动位移，仍可攻击",
            _ => "40JP参数为零且没有执行分支",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StatusConfig {
    pub duration_ms: f64,
    pub power: f64,
    pub secondary: f64,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StatusInstance {
    pub kind: ActiveStatus,
    pub level: u8,
    pub source: u64,
    pub elapsed_ms: f64,
    pub remaining_ms: f64,
    pub power: f64,
    pub secondary: f64,
    dot_total: f64,
    dot_paid: i32,
    next_tick_ms: f64,
    pub serial: u64,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StatusDamage {
    pub kind: ActiveStatus,
    pub amount: i32,
    pub splash: i32,
}
#[derive(Clone, Debug)]
pub struct ActiveStatuses {
    slots: [Option<StatusInstance>; 17],
    serial: u64,
    count: usize,
    mask: u32,
}
impl Default for ActiveStatuses {
    fn default() -> Self {
        Self {
            slots: [None; 17],
            serial: 0,
            count: 0,
            mask: 0,
        }
    }
}
impl ActiveStatuses {
    pub fn get(&self, kind: ActiveStatus) -> Option<&StatusInstance> {
        self.slots[kind as usize].as_ref()
    }
    pub fn has(&self, kind: ActiveStatus) -> bool {
        self.get(kind).is_some()
    }
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
    pub fn mask(&self) -> u32 {
        self.mask
    }
    pub fn iter(&self) -> impl Iterator<Item = &StatusInstance> {
        self.slots.iter().flatten()
    }
    pub fn apply(
        &mut self,
        kind: ActiveStatus,
        level: u8,
        config: StatusConfig,
        source: u64,
    ) -> bool {
        if !kind.implemented()
            || config.duration_ms <= 0.0
            || !config.duration_ms.is_finite()
            || !config.power.is_finite()
            || !config.secondary.is_finite()
        {
            return false;
        }
        if kind == ActiveStatus::Stun && self.has(ActiveStatus::Sleep)
            || kind == ActiveStatus::Sleep && self.has(ActiveStatus::Stun)
        {
            return false;
        }
        self.serial += 1;
        if !self.has(kind) {
            self.count += 1;
        }
        self.mask |= 1 << kind as usize;
        self.slots[kind as usize] = Some(StatusInstance {
            kind,
            level: level.clamp(1, 40),
            source,
            elapsed_ms: 0.0,
            remaining_ms: config.duration_ms,
            power: if kind == ActiveStatus::Bleeding {
                config.power.trunc()
            } else {
                config.power
            },
            secondary: config.secondary,
            dot_total: 0.0,
            dot_paid: 0,
            next_tick_ms: 1000.0,
            serial: self.serial,
        });
        true
    }
    pub fn remove(&mut self, kind: ActiveStatus) -> Option<StatusDamage> {
        let slot = self.slots[kind as usize].take()?;
        self.count -= 1;
        self.mask &= !(1 << kind as usize);
        if kind == ActiveStatus::Stone {
            let duration = slot.elapsed_ms + slot.remaining_ms;
            let amount = (slot.secondary
                * (1.0 - slot.elapsed_ms / (duration * 0.5)).clamp(0.0, 1.0))
            .trunc() as i32;
            if amount > 0 {
                return Some(StatusDamage {
                    kind,
                    amount,
                    splash: 0,
                });
            }
        }
        None
    }
    pub fn clear(&mut self) {
        self.slots = [None; 17];
        self.count = 0;
        self.mask = 0;
    }
    pub fn blocks_grab(&self) -> bool {
        self.has(ActiveStatus::Freeze) || self.has(ActiveStatus::Stone)
    }
    pub fn frozen(&self) -> bool {
        self.blocks_grab()
    }
    pub fn blocks_action(&self) -> bool {
        self.frozen() || self.has(ActiveStatus::Stun) || self.has(ActiveStatus::Sleep)
    }
    pub fn blocks_move(&self) -> bool {
        self.blocks_action() || self.has(ActiveStatus::Hold)
    }
    pub fn speed_rate(&self) -> f64 {
        // Attribute aggregation subtracts(1-slow) and adds(haste-1) independently.
        (1.0 - self.get(ActiveStatus::Slow).map_or(0.0, |s| 1.0 - s.power)
            + self.get(ActiveStatus::Haste).map_or(0.0, |s| s.power - 1.0))
        .max(0.01)
    }
    pub fn stat_rate(&self) -> f64 {
        (1.0 - self.get(ActiveStatus::Curse).map_or(0.0, |s| 1.0 - s.power)
            + self.get(ActiveStatus::Bless).map_or(0.0, |s| s.power - 1.0))
        .max(0.01)
    }
    pub fn stone_vitality(&self) -> i32 {
        if self.has(ActiveStatus::Stone) {
            100
        } else {
            0
        }
    }
    pub fn elemental_resistance_delta(&self, element: usize) -> i32 {
        if self.has(ActiveStatus::Freeze) {
            match element {
                0 => -50,
                1 => 50,
                _ => 0,
            }
        } else {
            0
        }
    }
    pub fn resistance_delta(&self, kind: ActiveStatus) -> i32 {
        if self.has(kind)
            && matches!(
                kind,
                ActiveStatus::Freeze
                    | ActiveStatus::Stun
                    | ActiveStatus::Stone
                    | ActiveStatus::Sleep
                    | ActiveStatus::Hold
            )
        {
            1000
        } else {
            0
        }
    }
    pub fn on_hit(&mut self) -> i32 {
        if self
            .get(ActiveStatus::Sleep)
            .is_some_and(|s| s.elapsed_ms > 200.0)
        {
            self.remove(ActiveStatus::Sleep);
        }
        self.get(ActiveStatus::Lightning)
            .map_or(0, |s| s.power.trunc() as i32)
    }
    pub fn advance(
        &mut self,
        dt_ms: f64,
        moving: bool,
        damage_factors: [f64; 3],
    ) -> [Option<StatusDamage>; 3] {
        let mut result = [None; 3];
        let mut cursor = 0;
        if self.is_empty() || !dt_ms.is_finite() || dt_ms <= 0.0 {
            return result;
        }
        for slot in &mut self.slots {
            let Some(s) = slot else { continue };
            let active = dt_ms.min(s.remaining_ms);
            s.elapsed_ms += active;
            s.remaining_ms = (s.remaining_ms - active).max(0.0);
            let mut splash = 0;
            match s.kind {
                ActiveStatus::Poison => {
                    s.dot_total += s.power * damage_factors[0].max(0.0) * active / 1000.0
                }
                ActiveStatus::Burn => {
                    s.dot_total += s.power * damage_factors[1].max(0.0) * active / 1000.0;
                }
                ActiveStatus::Bleeding => {
                    while s.elapsed_ms + 1e-7 >= s.next_tick_ms {
                        s.dot_total +=
                            (s.power * damage_factors[2].max(0.0) * if moving { 1.5 } else { 1.0 })
                                .trunc();
                        s.next_tick_ms += 1000.0;
                    }
                }
                _ => {}
            }
            let total = (s.dot_total + 1e-7).trunc() as i32;
            let damage = total - s.dot_paid;
            s.dot_paid = total;
            if s.kind == ActiveStatus::Burn && s.elapsed_ms + 1e-7 >= s.next_tick_ms {
                s.next_tick_ms += 1000.0;
                splash = s.secondary.trunc() as i32;
            }
            if damage > 0 || splash > 0 {
                result[cursor] = Some(StatusDamage {
                    kind: s.kind,
                    amount: damage.max(0),
                    splash,
                });
                cursor += 1;
            }
            if s.remaining_ms < 1e-7 {
                self.mask &= !(1 << s.kind as usize);
                *slot = None;
                self.count -= 1;
            }
        }
        result
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn config(seconds: f64, power: f64) -> StatusConfig {
        StatusConfig {
            duration_ms: seconds * 1000.0,
            power,
            secondary: 0.0,
        }
    }
    #[test]
    fn independent_slots_refresh_and_removal_preserve_other_states() {
        let mut s = ActiveStatuses::default();
        s.apply(ActiveStatus::Slow, 1, config(20.0, 0.9), 1);
        s.apply(ActiveStatus::Poison, 1, config(7.0, 6.0), 2);
        s.advance(1000.0, false, [1.0; 3]);
        let slow = *s.get(ActiveStatus::Slow).unwrap();
        s.apply(ActiveStatus::Poison, 2, config(7.0, 12.0), 3);
        assert_eq!(s.get(ActiveStatus::Slow), Some(&slow));
        s.remove(ActiveStatus::Poison);
        assert_eq!(s.mask(), 1);
        assert!(s.has(ActiveStatus::Slow));
        s.clear();
        assert!(s.is_empty());
        assert_eq!(s.mask(), 0);
    }
    #[test]
    fn modifiers_resistances_control_and_reserved_ids_follow_distinct_rules() {
        let mut s = ActiveStatuses::default();
        s.apply(ActiveStatus::Slow, 1, config(20.0, 0.9), 1);
        s.apply(ActiveStatus::Haste, 1, config(20.0, 1.1), 1);
        assert!((s.speed_rate() - 1.0).abs() < 1e-6);
        s.apply(ActiveStatus::Curse, 1, config(20.0, 0.9), 1);
        s.apply(ActiveStatus::Bless, 1, config(20.0, 1.1), 1);
        assert!((s.stat_rate() - 1.0).abs() < 1e-6);
        s.apply(ActiveStatus::Freeze, 1, config(2.0, 2.0), 1);
        assert!(s.frozen() && s.blocks_grab());
        assert_eq!(s.elemental_resistance_delta(0), -50);
        assert_eq!(s.elemental_resistance_delta(1), 50);
        assert_eq!(s.resistance_delta(ActiveStatus::Freeze), 1000);
        s.apply(ActiveStatus::Stone, 1, config(4.0, 4.0), 1);
        s.remove(ActiveStatus::Freeze);
        assert!(s.frozen());
        s.remove(ActiveStatus::Stone);
        s.apply(ActiveStatus::Hold, 1, config(4.0, 4.0), 1);
        assert!(s.blocks_move());
        assert!(!s.blocks_action());
        assert!(!s.apply(ActiveStatus::Element, 1, config(5.0, 20.0), 1));
        assert!(!s.apply(ActiveStatus::DeadlyStrike, 1, config(5.0, 20.0), 1));
        assert_eq!(ActiveStatus::from_id(17), None);
    }
    #[test]
    fn continuous_dots_and_discrete_bleed_are_rate_independent_and_resistible() {
        for hz in [60, 120, 300] {
            let mut s = ActiveStatuses::default();
            for kind in [
                ActiveStatus::Poison,
                ActiveStatus::Burn,
                ActiveStatus::Bleeding,
            ] {
                s.apply(kind, 1, config(4.0, 6.0), 1);
            }
            let mut totals = [0; 3];
            for _ in 0..hz * 4 {
                for d in s
                    .advance(1000.0 / hz as f64, false, [0.5, 1.0, 1.0])
                    .into_iter()
                    .flatten()
                {
                    totals[match d.kind {
                        ActiveStatus::Poison => 0,
                        ActiveStatus::Burn => 1,
                        _ => 2,
                    }] += d.amount;
                }
            }
            assert_eq!(totals, [12, 24, 24]);
            assert!(s.is_empty());
        }
        let mut s = ActiveStatuses::default();
        s.apply(ActiveStatus::Bleeding, 1, config(1.0, 17.9), 1);
        assert_eq!(s.advance(1000.0, true, [1.0; 3])[0].unwrap().amount, 25);
    }
    #[test]
    fn shock_is_hit_driven_sleep_has_grace_period_and_stone_early_exit_has_damage() {
        let mut s = ActiveStatuses::default();
        s.apply(ActiveStatus::Sleep, 1, config(5.0, 5.0), 1);
        assert!(!s.apply(ActiveStatus::Stun, 1, config(1.0, 1.0), 1));
        s.apply(ActiveStatus::Lightning, 1, config(15.0, 18.0), 1);
        assert_eq!(s.on_hit(), 18);
        assert!(s.has(ActiveStatus::Sleep));
        s.advance(201.0, false, [1.0; 3]);
        assert_eq!(s.on_hit(), 18);
        assert!(!s.has(ActiveStatus::Sleep));
        s.apply(
            ActiveStatus::Stone,
            1,
            StatusConfig {
                duration_ms: 4000.0,
                power: 4.0,
                secondary: 120.0,
            },
            1,
        );
        s.advance(1000.0, false, [1.0; 3]);
        assert_eq!(s.remove(ActiveStatus::Stone).unwrap().amount, 60);
    }
}
