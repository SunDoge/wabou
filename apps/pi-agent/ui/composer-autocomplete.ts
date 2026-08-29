import type { AgentCommand } from "./agent-state";

export const COMPOSER_AUTOCOMPLETE_LIMIT = 12;

export type ComposerTriggerKind = "command" | "file";

export interface ComposerTrigger {
  kind: ComposerTriggerKind;
  query: string;
  start: number;
  end: number;
}

export type ComposerAutocompleteRow =
  | { kind: "command"; id: string; label: string; description: string }
  | { kind: "file"; id: string; label: string; description: string };

/** Detect the token ending at a JavaScript UTF-16 cursor offset. */
export function detectComposerTrigger(
  text: string,
  cursor: number,
): ComposerTrigger | null {
  const end = Math.max(0, Math.min(cursor, text.length));
  const lineStart = text.lastIndexOf("\n", end - 1) + 1;
  const linePrefix = text.slice(lineStart, end);
  if (linePrefix.startsWith("/")) {
    const query = linePrefix.slice(1);
    if (/\s/u.test(query)) return null;
    return { kind: "command", query, start: lineStart, end };
  }

  let tokenStart = end;
  while (tokenStart > 0 && !/\s/u.test(text[tokenStart - 1] ?? "")) {
    tokenStart--;
  }
  const token = text.slice(tokenStart, end);
  if (!token.startsWith("@")) return null;
  return { kind: "file", query: token.slice(1), start: tokenStart, end };
}

function matchesQuery(value: string, query: string): boolean {
  const terms = query.trim().toLocaleLowerCase().split(/\s+/u).filter(Boolean);
  const candidate = value.toLocaleLowerCase();
  return terms.every((term) => candidate.includes(term));
}

export function composerAutocompleteRows(
  trigger: ComposerTrigger,
  commands: readonly AgentCommand[],
  files: readonly string[],
  limit = COMPOSER_AUTOCOMPLETE_LIMIT,
): ComposerAutocompleteRow[] {
  const rows: ComposerAutocompleteRow[] =
    trigger.kind === "command"
      ? commands.map((command) => ({
          kind: "command",
          id: `command:${command.name}`,
          label: `/${command.name}`,
          description: command.description ?? command.source,
        }))
      : files.map((path) => ({
          kind: "file",
          id: `file:${path}`,
          label: path,
          description: "Workspace file",
        }));
  return rows
    .filter((row) =>
      matchesQuery(`${row.label} ${row.description}`, trigger.query),
    )
    .slice(0, limit);
}

export function replaceComposerTrigger(
  text: string,
  trigger: ComposerTrigger,
  row: ComposerAutocompleteRow,
): { text: string; cursor: number } {
  const inserted = row.kind === "command" ? `${row.label} ` : `@${row.label} `;
  return {
    text: `${text.slice(0, trigger.start)}${inserted}${text.slice(trigger.end)}`,
    cursor: trigger.start + inserted.length,
  };
}
