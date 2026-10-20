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

test("provides semantic variants, sizing, banner chrome, and close delegation", () => {
  const close = vi.fn();
  const screen = renderComponent(() => (
    <>
      <Alert variant="success" size="lg" title="Published" onClose={close}>
        The release is available.
      </Alert>
      <Alert variant="warning" banner title="Network warning">
        Proxy settings need attention.
      </Alert>
      <Alert variant="info" title="Indexed" />
    </>
  ));
  const success = screen.getByRole("alert", { name: "Published" });
  const warning = screen.getByRole("alert", { name: "Network warning" });
  const informational = screen.getByRole("alert", { name: "Indexed" });

  expect(success.className).toContain("px-5 py-3");
  expect(success.className).toContain("bg-success-surface");
  expect(success.children[0]?.children[0]?.tag).toBe("svg");
  expect(warning.className).toContain("rounded-none");
  expect(warning.className).toContain("border-0");
  expect(informational.className).toContain("bg-selected");
  screen.getByRole("button", { name: "Dismiss Published" }).click();
  expect(close).toHaveBeenCalledOnce();
});

test("default and destructive alerts apply their semantic theme", () => {
  const screen = renderComponent(() => (
    <>
      <Alert title="Workspace ready">All checks passed.</Alert>
      <Alert variant="destructive" title="Workspace failed">
        Retry later.
      </Alert>
    </>
  ));
  const ready = screen.getByRole("alert", { name: "Workspace ready" });
  const failed = screen.getByRole("alert", { name: "Workspace failed" });
  expect(ready.text).toContain("All checks passed.");
  expect(ready.className).toContain("bg-surface");
  expect(failed.className).toContain("border-danger");
  expect(failed.className).toContain("bg-danger-surface");
});
