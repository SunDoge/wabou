import { expect, test } from "@wabou/test";

test("Wayland close-to-tray releases and recreates the native surface", async ({
  window,
}) => {
  await window.nativeClose(1, "wayland");
  await expect(window).toHaveState(1, {
    presence: "surface-released",
    surfaceGeneration: 1,
  });

  await window.show(1);
  await expect(window).toHaveState(1, {
    presence: "visible",
    surfaceGeneration: 2,
  });
});
