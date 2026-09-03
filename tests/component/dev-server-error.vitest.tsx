import { dispatchHostMessageForTest } from "@wabou/core/testing";
import { renderComponent } from "@wabou/test/component";
import { ComponentsProvider, Text } from "@wabou/ui";
import { expect, test } from "vitest";

test("components provider surfaces and clears Vite diagnostics", () => {
  const screen = renderComponent(() => (
    <ComponentsProvider>
      <Text>Last good interface</Text>
    </ComponentsProvider>
  ));

  expect(screen.roots.some((root) => root.text === "Last good interface")).toBe(
    true,
  );

  dispatchHostMessageForTest(
    "wabou:dev-server-error",
    JSON.stringify({
      message: "Expected `,` but found identifier",
      stack: "Error: transform failed",
      frame: "10 | <Button broken prop />",
      plugin: "solid",
      loc: { file: "/src/sidebar.tsx", line: 10, column: 18 },
    }),
  );
  screen.flush();

  const dialog = screen.getByRole("alertdialog", {
    name: "Development build failed",
  });
  expect(dialog.text).toContain("Expected `,` but found identifier");
  expect(dialog.text).toContain("/src/sidebar.tsx:10:18 · solid");
  expect(dialog.text).toContain("10 | <Button broken prop />");
  expect(screen.roots.some((root) => root.text === "Last good interface")).toBe(
    true,
  );

  dispatchHostMessageForTest("wabou:dev-server-ready", "{}");
  screen.flush();
  expect(
    screen.queryByRole("alertdialog", { name: "Development build failed" }),
  ).toBeNull();
});
