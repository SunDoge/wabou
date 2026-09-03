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

  const empty = screen.getByRole("status", { name: "No projects" });
  expect(empty.className).toContain("flex-1");
  expect(empty.className).toContain("rounded-xl");
  expect(screen.getByRole("heading", { name: "Nothing here" })).toBeTruthy();
  expect(
    screen.getByRole("label", {
      name: "Create a project to get started.",
    }).className,
  ).toContain("min-h-10");
  expect(screen.getByRole("img", { name: "Folder" }).className).toContain(
    "w-10",
  );
  expect(
    screen.getByRole("group", { name: "Empty actions" }).className,
  ).toContain("flex-col");
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
