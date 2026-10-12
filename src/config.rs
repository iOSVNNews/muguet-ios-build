// muguet - config.rs
// Constantin Clerc (c22dev) - 0.1

use std::path::{Path, PathBuf};

#[cfg_attr(target_os = "ios", allow(dead_code))]
pub struct Startup {
    pub wasm_path: PathBuf,
    pub cache_dir: PathBuf,

    pub engine_args: Vec<String>,
}

const DEFAULT_CONFIG: &str = r#"{
  "scale": 0.5,
  "mouse_sensitivity": 2.0,
  "fullscreen": true,
  "fps_limit": 60,
  "start_mode": 2,
  "mute": false,
  "multiplayer": false,
  "host": false,
  "server": "",
  "player_name": "",
  "extra_args": []
}
"#;

fn support_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    if cfg!(target_os = "ios") {
        return PathBuf::from(home).join("Documents");
    }
    PathBuf::from(home).join("Library/Application Support/Muguet")
}

pub fn game_dir() -> PathBuf {
    let dir = support_dir().join("game");
    let old = dir.join("gamedat");
    if !dir.join("data").exists() && old.join("data/manifest.json").exists() { old } else { dir }
}

pub fn game_ready() -> bool {
    let dir = game_dir();
    dir.join("data/manifest.json").exists() && build_dir(&dir).is_some_and(|b| !cfg!(feature = "jit") || crate::code_embedded() || b.join("game.wasm").exists())
}

pub fn bundled_resources() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let res = exe.parent()?.parent()?.join("Resources");
    (exe.parent()?.ends_with("Contents/MacOS") && res.is_dir()).then_some(res)
}

fn build_dir(root: &Path) -> Option<PathBuf> {
    let mut builds: Vec<_> = std::fs::read_dir(root.join("b")).ok()?.flatten().map(|e| e.path())
        .filter(|p| p.join("shaders/index.json").is_file()).collect();
    builds.sort();
    builds.into_iter().next()
}

pub fn validate_game_files() -> anyhow::Result<()> {
    validate_game_files_with_progress(|_, _| {})
}

pub fn validate_game_files_with_progress(progress: impl FnMut(usize, usize)) -> anyhow::Result<()> {
    use anyhow::Context;
    let root = game_dir();
    let data_manifest = root.join("data/manifest.json");
    anyhow::ensure!(data_manifest.is_file(), "Game data is missing. Import a complete compatible game.zip");
    // Validate old imports too, before streaming starts, and give HTTPFS the
    // reconciled manifest rather than the stale sizes in the original ZIP.
    let manifest = crate::game_data::load_manifest_with_progress(&root.join("data"), progress)?;
    let build = build_dir(&root).context("Missing shader pack")?;
    let index = build.join("shaders/index.json");
    anyhow::ensure!(std::fs::metadata(&index)?.len() <= 64 << 20, "Shader index is too large");
    let shaders: serde_json::Value = serde_json::from_slice(&std::fs::read(index)?).context("Invalid shader index")?;
    anyhow::ensure!(shaders.as_object().is_some_and(|o| !o.is_empty()), "Shader pack is empty");
    let _ = crate::game_data::RUNTIME_MANIFEST.set(std::sync::Arc::from(serde_json::to_vec(&manifest)?));
    eprintln!("[config] verified game manifest and shader pack");
    Ok(())
}

fn set_default(key: &str, value: impl AsRef<std::ffi::OsStr>) {
    if std::env::var_os(key).is_none() {
        std::env::set_var(key, value);
    }
}

fn redirect_logs_if_detached() {
    extern "C" {
        fn isatty(fd: i32) -> i32;
        fn dup2(old: i32, new: i32) -> i32;
    }

    if cfg!(target_os = "ios") {
        return;
    }
    let in_bundle = std::env::current_exe().map(|p| p.to_string_lossy().contains(".app/Contents/MacOS/")).unwrap_or(false);
    if !in_bundle || unsafe { isatty(2) } != 0 {
        return;
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    let path = PathBuf::from(home).join("Library/Logs/Muguet.log");
    if let Ok(f) = std::fs::File::create(&path) {
        use std::os::fd::IntoRawFd;
        let fd = f.into_raw_fd();
        unsafe {
            dup2(fd, 2);
            dup2(fd, 1);
        }
    }
}

pub fn setup() -> anyhow::Result<Startup> {
    redirect_logs_if_detached();
    let support = support_dir();
    std::fs::create_dir_all(support.join("saves"))?;

    let cfg_path = support.join("config.json");
    if !cfg_path.exists() {
        std::fs::write(&cfg_path, DEFAULT_CONFIG)?;
    }
    let engine_args = settings(&cfg_path);
    #[cfg(target_os = "ios")]
    crate::touch::update_settings_from_env();

    set_default("MUGUET_FSROOT", support.join("saves"));
    let build = locate();
    #[cfg(not(target_os = "ios"))]
    let build = build?;
    #[cfg(target_os = "ios")]
    let build = build.unwrap_or_default();
    Ok(Startup { wasm_path: build.join("game.wasm"), cache_dir: support, engine_args })
}

fn locate() -> anyhow::Result<PathBuf> {
    let root = game_dir();
    let build = build_dir(&root).filter(|_| game_ready()).ok_or_else(|| anyhow::anyhow!("no game data: import your game .zip from the launcher"))?;
    std::env::set_var("MUGUET_DATA", root.join("data"));
    std::env::set_var("MUGUET_SHADERS", build.join("shaders"));
    eprintln!("[config] game {}", root.display());
    Ok(build)
}

const SETTING_VARS: &[&str] = &[
    "MUGUET_SCALE", "MUGUET_MOUSE_SENS", "MUGUET_TOUCH_SENS", "MUGUET_AIM_SENS", "MUGUET_AIM_MODE", "MUGUET_FIRE_AIM",
    "MUGUET_RES", "MUGUET_FULLSCREEN", "MUGUET_MUTE", "MUGUET_HOST", "MUGUET_SERVER", "MUGUET_ROOM", "MUGUET_NAME", "MUGUET_PVP",
    "MUGUET_MODE", "MUGUET_PROMOTION_SYNC", "MUGUET_DEEP_OPTIMIZATION", "MUGUET_BUTTON_SCALE", "MUGUET_BUTTON_OFFSET_X", "MUGUET_BUTTON_OFFSET_Y",
    "MUGUET_BUTTON_OPACITY", "MUGUET_STICK_OFFSET_X", "MUGUET_STICK_OFFSET_Y"
];

#[no_mangle]
pub extern "C" fn muguet_reload_settings() {
    #[cfg(target_os = "ios")]
    {
        let _ = reload();
    }
}

#[cfg(target_os = "ios")]
pub fn reload() -> Vec<String> {
    for v in SETTING_VARS { std::env::remove_var(v); }
    let args = settings(&support_dir().join("config.json"));
    if let Err(e) = locate() {
        eprintln!("[config] {e:#}");
    }
    crate::touch::update_settings_from_env();
    eprintln!("[config] launcher settings applied: {}", args.join(" "));
    args
}

fn settings(cfg_path: &Path) -> Vec<String> {
    let _ = SETTING_VARS;
    let cfg: serde_json::Value = std::fs::read_to_string(&cfg_path)
        .ok()
        .and_then(|t| serde_json::from_str(&t).map_err(|e| eprintln!("[config] {}: {e}", cfg_path.display())).ok())
        .unwrap_or_else(|| serde_json::from_str(DEFAULT_CONFIG).unwrap());
    let num = |k: &str| cfg.get(k).and_then(|v| v.as_f64());
    let flag = |k: &str| cfg.get(k).and_then(|v| v.as_bool()).unwrap_or(false);
    if let Some(v) = num("scale") { set_default("MUGUET_SCALE", v.to_string()); }
    if let Some(v) = num("mouse_sensitivity") { set_default("MUGUET_MOUSE_SENS", v.to_string()); }
    if let Some(v) = num("touch_sensitivity") { set_default("MUGUET_TOUCH_SENS", v.to_string()); }
    set_default("MUGUET_AIM_SENS", num("aim_sensitivity").unwrap_or(0.8).clamp(0.1, 5.0).to_string());
    set_default("MUGUET_AIM_MODE", cfg.get("aim_mode").and_then(|v| v.as_str()).unwrap_or("toggle"));
    set_default("MUGUET_FIRE_AIM", if cfg.get("fire_aim").and_then(|v| v.as_bool()).unwrap_or(true) { "1" } else { "0" });
    if let Some(r) = cfg.get("resolution").and_then(|v| v.as_str()) { set_default("MUGUET_RES", r); }
    if flag("fullscreen") { set_default("MUGUET_FULLSCREEN", "1"); }
    if flag("mute") { set_default("MUGUET_MUTE", "1"); }

    if let Some(v) = num("button_scale") { set_default("MUGUET_BUTTON_SCALE", v.to_string()); }
    if let Some(v) = num("button_offset_x") { set_default("MUGUET_BUTTON_OFFSET_X", v.to_string()); }
    if let Some(v) = num("button_offset_y") { set_default("MUGUET_BUTTON_OFFSET_Y", v.to_string()); }
    if let Some(v) = num("button_opacity") { set_default("MUGUET_BUTTON_OPACITY", v.to_string()); }
    if let Some(v) = num("stick_offset_x") { set_default("MUGUET_STICK_OFFSET_X", v.to_string()); }
    if let Some(v) = num("stick_offset_y") { set_default("MUGUET_STICK_OFFSET_Y", v.to_string()); }

    let deep_optimization = cfg.get("deep_optimization").and_then(|v| v.as_bool()).unwrap_or(false);
    set_default("MUGUET_DEEP_OPTIMIZATION", if deep_optimization { "1" } else { "0" });

    let promotion_sync = cfg.get("promotion_sync").and_then(|v| v.as_bool()).unwrap_or(false);
    crate::performance::set_promotion_sync(promotion_sync);
    set_default("MUGUET_PROMOTION_SYNC", if promotion_sync { "1" } else { "0" });

    let text = |k: &str| cfg.get(k).and_then(|v| v.as_str()).map(str::trim).filter(|v| !v.is_empty());
    if flag("multiplayer") {
        if flag("host") {
            set_default("MUGUET_HOST", "1");
        } else {
            let srv = text("server").unwrap_or("18.143.103.253");
            set_default("MUGUET_SERVER", srv);
        }
    }
    if let Some(v) = text("room") {
        let pass = text("room_pass").unwrap_or("");
        let full_room = if pass.is_empty() {
            v.to_string()
        } else {
            format!("{v}#{pass}")
        };
        set_default("MUGUET_ROOM", full_room);
    }
    if let Some(v) = text("player_name") { set_default("MUGUET_NAME", v); }
    set_default("MUGUET_PVP", if flag("pvp") { "1" } else { "0" });

    let mut engine_args = Vec::new();

    let fps = std::env::var("MUGUET_FPS").ok().and_then(|v| v.parse::<f64>().ok()).or_else(|| num("fps_limit")).unwrap_or(60.0);
    crate::performance::set_frame_limit(fps);
    engine_args.push(format!("-frameLimit={}", crate::performance::engine_frame_limit(fps)));

    let start_mode = std::env::var("MUGUET_START").ok().and_then(|v| v.parse::<f64>().ok()).or_else(|| num("start_mode")).unwrap_or(2.0) as i64;
    let start_mode = if (1..=2).contains(&start_mode) { start_mode } else { 2 };
    let mode_name = if start_mode == 1 { "story" } else { "sandbox" };
    set_default("MUGUET_MODE", mode_name);
    if !std::env::args().any(|a| a.starts_with("-wasmstartchoice")) {
        engine_args.push(format!("-wasmstartchoice={start_mode}"));
    }
    if let Some(extra) = cfg.get("extra_args").and_then(|v| v.as_array()) {
        engine_args.extend(extra.iter().filter_map(|a| a.as_str().map(String::from)));
    }
    fix_settings(num("texture_quality").map(|q| q as i64), deep_optimization);
    engine_args
}

const DEFAULT_SAFE_SETTINGS_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<Settings>
  <version value="27" />
  <configSource>SMC_AUTO</configSource>
  <graphics>
    <TextureQuality value="0" />
    <ShaderQuality value="1" />
    <ShadowQuality value="1" />
    <ReflectionQuality value="1" />
    <ReflectionMSAA value="0" />
    <SSAO value="0" />
    <AnisotropicFiltering value="0" />
    <MSAA value="0" />
    <MSAAFragments value="0" />
    <MSAAPostProcess value="0" />
    <WaterQuality value="0" />
    <ParticleQuality value="0" />
    <GrassQuality value="0" />
    <DeepSurfaceQuality value="0" />
    <PedLodBias value="0.200000" />
    <VehicleLodBias value="0.000000" />
    <Shadow_Distance value="1.000000" />
    <Shadow_SoftShadows value="0" />
    <Shadow_ParticleShadows value="false" />
    <Shadow_SplitZStart value="0.930000" />
    <Shadow_SplitZEnd value="0.890000" />
    <Shadow_aircraftExpBloom value="0.300000" />
    <Shadow_DisableScreenSizeCheck value="false" />
    <Reflection_MipBlur value="true" />
    <FXAA value="true" />
    <TXAA value="false" />
    <Lighting_FogVolumes value="false" />
    <Shader_SSA value="false" />
    <DX_Version value="2" />
    <CityDensity value="0.000000" />
    <PedVarietyMultiplier value="0.000000" />
    <VehicleVarietyMultiplier value="0.000000" />
    <PostFX value="0" />
    <DoF value="false" />
    <HdStreamingInFlight value="false" />
    <MaxLodScale value="0.000000" />
    <MotionBlurStrength value="0.000000" />
  </graphics>
  <system>
    <Windowed value="2" />
  </system>
  <video>
    <Windowed value="2" />
    <VSync value="1" />
  </video>
</Settings>
"#;

fn find_all_settings_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if !dir.is_dir() { return out; }
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                out.extend(find_all_settings_files(&p));
            } else if p.file_name().and_then(|n| n.to_str()) == Some("settings.xml") {
                out.push(p);
            }
        }
    }
    out
}

#[no_mangle]
pub extern "C" fn muguet_reset_game_settings() -> bool {
    let base = support_dir().join("saves");
    let mut modified = false;
    let paths = find_all_settings_files(&base);
    for p in paths {
        let _ = std::fs::remove_file(&p);
        modified = true;
    }
    let default_dir = base.join("userdata/Documents/Rockstar Games/GTA V");
    let _ = std::fs::create_dir_all(&default_dir);
    let _ = std::fs::write(default_dir.join("settings.xml"), DEFAULT_SAFE_SETTINGS_XML);
    eprintln!("[config] GTA V settings.xml reset to safe defaults");
    modified
}

const LOW_RAM_POTATO_SETTINGS_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<Settings>
  <version value="27" />
  <configSource>SMC_AUTO</configSource>
  <graphics>
    <TextureQuality value="0" />
    <ShaderQuality value="1" />
    <ShadowQuality value="0" />
    <ReflectionQuality value="0" />
    <ReflectionMSAA value="0" />
    <SSAO value="0" />
    <AnisotropicFiltering value="0" />
    <MSAA value="0" />
    <MSAAFragments value="0" />
    <MSAAPostProcess value="0" />
    <WaterQuality value="0" />
    <ParticleQuality value="0" />
    <GrassQuality value="0" />
    <DeepSurfaceQuality value="0" />
    <PedLodBias value="-1.000000" />
    <VehicleLodBias value="-1.000000" />
    <Shadow_Distance value="0.000000" />
    <Shadow_SoftShadows value="0" />
    <Shadow_ParticleShadows value="false" />
    <Shadow_SplitZStart value="0.930000" />
    <Shadow_SplitZEnd value="0.890000" />
    <Shadow_aircraftExpBloom value="0.000000" />
    <Shadow_DisableScreenSizeCheck value="true" />
    <Reflection_MipBlur value="false" />
    <FXAA value="false" />
    <TXAA value="false" />
    <Lighting_FogVolumes value="false" />
    <Shader_SSA value="false" />
    <DX_Version value="2" />
    <CityDensity value="0.000000" />
    <PedVarietyMultiplier value="0.000000" />
    <VehicleVarietyMultiplier value="0.000000" />
    <PostFX value="0" />
    <DoF value="false" />
    <HdStreamingInFlight value="false" />
    <MaxLodScale value="0.000000" />
    <MotionBlurStrength value="0.000000" />
  </graphics>
  <system>
    <Windowed value="2" />
  </system>
  <video>
    <Windowed value="2" />
    <VSync value="1" />
  </video>
</Settings>
"#;

#[no_mangle]
pub extern "C" fn muguet_apply_low_ram_preset() -> bool {
    let base = support_dir().join("saves");
    let paths = find_all_settings_files(&base);
    for p in &paths {
        let _ = std::fs::write(p, LOW_RAM_POTATO_SETTINGS_XML);
    }
    let default_dir = base.join("userdata/Documents/Rockstar Games/GTA V");
    let _ = std::fs::create_dir_all(&default_dir);
    let _ = std::fs::write(default_dir.join("settings.xml"), LOW_RAM_POTATO_SETTINGS_XML);
    eprintln!("[config] applied Low-RAM Potato Mode preset (target ~800MB - 1GB RAM)");
    true
}

fn fix_settings(texture_quality: Option<i64>, deep_optimization: bool) {
    let base = support_dir().join("saves");
    let mut paths = find_all_settings_files(&base);
    let default_path = base.join("userdata/Documents/Rockstar Games/GTA V/settings.xml");
    if default_path.exists() && !paths.contains(&default_path) {
        paths.push(default_path);
    }
    for path in paths {
        let Ok(xml) = std::fs::read_to_string(&path) else { continue };
        let mut new = xml.clone();

        let mut set_num = |tag: &str, fix: &dyn Fn(i64) -> i64| {
            let re = regex::Regex::new(&format!(r#"<{tag}\s+value="(\d+)"\s*/>"#)).unwrap();
            if let Some(c) = re.captures(&new) {
                if let Ok(v) = c[1].parse::<i64>() {
                    let to = fix(v);
                    if to != v {
                        new = re.replace(&new, format!(r#"<{tag} value="{to}" />"#).as_str()).into_owned();
                        eprintln!("[config] {tag} {v} -> {to}");
                    }
                }
            }
        };

        let mut set_bool = |tag: &str, target: bool| {
            let re = regex::Regex::new(&format!(r#"<{tag}\s+value="(?:true|false|\d+)"\s*/>"#)).unwrap();
            let target_str = if target { "true" } else { "false" };
            if re.is_match(&new) {
                new = re.replace(&new, format!(r#"<{tag} value="{target_str}" />"#).as_str()).into_owned();
            }
        };

        // CRITICAL FIX: ShaderQuality MUST be 1 (High).
        // The shader pack (shaders/index.json) only contains WGSL shaders compiled for Quality 1.
        // Quality 2 (Very High) causes missing shaders -> pipeline returns None -> all draw calls skipped -> black screen!
        set_num("ShaderQuality", &|_| 1);

        // MSAA and TXAA must be disabled on iOS (causes massive VRAM spikes > 3GB and Metal attachment failures)
        set_num("MSAA", &|_| 0);
        set_num("MSAAFragments", &|_| 0);
        set_num("MSAAPostProcess", &|_| 0);
        set_num("ReflectionMSAA", &|_| 0);
        set_bool("TXAA", false);

        // Tessellation is unsupported and causes WebAssembly / Metal pipeline failures
        set_num("Tessellation", &|_| 0);

        // Cap texture quality to prevent OOM exceeding iOS 2.8GB app ceiling
        if let Some(q) = texture_quality {
            set_num("TextureQuality", &|_| q.clamp(0, 1));
        } else {
            set_num("TextureQuality", &|v| v.min(1));
        }

        // Disable heavy soft shadows and flying streaming
        set_num("Shadow_SoftShadows", &|_| 0);
        set_bool("DoF", false);
        set_bool("HdStreamingInFlight", false);

        if deep_optimization {
            set_num("ShadowQuality", &|v| v.min(1));
            set_num("GrassQuality", &|v| v.min(1));
            set_num("ReflectionQuality", &|v| v.min(1));
            set_num("WaterQuality", &|v| v.min(1));
            set_num("ParticleQuality", &|v| v.min(1));
            set_num("PostFX", &|v| v.min(1));
        } else {
            set_num("ShadowQuality", &|v| v.min(2));
            set_num("GrassQuality", &|v| v.min(2));
            set_num("ReflectionQuality", &|v| v.min(2));
            set_num("WaterQuality", &|v| v.min(1));
            set_num("PostFX", &|v| v.min(2));
        }

        if new != xml {
            let _ = std::fs::write(&path, new.as_bytes());
            eprintln!("[config] sanitized settings at {}", path.display());
        }
    }
}
