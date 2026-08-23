import { expect, test } from "bun:test";
import { installStreamsPolyfill } from "./streams";

test("WHATWG streams support transforms and async iteration", async () => {
  installStreamsPolyfill();

  const output = new ReadableStream<string>({
    start(controller) {
      controller.enqueue("wabou");
      controller.enqueue(" streams");
      controller.close();
    },
  }).pipeThrough(
    new TransformStream<string, string>({
      transform(chunk, controller) {
        controller.enqueue(chunk.toUpperCase());
      },
    }),
  );

  let text = "";
  for await (const chunk of output) text += chunk;
  expect(text).toBe("WABOU STREAMS");
});

test("WHATWG streams pipe into writable sinks", async () => {
  const chunks: number[] = [];
  const source = new ReadableStream<number>({
    start(controller) {
      controller.enqueue(1);
      controller.enqueue(2);
      controller.close();
    },
  });
  const sink = new WritableStream<number>({
    write(chunk) {
      chunks.push(chunk);
    },
  });

  await source.pipeTo(sink);
  expect(chunks).toEqual([1, 2]);
});
