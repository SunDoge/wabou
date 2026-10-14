# Resident Rust resources

Wabou applications are ordinary statically linked Rust programs. SolidJS owns
the reloadable UI, while the Rust host owns databases, network sessions,
background workers, models, and other long-lived native resources.

`ResidentResources<Family, Value>` stores those values outside the JavaScript
module graph. JavaScript receives only a typed generational `ResourceKey` with
the wire form `{ lo, hi }`. Removing a value invalidates every stale copy of its
handle, including handles retained by an old UI module.

```rust
use std::sync::Arc;
use wabou::{ResidentResources, ResourceKey};

enum Database {}
type DatabaseKey = ResourceKey<Database>;

let databases = ResidentResources::<Database, Arc<MyDatabase>>::new();

let open_databases = databases.clone();
host = host.capability(bindings::DATABASE, move |capability| {
    let open_databases = open_databases.clone();
    capability.method(bindings::OPEN, move |request| {
        let open_databases = open_databases.clone();
        async move {
            let key: DatabaseKey = open_databases.get_or_insert_with(
                request.workspace,
                || Arc::new(MyDatabase::open()),
            );
            Ok::<_, String>(key)
        }
    })
});
```

There are two useful reload cases:

- Solid Refresh preserves the component signal that contains a handle. The same
  `{ lo, hi }` continues to resolve because the Rust store never reloaded.
- A full UI remount loses JavaScript variables. Calling the application's open
  method again with the same semantic name returns the existing live resource
  and its original handle.

Use unnamed `insert` for resources whose lifetime is controlled by an explicit
close operation. Use `get_or_insert_with` for resources that UI code must be
able to rediscover after a full remount. Resource cleanup stays explicit:
dropping a component does not implicitly close a database or connection.

Registry access is synchronized. Keep `with` and `with_mut` callbacks short.
For async work, store an `Arc<T>`, clone it with `with`, release the registry
lock, and then await using the cloned value.
