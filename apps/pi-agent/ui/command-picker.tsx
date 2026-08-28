import { Button, Command, Icon, Popover, Text, View } from "@wabou/ui";
import terminal from "lucide-static/icons/terminal.svg?raw";
import { createMemo, createSignal } from "solid-js";
import type { AgentCommand } from "./agent-state";
import { i18n, m } from "./i18n";

export interface CommandPickerProps {
  commands: readonly AgentCommand[];
  choose(command: string): void;
}

export function CommandPicker(props: CommandPickerProps) {
  const [open, setOpen] = createSignal(false);
  const items = createMemo(() =>
    props.commands.map((command) => ({
      id: command.name,
      label: `/${command.name}`,
      description:
        command.description ??
        i18n.message(m.command_source, { source: command.source }),
    })),
  );
  const choose = (name: string) => {
    props.choose(`/${name} `);
    setOpen(false);
  };

  return (
    <Popover
      aria-label={i18n.message(m.available_commands, {})}
      placement="top-start"
      open={open()}
      onOpenChange={setOpen}
      contentClass="w-96 max-h-80"
      trigger={(trigger) => (
        <Button
          {...trigger}
          variant="ghost"
          size="icon"
          aria-label={i18n.message(m.available_commands, {})}
          disabled={props.commands.length === 0}
        >
          <Icon source={terminal} size={14} />
        </Button>
      )}
    >
      <View class="px-1 pb-2">
        <Text class="font-semibold">
          {i18n.message(m.available_commands, {})}
        </Text>
        <Text class="text-xs text-muted">
          {i18n.message(m.available_commands_detail, {})}
        </Text>
      </View>
      <Command
        aria-label={i18n.message(m.search_commands, {})}
        items={items()}
        placeholder={i18n.message(m.search_commands, {})}
        emptyText={i18n.message(m.no_commands_found, {})}
        listClass="max-h-52 overflow-y-auto"
        onAction={choose}
        onDismiss={() => setOpen(false)}
      />
    </Popover>
  );
}
