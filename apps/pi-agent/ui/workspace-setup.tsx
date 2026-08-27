import {
  Button,
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  DirectoryPicker,
  Field,
  FieldDescription,
  FieldLabel,
  Icon,
  Text,
  View,
} from "@wabou/ui";
import play from "lucide-static/icons/play.svg?raw";
import settings from "lucide-static/icons/settings-2.svg?raw";
import shield from "lucide-static/icons/shield-check.svg?raw";
import { createSignal } from "solid-js";
import { i18n, m } from "./i18n";

export function WorkspaceSetup(props: {
  path: string;
  error?: string;
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
    <View class="min-h-0 flex-1 overflow-y-auto bg-canvas">
      <View class="w-full max-w-3xl mx-auto p-8 gap-6">
        <View class="flex flex-row gap-3">
          <SetupStep
            number="1"
            title={i18n.message(m.setup_step_workspace, {})}
            detail={i18n.message(m.setup_step_workspace_detail, {})}
          />
          <SetupStep
            number="2"
            title={i18n.message(m.setup_step_runtime, {})}
            detail={i18n.message(m.setup_step_runtime_detail, {})}
          />
          <SetupStep
            number="3"
            title={i18n.message(m.setup_step_prompt, {})}
            detail={i18n.message(m.setup_step_prompt_detail, {})}
          />
        </View>

        <Card class="w-full shadow-md">
          <CardHeader class="items-center text-center">
            <CardTitle>{i18n.message(m.setup_welcome, {})}</CardTitle>
            <CardDescription>
              {i18n.message(m.setup_welcome_detail, {})}
            </CardDescription>
          </CardHeader>
          <CardContent class="gap-4">
            <Field>
              <FieldLabel>{i18n.message(m.setup_title, {})}</FieldLabel>
              <DirectoryPicker
                aria-label={i18n.message(m.workspace, {})}
                value={props.path}
                onValueChange={props.updatePath}
                placeholder={i18n.message(m.choose_repository, {})}
                browseLabel={i18n.message(m.browse, {})}
                browseAriaLabel={i18n.message(m.choose_repository, {})}
              />
              <FieldDescription>
                {i18n.message(m.setup_detail, {})}{" "}
                {i18n.message(m.setup_safety, {})}
              </FieldDescription>
            </Field>

            <View class="rounded-lg border border-subtle bg-canvas p-3 gap-2">
              <View class="flex flex-row items-center gap-2">
                <Icon source={shield} size={16} class="text-success-primary" />
                <Text class="font-medium text-sm">
                  {i18n.message(m.setup_configuration, {})}
                </Text>
              </View>
              <Text class="text-xs text-muted whitespace-normal">
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
              <Button
                variant="ghost"
                size="sm"
                class="self-start"
                onClick={props.openSettings}
              >
                <Icon source={settings} size={14} />
                {i18n.message(m.setup_change_settings, {})}
              </Button>
            </View>
            {props.error ? (
              <Text role="alert" class="text-sm text-danger whitespace-normal">
                {props.error}
              </Text>
            ) : null}
            <Button
              class="w-full"
              disabled={!props.path.trim() || starting()}
              onClick={() => void start()}
            >
              <Icon source={play} size={15} />
              {starting()
                ? i18n.message(m.starting, {})
                : i18n.message(m.start_agent, {})}
            </Button>
          </CardContent>
        </Card>
      </View>
    </View>
  );
}

function SetupStep(props: { number: string; title: string; detail: string }) {
  return (
    <View class="min-w-0 flex-1 rounded-lg border border-subtle bg-surface p-3 gap-1">
      <View class="w-6 h-6 rounded-full bg-selected flex items-center justify-center">
        <Text class="text-xs font-semibold text-accent">{props.number}</Text>
      </View>
      <Text class="text-sm font-medium">{props.title}</Text>
      <Text class="text-xs text-muted whitespace-normal">{props.detail}</Text>
    </View>
  );
}
