import { createContext, useContext } from "solid-js";

export type ButtonGroupOrientation = "horizontal" | "vertical";

export const ButtonGroupContext = createContext<ButtonGroupOrientation | null>(
  null,
);

export function useButtonGroupOrientation():
  | ButtonGroupOrientation
  | undefined {
  return useContext(ButtonGroupContext) ?? undefined;
}
