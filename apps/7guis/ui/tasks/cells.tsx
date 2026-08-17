import { Input } from "@wabou/components";
import type { Handle } from "@wabou/core";
import {
  Button as PrimitiveButton,
  Text,
  translate2d,
  View,
} from "@wabou/primitives";
import { createMemo, createSignal, For } from "solid-js";
import { TaskPage } from "../shared";
import {
  cellAddress,
  columnName,
  evaluateCell,
  formatCellValue,
} from "./formula";

const ROWS = 100;
const COLUMNS = 26;
const CELL_WIDTH = 120;
const CELL_HEIGHT = 34;
const VIEWPORT_HEIGHT = 410;
const RENDERED_ROWS = 16;
const RENDERED_COLUMNS = 12;

interface ScrollEvent {
  scrollX?: number;
  scrollY?: number;
}

export function CellsTask() {
  const [cells, setCells] = createSignal<Record<string, string>>({
    A1: "Quantity",
    B1: "Price",
    C1: "Total",
    A2: "2",
    B2: "12.5",
    C2: "=A2*B2",
    A3: "4",
    B3: "8",
    C3: "=A3*B3",
    C4: "=SUM(C2:C3)",
  });
  const [active, setActive] = createSignal("A1");
  const [scrollX, setScrollX] = createSignal(0);
  const [scrollY, setScrollY] = createSignal(0);
  let viewport: Handle | undefined;
  const startRow = createMemo(() =>
    Math.max(0, Math.floor(scrollY() / CELL_HEIGHT) - 1),
  );
  const startColumn = createMemo(() =>
    Math.max(0, Math.floor(scrollX() / CELL_WIDTH) - 1),
  );
  const visibleRows = createMemo(() =>
    Array.from(
      { length: Math.min(RENDERED_ROWS, ROWS - startRow()) },
      (_, index) => startRow() + index,
    ),
  );
  const visibleColumns = createMemo(() =>
    Array.from(
      { length: Math.min(RENDERED_COLUMNS, COLUMNS - startColumn()) },
      (_, index) => startColumn() + index,
    ),
  );
  const visibleCells = createMemo(() =>
    visibleRows().flatMap((row) =>
      visibleColumns().map((column) => ({
        row,
        column,
        address: cellAddress(row, column),
      })),
    ),
  );
  const raw = () => cells()[active()] ?? "";
  const update = (value: string) =>
    setCells((current) => {
      const next = { ...current };
      if (value === "") delete next[active()];
      else next[active()] = value;
      return next;
    });
  const navigate = (key: string) => {
    const match = /^([A-Z]+)(\d+)$/.exec(active());
    if (!match) return;
    let column = 0;
    for (const char of match[1]) column = column * 26 + char.charCodeAt(0) - 64;
    let row = Number(match[2]);
    if (key === "ArrowLeft") column -= 1;
    else if (key === "ArrowRight") column += 1;
    else if (key === "ArrowUp") row -= 1;
    else if (key === "ArrowDown") row += 1;
    else return;
    column = Math.max(1, Math.min(COLUMNS, column));
    row = Math.max(1, Math.min(ROWS, row));
    const next = `${columnName(column - 1)}${row}`;
    setActive(next);
    const left = (column - 1) * CELL_WIDTH;
    const top = (row - 1) * CELL_HEIGHT;
    if (left < scrollX()) viewport?.scrollTo({ left, top: scrollY() });
    else if (left + CELL_WIDTH > scrollX() + CELL_WIDTH * 8)
      viewport?.scrollTo({ left: left - CELL_WIDTH * 7, top: scrollY() });
    if (top < scrollY()) viewport?.scrollTo({ left: scrollX(), top });
    else if (top + CELL_HEIGHT > scrollY() + VIEWPORT_HEIGHT)
      viewport?.scrollTo({
        left: scrollX(),
        top: top - VIEWPORT_HEIGHT + CELL_HEIGHT,
      });
  };
  return (
    <TaskPage
      number={7}
      title="Cells"
      summary="A 100 × 26 spreadsheet uses two-dimensional virtualization and a small dependency-aware formula evaluator."
    >
      <View class="flex flex-col gap-3">
        <View class="flex items-center gap-2">
          <Text class="w-16 flex-none font-mono text-sm font-semibold text-accent">
            {active()}
          </Text>
          <Input
            aria-label="Cell formula"
            class="flex-1 font-mono"
            value={raw()}
            onInput={(event) => update(event.currentTarget.value)}
            onKeyDown={(event) => {
              if (
                ["ArrowLeft", "ArrowRight", "ArrowUp", "ArrowDown"].includes(
                  event.key,
                )
              )
                return;
            }}
          />
          <Text class="w-40 flex-none truncate text-right font-mono text-sm text-muted">
            {formatCellValue(evaluateCell(cells(), active()))}
          </Text>
        </View>
        <View
          role="grid"
          aria-label="Spreadsheet"
          class="w-full overflow-hidden rounded-lg border border-strong bg-input"
        >
          <View class="h-8 flex flex-none border-b border-strong bg-control">
            <View class="w-12 flex-none flex items-center justify-center border-r border-strong">
              <Text class="text-xs text-muted">fx</Text>
            </View>
            <View class="flex-1 overflow-hidden relative">
              <View
                transform={translate2d(-scrollX(), 0)}
                class="absolute h-8"
                style={{ width: `${COLUMNS * CELL_WIDTH}px` }}
              >
                <For each={visibleColumns()}>
                  {(column) => (
                    <View
                      class="absolute h-8 flex items-center justify-center border-r border-subtle"
                      style={{
                        left: `${column * CELL_WIDTH}px`,
                        width: `${CELL_WIDTH}px`,
                      }}
                    >
                      <Text class="text-xs font-semibold text-secondary">
                        {columnName(column)}
                      </Text>
                    </View>
                  )}
                </For>
              </View>
            </View>
          </View>
          <View class="flex" style={{ height: `${VIEWPORT_HEIGHT}px` }}>
            <View class="w-12 flex-none overflow-hidden relative border-r border-strong bg-control">
              <View
                transform={translate2d(0, -scrollY())}
                class="absolute w-12"
                style={{ height: `${ROWS * CELL_HEIGHT}px` }}
              >
                <For each={visibleRows()}>
                  {(row) => (
                    <View
                      class="absolute w-12 flex items-center justify-center border-b border-subtle"
                      style={{
                        top: `${row * CELL_HEIGHT}px`,
                        height: `${CELL_HEIGHT}px`,
                      }}
                    >
                      <Text class="text-xs font-mono text-muted">
                        {row + 1}
                      </Text>
                    </View>
                  )}
                </For>
              </View>
            </View>
            <View
              ref={(node) => (viewport = node)}
              class="flex-1 overflow-scroll relative"
              scrollbar={{ visibility: "auto", thickness: 8 }}
              onScroll={(event: ScrollEvent) => {
                setScrollX(event.scrollX ?? 0);
                setScrollY(event.scrollY ?? 0);
              }}
            >
              <View
                class="relative"
                style={{
                  width: `${COLUMNS * CELL_WIDTH}px`,
                  height: `${ROWS * CELL_HEIGHT}px`,
                }}
              >
                <For each={visibleCells()}>
                  {(cell) => {
                    const selected = () => active() === cell.address;
                    const displayed = () =>
                      formatCellValue(evaluateCell(cells(), cell.address));
                    return (
                      <PrimitiveButton
                        unstyled
                        role="gridcell"
                        aria-label={`Cell ${cell.address}`}
                        aria-selected={selected()}
                        class="absolute px-2 justify-start border-r border-b text-xs font-mono"
                        style={(state) => ({
                          left: `${cell.column * CELL_WIDTH}px`,
                          top: `${cell.row * CELL_HEIGHT}px`,
                          width: `${CELL_WIDTH}px`,
                          height: `${CELL_HEIGHT}px`,
                          "border-color": selected() ? "#0ea5e9" : "#253144",
                          "background-color": selected()
                            ? "#15395d"
                            : state.hovered
                              ? "#121f2f"
                              : "#0a1019",
                          color: displayed().startsWith("#")
                            ? "#fca5a5"
                            : "#c4cfdd",
                        })}
                        onClick={() => setActive(cell.address)}
                        onKeyDown={(event) => {
                          if (
                            [
                              "ArrowLeft",
                              "ArrowRight",
                              "ArrowUp",
                              "ArrowDown",
                            ].includes(event.key)
                          ) {
                            event.preventDefault();
                            navigate(event.key);
                          }
                        }}
                      >
                        {displayed()}
                      </PrimitiveButton>
                    );
                  }}
                </For>
              </View>
            </View>
          </View>
        </View>
        <Text class="text-xs text-muted">
          Formulas support arithmetic, cell references and SUM ranges. Only
          visible cells are mounted.
        </Text>
      </View>
    </TaskPage>
  );
}
