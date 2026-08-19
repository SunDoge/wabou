import { expect, test } from "@wabou/test";

test("Wayland close-to-tray releases and recreates the native surface", async ({
  window,
}) => {
  const key = window.current;
  await window.nativeClose(key, "wayland");
  await expect(window).toHaveState(key, {
    presence: "surface-released",
    surfaceGeneration: 1,
  });

  await window.show(key);
  await expect(window).toHaveState(key, {
    presence: "visible",
    surfaceGeneration: 2,
  });
});
