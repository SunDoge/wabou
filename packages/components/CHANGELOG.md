# @wabou/components

## 0.1.0-alpha.1

### Minor Changes

- 02a0e24: Add a reusable interactions layer with Elm-style machines, controlled state,
  collections, typeahead, selection, roving focus, and disclosure behavior. Use
  it to power the shadcn-inspired accordion, collapsible, tabs, and radio group.
- d1dd1c6: Add native single-select behavior and a shadcn-inspired Select component with
  keyboard navigation, typeahead, disabled options, and listbox semantics.
- 9ca7615: Add shadcn-inspired dialog, accordion, collapsible, avatar, field, input-group,
  empty-state, and button-group components for native Wabou applications.
- 84e447c: Add shadcn-inspired Checkbox, RadioGroup, Toggle, Tabs, Skeleton, Spinner, and
  Kbd components with semantic color themes, controlled state, keyboard behavior,
  and ARIA state forwarding from the headless button primitive.
- 14a6081: Use tree-shaken Lucide chevrons for Accordion and Select, and ignore Lucide's
  non-utility metadata classes by default.

### Patch Changes

- dd74da0: Add retargetable reduced-motion-aware transitions, measured presence and
  collapsible primitives, then animate Accordion content and disclosure icons.
- 44194a5: Add explicit Row, Column, and Center layout primitives and use Center for
  selection indicators and fixed-size component content.
- 74f054c: Add lifecycle-owned loop, rotation, and pulse controllers plus declarative
  Spin and Pulse primitives, then migrate built-in loading components.
- 5cb2992: Center radio and checkbox indicator contents with a shared flex layout.
- 2ef790a: Finish native Select long-list scrolling and add a standalone single-line
  ellipsis utility that does not require overflow clipping.
- d6afffb: Prevent collapsed overflow subtrees from flashing after their clip reaches
  zero, and vertically center text in Kbd.
- c118fe4: Fade collapsible subtrees while their height clip closes so glyphs do not
  flicker at the moving clip edge.
- ca4b28b: Render Spinner as a centered vector arc and recognize semantic theme colors in
  runtime-created utility classes.
- Updated dependencies [7d22458]
- Updated dependencies [dd74da0]
- Updated dependencies [02a0e24]
- Updated dependencies [44194a5]
- Updated dependencies [74f054c]
- Updated dependencies [d1dd1c6]
- Updated dependencies [2ef790a]
- Updated dependencies [d6afffb]
- Updated dependencies [c118fe4]
- Updated dependencies [ca4b28b]
- Updated dependencies [84e447c]
  - @wabou/solid-renderer@0.1.0-alpha.1
  - @wabou/primitives@0.1.0-alpha.1
  - @wabou/animation@0.1.0-alpha.1
  - @wabou/core@0.1.0-alpha.1
