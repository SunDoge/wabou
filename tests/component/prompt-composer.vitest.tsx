import { renderComponent } from "@wabou/test/component";
import {
  Button,
  PromptComposer,
  PromptComposerStatus,
  PromptComposerToolbar,
  PromptComposerTools,
  Text,
  TextArea,
} from "@wabou/ui";
import { expect, test } from "vitest";

test("prompt composer owns one focus-aware compound surface", () => {
  const screen = renderComponent(() => (
    <PromptComposer aria-label="Agent prompt">
      <PromptComposerStatus>
        <Text>4k tokens</Text>
      </PromptComposerStatus>
      <TextArea chrome="none" aria-label="Message" />
      <PromptComposerToolbar aria-label="Prompt controls">
        <PromptComposerTools aria-label="Prompt tools">
          <Button>Attach</Button>
        </PromptComposerTools>
        <Button>Send</Button>
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
  expect(screen.getByRole("group", { name: "Prompt tools" })).toBeDefined();

  editor.focus();
  expect(composer.className).toContain("border-focus");
  expect(composer.className).not.toContain("border-subtle");

  editor.blur();
  expect(composer.className).toContain("border-subtle");
});

test("invalid prompt composer takes precedence over focus styling", () => {
  const screen = renderComponent(() => (
    <PromptComposer invalid aria-label="Invalid prompt">
      <TextArea chrome="none" aria-label="Message" />
    </PromptComposer>
  ));
  const composer = screen.getByRole("group", { name: "Invalid prompt" });
  screen.getByRole("textbox", { name: "Message" }).focus();

  expect(composer.attribute("aria-invalid")).toBe("true");
  expect(composer.className).toContain("border-danger");
  expect(composer.className).not.toContain("border-focus");
});
