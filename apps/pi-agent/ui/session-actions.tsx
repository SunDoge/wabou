import { Button, DropdownMenu, Icon } from "@wabou/ui";
import ellipsis from "lucide-static/icons/ellipsis.svg?raw";
import { i18n, m } from "./i18n";

export function SessionActions(props: {
  disabled?: boolean;
  compact(): void;
  clone(): void;
  exportHtml(): void;
}) {
  const items = () => [
    {
      id: "compact",
      label: i18n.message(m.compact_session, {}),
      description: i18n.message(m.compact_session_detail, {}),
      disabled: props.disabled,
      onSelect: props.compact,
    },
    {
      id: "clone",
      label: i18n.message(m.clone_session, {}),
      description: i18n.message(m.clone_session_detail, {}),
      disabled: props.disabled,
      onSelect: props.clone,
    },
    {
      id: "export",
      label: i18n.message(m.export_session, {}),
      description: i18n.message(m.export_session_detail, {}),
      disabled: props.disabled,
      separatorBefore: true,
      onSelect: props.exportHtml,
    },
  ];

  return (
    <DropdownMenu
      aria-label={i18n.message(m.session_actions, {})}
      items={items()}
      contentClass="w-72"
      trigger={(trigger) => (
        <Button
          {...trigger}
          variant="ghost"
          size="icon"
          aria-label={i18n.message(m.session_actions, {})}
        >
          <Icon source={ellipsis} size={16} />
        </Button>
      )}
    />
  );
}
