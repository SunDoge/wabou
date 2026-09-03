# Component gallery

Interactive catalogue for `@wabou/ui` components and Wabou platform features,
including shadcn-inspired native controls, animation, native windows, and Rust
custom widgets. Use the header theme control to cycle through the compiled
`dark`, `light`, and `violet` semantic palettes.

```bash
mise exec -- bun run wabou dev apps/gallery
```

Enable the GPUI-native performance overlay without adding a reactive clock to
the Gallery's Solid tree:

```bash
WABOU_PERFORMANCE_HUD=1 mise exec -- bun run wabou dev apps/gallery
```

The HUD is an independent GPUI view boundary. Its updates do not materialize
the Gallery projection tree, so it is suitable for checking pages such as
Colors without making the measurement itself rebuild the page.
