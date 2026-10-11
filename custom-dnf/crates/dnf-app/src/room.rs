//! 真实房间：官方 `.map` + `.til` + NPK 贴图。
//!
//! 渲染顺序（依据 `docs/90` 会话 12/13 对官方层次实测的结论）：
//!
//! ```text
//! 远景层(distantback，视差 56%) → 中景层(middleback，视差 90%) → 瓦片条(世界锁定)
//!   → [closeback] 物件 → [bottom] 物件 → [normal] 物件(按 y 排序)
//!   → 角色(按 y 参与排序) → [close] 物件(前景，遮住角色)
//! ```
//!
//! * 官方内部渲染分辨率 **640×480**；瓦片条画布恰为 `224×480`，故 `map` 坐标可直接当屏幕 `y`。
//! * 视差率取自 `.map` 的 `[far/middle/near sight scroll]`（洛兰 104 = 56/90/110，÷100）。
//! * 远景/中景横向按 640 取模平铺（它们宽恰为 640，不平铺会在相机移动时露边）。

use crate::assets::ClientAssets;
use dnf_data::img::ImgFrame;
use dnf_data::mapfile::{MapFile, MapObject};
use dnf_render::framebuffer::Framebuffer;
use std::rc::Rc;

/// 一条已加载的瓦片带。
pub struct TileStrip {
    pub x: i32,
    /// 通行网格（行 × 列）。
    pub pass: Vec<Vec<i32>>,
    pub image: Rc<dnf_data::img::ImgFile>,
    pub frame: usize,
}

/// 一个已加载的场景物件。
pub struct SceneObject {
    pub layer: String,
    pub x: i32,
    pub y: i32,
    pub image: Rc<dnf_data::img::ImgFile>,
    pub frame: usize,
    /// 相对物件锚点的绘制偏移（`[IMAGE POS]` + 画布内 `(x, y)`）。
    pub offset: (i32, i32),
}

pub struct Room {
    pub map: MapFile,
    pub tiles: Vec<TileStrip>,
    pub backgrounds: Vec<Background>,
    pub objects: Vec<SceneObject>,
    pub walk_x: (i32, i32),
    /// 装载诊断（逐条说明成/败）。
    pub report: Vec<String>,
}

/// 一层带视差的远景/中景背景。
pub struct Background {
    /// 视差率（百分比，取自 `.map` 的 `[far/middle sight scroll]`）。
    pub rate: i32,
    pub image: Rc<dnf_data::img::ImgFile>,
    pub frame: usize,
    /// 锚点（`.ani` 的 `[IMAGE POS]`）。
    pub pos: (i32, i32),
}

impl Room {
    /// 从官方资源装载一个房间。`map_pvf_path` 形如 `map/lorien/104.map`。
    pub fn load(assets: &ClientAssets, map_pvf_path: &str) -> Result<Room, String> {
        let map = assets
            .map(map_pvf_path)
            .ok_or_else(|| format!("读不到 {}", map_pvf_path))?;
        let room_dir = map_pvf_path
            .rsplit_once('/')
            .map(|(d, _)| d.to_string())
            .unwrap_or_else(|| "map".into());
        let mut report = Vec::new();
        report.push(format!(
            "地图 {}: {} 条瓦片 / {} 层背景 / {} 个物件 / 视差 {}/{}/{}",
            map_pvf_path,
            map.tiles.len(),
            map.backgrounds.len(),
            map.objects.len(),
            map.far_scroll,
            map.middle_scroll,
            map.near_scroll
        ));

        // ---- 瓦片条 ----
        let mut tiles = Vec::new();
        for (i, t) in map.tiles.iter().enumerate() {
            let Some(til) = assets.til(&room_dir, &t.path) else {
                report.push(format!("瓦片 {} 读取失败：{}", i, t.path));
                continue;
            };
            let Some(img) = assets.img(&til.image) else {
                report.push(format!("瓦片 {} 贴图缺失：{}", i, til.image));
                continue;
            };
            let frame = til.image_index.max(0) as usize;
            if let Some(f) = img.frames.get(frame) {
                report.push(format!(
                    "瓦片 {} x={}: {} → {} 帧{} {}x{} @({},{}) 画布{}x{} 通行带{:?}",
                    i,
                    i as i32 * 224,
                    t.path,
                    til.image,
                    frame,
                    f.width,
                    f.height,
                    f.x,
                    f.y,
                    f.canvas_width,
                    f.canvas_height,
                    til.walk_band_at(0)
                ));
            } else {
                report.push(format!(
                    "瓦片 {} 帧号越界：{} 只有 {} 帧，却要帧{}",
                    i,
                    til.image,
                    img.frames.len(),
                    frame
                ));
            }
            tiles.push(TileStrip {
                x: i as i32 * 224,
                pass: til.pass.clone(),
                image: img,
                frame,
            });
        }
        if tiles.is_empty() {
            return Err(format!("{} 没有可用的瓦片条", map_pvf_path));
        }

        // ---- 背景层（带视差） ----
        let mut backgrounds = Vec::new();
        for (idx, bg) in map.backgrounds.iter().enumerate() {
            let name = format!("{}/{}", room_dir, bg.anim.replace('\\', "/"));
            let Some(ani) = assets.ani(&name) else {
                report.push(format!("背景 {} 动画缺失：{}", idx, name));
                continue;
            };
            let Some(f0) = ani.frames.first() else {
                report.push(format!("背景 {} 动画没有帧：{}", idx, name));
                continue;
            };
            let Some(img) = assets.img(&f0.image) else {
                report.push(format!("背景 {} 贴图缺失：{}", idx, f0.image));
                continue;
            };
            let rate = if idx == 0 {
                map.far_scroll
            } else {
                map.middle_scroll
            };
            let frame = f0.image_index.max(0) as usize;
            if let Some(f) = img.frames.get(frame) {
                report.push(format!(
                    "背景 {} 视差{}%: {} → {} 帧{} {}x{} @({},{})",
                    idx, rate, bg.anim, f0.image, frame, f.width, f.height, f.x, f.y
                ));
            }
            backgrounds.push(Background {
                rate,
                image: img,
                frame,
                pos: f0.image_pos,
            });
        }

        // ---- 物件 ----
        let mut objects = Vec::new();
        for o in &map.objects {
            match load_object(assets, &room_dir, o) {
                Some(so) => {
                    report.push(format!(
                        "物件 [{}] {} @({},{}) → 帧{} 偏移({},{})",
                        o.layer, o.anim, o.x, o.y, so.frame, so.offset.0, so.offset.1
                    ));
                    objects.push(so);
                }
                None => report.push(format!("物件失败：{} @({},{})", o.anim, o.x, o.y)),
            }
        }

        let walk_x = map.walkable_x();
        report.push(format!(
            "房间：{} 条瓦片带，宽 {}px，可走 x {}..{}",
            tiles.len(),
            map.room_width(),
            walk_x.0,
            walk_x.1
        ));
        Ok(Room {
            map,
            tiles,
            backgrounds,
            objects,
            walk_x,
            report,
        })
    }

    pub fn room_width(&self) -> i32 {
        self.map.room_width()
    }

    /// 世界 x 处的可通行纵深带（屏幕 y 的 min..=max）。
    ///
    /// 官方地形完全由通行网格描述（斜坡也是网格），所以这里按角色所在的**瓦片条**取带。
    pub fn walk_band(&self, world_x: i32) -> (i32, i32) {
        let idx = (world_x.max(0) / 224) as usize;
        let tile = self
            .tiles
            .get(idx)
            .or_else(|| self.tiles.last())
            .or_else(|| self.tiles.first());
        let Some(tile) = tile else { return (250, 400) };
        let col = ((world_x - tile.x).clamp(0, 223) / 16) as usize;
        let mut lo = None;
        let mut hi = None;
        for (r, row) in tile.pass.iter().enumerate() {
            if row.get(col).copied().unwrap_or(2) == 0 {
                let y = r as i32 * 16;
                lo.get_or_insert(y);
                hi = Some(y + 15);
            }
        }
        match (lo, hi) {
            (Some(a), Some(b)) => (a + 8, b - 4),
            _ => (250, 400),
        }
    }

    /// 背景 + 瓦片（角色之前的一切）。
    pub fn draw_backdrop(&self, fb: &mut Framebuffer, camera_x: i32) {
        for bg in &self.backgrounds {
            let Some(f) = bg.image.frames.get(bg.frame) else {
                continue;
            };
            let shift = camera_x * bg.rate / 100;
            let base_x = bg.pos.0 + f.x - shift;
            // 横向按 640 取模平铺（官方远景图宽恰为 640）
            let period = 640;
            let mut sx = base_x.rem_euclid(period) - period;
            while sx < fb.width as i32 {
                blit_frame(fb, f, sx, bg.pos.1 + f.y, false);
                sx += period;
            }
        }
        for t in &self.tiles {
            let Some(f) = t.image.frames.get(t.frame) else {
                continue;
            };
            // 瓦片条画布是 224x480，美术贴在画布 `(0, 247|277)` 处（`.til` 的网格 = 整块画布），
            // 所以必须叠上帧自己的画布偏移。
            blit_frame(fb, f, t.x - camera_x + f.x, f.y, false);
        }
    }

    /// 物件 + 角色按官方层次顺序绘制。
    pub fn draw_objects_and_actor(
        &self,
        fb: &mut Framebuffer,
        camera_x: i32,
        actor_x: i32,
        actor_y: i32,
        draw_actor: &mut dyn FnMut(&mut Framebuffer),
    ) {
        for layer in ["closeback", "bottom"] {
            for o in self.objects.iter().filter(|o| o.layer == layer) {
                self.draw_object(fb, camera_x, o);
            }
        }
        let mut normal: Vec<&SceneObject> = self
            .objects
            .iter()
            .filter(|o| o.layer == "normal")
            .collect();
        normal.sort_by_key(|o| o.y);
        for o in normal {
            self.draw_object(fb, camera_x, o);
        }
        // 角色（脚点 y 与 normal 层同轴；官方 close 层永远在角色之前绘制所以会遮住角色）
        let _ = (actor_x, actor_y);
        draw_actor(fb);
        for o in self.objects.iter().filter(|o| o.layer == "close") {
            self.draw_object(fb, camera_x, o);
        }
    }

    fn draw_object(&self, fb: &mut Framebuffer, camera_x: i32, o: &SceneObject) {
        let Some(f) = o.image.frames.get(o.frame) else {
            return;
        };
        blit_frame(fb, f, o.x - camera_x + o.offset.0, o.y + o.offset.1, false);
    }
}

fn load_object(assets: &ClientAssets, room_dir: &str, o: &MapObject) -> Option<SceneObject> {
    let name = format!("{}/{}", room_dir, o.anim.replace('\\', "/"));
    let ani = assets.ani(&name)?;
    let f0 = ani.frames.first()?;
    let img = assets.img(&f0.image)?;
    let frame = f0.image_index.max(0) as usize;
    let off = match img.frames.get(frame) {
        Some(f) => (f0.image_pos.0 + f.x, f0.image_pos.1 + f.y),
        None => f0.image_pos,
    };
    Some(SceneObject {
        layer: o.layer.clone(),
        x: o.x,
        y: o.y,
        image: img,
        frame,
        offset: off,
    })
}

/// 贴一帧：`(x, y)` 就是像素左上角。
///
/// **画布偏移（`f.x` / `f.y`）由调用方自行叠加**：角色与物件的偏移里已经含了
/// `[IMAGE POS] + 画布 (x, y)`（见 `character.rs` / `load_object`），这里再叠一次就会错位。
pub fn blit_frame(fb: &mut Framebuffer, f: &ImgFrame, x: i32, y: i32, flip: bool) {
    if f.rgba.is_empty() {
        return;
    }
    fb.blit_alpha(
        &f.rgba,
        f.width as usize,
        f.height as usize,
        x,
        y,
        flip,
        dnf_render::framebuffer::Color::WHITE,
    );
}
