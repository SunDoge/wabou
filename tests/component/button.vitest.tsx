import { renderComponent } from "@wabou/test/component";
import { Button } from "@wabou/ui";
import { createSignal } from "solid-js";
import { expect, test, vi } from "vitest";

test("Button exposes one stable disabled loading state", async () => {
  const activate = vi.fn();
  const App = () => {
    const [loading, setLoading] = createSignal(true);
    return (
      <>
        <Button
          aria-label="Start agent"
          loading={loading()}
          loadingLabel="Starting…"
          onClick={activate}
        >
          Start agent
        </Button>
        <Button aria-label="Finish loading" onClick={() => setLoading(false)}>
          Finish
        </Button>
      </>
    );
  };
  const screen = renderComponent(App);
  const start = screen.getByRole("button", {
    name: "Start agent",
    disabled: true,
  });

  expect(start.text).toContain("Starting…");
  expect(start.text).not.toContain("Start agent");
  expect(screen.getByRole("status", { name: "Starting…" })).toBeDefined();
  expect(() => start.click()).toThrow("cannot click disabled component");
  expect(activate).not.toHaveBeenCalled();

  screen.getByRole("button", { name: "Finish loading" }).click();
  await screen.waitFor(() =>
    expect(screen.getByRole("button", { name: "Start agent" }).disabled).toBe(
      false,
    ),
  );
  expect(screen.getByRole("button", { name: "Start agent" }).text).toContain(
    "Start agent",
  );
});
