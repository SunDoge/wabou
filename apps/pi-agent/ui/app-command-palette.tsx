import type { Handle } from "@wabou/core";
import { Command, type CommandItem, Dialog } from "@wabou/ui";

export interface AppCommandPaletteProps {
  open: boolean;
  items: readonly CommandItem[];
  close(): void;
  label: string;
  placeholder: string;
  emptyText: string;
}

/** Keyboard-first application navigation composed from Wabou overlay primitives. */
export function AppCommandPalette(props: AppCommandPaletteProps) {
  let search: Handle | undefined;
  return (
    <Dialog
      aria-label={props.label}
      open={props.open}
      onOpenChange={(open) => !open && props.close()}
      initialFocus={() => search}
      contentClass="w-[560px] max-w-full min-w-0 p-3 gap-2"
    >
      <Command
        aria-label={props.label}
        items={props.items}
        placeholder={props.placeholder}
        emptyText={props.emptyText}
        inputRef={(node) => (search = node)}
        listClass="max-h-80 overflow-y-auto"
        onAction={props.close}
        onDismiss={props.close}
      />
    </Dialog>
  );
}
