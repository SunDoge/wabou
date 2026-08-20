import { Column, Text } from "@wabou/ui";

export function App() {
  return (
    <Column class="h-full items-center justify-center gap-3 bg-background">
      <Text
        role="heading"
        aria-label="__WABOU_PROJECT_NAME__"
        class="text-2xl font-semibold text-foreground"
      >
        __WABOU_PROJECT_NAME__
      </Text>
      <Text class="text-muted">Your Wabou application is ready.</Text>
    </Column>
  );
}
