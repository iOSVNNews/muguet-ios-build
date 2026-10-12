#[cfg(target_os = "ios")]
pub fn install() {
    use std::os::fd::AsRawFd;
    let Some(home) = std::env::var_os("HOME") else { return };
    let dir = std::path::PathBuf::from(home).join("Documents");
    let path = dir.join("muguet-runtime.log");
    if path.exists() { let _ = std::fs::rename(&path, dir.join("muguet-previous.log")); }
    if let Ok(file) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
        extern "C" { fn dup2(from: i32, to: i32) -> i32; }
        unsafe { dup2(file.as_raw_fd(), 2); }
        eprintln!("[diagnostics] build {} | device RAM {} MB | available {} MB", env!("CARGO_PKG_VERSION"),
            crate::performance::physical_memory() >> 20, crate::performance::available_memory().unwrap_or(0) >> 20);
    }
}

#[cfg(not(target_os = "ios"))]
pub fn install() {}

pub fn trim() {
    #[cfg(target_os = "ios")]
    if let Some(home) = std::env::var_os("HOME") {
        let path = std::path::PathBuf::from(home).join("Documents/muguet-runtime.log");
        if std::fs::metadata(&path).map(|m| m.len() > 4 << 20).unwrap_or(false) {
            // stderr is opened O_APPEND, so it stays usable after truncation.
            if let Ok(file) = std::fs::OpenOptions::new().write(true).open(path) { let _ = file.set_len(0); }
        }
    }
}
