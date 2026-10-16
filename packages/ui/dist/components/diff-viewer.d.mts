import { ViewProps } from "../primitives/view.mjs";
import "../primitives.mjs";
//#region src/components/diff-viewer.d.ts
export type DiffFileStatus = "added" | "modified" | "deleted" | "renamed";
export interface DiffFile {
  path: string;
  oldPath?: string;
  status: DiffFileStatus;
  additions: number;
  deletions: number;
  /** Unified patch for this file. Kept out of the visible tree while collapsed. */
  patch: string;
}
export interface DiffViewerLabels {
  filesChanged: (count: number) => string;
  additions: (count: number) => string;
  deletions: (count: number) => string;
  empty: string;
  technicalDetails: string;
}
export interface DiffViewerProps extends Omit<ViewProps, "children"> {
  files: readonly DiffFile[];
  labels?: Partial<DiffViewerLabels>;
  /** Files opened initially. Details remain opt-in by default. */
  defaultExpanded?: readonly string[];
}
/**
 * A progressive-disclosure code change viewer.
 *
 * The summary and file metadata are ordinary Wabou components. Unified patch
 * text is mounted only after disclosure and uses Wabou's native editor viewport
 * for selection, copying, and large documents.
 */
export declare function DiffViewer(props: DiffViewerProps): import("@wabou/core/jsx-runtime").JSX.Element;
//#endregion
//# sourceMappingURL=diff-viewer.d.mts.map