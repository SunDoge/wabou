import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  CodeBlock,
  CopyButton,
  DirectionalRow,
  DirectionalText,
  DirectionProvider,
  DropZone,
  InlineEdit,
  NativeSelect,
  PropertyList,
  PropertyRow,
  ShortcutRecorder,
  SplitButton,
  StatCard,
  StatusBar,
  StatusBarItem,
  StatusBarSeparator,
  Stepper,
  Text,
  Timeline,
  TypographyBlockquote,
  TypographyH1,
  TypographyH2,
  TypographyInlineCode,
  TypographyLead,
  TypographyList,
  TypographyListItem,
  TypographyMuted,
  TypographyP,
  View,
} from "@wabou/ui";

export function InlineEditPage() {
  return (
    <Card class="max-w-xl">
      <CardHeader>
        <CardTitle>Project identity</CardTitle>
      </CardHeader>
      <CardContent class="flex flex-row items-center gap-3">
        <Text class="text-sm text-muted">Name</Text>
        <InlineEdit
          aria-label="Project name"
          class="min-w-0 flex-1"
          defaultValue="Native workspace"
        />
      </CardContent>
    </Card>
  );
}

export function DropZonePage() {
  return (
    <DropZone
      class="max-w-2xl"
      label="Import project files"
      activeLabel="Release to import"
      description="Accepts native filesystem paths without routing file contents through JavaScript."
      accept={(path) => path.endsWith(".json") || path.endsWith(".toml")}
      onDrop={() => {}}
    />
  );
}

export function NativeSelectPage() {
  return (
    <Card class="max-w-lg">
      <CardHeader>
        <CardTitle>Native select</CardTitle>
      </CardHeader>
      <CardContent class="flex flex-col gap-3">
        <NativeSelect
          aria-label="Preferred runtime"
          class="w-full"
          defaultValue="quickjs"
          options={[
            { value: "quickjs", label: "QuickJS" },
            { value: "v8", label: "V8" },
            { value: "javascriptcore", label: "JavaScriptCore" },
          ]}
        />
        <Text class="text-sm text-muted whitespace-normal">
          Immediate native popup behavior for ordinary form choices.
        </Text>
      </CardContent>
    </Card>
  );
}

export function CodeBlockPage() {
  return (
    <CodeBlock
      class="max-w-2xl"
      language="tsx"
      code={'<Button variant="outline">Save</Button>'}
    />
  );
}

export function CopyButtonPage() {
  return (
    <CopyButton
      value="wabou"
      idleLabel="Copy package name"
      copiedLabel="Package copied"
    />
  );
}

export function TimelinePage() {
  return (
    <Timeline
      class="max-w-xl"
      items={[
        {
          id: "design",
          title: "Design",
          description: "Define the native contract.",
          time: "09:30",
          status: "complete",
        },
        {
          id: "build",
          title: "Build",
          description: "Compile typed operations.",
          time: "10:10",
          status: "current",
        },
        {
          id: "verify",
          title: "Verify",
          description: "Inspect the layout snapshot.",
          status: "pending",
        },
      ]}
    />
  );
}

export function StepperPage() {
  return (
    <Stepper
      class="max-w-3xl"
      defaultValue="verify"
      steps={[
        {
          id: "details",
          label: "Details",
          description: "Describe the project",
        },
        { id: "runtime", label: "Runtime", description: "Choose capabilities" },
        { id: "verify", label: "Verify", description: "Review and create" },
      ]}
    />
  );
}

export function StatCardPage() {
  return (
    <View class="max-w-3xl grid grid-cols-2 gap-4">
      <StatCard
        label="Native frames"
        value="120 fps"
        trend="+18%"
        description="from last run"
      />
      <StatCard
        label="Layout checks"
        value="74"
        description="passing fixtures"
        indicatorClass="bg-success-primary"
      />
    </View>
  );
}

export function SplitButtonPage() {
  return (
    <SplitButton
      class="w-64"
      label="Run project"
      items={[
        {
          id: "debug",
          label: "Run with debugger",
          description: "Attach native diagnostics",
        },
        {
          id: "profile",
          label: "Profile performance",
          description: "Capture frame timings",
        },
      ]}
    />
  );
}

export function ShortcutRecorderPage() {
  return (
    <Card class="max-w-lg">
      <CardHeader>
        <CardTitle>Keyboard shortcuts</CardTitle>
      </CardHeader>
      <CardContent>
        <ShortcutRecorder
          label="Open command palette"
          defaultValue="Primary+Shift+P"
        />
      </CardContent>
    </Card>
  );
}

export function StatusBarPage() {
  return (
    <View class="w-full max-w-3xl overflow-hidden rounded-lg border border-subtle bg-surface">
      <View class="h-48 items-center justify-center">
        <Text class="text-sm text-muted">Editor workspace</Text>
      </View>
      <StatusBar aria-label="Editor status">
        <StatusBarItem grow>src/main.rs</StatusBarItem>
        <StatusBarSeparator />
        <StatusBarItem>Ln 12, Col 4</StatusBarItem>
        <StatusBarSeparator />
        <StatusBarItem>UTF-8</StatusBarItem>
      </StatusBar>
    </View>
  );
}

export function PropertyListPage() {
  return (
    <PropertyList aria-label="Runtime properties" class="max-w-2xl">
      <PropertyRow
        name="runtime"
        value="QuickJS with typed native capabilities"
      />
      <PropertyRow name="renderer" value="GPUI retained projection" />
      <PropertyRow name="layout" value="Taffy logical pixels" />
    </PropertyList>
  );
}

export function DirectionPage() {
  return (
    <View class="max-w-xl flex flex-col gap-4">
      <DirectionProvider dir="ltr">
        <DirectionalRow class="p-4 gap-3 rounded-lg border border-subtle bg-surface">
          <DirectionalText class="font-semibold">LTR</DirectionalText>
          <DirectionalText class="text-muted">First to last</DirectionalText>
        </DirectionalRow>
      </DirectionProvider>
      <DirectionProvider dir="rtl">
        <DirectionalRow class="p-4 gap-3 rounded-lg border border-subtle bg-surface">
          <DirectionalText class="font-semibold">RTL</DirectionalText>
          <DirectionalText class="text-muted">آخر إلى أول</DirectionalText>
        </DirectionalRow>
      </DirectionProvider>
    </View>
  );
}

export function TypographyPage() {
  return (
    <View class="max-w-3xl flex flex-col gap-5">
      <TypographyH1>Native applications, composed in TypeScript</TypographyH1>
      <TypographyLead>
        Explicit text hosts keep layout and updates predictable across the JS
        and Rust boundary.
      </TypographyLead>
      <TypographyH2>One rendering contract</TypographyH2>
      <TypographyP>
        Components own typography, wrapping, and spacing while applications
        remain free to compose the final hierarchy.
      </TypographyP>
      <View class="flex flex-col gap-1 rounded-lg border border-subtle bg-surface px-4 py-3">
        <Text class="font-medium">Native text selection</Text>
        <Text class="text-sm text-muted">
          Drag across this paragraph, then use the platform copy shortcut. Text
          selection and highlighting are retained by GPUI rather than simulated
          in JavaScript.
        </Text>
      </View>
      <TypographyBlockquote>
        Make invalid UI states visible to tests before pixels are rendered.
      </TypographyBlockquote>
      <TypographyList>
        <TypographyListItem>Solid reactivity</TypographyListItem>
        <TypographyListItem>Typed Style IR</TypographyListItem>
        <TypographyListItem>Native Taffy layout</TypographyListItem>
      </TypographyList>
      <View class="flex flex-row items-center gap-2">
        <TypographyMuted>Run</TypographyMuted>
        <TypographyInlineCode>bun run test:layout:quick</TypographyInlineCode>
      </View>
    </View>
  );
}
