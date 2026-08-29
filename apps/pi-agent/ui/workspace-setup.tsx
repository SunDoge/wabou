import {
  Button,
  CodeBlock,
  DirectoryPicker,
  Icon,
  Text,
  View,
} from "@wabou/ui";
import bot from "lucide-static/icons/bot.svg?raw";
import folder from "lucide-static/icons/folder.svg?raw";
import play from "lucide-static/icons/play.svg?raw";
import settings from "lucide-static/icons/settings-2.svg?raw";
import shield from "lucide-static/icons/shield-check.svg?raw";
import { createSignal } from "solid-js";
import { i18n, m } from "./i18n";

export function WorkspaceSetup(props: {
  path: string;
  error?: string;
  runtimeLogs?: readonly string[];
  provider?: string;
  model?: string;
  proxy?: string;
  updatePath: (path: string) => void;
  start: () => Promise<unknown>;
  openSettings: () => void;
}) {
  const [starting, setStarting] = createSignal(false);
  const start = async () => {
    if (starting() || !props.path.trim()) return;
    setStarting(true);
    try {
      await props.start();
    } finally {
      setStarting(false);
    }
  };
  return (
    <View class="min-h-0 flex-1 overflow-y-auto bg-canvas flex items-center justify-center">
      <View class="w-full max-w-2xl min-w-0 mx-auto px-8 py-12 gap-5">
        <View class="w-full min-w-0 flex flex-row items-center gap-3 mb-1">
          <View class="w-11 h-11 flex-none rounded-xl bg-control flex items-center justify-center">
            <Icon source={bot} size={21} class="text-secondary" />
          </View>
          <View class="min-w-0 flex-1 gap-2">
            <Text class="text-lg font-semibold text-primary">
              {i18n.message(m.setup_welcome, {})}
            </Text>
            <Text class="text-sm text-muted whitespace-normal">
              {i18n.message(m.setup_welcome_detail, {})}
            </Text>
          </View>
        </View>

        <View class="w-full min-w-0 rounded-xl border border-subtle bg-surface p-4 gap-3">
          <View class="min-w-0 flex flex-row items-center gap-3">
            <View class="w-9 h-9 flex-none rounded-lg bg-control flex items-center justify-center">
              <Icon source={folder} size={17} class="text-secondary" />
            </View>
            <View class="min-w-0 flex-1 gap-0">
              <Text class="text-sm font-medium">
                {i18n.message(m.setup_title, {})}
              </Text>
              <Text class="text-xs text-muted whitespace-normal">
                {i18n.message(m.setup_detail, {})}
              </Text>
            </View>
          </View>
          <DirectoryPicker
            aria-label={i18n.message(m.workspace, {})}
            value={props.path}
            onValueChange={props.updatePath}
            placeholder={i18n.message(m.choose_repository, {})}
            browseLabel={i18n.message(m.browse, {})}
            browseAriaLabel={i18n.message(m.choose_repository, {})}
          />
          <View class="min-w-0 flex flex-row items-center justify-between gap-3 border-t border-subtle pt-3">
            <View class="min-w-0 flex-1 flex flex-row items-center gap-2">
              <Icon
                source={shield}
                size={14}
                class="flex-none text-success-primary"
              />
              <Text class="min-w-0 flex-1 truncate text-xs text-muted">
                {i18n.message(m.setup_configuration_detail, {
                  provider:
                    props.provider?.trim() ||
                    i18n.message(m.setup_automatic, {}),
                  model:
                    props.model?.trim() || i18n.message(m.setup_automatic, {}),
                  proxy:
                    props.proxy?.trim() ||
                    i18n.message(m.setup_system_network, {}),
                })}
              </Text>
            </View>
            <Button variant="ghost" size="sm" onClick={props.openSettings}>
              <Icon source={settings} size={13} />
              {i18n.message(m.setup_change_settings, {})}
            </Button>
          </View>
          <Button
            variant="outline"
            class="w-full border-strong bg-control text-primary shadow-none"
            disabled={!props.path.trim() || starting()}
            onClick={() => void start()}
          >
            <Icon source={play} size={15} />
            {starting()
              ? i18n.message(m.starting, {})
              : i18n.message(m.start_agent, {})}
          </Button>
        </View>

        {props.error ? (
          <Text role="alert" class="text-sm text-danger whitespace-normal">
            {props.error}
          </Text>
        ) : null}
        {props.runtimeLogs && props.runtimeLogs.length > 0 ? (
          <View class="w-full min-w-0 max-h-40 overflow-y-auto">
            <CodeBlock
              aria-label={i18n.message(m.runtime_output, {})}
              code={props.runtimeLogs.slice(-12).join("\n")}
              language="log"
            />
          </View>
        ) : null}
        <View class="w-full min-w-0 pt-1">
          <Text class="text-xs text-muted whitespace-normal">
            {i18n.message(m.setup_safety, {})}
          </Text>
        </View>
      </View>
    </View>
  );
}
