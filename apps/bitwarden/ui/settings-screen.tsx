import { Text, View } from "@wabou/primitives";
import BugOff from "lucide-solid/icons/bug-off";
import Clipboard from "lucide-solid/icons/clipboard";
import Clock3 from "lucide-solid/icons/clock-3";
import Info from "lucide-solid/icons/info";
import Server from "lucide-solid/icons/server";
import ShieldCheck from "lucide-solid/icons/shield-check";
import UserRound from "lucide-solid/icons/user-round";
import type { JSX } from "solid-js";

interface SettingsScreenProps {
  email: string;
}

interface SettingRowProps {
  icon: JSX.Element;
  title: string;
  description: string;
  value: string;
  border?: boolean;
}

function SettingRow(props: SettingRowProps) {
  return (
    <View class={`p-4 flex items-center gap-4 ${props.border ? "border-t border-slate-800" : ""}`}>
      <View class="w-9 h-9 shrink-0 rounded-md bg-slate-800 text-slate-400 flex items-center justify-center">
        {props.icon}
      </View>
      <View class="min-w-0 flex-1 flex flex-col gap-1">
        <Text class="text-sm font-medium text-slate-100">{props.title}</Text>
        <Text class="text-xs text-slate-500">{props.description}</Text>
      </View>
      <Text class="shrink-0 text-sm text-slate-300">{props.value}</Text>
    </View>
  );
}

export function SettingsScreen(props: SettingsScreenProps) {
  return (
    <View class="min-w-0 flex-1 flex flex-col bg-slate-950 text-slate-100">
      <View class="h-16 shrink-0 border-b border-slate-800 px-6 flex items-center gap-2">
        <Text class="text-lg font-semibold text-slate-100">Settings</Text>
        <Text class="text-sm text-slate-500">Desktop preferences and security policy</Text>
      </View>
      <View class="min-h-0 flex-1 overflow-y-auto p-8">
        <View class="w-full flex flex-col gap-7">
          <View class="flex flex-col gap-3">
            <View class="flex flex-col gap-1">
              <Text class="text-sm font-semibold text-slate-200">Security</Text>
              <Text class="text-xs text-slate-500">
                These safeguards are fixed while the application is an experimental read-only client.
              </Text>
            </View>
            <View class="rounded-lg border border-slate-800 bg-slate-900">
              <SettingRow
                icon={<Clock3 size={17} />}
                title="Auto-lock"
                description="Drop the native SDK session and rendered vault."
                value="5 minutes"
              />
              <SettingRow
                border
                icon={<Clipboard size={17} />}
                title="Clear clipboard"
                description="Clear only the credential this app copied."
                value="30 seconds"
              />
              <SettingRow
                border
                icon={<BugOff size={17} />}
                title="Developer tools"
                description="Keep decrypted data out of debug snapshots."
                value="Disabled"
              />
            </View>
          </View>

          <View class="flex flex-col gap-3">
            <Text class="text-sm font-semibold text-slate-200">Account</Text>
            <View class="rounded-lg border border-slate-800 bg-slate-900">
              <SettingRow
                icon={<UserRound size={17} />}
                title="Signed-in account"
                description="Current native SDK session."
                value={props.email}
              />
              <SettingRow
                border
                icon={<Server size={17} />}
                title="Vault access"
                description="Sync and decrypt with the pinned Rust SDK."
                value="Read only"
              />
            </View>
          </View>

          <View class="flex flex-col gap-3">
            <Text class="text-sm font-semibold text-slate-200">About</Text>
            <View class="rounded-lg border border-slate-800 bg-slate-900">
              <SettingRow
                icon={<ShieldCheck size={17} />}
                title="Wabou Vault"
                description="Experimental native desktop proof of capability."
                value="0.1.0"
              />
              <SettingRow
                border
                icon={<Info size={17} />}
                title="Bitwarden SDK"
                description="Pinned dependency revision; internal and unstable upstream API."
                value="fbd21679"
              />
            </View>
          </View>
        </View>
      </View>
    </View>
  );
}
