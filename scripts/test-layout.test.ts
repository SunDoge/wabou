import { describe, expect, test } from "bun:test";
import { parseLayoutTestArgs } from "./test-layout";

describe("layout test selection", () => {
  test("runs every layout-tested application when no fixture is selected", () => {
    expect(parseLayoutTestArgs([])).toEqual({
      apps: ["gallery", "launcher", "meow-starter", "reader"],
      filters: [],
      skipBuild: false,
    });
  });

  test("routes component and application fixtures to their owning app", () => {
    expect(parseLayoutTestArgs(["component/Button"])).toEqual({
      apps: ["gallery"],
      filters: ["component/Button"],
      skipBuild: false,
    });
    expect(parseLayoutTestArgs(["launcher/default"])).toEqual({
      apps: ["launcher"],
      filters: ["launcher/default"],
      skipBuild: false,
    });
    expect(parseLayoutTestArgs(["reader/default"])).toEqual({
      apps: ["reader"],
      filters: ["reader/default"],
      skipBuild: false,
    });
    expect(parseLayoutTestArgs(["component/Button", "form/inputs"])).toEqual({
      apps: ["gallery"],
      filters: ["component/Button", "form/inputs"],
      skipBuild: false,
    });
  });

  test("targets one application without consuming fixture filters", () => {
    expect(parseLayoutTestArgs(["--app", "reader", "reader/default"])).toEqual({
      apps: ["reader"],
      filters: ["reader/default"],
      skipBuild: false,
    });
    expect(parseLayoutTestArgs(["--app=gallery", "component/Button"])).toEqual({
      apps: ["gallery"],
      filters: ["component/Button"],
      skipBuild: false,
    });
  });

  test("only reuses a fixture bundle when explicitly requested", () => {
    expect(parseLayoutTestArgs(["--skip-build", "component/Button"])).toEqual({
      apps: ["gallery"],
      filters: ["component/Button"],
      skipBuild: true,
    });
  });

  test("rejects misspelled applications and options", () => {
    expect(() => parseLayoutTestArgs(["--app", "agent"])).toThrow(
      "expected gallery, launcher, meow-starter, or reader",
    );
    expect(() => parseLayoutTestArgs(["--quick"])).toThrow(
      "unknown layout test option",
    );
  });
});
