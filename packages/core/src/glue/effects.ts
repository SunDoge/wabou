export { EFFECT_ABI_VERSION, effectOps } from "../generated/effect-abi";
import { EFFECT_ABI_VERSION } from "../generated/effect-abi";

export interface EffectOp {
  readonly capability: number;
  readonly method: number;
}

interface PendingEffect {
  readonly op: EffectOp;
  readonly resolve: (value: unknown) => void;
  readonly reject: (error: Error) => void;
}

const pending = new Map<number, PendingEffect>();

function assertAbi(): void {
  if (__wabou_effect_abi !== EFFECT_ABI_VERSION) {
    throw new Error(
      `Wabou effect ABI mismatch: bundle=${EFFECT_ABI_VERSION}, host=${__wabou_effect_abi}`,
    );
  }
}

function submit(op: EffectOp, payload: unknown): number {
  assertAbi();
  return __wabou_effect_submit(
    op.capability,
    op.method,
    JSON.stringify(payload ?? null),
  );
}

export function dispatchEffect<T>(op: EffectOp, payload?: unknown): Promise<T> {
  return new Promise<T>((resolve, reject) => {
    const id = submit(op, payload);
    pending.set(id, {
      op,
      resolve: resolve as (value: unknown) => void,
      reject,
    });
  });
}

/** Submit a command without retaining a Promise or callback. */
export function dispatchFireAndForget(op: EffectOp, payload?: unknown): void {
  submit(op, payload);
}

function complete(
  id: number,
  capability: number,
  method: number,
  status: number,
  payloadJson: string,
): void {
  const request = pending.get(id);
  if (!request) return;
  pending.delete(id);
  if (request.op.capability !== capability || request.op.method !== method) {
    request.reject(
      new Error(`Native effect ${id} completed with the wrong operation`),
    );
    return;
  }
  if (status === 1) {
    const error = new Error("Native effect was cancelled");
    error.name = "AbortError";
    request.reject(error);
    return;
  }
  const payload = JSON.parse(payloadJson) as unknown;
  if (status === 2) {
    const error = payload as { code?: string; message?: string };
    request.reject(new Error(error.message ?? "Native effect failed"));
    return;
  }
  request.resolve(payload);
}

(
  globalThis as typeof globalThis & {
    __wabou_effect_complete: typeof complete;
  }
).__wabou_effect_complete = complete;
