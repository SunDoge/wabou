//! Host-owned tooltip timing. No actor transaction or plugin callback is needed.
//!
//! Hosts supply monotonic elapsed time and a complete input snapshot, schedule
//! only `deadline()`, and repaint when the visible key changes. Each window owns
//! one controller. Reconcile keys after admitting a new projection, before
//! processing a timer; stale timer notifications are harmless.
use std::time::Duration;

/// Resolve current projection eligibility without storing or copying view data.
/// The host additionally gates window focus and active native selection popups.
pub fn eligible(view: &crate::View, hint: u64, modal: Option<u64>) -> bool {
    let Some(binding) = view.bindings.get(&hint) else {
        return false;
    };
    if binding.node.semantics.role != blockui_abi::semantics::TOOLTIP
        || binding.text.trim().is_empty()
    {
        return false;
    }
    let mut in_modal = modal.is_none();
    let mut key = binding.node.tooltip_anchor;
    while let Some(binding) = view.bindings.get(&key) {
        if binding.node.disabled != 0 || binding.node.semantics.hidden != 0 {
            return false;
        }
        in_modal |= modal == Some(key);
        key = binding.node.parent;
    }
    key == 0 && in_modal
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Trigger {
    pub key: u64,
    pub delay: Duration,
}

/// Disabled, hidden and detached triggers must be excluded by the host.
#[derive(Clone, Copy, Debug, Default)]
pub struct Input {
    pub hovered: Option<Trigger>,
    pub focused: Option<Trigger>,
    /// The visible tooltip whose content contains the pointer, if any.
    pub content: Option<u64>,
}

/// A bounded controller: one target and at most two dismissed input triggers.
#[derive(Debug)]
pub struct Controller {
    target: Option<u64>,
    visible: bool,
    closing: bool,
    deadline: Option<Duration>,
    last_close: Option<Duration>,
    suppressed: [Option<u64>; 2],
    now: Duration,
    skip_delay: Duration,
    close_grace: Duration,
}

impl Default for Controller {
    fn default() -> Self {
        Self::new(Duration::from_millis(300), Duration::from_millis(100))
    }
}

impl Controller {
    pub fn new(skip_delay: Duration, close_grace: Duration) -> Self {
        Self {
            target: None,
            visible: false,
            closing: false,
            deadline: None,
            last_close: None,
            suppressed: [None; 2],
            now: Duration::ZERO,
            skip_delay,
            close_grace,
        }
    }

    pub fn visible(&self) -> Option<u64> {
        self.target.filter(|_| self.visible)
    }

    pub fn deadline(&self) -> Option<Duration> {
        self.deadline
    }

    fn clock(&mut self, now: Duration) -> Duration {
        self.now = self.now.max(now);
        self.now
    }

    fn close(&mut self, now: Duration) {
        if self.visible {
            self.last_close = Some(now);
        }
        self.target = None;
        self.visible = false;
        self.closing = false;
        self.deadline = None;
    }

    /// Keyboard focus takes priority when focus and hover select different
    /// triggers. Focus opens immediately; hover uses the trigger's delay.
    /// Repeated pointer observations never restart an existing opening timer.
    /// Returns whether the painted tooltip identity changed.
    pub fn observe(&mut self, input: Input, now: Duration) -> bool {
        let before = self.visible();
        let now = self.clock(now);
        for key in &mut self.suppressed {
            if key.is_some_and(|key| {
                !input.hovered.is_some_and(|t| t.key == key)
                    && !input.focused.is_some_and(|t| t.key == key)
            }) {
                *key = None;
            }
        }
        let focused = input
            .focused
            .filter(|t| !self.suppressed.contains(&Some(t.key)));
        let hovered = input
            .hovered
            .filter(|t| !self.suppressed.contains(&Some(t.key)));
        if let Some(trigger) = focused.or(hovered) {
            if self.target != Some(trigger.key) {
                self.close(now);
                self.target = Some(trigger.key);
                let warm = self.last_close.is_some_and(|closed| {
                    self.skip_delay != Duration::ZERO
                        && now.saturating_sub(closed) < self.skip_delay
                });
                if focused.is_some() || warm || trigger.delay == Duration::ZERO {
                    self.visible = true;
                } else {
                    self.deadline = Some(now.saturating_add(trigger.delay));
                }
            } else if focused.is_some() {
                self.visible = true;
                self.deadline = None;
            }
            if self.closing {
                self.closing = false;
                self.deadline = None;
            }
        } else if self.visible && input.content == self.target {
            self.closing = false;
            self.deadline = None;
        } else if self.visible {
            if !self.closing {
                self.closing = true;
                self.deadline = Some(now.saturating_add(self.close_grace));
            }
        } else {
            self.close(now);
        }
        // Deal with zero delays without requesting a redundant host wake.
        self.poll(now);
        before != self.visible()
    }

    /// A late or superseded host wake cannot open an old trigger.
    pub fn poll(&mut self, now: Duration) -> bool {
        let before = self.visible();
        let now = self.clock(now);
        if self.deadline.is_some_and(|deadline| deadline <= now) {
            if self.closing {
                self.close(now);
            } else {
                self.deadline = None;
                self.visible = self.target.is_some();
            }
        }
        before != self.visible()
    }

    /// Escape or trigger activation also cancels pending display. Suppression
    /// lasts until the trigger loses both hover and focus, preventing a repaint
    /// or pointer movement from immediately reopening the dismissed tooltip.
    pub fn dismiss(&mut self, input: Input, now: Duration) -> bool {
        let before = self.visible();
        let now = self.clock(now);
        self.suppressed = [input.hovered.map(|t| t.key), input.focused.map(|t| t.key)];
        self.close(now);
        before != self.visible()
    }

    /// Removal, disablement, modal isolation and replacement all use this
    /// admission check. A surviving key retains its timer and visibility; no
    /// function pointer or projection reference is retained.
    pub fn reconcile(&mut self, mut is_eligible: impl FnMut(u64) -> bool) -> bool {
        let before = self.visible();
        for key in &mut self.suppressed {
            if key.is_some_and(|key| !is_eligible(key)) {
                *key = None;
            }
        }
        if self.target.is_some_and(|key| !is_eligible(key)) {
            self.close(self.now);
            // Disposing a target must not warm an unrelated future projection.
            self.last_close = None;
        }
        before != self.visible()
    }

    /// Window blur/unmount cancels all transient work, including warm timing.
    pub fn reset(&mut self) -> bool {
        let changed = self.visible().is_some();
        self.close(self.now);
        self.last_close = None;
        self.suppressed = [None; 2];
        changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn ms(value: u64) -> Duration {
        Duration::from_millis(value)
    }
    fn trigger(key: u64) -> Trigger {
        Trigger {
            key,
            delay: ms(700),
        }
    }
    fn hover(key: u64) -> Input {
        Input {
            hovered: Some(trigger(key)),
            ..Input::default()
        }
    }
    fn focus(key: u64) -> Input {
        Input {
            focused: Some(trigger(key)),
            ..Input::default()
        }
    }

    #[test]
    fn hover_waits_without_restarting_and_leaving_cancels() {
        let mut c = Controller::default();
        assert!(!c.observe(hover(1), ms(0)));
        c.observe(hover(1), ms(400));
        assert_eq!(c.deadline(), Some(ms(700)));
        assert!(!c.poll(ms(699)));
        assert!(c.poll(ms(700)));
        assert_eq!(c.visible(), Some(1));
        c.reset();
        c.observe(hover(1), ms(1000));
        c.observe(Input::default(), ms(1100));
        assert_eq!(c.deadline(), None);
        assert!(!c.poll(ms(3000)));
        assert_eq!(c.visible(), None);
    }

    #[test]
    fn focus_opens_pending_hover_immediately_and_has_priority() {
        let mut c = Controller::default();
        c.observe(hover(1), ms(0));
        assert!(c.observe(focus(1), ms(10)));
        assert_eq!(c.deadline(), None);
        c.observe(
            Input {
                hovered: Some(trigger(2)),
                ..focus(1)
            },
            ms(20),
        );
        assert_eq!(c.visible(), Some(1));
        c.observe(focus(2), ms(30));
        assert_eq!(c.visible(), Some(2));
    }

    #[test]
    fn grace_bridges_trigger_content_gap_and_never_restarts() {
        let mut c = Controller::default();
        c.observe(focus(1), ms(0));
        c.observe(Input::default(), ms(10));
        c.observe(Input::default(), ms(50));
        assert_eq!(c.deadline(), Some(ms(110)));
        c.observe(
            Input {
                content: Some(1),
                ..Input::default()
            },
            ms(80),
        );
        assert_eq!(c.deadline(), None);
        assert!(!c.poll(ms(110)));
        assert_eq!(c.visible(), Some(1));
        c.observe(Input::default(), ms(200));
        assert!(c.poll(ms(300)));
        assert_eq!(c.visible(), None);
    }

    #[test]
    fn content_cannot_open_a_closed_or_different_tooltip() {
        let mut c = Controller::default();
        c.observe(
            Input {
                content: Some(1),
                ..Input::default()
            },
            ms(0),
        );
        assert_eq!(c.visible(), None);
        c.observe(focus(1), ms(10));
        c.observe(
            Input {
                content: Some(2),
                ..Input::default()
            },
            ms(20),
        );
        c.poll(ms(120));
        assert_eq!(c.visible(), None);
    }

    #[test]
    fn escape_suppresses_both_active_triggers_until_they_leave() {
        let mut c = Controller::default();
        let input = Input {
            hovered: Some(trigger(2)),
            ..focus(1)
        };
        c.observe(input, ms(0));
        assert!(c.dismiss(input, ms(1)));
        c.observe(input, ms(2));
        c.observe(hover(2), ms(3));
        c.poll(ms(900));
        assert_eq!(c.visible(), None);
        c.observe(Input::default(), ms(901));
        c.observe(focus(1), ms(902));
        assert_eq!(c.visible(), Some(1));
    }

    #[test]
    fn activation_cancels_pending_timer_and_stale_wake() {
        let mut c = Controller::default();
        c.observe(hover(1), ms(0));
        assert!(!c.dismiss(hover(1), ms(10)));
        c.observe(hover(1), ms(20));
        assert_eq!(c.deadline(), None);
        c.poll(ms(700));
        assert_eq!(c.visible(), None);
    }

    #[test]
    fn nearby_triggers_skip_delay_but_cold_triggers_wait() {
        let mut c = Controller::default();
        c.observe(focus(1), ms(0));
        c.observe(hover(2), ms(10));
        assert_eq!(c.visible(), Some(2));
        c.observe(Input::default(), ms(20));
        c.poll(ms(120));
        c.observe(hover(3), ms(419));
        assert_eq!(c.visible(), Some(3));
        c.observe(Input::default(), ms(420));
        c.poll(ms(520));
        c.observe(hover(4), ms(820));
        assert_eq!(c.visible(), None);
        assert_eq!(c.deadline(), Some(ms(1520)));
    }

    #[test]
    fn superseded_timer_and_replacement_use_only_current_identity() {
        let mut c = Controller::default();
        c.observe(hover(1), ms(0));
        c.observe(hover(2), ms(100));
        assert!(!c.reconcile(|key| key == 2));
        assert_eq!(c.deadline(), Some(ms(800)));
        assert!(!c.poll(ms(700)));
        assert!(c.poll(ms(800)));
        assert_eq!(c.visible(), Some(2));
        assert!(!c.reconcile(|key| key == 2));
        assert!(c.reconcile(|_| false));
        assert_eq!(c.deadline(), None);
        c.poll(ms(5000));
        assert_eq!(c.visible(), None);
    }

    #[test]
    fn removal_and_window_blur_cancel_pending_and_warm_state() {
        let mut c = Controller::default();
        c.observe(hover(1), ms(0));
        c.reconcile(|_| false);
        assert_eq!(c.deadline(), None);
        c.observe(focus(2), ms(10));
        assert!(c.reset());
        c.observe(hover(3), ms(20));
        assert_eq!(c.deadline(), Some(ms(720)));
        assert!(!c.reset());
        assert!(!c.poll(ms(1000)));
    }

    #[test]
    fn zero_timing_and_regressing_clock_are_well_defined() {
        let mut c = Controller::new(Duration::ZERO, Duration::ZERO);
        c.observe(
            Input {
                hovered: Some(Trigger {
                    key: 1,
                    delay: Duration::ZERO,
                }),
                ..Input::default()
            },
            ms(100),
        );
        assert_eq!(c.visible(), Some(1));
        assert!(c.observe(Input::default(), ms(90)));
        c.observe(hover(2), ms(80));
        assert_eq!(c.deadline(), Some(ms(800)));
        assert!(!c.poll(ms(799)));
        assert!(c.poll(ms(800)));
    }

    #[test]
    fn independent_windows_do_not_share_warm_or_dismissed_state() {
        let mut a = Controller::default();
        let mut b = Controller::default();
        a.observe(focus(1), ms(0));
        a.dismiss(focus(1), ms(1));
        b.observe(hover(1), ms(2));
        assert_eq!(b.deadline(), Some(ms(702)));
        b.poll(ms(702));
        assert_eq!(b.visible(), Some(1));
        a.observe(focus(1), ms(702));
        assert_eq!(a.visible(), None);
    }
}
