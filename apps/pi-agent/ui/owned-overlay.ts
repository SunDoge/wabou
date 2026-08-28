import { createSignal, type Accessor } from "solid-js";

export interface OwnedOverlayValue<T> {
  ownerId: string;
  data: T;
}

export interface OwnedOverlay<T> {
  value: Accessor<OwnedOverlayValue<T> | undefined>;
  open(ownerId: string, data: T): void;
  close(): void;
  retainOwner(ownerId: string): void;
}

export function createOwnedOverlay<T>(): OwnedOverlay<T> {
  const [value, setValue] = createSignal<OwnedOverlayValue<T>>();
  return {
    value,
    open(ownerId, data) {
      setValue({ ownerId, data });
    },
    close() {
      setValue(undefined);
    },
    retainOwner(ownerId) {
      setValue((current) =>
        current?.ownerId === ownerId ? current : undefined,
      );
    },
  };
}
