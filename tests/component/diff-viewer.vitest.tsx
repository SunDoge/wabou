import { renderComponent } from "@wabou/test/component";
import { DiffViewer } from "@wabou/ui";
import { expect, test } from "vitest";

const files = [
  {
    path: "src/main.ts",
    status: "modified" as const,
    additions: 2,
    deletions: 1,
    patch: "@@ -1,2 +1,3 @@\n-old\n+new\n+line",
  },
];

test("diff viewer keeps technical CodeMirror details collapsed by default", () => {
  const screen = renderComponent(() => <DiffViewer files={files} />);
  const file = screen.getByRole("button", { name: "src/main.ts" });

  expect(file.expanded).toBe(false);
  expect(screen.getByRole("region", { name: "Technical diff" }).text).toContain(
    "1 file changed",
  );
  expect(
    screen.queryByRole("textbox", { name: "Technical diff: src/main.ts" }),
  ).toBeNull();

  file.click();
  expect(file.expanded).toBe(true);
  expect(
    screen.getByRole("textbox", { name: "Technical diff: src/main.ts" }).value,
  ).toContain("+new");
});
