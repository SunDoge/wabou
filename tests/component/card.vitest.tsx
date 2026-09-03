import { renderComponent } from "@wabou/test/component";
import {
  Badge,
  Card,
  CardAction,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
  Text,
} from "@wabou/ui";
import { expect, test } from "vitest";

function byText(screen: ReturnType<typeof renderComponent>, text: string) {
  const visit = (
    node: (typeof screen.roots)[number],
  ): typeof node | undefined =>
    node.children.map(visit).find((candidate) => candidate !== undefined) ??
    (node.text === text ? node : undefined);
  const result = screen.roots
    .map(visit)
    .find((candidate) => candidate !== undefined);
  if (!result) throw new Error(`missing card text ${JSON.stringify(text)}`);
  return result;
}

test("exposes the complete card anatomy and top-end action slot", () => {
  const screen = renderComponent(() => (
    <Card role="group" aria-label="Project">
      <CardHeader role="group" aria-label="Project heading">
        <CardTitle>Wabou</CardTitle>
        <CardDescription>Native Solid UI</CardDescription>
        <CardAction role="group" aria-label="Project state">
          <Badge>Ready</Badge>
        </CardAction>
      </CardHeader>
      <CardContent>
        <Text>Content</Text>
      </CardContent>
      <CardFooter>
        <Text>Footer</Text>
      </CardFooter>
    </Card>
  ));

  const card = screen.getByRole("group", { name: "Project" });
  expect(card.className).toContain("overflow-hidden");
  expect(card.className).toContain("flex-none");
  expect(card.className).toContain("rounded-lg");
  const header = screen.getByRole("group", { name: "Project heading" });
  expect(header.className).toContain("relative");
  expect(header.className).toContain("px-5");
  const action = screen.getByRole("group", { name: "Project state" });
  expect(action.className).toContain("absolute");
  expect(action.className).toContain("right-5");
});

test("provides container variants and propagates density to its anatomy", () => {
  const screen = renderComponent(() => (
    <>
      <Card
        role="group"
        aria-label="Compact filled card"
        variant="filled"
        size="sm"
      >
        <CardHeader role="group" aria-label="Compact header">
          <CardTitle>Compact</CardTitle>
          <CardDescription>Dense supporting copy</CardDescription>
          <CardAction role="group" aria-label="Compact action">
            <Badge>Ready</Badge>
          </CardAction>
        </CardHeader>
        <CardContent role="group" aria-label="Compact content">
          <Text>Content</Text>
        </CardContent>
        <CardFooter role="group" aria-label="Compact footer">
          <Text>Footer</Text>
        </CardFooter>
      </Card>
      <Card role="group" aria-label="Outline card" variant="outline" />
      <Card role="group" aria-label="Plain card" variant="plain" />
    </>
  ));

  const compact = screen.getByRole("group", { name: "Compact filled card" });
  expect(compact.className).toContain("bg-control");
  expect(compact.className).toContain("border-transparent");
  expect(compact.attribute("variant")).toBeNull();
  expect(compact.attribute("size")).toBeNull();
  expect(
    screen.getByRole("group", { name: "Compact header" }).className,
  ).toContain("px-4");
  expect(byText(screen, "Compact").parent?.className).toContain("text-sm");
  expect(byText(screen, "Dense supporting copy").parent?.className).toContain(
    "text-xs",
  );
  expect(
    screen.getByRole("group", { name: "Compact action" }).className,
  ).toContain("top-4");
  expect(
    screen.getByRole("group", { name: "Compact content" }).className,
  ).toContain("pb-4");
  expect(
    screen.getByRole("group", { name: "Compact footer" }).className,
  ).toContain("px-4");
  expect(
    screen.getByRole("group", { name: "Outline card" }).className,
  ).toContain("border-strong");
  expect(screen.getByRole("group", { name: "Plain card" }).className).toContain(
    "border-0",
  );
});
