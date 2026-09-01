import { PlatformProvider } from "@wabou/core";
import { renderComponent } from "@wabou/test/component";
import {
  ChartContainer,
  ChartLegend,
  CodeBlock,
  createTanStackDataTable,
  DataTable,
  DirectionalRow,
  DirectionalText,
  DirectionProvider,
  Select,
  StatCard,
  Stepper,
  Text,
  Timeline,
  TypographyBlockquote,
  TypographyH1,
  TypographyInlineCode,
  TypographyList,
  TypographyListItem,
  TypographyP,
  View,
} from "@wabou/ui";
import { expect, test } from "vitest";

function byText(screen: ReturnType<typeof renderComponent>, text: string) {
  const visit = (
    node: (typeof screen.roots)[number],
  ): typeof node | undefined =>
    node.children.map(visit).find((candidate) => candidate !== undefined) ??
    (node.text === text ? node : undefined);
  const result = screen.roots
    .map(visit)
    .find((candidate) => candidate !== undefined);
  if (!result)
    throw new Error(`missing component text ${JSON.stringify(text)}`);
  return result;
}

test("projects explicit logical direction into native layout classes", () => {
  const screen = renderComponent(() => (
    <DirectionProvider dir="rtl">
      <DirectionalRow role="group" aria-label="RTL actions">
        <DirectionalText>First</DirectionalText>
        <DirectionalText>Second</DirectionalText>
      </DirectionalRow>
    </DirectionProvider>
  ));

  const row = screen.getByRole("group", { name: "RTL actions" });
  expect(row.className).toContain("flex-row-reverse");
  expect(byText(screen, "First").parent?.className).toContain("text-right");
});

test("allows Select motion to be disabled explicitly", () => {
  const screen = renderComponent(() => (
    <Select
      aria-label="Runtime"
      defaultValue="quickjs"
      motion={false}
      options={[
        { value: "quickjs", label: "QuickJS" },
        { value: "v8", label: "V8" },
      ]}
    />
  ));

  const trigger = screen.getByRole("combobox", { name: "Runtime" });
  trigger.click();
  expect(
    screen.getByRole("listbox").closestByRole("presentation")?.transform,
  ).toEqual([1, 0, 0, 1, 0, 0]);
  screen.getByRole("option", { name: "V8" }).click();
  expect(trigger.text).toContain("V8");
});

test("provides composable shadcn typography without implicit text merging", () => {
  const screen = renderComponent(() => (
    <View>
      <TypographyH1>Native typography</TypographyH1>
      <TypographyP>One explicit text host per paragraph.</TypographyP>
      <TypographyBlockquote>Predictable by construction.</TypographyBlockquote>
      <TypographyList>
        <TypographyListItem>Fast tests</TypographyListItem>
      </TypographyList>
      <TypographyInlineCode>bun run test</TypographyInlineCode>
    </View>
  ));

  expect(byText(screen, "Native typography").parent?.className).toContain(
    "text-4xl",
  );
  expect(
    byText(screen, "One explicit text host per paragraph.").parent?.className,
  ).toContain("whitespace-normal");
  expect(
    byText(screen, "Predictable by construction.").parent?.parent?.className,
  ).toContain("items-stretch");
  expect(
    byText(screen, "Predictable by construction.").parent?.parent?.children[0]
      ?.className,
  ).toContain("w-1");
  expect(byText(screen, "bun run test").parent?.className).toContain(
    "font-mono",
  );
});

test("renders a reusable TanStack-backed DataTable and selection state", () => {
  const screen = renderComponent(() => {
    const model = createTanStackDataTable({
      data: [
        { id: "a", name: "Alpha", score: 9 },
        { id: "b", name: "Beta", score: 4 },
      ],
      columns: [
        { accessorKey: "name", header: "Name" },
        { accessorKey: "score", header: "Score" },
      ],
      getRowId: (row) => row.id,
      enableRowSelection: true,
    });
    return <DataTable model={model} aria-label="Projects" selectable />;
  });

  const alpha = screen.getByRole("row", { name: "Select row a" });
  alpha.click();
  expect(screen.getByRole("row", { name: "Select row a" }).selected).toBe(true);
  screen.getByRole("columnheader", { name: "Sort by Score" }).click();
  expect(
    screen.getByRole("columnheader", { name: "Sort by Score" }).text,
  ).toContain("Desc");
  screen.dispose();

  const readOnly = renderComponent(() => {
    const model = createTanStackDataTable({
      data: [{ id: "stable", name: "Stable" }],
      columns: [{ accessorKey: "name", header: "Name" }],
      getRowId: (row) => row.id,
    });
    return <DataTable model={model} aria-label="Read-only projects" />;
  });
  expect(readOnly.getByRole("row", { name: "Row stable" }).disabled).toBe(
    false,
  );
});

test("shares chart series configuration with a native legend", () => {
  const screen = renderComponent(() => (
    <ChartContainer
      label="Traffic"
      config={{
        download: { label: "Download", colorClass: "bg-accent" },
        upload: { label: "Upload", colorClass: "bg-success-primary" },
      }}
    >
      <ChartLegend />
      <Text>Path content</Text>
    </ChartContainer>
  ));

  expect(screen.getByRole("img", { name: "Traffic" })).not.toBeNull();
  expect(byText(screen, "Download").parent?.parent?.className).toContain(
    "gap-2",
  );
  expect(byText(screen, "Upload")).not.toBeNull();
});

test("renders a copyable code block and uses the clipboard capability", async () => {
  const writes: string[] = [];
  const screen = renderComponent(() => (
    <PlatformProvider
      value={{
        clipboard: {
          readText: async () => null,
          writeText: async (value) => {
            writes.push(value);
          },
        },
      }}
    >
      <CodeBlock language="tsx" code="const ready = true" />
    </PlatformProvider>
  ));
  expect(screen.getByRole("group", { name: "Code block" })).not.toBeNull();
  expect(byText(screen, "tsx").parent?.className).toContain("text-secondary");
  const copy = screen.getByRole("button", { name: "Copy" });
  expect(copy.text).toBe("");
  copy.click();
  await screen.waitFor(() => expect(copy.text).toBe(""));
  expect(writes).toEqual(["const ready = true"]);
});

test("renders timeline status and metadata", () => {
  const screen = renderComponent(() => (
    <Timeline
      items={[
        { id: "one", title: "Created", status: "complete" },
        { id: "two", title: "Verified", time: "now", status: "current" },
      ]}
    />
  ));
  expect(screen.getByRole("group", { name: "Timeline" })).not.toBeNull();
  expect(byText(screen, "Verified")).not.toBeNull();
  expect(byText(screen, "now")).not.toBeNull();
});

test("keeps stepper controlled state explicit", () => {
  const selected: string[] = [];
  const screen = renderComponent(() => (
    <Stepper
      defaultValue="one"
      onValueChange={(value) => selected.push(value)}
      steps={[
        { id: "one", label: "One" },
        { id: "two", label: "Two" },
      ]}
    />
  ));
  screen.getByRole("button", { name: "Go to Two" }).click();
  expect(selected).toEqual(["two"]);
  expect(screen.getByRole("button", { name: "Go to Two" }).className).toContain(
    "bg-accent",
  );
});

test("composes stat card values without implicit text nodes", () => {
  const screen = renderComponent(() => (
    <StatCard
      label="Frames"
      value="120 fps"
      trend="+18%"
      description="this run"
    />
  ));
  expect(screen.getByRole("group", { name: "Frames" })).not.toBeNull();
  expect(byText(screen, "120 fps")).not.toBeNull();
  expect(byText(screen, "+18%")).not.toBeNull();
});
