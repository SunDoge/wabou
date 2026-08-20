import { renderComponent } from "@wabou/test/component";
import {
  createResizablePanelState,
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
  Text,
} from "@wabou/ui";
import { createRoot, createSignal, flush } from "solid-js";
import { expect, test } from "vitest";

const panels = [
  { id: "navigation", defaultSize: 30, minSize: 20, maxSize: 60 },
  { id: "content", defaultSize: 70, minSize: 40, maxSize: 80 },
] as const;

test("keeps an explicit pair total while enforcing both panel constraints", () => {
  createRoot((dispose) => {
    const state = createResizablePanelState({ panels });

    state.resizePair("navigation", "content", 90);
    flush();
    expect(state.sizes()).toEqual({ navigation: 60, content: 40 });
    state.resizePair("navigation", "content", 0);
    flush();
    expect(state.sizes()).toEqual({ navigation: 20, content: 80 });
    expect(() => state.resizePair("missing", "content", 50)).toThrow(
      "unknown resizable panel",
    );
    dispose();
  });
});

test("resizes through a captured pointer sequence and keyboard commands", () => {
  const screen = renderComponent(() => (
    <ResizablePanelGroup panels={panels} aria-label="Workspace split">
      <ResizablePanel id="navigation">
        <Text>Navigation</Text>
      </ResizablePanel>
      <ResizableHandle
        before="navigation"
        after="content"
        keyboardStep={5}
        aria-label="Resize navigation"
      />
      <ResizablePanel id="content">
        <Text>Content</Text>
      </ResizablePanel>
    </ResizablePanelGroup>
  ));
  screen.getByRole("group", { name: "Workspace split" }).resize({
    width: 400,
    height: 200,
  });
  const handle = screen.getByRole("separator", { name: "Resize navigation" });

  handle.pointerDown({ clientX: 100, clientY: 20 });
  expect(handle.className).toContain("bg-accent");
  handle.pointerMove({ clientX: 180, clientY: 20 });
  handle.pointerUp({ clientX: 180, clientY: 20 });
  expect(handle.attribute("aria-valuenow")).toBe("50");

  handle.press("ArrowRight");
  expect(handle.attribute("aria-valuenow")).toBe("55");
  handle.press("End");
  expect(handle.attribute("aria-valuenow")).toBe("60");
});

test("supports application-owned sizes", () => {
  const Controlled = () => {
    const [sizes, setSizes] = createSignal({ navigation: 30, content: 70 });
    return (
      <ResizablePanelGroup
        panels={panels}
        value={sizes()}
        onValueChange={(next) =>
          setSizes({ navigation: next.navigation, content: next.content })
        }
        aria-label="Controlled split"
      >
        <ResizablePanel id="navigation" />
        <ResizableHandle
          before="navigation"
          after="content"
          aria-label="Controlled divider"
        />
        <ResizablePanel id="content" />
      </ResizablePanelGroup>
    );
  };
  const screen = renderComponent(Controlled);
  const handle = screen.getByRole("separator", { name: "Controlled divider" });

  handle.press("ArrowRight");
  expect(handle.attribute("aria-valuenow")).toBe("32");
});
