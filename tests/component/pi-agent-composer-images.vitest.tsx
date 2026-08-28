import type { Dialog } from "@wabou/core";
import { renderComponent } from "@wabou/test/component";
import { Text } from "@wabou/ui";
import { createSignal } from "solid-js";
import { expect, test, vi } from "vitest";
import {
  ComposerImagePicker,
  ComposerImages,
} from "../../apps/pi-agent/ui/composer-images";

test("selects, deduplicates, displays, and removes prompt images", async () => {
  const open = vi.fn(async () => ["/tmp/page.png", "/tmp/page.png"]);
  const dialog: Dialog = {
    open,
    save: async () => null,
    pickDirectory: async () => null,
    message: async () => "ok",
  };
  const App = () => {
    const [paths, setPaths] = createSignal<readonly string[]>([]);
    return (
      <>
        <ComposerImages paths={paths()} change={setPaths} />
        <ComposerImagePicker paths={paths()} change={setPaths} />
        <Text role="status">{paths().join("|")}</Text>
      </>
    );
  };
  const screen = renderComponent(App, { platform: { dialog } });

  screen.getByRole("button", { name: "Attach images" }).click();
  await screen.waitFor(() => {
    expect(screen.getByRole("status").text).toBe("/tmp/page.png");
  });
  expect(open).toHaveBeenCalledWith({
    title: "Attach images",
    multiple: true,
    filters: [
      { name: "Images", extensions: ["png", "jpg", "jpeg", "webp", "gif"] },
    ],
  });
  expect(screen.getByRole("group", { name: "Attached images" }).text).toContain(
    "page.pngImage attachment",
  );

  screen.getByRole("button", { name: "Remove page.png" }).click();
  expect(screen.queryByRole("group", { name: "Attached images" })).toBeNull();
});
