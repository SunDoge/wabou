import { renderComponent } from "@wabou/test/component";
import { ImageList, View } from "@wabou/ui";
import { createSignal } from "solid-js";
import { expect, test } from "vitest";

interface Page {
  id: number;
  path: string;
  title: string;
}

const pages: readonly Page[] = Array.from({ length: 100 }, (_, index) => ({
  id: index,
  path: `/pages/${index}.png`,
  title: `Page ${index + 1}`,
}));

test("delegates row virtualization to GPUI and exposes deterministic selection", () => {
  const Harness = () => {
    const [selected, setSelected] = createSignal<number>();
    return (
      <ImageList
        items={() => pages}
        getItemKey={(page) => page.id}
        renderThumbnail={() => <View class="w-full h-full bg-control" />}
        getLabel={(page) => page.title}
        itemHeight={80}
        viewportHeight={240}
        selectedKey={selected()}
        onSelectionChange={(page) => setSelected(page.id)}
        accessibilityLabel="Manga pages"
      />
    );
  };

  const screen = renderComponent(Harness);
  expect(screen.getByRole("listbox", { name: "Manga pages" })).toBeTruthy();
  const mounted = screen.getAllByRole("option");
  // Solid retains the keyed logical rows. The native VirtualList decides
  // which of these rows are materialized by GPUI for the current viewport.
  expect(mounted).toHaveLength(pages.length);

  const first = screen.getByRole("option", { name: "Page 1" });
  expect(first.style("background-color")).toBeNull();
  expect(first.className).not.toContain("rounded");
  expect(first.children[0]?.className).not.toContain("rounded");
  expect(first.selected).toBe(false);
  first.click();
  const selected = screen.getByRole("option", { name: "Page 1" });
  expect(selected.selected).toBe(true);
  expect(selected.className).toContain("bg-selected");
  expect(selected.style("background-color")).toBeNull();
});

test("rejects non-positive row and thumbnail geometry", () => {
  expect(() =>
    ImageList({
      items: () => pages,
      getItemKey: (page) => page.id,
      renderThumbnail: () => <View />,
      getLabel: (page) => page.title,
      itemHeight: 0,
    }),
  ).toThrow(RangeError);
});

test("keeps keyed image rows mounted across immutable data refreshes", () => {
  const [items, setItems] = createSignal<readonly Page[]>([
    { id: 1, path: "/old.png", title: "Old title" },
  ]);
  const screen = renderComponent(() => (
    <ImageList
      items={items}
      getItemKey={(page) => page.id}
      renderThumbnail={() => <View class="w-full h-full bg-control" />}
      getLabel={(page) => page.title}
      itemHeight={80}
      viewportHeight={160}
      accessibilityLabel="Stable images"
    />
  ));
  const identity = screen.getByRole("option", { name: "Old title" }).identity;

  setItems([{ id: 1, path: "/new.png", title: "New title" }]);
  screen.flush();

  expect(screen.getByRole("option", { name: "New title" }).identity).toEqual(
    identity,
  );
});
