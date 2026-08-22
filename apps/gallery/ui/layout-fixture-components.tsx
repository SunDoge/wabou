import {
  AdaptiveSplitPane,
  AdaptiveSplitPaneDetail,
  AdaptiveSplitPaneMain,
  Button,
  Dialog,
  DialogDescription,
  DialogScrollBody,
  DialogTitle,
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
import { For } from "solid-js";

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
