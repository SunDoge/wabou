import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
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
  Input,
  PageHeader,
  PageViewport,
  Switch,
  Text,
  View,
} from "@wabou/ui";
import type { AgentQueueMode, AgentViewState } from "./agent-state";
import { i18n, m } from "./i18n";
import { SessionBehaviorSettings } from "./session-behavior-settings";
import type { AgentWorkspace } from "./workspace";

export interface AppSettings {
  locale: "en" | "zh";
  proxy: string;
  noProxy: string;
  provider: string;
  model: string;
  subagentsEnabled: boolean;
}

export function SettingsPage(props: {
  app: AppSettings;
  updateApp: (patch: Partial<AppSettings>) => void;
  project: AgentWorkspace;
  state: AgentViewState;
  updateProject: (patch: Partial<AgentWorkspace>) => void;
  close: () => void;
  deleteProject: () => void;
  setAutoCompaction: (enabled: boolean) => void;
  setSteeringMode: (mode: AgentQueueMode) => void;
  setFollowUpMode: (mode: AgentQueueMode) => void;
}) {
  const setLocale = (locale: AppSettings["locale"]) => {
    i18n.set(locale);
    props.updateApp({ locale });
  };
  return (
    <PageViewport class="bg-canvas" contentClass="p-8">
      <View class="w-full max-w-3xl mx-auto flex flex-col gap-6 pb-8">
        <PageHeader
          title={i18n.message(m.settings, {})}
          description={i18n.message(m.settings_intro, {})}
          actions={
            <Button variant="outline" onClick={props.close}>
              {i18n.message(m.back_to_agents, {})}
            </Button>
          }
        />

        <Card>
          <CardHeader>
            <CardTitle>
              {i18n.message(m.current_agent, { name: props.project.name })}
            </CardTitle>
            <CardDescription>
              {i18n.message(m.current_agent_detail, {})}
            </CardDescription>
          </CardHeader>
          <CardContent>
            <Field>
              <FieldLabel>{i18n.message(m.agent_name, {})}</FieldLabel>
              <Input
                aria-label={i18n.message(m.agent_name, {})}
                value={props.project.name}
                onInput={(event) =>
                  props.updateProject({ name: event.currentTarget.value })
                }
              />
            </Field>
            <Field>
              <FieldLabel>{i18n.message(m.workspace, {})}</FieldLabel>
              <DirectoryPicker
                aria-label={i18n.message(m.workspace, {})}
                value={props.project.cwd}
                onValueChange={(cwd) => props.updateProject({ cwd })}
                placeholder={i18n.message(m.choose_repository, {})}
                browseLabel={i18n.message(m.browse, {})}
              />
            </Field>
            <Field>
              <FieldLabel>{i18n.message(m.provider, {})}</FieldLabel>
              <Input
                aria-label={i18n.message(m.provider, {})}
                value={props.project.provider}
                onInput={(event) =>
                  props.updateProject({ provider: event.currentTarget.value })
                }
                placeholder={i18n.message(m.provider_placeholder, {})}
              />
            </Field>
            <Field>
              <FieldLabel>{i18n.message(m.model, {})}</FieldLabel>
              <Input
                aria-label={i18n.message(m.model, {})}
                value={props.project.model}
                onInput={(event) =>
                  props.updateProject({ model: event.currentTarget.value })
                }
                placeholder={i18n.message(m.model_optional, {})}
              />
            </Field>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>{i18n.message(m.session_behavior, {})}</CardTitle>
            <CardDescription>
              {i18n.message(m.session_behavior_detail, {})}
            </CardDescription>
          </CardHeader>
          <CardContent>
            <SessionBehaviorSettings
              state={props.state}
              setAutoCompaction={props.setAutoCompaction}
              setSteeringMode={props.setSteeringMode}
              setFollowUpMode={props.setFollowUpMode}
            />
          </CardContent>
        </Card>

        <Text class="text-xs font-semibold tracking-wide text-muted">
          {i18n.message(m.application_settings, {})}
        </Text>

        <Card>
          <CardHeader>
            <CardTitle>{i18n.message(m.language, {})}</CardTitle>
            <CardDescription>
              {i18n.message(m.language_detail, {})}
            </CardDescription>
          </CardHeader>
          <CardContent class="flex-row">
            <Button
              variant={props.app.locale === "en" ? "default" : "outline"}
              onClick={() => setLocale("en")}
            >
              {i18n.message(m.english, {})}
            </Button>
            <Button
              variant={props.app.locale === "zh" ? "default" : "outline"}
              onClick={() => setLocale("zh")}
            >
              {i18n.message(m.chinese, {})}
            </Button>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>{i18n.message(m.subagents, {})}</CardTitle>
            <CardDescription>
              {i18n.message(m.subagents_detail, {})}
            </CardDescription>
          </CardHeader>
          <CardContent>
            <Switch
              label={i18n.message(m.enable_subagents, {})}
              checked={props.app.subagentsEnabled}
              onCheckedChange={(subagentsEnabled) =>
                props.updateApp({ subagentsEnabled })
              }
            />
            <FieldDescription>
              {i18n.message(m.enable_subagents_detail, {})}
            </FieldDescription>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>{i18n.message(m.default_provider, {})}</CardTitle>
            <CardDescription>
              {i18n.message(m.provider_detail, {})}
            </CardDescription>
          </CardHeader>
          <CardContent>
            <Field>
              <FieldLabel>{i18n.message(m.provider, {})}</FieldLabel>
              <Input
                aria-label="Default provider"
                value={props.app.provider}
                placeholder={i18n.message(m.provider_placeholder, {})}
                onInput={(event) =>
                  props.updateApp({ provider: event.currentTarget.value })
                }
              />
            </Field>
            <Field>
              <FieldLabel>{i18n.message(m.model, {})}</FieldLabel>
              <Input
                aria-label="Default model"
                value={props.app.model}
                placeholder={i18n.message(m.model_optional, {})}
                onInput={(event) =>
                  props.updateApp({ model: event.currentTarget.value })
                }
              />
            </Field>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>{i18n.message(m.default_proxy, {})}</CardTitle>
            <CardDescription>
              {i18n.message(m.proxy_detail, {})}
            </CardDescription>
          </CardHeader>
          <CardContent>
            <Field>
              <FieldLabel>{i18n.message(m.proxy_url, {})}</FieldLabel>
              <Input
                aria-label="Default proxy URL"
                value={props.app.proxy}
                placeholder="http://127.0.0.1:7890"
                onInput={(event) =>
                  props.updateApp({ proxy: event.currentTarget.value })
                }
              />
            </Field>
            <Field>
              <FieldLabel>{i18n.message(m.proxy_bypass, {})}</FieldLabel>
              <Input
                aria-label="Default proxy bypass list"
                value={props.app.noProxy}
                onInput={(event) =>
                  props.updateApp({ noProxy: event.currentTarget.value })
                }
              />
              <FieldDescription>127.0.0.1, localhost</FieldDescription>
            </Field>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>{i18n.message(m.runtime, {})}</CardTitle>
            <CardDescription>
              {i18n.message(m.runtime_kind, {})}
            </CardDescription>
          </CardHeader>
          <CardContent>
            <Text class="text-sm text-muted whitespace-normal">
              {i18n.message(m.runtime_detail, {})}
            </Text>
          </CardContent>
        </Card>

        <Card class="border-danger">
          <CardHeader>
            <CardTitle>{i18n.message(m.danger_zone, {})}</CardTitle>
            <CardDescription>
              {i18n.message(m.delete_agent_detail, {})}
            </CardDescription>
          </CardHeader>
          <CardContent class="items-start">
            <AlertDialog
              aria-label={i18n.message(m.delete_agent, {})}
              trigger={(trigger) => (
                <Button variant="destructive" {...trigger}>
                  {i18n.message(m.delete_agent, {})}
                </Button>
              )}
            >
              <AlertDialogHeader>
                <AlertDialogTitle>
                  {i18n.message(m.delete_agent_confirm, {
                    name: props.project.name,
                  })}
                </AlertDialogTitle>
                <AlertDialogDescription>
                  {i18n.message(m.delete_agent_files_safe, {})}
                </AlertDialogDescription>
              </AlertDialogHeader>
              <AlertDialogFooter>
                <AlertDialogCancel>
                  {i18n.message(m.cancel, {})}
                </AlertDialogCancel>
                <AlertDialogAction
                  variant="destructive"
                  aria-label={i18n.message(m.delete_agent_confirm, {
                    name: props.project.name,
                  })}
                  onClick={props.deleteProject}
                >
                  {i18n.message(m.delete_agent, {})}
                </AlertDialogAction>
              </AlertDialogFooter>
            </AlertDialog>
          </CardContent>
        </Card>
      </View>
    </PageViewport>
  );
}
