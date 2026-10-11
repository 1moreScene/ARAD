//! RDARScriptStringManager namespace references via string.lst and CP932 .jap.str.
use crate::pvf::PvfReader;
use std::collections::HashMap;

pub struct StringTable {
    namespace: i32,
    entries: HashMap<String, String>,
}
impl StringTable {
    pub fn load(pvf: &PvfReader, namespace: i32) -> Result<Self, String> {
        let list = pvf.read_text("string.lst").map_err(|e| e.to_string())?;
        let path = list
            .lines()
            .find_map(|line| {
                let mut words = line.split("//").next().unwrap_or("").split_whitespace();
                if words.next()?.parse::<i32>().ok()? == namespace {
                    words.next().map(|s| s.trim_matches('`').to_string())
                } else {
                    None
                }
            })
            .ok_or("missing string namespace")?;
        let bytes = pvf.read(&path).map_err(|e| e.to_string())?;
        Ok(Self::parse(namespace, &decode_cp932(&bytes)?))
    }
    pub fn parse(namespace: i32, text: &str) -> Self {
        let mut entries = HashMap::<String, String>::new();
        let mut current = String::new();
        for raw in text.lines() {
            let line = raw.trim_end_matches('\r');
            if let Some((key, value)) = line.split_once('>') {
                if !key.is_empty() && key.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_') {
                    current = key.into();
                    entries.insert(current.clone(), value.into());
                    continue;
                }
            }
            if !current.is_empty() && !line.trim().is_empty() {
                let value = entries.get_mut(&current).unwrap();
                value.push('\n');
                value.push_str(line);
            }
        }
        Self { namespace, entries }
    }
    pub fn resolve(&self, text: &str) -> String {
        let mut out = String::new();
        let mut rest = text;
        while let Some(begin) = rest.find('<') {
            out.push_str(&rest[..begin]);
            rest = &rest[begin..];
            let Some(end) = rest.find('>') else { break };
            let reference = &rest[1..end];
            let replacement = reference.split_once("::").and_then(|(ns, key)| {
                if ns.parse::<i32>().ok() == Some(self.namespace) {
                    self.entries.get(key)
                } else {
                    None
                }
            });
            out.push_str(replacement.map(String::as_str).unwrap_or(&rest[..=end]));
            rest = &rest[end + 1..];
        }
        out.push_str(rest);
        out
    }
}
#[cfg(windows)]
pub(crate) fn decode_cp932(bytes: &[u8]) -> Result<String, String> {
    #[link(name = "kernel32")]
    extern "system" {
        fn MultiByteToWideChar(
            page: u32,
            flags: u32,
            src: *const u8,
            len: i32,
            dst: *mut u16,
            size: i32,
        ) -> i32;
    }
    let len = i32::try_from(bytes.len()).map_err(|_| "string file too large")?;
    // This decoder is scoped to .jap.str; numeric/ASCII scripts keep their existing reader.
    unsafe {
        let n = MultiByteToWideChar(932, 0, bytes.as_ptr(), len, std::ptr::null_mut(), 0);
        if n <= 0 {
            return Err("CP932 string decoding failed".into());
        }
        let mut wide = vec![0u16; n as usize];
        if MultiByteToWideChar(932, 0, bytes.as_ptr(), len, wide.as_mut_ptr(), n) != n {
            return Err("CP932 decoding incomplete".into());
        }
        Ok(String::from_utf16_lossy(&wide))
    }
}
#[cfg(not(windows))]
pub(crate) fn decode_cp932(bytes: &[u8]) -> Result<String, String> {
    String::from_utf8(bytes.to_vec())
        .map_err(|_| "CP932 localization currently requires Windows".into())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn references_multiline_and_placeholders_are_distinct() {
        let s = StringTable::parse(
            12,
            "name>鬼斬り\nbody>Damage <int>\nsecond line\nother>Next",
        );
        assert_eq!(
            s.resolve("<12::name> / <12::body>"),
            "鬼斬り / Damage <int>\nsecond line"
        );
        assert_eq!(
            s.resolve("<int> <11::name> <12::missing>"),
            "<int> <11::name> <12::missing>"
        );
    }
    #[test]
    #[cfg(windows)]
    fn official_japanese_skill_strings_decode_without_replacement_characters() {
        let Some(pvf) = crate::test_support::pvf() else {
            return;
        };
        let table = StringTable::load(pvf, 12).unwrap();
        assert_eq!(table.resolve("<12::name_17745>"), "鬼斬り");
        let body = table.resolve("<12::explain_17750>");
        assert!(body.contains("物理攻撃") && !body.contains('\u{fffd}'));
    }
}
