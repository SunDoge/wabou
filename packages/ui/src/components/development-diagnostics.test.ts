import { expect, test } from "bun:test";
import { createRoot, flush } from "solid-js";
import { dispatchHostMessage } from "../../../core/src/glue/host-messages";
import { useDevelopmentDiagnostics } from "./development-diagnostics";

test("JS recovery preserves Rust errors, rebuilding preserves errors, matching Rust recovery clears them", () => {
  createRoot((dispose) => {
    const { diagnostic, dismiss } = useDevelopmentDiagnostics();
    const send = (topic: string, value: unknown) => {
      dispatchHostMessage(topic, JSON.stringify(value));
      flush();
    };
    send("wabou:reload-status", {
      source: "rust",
      target: "functions/Cargo.toml",
      phase: "error",
      error: "src/lib.rs:8:3: missing value",
    });
    expect(diagnostic()?.plugin).toBe("Rust");
    dispatchHostMessage("wabou:dev-server-ready", null);
    flush();
    expect(diagnostic()?.message).toContain("src/lib.rs:8:3");
    send("wabou:reload-status", {
      source: "rust",
      target: "functions/Cargo.toml",
      phase: "building",
    });
    expect(diagnostic()?.plugin).toBe("Rust");
    send("wabou:dev-server-error", { message: "broken TSX" });
    expect(diagnostic()?.message).toBe("broken TSX");
    dispatchHostMessage("wabou:dev-server-ready", null);
    flush();
    expect(diagnostic()?.plugin).toBe("Rust");
    send("wabou:reload-status", {
      source: "rust",
      target: "other/Cargo.toml",
      phase: "ready",
    });
    expect(diagnostic()?.plugin).toBe("Rust");
    send("wabou:reload-status", {
      source: "rust",
      target: "functions/Cargo.toml",
      phase: "ready",
    });
    expect(diagnostic()).toBeUndefined();
    send("wabou:dev-server-error", { message: "retry" });
    dismiss();
    flush();
    expect(diagnostic()).toBeUndefined();
    dispose();
    send("wabou:dev-server-error", { message: "after cleanup" });
    expect(diagnostic()).toBeUndefined();
  });
});
