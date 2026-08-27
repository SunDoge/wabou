import {
  AdaptiveSplitPane,
  AdaptiveSplitPaneDetail,
  AdaptiveSplitPaneMain,
  AnnotationLayer,
  Bubble,
  BubbleContent,
  Button,
  CodeEditor,
  Dialog,
  DialogDescription,
  DialogScrollBody,
  DialogTitle,
  IconFrame,
  ImageList,
  ImageViewport,
  InputGroup,
  InputGroupAddon,
  InputGroupInput,
  InputGroupText,
  Markdown,
  Message,
  MessageContent,
  MessageGroup,
  MessageScroller,
  QRCode,
  ScrollArea,
  Select,
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarHeader,
  SidebarMenuButton,
  Text,
  View,
} from "@wabou/ui";
import imageIcon from "lucide-static/icons/image.svg?raw";
import { For } from "solid-js";
import { ConversationItem } from "../../pi-agent/ui/conversation";
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

export function PiConversationLayoutFixture() {
  return (
    <View class="w-full h-full min-w-0 p-5 flex flex-col gap-3 bg-canvas">
      <ConversationItem
        item={{
          id: "fixture-assistant",
          kind: "assistant",
          thinkingText: "Inspect the router and verify the active session.",
          text: "## Change\n\nThe active session now remains selected.\n\n- Preserved navigation state\n- Added a regression test",
          streaming: true,
        }}
      />
      <ConversationItem
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

export function CodeEditorLayoutFixture() {
  return (
    <View class="w-full h-full min-w-0 min-h-0 p-4 bg-canvas">
      <CodeEditor
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
          <For each={Array.from({ length: 18 }, (_, index) => index + 1)}>
            {(index) => (
              <SidebarMenuButton aria-label={`Section ${index}`}>
                <Text>{`Section ${index}`}</Text>
              </SidebarMenuButton>
            )}
          </For>
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
        <For each={Array.from({ length: 12 }, (_, index) => index + 1)}>
          {(index) => (
            <View class="h-8 flex-none px-2 flex items-center rounded bg-surface-muted">
              <Text>{`Scrollable row ${index}`}</Text>
            </View>
          )}
        </For>
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
        <Message>
          <View class="w-8 h-8 flex-none rounded-full bg-control" />
          <MessageContent>
            <Bubble variant="destructive">
              <BubbleContent aria-label="Fixture failed message bubble">
                <Text class="whitespace-normal text-sm">
                  Delivery failed. Retry from the action menu.
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
          <For each={Array.from({ length: 12 }, (_, index) => index + 1)}>
            {(index) => <Text>{`Dialog row ${index}`}</Text>}
          </For>
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
