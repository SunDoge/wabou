import { EditorSelection, EditorState } from "@codemirror/state";
import { TreeFragment, type Tree } from "@lezer/common";
import { highlightTree, tagHighlighter, tags } from "@lezer/highlight";
import { parser as jsonParser } from "@lezer/json";

type HighlightRange = {
  from: number;
  to: number;
  classes: string;
};

function highlights(tree: Tree): HighlightRange[] {
  const ranges: HighlightRange[] = [];
  const highlighter = tagHighlighter([
    { tag: tags.propertyName, class: "property" },
    { tag: tags.string, class: "string" },
    { tag: tags.number, class: "number" },
    { tag: tags.bool, class: "boolean" },
    { tag: tags.null, class: "null" },
  ]);
  highlightTree(tree, highlighter, (from, to, classes) => {
    ranges.push({ from, to, classes });
  });
  return ranges;
}

function runExperiment() {
  const started = performance.now();
  let state = EditorState.create({
    doc: '{"enabled": true, "port": 9090}',
  });
  const initialTree = jsonParser.parse(state.doc.toString());
  const selectionBefore = state.selection;
  const port = state.doc.toString().indexOf("9090");
  const transaction = state.update({
    changes: { from: port, to: port + 4, insert: "8080" },
    selection: EditorSelection.range(port, port + 4),
  });
  const changedRanges: {
    fromA: number;
    toA: number;
    fromB: number;
    toB: number;
  }[] = [];
  transaction.changes.iterChangedRanges((fromA, toA, fromB, toB) => {
    changedRanges.push({ fromA, toA, fromB, toB });
  });
  const fragments = TreeFragment.applyChanges(
    TreeFragment.addTree(initialTree),
    changedRanges,
  );
  state = transaction.state;
  const editedTree = jsonParser.parse(state.doc.toString(), fragments);
  const edited = state.doc.toString();
  const editedSelection = {
    anchor: state.selection.main.anchor,
    head: state.selection.main.head,
  };

  const inverse = transaction.changes.invert(transaction.startState.doc);
  const undone = state.update({
    changes: inverse,
    selection: selectionBefore,
  }).state;
  const redone = undone.update({
    changes: transaction.changes,
    selection: transaction.state.selection,
  }).state;
  const ranges = highlights(editedTree);

  // Exercise repeated immutable transactions without involving a DOM view.
  let stress = redone;
  for (let index = 0; index < 1_000; index += 1) {
    const end = stress.doc.length;
    stress = stress.update({ changes: { from: end, insert: " " } }).state;
  }

  return {
    edited,
    editedSelection,
    undoText: undone.doc.toString(),
    redoText: redone.doc.toString(),
    syntaxTreeChanged: initialTree !== editedTree,
    syntaxTreeLength: editedTree.length,
    highlights: ranges,
    stressDocumentLength: stress.doc.length,
    durationMs: performance.now() - started,
  };
}

globalThis.__wabouCodeMirrorExperiment = runExperiment();

declare global {
  var __wabouCodeMirrorExperiment: ReturnType<typeof runExperiment>;
}
