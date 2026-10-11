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

impl Default for Framebuffer {
    fn default() -> Self {
        Self::new(WIDTH, HEIGHT)
    }
}

impl Framebuffer {
    /// 4916A0 -> 424760 rotates/scales about the actor anchor (pivot=-IMAGE POS).
    pub fn blit_transformed(&mut self, src: &[u8], size: (usize, usize), t: SpriteTransform) {
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
                self.blend(
                    x,
                    y,
                    rgba(
                        (pixel[0] as u32 * t.tint.r as u32 / 255) as u8,
                        (pixel[1] as u32 * t.tint.g as u32 / 255) as u8,
                        (pixel[2] as u32 * t.tint.b as u32 / 255) as u8,
                        (pixel[3] as u32 * t.tint.a as u32 / 255) as u8,
                    ),
                );
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
        if w <= 0 || h <= 0 {
            return;
        }
        let x0 = x.max(0);
        let y0 = y.max(0);
        let x1 = (x + w).min(self.width as i32);
        let y1 = (y + h).min(self.height as i32);
        for yy in y0..y1 {
            for xx in x0..x1 {
                self.blend(xx, yy, c);
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
        for sy in 0..src_h {
            let ty = dy + sy as i32;
            if ty < 0 || ty >= self.height as i32 {
                continue;
            }
            for sx in 0..src_w {
                let tx = dx + sx as i32;
                if tx < 0 || tx >= self.width as i32 {
                    continue;
                }
                let sxi = if flip_x { src_w - 1 - sx } else { sx };
                let si = (sy * src_w + sxi) * 4;
                let mut c = Color {
                    r: src[si],
                    g: src[si + 1],
                    b: src[si + 2],
                    a: src[si + 3],
                };
                if c.a == 0 {
                    continue;
                }
                if tint != Color::WHITE {
                    c.r = ((c.r as u32 * tint.r as u32) / 255) as u8;
                    c.g = ((c.g as u32 * tint.g as u32) / 255) as u8;
                    c.b = ((c.b as u32 * tint.b as u32) / 255) as u8;
                    c.a = ((c.a as u32 * tint.a as u32) / 255) as u8;
                }
                self.blend(tx, ty, c);
            }
        }
    }
}
