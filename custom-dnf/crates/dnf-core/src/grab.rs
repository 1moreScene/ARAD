//! Grab ownership is independent of ordinary damage reaction / SUPERARMOR.
use crate::protection::Protection;
#[derive(Clone, Debug)]
pub struct GrabState {
    pub grabable: bool,
    owner: Option<u64>,
}
impl Default for GrabState {
    fn default() -> Self {
        Self {
            grabable: true,
            owner: None,
        }
    }
}
impl GrabState {
    pub fn new(grabable: bool) -> Self {
        Self {
            grabable,
            owner: None,
        }
    }
    pub fn owner(&self) -> Option<u64> {
        self.owner
    }
    pub fn can_grab(&self, alive: bool, state: Protection) -> bool {
        alive && self.grabable && self.owner.is_none() && state != Protection::Invincible
    }
    pub fn acquire(&mut self, owner: u64, alive: bool, state: Protection) -> bool {
        if !self.can_grab(alive, state) {
            return false;
        }
        self.owner = Some(owner);
        true
    }
    pub fn release(&mut self, owner: u64) {
        if self.owner == Some(owner) {
            self.owner = None;
        }
    }
    pub fn clear(&mut self) {
        self.owner = None;
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn admission_and_ownership_matrix() {
        for alive in [false, true] {
            for grabable in [false, true] {
                for state in [
                    Protection::Normal,
                    Protection::SuperArmor,
                    Protection::Invincible,
                ] {
                    let mut g = GrabState {
                        grabable,
                        ..Default::default()
                    };
                    let expected = alive && grabable && state != Protection::Invincible;
                    assert_eq!(g.acquire(7, alive, state), expected);
                    if expected {
                        assert!(!g.acquire(8, true, Protection::Normal));
                        g.release(8);
                        assert_eq!(g.owner(), Some(7));
                        g.release(7);
                        assert_eq!(g.owner(), None);
                    }
                }
            }
        }
    }
}
