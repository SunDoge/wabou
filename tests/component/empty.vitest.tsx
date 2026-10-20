import { renderComponent } from "@wabou/test/component";
import {
  Button,
  Empty,
  EmptyContent,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
  Text,
} from "@wabou/ui";
import { expect, test } from "vitest";

test("composes the complete shadcn empty-state anatomy", () => {
  const screen = renderComponent(() => (
    <Empty role="status" aria-label="No projects">
      <EmptyHeader role="group" aria-label="Empty heading">
        <EmptyMedia variant="icon" role="img" aria-label="Folder">
          <Text>Icon</Text>
        </EmptyMedia>
        <EmptyTitle>Nothing here</EmptyTitle>
        <EmptyDescription>Create a project to get started.</EmptyDescription>
      </EmptyHeader>
      <EmptyContent role="group" aria-label="Empty actions">
        <Button>New project</Button>
      </EmptyContent>
    </Empty>
  ));

  expect(screen.getByRole("status", { name: "No projects" })).toBeDefined();
  expect(screen.getByRole("heading", { name: "Nothing here" })).toBeDefined();
  expect(
    screen.getByRole("label", { name: "Create a project to get started." }),
  ).toBeDefined();
  expect(screen.getByRole("img", { name: "Folder" })).toBeDefined();
  expect(screen.getByRole("group", { name: "Empty actions" })).toBeDefined();
  expect(screen.getByRole("button", { name: "New project" })).toBeDefined();
});

test("keeps plain empty states embeddable and default media unstyled", () => {
  const screen = renderComponent(() => (
    <Empty variant="plain" role="status" aria-label="Filtered empty">
      <EmptyMedia role="img" aria-label="Search" />
    </Empty>
  ));

  const empty = screen.getByRole("status", { name: "Filtered empty" });
  expect(empty.className).toContain("bg-transparent");
  expect(empty.className).not.toContain("border-subtle");
  expect(screen.getByRole("img", { name: "Search" }).className).not.toContain(
    "bg-control",
  );
});
