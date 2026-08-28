import { renderComponent } from "@wabou/test/component";
import { expect, test } from "vitest";
import { MarkdownPage } from "../../apps/gallery/ui/pages/markdown";

test("updates the native preview from edited Markdown", () => {
  const screen = renderComponent(() => <MarkdownPage />);

  expect(
    screen.getByRole("region", { name: "Markdown preview" }).text,
  ).toContain("Wabou Markdown");

  screen
    .getByRole("textbox", { name: "Markdown source" })
    .input("# Updated preview\n\nA reactive paragraph.");

  const preview = screen.getByRole("region", { name: "Markdown preview" });
  expect(preview.text).toContain("Updated preview");
  expect(preview.text).toContain("A reactive paragraph.");
  expect(preview.text).not.toContain("Wabou Markdown");
});

test("renders asynchronous Markdown chunks as they arrive", async () => {
  const screen = renderComponent(() => <MarkdownPage />, { clock: "fake" });

  screen.getByRole("button", { name: "Stream example" }).click();
  await screen.advanceTime(80);
  expect(
    screen.getByRole("region", { name: "Markdown preview" }).text,
  ).toContain("Streaming Markdown");

  await screen.advanceTime(800);
  const preview = screen.getByRole("region", { name: "Markdown preview" });
  expect(preview.text).toContain("for await (const chunk of response)");
  expect(screen.getByRole("button", { name: "Stream example" })).not.toBeNull();
});

test("keeps inline code in the surrounding paragraph flow", () => {
  const screen = renderComponent(() => <MarkdownPage />);

  screen
    .getByRole("textbox", { name: "Markdown source" })
    .input("Before `code` after.");

  const text = screen.getByRole("region", { name: "Markdown preview" }).text;
  expect(text).toContain("Before code after.");
  expect(text).not.toContain("Beforecodeafter.");
});
