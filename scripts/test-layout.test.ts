import { describe, expect, test } from "bun:test";
import { parseLayoutTestArgs } from "./test-layout";

describe("layout test selection", () => {
  test("runs both applications when no fixture is selected", () => {
    expect(parseLayoutTestArgs([])).toEqual({
      apps: ["gallery", "pi-agent"],
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
    expect(parseLayoutTestArgs(["shell/sidebar"])).toEqual({
      apps: ["pi-agent"],
      filters: ["shell/sidebar"],
      skipBuild: false,
    });
    expect(
      parseLayoutTestArgs(["component/Button", "conversation/complete-turn"]),
    ).toEqual({
      apps: ["gallery", "pi-agent"],
      filters: ["component/Button", "conversation/complete-turn"],
      skipBuild: false,
    });
  });

  test("targets one application without consuming fixture filters", () => {
    expect(parseLayoutTestArgs(["--app", "pi-agent", "shell/sidebar"])).toEqual(
      {
        apps: ["pi-agent"],
        filters: ["shell/sidebar"],
        skipBuild: false,
      },
    );
    expect(parseLayoutTestArgs(["--app=gallery", "component/Button"])).toEqual({
      apps: ["gallery"],
      filters: ["component/Button"],
      skipBuild: false,
    });
  });

  test("only reuses a fixture bundle when explicitly requested", () => {
    expect(
      parseLayoutTestArgs(["--skip-build", "conversation/complete-turn"]),
    ).toEqual({
      apps: ["pi-agent"],
      filters: ["conversation/complete-turn"],
      skipBuild: true,
    });
  });

  test("rejects misspelled applications and options", () => {
    expect(() => parseLayoutTestArgs(["--app", "agent"])).toThrow(
      "expected gallery or pi-agent",
    );
    expect(() => parseLayoutTestArgs(["--quick"])).toThrow(
      "unknown layout test option",
    );
  });
});
