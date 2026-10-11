//! # dnf-render —— DNF 同人客户端软件渲染层
//!
//! 无第三方依赖。逻辑分辨率 640×480（= 官方内部渲染分辨率）。
//!
//! * [`framebuffer`] —— RGBA8 帧缓冲 + 图元 + alpha 混合
//! * [`scene`] —— 白模场景 / 相机 / 白模角色剪影 / HUD
//! * [`font`] —— 5×7 点阵字体
//! * [`png`] —— PNG 导出（用于无头确定性验证）

pub mod font;
pub mod framebuffer;
pub mod png;
pub mod sampling;
pub mod scene;

pub use framebuffer::{rgba, Color, Framebuffer, HEIGHT, WIDTH};
pub use scene::{Camera, WhiteBoxScene};

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scripted_rotation_scale_and_mirror_use_actor_anchor() {
        use framebuffer::SpriteTransform;
        let source = [255, 0, 0, 255, 0, 0, 255, 255];
        let mut fb = Framebuffer::new(10, 10);
        fb.blit_transformed(
            &source,
            (2, 1),
            SpriteTransform {
                anchor: (5, 5),
                offset: (-1, -2),
                rate: (1.0, 1.0),
                angle: std::f32::consts::PI,
                tint: Color::WHITE,
            },
        );
        assert_eq!(fb.get(6, 7), rgba(255, 0, 0, 255));
        assert_eq!(fb.get(5, 7), rgba(0, 0, 255, 255));
        fb.clear(Color::TRANSPARENT);
        fb.blit_transformed(
            &source,
            (2, 1),
            SpriteTransform {
                anchor: (5, 5),
                offset: (-1, -2),
                rate: (-1.0, 1.0),
                angle: 0.0,
                tint: Color::WHITE,
            },
        );
        assert_eq!(fb.get(6, 3), rgba(255, 0, 0, 255));
        assert_eq!(fb.get(5, 3), rgba(0, 0, 255, 255));
    }

    #[test]
    fn 帧缓冲_基础图元与混合() {
        let mut fb = Framebuffer::new(16, 16);
        fb.clear(rgba(0, 0, 0, 255));
        fb.fill_rect(2, 2, 4, 4, rgba(255, 0, 0, 255));
        assert_eq!(fb.get(3, 3), rgba(255, 0, 0, 255));
        assert_eq!(fb.get(1, 1), rgba(0, 0, 0, 255));
        // 半透明白叠在黑上 → 灰
        fb.blend(8, 8, rgba(255, 255, 255, 128));
        let c = fb.get(8, 8);
        assert!(c.r > 100 && c.r < 160, "r={}", c.r);
        // 越界不 panic
        fb.fill_rect(-10, -10, 400, 400, rgba(1, 2, 3, 40));
        fb.put(-1, -1, Color::WHITE);
        assert_eq!(fb.get(-1, -1), Color::TRANSPARENT);
    }

    #[test]
    fn 帧缓冲_alpha混合写入() {
        let mut fb = Framebuffer::new(4, 4);
        fb.clear(rgba(0, 0, 0, 255));
        let src = vec![255u8, 255, 255, 255, 255, 255, 255, 0];
        fb.blit_alpha(&src, 2, 1, 0, 0, false, Color::WHITE);
        assert_eq!(fb.get(0, 0), rgba(255, 255, 255, 255));
        assert_eq!(fb.get(1, 0), rgba(0, 0, 0, 255)); // alpha=0 不画
                                                      // 水平镜像
        fb.clear(rgba(0, 0, 0, 255));
        let src2 = vec![9u8, 0, 0, 255, 7, 0, 0, 255];
        fb.blit_alpha(&src2, 2, 1, 0, 0, true, Color::WHITE);
        assert_eq!(fb.get(0, 0).r, 7);
        assert_eq!(fb.get(1, 0).r, 9);
    }

    #[test]
    fn png_头部与块结构正确() {
        let mut fb = Framebuffer::new(4, 2);
        fb.clear(rgba(10, 20, 30, 255));
        let bytes = png::encode(&fb);
        assert_eq!(
            &bytes[0..8],
            &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]
        );
        assert_eq!(&bytes[12..16], b"IHDR");
        // 宽高
        assert_eq!(
            u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]),
            4
        );
        assert_eq!(
            u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]),
            2
        );
        // 结尾 IEND
        assert_eq!(&bytes[bytes.len() - 8..bytes.len() - 4], b"IEND");
    }

    #[test]
    fn 字体_可打印字符都有字形() {
        let mut fb = Framebuffer::new(320, 40);
        fb.clear(rgba(0, 0, 0, 255));
        let s: String = (0x20u8..=0x60u8).map(|c| c as char).collect();
        let end = font::draw_text(&mut fb, 0, 0, &s, Color::WHITE);
        assert!(end > 0);
        // 大写 A 至少点亮若干像素
        fb.clear(rgba(0, 0, 0, 255));
        font::draw_char(&mut fb, 0, 0, 'A', Color::WHITE);
        let lit = fb.data.chunks(4).filter(|p| p[0] > 0).count();
        assert!(lit >= 8, "A 点亮像素过少: {}", lit);
    }

    #[test]
    fn 相机_跟随并夹取边界() {
        let scene = WhiteBoxScene::default();
        // 从 0 开始追 x=800：单帧最多 32px
        let c = Camera::follow(&scene, 800, 0, 32);
        assert_eq!(c, 32);
        // 夹取到右边界
        let c2 = Camera::follow(&scene, 100_000, 0, 100_000);
        assert_eq!(c2, scene.x_max - scene.view_w);
        // 夹取到左边界
        let c3 = Camera::follow(&scene, -100_000, 500, 100_000);
        assert_eq!(c3, scene.x_min);
    }

    #[test]
    fn 场景_可完整绘制() {
        let scene = WhiteBoxScene::default();
        let mut fb = Framebuffer::new(640, 480);
        let cam = Camera { x: 100 };
        scene.draw_background(&mut fb, &cam);
        scene.draw_actor_whitebox(&mut fb, &cam, (400, 330), 60.0, -1, 0.3, "JUMP");
        scene.draw_hud(&mut fb, "JUMP", (400, 330), 60.0, 3, 16, "TEST");
        // 角色躯干处应被绘制（白色）
        let c = fb.get(300, 300);
        assert!(c.a == 255);
    }
}
