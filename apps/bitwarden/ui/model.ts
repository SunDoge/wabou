export interface VaultItem {
  id: string;
  name: string;
  subtitle: string;
  kind: string;
  favorite: boolean;
  hasUsername: boolean;
  hasPassword: boolean;
  hasTotp: boolean;
}

export interface VaultSnapshot {
  email: string;
  items: VaultItem[];
  decryptFailures: number;
}

export interface TwoFactorOption {
  id: "authenticator" | "email" | "yubikey" | "duo" | "webauthn";
  label: string;
  hint?: string;
  supported: boolean;
}

export type LoginOutcome =
  | { status: "authenticated"; snapshot: VaultSnapshot }
  | { status: "twoFactorRequired"; providers: TwoFactorOption[] };

export interface ItemDetails {
  id: string;
  name: string;
  kind: string;
  username?: string;
  uris: string[];
  favorite: boolean;
  hasPassword: boolean;
  hasTotp: boolean;
}

interface Envelope<T> {
  ok: boolean;
  value?: T;
  error?: string;
}

export function unwrap<T>(raw: string): T {
  let envelope: Envelope<T>;
  try {
    envelope = JSON.parse(raw) as Envelope<T>;
  } catch {
    throw new Error("The native vault returned an invalid response.");
  }
  if (!envelope.ok || envelope.value === undefined) {
    throw new Error(envelope.error || "The native vault operation failed.");
  }
  return envelope.value;
}

export function matches(item: VaultItem, query: string): boolean {
  const needle = query.trim().toLocaleLowerCase();
  return (
    needle.length === 0 ||
    item.name.toLocaleLowerCase().includes(needle) ||
    item.subtitle.toLocaleLowerCase().includes(needle)
  );
}
