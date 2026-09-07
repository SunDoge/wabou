/**
 * Guards asynchronous UI work against a selection or request that has moved on.
 *
 * `begin` supersedes the previous request. `capture` lets parallel work share the
 * current selection epoch, as required by lazily loaded directory branches.
 */
export interface AsyncRequestGate {
  begin(): object;
  capture(): object;
  invalidate(): void;
  isCurrent(token: object): boolean;
}

export function createAsyncRequestGate(): AsyncRequestGate {
  let current = Object.freeze({});
  const advance = () => {
    current = Object.freeze({});
    return current;
  };
  return {
    begin: advance,
    capture: () => current,
    invalidate: () => {
      advance();
    },
    isCurrent: (token) => token === current,
  };
}
