import { JSX } from "solid-js";
//#region src/components/resizable.d.ts
export type ResizableDirection = "horizontal" | "vertical";
export interface ResizablePanelDefinition {
  id: string;
  defaultSize: number;
  minSize?: number;
  maxSize?: number;
}
export type ResizablePanelSizes = Readonly<Record<string, number>>;
export declare function validateResizableSizes(panels: readonly ResizablePanelDefinition[], sizes?: ResizablePanelSizes): ResizablePanelSizes;
export interface ResizablePanelState {
  readonly panels: readonly ResizablePanelDefinition[];
  sizes(): ResizablePanelSizes;
  size(id: string): number;
  pairRange(before: string, after: string): {
    min: number;
    max: number;
  };
  resizePair(before: string, after: string, beforeSize: number): boolean;
  resetPair(before: string, after: string): boolean;
}
export declare function createResizablePanelState(options: {
  panels: readonly ResizablePanelDefinition[];
  value?: () => ResizablePanelSizes | undefined;
  defaultValue?: ResizablePanelSizes;
  onValueChange?: (sizes: ResizablePanelSizes) => void;
}): ResizablePanelState;
export interface ResizablePanelGroupProps {
  panels: readonly ResizablePanelDefinition[];
  children?: JSX.Element;
  direction?: ResizableDirection;
  value?: ResizablePanelSizes;
  defaultValue?: ResizablePanelSizes;
  onValueChange?: (sizes: ResizablePanelSizes) => void;
  "aria-label": string;
  class?: string;
}
export declare function ResizablePanelGroup(props: ResizablePanelGroupProps): JSX.Element;
export declare function ResizablePanel(props: {
  id: string;
  children?: JSX.Element;
  class?: string;
}): JSX.Element;
export interface ResizableHandleProps {
  before: string;
  after: string;
  "aria-label": string;
  keyboardStep?: number;
  class?: string;
}
export declare function ResizableHandle(props: ResizableHandleProps): JSX.Element;
//#endregion
//# sourceMappingURL=resizable.d.mts.map