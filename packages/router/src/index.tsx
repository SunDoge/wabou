import {
  children,
  createComponent,
  createContext,
  createMemo,
  createSignal,
  onCleanup,
  Show,
  type Component,
  type JSX,
  useContext,
} from "solid-js";
import {
  createMemoryHistory,
  type HistoryEntry,
  type MemoryHistory,
  type MemoryHistoryOptions,
} from "./history";
import {
  joinPaths,
  matchPath,
  parsePath,
  resolvePath,
  type RouteParams,
  routeScore,
} from "./match";

export { createMemoryHistory } from "./history";
export type {
  HistoryEntry,
  HistoryListener,
  HistoryUpdate,
  MemoryHistory,
  MemoryHistoryOptions,
} from "./history";
export { matchPath, normalizePathname, parsePath, resolvePath } from "./match";
export type { PathMatch, PathParts, RouteParams } from "./match";

export interface NavigateOptions {
  replace?: boolean;
  resolve?: boolean;
  state?: unknown;
}

export type Navigate = (
  destination: string | number,
  options?: NavigateOptions,
) => void;

export interface Location {
  readonly pathname: string;
  readonly search: string;
  readonly hash: string;
  readonly state: unknown;
}

export interface RouteProps {
  path?: string | readonly string[];
  component?: Component<{ children?: JSX.Element }>;
  children?: JSX.Element;
}

interface RouteDefinition extends Omit<RouteProps, "children"> {
  readonly __wabouRoute: true;
  readonly children: readonly RouteDefinition[];
}

interface RouteMatch {
  definitions: readonly RouteDefinition[];
  params: RouteParams;
  score: number;
}

interface RouterContextValue {
  history: MemoryHistory;
  location: Location;
  navigate: Navigate;
  params: RouteParams;
}

const RouterContext = createContext<RouterContextValue>();

function requireRouter(): RouterContextValue {
  const router = useContext(RouterContext);
  if (!router)
    throw new Error("Wabou router hooks must be used inside <MemoryRouter>");
  return router;
}

function routeChildren(value: JSX.Element): readonly RouteDefinition[] {
  const values: unknown[] = [];
  const append = (candidate: unknown) => {
    if (Array.isArray(candidate)) {
      for (const child of candidate) append(child);
    } else {
      values.push(candidate);
    }
  };
  append(value);
  return values.filter(
    (candidate): candidate is RouteDefinition =>
      typeof candidate === "object" &&
      candidate !== null &&
      "__wabouRoute" in candidate,
  );
}

export function Route(props: RouteProps): JSX.Element {
  const resolved = children(() => props.children);
  return {
    __wabouRoute: true,
    path: props.path,
    component: props.component,
    get children() {
      return routeChildren(resolved());
    },
  } as unknown as JSX.Element;
}

function pathVariants(path: RouteProps["path"]): readonly string[] {
  return typeof path === "string" || path === undefined ? [path ?? ""] : path;
}

function collectMatches(
  definitions: readonly RouteDefinition[],
  pathname: string,
  parentPath = "",
  parents: readonly RouteDefinition[] = [],
): RouteMatch[] {
  const matches: RouteMatch[] = [];
  for (const definition of definitions) {
    for (const variant of pathVariants(definition.path)) {
      const pattern = joinPaths(parentPath, variant);
      const chain = [...parents, definition];
      if (definition.children.length) {
        matches.push(
          ...collectMatches(definition.children, pathname, pattern, chain),
        );
      }
      const matched = matchPath(pattern, pathname);
      if (matched) {
        matches.push({
          definitions: chain,
          params: matched.params,
          score: routeScore(pattern),
        });
      }
    }
  }
  return matches;
}

function renderDefinitions(
  definitions: readonly RouteDefinition[],
  index = 0,
): JSX.Element {
  const definition = definitions[index];
  if (!definition) return null;
  const outlet = () => renderDefinitions(definitions, index + 1);
  return definition.component
    ? createComponent(definition.component, {
        get children() {
          return outlet();
        },
      })
    : outlet();
}

function sameDefinitions(
  left: readonly RouteDefinition[] | undefined,
  right: readonly RouteDefinition[] | undefined,
): boolean {
  return (
    left === right ||
    (left !== undefined &&
      right !== undefined &&
      left.length === right.length &&
      left.every((definition, index) => definition === right[index]))
  );
}

export interface MemoryRouterProps extends MemoryHistoryOptions {
  history?: MemoryHistory;
  root?: Component<{ children?: JSX.Element }>;
  children: JSX.Element;
}

export function MemoryRouter(props: MemoryRouterProps): JSX.Element {
  const history =
    props.history ??
    createMemoryHistory({
      initialEntries: props.initialEntries,
      initialIndex: props.initialIndex,
    });
  const [entry, setEntry] = createSignal<HistoryEntry>(history.get());
  onCleanup(history.listen(setEntry));

  const resolvedRoutes = children(() => props.children);
  const definitions = createMemo(() => routeChildren(resolvedRoutes()));
  const path = createMemo(() => parsePath(entry().value));
  const match = createMemo(() => {
    const candidates = collectMatches(definitions(), path().pathname);
    return candidates.sort((a, b) => b.score - a.score)[0];
  });
  const branch = createMemo<readonly RouteDefinition[] | undefined>(
    (previous) => {
      const next = match()?.definitions;
      return sameDefinitions(previous, next) ? previous : next;
    },
  );
  const params = new Proxy({} as RouteParams, {
    get: (_target, property) =>
      typeof property === "string" ? match()?.params[property] : undefined,
    ownKeys: () => Reflect.ownKeys(match()?.params ?? {}),
    getOwnPropertyDescriptor: () => ({ enumerable: true, configurable: true }),
  });
  const location: Location = {
    get pathname() {
      return path().pathname;
    },
    get search() {
      return path().search;
    },
    get hash() {
      return path().hash;
    },
    get state() {
      return entry().state;
    },
  };
  const navigate: Navigate = (destination, options = {}) => {
    if (typeof destination === "number") {
      history.go(destination);
      return;
    }
    const value =
      options.resolve === false
        ? destination
        : resolvePath(destination, entry().value);
    history.set({ value, state: options.state, replace: options.replace });
  };
  const context: RouterContextValue = { history, location, navigate, params };
  const routed = () => {
    const outlet = createComponent(Show, {
      get when() {
        return branch();
      },
      keyed: true,
      children: (current: object) =>
        renderDefinitions(current as readonly RouteDefinition[]),
    });
    return props.root
      ? createComponent(props.root, {
          get children() {
            return outlet;
          },
        })
      : outlet;
  };

  return createComponent(RouterContext.Provider, {
    value: context,
    get children() {
      return routed();
    },
  });
}

export function useNavigate(): Navigate {
  return requireRouter().navigate;
}

export function useLocation(): Location {
  return requireRouter().location;
}

export function useParams<T extends RouteParams = RouteParams>(): T {
  return requireRouter().params as T;
}

export function useHistory(): MemoryHistory {
  return requireRouter().history;
}
