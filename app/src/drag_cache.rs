//! 用户显式拖出图片的持久副本；不计历史配额，清理须经界面确认。by AI.Coding
use localclip_core::domain::hash;
use std::{
    fs,
    io::{Read, Write},
    path::PathBuf,
};

pub struct DragCache {
    root: PathBuf,
}
pub struct Export {
    pub path: PathBuf,
    pub created: bool,
}
impl DragCache {
    /// 每个数据目录独立持有缓存，禁止采用未知目录或重解析点。
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }
    fn owned(&self, create: bool) -> std::io::Result<bool> {
        if !self.root.exists() {
            if !create {
                return Ok(false);
            }
            fs::create_dir(&self.root)?;
            fs::write(
                self.root.join(".localclip-drag-cache"),
                b"LocalClip image exports v1",
            )?;
        }
        if !regular_directory(&self.root)
            || fs::read(self.root.join(".localclip-drag-cache"))? != b"LocalClip image exports v1"
        {
            return Err(std::io::Error::other("unownedCache"));
        }
        Ok(true)
    }
    /// 按原字节去重；外部编辑后的副本保留，新拖出另建独立文件。
    pub fn prepare(&self, bytes: &[u8], extension: &str) -> std::io::Result<Export> {
        if !["png", "jpg", "bmp"].contains(&extension) {
            return Err(std::io::Error::other("unsupported"));
        }
        self.owned(true)?;
        let digest = hash(bytes);
        for index in 0..10000 {
            let path = self.root.join(format!("{digest}-{index}.{extension}"));
            if let Ok(meta) = fs::symlink_metadata(&path) {
                if regular_file(&meta) && meta.len() == bytes.len() as u64 {
                    let mut saved = Vec::new();
                    fs::File::open(&path)?
                        .take(bytes.len() as u64 + 1)
                        .read_to_end(&mut saved)?;
                    if saved == bytes {
                        return Ok(Export {
                            path,
                            created: false,
                        });
                    }
                }
                continue;
            }
            match fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
            {
                Ok(mut file) => {
                    if let Err(e) = file.write_all(bytes).and_then(|_| file.sync_all()) {
                        drop(file);
                        let _ = fs::remove_file(&path);
                        return Err(e);
                    }
                    return Ok(Export {
                        path,
                        created: true,
                    });
                }
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(e) => return Err(e),
            }
        }
        Err(std::io::Error::other("cacheFull"))
    }
    /// 取消时只收回本次新建文件；成功拖出和复用副本长期保留。
    pub fn cancel(&self, export: &Export) {
        if export.created && export.path.parent() == Some(self.root.as_path()) {
            let _ = fs::remove_file(&export.path);
        }
    }
    /// 不递归、不跟随链接，只枚举本应用命名的普通文件。
    fn files(&self) -> std::io::Result<Vec<(PathBuf, u64)>> {
        if !self.owned(false)? {
            return Ok(Vec::new());
        }
        let mut files = Vec::new();
        for entry in fs::read_dir(&self.root)? {
            let Ok(entry) = entry else { continue };
            let name = entry.file_name();
            let name = name.to_string_lossy();
            let Some((stem, ext)) = name.rsplit_once('.') else {
                continue;
            };
            let Some((digest, index)) = stem.split_once('-') else {
                continue;
            };
            if digest.len() != 64
                || !digest.bytes().all(|b| b.is_ascii_hexdigit())
                || index.parse::<u32>().is_err()
                || !["png", "jpg", "bmp"].contains(&ext)
            {
                continue;
            }
            if let Ok(meta) = fs::symlink_metadata(entry.path())
                && regular_file(&meta)
            {
                files.push((entry.path(), meta.len()));
            }
        }
        Ok(files)
    }
    pub fn bytes(&self) -> std::io::Result<u64> {
        Ok(self.files()?.iter().map(|(_, n)| n).sum())
    }
    /// 单项失败不阻止其他副本清理，但最终保留错误提示。
    pub fn clear(&self) -> std::io::Result<()> {
        let mut error = None;
        for (path, _) in self.files()? {
            if let Err(e) = fs::remove_file(path) {
                error = Some(e);
            }
        }
        error.map_or(Ok(()), Err)
    }
}
fn regular_file(meta: &fs::Metadata) -> bool {
    meta.is_file() && !reparse(meta)
}
fn regular_directory(path: &std::path::Path) -> bool {
    fs::symlink_metadata(path).is_ok_and(|m| m.is_dir() && !reparse(&m))
}
fn reparse(meta: &fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        meta.file_attributes() & 0x400 != 0
    }
    #[cfg(not(windows))]
    {
        meta.is_symlink()
    }
}
