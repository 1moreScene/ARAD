//! Dedicated hand-drawn training controls. F6 opens; all game commands are
//! captured while editing. Rendering diagnostics live here, never in the scene.
use crate::{
    game::{Game, GameData},
    ui,
};
use dnf_core::active_status::ActiveStatus;
use dnf_core::{input::InputState, keys::Dik, protection::Protection};
use dnf_render::framebuffer::{rgba, Color, Framebuffer};

pub const MAX_MONSTERS: usize = 32;
const PLAYER_PAGES: [&[usize]; 2] = [
    &[0, 1, 2, 3, 4, 5, 8, 9, 10, 11, 6, 7],
    &[13, 14, 15, 16, 17, 12, 19],
];
const MONSTER_LABELS: [&str; 6] = [
    "怪物数量",
    "HP 上限",
    "当前 HP",
    "物理防御",
    "重量",
    "硬直恢复",
];

pub enum Control {
    PlayerSet(usize, i32),
    MonsterSet {
        target: Option<usize>,
        field: usize,
        value: i32,
    },
    PlayerState(Protection),
    PlayerGrabable(bool),
    MonsterGrabable {
        target: Option<usize>,
        value: bool,
    },
    MonsterState {
        target: Option<usize>,
        state: Protection,
    },
    HealPlayer,
    ResetPlayer,
    Respawn(Option<usize>),
    ResetMonsters,
    MonsterStatusAdd {
        target: Option<usize>,
        kind: ActiveStatus,
        level: u8,
    },
    MonsterStatusRemove {
        target: Option<usize>,
        kind: ActiveStatus,
    },
    MonsterStatusClear {
        target: Option<usize>,
    },
    MonsterDemo {
        target: Option<usize>,
        enabled: bool,
    },
}

pub struct View {
    pub player: [i32; ui::COUNT],
    pub monster: [i32; 6],
    pub count: usize,
    pub player_state: Protection,
    pub monster_state: Option<Protection>,
    pub player_grabable: bool,
    pub monster_grabable: Option<bool>,
    pub mixed: [bool; 6],
    pub status_rows: Vec<(ActiveStatus, u8, f64, bool)>,
    pub status_mask: u32,
    pub demo: bool,
}
impl View {
    pub fn from_game(g: &Game) -> Self {
        let m = g.monster(g.debug_panel.selected_monster);
        let mut mixed = [false; 6];
        if !g.debug_panel.single_monster {
            if let Some(first) = m {
                for candidate in g.monsters() {
                    mixed[1] |= candidate.max_hp != first.max_hp;
                    mixed[2] |= candidate.hp != first.hp;
                    mixed[3] |= candidate.defense != first.defense;
                    mixed[4] |= candidate.weight != first.weight;
                    mixed[5] |= candidate.hit_recovery != first.hit_recovery;
                }
            }
        }
        let mut state = g.monster_protection(g.debug_panel.selected_monster);
        if !g.debug_panel.single_monster
            && (0..g.monster_count()).any(|i| g.monster_protection(i) != state)
        {
            state = None;
        }
        let mut player = g.debug_values();
        player[8] -= g.grab_skill.strength_bonus(); // edit base STR, never bake a timed buff into it
        Self {
            status_rows: ActiveStatus::ALL
                .into_iter()
                .filter_map(|kind| {
                    let selected = g.monsters().enumerate().filter(|(i, _)| {
                        !g.debug_panel.single_monster || *i == g.debug_panel.selected_monster
                    });
                    let mut entries = selected.filter_map(|(_, m)| m.statuses.get(kind));
                    let first = entries.next()?;
                    let mixed = entries.any(|s| {
                        s.level != first.level || (s.remaining_ms - first.remaining_ms).abs() > 1.0
                    }) || (!g.debug_panel.single_monster
                        && g.monsters().any(|m| !m.statuses.has(kind)));
                    Some((kind, first.level, first.remaining_ms, mixed))
                })
                .collect(),
            status_mask: g
                .monsters()
                .enumerate()
                .filter(|(i, _)| {
                    !g.debug_panel.single_monster || *i == g.debug_panel.selected_monster
                })
                .fold(0, |mask, (_, m)| mask | m.statuses.mask()),
            demo: if g.debug_panel.single_monster {
                m.is_some_and(|m| m.demo_enabled)
            } else {
                g.monsters().any(|m| m.demo_enabled)
            },
            player_grabable: g.actor.grab_state.grabable,
            monster_grabable: m.map(|m| m.grab.grabable).filter(|&value| {
                g.debug_panel.single_monster || g.monsters().all(|m| m.grab.grabable == value)
            }),
            mixed,
            player,
            count: g.monster_count(),
            monster: [
                g.monster_count() as i32,
                m.map_or(0, |m| m.max_hp),
                m.map_or(0, |m| m.hp),
                m.map_or(0, |m| m.defense),
                m.map_or(0, |m| m.weight as i32),
                m.map_or(0, |m| m.hit_recovery),
            ],
            player_state: g.actor.protection.effective(),
            monster_state: state,
        }
    }
}
#[derive(Default)]
pub struct DebugPanel {
    pub open: bool,
    pub tab: usize,
    pub player_page: usize,
    pub selected_monster: usize,
    pub single_monster: bool,
    pub monster_page: usize,
    pub status_choice: usize,
    pub status_level: u8,
    pub status_page: usize,
    pub status_dropdown: bool,
    selected: Option<usize>,
    draft: String,
    replace: bool,
    mouse_down: bool,
    pub message: String,
}
fn contains(r: (i32, i32, i32, i32), p: (i32, i32)) -> bool {
    p.0 >= r.0 && p.0 < r.0 + r.2 && p.1 >= r.1 && p.1 < r.1 + r.3
}
fn row_pos(row: usize, monsters: bool) -> (i32, i32) {
    (
        if row % 2 == 0 { 44 } else { 328 },
        (if monsters { 184 } else { 138 }) + (row / 2) as i32 * 28,
    )
}
fn monster_bounds(field: usize) -> (i32, i32) {
    match field {
        0 => (0, MAX_MONSTERS as i32),
        1 => (1, 999999),
        4 => (1000, 200000),
        5 => (0, 2000),
        _ => (0, 999999),
    }
}
impl DebugPanel {
    fn draw_status(&self, g: &Game, fb: &mut Framebuffer, v: &View) {
        let white = rgba(226, 232, 242, 255);
        let muted = rgba(146, 163, 183, 255);
        let cyan = rgba(88, 209, 215, 255);
        let kind = ActiveStatus::ALL[self.status_choice];
        let level = self.status_level.max(1);
        button(fb, (480, 148, 100, 22), "巡逻演示", v.demo);
        button(
            fb,
            (44, 184, 280, 24),
            &format!("{}   ▾", kind.name()),
            self.status_dropdown,
        );
        ui::text(fb, 338, 190, &format!("等级 {} / 40", level), white);
        button(fb, (438, 184, 26, 24), "−", false);
        button(fb, (470, 184, 26, 24), "+", false);
        let config = g.goblin_assets.status_assets.config(kind, level);
        ui::text(
            fb,
            504,
            190,
            &format!("{:.1}s", config.duration_ms / 1000.0),
            muted,
        );
        button(fb, (44, 220, 100, 24), "添加 / 刷新", kind.implemented());
        button(fb, (154, 220, 100, 24), "移除所选", false);
        button(fb, (264, 220, 100, 24), "清空状态", false);
        ui::text(fb, 44, 249, kind.description(), cyan);
        ui::text(
            fb,
            44,
            266,
            &format!(
                "当前 {} 种状态 · 点击状态移除 · 混合表示部分怪物拥有",
                v.status_rows.len()
            ),
            muted,
        );
        for (row, (kind, level, ms, mixed)) in v
            .status_rows
            .iter()
            .skip(self.status_page * 10)
            .take(10)
            .enumerate()
        {
            let x = 44 + (row % 2) as i32 * 284;
            let y = 282 + (row / 2) as i32 * 20;
            fb.fill_rect(x, y, 268, 19, rgba(12, 19, 29, 255));
            ui::text(
                fb,
                x + 5,
                y + 4,
                &format!(
                    "{} Lv{}  {:.1}s{}",
                    kind.name(),
                    level,
                    ms / 1000.0,
                    if *mixed { "（混合）" } else { "" }
                ),
                white,
            );
        }
        if v.status_rows.is_empty() {
            ui::text(
                fb,
                44,
                286,
                "暂无异常状态；选择后点击添加，已有状态会保留",
                muted,
            );
        }
        button(fb, (44, 382, 28, 24), "‹", false);
        button(fb, (206, 382, 28, 24), "›", false);
        ui::text(
            fb,
            80,
            389,
            &format!(
                "状态页 {} / {}",
                self.status_page + 1,
                v.status_rows.len().saturating_sub(1) / 10 + 1
            ),
            muted,
        );
        if let Some(m) = g.monster(self.selected_monster) {
            ui::text(
                fb,
                256,
                389,
                &format!(
                    "速度×{:.2} 四维×{:.2} 火抗{:+} 水抗{:+}",
                    m.statuses.speed_rate(),
                    m.statuses.stat_rate(),
                    m.statuses.elemental_resistance_delta(0),
                    m.statuses.elemental_resistance_delta(1)
                ),
                muted,
            );
        }
        ui::text(
            fb,
            44,
            409,
            if self.message.is_empty() {
                "计时在关闭面板后继续；眩晕与睡眠保留官方互斥规则"
            } else {
                &self.message
            },
            muted,
        );
        if self.status_dropdown {
            fb.fill_rect(42, 210, 552, 184, rgba(8, 14, 24, 255));
            fb.rect_outline(42, 210, 552, 184, cyan);
            for (id, kind) in ActiveStatus::ALL.into_iter().enumerate() {
                let x = 44 + (id / 9) as i32 * 274;
                let y = 212 + (id % 9) as i32 * 20;
                if id == self.status_choice {
                    fb.fill_rect(x, y, 272, 20, rgba(37, 75, 90, 255));
                }
                ui::text(
                    fb,
                    x + 5,
                    y + 5,
                    &format!(
                        "{} {}",
                        if v.status_mask & (1 << id) != 0 {
                            "✓"
                        } else {
                            " "
                        },
                        kind.name()
                    ),
                    if kind.implemented() { white } else { muted },
                );
            }
        }
    }
    pub fn pointer_baseline(&mut self, down: bool) {
        self.mouse_down = down;
    }
    fn target(&self) -> Option<usize> {
        self.single_monster.then_some(self.selected_monster)
    }
    fn fields(&self) -> &'static [usize] {
        if self.tab == 0 {
            PLAYER_PAGES[self.player_page]
        } else {
            &[0, 1, 2, 3, 4, 5]
        }
    }
    fn value(&self, field: usize, v: &View) -> i32 {
        if self.tab == 0 {
            v.player[field]
        } else {
            v.monster[field]
        }
    }
    fn bounds(&self, field: usize) -> (i32, i32) {
        if self.tab == 0 {
            ui::bounds(field)
        } else {
            monster_bounds(field)
        }
    }
    fn select(&mut self, field: usize, v: &View) {
        self.selected = Some(field);
        self.draft = self.value(field, v).to_string();
        self.replace = true;
        self.message.clear();
    }
    pub fn acknowledge(&mut self, v: &View) {
        if let Some(field) = self.selected {
            self.draft = self.value(field, v).to_string();
            self.replace = true;
        }
    }
    fn commit(&mut self) -> Option<Control> {
        let field = self.selected?;
        let (lo, hi) = self.bounds(field);
        match self.draft.parse::<i32>() {
            Ok(value) if (lo..=hi).contains(&value) => {
                self.message = "已应用".into();
                self.replace = true;
                Some(if self.tab == 0 {
                    Control::PlayerSet(field, value)
                } else {
                    Control::MonsterSet {
                        target: self.target(),
                        field,
                        value,
                    }
                })
            }
            _ => {
                self.message = format!("请输入 {lo}–{hi}");
                None
            }
        }
    }
    pub fn input(
        &mut self,
        keys: &InputState,
        pointer: (i32, i32, bool),
        v: &View,
    ) -> Option<Control> {
        let click = pointer.2 && !self.mouse_down;
        self.mouse_down = pointer.2;
        if !self.open {
            return None;
        }
        let p = (pointer.0, pointer.1);
        if self.status_dropdown {
            if keys.just_pressed(Dik::ESCAPE) {
                self.status_dropdown = false;
                return None;
            }
            if keys.just_pressed(Dik::UP) {
                self.status_choice = (self.status_choice + 16) % 17;
            }
            if keys.just_pressed(Dik::DOWN) {
                self.status_choice = (self.status_choice + 1) % 17;
            }
            if keys.just_pressed(Dik::RETURN) {
                self.status_dropdown = false;
                return None;
            }
            if click {
                if contains((44, 212, 548, 180), p) {
                    let column = (p.0 - 44) / 274;
                    let row = (p.1 - 212) / 20;
                    let id = (column * 9 + row) as usize;
                    if id < 17 {
                        self.status_choice = id;
                    }
                }
                self.status_dropdown = false;
            }
            return None;
        }
        if keys.just_pressed(Dik::ESCAPE) || click && contains((558, 43, 30, 24), p) {
            self.open = false;
            self.selected = None;
            return None;
        }
        if click {
            for tab in 0..3 {
                if contains((44 + tab * 180, 78, 172, 28), p) {
                    self.tab = tab as usize;
                    self.selected = None;
                    self.message.clear();
                    return None;
                }
            }
            if self.tab == 0 {
                for page in 0..2 {
                    if contains((44 + page * 120, 110, 112, 22), p) {
                        self.player_page = page as usize;
                        self.selected = None;
                        return None;
                    }
                }
            } else if self.tab == 1 {
                for page in 0..2 {
                    if contains((44 + page * 120, 148, 112, 22), p) {
                        self.monster_page = page as usize;
                        self.selected = None;
                        return None;
                    }
                }
                if contains((44, 116, 82, 26), p) {
                    self.single_monster = false;
                    self.selected = None;
                    return None;
                }
                if contains((136, 116, 82, 26), p) {
                    self.single_monster = true;
                    self.selected = None;
                    return None;
                }
                if contains((230, 116, 28, 26), p) && v.count > 0 {
                    self.selected_monster = if self.selected_monster == 0 {
                        v.count - 1
                    } else {
                        self.selected_monster - 1
                    };
                    self.single_monster = true;
                    self.selected = None;
                    return None;
                }
                if contains((390, 116, 28, 26), p) && v.count > 0 {
                    self.selected_monster = (self.selected_monster + 1) % v.count;
                    self.single_monster = true;
                    self.selected = None;
                    return None;
                }
            }
        }
        if self.tab == 2 {
            return None;
        }
        if self.tab == 1 && self.monster_page == 1 {
            let kind = ActiveStatus::ALL[self.status_choice];
            let level = self.status_level.max(1);
            if click {
                if contains((44, 184, 280, 24), p) {
                    self.status_dropdown = true;
                    return None;
                }
                if contains((438, 184, 26, 24), p) {
                    self.status_level = level.saturating_sub(1).max(1);
                    return None;
                }
                if contains((470, 184, 26, 24), p) {
                    self.status_level = (level + 1).min(40);
                    return None;
                }
                if contains((44, 220, 100, 24), p) {
                    return Some(Control::MonsterStatusAdd {
                        target: self.target(),
                        kind,
                        level,
                    });
                }
                if contains((154, 220, 100, 24), p) {
                    return Some(Control::MonsterStatusRemove {
                        target: self.target(),
                        kind,
                    });
                }
                if contains((264, 220, 100, 24), p) {
                    return Some(Control::MonsterStatusClear {
                        target: self.target(),
                    });
                }
                if contains((480, 148, 100, 22), p) {
                    return Some(Control::MonsterDemo {
                        target: self.target(),
                        enabled: !v.demo,
                    });
                }
                if contains((44, 382, 28, 24), p) {
                    self.status_page = self.status_page.saturating_sub(1);
                    return None;
                }
                if contains((206, 382, 28, 24), p) {
                    self.status_page =
                        (self.status_page + 1).min(v.status_rows.len().saturating_sub(1) / 10);
                    return None;
                }
                for (row, &(kind, _, _, _)) in v
                    .status_rows
                    .iter()
                    .skip(self.status_page * 10)
                    .take(10)
                    .enumerate()
                {
                    let x = 44 + (row % 2) as i32 * 284;
                    let y = 282 + (row / 2) as i32 * 20;
                    if contains((x, y, 268, 19), p) {
                        return Some(Control::MonsterStatusRemove {
                            target: self.target(),
                            kind,
                        });
                    }
                }
            }
            return None;
        }
        if keys.just_pressed(Dik::TAB) {
            let fields = self.fields();
            let i = self
                .selected
                .and_then(|f| fields.iter().position(|&n| n == f))
                .map_or(0, |i| (i + 1) % fields.len());
            self.select(fields[i], v);
        }
        if click {
            for (i, value) in [true, false].into_iter().enumerate() {
                if contains((124 + i as i32 * 112, 294, 104, 26), p) {
                    return Some(if self.tab == 0 {
                        Control::PlayerGrabable(value)
                    } else {
                        Control::MonsterGrabable {
                            target: self.target(),
                            value,
                        }
                    });
                }
            }
            for (row, &field) in self.fields().iter().enumerate() {
                let (x, y) = row_pos(row, self.tab == 1);
                if self.tab == 1 && field != 0 && v.count == 0 {
                    continue;
                }
                if contains((x, y, 252, 24), p) {
                    self.select(field, v);
                    if p.0 >= x + 202 {
                        let (lo, hi) = self.bounds(field);
                        let step = if (self.tab == 0 && matches!(field, 13 | 14 | 17))
                            || (self.tab == 1 && field == 4)
                        {
                            10
                        } else {
                            1
                        };
                        self.draft = (self.value(field, v)
                            + if p.0 < x + 226 { -step } else { step })
                        .clamp(lo, hi)
                        .to_string();
                        return self.commit();
                    }
                    return None;
                }
            }
            for (i, state) in [
                Protection::Normal,
                Protection::SuperArmor,
                Protection::Invincible,
            ]
            .iter()
            .enumerate()
            {
                if contains((124 + i as i32 * 112, 324, 104, 26), p) {
                    return Some(if self.tab == 0 {
                        Control::PlayerState(*state)
                    } else {
                        Control::MonsterState {
                            target: self.target(),
                            state: *state,
                        }
                    });
                }
            }
            if contains((44, 382, 122, 26), p) {
                return Some(if self.tab == 0 {
                    Control::HealPlayer
                } else {
                    Control::Respawn(self.target())
                });
            }
            if contains((178, 382, 122, 26), p) {
                self.selected = None;
                return Some(if self.tab == 0 {
                    Control::ResetPlayer
                } else {
                    Control::ResetMonsters
                });
            }
            if contains((456, 382, 124, 26), p) {
                return self.commit();
            }
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
                return self.commit();
            }
        }
        None
    }

    pub fn draw(&self, g: &Game, _data: &GameData, fb: &mut Framebuffer) {
        if !self.open {
            return;
        }
        let v = View::from_game(g);
        let white = rgba(226, 232, 242, 255);
        let muted = rgba(146, 163, 183, 255);
        let cyan = rgba(88, 209, 215, 255);
        fb.fill_rect(0, 0, 640, 480, rgba(5, 10, 18, 135));
        fb.fill_rect(32, 40, 584, 384, rgba(0, 0, 0, 130));
        fb.fill_rect(28, 36, 584, 384, rgba(20, 28, 40, 255));
        fb.rect_outline(28, 36, 584, 384, rgba(61, 86, 109, 255));
        ui::text(fb, 44, 48, "调试控制台", white);
        ui::text(fb, 182, 50, "世界已暂停  ·  F6 / Esc 关闭", muted);
        button(fb, (558, 43, 30, 24), "×", false);
        for (i, label) in ["人物", "怪物", "运行信息"].iter().enumerate() {
            button(fb, (44 + i as i32 * 180, 78, 172, 28), label, self.tab == i);
        }
        if self.tab == 2 {
            for (i, line) in [
                format!(
                    "动作 {}   帧 {}/{}   倍率 {:.0}%",
                    g.actor.motion.label(),
                    g.actor.anim().frame() + 1,
                    g.actor.anim().clip().frame_count(),
                    g.actor.anim().rate_percent()
                ),
                format!(
                    "位置 X{} Y{} Z{:.1}   朝向 {}",
                    g.actor.x,
                    g.actor.y,
                    g.actor.altitude(),
                    g.actor.facing
                ),
                format!(
                    "人物状态 {:?}  怪物 {}  世界300Hz  FPS {:.0}",
                    v.player_state, v.count, g.fps
                ),
                format!(
                    "上挑冷却 {:.1}s   鬼斩冷却 {:.1}s",
                    g.actor.upper_cooldown_ms() / 1000.0,
                    g.actor.ghost_cooldown_ms() / 1000.0
                ),
                format!(
                    "噬魂之手 {:.1}s  抓取目标 {:?}  力量加成 {}",
                    g.actor
                        .skill_cooldown_remaining_ms(31)
                        .unwrap_or(0.0)
                        .max(0.0)
                        / 1000.0,
                    g.grab_skill.target(),
                    g.grab_skill.strength_bonus()
                ),
            ]
            .iter()
            .enumerate()
            {
                ui::text(fb, 44, 120 + i as i32 * 20, line, white);
            }
            let p = g.performance;
            for (i, line) in [
                format!(
                    "帧时间 {:.2}ms  P95 {:.2}ms  P99 {:.2}ms",
                    p.avg.frame, p.p95, p.p99
                ),
                format!(
                    "逻辑 {:.2}ms  渲染 {:.2}ms  转换 {:.2}ms",
                    p.avg.logic, p.avg.render, p.avg.convert
                ),
                format!(
                    "缩放 {:.2}ms  呈现 {:.2}ms  超16.7ms {}/{}帧",
                    p.avg.scale, p.avg.gdi, p.slow_frames, p.samples
                ),
            ]
            .iter()
            .enumerate()
            {
                ui::text(fb, 44, 222 + i as i32 * 24, line, cyan);
            }
            ui::text(fb, 44, 300, "最近输入", cyan);
            for (i, r) in g.actor.commands.records.iter().rev().take(3).enumerate() {
                ui::text(
                    fb,
                    44,
                    324 + i as i32 * 22,
                    &format!("{:?}   按住 {}ms   按下 {}", r.command, r.hold_ms, r.held),
                    muted,
                );
            }
            return;
        }
        if self.tab == 0 {
            for (page, label) in ["基础属性", "动作参数"].iter().enumerate() {
                button(
                    fb,
                    (44 + page as i32 * 120, 110, 112, 22),
                    label,
                    self.player_page == page,
                );
            }
        } else {
            button(fb, (44, 116, 82, 26), "全部怪物", !self.single_monster);
            button(fb, (136, 116, 82, 26), "单只", self.single_monster);
            button(fb, (230, 116, 28, 26), "‹", false);
            ui::text(
                fb,
                268,
                122,
                &if v.count == 0 {
                    "暂无怪物".into()
                } else {
                    format!("怪物 {} / {}", self.selected_monster + 1, v.count)
                },
                white,
            );
            button(fb, (390, 116, 28, 26), "›", false);
            ui::text(fb, 432, 123, &format!("最多 {} 只", MAX_MONSTERS), muted);
            for (page, label) in ["怪物属性", "异常状态"].into_iter().enumerate() {
                button(
                    fb,
                    (44 + page as i32 * 120, 148, 112, 22),
                    label,
                    self.monster_page == page,
                );
            }
            if self.monster_page == 1 {
                self.draw_status(g, fb, &v);
                return;
            }
        }
        for (row, &field) in self.fields().iter().enumerate() {
            let (x, y) = row_pos(row, self.tab == 1);
            let selected = self.selected == Some(field);
            let disabled = self.tab == 1 && field != 0 && v.count == 0;
            ui::text(
                fb,
                x,
                y + 5,
                if self.tab == 0 {
                    ui::LABELS[field]
                } else {
                    MONSTER_LABELS[field]
                },
                if disabled { muted } else { white },
            );
            fb.fill_rect(x + 105, y, 94, 24, rgba(12, 19, 29, 255));
            fb.rect_outline(
                x + 105,
                y,
                94,
                24,
                if selected {
                    cyan
                } else {
                    rgba(49, 66, 85, 255)
                },
            );
            let value = if selected {
                self.draft.clone()
            } else if self.tab == 1 && v.mixed[field] {
                "混合".into()
            } else {
                self.value(field, &v).to_string()
            };
            ui::text(
                fb,
                x + 111,
                y + 5,
                &value,
                if disabled { muted } else { white },
            );
            button(fb, (x + 202, y, 22, 24), "−", false);
            button(fb, (x + 226, y, 22, 24), "+", false);
        }
        ui::text(fb, 44, 330, "附加状态", muted);
        ui::text(fb, 44, 300, "抓取判定", muted);
        let grabable = if self.tab == 0 {
            Some(v.player_grabable)
        } else {
            v.monster_grabable
        };
        for (i, (label, value)) in [("可抓取", true), ("不可抓取", false)]
            .into_iter()
            .enumerate()
        {
            button(
                fb,
                (124 + i as i32 * 112, 294, 104, 26),
                label,
                grabable == Some(value),
            );
        }
        let state = if self.tab == 0 {
            Some(v.player_state)
        } else {
            v.monster_state
        };
        for (i, (label, s)) in [
            ("普通", Protection::Normal),
            ("霸体", Protection::SuperArmor),
            ("无敌", Protection::Invincible),
        ]
        .iter()
        .enumerate()
        {
            button(
                fb,
                (124 + i as i32 * 112, 324, 104, 26),
                label,
                state == Some(*s),
            );
        }
        if self.tab == 1 && !self.single_monster {
            ui::text(
                fb,
                44,
                275,
                "全部模式：将输入值应用到所有怪物；不同数值显示“混合”",
                muted,
            );
        }
        ui::text(
            fb,
            44,
            361,
            if self.message.is_empty() {
                "点击数值输入  ·  Enter 应用  ·  Tab 下一项"
            } else {
                &self.message
            },
            muted,
        );
        button(
            fb,
            (44, 382, 122, 26),
            if self.tab == 0 {
                "补满 HP / MP"
            } else {
                "复活并重置"
            },
            false,
        );
        button(fb, (178, 382, 122, 26), "恢复默认", false);
        button(fb, (456, 382, 124, 26), "应用数值", true);
    }
}
fn button(fb: &mut Framebuffer, r: (i32, i32, i32, i32), label: &str, active: bool) {
    fb.fill_rect(
        r.0,
        r.1,
        r.2,
        r.3,
        if active {
            rgba(30, 85, 99, 255)
        } else {
            rgba(33, 46, 62, 255)
        },
    );
    fb.rect_outline(
        r.0,
        r.1,
        r.2,
        r.3,
        if active {
            rgba(85, 197, 203, 255)
        } else {
            rgba(66, 84, 106, 255)
        },
    );
    let (w, h) = ui::measure_text(label);
    ui::text(
        fb,
        r.0 + (r.2 - w) / 2,
        r.1 + (r.3 - h) / 2,
        label,
        Color::WHITE,
    );
}

pub fn selftest(data: &GameData, out: &std::path::Path) -> Result<(), String> {
    use dnf_core::KeyMap;
    std::fs::create_dir_all(out).map_err(|e| e.to_string())?;
    let mut g = data.build_game(KeyMap::official_default())?;
    g.actor.teleport(320, 329);
    g.camera_x = 0;
    g.apply_control(Control::ResetMonsters);
    let check = |ok: bool, msg: &str| if ok { Ok(()) } else { Err(msg.to_string()) };
    let shot = |g: &Game, name: &str| -> Result<(), String> {
        let mut fb = Framebuffer::default();
        g.render(data, &mut fb);
        dnf_render::png::write_file(&out.join(format!("{name}.png")), &fb)
            .map_err(|e| e.to_string())
    };
    let key = |g: &mut Game, k: Dik| {
        g.begin_input();
        g.set_key(k, true);
        g.step();
        g.begin_input();
        g.set_key(k, false);
        g.step();
    };
    let click = |g: &mut Game, x: i32, y: i32| {
        g.begin_input();
        g.panel.pointer = (x, y, true);
        g.step();
        g.begin_input();
        g.panel.pointer = (x, y, false);
        g.step();
    };
    shot(&g, "01_clean_scene")?;
    let mut before = Framebuffer::default();
    g.render(data, &mut before);
    g.fps = 999.0;
    g.status = "DEBUG SECRET".into();
    g.steps = 88;
    g.show_keys = true;
    let mut after = Framebuffer::default();
    g.render(data, &mut after);
    check(
        before
            .data
            .chunks_exact(4)
            .zip(after.data.chunks_exact(4))
            .enumerate()
            .all(|(i, (a, b))| (i % 640 >= 534 && i / 640 < 24) || a == b),
        "only the top-right FPS indicator may change scene diagnostics",
    )?;
    key(&mut g, Dik::F6);
    let pose = (g.actor.x, g.actor.y, g.actor.anim().frame());
    click(&mut g, 170, 200); // physical attack entry, row4
    key(&mut g, Dik::KEY_1);
    key(&mut g, Dik::KEY_2);
    key(&mut g, Dik::KEY_3);
    key(&mut g, Dik::RETURN);
    check(g.debug_values()[4] == 123, "player value keyboard edit")?;
    key(&mut g, Dik::X);
    key(&mut g, Dik::C);
    check(
        pose == (g.actor.x, g.actor.y, g.actor.anim().frame()) && g.goblin.hits == 0,
        "panel input leaked to combat",
    )?;
    shot(&g, "02_player_panel")?;
    click(&mut g, 268, 120); // action parameters
    shot(&g, "03_action_panel")?;
    click(&mut g, 274, 338); // armor button
    check(
        g.actor.protection.effective() == Protection::SuperArmor,
        "player armor button",
    )?;
    click(&mut g, 248, 90); // monster tab
    click(&mut g, 170, 196); // quantity
    key(&mut g, Dik::KEY_6);
    key(&mut g, Dik::RETURN);
    check(
        g.monster_count() == 6 && g.monsters().count() == 6,
        "real monster count",
    )?;
    g.apply_control(Control::MonsterSet {
        target: None,
        field: 1,
        value: 600,
    });
    g.apply_control(Control::MonsterSet {
        target: None,
        field: 2,
        value: 600,
    });
    check(
        g.monsters().all(|m| m.max_hp == 600 && m.hp == 600),
        "all-monster HP",
    )?;
    g.apply_control(Control::MonsterSet {
        target: Some(2),
        field: 3,
        value: 1000,
    });
    check(
        g.monster(2).unwrap().reduced_damage(100) == 50 && g.monster(1).unwrap().defense == 0,
        "single-monster defense",
    )?;
    g.apply_control(Control::MonsterState {
        target: Some(2),
        state: Protection::Invincible,
    });
    check(
        g.monster_protection(2) == Some(Protection::Invincible)
            && g.monster_protection(1) == Some(Protection::Normal),
        "single-monster state",
    )?;
    g.debug_panel.single_monster = true;
    g.debug_panel.selected_monster = 2;
    shot(&g, "04_monster_panel")?;
    key(&mut g, Dik::ESCAPE);
    check(
        !g.exit_requested && !g.debug_panel.open,
        "Esc should only close control panel",
    )?;
    shot(&g, "05_six_monsters")?;
    g.set_monster_count(0);
    check(g.monsters().count() == 0, "zero monsters")?;
    shot(&g, "06_zero_monsters")?;
    g.set_monster_count(MAX_MONSTERS + 10);
    check(g.monster_count() == MAX_MONSTERS, "count upper bound")?;
    g.set_monster_count(2);
    check(g.monsters().count() == 2, "despawn excess monsters")?;
    key(&mut g, Dik::F7);
    shot(&g, "07_runtime_panel")?;
    std::fs::write(out.join("debug_report.txt"),
        "PASS: F6 controls / captured text input / pause / player attack edit / player armor / real count0..32 / all-monster HP / per-monster defense and invulnerability / despawn / Esc / no world diagnostic text\nMonster defense is training damage *1000/(1000+defense); weight and hit recovery feed reaction physics. Full official stat derivation and attacking monster AI remain outside this panel.\n")
        .map_err(|e|e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn view() -> View {
        View {
            status_rows: Vec::new(),
            status_mask: 0,
            demo: false,
            player: [100; ui::COUNT],
            monster: [1, 300, 300, 0, 68000, 500],
            count: 1,
            player_state: Protection::Normal,
            monster_state: Some(Protection::Normal),
            player_grabable: true,
            monster_grabable: Some(true),
            mixed: [false; 6],
        }
    }
    #[test]
    fn invalid_quantity_and_close_click_do_not_commit_or_leak() {
        let mut p = DebugPanel {
            open: true,
            tab: 1,
            ..Default::default()
        };
        p.select(0, &view());
        p.draft = "33".into();
        assert!(p.commit().is_none());
        assert!(p.message.contains("32"));
        p.input(&InputState::new(), (560, 48, true), &view());
        assert!(!p.open);
    }
    #[test]
    fn a_held_mouse_click_changes_a_value_only_once_and_tracks_scope() {
        let mut p = DebugPanel {
            open: true,
            tab: 1,
            single_monster: true,
            selected_monster: 0,
            ..Default::default()
        };
        let key = InputState::new();
        let first = p.input(&key, (278, 191, true), &view());
        assert!(matches!(
            first,
            Some(Control::MonsterSet {
                target: Some(0),
                field: 0,
                value: 2
            })
        ));
        assert!(p.input(&key, (278, 191, true), &view()).is_none());
    }
}
