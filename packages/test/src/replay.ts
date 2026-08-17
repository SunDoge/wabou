import type { TestAction, TestContext, TestPage } from "./index";

/** Execute a recorded trace against explicit page and window capabilities. */
export async function replayActions(
  actions: readonly TestAction[],
  page: TestPage,
  window: TestContext["window"],
): Promise<void> {
  for (const action of actions) {
    if (action.action === "nativeClose") {
      await window.nativeClose(action.windowId, action.platform);
    } else if (action.action === "showWindow") {
      await window.show(action.windowId);
    } else if (action.action === "clickByRole") {
      await page
        .forWindow(action.windowId)
        .getByRole(action.role, { name: action.label })
        .click();
    } else {
      const locator = page
        .forWindow(action.windowId)
        .getByRole(action.role, { name: action.label });
      const input = action.input;
      if (input.type === "probe") await locator.waitFor();
      else if (input.type === "drag")
        await locator.dragBy(input.deltaX, input.deltaY);
      else if (input.type === "key")
        await locator.press(input.key, {
          shift: (input.modifiers & 1) !== 0,
          control: (input.modifiers & 2) !== 0,
          alt: (input.modifiers & 4) !== 0,
          meta: (input.modifiers & 8) !== 0,
        });
      else if (input.type === "text") await locator.type(input.text);
      else if (input.type === "paste") await locator.paste(input.text);
      else if (input.type === "ime") await locator.ime(input.text);
      else await locator.wheel(input.deltaY, input.deltaX);
    }
  }
}
