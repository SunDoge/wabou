import {
  createTanStackDataTable,
  type TanStackDataTableColumn,
} from "@wabou/ui";
import { createRoot, createSignal, flush } from "solid-js";
import { describe, expect, test } from "vitest";

interface RecordRow {
  id: string;
  name: string;
  score: number;
}

const columns: TanStackDataTableColumn<RecordRow>[] = [
  { accessorKey: "name", header: "Name" },
  { accessorKey: "score", header: "Score" },
];

describe("TanStack Table integration", () => {
  test("keeps filtering, sorting, and selection in Solid-owned state", () => {
    let dispose = () => {};
    const model = createRoot((rootDispose) => {
      dispose = rootDispose;
      return createTanStackDataTable<RecordRow>({
        data: [
          { id: "b", name: "Beta", score: 4 },
          { id: "a", name: "Alpha", score: 9 },
        ],
        columns,
        getRowId: (row) => row.id,
        enableRowSelection: true,
      });
    });

    expect(model.rows().map((row) => row.id)).toEqual(["b", "a"]);

    model.table.getColumn("score")?.toggleSorting(true);
    flush();
    expect(model.sorting()).toEqual([{ id: "score", desc: true }]);
    expect(model.rows().map((row) => row.id)).toEqual(["a", "b"]);

    model.setGlobalFilter("alpha");
    flush();
    expect(model.rows().map((row) => row.id)).toEqual(["a"]);

    model.setGlobalFilter("");
    flush();
    model.setSorting([{ id: "score", desc: false }]);
    flush();
    expect(model.rows().map((row) => row.id)).toEqual(["b", "a"]);

    model.rows()[0]?.toggleSelected(true);
    flush();
    expect(model.rowSelection()).toEqual({ b: true });
    expect(model.selectedCount()).toBe(1);
    dispose();
  });

  test("preserves initial state and respects per-row selection eligibility", () => {
    let dispose = () => {};
    const model = createRoot((rootDispose) => {
      dispose = rootDispose;
      return createTanStackDataTable<RecordRow>({
        data: [
          { id: "a", name: "Alpha", score: 9 },
          { id: "b", name: "Beta", score: 4 },
        ],
        columns,
        getRowId: (row) => row.id,
        initialSorting: [{ id: "score", desc: false }],
        initialGlobalFilter: "beta",
        initialRowSelection: { a: true },
        enableRowSelection: (row) => row.id === "a",
      });
    });

    expect(model.rows().map((row) => row.id)).toEqual(["b"]);
    model.rows()[0]?.toggleSelected(true);
    flush();
    expect(model.rowSelection()).toEqual({ a: true });
    expect(model.selectedCount()).toBe(1);

    model.table.setGlobalFilter("");
    flush();
    expect(model.globalFilter()).toBe("");
    expect(model.rows().map((row) => row.id)).toEqual(["b", "a"]);
    model.rows()[1]?.toggleSelected(false);
    flush();
    expect(model.selectedCount()).toBe(0);
    dispose();
  });

  test("accepts reactive application data", () => {
    let dispose = () => {};
    const state = createRoot((rootDispose) => {
      dispose = rootDispose;
      const [data, setData] = createSignal<readonly RecordRow[]>([
        { id: "a", name: "Alpha", score: 9 },
      ]);
      const model = createTanStackDataTable({
        data,
        columns,
        getRowId: (row) => row.id,
      });
      return { model, setData };
    });

    expect(state.model.rows()).toHaveLength(1);
    state.setData((rows) => [...rows, { id: "b", name: "Beta", score: 4 }]);
    flush();
    expect(state.model.rows().map((row) => row.id)).toEqual(["a", "b"]);
    dispose();
  });
});
