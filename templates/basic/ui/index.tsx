import { Column, mount, Text } from "@wabou/ui";
import "virtual:wabou-stylesheet";

mount(() => (
  <Column class="h-full items-center justify-center gap-3 bg-background">
    <Text class="text-2xl font-semibold text-foreground">
      __WABOU_PROJECT_NAME__
    </Text>
    <Text class="text-muted">Your Wabou application is ready.</Text>
  </Column>
));
