import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  Button,
  DirectoryPicker,
  Field,
  FieldDescription,
  FieldLabel,
  Input,
  mergeClasses,
  PageHeader,
  PageViewport,
  Separator,
  Switch,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
  Text,
  View,
} from "@wabou/ui";
import type { JSX } from "solid-js";
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

function SettingsSection(props: {
  title: string;
  description: string;
  children: JSX.Element;
  class?: string;
}) {
  return (
    <View class="min-w-0 flex flex-row items-start gap-7">
      <View class="w-52 flex-none flex flex-col gap-1 pt-1">
        <Text role="heading" class="text-base font-semibold text-primary">
          {props.title}
        </Text>
        <Text class="text-sm text-secondary whitespace-normal">
          {props.description}
        </Text>
      </View>
      <View
        class={mergeClasses(
          "min-w-0 flex-1 flex flex-col gap-5 rounded-xl border border-subtle bg-surface px-5 py-5 shadow-xs",
          props.class,
        )}
      >
        {props.children}
      </View>
    </View>
  );
}

function SettingsGroup(props: {
  title: string;
  description?: string;
  children: JSX.Element;
}) {
  return (
    <View class="min-w-0 flex flex-col gap-4">
      <View class="min-w-0 flex flex-col gap-1">
        <Text class="text-sm font-semibold text-primary">{props.title}</Text>
        {props.description ? (
          <Text class="text-sm text-secondary whitespace-normal">
            {props.description}
          </Text>
        ) : null}
      </View>
      <View class="min-w-0 flex flex-col gap-4">{props.children}</View>
    </View>
  );
}

export function SettingsPage(props: {
  app: AppSettings;
  updateApp: (patch: Partial<AppSettings>) => void;
  project: AgentWorkspace;
  state: AgentViewState;
  canDeleteProject: boolean;
  updateProject: (patch: Partial<AgentWorkspace>) => void;
  close: () => void;
  deleteProject: () => void;
  setAutoCompaction: (enabled: boolean) => void;
  setSteeringMode: (mode: AgentQueueMode) => void;
  setFollowUpMode: (mode: AgentQueueMode) => void;
  defaultSection?: "project" | "application";
}) {
  const setLocale = (locale: AppSettings["locale"]) => {
    i18n.set(locale);
    props.updateApp({ locale });
  };
  return (
    <PageViewport class="bg-canvas" contentClass="p-8">
      <View class="w-full max-w-5xl mx-auto flex flex-col gap-6 pb-8">
        <PageHeader
          title={i18n.message(m.settings, {})}
          description={i18n.message(m.settings_intro, {})}
          actions={
            <Button variant="outline" onClick={props.close}>
              {i18n.message(m.back_to_agents, {})}
            </Button>
          }
        />
        <Tabs
          defaultValue={props.defaultSection ?? "project"}
          class="w-full gap-6"
        >
          <TabsList aria-label={i18n.message(m.settings, {})}>
            <TabsTrigger value="project">
              {i18n.message(m.current_agent, { name: props.project.name })}
            </TabsTrigger>
            <TabsTrigger value="application">
              {i18n.message(m.application_settings, {})}
            </TabsTrigger>
          </TabsList>

          <TabsContent value="project" class="min-w-0 flex flex-col gap-7">
            <SettingsSection
              title={i18n.message(m.workspace, {})}
              description={i18n.message(m.current_agent_detail, {})}
            >
              <SettingsGroup title={i18n.message(m.workspace, {})}>
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
                      props.updateProject({
                        provider: event.currentTarget.value,
                      })
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
              </SettingsGroup>
              <Separator />
              <SettingsGroup
                title={i18n.message(m.session_behavior, {})}
                description={i18n.message(m.session_behavior_detail, {})}
              >
                <SessionBehaviorSettings
                  state={props.state}
                  setAutoCompaction={props.setAutoCompaction}
                  setSteeringMode={props.setSteeringMode}
                  setFollowUpMode={props.setFollowUpMode}
                />
              </SettingsGroup>
            </SettingsSection>

            <SettingsSection
              title={i18n.message(m.danger_zone, {})}
              description={i18n.message(m.delete_agent_detail, {})}
              class="border-danger"
            >
              <View class="min-w-0 flex flex-row items-center justify-between gap-4">
                <Text class="min-w-0 flex-1 text-sm text-secondary whitespace-normal">
                  {props.canDeleteProject
                    ? i18n.message(m.delete_agent_files_safe, {})
                    : i18n.message(m.delete_last_project_disabled, {})}
                </Text>
                <AlertDialog
                  aria-label={i18n.message(m.delete_agent, {})}
                  trigger={(trigger) => (
                    <Button
                      variant="destructive"
                      {...trigger}
                      disabled={!props.canDeleteProject}
                    >
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
                      disabled={!props.canDeleteProject}
                      onClick={props.deleteProject}
                    >
                      {i18n.message(m.delete_agent, {})}
                    </AlertDialogAction>
                  </AlertDialogFooter>
                </AlertDialog>
              </View>
            </SettingsSection>
          </TabsContent>

          <TabsContent value="application" class="min-w-0 flex flex-col gap-7">
            <SettingsSection
              title={i18n.message(m.application_settings, {})}
              description={i18n.message(m.application_settings_detail, {})}
            >
              <SettingsGroup
                title={i18n.message(m.language, {})}
                description={i18n.message(m.language_detail, {})}
              >
                <View class="min-w-0 flex flex-row items-center gap-2">
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
                </View>
              </SettingsGroup>
              <Separator />
              <SettingsGroup
                title={i18n.message(m.subagents, {})}
                description={i18n.message(m.subagents_detail, {})}
              >
                <Switch
                  label={i18n.message(m.enable_subagents, {})}
                  checked={props.app.subagentsEnabled}
                  onCheckedChange={(subagentsEnabled) =>
                    props.updateApp({ subagentsEnabled })
                  }
                />
                <FieldDescription class="text-secondary">
                  {i18n.message(m.enable_subagents_detail, {})}
                </FieldDescription>
              </SettingsGroup>
              <Separator />
              <SettingsGroup
                title={i18n.message(m.default_provider, {})}
                description={i18n.message(m.provider_detail, {})}
              >
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
              </SettingsGroup>
              <Separator />
              <SettingsGroup
                title={i18n.message(m.default_proxy, {})}
                description={i18n.message(m.proxy_detail, {})}
              >
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
                  <FieldDescription class="text-secondary">
                    127.0.0.1, localhost
                  </FieldDescription>
                </Field>
              </SettingsGroup>
              <Separator />
              <SettingsGroup
                title={i18n.message(m.runtime, {})}
                description={i18n.message(m.runtime_kind, {})}
              >
                <Text class="text-sm text-secondary whitespace-normal">
                  {i18n.message(m.runtime_detail, {})}
                </Text>
              </SettingsGroup>
            </SettingsSection>
          </TabsContent>
        </Tabs>
      </View>
    </PageViewport>
  );
}
