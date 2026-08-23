import { expect, test } from "bun:test";
import { installEncodingStreamsPolyfill } from "./encoding-streams";

test("TextDecoderStream preserves split UTF-8 sequences", async () => {
  installEncodingStreamsPolyfill();
  const bytes = new TextEncoder().encode("漫画");
  const stream = new ReadableStream<BufferSource>({
    start(controller) {
      controller.enqueue(bytes.slice(0, 2));
      controller.enqueue(bytes.slice(2));
      controller.close();
    },
  }).pipeThrough(new TextDecoderStream());

  let text = "";
  for await (const chunk of stream) text += chunk;
  expect(text).toBe("漫画");
});

test("TextEncoderStream preserves surrogate pairs split across chunks", async () => {
  installEncodingStreamsPolyfill();
  const stream = new ReadableStream<string>({
    start(controller) {
      controller.enqueue("\ud83d");
      controller.enqueue("\ude80");
      controller.close();
    },
  }).pipeThrough(new TextEncoderStream());

  const chunks: Uint8Array[] = [];
  for await (const chunk of stream) chunks.push(chunk);
  expect(chunks).toEqual([new TextEncoder().encode("🚀")]);
});
