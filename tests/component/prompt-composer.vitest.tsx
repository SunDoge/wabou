import { renderComponent } from "@wabou/test/component";
import {
  Button,
  PromptComposer,
  PromptComposerAction,
  PromptComposerEditor,
  PromptComposerStatus,
  PromptComposerToolbar,
  PromptComposerTools,
  Text,
} from "@wabou/ui";
import { createSignal } from "solid-js";
import { expect, test } from "vitest";

test("prompt composer owns one focus-aware compound surface", () => {
  const screen = renderComponent(() => (
    <PromptComposer aria-label="Agent prompt">
      <PromptComposerStatus>
        <Text>4k tokens</Text>
      </PromptComposerStatus>
      <PromptComposerEditor
        value="Review this repository"
        aria-label="Message"
      />
      <PromptComposerToolbar aria-label="Prompt controls">
        <PromptComposerTools aria-label="Prompt tools">
          <Button>Attach</Button>
        </PromptComposerTools>
        <PromptComposerAction aria-label="Send">Send</PromptComposerAction>
      </PromptComposerToolbar>
    </PromptComposer>
  ));

  const composer = screen.getByRole("group", { name: "Agent prompt" });
  const editor = screen.getByRole("textbox", { name: "Message" });
  expect(composer.className).toContain("border-subtle");
  expect(composer.attribute("data-wabou-owns")).toBe("surface focus-ring");
  expect(
    screen.getByRole("toolbar", { name: "Prompt controls" }),
  ).toBeDefined();
  const toolbar = screen.getByRole("toolbar", { name: "Prompt controls" });
  const tools = screen.getByRole("group", { name: "Prompt tools" });
  expect(toolbar.className).toContain("flex-nowrap");
  expect(tools.className).toContain("flex-nowrap");
  expect(editor.className).toContain("h-12");
  expect(editor.attribute("data-wabou-owns")).toBe("native-editor");
  expect(screen.getByRole("button", { name: "Send" }).className).toContain(
    "rounded-full",
  );

  editor.focus();
  expect(composer.className).toContain("border-focus");
  expect(composer.className).not.toContain("border-subtle");

  editor.blur();
  expect(composer.className).toContain("border-subtle");
});

test("composer rows only wrap when the embedding surface opts in", () => {
  const screen = renderComponent(() => (
    <PromptComposerToolbar wrap aria-label="Adaptive prompt controls">
      <PromptComposerTools wrap aria-label="Adaptive prompt tools" />
    </PromptComposerToolbar>
  ));

  const toolbar = screen.getByRole("toolbar", {
    name: "Adaptive prompt controls",
  });
  const tools = screen.getByRole("group", { name: "Adaptive prompt tools" });
  expect(toolbar.className).toContain("flex-wrap");
  expect(toolbar.className).not.toContain("flex-nowrap");
  expect(tools.className).toContain("flex-wrap");
  expect(tools.className).not.toContain("flex-nowrap");
});

test("invalid prompt composer takes precedence over focus styling", () => {
  const screen = renderComponent(() => (
    <PromptComposer invalid aria-label="Invalid prompt">
      <PromptComposerEditor aria-label="Message" />
    </PromptComposer>
  ));
  const composer = screen.getByRole("group", { name: "Invalid prompt" });
  screen.getByRole("textbox", { name: "Message" }).focus();

  expect(composer.attribute("aria-invalid")).toBe("true");
  expect(composer.className).toContain("border-danger");
  expect(composer.className).not.toContain("border-focus");
});

test("prompt composer status does not reserve an empty layout row", () => {
  const [usage, setUsage] = createSignal<string>();
  const screen = renderComponent(() => (
    <PromptComposer aria-label="Agent prompt">
      <PromptComposerStatus role="status" aria-label="Usage">
        {usage() ? <Text>{usage()}</Text> : null}
      </PromptComposerStatus>
      <PromptComposerEditor aria-label="Message" />
    </PromptComposer>
  ));

  expect(screen.queryByRole("status", { name: "Usage" })).toBeNull();
  setUsage("4k tokens");
  screen.flush();
  expect(screen.getByRole("status", { name: "Usage" }).text).toBe("4k tokens");
  setUsage(undefined);
  screen.flush();
  expect(screen.queryByRole("status", { name: "Usage" })).toBeNull();
});
