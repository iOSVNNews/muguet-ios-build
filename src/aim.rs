// Touch motion is expressed in points, independent of display resolution and
// event frequency. Avoid per-event acceleration: it changes at 60/120 Hz.
#[derive(Clone, Copy)]
pub struct DragLook { start: (f64, f64), last: (f64, f64), active: bool }

impl DragLook {
    pub fn new(at: (f64, f64)) -> Self { Self { start: at, last: at, active: false } }
    pub fn move_to(&mut self, at: (f64, f64)) -> (f64, f64) {
        let delta = (at.0 - self.last.0, at.1 - self.last.1);
        self.last = at;
        if !self.active {
            if (at.0 - self.start.0).hypot(at.1 - self.start.1) < 2.5 { return (0.0, 0.0); }
            self.active = true;
        }
        delta
    }
}

pub fn scale_delta(delta: (f64, f64), sensitivity: f64, aimed: bool, aim_scale: f64) -> (f64, f64) {
    let speed = delta.0.hypot(delta.1);
    let accel = if aimed { 1.0 } else { 1.0 + (speed / 10.0).min(1.5) };
    let sens = sensitivity.clamp(0.1, 30.0);
    let aim = if aimed { aim_scale.clamp(0.05, 10.0) } else { 1.0 };
    let gain = sens * aim * accel;
    (delta.0 * gain, delta.1 * gain * 0.85)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tap_jitter_does_not_move_the_camera() {
        let mut drag = DragLook::new((10.0, 20.0));
        assert_eq!(drag.move_to((11.0, 20.0)), (0.0, 0.0));
        assert_eq!(drag.move_to((10.0, 21.0)), (0.0, 0.0));
        assert_eq!(drag.move_to((14.0, 21.0)), (4.0, 0.0));
        assert_eq!(drag.move_to((14.25, 21.0)), (0.25, 0.0));
    }
    #[test]
    fn precision_aim_is_linear_and_preserves_small_adjustments() {
        let whole = scale_delta((24.0, 8.0), 1.0, true, 0.4);
        let small = scale_delta((2.0, 8.0 / 12.0), 1.0, true, 0.4);
        assert!((whole.0 - small.0 * 12.0).abs() < 1e-9);
        assert!((whole.1 - small.1 * 12.0).abs() < 1e-9);
        assert!(whole.0 < scale_delta((24.0, 8.0), 1.0, false, 0.4).0);
    }
}
