//! `NeoplePack_Bill` 资源包（NPK）读取器 —— 2006/40JP 版（目录名明文）。
//!
//! 布局（`ARAD.exe` `sub_40E9E0` NPK 加载器；本仓库 `tools/npk_reader.py` 已用真实
//! `ImagePacks2/*.NPK` 全量验证）：
//!
//! ```text
//! [16B 魔法 "NeoplePack_Bill\0"]
//! [u32 条目数 N]
//! [N × 264B 目录条目]  每条 = u32 数据偏移 + u32 数据大小 + 256B 明文文件名
//! [文件数据区]
//! ```
//!
//! 本模块提供两类 API：
//! * [`NpkArchive`] —— 单个 NPK；
//! * [`NpkSet`] —— 整个 `ImagePacks2/` 目录的**名字索引**（只读各 NPK 的目录区，
//!   不读数据），用于按 `sprite/...` 路径直接取文件。

use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

const MAGIC: &[u8; 16] = b"NeoplePack_Bill\0";

#[derive(Clone, Debug)]
pub struct NpkEntry {
    pub offset: u32,
    pub size: u32,
    pub name: String,
}

#[derive(Debug)]
pub enum NpkError {
    NotNpk,
    Io(std::io::Error),
    Truncated(String),
    NotFound(String),
}

impl std::fmt::Display for NpkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NpkError::NotNpk => write!(f, "不是 NPK（魔法头不符）"),
            NpkError::Io(e) => write!(f, "IO 错误：{}", e),
            NpkError::Truncated(s) => write!(f, "NPK 截断：{}", s),
            NpkError::NotFound(s) => write!(f, "NPK 内不存在：{}", s),
        }
    }
}

impl std::error::Error for NpkError {}
impl From<std::io::Error> for NpkError {
    fn from(e: std::io::Error) -> Self {
        NpkError::Io(e)
    }
}

/// 单个 NPK 包。
pub struct NpkArchive {
    path: PathBuf,
    pub entries: Vec<NpkEntry>,
    by_name: HashMap<String, usize>,
}

impl NpkArchive {
    /// 只解析目录（不读数据区）。
    pub fn open(path: &Path) -> Result<Self, NpkError> {
        let mut f = File::open(path)?;
        let mut head = [0u8; 20];
        f.read_exact(&mut head)
            .map_err(|_| NpkError::Truncated("头部".into()))?;
        if &head[0..16] != MAGIC {
            return Err(NpkError::NotNpk);
        }
        let count = u32::from_le_bytes([head[16], head[17], head[18], head[19]]) as usize;

        // 一次性读目录区
        let mut dir = vec![0u8; count * 264];
        f.read_exact(&mut dir)
            .map_err(|_| NpkError::Truncated("目录区".into()))?;

        let mut entries = Vec::with_capacity(count);
        for i in 0..count {
            let o = i * 264;
            let offset = u32::from_le_bytes([dir[o], dir[o + 1], dir[o + 2], dir[o + 3]]);
            let size = u32::from_le_bytes([dir[o + 4], dir[o + 5], dir[o + 6], dir[o + 7]]);
            let raw = &dir[o + 8..o + 264];
            let end = raw.iter().position(|b| *b == 0).unwrap_or(raw.len());
            let name = String::from_utf8_lossy(&raw[..end]).into_owned();
            entries.push(NpkEntry { offset, size, name });
        }
        let mut by_name = HashMap::with_capacity(entries.len());
        for (i, e) in entries.iter().enumerate() {
            by_name.entry(e.name.to_ascii_lowercase()).or_insert(i);
        }
        Ok(NpkArchive {
            path: path.to_path_buf(),
            entries,
            by_name,
        })
    }

    pub fn contains(&self, name: &str) -> bool {
        self.by_name.contains_key(&name.to_ascii_lowercase())
    }

    pub fn read(&self, name: &str) -> Result<Vec<u8>, NpkError> {
        let idx = *self
            .by_name
            .get(&name.to_ascii_lowercase())
            .ok_or_else(|| NpkError::NotFound(name.to_string()))?;
        let e = &self.entries[idx];
        let mut f = File::open(&self.path)?;
        f.seek(SeekFrom::Start(e.offset as u64))?;
        let mut buf = vec![0u8; e.size as usize];
        f.read_exact(&mut buf)?;
        Ok(buf)
    }

    /// 目录完整性自检：偏移单调不减、末条不越界。
    pub fn validate(&self) -> Vec<String> {
        let fsize = std::fs::metadata(&self.path).map(|m| m.len()).unwrap_or(0);
        let mut problems = Vec::new();
        let mut prev_end = 0u64;
        for e in &self.entries {
            if (e.offset as u64) < prev_end {
                problems.push(format!(
                    "偏移回退: {} off={} < {}",
                    e.name, e.offset, prev_end
                ));
            }
            if e.offset as u64 + e.size as u64 > fsize {
                problems.push(format!(
                    "越界: {} off={} size={} > 文件 {}",
                    e.name, e.offset, e.size, fsize
                ));
            }
            prev_end = prev_end.max(e.offset as u64 + e.size as u64);
        }
        problems
    }
}

/// 整个素材目录（如 `ImagePacks2/`）的名字索引。
///
/// 只读每个 NPK 的目录区；命中后按需打开对应 NPK 取数据。
pub struct NpkSet {
    archives: Vec<NpkArchive>,
    index: HashMap<String, (usize, usize)>,
    /// 目录扫描诊断。
    pub report: Vec<String>,
}

impl NpkSet {
    /// 扫描目录下所有 `*.NPK` / `*.npk`。
    pub fn scan_dir(dir: &Path) -> Result<Self, NpkError> {
        let mut paths: Vec<PathBuf> = Vec::new();
        if let Ok(rd) = std::fs::read_dir(dir) {
            for e in rd.flatten() {
                let p = e.path();
                if let Some(ext) = p.extension().and_then(|s| s.to_str()) {
                    if ext.eq_ignore_ascii_case("npk") {
                        paths.push(p);
                    }
                }
            }
        }
        paths.sort();
        Self::open_many(&paths)
    }

    pub fn open_many(paths: &[PathBuf]) -> Result<Self, NpkError> {
        let mut archives = Vec::new();
        let mut index = HashMap::new();
        let mut report = Vec::new();
        let mut total = 0usize;
        for p in paths {
            match NpkArchive::open(p) {
                Ok(a) => {
                    let ai = archives.len();
                    let problems = a.validate();
                    if !problems.is_empty() {
                        report.push(format!(
                            "{}: 目录自检 {} 项异常",
                            p.file_name().and_then(|s| s.to_str()).unwrap_or("?"),
                            problems.len()
                        ));
                    }
                    for (ei, e) in a.entries.iter().enumerate() {
                        index.entry(e.name.to_ascii_lowercase()).or_insert((ai, ei));
                    }
                    total += a.entries.len();
                    archives.push(a);
                }
                Err(e) => report.push(format!(
                    "跳过 {}：{}",
                    p.file_name().and_then(|s| s.to_str()).unwrap_or("?"),
                    e
                )),
            }
        }
        report.push(format!(
            "索引 {} 个 NPK，共 {} 个资源条目",
            archives.len(),
            total
        ));
        Ok(NpkSet {
            archives,
            index,
            report,
        })
    }

    pub fn len(&self) -> usize {
        self.index.len()
    }

    pub fn is_empty(&self) -> bool {
        self.index.is_empty()
    }

    pub fn contains(&self, name: &str) -> bool {
        self.index.contains_key(normalize(name).as_str())
    }

    pub fn read(&self, name: &str) -> Result<Vec<u8>, NpkError> {
        let key = normalize(name);
        let (ai, ei) = *self
            .index
            .get(&key)
            .ok_or_else(|| NpkError::NotFound(name.to_string()))?;
        self.archives[ai].read(&self.archives[ai].entries[ei].name)
    }

    /// 按后缀列出名字（诊断用）。
    pub fn names_with_suffix(&self, suffix: &str) -> Vec<String> {
        let s = suffix.to_ascii_lowercase();
        self.index
            .keys()
            .filter(|k| k.ends_with(&s))
            .cloned()
            .collect()
    }
}

/// 统一成 `/` 分隔的小写路径（NPK 内名字大小写不统一）。
pub fn normalize(name: &str) -> String {
    name.replace('\\', "/").to_ascii_lowercase()
}

#[cfg(test)]
fn dnf_client_imagepacks() -> Option<PathBuf> {
    let dir = crate::locate_client_dir()?;
    let p = crate::imagepacks_dir(&dir);
    if p.is_dir() {
        Some(p)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 路径归一() {
        assert_eq!(
            normalize("Character\\Swordman\\sm_body.img"),
            "character/swordman/sm_body.img"
        );
    }

    #[test]
    fn 目录扫描并读取真值() {
        let Some(dir) = dnf_client_imagepacks() else {
            eprintln!("跳过：未找到 ImagePacks2");
            return;
        };
        let set = NpkSet::scan_dir(&dir).expect("扫描 ImagePacks2");
        assert!(set.len() > 5_000, "索引条目过少: {}", set.len());
        // 官方剑士身体精灵
        let body = set
            .read("sprite/character/swordman/sm_body.img")
            .expect("读 sm_body.img");
        assert_eq!(&body[0..18], b"Neople Image File\0");
        // 官方瓦片条与背景
        let tile = set
            .read("sprite/map/01tile00.img")
            .expect("读 01tile00.img");
        assert!(tile.len() > 1000);
    }
}
