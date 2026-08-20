import type { Locator, TestAction, TestContext, TestPage } from "./index";

export type ReplayLocatorAssertion = (
  locator: Locator,
  action: Extract<TestAction, { action: "assertByRole" }>,
) => void | Promise<void>;

export type ReplayWindowAssertion = (
  window: TestContext["window"],
  action: Extract<TestAction, { action: "assertWindowState" }>,
) => void | Promise<void>;

/** Execute a recorded trace against explicit page and window capabilities. */
export async function replayActions(
  actions: readonly TestAction[],
  page: TestPage,
  window: TestContext["window"],
  assertLocator: ReplayLocatorAssertion,
  assertWindow: ReplayWindowAssertion,
): Promise<void> {
  for (const action of actions) {
    if (action.action === "respondToEffect") {
      page.effects.respond(action.operation, action.result as never);
    } else if (action.action === "nativeClose") {
      await window.nativeClose(action.windowId, action.platform);
    } else if (action.action === "showWindow") {
      await window.show(action.windowId);
    } else if (action.action === "resizeWindow") {
      await window.resize(action.windowId, action.width, action.height);
    } else if (action.action === "fileDrop") {
      await window.fileDrop(action.windowId, action.phase, action.paths);
    } else if (action.action === "clickByRole") {
      await page
        .forWindow(action.windowId)
        .getByRole(action.role, { name: action.label, index: action.index })
        .click(action.wait);
    } else if (action.action === "waitForByRole") {
      await page
        .forWindow(action.windowId)
        .getByRole(action.role, { name: action.label, index: action.index })
        .waitFor(action.wait);
    } else if (action.action === "assertByRole") {
      await assertLocator(
        page
          .forWindow(action.windowId)
          .getByRole(action.role, { name: action.label, index: action.index }),
        action,
      );
    } else if (action.action === "assertWindowState") {
      await assertWindow(window, action);
    } else {
      const locator = page
        .forWindow(action.windowId)
        .getByRole(action.role, { name: action.label, index: action.index });
      const input = action.input;
      if (input.type === "probe") await locator.waitFor(action.wait);
      else if (input.type === "drag")
        await locator.dragBy(input.deltaX, input.deltaY, action.wait);
      else if (input.type === "key")
        await locator.press(
          input.key,
          {
            shift: (input.modifiers & 1) !== 0,
            control: (input.modifiers & 2) !== 0,
            alt: (input.modifiers & 4) !== 0,
            meta: (input.modifiers & 8) !== 0,
          },
          action.wait,
        );
      else if (input.type === "text")
        await locator.type(input.text, action.wait);
      else if (input.type === "paste")
        await locator.paste(input.text, action.wait);
      else if (input.type === "ime") await locator.ime(input.text, action.wait);
      else await locator.wheel(input.deltaY, input.deltaX, action.wait);
    }
  }
}
