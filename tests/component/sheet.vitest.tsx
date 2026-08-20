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
  expect(top.getByRole("dialog").className).toContain("border-b");
  top.dispose();

  const bottom = renderComponent(() => (
    <Sheet
      aria-label="Bottom sheet"
      side="bottom"
      trigger={(trigger) => <Button {...trigger}>Open bottom</Button>}
    />
  ));
  bottom.getByRole("button", { name: "Open bottom" }).click();
  expect(bottom.getByRole("dialog").className).toContain("border-t");
});
