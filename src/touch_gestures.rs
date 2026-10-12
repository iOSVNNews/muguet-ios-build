// State shared by the overlay and its platform-independent regression tests.
#[derive(Default)]
pub struct MenuState {
    pub nav: bool,
    // This records an Escape request, not an engine-reported pause state.
    pub pause_requested: bool,
    pause_keys_visible: bool,
}

impl MenuState {
    pub fn pause(&mut self) {
        self.pause_requested = !self.pause_requested;
        self.pause_keys_visible = self.pause_requested;
    }

    pub fn toggle_keys(&mut self) {
        if self.pause_requested {
            self.pause_keys_visible = !self.pause_keys_visible;
        } else {
            self.nav = !self.nav;
        }
    }

    pub fn keys_visible(&self) -> bool {
        if self.pause_requested { self.pause_keys_visible } else { self.nav }
    }
}

#[derive(Default)]
pub struct Pinch {
    remainder: f64,
}

impl Pinch {
    pub fn reset(&mut self) { self.remainder = 0.0; }

    pub fn update(&mut self, before: f64, after: f64) -> i32 {
        if before < 10.0 || after < 10.0 { return 0; }
        // One wheel step per 12% change, independent of touch event frequency.
        self.remainder += (after / before).ln() / 1.12_f64.ln();
        let steps = self.remainder.trunc() as i32;
        self.remainder -= steps as f64;
        steps
    }
}

pub fn pointer_position(p: (f64, f64), screen: (f64, f64), render: (i32, i32)) -> (i32, i32) {
    let x = (p.0 * render.0 as f64 / screen.0.max(1.0)).round() as i32;
    let y = (p.1 * render.1 as f64 / screen.1.max(1.0)).round() as i32;
    (x.clamp(0, (render.0 - 1).max(0)), y.clamp(0, (render.1 - 1).max(0)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pause_and_menu_keys_are_independent() {
        let mut state = MenuState::default();
        state.pause();
        assert!(!state.nav && state.pause_requested);
        assert!(state.keys_visible());
        state.toggle_keys();
        assert!(!state.nav && state.pause_requested && !state.keys_visible());
        state.toggle_keys();
        assert!(!state.nav && state.pause_requested && state.keys_visible());
        state.pause();
        assert!(!state.nav && !state.pause_requested && !state.keys_visible());
    }

    #[test]
    fn pause_preserves_explicit_menu_keys_selection() {
        let mut state = MenuState::default();
        state.toggle_keys();
        state.pause();
        state.pause();
        assert!(state.nav && !state.pause_requested);
    }

    #[test]
    fn hiding_pause_buttons_does_not_unpause_or_change_gameplay_keys() {
        let mut state = MenuState::default();
        state.pause();
        state.toggle_keys();
        assert!(state.pause_requested && !state.keys_visible() && !state.nav);
        state.pause();
        assert!(!state.pause_requested && !state.nav);
        state.pause();
        assert!(state.pause_requested && state.keys_visible() && !state.nav);
    }

    #[test]
    fn menu_keys_does_not_send_a_pause_request() {
        let mut state = MenuState::default();
        state.toggle_keys();
        assert!(state.nav && !state.pause_requested);
        state.toggle_keys();
        assert!(!state.nav && !state.pause_requested);
    }

    #[test]
    fn pinch_accumulates_small_events_and_reverses() {
        let mut pinch = Pinch::default();
        assert_eq!(pinch.update(100.0, 106.0), 0);
        assert_eq!(pinch.update(106.0, 113.0), 1);
        pinch.reset();
        assert_eq!(pinch.update(113.0, 100.0), -1);
        assert_eq!(pinch.update(0.0, 100.0), 0);
    }

    #[test]
    fn positions_follow_render_resolution_and_clamp_edges() {
        assert_eq!(pointer_position((400.0, 200.0), (800.0, 400.0), (1280, 720)), (640, 360));
        assert_eq!(pointer_position((-10.0, 500.0), (800.0, 400.0), (1280, 720)), (0, 719));
    }
}
