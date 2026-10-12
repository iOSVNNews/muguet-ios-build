use anyhow::{Context, Result};
use serde_json::Value;
use std::path::{Component, Path};
use std::sync::{Arc, OnceLock};

pub const SKIPPED_DIRS: &[&str] = &["env_test", "verityradio"];
pub static RUNTIME_MANIFEST: OnceLock<Arc<[u8]>> = OnceLock::new();

// HTTPFS and native I/O must use this exact snapshot: IDs are array indices.
pub fn runtime_manifest(data: &Path) -> Result<Arc<[u8]>> {
    if let Some(bytes) = RUNTIME_MANIFEST.get() { return Ok(bytes.clone()); }
    std::fs::read(data.join("manifest.json")).map(Arc::from).context("Cannot read runtime manifest")
}

pub fn runtime_files(data: &Path) -> Result<Vec<(String, u64)>> {
    let bytes = runtime_manifest(data)?;
    let manifest: Value = serde_json::from_slice(&bytes).context("Invalid runtime manifest")?;
    manifest.get("files").and_then(Value::as_array).context("Runtime manifest has no files")?
        .iter().map(|record| {
            let name = record.get(0).and_then(Value::as_str).context("Invalid runtime file name")?;
            let size = record.get(1).and_then(Value::as_u64).context("Invalid runtime file size")?;
            Ok((name.to_owned(), size))
        }).collect()
}

fn optional_web_document(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower.starts_with("common/non_final/www/")
        && [".html", ".htm", ".css", ".js"].iter().any(|ext| lower.ends_with(ext))
}

// Non-final web documentation is not game asset data. Packs may omit it or
// normalize its text/line endings, while leaving the original manifest sizes.
// Keep the manifest supplied to the engine consistent with the files on disk.
pub fn load_manifest(data: &Path) -> Result<Value> {
    load_manifest_with_progress(data, |_, _| {})
}

pub fn load_manifest_with_progress(data: &Path, mut progress: impl FnMut(usize, usize)) -> Result<Value> {
    let path = data.join("manifest.json");
    anyhow::ensure!(std::fs::metadata(&path)?.len() <= 64 << 20, "Game manifest is too large");
    let mut manifest: Value = serde_json::from_slice(&std::fs::read(path)?).context("Invalid game manifest")?;
    let files = manifest.get_mut("files").and_then(Value::as_array_mut).context("Game manifest has no files list")?;
    anyhow::ensure!(!files.is_empty(), "Game data is empty. Import a complete compatible game.zip");
    let mut kept = Vec::with_capacity(files.len());
    let mut adjusted = 0;
    let total = files.len();
    progress(0, total);
    for (index, mut record) in files.drain(..).enumerate() {
        if index % 512 == 0 { progress(index, total); }
        let name = record.get(0).and_then(Value::as_str).context("Invalid file in game manifest")?.to_owned();
        let size = record.get(1).and_then(Value::as_u64).context("Invalid size in game manifest")?;
        let relative = Path::new(&name);
        anyhow::ensure!(!name.is_empty() && relative.components().all(|c| matches!(c, Component::Normal(_))), "Invalid game data path: {name}");
        if name.split('/').any(|part| SKIPPED_DIRS.contains(&part.to_ascii_lowercase().as_str())) {
            adjusted += 1;
            continue;
        }
        let metadata = match std::fs::metadata(data.join(relative)) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound && optional_web_document(&name) => {
                adjusted += 1;
                continue;
            }
            Err(error) => return Err(error).with_context(|| format!("Game data missing: {name}")),
        };
        anyhow::ensure!(metadata.is_file(), "Game data is not a file: {name}");
        let trimmed_dlc = name.eq_ignore_ascii_case("common/data/dlcList.xml");
        if metadata.len() != size {
            anyhow::ensure!(optional_web_document(&name) || trimmed_dlc,
                "Game data incomplete: {name} (expected {size} bytes, found {} bytes). Import the complete game.zip again", metadata.len());
            record[1] = Value::from(metadata.len());
            adjusted += 1;
        }
        kept.push(record);
    }
    anyhow::ensure!(!kept.is_empty(), "Game data contains no runtime files. Import a complete compatible game.zip");
    *files = kept;
    progress(total, total);
    if adjusted != 0 { eprintln!("[config] reconciled {adjusted} optional or modified manifest entries"); }
    Ok(manifest)
}
