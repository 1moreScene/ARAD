//! `.map` 房间定义解析。
//!
//! 以官方 `map/lorien/104.map`（洛兰森林）为样本，实测标签：
//!
//! ```text
//! [type]  `[normal]`
//! [tile]                        ← 房间由 N 条 224px 瓦片条横向拼接
//! `Tile/ForestOverUnder.til`
//! ...
//! [/tile]
//! [far sight scroll]  56        ← 视差滚动率（千分？实为百分：56 → 0.56）
//! [middle sight scroll]  90
//! [near sight scroll]  110
//! [background animation]
//! `Animation/far1.ani` `[distantback]` `[below]`
//! `Animation/mid1.ani` `[middleback]` `[below]`
//! [/background animation]
//! [pathgate pos]  16 216  /  1104 216  /  560 144  /  560 348   ← 门的坐标
//! [animation]      `Animation/DeadTree0.ani` `[close]` 268 483 0 ← 图层 / x / y / z
//! ```

/// 瓦片条引用。
#[derive(Clone, Debug, PartialEq)]
pub struct TileRef {
    /// `.til` 路径（相对 `map/<room>/`，如 `Tile/ForestUnder.til`）。
    pub path: String,
}

/// 背景动画条目。
#[derive(Clone, Debug, PartialEq)]
pub struct BackgroundLayer {
    pub anim: String,
    /// `[distantback]` / `[middleback]` / `[below]` …
    pub layer: String,
    pub below: bool,
}

/// 场景物件（树/石/草等）。
#[derive(Clone, Debug, PartialEq)]
pub struct MapObject {
    pub anim: String,
    pub layer: String,
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Clone, Debug, Default)]
pub struct MapFile {
    pub map_type: String,
    pub tiles: Vec<TileRef>,
    pub far_scroll: i32,
    pub middle_scroll: i32,
    pub near_scroll: i32,
    pub backgrounds: Vec<BackgroundLayer>,
    pub objects: Vec<MapObject>,
    pub pathgates: Vec<(i32, i32)>,
    pub map_name: String,
}

impl MapFile {
    /// 房间水平宽度 = 瓦片条数 × 224。
    pub fn room_width(&self) -> i32 {
        self.tiles.len() as i32 * 224
    }

    /// 水平可走范围：左右门坐标（官方门坐标内缩 16px）。
    pub fn walkable_x(&self) -> (i32, i32) {
        let w = self.room_width();
        let left = self.pathgates.iter().map(|g| g.0).min().unwrap_or(16);
        let right = self.pathgates.iter().map(|g| g.0).max().unwrap_or(w - 16);
        (left, right)
    }
}

fn split_tag(line: &str) -> Option<(String, &str)> {
    let line = line.trim();
    if !line.starts_with('[') {
        return None;
    }
    let end = line.find(']')?;
    Some((
        line[1..end].trim().to_ascii_uppercase(),
        line[end + 1..].trim(),
    ))
}

fn dequote(s: &str) -> String {
    let raw = if let Some(a) = s.find('`') {
        let r = &s[a + 1..];
        match r.find('`') {
            Some(b) => r[..b].to_string(),
            None => r.trim().to_string(),
        }
    } else {
        s.trim().to_string()
    };
    raw.trim_matches(|c| c == '[' || c == ']' || c == ' ')
        .to_string()
}

fn ints(s: &str) -> Vec<i32> {
    let cleaned: String = s
        .chars()
        .map(|c| {
            if c.is_ascii_digit() || c == '-' || c == '+' {
                c
            } else {
                ' '
            }
        })
        .collect();
    cleaned
        .split_whitespace()
        .filter_map(|t| t.parse::<i32>().ok())
        .collect()
}

/// 取第 4 个反引号段（`[图层]`）并去掉方括号、转小写：
/// `` `Animation/x.ani` `[close]` 268 483 0 `` → `close`
fn layer_tag(line: &str) -> String {
    line.split('`')
        .nth(3)
        .unwrap_or("")
        .trim_matches(|c| c == '[' || c == ']' || c == ' ')
        .to_ascii_lowercase()
}

/// 最后一个反引号之后的内容：`` `ani` `[layer]` 886 488 0 `` → ` 886 488 0`。
fn tail_after_backticks(s: &str) -> &str {
    match s.rfind('`') {
        Some(i) => &s[i + 1..],
        None => s,
    }
}

/// 解析 `.map` 文本。
pub fn parse(text: &str) -> MapFile {
    let mut out = MapFile::default();
    let mut section = String::new();

    for raw in text.lines() {
        let line = match raw.find("//") {
            Some(i) => &raw[..i],
            None => raw,
        };
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Some((tag, rest)) = split_tag(trimmed) {
            if let Some(name) = tag.strip_prefix('/') {
                if section == name {
                    section.clear();
                }
                continue;
            }
            match tag.as_str() {
                "TILE" => section = "TILE".into(),
                "BACKGROUND ANIMATION" => section = "BG".into(),
                "ANIMATION" => section = "OBJ".into(),
                "PATHGATE POS" => section = "GATE".into(),
                "TYPE" => {
                    out.map_type = dequote(rest);
                    section.clear();
                }
                "FAR SIGHT SCROLL" => out.far_scroll = ints(rest).first().copied().unwrap_or(100),
                "MIDDLE SIGHT SCROLL" => {
                    out.middle_scroll = ints(rest).first().copied().unwrap_or(100)
                }
                "NEAR SIGHT SCROLL" => out.near_scroll = ints(rest).first().copied().unwrap_or(100),
                "MAP NAME" => {
                    out.map_name = rest.to_string();
                    section.clear();
                }
                _ => section.clear(),
            }
            continue;
        }

        match section.as_str() {
            "TILE" => out.tiles.push(TileRef {
                path: dequote(trimmed),
            }),
            "BG" => {
                let layer = layer_tag(trimmed);
                let below = trimmed.contains("[below]");
                out.backgrounds.push(BackgroundLayer {
                    anim: dequote(trimmed),
                    layer,
                    below,
                });
            }
            "OBJ" => {
                // 坐标写在 `` `ani` `[layer]` `` 之后；直接对整行取数字会把
                // 动画名里的数字（smallTree0 / DeadTree1）当坐标，必须只取尾巴。
                let v = ints(tail_after_backticks(trimmed));
                let layer = layer_tag(trimmed);
                if v.len() >= 3 {
                    out.objects.push(MapObject {
                        anim: dequote(trimmed),
                        layer,
                        x: v[0],
                        y: v[1],
                        z: *v.get(2).unwrap_or(&0),
                    });
                }
            }
            "GATE" => {
                let v = ints(trimmed);
                if v.len() >= 2 {
                    out.pathgates.push((v[0], v[1]));
                }
            }
            _ => {}
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 官方lorien104真值() {
        let Some(pvf) = crate::test_support::pvf() else {
            return;
        };
        let m = parse(&pvf.read_text("map/lorien/104.map").unwrap());
        assert_eq!(m.map_type, "normal");
        assert_eq!(m.tiles.len(), 5, "104 房间 5 条瓦片");
        assert_eq!(m.tiles[0].path, "Tile/ForestOverUnder.til");
        assert_eq!(m.room_width(), 1120);
        assert_eq!(m.far_scroll, 56);
        assert_eq!(m.middle_scroll, 90);
        assert_eq!(m.near_scroll, 110);
        assert_eq!(m.backgrounds.len(), 2);
        assert_eq!(m.backgrounds[0].anim, "Animation/far1.ani");
        assert_eq!(m.backgrounds[0].layer, "distantback");
        assert_eq!(m.backgrounds[1].anim, "Animation/mid1.ani");
        assert!(m.backgrounds[0].below);
        // 门坐标（官方）：左右 x = 16 / 1104
        assert!(m.pathgates.contains(&(16, 216)));
        assert!(m.pathgates.contains(&(1104, 216)));
        assert_eq!(m.walkable_x(), (16, 1104));
        // 物件
        assert!(m.objects.len() >= 10);
        assert!(m
            .objects
            .iter()
            .any(|o| o.anim == "Animation/DeadTree0.ani" && o.layer == "close"));
        // 官方 104：`` `Animation/smallTree0.ani` `[close]` 886 488 0 ``
        // 坐标在反引号之后，动画名里的 "0" 不能当坐标（回归：曾把 x 读成 0、y 读成 886）
        let tree = m
            .objects
            .iter()
            .find(|o| o.anim == "Animation/smallTree0.ani")
            .unwrap();
        assert_eq!(
            (tree.x, tree.y, tree.z),
            (886, 488, 0),
            "物件坐标必须只取反引号之后的数字"
        );
        assert_eq!(tree.layer, "close");
        let flower = m
            .objects
            .iter()
            .find(|o| o.anim == "Animation/Flower1.ani")
            .unwrap();
        assert_eq!((flower.x, flower.y), (562, 228), "Flower1 官方坐标");
    }
}
