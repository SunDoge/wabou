export type RouteParams = Record<string, string>;

export interface PathParts {
  pathname: string;
  search: string;
  hash: string;
}

function safeDecode(value: string): string {
  try {
    return decodeURIComponent(value);
  } catch {
    return value;
  }
}

export function normalizePathname(pathname: string): string {
  const rooted = pathname.startsWith("/") ? pathname : `/${pathname}`;
  const collapsed = rooted.replace(/\/{2,}/g, "/");
  return collapsed.length > 1 ? collapsed.replace(/\/+$/, "") : collapsed;
}

export function parsePath(value: string): PathParts {
  const hashAt = value.indexOf("#");
  const hash = hashAt >= 0 ? value.slice(hashAt) : "";
  const withoutHash = hashAt >= 0 ? value.slice(0, hashAt) : value;
  const searchAt = withoutHash.indexOf("?");
  const search = searchAt >= 0 ? withoutHash.slice(searchAt) : "";
  const pathname = searchAt >= 0 ? withoutHash.slice(0, searchAt) : withoutHash;
  return { pathname: normalizePathname(pathname || "/"), search, hash };
}

export function resolvePath(to: string, from: string): string {
  if (to.startsWith("/")) return to;
  if (to.startsWith("?") || to.startsWith("#")) {
    const current = parsePath(from);
    if (to.startsWith("?")) return `${current.pathname}${to}`;
    return `${current.pathname}${current.search}${to}`;
  }

  const fromParts = parsePath(from);
  const segments = fromParts.pathname.split("/").filter(Boolean);
  for (const segment of to.split("/")) {
    if (!segment || segment === ".") continue;
    if (segment === "..") segments.pop();
    else segments.push(segment);
  }
  return `/${segments.join("/")}`;
}

export interface PathMatch {
  params: RouteParams;
  path: string;
}

/** Match a complete pathname. `:name`, `:name?`, and `*rest` are supported. */
export function matchPath(pattern: string, pathname: string): PathMatch | null {
  const patternSegments = normalizePathname(pattern).split("/").filter(Boolean);
  const pathSegments = normalizePathname(pathname).split("/").filter(Boolean);
  const params: RouteParams = {};
  let pathIndex = 0;

  for (const patternSegment of patternSegments) {
    if (patternSegment.startsWith("*")) {
      const name = patternSegment.slice(1) || "rest";
      params[name] = safeDecode(pathSegments.slice(pathIndex).join("/"));
      pathIndex = pathSegments.length;
      break;
    }

    const optional =
      patternSegment.startsWith(":") && patternSegment.endsWith("?");
    if (patternSegment.startsWith(":")) {
      const name = patternSegment.slice(1, optional ? -1 : undefined);
      const value = pathSegments[pathIndex];
      if (value === undefined) {
        if (optional) continue;
        return null;
      }
      params[name] = safeDecode(value);
      pathIndex += 1;
      continue;
    }

    if (pathSegments[pathIndex] !== patternSegment) return null;
    pathIndex += 1;
  }

  if (pathIndex !== pathSegments.length) return null;
  return { params, path: normalizePathname(pathname) };
}

export function routeScore(pattern: string): number {
  return normalizePathname(pattern)
    .split("/")
    .filter(Boolean)
    .reduce((score, segment) => {
      if (segment.startsWith("*")) return score + 1;
      if (segment.startsWith(":"))
        return score + (segment.endsWith("?") ? 2 : 3);
      return score + 4;
    }, 0);
}

export function joinPaths(parent: string, child: string): string {
  if (child.startsWith("/")) return normalizePathname(child);
  if (!child) return normalizePathname(parent || "/");
  return normalizePathname(`${parent}/${child}`);
}
