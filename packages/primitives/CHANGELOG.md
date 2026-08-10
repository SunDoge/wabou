# @wabou/primitives

## 0.1.0

### Minor Changes

- 7d22458: Add configurable native auto-hide scrollbar timing and keep automatic
  scrollbars visible while they are hovered or dragged.
- dd74da0: Add retargetable reduced-motion-aware transitions, measured presence and
  collapsible primitives, then animate Accordion content and disclosure icons.
- 44194a5: Add explicit Row, Column, and Center layout primitives and use Center for
  selection indicators and fixed-size component content.
- 74f054c: Add lifecycle-owned loop, rotation, and pulse controllers plus declarative
  Spin and Pulse primitives, then migrate built-in loading components.
- 2ef790a: Finish native Select long-list scrolling and add a standalone single-line
  ellipsis utility that does not require overflow clipping.
- ca4b28b: Render Spinner as a centered vector arc and recognize semantic theme colors in
  runtime-created utility classes.

### Patch Changes

- d6afffb: Prevent collapsed overflow subtrees from flashing after their clip reaches
  zero, and vertically center text in Kbd.
- c118fe4: Fade collapsible subtrees while their height clip closes so glyphs do not
  flicker at the moving clip edge.
- 84e447c: Add shadcn-inspired Checkbox, RadioGroup, Toggle, Tabs, Skeleton, Spinner, and
  Kbd components with semantic color themes, controlled state, keyboard behavior,
  and ARIA state forwarding from the headless button primitive.
- Updated dependencies [7d22458]
- Updated dependencies [dd74da0]
- Updated dependencies [a733c86]
- Updated dependencies [74f054c]
- Updated dependencies [2ef790a]
- Updated dependencies [ca4b28b]
  - @wabou/solid-renderer@0.1.0
  - @wabou/animation@0.1.0
  - @wabou/style@0.1.0
