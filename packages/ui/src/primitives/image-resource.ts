import {
  bindJsonCapability,
  type Host,
  type NativeCapability,
  useHost,
} from "@wabou/core";
import type { ImageResourceHandle } from "./view";
import { createEffect, createSignal, onCleanup, type Accessor } from "solid-js";

interface ImageResourcesCapability extends NativeCapability {
  __wabouCapabilityVersion: number;
  createFile(request: string): string | PromiseLike<string>;
  createNetwork(request: string): string | PromiseLike<string>;
  release(request: string): string | PromiseLike<string>;
}

interface ResourceHost extends Host {
  imageResources: ImageResourcesCapability;
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

function call() {
  return bindJsonCapability(useHost<ResourceHost>().imageResources, {
    name: "imageResources",
    version: 1,
  });
}

/** Explicitly create a new resource from a host file. No identity deduplication occurs. */
export function createFileImageResource(path: string): Promise<ImageResourceDescriptor> {
  return call()<ImageResourceDescriptor>("createFile", { path });
}

/** Explicitly create a new resource from an HTTP(S) response. */
export function createNetworkImageResource(url: string): Promise<ImageResourceDescriptor> {
  return call()<ImageResourceDescriptor>("createNetwork", { url });
}

/** Deterministically release a resource. Images only borrow their handle. */
export function releaseImageResource(handle: ImageResourceHandle): Promise<boolean> {
  return call()<boolean>("release", handle);
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
