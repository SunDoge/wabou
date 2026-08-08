import { describe, expect, test } from "bun:test";
import { matchPath, parsePath, resolvePath, routeScore } from "./match";

describe("route matching", () => {
  test("matches static, decoded parameter, optional, and wildcard segments", () => {
    expect(matchPath("/story/:id", "/story/42")?.params).toEqual({ id: "42" });
    expect(matchPath("/user/:name?", "/user")?.params).toEqual({});
    expect(matchPath("/files/*path", "/files/a%20b/c")?.params).toEqual({
      path: "a b/c",
    });
    expect(matchPath("/story/:id", "/story/42/edit")).toBeNull();
  });

  test("ranks static routes above parameters and wildcards", () => {
    expect(routeScore("/story/new")).toBeGreaterThan(routeScore("/story/:id"));
    expect(routeScore("/story/:id")).toBeGreaterThan(
      routeScore("/story/*rest"),
    );
  });

  test("parses and resolves paths without browser URL globals", () => {
    expect(parsePath("/story/1?q=x#comments")).toEqual({
      pathname: "/story/1",
      search: "?q=x",
      hash: "#comments",
    });
    expect(resolvePath("../2", "/story/1")).toBe("/story/2");
    expect(resolvePath("?page=2", "/story/1#top")).toBe("/story/1?page=2");
  });
});
