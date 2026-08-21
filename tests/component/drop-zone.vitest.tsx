import { dispatchFileDropEvent } from "@wabou/core/testing";
import { createTestHost, renderComponent } from "@wabou/test/component";
import { DropZone, pointInLayoutRect } from "@wabou/ui";
import { describe, expect, test, vi } from "vitest";

describe("DropZone", () => {
  test("uses inclusive logical-coordinate hit testing", () => {
    const rect = { x: 10, y: 20, width: 200, height: 100 };
    expect(pointInLayoutRect({ x: 10, y: 20 }, rect)).toBe(true);
    expect(pointInLayoutRect({ x: 210, y: 120 }, rect)).toBe(true);
    expect(pointInLayoutRect({ x: 211, y: 120 }, rect)).toBe(false);
  });

  test("claims only drops inside its measured native bounds", () => {
    const onDrop = vi.fn();
    const onRejected = vi.fn();
    const fixture = createTestHost(undefined, {
      layout: {
        measure: () => ({ x: 10, y: 20, width: 200, height: 100 }),
      },
    });
    const screen = renderComponent(
      () => (
        <DropZone
          label="Import files"
          accept={(path) => path.endsWith(".json")}
          onDrop={onDrop}
          onRejected={onRejected}
        />
      ),
      { host: fixture.host },
    );
    const zone = () => screen.getByRole("group", { name: "Import files" });

    dispatchFileDropEvent({
      phase: "moved",
      paths: [],
      position: { x: 50, y: 50 },
    });
    screen.flush();
    expect(zone().className).toContain("border-accent");

    dispatchFileDropEvent({
      phase: "dropped",
      paths: ["/tmp/project.json", "/tmp/readme.txt"],
      position: { x: 50, y: 50 },
    });
    screen.flush();
    expect(onDrop).toHaveBeenCalledWith(["/tmp/project.json"]);
    expect(onRejected).toHaveBeenCalledWith(["/tmp/readme.txt"]);
    expect(zone().className).toContain("border-strong");

    dispatchFileDropEvent({
      phase: "dropped",
      paths: ["/tmp/outside.json"],
      position: { x: 300, y: 50 },
    });
    dispatchFileDropEvent({
      phase: "dropped",
      paths: ["/tmp/unknown.json"],
      position: null,
    });
    expect(onDrop).toHaveBeenCalledTimes(1);
    expect(fixture.callsTo("layout.measure").length).toBeGreaterThan(0);
  });
});
