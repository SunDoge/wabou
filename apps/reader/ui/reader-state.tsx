import { dialog } from "@wabou/ui";
import {
  createContext,
  createMemo,
  createSignal,
  type JSX,
  useContext,
} from "solid-js";
import {
  documentFromPath,
  filterDocuments,
  gettingStartedDocument,
  type ReaderDocument,
  type ReaderNote,
  upsertDocument,
} from "./reader-model";

type ReaderState = ReturnType<typeof createReaderState>;

const ReaderContext = createContext<ReaderState>();

function createReaderState() {
  const [documents, setDocuments] = createSignal<ReaderDocument[]>([
    gettingStartedDocument,
  ]);
  const [activeId, setActiveId] = createSignal(gettingStartedDocument.id);
  const [query, setQuery] = createSignal("");
  const [page, setPage] = createSignal(1);
  const [zoom, setZoom] = createSignal(1);
  const [inspectorOpen, setInspectorOpen] = createSignal(true);
  const [notes, setNotes] = createSignal<ReaderNote[]>([]);
  const [opening, setOpening] = createSignal(false);

  const activeDocument = createMemo(
    () =>
      documents().find((document) => document.id === activeId()) ??
      gettingStartedDocument,
  );
  const visibleDocuments = createMemo(() =>
    filterDocuments(documents(), query()),
  );
  const activeNotes = createMemo(() =>
    notes().filter((note) => note.documentId === activeDocument().id),
  );

  const selectDocument = (id: string) => {
    if (id === activeId()) return;
    setActiveId(id);
    setPage(1);
  };

  const openPdf = async () => {
    setOpening(true);
    try {
      const paths = await dialog.open({
        title: "Open a PDF in Wabou Reader",
        filters: [{ name: "PDF documents", extensions: ["pdf"] }],
      });
      const path = paths?.[0];
      if (!path) return false;
      const document = documentFromPath(path);
      setDocuments((current) => upsertDocument(current, document));
      selectDocument(document.id);
      return true;
    } finally {
      setOpening(false);
    }
  };

  const addNote = (body: string) => {
    const trimmed = body.trim();
    if (!trimmed) return;
    setNotes((current) => [
      ...current,
      {
        id: `note:${Date.now()}:${current.length}`,
        documentId: activeDocument().id,
        page: page(),
        body: trimmed,
      },
    ]);
  };

  return {
    documents,
    visibleDocuments,
    activeDocument,
    activeId,
    selectDocument,
    query,
    setQuery,
    page,
    setPage,
    zoom,
    setZoom,
    inspectorOpen,
    setInspectorOpen,
    notes: activeNotes,
    addNote,
    openPdf,
    opening,
  };
}

export function ReaderProvider(props: { children?: JSX.Element }) {
  const state = createReaderState();
  return <ReaderContext value={state}>{props.children}</ReaderContext>;
}

export function useReader(): ReaderState {
  const state = useContext(ReaderContext);
  if (!state) throw new Error("useReader must be used inside ReaderProvider");
  return state;
}
