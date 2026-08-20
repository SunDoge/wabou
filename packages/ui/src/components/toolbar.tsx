import type { Handle } from "@wabou/core/renderer";
import {
  createComponent,
  createContext,
  createSignal,
  createUniqueId,
  type JSX,
  omit,
  onCleanup,
  useContext,
} from "solid-js";
import { match } from "ts-pattern";
import { View } from "../primitives";
import {
  createControllableState,
  createRovingFocus,
} from "../primitives/interactions";
import { Button, type ButtonProps } from "./button";
import { join } from "./class-names";

export type ToolbarOrientation = "horizontal" | "vertical";

interface ToolbarEntry {
  id: string;
  disabled: () => boolean;
}

interface ToolbarContextValue {
  orientation: () => ToolbarOrientation;
  register(id: string, target: Handle, disabled: () => boolean): () => void;
  activate(id: string): void;
  isTabStop(id: string): boolean;
  move(id: string, key: string): boolean;
}

const ToolbarContext = createContext<ToolbarContextValue>();

export interface ToolbarProps {
  "aria-label": string;
  /** Semantic role used by composite controls built on the toolbar primitive. */
  role?: "toolbar" | "menubar";
  orientation?: ToolbarOrientation;
  loop?: boolean;
  class?: string;
  children?: JSX.Element;
}

/** A compact command surface with one native tab stop and arrow navigation. */
export function Toolbar(props: ToolbarProps): JSX.Element {
  const entries: ToolbarEntry[] = [];
  const [activeId, setActiveId] = createSignal<string | undefined>(undefined, {
    ownedWrite: true,
  });
  const [registryVersion, setRegistryVersion] = createSignal(0, {
    ownedWrite: true,
  });
  const orientation = () => props.orientation ?? "horizontal";
  const enabled = () => entries.filter((entry) => !entry.disabled());
  const roving = createRovingFocus({
    orientation,
    loop: props.loop,
    onMove: setActiveId,
  });
  const context: ToolbarContextValue = {
    orientation,
    register(id, target, disabled) {
      const entry = { id, disabled };
      entries.push(entry);
      const unregisterRoving = roving.register({ id, target, disabled });
      setRegistryVersion((version) => version + 1);
      return () => {
        unregisterRoving();
        const index = entries.indexOf(entry);
        if (index >= 0) entries.splice(index, 1);
        setRegistryVersion((version) => version + 1);
      };
    },
    activate: setActiveId,
    isTabStop(id) {
      registryVersion();
      const candidates = enabled();
      const active = activeId();
      const current = candidates.some((entry) => entry.id === active)
        ? active
        : candidates[0]?.id;
      return id === current;
    },
    move: roving.move,
  };
  return createComponent(ToolbarContext, {
    value: context,
    get children() {
      return (
        <View
          role={props.role ?? "toolbar"}
          aria-label={props["aria-label"]}
          aria-orientation={orientation()}
          class={join(
            "flex-none flex items-center gap-1 rounded-md border border-subtle bg-control p-1",
            match(orientation())
              .with("horizontal", () => "flex-row")
              .with("vertical", () => "flex-col")
              .exhaustive(),
            props.class,
          )}
        >
          {props.children}
        </View>
      );
    },
  });
}

export interface ToolbarButtonProps extends Omit<ButtonProps, "focusOrder"> {}

export function ToolbarButton(props: ToolbarButtonProps): JSX.Element {
  const toolbar = useContext(ToolbarContext);
  if (!toolbar) throw new Error("ToolbarButton must be used inside Toolbar");
  const id = createUniqueId();
  const forwarded = omit(props, "ref", "onFocus", "onKeyDown");
  let unregister: (() => void) | undefined;
  onCleanup(() => unregister?.());
  return (
    <Button
      {...forwarded}
      variant={props.variant ?? "ghost"}
      size={props.size ?? "sm"}
      focusOrder={toolbar.isTabStop(id) ? 0 : -1}
      ref={(node) => {
        unregister?.();
        unregister = toolbar.register(id, node, () => props.disabled ?? false);
        props.ref?.(node);
      }}
      onFocus={(event) => {
        toolbar.activate(id);
        props.onFocus?.(event);
      }}
      onKeyDown={(event) => {
        props.onKeyDown?.(event);
        if (event.defaultPrevented) return;
        if (toolbar.move(id, event.key)) event.preventDefault();
      }}
    />
  );
}

export interface ToolbarToggleProps
  extends Omit<ToolbarButtonProps, "aria-pressed" | "onClick" | "variant"> {
  pressed?: boolean;
  defaultPressed?: boolean;
  onPressedChange?(pressed: boolean): void;
}

export function ToolbarToggle(props: ToolbarToggleProps): JSX.Element {
  const state = createControllableState({
    value: () => props.pressed,
    defaultValue: props.defaultPressed ?? false,
    disabled: () => props.disabled ?? false,
    onChange: props.onPressedChange,
  });
  const forwarded = omit(props, "pressed", "defaultPressed", "onPressedChange");
  return (
    <ToolbarButton
      {...forwarded}
      aria-pressed={state.value()}
      variant={state.value() ? "secondary" : "ghost"}
      onClick={() => state.set(!state.value())}
    />
  );
}

export function ToolbarGroup(props: {
  "aria-label"?: string;
  class?: string;
  children?: JSX.Element;
}): JSX.Element {
  const toolbar = useContext(ToolbarContext);
  if (!toolbar) throw new Error("ToolbarGroup must be used inside Toolbar");
  return (
    <View
      role="group"
      aria-label={props["aria-label"]}
      class={join(
        "flex items-center gap-0.5",
        toolbar.orientation() === "horizontal" ? "flex-row" : "flex-col",
        props.class,
      )}
    >
      {props.children}
    </View>
  );
}

export function ToolbarSeparator(props: { class?: string }): JSX.Element {
  const toolbar = useContext(ToolbarContext);
  if (!toolbar) throw new Error("ToolbarSeparator must be used inside Toolbar");
  return (
    <View
      aria-hidden="true"
      class={join(
        "flex-none bg-subtle",
        toolbar.orientation() === "horizontal" ? "w-px h-5" : "h-px w-5",
        props.class,
      )}
    />
  );
}
