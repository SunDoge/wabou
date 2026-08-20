# Wabou component implementation

This is the styled component layer inside the public `@wabou/ui` package.

Opinionated native components for Wabou applications. The visual language is
inspired by shadcn: useful defaults, straightforward source, and composition
over a closed theme engine.

Behavior lives in the sibling primitive layer; this directory adds UnoCSS geometry,
typography and colors. Runtime-only values such as pressed state and progress
width use reactive inline styles.

```tsx
import { Button, Card, CardContent, Switch } from "@wabou/ui";

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

Current components include Button, Toggle, Checkbox, RadioGroup, Rating, Switch,
Tabs, Badge, Card, Alert, Input, NumberField, TextArea, DirectoryPicker,
Progress, Pagination, Skeleton, Spinner, Kbd, FPS, and Separator. Run the
gallery while developing:

`Pagination` supports both a legacy composition-only surface and managed,
1-indexed state. Supplying `count` enables automatic boundary/sibling ranges;
`PaginationItems`, `PaginationPrevious`, and `PaginationNext` then consume the
shared state without requiring application-side range or clamping logic:

```tsx
<Pagination count={24} page={page()} onPageChange={setPage}>
  <PaginationContent>
    <PaginationPrevious />
    <PaginationItems />
    <PaginationNext />
  </PaginationContent>
</Pagination>
```

`TabsList` and `TabsTrigger` accept `unstyled` when an application needs a
card, sidebar, or other custom tab presentation. The components retain native
tab semantics, selection and roving keyboard focus; the caller owns layout,
paint and focus-visible styling. Style the trigger itself as the interactive
surface instead of nesting another independently sized button or card shell.

`AdaptiveSplitPane` keeps a master region elastic and moves an open detail
region into a modal plane when the application-owned window query becomes
compact. Selection remains application state; the component only owns the
presentation boundary:

```tsx
<AdaptiveSplitPane compact={compact()}>
  <AdaptiveSplitPaneMain>{list}</AdaptiveSplitPaneMain>
  <Show when={selected()}>
    <AdaptiveSplitPaneDetail
      open={true}
      aria-label="Item details"
      onOpenChange={(open) => !open && setSelected(undefined)}
    >
      {details}
    </AdaptiveSplitPaneDetail>
  </Show>
</AdaptiveSplitPane>
```

In compact mode, the detail surface uses most of the viewport while retaining
a safe outer margin. Put independently scrollable detail content inside it;
`modalClass` can override the default dimensions for specialized surfaces.

For dialogs whose content can grow, bound the dialog surface and place only
the shrinking middle region in `DialogScrollBody`. Headers and footers remain
fixed while the body receives the native scroll viewport:

```tsx
<Dialog contentClass="max-h-11/12 overflow-hidden">
  <DialogHeader>{title}</DialogHeader>
  <DialogScrollBody contentClass="flex flex-col gap-3">
    {form}
  </DialogScrollBody>
  <DialogFooter>{actions}</DialogFooter>
</Dialog>
```

```bash
mise exec -- bun run wabou dev apps/gallery
```
