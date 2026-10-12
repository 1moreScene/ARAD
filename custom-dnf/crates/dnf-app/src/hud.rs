//! 469170/469DD0 base interface, 4633D0 drag binding, 47DF50/4F2E80 cooldown and 570FD0 tooltip.
use crate::{
    assets::ClientAssets,
    game::{Game, GameData},
    room::blit_frame,
    ui::text,
};
use dnf_core::{input::InputState, keymap::Command, keys::Dik};
use dnf_data::img::{ImgFile, ImgFrame};
use dnf_render::framebuffer::{rgba, Color, Framebuffer};
use std::rc::Rc;

pub struct HudAssets {
    pub base: Rc<ImgFile>,
    pub book: Rc<ImgFile>,
    pub popup: Rc<ImgFile>,
    pub ready: Rc<ImgFile>,
}
impl HudAssets {
    pub fn load(a: &ClientAssets) -> Result<Self, String> {
        let load = |p: &str| a.img(p).ok_or_else(|| format!("HUD asset missing {p}"));
        let out = Self {
            base: load("Interface/Base.img")?,
            book: load("Interface/SkillInventoryWindow.img")?,
            popup: load("Interface/Popup.img")?,
            ready: load("Common/Etc/CoolTimeEnd.img")?,
        };
        if out.base.frames.len() < 104
            || out.book.frames.is_empty()
            || out.popup.frames.len() < 9
            || out.ready.frames.len() != 9
        {
            return Err("HUD atlas frame mismatch".into());
        }
        Ok(out)
    }
}
pub const BOOK: (i32, i32, i32, i32) = (174, 12, 291, 380);
pub const SKILLS: [i32; 3] = [5, 46, 31];
pub const SLOT_Y: i32 = 447;
pub fn slot_rect(i: usize) -> (i32, i32, i32, i32) {
    (381 + 31 * i as i32, SLOT_Y, 28, 28)
}
pub fn learned_rect(i: usize) -> (i32, i32, i32, i32) {
    (BOOK.0 + 38 + 36 * i as i32, BOOK.1 + 57, 28, 28)
}
fn inside(p: (i32, i32), r: (i32, i32, i32, i32)) -> bool {
    p.0 >= r.0 && p.0 < r.0 + r.2 && p.1 >= r.1 && p.1 < r.1 + r.3
}
fn slot_at(p: (i32, i32)) -> Option<usize> {
    (0..6).find(|&i| inside(p, slot_rect(i)))
}
pub fn skill_name(id: i32) -> &'static str {
    match id {
        5 => "鬼斩",
        46 => "上挑",
        31 => "噬魂之手",
        _ => "未知技能",
    }
}
#[derive(Copy, Clone, Debug)]
struct Drag {
    id: i32,
    source: Option<usize>,
}
#[derive(Default)]
pub struct InputResult {
    pub captures_pointer: bool,
    pub escape_handled: bool,
    pub changed: bool,
}
/// 4688A0 keeps each lost-height segment for 500ms, then drains 65px per 1000ms.
#[derive(Default)]
pub struct Gauge {
    pub height: i32,
    initialized: bool,
    losses: Vec<(i32, f64)>,
}
impl Gauge {
    fn update(&mut self, current: i32, max: i32, dt_ms: f64) {
        for (_, age) in &mut self.losses {
            *age += dt_ms;
        }
        let height = (65.0 * current.max(0) as f64 / max.max(1) as f64).clamp(0.0, 65.0) as i32;
        if self.initialized && height < self.height {
            self.losses.insert(0, (self.height, 0.0));
        }
        if height > self.height {
            self.losses.retain(|&(old, _)| old > height);
        }
        self.height = height;
        self.initialized = true;
        self.losses
            .retain(|&(old, age)| age < 500.0 || old - drain(age) > height);
    }
    fn draw(&self, base: &ImgFile, fb: &mut Framebuffer, index: usize, x: i32) {
        let f = &base.frames[index];
        let band = |fb: &mut Framebuffer, frame: &ImgFrame, low: i32, high: i32, tint: Color| {
            if high > low {
                crop(fb, frame, x, 412, (x, 477 - high, 35, high - low), tint);
            }
        };
        band(fb, f, 0, self.height, Color::WHITE);
        let mut low = self.height;
        for &(old, age) in &self.losses {
            if age >= 500.0 {
                break;
            }
            band(fb, f, low, old, rgba(150, 150, 150, 255));
            band(
                fb,
                &base.frames[103],
                low,
                old,
                rgba(255, 255, 255, (255.0 * (1.0 - age / 500.0)) as u8),
            );
            low = old;
        }
        if let Some(&(old, age)) = self.losses.last() {
            if age >= 500.0 {
                band(fb, f, low, old - drain(age), rgba(150, 150, 150, 255));
            }
        }
    }
}
fn drain(age: f64) -> i32 {
    (65.0 * ((age - 500.0) / 1000.0).clamp(0.0, 1.0)) as i32
}
pub struct Hud {
    pub slots: [Option<i32>; 6],
    pub library_open: bool,
    pub pointer: (i32, i32, bool),
    down: bool,
    captured: bool,
    drag: Option<Drag>,
    pub dirty: bool,
    pub message: String,
    pub gauges: [Gauge; 2],
}
impl Default for Hud {
    fn default() -> Self {
        Self {
            slots: [Some(5), Some(46), Some(31), None, None, None],
            library_open: false,
            pointer: (-1, -1, false),
            down: false,
            captured: false,
            drag: None,
            dirty: false,
            message: String::new(),
            gauges: [Gauge::default(), Gauge::default()],
        }
    }
}
impl Hud {
    pub fn update_gauges(&mut self, hp: i32, hp_max: i32, mp: i32, mp_max: i32, dt: f64) {
        self.gauges[0].update(hp, hp_max, dt);
        self.gauges[1].update(mp, mp_max, dt);
    }
    pub fn capsule_hover(&self) -> Option<usize> {
        let p = (self.pointer.0, self.pointer.1);
        // 464C60 inclusive original HP/MP hover coordinates.
        [(6, 412, 36, 66), (599, 412, 41, 66)]
            .iter()
            .position(|&r| inside(p, r))
    }
    pub fn dragging(&self) -> bool {
        self.drag.is_some()
    }
    pub fn cancel_drag(&mut self) {
        self.drag = None;
        self.down = false;
        self.captured = false;
        self.pointer = (-1, -1, false);
    }
    pub fn pointer_baseline(&mut self, p: (i32, i32, bool)) {
        self.cancel_drag();
        self.pointer = p;
        self.down = p.2;
        self.captured = p.2;
    }
    pub fn hovered_skill(&self) -> Option<i32> {
        if self.drag.is_some() {
            return None;
        }
        let p = (self.pointer.0, self.pointer.1);
        if self.library_open && inside(p, BOOK) {
            return SKILLS
                .iter()
                .enumerate()
                .find_map(|(i, &id)| inside(p, learned_rect(i)).then_some(id));
        }
        slot_at(p).and_then(|i| self.slots[i])
    }
    fn place(&mut self, drag: Drag, target: usize) {
        let source = drag
            .source
            .or_else(|| self.slots.iter().position(|id| *id == Some(drag.id)));
        if source == Some(target) {
            return;
        }
        let replaced = self.slots[target];
        self.slots[target] = Some(drag.id);
        if let Some(source) = source {
            self.slots[source] = replaced;
        }
    }
    pub fn input(&mut self, keys: &InputState, pointer: (i32, i32, bool)) -> InputResult {
        self.pointer = pointer;
        let mut result = InputResult::default();
        if keys.just_pressed(Dik::K) {
            self.library_open = !self.library_open;
            self.drag = None;
        }
        if keys.just_pressed(Dik::ESCAPE) && (self.drag.is_some() || self.library_open) {
            result.escape_handled = true;
            if self.drag.is_some() {
                self.drag = None;
            } else {
                self.library_open = false;
            }
        }
        let p = (pointer.0, pointer.1);
        let pressed = pointer.2 && !self.down;
        let released = !pointer.2 && self.down;
        let book_hover = self.library_open && inside(p, BOOK);
        if pressed {
            self.captured = book_hover || p.1 >= 402;
            if inside(p, (286, 427, 68, 18)) {
                self.library_open = !self.library_open;
            } else if book_hover {
                if inside(p, (BOOK.0 + 263, BOOK.1 + 6, 20, 18)) {
                    self.library_open = false;
                } else if let Some(i) = (0..2).find(|&i| inside(p, learned_rect(i))) {
                    self.drag = Some(Drag {
                        id: SKILLS[i],
                        source: None,
                    });
                }
            } else if let Some(i) = slot_at(p) {
                if let Some(id) = self.slots[i] {
                    self.drag = Some(Drag {
                        id,
                        source: Some(i),
                    });
                }
            }
        }
        if released {
            if let Some(drag) = self.drag.take() {
                let before = self.slots;
                if let Some(target) = slot_at(p) {
                    self.place(drag, target);
                } else if book_hover {
                    if let Some(source) = drag.source {
                        self.slots[source] = None;
                    }
                }
                result.changed = self.slots != before;
                self.dirty |= result.changed;
            }
        }
        result.captures_pointer = book_hover || p.1 >= 402 || self.drag.is_some() || self.captured;
        if released {
            self.captured = false;
        }
        self.down = pointer.2;
        result
    }
    pub fn bindings_text(&self) -> String {
        let mut out = String::from(
            "# 技能ID：5=鬼斩，46=上挑，31=噬魂之手，0=空。slot1..6默认对应A/S/D/F/G/H。\n",
        );
        for (i, id) in self.slots.iter().enumerate() {
            out.push_str(&format!("slot{} = {}\n", i + 1, id.unwrap_or(0)));
        }
        out
    }
    pub fn load_bindings(&mut self, raw: &str) -> Result<(), String> {
        let mut slots = [None; 6];
        for line in raw
            .lines()
            .map(|s| s.split('#').next().unwrap_or("").trim())
            .filter(|s| !s.is_empty())
        {
            let (key, value) = line.split_once('=').ok_or("invalid skill slot setting")?;
            let i = key
                .trim()
                .strip_prefix("slot")
                .and_then(|s| s.parse::<usize>().ok())
                .filter(|&i| i > 0 && i <= 6)
                .ok_or("invalid skill slot index")?
                - 1;
            let id = value
                .trim()
                .parse::<i32>()
                .map_err(|_| "invalid skill id")?;
            if id != 0 && !SKILLS.contains(&id) {
                return Err("unsupported skill slot id".into());
            }
            if id != 0 && slots.contains(&Some(id)) {
                return Err("duplicate skill slot id".into());
            }
            slots[i] = (id != 0).then_some(id);
        }
        self.slots = slots;
        self.dirty = false;
        Ok(())
    }
    pub fn draw(&self, game: &Game, data: &GameData, fb: &mut Framebuffer) {
        let a = &data.hud_assets;
        if self.library_open {
            blit_frame(fb, &a.book.frames[0], BOOK.0, BOOK.1, false);
            text(
                fb,
                BOOK.0 + 14,
                BOOK.1 + 13,
                "技能 · 已学习",
                rgba(248, 225, 170, 255),
            );
            text(fb, BOOK.0 + 268, BOOK.1 + 9, "×", Color::WHITE);
            for (i, &id) in SKILLS.iter().enumerate() {
                let r = learned_rect(i);
                draw_icon(data, fb, id, (r.0, r.1), Color::WHITE, false);
                text(
                    fb,
                    r.0 - 1,
                    r.1 + 29,
                    skill_name(id),
                    rgba(240, 230, 192, 255),
                );
            }
            fb.fill_rect(BOOK.0 + 15, BOOK.1 + 325, 260, 48, rgba(0, 0, 0, 215));
            fb.rect_outline(BOOK.0 + 15, BOOK.1 + 325, 260, 48, rgba(130, 114, 78, 255));
            text(
                fb,
                BOOK.0 + 18,
                BOOK.1 + 329,
                "拖动图标到下方技能快捷栏",
                rgba(245, 232, 185, 255),
            );
            text(
                fb,
                BOOK.0 + 18,
                BOOK.1 + 346,
                "拖回此窗口可移除快捷栏绑定",
                Color::WHITE,
            );
            if !self.message.is_empty() {
                text(
                    fb,
                    BOOK.0 + 16,
                    BOOK.1 + 365,
                    &self.message,
                    rgba(255, 150, 100, 255),
                );
            }
        }
        blit_frame(fb, &a.base.frames[0], 0, 402, false);
        let v = game.debug_values();
        for (i, x) in [(0, 6), (1, 599)] {
            self.gauges[i].draw(&a.base, fb, i + 1, x);
        }
        if let Some(i) = self.capsule_hover() {
            let label = if i == 0 {
                format!("HP:{}/{}", v[0], v[1])
            } else {
                format!("MP:{}/{}", game.actor.mp, v[3])
            };
            let (width, height) = crate::ui::measure_text(&label);
            let (w, h) = (width + 10, height + 10);
            let x = if i == 0 { 0 } else { 640 - w };
            popup(&a.popup, fb, (x, 402 - h, w, h));
            text(fb, x + 5, 408 - h, &label, Color::WHITE);
        }
        text(fb, 289, 429, "技能[K]", rgba(245, 218, 166, 255));
        for i in 0..6 {
            let r = slot_rect(i);
            let Some(id) = self.slots[i] else { continue };
            if self.drag.is_some_and(|d| d.source == Some(i)) {
                continue;
            }
            let Some(skill) = game.actor.active_skill(id) else {
                continue;
            };
            let inactive =
                !game.actor.resource_failures(skill).is_empty() || !game.actor.can_skill(skill);
            draw_icon(data, fb, id, (r.0, r.1), Color::WHITE, inactive);
            let remaining = cooldown(game, id);
            if remaining > 0.0 {
                let height = (28.0 * remaining / game.actor.skill_cooldown_total_ms(id).max(1.0))
                    .clamp(0.0, 28.0) as i32;
                crop(
                    fb,
                    &a.base.frames[84],
                    r.0,
                    r.1,
                    (r.0, r.1, 28, height),
                    rgba(255, 255, 255, 128),
                );
                let label = format!("{:.1}", (remaining / 1000.0).max(0.1));
                let (w, h) = crate::ui::measure_text(&label);
                text(
                    fb,
                    r.0 + (28 - w) / 2 + 1,
                    r.1 + (28 - h) / 2 + 1,
                    &label,
                    Color::BLACK,
                );
                text(
                    fb,
                    r.0 + (28 - w) / 2,
                    r.1 + (28 - h) / 2,
                    &label,
                    Color::WHITE,
                );
            }
            if let Some(index) = ready_flash_index(game.actor.skill_cooldown_remaining_ms(id)) {
                additive(fb, &a.ready.frames[index], r.0, r.1);
            }
        }
        for i in 0..6 {
            draw_key(
                &a.base,
                fb,
                381 + 31 * i as i32,
                &game.keymap.get(Command::ALL[9 + i]).to_string(),
            );
            draw_key(
                &a.base,
                fb,
                76 + 31 * i as i32,
                &game.keymap.get(Command::ALL[15 + i]).to_string(),
            );
        }
        if let Some(id) = self.hovered_skill() {
            self.tooltip(game, data, fb, id);
        }
        if let Some(drag) = self.drag {
            if let Some(slot) = slot_at((self.pointer.0, self.pointer.1)) {
                let r = slot_rect(slot);
                fb.rect_outline(r.0 - 1, r.1 - 1, 30, 30, rgba(255, 227, 110, 255));
            }
            draw_icon(
                data,
                fb,
                drag.id,
                (self.pointer.0 - 14, self.pointer.1 - 14),
                rgba(255, 255, 255, 220),
                false,
            );
        }
    }
    fn tooltip(&self, game: &Game, data: &GameData, fb: &mut Framebuffer, id: i32) {
        let Some(definition) = data.skills.iter().find(|s| s.id == id) else {
            return;
        };
        let Some(skill) = game.actor.active_skill(id) else {
            return;
        };
        let mut lines = vec![format!("{}  Lv.{}", skill_name(id), skill.level)];
        lines.extend(wrap(&definition.text("explain", &data.skill_strings), 260));
        lines.push(String::new());
        lines.push(format!(
            "MP {}   冷却 {:.1} 秒",
            skill.mp,
            skill.cooldown_ms / 1000.0
        ));
        if let Some(property) = definition.level_description(skill.level, &data.skill_strings) {
            lines.extend(wrap(&property, 260));
        }
        let bindings = self
            .slots
            .iter()
            .enumerate()
            .filter(|(_, slot)| **slot == Some(id))
            .map(|(i, _)| game.keymap.get(Command::ALL[9 + i]).to_string())
            .collect::<Vec<_>>();
        let mut command = if id == 46 {
            vec![game.keymap.get(Command::Skill).to_string()]
        } else {
            Vec::new()
        };
        command.extend(bindings);
        lines.push(format!(
            "快捷键：{}",
            if command.is_empty() {
                "未绑定".into()
            } else {
                command.join(" / ")
            }
        ));
        if skill.cancel_skill {
            lines.push("可强制中断普通攻击".into());
        }
        let remain = cooldown(game, id);
        if remain > 0.0 {
            lines.push(format!("剩余冷却：{:.1} 秒", remain / 1000.0));
        }
        let width = 278;
        let height = 14 + lines.len() as i32 * 14;
        let library = self.library_open && inside((self.pointer.0, self.pointer.1), BOOK);
        let x = (if library {
            self.pointer.0 + 20
        } else {
            self.pointer.0 - width / 2
        })
        .clamp(0, 640 - width);
        let y = (if library {
            self.pointer.1 + 20
        } else {
            SLOT_Y - height - 12
        })
        .clamp(0, 480 - height);
        popup(&data.hud_assets.popup, fb, (x, y, width, height));
        for (i, line) in lines.iter().enumerate() {
            text(
                fb,
                x + 9,
                y + 7 + i as i32 * 14,
                line,
                if i == 0 {
                    rgba(255, 228, 160, 255)
                } else {
                    rgba(235, 230, 210, 255)
                },
            );
        }
    }
}
fn cooldown(game: &Game, id: i32) -> f64 {
    game.actor
        .skill_cooldown_remaining_ms(id)
        .unwrap_or(0.0)
        .max(0.0)
}
fn ready_flash_index(remaining: Option<f64>) -> Option<usize> {
    let value = remaining?;
    (value <= 0.0 && value > -450.0).then_some((-value / 50.0) as usize)
}
fn additive(fb: &mut Framebuffer, f: &ImgFrame, x: i32, y: i32) {
    for yy in 0..f.height as usize {
        for xx in 0..f.width as usize {
            let i = (yy * f.width as usize + xx) * 4;
            let alpha = f.rgba[i + 3] as u32;
            if alpha == 0 {
                continue;
            }
            let dx = x + xx as i32 + f.x;
            let dy = y + yy as i32 + f.y;
            let old = fb.get(dx, dy);
            let add = |a: u8, b: u8| (a as u32 + b as u32 * alpha / 255).min(255) as u8;
            fb.put(
                dx,
                dy,
                rgba(
                    add(old.r, f.rgba[i]),
                    add(old.g, f.rgba[i + 1]),
                    add(old.b, f.rgba[i + 2]),
                    255,
                ),
            );
        }
    }
}
fn draw_key(base: &ImgFile, fb: &mut Framebuffer, x: i32, key: &str) {
    blit_frame(fb, &base.frames[25], x + 6, 470, false);
    if let Some(index) = ["1", "2", "3", "4", "5", "6", "A", "S", "D", "F", "G", "H"]
        .iter()
        .position(|&s| s == key)
    {
        blit_frame(fb, &base.frames[38 + index], x + 9, 473, false);
    } else {
        let label = key.chars().take(2).collect::<String>();
        dnf_render::font::draw_text(
            fb,
            x + (28 - dnf_render::font::text_width(&label)) / 2,
            473,
            &label,
            Color::BLACK,
        );
    }
}
fn draw_icon(
    data: &GameData,
    fb: &mut Framebuffer,
    id: i32,
    pos: (i32, i32),
    tint: Color,
    inactive: bool,
) {
    let Some(skill) = data.skills.iter().find(|s| s.id == id) else {
        return;
    };
    let Some((path, index)) = skill
        .icons
        .get(usize::from(inactive))
        .or_else(|| skill.icons.first())
    else {
        return;
    };
    let Some(img) = data.assets.img(path) else {
        return;
    };
    let Some(f) = img.frames.get(*index) else {
        return;
    };
    crop(fb, f, pos.0, pos.1, (pos.0, pos.1, 28, 28), tint);
}
fn crop(fb: &mut Framebuffer, f: &ImgFrame, x: i32, y: i32, r: (i32, i32, i32, i32), tint: Color) {
    if f.rgba.is_empty() {
        return;
    }
    for yy in r.1.max(y)..(r.1 + r.3).min(y + f.height as i32) {
        for xx in r.0.max(x)..(r.0 + r.2).min(x + f.width as i32) {
            let i = ((yy - y) as usize * f.width as usize + (xx - x) as usize) * 4;
            let mul = |a: u8, b: u8| (a as u32 * b as u32 / 255) as u8;
            fb.blend(
                xx,
                yy,
                rgba(
                    mul(f.rgba[i], tint.r),
                    mul(f.rgba[i + 1], tint.g),
                    mul(f.rgba[i + 2], tint.b),
                    mul(f.rgba[i + 3], tint.a),
                ),
            );
        }
    }
}
fn popup(img: &ImgFile, fb: &mut Framebuffer, r: (i32, i32, i32, i32)) {
    for row in 0..3 {
        for col in 0..3 {
            let f = &img.frames[row * 3 + col];
            let x = if col == 0 {
                r.0
            } else if col == 2 {
                r.0 + r.2 - 6
            } else {
                r.0 + 6
            };
            let y = if row == 0 {
                r.1
            } else if row == 2 {
                r.1 + r.3 - 6
            } else {
                r.1 + 6
            };
            let w = if col == 1 { r.2 - 12 } else { 6 };
            let h = if row == 1 { r.3 - 12 } else { 6 };
            let mut yy = y;
            while yy < y + h {
                let mut xx = x;
                while xx < x + w {
                    crop(fb, f, xx, yy, (x, y, w, h), Color::WHITE);
                    xx += f.width.max(1) as i32;
                }
                yy += f.height.max(1) as i32;
            }
        }
    }
}
pub(crate) fn wrap(s: &str, width: i32) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    let mut used = 0;
    for ch in s.chars() {
        if ch == '\r' {
            continue;
        }
        let size = if ch.is_ascii() { 6 } else { 12 };
        if ch == '\n' || used + size > width {
            lines.push(std::mem::take(&mut line));
            used = 0;
        }
        if ch != '\n' {
            line.push(ch);
            used += size;
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn loss_band_holds_500ms_then_drains_65px_per_second_and_recovery_trims() {
        let mut g = Gauge::default();
        g.update(150, 150, 0.0);
        g.update(75, 150, 0.0);
        assert_eq!(g.height, 32);
        assert_eq!(g.losses, vec![(65, 0.0)]);
        g.update(75, 150, 499.0);
        assert_eq!(drain(g.losses[0].1), 0);
        g.update(75, 150, 251.0);
        assert_eq!(65 - drain(g.losses[0].1), 49);
        g.update(100, 150, 0.0);
        assert_eq!(g.height, 43);
        g.update(100, 150, 350.0);
        assert!(g.losses.is_empty());
        g.update(20, 150, 0.0);
        g.update(0, 150, 20.0);
        assert_eq!(g.losses.len(), 2);
        g.update(150, 150, 0.0);
        assert!(g.losses.is_empty());
    }
    #[test]
    fn numeric_capsule_values_only_draw_on_capsule_hover_and_keys_use_base_glyphs() {
        let Some((data, mut game)) = fixture() else {
            return;
        };
        let mut hidden = Framebuffer::default();
        game.render(&data, &mut hidden);
        for (i, p) in [(0, (22, 440, false)), (1, (617, 440, false))] {
            game.hud.pointer = p;
            assert_eq!(game.hud.capsule_hover(), Some(i));
            let mut hover = Framebuffer::default();
            game.render(&data, &mut hover);
            assert_ne!(
                &hidden.data[390 * 640 * 4..410 * 640 * 4],
                &hover.data[390 * 640 * 4..410 * 640 * 4]
            );
        }
        game.hud.pointer = (50, 440, false);
        assert_eq!(game.hud.capsule_hover(), None);
        for (i, key) in ["A", "S", "D", "F", "G", "H"].iter().enumerate() {
            let mut actual = Framebuffer::default();
            draw_key(&data.hud_assets.base, &mut actual, 381, key);
            let mut expected = Framebuffer::default();
            blit_frame(
                &mut expected,
                &data.hud_assets.base.frames[25],
                387,
                470,
                false,
            );
            blit_frame(
                &mut expected,
                &data.hud_assets.base.frames[44 + i],
                390,
                473,
                false,
            );
            assert_eq!(actual.data, expected.data);
        }
    }
    #[test]
    fn completed_cooldown_uses_original_nine_50ms_frames_without_startup_flash() {
        let Some((data, mut game)) = fixture() else {
            return;
        };
        assert_eq!(data.hud_assets.ready.frames.len(), 9);
        assert_eq!(
            ready_flash_index(game.actor.skill_cooldown_remaining_ms(5)),
            None
        );
        tick(&mut game, Some(Dik::A));
        game.actor.advance_hit_stop(8000.0);
        assert_eq!(
            ready_flash_index(game.actor.skill_cooldown_remaining_ms(5)),
            Some(0)
        );
        game.actor.advance_hit_stop(50.0);
        assert_eq!(
            ready_flash_index(game.actor.skill_cooldown_remaining_ms(5)),
            Some(1)
        );
        game.actor.advance_hit_stop(350.0);
        assert_eq!(
            ready_flash_index(game.actor.skill_cooldown_remaining_ms(5)),
            Some(8)
        );
        game.actor.advance_hit_stop(50.0);
        assert_eq!(
            ready_flash_index(game.actor.skill_cooldown_remaining_ms(5)),
            None
        );
    }
    #[test]
    fn closing_book_captures_whole_mouse_press_and_focus_baseline_waits_for_release() {
        let mut hud = Hud {
            library_open: true,
            ..Hud::default()
        };
        let keys = InputState::new();
        assert!(hud.input(&keys, (449, 25, true)).captures_pointer);
        assert!(!hud.library_open);
        assert!(hud.input(&keys, (527, 250, true)).captures_pointer);
        assert!(hud.input(&keys, (527, 250, false)).captures_pointer);
        assert!(!hud.input(&keys, (527, 250, false)).captures_pointer);
        hud.pointer_baseline((395, 461, true));
        hud.input(&keys, (395, 461, true));
        assert!(!hud.dragging());
        hud.input(&keys, (395, 461, false));
        hud.input(&keys, (395, 461, true));
        assert!(hud.dragging());
    }
    #[test]
    fn library_drag_moves_existing_binding_and_shortcut_drag_swaps_without_duplicates() {
        let mut hud = Hud {
            library_open: true,
            ..Hud::default()
        };
        hud.slots[2] = None; // explicit free destination for this drag test
        let keys = InputState::new();
        assert!(hud.input(&keys, (226, 83, true)).captures_pointer);
        assert_eq!(hud.slots, [Some(5), Some(46), None, None, None, None]);
        assert!(hud.input(&keys, (457, 461, false)).changed);
        assert_eq!(hud.slots, [None, Some(46), Some(5), None, None, None]);
        hud.input(&keys, (457, 461, true));
        assert!(hud.input(&keys, (426, 461, false)).changed);
        assert_eq!(hud.slots, [None, Some(5), Some(46), None, None, None]);
        hud.input(&keys, (426, 461, true));
        hud.input(&keys, (300, 150, false));
        assert_eq!(hud.slots, [None, None, Some(46), None, None, None]);
        let raw = hud.bindings_text();
        let mut loaded = Hud::default();
        loaded.load_bindings(&raw).unwrap();
        assert_eq!(loaded.slots, hud.slots);
        let before = loaded.slots;
        assert!(loaded.load_bindings("slot1=5\nslot2=5").is_err());
        assert!(loaded.load_bindings("slot1=117").is_err());
        assert_eq!(loaded.slots, before);
    }
    #[test]
    fn escape_and_focus_loss_cancel_pending_drag_without_losing_binding() {
        let mut hud = Hud::default();
        let mut keys = InputState::new();
        hud.input(&keys, (395, 461, true));
        keys.set(Dik::ESCAPE, true);
        assert!(hud.input(&keys, (457, 461, true)).escape_handled);
        assert!(!hud.input(&keys, (457, 461, false)).changed);
        assert_eq!(hud.slots, Hud::default().slots);
        keys.begin_frame();
        hud.input(&keys, (395, 461, true));
        hud.cancel_drag();
        hud.input(&keys, (457, 461, false));
        assert_eq!(hud.slots, Hud::default().slots);
    }
    #[test]
    fn wrapped_japanese_description_stays_inside_tooltip_width() {
        let lines = wrap(
            "武器に鬼神の力を乗せて前方の広範囲に強い物理攻撃を行う。",
            260,
        );
        assert!(lines.len() > 1);
        for line in lines {
            assert!(crate::ui::measure_text(&line).0 <= 260);
        }
    }
    fn fixture() -> Option<(GameData, Game)> {
        let dir = dnf_data::locate_client_dir()?;
        let data = GameData::load(
            &dir,
            crate::game::DEFAULT_MAP,
            crate::character::Outfit::default_basic(),
        )
        .unwrap();
        let game = data
            .build_game(dnf_core::keymap::KeyMap::official_default())
            .unwrap();
        Some((data, game))
    }
    fn tick(game: &mut Game, key: Option<Dik>) {
        game.begin_input();
        if let Some(key) = key {
            game.set_key(key, true);
        }
        game.step();
    }
    #[test]
    fn moved_binding_casts_its_skill_and_keeps_same_cooldown_while_library_is_open() {
        let Some((_data, mut game)) = fixture() else {
            return;
        };
        game.hud.slots[2] = None;
        tick(&mut game, Some(Dik::K));
        game.panel.pointer = (226, 83, true);
        tick(&mut game, None);
        game.panel.pointer = (457, 461, false);
        tick(&mut game, None);
        assert_eq!(game.hud.slots[2], Some(5));
        assert_eq!(game.hud.slots[0], None);
        assert!(game.sound_requests.contains(&"ICON_QUICK_SET"));
        tick(&mut game, Some(Dik::A));
        assert_eq!(game.actor.mp, 150);
        tick(&mut game, Some(Dik::D));
        assert_eq!(game.actor.mp, 130);
        let remain = game.actor.ghost_cooldown_ms();
        game.panel.pointer = (457, 461, true);
        tick(&mut game, None);
        game.panel.pointer = (550, 461, false);
        tick(&mut game, None);
        assert_eq!(game.hud.slots[5], Some(5));
        assert!(
            game.actor.ghost_cooldown_ms() < remain
                && game.actor.ghost_cooldown_ms() > remain - 40.0
        );
        assert_eq!(game.actor.skill_slots[5], Some(5));
        for _ in 0..520 {
            tick(&mut game, None);
        }
        tick(&mut game, Some(Dik::H));
        assert_eq!(game.actor.mp, 110);
        game.actor.teleport(200, 329);
        tick(&mut game, None);
        tick(&mut game, Some(Dik::S));
        assert_eq!(game.actor.mp, 104);
        assert_eq!(game.actor.motion, dnf_core::actor::Motion::UpperSlash);
    }
    #[test]
    fn actual_gauges_crop_and_hud_draws_after_panel_and_combat_popups() {
        let Some((data, mut game)) = fixture() else {
            return;
        };
        let mut full = Framebuffer::default();
        game.render(&data, &mut full);
        game.apply_debug(crate::ui::Command::Set(0, 0));
        game.apply_debug(crate::ui::Command::Set(2, 0));
        game.hud.update_gauges(0, 150, 0, 150, 0.0);
        game.hud.update_gauges(0, 150, 0, 150, 1501.0);
        game.panel.open = true;
        game.popups.push(crate::damage::DamagePopup {
            damage: 888,
            point: [25.0, 470.0, 0.0],
            age_ms: 0.0,
            style: 0,
        });
        let mut empty = Framebuffer::default();
        game.render(&data, &mut empty);
        assert!(full.get(22, 440).r > empty.get(22, 440).r);
        assert!(full.get(617, 440).b > empty.get(617, 440).b);
        let expected = &data.hud_assets.base.frames[0];
        let mut pixels = Framebuffer::default();
        blit_frame(&mut pixels, expected, 0, 402, false);
        assert_eq!(empty.get(48, 462), pixels.get(48, 462));
        game.hud.library_open = true;
        game.hud.pointer = (226, 83, false);
        assert_eq!(game.hud.hovered_skill(), Some(5));
        let skill = data.skills.iter().find(|s| s.id == 5).unwrap();
        assert!(skill.text("explain", &data.skill_strings).contains("鬼神"));
        assert!(skill
            .level_description(1, &data.skill_strings)
            .unwrap()
            .contains("+40"));
    }
}
