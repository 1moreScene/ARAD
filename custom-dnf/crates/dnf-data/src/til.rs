//! `.til` 瓦片条定义。
//!
//! ```text
//! [IMAGE] `Map/01tile00.img` 0     ← 瓦片条图集（画布 224x480）
//! [pass type]
//! 2 2 2 ... (每行 14 个整数，共 30 行)
//! 0 0 0 ...
//! ```
//!
//! * 通行网格 = **14 列 × 30 行 × 16px = 224 × 480**（与官方内部渲染分辨率一致）。
//! * `0` = 可通行，`2` = 阻挡（`docs/90` 会话 12/13 实测）。
//! * 一个 `.til` 就是**一条 224px 宽的地图横条**，房间宽 = 条数 × 224。

/// 一个 `.til`。
#[derive(Clone, Debug, Default)]
pub struct TilFile {
    /// `[IMAGE]` 的图集路径（如 `Map/01tile00.img`）。
    pub image: String,
    pub image_index: i32,
    /// 通行网格（行 × 列）。
    pub pass: Vec<Vec<i32>>,
}

/// 通行网格单元尺寸（官方）。
pub const CELL: i32 = 16;
/// 每行单元数（官方 14 列）。
pub const COLS: usize = 14;

impl TilFile {
    pub fn width(&self) -> i32 {
        self.pass
            .first()
            .map(|r| r.len() as i32)
            .unwrap_or(COLS as i32)
            * CELL
    }

    pub fn height(&self) -> i32 {
        self.pass.len() as i32 * CELL
    }

    pub fn is_walkable(&self, row: usize, col: usize) -> bool {
        self.pass
            .get(row)
            .and_then(|r| r.get(col))
            .copied()
            .unwrap_or(2)
            == 0
    }

    /// 该 x 像素列（0..width）上的**可通行纵深范围**（屏幕 y 的 min..=max）。
    pub fn walk_band_at(&self, x: i32) -> Option<(i32, i32)> {
        let col = (x / CELL).clamp(0, COLS as i32 - 1) as usize;
        let mut lo = None;
        let mut hi = None;
        for (r, row) in self.pass.iter().enumerate() {
            if row.get(col).copied().unwrap_or(2) == 0 {
                let y = r as i32 * CELL;
                lo.get_or_insert(y);
                hi = Some(y + CELL - 1);
            }
        }
        match (lo, hi) {
            (Some(a), Some(b)) => Some((a, b)),
            _ => None,
        }
    }
}

/// 解析 `.til` 文本。
pub fn parse(text: &str) -> TilFile {
    let mut out = TilFile::default();
    let mut in_pass = false;
    for raw in text.lines() {
        let line = match raw.find("//") {
            Some(i) => &raw[..i],
            None => raw,
        };
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') {
            let end = line.find(']').unwrap_or(line.len() - 1);
            let tag = line[1..end].trim().to_ascii_uppercase();
            let rest = line[end + 1..].trim();
            in_pass = tag == "PASS TYPE";
            if tag == "IMAGE" {
                if let Some(a) = rest.find('`') {
                    let r2 = &rest[a + 1..];
                    if let Some(b) = r2.find('`') {
                        out.image = r2[..b].to_string();
                        let tail: String = r2[b + 1..]
                            .chars()
                            .filter(|c| c.is_ascii_digit() || *c == '-')
                            .collect();
                        out.image_index = tail.parse().unwrap_or(0);
                    }
                }
            }
            continue;
        }
        if in_pass {
            let row: Vec<i32> = line
                .split_whitespace()
                .filter_map(|t| t.parse::<i32>().ok())
                .collect();
            if !row.is_empty() {
                out.pass.push(row);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "[IMAGE] `Map/01tile00.img` 0\n[pass type]\n2 2 2 2 2 2 2 2 2 2 2 2 2 2\n2 2 2 2 2 2 2 2 2 2 2 2 2 2\n0 0 0 0 0 0 0 0 0 0 0 0 0 0\n2 2 2 2 2 2 2 2 2 2 2 2 2 2\n";

    #[test]
    fn 解析til() {
        let t = parse(SAMPLE);
        assert_eq!(t.image, "Map/01tile00.img");
        assert_eq!(t.pass.len(), 4);
        assert_eq!(t.width(), 224);
        assert_eq!(t.height(), 64);
        assert!(t.is_walkable(2, 0));
        assert!(!t.is_walkable(0, 0));
        assert_eq!(t.walk_band_at(10), Some((32, 47)));
    }

    #[test]
    fn 官方til真值() {
        let Some(pvf) = crate::test_support::pvf() else {
            return;
        };
        let t = parse(&pvf.read_text("map/lorien/tile/forestunder.til").unwrap());
        assert_eq!(t.image, "Map/01tile00.img");
        assert_eq!(t.pass.len(), 30, "官方通行网格 30 行");
        assert_eq!(t.pass[0].len(), 14, "官方通行网格 14 列");
        // forestunder：前 14 行阻挡 → 15 行可走 → 最后 1 行阻挡
        assert!(!t.is_walkable(13, 0));
        assert!(t.is_walkable(14, 0));
        assert!(t.is_walkable(28, 0));
        assert!(!t.is_walkable(29, 0));
        assert_eq!(t.walk_band_at(0), Some((224, 463)));
    }
}
