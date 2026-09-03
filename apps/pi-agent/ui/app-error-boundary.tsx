import { ContentState, View } from "@wabou/ui";
import { Errored, type JSX } from "solid-js";

function describeError(error: unknown): string {
  if (error instanceof Error) return error.stack ?? error.message;
  return String(error);
}

export function AppErrorBoundary(props: { children: JSX.Element }) {
  return (
    <Errored
      fallback={(error, reset) => {
        const message = describeError(error());
        console.error("[pi-agent] application render failed", message);
        return (
          <View class="w-full h-full p-8 bg-canvas flex items-center justify-center">
            <ContentState
              state="error"
              title="Pi Agent failed to render"
              description={message}
              aria-label="Pi Agent failed to render"
              class="max-w-2xl h-auto flex-none rounded-xl border border-danger bg-danger-surface"
              action={{ label: "Retry", onAction: reset }}
            />
          </View>
        );
      }}
    >
      {props.children}
    </Errored>
  );
}
