/** Keep semantic ID references live for the same lifetime as the popup node. */
export function selectControlsId(
  listboxId: string,
  open: boolean,
): string | undefined {
  return open ? listboxId : undefined;
}
