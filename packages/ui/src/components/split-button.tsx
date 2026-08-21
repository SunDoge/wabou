import chevronDown from "lucide-static/icons/chevron-down.svg?raw";
import type { JSX } from "solid-js";
import { Icon } from "../primitives";
import { Button, type ButtonProps } from "./button";
import { ButtonGroup, ButtonGroupSeparator } from "./button-group";
import { DropdownMenu, type DropdownMenuItem } from "./dropdown-menu";

export interface SplitButtonProps
  extends Omit<ButtonProps, "children" | "aria-label"> {
  label: string;
  menuLabel?: string;
  items: readonly DropdownMenuItem[];
  onAction?: (id: string) => void;
  onClick?: ButtonProps["onClick"];
  class?: string;
}

/**
 * Desktop split action: the label always performs the primary command while
 * the adjacent arrow owns alternative commands and their keyboard behavior.
 */
export function SplitButton(props: SplitButtonProps): JSX.Element {
  const menuLabel = () => props.menuLabel ?? `${props.label} alternatives`;
  return (
    <ButtonGroup aria-label={props.label} class={props.class}>
      <Button
        variant={props.variant}
        size={props.size}
        disabled={props.disabled}
        onClick={props.onClick}
        class="flex-1"
      >
        {props.label}
      </Button>
      <ButtonGroupSeparator />
      <DropdownMenu
        aria-label={menuLabel()}
        items={props.items}
        onAction={props.onAction}
        trigger={(trigger) => (
          <Button
            {...trigger}
            aria-label={menuLabel()}
            variant={props.variant}
            size="icon"
            disabled={props.disabled}
          >
            <Icon source={chevronDown} size={14} />
          </Button>
        )}
      />
    </ButtonGroup>
  );
}
