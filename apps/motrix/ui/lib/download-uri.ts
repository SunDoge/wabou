export function downloadUris(value: string): string[] {
  return value
    .split(/\r?\n/)
    .map((item) => item.trim())
    .filter((item) => item && !item.startsWith("#"));
}

export function downloadUriError(value: string): string | undefined {
  const uris = downloadUris(value);
  for (const [index, uri] of uris.entries()) {
    let protocol: string;
    try {
      protocol = new URL(uri).protocol;
    } catch {
      return `Line ${index + 1} is not a valid download URI.`;
    }
    if (
      protocol !== "http:" &&
      protocol !== "https:" &&
      protocol !== "magnet:"
    ) {
      return `Line ${index + 1} must use HTTP, HTTPS, or magnet.`;
    }
  }
  return undefined;
}
