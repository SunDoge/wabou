import { renderComponent } from "@wabou/test/component";
import { expect, test } from "vitest";
import {
  type OperationProgressEvent,
  OperationProgressStatus,
  decodeOperationProgressEvent,
} from "../../apps/timestow/ui/operation-progress";

test("backup progress exposes determinate byte counts", () => {
  const progress: OperationProgressEvent = {
    profileId: "photos",
    operation: "backup",
    operationId: "backup:photos",
    state: "running",
    unit: "bytes",
    title: "Writing pack",
    current: 1_024,
    total: 4_096,
  };
  const screen = renderComponent(() => (
    <OperationProgressStatus progress={progress} />
  ));

  const bar = screen.getByRole("progressbar", { name: "Backup progress" });
  expect(bar.numericValue).toBe(1_024);
  expect(bar.maxNumericValue).toBe(4_096);
  expect(bar.valueText).toBe("25 percent");
  expect(screen.getByRole("status").text).toContain("1.0 KB of 4.0 KB");
});

test("backup progress keeps unknown-length rustic phases indeterminate", () => {
  const screen = renderComponent(() => (
    <OperationProgressStatus
      progress={{
        profileId: "photos",
        operation: "backup",
        operationId: "backup:photos",
        state: "running",
        unit: "spinner",
        title: "Reading repository",
        current: 0,
      }}
    />
  ));

  expect(
    screen.getByRole("progressbar", { name: "Backup progress" }).numericValue,
  ).toBeNull();
  expect(screen.getByRole("status").text).toContain("Working…");
});

test("backup progress messages reject malformed transport payloads", () => {
  expect(() =>
    decodeOperationProgressEvent({
      profileId: "photos",
      operation: "backup",
      operationId: "backup:photos",
      state: "running",
      unit: "bytes",
      title: "Writing pack",
      current: "1024",
    }),
  ).toThrow("operation progress has invalid fields");
});
