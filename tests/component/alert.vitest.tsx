import { renderComponent } from "@wabou/test/component";
import {
  Alert,
  AlertActions,
  AlertDescription,
  AlertTitle,
  Button,
} from "@wabou/ui";
import { expect, test, vi } from "vitest";

test("alert actions provide a stable recovery region", () => {
  const retry = vi.fn();
  const screen = renderComponent(() => (
    <Alert variant="destructive" aria-label="Workspace load failed">
      <AlertTitle>Workspace load failed</AlertTitle>
      <AlertDescription>The local index could not be opened.</AlertDescription>
      <AlertActions aria-label="Recovery actions">
        <Button size="sm" onClick={retry}>
          Retry
        </Button>
      </AlertActions>
    </Alert>
  ));

  const actions = screen.getByRole("group", { name: "Recovery actions" });
  expect(actions.className).toContain("flex-wrap");
  expect(actions.className).toContain("gap-2");
  screen.getByRole("button", { name: "Retry" }).click();
  expect(retry).toHaveBeenCalledOnce();
});

test("alert uses the compact inline feedback surface contract", () => {
  const screen = renderComponent(() => (
    <Alert title="Workspace ready">All checks passed.</Alert>
  ));

  const alert = screen.getByRole("alert", { name: "Workspace ready" });
  expect(alert.className).toContain("rounded-lg");
  expect(alert.className).toContain("px-4");
  expect(alert.className).toContain("py-2.5");
  expect(alert.className).not.toContain("rounded-xl");
  expect(alert.className).not.toContain("shadow-xs");
});
