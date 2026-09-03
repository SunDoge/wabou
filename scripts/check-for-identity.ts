import ts from "typescript";

export interface AmbiguousForUsage {
  file: string;
  line: number;
  column: number;
}

export function findAmbiguousForUsage(
  sourceText: string,
  file = "source.tsx",
): AmbiguousForUsage[] {
  const source = ts.createSourceFile(
    file,
    sourceText,
    ts.ScriptTarget.Latest,
    true,
    ts.ScriptKind.TSX,
  );
  const issues: AmbiguousForUsage[] = [];
  const visit = (node: ts.Node) => {
    if (
      (ts.isJsxOpeningElement(node) || ts.isJsxSelfClosingElement(node)) &&
      ts.isIdentifier(node.tagName) &&
      node.tagName.text === "For"
    ) {
      const position = source.getLineAndCharacterOfPosition(
        node.tagName.getStart(source),
      );
      issues.push({
        file,
        line: position.line + 1,
        column: position.character + 1,
      });
    }
    ts.forEachChild(node, visit);
  };
  visit(source);
  return issues;
}

async function sourceFiles(): Promise<string[]> {
  const files: string[] = [];
  for (const root of ["apps", "packages", "tests"]) {
    const glob = new Bun.Glob(`${root}/**/*.tsx`);
    for await (const file of glob.scan({
      cwd: process.cwd(),
      onlyFiles: true,
    })) {
      if (
        file === "packages/core/src/glue/entity-list.tsx" ||
        file.includes("/dist/") ||
        file.includes("/node_modules/")
      ) {
        continue;
      }
      files.push(file);
    }
  }
  return files.sort();
}

async function main(): Promise<void> {
  const issues: AmbiguousForUsage[] = [];
  for (const file of await sourceFiles()) {
    issues.push(...findAmbiguousForUsage(await Bun.file(file).text(), file));
  }
  if (issues.length === 0) return;
  const details = issues
    .map(({ file, line, column }) => `  - ${file}:${line}:${column}`)
    .join("\n");
  throw new Error(
    `Ambiguous Solid <For> identity:\n${details}\nAlias Solid For as ForValue for ordinary values, or use Wabou ForEntity for stateful entities/resources.`,
  );
}

if (import.meta.main) await main();
