import { Menubar, MenubarMenu, Text, View } from "@wabou/ui";
import { createSignal } from "solid-js";
import { Preview } from "../preview";

const fileItems = [
  { id: "new", label: "New window" },
  { id: "open", label: "Open workspace" },
  { id: "save", label: "Save", separatorBefore: true },
] as const;
const editItems = [
  { id: "undo", label: "Undo" },
  { id: "redo", label: "Redo", disabled: true },
  { id: "copy", label: "Copy", separatorBefore: true },
  { id: "paste", label: "Paste" },
] as const;
const viewItems = [
  { id: "command", label: "Command palette" },
  { id: "sidebar", label: "Toggle sidebar" },
] as const;

export function MenubarPage() {
  const [lastAction, setLastAction] = createSignal("No command selected");
  const [openMenu, setOpenMenu] = createSignal<string | null>(null);
  return (
    <View class="flex flex-col gap-5">
      <Preview title="Application menu">
        <View class="flex flex-col items-center gap-3">
          <Menubar
            aria-label="Editor menu"
            value={openMenu()}
            onValueChange={setOpenMenu}
          >
            <MenubarMenu
              value="file"
              label="File"
              items={fileItems}
              onAction={setLastAction}
            />
            <MenubarMenu
              value="edit"
              label="Edit"
              items={editItems}
              onAction={setLastAction}
            />
            <MenubarMenu
              value="view"
              label="View"
              items={viewItems}
              onAction={setLastAction}
            />
            <MenubarMenu value="help" label="Help" items={[]} disabled />
          </Menubar>
          <Text
            role="status"
            aria-label="Last menu command"
            class="text-xs text-muted"
          >
            {lastAction()}
          </Text>
        </View>
      </Preview>
    </View>
  );
}
