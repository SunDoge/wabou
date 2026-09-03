export class ScopedHandleRegistry<T> {
  #scope = "";
  #handles = new Map<string, T>();

  synchronize(scope: string, retainedIds: Iterable<string>): void {
    if (scope !== this.#scope) {
      this.#scope = scope;
      this.#handles.clear();
      return;
    }
    const retained = new Set(retainedIds);
    for (const id of this.#handles.keys()) {
      if (!retained.has(id)) this.#handles.delete(id);
    }
  }

  register(scope: string, id: string, handle: T): boolean {
    if (scope !== this.#scope) return false;
    this.#handles.set(id, handle);
    return true;
  }

  resolve(scope: string, id: string): T | undefined {
    return scope === this.#scope ? this.#handles.get(id) : undefined;
  }

  clear(): void {
    this.#handles.clear();
  }

  get size(): number {
    return this.#handles.size;
  }
}
