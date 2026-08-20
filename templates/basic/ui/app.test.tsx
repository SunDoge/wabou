import { renderComponent } from "@wabou/test/component";
import { expect, test } from "vitest";
import { App } from "./app";

test("renders the application heading", () => {
  const screen = renderComponent(() => <App />);
  expect(
    screen.getByRole("heading", { name: "__WABOU_PROJECT_NAME__" }).text,
  ).toBe("__WABOU_PROJECT_NAME__");
});
