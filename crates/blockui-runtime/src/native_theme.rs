//! Stable-host theme inheritance and recipes; no plugin callbacks are retained.
use super::*;
use blockui_abi::theme::{Metrics, Palette};
pub(super) fn color(value: u32) -> Color {
    let [r, g, b, a] = value.to_be_bytes();
    Color::from_rgba8(r, g, b, a)
}
impl<M: NativeModel> NativeSource<M> {
    pub(super) fn rebuild_themes(&mut self) {
        let system_dark = self
            .metrics
            .is_some_and(|m| m.color_scheme == Some(shell::ColorScheme::Dark));
        #[cfg(test)]
        {
            self.theme_rebuilds += 1;
        }
        let mut next = BTreeMap::new();
        for key in &self.actor.view().order {
            let n = self.actor.view().bindings[key].node;
            let (parent_theme, parent_density) = next.get(&n.parent).copied().unwrap_or((0, 0));
            let mode = match n.presentation.theme {
                0 => parent_theme,
                3 => {
                    if system_dark {
                        2
                    } else {
                        1
                    }
                }
                mode => mode,
            };
            let density = if n.presentation.density == 0 {
                parent_density
            } else {
                n.presentation.density
            };
            let resolved = (mode, density);
            if self.themes.get(key) != Some(&resolved) {
                self.paint_dirty.insert(*key);
            }
            next.insert(*key, resolved);
        }
        self.themes = next;
        self.theme_system_dark = system_dark;
    }
    pub(super) fn palette(&self, key: u64) -> Option<Palette> {
        self.themes
            .get(&key)
            .filter(|(mode, _)| *mode != 0)
            .map(|(mode, _)| Palette::resolve(*mode, false))
    }
    pub(super) fn style_theme(
        &self,
        binding: &crate::Binding,
        style: &mut Style,
        paint: &mut Paint,
    ) {
        let n = binding.node;
        let (mode, density) = self.themes.get(&n.key).copied().unwrap_or_default();
        if mode == 0 && density == 0 {
            return;
        }
        let metrics = Metrics::resolve(density);
        if n.presentation.font_size == 0.0 {
            paint.font_size = metrics.font_size;
        }
        paint.line_height = Some((1.45, true));
        if n.semantics.role == semantics::HEADING {
            paint.font_weight = 600.0;
        }
        if (matches!(n.kind, BUTTON | CHECKBOX | SELECT) && n.height <= 40.0
            || n.kind == ENTRY && n.height <= 44.0)
            && n.semantics.role != semantics::RADIO
        {
            style.size.height = Dimension::length(metrics.control_height);
            style.padding.top = LengthPercentage::length(4.0);
            style.padding.bottom = LengthPercentage::length(4.0);
            if n.kind != CHECKBOX {
                style.padding.left = LengthPercentage::length(metrics.control_padding_x);
                style.padding.right = LengthPercentage::length(if n.kind == SELECT {
                    28.0
                } else {
                    metrics.control_padding_x
                });
            }
        }
        let Some(p) = self.palette(n.key) else {
            return;
        };
        paint.text_color = color(match n.presentation.text_tone {
            1 => p.text_muted,
            2 => p.accent,
            3 => p.danger,
            _ => p.text,
        });
        if paint.border.is_some() {
            paint.border = Some((1.0, color(p.border)));
        }
        if n.parent == 0 || n.kind == SCROLL || n.presentation.surface == 1 {
            paint.background = Some(color(p.canvas));
        }
        if n.presentation.surface == 2
            || n.kind == DIALOG
            || semantics::is_anchored_panel(n.semantics.role)
        {
            paint.background = Some(color(p.panel));
            paint.border = Some((1.0, color(p.border)));
        }
        if matches!(n.kind, ENTRY | TEXTAREA | SELECT) {
            paint.background = Some(color(if n.disabled != 0 { p.muted } else { p.panel }));
            paint.border_radii = CornerRadii::uniform(metrics.control_radius);
        }
        let disclosure =
            n.semantics.expanded != 0 && !semantics::is_anchored_trigger(n.semantics.role);
        if n.kind == BUTTON {
            let is_menu = matches!(
                n.semantics.role,
                semantics::MENU_ITEM
                    | semantics::MENU_CHECKBOX
                    | semantics::MENU_RADIO
                    | semantics::SUBMENU_TRIGGER
            );
            let quiet = n.button_variant == 3
                || disclosure
                || is_menu
                || n.semantics.role == semantics::TAB
                || n.semantics.role == semantics::RADIO;
            if n.presentation.width == 0.0 && !quiet && self.grid_owner(n.key).is_none() {
                style.size.width = Dimension::auto();
                style.flex_basis = Dimension::auto();
                style.flex_grow = 0.0;
                style.align_self = Some(AlignSelf::FLEX_START);
            }
            paint.font_weight = 500.0;
            paint.border_radii = CornerRadii::uniform(metrics.control_radius);
            let (bg, fg) = if quiet {
                (
                    None,
                    if n.button_variant == 2 {
                        p.danger
                    } else {
                        p.text
                    },
                )
            } else {
                match n.button_variant {
                    1 => (Some(p.panel), p.text),
                    2 => (Some(p.danger), p.on_danger),
                    _ => (Some(p.accent), p.on_accent),
                }
            };
            paint.background = bg.map(color);
            paint.text_color = color(fg);
            paint.border = if quiet {
                None
            } else {
                Some((
                    1.0,
                    color(if n.button_variant == 1 {
                        p.border
                    } else {
                        bg.unwrap_or(p.border)
                    }),
                ))
            };
            style.border = Rect::length(1.0);
            if n.semantics.role == semantics::TAB && n.semantics.selected != 0 {
                paint.background = Some(color(p.panel));
            }
            if n.semantics.current == 1 && n.button_variant == 3 {
                paint.background = Some(color(p.muted));
                paint.text_color = color(p.accent);
            }
            if n.disabled != 0 {
                paint.background = (!quiet).then_some(color(p.muted));
                paint.text_color = color(p.text_muted);
                paint.border = (!quiet).then_some((1.0, color(p.border)));
            }
        }
        if n.presentation.surface == 2 {
            paint.shadows = vec![shell::style::Shadow {
                offset_x: 0.0,
                offset_y: 1.0,
                spread: 0.0,
                std_dev: 1.0,
                color: Color::from_rgba8(0, 0, 0, 8),
                radius: None,
            }];
        }
        if n.kind == DIALOG || semantics::is_anchored_panel(n.semantics.role) {
            paint.shadows = vec![shell::style::Shadow {
                offset_x: 0.0,
                offset_y: 8.0,
                spread: -2.0,
                std_dev: 12.0,
                color: Color::from_rgba8(0, 0, 0, 45),
                radius: None,
            }];
        }
        if n.semantics.role == semantics::TAB_LIST {
            paint.background = Some(color(p.muted));
        }
        if matches!(
            n.semantics.role,
            semantics::HORIZONTAL_SEPARATOR | semantics::VERTICAL_SEPARATOR
        ) {
            paint.background = Some(color(p.border));
        }
        if n.semantics.role == semantics::BADGE {
            let (bg, fg) = match n.button_variant {
                1 => (p.muted, p.text),
                2 => (p.panel, p.text),
                3 => (p.danger, p.on_danger),
                _ => (p.accent, p.on_accent),
            };
            paint.background = Some(color(bg));
            paint.text_color = color(fg);
        }
        if matches!(
            n.semantics.role,
            semantics::BADGE | semantics::TOOLTIP | semantics::MENU_LABEL
        ) {
            paint.font_size = 12.0;
        }
        if n.semantics.role == semantics::MENU_LABEL {
            paint.text_color = color(p.text_muted);
        }
        if n.semantics.role == semantics::TOOLTIP {
            paint.background = Some(color(p.text));
            paint.text_color = color(p.panel);
        }
        if n.disabled != 0 {
            paint.text_color = color(p.text_muted);
        }
    }
    pub(super) fn theme_interaction(&self, key: u64, paint: &mut Paint, pressed: bool) -> bool {
        let Some(p) = self.palette(key) else {
            return false;
        };
        let n = self.actor.view().bindings[&key].node;
        if n.kind == BUTTON {
            let quiet = n.button_variant == 3
                || (n.semantics.expanded != 0 && !semantics::is_anchored_trigger(n.semantics.role))
                || matches!(
                    n.semantics.role,
                    semantics::RADIO
                        | semantics::TAB
                        | semantics::MENU_ITEM
                        | semantics::MENU_CHECKBOX
                        | semantics::MENU_RADIO
                        | semantics::SUBMENU_TRIGGER
                );
            paint.background = Some(color(if quiet || n.button_variant == 1 {
                if pressed { p.pressed } else { p.hover }
            } else if n.button_variant == 2 {
                if pressed {
                    p.danger_pressed
                } else {
                    p.danger_hover
                }
            } else if pressed {
                p.accent_pressed
            } else {
                p.accent_hover
            }));
        } else if matches!(n.kind, ENTRY | TEXTAREA | SELECT) {
            paint.border = Some((1.0, color(p.focus)));
        } else if n.kind == CHECKBOX {
            paint.background = Some(color(p.hover));
        }
        true
    }
}
