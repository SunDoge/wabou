import { createFps } from "@wabou/core/renderer";
import { mergeClasses } from "@wabou/core/style";
import type { JSX } from "solid-js";
import { match, P } from "ts-pattern";
import { Badge, type BadgeProps } from "./badge";

export * from "./activity-status";
export * from "./alert";
export * from "./alert-dialog";
export * from "./aspect-ratio";
export * from "./attachment";
export * from "./avatar";
export * from "./badge";
export * from "./button";
export * from "./button-group";
export * from "./card";
export * from "./carousel";
export * from "./chart";
export * from "./code-block";
export * from "./combobox";
export * from "./command";
export * from "./content-state";
export * from "./context-menu";
export * from "./copy-button";
export * from "./data-table";
export * from "./date-picker";
export * from "./dev-server-error";
export * from "./dialog";
export * from "./diff-viewer";
export * from "./direction";
export * from "./directory-picker";
export * from "./disclosure";
export {
  Kbd,
  KbdGroup,
  Skeleton,
  type SkeletonProps,
  Spinner,
} from "./display";
export * from "./drawer";
export * from "./drop-zone";
export * from "./dropdown-menu";
export * from "./empty";
export * from "./forms";
export * from "./group-box";
export * from "./hover-card";
export * from "./icon-frame";
export * from "./image-list";
export * from "./image-viewport";
export * from "./inline-edit";
export * from "./input";
export * from "./input-otp";
export * from "./item";
export * from "./label";
export * from "./layout";
export * from "./listbox";
export * from "./markdown";
export * from "./menubar";
export * from "./message";
export * from "./message-scroller";
export * from "./navigation";
export * from "./navigation-menu";
export * from "./number-field";
export * from "./onboarding";
export * from "./page";
export * from "./popover";
export * from "./progress";
export * from "./prompt-composer";
export * from "./prompt-suggestion";
export * from "./property-list";
export * from "./qr-code";
export * from "./rating";
export * from "./reasoning";
export * from "./resizable";
export * from "./search-field";
export * from "./select";
export {
  Checkbox,
  type CheckboxProps,
  RadioGroup,
  RadioGroupItem,
  type RadioGroupItemProps,
  type RadioGroupProps,
  Toggle,
  ToggleGroup,
  ToggleGroupItem,
  type ToggleGroupItemProps,
  type ToggleGroupProps,
  type ToggleProps,
} from "./selection";
export * from "./separator";
export * from "./settings";
export * from "./sheet";
export * from "./shortcut-recorder";
export * from "./sidebar";
export * from "./slider";
export * from "./split-button";
export * from "./stat-card";
export * from "./status-bar";
export * from "./stepper";
export { Switch, type SwitchProps } from "./switch";
export * from "./table";
export {
  Tabs,
  TabsContent,
  TabsItem,
  type TabsItemProps,
  type TabsItemState,
  TabsList,
  type TabsProps,
  TabsTrigger,
  type TabsTriggerProps,
} from "./tabs";
export {
  type ComponentsControlSize,
  type ComponentsElevation,
  ComponentsProvider,
  type ComponentsProviderProps,
  type ComponentsTheme,
  componentsControlSize,
  componentsElevation,
  componentsThemeContract,
  useComponentsTheme,
} from "./theme";
export * from "./timeline";
export * from "./title-bar";
export * from "./toast";
export * from "./tool";
export * from "./toolbar";
export * from "./tooltip";
export * from "./tree-view";
export * from "./typography";
export * from "./workbench";

interface FpsBaseProps {
  /** Text displayed after the value. Set to an empty string for value only. */
  label?: string;
  /** FPS at or above this value uses the success treatment. */
  goodAt?: number;
  /** FPS below this value uses the destructive treatment. */
  warningBelow?: number;
  class?: string;
}

export type FpsProps = FpsBaseProps &
  (
    | {
        /** Explicitly drive the native animation clock to measure live FPS. */
        live: true;
        value?: never;
      }
    | {
        /** Render an externally measured FPS value without scheduling frames. */
        value: number;
        live?: false;
      }
  );

/** Frame-rate indicator. Live measurement is intentionally opt-in because it
 * keeps the platform frame clock active. */
export function Fps(props: FpsProps): JSX.Element {
  const measured = props.live ? createFps() : () => props.value;
  const value = () => Math.max(0, Math.round(measured()));
  const variant = (): BadgeProps["variant"] =>
    match(value())
      .with(0, () => "outline" as const)
      .with(
        P.when((fps) => fps >= (props.goodAt ?? 55)),
        () => "success" as const,
      )
      .with(
        P.when((fps) => fps < (props.warningBelow ?? 30)),
        () => "destructive" as const,
      )
      .otherwise(() => "secondary");
  return (
    <Badge
      variant={variant()}
      weight="normal"
      class={mergeClasses("font-mono", props.class)}
    >
      {value()}
      {props.label === "" ? "" : ` ${props.label ?? "fps"}`}
    </Badge>
  );
}
