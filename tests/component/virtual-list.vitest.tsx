import { renderComponent } from "@wabou/test/component";
import { Text } from "@wabou/ui";
import { createSignal } from "solid-js";
import { expect, test } from "vitest";
import {
  VirtualList,
  type VirtualListController,
} from "../../packages/core/src/renderer/virtual-list";

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
  expect(list.projectionBoundary).toBe(true);
});

test("mounts only visible rows and moves the window with native scrolling", () => {
  const items = Array.from({ length: 10_000 }, (_, index) => `Row ${index}`);
  const screen = renderComponent(() => (
    <VirtualList
      items={() => items}
      itemHeight={20}
      viewportHeight={100}
      overscan={1}
      getItemKey={(item) => item}
      role="listbox"
      accessibilityLabel="Large list"
    >
      {(item) => <Text role="option">{item()}</Text>}
    </VirtualList>
  ));

  expect(screen.getAllByRole("option")).toHaveLength(6);
  expect(screen.getByRole("option", { name: "Row 0" })).toBeDefined();
  expect(screen.queryByRole("option", { name: "Row 7" })).toBeNull();

  screen
    .getByRole("listbox", { name: "Large list" })
    .emit("scroll", { scrollY: 200 });

  expect(screen.getAllByRole("option")).toHaveLength(7);
  expect(screen.getByRole("option", { name: "Row 9" })).toBeDefined();
  expect(screen.getByRole("option", { name: "Row 15" })).toBeDefined();
  expect(screen.queryByRole("option", { name: "Row 0" })).toBeNull();
});

test("observes a parent-bounded viewport when no fixed height is supplied", () => {
  const items = Array.from({ length: 100 }, (_, index) => `Item ${index}`);
  const screen = renderComponent(() => (
    <VirtualList
      items={() => items}
      itemHeight={25}
      getItemKey={(item) => item}
      role="listbox"
      accessibilityLabel="Measured list"
    >
      {(item) => <Text role="option">{item()}</Text>}
    </VirtualList>
  ));
  const list = screen.getByRole("listbox", { name: "Measured list" });

  expect(screen.getAllByRole("option")).toHaveLength(3);
  list.resize({ width: 300, height: 100 });
  expect(screen.getAllByRole("option")).toHaveLength(6);
});

test("keeps item and index coherent while a filtered list shrinks", () => {
  const [items, setItems] = createSignal(
    Array.from({ length: 20 }, (_, index) => `Entry ${index}`),
  );
  const screen = renderComponent(() => (
    <VirtualList
      items={items}
      itemHeight={20}
      viewportHeight={100}
      getItemKey={(item) => item}
      role="listbox"
      accessibilityLabel="Filtered list"
    >
      {(item) => <Text role="option">{item()}</Text>}
    </VirtualList>
  ));
  const list = screen.getByRole("listbox", { name: "Filtered list" });
  list.emit("scroll", { scrollY: 200 });

  setItems(["Filtered result"]);
  screen.flush();

  expect(screen.getAllByRole("option")).toHaveLength(1);
  expect(screen.getByRole("option", { name: "Filtered result" })).toBeDefined();
});

test("scrolls an offscreen item into the mounted window for keyboard owners", () => {
  const items = Array.from({ length: 100 }, (_, index) => `Item ${index}`);
  let controller: VirtualListController | undefined;
  const screen = renderComponent(() => (
    <VirtualList
      items={() => items}
      itemHeight={20}
      viewportHeight={100}
      overscan={1}
      getItemKey={(item) => item}
      role="listbox"
      accessibilityLabel="Controlled list"
      controllerRef={(next) => {
        controller = next;
      }}
    >
      {(item) => <Text role="option">{item()}</Text>}
    </VirtualList>
  ));

  controller?.scrollToIndex(99);
  screen.flush();

  expect(screen.getByRole("option", { name: "Item 99" })).toBeDefined();
  expect(screen.queryByRole("option", { name: "Item 0" })).toBeNull();
  expect(() => controller?.scrollToIndex(100)).toThrow(
    "VirtualList index 100 is outside 0..99",
  );
});

test("reports the mounted range only when its overscanned window changes", () => {
  const ranges: Array<{ start: number; end: number }> = [];
  const items = Array.from({ length: 100 }, (_, index) => `Item ${index}`);
  const screen = renderComponent(() => (
    <VirtualList
      items={() => items}
      itemHeight={20}
      viewportHeight={100}
      overscan={1}
      getItemKey={(item) => item}
      role="listbox"
      accessibilityLabel="Range list"
      onVisibleRangeChange={(range) => ranges.push({ ...range })}
    >
      {(item) => <Text role="option">{item()}</Text>}
    </VirtualList>
  ));

  expect(ranges).toEqual([{ start: 0, end: 6 }]);
  screen
    .getByRole("listbox", { name: "Range list" })
    .emit("scroll", { scrollY: 200 });
  screen.flush();
  expect(ranges).toEqual([
    { start: 0, end: 6 },
    { start: 9, end: 16 },
  ]);
});

test("keeps an end-pinned list attached when new items are appended", () => {
  const [items, setItems] = createSignal(
    Array.from({ length: 20 }, (_, index) => `Message ${index}`),
  );
  let controller: VirtualListController | undefined;
  const screen = renderComponent(() => (
    <VirtualList
      items={items}
      itemHeight={20}
      viewportHeight={100}
      getItemKey={(item) => item}
      role="log"
      accessibilityLabel="Conversation"
      anchorTo="end"
      followOnAppend
      controllerRef={(next) => {
        controller = next;
      }}
    >
      {(item) => <Text role="listitem">{item()}</Text>}
    </VirtualList>
  ));

  controller?.scrollToEnd();
  screen.flush();
  expect(controller?.isAtEnd()).toBe(true);
  expect(screen.getByRole("listitem", { name: "Message 19" })).toBeDefined();

  setItems((current) => [...current, "Message 20"]);
  screen.flush();

  expect(controller?.isAtEnd()).toBe(true);
  expect(screen.getByRole("listitem", { name: "Message 20" })).toBeDefined();
});
