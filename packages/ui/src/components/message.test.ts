import { describe, expect, test } from "bun:test";
import {
  bubbleClass,
  bubbleContentClass,
  messageActionsClass,
  messageClass,
} from "./message";

describe("Message anatomy", () => {
  test("reverses an outgoing message without descendant selectors", () => {
    expect(messageClass("start")).toContain("flex-row");
    expect(messageClass("end")).toContain("flex-row-reverse");
  });

  test("aligns bubbles explicitly and bounds readable width", () => {
    expect(bubbleClass("default", "start")).toContain("self-start");
    expect(bubbleClass("default", "end")).toContain("self-end");
    expect(bubbleClass("default", "end")).toContain("max-w-4/5");
    expect(bubbleClass("ghost", "end")).toContain("max-w-full");
  });

  test("maps every visual variant to supported semantic colors", () => {
    expect(bubbleContentClass("default")).toContain("bg-accent");
    expect(bubbleContentClass("tinted")).toContain("bg-selected");
    expect(bubbleContentClass("destructive")).toContain("border-danger");
    expect(bubbleContentClass("ghost")).toContain("bg-transparent");
  });

  test("aligns compact actions with their message direction", () => {
    expect(messageActionsClass("start")).toContain("self-start");
    expect(messageActionsClass("end")).toContain("self-end");
    expect(messageActionsClass("end")).toContain("justify-end");
  });
});
