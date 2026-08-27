import { Select } from "@wabou/ui";
import { i18n, m } from "./i18n";

export type ComposerDeliveryMode = "steer" | "followUp";

export function ComposerDeliveryControl(props: {
  value: ComposerDeliveryMode;
  change(value: ComposerDeliveryMode): void;
}) {
  const options = () => [
    { value: "steer", label: i18n.message(m.steer, {}) },
    { value: "followUp", label: i18n.message(m.follow_up, {}) },
  ];

  return (
    <Select
      aria-label={i18n.message(m.delivery_mode, {})}
      class="w-36 border-transparent bg-transparent shadow-none"
      contentClass="w-56"
      options={options()}
      value={props.value}
      onValueChange={(value) => props.change(value as ComposerDeliveryMode)}
    />
  );
}
