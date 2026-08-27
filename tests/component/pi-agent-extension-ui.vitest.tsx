import { renderComponent } from "@wabou/test/component";
import { Button } from "@wabou/ui";
import { createSignal } from "solid-js";
import { expect, test, vi } from "vitest";
import {
  type ExtensionUiAnswer,
  ExtensionUiDialog,
  type ExtensionUiDialogRequest,
  parseExtensionUiRequest,
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
