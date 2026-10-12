//! Host-owned submenu hover intent. Geometry is supplied by the current renderer.
#[derive(Clone, Copy, Debug)]
pub struct Target {
    pub key: u64,
    pub opens_submenu: bool,
}
#[derive(Clone, Copy, Debug)]
pub struct Branch {
    pub trigger: u64,
    pub anchor: [f64; 4],
    pub panel: [f64; 4],
}
#[derive(Default, Debug)]
pub struct Controller {
    branch: Option<u64>,
    origin: Option<[f64; 2]>,
    previous: Option<[f64; 2]>,
    grace_until: u64,
    pending_focus: Option<u64>,
    pending_open: Option<(u64, u64)>,
    focused: Option<u64>,
    last_time: Option<u64>,
}
#[derive(Default, Debug, PartialEq, Eq)]
pub struct Decision {
    pub focus: Option<u64>,
    pub open: Option<u64>,
}
fn contains(rect: [f64; 4], point: [f64; 2]) -> bool {
    point[0] >= rect[0] && point[0] <= rect[2] && point[1] >= rect[1] && point[1] <= rect[3]
}
fn in_triangle(p: [f64; 2], a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> bool {
    let cross = |a: [f64; 2], b: [f64; 2], c: [f64; 2]| {
        (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
    };
    if cross(a, b, c).abs() < 0.01 {
        return false;
    }
    let signs = [cross(a, b, p), cross(b, c, p), cross(c, a, p)];
    signs.iter().all(|v| *v >= 0.0) || signs.iter().all(|v| *v <= 0.0)
}
impl Controller {
    pub fn reset(&mut self) {
        *self = Self::default();
    }
    pub fn deadline(&self) -> Option<u64> {
        self.pending_open
            .map(|(_, deadline)| deadline)
            .into_iter()
            .chain(self.pending_focus.map(|_| self.grace_until))
            .min()
    }
    pub fn observe(
        &mut self,
        point: [f64; 2],
        target: Option<Target>,
        branch: Option<Branch>,
        now: u64,
    ) -> Decision {
        if point.iter().any(|v| !v.is_finite()) || self.last_time.is_some_and(|last| now < last) {
            self.reset();
        }
        if point.iter().any(|v| !v.is_finite()) {
            return Decision::default();
        }
        self.last_time = Some(now);
        if self.branch != branch.map(|b| b.trigger) {
            self.branch = branch.map(|b| b.trigger);
            self.origin = branch.and_then(|b| self.previous.filter(|p| contains(b.anchor, *p)));
            self.grace_until = now.saturating_add(300);
            self.pending_focus = None;
        }
        let mut protect = false;
        if let Some(branch) = branch {
            if contains(branch.anchor, point) {
                self.origin = Some(point);
                self.grace_until = now.saturating_add(300);
            } else if contains(branch.panel, point) {
                self.origin = None;
            } else if let Some(origin) = self.origin {
                let right =
                    branch.panel[0] + branch.panel[2] >= branch.anchor[0] + branch.anchor[2];
                let edge = if right {
                    branch.panel[0]
                } else {
                    branch.panel[2]
                };
                let forward = self.previous.is_none_or(|p| {
                    if right {
                        point[0] >= p[0] - 0.5
                    } else {
                        point[0] <= p[0] + 0.5
                    }
                });
                protect = now < self.grace_until
                    && forward
                    && in_triangle(
                        point,
                        origin,
                        [edge, branch.panel[1] - 6.0],
                        [edge, branch.panel[3] + 6.0],
                    );
            }
        }
        self.previous = Some(point);
        if protect {
            self.pending_focus = target.map(|t| t.key);
            self.pending_open = None;
            return Decision::default();
        }
        self.pending_focus = None;
        let Some(target) = target else {
            self.focused = None;
            self.pending_open = None;
            return Decision::default();
        };
        let focus = (self.focused != Some(target.key)).then_some(target.key);
        self.focused = Some(target.key);
        let mut open = None;
        if target.opens_submenu {
            let (_, deadline) = *self
                .pending_open
                .get_or_insert((target.key, now.saturating_add(100)));
            if self.pending_open.is_some_and(|(key, _)| key != target.key) {
                self.pending_open = Some((target.key, now.saturating_add(100)));
            } else if now >= deadline {
                open = Some(target.key);
                self.pending_open = None;
            }
        } else {
            self.pending_open = None;
        }
        Decision { focus, open }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn trigger() -> Target {
        Target {
            key: 1,
            opens_submenu: true,
        }
    }
    fn branch() -> Branch {
        Branch {
            trigger: 1,
            anchor: [0., 0., 100., 32.],
            panel: [108., 0., 208., 160.],
        }
    }
    #[test]
    fn opening_is_delayed_and_leaving_or_reset_cancels_it() {
        let mut c = Controller::default();
        assert_eq!(
            c.observe([90., 16.], Some(trigger()), None, 0).focus,
            Some(1)
        );
        assert_eq!(c.deadline(), Some(100));
        assert_eq!(c.observe([90., 16.], Some(trigger()), None, 99).open, None);
        assert_eq!(
            c.observe([90., 16.], Some(trigger()), None, 100).open,
            Some(1)
        );
        c.observe([90., 16.], Some(trigger()), None, 101);
        c.observe([300., 300.], None, None, 110);
        assert_eq!(c.deadline(), None);
        c.observe([90., 16.], Some(trigger()), None, 120);
        c.reset();
        assert_eq!(c.deadline(), None);
    }
    #[test]
    fn diagonal_travel_defers_siblings_until_grace_expires() {
        let mut c = Controller::default();
        c.observe(
            [80., 16.],
            Some(Target {
                opens_submenu: false,
                ..trigger()
            }),
            Some(branch()),
            0,
        );
        let sibling = Some(Target {
            key: 2,
            opens_submenu: false,
        });
        assert_eq!(
            c.observe([95., 45.], sibling, Some(branch()), 10),
            Decision::default()
        );
        assert_eq!(c.deadline(), Some(300));
        assert_eq!(
            c.observe([95., 45.], sibling, Some(branch()), 300).focus,
            Some(2)
        );
    }
    #[test]
    fn child_entry_and_reverse_movement_cancel_grace() {
        let mut c = Controller::default();
        c.observe([80., 16.], None, Some(branch()), 0);
        assert_eq!(
            c.observe(
                [120., 45.],
                Some(Target {
                    key: 3,
                    opens_submenu: false
                }),
                Some(branch()),
                10
            )
            .focus,
            Some(3)
        );
        assert_eq!(c.deadline(), None);
        assert_eq!(
            c.observe(
                [90., 45.],
                Some(Target {
                    key: 2,
                    opens_submenu: false
                }),
                Some(branch()),
                20
            )
            .focus,
            Some(2)
        );
    }
    #[test]
    fn flipped_panels_protect_leftward_diagonal_travel() {
        let mut c = Controller::default();
        let b = Branch {
            trigger: 1,
            anchor: [108., 0., 208., 32.],
            panel: [0., 0., 100., 160.],
        };
        c.observe([128., 16.], None, Some(b), 0);
        assert_eq!(
            c.observe(
                [110., 45.],
                Some(Target {
                    key: 2,
                    opens_submenu: false
                }),
                Some(b),
                10
            ),
            Decision::default()
        );
        assert_eq!(c.deadline(), Some(300));
    }
    #[test]
    fn backwards_clock_and_invalid_coordinates_cancel_old_deadlines() {
        let mut c = Controller::default();
        c.observe([90., 16.], Some(trigger()), None, 1000);
        assert_eq!(c.deadline(), Some(1100));
        assert_eq!(c.observe([90., 16.], Some(trigger()), None, 0).open, None);
        assert_eq!(c.deadline(), Some(100));
        c.observe([f64::NAN, 16.], Some(trigger()), None, 10);
        assert_eq!(c.deadline(), None);
        assert_eq!(
            c.observe([90., 16.], Some(trigger()), None, 20).focus,
            Some(1)
        );
    }
}
