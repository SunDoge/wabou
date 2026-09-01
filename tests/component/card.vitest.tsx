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
