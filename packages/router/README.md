# `@wabou/router`

Solid-native in-memory routing for Wabou applications. It provides navigation
without emulating browser history, location, links, or DOM events.

```tsx
import { MemoryRouter, Route, useNavigate, useParams } from "@wabou/router";

function Story() {
  const params = useParams<{ id: string }>();
  const navigate = useNavigate();
  return <Button onClick={() => navigate(-1)}>Back from {params.id}</Button>;
}

mount(() => (
  <MemoryRouter>
    <Route path="/" component={Home} />
    <Route path="/story/:id" component={Story} />
  </MemoryRouter>
));
```

The router intentionally focuses on native application needs: a deterministic
memory stack, reactive locations and parameters, root/nested layouts, static,
optional and wildcard path segments, and push/replace/back/forward navigation.
It does not implement browser history, SSR, document links, or Solid Router's
data APIs.
