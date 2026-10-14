import { resolveWabouUtility, validateWabouUtility } from "../preset";
import type { StyleRule } from "./ir";

function semanticColorDeclaration(
  candidate: string,
  tokens: ReadonlySet<string>,
): StyleRule["declarations"][number] | undefined {
  const match = candidate.match(/^(bg|text|border)-(.+)$/);
  if (!match || !tokens.has(match[2])) return;
  return {
    property:
      match[1] === "bg"
        ? "background-color"
        : match[1] === "text"
          ? "color"
          : "border-color",
    value: { type: "color", value: { kind: "token", name: match[2] } },
  };
}
export function assertSupportedWabouCandidates(
  candidates: Iterable<string>,
  semanticTokens: ReadonlySet<string> = new Set(),
): void {
  const unsupported = [...candidates]
    .filter((candidate) => !semanticColorDeclaration(candidate, semanticTokens))
    .map((candidate) => validateWabouUtility(candidate))
    .filter((diagnostic) => diagnostic !== undefined);
  if (unsupported.length) {
    throw new Error(
      `unsupported Wabou utilities:\n${unsupported
        .map(({ message }) => `  - ${message}`)
        .join("\n")}`,
    );
  }
}

export function compileWabouUtilities(
  candidates: Iterable<string>,
  sourceOrderStart = 0,
  semanticTokens: ReadonlySet<string> = new Set(),
): StyleRule[] {
  const ordered = [...candidates].sort();
  assertSupportedWabouCandidates(ordered, semanticTokens);
  return ordered.map((candidate, index) => {
    const semantic = semanticColorDeclaration(candidate, semanticTokens);
    if (semantic) {
      return {
        className: candidate,
        specificity: 10,
        sourceOrder: sourceOrderStart + index,
        declarations: [semantic],
      };
    }
    const utility = resolveWabouUtility(candidate);
    if (!utility) throw new Error(`unsupported Wabou utility \`${candidate}\``);
    return {
      className: candidate,
      specificity: 10,
      sourceOrder: sourceOrderStart + index,
      declarations: utility.declarations,
    };
  });
}
