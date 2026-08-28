import { mergeClasses } from "@wabou/core/style";
import { For, Show } from "solid-js";
import { CodeEditor, Text, View, type ViewProps } from "../primitives";
import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from "./disclosure";

export type DiffFileStatus = "added" | "modified" | "deleted" | "renamed";

export interface DiffFile {
  path: string;
  oldPath?: string;
  status: DiffFileStatus;
  additions: number;
  deletions: number;
  /** Unified patch for this file. Kept out of the visible tree while collapsed. */
  patch: string;
}

export interface DiffViewerLabels {
  filesChanged: (count: number) => string;
  additions: (count: number) => string;
  deletions: (count: number) => string;
  empty: string;
  technicalDetails: string;
}

const defaultLabels: DiffViewerLabels = {
  filesChanged: (count) => `${count} file${count === 1 ? "" : "s"} changed`,
  additions: (count) => `${count} addition${count === 1 ? "" : "s"}`,
  deletions: (count) => `${count} deletion${count === 1 ? "" : "s"}`,
  empty: "No code changes.",
  technicalDetails: "Technical diff",
};

export interface DiffViewerProps extends Omit<ViewProps, "children"> {
  files: readonly DiffFile[];
  labels?: Partial<DiffViewerLabels>;
  /** Files opened initially. Details remain opt-in by default. */
  defaultExpanded?: readonly string[];
}

const statusClasses: Record<DiffFileStatus, string> = {
  added: "bg-success-surface text-success-primary",
  modified: "bg-control text-secondary",
  deleted: "bg-danger-surface text-danger-primary",
  renamed: "bg-control text-secondary",
};
const statusLabels: Record<DiffFileStatus, string> = {
  added: "A",
  modified: "M",
  deleted: "D",
  renamed: "R",
};

/**
 * A progressive-disclosure code change viewer.
 *
 * The summary and file metadata are ordinary Wabou components. Unified patch
 * text is mounted only after disclosure and uses the DOM-free CodeMirror
 * document/native editor viewport for selection, copying, and large documents.
 */
export function DiffViewer(props: DiffViewerProps) {
  const labels = (): DiffViewerLabels => ({
    ...defaultLabels,
    ...props.labels,
  });
  const additions = () =>
    props.files.reduce((total, file) => total + file.additions, 0);
  const deletions = () =>
    props.files.reduce((total, file) => total + file.deletions, 0);

  return (
    <View
      role="region"
      aria-label={labels().technicalDetails}
      class={mergeClasses(
        "min-w-0 flex flex-col rounded-xl border border-subtle bg-surface overflow-hidden",
        props.class,
      )}
    >
      <View class="flex-none px-4 py-3 flex flex-row items-center justify-between gap-3 bg-surface-muted">
        <Text class="font-medium">
          {labels().filesChanged(props.files.length)}
        </Text>
        <View class="flex-none flex flex-row items-center gap-2">
          <Text class="text-sm text-success-primary">+{additions()}</Text>
          <Text class="text-sm text-danger-primary">-{deletions()}</Text>
        </View>
      </View>
      <Show
        when={props.files.length > 0}
        fallback={
          <Text role="status" class="p-4 text-sm text-muted">
            {labels().empty}
          </Text>
        }
      >
        <Accordion
          type="multiple"
          defaultValue={props.defaultExpanded ?? []}
          class="min-w-0"
        >
          <For each={props.files}>
            {(file) => (
              <AccordionItem value={file.path} class="px-4">
                <AccordionTrigger aria-label={file.path} class="py-3 gap-3">
                  <View class="min-w-0 flex-1 flex flex-row items-center gap-3">
                    <View
                      aria-hidden="true"
                      class={mergeClasses(
                        "w-6 h-6 flex-none rounded flex items-center justify-center",
                        statusClasses[file.status],
                      )}
                    >
                      <Text class="text-xs font-semibold">
                        {statusLabels[file.status]}
                      </Text>
                    </View>
                    <View class="min-w-0 flex-1 flex flex-col items-start">
                      <Text class="max-w-full truncate text-sm">
                        {file.path}
                      </Text>
                      <Show when={file.oldPath}>
                        <Text class="max-w-full truncate text-xs text-muted">
                          {file.oldPath}
                        </Text>
                      </Show>
                    </View>
                    <View class="flex-none flex flex-row items-center gap-2">
                      <Text
                        aria-label={labels().additions(file.additions)}
                        class="text-xs text-success-primary"
                      >
                        +{file.additions}
                      </Text>
                      <Text
                        aria-label={labels().deletions(file.deletions)}
                        class="text-xs text-danger-primary"
                      >
                        -{file.deletions}
                      </Text>
                    </View>
                  </View>
                </AccordionTrigger>
                <AccordionContent class="min-w-0">
                  <CodeEditor
                    readOnly
                    value={file.patch}
                    aria-label={`${labels().technicalDetails}: ${file.path}`}
                    class="h-64 w-full rounded-lg border border-strong bg-input text-primary"
                  />
                </AccordionContent>
              </AccordionItem>
            )}
          </For>
        </Accordion>
      </Show>
    </View>
  );
}
