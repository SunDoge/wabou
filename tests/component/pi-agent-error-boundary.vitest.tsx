import { renderComponent } from "@wabou/test/component";
import { expect, test, vi } from "vitest";
import { AppErrorBoundary } from "../../apps/pi-agent/ui/app-error-boundary";

test("Pi Agent renders and logs a recoverable root failure", () => {
  const log = vi.spyOn(console, "error").mockImplementation(() => {});
  const Broken = () => {
    throw new Error("missing generated message");
  };
  const screen = renderComponent(() => (
    <AppErrorBoundary>
      <Broken />
    </AppErrorBoundary>
  ));

  expect(
    screen.getByRole("alert", { name: "Pi Agent failed to render" }).text,
  ).toContain("missing generated message");
  expect(log).toHaveBeenCalledWith(
    "[pi-agent] application render failed",
    expect.stringContaining("missing generated message"),
  );
  expect(screen.getByRole("button", { name: "Retry" })).toBeDefined();
});
