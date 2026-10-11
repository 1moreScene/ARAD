//! 游戏世界：官方资源 → 房间 + 角色 → 逻辑与渲染。

use crate::assets::ClientAssets;
use crate::battle::{Box3, Goblin, GoblinAssets, Hit};
use crate::character::{self, CharacterVisual, Outfit};
use crate::room::{blit_frame, Room};
use dnf_core::{
    actor::{Actor, ActorEvent, ActorSpec, Motion},
    input::InputState,
    jump::JumpTuning,
    keymap::KeyMap,
    keys::Dik,
    movement::MoveTuning,
};
use dnf_render::font;
use dnf_render::framebuffer::{rgba, Framebuffer};
use std::path::Path;

/// 默认房间（洛兰森林 104：5 条瓦片带、视差 56/90/110、门 16..1104）。
pub const DEFAULT_MAP: &str = "map/lorien/104.map";

/// 官方角色资源路径。
pub const SWORDMAN_CHR: &str = "character/swordman/swordman.chr";

pub struct GameData {
    pub assets: ClientAssets,
    pub spec: ActorSpec,
    pub visual: CharacterVisual,
    /// Official attack animation and damage definitions (damage resolution pending).
    pub combat: Option<dnf_data::combat::PreparedCombat>,
    pub weapon: dnf_data::avatar::AvatarItem,
    pub weapon_feedback: dnf_data::combat::HitInfo,
    pub audio: Option<dnf_data::audio::PreparedAudio>,
    pub goblin: std::rc::Rc<GoblinAssets>,
    pub ui: crate::ui::UiAssets,
    pub debug_defaults: [i32; crate::ui::COUNT],
    pub resistances: [i32; 4],
    pub command_templates: Vec<(i32, String, dnf_core::key_manager::Definition)>,
    pub skills: Vec<dnf_data::skill::SkillDefinition>,
    pub skill_strings: dnf_data::strings::StringTable,
    pub system_messages: std::rc::Rc<dnf_data::messages::SystemMessages>,
    pub hud_assets: crate::hud::HudAssets,
    pub learned_skills: Vec<(i32, i32)>,
    pub ghost_effects: Vec<std::rc::Rc<dnf_data::AnimFile>>,
    pub outfit: Outfit,
    /// 官方时装部件表（层值 + 部件目录）。
    pub avatar: dnf_data::avatar::AvatarTable,
    pub map_path: String,
    pub report: Vec<String>,
}

impl GameData {
    pub fn load(client_dir: &Path, map_path: &str, outfit: Outfit) -> Result<Self, String> {
        let assets = ClientAssets::load(client_dir)?;
        let goblin = std::rc::Rc::new(GoblinAssets::load(&assets)?);
        let mut report = assets.report.clone();
        let audio = match dnf_data::audio::PreparedAudio::load(client_dir) {
            Ok(audio) => {
                report.push(format!(
                    "音效索引: {} EFFECT / {} RANDOM / {}音效资源（窗口模式异步播放）",
                    audio.catalog.effects.len(),
                    audio.catalog.groups.len(),
                    audio.packs.len()
                ));
                Some(audio)
            }
            Err(error) => {
                report.push(format!("音效准备失败: {error}"));
                None
            }
        };

        // ---- 官方角色数值 + 官方动作表 ----
        let mut spec = ActorSpec::default();
        let mut chr = dnf_data::chr::ChrFile::default();
        match assets.pvf.read_text(SWORDMAN_CHR) {
            Ok(text) => {
                let c = dnf_data::chr::parse(&text);
                spec = ActorSpec {
                    move_speed: c.move_speed() as f32,
                    attack_speed: c.attack_speed() as f32,
                    jump_power: c.jump_power() as f32,
                    jump_speed: c.jump_speed() as f32,
                    weight: c.weight() as f32,
                    width: c.width,
                };
                report.push(format!(
                    "角色: {}  move speed={} attack speed={} jump power={} jump speed={} weight={} width={:?}",
                    c.job,
                    c.move_speed(),
                    c.attack_speed(),
                    c.jump_power(),
                    c.jump_speed(),
                    c.weight(),
                    c.width
                ));
                report.push(format!(
                    "官方动作表（.chr，序号 = 官方动作 ID）: {}",
                    c.motions_ordered
                        .iter()
                        .enumerate()
                        .map(|(i, (t, p))| format!("{}={}({})", i, t, p))
                        .collect::<Vec<_>>()
                        .join(" ")
                ));
                chr = c;
            }
            Err(e) => report.push(format!("读 .chr 失败：{}", e)),
        }

        // ---- 官方动画帧序（动作路径取自 .chr 动作表）----
        let visual = character::build_visual(&assets, &chr);
        let weapon_text = assets
            .pvf
            .read_text(character::DEFAULT_WEAPON)
            .map_err(|e| format!("默认武器: {e}"))?;
        let mut weapon = dnf_data::avatar::parse(&weapon_text);
        let weapon_type = dnf_data::combat::parse_attack_info(&weapon_text)
            .number("sub type")
            .unwrap_or(0)
            .max(0) as usize;
        let weapon_feedback = dnf_data::combat::weapon_hit_info(
            &assets
                .pvf
                .read_text(SWORDMAN_CHR)
                .map_err(|e| e.to_string())?,
            weapon_type,
        )
        .ok_or("missing scripted weapon hit info")?;
        report.push(format!("受击取证参数: 武器hit info={:?}；哥布林hit recovery={} weight={}，{}个脚本动作，装备={}",weapon_feedback,
            goblin.hit_recovery,goblin.weight,goblin.definition.motions.len(),goblin.layers.iter().map(|l|l.path.as_str()).collect::<Vec<_>>().join(",")));
        weapon.path = character::DEFAULT_WEAPON.to_string();
        report.push(format!(
            "默认武器: {} (官方装备27000)，variation={:?}，{}个官方LAYER部件",
            weapon.name2,
            weapon.variation,
            weapon.parts.len()
        ));
        let combat = match dnf_data::combat::PreparedCombat::load(
            &assets.pvf,
            &chr,
            character::CHARACTER_DIR,
        ) {
            Ok(combat) => {
                report.push(format!(
                    "攻击资源已准备: 普通攻击{}段 + 空中攻击；身体动画、逐帧标签、.atk均载入",
                    combat.normal.len()
                ));
                Some(combat)
            }
            Err(error) => {
                report.push(format!("攻击资源准备失败: {error}"));
                None
            }
        };
        report.push(format!(
            "动画: 载入 {} 个动作（{}）",
            visual.motions.len(),
            visual
                .motions
                .iter()
                .map(|(n, f)| {
                    let id = character::MOTION_TAGS
                        .iter()
                        .find(|(m, _)| m == n)
                        .and_then(|(_, tag)| character::motion_index(&chr, tag));
                    match id {
                        Some(i) => format!("{}:{}帧(官方ID {})", n, f.len(), i),
                        None => format!("{}:{}帧", n, f.len()),
                    }
                })
                .collect::<Vec<_>>()
                .join(" ")
        ));

        report.push(format!(
            "时装: {}",
            outfit
                .slots
                .iter()
                .map(|s| format!("{}={}{}", s.slot.key(), s.variation.0, s.variation.1))
                .collect::<Vec<_>>()
                .join(" ")
        ));

        // ---- 官方时装部件表（层值 + 部件目录）----
        let avatar = dnf_data::avatar::AvatarTable::load(&assets.pvf, "swordman");
        let mut layers: Vec<String> = avatar
            .dir_layer_map()
            .iter()
            .map(|(d, l)| format!("{}={}", d, l))
            .collect();
        layers.sort();
        report.push(format!(
            "时装部件表: {} 件装备 / 动作目录 {} 个（官方 [LAYER]：{}）",
            avatar.items.len(),
            avatar.dir_layer_map().len(),
            layers.join(" ")
        ));
        report.push(format!("房间: {}", map_path));
        let skill_list = assets
            .pvf
            .read_text("skill/swordmanskill.lst")
            .map_err(|e| e.to_string())?;
        let mut command_templates = Vec::new();
        let mut skills = Vec::new();
        for line in skill_list.lines() {
            let mut words = line.split("//").next().unwrap_or("").split_whitespace();
            let Some(id) = words.next().and_then(|s| s.parse::<i32>().ok()) else {
                continue;
            };
            let Some(relative) = words.next() else {
                continue;
            };
            let path = format!("skill/{}", relative.trim_matches('`'));
            let text = assets
                .pvf
                .read_text(&path)
                .map_err(|e| format!("skill commands {path}: {e}"))?;
            skills.push(dnf_data::skill::SkillDefinition::parse(id, &path, &text)?);
            if let Some(template) =
                dnf_core::key_manager::Definition::parse_skill(1000 + id as u32, &text)?
            {
                if !template.stages.is_empty() {
                    command_templates.push((id, path, template));
                }
            }
        }
        report.push(format!("按键管理器: CNCommandChecker统一9类历史/四触发模式/优先级；{}个非空鬼剑士技能命令模板；UpperSlash.skl无command，基础Z入口保持角色专属映射",command_templates.len()));
        let learned_skills = chr.skills.clone();
        let skill_strings = dnf_data::strings::StringTable::load(&assets.pvf, 12)?;
        let system_messages = std::rc::Rc::new(dnf_data::messages::SystemMessages::load(
            &client_dir.join("ARAD.exe"),
        )?);
        let hud_assets = crate::hud::HudAssets::load(&assets)?;
        let ghost_effects = ["HardAttack1", "HardAttack2"]
            .into_iter()
            .map(|name| {
                assets
                    .ani(&format!("character/swordman/effect/animation/{name}.ani"))
                    .ok_or_else(|| format!("missing {name}"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        for id in [5, 46] {
            let s = skills
                .iter()
                .find(|s| s.id == id)
                .ok_or("missing base skill")?;
            let level = learned_skills
                .iter()
                .find(|(i, _)| *i == id)
                .map(|(_, lv)| *lv as usize)
                .unwrap_or(0);
            let runtime = s.at_level(level).ok_or("missing learned skill level")?;
            report.push(format!(
                "技能{} {} lv{}: MP{} CD{}ms casting{}ms values{:?}；默认cancelSkill开启",
                id,
                if id == 5 { "鬼斩(A)" } else { "上挑(Z)" },
                level,
                runtime.mp,
                runtime.cooldown_ms,
                runtime.casting_ms,
                runtime.values
            ));
        }

        let ui = crate::ui::UiAssets::load(&assets, character::DEFAULT_WEAPON)?;
        let base = |name: &str| chr.attr(name).unwrap_or(0.0) as i32;
        let equ = dnf_data::combat::parse_attack_info(&weapon_text);
        let physical = base("physical attack");
        let magical = base("magical attack");
        let defense = base("physical defense");
        let magic_defense = base("magical defense");
        let debug_defaults = [
            base("hp max"),
            base("hp max"),
            base("mp max"),
            base("mp max"),
            physical + equ.number("equipment physical attack").unwrap_or(0),
            defense,
            magical,
            magic_defense,
            physical,
            magical,
            defense,
            magic_defense,
            1,
            spec.move_speed as i32,
            spec.attack_speed as i32,
            spec.jump_power as i32,
            spec.jump_speed as i32,
            spec.weight as i32,
            300,
            0,
        ];
        let resistances = [
            base("fire resistance"),
            base("ice resistance"),
            base("light resistance"),
            base("dark resistance"),
        ];
        report.push(format!("UI: StatusWindow[0] 291x380, StatusIllust[0], ItemInventoryWindow[0]；伤害Combo.img[44..83,143..152]；武器icon[{}]",ui.icon_index));
        Ok(GameData {
            assets,
            spec,
            visual,
            combat,
            weapon,
            weapon_feedback,
            audio,
            goblin,
            ui,
            debug_defaults,
            resistances,
            command_templates,
            skills,
            skill_strings,
            system_messages,
            hud_assets,
            learned_skills,
            ghost_effects,
            outfit,
            avatar,
            map_path: map_path.to_string(),
            report,
        })
    }

    /// 生成游戏（装载房间）。
    pub fn build_game(&self, keymap: KeyMap) -> Result<Game, String> {
        let room = Room::load(&self.assets, &self.map_path)?;
        // 出生点 = 最左侧的门（官方 `[pathgate pos]`），y 取该处可通行纵深的中间。
        // 官方副本就是从左门进房，这样也避免一进场就站在 [close] 前景物件背后。
        let spawn_x = room
            .map
            .pathgates
            .iter()
            .map(|g| g.0)
            .filter(|x| *x >= room.walk_x.0 && *x <= room.walk_x.1)
            .min()
            .unwrap_or((room.walk_x.0 + room.walk_x.1) / 2);
        let (y0, y1) = room.walk_band(spawn_x);
        let spawn_y = (y0 + y1) / 2;
        let mut actor = Actor::new(self.spec, self.visual.clips.clone(), spawn_x, spawn_y);
        actor.facing = 1;
        actor.mp = self.debug_defaults[2];
        for (id, dst) in [(5, &mut actor.ghost_skill), (46, &mut actor.upper_skill)] {
            let definition = self
                .skills
                .iter()
                .find(|s| s.id == id)
                .ok_or("missing active skill")?;
            let lv = self
                .learned_skills
                .iter()
                .find(|(i, _)| *i == id)
                .map(|(_, lv)| *lv as usize)
                .unwrap_or(0);
            let s = definition
                .at_level(lv)
                .ok_or("invalid learned skill level")?;
            *dst = dnf_core::actor::ActiveSkill {
                level: lv,
                mp: s.mp,
                hp: 0,
                items: s.items,
                cooldown_ms: s.cooldown_ms as f64,
                values: s.values,
                cancel_skill: true,
            };
        }
        for (_, _, template) in &self.command_templates {
            let mut definition = template.clone();
            definition.enabled = false; // Register script commands without enabling unimplemented skills.
            actor.commands.register(definition);
        }
        let camera_x = clamp_cam(&room, spawn_x);
        let goblin = Goblin::new(
            (spawn_x + 110).min(room.walk_x.1 - 20),
            spawn_y,
            &self.goblin,
        );
        actor.hp = self.debug_defaults[0];
        let mut hud = crate::hud::Hud::default();
        hud.update_gauges(
            actor.hp,
            self.debug_defaults[1],
            actor.mp,
            self.debug_defaults[3],
            0.0,
        );
        Ok(Game {
            actor,
            room,
            camera_x,
            keymap,
            mt: MoveTuning::default(),
            jt: JumpTuning::default(),
            input: InputState::new(),
            steps: 0,
            last_event: ActorEvent::None,
            status: String::new(),
            sound_requests: Vec::new(),
            swing_cues: self.swing_cues(),
            pending_swing: None,
            loop_sound: None,
            goblin,
            goblin_assets: self.goblin.clone(),
            attack_frames: self.attack_frames(),
            attack_infos: self.attack_infos(),
            hit_this_action: false,
            actor_stop: dnf_core::feedback::HitStop::default(),
            weapon_feedback: self.weapon_feedback.clone(),
            panel: crate::ui::Panel::default(),
            debug: self.debug_defaults,
            debug_defaults: self.debug_defaults,
            defaults: self.spec,
            popups: Vec::new(),
            exit_requested: false,
            show_keys: false,
            afterimages: Vec::new(),
            ghost_fx: Vec::new(),
            ghost_effects: self.ghost_effects.clone(),
            hud,
            notices: crate::notice::Notices::default(),
            system_messages: self.system_messages.clone(),
            fps: 0.0,
        })
    }

    /// The weapon's first offensive frame, rather than the body's hurt boxes.
    /// Until official sound-frame gates are measured, this is an explicit sync policy.
    fn swing_cues(&self) -> std::collections::HashMap<&'static str, (usize, &'static str)> {
        let mut cues = std::collections::HashMap::new();
        if let Some((directory, _)) = self.weapon.path.rsplit_once('/') {
            for motion in [
                "Attack1",
                "Attack2",
                "Attack3",
                "JumpAttack",
                "DashAttack",
                "HardAttack",
            ] {
                let frame = self
                    .weapon
                    .parts
                    .iter()
                    .filter_map(|part| {
                        self.assets
                            .ani(&format!("{directory}/{}/{motion}.ani", part.dir))
                            .and_then(|ani| {
                                ani.frames.iter().position(|f| !f.attack_boxes.is_empty())
                            })
                    })
                    .min();
                if let Some(frame) = frame {
                    let tag = if motion == "Attack3" {
                        "MINERALSWDB"
                    } else {
                        "R_MINERALSWDA"
                    };
                    cues.insert(motion, (frame, tag));
                }
            }
        }
        cues
    }
    fn attack_frames(&self) -> std::collections::HashMap<&'static str, Vec<Vec<[i32; 6]>>> {
        let mut out = std::collections::HashMap::new();
        if let Some((directory, _)) = self.weapon.path.rsplit_once('/') {
            for motion in [
                "Attack1",
                "Attack2",
                "Attack3",
                "JumpAttack",
                "DashAttack",
                "HardAttack",
            ] {
                let mut frames = Vec::<Vec<[i32; 6]>>::new();
                for part in &self.weapon.parts {
                    if let Some(ani) = self
                        .assets
                        .ani(&format!("{directory}/{}/{motion}.ani", part.dir))
                    {
                        frames.resize_with(frames.len().max(ani.frames.len()), Vec::new);
                        for (dst, frame) in frames.iter_mut().zip(&ani.frames) {
                            for bounds in &frame.attack_boxes {
                                if !dst.contains(bounds) {
                                    dst.push(*bounds);
                                }
                            }
                        }
                    }
                }
                out.insert(motion, frames);
            }
        }
        out
    }
    fn attack_infos(
        &self,
    ) -> std::collections::HashMap<&'static str, dnf_data::combat::AttackInfo> {
        let mut out = std::collections::HashMap::new();
        if let Some(combat) = &self.combat {
            for (name, attack) in ["Attack1", "Attack2", "Attack3"]
                .into_iter()
                .zip(&combat.normal)
            {
                out.insert(name, attack.info.clone());
            }
            out.insert("JumpAttack", combat.jump_attack.info.clone());
            out.insert("DashAttack", combat.dash_attack.info.clone());
            out.insert("UpperSlash", combat.upper_slash.clone());
            out.insert("HardAttack", combat.hard_attack.info.clone());
        }
        out
    }
}

/// 一层的屏幕左上角。
///
/// 官方镜像靠**动画速率取负**（`IRDActiveObject_vmt37C`），效果 = 以角色锚点（脚点）为轴
/// 把画布水平翻转。位图按像素镜像时锚点所在的列是自身的镜像，所以：
///
/// * 未翻转：左边缘 = `角色屏幕 x + 偏移`，占据列 `[a, a+w-1]`（`a = sx + offset`）
/// * 翻转：占据列 `[2·sx − (a+w−1), 2·sx − a]` ⇒ 左边缘 = `sx − (偏移 + 宽) + 1`
///
/// **只把精灵原地翻面而不改 x 会让非对称部件错位**（头发/护肩/护臂跑到另一侧）；
/// 少加这个 `+1` 会让整层差 1 像素 —— 左朝向曾经的毛病就是这两件事。
pub fn layer_screen_pos(
    actor_x: i32,
    camera_x: i32,
    offset: (i32, i32),
    sprite_w: i32,
    flip: bool,
) -> (i32, i32) {
    let sx = actor_x - camera_x;
    let x = if flip {
        sx - (offset.0 + sprite_w) + 1
    } else {
        sx + offset.0
    };
    (x, offset.1)
}

/// 相机水平位置：始终让 640 宽视口落在房间内（房间比视口窄时才允许留黑边）。
fn clamp_cam(room: &Room, target_x: i32) -> i32 {
    let view_width = dnf_render::framebuffer::WIDTH as i32;
    let desired = target_x - view_width / 2;
    let hi = (room.room_width() - view_width).max(0);
    desired.clamp(0, hi)
}

pub struct Afterimage {
    pub pose: dnf_core::actor::CancelPose,
    pub age_ms: f64,
}
/// Local fade parameter until the official cancel snapshot lifetime is measured.
const CANCEL_AFTERIMAGE_MS: f64 = 600.0;
const CANCEL_AFTERIMAGE_HOLD_MS: f64 = 120.0;
pub struct Game {
    pub actor: Actor,
    pub room: Room,
    pub camera_x: i32,
    pub keymap: KeyMap,
    pub mt: MoveTuning,
    pub jt: JumpTuning,
    pub input: InputState,
    pub steps: u64,
    pub last_event: ActorEvent,
    pub status: String,
    /// One-shot semantic requests consumed once after each logic tick.
    pub sound_requests: Vec<&'static str>,
    swing_cues: std::collections::HashMap<&'static str, (usize, &'static str)>,
    pending_swing: Option<(usize, &'static str)>,
    pub loop_sound: Option<&'static str>,
    pub goblin: Goblin,
    goblin_assets: std::rc::Rc<GoblinAssets>,
    attack_frames: std::collections::HashMap<&'static str, Vec<Vec<[i32; 6]>>>,
    attack_infos: std::collections::HashMap<&'static str, dnf_data::combat::AttackInfo>,
    hit_this_action: bool,
    actor_stop: dnf_core::feedback::HitStop,
    weapon_feedback: dnf_data::combat::HitInfo,
    pub panel: crate::ui::Panel,
    debug: [i32; crate::ui::COUNT],
    debug_defaults: [i32; crate::ui::COUNT],
    defaults: ActorSpec,
    pub popups: Vec<crate::damage::DamagePopup>,
    pub exit_requested: bool,
    pub show_keys: bool,
    pub afterimages: Vec<Afterimage>,
    ghost_fx: Vec<dnf_core::anim::AnimPlayer>,
    ghost_effects: Vec<std::rc::Rc<dnf_data::AnimFile>>,
    pub hud: crate::hud::Hud,
    pub notices: crate::notice::Notices,
    system_messages: std::rc::Rc<dnf_data::messages::SystemMessages>,
    pub fps: f64,
}

impl Game {
    pub fn begin_input(&mut self) {
        self.input.begin_frame();
    }

    pub fn set_key(&mut self, dik: Dik, down: bool) {
        self.input.set(dik, down);
    }

    pub fn clear_input(&mut self) {
        self.input = InputState::new();
        self.actor.clear_direction_input();
        self.panel.pointer = (-1, -1, false);
        self.hud.cancel_drag();
    }

    pub fn debug_values(&self) -> [i32; crate::ui::COUNT] {
        let mut values = self.debug;
        values[0] = self.actor.hp;
        values[2] = self.actor.mp;
        values[13] = self.actor.spec.move_speed as i32;
        values[14] = self.actor.spec.attack_speed as i32;
        values[15] = self.actor.spec.jump_power as i32;
        values[16] = self.actor.spec.jump_speed as i32;
        values[17] = self.actor.spec.weight as i32;
        values[18] = self.goblin.max_hp;
        values
    }
    pub fn apply_debug(&mut self, command: crate::ui::Command) {
        match command {
            crate::ui::Command::Set(field, value) => {
                if field >= crate::ui::COUNT {
                    return;
                }
                let (lo, hi) = crate::ui::bounds(field);
                if !(lo..=hi).contains(&value) {
                    return;
                }
                self.debug[field] = value;
                match field {
                    0 => self.debug[0] = value.min(self.debug[1]),
                    1 => self.debug[0] = self.debug[0].min(value),
                    2 => self.actor.mp = value.min(self.debug[3]),
                    3 => self.actor.mp = self.actor.mp.min(value),
                    13 => self.actor.spec.move_speed = value as f32,
                    14 => self.actor.spec.attack_speed = value as f32,
                    15 => self.actor.spec.jump_power = value as f32,
                    16 => self.actor.spec.jump_speed = value as f32,
                    17 => self.actor.spec.weight = value as f32,
                    18 => {
                        self.goblin.max_hp = value;
                        self.goblin.reset(
                            self.goblin.x as i32,
                            self.goblin.y as i32,
                            &self.goblin_assets,
                        );
                    }
                    _ => {}
                }
            }
            crate::ui::Command::Reset => {
                self.actor.spec = self.defaults;
                self.debug = self.debug_defaults;
                self.actor.mp = self.debug[2];
                self.goblin.max_hp = self.debug[18];
                self.goblin.reset(
                    (self.actor.x + 110).min(self.room.walk_x.1 - 20),
                    self.actor.y,
                    &self.goblin_assets,
                );
                self.popups.clear();
            }
        }
        self.actor.hp = self.debug[0];
        let values = self.debug_values();
        self.panel.acknowledge(&values);
    }

    /// 推进一个逻辑步（官方 60Hz 定步）。
    pub fn step(&mut self) {
        self.sound_requests.clear();
        self.actor.skill_failures.clear();
        self.notices.advance(self.jt.logic_ms);
        let v = self.debug_values();
        self.hud
            .update_gauges(v[0], v[1], v[2], v[3], self.jt.logic_ms);
        if self.input.just_pressed(Dik::F7) {
            self.show_keys = !self.show_keys;
        }
        let values = self.debug_values();
        let hud_input = self.hud.input(&self.input, self.panel.pointer);
        if hud_input.changed {
            self.sound_requests.push("ICON_QUICK_SET");
        }
        let pointer = self.panel.pointer;
        if hud_input.captures_pointer {
            self.panel.pointer = (-1, -1, false);
        }
        let (consumed, command) = if hud_input.escape_handled {
            (self.panel.open, None)
        } else {
            self.panel.input(&self.input, &values)
        };
        self.panel.pointer = pointer;
        if let Some(command) = command {
            self.apply_debug(command);
            let v = self.debug_values();
            self.hud.update_gauges(v[0], v[1], v[2], v[3], 0.0);
        }
        if consumed {
            self.loop_sound = None;
            self.actor.clear_direction_input();
            self.steps += 1;
            self.last_event = ActorEvent::None;
            return;
        }
        if self.input.just_pressed(Dik::ESCAPE) && !hud_input.escape_handled {
            self.exit_requested = true;
        }
        if self.input.just_pressed(Dik::F5) {
            self.goblin.reset(
                (self.actor.x + 110).min(self.room.walk_x.1 - 20),
                self.actor.y,
                &self.goblin_assets,
            );
            self.actor.mp = self.debug[3];
            self.popups.clear();
            self.hit_this_action = true;
            self.actor_stop = dnf_core::feedback::HitStop::default();
        }
        self.actor.skill_slots = self.hud.slots;
        self.actor
            .update_commands(&self.input, &self.keymap, self.jt.logic_ms);
        for image in &mut self.afterimages {
            image.age_ms += self.jt.logic_ms;
        }
        self.afterimages
            .retain(|image| image.age_ms < CANCEL_AFTERIMAGE_MS);
        self.actor.advance_hit_stop(self.jt.logic_ms);
        let skill_event = self.actor.try_skill();
        let active_ms = self.actor_stop.active_ms(self.jt.logic_ms);
        self.last_event = if active_ms > 0.0 {
            let jt = JumpTuning {
                logic_ms: active_ms,
                ..self.jt
            };
            self.actor.step_at_current_time(&self.mt, &jt)
        } else {
            ActorEvent::None
        };
        if let Some(event) = skill_event {
            self.last_event = event;
        }
        if let Some(pose) = self.actor.cancelled_pose.take() {
            self.afterimages.push(Afterimage { pose, age_ms: 0.0 });
        }
        if self.last_event == ActorEvent::GhostSlashed {
            self.ghost_fx = self
                .ghost_effects
                .iter()
                .map(|a| {
                    dnf_core::anim::AnimPlayer::new(
                        dnf_core::anim::Clip::new("HardAttackFX", a.delays(), false),
                        self.actor.anim().rate_percent(),
                    )
                })
                .collect();
        }
        if self.actor.motion == Motion::GhostSlash {
            for fx in &mut self.ghost_fx {
                fx.advance(active_ms);
            }
        } else {
            self.ghost_fx.clear();
        }
        self.steps += 1;
        match self.last_event {
            ActorEvent::Jumped => self.sound_requests.extend(["JUMP_START", "SM_JUMP"]),
            ActorEvent::Landed => self.sound_requests.push("JUMP_END"),
            ActorEvent::Attacked(_) => self.sound_requests.push("R_SM_ATK"),
            ActorEvent::AirAttacked => self.sound_requests.push("R_SM_JUMP_ATK"),
            ActorEvent::DashAttacked => self.sound_requests.push("SM_THRUST"),
            ActorEvent::UpperSlashed => self.sound_requests.push("SM_UPPER_SLASH"),
            ActorEvent::GhostSlashed => self.sound_requests.push("R_SM_GUE"),
            ActorEvent::NoMana => self.sound_requests.push("SM_NOMANA"),
            ActorEvent::SkillRejected(_) => {}
            ActorEvent::None => {}
        }
        for &failure in &self.actor.skill_failures {
            self.notices.skill_failure(failure, &self.system_messages);
            match failure {
                dnf_core::actor::SkillFailure::Cooldown => self.sound_requests.push("ALERT_1"),
                dnf_core::actor::SkillFailure::Mp if self.last_event == ActorEvent::NoMana => {}
                _ => self.sound_requests.push("SM_NOMANA"),
            }
        }
        self.debug[0] = self.actor.hp;
        let v = self.debug_values();
        self.hud.update_gauges(v[0], v[1], v[2], v[3], 0.0);
        if matches!(
            self.last_event,
            ActorEvent::Attacked(_)
                | ActorEvent::AirAttacked
                | ActorEvent::DashAttacked
                | ActorEvent::UpperSlashed
                | ActorEvent::GhostSlashed
        ) {
            self.pending_swing = self.swing_cues.get(self.motion_name()).copied();
            self.hit_this_action = false;
        }
        if !matches!(
            self.actor.motion,
            Motion::Attack
                | Motion::JumpAttack
                | Motion::DashAttack
                | Motion::UpperSlash
                | Motion::GhostSlash
        ) {
            self.pending_swing = None;
        }
        if let Some((frame, tag)) = self.pending_swing {
            if self.actor.anim().frame() >= frame {
                self.sound_requests.push(tag);
                self.pending_swing = None;
            }
        }

        // 官方地形：可走范围由通行网格描述，左右边界 = 门坐标
        let half_w = self.actor.spec.width.0.max(8) / 2;
        self.actor.x = self
            .actor
            .x
            .clamp(self.room.walk_x.0 + half_w, self.room.walk_x.1 - half_w);
        let (y0, y1) = self.room.walk_band(self.actor.x);
        self.actor.y = self.actor.y.clamp(y0 + 4, y1);
        self.loop_sound = match self.actor.motion {
            Motion::Walk => Some("WALK_SOIL_LOOP"),
            Motion::Run => Some("RUN_SOIL_LOOP"),
            _ => None,
        };
        self.step_battle(active_ms > 0.0);

        // 相机水平跟随（单帧最多 32px，夹取到房间边界）
        let desired = clamp_cam(&self.room, self.actor.x);
        let delta = (desired - self.camera_x).clamp(-32, 32);
        self.camera_x += delta;

        self.status = match self.last_event {
            ActorEvent::Jumped => "JUMP!".into(),
            ActorEvent::Landed => "LAND".into(),
            ActorEvent::None => String::new(),
            ActorEvent::Attacked(stage) => format!("ATTACK {}", stage + 1),
            ActorEvent::AirAttacked => "AIR ATTACK".into(),
            ActorEvent::DashAttacked => "DASH ATTACK".into(),
            ActorEvent::UpperSlashed => "UPPER SLASH".into(),
            ActorEvent::GhostSlashed => "鬼斩".into(),
            ActorEvent::NoMana => "NO MP".into(),
            ActorEvent::SkillRejected(failure) => format!("SKILL REJECTED {failure:?}"),
        };
    }

    fn step_battle(&mut self, actor_active: bool) {
        for popup in &mut self.popups {
            popup.age_ms += self.jt.logic_ms;
        }
        self.popups.retain(|p| p.age_ms < 500.0);
        self.goblin
            .step(&self.goblin_assets, self.room.walk_x, self.jt.logic_ms);
        let attacking = matches!(
            self.actor.motion,
            Motion::Attack
                | Motion::JumpAttack
                | Motion::DashAttack
                | Motion::UpperSlash
                | Motion::GhostSlash
        );
        if !actor_active
            || !attacking
            || self.actor.anim().finished()
            || self.hit_this_action
            || self.goblin.hp <= 0
        {
            return;
        }
        let name = self.motion_name();
        let Some(frames) = self.attack_frames.get(name) else {
            return;
        };
        let Some(bounds) = frames.get(self.actor.anim().frame()) else {
            return;
        };
        let origin = [
            self.actor.x as f32,
            self.actor.y as f32,
            self.actor.altitude(),
        ];
        let hurt = self.goblin.hurt_boxes(&self.goblin_assets);
        let contact = bounds.iter().find_map(|&b| {
            let attack = Box3::translated(b, origin, self.actor.facing);
            hurt.iter()
                .find_map(|&h| attack.intersection(h))
                .map(Box3::center)
        });
        let Some(mut point) = contact else {
            return;
        };
        let popup_point = point;
        point[1] = self.goblin.y;
        let upper = self.actor.motion == Motion::UpperSlash;
        let Some(info) = self
            .attack_infos
            .get(if upper { "UpperSlash" } else { name })
        else {
            return;
        };
        // Test-only base physical damage; both skill columns add flat damage (+40/+10 at lv1).
        let bonus = info.number("damage bonus").unwrap_or(0);
        let flat = if self.actor.motion == Motion::GhostSlash {
            self.actor.ghost_skill.values.first().copied().unwrap_or(0)
        } else if upper {
            self.actor.upper_skill.values.first().copied().unwrap_or(0)
        } else {
            0
        };
        let damage = (self.debug[4] * (100 + bonus) / 100 + flat).max(1);
        let feedback = info.feedback(&self.weapon_feedback);
        if self.goblin.hit(
            &self.goblin_assets,
            info,
            &feedback,
            Hit {
                damage,
                facing: self.actor.facing,
                upper,
                point,
            },
        ) {
            self.hit_this_action = true;
            self.popups.push(crate::damage::DamagePopup {
                damage,
                point: popup_point,
                age_ms: 0.0,
                style: self.debug[19],
            });
            self.actor_stop.apply(feedback.stop_ms.max(0) as f64);
            self.sound_requests.push(if name == "Attack3" {
                "MINERAL_SWDB_HIT"
            } else {
                "R_MINERALSWDA_HIT"
            });
            self.sound_requests.push(if self.goblin.hp == 0 {
                "R_GOBLIN_DIE"
            } else {
                "R_GOBLIN_DMG"
            });
        }
    }

    /// 当前动作名（与官方 `.ani` 文件名一致）。
    pub fn motion_name(&self) -> &'static str {
        match self.actor.motion {
            Motion::Stand => "Stay",
            Motion::Walk => "Move",
            Motion::Run => "Dash",
            Motion::Jump => "Jump",
            Motion::Attack => match self.actor.anim().clip().name.as_str() {
                "Attack2" => "Attack2",
                "Attack3" => "Attack3",
                _ => "Attack1",
            },
            Motion::JumpAttack => "JumpAttack",
            Motion::DashAttack => "DashAttack",
            Motion::UpperSlash => "Attack3",
            Motion::GhostSlash => "HardAttack",
        }
    }

    /// 把角色按官方分层合成后画到帧缓冲。
    pub fn draw_actor(&self, data: &GameData, fb: &mut Framebuffer) {
        for image in &self.afterimages {
            let motion = match image.pose.stage {
                1 => "Attack2",
                2 => "Attack3",
                _ => "Attack1",
            };
            let layers = character::compose(
                &data.assets,
                &data.avatar,
                &data.visual,
                &data.outfit,
                &data.weapon,
                motion,
                image.pose.frame,
            );
            let flip = image.pose.facing < 0;
            let fade = (image.age_ms - CANCEL_AFTERIMAGE_HOLD_MS).max(0.0)
                / (CANCEL_AFTERIMAGE_MS - CANCEL_AFTERIMAGE_HOLD_MS);
            let alpha = (255.0 * (1.0 - fade)).clamp(0.0, 255.0) as u8;
            for layer in layers {
                if let Some(f) = layer.image.frames.get(layer.frame) {
                    let (x, y) = layer_screen_pos(
                        image.pose.position.0,
                        self.camera_x,
                        layer.offset,
                        f.width as i32,
                        flip,
                    );
                    for py in 0..f.height as usize {
                        for px in 0..f.width as usize {
                            let a = (f.rgba[(py * f.width as usize + px) * 4 + 3] as u32
                                * alpha as u32
                                / 255) as u8;
                            let xx = if flip { f.width as usize - 1 - px } else { px };
                            fb.blend(
                                x + xx as i32,
                                image.pose.position.1 + y + py as i32,
                                rgba(255, 255, 255, a),
                            );
                        }
                    }
                }
            }
        }
        let motion = self.motion_name();
        let frame = self.actor.anim().frame();
        let layers = character::compose(
            &data.assets,
            &data.avatar,
            &data.visual,
            &data.outfit,
            &data.weapon,
            motion,
            frame,
        );
        let sy = self.actor.y - self.actor.altitude().round() as i32;
        let flip = self.actor.facing < 0;

        // 影子（随高度缩小变淡）
        let alt = self.actor.altitude();
        let shrink = (1.0 - (alt / 300.0)).clamp(0.3, 1.0);
        let rx = (24.0 * shrink) as i32;
        let ry = (7.0 * shrink) as i32;
        fb.fill_ellipse(
            self.actor.x - self.camera_x,
            self.actor.y,
            rx,
            ry,
            rgba(0, 0, 0, (110.0 * shrink) as u8),
        );

        for l in &layers {
            if let Some(f) = l.image.frames.get(l.frame) {
                let (dx, dy) = layer_screen_pos(
                    self.actor.x + self.actor.cast_vibration.offset(),
                    self.camera_x,
                    l.offset,
                    f.width as i32,
                    flip,
                );
                blit_frame(fb, f, dx, sy + dy, flip);
            }
        }
        for (effect, player) in self.ghost_effects.iter().zip(&self.ghost_fx) {
            if player.finished() {
                continue;
            }
            let Some(frame) = effect.frames.get(player.frame()) else {
                continue;
            };
            let Some(img) = data.assets.img(&frame.image) else {
                continue;
            };
            let Some(f) = img.frames.get(frame.image_index.max(0) as usize) else {
                continue;
            };
            let (x, y) = layer_screen_pos(
                self.actor.x,
                self.camera_x,
                (frame.image_pos.0 + f.x, frame.image_pos.1 + f.y),
                f.width as i32,
                flip,
            );
            for py in 0..f.height as usize {
                for px in 0..f.width as usize {
                    let i = (py * f.width as usize + px) * 4;
                    let a = f.rgba[i + 3] as u32 * frame.rgba[3] as u32 / 255;
                    let xx = x + if flip {
                        f.width as i32 - 1 - px as i32
                    } else {
                        px as i32
                    };
                    let yy = sy + y + py as i32;
                    let dst = fb.get(xx, yy);
                    // HardAttack1/2 explicitly request LINEARDODGE (additive light).
                    let add = |v: u8, c: u8, d: u8| {
                        (d as u32 + v as u32 * c as u32 * a / 65025).min(255) as u8
                    };
                    fb.put(
                        xx,
                        yy,
                        rgba(
                            add(f.rgba[i], frame.rgba[0], dst.r),
                            add(f.rgba[i + 1], frame.rgba[1], dst.g),
                            add(f.rgba[i + 2], frame.rgba[2], dst.b),
                            255,
                        ),
                    );
                }
            }
        }
    }

    /// 完整一帧。
    pub fn render(&self, data: &GameData, fb: &mut Framebuffer) {
        fb.clear(rgba(12, 14, 20, 255));
        self.room.draw_backdrop(fb, self.camera_x);
        let cam = self.camera_x;
        self.room.draw_objects_and_actor(
            fb,
            cam,
            self.actor.x,
            self.actor.y,
            &mut |fb: &mut Framebuffer| {
                if self.goblin.y < self.actor.y as f32 {
                    self.goblin.draw(&data.goblin, &data.assets, fb, cam);
                    self.draw_actor(data, fb);
                } else {
                    self.draw_actor(data, fb);
                    self.goblin.draw(&data.goblin, &data.assets, fb, cam);
                }
            },
        );
        for popup in &self.popups {
            popup.draw(&data.ui.numbers, fb, cam);
        }
        self.draw_hud(fb);
        self.panel.draw(self, data, fb);
        self.hud.draw(self, data, fb);
        self.notices.draw(fb);
        let label = format!("FPS {:.0}", self.fps);
        let w = crate::ui::measure_text(&label).0;
        fb.fill_rect(634 - w, 2, w + 6, 16, rgba(0, 0, 0, 180));
        crate::ui::text(fb, 637 - w, 4, &label, rgba(255, 238, 160, 255));
    }

    fn draw_hud(&self, fb: &mut Framebuffer) {
        let white = rgba(245, 247, 250, 255);
        let accent = rgba(150, 230, 170, 255);
        fb.fill_rect(0, 0, 640, 14, rgba(0, 0, 0, 150));
        font::draw_text(fb, 4, 4, self.actor.motion.label(), accent);
        fb.fill_rect(0, 14, 640, 14, rgba(0, 0, 0, 150));
        font::draw_text(
            fb,
            4,
            18,
            &format!(
                "GOBLIN {}/{} HIT {}",
                self.goblin.hp, self.goblin.max_hp, self.goblin.hits
            ),
            white,
        );
        font::draw_text(
            fb,
            340,
            18,
            &format!(
                "MP {} Z {:.1}s A {:.1}s",
                self.actor.mp,
                self.actor.upper_cooldown_ms() / 1000.0,
                self.actor.ghost_cooldown_ms() / 1000.0
            ),
            accent,
        );
        fb.fill_rect(0, 466, 640, 14, rgba(0, 0, 0, 150));
        font::draw_text(
            fb,
            4,
            470,
            "X ATTACK / Z UPPER / A GHOST / M INFO / F6 DEBUG / F5 RESET",
            white,
        );
        let line = format!(
            "X{:>5} Y{:>4} Z{:>4.0}",
            self.actor.x,
            self.actor.y,
            self.actor.altitude()
        );
        font::draw_text(fb, 60, 4, &line, white);
        let l2 = format!(
            "{:02}/{:02}",
            self.actor.anim().frame() + 1,
            self.actor.anim().clip().frame_count()
        );
        font::draw_text(fb, 238, 4, &l2, white);
        let mut extra = format!(
            "{} {:.0}%",
            self.motion_name(),
            self.actor.anim().rate_percent()
        );
        if !self.status.is_empty() {
            extra.push(' ');
            extra.push_str(&self.status);
        }
        font::draw_text(fb, 306, 4, &extra, rgba(250, 200, 120, 255));
        if self.show_keys {
            fb.fill_rect(0, 32, 640, 110, rgba(0, 0, 0, 210));
            font::draw_text(
                fb,
                4,
                36,
                &format!("KEY MANAGER {}ms / F7 CLOSE", self.actor.commands.now_ms),
                accent,
            );
            for (row, record) in self.actor.commands.records.iter().rev().take(7).enumerate() {
                font::draw_text(
                    fb,
                    4,
                    50 + row as i32 * 12,
                    &format!(
                        "{:?}  T{} HOLD{}  DOWN{} EDGE{}",
                        record.command,
                        record.press_ms,
                        record.hold_ms,
                        record.held as u8,
                        record.changed as u8
                    ),
                    white,
                );
            }
            for (row, d) in self
                .actor
                .commands
                .definitions
                .iter()
                .filter(|d| d.enabled || d.state != dnf_core::key_manager::MatchState::Idle)
                .take(7)
                .enumerate()
            {
                font::draw_text(
                    fb,
                    390,
                    50 + row as i32 * 12,
                    &format!("ID{} {:?} EN{}", d.id, d.state, d.enabled as u8),
                    accent,
                );
            }
        }
    }

    pub fn trace_line(&self) -> String {
        format!(
            "step={:4} motion={:5} x={:5} y={:4} z={:6.1} facing={:+} frame={:2}/{:2} phase={:?} dash_candidate={:+} attack_stage={} air_attacks={} audio_tags={:?} loop={:?} goblin_hp={} hits={} goblin_z={:.1} mp={} upper_cd={:.0} goblin_motion={} goblin_frame={} bounce={} actor_stop={:.1} target_stop={:.1} panel={} debug={} move_speed={:.0} physical_attack={} popups={} key_records={} key_clock={} ghost_cd={:.0} afterimages={} ghost_fx={} hud_slots={:?} hud_open={} hud_hover={:?} hud_drag={} failures={:?} vibration={} notices={} hp={} capsule_hover={:?}",
            self.steps,
            self.actor.motion.label(),
            self.actor.x,
            self.actor.y,
            self.actor.altitude(),
            self.actor.facing,
            self.actor.anim().frame() + 1,
            self.actor.anim().clip().frame_count(),
            self.actor.jump_phase(),
            self.actor.dash_candidate,
            self.actor.attack_stage().map(|s| s+1).unwrap_or(0),
            self.actor.air_attacks_used(),
            self.sound_requests,
            self.loop_sound,self.goblin.hp,self.goblin.hits,self.goblin.z,self.actor.mp,self.actor.upper_cooldown_ms(),
            self.goblin.motion(),self.goblin.frame(),self.goblin.is_bouncing(),self.actor_stop.remaining_ms,self.goblin.stop.remaining_ms,
            self.panel.open,self.panel.debug,self.actor.spec.move_speed,self.debug[4],self.popups.len(),
            self.actor.commands.records.len(),self.actor.commands.now_ms,
            self.actor.ghost_cooldown_ms(),self.afterimages.len(),self.ghost_fx.len(),
            self.hud.slots,self.hud.library_open,self.hud.hovered_skill(),self.hud.dragging(),
            self.actor.skill_failures,self.actor.cast_vibration.offset(),self.notices.len(),self.actor.hp,self.hud.capsule_hover(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dnf_core::Command;
    #[test]
    fn ghost_uses_real_skl_weapon_boxes_effects_sound_and_once_per_action_hit() {
        let Some(dir) = dnf_data::locate_client_dir() else {
            return;
        };
        let data = GameData::load(&dir, DEFAULT_MAP, Outfit::default_basic()).unwrap();
        let s = data.skills.iter().find(|s| s.id == 5).unwrap();
        assert_eq!(s.maximum_level, 30);
        assert_eq!(s.at_level(1).unwrap().values, vec![40]);
        assert_eq!(s.at_level(2).unwrap().values, vec![89]);
        assert_eq!(s.mp[29], 65);
        let mut game = data.build_game(KeyMap::official_default()).unwrap();
        game.actor.teleport(200, 329);
        game.goblin.reset(280, 329, &data.goblin);
        game.begin_input();
        game.set_key(Dik::A, true);
        game.step();
        assert_eq!(game.actor.motion, Motion::GhostSlash);
        assert_eq!(game.actor.mp, 130);
        assert_eq!(game.actor.anim().clip().delays_ms.len(), 18);
        assert!(game.sound_requests.contains(&"R_SM_GUE"));
        assert_eq!(game.ghost_fx.len(), 2);
        let mut saw_blade = false;
        let mut saw_down = false;
        for _ in 0..110 {
            game.begin_input();
            game.set_key(Dik::A, true);
            game.step();
            saw_blade |= game.ghost_fx.iter().any(|p| p.frame() > 0);
            saw_down |= game.goblin.motion() == "Down";
        }
        assert!(saw_blade && saw_down);
        assert_eq!(game.goblin.hits, 1);
        assert_eq!(game.goblin.hp, 242); // Test damage baseline18 + skl flat40.
        assert_eq!(game.actor.mp, 130);
        assert_eq!(game.actor.motion, Motion::Stand);
        assert!(game.ghost_fx.is_empty());
        let layers = character::compose(
            &data.assets,
            &data.avatar,
            &data.visual,
            &data.outfit,
            &data.weapon,
            "HardAttack",
            5,
        );
        assert!(layers.iter().any(|l| l.label.contains("weapon")));
    }

    #[test]
    fn cancel_during_impact_replaces_action_without_resetting_world_or_target_freeze() {
        let Some(dir) = dnf_data::locate_client_dir() else {
            return;
        };
        let data = GameData::load(&dir, DEFAULT_MAP, Outfit::default_basic()).unwrap();
        for key in [Dik::Z, Dik::A] {
            let mut game = data.build_game(KeyMap::official_default()).unwrap();
            game.actor.teleport(200, 329);
            game.goblin.reset(280, 329, &data.goblin);
            game.begin_input();
            game.set_key(Dik::X, true);
            game.step();
            for _ in 0..45 {
                if game.actor_stop.remaining_ms > 0.0 {
                    break;
                }
                game.begin_input();
                game.step();
            }
            assert!(game.actor_stop.remaining_ms > 0.0);
            let old = game.actor.anim().frame();
            let target_stop = game.goblin.stop.remaining_ms;
            game.begin_input();
            game.set_key(key, true);
            game.step();
            assert_eq!(game.afterimages.len(), 1);
            assert_eq!(game.afterimages[0].pose.frame, old);
            assert_eq!(game.actor.anim().frame(), 0);
            assert!(game.goblin.stop.remaining_ms < target_stop);
            assert!(!game.hit_this_action);
            assert!(matches!(
                game.actor.motion,
                Motion::UpperSlash | Motion::GhostSlash
            ));
            let mut with = Framebuffer::default();
            game.draw_actor(&data, &mut with);
            let afterimage = game.afterimages.pop().unwrap();
            let mut without = Framebuffer::default();
            game.draw_actor(&data, &mut without);
            assert!(with
                .data
                .chunks_exact(4)
                .zip(without.data.chunks_exact(4))
                .any(|(a, b)| a[0] > b[0] && a[0] == a[1] && a[1] == a[2]));
            game.afterimages.push(afterimage);
            for _ in 0..14 {
                game.begin_input();
                game.step();
            }
            assert_eq!(
                game.afterimages.len(),
                1,
                "afterimage must still be visible beyond the old 200ms lifetime"
            );
            assert_eq!(game.afterimages[0].pose.frame, old);
            for _ in 0..24 {
                game.begin_input();
                game.step();
            }
            assert!(game.afterimages.is_empty());
        }
    }
    #[test]
    fn real_skill_command_templates_and_hit_stop_input_clock_are_loaded() {
        let Some(dir) = dnf_data::locate_client_dir() else {
            return;
        };
        let data = GameData::load(&dir, DEFAULT_MAP, Outfit::default_basic()).unwrap();
        let (_, _, wave) = data
            .command_templates
            .iter()
            .find(|(_, path, _)| path.to_ascii_lowercase().ends_with("grandwave.skl"))
            .unwrap();
        assert_eq!(
            wave.stages.iter().map(|s| s.keys[0]).collect::<Vec<_>>(),
            vec![
                Command::Right,
                Command::Left,
                Command::Down,
                Command::Right,
                Command::Attack
            ]
        );
        let mut game = data.build_game(KeyMap::official_default()).unwrap();
        assert!(game
            .actor
            .commands
            .definitions
            .iter()
            .any(|d| d.id == 1050 && !d.enabled));
        game.actor_stop.apply(100.0);
        let start = game.actor.commands.now_ms;
        for step in 0..8 {
            game.begin_input();
            if step == 0 {
                game.set_key(Dik::X, true);
            }
            game.step();
        }
        assert!(game.actor.commands.now_ms >= start + 130);
        assert_eq!(game.actor.motion, Motion::Stand);
        assert_eq!(game.goblin.hits, 0);
    }

    #[test]
    fn debug_movement_values_change_actual_distance_and_restore_defaults() {
        let Some(dir) = dnf_data::locate_client_dir() else {
            return;
        };
        let data = GameData::load(&dir, DEFAULT_MAP, Outfit::default_basic()).unwrap();
        let mut normal = data.build_game(KeyMap::official_default()).unwrap();
        let mut edited = data.build_game(KeyMap::official_default()).unwrap();
        normal.actor.teleport(200, 329);
        edited.actor.teleport(200, 329);
        edited.apply_debug(crate::ui::Command::Set(13, 1700));
        for _ in 0..30 {
            for game in [&mut normal, &mut edited] {
                game.begin_input();
                game.set_key(Dik::RIGHT, true);
                game.step();
            }
        }
        assert!(edited.actor.x - 200 > (normal.actor.x - 200) * 3 / 2);
        edited.apply_debug(crate::ui::Command::Set(3, 10));
        edited.apply_debug(crate::ui::Command::Set(2, 999));
        assert_eq!(edited.actor.mp, 10);
        edited.apply_debug(crate::ui::Command::Reset);
        assert_eq!(edited.actor.spec.move_speed, data.spec.move_speed);
        assert_eq!(edited.actor.mp, 150);
    }

    #[test]
    fn goblin_hit_is_once_per_action_and_real_collision_is_required() {
        let Some(dir) = dnf_data::locate_client_dir() else {
            return;
        };
        let data = GameData::load(&dir, DEFAULT_MAP, Outfit::default_basic()).unwrap();
        for (dx, dy, expect) in [
            (110.0, 0.0, 1),
            (400.0, 0.0, 0),
            (110.0, 40.0, 0),
            (-80.0, 0.0, 0),
        ] {
            let mut game = data.build_game(KeyMap::official_default()).unwrap();
            game.actor.teleport(300, 329);
            game.goblin.x = game.actor.x as f32 + dx;
            game.goblin.y = game.actor.y as f32 + dy;
            for i in 0..100 {
                game.begin_input();
                if i == 0 {
                    game.set_key(Dik::X, true);
                }
                game.step();
            }
            assert_eq!(game.goblin.hits, expect, "offset {dx},{dy}");
        }
    }
    #[test]
    fn upper_floats_goblin_and_death_and_f5_reset_are_usable() {
        let Some(dir) = dnf_data::locate_client_dir() else {
            return;
        };
        let data = GameData::load(&dir, DEFAULT_MAP, Outfit::default_basic()).unwrap();
        let mut game = data.build_game(KeyMap::official_default()).unwrap();
        let mut apex = 0.0f32;
        for i in 0..100 {
            game.begin_input();
            if i == 0 {
                game.set_key(Dik::Z, true);
            }
            game.step();
            apex = apex.max(game.goblin.z);
        }
        assert_eq!(game.goblin.hits, 1);
        assert!(apex > 70.0);
        assert_eq!(game.goblin.hp, 272);
        game.goblin.hp = 1;
        game.goblin.x = game.actor.x as f32 + 110.0;
        game.goblin.z = 0.0;
        let mut died = false;
        for i in 0..80 {
            game.begin_input();
            if i == 0 {
                game.set_key(Dik::X, true);
            }
            game.step();
            died |= game.sound_requests.contains(&"R_GOBLIN_DIE");
        }
        assert!(died);
        assert_eq!(game.goblin.hp, 0);
        game.begin_input();
        game.set_key(Dik::F5, true);
        game.step();
        assert_eq!(game.goblin.hp, Goblin::MAX_HP);
        assert_eq!(game.actor.mp, 150);
        assert_eq!(game.goblin.hits, 0);
    }
    #[test]
    fn normal_combo_hits_each_stage_and_falling_air_attack_can_hit() {
        let Some(dir) = dnf_data::locate_client_dir() else {
            return;
        };
        let data = GameData::load(&dir, DEFAULT_MAP, Outfit::default_basic()).unwrap();
        let mut game = data.build_game(KeyMap::official_default()).unwrap();
        for i in 0..120 {
            game.begin_input();
            for dik in crate::script::sample(crate::script::ScriptKind::Attack, i, &game.keymap) {
                game.set_key(dik, true);
            }
            game.step();
        }
        assert_eq!(game.goblin.hits, 3);
        assert_eq!(game.goblin.hp, 246);
        let mut game = data.build_game(KeyMap::official_default()).unwrap();
        for i in 0..100 {
            game.begin_input();
            if i == 0 {
                game.set_key(Dik::C, true);
            }
            if i == 40 {
                game.set_key(Dik::X, true);
            }
            game.step();
        }
        assert_eq!(game.goblin.hits, 1);
        assert_eq!(game.actor.air_attacks_used(), 1);
    }

    #[test]
    fn action_audio_is_one_shot_and_swing_tracks_weapon_frame() {
        let Some(dir) = dnf_data::locate_client_dir() else {
            return;
        };
        let data = GameData::load(&dir, DEFAULT_MAP, Outfit::default_basic()).unwrap();
        for (kind, expected) in [
            (
                crate::script::ScriptKind::Attack,
                vec![("R_SM_ATK", 3), ("R_MINERALSWDA", 2), ("MINERALSWDB", 1)],
            ),
            (
                crate::script::ScriptKind::AirAttack,
                vec![
                    ("R_SM_JUMP_ATK", 1),
                    ("R_MINERALSWDA", 1),
                    ("JUMP_START", 1),
                    ("SM_JUMP", 1),
                    ("JUMP_END", 1),
                ],
            ),
        ] {
            let mut game = data.build_game(KeyMap::official_default()).unwrap();
            let mut counts = std::collections::BTreeMap::new();
            for step in 0..120 {
                game.begin_input();
                for dik in crate::script::sample(kind, step, &game.keymap) {
                    game.set_key(dik, true);
                }
                game.step();
                for tag in &game.sound_requests {
                    if expected.iter().any(|&(expected, _)| expected == *tag) {
                        *counts.entry(*tag).or_insert(0) += 1;
                    }
                    if ["R_MINERALSWDA", "MINERALSWDB"].contains(tag) {
                        assert!(matches!(
                            game.actor.motion,
                            Motion::Attack | Motion::JumpAttack
                        ));
                        assert_eq!(
                            game.actor.anim().frame(),
                            data.swing_cues()[game.motion_name()].0
                        );
                    }
                }
            }
            assert_eq!(counts, expected.into_iter().collect());
        }
    }

    #[test]
    fn default_weapon_tracks_every_official_actor_motion() {
        let Some(dir) = dnf_data::locate_client_dir() else {
            return;
        };
        let data = GameData::load(&dir, DEFAULT_MAP, Outfit::default_basic()).unwrap();
        assert_eq!(data.weapon.variation, (0, 0));
        assert_eq!(
            data.weapon
                .parts
                .iter()
                .map(|p| p.layer)
                .collect::<Vec<_>>(),
            vec![2150, 650]
        );
        for (name, frames) in &data.visual.motions {
            for frame in 0..frames.len() {
                let layers = character::compose(
                    &data.assets,
                    &data.avatar,
                    &data.visual,
                    &data.outfit,
                    &data.weapon,
                    name,
                    frame,
                );
                let weapon: Vec<_> = layers
                    .iter()
                    .filter(|l| l.label.starts_with("weapon_"))
                    .collect();
                assert_eq!(weapon.len(), 2, "{name} frame{frame}");
            }
        }
    }

    /// 统计"左朝向不是右朝向镜像"的错位像素。
    ///
    /// 只比较**两侧都真的画出了角色**的像素：
    /// * 脚下影子是 alpha 混合到背景上的，左右背后草色不同 ⇒ 影子带跳过；
    /// * 场景物件（尤其 `[close]` 前景树）**不参与镜像**，会挡住角色的一部分 ⇒
    ///   某一侧被挡住的像素不可比，必须跳过（否则会把"树挡住"误判成镜像错误）。
    fn mirror_mismatch(
        right: &Framebuffer,
        left: &Framebuffer,
        backdrop: &Framebuffer,
        ax: i32,
        ay: i32,
    ) -> (i32, i32) {
        let (mut painted, mut bad) = (0, 0);
        for dy in -190..20 {
            for dx in -150..150 {
                let (px, py) = (ax + dx, ay + dy);
                let (mx, my) = (ax - dx, py);
                let r = right.get(px, py);
                if r == backdrop.get(px, py) {
                    continue; // 右帧这里不是角色画的
                }
                if left.get(mx, my) == backdrop.get(mx, my) {
                    continue; // 左帧镜像处被前景物件挡住（或超出屏幕）⇒ 不可比
                }
                painted += 1;
                if !(-8..=8).contains(&dy) && left.get(mx, my) != r {
                    bad += 1;
                }
            }
        }
        (painted, bad)
    }

    /// Normal jump: prepare first, then x inherits takeoff mode and y is halved.
    #[test]
    fn 跳跃中可调整落地点() {
        let Some(dir) = dnf_data::locate_client_dir() else {
            return;
        };
        let data = match GameData::load(&dir, DEFAULT_MAP, Outfit::default_basic()) {
            Ok(d) => d,
            Err(e) if dnf_data::client_busy(&e) => return,
            Err(e) => panic!("装载官方资源失败: {}", e),
        };
        let mut game = data.build_game(KeyMap::default()).expect("建房");

        // 原地起跳
        game.begin_input();
        game.set_key(Dik::C, true);
        game.step();
        assert_eq!(game.actor.motion, Motion::Jump, "应已起跳");
        for _ in 0..5 {
            game.begin_input();
            game.step();
        }
        while !game.actor.is_airborne() {
            game.begin_input();
            game.step();
        }
        let x0 = game.actor.x;

        // 空中按右 ⇒ 落地点应右移
        for _ in 0..6 {
            game.begin_input();
            game.set_key(Dik::RIGHT, true);
            game.step();
        }
        let dx_right = game.actor.x - x0;
        // 原地起跳 = 走档继承 ⇒ 143×0.85 = 121.55 px/s ⇒ 6 步 ≈ 12px
        assert!(
            (dx_right as f32 - 12.2).abs() <= 2.0,
            "空中按右键未按走档调整落地点: dx={}",
            dx_right
        );

        // Y is 114/2 * 0.85 = 48.45 px/s; six steps about 4.845px.
        let y0 = game.actor.y;
        for _ in 0..6 {
            game.begin_input();
            game.set_key(Dik::UP, true);
            game.step();
        }
        let dy_up = y0 - game.actor.y;
        assert!(
            (dy_up as f32 - 4.845).abs() <= 1.0,
            "空中按上键未调整纵深落点: dy={}",
            dy_up
        );

        // 空中改按左 ⇒ 落地点应左移
        let x1 = game.actor.x;
        for _ in 0..6 {
            game.begin_input();
            game.set_key(Dik::LEFT, true);
            game.step();
        }
        assert!(
            game.actor.x < x1,
            "空中改按左键未反向: x {} -> {}",
            x1,
            game.actor.x
        );
    }

    /// 把角色挪出屏幕，渲染一张纯背景参考帧（相机不动）。
    fn render_backdrop(data: &GameData, game: &mut Game) -> Framebuffer {
        let (saved_x, saved_y) = (game.actor.x, game.actor.y);
        game.actor.x = -100_000;
        let mut fb = Framebuffer::new(640, 480);
        game.render(data, &mut fb);
        game.actor.x = saved_x;
        game.actor.y = saved_y;
        fb
    }

    /// ★ 左朝向必须正好是右朝向**关于角色锚点的水平镜像**。
    ///
    /// 官方镜像 = 动画速率取负（`IRDActiveObject_vmt37C`），等价于以角色脚点为轴翻转画布。
    /// 回归：曾经只把精灵原地翻面而不改 x ⇒ 头发/护肩/护臂在左朝向错位；
    /// 后来 x 改了但少了 1 像素 ⇒ 整层差一列。
    #[test]
    fn 左右朝向互为镜像_站立与行走() {
        let Some(dir) = dnf_data::locate_client_dir() else {
            return;
        };
        let data = match GameData::load(&dir, DEFAULT_MAP, Outfit::default_basic()) {
            Ok(d) => d,
            // 官方客户端正开着会独占 Script.pvf/NPK ⇒ 跳过（本工程只读素材）
            Err(e) if dnf_data::client_busy(&e) => {
                eprintln!("跳过：{}", e);
                return;
            }
            Err(e) => panic!("装载官方资源失败: {}", e),
        };
        let mut game = data.build_game(KeyMap::default()).expect("建房");
        game.actor.x = 560;
        game.actor.y = 345;
        game.camera_x = 240;

        // ---- ① 站立 ----
        let backdrop = render_backdrop(&data, &mut game);
        let mut right = Framebuffer::new(640, 480);
        let mut left = Framebuffer::new(640, 480);
        game.actor.facing = 1;
        game.render(&data, &mut right);
        game.actor.facing = -1;
        game.render(&data, &mut left);
        game.actor.facing = 1;
        let (painted, bad) = mirror_mismatch(
            &right,
            &left,
            &backdrop,
            game.actor.x - game.camera_x,
            game.actor.y,
        );
        assert!(
            painted > 2_000,
            "站立时角色几乎没画出来（{} 像素）",
            painted
        );
        assert_eq!(bad, 0, "站立：镜像后有 {} 个像素错位", bad);

        // ---- ② 行走（按住左，走 20 步到行走帧）----
        game.begin_input();
        game.set_key(Dik::LEFT, true);
        for _ in 0..20 {
            game.step();
        }
        game.begin_input();
        game.actor.facing = -1; // 保持朝左
        let backdrop = render_backdrop(&data, &mut game);
        let mut wl = Framebuffer::new(640, 480);
        let mut wr = Framebuffer::new(640, 480);
        game.render(&data, &mut wl);
        game.actor.facing = 1;
        game.render(&data, &mut wr);
        game.actor.facing = -1;
        eprintln!(
            "WALK ax={} actor=({},{}) cam={} motion={:?} frame={}",
            game.actor.x - game.camera_x,
            game.actor.x,
            game.actor.y,
            game.camera_x,
            game.actor.motion,
            game.actor.anim().frame()
        );
        let (painted, bad) = mirror_mismatch(
            &wr,
            &wl,
            &backdrop,
            game.actor.x - game.camera_x,
            game.actor.y,
        );
        // 走动的位置正好被 [close] 前景枯树挡住大半（官方行为：前景层遮角色），
        // 所以可比像素少；但留下来的每一个都必须严格镜像。
        assert!(painted > 400, "行走时可比像素过少（{}）", painted);
        assert_eq!(bad, 0, "行走：镜像后有 {} 个像素错位", bad);
    }

    /// 纯几何自检（不依赖资源）：镜像后的列集合必须正好是原列集合的镜像。
    #[test]
    fn 图层镜像几何() {
        for (offset_x, w) in [(-32, 64), (-2, 18), (5, 40), (0, 1)] {
            let (rx, _) = layer_screen_pos(320, 0, (offset_x, 0), w, false);
            let (lx, _) = layer_screen_pos(320, 0, (offset_x, 0), w, true);
            assert_eq!(rx, 320 + offset_x);
            assert_eq!(lx, 321 - (offset_x + w));
            let mut mirrored: Vec<i32> = (rx..rx + w).map(|c| 2 * 320 - c).collect();
            mirrored.sort();
            assert_eq!(
                mirrored,
                (lx..lx + w).collect::<Vec<_>>(),
                "offset_x={} w={}",
                offset_x,
                w
            );
        }
    }
}
