import { describe, expect, test } from "bun:test";
import { createRoot, flush } from "solid-js";
import { createAgentProfiles } from "./agent-profiles";
import type { DeferredWriterScheduler } from "./deferred-writer";

function manualScheduler() {
  let nextHandle = 0;
  const tasks = new Map<number, () => void>();
  const scheduler: DeferredWriterScheduler = {
    set(callback) {
      const handle = ++nextHandle;
      tasks.set(handle, callback);
      return handle;
    },
    clear(handle) {
      tasks.delete(handle as number);
    },
  };
  return {
    scheduler,
    run() {
      for (const [handle, task] of [...tasks]) {
        tasks.delete(handle);
        task();
      }
    },
  };
}

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((accept) => {
    resolve = accept;
  });
  return { promise, resolve };
}

const storedProject = {
  id: "agent-4",
  name: "Wabou",
  cwd: "/work/wabou",
  provider: "openai-codex",
  model: "gpt-5.6-codex",
};

describe("agent profiles", () => {
  test("hydrates stored projects without writing them back", async () => {
    const clock = manualScheduler();
    const saved: unknown[] = [];
    let defaultWorkspaceCalls = 0;
    let profiles!: ReturnType<typeof createAgentProfiles>;
    let dispose!: () => void;
    createRoot((rootDispose) => {
      dispose = rootDispose;
      profiles = createAgentProfiles({
        api: {
          listAgents: async () => [storedProject],
          saveAgents: async (agents: unknown) => saved.push(agents),
          defaultWorkspace: async () => {
            defaultWorkspaceCalls += 1;
            return "/work/default";
          },
        } as never,
        routeAgentId: () => undefined,
        scheduler: clock.scheduler,
      });
      flush();
    });

    await new Promise<void>((resolve) => setTimeout(resolve, 0));
    flush();
    clock.run();

    expect(profiles.agents().map((agent) => agent.id)).toEqual(["agent-4"]);
    expect(profiles.active().cwd).toBe("/work/wabou");
    expect(defaultWorkspaceCalls).toBe(0);
    expect(saved).toEqual([]);
    dispose();
  });

  test("creates and persists the first project when storage is empty", async () => {
    const clock = manualScheduler();
    const saved: unknown[] = [];
    let defaultWorkspaceCalls = 0;
    let profiles!: ReturnType<typeof createAgentProfiles>;
    let dispose!: () => void;
    createRoot((rootDispose) => {
      dispose = rootDispose;
      profiles = createAgentProfiles({
        api: {
          listAgents: async () => [],
          saveAgents: async (agents: unknown) => saved.push(agents),
          defaultWorkspace: async () => {
            defaultWorkspaceCalls += 1;
            return "/work/PiWorkspace";
          },
        } as never,
        routeAgentId: () => undefined,
        scheduler: clock.scheduler,
      });
      flush();
    });

    await new Promise<void>((resolve) => setTimeout(resolve, 0));
    flush();
    clock.run();
    await Promise.resolve();

    expect(defaultWorkspaceCalls).toBe(1);
    expect(profiles.active().cwd).toBe("/work/PiWorkspace");
    expect(saved).toEqual([
      [
        {
          id: "agent-1",
          name: "Project 1",
          cwd: "/work/PiWorkspace",
          provider: "",
          model: "",
        },
      ],
    ]);
    dispose();
  });

  test("exposes one pending default workspace request per project", async () => {
    const workspace = deferred<string>();
    let calls = 0;
    let profiles!: ReturnType<typeof createAgentProfiles>;
    createRoot(() => {
      profiles = createAgentProfiles({
        api: {
          listAgents: () => [storedProject],
          saveAgents: () => {},
          defaultWorkspace: () => {
            calls += 1;
            return workspace.promise;
          },
        } as never,
        routeAgentId: () => undefined,
      });
      flush();
    });
    await Promise.resolve();
    flush();

    const first = profiles.prepareDefaultWorkspace("agent-4");
    const duplicate = profiles.prepareDefaultWorkspace("agent-4");
    expect(profiles.workspacePending("agent-4")).toBe(true);
    expect(calls).toBe(1);

    workspace.resolve("/work/default/agent-4");
    await Promise.all([first, duplicate]);
    flush();
    expect(profiles.workspacePending("agent-4")).toBe(false);
    expect(profiles.active().cwd).toBe("/work/wabou");
  });

  test("repairs and persists a stored project with no workspace", async () => {
    const clock = manualScheduler();
    const saved: unknown[] = [];
    let defaultWorkspaceCalls = 0;
    let profiles!: ReturnType<typeof createAgentProfiles>;
    let dispose!: () => void;
    createRoot((rootDispose) => {
      dispose = rootDispose;
      profiles = createAgentProfiles({
        api: {
          listAgents: async () => [{ ...storedProject, cwd: "  " }],
          saveAgents: async (agents: unknown) => saved.push(agents),
          defaultWorkspace: async (id: string) => {
            defaultWorkspaceCalls += 1;
            return `/work/default/${id}`;
          },
        } as never,
        routeAgentId: () => undefined,
        scheduler: clock.scheduler,
      });
      flush();
    });

    await new Promise<void>((resolve) => setTimeout(resolve, 0));
    flush();
    clock.run();
    await Promise.resolve();

    expect(defaultWorkspaceCalls).toBe(1);
    expect(profiles.active().cwd).toBe("/work/default/agent-4");
    expect(saved).toEqual([
      [
        {
          ...storedProject,
          cwd: "/work/default/agent-4",
        },
      ],
    ]);
    dispose();
  });

  test("exposes load and save failures with explicit retries", async () => {
    const clock = manualScheduler();
    let loadAttempts = 0;
    let saveAttempts = 0;
    let profiles!: ReturnType<typeof createAgentProfiles>;
    let dispose!: () => void;
    createRoot((rootDispose) => {
      dispose = rootDispose;
      profiles = createAgentProfiles({
        api: {
          listAgents: async () => {
            loadAttempts += 1;
            if (loadAttempts === 1) throw new Error("database unavailable");
            return [storedProject];
          },
          saveAgents: async () => {
            saveAttempts += 1;
            if (saveAttempts === 1) throw new Error("database is read-only");
          },
          defaultWorkspace: async () => "/work/default",
        } as never,
        routeAgentId: () => undefined,
        scheduler: clock.scheduler,
      });
      flush();
    });

    await Promise.resolve();
    await Promise.resolve();
    flush();
    expect(String(profiles.loadError())).toContain("database unavailable");

    await profiles.reload();
    flush();
    expect(profiles.loadError()).toBeUndefined();
    expect(profiles.active().id).toBe("agent-4");

    profiles.patchActive({ name: "Wabou desktop" });
    clock.run();
    await Promise.resolve();
    await Promise.resolve();
    expect(String(profiles.saveError())).toContain("database is read-only");

    profiles.retrySave();
    await Promise.resolve();
    expect(profiles.saveError()).toBeUndefined();
    expect(saveAttempts).toBe(2);
    dispose();
  });
});
