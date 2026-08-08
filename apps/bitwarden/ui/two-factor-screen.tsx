import { Button, Input } from "@wabou/components";
import { Text, View } from "@wabou/primitives";
import ArrowLeft from "lucide-solid/icons/arrow-left";
import KeyRound from "lucide-solid/icons/key-round";
import Mail from "lucide-solid/icons/mail";
import ShieldCheck from "lucide-solid/icons/shield-check";
import Smartphone from "lucide-solid/icons/smartphone";
import Usb from "lucide-solid/icons/usb";
import { For, Show } from "solid-js";
import type { TwoFactorOption } from "./model";

interface TwoFactorScreenProps {
  providers: TwoFactorOption[];
  provider: string;
  code: string;
  busy: boolean;
  error: string;
  notice: string;
  setProvider(provider: string): void;
  setCode(code: string): void;
  verify(): void;
  sendEmail(): void;
  cancel(): void;
}

function ProviderIcon(props: { id: string }) {
  if (props.id === "email") return <Mail size={17} />;
  if (props.id === "yubikey") return <Usb size={17} />;
  if (props.id === "authenticator") return <Smartphone size={17} />;
  return <KeyRound size={17} />;
}

export function TwoFactorScreen(props: TwoFactorScreenProps) {
  const selected = () => props.providers.find((provider) => provider.id === props.provider);

  return (
    <View class="h-full w-full flex items-center justify-center p-8 bg-slate-950 text-slate-100">
      <View class="w-96 rounded-xl border border-slate-800 bg-slate-900 p-6 flex flex-col gap-4">
        <View class="flex items-center gap-3">
          <View class="w-10 h-10 rounded-lg bg-sky-900 text-sky-300 flex items-center justify-center">
            <ShieldCheck size={21} />
          </View>
          <View class="flex flex-col gap-1">
            <Text class="text-xl font-semibold text-slate-100">Two-step login</Text>
            <Text class="text-xs text-slate-500">Verify this sign-in to continue.</Text>
          </View>
        </View>

        <View class="flex flex-col gap-2">
          <Text class="text-xs font-medium text-slate-400">Verification method</Text>
          <For each={props.providers}>
            {(provider) => (
              <Button
                class="w-full justify-start"
                variant={props.provider === provider.id ? "secondary" : "outline"}
                disabled={!provider.supported || props.busy}
                onClick={() => {
                  props.setProvider(provider.id);
                  props.setCode("");
                }}
              >
                <View class="w-full flex items-center gap-3">
                  <ProviderIcon id={provider.id} />
                  <Text class="min-w-0 flex-1 text-sm">
                    {provider.hint ? `${provider.label} · ${provider.hint}` : provider.label}
                  </Text>
                  <Show when={!provider.supported}>
                    <Text class="text-xs text-slate-500">Not supported</Text>
                  </Show>
                </View>
              </Button>
            )}
          </For>
        </View>

        <Show when={selected()?.supported}>
          <View class="flex flex-col gap-2">
            <Text class="text-xs font-medium text-slate-400">Verification code</Text>
            <Input
              placeholder={props.provider === "yubikey" ? "Touch your YubiKey" : "Enter code"}
              value={props.code}
              onInput={(event) => props.setCode(event.currentTarget.value)}
            />
          </View>
          <Show when={props.provider === "email"}>
            <Button variant="outline" disabled={props.busy} onClick={props.sendEmail}>
              <View class="flex items-center gap-2">
                <Mail size={16} />
                <Text>Send email code</Text>
              </View>
            </Button>
          </Show>
        </Show>

        <Show when={props.error}>
          <Text class="text-sm text-red-500">{props.error}</Text>
        </Show>
        <Show when={!props.error && props.notice}>
          <Text class="text-sm text-sky-400">{props.notice}</Text>
        </Show>

        <Button disabled={props.busy || !selected()?.supported} onClick={props.verify}>
          {props.busy ? "Verifying…" : "Verify and unlock"}
        </Button>
        <Button variant="ghost" disabled={props.busy} onClick={props.cancel}>
          <View class="flex items-center gap-2">
            <ArrowLeft size={16} />
            <Text>Back to sign in</Text>
          </View>
        </Button>
      </View>
    </View>
  );
}
