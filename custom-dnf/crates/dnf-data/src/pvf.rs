//! `Script.pvf`（v1，2006–2008 客户端格式）读取器。
//!
//! 忠实移植自本仓库已验证的 Python 参考实现（`tools/pvf_reader.py`），
//! 后者依据 `ARAD.exe` 的 `sub_528DB0`（加载器）+ `sub_527AC0`（目录解密）
//! + `sub_528650`（内容读取）逆向并用真实 `Script.pvf` 全量验证。
//!
//! ```text
//! [15B "DNF_SCRIPT_PACK"]
//! [u32 S = 加密目录块字节数]
//! [u32 A = 目录 XOR 密钥（并作为完整性校验）]
//! [u32 B = 目录条目数]
//! [S 字节 加密目录块]
//!   B 条目录，每条 = u32 索引 + u32 内容偏移 + u32 内容大小 + u32 内容密钥 + u32 名字长 + 名字
//! [文件内容区]
//! ```
//!
//! 目录解密：`state = ~B`；每 4 字节 `dec = ror32(A ^ 密文, 6)`，并把 `dec` 的 4 字节
//! 喂进标准 CRC32 滚动；全部解完后要求 `state == ~A`。
//!
//! 内容读取：`seek = 偏移 + 27 + S`，长度向上按 4 字节对齐；
//! `dec = ror32(内容key ^ 密文, 6)`，随后按实际大小截断。

use std::collections::HashMap;
use std::path::Path;

const MAGIC: &[u8; 15] = b"DNF_SCRIPT_PACK";
const CURSOR: usize = 15 + 12;

const fn crc_table() -> [u32; 256] {
    let mut t = [0u32; 256];
    let mut i = 0usize;
    while i < 256 {
        let mut c = i as u32;
        let mut k = 0;
        while k < 8 {
            c = if c & 1 != 0 {
                0xEDB8_8320 ^ (c >> 1)
            } else {
                c >> 1
            };
            k += 1;
        }
        t[i] = c;
        i += 1;
    }
    t
}

static CRC_TABLE: [u32; 256] = crc_table();

#[inline]
fn ror32(v: u32, n: u32) -> u32 {
    v.rotate_right(n)
}

/// 目录/内容解密。
fn decrypt_blob(blob: &[u8], init_key: u32, xor_key: u32) -> (Vec<u8>, u32) {
    let mut state = !init_key;
    let n = blob.len() - (blob.len() % 4);
    let mut out = Vec::with_capacity(n);
    let mut off = 0usize;
    while off < n {
        let dw = u32::from_le_bytes([blob[off], blob[off + 1], blob[off + 2], blob[off + 3]]);
        let dec = ror32(xor_key ^ dw, 6);
        for b in dec.to_le_bytes() {
            state = CRC_TABLE[((state ^ b as u32) & 0xFF) as usize] ^ (state >> 8);
        }
        out.extend_from_slice(&dec.to_le_bytes());
        off += 4;
    }
    (out, state)
}

/// 目录条目。
#[derive(Clone, Debug)]
pub struct PvfEntry {
    pub index: u32,
    pub offset: u32,
    pub size: u32,
    pub key: u32,
    pub name: String,
}

/// PVF 读取器。
pub struct PvfReader {
    data: Vec<u8>,
    pub dir_size: u32,
    pub dir_key: u32,
    pub entry_count: u32,
    pub entries: Vec<PvfEntry>,
    by_name: HashMap<String, usize>,
}

#[derive(Debug)]
pub enum PvfError {
    NotPvf,
    CrcMismatch { got: u32, want: u32 },
    Truncated(String),
    NotFound(String),
    Io(std::io::Error),
}

impl std::fmt::Display for PvfError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PvfError::NotPvf => write!(f, "不是 PVF（魔法头不符）"),
            PvfError::CrcMismatch { got, want } => {
                write!(
                    f,
                    "目录块 CRC 校验失败（终态 {:#010X} != ~A {:#010X}）",
                    got, want
                )
            }
            PvfError::Truncated(s) => write!(f, "文件截断：{}", s),
            PvfError::NotFound(s) => write!(f, "PVF 中不存在：{}", s),
            PvfError::Io(e) => write!(f, "IO 错误：{}", e),
        }
    }
}

impl std::error::Error for PvfError {}

impl From<std::io::Error> for PvfError {
    fn from(e: std::io::Error) -> Self {
        PvfError::Io(e)
    }
}

impl PvfReader {
    pub fn open(path: &Path) -> Result<Self, PvfError> {
        let data = std::fs::read(path)?;
        Self::from_bytes(data)
    }

    pub fn from_bytes(data: Vec<u8>) -> Result<Self, PvfError> {
        if data.len() < 27 || &data[0..15] != MAGIC {
            return Err(PvfError::NotPvf);
        }
        let dir_size = u32::from_le_bytes([data[15], data[16], data[17], data[18]]);
        let dir_key = u32::from_le_bytes([data[19], data[20], data[21], data[22]]);
        let entry_count = u32::from_le_bytes([data[23], data[24], data[25], data[26]]);

        let blob_end = CURSOR + dir_size as usize;
        if data.len() < blob_end {
            return Err(PvfError::Truncated("目录块越过文件尾".into()));
        }
        let (dec, state) = decrypt_blob(&data[CURSOR..blob_end], entry_count, dir_key);
        let want = !dir_key;
        if state != want {
            return Err(PvfError::CrcMismatch { got: state, want });
        }

        let mut entries = Vec::with_capacity(entry_count as usize);
        let mut off = 0usize;
        for i in 0..entry_count {
            if off + 20 > dec.len() {
                return Err(PvfError::Truncated(format!("目录条目越界 @{}", i)));
            }
            let w0 = u32::from_le_bytes([dec[off], dec[off + 1], dec[off + 2], dec[off + 3]]);
            let w1 = u32::from_le_bytes([dec[off + 4], dec[off + 5], dec[off + 6], dec[off + 7]]);
            let w2 = u32::from_le_bytes([dec[off + 8], dec[off + 9], dec[off + 10], dec[off + 11]]);
            let w3 =
                u32::from_le_bytes([dec[off + 12], dec[off + 13], dec[off + 14], dec[off + 15]]);
            let namelen =
                u32::from_le_bytes([dec[off + 16], dec[off + 17], dec[off + 18], dec[off + 19]])
                    as usize;
            off += 20;
            if off + namelen > dec.len() {
                return Err(PvfError::Truncated(format!("名字越界 @{}", i)));
            }
            let name = String::from_utf8_lossy(&dec[off..off + namelen]).into_owned();
            off += namelen;
            entries.push(PvfEntry {
                index: w0,
                offset: w1,
                size: w2,
                key: w3,
                name,
            });
        }

        let mut by_name = HashMap::with_capacity(entries.len());
        for (i, e) in entries.iter().enumerate() {
            by_name.entry(e.name.to_ascii_lowercase()).or_insert(i);
        }

        Ok(PvfReader {
            data,
            dir_size,
            dir_key,
            entry_count,
            entries,
            by_name,
        })
    }

    /// 按路径读内容（大小写不敏感）。
    pub fn read(&self, name: &str) -> Result<Vec<u8>, PvfError> {
        let idx = *self
            .by_name
            .get(&name.to_ascii_lowercase())
            .ok_or_else(|| PvfError::NotFound(name.to_string()))?;
        let e = &self.entries[idx];
        let seek = e.offset as usize + CURSOR + self.dir_size as usize;
        let rounded = (e.size as usize + 3) & !3;
        if seek + rounded > self.data.len() {
            return Err(PvfError::Truncated(format!("内容越界：{}", e.name)));
        }
        let (dec, _) = decrypt_blob(&self.data[seek..seek + rounded], e.offset, e.key);
        Ok(dec[..e.size as usize].to_vec())
    }

    /// 按路径读内容（UTF-8 文本，供 `.ani` / `.chr` / `.skl`）。
    pub fn read_text(&self, name: &str) -> Result<String, PvfError> {
        let bytes = self.read(name)?;
        // 官方脚本文件是 EUC-KR / SJIS 的西文兼容子集；标签与数字都是 ASCII。
        Ok(String::from_utf8_lossy(&bytes).into_owned())
    }

    pub fn contains(&self, name: &str) -> bool {
        self.by_name.contains_key(&name.to_ascii_lowercase())
    }

    /// 路径前缀过滤。
    pub fn names_with_prefix(&self, prefix: &str) -> Vec<&str> {
        let p = prefix.to_ascii_lowercase();
        self.entries
            .iter()
            .filter(|e| e.name.to_ascii_lowercase().starts_with(&p))
            .map(|e| e.name.as_str())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 解密原语_旋转与crc表自洽() {
        assert_eq!(ror32(0x4000_0000, 6), 0x0100_0000);
        assert_eq!(ror32(1, 1), 0x8000_0000);
        // CRC32 表首项与标准一致
        assert_eq!(CRC_TABLE[0], 0);
        assert_eq!(CRC_TABLE[1], 0x7707_3096);
        assert_eq!(CRC_TABLE[255], 0x2D02_EF8D);
    }

    #[test]
    fn 非pvf文件报错() {
        match PvfReader::from_bytes(vec![0u8; 64]) {
            Err(PvfError::NotPvf) => {}
            Err(other) => panic!("期望 NotPvf，实得 {:?}", other),
            Ok(_) => panic!("非 PVF 文件不应解析成功"),
        }
    }
}
