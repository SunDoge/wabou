import { CommandStateItem, filterCommandItems, reconcileCommandHighlight } from "./command-state.mjs";
import { Accessor, JSX } from "solid-js";
import { Handle } from "@wabou/core/renderer";
//#region src/components/command.d.ts
export interface CommandItem extends CommandStateItem {
  description?: string;
  /** Human-readable platform shortcut, such as `Ctrl K` or `⌘ K`. */
  shortcut?: string;
  onSelect?: () => void;
}
export interface CommandProps {
  items: readonly CommandItem[];
  "aria-label": string;
  query?: string;
  defaultQuery?: string;
  placeholder?: string;
  emptyText?: string;
  loading?: boolean;
  loadingText?: string;
  error?: unknown;
  errorText?: string;
  retryLabel?: string;
  class?: string;
  listClass?: string;
  onQueryChange?: (query: string) => void;
  onAction?: (id: string) => void;
  onRetry?: () => void;
  onDismiss?: () => void;
  inputRef?: (node: Handle) => void;
}
export interface CommandListProps {
  items: readonly CommandItem[];
  "aria-label": string;
  highlighted?: string;
  emptyText?: string;
  class?: string;
  itemClass?: string;
  loading?: boolean;
  loadingText?: string;
  error?: unknown;
  errorText?: string;
  retryLabel?: string;
  onRetry?: () => void;
  onHighlightChange?: (id: string) => void;
  onAction?: (id: string) => void;
  renderLeading?: (item: CommandItem) => JSX.Element;
}
/** Reusable command-result surface for search fields and inline completions. */
export declare function CommandList(props: CommandListProps): JSX.Element;
export interface CommandListNavigationOptions {
  onAction?: (id: string) => void;
  onDismiss?: () => void;
}
/**
 * Shared command-list navigation for any focus owner, including native editors.
 * The caller forwards key events while the list remains a passive popup.
 */
export declare function createCommandListNavigation(items: Accessor<readonly CommandStateItem[]>, options?: CommandListNavigationOptions): {
  highlighted: import("solid-js").SourceAccessor<string | undefined>;
  setHighlighted: import("solid-js").Setter<string | undefined>;
  select: (id: string | undefined) => boolean;
  move: (direction: "first" | "last" | "next" | "previous") => boolean;
  handleKeyDown: (event: {
    key: string;
    preventDefault(): void;
  }) => boolean;
};
/** Searchable command list whose filtering and keyboard behavior are host-independent. */
export declare function Command(props: CommandProps): JSX.Element;
//#endregion
export { filterCommandItems, reconcileCommandHighlight };
//# sourceMappingURL=command.d.mts.map