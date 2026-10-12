// muguet - input.rs
// Constantin Clerc (c22dev) - 0.1

use crate::RT;
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU64, AtomicU8, Ordering};
use std::sync::{LazyLock, Mutex};

static BLOCK: AtomicU64 = AtomicU64::new(0);
static MENU_TOUCH: AtomicBool = AtomicBool::new(false);
static SIZE: Mutex<(i32, i32)> = Mutex::new((1280, 720));
static HELD: LazyLock<Mutex<HashSet<String>>> = LazyLock::new(|| Mutex::new(HashSet::new()));

const MX: u64 = 0;
const MY: u64 = 1;
const DX: u64 = 2;
const DY: u64 = 3;
const DZ: u64 = 4;
const BTN: u64 = 5;
const FOCUS: u64 = 6;
const LOCKED: u64 = 7;
const HEAD: u64 = 8;
const TAIL: u64 = 9;

fn base() -> Option<*mut u8> {
    let b = BLOCK.load(Ordering::Acquire);
    if b == 0 { return None; }
    let (p, _) = crate::mem_base_len(&RT.get()?.mem);
    Some(unsafe { p.add(b as usize) })
}
fn word(i: u64) -> Option<&'static AtomicI32> {
    base().map(|p| unsafe { &*(p.add(256 + 4 * i as usize) as *const AtomicI32) })
}
fn ext(i: u64) -> Option<&'static AtomicI32> {
    base().map(|p| unsafe { &*(p.add(424 + 4 * i as usize) as *const AtomicI32) })
}
fn key_byte(vk: u8) -> Option<&'static AtomicU8> {
    base().map(|p| unsafe { &*(p.add(vk as usize) as *const AtomicU8) })
}

pub fn publish(block: u64) {
    BLOCK.store(block, Ordering::Release);
    let (w, h) = *SIZE.lock().unwrap();
    if let Some(e) = ext(2) { e.store(w, Ordering::SeqCst); }
    if let Some(e) = ext(3) { e.store(h, Ordering::SeqCst); }
    if let Some(f) = word(FOCUS) { f.store(1, Ordering::SeqCst); }
    if cfg!(target_os = "ios") {
        let pointer = MENU_TOUCH.load(Ordering::Relaxed);
        if let Some(m) = word(MX) { m.store(if pointer { w >> 1 } else { -9999 }, Ordering::SeqCst); }
        if let Some(m) = word(MY) { m.store(if pointer { h >> 1 } else { -9999 }, Ordering::SeqCst); }
        set_locked(!MENU_TOUCH.load(Ordering::Relaxed));
    } else {
        if let Some(m) = word(MX) { m.store(w >> 1, Ordering::SeqCst); }
        if let Some(m) = word(MY) { m.store(h >> 1, Ordering::SeqCst); }
    }
    if let Ok(mode) = std::env::var("MUGUET_START_MODE") {
        if let (Ok(m), Some(p)) = (mode.parse::<i32>(), base()) {
            unsafe { &*(p.add(440) as *const AtomicI32) }.store(m, Ordering::SeqCst);
        }
    }
    eprintln!("[input] engine input block at 0x{block:x}");
}

pub fn set_size(w: i32, h: i32) {
    *SIZE.lock().unwrap() = (w, h);
    if let Some(e) = ext(2) { e.store(w, Ordering::SeqCst); }
    if let Some(e) = ext(3) { e.store(h, Ordering::SeqCst); }
}
pub fn size() -> (i32, i32) {
    *SIZE.lock().unwrap()
}

pub fn set_focus(focused: bool) {
    if let Some(f) = word(FOCUS) { f.store(focused as i32, Ordering::SeqCst); }
    if !focused { release_all(); }
}
pub fn set_locked(locked: bool) {
    let pointer = MENU_TOUCH.load(Ordering::Relaxed);
    // The original browser locks its canvas on mousedown, including in menus.
    // Touch coordinates remain valid while the native window cursor is hidden.
    let locked = if cfg!(target_os = "ios") { true } else { locked };
    if let Some(l) = word(LOCKED) { l.store(locked as i32, Ordering::SeqCst); }
    if cfg!(target_os = "ios") && !pointer {
        if let Some(m) = word(MX) { m.store(-9999, Ordering::SeqCst); }
        if let Some(m) = word(MY) { m.store(-9999, Ordering::SeqCst); }
    }
}

pub fn set_menu_touch(enabled: bool) {
    if MENU_TOUCH.swap(enabled, Ordering::Relaxed) == enabled { return; }
    // Prevent a look delta / old click from leaking across menu transitions.
    for index in [DX, DY, DZ, BTN] {
        if let Some(value) = word(index) { value.store(0, Ordering::SeqCst); }
    }
    let (w, h) = size();
    if let Some(mx) = word(MX) { mx.store(if enabled { w >> 1 } else { -9999 }, Ordering::SeqCst); }
    if let Some(my) = word(MY) { my.store(if enabled { h >> 1 } else { -9999 }, Ordering::SeqCst); }
    set_locked(!enabled);
    eprintln!("[input] menu touch={enabled}, input block ready={}", BLOCK.load(Ordering::Acquire) != 0);
}
fn release_all() {
    HELD.lock().unwrap().clear();
    for i in 0..=255u8 { if let Some(k) = key_byte(i) { k.store(0, Ordering::SeqCst); } }
    if let Some(b) = word(BTN) { b.store(0, Ordering::SeqCst); }
}

fn vk(code: &str) -> Option<u8> {
    let v = match code {
        "Backspace" => 0x08, "Tab" => 0x09, "Enter" => 0x0D, "NumpadEnter" => 0xFD, "Pause" => 0x13, "CapsLock" => 0x14, "Escape" => 0x1B,
        "Space" => 0x20, "PageUp" => 0x21, "PageDown" => 0x22, "End" => 0x23, "Home" => 0x24, "ArrowLeft" => 0x25, "ArrowUp" => 0x26,
        "ArrowRight" => 0x27, "ArrowDown" => 0x28, "PrintScreen" => 0x2C, "Insert" => 0x2D, "Delete" => 0x2E, "MetaLeft" => 0x5B,
        "MetaRight" => 0x5C, "ContextMenu" => 0x5D, "NumpadMultiply" => 0x6A, "NumpadAdd" => 0x6B, "NumpadSubtract" => 0x6D,
        "NumpadDecimal" => 0x6E, "NumpadDivide" => 0x6F, "NumLock" => 0x90, "ScrollLock" => 0x91, "ShiftLeft" => 0xA0, "ShiftRight" => 0xA1,
        "ControlLeft" => 0xA2, "ControlRight" => 0xA3, "AltLeft" => 0xA4, "AltRight" => 0xA5, "Semicolon" => 0xBA, "Equal" => 0xBB,
        "Comma" => 0xBC, "Minus" => 0xBD, "Period" => 0xBE, "Slash" => 0xBF, "Backquote" => 0xC0, "BracketLeft" => 0xDB,
        "Backslash" => 0xDC, "BracketRight" => 0xDD, "Quote" => 0xDE, "IntlBackslash" => 0xE2,
        c => {
            if let Some(d) = c.strip_prefix("Digit") { return d.parse::<u8>().ok().filter(|d| *d < 10).map(|d| 0x30 + d); }
            if let Some(d) = c.strip_prefix("Numpad") { return d.parse::<u8>().ok().filter(|d| *d < 10).map(|d| 0x60 + d); }
            if let Some(k) = c.strip_prefix("Key") { let b = k.as_bytes(); return (b.len() == 1 && b[0].is_ascii_uppercase()).then(|| b[0]); }
            if let Some(f) = c.strip_prefix('F') { return f.parse::<u8>().ok().filter(|f| (1..=24).contains(f)).map(|f| 0x6F + f); }
            return None;
        }
    };
    Some(v)
}

fn dom_code(winit_name: &str) -> &str {
    match winit_name { "SuperLeft" => "MetaLeft", "SuperRight" => "MetaRight", n => n }
}

const FLY_ALIAS: [(&str, &str); 6] = [("KeyI", "Numpad8"), ("KeyK", "Numpad2"), ("KeyJ", "Numpad4"), ("KeyL", "Numpad6"), ("KeyU", "Numpad7"), ("KeyO", "Numpad9")];

static PICKING_MAP: AtomicBool = AtomicBool::new(false);
pub fn start_mode() -> i32 {
    base().map(|p| unsafe { &*(p.add(440) as *const AtomicI32) }.load(Ordering::SeqCst)).unwrap_or(0)
}
fn start_menu_key(code: &str) {
    let Some(p) = base() else { return };
    let word = unsafe { &*(p.add(440) as *const AtomicI32) };
    if word.load(Ordering::SeqCst) != 0 { return; }
    let picking = PICKING_MAP.load(Ordering::SeqCst);
    let choice = match code {
        "Enter" | "NumpadEnter" if !picking => 1,
        "Space" => { PICKING_MAP.store(true, Ordering::SeqCst); 0 }
        "Backspace" => { PICKING_MAP.store(false, Ordering::SeqCst); 0 }
        "Digit5" | "Numpad5" => 2,
        _ => 0,
    };
    if choice != 0 {
        word.store(choice, Ordering::SeqCst);
        eprintln!("[input] start mode chosen: {}", ["", "Story Mode", "Sandbox"][choice as usize]);
    }
}

pub fn key(winit_name: &str, pressed: bool, repeat: bool, text: Option<&str>, ctrl_or_cmd: bool) {
    let code = dom_code(winit_name);
    if pressed && !repeat { start_menu_key(code); }

    let is_alias = FLY_ALIAS.iter().any(|(src, _)| *src == code);
    let v = if is_alias { None } else { vk(code) };
    if let Some(v) = v { if let Some(k) = key_byte(v) { k.store(if pressed { 0x80 } else { 0 }, Ordering::SeqCst); } }
    {
        let mut held = HELD.lock().unwrap();
        if pressed { held.insert(code.to_string()); } else { held.remove(code); }
        for (src, target) in FLY_ALIAS {
            let down = held.contains(src) || held.contains(target);
            if let Some(k) = vk(target).and_then(key_byte) { k.store(if down { 0x80 } else { 0 }, Ordering::SeqCst); }
        }
    }
    if !pressed { return; }
    let _ = repeat;
    let ch: Option<u16> = match text {
        Some(t) if t.chars().count() == 1 && !ctrl_or_cmd && !t.chars().next().unwrap().is_control() => t.encode_utf16().next(),
        _ if code == "Enter" => Some(13),
        _ if code == "Backspace" => Some(8),
        _ => None,
    };
    if let (Some(ch), Some(head), Some(tail), Some(b)) = (ch, word(HEAD), word(TAIL), base()) {
        let h = head.load(Ordering::SeqCst);
        if (h.wrapping_sub(tail.load(Ordering::SeqCst)) as u32) < 64 {
            unsafe { std::ptr::write_volatile(b.add(296 + 2 * (h & 63) as usize) as *mut u16, ch) };
            head.store(h.wrapping_add(1), Ordering::SeqCst);
        }
    }
}

pub fn button(dom_button: usize, pressed: bool) {
    let bit = 1 << [0, 2, 1, 3, 4].get(dom_button).copied().unwrap_or(0);
    if let Some(b) = word(BTN) {
        if pressed { b.fetch_or(bit, Ordering::SeqCst); } else { b.fetch_and(!bit, Ordering::SeqCst); }
    }
}

pub fn mouse_pos(x: i32, y: i32) {
    if cfg!(target_os = "ios") && !MENU_TOUCH.load(Ordering::Relaxed) {
        if let Some(mx) = word(MX) { mx.store(-9999, Ordering::SeqCst); }
        if let Some(my) = word(MY) { my.store(-9999, Ordering::SeqCst); }
        return;
    }
    let (w, h) = size();
    let (x, y) = (x.clamp(0, (w - 1).max(0)), y.clamp(0, (h - 1).max(0)));
    if let (Some(mx), Some(my), Some(dx), Some(dy)) = (word(MX), word(MY), word(DX), word(DY)) {
        let (old_x, old_y) = (mx.load(Ordering::SeqCst), my.load(Ordering::SeqCst));
        // Match browser mousemove: the engine consumes both absolute position
        // and relative movement. Never subtract the hidden-cursor sentinel.
        if old_x >= 0 && old_y >= 0 {
            dx.fetch_add(x - old_x, Ordering::SeqCst);
            dy.fetch_add(y - old_y, Ordering::SeqCst);
        }
        mx.store(x, Ordering::SeqCst);
        my.store(y, Ordering::SeqCst);
    }
}

static SENS: LazyLock<f64> = LazyLock::new(|| std::env::var("MUGUET_MOUSE_SENS").ok().and_then(|v| v.parse().ok()).unwrap_or(2.0));
static FRAC: Mutex<(f64, f64)> = Mutex::new((0.0, 0.0));

pub fn mouse_delta_f(dx: f64, dy: f64) {
    let (ix, iy) = {
        let mut f = FRAC.lock().unwrap();
        f.0 += dx * *SENS;
        f.1 += dy * *SENS;
        let (ix, iy) = (f.0.trunc(), f.1.trunc());
        f.0 -= ix;
        f.1 -= iy;
        (ix as i32, iy as i32)
    };
    if ix != 0 || iy != 0 { mouse_delta(ix, iy); }
}

fn mouse_delta(dx_: i32, dy_: i32) {
    if let (Some(dx), Some(dy)) = (word(DX), word(DY)) {
        dx.fetch_add(dx_, Ordering::SeqCst);
        dy.fetch_add(dy_, Ordering::SeqCst);
    }
    if cfg!(target_os = "ios") && !MENU_TOUCH.load(Ordering::Relaxed) {
        if let (Some(mx), Some(my)) = (word(MX), word(MY)) {
            mx.store(-9999, Ordering::SeqCst);
            my.store(-9999, Ordering::SeqCst);
        }
        if let Some(l) = word(LOCKED) {
            l.store(1, Ordering::SeqCst);
        }
    } else {
        let (w, h) = size();
        if let (Some(mx), Some(my)) = (word(MX), word(MY)) {
            mx.store((mx.load(Ordering::SeqCst) + dx_).clamp(0, w - 1), Ordering::SeqCst);
            my.store((my.load(Ordering::SeqCst) + dy_).clamp(0, h - 1), Ordering::SeqCst);
        }
    }
}
pub fn wheel(notches: i32) {
    if notches != 0 { if let Some(z) = word(DZ) { z.fetch_add(notches, Ordering::SeqCst); } }
}

pub fn apply_cheat(code: &str) {
    if std::env::var("MUGUET_SERVER").is_ok() || std::env::var("MUGUET_HOST").is_ok() {
        eprintln!("[cheat] blocked in multiplayer mode");
        return;
    }
    let code = code.trim().to_uppercase();
    if code.is_empty() { return; }
    eprintln!("[cheat] applying offline cheat: {code}");
    std::thread::spawn(move || {
        // Toggle GTA V cheat console: Backquote
        key("Backquote", true, false, Some("`"), false);
        std::thread::sleep(std::time::Duration::from_millis(60));
        key("Backquote", false, false, None, false);
        std::thread::sleep(std::time::Duration::from_millis(120));

        // Type characters
        for c in code.chars() {
            let s = c.to_string();
            let key_name = format!("Key{}", c);
            key(&key_name, true, false, Some(&s), false);
            std::thread::sleep(std::time::Duration::from_millis(45));
            key(&key_name, false, false, None, false);
            std::thread::sleep(std::time::Duration::from_millis(30));
        }

        // Enter to execute
        std::thread::sleep(std::time::Duration::from_millis(60));
        key("Enter", true, false, Some("\r"), false);
        std::thread::sleep(std::time::Duration::from_millis(60));
        key("Enter", false, false, None, false);
    });
}

#[no_mangle]
pub extern "C" fn muguet_apply_cheat(code: *const std::ffi::c_char) {
    if code.is_null() { return; }
    let s = unsafe { std::ffi::CStr::from_ptr(code) }.to_string_lossy();
    apply_cheat(&s);
}

#[no_mangle]
pub extern "C" fn muguet_is_multiplayer() -> bool {
    std::env::var("MUGUET_SERVER").is_ok() || std::env::var("MUGUET_HOST").is_ok()
}
