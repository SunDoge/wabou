import { type Accessor, createSignal } from "solid-js";

export interface SessionLocation {
  agentId: string;
  sessionId: string;
}

export interface SessionNavigationController {
  canGoBack: Accessor<boolean>;
  canGoForward: Accessor<boolean>;
  visit(location: SessionLocation): void;
  back(): SessionLocation | undefined;
  forward(): SessionLocation | undefined;
  removeAgent(agentId: string): void;
}

function sameLocation(left: SessionLocation, right: SessionLocation): boolean {
  return left.agentId === right.agentId && left.sessionId === right.sessionId;
}

/** Explicit conversation history, independent from settings and panel routes. */
export class SessionNavigation {
  readonly #entries: SessionLocation[] = [];
  #index = -1;

  get canGoBack(): boolean {
    return this.#index > 0;
  }

  get canGoForward(): boolean {
    return this.#index >= 0 && this.#index < this.#entries.length - 1;
  }

  visit(location: SessionLocation): boolean {
    const current = this.#entries[this.#index];
    if (current && sameLocation(current, location)) return false;
    this.#entries.splice(this.#index + 1);
    this.#entries.push({ ...location });
    this.#index = this.#entries.length - 1;
    return true;
  }

  back(): SessionLocation | undefined {
    if (!this.canGoBack) return undefined;
    this.#index -= 1;
    return { ...this.#entries[this.#index] };
  }

  forward(): SessionLocation | undefined {
    if (!this.canGoForward) return undefined;
    this.#index += 1;
    return { ...this.#entries[this.#index] };
  }

  removeAgent(agentId: string): boolean {
    const retainedBeforeCurrent = this.#entries
      .slice(0, this.#index + 1)
      .filter((entry) => entry.agentId !== agentId).length;
    const retained = this.#entries.filter((entry) => entry.agentId !== agentId);
    if (retained.length === this.#entries.length) return false;
    this.#entries.splice(0, this.#entries.length, ...retained);
    this.#index = Math.min(retainedBeforeCurrent - 1, retained.length - 1);
    return true;
  }
}

/** Solid adapter that keeps mutation tracking inside the navigation boundary. */
export function createSessionNavigation(): SessionNavigationController {
  const navigation = new SessionNavigation();
  const [revision, setRevision] = createSignal(0);
  const changed = (change: () => boolean) => {
    if (change()) setRevision((value) => value + 1);
  };
  const traverse = (
    move: () => SessionLocation | undefined,
  ): SessionLocation | undefined => {
    const target = move();
    if (target) setRevision((value) => value + 1);
    return target;
  };
  return {
    canGoBack: () => {
      revision();
      return navigation.canGoBack;
    },
    canGoForward: () => {
      revision();
      return navigation.canGoForward;
    },
    visit: (location) => changed(() => navigation.visit(location)),
    back: () => traverse(() => navigation.back()),
    forward: () => traverse(() => navigation.forward()),
    removeAgent: (agentId) => changed(() => navigation.removeAgent(agentId)),
  };
}
