import { renderComponent } from "@wabou/test/component";
import { Button } from "@wabou/ui";
import { createSignal } from "solid-js";
import { expect, test, vi } from "vitest";

test("Button uses the shared native desktop geometry across sizes", () => {
  const screen = renderComponent(() => (
    <>
      <Button aria-label="Small action" size="sm" />
      <Button aria-label="Default action" />
      <Button aria-label="Large action" size="lg" />
    </>
  ));

  expect(
    screen.getByRole("button", { name: "Small action" }).className,
  ).toContain("h-7 px-2 gap-1 text-xs rounded-md");
  expect(
    screen.getByRole("button", { name: "Default action" }).className,
  ).toContain("h-8 px-2.5 gap-2 text-sm rounded-md");
  expect(
    screen.getByRole("button", { name: "Large action" }).className,
  ).toContain("h-10 px-3 gap-2 text-base rounded-md");
});

test("Button clips every visual state to its control radius", () => {
  const screen = renderComponent(() => (
    <Button aria-label="Delete" variant="destructive">
      Delete
    </Button>
  ));

  const button = screen.getByRole("button", { name: "Delete" });
  expect(button.className).toContain("rounded-md");
  expect(button.className).toContain("overflow-hidden");
  expect(button.className).toContain("bg-danger");
});

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
    busy: true,
  });

  expect(start.busy).toBe(true);
  expect(start.text).toContain("Starting…");
  expect(start.text).not.toContain("Start agent");
  expect(screen.getByRole("status", { name: "Starting…" })).toBeDefined();
  expect(() => start.click()).toThrow("cannot click disabled component");
  expect(activate).not.toHaveBeenCalled();

  screen.getByRole("button", { name: "Finish loading" }).click();
  await screen.waitFor(() =>
    expect(
      screen.getByRole("button", { name: "Start agent", busy: false }).disabled,
    ).toBe(false),
  );
  expect(start.busy).toBe(false);
  expect(screen.getByRole("button", { name: "Start agent" }).text).toContain(
    "Start agent",
  );
});
