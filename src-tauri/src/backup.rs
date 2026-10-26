//! Save backups.
//!
//! The original editor copied the *entire folder* containing `main.dat` on every load, so opening
//! a save from e.g. Downloads produced backups of tens of gigabytes that could not be deleted.
//! Here only the save files themselves are archived, compressed, into the app's data folder,
//! duplicates are skipped and old backups are pruned automatically.
use nhse_core::save::SaveSource;
use serde::Serialize;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize)]
pub struct BackupInfo {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub modified: String,
}

fn fingerprint(src: &SaveSource) -> nhse_core::Result<u64> {
    // cheap content hash of the encrypted files (FNV-1a over main.dat + headers)
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for f in ["mainHeader.dat", "main.dat"] {
        for b in src.read(f)?.iter().step_by(61) {
            h ^= *b as u64;
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
    }
    Ok(h)
}

/// Creates a compressed backup of the save files. Returns the backup path, or `None` when an
/// identical backup already exists.
pub fn create(
    dir: &Path,
    src: &SaveSource,
    label: &str,
    keep: usize,
) -> nhse_core::Result<Option<PathBuf>> {
    std::fs::create_dir_all(dir)?;
    let fp = fingerprint(src)?;
    let tag = format!("{fp:016x}");
    let safe = nhse_core::util::strings::clean_file_name(label);
    // skip if the most recent backup of this island has the same fingerprint
    if list(dir).iter().any(|b| b.name.contains(&tag)) {
        return Ok(None);
    }
    let stamp = chrono::Local::now().format("%Y-%m-%d %H.%M.%S");
    let path = dir.join(format!("{safe} — {stamp} [{tag}].zip"));
    let tmp = path.with_extension("zip.part");
    {
        let f = std::fs::File::create(&tmp)?;
        let mut z = zip::ZipWriter::new(f);
        let opts = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .compression_level(Some(6));
        for rel in src.save_files()? {
            z.start_file(rel.as_str(), opts)?;
            z.write_all(&src.read(&rel)?)?;
        }
        z.finish()?;
    }
    std::fs::rename(&tmp, &path)?;
    prune(dir, &safe, keep);
    Ok(Some(path))
}

fn prune(dir: &Path, prefix: &str, keep: usize) {
    if keep == 0 {
        return;
    }
    let mut mine: Vec<BackupInfo> = list(dir)
        .into_iter()
        .filter(|b| b.name.starts_with(prefix))
        .collect();
    mine.sort_by(|a, b| b.modified.cmp(&a.modified));
    for old in mine.into_iter().skip(keep) {
        let _ = std::fs::remove_file(&old.path);
    }
}

pub fn list(dir: &Path) -> Vec<BackupInfo> {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut v: Vec<BackupInfo> = rd
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|x| x == "zip"))
        .filter_map(|e| {
            let md = e.metadata().ok()?;
            let modified: chrono::DateTime<chrono::Local> = md.modified().ok()?.into();
            Some(BackupInfo {
                name: e.file_name().to_string_lossy().to_string(),
                path: e.path().to_string_lossy().to_string(),
                size: md.len(),
                modified: modified.format("%Y-%m-%d %H:%M:%S").to_string(),
            })
        })
        .collect();
    v.sort_by(|a, b| b.modified.cmp(&a.modified));
    v
}

pub fn delete(dir: &Path, path: &str) -> nhse_core::Result<()> {
    let p = PathBuf::from(path);
    // only allow deleting inside the backup folder
    let canon_dir = dir.canonicalize()?;
    let canon = p.canonicalize()?;
    if !canon.starts_with(&canon_dir) {
        return Err(nhse_core::Error::Invalid(
            "refusing to delete a file outside the backup folder".into(),
        ));
    }
    std::fs::remove_file(canon)?;
    Ok(())
}
