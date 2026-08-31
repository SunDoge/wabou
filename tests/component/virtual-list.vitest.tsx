import { renderComponent } from "@wabou/test/component";
import { Text } from "@wabou/ui";
import { expect, test } from "vitest";
import { VirtualList } from "../../packages/core/src/renderer/virtual-list";

test("owns the native scrolling and clipping viewport contract", () => {
  const screen = renderComponent(() => (
    <VirtualList
      items={() => ["one", "two"]}
      itemHeight={40}
      getItemKey={(item) => item}
      role="listbox"
      accessibilityLabel="Files"
      class="h-full"
    >
      {(item) => <Text>{item()}</Text>}
    </VirtualList>
  ));

  const list = screen.getByRole("listbox", { name: "Files" });
  expect(list.className).toContain("overflow-x-hidden");
  expect(list.className).toContain("overflow-y-auto");
  expect(list.className).toContain("h-full");
});
