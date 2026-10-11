//! Linear filtering; physical DPI enlargement uses a reusable fixed-point plan.
#[derive(Clone, Copy)]
struct Tap {
    first: usize,
    second: usize,
    weight: u32,
}
fn taps(source: usize, dest: usize) -> Vec<Tap> {
    (0..dest)
        .map(|p| {
            let value = ((p as f64 + 0.5) * source as f64 / dest as f64 - 0.5)
                .clamp(0.0, (source - 1) as f64);
            let first = value.floor() as usize;
            Tap {
                first,
                second: (first + 1).min(source - 1),
                weight: ((value - first as f64) * 256.0).round() as u32,
            }
        })
        .collect()
}
pub struct LinearScaler {
    pub source: (usize, usize),
    pub dest: (usize, usize),
    x: Vec<Tap>,
    y: Vec<Tap>,
}
impl LinearScaler {
    pub fn new(source: (usize, usize), dest: (usize, usize)) -> Self {
        assert!(source.0 > 0 && source.1 > 0 && dest.0 > 0 && dest.1 > 0);
        Self {
            source,
            dest,
            x: taps(source.0, dest.0),
            y: taps(source.1, dest.1),
        }
    }
    /// Channels are arbitrary: this works for RGBA frames and GDI BGRA buffers alike.
    pub fn render(&self, src: &[u8], out: &mut Vec<u8>) {
        assert!(src.len() >= self.source.0 * self.source.1 * 4);
        if self.source == self.dest {
            out.clear();
            out.extend_from_slice(src);
            return;
        }
        out.resize(self.dest.0 * self.dest.1 * 4, 0);
        for (row, y) in self.y.iter().enumerate() {
            let row0 = y.first * self.source.0 * 4;
            let row1 = y.second * self.source.0 * 4;
            for (col, x) in self.x.iter().enumerate() {
                let a = row0 + x.first * 4;
                let b = row0 + x.second * 4;
                let c = row1 + x.first * 4;
                let d = row1 + x.second * 4;
                let at = (row * self.dest.0 + col) * 4;
                for channel in 0..4 {
                    let top = src[a + channel] as u32 * (256 - x.weight)
                        + src[b + channel] as u32 * x.weight;
                    let bottom = src[c + channel] as u32 * (256 - x.weight)
                        + src[d + channel] as u32 * x.weight;
                    out[at + channel] =
                        ((top * (256 - y.weight) + bottom * y.weight + 32768) >> 16) as u8;
                }
            }
        }
    }
}
/// D3DXSprite Begin67F1C6 sets texture stage MAG/MINFILTER=LINEAR (2).
pub fn sample_linear(src: &[u8], size: (usize, usize), u: f32, v: f32) -> [u8; 4] {
    let (w, h) = size;
    let u = u.clamp(0.0, (w - 1) as f32);
    let v = v.clamp(0.0, (h - 1) as f32);
    let x = u.floor() as usize;
    let y = v.floor() as usize;
    let fx = u - x as f32;
    let fy = v - y as f32;
    let positions = [(x, y), (x + 1, y), (x, y + 1), (x + 1, y + 1)];
    let weights = [
        (1.0 - fx) * (1.0 - fy),
        fx * (1.0 - fy),
        (1.0 - fx) * fy,
        fx * fy,
    ];
    std::array::from_fn(|channel| {
        positions
            .iter()
            .zip(weights)
            .map(|(&(x, y), weight)| {
                src[(y.min(h - 1) * w + x.min(w - 1)) * 4 + channel] as f32 * weight
            })
            .sum::<f32>()
            .round() as u8
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn linear_enlargement_interpolates_and_identity_is_exact() {
        let pixels = [0, 0, 0, 255, 255, 255, 255, 255];
        let mut out = Vec::new();
        LinearScaler::new((2, 1), (4, 1)).render(&pixels, &mut out);
        assert_eq!(
            out.chunks_exact(4).map(|p| p[0]).collect::<Vec<_>>(),
            vec![0, 64, 191, 255]
        );
        LinearScaler::new((2, 1), (2, 1)).render(&pixels, &mut out);
        assert_eq!(out, pixels);
        assert_eq!(
            sample_linear(&pixels, (2, 1), 0.5, 0.0),
            [128, 128, 128, 255]
        );
    }
}
