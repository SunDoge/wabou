import { renderComponent } from "@wabou/test/component";
import { Avatar, AvatarGroup, avatarInitials } from "@wabou/ui";
import { expect, test } from "vitest";

test("Avatar derives stable initials from a name", () => {
  expect(avatarInitials("Jason Lee")).toBe("JL");
  expect(avatarInitials("  Wabou   Project  ")).toBe("WP");
  expect(avatarInitials("solid")).toBe("SO");
  expect(avatarInitials("漫画")).toBe("漫画");
  expect(avatarInitials("   ")).toBe("");
});

test("Avatar owns fallback, semantics, clipping and size", () => {
  const screen = renderComponent(() => (
    <AvatarGroup aria-label="Contributors">
      <Avatar name="Wabou Project" size="sm" />
      <Avatar name="Solid Renderer" fallback="SR2" alt="Renderer" />
    </AvatarGroup>
  ));

  const generated = screen.getByRole("img", { name: "Wabou Project" });
  const explicit = screen.getByRole("img", { name: "Renderer" });

  expect(generated.text).toContain("WP");
  for (const className of [
    "w-8",
    "h-8",
    "text-xs",
    "flex-none",
    "overflow-hidden",
    "rounded-full",
  ])
    expect(generated.className).toContain(className);
  expect(explicit.text).toContain("SR2");
  expect(screen.getByRole("group", { name: "Contributors" })).toBeDefined();
});
