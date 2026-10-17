import { type Host, useHost } from "@wabou/core";
import { type Accessor, createEffect, createSignal, onCleanup } from "solid-js";
import type { ImageResourceHandle } from "./view";

interface ImageResourcesHost {
  createFile(request: { path: string }): Promise<ImageResourceDescriptor>;
  createNetwork(request: { url: string }): Promise<ImageResourceDescriptor>;
  release(handle: ImageResourceHandle): boolean | PromiseLike<boolean>;
}

interface ResourceHost extends Host {
  imageResources: ImageResourcesHost;
}

export interface ImageResourceDescriptor {
  handle: ImageResourceHandle;
  width: number;
  height: number;
}

export type ImageResourceRequest =
  | { kind: "file"; path: string }
  | { kind: "network"; url: string };

export interface OwnedImageResource {
  resource: Accessor<ImageResourceDescriptor | undefined>;
  loading: Accessor<boolean>;
  error: Accessor<unknown>;
}

function host(): ImageResourcesHost {
  return useHost<ResourceHost>().imageResources;
}

/** Explicitly create a new resource from a host file. No identity deduplication occurs. */
export function createFileImageResource(
  path: string,
): Promise<ImageResourceDescriptor> {
  return host().createFile({ path });
}

/** Explicitly create a new resource from an HTTP(S) response. */
export function createNetworkImageResource(
  url: string,
): Promise<ImageResourceDescriptor> {
  return host().createNetwork({ url });
}

/** Deterministically release a resource. Images only borrow their handle. */
export function releaseImageResource(
  handle: ImageResourceHandle,
): Promise<boolean> {
  return Promise.resolve(host().release(handle));
}

/**
 * Create a resource owned by the current Solid owner. Source replacement and
 * owner cleanup clear the borrowed handle before releasing the native resource.
 */
export function createOwnedImageResource(
  request: Accessor<ImageResourceRequest | undefined>,
): OwnedImageResource {
  const [resource, setResource] = createSignal<ImageResourceDescriptor>();
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal<unknown>();
  let owned: ImageResourceDescriptor | undefined;
  let revision = 0;

  const releaseOwned = () => {
    const previous = owned;
    owned = undefined;
    setResource(undefined);
    if (previous) void releaseImageResource(previous.handle);
  };

  createEffect(request, (source) => {
    const currentRevision = ++revision;
    releaseOwned();
    setError(undefined);
    if (!source) {
      setLoading(false);
      return;
    }
    setLoading(true);
    const pending =
      source.kind === "file"
        ? createFileImageResource(source.path)
        : createNetworkImageResource(source.url);
    void pending.then(
      (next) => {
        if (currentRevision !== revision) {
          void releaseImageResource(next.handle);
          return;
        }
        owned = next;
        setResource(next);
        setLoading(false);
      },
      (reason) => {
        if (currentRevision !== revision) return;
        setError(reason);
        setLoading(false);
      },
    );
  });

  onCleanup(() => {
    revision++;
    releaseOwned();
  });
  return { resource, loading, error };
}
