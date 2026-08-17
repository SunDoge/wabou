import { Component, JSX } from "solid-js";
//#region src/history.d.ts
interface HistoryEntry {
  value: string;
  state?: unknown;
}
interface HistoryUpdate extends HistoryEntry {
  replace?: boolean;
}
type HistoryListener = (entry: HistoryEntry) => void;
interface MemoryHistory {
  readonly back: () => void;
  readonly forward: () => void;
  readonly go: (delta: number) => void;
  readonly get: () => HistoryEntry;
  readonly listen: (listener: HistoryListener) => () => void;
  readonly set: (update: HistoryUpdate) => void;
  readonly canGoBack: () => boolean;
  readonly canGoForward: () => boolean;
}
interface MemoryHistoryOptions {
  initialEntries?: readonly (string | HistoryEntry)[];
  initialIndex?: number;
}
/** A deterministic, window-independent navigation stack. */
declare function createMemoryHistory(options?: MemoryHistoryOptions): MemoryHistory;
//#endregion
//#region src/match.d.ts
type RouteParams = Record<string, string>;
interface PathParts {
  pathname: string;
  search: string;
  hash: string;
}
declare function normalizePathname(pathname: string): string;
declare function parsePath(value: string): PathParts;
declare function resolvePath(to: string, from: string): string;
interface PathMatch {
  params: RouteParams;
  path: string;
}
/** Match a complete pathname. `:name`, `:name?`, and `*rest` are supported. */
declare function matchPath(pattern: string, pathname: string): PathMatch | null;
//#endregion
//#region src/index.d.ts
interface NavigateOptions {
  replace?: boolean;
  resolve?: boolean;
  state?: unknown;
}
type Navigate = (destination: string | number, options?: NavigateOptions) => void;
interface Location {
  readonly pathname: string;
  readonly search: string;
  readonly hash: string;
  readonly state: unknown;
}
interface RouteProps {
  path?: string | readonly string[];
  component?: Component<{
    children?: JSX.Element;
  }>;
  children?: JSX.Element;
}
declare function Route(props: RouteProps): JSX.Element;
interface MemoryRouterProps extends MemoryHistoryOptions {
  history?: MemoryHistory;
  root?: Component<{
    children?: JSX.Element;
  }>;
  children: JSX.Element;
}
declare function MemoryRouter(props: MemoryRouterProps): JSX.Element;
declare function useNavigate(): Navigate;
declare function useLocation(): Location;
declare function useParams<T extends RouteParams = RouteParams>(): T;
declare function useHistory(): MemoryHistory;
//#endregion
export { type HistoryEntry, type HistoryListener, type HistoryUpdate, Location, type MemoryHistory, type MemoryHistoryOptions, MemoryRouter, MemoryRouterProps, Navigate, NavigateOptions, type PathMatch, type PathParts, Route, type RouteParams, RouteProps, createMemoryHistory, matchPath, normalizePathname, parsePath, resolvePath, useHistory, useLocation, useNavigate, useParams };
//# sourceMappingURL=index.d.mts.map