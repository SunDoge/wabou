# Accessibility

Wabou exposes native accessibility through AccessKit. The operating-system
adapter and retained accessibility tree live in `wabou-shell`, alongside the
authoritative window, focus, layout, and native-widget state.

The initial integration publishes every Wabou window as an AccessKit `Window`
node with its title and physical bounds. The adapter receives native focus,
move, and resize events before normal application event handling. It is created
while the winit window is hidden, as required by AccessKit, and requests a
redraw when the platform asks for the initial tree.

Wabou currently uses `accesskit_xplat` as a narrow compatibility bridge because
the official `accesskit_winit` adapter targets winit 0.30 while Wabou uses winit
0.31 beta. This dependency is isolated in `wabou-shell::accessibility` and can
be replaced without changing application APIs when the official adapter catches
up.

Semantic descendants and actions are the next layer. Solid primitives will
declare typed roles, labels, values, and states; Rust will merge those
declarations with final Taffy bounds, native focus, and widget state before
publishing incremental AccessKit updates. Platform accessibility APIs never
run inside QuickJS.
