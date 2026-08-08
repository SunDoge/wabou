import "@wabou/core";
import "virtual:wabou-stylesheet";

import { mount } from "@wabou/solid-renderer";
import { createSignal } from "solid-js";
import type { ItemDetails, VaultItem, VaultSnapshot } from "./model";
import { TwoFactorScreen } from "./two-factor-screen";
import { VaultScreen } from "./vault-screen";

const items: VaultItem[] = [
  {
    id: "github",
    name: "GitHub",
    subtitle: "octocat@example.test",
    kind: "login",
    favorite: true,
    hasUsername: true,
    hasPassword: true,
    hasTotp: true,
  },
  {
    id: "bank",
    name: "Example Community Bank",
    subtitle: "checking •••• 4821",
    kind: "login",
    favorite: false,
    hasUsername: true,
    hasPassword: true,
    hasTotp: false,
  },
  {
    id: "note",
    name: "Emergency kit",
    subtitle: "Secure note",
    kind: "note",
    favorite: false,
    hasUsername: false,
    hasPassword: false,
    hasTotp: false,
  },
  ...Array.from({ length: 12 }, (_, index): VaultItem => ({
    id: `fixture-${index}`,
    name: `Fixture login ${index + 1}`,
    subtitle: `person${index + 1}@example.test`,
    kind: "login",
    favorite: false,
    hasUsername: true,
    hasPassword: true,
    hasTotp: index % 3 === 0,
  })),
];

const snapshot: VaultSnapshot = {
  email: "fixture@example.test",
  items,
  decryptFailures: 1,
};

function detailsFor(id: string): ItemDetails {
  const item = items.find((candidate) => candidate.id === id) ?? items[0]!;
  return {
    id: item.id,
    name: item.name,
    kind: item.kind,
    username: item.hasUsername ? item.subtitle : undefined,
    uris: item.kind === "login" ? [`https://${item.id}.example.test`] : [],
    favorite: item.favorite,
    hasPassword: item.hasPassword,
    hasTotp: item.hasTotp,
  };
}

function Preview() {
  const [selected, setSelected] = createSignal<ItemDetails>();
  const [query, setQuery] = createSignal("");
  const [notice, setNotice] = createSignal("");

  return (
    <VaultScreen
      snapshot={() => snapshot}
      selected={selected}
      query={query}
      busy={() => false}
      error={() => ""}
      notice={notice}
      setQuery={setQuery}
      refresh={() => setNotice("Fixture vault synchronized.")}
      lock={() => setNotice("Fixture lock requested.")}
      selectItem={(id) => setSelected(detailsFor(id))}
      copy={(field) => setNotice(`Fixture ${field} copy requested.`)}
    />
  );
}

function TwoFactorPreview() {
  const [provider, setProvider] = createSignal("authenticator");
  const [code, setCode] = createSignal("");
  const [notice, setNotice] = createSignal("");
  return (
    <TwoFactorScreen
      providers={[
        {
          id: "authenticator",
          label: "Authenticator app",
          supported: true,
        },
        {
          id: "email",
          label: "Email",
          hint: "f••••••@example.test",
          supported: true,
        },
        { id: "duo", label: "Duo", supported: false },
      ]}
      provider={provider()}
      code={code()}
      busy={false}
      error=""
      notice={notice()}
      setProvider={setProvider}
      setCode={setCode}
      verify={() => setNotice("Fixture verification requested.")}
      sendEmail={() => setNotice("Fixture email code sent.")}
      cancel={() => setNotice("Fixture cancellation requested.")}
    />
  );
}

mount(() => (__wabou_window_id === 2 ? <TwoFactorPreview /> : <Preview />));
