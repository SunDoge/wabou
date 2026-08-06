# @wabou/components

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

Themes are explicit Solid context rather than CSS custom properties:

```tsx
<ComponentsProvider theme={theme()}>
  <App />
</ComponentsProvider>
```

Both `light` and `dark` update reactively without remounting the component tree.

Current components: Button, Badge, Card, Alert, Input, Switch, Progress, and
Separator. Run the gallery while developing:

```bash
mise exec -- bun run wabou dev --app-dir apps/gallery
```
