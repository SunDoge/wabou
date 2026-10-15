import { describe, expect, test } from "vitest";
import {
  documentFromPath,
  filterDocuments,
  gettingStartedDocument,
  titleFromPath,
  upsertDocument,
} from "./reader-model";

describe("reader document model", () => {
  test("derives a readable title on Unix and Windows", () => {
    expect(titleFromPath("/books/The Left Hand of Darkness.pdf")).toBe(
      "The Left Hand of Darkness",
    );
    expect(titleFromPath("C:\\Books\\notes.PDF")).toBe("notes");
  });

  test("reopening a document keeps one stable library identity", () => {
    const first = documentFromPath("/books/paper.pdf", 1);
    const reopened = documentFromPath("/books/paper.pdf", 2);
    expect(upsertDocument([gettingStartedDocument, first], reopened)).toEqual([
      gettingStartedDocument,
      reopened,
    ]);
  });

  test("filters titles and paths without changing the source collection", () => {
    const pdf = documentFromPath("/research/rendering/Parley notes.pdf", 1);
    const documents = [gettingStartedDocument, pdf];
    expect(filterDocuments(documents, "rendering")).toEqual([pdf]);
    expect(documents).toHaveLength(2);
  });
});
