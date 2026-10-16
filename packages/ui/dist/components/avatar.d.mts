import { ImageResourceHandle, ViewProps } from "../primitives/view.mjs";
import "../primitives.mjs";
import { JSX } from "solid-js";
//#region src/components/avatar.d.ts
export type AvatarSize = "sm" | "default" | "lg";
export interface AvatarProps extends Omit<ViewProps, "children"> {
  image?: ImageResourceHandle;
  /** Full accessible name and source for generated initials. */
  name?: string;
  /** Accessible label override when it should differ from `name`. */
  alt?: string;
  /** Explicit visual fallback. Defaults to initials derived from `name`. */
  fallback?: string;
  size?: AvatarSize;
}
/** Derive at most two stable initials without depending on Intl. */
export declare function avatarInitials(name: string): string;
export declare function Avatar(props: AvatarProps): JSX.Element;
export type AvatarGroupProps = ViewProps;
export declare function AvatarGroup(props: AvatarGroupProps): JSX.Element;
export type AvatarGroupCountProps = ViewProps;
export declare function AvatarGroupCount(props: AvatarGroupCountProps): JSX.Element;
//#endregion
//# sourceMappingURL=avatar.d.mts.map