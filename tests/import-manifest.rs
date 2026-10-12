#[path = "../src/import.rs"]
mod import;
#[path = "../src/game_data.rs"]
mod game_data;

use std::io::Write;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(0);

fn fixture() -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!("muguet-manifest-fixture-{}-{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
    std::fs::create_dir_all(&root).unwrap();
    root
}

#[test]
fn legacy_html_size_is_reconciled_without_changing_the_source_manifest() {
    let data = fixture();
    let html = "common/non_final/www/parser/psomismatch.html";
    let original = serde_json::json!({"files": [["required.rpf",4], [html,100]]});
    std::fs::write(data.join("manifest.json"), original.to_string()).unwrap();
    std::fs::write(data.join("required.rpf"), b"RPF7").unwrap();
    std::fs::create_dir_all(data.join("common/non_final/www/parser")).unwrap();
    std::fs::write(data.join(html), b"<html/>").unwrap();
    let normalized = game_data::load_manifest(&data).unwrap();
    assert_eq!(normalized["files"][1][1], 7);
    assert_eq!(serde_json::from_slice::<serde_json::Value>(&std::fs::read(data.join("manifest.json")).unwrap()).unwrap(), original);
    std::fs::remove_dir_all(data).unwrap();
}

#[test]
fn an_archive_without_non_final_web_docs_still_imports() {
    let root = fixture();
    let file = std::fs::File::create(root.join("game.zip")).unwrap();
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default();
    for (name, bytes) in [
        ("data/manifest.json", br#"{"files":[["required.rpf",4],["common/non_final/www/parser/psomismatch.html",100]]}"#.as_slice()),
        ("data/required.rpf", b"RPF7".as_slice()),
        ("b/test/shaders/index.json", br#"{"shader":{}}"#.as_slice()),
    ] {
        zip.start_file(name, options).unwrap();
        zip.write_all(bytes).unwrap();
    }
    zip.finish().unwrap();
    let dest = root.join("game");
    import::import(&root.join("game.zip"), &dest, false, |_, _| {}).unwrap();
    let normalized = game_data::load_manifest(&dest.join("data")).unwrap();
    assert_eq!(normalized["files"].as_array().unwrap().len(), 1);
    assert_eq!(std::fs::read(dest.join("data/required.rpf")).unwrap(), b"RPF7");
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn truncated_runtime_assets_and_unsafe_paths_are_still_rejected() {
    let data = fixture();
    for name in ["required.rpf", "common/non_final/www/parser/required.rpf", "common/data/handling.meta"] {
        let path = data.join(name);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, b"bad").unwrap();
        std::fs::write(data.join("manifest.json"), serde_json::json!({"files": [[name,4]]}).to_string()).unwrap();
        assert!(game_data::load_manifest(&data).unwrap_err().to_string().contains("Game data incomplete"));
    }
    std::fs::write(data.join("manifest.json"), r#"{"files":[["../common/non_final/www/page.html",1]]}"#).unwrap();
    assert!(game_data::load_manifest(&data).unwrap_err().to_string().contains("Invalid game data path"));
    std::fs::remove_dir_all(data).unwrap();
}

#[test]
fn an_incomplete_archive_preserves_the_previous_game() {
    let root = std::env::temp_dir().join(format!("muguet-import-test-{}-{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
    std::fs::create_dir_all(&root).unwrap();
    let dest = root.join("game");
    std::fs::create_dir_all(&dest).unwrap();
    std::fs::write(dest.join("previous-game"), b"preserve me").unwrap();
    let file = std::fs::File::create(root.join("bad.zip")).unwrap();
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default();
    zip.start_file("data/manifest.json", options).unwrap();
    zip.write_all(br#"{"files":[["required.rpf",4]]}"#).unwrap();
    zip.start_file("b/test/shaders/index.json", options).unwrap();
    zip.write_all(br#"{"shader":{}}"#).unwrap();
    zip.finish().unwrap();
    let result = import::import(&root.join("bad.zip"), &dest, false, |_, _| {});
    assert!(result.unwrap_err().to_string().contains("Game data missing: required.rpf"));
    assert_eq!(std::fs::read(dest.join("previous-game")).unwrap(), b"preserve me");
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn a_zip_without_its_directory_preserves_installed_data_and_explains_recovery() {
    let root = fixture();
    let dest = root.join("game");
    std::fs::create_dir_all(&dest).unwrap();
    std::fs::write(dest.join("previous-game"), b"preserve me").unwrap();
    std::fs::write(root.join("truncated.zip"), b"PK\x03\x04truncated").unwrap();
    let error = import::import(&root.join("truncated.zip"), &dest, false, |_, _| {}).unwrap_err();
    assert!(error.to_string().contains("Download the complete game.zip"));
    assert_eq!(std::fs::read(dest.join("previous-game")).unwrap(), b"preserve me");
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn httpfs_and_native_io_use_identical_ids_after_optional_entries_are_removed() {
    let data = fixture();
    std::fs::write(data.join("manifest.json"),
        r#"{"files":[["common/non_final/www/missing.html",10],["required.rpf",4]]}"#).unwrap();
    std::fs::write(data.join("required.rpf"), b"RPF7").unwrap();
    let normalized = game_data::load_manifest(&data).unwrap();
    game_data::RUNTIME_MANIFEST.set(std::sync::Arc::from(serde_json::to_vec(&normalized).unwrap())).unwrap();
    let httpfs: serde_json::Value = serde_json::from_slice(&game_data::runtime_manifest(&data).unwrap()).unwrap();
    let io_files = game_data::runtime_files(&data).unwrap();
    assert_eq!(io_files.len(), 1);
    assert_eq!(io_files[0], ("required.rpf".to_owned(), 4));
    assert_eq!(httpfs["files"][0][0], io_files[0].0);
    assert_eq!(httpfs["files"][0][1], io_files[0].1);
    // The stale manifest on disk still has the optional file at ID 0.
    let old: serde_json::Value = serde_json::from_slice(&std::fs::read(data.join("manifest.json")).unwrap()).unwrap();
    assert_ne!(old["files"][0][0], httpfs["files"][0][0]);
    std::fs::remove_dir_all(data).unwrap();
}

#[test]
fn manifest_validation_reports_progress_without_skipping_required_files() {
    let data = fixture();
    std::fs::write(data.join("manifest.json"), r#"{"files":[["required.rpf",4]]}"#).unwrap();
    std::fs::write(data.join("required.rpf"), b"RPF7").unwrap();
    let mut progress = Vec::new();
    game_data::load_manifest_with_progress(&data, |done, total| progress.push((done, total))).unwrap();
    assert_eq!(progress.first(), Some(&(0, 1)));
    assert_eq!(progress.last(), Some(&(1, 1)));
    std::fs::remove_dir_all(data).unwrap();
}
