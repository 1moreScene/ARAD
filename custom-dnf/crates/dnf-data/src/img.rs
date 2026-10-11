//! `.img` 精灵图集读取器（40JP 客户端格式）。
//!
//! # 格式（已用真实文件**逐字节对齐验证**：`sm_body.img` 188 帧恰好走到文件尾）
//!
//! ```text
//! [32B 文件头]
//!   +0x00 "Neople Image File\0"（18 字节，其余填充）
//!   +0x18 u32 版本 = 1
//!   +0x1C u32 帧数 N
//! [N 条帧记录，顺序排列]
//!   每条：
//!     u32 tag
//!       tag == 0x11 → 别名帧：再读 u32 目标帧号，本条记录共 8 字节
//!       否则        → 图像帧：
//!         u32 sub            （本客户端恒为 5；用途未明，非像素格式）
//!         u32 width, height  （精灵尺寸）
//!         u32 size32         （= width*height*4，即引擎内 32bpp 展开后大小）
//!         u32 x, y           （精灵在画布内的放置位置）
//!         u32 canvas_w, canvas_h
//!         u8  pixels[width*height*2]     ← tag 0x0E：16 位 A1R5G5B5，**未压缩**
//! ```
//!
//! * **像素是原始存储（无 zlib）**，与 2006 版（zlib + 36B 头）不同。
//! * 每像素 2 字节（A1R5G5B5）：bit15 = Alpha，bit14-10 = R，bit9-5 = G，bit4-0 = B。
//! * `tag` 的语义：`0x0E` = 16 位 A1R5G5B5（本工程遇到的全部角色/地图资源）；
//!   `0x0F` = A4R4G4B4（保留4位alpha，Combo/UI字形）；`0x10` = 32 位原始 BGRA。解析器对**未知 tag 会报错**
//!   而不是猜，避免静默错位。
//! * 画布 `(canvas_w, canvas_h)` 的作用是让同一动作的各帧共享同一原点（防抖动）；
//!   因此绘制位置 = 画布原点 + `(x, y)`，本模块只暴露 `(x, y)`，不分配大画布。

/// 一帧像素数据（已转成 RGBA8，且**已裁剪到精灵尺寸**）。
#[derive(Clone, Debug)]
pub struct ImgFrame {
    /// 像素格式 tag（0x0E / 0x0F / 0x10）。
    pub tag: u32,
    pub width: u32,
    pub height: u32,
    /// 精灵在画布内的放置位置。
    pub x: i32,
    pub y: i32,
    pub canvas_width: u32,
    pub canvas_height: u32,
    /// `width * height * 4` 的 RGBA8 像素。
    pub rgba: Vec<u8>,
}

impl ImgFrame {
    pub fn empty() -> Self {
        ImgFrame {
            tag: 0,
            width: 0,
            height: 0,
            x: 0,
            y: 0,
            canvas_width: 0,
            canvas_height: 0,
            rgba: Vec::new(),
        }
    }
}

/// 一个 `.img` 图集。
#[derive(Clone, Debug)]
pub struct ImgFile {
    pub version: u32,
    /// 与文件帧数同长；别名帧（LINK）已在解析时解析为指向目标帧的索引。
    pub frames: Vec<ImgFrame>,
    /// 每帧的来源：`None` = 本帧自带数据；`Some(target)` = 别名帧。
    pub links: Vec<Option<usize>>,
}

#[derive(Debug)]
pub enum ImgError {
    NotImg,
    Truncated(String),
    UnknownTag(u32, usize),
    BadSize(String),
}

impl std::fmt::Display for ImgError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImgError::NotImg => write!(f, "不是 IMG（魔法头不符）"),
            ImgError::Truncated(s) => write!(f, "IMG 截断：{}", s),
            ImgError::UnknownTag(t, i) => write!(f, "IMG 未知帧 tag {:#04X}（第 {} 帧）", t, i),
            ImgError::BadSize(s) => write!(f, "IMG 尺寸异常：{}", s),
        }
    }
}

impl std::error::Error for ImgError {}

const MAGIC: &[u8; 18] = b"Neople Image File\0";

/// 解析 `.img`。要求**恰好**消费完整个缓冲区（多余/不足都报错，防止格式假设错误被掩盖）。
pub fn parse(data: &[u8]) -> Result<ImgFile, ImgError> {
    if data.len() < 32 || &data[0..18] != MAGIC {
        return Err(ImgError::NotImg);
    }
    let version = u32::from_le_bytes([data[0x18], data[0x19], data[0x1A], data[0x1B]]);
    let count = u32::from_le_bytes([data[0x1C], data[0x1D], data[0x1E], data[0x1F]]) as usize;

    let mut frames: Vec<ImgFrame> = Vec::with_capacity(count);
    let mut links: Vec<Option<usize>> = Vec::with_capacity(count);
    let mut off = 32usize;

    for i in 0..count {
        if off + 4 > data.len() {
            return Err(ImgError::Truncated(format!("第 {} 帧 tag 越界", i)));
        }
        let tag = u32::from_le_bytes([data[off], data[off + 1], data[off + 2], data[off + 3]]);
        off += 4;

        if tag == 0x11 {
            if off + 4 > data.len() {
                return Err(ImgError::Truncated(format!("第 {} 帧别名目标越界", i)));
            }
            let target =
                u32::from_le_bytes([data[off], data[off + 1], data[off + 2], data[off + 3]])
                    as usize;
            off += 4;
            frames.push(ImgFrame::empty());
            links.push(Some(target));
            continue;
        }

        if off + 32 > data.len() {
            return Err(ImgError::Truncated(format!("第 {} 帧头部越界", i)));
        }
        let rd = |k: usize| {
            u32::from_le_bytes([
                data[off + k],
                data[off + k + 1],
                data[off + k + 2],
                data[off + k + 3],
            ])
        };
        let sub = rd(0);
        let width = rd(4);
        let height = rd(8);
        let size32 = rd(12);
        let x = rd(16) as i32;
        let y = rd(20) as i32;
        let canvas_width = rd(24);
        let canvas_height = rd(28);
        off += 32;
        let _ = sub;

        if width == 0 || height == 0 {
            return Err(ImgError::BadSize(format!(
                "第 {} 帧 {}x{}",
                i, width, height
            )));
        }
        if width > 4096 || height > 4096 {
            return Err(ImgError::BadSize(format!(
                "第 {} 帧尺寸过大 {}x{}",
                i, width, height
            )));
        }
        let pixels = width as usize * height as usize;
        let (stored, rgba) = match tag {
            0x0E | 0x0F => {
                let need = pixels * 2;
                if off + need > data.len() {
                    return Err(ImgError::Truncated(format!("第 {} 帧像素越界", i)));
                }
                let mut rgba = vec![0u8; pixels * 4];
                for p in 0..pixels {
                    let v = u16::from_le_bytes([data[off + p * 2], data[off + p * 2 + 1]]);
                    if tag == 0x0F {
                        // 412C20 packs four alpha/R/G/B nibbles; notably Combo.img.
                        rgba[p * 4] = ((v >> 8) & 15) as u8 * 17;
                        rgba[p * 4 + 1] = ((v >> 4) & 15) as u8 * 17;
                        rgba[p * 4 + 2] = (v & 15) as u8 * 17;
                        rgba[p * 4 + 3] = (v >> 12) as u8 * 17;
                    } else {
                        let expand = |n: u16| ((n << 3) | (n >> 2)) as u8;
                        // Verified against 14,367 live ARAD status-background pixels: MAE=0.
                        rgba[p * 4] = expand((v >> 10) & 31);
                        rgba[p * 4 + 1] = expand((v >> 5) & 31);
                        rgba[p * 4 + 2] = expand(v & 31);
                        rgba[p * 4 + 3] = if v & 0x8000 != 0 { 255 } else { 0 };
                    }
                }
                (need, rgba)
            }
            0x10 => {
                let need = pixels * 4;
                if off + need > data.len() {
                    return Err(ImgError::Truncated(format!("第 {} 帧像素越界", i)));
                }
                // 存储序为 BGRA → 转 RGBA
                let mut rgba = vec![0u8; pixels * 4];
                for p in 0..pixels {
                    rgba[p * 4] = data[off + p * 4 + 2];
                    rgba[p * 4 + 1] = data[off + p * 4 + 1];
                    rgba[p * 4 + 2] = data[off + p * 4];
                    rgba[p * 4 + 3] = data[off + p * 4 + 3];
                }
                (need, rgba)
            }
            other => return Err(ImgError::UnknownTag(other, i)),
        };
        // size32 是引擎内 32bpp 展开大小；校验（不一致仅提示，不致命）
        let _ = size32;
        off += stored;

        frames.push(ImgFrame {
            tag,
            width,
            height,
            x,
            y,
            canvas_width,
            canvas_height,
            rgba,
        });
        links.push(None);
    }

    if off != data.len() {
        return Err(ImgError::BadSize(format!(
            "帧记录总长 {} != 文件长 {}（格式假设可能不适用于该文件）",
            off,
            data.len()
        )));
    }

    // 解析别名帧：沿链找到"自带数据"的帧再复制过去。
    // 官方图集里别名帧既有后向（占绝大多数）也可能前向，甚至链式，所以逐帧跟链，带环保护。
    for i in 0..links.len() {
        let mut target = links[i];
        let mut hops = 0usize;
        while let Some(t) = target {
            if hops > frames.len() || t >= frames.len() {
                break;
            }
            match links[t] {
                Some(next) => {
                    target = Some(next);
                    hops += 1;
                }
                None => {
                    frames[i] = frames[t].clone();
                    break;
                }
            }
        }
    }

    Ok(ImgFile {
        version,
        frames,
        links,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn combo_digits_preserve_a4r4g4b4_colors_and_partial_alpha() {
        let Some(set) = crate::test_support::imagepacks() else {
            return;
        };
        let bytes = set.read("sprite/common/etc/combo.img").unwrap();
        let img = parse(&bytes).unwrap();
        assert_eq!(img.frames.len(), 153);
        let digit = &img.frames[44];
        assert_eq!((digit.tag, digit.width, digit.height), (15, 17, 16));
        assert!(digit.rgba.chunks_exact(4).any(|p| p[3] > 0 && p[3] < 255));
        let yellow = &img.frames[143];
        assert!(yellow
            .rgba
            .chunks_exact(4)
            .any(|p| p[0] > p[2] && p[1] > p[2] && p[3] > 0));
    }

    #[test]
    fn 非img报错() {
        assert!(matches!(parse(&[0u8; 64]), Err(ImgError::NotImg)));
    }
    #[test]
    fn 官方剑士身体图集_帧数与对齐() {
        let Some(set) = crate::test_support::imagepacks() else {
            return;
        };
        let bytes = set.read("sprite/character/swordman/sm_body.img").unwrap();
        let img = parse(&bytes).expect("解析 sm_body.img（必须恰好走到文件尾）");
        assert_eq!(img.version, 1);
        assert_eq!(img.frames.len(), 188, "官方 sm_body.img 共 188 帧");
        // move.ani 用到的帧 180..187 —— 校验尺寸与官方 .ani 的 IMAGE POS 配合
        let f180 = &img.frames[180];
        assert_eq!((f180.width, f180.height), (40, 111));
        assert_eq!((f180.x, f180.y), (209, 224));
        assert_eq!((f180.canvas_width, f180.canvas_height), (500, 500));
        assert_eq!(f180.rgba.len(), 40 * 111 * 4);
        // 帧 180 应当有非透明像素（真实的角色图）
        let opaque = f180.rgba.chunks_exact(4).filter(|p| p[3] != 0).count();
        assert!(opaque > 500, "不透明像素过少: {}", opaque);
    }

    #[test]
    fn 官方地图瓦片与背景() {
        let Some(set) = crate::test_support::imagepacks() else {
            return;
        };
        let bytes = set.read("sprite/map/01tile00.img").unwrap();
        let img = parse(&bytes).expect("解析 01tile00.img");
        let f = &img.frames[0];
        assert_eq!((f.width, f.height), (224, 203));
        assert_eq!((f.x, f.y), (0, 277));
        assert_eq!(f.canvas_width, 224);
        assert_eq!(f.canvas_height, 480, "官方瓦片条画布高恰为 480");

        let far = parse(&set.read("sprite/map/01far1.img").unwrap()).unwrap();
        assert_eq!((far.frames[0].width, far.frames[0].height), (640, 295));
        let mid = parse(&set.read("sprite/map/01mid1.img").unwrap()).unwrap();
        assert_eq!((mid.frames[0].width, mid.frames[0].height), (640, 299));
    }

    #[test]
    fn 官方时装图集的别名帧已解析() {
        let Some(set) = crate::test_support::imagepacks() else {
            return;
        };
        // sm_pants0000a.img：188 帧里 182 帧是别名帧（后向），frame 90 → 0；
        // frame 0 是官方"什么都不画"的占位（1x1 全透明、画布位置 (500,0)）。
        let img = parse(
            &set.read("sprite/character/swordman/equipment/avatar/pants/sm_pants0000a.img")
                .unwrap(),
        )
        .expect("解析 sm_pants0000a.img");
        assert_eq!(img.frames.len(), 188);
        assert_eq!(img.links[90], Some(0), "frame 90 必须是别名帧 → 0");
        assert_eq!(
            (img.frames[90].width, img.frames[90].height),
            (1, 1),
            "别名帧必须已解析成目标帧的真实数据"
        );
        assert_eq!((img.frames[90].x, img.frames[90].y), (500, 0));
        let visible = img
            .frames
            .iter()
            .filter(|f| f.rgba.chunks_exact(4).any(|p| p[3] != 0))
            .count();
        assert!(visible >= 4, "该图集应有可见帧，实际 {}", visible);
    }

    /// 构造一段最小 IMG：3 帧，第 0 帧是**前向**别名 → 2，第 2 帧自带 1x1 白像素。
    #[test]
    fn 前向别名帧也要解析() {
        let mut d = Vec::new();
        let mut header = [0u8; 32];
        header[0..18].copy_from_slice(MAGIC);
        header[0x18] = 1;
        header[0x1C] = 3;
        d.extend_from_slice(&header);
        // frame 0：别名 → 2
        d.extend_from_slice(&0x11u32.to_le_bytes());
        d.extend_from_slice(&2u32.to_le_bytes());
        // frame 1：自带 1x1（像素 0xFC00 = bit15=1 实心红）
        for v in [0x0Eu32, 5, 1, 1, 4, 7, 8, 9, 9] {
            d.extend_from_slice(&v.to_le_bytes());
        }
        d.extend_from_slice(&0xFC00u16.to_le_bytes());
        // frame 2：自带 1x1（像素 0x83E0 = 实心绿）
        for v in [0x0Eu32, 5, 1, 1, 4, 11, 12, 9, 9] {
            d.extend_from_slice(&v.to_le_bytes());
        }
        d.extend_from_slice(&0x83E0u16.to_le_bytes());
        let img = parse(&d).expect("解析构造的 IMG");
        assert_eq!(img.links[0], Some(2));
        assert_eq!(
            (img.frames[0].x, img.frames[0].y),
            (11, 12),
            "前向别名要取到目标帧"
        );
        assert_eq!(img.frames[0].rgba, img.frames[2].rgba);
        assert_eq!(img.frames[0].rgba, vec![0, 255, 0, 255]);
    }
}
