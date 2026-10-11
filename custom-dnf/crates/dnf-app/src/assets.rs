//! 官方客户端资源门面：PVF + NPK 目录索引 + 各类解析结果的进程内缓存。

use dnf_data::{
    ani::AnimFile, img::ImgFile, mapfile::MapFile, npk::NpkSet, pvf::PvfReader, til::TilFile,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;

/// 把官方资源路径统一成 NPK 里的 `sprite/...` 小写路径。
pub fn img_path(p: &str) -> String {
    let s = p.replace('\\', "/").to_ascii_lowercase();
    if s.starts_with("sprite/") {
        s
    } else {
        format!("sprite/{}", s)
    }
}

pub struct ClientAssets {
    pub pvf: PvfReader,
    pub packs: NpkSet,
    pub client_dir: std::path::PathBuf,
    imgs: RefCell<HashMap<String, Rc<ImgFile>>>,
    anis: RefCell<HashMap<String, Rc<AnimFile>>>,
    tils: RefCell<HashMap<String, Rc<TilFile>>>,
    pub report: Vec<String>,
}

impl ClientAssets {
    pub fn load(client_dir: &Path) -> Result<Self, String> {
        let pvf = PvfReader::open(&client_dir.join("Script.pvf")).map_err(|e| e.to_string())?;
        let packs =
            NpkSet::scan_dir(&dnf_data::imagepacks_dir(client_dir)).map_err(|e| e.to_string())?;
        let mut report = vec![
            format!("PVF: {} 条目录条目（CRC 校验通过）", pvf.entry_count),
            format!(
                "NPK: {} 个包 / {} 个资源条目",
                packs.report.len().saturating_sub(1),
                packs.len()
            ),
        ];
        report.extend(packs.report.iter().cloned());
        Ok(ClientAssets {
            pvf,
            packs,
            client_dir: client_dir.to_path_buf(),
            imgs: RefCell::new(HashMap::new()),
            anis: RefCell::new(HashMap::new()),
            tils: RefCell::new(HashMap::new()),
            report,
        })
    }

    /// 取 `.img`（带缓存）。
    pub fn img(&self, name: &str) -> Option<Rc<ImgFile>> {
        let key = img_path(name);
        if let Some(v) = self.imgs.borrow().get(&key) {
            return Some(v.clone());
        }
        let bytes = self.packs.read(&key).ok()?;
        let parsed = dnf_data::img::parse(&bytes).ok()?;
        let rc = Rc::new(parsed);
        self.imgs.borrow_mut().insert(key, rc.clone());
        Some(rc)
    }

    /// 取 `.ani`（来自 PVF，按路径小写查找）。
    pub fn ani(&self, pvf_path: &str) -> Option<Rc<AnimFile>> {
        let key = pvf_path.replace('\\', "/").to_ascii_lowercase();
        if let Some(v) = self.anis.borrow().get(&key) {
            return Some(v.clone());
        }
        let text = self.pvf.read_text(&key).ok()?;
        let parsed = dnf_data::ani::parse(&text);
        let rc = Rc::new(parsed);
        self.anis.borrow_mut().insert(key, rc.clone());
        Some(rc)
    }

    /// 取 `.til`（`.map` 里的相对路径 + 房间目录）。
    pub fn til(&self, room_dir: &str, rel: &str) -> Option<Rc<TilFile>> {
        let key = format!(
            "{}/{}",
            room_dir.trim_end_matches('/'),
            rel.replace('\\', "/")
        )
        .to_ascii_lowercase();
        if let Some(v) = self.tils.borrow().get(&key) {
            return Some(v.clone());
        }
        let text = self.pvf.read_text(&key).ok()?;
        let parsed = dnf_data::til::parse(&text);
        let rc = Rc::new(parsed);
        self.tils.borrow_mut().insert(key, rc.clone());
        Some(rc)
    }

    pub fn map(&self, pvf_path: &str) -> Option<MapFile> {
        let key = pvf_path.replace('\\', "/").to_ascii_lowercase();
        let text = self.pvf.read_text(&key).ok()?;
        Some(dnf_data::mapfile::parse(&text))
    }

    pub fn has_img(&self, name: &str) -> bool {
        self.packs.contains(&img_path(name))
    }

    /// 缓存统计（诊断）。
    pub fn cache_stats(&self) -> (usize, usize, usize) {
        (
            self.imgs.borrow().len(),
            self.anis.borrow().len(),
            self.tils.borrow().len(),
        )
    }
}
