//! CStringManager 421300/422BD0/422E70: File resource 115, CP932 id>text.
use std::{collections::HashMap, path::Path};

pub struct SystemMessages(pub HashMap<i32, String>);
impl SystemMessages {
    pub fn load(exe: &Path) -> Result<Self, String> {
        let bytes = std::fs::read(exe).map_err(|e| e.to_string())?;
        let raw = file_resource(&bytes, 115)?;
        let text = crate::strings::decode_cp932(raw)?;
        Ok(Self(
            text.lines()
                .filter_map(|line| {
                    let (id, value) = line.split_once('>')?;
                    Some((id.parse().ok()?, value.replace("\\n", "\n")))
                })
                .collect(),
        ))
    }
    pub fn get(&self, id: i32) -> Option<&str> {
        self.0.get(&id).map(String::as_str)
    }
}
fn u16_at(b: &[u8], p: usize) -> Result<u16, String> {
    Ok(u16::from_le_bytes(
        b.get(p..p + 2).ok_or("truncated PE")?.try_into().unwrap(),
    ))
}
fn u32_at(b: &[u8], p: usize) -> Result<u32, String> {
    Ok(u32::from_le_bytes(
        b.get(p..p + 4).ok_or("truncated PE")?.try_into().unwrap(),
    ))
}
fn file_resource(b: &[u8], id: u32) -> Result<&[u8], String> {
    let pe = u32_at(b, 0x3c)? as usize;
    if b.get(pe..pe + 4) != Some(b"PE\0\0") || u16_at(b, pe + 24)? != 0x10b {
        return Err("expected official PE32 executable".into());
    }
    let sections = u16_at(b, pe + 6)? as usize;
    let table = pe + 24 + u16_at(b, pe + 20)? as usize;
    let map = |rva: u32| -> Result<usize, String> {
        for i in 0..sections {
            let s = table + 40 * i;
            let va = u32_at(b, s + 12)?;
            let size = u32_at(b, s + 16)?;
            if rva >= va && rva - va < size {
                return Ok((u32_at(b, s + 20)? + rva - va) as usize);
            }
        }
        Err("PE resource RVA unmapped".into())
    };
    let base = map(u32_at(b, pe + 24 + 96 + 16)?)?;
    let entries = |dir: usize| -> Result<Vec<(u32, u32)>, String> {
        let count = u16_at(b, dir + 12)? as usize + u16_at(b, dir + 14)? as usize;
        (0..count)
            .map(|i| Ok((u32_at(b, dir + 16 + 8 * i)?, u32_at(b, dir + 20 + 8 * i)?)))
            .collect()
    };
    let mut file = None;
    for (name, target) in entries(base)? {
        if name & 0x80000000 == 0 {
            continue;
        }
        let p = base + (name & 0x7fffffff) as usize;
        let len = u16_at(b, p)? as usize;
        let text = (0..len)
            .map(|i| u16_at(b, p + 2 + 2 * i))
            .collect::<Result<Vec<_>, _>>()?;
        if String::from_utf16_lossy(&text).eq_ignore_ascii_case("File") {
            file = Some(target);
            break;
        }
    }
    let file = file.ok_or("missing File resource directory")?;
    let target = entries(base + (file & 0x7fffffff) as usize)?
        .into_iter()
        .find(|(name, _)| *name == id)
        .ok_or("missing system string resource 115")?
        .1;
    let leaf = entries(base + (target & 0x7fffffff) as usize)?
        .first()
        .ok_or("missing resource language")?
        .1;
    let data = base + leaf as usize;
    let start = map(u32_at(b, data)?)?;
    let end = start
        .checked_add(u32_at(b, data + 4)? as usize)
        .ok_or("resource size overflow")?;
    b.get(start..end)
        .ok_or_else(|| "truncated system string resource".into())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reads_official_failure_messages_from_exe_not_pvf() {
        let Some(dir) = crate::locate_client_dir() else {
            return;
        };
        let messages = SystemMessages::load(&dir.join("ARAD.exe")).unwrap();
        assert_eq!(messages.get(853), Some("クールタイム中です。"));
        assert_eq!(messages.get(854), Some("MPが足りません。"));
        assert_eq!(messages.get(855), Some("アイテムが足りません。"));
        assert!(file_resource(b"invalid", 115).is_err());
    }
}
