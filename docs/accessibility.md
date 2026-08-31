# Accessibility

Wabou exposes proof-level native accessibility through GPUI-CE's AccessKit
integration. Explicit semantics are retained with each projected element in
`wabou-shell`, alongside the authoritative focus, layout, and native-widget
state. There is no separate Winit adapter in the production runtime.

The current developer-preview integration writes semantics into GPUI elements.
GPUI publishes the resulting AccessKit tree for each native window. Supported
roles are label, button, text input, image, link, dialog, live regions,
selection controls, combobox/listbox, menu/tree, table/grid, tabs, slider, and
progress indicator. Nodes expose labels, textual and numeric values, disabled
state, common checked/pressed/selected/expanded states, and orientation.

GPUI owns platform focus, window geometry, and AccessKit publication. Wabou's
projection supplies roles and state through GPUI's `Element::a11y_role` and
`Element::write_a11y_info` hooks, so accessibility uses the same retained nodes
and completed GPUI layout as painting and hit testing.

Solid primitives declare the semantic contract; Rust validates and projects
that explicit intent. Platform accessibility APIs never run inside QuickJS.
Behavior tests and DevTools use Wabou's retained semantic snapshots for stable
role/name lookup, but those diagnostics are not a substitute for platform
screen-reader testing.

This is intentionally proof-level accessibility for 0.1. AccessKit action
routing, rich text-editing actions, text-selection announcements,
relationships such as `aria-labelledby`/`aria-describedby`, and broader
screen-reader/platform validation remain future work. Components must still
provide an explicit accessible label where the preview projection cannot derive
one.
