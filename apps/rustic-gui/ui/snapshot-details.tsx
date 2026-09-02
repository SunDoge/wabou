import {
  Badge,
  Button,
  Dialog,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogScrollBody,
  DialogTitle,
  Icon,
  Text,
  View,
} from "@wabou/ui";
import info from "lucide-static/icons/info.svg?raw";
import { For } from "solid-js";
import type { SnapshotEntry } from "./api";

export function formatSnapshotTime(value: string): string {
  const match = value.match(/^(\d{4})-(\d{2})-(\d{2})[T ](\d{2}):(\d{2})/);
  return match
    ? `${match[1]}-${match[2]}-${match[3]} ${match[4]}:${match[5]}`
    : value;
}

export function SnapshotDetails(props: { snapshot: SnapshotEntry }) {
  return (
    <Dialog
      aria-label="Snapshot details"
      contentClass="max-h-[640px]"
      trigger={(trigger) => (
        <Button {...trigger} size="sm" variant="ghost">
          <Icon source={info} size={14} /> Details
        </Button>
      )}
    >
      {(dialog) => (
        <View class="min-w-0 flex flex-col gap-4">
          <DialogHeader>
            <DialogTitle>Snapshot details</DialogTitle>
            <DialogDescription>
              Metadata recorded when this backup snapshot was created.
            </DialogDescription>
          </DialogHeader>
          <View class="grid grid-cols-2 gap-4 rounded-lg border border-subtle bg-surface-muted p-4">
            <Detail label="Created" value={props.snapshot.time} />
            <Detail
              label="Hostname"
              value={props.snapshot.hostname || "Unknown"}
            />
            <Detail label="New files" value={String(props.snapshot.filesNew)} />
            <Detail
              label="Changed files"
              value={String(props.snapshot.filesChanged)}
            />
          </View>
          <View class="min-h-0 flex flex-1 flex-col gap-2">
            <View class="flex flex-row items-center justify-between">
              <Text class="text-sm font-medium">Source paths</Text>
              <Badge variant="secondary">{props.snapshot.paths.length}</Badge>
            </View>
            <DialogScrollBody
              class="max-h-48 rounded-lg border border-subtle"
              contentClass="flex flex-col gap-1.5 p-3"
            >
              <For each={props.snapshot.paths}>
                {(path) => (
                  <Text class="whitespace-normal text-sm text-secondary">
                    {path}
                  </Text>
                )}
              </For>
            </DialogScrollBody>
          </View>
          <View class="flex flex-col gap-1">
            <Text class="text-xs font-medium text-muted">Snapshot ID</Text>
            <Text class="whitespace-normal text-xs text-secondary">
              {props.snapshot.id}
            </Text>
          </View>
          <DialogFooter>
            <Button onClick={dialog.close}>Done</Button>
          </DialogFooter>
        </View>
      )}
    </Dialog>
  );
}

function Detail(props: { label: string; value: string }) {
  return (
    <View class="min-w-0 flex flex-col gap-1">
      <Text class="text-xs font-medium text-muted">{props.label}</Text>
      <Text class="whitespace-normal font-medium">{props.value}</Text>
    </View>
  );
}
