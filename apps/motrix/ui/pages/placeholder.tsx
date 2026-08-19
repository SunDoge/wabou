import { Icon, Text, View } from "@wabou/ui";
import boxes from "lucide-static/icons/boxes.svg?raw";

export function PlaceholderPage(props: { title: string }) {
  return (
    <View class="h-96 flex flex-col items-center justify-center gap-3">
      <Icon source={boxes} size={38} class="text-muted" />
      <Text class="text-2xl font-semibold">{props.title}</Text>
      <Text class="text-sm text-muted">
        This surface is reserved for the aria2-backed implementation.
      </Text>
    </View>
  );
}
