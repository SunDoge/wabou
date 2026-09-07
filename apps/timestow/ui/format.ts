export function formatBytes(bytes?: number): string {
  if (bytes === undefined) return "—";
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 ** 2) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 ** 3) return `${(bytes / 1024 ** 2).toFixed(1)} MB`;
  return `${(bytes / 1024 ** 3).toFixed(1)} GB`;
}

export function formatTimestamp(value: string): string {
  const match = value.match(/^(\d{4})-(\d{2})-(\d{2})[T ](\d{2}):(\d{2})/);
  return match
    ? `${match[1]}-${match[2]}-${match[3]} ${match[4]}:${match[5]}`
    : value;
}

export function formatDetailedTimestamp(value: string): string {
  const match = value.match(
    /^(\d{4})-(\d{2})-(\d{2})[T ](\d{2}):(\d{2})(?::(\d{2})(?:\.\d+)?)?(Z|[+-]\d{2}:?\d{2})?$/,
  );
  if (!match) return value;

  const [, year, month, day, hour, minute, second = "00", rawZone] = match;
  const zone = formatTimeZone(rawZone);
  return `${year}-${month}-${day} ${hour}:${minute}:${second}${zone}`;
}

function formatTimeZone(rawZone?: string): string {
  if (!rawZone) return "";
  if (rawZone === "Z") return " UTC";
  const offset = rawZone.includes(":")
    ? rawZone
    : `${rawZone.slice(0, 3)}:${rawZone.slice(3)}`;
  return ` UTC${offset}`;
}

export function formatOptionalTimestamp(value?: string): string {
  return value ? formatTimestamp(value) : "—";
}

export function formatOptionalDetailedTimestamp(
  value?: string,
  fallback = "—",
): string {
  return value ? formatDetailedTimestamp(value) : fallback;
}

export function formatFileKind(
  kind: "directory" | "file" | "symlink" | "special",
): string {
  switch (kind) {
    case "directory":
      return "Folder";
    case "file":
      return "File";
    case "symlink":
      return "Symbolic link";
    case "special":
      return "Special file";
  }
}
