//! Protection is orthogonal to movement/action state. PVF frames and skill/buff
//! sources are independent, so removing one source cannot clear another.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum Protection {
    #[default]
    Normal,
    SuperArmor,
    Invincible,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DamageOutcome {
    Rejected,
    Armored,
    React,
    Lethal,
}

#[derive(Clone, Debug)]
struct Grant {
    source: u32,
    protection: Protection,
    remaining_ms: Option<f64>,
}

#[derive(Clone, Debug, Default)]
pub struct ProtectionState {
    frame: Protection,
    action: Protection,
    grants: Vec<Grant>,
}

impl ProtectionState {
    pub fn set_frame(&mut self, protection: Protection) {
        self.frame = protection;
    }

    pub fn set_action(&mut self, protection: Protection) {
        self.action = protection;
    }

    /// Same source refreshes/replaces its grant. None lasts until explicitly removed.
    pub fn grant(&mut self, source: u32, protection: Protection, duration_ms: Option<f64>) {
        self.remove(source);
        if protection == Protection::Normal
            || duration_ms.is_some_and(|ms| !ms.is_finite() || ms <= 0.0)
        {
            return;
        }
        self.grants.push(Grant {
            source,
            protection,
            remaining_ms: duration_ms,
        });
    }

    pub fn remove(&mut self, source: u32) {
        self.grants.retain(|g| g.source != source);
    }

    pub fn has_source(&self, source: u32) -> bool {
        self.grants.iter().any(|g| g.source == source)
    }

    pub fn advance(&mut self, dt_ms: f64) {
        if !dt_ms.is_finite() || dt_ms <= 0.0 {
            return;
        }
        for grant in &mut self.grants {
            if let Some(left) = &mut grant.remaining_ms {
                *left -= dt_ms;
            }
        }
        self.grants
            .retain(|g| g.remaining_ms.map_or(true, |left| left > 1e-7));
    }

    pub fn effective(&self) -> Protection {
        self.grants
            .iter()
            .map(|g| g.protection)
            .fold(self.frame.max(self.action), Protection::max)
    }

    /// Invulnerability rejects the complete hit before HP, stop, sounds or reactions.
    /// Armor takes damage and suppresses ordinary hit reactions, but cannot prevent death.
    pub fn apply_damage(&self, hp: &mut i32, damage: i32) -> DamageOutcome {
        if *hp <= 0 || damage <= 0 || self.effective() == Protection::Invincible {
            return DamageOutcome::Rejected;
        }
        *hp = hp.saturating_sub(damage).max(0);
        if *hp == 0 {
            DamageOutcome::Lethal
        } else if self.effective() == Protection::SuperArmor {
            DamageOutcome::Armored
        } else {
            DamageOutcome::React
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlapping_sources_expire_and_restore_underlying_frame_and_buff() {
        let mut s = ProtectionState::default();
        s.set_frame(Protection::SuperArmor);
        s.grant(1, Protection::SuperArmor, Some(200.0));
        s.grant(2, Protection::Invincible, Some(50.0));
        assert_eq!(s.effective(), Protection::Invincible);
        s.advance(50.0);
        assert_eq!(s.effective(), Protection::SuperArmor);
        s.set_frame(Protection::Normal);
        s.advance(150.0);
        assert_eq!(s.effective(), Protection::Normal);
        s.set_action(Protection::SuperArmor);
        s.grant(3, Protection::Invincible, None);
        s.remove(3);
        assert_eq!(s.effective(), Protection::SuperArmor);
        s.set_action(Protection::Normal);
        assert_eq!(s.effective(), Protection::Normal);
    }

    #[test]
    fn damage_invincibility_armor_and_death_are_distinct() {
        let mut s = ProtectionState::default();
        let mut hp = 100;
        s.set_frame(Protection::Invincible);
        assert_eq!(s.apply_damage(&mut hp, 10), DamageOutcome::Rejected);
        assert_eq!(hp, 100);
        s.set_frame(Protection::SuperArmor);
        assert_eq!(s.apply_damage(&mut hp, 10), DamageOutcome::Armored);
        assert_eq!(hp, 90);
        assert_eq!(s.apply_damage(&mut hp, 100), DamageOutcome::Lethal);
        assert_eq!(hp, 0);
    }

    #[test]
    fn refresh_and_remove_are_per_source() {
        let mut s = ProtectionState::default();
        s.grant(1, Protection::Invincible, Some(10.0));
        s.grant(1, Protection::Invincible, Some(30.0));
        s.grant(2, Protection::SuperArmor, None);
        s.advance(10.0);
        assert_eq!(s.effective(), Protection::Invincible);
        s.remove(2);
        assert_eq!(s.effective(), Protection::Invincible);
        s.advance(20.0);
        assert_eq!(s.effective(), Protection::Normal);
    }
}
