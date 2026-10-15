export type ReaderDocument = {
  id: string;
  title: string;
  path?: string;
  kind: "guide" | "pdf";
  addedAt: number;
};

export type ReaderNote = {
  id: string;
  documentId: string;
  page: number;
  body: string;
};

export const gettingStartedDocument: ReaderDocument = {
  id: "getting-started",
  title: "A quieter place to read",
  kind: "guide",
  addedAt: 0,
};

export function titleFromPath(path: string): string {
  const name = path.split(/[\\/]/).at(-1)?.trim() || "Untitled";
  return name.replace(/\.pdf$/i, "") || "Untitled";
}

export function documentFromPath(
  path: string,
  addedAt = Date.now(),
): ReaderDocument {
  return {
    id: `pdf:${path}`,
    title: titleFromPath(path),
    path,
    kind: "pdf",
    addedAt,
  };
}

export function upsertDocument(
  documents: readonly ReaderDocument[],
  document: ReaderDocument,
): ReaderDocument[] {
  const existing = documents.findIndex((entry) => entry.id === document.id);
  if (existing < 0) return [...documents, document];
  return documents.map((entry, index) =>
    index === existing ? document : entry,
  );
}

export function filterDocuments(
  documents: readonly ReaderDocument[],
  query: string,
): ReaderDocument[] {
  const needle = query.trim().toLocaleLowerCase();
  if (!needle) return [...documents];
  return documents.filter((document) =>
    `${document.title} ${document.path ?? ""}`
      .toLocaleLowerCase()
      .includes(needle),
  );
}
