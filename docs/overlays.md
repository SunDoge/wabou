# Overlays

Wabou overlays are retained native subtrees, not CSS stacking tricks. Use
`Portal` only when building a new primitive; applications should normally use
`Popover`, `Modal`, or a component built from them.

The renderer owns two public window-level planes:

- `floating` is for popovers, menus, tooltips, selects, and notifications.
- `modal` is for dialogs that make the rest of the window inert.

Instances on the same plane share one synthetic root. Their containers retain
mount order, so the most recently mounted instance paints and hit-tests last.
The Rust host uses the same plane ordering for painting, hit testing, focus,
and accessibility. A modal restricts focus and semantic actions to the last
painted modal subtree; background scrollbars and pointer targets are blocked by
the modal backdrop.

`Popover` delegates anchor measurement and collision-aware placement to the
native GPUI positioner. It also supplies outside click dismissal, Escape
dismissal, and trigger focus restoration. `Modal`
supplies backdrop and Escape dismissal, initial focus, focus restoration, and
host-enforced modal isolation. Keep open state in Solid; the host owns only the
derived stacking, hit-test, and semantic projections.

When adding an overlay component:

1. Reuse `Popover` or `Modal` before using `Portal` directly.
2. Give the content an explicit semantic role and accessible name.
3. Stop pointer propagation at the content boundary so the full-window layer
   remains the outside-click target.
4. Test Escape, outside click, controlled open state, and focus restoration.
5. Add a native overlay test when geometry, scrollbars, or modal semantics are
   involved; a JavaScript tree test alone does not prove native ordering.

The `system` and `debug` planes are reserved by the host and are intentionally
not exposed by the public `Portal` API.
