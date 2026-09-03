import { describe, expect, test } from "bun:test";
import { flush } from "solid-js";
import { createOwnedOverlay } from "./owned-overlay";

describe("owned overlay", () => {
  test("keeps the owner identity with the pending action", () => {
    const overlay = createOwnedOverlay<{ entryId: string }>();
    overlay.open("agent-1", { entryId: "entry-a" });
    flush();
    expect(overlay.value()).toEqual({
      ownerId: "agent-1",
      data: { entryId: "entry-a" },
    });
  });

  test("dismisses an overlay when navigation changes its owner", () => {
    const overlay = createOwnedOverlay<{ entryId: string }>();
    overlay.open("agent-1", { entryId: "entry-a" });
    flush();
    overlay.retainOwner("agent-2");
    flush();
    expect(overlay.value()).toBeUndefined();
  });

  test("does not dismiss an overlay for its current owner", () => {
    const overlay = createOwnedOverlay<{ entryId: string }>();
    overlay.open("agent-1", { entryId: "entry-a" });
    flush();
    overlay.retainOwner("agent-1");
    flush();
    expect(overlay.value()?.data.entryId).toBe("entry-a");
    overlay.close();
    flush();
    expect(overlay.value()).toBeUndefined();
  });
});
