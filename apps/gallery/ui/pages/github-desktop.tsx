import {
  Button,
  Checkbox,
  Editor,
  Icon,
  Input,
  mergeClasses,
  ScrollArea,
  Tabs,
  TabsList,
  TabsTrigger,
  Text,
  TextArea,
  View,
} from "@wabou/ui";
import { Button as PrimitiveButton } from "@wabou/ui/primitives";
import chevronDown from "lucide-static/icons/chevron-down.svg?raw";
import cloudDownload from "lucide-static/icons/cloud-download.svg?raw";
import code from "lucide-static/icons/code-2.svg?raw";
import moreHorizontal from "lucide-static/icons/ellipsis.svg?raw";
import fileCode from "lucide-static/icons/file-code-2.svg?raw";
import repository from "lucide-static/icons/folder-git-2.svg?raw";
import gitBranch from "lucide-static/icons/git-branch.svg?raw";
import listFilter from "lucide-static/icons/list-filter.svg?raw";
import { createSignal, For as ForValue, Show } from "solid-js";

interface ChangedFile {
  path: string;
  directory: string;
  status: "M" | "A";
  additions: number;
  deletions: number;
  patch: string;
}

const changedFiles: readonly ChangedFile[] = [
  {
    path: "github-desktop.tsx",
    directory: "apps/gallery/ui/pages",
    status: "A",
    additions: 214,
    deletions: 0,
    patch: `@@ -0,0 +1,8 @@
+import { Editor, View } from "@wabou/ui";
+
+export function GithubDesktopReference() {
+  return (
+    <View class="h-full">
+      <Editor readOnly language="diff" value={patch} />
+    </View>
+  );
+}`,
  },
  {
    path: "layout-fixture-pages.tsx",
    directory: "apps/gallery/ui",
    status: "M",
    additions: 12,
    deletions: 2,
    patch: `@@ -42,7 +42,8 @@
 const selected = state.value;
-const paneWidth = 280;
+const paneWidth = 320;
 return (
   <Workbench>
+    <RepositoryToolbar />
     <ChangesPane selected={selected} />`,
  },
  {
    path: "ui-reference-apps.md",
    directory: "docs",
    status: "M",
    additions: 9,
    deletions: 1,
    patch: `@@ -18,4 +18,6 @@ Native examples
-The gallery includes component examples.
+The gallery includes complete native application references.
+Git workbench demonstrates the shared read-only Editor.
 Headless layout fixtures protect their geometry.`,
  },
];

const includedByDefault = new Set(changedFiles.map((file) => file.path));

export function GithubDesktopReference(props: { class?: string }) {
  const [tab, setTab] = createSignal("changes");
  const [selected, setSelected] = createSignal(changedFiles[0]?.path ?? "");
  const [included, setIncluded] =
    createSignal<ReadonlySet<string>>(includedByDefault);
  const [summary, setSummary] = createSignal("Add GitHub Desktop reference");
  const selectedFile = () =>
    changedFiles.find((file) => file.path === selected()) ?? changedFiles[0];
  const toggleFile = (path: string, checked: boolean) => {
    setIncluded((current) => {
      const next = new Set(current);
      if (checked) next.add(path);
      else next.delete(path);
      return next;
    });
  };

  return (
    <View
      role="region"
      aria-label="GitHub Desktop reference workbench"
      class={mergeClasses(
        "w-full h-full min-w-0 min-h-0 overflow-hidden rounded-lg border border-strong bg-canvas shadow-lg flex flex-col",
        props.class,
      )}
    >
      <View
        role="toolbar"
        aria-label="Repository controls"
        class="h-16 flex-none flex flex-row border-b border-strong bg-surface"
      >
        <View class="w-80 flex-none px-4 flex flex-row items-center gap-3 border-r border-subtle">
          <View class="w-8 h-8 rounded-md bg-accent flex items-center justify-center">
            <Icon source={repository} size={18} class="text-on-accent" />
          </View>
          <View class="min-w-0 flex-1 gap-0">
            <Text class="text-xs text-muted">Current repository</Text>
            <Text class="truncate text-sm font-semibold">wabou</Text>
          </View>
          <Icon source={chevronDown} size={14} class="text-muted" />
        </View>
        <View class="w-64 flex-none px-4 flex flex-row items-center gap-3 border-r border-subtle">
          <Icon source={gitBranch} size={17} class="text-secondary" />
          <View class="min-w-0 flex-1 gap-0">
            <Text class="text-xs text-muted">Current branch</Text>
            <Text class="truncate text-sm font-semibold">
              feat/electron-reference
            </Text>
          </View>
          <Icon source={chevronDown} size={14} class="text-muted" />
        </View>
        <View class="min-w-0 flex-1 px-5 flex flex-row items-center justify-between gap-4">
          <Button variant="ghost" class="gap-2">
            <Icon source={cloudDownload} size={17} />
            <View class="items-start gap-0">
              <Text class="text-sm font-semibold">Fetch origin</Text>
              <Text class="text-xs text-muted">Last fetched just now</Text>
            </View>
          </Button>
          <Button variant="ghost" size="icon" aria-label="Repository menu">
            <Icon source={moreHorizontal} size={18} />
          </Button>
        </View>
      </View>

      <View class="min-w-0 min-h-0 flex-1 flex flex-row">
        <View class="w-80 flex-none min-h-0 flex flex-col border-r border-strong bg-surface">
          <Tabs
            value={tab()}
            onValueChange={setTab}
            class="min-h-0 flex-1 gap-0"
          >
            <TabsList
              variant="line"
              aria-label="Repository activity"
              class="h-12 px-3 flex-none border-b border-subtle"
            >
              <TabsTrigger value="changes" class="flex-1">
                Changes (3)
              </TabsTrigger>
              <TabsTrigger value="history" class="flex-1">
                History
              </TabsTrigger>
            </TabsList>
            <Show
              when={tab() === "changes"}
              fallback={
                <View class="flex-1 items-center justify-center p-6">
                  <Text class="text-sm text-muted">
                    No history in this reference.
                  </Text>
                </View>
              }
            >
              <View class="h-11 flex-none px-3 flex flex-row items-center gap-2 border-b border-subtle">
                <Checkbox
                  aria-label="Include all changed files"
                  checked={included().size === changedFiles.length}
                  indeterminate={
                    included().size > 0 && included().size < changedFiles.length
                  }
                  onCheckedChange={(checked) =>
                    setIncluded(
                      checked ? new Set(includedByDefault) : new Set<string>(),
                    )
                  }
                />
                <Text class="min-w-0 flex-1 text-sm font-medium">
                  {changedFiles.length} changed files
                </Text>
                <Button size="icon" variant="ghost" aria-label="Filter files">
                  <Icon source={listFilter} size={15} />
                </Button>
              </View>
              <ScrollArea
                role="region"
                aria-label="Changed files"
                class="min-h-0 flex-1"
                contentClass="gap-0"
              >
                <ForValue each={changedFiles} keyed={(file) => file.path}>
                  {(file) => (
                    <PrimitiveButton
                      unstyled
                      aria-label={`Open ${file().path}`}
                      aria-selected={selected() === file().path}
                      selected={selected() === file().path}
                      class={(state) =>
                        mergeClasses(
                          "w-full h-14 px-3 flex flex-row items-center gap-2 border-b border-subtle text-left",
                          state.selected
                            ? "bg-selected"
                            : state.hovered && "bg-control-hover",
                        )
                      }
                      onClick={() => setSelected(file().path)}
                    >
                      <Checkbox
                        aria-label={`Include ${file().path}`}
                        checked={included().has(file().path)}
                        onCheckedChange={(checked) =>
                          toggleFile(file().path, checked)
                        }
                      />
                      <Icon
                        source={fileCode}
                        size={16}
                        class="flex-none text-muted"
                      />
                      <View class="min-w-0 flex-1 items-start gap-0">
                        <Text class="max-w-full truncate text-sm font-medium">
                          {file().path}
                        </Text>
                        <Text class="max-w-full truncate text-xs text-muted">
                          {file().directory}
                        </Text>
                      </View>
                      <Text
                        class={mergeClasses(
                          "flex-none text-xs font-semibold",
                          file().status === "A"
                            ? "text-success-primary"
                            : "text-secondary",
                        )}
                      >
                        {file().status}
                      </Text>
                    </PrimitiveButton>
                  )}
                </ForValue>
              </ScrollArea>
              <View
                role="group"
                aria-label="Commit changes"
                class="flex-none p-3 gap-2 border-t border-strong bg-surface"
              >
                <Input
                  aria-label="Commit summary"
                  value={summary()}
                  onInput={(event) => setSummary(event.currentTarget.value)}
                  placeholder="Summary (required)"
                />
                <TextArea
                  aria-label="Commit description"
                  class="h-16"
                  placeholder="Description"
                />
                <View class="min-w-0 flex flex-row items-center justify-between gap-2">
                  <Text role="status" class="text-xs text-muted">
                    {included().size} of {changedFiles.length} files selected
                  </Text>
                  <Text class="text-xs text-muted">Local commit</Text>
                </View>
                <Button
                  class="w-full"
                  aria-label="Commit to feat/electron-reference"
                  disabled={!summary().trim() || included().size === 0}
                >
                  Commit to feat/electron-reference
                </Button>
              </View>
            </Show>
          </Tabs>
        </View>

        <View class="min-w-0 min-h-0 flex-1 flex flex-col bg-canvas">
          <View class="h-14 flex-none px-5 flex flex-row items-center justify-between gap-4 border-b border-subtle bg-surface">
            <View class="min-w-0 flex-1 flex flex-row items-center gap-2">
              <Icon source={code} size={16} class="flex-none text-muted" />
              <Text class="truncate text-sm font-semibold">
                {selectedFile()?.directory}/{selectedFile()?.path}
              </Text>
            </View>
            <View class="flex-none flex flex-row items-center gap-3">
              <Text class="text-xs text-success-primary">
                +{selectedFile()?.additions ?? 0}
              </Text>
              <Text class="text-xs text-danger-primary">
                −{selectedFile()?.deletions ?? 0}
              </Text>
              <Button size="sm" variant="outline">
                Unified
                <Icon source={chevronDown} size={13} />
              </Button>
            </View>
          </View>
          <View
            role="region"
            aria-label={`Selected file diff: ${selectedFile()?.path ?? "none"}`}
            class="min-w-0 min-h-0 flex-1 p-5"
          >
            <Editor
              readOnly
              language="diff"
              value={selectedFile()?.patch ?? ""}
              aria-label={`Diff editor: ${selectedFile()?.path ?? "none"}`}
              class="w-full h-full min-w-0 min-h-0 rounded-lg border border-strong bg-input text-primary shadow-xs"
            />
          </View>
        </View>
      </View>
    </View>
  );
}

export function GithubDesktopReferencePage() {
  return <GithubDesktopReference class="h-[620px]" />;
}
