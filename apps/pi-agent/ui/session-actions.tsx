import {
  Button,
  createKeyedAsyncAction,
  DropdownMenu,
  type DropdownMenuTriggerProps,
  Icon,
} from "@wabou/ui";
import ellipsis from "lucide-static/icons/ellipsis.svg?raw";
import type { JSX } from "solid-js";
import { i18n, m } from "./i18n";

export function SessionActions(props: {
  disabled?: boolean;
  compact(): void | Promise<void>;
  clone(): void | Promise<void>;
  exportHtml(): void | Promise<void>;
  onActionError?: (action: SessionAction, error: unknown) => void;
  trigger?: (props: DropdownMenuTriggerProps) => JSX.Element;
}) {
  const action = createKeyedAsyncAction(
    (id: SessionAction, _perform: () => void | Promise<void>) => id,
    (_id: SessionAction, perform: () => void | Promise<void>) => perform(),
  );
  const run = async (
    id: SessionAction,
    perform: () => void | Promise<void>,
  ) => {
    const result = await action.run(id, perform);
    if (!result.ok) props.onActionError?.(id, result.error);
  };
  const items = () => [
    {
      id: "compact",
      label: i18n.message(m.compact_session, {}),
      description: i18n.message(m.compact_session_detail, {}),
      disabled: props.disabled || action.pending("compact"),
      onSelect: () => void run("compact", props.compact),
    },
    {
      id: "clone",
      label: i18n.message(m.clone_session, {}),
      description: i18n.message(m.clone_session_detail, {}),
      disabled: props.disabled || action.pending("clone"),
      onSelect: () => void run("clone", props.clone),
    },
    {
      id: "export",
      label: i18n.message(m.export_session, {}),
      description: i18n.message(m.export_session_detail, {}),
      disabled: props.disabled || action.pending("export"),
      separatorBefore: true,
      onSelect: () => void run("export", props.exportHtml),
    },
  ];

  return (
    <DropdownMenu
      aria-label={i18n.message(m.session_actions, {})}
      items={items()}
      contentClass="w-72"
      trigger={(trigger) =>
        props.trigger?.(trigger) ?? (
          <Button
            {...trigger}
            variant="ghost"
            size="icon"
            aria-label={i18n.message(m.session_actions, {})}
          >
            <Icon source={ellipsis} size={16} />
          </Button>
        )
      }
    />
  );
}

export type SessionAction = "compact" | "clone" | "export";
