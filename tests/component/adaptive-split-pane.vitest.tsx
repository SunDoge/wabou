import { renderComponent } from "@wabou/test/component";
import {
  AdaptiveSplitPane,
  AdaptiveSplitPaneDetail,
  AdaptiveSplitPaneMain,
  Text,
} from "@wabou/ui";
import { expect, test, vi } from "vitest";

test("adaptive split pane derives compact mode from its native content width", () => {
  const onCompactChange = vi.fn();
  const screen = renderComponent(() => (
    <AdaptiveSplitPane
      aria-label="Workspace split"
      compactAt={640}
      onCompactChange={onCompactChange}
      class="h-full"
    >
      <AdaptiveSplitPaneMain>
        <Text>Main content</Text>
      </AdaptiveSplitPaneMain>
      <AdaptiveSplitPaneDetail
        open
        onOpenChange={() => {}}
        aria-label="Inspector"
      >
        <Text>Detail content</Text>
      </AdaptiveSplitPaneDetail>
    </AdaptiveSplitPane>
  ));
  const split = screen.getByRole("group", { name: "Workspace split" });

  split.resize({ width: 900, height: 600 });
  expect(onCompactChange).toHaveBeenLastCalledWith(false);
  expect(screen.queryByRole("dialog", { name: "Inspector" })).toBeNull();

  split.resize({ width: 520, height: 600 });
  expect(onCompactChange).toHaveBeenLastCalledWith(true);
  expect(screen.getByRole("dialog", { name: "Inspector" }).text).toContain(
    "Detail content",
  );
});
