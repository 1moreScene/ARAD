//! 43BEE0 -> 43BD50 channel4; original transient textbox at (9,74), max8 lines, 10s.
use crate::{hud::wrap, ui::text};
use dnf_core::actor::SkillFailure;
use dnf_data::messages::SystemMessages;
use dnf_render::framebuffer::{rgba, Color, Framebuffer};
use std::collections::VecDeque;
#[derive(Default)]
pub struct Notices {
    lines: VecDeque<(String, f64)>,
}
impl Notices {
    pub fn advance(&mut self, dt: f64) {
        for (_, age) in &mut self.lines {
            *age += dt;
        }
        self.lines.retain(|(_, age)| *age <= 10000.0);
    }
    pub fn push(&mut self, message: &str) {
        for line in wrap(message, 330) {
            self.lines.push_back((line, 0.0));
            if self.lines.len() > 8 {
                self.lines.pop_front();
            }
        }
    }
    pub fn skill_failure(&mut self, failure: SkillFailure, messages: &SystemMessages) {
        let message = match failure {
            SkillFailure::Cooldown => messages.get(853).unwrap_or("技能冷却中。"),
            SkillFailure::Mp => messages.get(854).unwrap_or("MP不足。"),
            SkillFailure::Item(_) => messages.get(855).unwrap_or("物品不足。"),
            // HP-spending skills use their own behavior; no global HP string in 4B49B0.
            SkillFailure::Hp => "HP不足，无法施放技能。",
        };
        if let SkillFailure::Item(id) = failure {
            let name = if id == 3037 {
                "无色小晶块".into()
            } else {
                format!("物品 {id}")
            };
            self.push(&format!("{message} ({name})"));
        } else {
            self.push(message);
        }
    }
    pub fn draw(&self, fb: &mut Framebuffer) {
        for (i, (line, _)) in self.lines.iter().enumerate() {
            let y = 74 + i as i32 * 13;
            text(fb, 10, y + 1, line, Color::BLACK);
            text(fb, 9, y, line, rgba(155, 200, 230, 255));
        }
    }
    pub fn len(&self) -> usize {
        self.lines.len()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn overflowing_and_expiring_notices_keep_the_newest_eight_lines() {
        let mut n = Notices::default();
        n.push("old");
        n.advance(9999.0);
        n.push("new");
        n.advance(2.0);
        assert_eq!(n.len(), 1);
        assert_eq!(n.lines[0].0, "new");
        for i in 0..10 {
            n.push(&i.to_string());
        }
        assert_eq!(n.len(), 8);
        assert_eq!(n.lines[0].0, "2");
    }
}
