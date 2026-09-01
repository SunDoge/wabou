import { ColorThemeProvider } from "@wabou/core";
import {
  ActivityStatus,
  AdaptiveSplitPane,
  AdaptiveSplitPaneDetail,
  AdaptiveSplitPaneMain,
  Alert,
  AlertActions,
  AlertDescription,
  AlertTitle,
  AnnotationLayer,
  Avatar,
  Bubble,
  BubbleContent,
  Button,
  ButtonGroup,
  ButtonGroupSeparator,
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
  Checkbox,
  ComponentsProvider,
  ContentState,
  createToasts,
  Dialog,
  DialogDescription,
  DialogScrollBody,
  DialogTitle,
  Editor,
  Empty,
  EmptyContent,
  EmptyDescription,
  EmptyHeader,
  EmptyTitle,
  Icon,
  IconFrame,
  ImageList,
  ImageViewport,
  Input,
  InputGroup,
  InputGroupAddon,
  InputGroupInput,
  InputGroupText,
  Kbd,
  LabeledSeparator,
  Listbox,
  Markdown,
  Message,
  MessageActions,
  MessageContent,
  MessageGroup,
  MessageScroller,
  Onboarding,
  OnboardingDescription,
  OnboardingFooter,
  OnboardingHeader,
  OnboardingHeading,
  OnboardingTitle,
  Progress,
  PromptSuggestion,
  PromptSuggestions,
  QRCode,
  RadioGroup,
  RadioGroupItem,
  Reasoning,
  ReasoningContent,
  ReasoningTrigger,
  ResourceBoundary,
  ScrollArea,
  Select,
  Sheet,
  SheetTitle,
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuLabel,
  Slider,
  Spinner,
  Switch,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
  Text,
  Toaster,
  Tool,
  ToolContent,
  ToolHeader,
  ToolInput,
  ToolOutput,
  TypographyH3,
  TypographyP,
  View,
  WorkbenchHeader,
} from "@wabou/ui";
import imageIcon from "lucide-static/icons/image.svg?raw";
import { For as ForValue } from "solid-js";
import { AgentActivityStatus } from "../../pi-agent/ui/agent-activity";
import { initialAgentState } from "../../pi-agent/ui/agent-state";
import { ComposerContextFiles } from "../../pi-agent/ui/composer-context";
import { ComposerDeliveryControl } from "../../pi-agent/ui/composer-delivery";
import { ComposerImages } from "../../pi-agent/ui/composer-images";
import { ConversationItem } from "../../pi-agent/ui/conversation";
import { ExtensionUiChrome } from "../../pi-agent/ui/extension-ui";
import { ModelControls } from "../../pi-agent/ui/model-controls";
import { SessionActions } from "../../pi-agent/ui/session-actions";
import { SessionBehaviorSettings } from "../../pi-agent/ui/session-behavior-settings";
import { TranscriptSearch } from "../../pi-agent/ui/transcript-search";
import { MangaPageMock } from "./pages/image-viewport";
import { MarkdownPreview } from "./pages/markdown";

export function MarkdownInlineLayoutFixture() {
  return (
    <View class="w-80 p-4 bg-canvas">
      <MarkdownPreview source="Before `code` after." />
    </View>
  );
}

export function MarkdownConversationLayoutFixture() {
  return (
    <View class="w-full h-full min-w-0 p-5 flex flex-col gap-3 bg-canvas">
      <Markdown
        variant="conversation"
        aria-label="Conversation Markdown fixture"
        source={
          "## Change\n\nUpdated the request path and kept `healthz` backward compatible.\n\n- Added validation\n- Preserved existing callers\n\n| Check | Result |\n| --- | --- |\n| Types | Passed |\n| Layout | Passed |\n\n```sh\ncargo test -p server\n```"
        }
      />
    </View>
  );
}

export function LabeledSeparatorLayoutFixture() {
  return (
    <View class="w-full h-full min-w-0 p-5 bg-canvas">
      <LabeledSeparator role="group" aria-label="Labeled separator fixture">
        <Text class="text-sm text-muted">
          Worked for 12s · 3 tool calls with a deliberately long label
        </Text>
      </LabeledSeparator>
    </View>
  );
}

export function PiConversationLayoutFixture() {
  return (
    <View
      role="region"
      aria-label="Scrollable Pi conversation"
      class="w-full h-full min-w-0 p-5 flex flex-col gap-3 overflow-y-auto bg-canvas"
    >
      <ConversationItem
        animate={false}
        item={{
          id: "fixture-assistant",
          kind: "assistant",
          thinkingText: "Inspect the router and verify the active session.",
          text: "## Change\n\nThe active session now remains selected.\n\n- Preserved navigation state\n- Added a regression test",
          streaming: true,
        }}
      />
      <ConversationItem
        animate={false}
        item={{
          id: "fixture-tool",
          kind: "tool",
          name: "bash",
          state: "running",
          input: JSON.stringify({ command: "cargo test -p wabou-runtime" }),
          output: "running 245 tests",
        }}
      />
    </View>
  );
}

export function ToolLayoutFixture() {
  return (
    <View class="w-full h-full min-w-0 p-5 bg-canvas">
      <Tool defaultOpen reducedMotion role="group" aria-label="Tool fixture">
        <ToolHeader
          title="mcp__workspace__read_repository_file"
          summary="crates/wabou-runtime/src/gpui_projection_boundary.rs"
          status="running"
        />
        <ToolContent role="region" aria-label="Tool details">
          <ToolInput
            code={'{"path":"crates/wabou-runtime/src/host.rs"}'}
            language="json"
          />
          <ToolOutput code="Loaded 240 lines" language="text" />
        </ToolContent>
      </Tool>
    </View>
  );
}

export function ReasoningLayoutFixture() {
  return (
    <View class="w-full h-full min-w-0 p-5 bg-canvas">
      <Reasoning
        class="max-w-[560px]"
        defaultOpen
        reducedMotion
        role="group"
        aria-label="Reasoning fixture"
      >
        <ReasoningTrigger label="Reasoning" />
        <ReasoningContent role="region" aria-label="Reasoning details">
          <Text class="whitespace-normal text-sm text-secondary">
            Inspect the route state, preserve the active session, then verify
            the result with a focused layout contract.
          </Text>
        </ReasoningContent>
      </Reasoning>
    </View>
  );
}

export function PromptSuggestionLayoutFixture() {
  return (
    <View class="w-full h-full min-w-0 p-5 bg-canvas">
      <PromptSuggestions role="group" aria-label="Prompt suggestion fixture">
        <PromptSuggestion
          title="Review current changes"
          description="Find the highest-risk issue."
        />
        <PromptSuggestion
          title="Run project checks"
          description="Discover the validation commands."
        />
        <PromptSuggestion
          title="Plan a feature"
          description="Turn an idea into an implementation path."
        />
      </PromptSuggestions>
    </View>
  );
}

export function PiModelControlsLayoutFixture() {
  return (
    <View class="w-full h-full min-w-0 p-5 flex items-start bg-canvas">
      <ModelControls
        models={[
          {
            provider: "anthropic",
            id: "claude-sonnet-4-5-with-a-deliberately-long-model-id",
            name: "Claude Sonnet 4.5 with an unusually long display name",
            reasoning: true,
            contextWindow: 200_000,
          },
        ]}
        modelProvider="anthropic"
        modelId="claude-sonnet-4-5-with-a-deliberately-long-model-id"
        thinking="medium"
        thinkingLevels={["off", "medium", "high", "xhigh"]}
        chooseModel={() => {}}
        chooseThinking={() => {}}
      />
    </View>
  );
}

export function PiSessionBehaviorLayoutFixture() {
  return (
    <View class="w-full min-w-0 p-5 bg-canvas">
      <SessionBehaviorSettings
        state={{
          ...initialAgentState,
          connection: "ready",
          autoCompactionEnabled: true,
          steeringMode: "one-at-a-time",
          followUpMode: "all",
        }}
        setAutoCompaction={() => {}}
        setSteeringMode={() => {}}
        setFollowUpMode={() => {}}
      />
    </View>
  );
}

export function PiAgentHeaderLayoutFixture() {
  return (
    <View class="w-full h-12 min-w-0 flex flex-row bg-canvas">
      <SidebarHeader
        aria-label="Pi agent sidebar header"
        class="w-72 flex items-center gap-3 px-4"
      >
        <Text class="font-semibold">Pi Agent</Text>
      </SidebarHeader>
      <WorkbenchHeader
        aria-label="Pi agent content header"
        class="flex-1 justify-between"
      >
        <View class="min-w-0 flex-1 flex flex-col gap-0">
          <Text class="font-semibold truncate">Implement session controls</Text>
          <View class="min-w-0 flex flex-row items-center gap-2">
            <Text class="min-w-0 flex-1 text-xs text-muted truncate">
              Claude Sonnet 4.5 · medium thinking
            </Text>
            <AgentActivityStatus
              state={{
                ...initialAgentState,
                connection: "running",
                activity: { kind: "retrying", attempt: 2, maxAttempts: 3 },
                queue: { steering: 1, followUp: 2 },
              }}
            />
          </View>
        </View>
        <View class="flex-none flex flex-row items-center gap-1">
          <ModelControls
            models={[
              {
                provider: "anthropic",
                id: "claude-sonnet-4-5",
                name: "Claude Sonnet 4.5",
                reasoning: true,
                contextWindow: 200_000,
              },
            ]}
            modelProvider="anthropic"
            modelId="claude-sonnet-4-5"
            thinking="medium"
            thinkingLevels={["off", "medium", "high"]}
            chooseModel={() => {}}
            chooseThinking={() => {}}
          />
          <SessionActions
            compact={() => {}}
            clone={() => {}}
            exportHtml={() => {}}
          />
        </View>
      </WorkbenchHeader>
    </View>
  );
}

export function PiTranscriptSearchLayoutFixture() {
  return (
    <View class="w-full h-full min-w-0 min-h-0 bg-canvas">
      <MessageScroller aria-label="Searchable transcript fixture">
        <TranscriptSearch
          items={[
            { id: "user", kind: "user", text: "Run the cargo tests" },
            {
              id: "tool",
              kind: "tool",
              name: "bash",
              state: "success",
              input: '{"command":"cargo test"}',
              output: "all tests passed",
            },
          ]}
          resolveItem={() => undefined}
          activeChanged={() => {}}
          close={() => {}}
        />
        <View class="p-4">
          <Text>Transcript content</Text>
        </View>
      </MessageScroller>
    </View>
  );
}

export function PiComposerImagesLayoutFixture() {
  return (
    <View class="w-full h-full min-w-0 p-3 bg-canvas">
      <ComposerImages
        paths={[
          "/home/user/screenshots/very-long-diagnostic-screenshot-name.png",
          "/home/user/screenshots/layout.png",
        ]}
        change={() => {}}
      />
    </View>
  );
}

export function PiComposerContextLayoutFixture() {
  return (
    <View class="w-full h-full min-w-0 p-3 bg-canvas">
      <ComposerContextFiles
        paths={[
          "packages/runtime/src/very/deep/module/with-a-long-file-name.ts",
          "crates/wabou-runtime/src/host.rs",
        ]}
        change={() => {}}
      />
    </View>
  );
}

export function PiComposerDeliveryLayoutFixture() {
  return (
    <View class="w-full h-full p-4 flex items-start bg-surface">
      <ComposerDeliveryControl value="followUp" change={() => {}} />
    </View>
  );
}

export function PiExtensionUiLayoutFixture() {
  return (
    <View class="w-full h-full min-w-0 p-3 bg-canvas">
      <View class="w-full min-w-0 rounded-xl border border-strong bg-input p-2 gap-2">
        <ExtensionUiChrome
          placement="aboveEditor"
          statuses={[
            {
              agentId: "agent-1",
              key: "branch",
              text: "Working on a long extension-provided status without overflowing the composer",
            },
          ]}
          widgets={[
            {
              agentId: "agent-1",
              key: "tasks",
              lines: [
                "Indexing workspace files",
                "Checking packages/ui/src/components/markdown.tsx",
              ],
              placement: "aboveEditor",
            },
          ]}
        />
      </View>
    </View>
  );
}

export function EditorLayoutFixture() {
  return (
    <View class="w-full h-full min-w-0 min-h-0 p-4 bg-canvas">
      <Editor
        aria-label="Fixture config editor"
        language="json"
        value={'{\n  "enabled": true,\n  "emoji": "😀"\n}'}
        class="w-full h-full min-w-0 min-h-0 rounded-md border border-strong bg-input text-primary"
      />
    </View>
  );
}

export function SidebarLayoutFixture() {
  return (
    <View
      role="group"
      aria-label="Sidebar fixture boundary"
      class="w-72 h-full min-h-0 p-3 bg-canvas"
    >
      <Sidebar aria-label="Fixture sidebar" class="w-full rounded-lg">
        <SidebarHeader class="h-12 px-3 flex items-center">
          <Text class="font-semibold">Workspace</Text>
        </SidebarHeader>
        <SidebarContent
          role="group"
          aria-label="Fixture navigation"
          contentClass="gap-1"
        >
          <SidebarMenu value="section-4">
            <ForValue
              each={Array.from({ length: 18 }, (_, index) => index + 1)}
            >
              {(index) => (
                <SidebarMenuButton
                  value={`section-${index}`}
                  aria-label={`Section ${index}`}
                >
                  <SidebarMenuLabel>{`Section ${index}`}</SidebarMenuLabel>
                </SidebarMenuButton>
              )}
            </ForValue>
          </SidebarMenu>
        </SidebarContent>
        <SidebarFooter
          role="group"
          aria-label="Fixture sidebar footer"
          class="h-12 px-3 flex items-center"
        >
          <Text>Settings</Text>
        </SidebarFooter>
      </Sidebar>
    </View>
  );
}

export function ScrollAreaLayoutFixture() {
  return (
    <View class="w-full h-full p-4 flex items-start bg-canvas">
      <ScrollArea
        role="region"
        aria-label="Fixture scroll viewport"
        class="w-64 h-40 rounded-lg border border-subtle bg-surface"
        contentClass="p-2 gap-1"
      >
        <ForValue each={Array.from({ length: 12 }, (_, index) => index + 1)}>
          {(index) => (
            <View class="h-8 flex-none px-2 flex items-center rounded bg-surface-muted">
              <Text>{`Scrollable row ${index}`}</Text>
            </View>
          )}
        </ForValue>
      </ScrollArea>
    </View>
  );
}

export function QRCodeLayoutFixture() {
  return (
    <View class="w-full h-full p-6 items-center justify-center bg-canvas">
      <QRCode
        value="https://github.com/SunDoge/wabou"
        aria-label="Fixture QR code"
        size={196}
        class="rounded-lg shadow-md"
      />
    </View>
  );
}

export function IconFrameLayoutFixture() {
  return (
    <View class="w-full h-full p-6 flex items-center justify-center bg-canvas">
      <IconFrame
        source={imageIcon}
        size="lg"
        variant="selected"
        label="Fixture framed icon"
        aria-label="Fixture icon frame"
      />
    </View>
  );
}

export function InputGroupLayoutFixture() {
  return (
    <View class="w-full h-full p-6 items-center justify-center bg-canvas">
      <InputGroup aria-label="Fixture input group" class="w-80">
        <InputGroupAddon aria-label="Fixture scheme addon" align="inline-start">
          <InputGroupText>https://</InputGroupText>
        </InputGroupAddon>
        <InputGroupInput
          aria-label="Fixture hostname input"
          placeholder="example.com"
        />
      </InputGroup>
    </View>
  );
}

export function MessageLayoutFixture() {
  return (
    <View class="w-full p-4">
      <MessageGroup aria-label="Fixture message group">
        <Message aria-label="Fixture failed message">
          <View class="w-8 h-8 flex-none rounded-full bg-control" />
          <MessageContent>
            <Bubble variant="destructive">
              <BubbleContent aria-label="Fixture failed message bubble">
                <Text class="whitespace-normal text-sm">
                  Delivery failed. Retry from the action menu.
                </Text>
              </BubbleContent>
            </Bubble>
            <MessageActions
              visibility="interaction"
              aria-label="Fixture message actions"
            >
              <Button size="sm">Retry</Button>
            </MessageActions>
          </MessageContent>
        </Message>
        <Message aria-label="Fixture following message">
          <MessageContent>
            <Bubble variant="ghost">
              <BubbleContent>
                <Text class="whitespace-normal text-sm">
                  Ready to continue.
                </Text>
              </BubbleContent>
            </Bubble>
          </MessageContent>
        </Message>
      </MessageGroup>
    </View>
  );
}

export function SelectLayoutFixture() {
  return (
    <View class="w-full h-full p-6 items-start bg-canvas">
      <Select
        aria-label="Fixture select"
        defaultOpen
        motion={false}
        defaultValue="second"
        options={[
          { value: "first", label: "First option" },
          { value: "second", label: "Second option" },
          { value: "third", label: "Third option" },
          { value: "disabled", label: "Disabled option", disabled: true },
        ]}
      />
    </View>
  );
}

export function ControlBaselineLayoutFixture() {
  return (
    <View class="w-full h-full p-6 items-start gap-4 bg-canvas">
      <TypographyH3 role="heading" aria-label="Fixture section title">
        Native control baseline
      </TypographyH3>
      <TypographyP>
        Controls share one compact desktop rhythm without local offsets.
      </TypographyP>
      <View class="flex flex-row items-center gap-3">
        <Button size="sm" aria-label="Fixture small button">
          Small
        </Button>
        <Button aria-label="Fixture default button">Default</Button>
        <Button size="lg" aria-label="Fixture large button">
          Large
        </Button>
        <Button
          variant="outline"
          selected
          aria-pressed={true}
          aria-label="Fixture selected button"
        >
          Selected
        </Button>
        <Kbd aria-label="Fixture keyboard key">Ctrl</Kbd>
        <Avatar name="Wabou Project" size="sm" />
      </View>
      <Input
        class="w-72"
        aria-label="Fixture baseline input"
        placeholder="Native text input"
      />
      <View class="w-72">
        <Progress label="Fixture progress" value={40} size="lg" />
      </View>
      <Switch label="Fixture default switch" defaultChecked />
      <Switch label="Fixture compact switch" size="sm" />
      <ButtonGroup
        aria-label="Fixture destructive button group"
        size="lg"
        variant="secondary"
      >
        <Button aria-label="Fixture inherited group action">Keep</Button>
        <ButtonGroupSeparator />
        <Button
          variant="destructive"
          aria-label="Fixture destructive group action"
        >
          Delete
        </Button>
      </ButtonGroup>
    </View>
  );
}

export function SelectionControlsLayoutFixture() {
  return (
    <View class="w-full h-full p-6 items-start gap-5 bg-canvas">
      <Checkbox aria-label="Fixture icon-only checkbox" />
      <Checkbox
        class="w-64"
        label="Keep completed downloads available after the application restarts"
        defaultChecked
      />
      <Checkbox class="w-64" size="lg" label="Disabled selection" disabled />
      <Switch
        class="w-64"
        label="Warn before quitting while active downloads are still running"
        defaultChecked
      />
      <RadioGroup aria-label="Fixture download policy" class="w-64">
        <RadioGroupItem value="system" aria-label="Fixture icon-only radio" />
        <RadioGroupItem
          value="automatic"
          label="Automatically choose the best download policy for this network"
        />
        <RadioGroupItem
          value="manual"
          size="lg"
          label="Choose manually"
          disabled
        />
      </RadioGroup>
    </View>
  );
}

export function TabsLayoutFixture() {
  return (
    <View class="w-full h-full min-w-0 p-6 items-start bg-canvas">
      <Tabs defaultValue="account" class="w-[460px]">
        <TabsList aria-label="Fixture settings sections">
          <TabsTrigger value="account">Account</TabsTrigger>
          <TabsTrigger value="security">Security</TabsTrigger>
          <TabsTrigger value="billing">Billing</TabsTrigger>
        </TabsList>
        <TabsContent value="account">
          <Card role="group" aria-label="Fixture account card">
            <CardContent>
              <Text class="text-sm font-medium text-primary">Account</Text>
              <Text class="whitespace-normal text-sm text-muted">
                Update your public profile and contact details without
                compressing the panel into a narrow column.
              </Text>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </View>
  );
}

export function VerticalTabsLayoutFixture() {
  return (
    <View class="w-full h-full min-w-0 p-6 items-start bg-canvas">
      <Tabs defaultValue="general" orientation="vertical" class="w-[460px]">
        <TabsList aria-label="Fixture vertical settings sections">
          <TabsTrigger value="general">General preferences</TabsTrigger>
          <TabsTrigger value="integrations">Integrations</TabsTrigger>
        </TabsList>
        <TabsContent value="general">
          <Card role="group" aria-label="Fixture vertical account card">
            <CardContent>
              <Text class="text-sm font-medium text-primary">Preferences</Text>
              <Text class="whitespace-normal text-sm text-muted">
                Vertical tabs reserve the remaining width for their active panel
                instead of forcing both columns to occupy the full row.
              </Text>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </View>
  );
}

export function AlertLayoutFixture() {
  return (
    <View class="w-full h-full min-w-0 p-5 items-start bg-canvas">
      <Alert
        variant="destructive"
        size="lg"
        aria-label="Fixture failed native build"
        class="w-full"
        icon={<Icon source={imageIcon} size={16} aria-hidden="true" />}
        onClose={() => {}}
      >
        <AlertTitle>Native build failed</AlertTitle>
        <AlertDescription>
          The linker could not create the application bundle. Review the output
          before retrying the build.
        </AlertDescription>
        <AlertActions aria-label="Fixture recovery actions">
          <Button size="sm" variant="outline">
            Retry build
          </Button>
          <Button size="sm" variant="ghost">
            Open output
          </Button>
        </AlertActions>
      </Alert>
    </View>
  );
}

export function ToastLayoutFixture() {
  const toasts = createToasts({ defaultDuration: 0 });
  toasts.success("Project saved", {
    description:
      "Your changes were written to disk and are ready for the next build.",
    action: { label: "View output", onAction: () => {} },
  });
  return (
    <View class="w-full h-full min-w-0 bg-canvas">
      <Toaster toasts={toasts} placement="top-end" motion={false} />
    </View>
  );
}

export function CardSurfaceLayoutFixture() {
  return (
    <View class="w-full h-full p-6 items-start bg-canvas">
      <Card role="group" aria-label="Fixture card surface" class="w-96">
        <CardHeader aria-label="Fixture card header">
          <CardTitle>Project activity</CardTitle>
          <CardDescription>
            A raised surface keeps its content inside one compact rhythm.
          </CardDescription>
        </CardHeader>
        <CardContent aria-label="Fixture card content">
          <Text>Three tasks completed today.</Text>
        </CardContent>
        <CardFooter aria-label="Fixture card footer">
          <Button size="sm">View details</Button>
        </CardFooter>
      </Card>
    </View>
  );
}

export function DarkSurfaceLayoutFixture() {
  return (
    <ColorThemeProvider theme="dark" transition={false}>
      <ComponentsProvider theme="dark">
        <View class="w-full h-full min-w-0 p-4 gap-3 bg-canvas text-primary">
          <Card role="group" aria-label="Fixture dark card" class="w-full">
            <CardHeader>
              <CardTitle role="heading" aria-label="Fixture dark title">
                Dark surface
              </CardTitle>
              <CardDescription>
                Foreground and muted text must remain distinct from the panel.
              </CardDescription>
            </CardHeader>
            <CardContent>
              <Input aria-label="Fixture dark input" value="Native control" />
            </CardContent>
          </Card>
          <SidebarMenu value="active">
            <SidebarMenuButton value="active" aria-label="Fixture dark item">
              <SidebarMenuLabel>Selected destination</SidebarMenuLabel>
            </SidebarMenuButton>
          </SidebarMenu>
        </View>
      </ComponentsProvider>
    </ColorThemeProvider>
  );
}

export function CompactSurfaceLayoutFixture() {
  return (
    <View class="w-full h-full min-w-0 p-3 gap-3 bg-canvas text-primary">
      <Card role="group" aria-label="Fixture compact card" class="w-full">
        <CardHeader>
          <CardTitle>Compact workspace</CardTitle>
          <CardDescription>
            Component surfaces must shrink without clipping their content.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <Input aria-label="Fixture compact input" value="Compact input" />
          <Button class="w-full">Continue</Button>
        </CardContent>
      </Card>
    </View>
  );
}

export function ListboxLayoutFixture() {
  return (
    <View class="w-full h-full p-6 bg-canvas">
      <Listbox
        aria-label="Fixture branches"
        defaultValue="feature"
        options={[
          { value: "main", label: "Main branch" },
          {
            value: "feature",
            label: "Feature branch",
            description: "Current workspace branch",
          },
          { value: "release", label: "Release branch", disabled: true },
        ]}
      />
    </View>
  );
}

export function EmptyLayoutFixture() {
  return (
    <View class="w-full h-full min-w-0 p-4 bg-canvas">
      <Empty aria-label="Fixture empty state">
        <EmptyHeader aria-label="Fixture empty header">
          <EmptyTitle>No matching projects</EmptyTitle>
          <EmptyDescription>
            Try another search or create a project to continue working from this
            device.
          </EmptyDescription>
        </EmptyHeader>
        <EmptyContent>
          <Button>Create project</Button>
        </EmptyContent>
      </Empty>
    </View>
  );
}

export function SpinnerLayoutFixture() {
  return (
    <View class="w-full h-full flex items-center justify-center bg-canvas">
      <View class="flex items-center gap-2">
        <Spinner label="Fixture loading indicator" />
        <Text class="text-sm text-secondary">Loading</Text>
      </View>
    </View>
  );
}

export function ActivityStatusLayoutFixture() {
  return (
    <View class="w-full h-full min-w-0 p-4 flex items-center bg-canvas">
      <ActivityStatus
        label="Indexing a workspace with a deliberately long descriptive name"
        animated
        class="w-full"
      />
    </View>
  );
}

export function ContentStateLayoutFixture() {
  return (
    <View class="w-full h-full min-w-0 p-3 bg-canvas">
      <ContentState
        state="error"
        title="Could not load this workspace"
        description="Permission denied while reading a deliberately long workspace path that must wrap inside the bounded region."
        action={{ label: "Try again", onAction: () => {} }}
        class="rounded-lg border border-subtle bg-surface"
      />
    </View>
  );
}

export function ResourceBoundaryLayoutFixture() {
  return (
    <View class="w-full h-full min-w-0 p-3 bg-canvas">
      <ResourceBoundary
        loading={false}
        error={new Error("Permission denied while reading this workspace")}
        hasContent={false}
        loadingTitle="Loading workspace files"
        errorTitle="Could not load this workspace"
        emptyTitle="No workspace files"
        retryLabel="Try again"
        onRetry={() => {}}
        renderContent={() => <Text>README.md</Text>}
        class="rounded-lg border border-subtle bg-surface"
      />
    </View>
  );
}

export function OnboardingLayoutFixture() {
  return (
    <View class="w-full h-full min-w-0 bg-canvas">
      <Onboarding aria-label="Fixture onboarding">
        <OnboardingHeader aria-label="Fixture onboarding header">
          <IconFrame
            source={imageIcon}
            size="default"
            variant="muted"
            aria-label="Fixture onboarding icon"
          />
          <OnboardingHeading>
            <OnboardingTitle>
              Prepare a workspace for your first native agent
            </OnboardingTitle>
            <OnboardingDescription>
              Keep the default location or choose another project directory.
            </OnboardingDescription>
          </OnboardingHeading>
        </OnboardingHeader>
        <View
          role="group"
          aria-label="Fixture onboarding task"
          class="w-full min-w-0 rounded-xl border border-subtle bg-surface p-5 gap-4"
        >
          <Text class="whitespace-normal text-sm text-secondary">
            The selected directory remains visible and editable before the
            runtime starts.
          </Text>
          <Button class="w-full" aria-label="Continue">
            Continue
          </Button>
        </View>
        <OnboardingFooter>
          <Text class="whitespace-normal text-xs text-muted">
            You can change this later in settings.
          </Text>
        </OnboardingFooter>
      </Onboarding>
    </View>
  );
}

export function SliderLayoutFixture() {
  return (
    <View class="w-full h-full flex items-center justify-center gap-8 bg-canvas">
      <Slider class="w-96" value={50} label="Fixture volume" />
      <Slider
        orientation="vertical"
        reversed
        value={25}
        label="Fixture remaining"
      />
    </View>
  );
}

export function DialogLayoutFixture() {
  return (
    <View class="w-full h-full bg-canvas">
      <Dialog
        open
        motion={false}
        aria-label="Fixture dialog"
        contentClass="h-72"
      >
        <DialogTitle>Dialog title</DialogTitle>
        <DialogDescription>
          Fixed header and footer surround an independently scrolling body.
        </DialogDescription>
        <DialogScrollBody aria-label="Fixture dialog body" contentClass="gap-2">
          <ForValue each={Array.from({ length: 12 }, (_, index) => index + 1)}>
            {(index) => <Text>{`Dialog row ${index}`}</Text>}
          </ForValue>
        </DialogScrollBody>
        <View
          role="group"
          aria-label="Fixture dialog footer"
          class="flex-none flex justify-end"
        >
          <Button>Done</Button>
        </View>
      </Dialog>
    </View>
  );
}

export function SheetLayoutFixture() {
  return (
    <View class="w-full h-full bg-canvas">
      <Sheet open motion={false} side="right" aria-label="Fixture sheet">
        <SheetTitle>Sheet title</SheetTitle>
        <Text class="whitespace-normal text-sm text-muted">
          The edge panel stays within the viewport and preserves its authored
          width.
        </Text>
      </Sheet>
    </View>
  );
}

export function AdaptiveSplitPaneLayoutFixture() {
  return (
    <View
      role="group"
      aria-label="Adaptive split pane boundary"
      class="w-full h-full min-h-0 p-4 bg-canvas"
    >
      <AdaptiveSplitPane compact={false} class="h-full gap-3">
        <AdaptiveSplitPaneMain class="h-full">
          <View
            role="group"
            aria-label="Fixture split main"
            class="w-full h-full rounded-lg border border-subtle bg-surface"
          >
            <Text>Main</Text>
          </View>
        </AdaptiveSplitPaneMain>
        <AdaptiveSplitPaneDetail
          open
          onOpenChange={() => {}}
          aria-label="Fixture split detail dialog"
          class="w-48 h-full"
        >
          <View
            role="group"
            aria-label="Fixture split detail"
            class="w-full h-full rounded-lg border border-subtle bg-surface"
          >
            <Text>Detail</Text>
          </View>
        </AdaptiveSplitPaneDetail>
      </AdaptiveSplitPane>
    </View>
  );
}

export function ImageViewportLayoutFixture() {
  return (
    <View class="w-full h-full p-4 bg-canvas">
      <ImageViewport
        aria-label="Fixture image viewport"
        class="w-full h-full rounded-lg border border-subtle"
        imageSize={{ width: 800, height: 1200 }}
        media={<MangaPageMock />}
      >
        <AnnotationLayer
          aria-label="Fixture annotation layer"
          regions={[
            {
              id: "speech",
              label: "Fixture speech region",
              x: 80,
              y: 120,
              width: 240,
              height: 180,
            },
          ]}
        />
      </ImageViewport>
    </View>
  );
}

const imageListPages = Array.from({ length: 30 }, (_, index) => ({
  id: index,
  path: `/fixture/page-${index + 1}.png`,
  title: `Fixture page ${index + 1}`,
}));

export function ImageListLayoutFixture() {
  return (
    <View class="w-full h-full min-h-0 p-4 bg-canvas">
      <ImageList
        items={() => imageListPages}
        getItemKey={(page) => page.id}
        getLabel={(page) => page.title}
        renderThumbnail={() => <View class="w-full h-full bg-control" />}
        getDescription={(_, index) =>
          `${index + 1} of ${imageListPages.length}`
        }
        itemHeight={80}
        thumbnailWidth={48}
        thumbnailHeight={64}
        accessibilityLabel="Fixture image list"
        class="w-full h-full border border-subtle bg-surface"
      />
    </View>
  );
}
