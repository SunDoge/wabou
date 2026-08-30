import { describe, expect, test } from "bun:test";
import { parseLayoutTestArgs } from "./test-layout";

describe("layout test selection", () => {
  test("runs both applications when no fixture is selected", () => {
    expect(parseLayoutTestArgs([])).toEqual({
      apps: ["gallery", "pi-agent"],
      filters: [],
    });
  });

  test("routes component and application fixtures to their owning app", () => {
    expect(parseLayoutTestArgs(["component/Button"])).toEqual({
      apps: ["gallery"],
      filters: ["component/Button"],
    });
    expect(parseLayoutTestArgs(["shell/sidebar"])).toEqual({
      apps: ["pi-agent"],
      filters: ["shell/sidebar"],
    });
    expect(
      parseLayoutTestArgs(["component/Button", "conversation/complete-turn"]),
    ).toEqual({
      apps: ["gallery", "pi-agent"],
      filters: ["component/Button", "conversation/complete-turn"],
    });
  });

  test("targets one application without consuming fixture filters", () => {
    expect(parseLayoutTestArgs(["--app", "pi-agent", "shell/sidebar"])).toEqual(
      {
        apps: ["pi-agent"],
        filters: ["shell/sidebar"],
      },
    );
    expect(parseLayoutTestArgs(["--app=gallery", "component/Button"])).toEqual({
      apps: ["gallery"],
      filters: ["component/Button"],
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
