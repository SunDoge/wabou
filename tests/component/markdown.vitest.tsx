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

test("renders compact conversation Markdown including GFM tables", () => {
  const screen = renderComponent(() => (
    <Markdown
      variant="conversation"
      aria-label="Compact response"
      source={
        "## Result\n\n| File | State |\n| --- | --- |\n| api.ts | Updated |"
      }
    />
  ));

  const response = screen.getByRole("region", { name: "Compact response" });
  expect(response.className).toContain("gap-2.5");
  expect(response.text).toContain("FileState");
  expect(response.text).toContain("api.tsUpdated");
  expect(response.snapshot()).toMatchObject({ role: "region" });
});

test("repairs incomplete inline Markdown only while streaming", () => {
  const [streaming, setStreaming] = createSignal(true);
  const screen = renderComponent(() => (
    <Markdown
      source="The **important answer"
      streaming={streaming()}
      aria-label="Streaming response"
    />
  ));

  const response = screen.getByRole("region", { name: "Streaming response" });
  expect(response.text).toBe("The important answer");

  setStreaming(false);
  screen.flush();
  expect(response.text).toContain("**important answer");
});
