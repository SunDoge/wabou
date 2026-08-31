import { renderComponent } from "@wabou/test/component";
import { expect, test, vi } from "vitest";
import type { PiSkill } from "../../apps/pi-agent/ui/api";
import { SkillsPage } from "../../apps/pi-agent/ui/skills-page";

const skills: readonly PiSkill[] = [
  {
    id: "project:review",
    name: "Review changes",
    description: "Review the current working tree",
    scope: "project",
    source: "pi",
    path: "/work/project/.pi/skills/review",
    content: "# Review changes\n\nInspect the current diff before editing.",
  },
  {
    id: "user:frontend",
    name: "Frontend design",
    description: "Improve visual hierarchy and interaction states",
    scope: "user",
    source: "shared",
    path: "/home/user/.agents/skills/frontend-design",
    content: "# Frontend design\n\nStart from the primary user task.",
  },
];

test("skills page loads, filters, and selects native skill records", async () => {
  const load = vi.fn(async () => skills);
  const close = vi.fn();
  const screen = renderComponent(() => (
    <SkillsPage
      cwd="/work/project"
      project="Project"
      load={load}
      close={close}
    />
  ));
  screen.getByRole("group", { name: "Skill browser layout" }).resize({
    width: 960,
    height: 560,
  });

  await screen.waitFor(() =>
    expect(
      screen.getByRole("button", { name: "Review changes" }),
    ).toBeDefined(),
  );
  expect(load).toHaveBeenCalledWith("/work/project");
  expect(screen.getByRole("region", { name: "Review changes" }).text).toContain(
    "Inspect the current diff before editing.",
  );

  screen.getByRole("textbox", { name: "Search skills" }).input("frontend");
  expect(screen.queryByRole("button", { name: "Review changes" })).toBeNull();
  screen.getByRole("button", { name: "Frontend design" }).click();
  expect(
    screen.getByRole("region", { name: "Frontend design" }).text,
  ).toContain("Start from the primary user task.");

  screen.getByRole("button", { name: "Back to projects" }).click();
  expect(close).toHaveBeenCalledTimes(1);
});

test("skills page presents details as a controlled dialog in compact layouts", async () => {
  const screen = renderComponent(() => (
    <SkillsPage
      cwd="/work/project"
      project="Project"
      load={async () => skills}
      close={() => {}}
    />
  ));
  screen.getByRole("group", { name: "Skill browser layout" }).resize({
    width: 520,
    height: 560,
  });

  await screen.waitFor(() =>
    expect(
      screen.getByRole("button", { name: "Review changes" }),
    ).toBeDefined(),
  );
  expect(screen.queryByRole("dialog", { name: "Review changes" })).toBeNull();

  screen.getByRole("button", { name: "Review changes" }).click();
  expect(screen.getByRole("dialog", { name: "Review changes" }).text).toContain(
    "Inspect the current diff before editing.",
  );
});

test("skills page exposes loading failures without losing page navigation", async () => {
  const close = vi.fn();
  let attempt = 0;
  const screen = renderComponent(() => (
    <SkillsPage
      cwd="/work/project"
      project="Project"
      load={async () => {
        attempt += 1;
        if (attempt === 1) throw new Error("permission denied");
        return skills;
      }}
      close={close}
    />
  ));

  await screen.waitFor(() =>
    expect(screen.getByRole("alert").text).toContain("permission denied"),
  );
  expect(
    screen.getByRole("heading", { name: "Could not load skills" }),
  ).toBeDefined();
  screen.getByRole("button", { name: "Try again" }).click();
  await screen.waitFor(() =>
    expect(
      screen.getByRole("button", { name: "Review changes" }),
    ).toBeDefined(),
  );
  expect(attempt).toBe(2);
  screen.getByRole("button", { name: "Back to projects" }).click();
  expect(close).toHaveBeenCalledTimes(1);
});
