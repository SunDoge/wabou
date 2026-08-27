import { renderComponent } from "@wabou/test/component";
import { Markdown } from "@wabou/ui";
import { createSignal } from "solid-js";
import { expect, test } from "vitest";

test("renders reactive GFM as native semantic components", () => {
  const [source, setSource] = createSignal(
    "## Result\n\n- **Fast** updates\n\n```ts\nconst ready = true\n```",
  );
  const screen = renderComponent(() => (
    <Markdown source={source()} aria-label="Agent response" />
  ));

  const response = screen.getByRole("region", { name: "Agent response" });
  expect(response.text).toContain("Result");
  expect(response.text).toContain("Fast updates");
  expect(screen.getByRole("group", { name: "Code block" }).text).toContain(
    "const ready = true",
  );

  setSource("A streamed **answer**.");
  screen.flush();
  expect(screen.getByRole("region", { name: "Agent response" }).text).toContain(
    "A streamed answer.",
  );
  expect(screen.queryByRole("group", { name: "Code block" })).toBeNull();
});
