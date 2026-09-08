# Component gallery

Interactive catalogue for `@wabou/ui` components and Wabou platform features,
including shadcn-inspired native controls, animation, native windows, and Rust
custom widgets. Use the header theme control to cycle through the compiled
`dark`, `light`, and `violet` semantic palettes.

```bash
mise exec -- bun run wabou dev apps/gallery
```

Enable the renderer-native performance HUD without adding a reactive clock to
the Gallery's Solid tree:

```bash
WABOU_PERFORMANCE_HUD=1 mise exec -- bun run wabou dev apps/gallery
```

The shell paints the HUD after the application scene. Its updates do not mutate
the Gallery's Solid tree, so measurement does not itself rebuild the page.
