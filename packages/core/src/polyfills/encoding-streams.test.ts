import { afterAll, beforeAll, expect, test } from "bun:test";
import {
  ReadableStream as PolyfillReadableStream,
  TransformStream,
} from "web-streams-polyfill";

const ReadableStream =
  PolyfillReadableStream as unknown as typeof globalThis.ReadableStream;
const previousTransform = globalThis.TransformStream;
beforeAll(() => {
  globalThis.TransformStream =
    TransformStream as typeof globalThis.TransformStream;
});
afterAll(() => {
  globalThis.TransformStream = previousTransform;
});

import {
  installEncodingStreamsPolyfill,
  WabouTextDecoderStream,
  WabouTextEncoderStream,
} from "./encoding-streams";

test("TextDecoderStream preserves split UTF-8 sequences", async () => {
  installEncodingStreamsPolyfill();
  const bytes = new TextEncoder().encode("漫画");
  const stream = new ReadableStream<BufferSource>({
    start(controller) {
      controller.enqueue(bytes.slice(0, 2));
      controller.enqueue(bytes.slice(2));
      controller.close();
    },
  }).pipeThrough(new WabouTextDecoderStream());

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
  }).pipeThrough(new WabouTextEncoderStream());

  const chunks: Uint8Array[] = [];
  for await (const chunk of stream) chunks.push(chunk);
  expect(chunks).toEqual([new TextEncoder().encode("🚀")]);
});

test("decoder flush replaces incomplete input and fatal mode rejects", async () => {
  const source = () =>
    new ReadableStream<BufferSource>({
      start(c) {
        c.enqueue(new Uint8Array([0xe2, 0x82]));
        c.close();
      },
    });
  const chunks: string[] = [];
  for await (const chunk of source().pipeThrough(new WabouTextDecoderStream()))
    chunks.push(chunk);
  expect(chunks).toEqual(["�"]);
  await expect(
    (async () => {
      for await (const _ of source().pipeThrough(
        new WabouTextDecoderStream("utf-8", { fatal: true }),
      )) {
      }
    })(),
  ).rejects.toBeInstanceOf(TypeError);
});

test("encoder preserves pending surrogate across empty chunks and flushes it", async () => {
  const source = new ReadableStream({
    start(c) {
      for (const s of ["\ud83d", "", "\ude80", "\ud800"]) c.enqueue(s);
      c.close();
    },
  });
  const chunks: Uint8Array[] = [];
  for await (const chunk of source.pipeThrough(new WabouTextEncoderStream()))
    chunks.push(chunk);
  expect(chunks).toEqual([
    new TextEncoder().encode("🚀"),
    new TextEncoder().encode("�"),
  ]);
});

test("decoder exposes encoding options and handles split BOM", async () => {
  const transform = new WabouTextDecoderStream("utf-8", { ignoreBOM: true });
  expect(transform.encoding).toBe("utf-8");
  expect(transform.ignoreBOM).toBe(true);
  expect(transform.fatal).toBe(false);
  const source = new ReadableStream<BufferSource>({
    start(c) {
      c.enqueue(new Uint8Array([0xef]));
      c.enqueue(new Uint8Array([0xbb, 0xbf, 65]));
      c.close();
    },
  });
  const chunks: string[] = [];
  for await (const chunk of source.pipeThrough(transform)) chunks.push(chunk);
  expect(chunks).toEqual(["\ufeffA"]);
});

test("readable cancellation propagates its reason to writable", async () => {
  const transform = new WabouTextEncoderStream();
  const reason = new Error("stop stream");
  const writer = transform.writable.getWriter();
  const rejected = writer.closed.catch((error) => error);
  await transform.readable.cancel(reason);
  expect(await rejected).toBe(reason);
  await expect(writer.write("ignored")).rejects.toBe(reason);
});
