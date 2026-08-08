import { Button, Input, TextArea } from "@wabou/components";
import { ScrollArea, Text, View } from "@wabou/primitives";
import CreditCard from "lucide-solid/icons/credit-card";
import KeyRound from "lucide-solid/icons/key-round";
import Save from "lucide-solid/icons/save";
import StickyNote from "lucide-solid/icons/sticky-note";
import X from "lucide-solid/icons/x";
import { For, Show, createEffect, createSignal, type JSX } from "solid-js";
import type { EditableItemKind, ItemDetails, ItemDraft } from "./model";

interface ItemEditorProps {
  item?: ItemDetails;
  busy: boolean;
  onSave(draft: ItemDraft): void | Promise<void>;
  onCancel(): void;
}

function Field(props: { label: string; children: JSX.Element }) {
  return (
    <View class="flex flex-col gap-2">
      <Text class="text-xs font-medium text-slate-400">{props.label}</Text>
      {props.children}
    </View>
  );
}

export function ItemEditor(props: ItemEditorProps) {
  const [kind, setKind] = createSignal<EditableItemKind>("login");
  const [name, setName] = createSignal("");
  const [notes, setNotes] = createSignal("");
  const [favorite, setFavorite] = createSignal(false);
  const [username, setUsername] = createSignal("");
  const [password, setPassword] = createSignal("");
  const [uri, setUri] = createSignal("");
  const [totp, setTotp] = createSignal("");
  const [cardholderName, setCardholderName] = createSignal("");
  const [cardBrand, setCardBrand] = createSignal("");
  const [cardNumber, setCardNumber] = createSignal("");
  const [cardExpMonth, setCardExpMonth] = createSignal("");
  const [cardExpYear, setCardExpYear] = createSignal("");
  const [cardCode, setCardCode] = createSignal("");

  createEffect(() => {
    const item = props.item;
    setKind(item?.kind === "note" || item?.kind === "card" ? item.kind : "login");
    setName(item?.name ?? "");
    setNotes(item?.notes ?? "");
    setFavorite(item?.favorite ?? false);
    setUsername(item?.username ?? "");
    setPassword(item?.password ?? "");
    setUri(item?.uris[0] ?? "");
    setTotp(item?.totp ?? "");
    setCardholderName(item?.cardholderName ?? "");
    setCardBrand(item?.cardBrand ?? "");
    setCardNumber(item?.cardNumber ?? "");
    setCardExpMonth(item?.cardExpMonth ?? "");
    setCardExpYear(item?.cardExpYear ?? "");
    setCardCode(item?.cardCode ?? "");
  });

  const kinds: ReadonlyArray<{ id: EditableItemKind; label: string }> = [
    { id: "login", label: "Login" },
    { id: "note", label: "Secure note" },
    { id: "card", label: "Card" },
  ];

  function icon(value: EditableItemKind) {
    if (value === "note") return <StickyNote size={15} />;
    if (value === "card") return <CreditCard size={15} />;
    return <KeyRound size={15} />;
  }

  function save() {
    props.onSave({
      kind: kind(),
      name: name(),
      notes: notes(),
      favorite: favorite(),
      username: username(),
      password: password(),
      uri: uri(),
      totp: totp(),
      cardholderName: cardholderName(),
      cardBrand: cardBrand(),
      cardNumber: cardNumber(),
      cardExpMonth: cardExpMonth(),
      cardExpYear: cardExpYear(),
      cardCode: cardCode(),
    });
  }

  return (
    <View class="min-w-0 flex-1 flex flex-col">
      <View class="h-16 shrink-0 border-b border-slate-800 px-6 flex items-center gap-3">
        <Text class="text-lg font-semibold text-slate-100">
          {props.item ? "Edit item" : "New item"}
        </Text>
        <View class="flex-1" />
        <Button variant="ghost" disabled={props.busy} onClick={props.onCancel}>
          <View class="flex items-center gap-2"><X size={15} /><Text>Cancel</Text></View>
        </Button>
        <Button disabled={props.busy} onClick={save}>
          <View class="flex items-center gap-2"><Save size={15} /><Text>{props.busy ? "Saving…" : "Save"}</Text></View>
        </Button>
      </View>
      <ScrollArea class="min-h-0 flex-1">
        <View class="p-8 flex flex-col gap-5">
          <Field label="Type">
            <View class="flex gap-2">
              <For each={kinds}>{(entry) => (
                <Button
                  size="sm"
                  variant={kind() === entry.id ? "secondary" : "outline"}
                  disabled={Boolean(props.item) || props.busy}
                  onClick={() => setKind(entry.id)}
                >
                  <View class="flex items-center gap-2">{icon(entry.id)}<Text>{entry.label}</Text></View>
                </Button>
              )}</For>
            </View>
          </Field>
          <Field label="Name">
            <Input value={name()} onInput={(event) => setName(event.currentTarget.value)} />
          </Field>
          <Button
            class="justify-start"
            variant={favorite() ? "secondary" : "outline"}
            disabled={props.busy}
            onClick={() => setFavorite(!favorite())}
          >
            {favorite() ? "★ Favorite" : "☆ Add to favorites"}
          </Button>

          <Show when={kind() === "login"}>
            <Field label="Username">
              <Input value={username()} onInput={(event) => setUsername(event.currentTarget.value)} />
            </Field>
            <Field label="Password">
              <Input type="password" value={password()} onInput={(event) => setPassword(event.currentTarget.value)} />
            </Field>
            <Field label="Website">
              <Input placeholder="https://example.com" value={uri()} onInput={(event) => setUri(event.currentTarget.value)} />
            </Field>
            <Field label="Authenticator key">
              <Input type="password" value={totp()} onInput={(event) => setTotp(event.currentTarget.value)} />
            </Field>
          </Show>

          <Show when={kind() === "card"}>
            <Field label="Cardholder name"><Input value={cardholderName()} onInput={(event) => setCardholderName(event.currentTarget.value)} /></Field>
            <View class="flex gap-3">
              <View class="flex-1"><Field label="Brand"><Input value={cardBrand()} onInput={(event) => setCardBrand(event.currentTarget.value)} /></Field></View>
              <View class="flex-1"><Field label="Number"><Input type="password" value={cardNumber()} onInput={(event) => setCardNumber(event.currentTarget.value)} /></Field></View>
            </View>
            <View class="flex gap-3">
              <View class="flex-1"><Field label="Expiry month"><Input value={cardExpMonth()} onInput={(event) => setCardExpMonth(event.currentTarget.value)} /></Field></View>
              <View class="flex-1"><Field label="Expiry year"><Input value={cardExpYear()} onInput={(event) => setCardExpYear(event.currentTarget.value)} /></Field></View>
              <View class="flex-1"><Field label="Security code"><Input type="password" value={cardCode()} onInput={(event) => setCardCode(event.currentTarget.value)} /></Field></View>
            </View>
          </Show>

          <Field label={kind() === "note" ? "Secure note" : "Notes"}>
            <TextArea class="h-32" value={notes()} onInput={(event) => setNotes(event.currentTarget.value)} />
          </Field>
        </View>
      </ScrollArea>
    </View>
  );
}
