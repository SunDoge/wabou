export interface CurlDownload {
  urls: string[];
  headers: string[];
  output?: string;
  proxy?: string;
}

const URL = /^(?:https?|ftps?|sftp):\/\/|^magnet:\?/i;

function shellWords(command: string): string[] | undefined {
  const words: string[] = [];
  let word = "";
  let quote: "'" | '"' | undefined;
  let escaped = false;
  const source = command.replace(/\\\r?\n/g, " ");
  const commit = () => {
    if (!word) return;
    words.push(word);
    word = "";
  };
  for (const character of source) {
    if (escaped) {
      word += character;
      escaped = false;
    } else if (character === "\\" && quote !== "'") {
      escaped = true;
    } else if (quote) {
      if (character === quote) quote = undefined;
      else word += character;
    } else if (character === "'" || character === '"') {
      quote = character;
    } else if (/\s/.test(character)) {
      commit();
    } else {
      word += character;
    }
  }
  if (quote || escaped) return undefined;
  commit();
  return words;
}

function optionValue(
  words: readonly string[],
  index: number,
  longName: string,
): [string | undefined, number] {
  const inline = words[index]?.match(new RegExp(`^--${longName}=(.*)$`));
  if (inline) return [inline[1], index];
  return [words[index + 1], index + 1];
}

/** Interpret the download-related subset emitted by browser “Copy as cURL”. */
export function parseCurlDownload(source: string): CurlDownload | undefined {
  const words = shellWords(source.trim());
  if (words?.[0]?.toLowerCase() !== "curl") return undefined;
  const result: CurlDownload = { urls: [], headers: [] };
  for (let index = 1; index < words.length; index += 1) {
    const word = words[index];
    if (
      [
        "-d",
        "--data",
        "--data-raw",
        "--data-binary",
        "-F",
        "--form",
        "-T",
        "--upload-file",
      ].some((option) => word === option || word.startsWith(`${option}=`))
    )
      return undefined;
    if (URL.test(word)) {
      result.urls.push(word);
      continue;
    }
    const consumes = (short: string, long: string) =>
      word === short || word === `--${long}` || word.startsWith(`--${long}=`);
    if (consumes("-H", "header")) {
      const [value, next] = optionValue(words, index, "header");
      if (value?.includes(":")) result.headers.push(value);
      index = next;
    } else if (consumes("-A", "user-agent")) {
      const [value, next] = optionValue(words, index, "user-agent");
      if (value) result.headers.push(`User-Agent: ${value}`);
      index = next;
    } else if (consumes("-e", "referer")) {
      const [value, next] = optionValue(words, index, "referer");
      if (value) result.headers.push(`Referer: ${value}`);
      index = next;
    } else if (consumes("-b", "cookie")) {
      const [value, next] = optionValue(words, index, "cookie");
      if (value) result.headers.push(`Cookie: ${value}`);
      index = next;
    } else if (consumes("-x", "proxy")) {
      const [value, next] = optionValue(words, index, "proxy");
      result.proxy = value;
      index = next;
    } else if (consumes("-o", "output")) {
      const [value, next] = optionValue(words, index, "output");
      result.output = value;
      index = next;
    } else if (word === "--url" || word.startsWith("--url=")) {
      const [value, next] = optionValue(words, index, "url");
      if (value && URL.test(value)) result.urls.push(value);
      index = next;
    } else if (["-X", "--request"].includes(word)) {
      const method = words[index + 1]?.toUpperCase();
      if (method && method !== "GET") return undefined;
      index += 1;
    }
  }
  return result.urls.length ? result : undefined;
}
