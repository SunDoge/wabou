import { type FileDropPosition, useFileDrop, useHost } from "@wabou/core";
import type { Handle, LayoutRect } from "@wabou/core/renderer";
import fileUp from "lucide-static/icons/file-up.svg?raw";
import { createEffect, createSignal, type JSX, Show } from "solid-js";
import { Icon, Text, View, type ViewProps } from "../primitives";
import { mergeClasses } from "@wabou/core/style";

export interface DropZoneProps extends Omit<ViewProps, "class" | "children"> {
  /** Called with paths accepted by this zone after a native drop. */
  onDrop: (paths: readonly string[]) => void;
  /** Return true for paths this zone accepts. All paths are accepted by default. */
  accept?: (path: string) => boolean;
  /** Reports paths rejected by `accept` without hiding a partially valid drop. */
  onRejected?: (paths: readonly string[]) => void;
  disabled?: boolean;
  label?: string;
  activeLabel?: string;
  description?: string;
  class?: string;
}

/** Inclusive hit test in the logical coordinate space shared with native DnD. */
export function pointInLayoutRect(
  point: FileDropPosition,
  rect: LayoutRect,
): boolean {
  return (
    point.x >= rect.x &&
    point.y >= rect.y &&
    point.x <= rect.x + rect.width &&
    point.y <= rect.y + rect.height
  );
}

/**
 * A component-local target for native filesystem drops.
 *
 * Native backends publish window-relative coordinates, so the zone measures
 * its completed native layout before claiming an event. Events without a
 * position are deliberately ignored instead of being delivered ambiguously to
 * every mounted zone.
 */
export function DropZone(props: DropZoneProps): JSX.Element {
  const host = useHost();
  const [active, setActive] = createSignal(false);
  let target: Handle | undefined;

  const contains = (position: FileDropPosition | null): boolean => {
    if (!position || !target) return false;
    const rect = host.layout.measure(target);
    return rect !== null && pointInLayoutRect(position, rect);
  };

  useFileDrop((event) => {
    if (props.disabled) {
      setActive(false);
      return;
    }
    if (event.phase === "left") {
      setActive(false);
      return;
    }
    const inside = contains(event.position);
    if (event.phase !== "dropped") {
      setActive(inside);
      return;
    }

    setActive(false);
    if (!inside) return;
    const accepted = props.accept
      ? event.paths.filter((path) => props.accept?.(path))
      : event.paths;
    const rejected = props.accept
      ? event.paths.filter((path) => !props.accept?.(path))
      : [];
    if (accepted.length > 0) props.onDrop(accepted);
    if (rejected.length > 0) props.onRejected?.(rejected);
  });

  createEffect(
    () => props.disabled,
    (disabled) => {
      if (disabled) setActive(false);
    },
  );

  const surface = (isActive: boolean) => (
    <View
      {...props}
      ref={(node) => {
        target = node;
      }}
      role="group"
      aria-label={props.label ?? "Drop files"}
      aria-disabled={props.disabled ? "true" : undefined}
      class={mergeClasses(
        "w-full min-w-0 min-h-36 px-6 py-5 flex flex-col items-center justify-center gap-3 rounded-xl border-2 text-center",
        isActive ? "border-accent bg-selected" : "border-strong bg-input",
        props.disabled && "opacity-50",
        props.class,
      )}
    >
      <View
        class={mergeClasses(
          "w-10 h-10 flex-none items-center justify-center rounded-lg",
          isActive ? "bg-accent text-on-accent" : "bg-control text-secondary",
        )}
      >
        <Icon source={fileUp} size={20} />
      </View>
      <Text class="text-sm font-semibold text-primary">
        {active()
          ? (props.activeLabel ?? "Release to add files")
          : (props.label ?? "Drop files here")}
      </Text>
      <Text class="max-w-md text-xs text-muted whitespace-normal">
        {props.description ?? "Drag files from the desktop into this area."}
      </Text>
    </View>
  );

  return (
    <Show when={active()} fallback={surface(false)}>
      {surface(true)}
    </Show>
  );
}
