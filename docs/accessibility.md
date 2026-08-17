# Accessibility

Wabou exposes native accessibility through AccessKit. The operating-system
adapter and retained accessibility tree live in `wabou-shell`, alongside the
authoritative window, focus, layout, and native-widget state.

The current developer-preview integration publishes a retained semantic tree
under every AccessKit `Window`. Supported roles are label, button, text input,
image, link, dialog, live regions, selection controls, combobox/listbox,
menu/tree, table/grid, tabs, slider, and progress indicator. Nodes expose
labels, textual and numeric values, disabled state, common
checked/pressed/selected/expanded states, completed layout bounds, child order,
and native focus. Click, focus, and blur requests are routed back through the
same host interaction path used by pointer and keyboard input.

`aria-controls` IDREF lists and `aria-activedescendant` are resolved to stable
native node identifiers after Solid reconciliation. Missing, hidden, or
presentation-only targets are ignored. AccessKit publication also removes any
relationship target outside the currently exposed modal subtree, so a retained
diagnostic snapshot cannot create a dangling platform relationship.

The adapter receives native focus, move, and resize events before normal
application event handling. It is created while the winit window is hidden, as
required by AccessKit, and requests a redraw when the platform asks for the
initial tree. Semantic snapshots are revisioned so an unchanged tree is not
republished every frame.

Wabou currently uses `accesskit_xplat` as a narrow compatibility bridge because
the official `accesskit_winit` adapter targets winit 0.30 while Wabou uses winit
0.31 beta. This dependency is isolated in `wabou-shell::accessibility` and can
be replaced without changing application APIs when the official adapter catches
up.

Modal planes expose only the topmost modal subtree and reject background focus
or semantic actions. Solid primitives declare the semantic contract; Rust
merges it with final Taffy bounds, native focus, and widget state. Platform
accessibility APIs never run inside QuickJS.

This is intentionally proof-level accessibility for 0.1. Rich text-editing
actions, text-selection announcements, relationships such as
`aria-labelledby`/`aria-describedby`, and broader screen-reader/platform
validation remain future work. Components must still provide an explicit
accessible label where the preview bridge cannot derive one.
