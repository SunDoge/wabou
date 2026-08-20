import { afterEach, expect, test } from "bun:test";
import { installCryptoPolyfill } from "./crypto";

const runtime = globalThis as typeof globalThis & {
  __wabou_crypto_random?: (output: Uint8Array) => void;
  __wabou_crypto_digest?: (
    algorithm: number,
    input: Uint8Array,
  ) => Promise<Uint8Array>;
};
const originalCrypto = Object.getOwnPropertyDescriptor(globalThis, "crypto");
const originalRandom = runtime.__wabou_crypto_random;
const originalDigest = runtime.__wabou_crypto_digest;

afterEach(() => {
  if (originalCrypto)
    Object.defineProperty(globalThis, "crypto", originalCrypto);
  else Reflect.deleteProperty(globalThis, "crypto");
  if (originalRandom) runtime.__wabou_crypto_random = originalRandom;
  else Reflect.deleteProperty(runtime, "__wabou_crypto_random");
  if (originalDigest) runtime.__wabou_crypto_digest = originalDigest;
  else Reflect.deleteProperty(runtime, "__wabou_crypto_digest");
});

function installDeterministicCrypto(): void {
  Reflect.deleteProperty(globalThis, "crypto");
  runtime.__wabou_crypto_random = (output) => {
    for (let index = 0; index < output.length; index++) output[index] = index;
  };
  runtime.__wabou_crypto_digest = async (algorithm, input) =>
    new Uint8Array([algorithm, ...input]);
  installCryptoPolyfill();
}

test("fills integer views in place and creates RFC 4122 v4 UUIDs", () => {
  installDeterministicCrypto();
  const words = new Uint32Array(2);
  expect(crypto.getRandomValues(words)).toBe(words);
  expect(new Uint8Array(words.buffer)).toEqual(
    new Uint8Array([0, 1, 2, 3, 4, 5, 6, 7]),
  );
  expect(crypto.randomUUID()).toBe("00010203-0405-4607-8809-0a0b0c0d0e0f");
  expect(() => crypto.getRandomValues(new Float32Array(1) as never)).toThrow();
  expect(() => crypto.getRandomValues(new Uint8Array(65_537))).toThrow();
});

test("normalizes digest names and preserves BufferSource slices", async () => {
  installDeterministicCrypto();
  const source = new Uint8Array([99, 10, 20, 88]);
  const result = await crypto.subtle.digest(
    { name: "sha-256" },
    source.subarray(1, 3),
  );
  expect(new Uint8Array(result)).toEqual(new Uint8Array([2, 10, 20]));
  await expect(crypto.subtle.digest("MD5", source)).rejects.toThrow();
});
