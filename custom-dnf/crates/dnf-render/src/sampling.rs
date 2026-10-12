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
        #[cfg(target_arch = "x86_64")]
        if std::is_x86_feature_detected!("sse4.1") {
            // Guarded by CPU detection; all tap offsets and output bounds are
            // established by new() and the source-length assertion above.
            unsafe {
                self.render_sse41(src, out);
            }
            return;
        }
        self.render_scalar(src, out);
    }
    fn render_scalar(&self, src: &[u8], out: &mut [u8]) {
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

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "sse4.1")]
    unsafe fn render_sse41(&self, src: &[u8], out: &mut [u8]) {
        use std::arch::x86_64::*;
        let zero = _mm_setzero_si128();
        let rounding = _mm_set1_epi32(32768);
        for (row, y) in self.y.iter().enumerate() {
            let row0 = y.first * self.source.0 * 4;
            let row1 = y.second * self.source.0 * 4;
            let wy0 = _mm_set1_epi32((256 - y.weight) as i32);
            let wy1 = _mm_set1_epi32(y.weight as i32);
            for (col, x) in self.x.iter().enumerate() {
                let first = x.first * 4;
                let second = x.second * 4;
                let a = _mm_cvtepu8_epi32(_mm_cvtsi32_si128(std::ptr::read_unaligned(
                    src.as_ptr().add(row0 + first).cast::<i32>(),
                )));
                let b = _mm_cvtepu8_epi32(_mm_cvtsi32_si128(std::ptr::read_unaligned(
                    src.as_ptr().add(row0 + second).cast::<i32>(),
                )));
                let c = _mm_cvtepu8_epi32(_mm_cvtsi32_si128(std::ptr::read_unaligned(
                    src.as_ptr().add(row1 + first).cast::<i32>(),
                )));
                let d = _mm_cvtepu8_epi32(_mm_cvtsi32_si128(std::ptr::read_unaligned(
                    src.as_ptr().add(row1 + second).cast::<i32>(),
                )));
                let wx0 = _mm_set1_epi32((256 - x.weight) as i32);
                let wx1 = _mm_set1_epi32(x.weight as i32);
                let top = _mm_add_epi32(_mm_mullo_epi32(a, wx0), _mm_mullo_epi32(b, wx1));
                let bottom = _mm_add_epi32(_mm_mullo_epi32(c, wx0), _mm_mullo_epi32(d, wx1));
                let sum = _mm_add_epi32(
                    _mm_add_epi32(_mm_mullo_epi32(top, wy0), _mm_mullo_epi32(bottom, wy1)),
                    rounding,
                );
                let values = _mm_srli_epi32::<16>(sum);
                let packed = _mm_packus_epi16(_mm_packs_epi32(values, zero), zero);
                std::ptr::write_unaligned(
                    out.as_mut_ptr()
                        .add((row * self.dest.0 + col) * 4)
                        .cast::<i32>(),
                    _mm_cvtsi128_si32(packed),
                );
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
    fn optimized_scaling_is_byte_exact_for_alpha_edges_and_arbitrary_sizes() {
        for (source, dest) in [
            ((7, 5), (11, 9)),
            ((7, 5), (3, 2)),
            ((1, 3), (8, 7)),
            ((640, 480), (800, 600)),
        ] {
            let pixels: Vec<u8> = (0..source.0 * source.1 * 4)
                .map(|i| ((i * 37 + i / 13) % 256) as u8)
                .collect();
            let plan = LinearScaler::new(source, dest);
            let mut reference = vec![0; dest.0 * dest.1 * 4];
            plan.render_scalar(&pixels, &mut reference);
            let mut actual = Vec::new();
            plan.render(&pixels, &mut actual);
            assert_eq!(actual, reference, "{source:?} -> {dest:?}");
        }
    }
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
