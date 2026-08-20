type DigestAlgorithm = "SHA-1" | "SHA-256" | "SHA-384" | "SHA-512";

type CryptoRuntime = typeof globalThis & {
  __wabou_crypto_random(output: Uint8Array): void;
  __wabou_crypto_digest(
    algorithm: number,
    input: Uint8Array,
  ): Promise<Uint8Array>;
};

const DIGEST_IDS: Record<DigestAlgorithm, number> = {
  "SHA-1": 1,
  "SHA-256": 2,
  "SHA-384": 3,
  "SHA-512": 4,
};

function digestName(algorithm: AlgorithmIdentifier): DigestAlgorithm {
  const raw = typeof algorithm === "string" ? algorithm : algorithm.name;
  const name = raw.toUpperCase() as DigestAlgorithm;
  if (!(name in DIGEST_IDS))
    throw new DOMException(
      `Unsupported digest algorithm: ${raw}`,
      "NotSupportedError",
    );
  return name;
}

function bytesOf(source: BufferSource): Uint8Array {
  if (ArrayBuffer.isView(source))
    return new Uint8Array(source.buffer, source.byteOffset, source.byteLength);
  return new Uint8Array(source);
}

function isIntegerArray(value: ArrayBufferView): boolean {
  return (
    value instanceof Int8Array ||
    value instanceof Uint8Array ||
    value instanceof Uint8ClampedArray ||
    value instanceof Int16Array ||
    value instanceof Uint16Array ||
    value instanceof Int32Array ||
    value instanceof Uint32Array ||
    (typeof BigInt64Array !== "undefined" && value instanceof BigInt64Array) ||
    (typeof BigUint64Array !== "undefined" && value instanceof BigUint64Array)
  );
}

class WabouSubtleCrypto {
  async digest(
    algorithm: AlgorithmIdentifier,
    data: BufferSource,
  ): Promise<ArrayBuffer> {
    const name = digestName(algorithm);
    const result = await (globalThis as CryptoRuntime).__wabou_crypto_digest(
      DIGEST_IDS[name],
      bytesOf(data),
    );
    return result.buffer as ArrayBuffer;
  }
}

class WabouCrypto {
  readonly subtle = new WabouSubtleCrypto();

  getRandomValues<T extends ArrayBufferView<ArrayBuffer> | null>(array: T): T {
    if (array === null || !isIntegerArray(array))
      throw new DOMException(
        "getRandomValues requires an integer TypedArray",
        "TypeMismatchError",
      );
    if (array.byteLength > 65_536)
      throw new DOMException(
        "getRandomValues cannot fill more than 65536 bytes",
        "QuotaExceededError",
      );
    (globalThis as CryptoRuntime).__wabou_crypto_random(bytesOf(array));
    return array;
  }

  randomUUID(): `${string}-${string}-${string}-${string}-${string}` {
    const bytes = this.getRandomValues(new Uint8Array(16));
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    const hex = Array.from(bytes, (byte) => byte.toString(16).padStart(2, "0"));
    return `${hex.slice(0, 4).join("")}-${hex.slice(4, 6).join("")}-${hex.slice(6, 8).join("")}-${hex.slice(8, 10).join("")}-${hex.slice(10).join("")}`;
  }
}

/** Install the native random and digest subset when Wabou's ABI is present. */
export function installCryptoPolyfill(): void {
  if (!("__wabou_crypto_random" in globalThis)) return;
  if (!("crypto" in globalThis)) {
    Object.defineProperty(globalThis, "crypto", {
      configurable: true,
      writable: true,
      value: new WabouCrypto(),
    });
  }
}

installCryptoPolyfill();
