export function join(...values: Array<string | undefined>): string {
  return values.filter(Boolean).join(" ");
}
