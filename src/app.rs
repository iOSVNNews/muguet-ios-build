// muguet - app.rs
// Constantin Clerc (c22dev) - 0.1

use crate::{gpu, input, Runtime};
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalSize, PhysicalPosition};
use winit::event::{DeviceEvent, DeviceId, ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{ModifiersState, PhysicalKey};
use winit::window::{CursorGrabMode, Fullscreen, Window, WindowId};

const TITLE: &str = "Muguet";
const PICK_TITLE: &str = "Muguet — loading: press Enter for Story Mode, or Space then 5 for Sandbox";
static ENGINE_ERROR: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);
static STARTUP_STATUS: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);

pub(crate) fn startup_status(message: impl Into<String>) {
    let message = message.into();
    eprintln!("[startup] {message}");
    *STARTUP_STATUS.lock().unwrap() = Some(message);
}

pub(crate) fn engine_error(message: String) {
    *ENGINE_ERROR.lock().unwrap() = Some(message);
}

struct App {
    mode_chosen: bool,
    rt: Option<Arc<Runtime>>,
    startup: crate::config::Startup,
    args: Vec<String>,
    window: Option<Arc<Window>>,
    started: bool,
    runtime_rx: Option<std::sync::mpsc::Receiver<Result<Arc<Runtime>, String>>>,
    startup_since: Option<std::time::Instant>,
    startup_stage: String,
    startup_warning_shown: bool,

    #[cfg_attr(not(target_os = "ios"), allow(dead_code))]
    launcher_shown: bool,
    locked: bool,
    mods: ModifiersState,
    wheel_accum: f64,
}

fn backbuffer_size(window: &Window) -> (i32, i32) {
    let s = if cfg!(target_os = "ios") { window.outer_size() } else { window.inner_size() };
    let scale: f64 = std::env::var("MUGUET_SCALE").ok().and_then(|v| v.parse().ok()).unwrap_or(1.0);
    if let Some((w, h)) = std::env::var("MUGUET_RES").ok().and_then(|r| { let (a, b) = r.split_once('x')?; Some((a.parse().ok()?, b.parse().ok()?)) }) {
        return (w, h);
    }
    let (mut w, mut h) = (s.width as f64 * scale, s.height as f64 * scale);
    let k = (2560.0 / w).min(1440.0 / h).min(1.0);
    w *= k;
    h *= k;
    (((w as i32) & !1).max(320), ((h as i32) & !1).max(240))
}

impl ApplicationHandler for App {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        gpu::SUSPENDED.store(false, std::sync::atomic::Ordering::Relaxed);

        #[cfg(target_os = "ios")]
        input::set_focus(true);
        if self.window.is_some() {
            return;
        }
        let attrs = Window::default_attributes()
            .with_title(if self.args.iter().any(|a| a.starts_with("-wasmstartchoice")) || std::env::var("MUGUET_START").map(|v| v != "0").unwrap_or(false) { TITLE } else { PICK_TITLE })
            .with_inner_size(LogicalSize::new(1280.0, 720.0));

        #[cfg(target_os = "ios")]
        let attrs = {
            use winit::platform::ios::{ScreenEdge, WindowAttributesExtIOS};
            attrs.with_prefers_status_bar_hidden(true).with_prefers_home_indicator_hidden(true).with_preferred_screen_edges_deferring_system_gestures(ScreenEdge::ALL)
        };
        let window = Arc::new(el.create_window(attrs).expect("create window"));

        if std::env::var("MUGUET_FULLSCREEN").map(|v| v != "0").unwrap_or(false) {
            window.set_fullscreen(Some(Fullscreen::Borderless(None)));
        }
        let mut desc = wgpu::InstanceDescriptor::new_without_display_handle();
        desc.backends = wgpu::Backends::METAL;
        let instance = wgpu::Instance::new(desc);
        let surface = instance.create_surface(window.clone()).expect("create surface");
        *gpu::SURFACE.lock().unwrap() = Some(gpu::SurfaceHandoff { instance, surface, window: window.clone() });
        let (w, h) = backbuffer_size(&window);
        input::set_size(w, h);
        eprintln!("[app] window {}x{} px, back buffer {w}x{h}", window.inner_size().width, window.inner_size().height);
        #[cfg(target_os = "ios")]
        {
            crate::touch::install(&window);
            window.set_cursor_visible(false);
            self.locked = true;
            input::set_locked(true);
        }
        self.window = Some(window);

        if !cfg!(target_os = "ios") {
            self.start_engine();
        }
    }

    fn suspended(&mut self, _el: &ActiveEventLoop) {
        gpu::SUSPENDED.store(true, std::sync::atomic::Ordering::Relaxed);
        input::set_focus(false);
        #[cfg(target_os = "ios")]
        crate::touch::release();
    }

    fn window_event(&mut self, _el: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Some(window) = self.window.clone() else { return };
        match event {
            WindowEvent::CloseRequested => {
                crate::mp::shutdown();
                std::process::exit(0);
            }
            WindowEvent::Resized(_) => {
                let (w, h) = backbuffer_size(&window);
                input::set_size(w, h);
            }
            WindowEvent::Occluded(hidden) => {
                gpu::OCCLUDED.store(hidden, std::sync::atomic::Ordering::Relaxed);
            }
            WindowEvent::Focused(f) => {
                input::set_focus(f);
                if !f { self.unlock(&window); }
            }
            WindowEvent::ModifiersChanged(m) => self.mods = m.state(),
            WindowEvent::KeyboardInput { event, .. } => {
                if !self.mode_chosen && input::start_mode() != 0 {
                    self.mode_chosen = true;
                    window.set_title(TITLE);
                }

                if self.mods.super_key() {
                    if let PhysicalKey::Code(winit::keyboard::KeyCode::KeyQ) = event.physical_key {
                        crate::mp::shutdown();
                        std::process::exit(0);
                    }

                    if let PhysicalKey::Code(winit::keyboard::KeyCode::KeyF) = event.physical_key {
                        if event.state == ElementState::Pressed && !event.repeat {
                            let fs = window.fullscreen().is_some();
                            window.set_fullscreen(if fs { None } else { Some(Fullscreen::Borderless(None)) });
                        }
                        return;
                    }
                    if event.state == ElementState::Pressed {
                        if !cfg!(target_os = "ios") { self.unlock(&window); }
                    }
                    return;
                }
                if let PhysicalKey::Code(code) = event.physical_key {
                    let name = format!("{code:?}");
                    if event.state == ElementState::Pressed && !event.repeat {
                        crate::mp::key_pressed(&name);
                    }
                    input::key(&name, event.state == ElementState::Pressed, event.repeat, event.text.as_deref(), self.mods.control_key());
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if state == ElementState::Pressed && !self.locked { self.lock(&window); }
                let b = match button { MouseButton::Left => 0, MouseButton::Middle => 1, MouseButton::Right => 2, MouseButton::Back => 3, MouseButton::Forward => 4, MouseButton::Other(_) => return };
                input::button(b, state == ElementState::Pressed);
            }
            WindowEvent::CursorMoved { position, .. } => {
                if !cfg!(target_os = "ios") && !self.locked { self.cursor(&window, position); }
            }
            #[cfg(target_os = "ios")]
            WindowEvent::Touch(t) => crate::touch::touch(&t),
            WindowEvent::MouseWheel { delta, .. } => {
                self.wheel_accum += match delta { MouseScrollDelta::LineDelta(_, y) => y as f64 / 3.0 * 3.0, MouseScrollDelta::PixelDelta(p) => p.y / 100.0 };
                let n = self.wheel_accum.trunc();
                self.wheel_accum -= n;
                input::wheel(n as i32);
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, el: &ActiveEventLoop) {
        if let Some(message) = STARTUP_STATUS.lock().unwrap().take() {
            self.startup_stage = message;
            #[cfg(target_os = "ios")]
            if self.startup_since.is_some() { crate::touch::loading(Some(&self.startup_stage)); }
        }
        if let Some(message) = ENGINE_ERROR.lock().unwrap().take() {
            self.startup_since = None;
            gpu::SUSPENDED.store(true, std::sync::atomic::Ordering::Relaxed);
            input::set_focus(false);
            #[cfg(target_os = "ios")]
            crate::touch::loading(Some(&format!("Game stopped\n{message}\nReopen Muguet to share its diagnostic log.")));
        }
        if let Some(rx) = &self.runtime_rx {
            match rx.try_recv() {
                Ok(Ok(rt)) => {
                    self.runtime_rx = None;
                    self.rt = Some(rt.clone());
                    self.launch_engine(rt);
                }
                Ok(Err(message)) => {
                    self.runtime_rx = None;
                    self.startup_since = None;
                    eprintln!("[host] startup failed: {message}");
                    #[cfg(target_os = "ios")]
                    crate::touch::loading(Some(&format!("Could not start game\n{message}\nDetails: Files → Muguet → muguet-runtime.log")));
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.runtime_rx = None;
                    self.startup_since = None;
                    #[cfg(target_os = "ios")]
                    crate::touch::loading(Some("Runtime preparation failed. Reopen Muguet and share its diagnostic log."));
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {}
            }
        }
        if let Some(since) = self.startup_since {
            if gpu::PRESENTS.load(std::sync::atomic::Ordering::Relaxed) > 0 {
                eprintln!("[startup] first displayed frame after {:.2}s", since.elapsed().as_secs_f64());
                self.startup_since = None;
                #[cfg(target_os = "ios")]
                crate::touch::loading(None);
            } else if since.elapsed().as_secs() >= 45 && !self.startup_warning_shown {
                self.startup_warning_shown = true;
                eprintln!("[startup] still waiting for first frame: {} (GPU started={}, occluded={}, suspended={})",
                    self.startup_stage, gpu::STARTED.load(std::sync::atomic::Ordering::Relaxed),
                    gpu::OCCLUDED.load(std::sync::atomic::Ordering::Relaxed), gpu::SUSPENDED.load(std::sync::atomic::Ordering::Relaxed));
                #[cfg(target_os = "ios")]
                crate::touch::loading(Some(&format!("{}\nChưa có hình ảnh sau 45 giây. Game vẫn đang khởi động.\nNếu không tiếp tục, mở lại Muguet và chia sẻ muguet-runtime.log.", self.startup_stage)));
            }
        }
        #[allow(unused_mut)]
        let mut wait = if crate::pad::poll() { 8 } else if cfg!(target_os = "ios") { 250 } else { 1000 };
        #[cfg(target_os = "ios")]
        if crate::touch::refresh() { wait = 8; }
        #[cfg(target_os = "ios")]
        if !self.started {
            self.launcher();
            wait = 50;
        }
        el.set_control_flow(ControlFlow::WaitUntil(std::time::Instant::now() + std::time::Duration::from_millis(wait)));
    }

    fn device_event(&mut self, _el: &ActiveEventLoop, _id: DeviceId, event: DeviceEvent) {
        if let DeviceEvent::MouseMotion { delta } = event {
            if self.locked { input::mouse_delta_f(delta.0, delta.1); }
        }
    }

    fn memory_warning(&mut self, _el: &ActiveEventLoop) {
        gpu::request_memory_cleanup();
        eprintln!("[memory] iOS warning: footprint {} MB, remaining {} MB", crate::footprint_mb(), crate::performance::available_memory().unwrap_or(0) >> 20);
    }
}

impl App {
    fn start_engine(&mut self) {
        if self.window.is_none() { return; }
        if self.started { return; }
        self.started = true;
        self.startup_since = Some(std::time::Instant::now());
        self.startup_stage = "Đang kiểm tra dữ liệu game…".into();
        #[cfg(target_os = "ios")]
        crate::touch::loading(Some(&self.startup_stage));
        #[cfg(target_os = "ios")]
        if let Some(rt) = self.rt.clone() { self.launch_engine(rt); return; }
        let startup = crate::config::Startup {
            wasm_path: self.startup.wasm_path.clone(), cache_dir: self.startup.cache_dir.clone(),
            engine_args: self.startup.engine_args.clone(),
        };
        let (tx, rx) = std::sync::mpsc::channel();
        self.runtime_rx = Some(rx);
        let result = std::thread::Builder::new().name("runtime-loader".into()).stack_size(8 << 20).spawn(move || {
            let mut reported = std::time::Instant::now();
            let result = crate::config::validate_game_files_with_progress(|done, total| {
                if done == 0 || done == total || reported.elapsed().as_millis() >= 500 {
                    startup_status(format!("Đang kiểm tra dữ liệu game: {done}/{total} tệp…"));
                    reported = std::time::Instant::now();
                }
            }).and_then(|()| {
                startup_status("Đang nạp runtime game…");
                crate::make_runtime(&startup)
            }).map_err(|e| format!("{e:#}"));
            let _ = tx.send(result);
        });
        if let Err(e) = result {
            self.runtime_rx = None;
            self.startup_since = None;
            eprintln!("[host] cannot prepare runtime: {e}");
            #[cfg(target_os = "ios")]
            crate::touch::loading(Some("Unable to prepare runtime. Reopen Muguet and use lower graphics settings."));
        }
    }

    fn launch_engine(&mut self, rt: Arc<Runtime>) {
        let Some(window) = self.window.clone() else { return };
        let (w, h) = backbuffer_size(&window);
        input::set_size(w, h);
        let mut args = self.args.clone();
        args.extend(["-wgpu".to_string(), format!("-width={w}"), format!("-height={h}")]);
        let result = std::thread::Builder::new()
            .name("engine-main".into())
            .stack_size(64 << 20)
            .spawn(move || {
                if let Err(e) = crate::run_engine(&rt, args) {
                    eprintln!("[host] engine failed: {e:?}");
                    *ENGINE_ERROR.lock().unwrap() = Some(format!("{e:#}"));
                    return;
                }
                std::process::exit(0);
            });
        if let Err(e) = result { *ENGINE_ERROR.lock().unwrap() = Some(format!("Cannot start game thread: {e}")); }
    }

    #[cfg(target_os = "ios")]
    fn launcher(&mut self) {
        use std::sync::atomic::{AtomicBool, Ordering};
        static PLAY: AtomicBool = AtomicBool::new(false);
        extern "C" fn played() { PLAY.store(true, Ordering::SeqCst); }
        extern "C" {
            fn muguet_show_launcher(view_controller: *mut std::ffi::c_void, done: extern "C" fn());
        }
        if self.started { return; }
        let Some(window) = self.window.clone() else { return };
        if !self.launcher_shown {
            use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
            let vc = match window.window_handle().map(|h| h.as_raw()) {
                Ok(RawWindowHandle::UiKit(h)) => h.ui_view_controller,
                _ => None,
            };
            self.launcher_shown = true;
            match vc {
                Some(vc) => unsafe { muguet_show_launcher(vc.as_ptr(), played) },
                None => PLAY.store(true, Ordering::SeqCst),
            }
        }
        if PLAY.load(Ordering::SeqCst) {
            let settings = crate::config::reload();
            self.args.retain(|a| !a.starts_with("-frameLimit=") && !a.starts_with("-wasmstartchoice="));
            self.args.extend(settings);
            self.start_engine();
        }
    }

    fn cursor(&self, window: &Window, p: PhysicalPosition<f64>) {
        let s = window.inner_size();
        let (w, h) = input::size();
        input::mouse_pos((p.x * w as f64 / s.width.max(1) as f64).round() as i32, (p.y * h as f64 / s.height.max(1) as f64).round() as i32);
    }
    fn lock(&mut self, window: &Window) {
        let ok = window.set_cursor_grab(CursorGrabMode::Locked).or_else(|_| window.set_cursor_grab(CursorGrabMode::Confined)).is_ok();
        window.set_cursor_visible(false);
        self.locked = ok;
        input::set_locked(ok);
    }
    fn unlock(&mut self, window: &Window) {
        if cfg!(target_os = "ios") { return; }
        if !self.locked { return; }
        let _ = window.set_cursor_grab(CursorGrabMode::None);
        window.set_cursor_visible(true);
        self.locked = false;
        input::set_locked(false);
    }
}

pub fn run(rt: Option<Arc<Runtime>>, startup: crate::config::Startup, args: Vec<String>) -> anyhow::Result<()> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut app = App { mode_chosen: false, rt, startup, args, window: None, started: false, runtime_rx: None,
        startup_since: None, startup_stage: String::new(), startup_warning_shown: false,
        launcher_shown: false, locked: cfg!(target_os = "ios"), mods: ModifiersState::empty(), wheel_accum: 0.0 };
    event_loop.run_app(&mut app)?;
    Ok(())
}
