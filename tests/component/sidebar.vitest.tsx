import { renderComponent } from "@wabou/test/component";
import {
  filterSidebarGroups,
  Sidebar,
  SidebarContent,
  SidebarEmpty,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarSearch,
} from "@wabou/ui";
import { createMemo, createSignal, For as ForValue, Show } from "solid-js";
import { expect, test } from "vitest";

const groups = [
  {
    label: "Workspace",
    items: [
      { id: "files", label: "Files", keywords: "documents folders" },
      { id: "search", label: "Search", keywords: "find text" },
    ],
  },
  {
    label: "Account",
    items: [{ id: "settings", label: "Settings", keywords: "preferences" }],
  },
] as const;

test("filters items by group, label, or application keywords", () => {
  expect(
    filterSidebarGroups(
      groups,
      "documents",
      (item) => `${item.label} ${item.keywords}`,
    ).map((group) => group.items.map((item) => item.id)),
  ).toEqual([["files"]]);

  expect(
    filterSidebarGroups(groups, "account", (item) => item.label).map((group) =>
      group.items.map((item) => item.id),
    ),
  ).toEqual([["settings"]]);

  expect(filterSidebarGroups(groups, "missing", (item) => item.label)).toEqual(
    [],
  );
});

test("controls one selected sidebar destination while leaving actions neutral", () => {
  const [value, setValue] = createSignal("files");
  const screen = renderComponent(() => (
    <SidebarMenu
      aria-label="Destinations"
      value={value()}
      onValueChange={setValue}
    >
      <SidebarMenuButton value="files" aria-label="Files" />
      <SidebarMenuButton value="search" aria-label="Search" />
      <SidebarMenuButton aria-label="Create file" />
    </SidebarMenu>
  ));

  const files = screen.getByRole("button", { name: "Files" });
  const search = screen.getByRole("button", { name: "Search" });
  const create = screen.getByRole("button", { name: "Create file" });
  expect(files.selected).toBe(true);
  expect(search.selected).toBe(false);
  expect(create.selected).toBe(false);

  search.click();
  screen.flush();
  expect(files.selected).toBe(false);
  expect(search.selected).toBe(true);
  expect(create.selected).toBe(false);
});

test("composes fixed chrome, searchable content, navigation and empty state", () => {
  let selected = "files";
  const Example = () => {
    const [query, setQuery] = createSignal("");
    const filtered = createMemo(() =>
      filterSidebarGroups(groups, query(), (item) => item.label),
    );
    return (
      <Sidebar aria-label="Workspace navigation">
        <SidebarHeader aria-label="Workspace header" />
        <SidebarSearch
          aria-label="Search workspace"
          value={query()}
          onValueChange={setQuery}
        />
        <SidebarContent>
          <ForValue each={filtered()}>
            {(group) => (
              <SidebarGroup aria-label={group.label}>
                <SidebarGroupLabel>{group.label}</SidebarGroupLabel>
                <ForValue each={group.items}>
                  {(item) => (
                    <SidebarMenuButton
                      aria-label={item.label}
                      selected={selected === item.id}
                      onClick={() => (selected = item.id)}
                    >
                      {item.label}
                    </SidebarMenuButton>
                  )}
                </ForValue>
              </SidebarGroup>
            )}
          </ForValue>
          <Show when={filtered().length === 0}>
            <SidebarEmpty title="Nothing here" />
          </Show>
        </SidebarContent>
        <SidebarFooter aria-label="Workspace footer" />
      </Sidebar>
    );
  };
  const screen = renderComponent(Example);

  expect(
    screen.getByRole("group", { name: "Workspace navigation" }),
  ).not.toBeNull();
  const search = screen.getByRole("textbox", { name: "Search workspace" });
  search.input("settings");
  expect(screen.getByRole("button", { name: "Settings" })).not.toBeNull();
  expect(screen.queryByRole("button", { name: "Files" })).toBeNull();

  search.input("missing");
  expect(screen.getByRole("status", { name: "Nothing here" })).not.toBeNull();
  search.press("Escape");
  const searchButton = screen.getByRole("button", { name: "Search" });
  expect(searchButton.className).toContain("rounded-lg");
  expect(searchButton.className).toContain("gap-2.5");
  searchButton.click();
  expect(selected).toBe("search");
});
