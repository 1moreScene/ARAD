//! Exact 415240 engine slot -> DIK table. Evidence: tools/key_manager_evidence.py.
use crate::keys::Dik;
pub const SLOTS: [u8; 107] = [
    203, 205, 200, 208, 29, 29, 157, 42, 42, 54, 56, 184, 56, 57, 13, 14, 28, 210, 211, 201, 209,
    199, 207, 78, 12, 55, 1, 39, 52, 51, 181, 15, 58, 74, 53, 82, 79, 80, 81, 75, 76, 77, 71, 72,
    73, 11, 2, 3, 4, 5, 6, 7, 8, 9, 10, 30, 48, 46, 32, 18, 33, 34, 35, 23, 36, 37, 38, 50, 49, 24,
    25, 16, 19, 31, 20, 22, 47, 17, 45, 21, 44, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 87, 88,
    183, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 26, 27, 40,
];
pub const LOGICAL_SLOTS: [usize; 9] = [2, 3, 0, 1, 80, 78, 57, 76, 13];
pub const SKILL_SLOTS: [usize; 6] = [55, 73, 58, 60, 61, 62];
pub const ITEM_SLOTS: [usize; 6] = [46, 47, 48, 49, 50, 51];
pub const fn dik(slot: usize) -> Dik {
    Dik::new(SLOTS[slot])
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn actual_default_slots_include_space_for_buff_and_modifier_aliases() {
        assert_eq!(
            LOGICAL_SLOTS.map(|slot| dik(slot).code()),
            [200, 208, 203, 205, 44, 45, 46, 47, 57]
        );
        assert_eq!(
            SKILL_SLOTS.map(|slot| dik(slot).code()),
            [30, 31, 32, 33, 34, 35]
        );
        assert_eq!(ITEM_SLOTS.map(|slot| dik(slot).code()), [2, 3, 4, 5, 6, 7]);
        assert_eq!(dik(4), dik(5));
        assert_eq!(dik(7), dik(8));
        assert_eq!(dik(10), dik(12));
    }
}
