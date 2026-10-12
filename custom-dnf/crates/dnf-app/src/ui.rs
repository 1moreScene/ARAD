//! Original status/inventory artwork with session-only debug editors.
use crate::{
    assets::ClientAssets,
    character,
    game::{Game, GameData},
    room::blit_frame,
};
use dnf_core::{input::InputState, keys::Dik};
use dnf_data::img::ImgFile;
use dnf_render::framebuffer::{rgba, Color, Framebuffer};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub struct UiAssets {
    pub status: Rc<ImgFile>,
    pub portrait: Rc<ImgFile>,
    pub inventory: Rc<ImgFile>,
    pub buttons: Rc<ImgFile>,
    pub numbers: Rc<ImgFile>,
    pub weapon_icon: Rc<ImgFile>,
    pub icon_index: usize,
}
impl UiAssets {
    pub fn load(a: &ClientAssets, weapon: &str) -> Result<Self, String> {
        let load = |path: &str| {
            a.img(path)
                .ok_or_else(|| format!("UI asset missing {path}"))
        };
        let script = a.pvf.read_text(weapon).map_err(|e| e.to_string())?;
        let at = script.find("[icon]").ok_or("weapon icon missing")?;
        let mut tokens = script[at + 6..].split_whitespace();
        let icon = tokens.next().ok_or("icon path missing")?.trim_matches('`');
        let icon_index: usize = tokens
            .next()
            .ok_or("icon index missing")?
            .parse()
            .map_err(|_| "icon index invalid")?;
        let out = Self {
            status: load("Interface/StatusWindow.img")?,
            portrait: load("Interface/StatusIllust.img")?,
            inventory: load("Interface/ItemInventoryWindow.img")?,
            buttons: load("Interface/SkillBuyWindow.img")?,
            numbers: load("Common/Etc/Combo.img")?,
            weapon_icon: load(icon)?,
            icon_index,
        };
        if out.numbers.frames.len() < 153
            || out.buttons.frames.len() < 14
            || out.weapon_icon.frames.get(icon_index).is_none()
        {
            return Err("UI atlas frame mismatch".into());
        }
        Ok(out)
    }
}
pub const COUNT: usize = 20;
pub const LABELS: [&str; COUNT] = [
    "当前 HP",
    "HP 上限",
    "当前 MP",
    "MP 上限",
    "物理攻击",
    "物理防御",
    "魔法攻击",
    "魔法防御",
    "力量",
    "智力",
    "体力",
    "精神",
    "等级",
    "移动速度",
    "攻击速度",
    "跳跃力",
    "跳跃速度",
    "重量",
    "哥布林上限",
    "飘字样式",
];
const PAGES: [&[usize]; 3] = [
    &[0, 1, 2, 3, 4, 18, 19],
    &[13, 14, 15, 16, 17, 12],
    &[8, 9, 10, 11, 5, 6, 7],
];
pub fn bounds(field: usize) -> (i32, i32) {
    match field {
        12 => (1, 60),
        13 | 14 => (100, 3000),
        15 => (1, 2000),
        16 => (1, 500),
        17 => (1000, 200000),
        19 => (0, 5),
        1 | 3 | 18 => (1, 999999),
        _ => (0, 999999),
    }
}
#[derive(Default)]
pub struct Panel {
    pub open: bool,
    pub debug: bool,
    pub page: usize,
    pub selected: Option<usize>,
    draft: String,
    replace: bool,
    pub pointer: (i32, i32, bool),
    mouse_down: bool,
    pub message: String,
}
pub enum Command {
    Set(usize, i32),
    Reset,
}
impl Panel {
    pub fn acknowledge(&mut self, values: &[i32; COUNT]) {
        if let Some(field) = self.selected {
            self.draft = values[field].to_string();
            self.replace = true;
        }
    }
    fn select(&mut self, field: usize, values: &[i32; COUNT]) {
        self.selected = Some(field);
        self.draft = values[field].to_string();
        self.replace = true;
        self.message.clear();
    }
    fn commit(&mut self) -> Option<Command> {
        let field = self.selected?;
        let (low, high) = bounds(field);
        match self.draft.parse::<i32>() {
            Ok(v) if (low..=high).contains(&v) => {
                self.message = "已应用，本次运行有效".into();
                self.replace = true;
                Some(Command::Set(field, v))
            }
            _ => {
                self.message = format!("请输入 {low}–{high}");
                None
            }
        }
    }
    pub fn input(&mut self, keys: &InputState, values: &[i32; COUNT]) -> (bool, Option<Command>) {
        let click = self.pointer.2 && !self.mouse_down;
        self.mouse_down = self.pointer.2;
        if keys.just_pressed(Dik::M) || keys.just_pressed(Dik::I) {
            self.open = !self.open;
            self.selected = None;
            return (true, None);
        }
        if !self.open {
            return (false, None);
        }
        if keys.just_pressed(Dik::ESCAPE) {
            self.open = false;
            self.selected = None;
            return (true, None);
        }
        if keys.just_pressed(Dik::F6) {
            self.debug = !self.debug;
            self.selected = None;
        }
        let (x, y, _) = self.pointer;
        if click
            && ((250..269).contains(&x) && (416..435).contains(&y)
                || (566..585).contains(&x) && (415..434).contains(&y))
        {
            self.open = false;
            return (true, None);
        }
        if !self.debug {
            return (true, None);
        }
        if click && (340..592).contains(&x) && (218..241).contains(&y) {
            self.page = ((x - 340) / 84).min(2) as usize;
            self.selected = None;
        }
        if keys.just_pressed(Dik::TAB) {
            let fields = PAGES[self.page];
            let index = self
                .selected
                .and_then(|f| fields.iter().position(|&x| x == f))
                .map_or(0, |i| (i + 1) % fields.len());
            self.select(fields[index], values);
        }
        if click && (340..592).contains(&x) && (248..395).contains(&y) {
            let row = ((y - 248) / 21) as usize;
            if let Some(&f) = PAGES[self.page].get(row) {
                if x < 548 {
                    self.select(f, values);
                } else {
                    let (lo, hi) = bounds(f);
                    let step = if matches!(f, 13 | 14 | 17) { 10 } else { 1 };
                    let value = (values[f] + if x < 570 { -step } else { step }).clamp(lo, hi);
                    self.select(f, values);
                    self.draft = value.to_string();
                    return (true, self.commit());
                }
            }
        }
        if click && (340..452).contains(&x) && (403..425).contains(&y) {
            return (true, self.commit());
        }
        if click && (466..562).contains(&x) && (403..425).contains(&y) {
            self.selected = None;
            self.message = "已恢复默认值".into();
            return (true, Some(Command::Reset));
        }
        if self.selected.is_some() {
            for (key, digit) in [
                (Dik::KEY_0, '0'),
                (Dik::KEY_1, '1'),
                (Dik::KEY_2, '2'),
                (Dik::KEY_3, '3'),
                (Dik::KEY_4, '4'),
                (Dik::KEY_5, '5'),
                (Dik::KEY_6, '6'),
                (Dik::KEY_7, '7'),
                (Dik::KEY_8, '8'),
                (Dik::KEY_9, '9'),
                (Dik::NUMPAD0, '0'),
                (Dik::NUMPAD1, '1'),
                (Dik::NUMPAD2, '2'),
                (Dik::NUMPAD3, '3'),
                (Dik::NUMPAD4, '4'),
                (Dik::NUMPAD5, '5'),
                (Dik::NUMPAD6, '6'),
                (Dik::NUMPAD7, '7'),
                (Dik::NUMPAD8, '8'),
                (Dik::NUMPAD9, '9'),
            ] {
                if keys.just_pressed(key) {
                    if self.replace {
                        self.draft.clear();
                        self.replace = false;
                    }
                    if self.draft.len() < 6 {
                        self.draft.push(digit);
                    }
                }
            }
            if keys.just_pressed(Dik::BACK) {
                if self.replace {
                    self.draft.clear();
                    self.replace = false;
                } else {
                    self.draft.pop();
                }
            }
            if keys.just_pressed(Dik::RETURN) {
                return (true, self.commit());
            }
        }
        (true, None)
    }
    pub fn draw(&self, g: &Game, data: &GameData, fb: &mut Framebuffer) {
        if !self.open {
            return;
        }
        let a = &data.ui;
        let white = rgba(238, 232, 216, 255);
        let gold = rgba(225, 195, 126, 255);
        draw_img(fb, &a.status, 0, 29, 63);
        draw_img(fb, &a.portrait, 0, 48, 85);
        draw_img(fb, &a.inventory, 0, 320, 33);
        let v = g.debug_values();
        text(fb, 159, 89, "冒险家", white);
        text(fb, 159, 110, "鬼剑士", white);
        text(fb, 172, 131, &format!("Lv.{} / 0%", v[12]), white);
        text(fb, 159, 153, "未参与", white);
        for (field, x, y) in [
            (0, 116, 191),
            (2, 249, 191),
            (8, 116, 218),
            (10, 249, 218),
            (9, 116, 245),
            (11, 249, 245),
            (4, 116, 279),
            (5, 249, 279),
            (6, 116, 306),
            (7, 249, 306),
        ] {
            let s = if field == 0 {
                format!("{}/{}", v[0], v[1])
            } else if field == 2 {
                format!("{}/{}", v[2], v[3])
            } else {
                v[field].to_string()
            };
            text(fb, x, y, &s, white);
        }
        for (i, (x, y)) in [(116, 361), (249, 361), (116, 388), (249, 388)]
            .iter()
            .enumerate()
        {
            text(fb, *x, *y, &data.resistances[i].to_string(), white);
        }
        draw_img(fb, &a.buttons, 11, 250, 416);
        draw_img(fb, &a.buttons, 11, 566, 415);
        // Inventory mannequin uses exactly the same script-layer composition as the actor.
        for layer in character::compose(
            &data.assets,
            &data.avatar,
            &data.visual,
            &data.outfit,
            &data.weapon,
            "Stay",
            0,
        ) {
            if let Some(f) = layer.image.frames.get(layer.frame) {
                blit_frame(fb, f, 465 + layer.offset.0, 171 + layer.offset.1, false);
            }
        }
        draw_img(fb, &a.weapon_icon, a.icon_index, 531, 112);
        text(fb, 342, 174, "损坏的铁剑 · 已装备", gold);
        if self.debug {
            fb.fill_rect(336, 212, 260, 217, rgba(21, 22, 21, 250));
            for (i, label) in ["基本 / 战斗", "动作参数", "角色属性"].iter().enumerate()
            {
                button(
                    fb,
                    (340 + i as i32 * 84, 218, 82, 23),
                    label,
                    self.page == i,
                );
            }
            for (row, &field) in PAGES[self.page].iter().enumerate() {
                let y = 248 + row as i32 * 21;
                text(fb, 344, y + 3, LABELS[field], gold);
                fb.fill_rect(446, y, 98, 19, rgba(8, 9, 10, 255));
                let selected = self.selected == Some(field);
                fb.rect_outline(
                    446,
                    y,
                    98,
                    19,
                    if selected {
                        gold
                    } else {
                        rgba(78, 76, 65, 255)
                    },
                );
                let value = if selected {
                    self.draft.clone()
                } else {
                    v[field].to_string()
                };
                text(fb, 452, y + 3, &value, white);
                button(fb, (548, y, 20, 19), "−", false);
                button(fb, (572, y, 20, 19), "+", false);
            }
            button(fb, (340, 403, 112, 22), "应用 [Enter]", false);
            button(fb, (466, 403, 96, 22), "恢复默认", false);
        } else {
            text(fb, 342, 231, "装备栏：当前加载的默认武器", white);
            text(fb, 342, 256, "F6 打开独立调试控制台", gold);
            text(fb, 342, 281, "打开面板时暂停战斗", white);
        }
        if (528..562).contains(&self.pointer.0) && (108..144).contains(&self.pointer.1) {
            fb.fill_rect(384, 137, 214, 46, rgba(10, 12, 16, 245));
            fb.rect_outline(384, 137, 214, 46, gold);
            text(fb, 392, 144, "损坏的铁剑 · 短剑 · Lv.1", white);
            text(fb, 392, 162, "官方装备 27000 / 武器物攻 13", gold);
        }
        fb.fill_rect(29, 446, 582, 17, rgba(0, 0, 0, 215));
        text(
            fb,
            34,
            448,
            if self.selected == Some(19) {
                "飘字：0 黄普通 · 1 红暴击 · 2 绿 · 3 蓝 · 4 受伤 · 5 灰白附加"
            } else if self.debug && self.page == 2 && self.message.is_empty() {
                "属性用于面板调试；当前攻击结算使用“物理攻击”"
            } else if self.message.is_empty() {
                "M / I 关闭 · F6 调试控制台 · Esc 关闭面板"
            } else {
                &self.message
            },
            gold,
        );
    }
}
fn draw_img(fb: &mut Framebuffer, img: &ImgFile, index: usize, x: i32, y: i32) {
    if let Some(f) = img.frames.get(index) {
        blit_frame(fb, f, x + f.x, y + f.y, false);
    }
}
fn button(fb: &mut Framebuffer, r: (i32, i32, i32, i32), s: &str, active: bool) {
    fb.fill_rect(
        r.0,
        r.1,
        r.2,
        r.3,
        if active {
            rgba(78, 66, 44, 255)
        } else {
            rgba(40, 39, 34, 255)
        },
    );
    fb.rect_outline(r.0, r.1, r.2, r.3, rgba(144, 124, 80, 255));
    text(fb, r.0 + 5, r.1 + 4, s, rgba(230, 221, 200, 255));
}
type TextBitmap = (usize, usize, Vec<u8>);
thread_local! { static TEXT: RefCell<HashMap<String,TextBitmap>> = RefCell::new(HashMap::new()); }
pub fn measure_text(s: &str) -> (i32, i32) {
    #[cfg(windows)]
    {
        TEXT.with(|cache| {
            let mut cache = cache.borrow_mut();
            if !cache.contains_key(s) {
                if let Some(bitmap) = crate::win32::raster_text(s) {
                    cache.insert(s.into(), bitmap);
                }
            }
            cache
                .get(s)
                .map(|(w, h, _)| (*w as i32, *h as i32))
                .unwrap_or((0, 0))
        })
    }
    #[cfg(not(windows))]
    {
        (dnf_render::font::text_width(s), 7)
    }
}
pub fn text(fb: &mut Framebuffer, x: i32, y: i32, s: &str, color: Color) {
    #[cfg(windows)]
    TEXT.with(|cache| {
        let mut cache = cache.borrow_mut();
        if !cache.contains_key(s) {
            if cache.len() > 512 {
                cache.clear();
            }
            if let Some(bitmap) = crate::win32::raster_text(s) {
                cache.insert(s.into(), bitmap);
            }
        }
        if let Some((w, h, pixels)) = cache.get(s) {
            fb.blit_alpha(pixels, *w, *h, x, y, false, color);
        }
    });
    #[cfg(not(windows))]
    dnf_render::font::draw_text(fb, x, y, s, color);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invalid_edit_is_rejected_and_escape_only_closes_panel() {
        let mut p = Panel {
            open: true,
            debug: true,
            page: 1,
            ..Panel::default()
        };
        let mut keys = InputState::new();
        let values = [850; COUNT];
        keys.begin_frame();
        keys.set(Dik::TAB, true);
        p.input(&keys, &values);
        assert_eq!(p.selected, Some(13));
        keys.begin_frame();
        keys.set(Dik::KEY_0, true);
        p.input(&keys, &values);
        keys.begin_frame();
        keys.set(Dik::RETURN, true);
        let (consumed, command) = p.input(&keys, &values);
        assert!(consumed && command.is_none());
        assert!(p.message.contains("100–3000"));
        keys.begin_frame();
        keys.set(Dik::ESCAPE, true);
        assert!(p.input(&keys, &values).0);
        assert!(!p.open);
    }
    #[test]
    fn mouse_plus_is_one_click_and_acknowledged_clamp_is_visible() {
        let mut p = Panel {
            open: true,
            debug: true,
            pointer: (580, 250, true),
            ..Panel::default()
        };
        let keys = InputState::new();
        let mut values = [150; COUNT];
        assert!(matches!(
            p.input(&keys, &values).1,
            Some(Command::Set(0, 151))
        ));
        assert!(p.input(&keys, &values).1.is_none());
        values[0] = 150;
        p.acknowledge(&values);
        assert_eq!(p.draft, "150");
    }
}
