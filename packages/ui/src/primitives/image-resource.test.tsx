import { expect, test } from "bun:test";
import { defaultHost } from "@wabou/core";
import { createRoot, createSignal, flush } from "solid-js";
import {
  createOwnedImageResource,
  type ImageResourceDescriptor,
} from "./image-resource";

const settle = async () => {
  for (let index = 0; index < 6; index++) await Promise.resolve();
  flush();
};

test("owned image resources release on replacement and late completion", async () => {
  const pending: Array<(value: ImageResourceDescriptor) => void> = [];
  const released: unknown[] = [];
  const previous = (defaultHost as unknown as { imageResources?: unknown })
    .imageResources;
  (defaultHost as unknown as { imageResources: unknown }).imageResources = {
    createFile() {
      return new Promise<ImageResourceDescriptor>((resolve) =>
        pending.push(resolve),
      );
    },
    createNetwork() {
      throw new Error("unused");
    },
    release(handle: unknown) {
      released.push(handle);
      return Promise.resolve(true);
    },
  };

  const first: ImageResourceDescriptor = {
    handle: { lo: 2, hi: 1 },
    width: 10,
    height: 20,
  };
  const second: ImageResourceDescriptor = {
    handle: { lo: 3, hi: 1 },
    width: 30,
    height: 40,
  };
  let setPath!: (path: string) => void;
  let current!: () => ImageResourceDescriptor | undefined;
  const dispose = createRoot((dispose) => {
    const [path, updatePath] = createSignal("first.png");
    setPath = updatePath;
    current = createOwnedImageResource(() => ({
      kind: "file",
      path: path(),
    })).resource;
    return dispose;
  });

  flush();
  await settle();
  expect(pending).toHaveLength(1);
  pending.shift()?.(first);
  await settle();
  expect(current()).toEqual(first);

  setPath("second.png");
  flush();
  expect(current()).toBeUndefined();
  expect(released).toEqual([first.handle]);

  dispose();
  expect(pending).toHaveLength(1);
  pending.shift()?.(second);
  await settle();
  expect(released).toEqual([first.handle, second.handle]);
  (defaultHost as unknown as { imageResources?: unknown }).imageResources =
    previous;
});
