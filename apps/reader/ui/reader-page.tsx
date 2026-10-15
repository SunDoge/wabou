import { PdfView } from "@wabou/pdf";
import {
  Badge,
  Button,
  Icon,
  Row,
  ScrollArea,
  Text,
  TextArea,
  View,
  WorkbenchContent,
  WorkbenchHeader,
  WorkbenchInspector,
  WorkbenchInspectorContent,
  WorkbenchInspectorTitlebar,
} from "@wabou/ui";
import chevronLeft from "lucide-static/icons/chevron-left.svg?raw";
import chevronRight from "lucide-static/icons/chevron-right.svg?raw";
import highlighter from "lucide-static/icons/highlighter.svg?raw";
import panelRight from "lucide-static/icons/panel-right.svg?raw";
import stickyNote from "lucide-static/icons/sticky-note.svg?raw";
import zoomIn from "lucide-static/icons/zoom-in.svg?raw";
import zoomOut from "lucide-static/icons/zoom-out.svg?raw";
import { createSignal, For as ForValue, Show } from "solid-js";
import { useReader } from "./reader-state";

function GuidePage() {
  return (
    <ScrollArea class="w-full h-full" contentClass="px-10 py-12">
      <View class="w-full max-w-3xl mx-auto px-16 py-14 bg-[#fffefa] border border-[#d7d1c5] shadow-lg">
        <Text class="text-xs font-medium tracking-widest text-accent">
          WABOU READER · FIELD NOTE 01
        </Text>
        <Text class="mt-7 text-3xl font-semibold tracking-tight">
          A quieter place to read
        </Text>
        <Text class="mt-3 text-sm text-muted">
          A short guide to the reading workspace
        </Text>
        <View class="w-14 h-px mt-8 mb-8 bg-accent" />
        <Text
          class="text-base text-secondary"
          style={{ "line-height": "28px" }}
        >
          Reading software should keep the document at the center. The library
          stays close, notes keep their connection to a page, and the
          application remembers where the work stopped without turning every
          action into a mode.
        </Text>
        <Text class="mt-7 text-lg font-semibold">
          Start with a real document
        </Text>
        <Text
          class="mt-3 text-base text-secondary"
          style={{ "line-height": "28px" }}
        >
          Open a PDF from the sidebar. Wabou parses and rasterizes it in Rust on
          a worker thread, while this surrounding workspace remains reactive
          Solid code that can reload during development.
        </Text>
        <View class="my-8 px-5 py-4 border-l border-accent bg-[#f6f1e9]">
          <Text
            class="text-sm text-secondary"
            style={{ "line-height": "24px" }}
          >
            The useful boundary is visible here: document rendering and native
            resources belong to Rust; navigation, reading controls, notes, and
            product iteration stay in TSX.
          </Text>
        </View>
        <Text class="text-lg font-semibold">
          What this app should prove next
        </Text>
        <Text
          class="mt-3 text-base text-secondary"
          style={{ "line-height": "28px" }}
        >
          Persistent reading positions, searchable text, EPUB pagination,
          highlights anchored to document locations, and a library that survives
          restarts. Each feature improves the product while testing a real
          framework boundary.
        </Text>
      </View>
    </ScrollArea>
  );
}

function NotesInspector() {
  const reader = useReader();
  const [draft, setDraft] = createSignal("");
  const submit = () => {
    reader.addNote(draft());
    setDraft("");
  };
  return (
    <WorkbenchInspector class="w-80">
      <WorkbenchInspectorTitlebar
        title="Notes"
        description={reader.activeDocument().title}
        closeLabel="Close notes"
        onClose={() => reader.setInspectorOpen(false)}
      />
      <WorkbenchInspectorContent>
        <ScrollArea class="min-h-0 flex-1" contentClass="p-4 gap-3">
          <Show
            when={reader.notes().length > 0}
            fallback={
              <View class="px-3 py-8 flex flex-col items-center text-center">
                <View class="w-9 h-9 rounded-full bg-control flex items-center justify-center">
                  <Icon source={stickyNote} size={16} class="text-muted" />
                </View>
                <Text class="mt-3 text-sm font-medium">
                  No notes on this document
                </Text>
                <Text
                  class="mt-1 text-xs text-muted"
                  style={{ "line-height": "20px" }}
                >
                  Capture a thought without leaving the page.
                </Text>
              </View>
            }
          >
            <ForValue each={reader.notes()}>
              {(note) => (
                <View class="p-3 rounded-lg border border-subtle bg-surface-muted">
                  <Row class="items-center gap-2">
                    <Badge variant="outline" size="sm">
                      Page {note.page}
                    </Badge>
                    <Text class="text-xs text-muted">Note</Text>
                  </Row>
                  <Text
                    class="mt-3 text-sm text-secondary"
                    style={{ "line-height": "24px" }}
                  >
                    {note.body}
                  </Text>
                </View>
              )}
            </ForValue>
          </Show>
        </ScrollArea>
        <View class="p-3 flex-none border-t border-subtle bg-surface">
          <TextArea
            aria-label="New note"
            class="w-full h-24"
            placeholder="Write a note for this page…"
            value={draft()}
            onInput={(event) => setDraft(event.currentTarget.value)}
          />
          <Row class="mt-2 items-center">
            <Text class="flex-1 text-xs text-muted">Page {reader.page()}</Text>
            <Button size="sm" disabled={!draft().trim()} onClick={submit}>
              Add note
            </Button>
          </Row>
        </View>
      </WorkbenchInspectorContent>
    </WorkbenchInspector>
  );
}

export function ReaderPage() {
  const reader = useReader();
  const document = reader.activeDocument;
  const isPdf = () => document().kind === "pdf";
  const previousPage = () => reader.setPage((page) => Math.max(1, page - 1));
  const nextPage = () => reader.setPage((page) => page + 1);
  return (
    <View class="w-full h-full min-w-0 flex flex-row overflow-hidden">
      <View class="min-w-0 h-full flex-1 flex flex-col overflow-hidden">
        <WorkbenchHeader class="px-3 bg-surface-muted">
          <View class="min-w-0 flex-1">
            <Text class="truncate text-sm font-medium">{document().title}</Text>
            <Text class="truncate text-xs text-muted">
              {document().kind === "pdf" ? document().path : "Built-in guide"}
            </Text>
          </View>
          <Row class="items-center gap-1">
            <Button
              variant="ghost"
              size="icon"
              aria-label="Previous page"
              disabled={!isPdf() || reader.page() <= 1}
              onClick={previousPage}
            >
              <Icon source={chevronLeft} size={15} />
            </Button>
            <Text class="w-16 text-center text-xs text-secondary">
              Page {reader.page()}
            </Text>
            <Button
              variant="ghost"
              size="icon"
              aria-label="Next page"
              disabled={!isPdf()}
              onClick={nextPage}
            >
              <Icon source={chevronRight} size={15} />
            </Button>
          </Row>
          <View class="w-px h-5 mx-1 bg-subtle" />
          <Row class="items-center gap-1">
            <Button
              variant="ghost"
              size="icon"
              aria-label="Zoom out"
              disabled={!isPdf()}
              onClick={() =>
                reader.setZoom((zoom) => Math.max(0.6, zoom - 0.1))
              }
            >
              <Icon source={zoomOut} size={15} />
            </Button>
            <Text class="w-12 text-center text-xs text-secondary">
              {Math.round(reader.zoom() * 100)}%
            </Text>
            <Button
              variant="ghost"
              size="icon"
              aria-label="Zoom in"
              disabled={!isPdf()}
              onClick={() => reader.setZoom((zoom) => Math.min(2, zoom + 0.1))}
            >
              <Icon source={zoomIn} size={15} />
            </Button>
          </Row>
          <Button
            variant="ghost"
            size="icon"
            aria-label="Highlight selection"
            disabled
          >
            <Icon source={highlighter} size={15} />
          </Button>
          <Button
            variant={reader.inspectorOpen() ? "secondary" : "ghost"}
            size="icon"
            aria-label={reader.inspectorOpen() ? "Close notes" : "Open notes"}
            onClick={() => reader.setInspectorOpen((open) => !open)}
          >
            <Icon source={panelRight} size={15} />
          </Button>
        </WorkbenchHeader>
        <WorkbenchContent class="bg-[#d8d5ce]">
          <Show
            when={document().kind === "pdf" && document().path}
            fallback={<GuidePage />}
          >
            {(path) => (
              <ScrollArea
                class="w-full h-full"
                contentClass="min-w-full min-h-full p-6 flex items-center justify-center"
              >
                <View
                  class="flex-none"
                  style={{
                    width: `${reader.zoom() * 100}%`,
                    height: `${reader.zoom() * 100}%`,
                  }}
                >
                  <PdfView
                    path={path()}
                    page={reader.page()}
                    renderScale={Math.max(1.5, reader.zoom() * 2)}
                    class="w-full h-full"
                    aria-label={`${document().title}, page ${reader.page()}`}
                  />
                </View>
              </ScrollArea>
            )}
          </Show>
        </WorkbenchContent>
        <View class="h-7 px-4 flex-none flex flex-row items-center border-t border-subtle bg-surface-muted">
          <Text class="flex-1 text-xs text-muted">
            Local document · rendered by Wabou PDF
          </Text>
          <Text class="text-xs text-muted">{reader.notes().length} notes</Text>
        </View>
      </View>
      <Show when={reader.inspectorOpen()}>
        <NotesInspector />
      </Show>
    </View>
  );
}
