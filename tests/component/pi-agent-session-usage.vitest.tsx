import { renderComponent } from "@wabou/test/component";
import { expect, test } from "vitest";
import {
  formatTokenCount,
  SessionUsage,
} from "../../apps/pi-agent/ui/session-usage";

test("formats compact token counts without requiring Intl", () => {
  expect(formatTokenCount(999)).toBe("999");
  expect(formatTokenCount(1_250)).toBe("1.3k");
  expect(formatTokenCount(24_000)).toBe("24k");
  expect(formatTokenCount(2_500_000)).toBe("2.5m");
});

test("summarizes authoritative context, token and cost usage", () => {
  const screen = renderComponent(() => (
    <SessionUsage
      stats={{
        userMessages: 2,
        assistantMessages: 2,
        toolCalls: 3,
        totalMessages: 7,
        tokens: {
          input: 10_000,
          output: 2_000,
          cacheRead: 8_000,
          cacheWrite: 500,
          total: 20_500,
        },
        cost: 0.1234,
        contextUsage: {
          tokens: 24_000,
          contextWindow: 128_000,
          percent: 18.75,
        },
      }}
    />
  ));

  expect(screen.getByRole("button", { name: "Session usage" }).text).toContain(
    "Context 19%21k tokens$0.123",
  );
});

test("opens an auditable breakdown of session activity", () => {
  const screen = renderComponent(() => (
    <SessionUsage
      stats={{
        userMessages: 2,
        assistantMessages: 2,
        toolCalls: 3,
        totalMessages: 7,
        tokens: {
          input: 10_000,
          output: 2_000,
          cacheRead: 8_000,
          cacheWrite: 500,
          total: 20_500,
        },
        cost: 0.1234,
        contextUsage: {
          tokens: 24_000,
          contextWindow: 128_000,
          percent: 18.75,
        },
      }}
    />
  ));

  screen.getByRole("button", { name: "Session usage" }).click();
  const dialog = screen.getByRole("dialog", { name: "Session usage" });
  expect(dialog.text).toContain("24k / 128k · 19%");
  expect(dialog.text).toContain("Input tokens10k");
  expect(dialog.text).toContain("Output tokens2.0k");
  expect(dialog.text).toContain("Cache read8.0k");
  expect(dialog.text).toContain("Cache write500");
  expect(dialog.text).toContain("Messages7");
  expect(dialog.text).toContain("Tool calls3");
  expect(dialog.text).toContain("Estimated cost$0.1234");
  const context = screen.getByRole("progressbar", {
    name: "Context window usage",
  });
  expect(context.numericValue).toBe(18.75);
  expect(context.valueText).toBe("19 percent");
});
