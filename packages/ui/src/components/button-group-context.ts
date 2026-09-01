import {
  type Accessor,
  createContext,
  createSignal,
  onCleanup,
  useContext,
} from "solid-js";

export type ButtonGroupOrientation = "horizontal" | "vertical";
export type ButtonGroupButtonSize = "sm" | "default" | "lg" | "icon";
export type ButtonGroupButtonVariant =
  | "default"
  | "secondary"
  | "outline"
  | "ghost"
  | "destructive";

export type ButtonGroupItemPosition = "only" | "first" | "middle" | "last";

export interface ButtonGroupItemContext {
  orientation: Accessor<ButtonGroupOrientation>;
  position: Accessor<ButtonGroupItemPosition>;
  size: Accessor<ButtonGroupButtonSize>;
  variant: Accessor<ButtonGroupButtonVariant>;
  disabled: Accessor<boolean>;
}

export interface ButtonGroupContextValue {
  orientation: Accessor<ButtonGroupOrientation>;
  registerItem(): ButtonGroupItemContext;
}

export const ButtonGroupContext = createContext<ButtonGroupContextValue | null>(
  null,
);

export function createButtonGroupContext(
  orientation: Accessor<ButtonGroupOrientation>,
  options: {
    size: Accessor<ButtonGroupButtonSize>;
    variant: Accessor<ButtonGroupButtonVariant>;
    disabled: Accessor<boolean>;
  },
): ButtonGroupContextValue {
  const [items, setItems] = createSignal<readonly object[]>([], {
    ownedWrite: true,
  });
  return {
    orientation,
    registerItem() {
      const token = {};
      setItems((current) => [...current, token]);
      onCleanup(() =>
        setItems((current) => current.filter((item) => item !== token)),
      );
      return {
        orientation,
        size: options.size,
        variant: options.variant,
        disabled: options.disabled,
        position: () => {
          const current = items();
          const index = current.indexOf(token);
          if (current.length <= 1) return "only";
          if (index === 0) return "first";
          if (index === current.length - 1) return "last";
          return "middle";
        },
      };
    },
  };
}

export function buttonGroupItemCorners(item: ButtonGroupItemContext): string {
  const position = item.position();
  if (position === "only") return "rounded-lg";
  if (position === "middle") return "rounded-none";
  if (item.orientation() === "horizontal")
    return position === "first" ? "rounded-l-lg" : "rounded-r-lg";
  return position === "first" ? "rounded-t-lg" : "rounded-b-lg";
}

export function useButtonGroupItem(): ButtonGroupItemContext | undefined {
  return useContext(ButtonGroupContext)?.registerItem();
}

export function useButtonGroupOrientation():
  | ButtonGroupOrientation
  | undefined {
  return useContext(ButtonGroupContext)?.orientation();
}
