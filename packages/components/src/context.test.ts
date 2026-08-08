import { expect, test } from "bun:test";
import { createComponent, createRoot } from "solid-js";
import {
  ComponentsProvider,
  type ComponentsTheme,
  useComponentsTheme,
} from "./theme";

test("useComponentsTheme reads the nearest provider and has a stable default", () => {
  expect(useComponentsTheme()()).toBe("dark");
  let received: ComponentsTheme | undefined;

  createRoot((dispose) => {
    createComponent(ComponentsProvider, {
      theme: "light",
      get children() {
        received = useComponentsTheme()();
        return null;
      },
    });
    dispose();
  });

  expect(received).toBe("light");
});
