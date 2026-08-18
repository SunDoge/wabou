export function join(...values: Array<string | undefined | false>): string {
  return values.filter(Boolean).join(" ");
}
