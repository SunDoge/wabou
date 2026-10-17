import { ImageResourceHandle } from "./view.mjs";
import { Accessor } from "solid-js";
//#region src/primitives/image-resource.d.ts
export interface ImageResourceDescriptor {
  handle: ImageResourceHandle;
  width: number;
  height: number;
}
export type ImageResourceRequest = {
  kind: "file";
  path: string;
} | {
  kind: "network";
  url: string;
};
export interface OwnedImageResource {
  resource: Accessor<ImageResourceDescriptor | undefined>;
  loading: Accessor<boolean>;
  error: Accessor<unknown>;
}
/** Explicitly create a new resource from a host file. No identity deduplication occurs. */
export declare function createFileImageResource(path: string): Promise<ImageResourceDescriptor>;
/** Explicitly create a new resource from an HTTP(S) response. */
export declare function createNetworkImageResource(url: string): Promise<ImageResourceDescriptor>;
/** Deterministically release a resource. Images only borrow their handle. */
export declare function releaseImageResource(handle: ImageResourceHandle): Promise<boolean>;
/**
 * Create a resource owned by the current Solid owner. Source replacement and
 * owner cleanup clear the borrowed handle before releasing the native resource.
 */
export declare function createOwnedImageResource(request: Accessor<ImageResourceRequest | undefined>): OwnedImageResource;
//#endregion
//# sourceMappingURL=image-resource.d.mts.map