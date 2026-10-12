//! RGBA8 软件帧缓冲 + 图元 + alpha 混合。
//!
//! 官方内部渲染分辨率是 **640×480**（docs/90 会话 12：视差背景图宽恰为 640、
//! 瓦片条画布高恰为 480），本帧缓冲以该尺寸为逻辑分辨率。
//!
//! 白模阶段只需要矩形/线段/椭圆等图元；后续接真实素材时用 [`Framebuffer::blit_alpha`]
//! 贴 IMG 帧（官方画瓦片图必须 alpha 混合，否则会把背景整片擦掉 —— docs/90 会话 13 第 6 条）。

/// Official 420190 explicitly calls 404330(640,480), then 407C40(640,480,16,...).
/// This is the game canvas/client resolution, not the generic 800x600 default.
pub const WIDTH: usize = 640;
pub const HEIGHT: usize = 480;

#[derive(Clone, Copy, Debug)]
pub struct SpriteTransform {
    pub anchor: (i32, i32),
    pub offset: (i32, i32),
    pub rate: (f32, f32),
    pub angle: f32,
    pub tint: Color,
}

#[cfg(test)]
mod silhouette_tests {
    use super::*;
    #[test]
    fn vector_white_rows_preserve_integer_rounding_destination_alpha_and_short_tails() {
        for count in [1, 3, 4, 7, 8, 19, 83] {
            for alpha in [0, 1, 17, 128, 150, 254, 255] {
                for additive in [false, true] {
                    let src: Vec<u8> = (0..count * 4)
                        .map(|i| ((i * 73 + i / 5) % 256) as u8)
                        .collect();
                    let mut actual: Vec<u8> = (0..count * 4)
                        .map(|i| ((i * 41 + i / 3) % 256) as u8)
                        .collect();
                    let mut expected = actual.clone();
                    blend_white_row(&mut actual, &src, alpha, additive);
                    for (d, p) in expected.chunks_exact_mut(4).zip(src.chunks_exact(4)) {
                        let a = (p[3] as u32 * alpha as u32 / 255) as u8;
                        if additive {
                            if a > 0 {
                                for i in 0..3 {
                                    d[i] =
                                        (d[i] as u32 + p[i] as u32 * a as u32 / 255).min(255) as u8;
                                }
                                d[3] = 255;
                            }
                        } else {
                            blend_slice(d, p[0], p[1], p[2], a);
                        }
                    }
                    assert_eq!(
                        actual, expected,
                        "count={count} alpha={alpha} additive={additive}"
                    );
                }
            }
        }
    }
    #[test]
    fn prepared_geometry_matches_live_linear_sampling_at_clipped_integer_anchors() {
        let src: Vec<u8> = (0..9 * 7 * 4)
            .map(|i| ((i * 37 + i / 6) % 256) as u8)
            .collect();
        for rate in [
            (0.45, 0.45),
            (0.4, 0.1),
            (1.7, 1.7),
            (-1.0, 1.0),
            (0.6, 0.8),
        ] {
            for angle in [0.0, 0.3, 1.2] {
                for offset in [(-4, -7), (2, 3)] {
                    let prepared = prepare_sprite(&src, (9, 7), offset, rate, angle);
                    for anchor in [(-2, -5), (0, 0), (3, 6), (15, 17)] {
                        for tint in [Color::WHITE, rgba(110, 35, 200, 80)] {
                            let mut a = Framebuffer::new(23, 21);
                            a.clear(rgba(23, 50, 90, 255));
                            let mut b = a.clone();
                            let t = SpriteTransform {
                                anchor,
                                offset,
                                rate,
                                angle,
                                tint,
                            };
                            a.blit_transformed(&src, (9, 7), t);
                            b.blit_alpha(
                                &prepared.pixels,
                                prepared.width,
                                prepared.height,
                                anchor.0 + prepared.offset.0,
                                anchor.1 + prepared.offset.1,
                                false,
                                tint,
                            );
                            assert_eq!(a.data, b.data);
                            a.blit_transformed_additive(&src, (9, 7), t);
                            b.blit_additive(
                                &prepared.pixels,
                                prepared.width,
                                prepared.height,
                                anchor.0 + prepared.offset.0,
                                anchor.1 + prepared.offset.1,
                                false,
                                tint,
                            );
                            assert_eq!(a.data, b.data);
                        }
                    }
                }
            }
        }
    }
    #[test]
    fn additive_blending_clips_mirrors_saturates_and_keeps_transparent_pixels() {
        let src = [255, 30, 20, 128, 0, 255, 0, 255, 100, 50, 200, 0];
        let tint = rgba(120, 200, 255, 180);
        for x in [-2, -1, 0, 2, 4] {
            for flip in [true, false] {
                let mut fb = Framebuffer::new(4, 2);
                fb.clear(rgba(180, 40, 60, 80));
                let mut expected = fb.clone();
                fb.blit_additive(&src, 3, 1, x, 0, flip, tint);
                for px in 0..3 {
                    let dx = x + px;
                    if !(0..4).contains(&dx) {
                        continue;
                    }
                    let sx = if flip { 2 - px } else { px } as usize;
                    let p = &src[sx * 4..sx * 4 + 4];
                    let alpha = p[3] as u32 * tint.a as u32 / 255;
                    if alpha == 0 {
                        continue;
                    }
                    let mut c = expected.get(dx, 0);
                    c.r = (c.r as u32 + (p[0] as u32 * tint.r as u32 / 255) * alpha / 255).min(255)
                        as u8;
                    c.g = (c.g as u32 + (p[1] as u32 * tint.g as u32 / 255) * alpha / 255).min(255)
                        as u8;
                    c.b = (c.b as u32 + (p[2] as u32 * tint.b as u32 / 255) * alpha / 255).min(255)
                        as u8;
                    c.a = 255;
                    expected.put(dx, 0, c);
                }
                assert_eq!(fb.data, expected.data);
            }
        }
    }
    #[test]
    fn rows_and_silhouettes_preserve_partial_alpha_clipping_and_overlaps() {
        let src: Vec<u8> = (0..9 * 7 * 4)
            .map(|i| ((i * 43 + i / 5) % 256) as u8)
            .collect();
        for (x, y) in [(-7, -5), (-1, 0), (0, 0), (4, 3), (10, 10)] {
            for color in [
                rgba(255, 0, 0, 150),
                rgba(255, 255, 255, 150),
                rgba(20, 50, 70, 0),
                Color::WHITE,
            ] {
                for flip in [false, true] {
                    let mut actual = Framebuffer::new(11, 9);
                    actual.clear(rgba(40, 30, 20, 100));
                    let mut reference = actual.clone();
                    for _ in 0..3 {
                        actual.blit_silhouette(&src, 9, 7, x, y, flip, color);
                        for py in 0..7 {
                            for px in 0..9 {
                                let sx = if flip { 8 - px } else { px };
                                let a = (src[(py * 9 + sx) * 4 + 3] as u32 * color.a as u32 / 255)
                                    as u8;
                                reference.blend(x + px as i32, y + py as i32, Color { a, ..color });
                            }
                        }
                    }
                    assert_eq!(actual.data, reference.data);
                    actual.fill_rect(x, y, 9, 7, color);
                    for py in 0..7 {
                        for px in 0..9 {
                            reference.blend(x + px, y + py, color);
                        }
                    }
                    assert_eq!(actual.data, reference.data);
                }
            }
        }
    }
    #[test]
    fn clipped_row_blits_match_pixel_reference_for_mirroring_tint_and_partial_alpha() {
        let src: Vec<u8> = (0..9 * 7 * 4)
            .map(|i| ((i * 43 + i / 5) % 256) as u8)
            .collect();
        for pos in [(-7, -5), (-1, 0), (0, 0), (4, 3), (10, 10)] {
            for flip in [false, true] {
                for tint in [Color::WHITE, rgba(120, 210, 30, 140)] {
                    let mut actual = Framebuffer::new(11, 9);
                    actual.clear(rgba(22, 70, 90, 255));
                    let mut reference = actual.clone();
                    actual.blit_alpha(&src, 9, 7, pos.0, pos.1, flip, tint);
                    for y in 0..7 {
                        for x in 0..9 {
                            let sx = if flip { 8 - x } else { x };
                            let i = (y * 9 + sx) * 4;
                            reference.blend(
                                pos.0 + x as i32,
                                pos.1 + y as i32,
                                rgba(
                                    (src[i] as u32 * tint.r as u32 / 255) as u8,
                                    (src[i + 1] as u32 * tint.g as u32 / 255) as u8,
                                    (src[i + 2] as u32 * tint.b as u32 / 255) as u8,
                                    (src[i + 3] as u32 * tint.a as u32 / 255) as u8,
                                ),
                            );
                        }
                    }
                    assert_eq!(actual.data, reference.data, "{pos:?} {flip} {tint:?}");
                }
            }
        }
    }
    #[test]
    fn black_pixels_are_colored_and_transparency_flip_and_clipping_are_preserved() {
        let src = [0, 0, 0, 255, 200, 200, 200, 0];
        let mut fb = Framebuffer::new(4, 3);
        fb.clear(Color::BLACK);
        fb.blit_silhouette(&src, 2, 1, 1, 1, false, rgba(255, 0, 0, 255));
        assert_eq!(fb.get(1, 1), rgba(255, 0, 0, 255));
        assert_eq!(fb.get(2, 1), Color::BLACK);
        fb.clear(Color::BLACK);
        fb.blit_silhouette(&src, 2, 1, -1, 1, true, rgba(255, 0, 0, 128));
        assert_eq!(fb.get(0, 1), rgba(128, 0, 0, 255));
        assert_eq!(fb.get(1, 1), Color::BLACK);
    }
}

/// RGBA8 颜色。
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
    Color { r, g, b, a }
}

impl Color {
    pub const WHITE: Color = rgba(255, 255, 255, 255);
    pub const BLACK: Color = rgba(0, 0, 0, 255);
    pub const TRANSPARENT: Color = rgba(0, 0, 0, 0);

    /// 以 `t`(0..=1) 在自身与 `other` 之间线性插值。
    pub fn lerp(self, other: Color, t: f32) -> Color {
        let t = t.clamp(0.0, 1.0);
        let f = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * t).round() as u8;
        Color {
            r: f(self.r, other.r),
            g: f(self.g, other.g),
            b: f(self.b, other.b),
            a: f(self.a, other.a),
        }
    }

    pub fn scaled(self, k: f32) -> Color {
        let f = |v: u8| (v as f32 * k).clamp(0.0, 255.0).round() as u8;
        Color {
            r: f(self.r),
            g: f(self.g),
            b: f(self.b),
            a: self.a,
        }
    }
}

/// RGBA8 帧缓冲（行优先）。
#[derive(Clone)]
pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    pub data: Vec<u8>,
}
pub struct PreparedSprite {
    pub pixels: Vec<u8>,
    pub width: usize,
    pub height: usize,
    pub offset: (i32, i32),
    pub runs: Vec<(usize, usize, usize)>,
}
impl PreparedSprite {
    pub fn from_rgba(pixels: Vec<u8>, width: usize, height: usize, offset: (i32, i32)) -> Self {
        let mut runs = Vec::new();
        for y in 0..height {
            let mut x = 0;
            while x < width {
                while x < width && pixels[(y * width + x) * 4 + 3] == 0 {
                    x += 1;
                }
                let begin = x;
                while x < width && pixels[(y * width + x) * 4 + 3] != 0 {
                    x += 1;
                }
                if begin < x {
                    runs.push((y, begin, x));
                }
            }
        }
        Self {
            pixels,
            width,
            height,
            offset,
            runs,
        }
    }
}
/// Bake geometry/linear sampling; leave tint and alpha dynamic.
pub fn prepare_sprite(
    src: &[u8],
    size: (usize, usize),
    offset: (i32, i32),
    rate: (f32, f32),
    angle: f32,
) -> PreparedSprite {
    let (w, h) = size;
    if w == 0 || h == 0 || rate.0 == 0.0 || rate.1 == 0.0 {
        return PreparedSprite {
            pixels: Vec::new(),
            width: 0,
            height: 0,
            offset: (0, 0),
            runs: Vec::new(),
        };
    }
    let (sin, cos) = angle.sin_cos();
    let transform = |x: f32, y: f32| {
        let x = x * rate.0;
        let y = y * rate.1;
        (x * cos - y * sin, x * sin + y * cos)
    };
    let corners = [
        transform(offset.0 as f32, offset.1 as f32),
        transform(offset.0 as f32 + w as f32 - 1.0, offset.1 as f32),
        transform(offset.0 as f32, offset.1 as f32 + h as f32 - 1.0),
        transform(
            offset.0 as f32 + w as f32 - 1.0,
            offset.1 as f32 + h as f32 - 1.0,
        ),
    ];
    let x0 = corners
        .iter()
        .map(|p| p.0)
        .fold(f32::INFINITY, f32::min)
        .floor() as i32;
    let x1 = corners
        .iter()
        .map(|p| p.0)
        .fold(f32::NEG_INFINITY, f32::max)
        .ceil() as i32;
    let y0 = corners
        .iter()
        .map(|p| p.1)
        .fold(f32::INFINITY, f32::min)
        .floor() as i32;
    let y1 = corners
        .iter()
        .map(|p| p.1)
        .fold(f32::NEG_INFINITY, f32::max)
        .ceil() as i32;
    let width = (x1 - x0 + 1).max(0) as usize;
    let height = (y1 - y0 + 1).max(0) as usize;
    let mut pixels = vec![0; width * height * 4];
    for y in y0..=y1 {
        for x in x0..=x1 {
            let dx = x as f32;
            let dy = y as f32;
            let u = (dx * cos + dy * sin) / rate.0 - offset.0 as f32;
            let v = (-dx * sin + dy * cos) / rate.1 - offset.1 as f32;
            if u < -0.0001
                || v < -0.0001
                || u > (w - 1) as f32 + 0.0001
                || v > (h - 1) as f32 + 0.0001
            {
                continue;
            }
            let rgba = crate::sampling::sample_linear(src, size, u, v);
            let index = ((y - y0) as usize * width + (x - x0) as usize) * 4;
            pixels[index..index + 4].copy_from_slice(&rgba);
        }
    }
    PreparedSprite::from_rgba(pixels, width, height, (x0, y0))
}

impl Default for Framebuffer {
    fn default() -> Self {
        Self::new(WIDTH, HEIGHT)
    }
}

impl Framebuffer {
    pub fn blit_prepared(
        &mut self,
        sprite: &PreparedSprite,
        anchor: (i32, i32),
        tint: Color,
        additive: bool,
    ) {
        if tint.a == 0 {
            return;
        }
        let x = anchor.0 + sprite.offset.0;
        let y = anchor.1 + sprite.offset.1;
        let white = tint.r == 255 && tint.g == 255 && tint.b == 255;
        for &(row, start, end) in &sprite.runs {
            let dy = y + row as i32;
            if dy < 0 || dy >= self.height as i32 {
                continue;
            }
            let begin = start.max((-i64::from(x)).max(0) as usize);
            let end = end.min((self.width as i64 - x as i64).max(0) as usize);
            if begin >= end {
                continue;
            }
            let source =
                &sprite.pixels[(row * sprite.width + begin) * 4..(row * sprite.width + end) * 4];
            let base = (dy as usize * self.width + (x as i64 + begin as i64) as usize) * 4;
            let target = &mut self.data[base..base + (end - begin) * 4];
            if white {
                blend_white_row(target, source, tint.a, additive);
            } else {
                for (d, p) in target.chunks_exact_mut(4).zip(source.chunks_exact(4)) {
                    let a = p[3] as u32 * tint.a as u32 / 255;
                    let r = p[0] as u32 * tint.r as u32 / 255;
                    let g = p[1] as u32 * tint.g as u32 / 255;
                    let b = p[2] as u32 * tint.b as u32 / 255;
                    if additive {
                        d[0] = (d[0] as u32 + r * a / 255).min(255) as u8;
                        d[1] = (d[1] as u32 + g * a / 255).min(255) as u8;
                        d[2] = (d[2] as u32 + b * a / 255).min(255) as u8;
                        if a > 0 {
                            d[3] = 255;
                        }
                    } else {
                        blend_slice(d, r as u8, g as u8, b as u8, a as u8);
                    }
                }
            }
        }
    }
    /// LINEARDODGE: source-alpha contribution plus destination, per channel.
    pub fn blit_additive(
        &mut self,
        src: &[u8],
        w: usize,
        h: usize,
        x: i32,
        y: i32,
        flip: bool,
        tint: Color,
    ) {
        if src.len() < w.saturating_mul(h).saturating_mul(4) || tint.a == 0 {
            return;
        }
        let x0 = (-i64::from(x)).max(0).min(w as i64) as usize;
        let y0 = (-i64::from(y)).max(0).min(h as i64) as usize;
        let x1 = (self.width as i64 - x as i64).max(0).min(w as i64) as usize;
        let y1 = (self.height as i64 - y as i64).max(0).min(h as i64) as usize;
        if x0 >= x1 || y0 >= y1 {
            return;
        }
        for py in y0..y1 {
            let begin = ((y as i64 + py as i64) as usize * self.width
                + (x as i64 + x0 as i64) as usize)
                * 4;
            for (i, dst) in self.data[begin..begin + (x1 - x0) * 4]
                .chunks_exact_mut(4)
                .enumerate()
            {
                let sx = if flip { w - 1 - x0 - i } else { x0 + i };
                let p = &src[(py * w + sx) * 4..(py * w + sx) * 4 + 4];
                let alpha = p[3] as u32 * tint.a as u32 / 255;
                if alpha == 0 {
                    continue;
                }
                for (ch, c) in [tint.r, tint.g, tint.b].into_iter().enumerate() {
                    let v = p[ch] as u32 * c as u32 / 255;
                    dst[ch] = (dst[ch] as u32 + v * alpha / 255).min(255) as u8;
                }
                dst[3] = 255;
            }
        }
    }
    /// Flat-color sprite copy: preserve source alpha, replace RGB (not a tint).
    /// Used for protection outlines; black pixels must still produce a colored edge.
    pub fn blit_silhouette(
        &mut self,
        src: &[u8],
        w: usize,
        h: usize,
        x: i32,
        y: i32,
        flip: bool,
        color: Color,
    ) {
        if src.len() < w.saturating_mul(h).saturating_mul(4) {
            return;
        }
        let x0 = (-i64::from(x)).max(0).min(w as i64) as usize;
        let y0 = (-i64::from(y)).max(0).min(h as i64) as usize;
        let x1 = (self.width as i64 - i64::from(x)).max(0).min(w as i64) as usize;
        let y1 = (self.height as i64 - i64::from(y)).max(0).min(h as i64) as usize;
        if x0 >= x1 || y0 >= y1 || color.a == 0 {
            return;
        }
        for py in y0..y1 {
            let begin = ((y as i64 + py as i64) as usize * self.width
                + (x as i64 + x0 as i64) as usize)
                * 4;
            let row = &src[py * w * 4..(py + 1) * w * 4];
            let dst = &mut self.data[begin..begin + (x1 - x0) * 4];
            for (i, target) in dst.chunks_exact_mut(4).enumerate() {
                let sx = if flip { w - 1 - (x0 + i) } else { x0 + i };
                let alpha = (row[sx * 4 + 3] as u32 * color.a as u32 / 255) as u8;
                blend_slice(target, color.r, color.g, color.b, alpha);
            }
        }
    }

    /// 4916A0 -> 424760 rotates/scales about the actor anchor (pivot=-IMAGE POS).
    pub fn blit_transformed(&mut self, src: &[u8], size: (usize, usize), t: SpriteTransform) {
        self.blit_transform_mode(src, size, t, false);
    }
    pub fn blit_transformed_additive(
        &mut self,
        src: &[u8],
        size: (usize, usize),
        t: SpriteTransform,
    ) {
        self.blit_transform_mode(src, size, t, true);
    }
    fn blit_transform_mode(
        &mut self,
        src: &[u8],
        size: (usize, usize),
        t: SpriteTransform,
        additive: bool,
    ) {
        let (w, h) = size;
        if w == 0 || h == 0 || src.len() < w * h * 4 || t.rate.0 == 0.0 || t.rate.1 == 0.0 {
            return;
        }
        if t.angle == 0.0 && t.rate.1 == 1.0 && t.rate.0.abs() == 1.0 {
            let flip = t.rate.0 < 0.0;
            let x = if flip {
                -(t.offset.0 + w as i32 - 1)
            } else {
                t.offset.0
            };
            if additive {
                self.blit_additive(
                    src,
                    w,
                    h,
                    t.anchor.0 + x,
                    t.anchor.1 + t.offset.1,
                    flip,
                    t.tint,
                );
                return;
            }
            self.blit_alpha(
                src,
                w,
                h,
                t.anchor.0 + x,
                t.anchor.1 + t.offset.1,
                flip,
                t.tint,
            );
            return;
        }
        let (sin, cos) = t.angle.sin_cos();
        let transform = |x: f32, y: f32| {
            let x = x * t.rate.0;
            let y = y * t.rate.1;
            (
                x * cos - y * sin + t.anchor.0 as f32,
                x * sin + y * cos + t.anchor.1 as f32,
            )
        };
        let points = [
            transform(t.offset.0 as f32, t.offset.1 as f32),
            transform(t.offset.0 as f32 + w as f32 - 1.0, t.offset.1 as f32),
            transform(t.offset.0 as f32, t.offset.1 as f32 + h as f32 - 1.0),
            transform(
                t.offset.0 as f32 + w as f32 - 1.0,
                t.offset.1 as f32 + h as f32 - 1.0,
            ),
        ];
        let x0 = points
            .iter()
            .map(|p| p.0)
            .fold(f32::INFINITY, f32::min)
            .floor()
            .max(0.0) as i32;
        let x1 = points
            .iter()
            .map(|p| p.0)
            .fold(f32::NEG_INFINITY, f32::max)
            .ceil()
            .min(self.width as f32 - 1.0) as i32;
        let y0 = points
            .iter()
            .map(|p| p.1)
            .fold(f32::INFINITY, f32::min)
            .floor()
            .max(0.0) as i32;
        let y1 = points
            .iter()
            .map(|p| p.1)
            .fold(f32::NEG_INFINITY, f32::max)
            .ceil()
            .min(self.height as f32 - 1.0) as i32;
        for y in y0..=y1 {
            for x in x0..=x1 {
                let dx = (x - t.anchor.0) as f32;
                let dy = (y - t.anchor.1) as f32;
                let u = (dx * cos + dy * sin) / t.rate.0 - t.offset.0 as f32;
                let v = (-dx * sin + dy * cos) / t.rate.1 - t.offset.1 as f32;
                if u < -0.0001
                    || v < -0.0001
                    || u > (w - 1) as f32 + 0.0001
                    || v > (h - 1) as f32 + 0.0001
                {
                    continue;
                }
                let pixel = crate::sampling::sample_linear(src, (w, h), u, v);
                let color = rgba(
                    (pixel[0] as u32 * t.tint.r as u32 / 255) as u8,
                    (pixel[1] as u32 * t.tint.g as u32 / 255) as u8,
                    (pixel[2] as u32 * t.tint.b as u32 / 255) as u8,
                    (pixel[3] as u32 * t.tint.a as u32 / 255) as u8,
                );
                if additive {
                    if color.a > 0 {
                        let i = self.idx(x as usize, y as usize);
                        let a = color.a as u32;
                        self.data[i] =
                            (self.data[i] as u32 + color.r as u32 * a / 255).min(255) as u8;
                        self.data[i + 1] =
                            (self.data[i + 1] as u32 + color.g as u32 * a / 255).min(255) as u8;
                        self.data[i + 2] =
                            (self.data[i + 2] as u32 + color.b as u32 * a / 255).min(255) as u8;
                        self.data[i + 3] = 255;
                    }
                } else {
                    self.blend(x, y, color);
                }
            }
        }
    }
    pub fn new(width: usize, height: usize) -> Self {
        Framebuffer {
            width,
            height,
            data: vec![0; width * height * 4],
        }
    }

    #[inline]
    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height
    }

    #[inline]
    fn idx(&self, x: usize, y: usize) -> usize {
        (y * self.width + x) * 4
    }

    pub fn clear(&mut self, c: Color) {
        for px in self.data.chunks_exact_mut(4) {
            px[0] = c.r;
            px[1] = c.g;
            px[2] = c.b;
            px[3] = c.a;
        }
    }

    /// 直接写入（替换，不混合）。
    #[inline]
    pub fn put(&mut self, x: i32, y: i32, c: Color) {
        if !self.in_bounds(x, y) {
            return;
        }
        let i = self.idx(x as usize, y as usize);
        self.data[i] = c.r;
        self.data[i + 1] = c.g;
        self.data[i + 2] = c.b;
        self.data[i + 3] = c.a;
    }

    /// 取像素。
    #[inline]
    pub fn get(&self, x: i32, y: i32) -> Color {
        if !self.in_bounds(x, y) {
            return Color::TRANSPARENT;
        }
        let i = self.idx(x as usize, y as usize);
        Color {
            r: self.data[i],
            g: self.data[i + 1],
            b: self.data[i + 2],
            a: self.data[i + 3],
        }
    }

    /// alpha 混合写入（dst = src*a + dst*(1-a)）。
    #[inline]
    pub fn blend(&mut self, x: i32, y: i32, c: Color) {
        if c.a == 0 || !self.in_bounds(x, y) {
            return;
        }
        if c.a == 255 {
            self.put(x, y, c);
            return;
        }
        let i = self.idx(x as usize, y as usize);
        let a = c.a as u32;
        let inv = 255 - a;
        self.data[i] = ((c.r as u32 * a + self.data[i] as u32 * inv) / 255) as u8;
        self.data[i + 1] = ((c.g as u32 * a + self.data[i + 1] as u32 * inv) / 255) as u8;
        self.data[i + 2] = ((c.b as u32 * a + self.data[i + 2] as u32 * inv) / 255) as u8;
        self.data[i + 3] = 255;
    }

    pub fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, c: Color) {
        if w <= 0 || h <= 0 || c.a == 0 {
            return;
        }
        let x0 = (x as i64).clamp(0, self.width as i64) as usize;
        let y0 = (y as i64).clamp(0, self.height as i64) as usize;
        let x1 = (x as i64 + w as i64).clamp(0, self.width as i64) as usize;
        let y1 = (y as i64 + h as i64).clamp(0, self.height as i64) as usize;
        if x0 >= x1 || y0 >= y1 {
            return;
        }
        for yy in y0..y1 {
            let begin = (yy * self.width + x0) * 4;
            let row = &mut self.data[begin..begin + (x1 - x0) * 4];
            if c.a == 255 {
                for target in row.chunks_exact_mut(4) {
                    target.copy_from_slice(&[c.r, c.g, c.b, 255]);
                }
            } else {
                let a = c.a as u32;
                let inv = 255 - a;
                let (r, g, b) = (c.r as u32 * a, c.g as u32 * a, c.b as u32 * a);
                for target in row.chunks_exact_mut(4) {
                    target[0] = ((r + target[0] as u32 * inv) / 255) as u8;
                    target[1] = ((g + target[1] as u32 * inv) / 255) as u8;
                    target[2] = ((b + target[2] as u32 * inv) / 255) as u8;
                    target[3] = 255;
                }
            }
        }
    }

    pub fn rect_outline(&mut self, x: i32, y: i32, w: i32, h: i32, c: Color) {
        if w <= 0 || h <= 0 {
            return;
        }
        self.fill_rect(x, y, w, 1, c);
        self.fill_rect(x, y + h - 1, w, 1, c);
        self.fill_rect(x, y, 1, h, c);
        self.fill_rect(x + w - 1, y, 1, h, c);
    }

    pub fn hline(&mut self, x0: i32, x1: i32, y: i32, c: Color) {
        let (a, b) = if x0 <= x1 { (x0, x1) } else { (x1, x0) };
        self.fill_rect(a, y, b - a + 1, 1, c);
    }

    pub fn vline(&mut self, x: i32, y0: i32, y1: i32, c: Color) {
        let (a, b) = if y0 <= y1 { (y0, y1) } else { (y1, y0) };
        self.fill_rect(x, a, 1, b - a + 1, c);
    }

    /// 细线段（Bresenham）。
    pub fn line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, c: Color) {
        let (mut x, mut y) = (x0, y0);
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        loop {
            self.blend(x, y, c);
            if x == x1 && y == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }

    /// 实心椭圆（用于脚下的影子）。
    pub fn fill_ellipse(&mut self, cx: i32, cy: i32, rx: i32, ry: i32, c: Color) {
        if rx <= 0 || ry <= 0 {
            return;
        }
        for dy in -ry..=ry {
            let t = dy as f32 / ry as f32;
            let w = ((1.0 - t * t).max(0.0)).sqrt() * rx as f32;
            let w = w.round() as i32;
            self.fill_rect(cx - w, cy + dy, w * 2 + 1, 1, c);
        }
    }

    /// 贴一张 RGBA8 图（alpha 混合；整行全不透明走快路径 —— 对标官方 `blit_alpha`）。
    #[allow(clippy::too_many_arguments)]
    pub fn blit_alpha(
        &mut self,
        src: &[u8],
        src_w: usize,
        src_h: usize,
        dx: i32,
        dy: i32,
        flip_x: bool,
        tint: Color,
    ) {
        if src.len() < src_w.saturating_mul(src_h).saturating_mul(4) {
            return;
        }
        let x0 = (-i64::from(dx)).max(0).min(src_w as i64) as usize;
        let y0 = (-i64::from(dy)).max(0).min(src_h as i64) as usize;
        let x1 = (self.width as i64 - i64::from(dx)).max(0).min(src_w as i64) as usize;
        let y1 = (self.height as i64 - i64::from(dy))
            .max(0)
            .min(src_h as i64) as usize;
        if x0 >= x1 || y0 >= y1 {
            return;
        }
        for sy in y0..y1 {
            let begin = ((dy as i64 + sy as i64) as usize * self.width
                + (dx as i64 + x0 as i64) as usize)
                * 4;
            let dst = &mut self.data[begin..begin + (x1 - x0) * 4];
            let row = &src[sy * src_w * 4..(sy + 1) * src_w * 4];
            if !flip_x && tint == Color::WHITE {
                for (target, pixel) in dst
                    .chunks_exact_mut(4)
                    .zip(row[x0 * 4..x1 * 4].chunks_exact(4))
                {
                    blend_slice(target, pixel[0], pixel[1], pixel[2], pixel[3]);
                }
            } else {
                for (i, target) in dst.chunks_exact_mut(4).enumerate() {
                    let sx = if flip_x { src_w - 1 - (x0 + i) } else { x0 + i };
                    let p = &row[sx * 4..sx * 4 + 4];
                    let alpha = (p[3] as u32 * tint.a as u32 / 255) as u8;
                    if alpha == 0 {
                        continue;
                    }
                    blend_slice(
                        target,
                        (p[0] as u32 * tint.r as u32 / 255) as u8,
                        (p[1] as u32 * tint.g as u32 / 255) as u8,
                        (p[2] as u32 * tint.b as u32 / 255) as u8,
                        alpha,
                    );
                }
            }
        }
    }
}

#[inline]
fn blend_slice(dst: &mut [u8], r: u8, g: u8, b: u8, alpha: u8) {
    if alpha == 0 {
        return;
    }
    if alpha == 255 {
        dst.copy_from_slice(&[r, g, b, 255]);
        return;
    }
    let a = alpha as u32;
    let inv = 255 - a;
    dst[0] = ((r as u32 * a + dst[0] as u32 * inv) / 255) as u8;
    dst[1] = ((g as u32 * a + dst[1] as u32 * inv) / 255) as u8;
    dst[2] = ((b as u32 * a + dst[2] as u32 * inv) / 255) as u8;
    dst[3] = 255;
}
fn blend_white_row(dst: &mut [u8], src: &[u8], alpha: u8, additive: bool) {
    let mut done = 0;
    #[cfg(target_arch = "x86_64")]
    if std::arch::is_x86_feature_detected!("sse4.1") {
        done = unsafe { blend_white_sse(dst, src, alpha, additive) };
    }
    for (d, p) in dst[done..]
        .chunks_exact_mut(4)
        .zip(src[done..].chunks_exact(4))
    {
        let a = (p[3] as u32 * alpha as u32 / 255) as u8;
        if additive {
            if a > 0 {
                d[0] = (d[0] as u32 + p[0] as u32 * a as u32 / 255).min(255) as u8;
                d[1] = (d[1] as u32 + p[1] as u32 * a as u32 / 255).min(255) as u8;
                d[2] = (d[2] as u32 + p[2] as u32 * a as u32 / 255).min(255) as u8;
                d[3] = 255;
            }
        } else {
            blend_slice(d, p[0], p[1], p[2], a);
        }
    }
}
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse4.1")]
unsafe fn blend_white_sse(dst: &mut [u8], src: &[u8], alpha: u8, additive: bool) -> usize {
    use std::arch::x86_64::*;
    let limit = dst.len().min(src.len()) / 16 * 16;
    let one = _mm_set1_epi16(1);
    let max = _mm_set1_epi16(255);
    let scale = _mm_set1_epi16(alpha as i16);
    let shuffle = _mm_setr_epi8(3, 3, 3, 3, 7, 7, 7, 7, 11, 11, 11, 11, 15, 15, 15, 15);
    let mask = _mm_setr_epi8(0, 0, 0, -1, 0, 0, 0, -1, 0, 0, 0, -1, 0, 0, 0, -1);
    let divide = |x: __m128i| {
        _mm_srli_epi16(
            _mm_add_epi16(_mm_add_epi16(x, one), _mm_srli_epi16(x, 8)),
            8,
        )
    };
    for i in (0..limit).step_by(16) {
        let s = _mm_loadu_si128(src.as_ptr().add(i) as *const __m128i);
        let d = _mm_loadu_si128(dst.as_ptr().add(i) as *const __m128i);
        let ab = _mm_shuffle_epi8(s, shuffle);
        let al = divide(_mm_mullo_epi16(_mm_cvtepu8_epi16(ab), scale));
        let ah = divide(_mm_mullo_epi16(
            _mm_cvtepu8_epi16(_mm_srli_si128(ab, 8)),
            scale,
        ));
        let sl = _mm_cvtepu8_epi16(s);
        let sh = _mm_cvtepu8_epi16(_mm_srli_si128(s, 8));
        let dl = _mm_cvtepu8_epi16(d);
        let dh = _mm_cvtepu8_epi16(_mm_srli_si128(d, 8));
        let out = if additive {
            _mm_adds_epu8(
                d,
                _mm_packus_epi16(
                    divide(_mm_mullo_epi16(sl, al)),
                    divide(_mm_mullo_epi16(sh, ah)),
                ),
            )
        } else {
            _mm_packus_epi16(
                divide(_mm_add_epi16(
                    _mm_mullo_epi16(sl, al),
                    _mm_mullo_epi16(dl, _mm_sub_epi16(max, al)),
                )),
                divide(_mm_add_epi16(
                    _mm_mullo_epi16(sh, ah),
                    _mm_mullo_epi16(dh, _mm_sub_epi16(max, ah)),
                )),
            )
        };
        let a = _mm_packus_epi16(al, ah);
        let force = _mm_andnot_si128(_mm_cmpeq_epi8(a, _mm_setzero_si128()), mask);
        let out = _mm_or_si128(_mm_andnot_si128(force, out), force);
        _mm_storeu_si128(dst.as_mut_ptr().add(i) as *mut __m128i, out);
    }
    limit
}
