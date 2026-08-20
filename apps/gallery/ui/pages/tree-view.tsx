import { Badge, Text, type TreeNode, TreeView, View } from "@wabou/ui";
import { createSignal } from "solid-js";
import { Preview } from "../preview";

const workspace: readonly TreeNode[] = [
  {
    id: "wabou",
    label: "wabou",
    children: [
      {
        id: "packages",
        label: "packages",
        children: [
          { id: "core", label: "core" },
          { id: "ui", label: "ui" },
          { id: "test", label: "test" },
        ],
      },
      {
        id: "apps",
        label: "apps",
        children: [
          { id: "gallery", label: "gallery" },
          { id: "stress", label: "stress" },
        ],
      },
      { id: "target", label: "target (ignored)", disabled: true },
    ],
  },
];

export function TreeViewPage() {
  const [selected, setSelected] = createSignal<string | null>("ui");
  return (
    <View class="flex flex-col gap-5">
      <Preview title="Workspace navigation">
        <View class="w-[360px] flex flex-col gap-3">
          <TreeView
            aria-label="Workspace files"
            items={workspace}
            defaultExpandedIds={["wabou", "packages"]}
            selectedId={selected()}
            onSelectedChange={setSelected}
            class="rounded-lg border border-subtle bg-surface p-2 shadow-sm"
          />
          <View class="flex items-center gap-2">
            <Text class="text-xs text-muted">Selected node</Text>
            <Text role="status" aria-label="Selected tree node" class="text-xs">
              {selected() ?? "None"}
            </Text>
          </View>
        </View>
      </Preview>
      <Preview title="Application-rendered rows">
        <TreeView
          aria-label="Deployment tree"
          items={[
            {
              id: "production",
              label: "Production",
              children: [
                { id: "api", label: "API service" },
                { id: "worker", label: "Background worker" },
              ],
            },
            { id: "preview", label: "Preview" },
          ]}
          defaultExpandedIds={["production"]}
          class="w-[360px] rounded-lg border border-subtle bg-surface p-2"
          renderItem={(node, state) => (
            <View class="min-w-0 flex-1 flex items-center justify-between gap-3">
              <Text maxLines={1} class="min-w-0 text-sm">
                {node.label}
              </Text>
              <Badge variant={state.selected ? "default" : "secondary"}>
                {node.children?.length ? "Group" : "Service"}
              </Badge>
            </View>
          )}
        />
      </Preview>
    </View>
  );
}
