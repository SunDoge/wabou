# Accessibility

Wabou publishes its retained semantic tree through AccessKit. Solid primitives
declare roles, labels, values, state, and relationships; the Rust projection
combines those declarations with completed Taffy geometry and native focus.
Platform accessibility APIs never execute inside QuickJS.

The developer-preview integration supports labels, buttons, text inputs,
images, links, dialogs, live regions, selection controls, combobox/listbox,
menu/tree, table/grid, tabs, sliders, and progress indicators. Nodes can expose
textual or numeric values, disabled state, checked/pressed/selected/expanded
state, and orientation.

Behavior tests and DevTools query the same retained semantic snapshot for
stable role/name lookup. That makes structural regressions testable, but it is
not a substitute for screen-reader testing on each platform.

Accessibility remains preview-level for 0.1. Broader action routing, rich text
editing actions, selection announcements, relationship coverage, and platform
validation are ongoing. Components must provide an explicit accessible label
when it cannot be derived reliably from their authored descendants.
