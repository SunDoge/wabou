import { renderComponent } from "@wabou/test/component";
import { Text, View } from "@wabou/ui";
import { createSignal } from "solid-js";
import { expect, test } from "vitest";
import {
  ComposerDeliveryControl,
  type ComposerDeliveryMode,
} from "../../apps/pi-agent/ui/composer-delivery";

test("chooses whether a running-agent message steers or follows up", () => {
  const [mode, setMode] = createSignal<ComposerDeliveryMode>("followUp");
  const screen = renderComponent(() => (
    <View>
      <ComposerDeliveryControl value={mode()} change={setMode} />
      <Text role="status">{mode()}</Text>
    </View>
  ));

  const trigger = screen.getByRole("combobox", { name: "Message delivery" });
  expect(trigger.text).toContain("Follow up");
  trigger.click();
  screen.getByRole("option", { name: "Steer now" }).click();

  expect(screen.getByRole("status").text).toBe("steer");
  expect(trigger.text).toContain("Steer now");
});
