import {
  Alert,
  Button,
  Card,
  CardContent,
  CodeBlock,
  ContentState,
  DirectoryPicker,
  Icon,
  IconFrame,
  Onboarding,
  OnboardingDescription,
  OnboardingFooter,
  OnboardingHeader,
  OnboardingHeading,
  OnboardingTitle,
  Text,
  View,
} from "@wabou/ui";
import bot from "lucide-static/icons/bot.svg?raw";
import circleAlert from "lucide-static/icons/circle-alert.svg?raw";
import folder from "lucide-static/icons/folder.svg?raw";
import play from "lucide-static/icons/play.svg?raw";
import settings from "lucide-static/icons/settings-2.svg?raw";
import shield from "lucide-static/icons/shield-check.svg?raw";
import { createSignal, omit, Show } from "solid-js";
import { i18n, m } from "./i18n";

export interface WorkspaceSetupProps {
  path: string;
  error?: string;
  runtimeLogs?: readonly string[];
  provider?: string;
  model?: string;
  proxy?: string;
  updatePath: (path: string) => void;
  start: () => Promise<unknown>;
  openSettings: () => void;
}

export function WorkspaceSetupBoundary(
  props: WorkspaceSetupProps & { pending: boolean },
) {
  const setup = omit(props, "pending");
  return (
    <Show
      when={!props.pending}
      fallback={
        <ContentState
          state="loading"
          title={i18n.message(m.preparing_workspace, {})}
          description={i18n.message(m.preparing_workspace_detail, {})}
          class="bg-canvas"
        />
      }
    >
      <WorkspaceSetup {...setup} />
    </Show>
  );
}

export function WorkspaceSetup(props: WorkspaceSetupProps) {
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
    <Onboarding aria-label={i18n.message(m.setup_welcome, {})}>
      <OnboardingHeader>
        <IconFrame source={bot} size="default" variant="muted" iconSize={21} />
        <OnboardingHeading>
          <OnboardingTitle>{i18n.message(m.setup_welcome, {})}</OnboardingTitle>
          <OnboardingDescription>
            {i18n.message(m.setup_welcome_detail, {})}
          </OnboardingDescription>
        </OnboardingHeading>
      </OnboardingHeader>

      <Card class="w-full">
        <CardContent class="gap-3 px-4 pt-4 pb-4">
          <View class="min-w-0 flex flex-row items-center gap-3">
            <IconFrame
              source={folder}
              size="sm"
              variant="muted"
              iconSize={17}
            />
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
            <Button
              variant="ghost"
              size="sm"
              aria-label={i18n.message(m.setup_change_settings, {})}
              onClick={props.openSettings}
            >
              <Icon source={settings} size={13} />
              {i18n.message(m.setup_change_settings, {})}
            </Button>
          </View>
          <Button
            variant="default"
            class="w-full"
            aria-label={i18n.message(m.start_agent, {})}
            disabled={!props.path.trim()}
            loading={starting()}
            loadingLabel={i18n.message(m.starting, {})}
            onClick={() => void start()}
          >
            <Icon source={play} size={15} />
            {i18n.message(m.start_agent, {})}
          </Button>
        </CardContent>
      </Card>

      {props.error ? (
        <Alert
          variant="destructive"
          aria-label={props.error}
          icon={
            <Icon source={circleAlert} size={16} class="text-danger-primary" />
          }
        >
          {props.error}
        </Alert>
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
      <OnboardingFooter>
        <Text class="text-xs text-muted whitespace-normal">
          {i18n.message(m.setup_safety, {})}
        </Text>
      </OnboardingFooter>
    </Onboarding>
  );
}
