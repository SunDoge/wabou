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
  Input,
  PageHeader,
  PageViewport,
  Text,
  View,
} from "@wabou/ui";
import { i18n, m } from "./i18n";
import type { AgentWorkspace } from "./workspace";

export interface AgentDefaults {
  proxy: string;
  noProxy: string;
  provider: string;
  model: string;
}

export function SettingsPage(props: {
  value: AgentDefaults;
  update: (patch: Partial<AgentDefaults>) => void;
  agent: AgentWorkspace;
  updateAgent: (patch: Partial<AgentWorkspace>) => void;
  close: () => void;
}) {
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
              {i18n.message(m.current_agent, { name: props.agent.name })}
            </CardTitle>
            <CardDescription>
              {i18n.message(m.current_agent_detail, {})}
            </CardDescription>
          </CardHeader>
          <CardContent>
            <Field>
              <FieldLabel>{i18n.message(m.workspace, {})}</FieldLabel>
              <DirectoryPicker
                value={props.agent.cwd}
                onValueChange={(cwd) => props.updateAgent({ cwd })}
                placeholder={i18n.message(m.choose_repository, {})}
                browseLabel={i18n.message(m.browse, {})}
              />
            </Field>
            <Field>
              <FieldLabel>{i18n.message(m.provider, {})}</FieldLabel>
              <Input
                value={props.agent.provider}
                onInput={(event) =>
                  props.updateAgent({ provider: event.currentTarget.value })
                }
                placeholder={i18n.message(m.provider_placeholder, {})}
              />
            </Field>
            <Field>
              <FieldLabel>{i18n.message(m.model, {})}</FieldLabel>
              <Input
                value={props.agent.model}
                onInput={(event) =>
                  props.updateAgent({ model: event.currentTarget.value })
                }
                placeholder={i18n.message(m.model_optional, {})}
              />
            </Field>
            <Field>
              <FieldLabel>{i18n.message(m.proxy_url, {})}</FieldLabel>
              <Input
                value={props.agent.proxy}
                onInput={(event) =>
                  props.updateAgent({ proxy: event.currentTarget.value })
                }
                placeholder={i18n.message(m.proxy_placeholder, {})}
              />
            </Field>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>{i18n.message(m.language, {})}</CardTitle>
            <CardDescription>
              {i18n.message(m.language_detail, {})}
            </CardDescription>
          </CardHeader>
          <CardContent class="flex-row">
            <Button
              variant={i18n.locale() === "en" ? "default" : "outline"}
              onClick={() => i18n.set("en")}
            >
              {i18n.message(m.english, {})}
            </Button>
            <Button
              variant={i18n.locale() === "zh" ? "default" : "outline"}
              onClick={() => i18n.set("zh")}
            >
              {i18n.message(m.chinese, {})}
            </Button>
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
                value={props.value.provider}
                placeholder={i18n.message(m.provider_placeholder, {})}
                onInput={(event) =>
                  props.update({ provider: event.currentTarget.value })
                }
              />
            </Field>
            <Field>
              <FieldLabel>{i18n.message(m.model, {})}</FieldLabel>
              <Input
                aria-label="Default model"
                value={props.value.model}
                placeholder={i18n.message(m.model_optional, {})}
                onInput={(event) =>
                  props.update({ model: event.currentTarget.value })
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
                value={props.value.proxy}
                placeholder="http://127.0.0.1:7890"
                onInput={(event) =>
                  props.update({ proxy: event.currentTarget.value })
                }
              />
            </Field>
            <Field>
              <FieldLabel>{i18n.message(m.proxy_bypass, {})}</FieldLabel>
              <Input
                aria-label="Default proxy bypass list"
                value={props.value.noProxy}
                onInput={(event) =>
                  props.update({ noProxy: event.currentTarget.value })
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
      </View>
    </PageViewport>
  );
}
