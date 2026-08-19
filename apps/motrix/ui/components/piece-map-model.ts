const MAX_CELLS = 1_200;

export function decodePieceStates(
  bitfield: string,
  pieceCount: number,
): boolean[] {
  const count = Math.max(0, Math.floor(pieceCount));
  const states = Array.from({ length: count }, (_, index) => {
    const nibble = Number.parseInt(bitfield[index >> 2] ?? "0", 16);
    const bit = 3 - (index & 3);
    return Number.isFinite(nibble) && ((nibble >> bit) & 1) === 1;
  });
  if (states.length <= MAX_CELLS) return states;
  const stride = Math.ceil(states.length / MAX_CELLS);
  const sampled: boolean[] = [];
  for (let offset = 0; offset < states.length; offset += stride) {
    const group = states.slice(offset, offset + stride);
    sampled.push(group.filter(Boolean).length * 2 >= group.length);
  }
  return sampled;
}
