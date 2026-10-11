//! 4D7960 builds one five-frame animation per digit, not five simultaneous ghosts.
use dnf_data::img::ImgFile;
use dnf_render::framebuffer::{rgba, Framebuffer, SpriteTransform};

#[derive(Clone, Debug)]
pub struct DamagePopup {
    pub damage: i32,
    pub point: [f32; 3],
    pub age_ms: f64,
    /// 0=normal yellow,1=critical red,2=recovery,3=MP,4=player damage,5=additional gray-white.
    pub style: i32,
}
/// scale, vertical displacement, alpha, monochrome first-frame flash.
pub fn pose(age: f64, style: i32) -> Option<(f32, i32, u8, bool)> {
    if !(0.0..500.0).contains(&age) {
        return None;
    }
    let small = style == 4;
    let scale = if small {
        1.4
    } else if age < 100.0 {
        2.0 - age as f32 / 100.0
    } else {
        1.0
    };
    let (y, alpha) = if age < 300.0 {
        (0, 255)
    } else if age < 400.0 {
        let t = (age - 300.0) / 100.0;
        ((-15.0 * t) as i32, (255.0 - 127.0 * t) as u8)
    } else {
        let t = (age - 400.0) / 100.0;
        ((-15.0 - 15.0 * t) as i32, (128.0 - 128.0 * t) as u8)
    };
    Some((scale, y, alpha, age < 100.0 && (style == 1 || small)))
}
/// Semantic styles are separate from the raw 4D7960 argument/ownership flags.
pub fn glyph_base(style: i32) -> usize {
    match style {
        1 => 88,
        2 => 64,
        3 => 74,
        4 => 54,
        5 => 44,
        _ => 143,
    }
}
impl DamagePopup {
    pub fn draw(&self, atlas: &ImgFile, fb: &mut Framebuffer, camera: i32) {
        let Some((scale, dy, alpha, flash)) = pose(self.age_ms, self.style) else {
            return;
        };
        let base = glyph_base(self.style);
        let digits = self.damage.unsigned_abs().to_string();
        let glyphs: Vec<_> = digits
            .bytes()
            .filter_map(|n| atlas.frames.get(base + (n - b'0') as usize))
            .collect();
        let width: i32 =
            glyphs.iter().map(|g| g.width as i32).sum::<i32>() - 5 * (glyphs.len() as i32 - 1);
        let mut x = width / -2;
        let anchor = (
            self.point[0] as i32 - camera,
            (self.point[1] - self.point[2] - 50.0) as i32,
        );
        for g in glyphs {
            let white;
            let pixels = if flash {
                white = g
                    .rgba
                    .chunks_exact(4)
                    .flat_map(|p| [255, 255, 255, p[3]])
                    .collect::<Vec<_>>();
                &white
            } else {
                &g.rgba
            };
            fb.blit_transformed(
                pixels,
                (g.width as usize, g.height as usize),
                SpriteTransform {
                    anchor,
                    offset: (x, -(g.height as i32) / 2 + dy),
                    rate: (scale, scale),
                    angle: 0.0,
                    tint: rgba(255, 255, 255, alpha),
                },
            );
            x += g.width as i32 - 5;
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_five_frame_timeline_and_variant_scale() {
        assert_eq!(pose(0.0, 0), Some((2.0, 0, 255, false)));
        assert_eq!(pose(100.0, 0), Some((1.0, 0, 255, false)));
        assert_eq!(pose(299.0, 0), Some((1.0, 0, 255, false)));
        assert_eq!(pose(400.0, 0), Some((1.0, -15, 128, false)));
        assert_eq!(pose(500.0, 0), None);
        assert_eq!(pose(150.0, 4), Some((1.4, 0, 255, false)));
    }
    #[test]
    fn normal_additional_and_critical_use_original_color_glyphs() {
        assert_eq!(glyph_base(0), 143);
        assert_eq!(glyph_base(5), 44);
        assert_eq!(glyph_base(1), 88);
        let Some(dir) = dnf_data::locate_client_dir() else {
            return;
        };
        let set = dnf_data::npk::NpkSet::scan_dir(&dnf_data::imagepacks_dir(&dir)).unwrap();
        let atlas =
            dnf_data::img::parse(&set.read("sprite/common/etc/combo.img").unwrap()).unwrap();
        for (style, predicate) in [(0, 0), (1, 1), (5, 2)] {
            let pixels = &atlas.frames[glyph_base(style)].rgba;
            assert!(pixels
                .chunks_exact(4)
                .filter(|p| p[3] > 200)
                .any(|p| match predicate {
                    0 => p[0] > 180 && p[1] > 150 && p[2] < 80,
                    1 => p[0] > 180 && p[1] < 100 && p[2] < 100,
                    _ => p[0] > 180 && p[0] == p[1] && p[1] == p[2],
                }));
            assert!(!pose(100.0, style).unwrap().3);
        }
    }
}
