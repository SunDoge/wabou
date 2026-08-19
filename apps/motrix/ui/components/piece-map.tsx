import {
  createMeasuredSize,
  Path,
  PathBuilder,
  px,
  Text,
  View,
} from "@wabou/ui";
import { createMemo, Show } from "solid-js";
import { formatBytes } from "../lib/format";
import { decodePieceStates } from "./piece-map-model";

const CELL_SIZE = 4;
const CELL_GAP = 2;
function appendCell(path: PathBuilder, x: number, y: number) {
  path
    .moveTo(x, y)
    .lineTo(x + CELL_SIZE, y)
    .lineTo(x + CELL_SIZE, y + CELL_SIZE)
    .lineTo(x, y + CELL_SIZE)
    .close();
}

export function PieceMap(props: {
  bitfield: string;
  pieceCount: number;
  pieceLength: number;
}) {
  const measured = createMeasuredSize();
  const states = createMemo(() =>
    decodePieceStates(props.bitfield, props.pieceCount),
  );
  const model = createMemo(() => {
    const width = Math.max(1, measured.width() || 248);
    const columns = Math.max(1, Math.floor(width / (CELL_SIZE + CELL_GAP)));
    const done = new PathBuilder();
    const pending = new PathBuilder();
    let doneCount = 0;
    states().forEach((complete, index) => {
      const x = (index % columns) * (CELL_SIZE + CELL_GAP);
      const y = Math.floor(index / columns) * (CELL_SIZE + CELL_GAP);
      appendCell(complete ? done : pending, x, y);
      if (complete) doneCount++;
    });
    const rows = Math.max(1, Math.ceil(states().length / columns));
    return {
      done: done.build({ fill: 0x22c55eff }),
      pending: pending.build({ fill: 0x94a3b866 }),
      height: rows * (CELL_SIZE + CELL_GAP) - CELL_GAP,
      doneCount,
    };
  });

  return (
    <View class="flex flex-col gap-3">
      <Show
        when={props.pieceCount > 0}
        fallback={
          <Text class="py-6 text-center text-sm text-muted">
            Piece information is not available for this task.
          </Text>
        }
      >
        <View
          ref={measured.ref}
          role="img"
          aria-label={`${model().doneCount} of ${states().length} displayed piece groups complete; ${props.pieceCount} total pieces`}
          class="relative w-full overflow-hidden rounded-lg bg-control"
          style={{ height: px(model().height) }}
        >
          <Path
            class="absolute inset-0 w-full h-full"
            source={model().pending}
          />
          <Path class="absolute inset-0 w-full h-full" source={model().done} />
        </View>
        <View class="flex items-center justify-between">
          <View class="flex items-center gap-3">
            <View class="flex items-center gap-1">
              <View class="w-2 h-2 rounded-sm bg-success-primary" />
              <Text class="text-xs text-muted">Complete</Text>
            </View>
            <View class="flex items-center gap-1">
              <View class="w-2 h-2 rounded-sm bg-muted" />
              <Text class="text-xs text-muted">Pending</Text>
            </View>
          </View>
          <Text class="text-xs text-muted">
            {props.pieceCount} × {formatBytes(props.pieceLength)}
          </Text>
        </View>
      </Show>
    </View>
  );
}
