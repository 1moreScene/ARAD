//! # dnf-data —— DNF 官方客户端资源读取
//!
//! 无第三方依赖。覆盖白模→真实素材所需的全部格式：
//!
//! | 模块 | 格式 | 说明 |
//! |---|---|---|
//! | [`pvf`] | `Script.pvf` v1 | 官方「数据大脑」：角色/技能/地图脚本 |
//! | [`npk`] | `NeoplePack_Bill` NPK | `ImagePacks2/*.NPK` 图片包（目录名明文） |
//! | [`img`] | `Neople Image File` | 精灵图集（16 位 A1R5G5B5 / 32 位原始，无 zlib） |
//! | [`ani`] | `.ani` | 动画帧序列 + 毫秒时长 + 判定盒 |
//! | [`chr`] | `.chr` | 角色数值 / 动作映射 / 技能表 |
//! | [`til`] | `.til` | 瓦片条 + `[pass type]` 通行网格（14×30×16px） |
//! | [`mapfile`] | `.map` | 房间：瓦片拼接、视差率、背景层、物件、门坐标 |
//! | [`equ`] | `.equ` | 装备/时装：`[variation]` → 精灵模板、`[LAYER]`、动作映射 |
//! | [`avatar`] | `.equ` 部件表 | 官方绘制层（`[LAYER]`）+ 时装部件目录，角色外观分层合成 |

pub mod ani;
pub mod audio;
pub mod avatar;
pub mod chr;
pub mod combat;
pub mod equ;
pub mod img;
pub mod mapfile;
pub mod messages;
pub mod mob;
pub mod npk;
pub mod pvf;
pub mod skill;
pub mod strings;
pub mod til;

pub use ani::{AnimFile, AnimFrame};
pub use avatar::{AvatarItem, AvatarPart, AvatarTable};
pub use chr::ChrFile;
pub use equ::EquFile;
pub use img::{ImgFile, ImgFrame};
pub use mapfile::MapFile;
pub use npk::{NpkArchive, NpkError, NpkSet};
pub use pvf::{PvfEntry, PvfError, PvfReader};
pub use til::TilFile;

use std::path::{Path, PathBuf};

/// 本机官方客户端目录中 `Script.pvf` 的常用相对位置。
const CLIENT_SUBPATHS: [&str; 3] = [
    "40jp-client/40jp单机版",
    "../40jp-client/40jp单机版",
    "../../40jp-client/40jp单机版",
];

/// 定位官方客户端目录。
///
/// 顺序：环境变量 `DNF_CLIENT_DIR` → 编译期 crate 目录推导的若干相对路径 →
/// 当前工作目录下的相对路径。返回第一个含 `Script.pvf` 的目录。
pub fn locate_client_dir() -> Option<PathBuf> {
    if let Ok(dir) = std::env::var("DNF_CLIENT_DIR") {
        let p = PathBuf::from(dir);
        if p.join("Script.pvf").is_file() {
            return Some(p);
        }
    }

    let mut roots: Vec<PathBuf> = Vec::new();
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo = manifest.join("..").join("..").join("..");
    roots.push(repo.clone());
    if let Ok(cwd) = std::env::current_dir() {
        roots.push(cwd.clone());
        roots.push(cwd.join(".."));
        roots.push(cwd.join("..").join(".."));
    }

    for root in roots {
        for sub in CLIENT_SUBPATHS {
            let cand = normalize_path(&root.join(sub));
            if cand.join("Script.pvf").is_file() {
                return Some(cand);
            }
        }
    }
    None
}

/// 判断错误是否只是「官方客户端正在运行，素材文件被占用」。
///
/// 本工程**只读**官方素材（`Script.pvf` / `*.NPK`），所以出现「文件被另一个进程占用」
/// 必然是外部原因（例如玩家正开着官方 `ARAD.exe` 做对比）。测试遇到这种情况应当**跳过**
/// 而不是判失败。
pub fn client_busy(err: &str) -> bool {
    err.contains("另一个程序正在使用")
        || err.contains("being used by another process")
        || err.contains("os error 32")
}

/// 去掉路径里的 `..` / `.` 成分。
pub fn normalize_path(p: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for comp in p.components() {
        match comp {
            std::path::Component::ParentDir => {
                out.pop();
            }
            std::path::Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    out
}

/// 官方客户端资源目录下 `ImagePacks2/` 的路径。
pub fn imagepacks_dir(client_dir: &Path) -> PathBuf {
    client_dir.join("ImagePacks2")
}

/// 测试与工具用的懒加载单例（只读，进程内缓存，避免每个测试重扫目录）。
#[cfg(test)]
pub(crate) mod test_support {
    use super::*;
    use std::sync::OnceLock;

    static PVF: OnceLock<Option<PvfReader>> = OnceLock::new();
    static PACKS: OnceLock<Option<NpkSet>> = OnceLock::new();

    pub fn pvf() -> Option<&'static PvfReader> {
        PVF.get_or_init(|| {
            let dir = locate_client_dir()?;
            PvfReader::open(&dir.join("Script.pvf")).ok()
        })
        .as_ref()
    }

    pub fn imagepacks() -> Option<&'static NpkSet> {
        PACKS
            .get_or_init(|| {
                let dir = locate_client_dir()?;
                NpkSet::scan_dir(&imagepacks_dir(&dir)).ok()
            })
            .as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 能定位官方客户端目录() {
        match locate_client_dir() {
            Some(dir) => {
                assert!(dir.join("Script.pvf").is_file(), "{:?}", dir);
                eprintln!("客户端目录: {:?}", dir);
            }
            None => eprintln!("未找到官方客户端（跳过）"),
        }
    }
}
