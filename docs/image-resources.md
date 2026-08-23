# Image resources

Wabou treats decoded images as explicit host resources, not as web-style `src`
attributes. A path or URL never becomes renderer state by assignment.

```tsx
const image = createOwnedImageResource(() => ({
  kind: "file",
  path: selectedPath(),
}));

<Image resource={image.resource()?.handle} />
```

The ownership rules are deliberately small:

- `createFileImageResource` and `createNetworkImageResource` create a new identity.
  They do not deduplicate handles.
- `Image`, `ImageList`, and `ImageViewport` borrow a handle and never release it.
- `releaseImageResource` deterministically invalidates the handle. A reused slot has
  a different generation, so stale handles cannot address the replacement.
- `createOwnedImageResource` is the Solid convenience API. It clears the borrowed
  handle before releasing on source replacement or owner cleanup, and releases a
  resource whose asynchronous creation finishes after disposal.
- Rust applications can share the same registry with rendering through
  `HostBuilder::with_image_resources` and use the handle for OCR, export, or other
  native work.

An `ImageResource` retains original decoded pixels and their original dimensions.
Rendering uses a bounded drawable derived from those pixels; view fit, zoom, pan,
HiDPI scaling, and thumbnails do not change the source resource or annotation
coordinates. Future resolution-specific drawable caches can therefore be added
without changing the public handle or application semantics.
