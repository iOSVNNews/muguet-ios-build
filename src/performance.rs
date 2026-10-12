use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::{Duration, Instant};

pub const MIB: u64 = 1 << 20;
pub const HEAP_CEILING: u64 = 16 << 30;
pub const IOS_HEAP_MAX: u64 = 3072 * MIB;
pub static MEMORY_PRESSURE: AtomicBool = AtomicBool::new(false);
static FRAME_LIMIT: AtomicU32 = AtomicU32::new(60);
static PROMOTION_SYNC: AtomicBool = AtomicBool::new(false);

pub fn set_promotion_sync(enabled: bool) {
    PROMOTION_SYNC.store(enabled, Ordering::Relaxed);
}

pub fn promotion_sync_enabled() -> bool {
    PROMOTION_SYNC.load(Ordering::Relaxed)
}

pub fn set_frame_limit(fps: f64) {
    let fps = if !fps.is_finite() { 60 } else if fps <= 0.0 { 0 }
        else if fps <= 30.0 { 30 } else if fps <= 60.0 { 60 } else { 120 };
    FRAME_LIMIT.store(fps, Ordering::Relaxed);
}

pub fn frame_limit() -> u32 { FRAME_LIMIT.load(Ordering::Relaxed) }

pub fn engine_frame_limit(fps: f64) -> u8 {
    if fps <= 0.0 || fps > 60.0 { 0 } else if fps <= 30.0 { 2 } else { 1 }
}

pub fn available_memory() -> Option<u64> {
    #[cfg(target_os = "ios")]
    {
        extern "C" { fn os_proc_available_memory() -> usize; }
        Some(unsafe { os_proc_available_memory() } as u64)
    }
    #[cfg(not(target_os = "ios"))]
    { None }
}

pub fn physical_memory() -> u64 {
    #[cfg(target_os = "ios")]
    {
        extern "C" { fn muguet_physical_memory() -> u64; }
        unsafe { muguet_physical_memory() }
    }
    #[cfg(not(target_os = "ios"))]
    { HEAP_CEILING * 2 }
}

// Leave room for Metal, native code, audio and the operating system. This is
// a limit on the game heap, not a request to allocate this many physical pages.
pub fn heap_budget(physical: u64, available: Option<u64>, current_heap: u64) -> u64 {
    let device_budget = physical.saturating_mul(65) / 100;
    let process_budget = available.map(|a| current_heap.saturating_add(a.saturating_sub(768 * MIB)))
        .unwrap_or(device_budget);
    device_budget.min(process_budget).min(HEAP_CEILING).max(current_heap)
}

pub fn ios_heap_budget(physical: u64, available: Option<u64>, current_heap: u64) -> u64 {
    heap_budget(physical, available, current_heap).min(IOS_HEAP_MAX)
}

pub fn growth_fits(old: u64, requested: u64, maximum: u64, available: Option<u64>) -> bool {
    if requested <= old { return true; }
    requested <= maximum && available.map(|a| requested - old <= a.saturating_sub(256 * MIB)).unwrap_or(true)
}

#[derive(Default)]
pub struct FramePacer { next: Option<Instant>, fps: u32 }

impl FramePacer {
    pub fn delay(&mut self, now: Instant, fps: u32) -> Duration {
        if fps == 0 { self.next = None; self.fps = 0; return Duration::ZERO; }
        let interval = Duration::from_secs_f64(1.0 / fps as f64);
        let deadline = if self.fps != fps { now } else { self.next.unwrap_or(now).max(now) };
        self.fps = fps;
        self.next = Some(deadline + interval);
        deadline.saturating_duration_since(now)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn low_ram_never_receives_the_sixteen_gib_budget() {
        assert_eq!(heap_budget(4 << 30, Some(2 << 30), 0), 1280 * MIB);
        assert!(heap_budget(8 << 30, Some(6 << 30), 0) < 6 << 30);
        assert_eq!(heap_budget(32 << 30, Some(24 << 30), 0), HEAP_CEILING);
        assert_eq!(heap_budget(4 << 30, Some(128 * MIB), 1 << 30), 1 << 30);
        assert_eq!(ios_heap_budget(8 << 30, Some(6 << 30), 1 << 30), 3 << 30);
        assert!(ios_heap_budget(4 << 30, Some(2 << 30), 0) < 3 << 30);
    }
    #[test]
    fn refuses_growth_that_consumes_native_headroom() {
        assert!(!growth_fits(1 << 30, 2 << 30, 4 << 30, Some(1100 * MIB)));
        assert!(growth_fits(1 << 30, 1 << 30, 4 << 30, Some(0)));
        assert!(!growth_fits(1 << 30, 5 << 30, 4 << 30, None));
    }
    #[test]
    fn frame_pacing_does_not_burst_after_stalls_or_rate_changes() {
        let now = Instant::now();
        let mut pacer = FramePacer::default();
        assert_eq!(pacer.delay(now, 120), Duration::ZERO);
        assert!(pacer.delay(now, 120).as_secs_f64() < 0.00834);
        assert_eq!(pacer.delay(now + Duration::from_secs(3), 120), Duration::ZERO);
        assert_eq!(pacer.delay(now + Duration::from_secs(3), 30), Duration::ZERO);
        assert_eq!(engine_frame_limit(120.0), 0);
        assert_eq!(engine_frame_limit(60.0), 1);
    }
}
