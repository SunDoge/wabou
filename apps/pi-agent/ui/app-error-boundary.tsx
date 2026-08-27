import { Button, Text, View } from "@wabou/ui";
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
            <View
              role="alert"
              aria-label="Pi Agent failed to render"
              class="w-full max-w-2xl p-6 rounded-xl border border-danger bg-danger-surface gap-4"
            >
              <Text class="text-lg font-semibold text-danger-primary">
                Pi Agent failed to render
              </Text>
              <Text class="text-sm text-danger-primary whitespace-normal">
                {message}
              </Text>
              <Button variant="outline" onClick={reset}>
                Retry
              </Button>
            </View>
          </View>
        );
      }}
    >
      {props.children}
    </Errored>
  );
}
