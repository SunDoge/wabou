import { LayoutSnapshot } from "./layout.mjs";
//#region src/layout-node.d.ts
interface RenderAppLayoutOptions {
  readonly app: string;
  readonly out: string;
  readonly width?: number;
  readonly height?: number;
  readonly scaleFactor?: number;
  readonly mode?: string;
  readonly skipBuild?: boolean;
  readonly waitMs?: number;
  /** Executable and any fixed prefix arguments. Defaults to `["wabou"]`. */
  readonly command?: readonly string[];
}
declare function layoutCommandArgs(options: RenderAppLayoutOptions): readonly string[];
declare function renderAppLayout(options: RenderAppLayoutOptions): Promise<LayoutSnapshot>;
//#endregion
export { RenderAppLayoutOptions, layoutCommandArgs, renderAppLayout };
//# sourceMappingURL=layout-node.d.mts.map