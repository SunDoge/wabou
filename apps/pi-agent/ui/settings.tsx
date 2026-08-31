import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  Button,
  createContainerMatch,
  DirectoryPicker,
  FieldDescription,
  Input,
  LabeledField,
  PageHeader,
  PageViewport,
  RadioGroup,
  RadioGroupItem,
  Separator,
  SettingsGroup,
  SettingsSection,
  Switch,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
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
  canDeleteProject: boolean;
  updateProject: (patch: Partial<AgentWorkspace>) => void;
  close: () => void;
  deleteProject: () => void;
  setAutoCompaction: (enabled: boolean) => void;
  setSteeringMode: (mode: AgentQueueMode) => void;
  setFollowUpMode: (mode: AgentQueueMode) => void;
  defaultSection?: "project" | "application";
}) {
  const compact = createContainerMatch({ maxWidth: 640 });
  const setLocale = (locale: AppSettings["locale"]) => {
    i18n.set(locale);
    props.updateApp({ locale });
  };
  return (
    <PageViewport class="bg-canvas" contentClass="p-8">
      <View
        ref={compact.ref}
        class="w-full max-w-5xl mx-auto flex flex-col gap-6 pb-8"
      >
        <PageHeader
          title={i18n.message(m.settings, {})}
          description={i18n.message(m.settings_intro, {})}
          stacked={compact.matches()}
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
              {i18n.message(m.project_settings, {})}
            </TabsTrigger>
            <TabsTrigger value="application">
              {i18n.message(m.application_settings, {})}
            </TabsTrigger>
          </TabsList>

          <TabsContent value="project" class="min-w-0 flex flex-col gap-7">
            <SettingsSection
              title={i18n.message(m.workspace, {})}
              description={i18n.message(m.current_agent_detail, {})}
              stacked={compact.matches()}
            >
              <View class="min-w-0 flex flex-col gap-4">
                <LabeledField
                  label={i18n.message(m.agent_name, {})}
                  renderControl={(ref) => (
                    <Input
                      ref={ref}
                      aria-label={i18n.message(m.agent_name, {})}
                      value={props.project.name}
                      onInput={(event) =>
                        props.updateProject({ name: event.currentTarget.value })
                      }
                    />
                  )}
                />
                <LabeledField
                  label={i18n.message(m.workspace, {})}
                  renderControl={(ref) => (
                    <DirectoryPicker
                      ref={ref}
                      aria-label={i18n.message(m.workspace, {})}
                      value={props.project.cwd}
                      onValueChange={(cwd) => props.updateProject({ cwd })}
                      placeholder={i18n.message(m.choose_repository, {})}
                      browseLabel={i18n.message(m.browse, {})}
                    />
                  )}
                />
                <LabeledField
                  label={i18n.message(m.provider, {})}
                  renderControl={(ref) => (
                    <Input
                      ref={ref}
                      aria-label={i18n.message(m.provider, {})}
                      value={props.project.provider}
                      onInput={(event) =>
                        props.updateProject({
                          provider: event.currentTarget.value,
                        })
                      }
                      placeholder={i18n.message(m.provider_placeholder, {})}
                    />
                  )}
                />
                <LabeledField
                  label={i18n.message(m.model, {})}
                  renderControl={(ref) => (
                    <Input
                      ref={ref}
                      aria-label={i18n.message(m.model, {})}
                      value={props.project.model}
                      onInput={(event) =>
                        props.updateProject({
                          model: event.currentTarget.value,
                        })
                      }
                      placeholder={i18n.message(m.model_optional, {})}
                    />
                  )}
                />
              </View>
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
              stacked={compact.matches()}
              contentClass="border-danger"
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
              stacked={compact.matches()}
            >
              <SettingsGroup
                title={i18n.message(m.language, {})}
                description={i18n.message(m.language_detail, {})}
              >
                <RadioGroup
                  appearance="segment"
                  orientation="horizontal"
                  value={props.app.locale}
                  aria-label={i18n.message(m.language, {})}
                  class="w-full max-w-sm"
                  onValueChange={(value) =>
                    setLocale(value as AppSettings["locale"])
                  }
                >
                  <RadioGroupItem
                    value="en"
                    label={i18n.message(m.english, {})}
                  />
                  <RadioGroupItem
                    value="zh"
                    label={i18n.message(m.chinese, {})}
                  />
                </RadioGroup>
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
                <LabeledField
                  label={i18n.message(m.provider, {})}
                  renderControl={(ref) => (
                    <Input
                      ref={ref}
                      aria-label="Default provider"
                      value={props.app.provider}
                      placeholder={i18n.message(m.provider_placeholder, {})}
                      onInput={(event) =>
                        props.updateApp({ provider: event.currentTarget.value })
                      }
                    />
                  )}
                />
                <LabeledField
                  label={i18n.message(m.model, {})}
                  renderControl={(ref) => (
                    <Input
                      ref={ref}
                      aria-label="Default model"
                      value={props.app.model}
                      placeholder={i18n.message(m.model_optional, {})}
                      onInput={(event) =>
                        props.updateApp({ model: event.currentTarget.value })
                      }
                    />
                  )}
                />
              </SettingsGroup>
              <Separator />
              <SettingsGroup
                title={i18n.message(m.default_proxy, {})}
                description={i18n.message(m.proxy_detail, {})}
              >
                <LabeledField
                  label={i18n.message(m.proxy_url, {})}
                  renderControl={(ref) => (
                    <Input
                      ref={ref}
                      aria-label="Default proxy URL"
                      value={props.app.proxy}
                      placeholder="http://127.0.0.1:7890"
                      onInput={(event) =>
                        props.updateApp({ proxy: event.currentTarget.value })
                      }
                    />
                  )}
                />
                <LabeledField
                  label={i18n.message(m.proxy_bypass, {})}
                  description="127.0.0.1, localhost"
                  renderControl={(ref) => (
                    <Input
                      ref={ref}
                      aria-label="Default proxy bypass list"
                      value={props.app.noProxy}
                      onInput={(event) =>
                        props.updateApp({ noProxy: event.currentTarget.value })
                      }
                    />
                  )}
                />
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
