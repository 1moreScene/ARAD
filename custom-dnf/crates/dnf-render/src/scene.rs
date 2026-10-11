//! 白模场景与角色白模绘制。
//!
//! 「白模」= 不含任何官方贴图，用几何图元表达场景与角色剪影；目的是把
//! **移动 / 奔跑 / 跳跃逻辑**与**坐标/层次关系**先跑通并可目视验证。
//!
//! 坐标约定（与官方一致，见 `docs/90` 会话 13 第 5 条）：
//! * 世界 `(x, y)`：`x` = 水平轴，`y` = 纵深轴；
//! * 纵深轴 `y` **直接就是屏幕 y**（`.map` 里的对象锚点 y 可直接当屏幕 y）；
//! * 跳跃高度 `z` 只把绘制位置向上偏移，并让影子变小变淡。

use crate::font;
use crate::framebuffer::{rgba, Framebuffer};

/// 场景矩形（世界坐标）。
#[derive(Copy, Clone, Debug)]
pub struct WhiteBoxScene {
    /// 房间水平范围（世界 x）。
    pub x_min: i32,
    pub x_max: i32,
    /// 可通行纵深范围（世界 y）。
    pub y_min: i32,
    pub y_max: i32,
    /// 视口尺寸。
    pub view_w: i32,
    pub view_h: i32,
}

impl Default for WhiteBoxScene {
    fn default() -> Self {
        // 参考官方洛兰 b1 房间：宽 7 条 × 224px = 1552px；纵深可走带与官方通行网格同量级。
        WhiteBoxScene {
            x_min: 0,
            x_max: 1552,
            y_min: 250,
            y_max: 400,
            view_w: 640,
            view_h: 480,
        }
    }
}

/// 相机（水平跟随 + 夹取到房间边界）。
#[derive(Copy, Clone, Debug)]
pub struct Camera {
    pub x: i32,
}

impl Camera {
    pub fn follow(scene: &WhiteBoxScene, target_x: i32, current: i32, max_step: i32) -> i32 {
        // 目标：把角色放在视口中心；随后限制单帧最大移动量并夹取到房间边界。
        let desired = target_x - scene.view_w / 2;
        let delta = (desired - current).clamp(-max_step, max_step);
        let x = current + delta;
        let lo = scene.x_min;
        let hi = (scene.x_max - scene.view_w).max(lo);
        x.clamp(lo, hi)
    }

    pub fn world_to_screen_x(&self, world_x: i32) -> i32 {
        world_x - self.x
    }
}

impl WhiteBoxScene {
    /// 场景绘制（背景 → 地面 → 地面网格 → 后墙），必须在角色之前调用。
    pub fn draw_background(&self, fb: &mut Framebuffer, cam: &Camera) {
        let horizon = self.y_min - 40;

        // 天空/远景
        fb.fill_rect(0, 0, self.view_w, horizon.max(1), rgba(38, 42, 58, 255));
        // 后墙
        fb.fill_rect(
            0,
            horizon.max(1),
            self.view_w,
            (self.y_min - horizon).max(1),
            rgba(54, 58, 76, 255),
        );
        // 地面（可通行带）
        let floor_h = (self.y_max - self.y_min).max(1);
        fb.fill_rect(0, self.y_min, self.view_w, floor_h, rgba(96, 100, 116, 255));
        // 地面更远侧（房间纵深以外）
        fb.fill_rect(
            0,
            self.y_max,
            self.view_w,
            (self.view_h - self.y_max).max(1),
            rgba(66, 70, 84, 255),
        );

        // 房间水平边界
        let lx = cam.world_to_screen_x(self.x_min);
        let rx = cam.world_to_screen_x(self.x_max);
        fb.vline(lx, horizon, self.y_max + 60, rgba(210, 214, 226, 200));
        fb.vline(rx, horizon, self.y_max + 60, rgba(210, 214, 226, 200));

        // 纵深带边界
        fb.hline(0, self.view_w - 1, self.y_min, rgba(180, 186, 200, 160));
        fb.hline(0, self.view_w - 1, self.y_max, rgba(180, 186, 200, 160));

        // 水平网格（每 224px = 一条官方瓦片带宽度）
        let mut wx = self.x_min;
        while wx <= self.x_max {
            let sx = cam.world_to_screen_x(wx);
            fb.vline(sx, self.y_min, self.y_max, rgba(140, 146, 162, 90));
            wx += 224;
        }
        // 纵深网格
        let mut wy = self.y_min;
        while wy <= self.y_max {
            fb.hline(0, self.view_w - 1, wy, rgba(140, 146, 162, 70));
            wy += 16; // 官方通行网格单元 = 16px
        }
    }

    /// 绘制一个白模角色剪影。
    ///
    /// * `world` = 地面平面坐标；`alt` = 跳跃高度（像素）；
    /// * `phase` = 0..1 的动画相位，用来驱动手脚摆动（白模也要能看出在走）。
    #[allow(clippy::too_many_arguments)]
    pub fn draw_actor_whitebox(
        &self,
        fb: &mut Framebuffer,
        cam: &Camera,
        world: (i32, i32),
        alt: f32,
        facing: i32,
        phase: f32,
        motion: &str,
    ) {
        let sx = cam.world_to_screen_x(world.0);
        let sy = world.1 - alt.round() as i32;

        // 影子：随高度缩小变淡
        let shrink = (1.0 - (alt / 260.0)).clamp(0.25, 1.0);
        let rx = (26.0 * shrink) as i32;
        let ry = (7.0 * shrink) as i32;
        let alpha = (150.0 * shrink) as u8;
        fb.fill_ellipse(sx, world.1, rx, ry, rgba(16, 18, 24, alpha));

        // 身体参数（白模：量级贴合官方鬼剑士在 640×480 下的实际身高 ~100px）
        let body_h = 92;
        let body_w = 30;
        let head_r = 11;
        let leg = 22;

        let white = rgba(238, 240, 246, 255);
        let shade = rgba(196, 202, 216, 255);

        // 腿（行走/奔跑时前后摆）
        let swing = if motion == "STAND" {
            0.0
        } else {
            (phase * std::f32::consts::TAU).sin()
        };
        let swing_amt = if motion == "RUN" { 12.0 } else { 6.0 };
        let off = (swing * swing_amt) as i32;
        let hip_y = sy - body_h;
        fb.fill_rect(sx - 8 + off, hip_y + body_h - leg, 6, leg, shade);
        fb.fill_rect(sx + 2 - off, hip_y + body_h - leg, 6, leg, shade);

        // 躯干
        fb.fill_rect(sx - body_w / 2, hip_y, body_w, body_h - leg, white);
        // 躯干描边（让白模在浅背景上也能分辨）
        fb.rect_outline(
            sx - body_w / 2,
            hip_y,
            body_w,
            body_h - leg,
            rgba(70, 76, 92, 220),
        );

        // 手臂
        let arm_off = (-swing * swing_amt) as i32;
        fb.fill_rect(sx - body_w / 2 - 5, hip_y + 12 + arm_off, 5, 32, shade);
        fb.fill_rect(sx + body_w / 2, hip_y + 12 - arm_off, 5, 32, shade);

        // 头
        fb.fill_ellipse(sx, hip_y - head_r - 2, head_r, head_r, white);
        fb.rect_outline(
            sx - head_r,
            hip_y - 2 * head_r - 2,
            2 * head_r + 1,
            2 * head_r + 1,
            rgba(70, 76, 92, 200),
        );

        // 朝向指示（面朝哪边，在眼睛一侧画一个深色块）
        let eye_x = if facing >= 0 { sx + 2 } else { sx - 5 };
        fb.fill_rect(eye_x, hip_y - head_r - 3, 3, 3, rgba(40, 44, 56, 255));
    }

    /// HUD：状态 / 坐标 / 高度 / 帧号。
    #[allow(clippy::too_many_arguments)]
    pub fn draw_hud(
        &self,
        fb: &mut Framebuffer,
        motion: &str,
        world: (i32, i32),
        alt: f32,
        frame: usize,
        frame_count: usize,
        extra: &str,
    ) {
        let white = rgba(245, 247, 250, 255);
        let accent = rgba(120, 220, 160, 255);
        fb.fill_rect(0, 0, self.view_w, 14, rgba(0, 0, 0, 120));
        font::draw_text(fb, 4, 4, motion, accent);
        let line = format!("X{:>5} Y{:>4} Z{:>5.0}", world.0, world.1, alt);
        font::draw_text(fb, 60, 4, &line, white);
        let l2 = format!("FRM {:02}/{:02}", frame + 1, frame_count);
        font::draw_text(fb, 250, 4, &l2, white);
        if !extra.is_empty() {
            font::draw_text(fb, 350, 4, extra, rgba(250, 200, 120, 255));
        }
    }
}
