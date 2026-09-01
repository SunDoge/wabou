import { renderComponent } from "@wabou/test/component";
import {
  TypographyH1,
  TypographyH3,
  TypographyLead,
  TypographyP,
} from "@wabou/ui";
import { expect, test } from "vitest";

test("Typography owns a stable native hierarchy and line-height rhythm", () => {
  const screen = renderComponent(() => (
    <>
      <TypographyH1 role="heading" aria-label="Page title">
        Page title
      </TypographyH1>
      <TypographyH3 role="heading" aria-label="Section title">
        Section title
      </TypographyH3>
      <TypographyLead role="paragraph" aria-label="Lead copy">
        Lead copy
      </TypographyLead>
      <TypographyP role="paragraph" aria-label="Body copy">
        Body copy
      </TypographyP>
    </>
  ));

  expect(
    screen.getByRole("heading", { name: "Page title" }).className,
  ).toContain("text-4xl leading-tight font-bold tracking-tight");
  expect(
    screen.getByRole("heading", { name: "Section title" }).className,
  ).toContain("text-2xl leading-tight font-semibold tracking-tight");
  expect(
    screen.getByRole("paragraph", { name: "Lead copy" }).className,
  ).toContain("text-xl leading-normal text-muted");
  expect(
    screen.getByRole("paragraph", { name: "Body copy" }).className,
  ).toContain("text-base leading-relaxed text-secondary");
});
