import { expect, test } from "bun:test";
import {
  cellAddress,
  columnName,
  evaluateCell,
  formatCellValue,
} from "./formula";

test("names spreadsheet columns", () => {
  expect(columnName(0)).toBe("A");
  expect(columnName(25)).toBe("Z");
  expect(columnName(26)).toBe("AA");
  expect(cellAddress(9, 1)).toBe("B10");
});

test("evaluates arithmetic, references and ranges", () => {
  const cells = { A1: "2", A2: "3", B1: "=A1*A2+1", B2: "=SUM(A1:B1)" };
  expect(formatCellValue(evaluateCell(cells, "B1"))).toBe("7");
  expect(formatCellValue(evaluateCell(cells, "B2"))).toBe("9");
});

test("reports cycles and invalid formulas", () => {
  expect(evaluateCell({ A1: "=B1", B1: "=A1" }, "A1")).toBe("#CYCLE!");
  expect(evaluateCell({ A1: "=oops" }, "A1")).toBe("#ERR!");
});
