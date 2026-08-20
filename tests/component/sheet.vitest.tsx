import { renderComponent } from "@wabou/test/component";
import { Button, Sheet, SheetTitle } from "@wabou/ui";
import { expect, test } from "vitest";

test("opens an edge panel and closes through its controls", () => {
  const screen = renderComponent(() => (
    <Sheet
      aria-label="Preferences"
      side="right"
      trigger={(trigger) => <Button {...trigger}>Preferences</Button>}
    >
      {(controls) => (
        <>
          <SheetTitle>Preferences</SheetTitle>
          <Button onClick={controls.close}>Done</Button>
        </>
      )}
    </Sheet>
  ));

  screen.getByRole("button", { name: "Preferences" }).click();
  const sheet = screen.getByRole("dialog", { name: "Preferences" });
  expect(sheet.className).toContain("w-[400px]");
  expect(sheet.className).toContain("border-l");
  expect(sheet.transform).toEqual([1, 0, 0, 1, 32, 0]);
  screen.getByRole("button", { name: "Done" }).click();
  expect(screen.queryByRole("dialog")).toBeNull();
});

test("supports top and bottom placement without a separate overlay primitive", () => {
  const top = renderComponent(() => (
    <Sheet
      aria-label="Top sheet"
      side="top"
      trigger={(trigger) => <Button {...trigger}>Open top</Button>}
    />
  ));
  top.getByRole("button", { name: "Open top" }).click();
  const topSheet = top.getByRole("dialog");
  expect(topSheet.className).toContain("border-b");
  expect(topSheet.transform).toEqual([1, 0, 0, 1, 0, -32]);
  top.dispose();

  const bottom = renderComponent(() => (
    <Sheet
      aria-label="Bottom sheet"
      side="bottom"
      trigger={(trigger) => <Button {...trigger}>Open bottom</Button>}
    />
  ));
  bottom.getByRole("button", { name: "Open bottom" }).click();
  const bottomSheet = bottom.getByRole("dialog");
  expect(bottomSheet.className).toContain("border-t");
  expect(bottomSheet.transform).toEqual([1, 0, 0, 1, 0, 32]);
});
