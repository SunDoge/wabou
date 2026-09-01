import { renderComponent } from "@wabou/test/component";
import {
  Button,
  Item,
  ItemActions,
  ItemContent,
  ItemDescription,
  ItemGroup,
  ItemTitle,
} from "@wabou/ui";
import { expect, test } from "vitest";

test("owns list semantics, selection, and overflow-safe text anatomy", () => {
  const screen = renderComponent(() => (
    <ItemGroup aria-label="Projects">
      <Item selected aria-label="Selected project">
        <ItemContent>
          <ItemTitle>A project title that must remain on one line</ItemTitle>
          <ItemDescription>A concise project description.</ItemDescription>
        </ItemContent>
      </Item>
    </ItemGroup>
  ));

  expect(screen.getByRole("list", { name: "Projects" })).toBeTruthy();
  const item = screen.getByRole("listitem", { name: "Selected project" });
  expect(item.selected).toBe(true);
  expect(item.className).toContain("bg-selected");
  expect(
    screen.getByRole("label", {
      name: "A project title that must remain on one line",
    }).className,
  ).toContain("truncate");
});

test("a disabled item authors a blocked and visibly subdued subtree", () => {
  const screen = renderComponent(() => (
    <Item disabled aria-label="Unavailable project">
      <ItemContent>
        <ItemTitle>Unavailable</ItemTitle>
      </ItemContent>
      <ItemActions>
        <Button>Open</Button>
      </ItemActions>
    </Item>
  ));

  const item = screen.getByRole("listitem", { name: "Unavailable project" });
  expect(item.interactionBlocked).toBe(true);
  expect(item.style("opacity")).toBe("0.45");
  expect(screen.getByRole("button", { name: "Open" })).toBeTruthy();
});
