import { renderComponent } from "@wabou/test/component";
import { Button } from "@wabou/ui";
import { createSignal } from "solid-js";
import { expect, test, vi } from "vitest";
import {
  type ExtensionUiAnswer,
  ExtensionUiChrome,
  ExtensionUiDialog,
  type ExtensionUiDialogRequest,
  parseExtensionUiEffect,
  parseExtensionUiRequest,
  reduceExtensionUiStatuses,
  reduceExtensionUiWidgets,
} from "../../apps/pi-agent/ui/extension-ui";

test("parses only blocking Pi extension dialog requests", () => {
  expect(
    parseExtensionUiRequest({
      type: "extension_ui_request",
      agentId: "agent-2",
      id: "choice-1",
      method: "select",
      title: "Choose a branch",
      options: ["main", "dev", 42],
    }),
  ).toEqual({
    agentId: "agent-2",
    id: "choice-1",
    method: "select",
    title: "Choose a branch",
    options: ["main", "dev"],
  });
  expect(
    parseExtensionUiRequest({
      type: "extension_ui_request",
      agentId: "agent-2",
      id: "notice-1",
      method: "notify",
      title: "Done",
    }),
  ).toBeUndefined();
});

function dialogHarness(
  request: ExtensionUiDialogRequest,
  respond: (answer: ExtensionUiAnswer) => void,
) {
  return () => {
    const [current, setCurrent] = createSignal<ExtensionUiDialogRequest>();
    return (
      <>
        <Button onClick={() => setCurrent(request)}>Open extension UI</Button>
        <ExtensionUiDialog
          request={current()}
          respond={(answer) => {
            respond(answer);
            setCurrent(undefined);
          }}
        />
      </>
    );
  };
}

test("returns a selected extension value exactly once", () => {
  const respond = vi.fn();
  const screen = renderComponent(
    dialogHarness(
      {
        agentId: "agent-1",
        id: "choice-1",
        method: "select",
        title: "Choose a branch",
        options: ["main", "dev"],
      },
      respond,
    ),
  );

  screen.getByRole("button", { name: "Open extension UI" }).click();
  screen.getByRole("option", { name: "dev" }).click();
  expect(respond).toHaveBeenCalledOnce();
  expect(respond).toHaveBeenCalledWith({ value: "dev" });
  expect(screen.queryByRole("dialog")).toBeNull();
});

test("supports keyboard-only extension selection and cancellation", () => {
  const respond = vi.fn();
  const screen = renderComponent(
    dialogHarness(
      {
        agentId: "agent-1",
        id: "choice-keyboard",
        method: "select",
        title: "Choose a branch",
        options: ["main", "dev"],
      },
      respond,
    ),
  );

  screen.getByRole("button", { name: "Open extension UI" }).click();
  const listbox = screen.getByRole("listbox", { name: "Choose a branch" });
  listbox.press("ArrowDown");
  listbox.press("Enter");
  expect(respond).toHaveBeenCalledWith({ value: "dev" });

  screen.getByRole("button", { name: "Open extension UI" }).click();
  screen.getByRole("listbox", { name: "Choose a branch" }).press("Escape");
  expect(respond).toHaveBeenLastCalledWith({ cancelled: true });
});

test("returns typed extension input", () => {
  const inputResponse = vi.fn();
  const input = renderComponent(
    dialogHarness(
      {
        agentId: "agent-1",
        id: "input-1",
        method: "input",
        title: "Name this branch",
        prefill: "feature/",
      },
      inputResponse,
    ),
  );
  input.getByRole("button", { name: "Open extension UI" }).click();
  const field = input.getByRole("textbox", { name: "Name this branch" });
  field.input("feature/dialogs");
  field.press("Enter");
  expect(inputResponse).toHaveBeenCalledWith({ value: "feature/dialogs" });
});

test("returns an explicit negative confirmation", () => {
  const confirmationResponse = vi.fn();
  const confirmation = renderComponent(
    dialogHarness(
      {
        agentId: "agent-1",
        id: "confirm-1",
        method: "confirm",
        title: "Continue?",
      },
      confirmationResponse,
    ),
  );
  confirmation.getByRole("button", { name: "Open extension UI" }).click();
  confirmation.getByRole("button", { name: "No" }).click();
  expect(confirmationResponse).toHaveBeenCalledOnce();
  expect(confirmationResponse).toHaveBeenCalledWith({ confirmed: false });
});

test("parses Pi extension notifications and editor effects", () => {
  expect(
    parseExtensionUiEffect({
      type: "extension_ui_request",
      agentId: "agent-2",
      id: "notice-1",
      method: "notify",
      message: "Indexing finished",
      notifyType: "warning",
    }),
  ).toEqual({
    kind: "notify",
    agentId: "agent-2",
    id: "notice-1",
    message: "Indexing finished",
    tone: "warning",
  });
  expect(
    parseExtensionUiEffect({
      type: "extension_ui_request",
      agentId: "agent-1",
      id: "editor-1",
      method: "set_editor_text",
      text: "Review @src/main.ts",
    }),
  ).toEqual({
    kind: "editorText",
    agentId: "agent-1",
    text: "Review @src/main.ts",
  });
  expect(
    parseExtensionUiEffect({
      type: "extension_ui_request",
      agentId: "agent-1",
      method: "setWidget",
      widgetKey: "build",
      widgetLines: "not an array",
    }),
  ).toBeUndefined();
});

test("updates and clears extension status and widget keys independently", () => {
  const statuses = reduceExtensionUiStatuses([], {
    kind: "status",
    agentId: "agent-1",
    key: "branch",
    text: "main",
  });
  expect(
    reduceExtensionUiStatuses(statuses, {
      kind: "status",
      agentId: "agent-1",
      key: "branch",
    }),
  ).toEqual([]);

  const widgets = reduceExtensionUiWidgets([], {
    kind: "widget",
    agentId: "agent-1",
    key: "tasks",
    lines: ["Build", "Test"],
    placement: "aboveEditor",
  });
  expect(widgets).toHaveLength(1);
  expect(
    reduceExtensionUiWidgets(widgets, {
      kind: "widget",
      agentId: "agent-2",
      key: "tasks",
      placement: "belowEditor",
    }),
  ).toEqual(widgets);
});

test("renders extension status and widgets at their requested placement", () => {
  const screen = renderComponent(() => (
    <ExtensionUiChrome
      placement="aboveEditor"
      statuses={[{ agentId: "agent-1", key: "git", text: "On main" }]}
      widgets={[
        {
          agentId: "agent-1",
          key: "tasks",
          lines: ["Build complete", "Tests passing"],
          placement: "aboveEditor",
        },
        {
          agentId: "agent-1",
          key: "hint",
          lines: ["Hidden below"],
          placement: "belowEditor",
        },
      ]}
    />
  ));

  expect(screen.getByRole("status", { name: "Extension status" }).text).toBe(
    "On main",
  );
  expect(
    screen.getByRole("region", { name: "Extension widget tasks" }).text,
  ).toBe("Build complete\nTests passing");
  expect(
    screen.queryByRole("region", { name: "Extension widget hint" }),
  ).toBeNull();
});
