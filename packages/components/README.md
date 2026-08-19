# Wabou component implementation

This is a private source workspace bundled into the public `@wabou/ui`
package. Applications import styled components from `@wabou/ui`.

Opinionated native components for Wabou applications. The visual language is
inspired by shadcn: useful defaults, straightforward source, and composition
over a closed theme engine.

Behavior lives in `@wabou/primitives`; this package adds UnoCSS geometry,
typography and colors. Runtime-only values such as pressed state and progress
width use reactive inline styles.

```tsx
import { Button, Card, CardContent, Switch } from "@wabou/components";

<Card>
  <CardContent>
    <Switch label="Notifications" />
    <Button>Save changes</Button>
  </CardContent>
</Card>;
```

Themes use build-time checked semantic tokens and a named native color palette.
`ComponentsProvider` exposes the light/dark appearance to components that need
structural choices, while `ColorThemeProvider` switches every semantic color
without remounting the component tree:

```tsx
<ColorThemeProvider theme={theme()} transition={false}>
  <ComponentsProvider theme={theme() === "light" ? "light" : "dark"}>
    <App />
  </ComponentsProvider>
</ColorThemeProvider>
```

Applications define the palette names and semantic tokens in their Wabou Vite
config. The gallery demonstrates `dark`, `light`, and `violet`; components do
not contain theme-name-specific branches.

Current components include Button, Toggle, Checkbox, RadioGroup, Switch, Tabs,
Badge, Card, Alert, Input, TextArea, Progress, Skeleton, Spinner, Kbd, FPS, and
Separator. Run the gallery while developing:

```bash
mise exec -- bun run wabou dev apps/gallery
```
