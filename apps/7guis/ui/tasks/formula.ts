export type CellValues = Readonly<Record<string, string>>;
export type FormulaValue = number | string;

const CELL = /^[A-Z]+[1-9]\d*$/;

export function columnName(index: number): string {
  let value = index + 1;
  let name = "";
  while (value > 0) {
    value -= 1;
    name = String.fromCharCode(65 + (value % 26)) + name;
    value = Math.floor(value / 26);
  }
  return name;
}

export function cellAddress(row: number, column: number): string {
  return `${columnName(column)}${row + 1}`;
}

function parseAddress(address: string): [number, number] {
  const match = /^([A-Z]+)([1-9]\d*)$/.exec(address);
  if (!match) throw new Error("invalid cell reference");
  let column = 0;
  for (const char of match[1]) column = column * 26 + char.charCodeAt(0) - 64;
  return [Number(match[2]) - 1, column - 1];
}

function range(start: string, end: string): string[] {
  const [startRow, startColumn] = parseAddress(start);
  const [endRow, endColumn] = parseAddress(end);
  const cells: string[] = [];
  for (
    let row = Math.min(startRow, endRow);
    row <= Math.max(startRow, endRow);
    row += 1
  ) {
    for (
      let column = Math.min(startColumn, endColumn);
      column <= Math.max(startColumn, endColumn);
      column += 1
    ) {
      cells.push(cellAddress(row, column));
    }
  }
  return cells;
}

type Token =
  | { kind: "number"; value: number }
  | { kind: "name"; value: string }
  | { kind: "symbol"; value: string }
  | { kind: "eof" };

function tokenize(source: string): Token[] {
  const tokens: Token[] = [];
  let index = 0;
  while (index < source.length) {
    const char = source[index];
    if (/\s/.test(char)) {
      index += 1;
      continue;
    }
    const number = /^(?:\d+(?:\.\d*)?|\.\d+)/.exec(source.slice(index));
    if (number) {
      tokens.push({ kind: "number", value: Number(number[0]) });
      index += number[0].length;
      continue;
    }
    const name = /^[A-Za-z]+\d*/.exec(source.slice(index));
    if (name) {
      tokens.push({ kind: "name", value: name[0].toUpperCase() });
      index += name[0].length;
      continue;
    }
    if ("+-*/():,".includes(char)) {
      tokens.push({ kind: "symbol", value: char });
      index += 1;
      continue;
    }
    throw new Error(`unexpected ${char}`);
  }
  tokens.push({ kind: "eof" });
  return tokens;
}

function numeric(value: FormulaValue): number {
  if (typeof value === "number") return value;
  if (value === "") return 0;
  const number = Number(value);
  if (!Number.isFinite(number)) throw new Error("expected number");
  return number;
}

class Parser {
  private index = 0;
  constructor(
    private readonly tokens: Token[],
    private readonly resolve: (address: string) => FormulaValue,
  ) {}
  parse(): number {
    const value = this.expression();
    if (this.peek().kind !== "eof") throw new Error("unexpected token");
    return value;
  }
  private peek() {
    return this.tokens[this.index] ?? ({ kind: "eof" } as const);
  }
  private take() {
    const token = this.peek();
    this.index += 1;
    return token;
  }
  private symbol(value: string) {
    const token = this.peek();
    if (token.kind === "symbol" && token.value === value) {
      this.index += 1;
      return true;
    }
    return false;
  }
  private expression(): number {
    let value = this.term();
    while (true) {
      if (this.symbol("+")) value += this.term();
      else if (this.symbol("-")) value -= this.term();
      else return value;
    }
  }
  private term(): number {
    let value = this.factor();
    while (true) {
      if (this.symbol("*")) value *= this.factor();
      else if (this.symbol("/")) value /= this.factor();
      else return value;
    }
  }
  private factor(): number {
    if (this.symbol("-")) return -this.factor();
    if (this.symbol("(")) {
      const value = this.expression();
      if (!this.symbol(")")) throw new Error("missing )");
      return value;
    }
    const token = this.take();
    if (token.kind === "number") return token.value;
    if (token.kind !== "name") throw new Error("expected value");
    if (token.value === "SUM" && this.symbol("(")) {
      const start = this.take();
      if (start.kind !== "name" || !CELL.test(start.value))
        throw new Error("expected cell");
      const addresses = this.symbol(":")
        ? (() => {
            const end = this.take();
            if (end.kind !== "name" || !CELL.test(end.value))
              throw new Error("expected cell");
            return range(start.value, end.value);
          })()
        : [start.value];
      if (!this.symbol(")")) throw new Error("missing )");
      return addresses.reduce(
        (sum, address) => sum + numeric(this.resolve(address)),
        0,
      );
    }
    if (!CELL.test(token.value)) throw new Error("unknown name");
    return numeric(this.resolve(token.value));
  }
}

export function evaluateCell(cells: CellValues, address: string): FormulaValue {
  const cache = new Map<string, FormulaValue>();
  const active = new Set<string>();
  const evaluate = (key: string): FormulaValue => {
    const cached = cache.get(key);
    if (cached !== undefined) return cached;
    if (active.has(key)) throw new Error("cycle");
    active.add(key);
    const raw = cells[key] ?? "";
    let value: FormulaValue = raw;
    if (raw.startsWith("="))
      value = new Parser(tokenize(raw.slice(1)), evaluate).parse();
    else if (raw.trim() !== "" && Number.isFinite(Number(raw)))
      value = Number(raw);
    active.delete(key);
    cache.set(key, value);
    return value;
  };
  try {
    return evaluate(address);
  } catch (error) {
    return error instanceof Error && error.message === "cycle"
      ? "#CYCLE!"
      : "#ERR!";
  }
}

export function formatCellValue(value: FormulaValue): string {
  return typeof value === "number"
    ? Number.isFinite(value)
      ? String(Math.round(value * 1e8) / 1e8)
      : "#ERR!"
    : value;
}
