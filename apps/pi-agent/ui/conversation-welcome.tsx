import { Button, Icon, Text, View } from "@wabou/ui";
import bot from "lucide-static/icons/bot.svg?raw";
import gitPullRequest from "lucide-static/icons/git-pull-request.svg?raw";
import lightbulb from "lucide-static/icons/lightbulb.svg?raw";
import testTube from "lucide-static/icons/test-tube-2.svg?raw";
import { i18n, m } from "./i18n";

export function ConversationWelcome(props: {
  workspace: string;
  choosePrompt: (prompt: string) => void;
}) {
  const prompts = () => [
    {
      icon: gitPullRequest,
      title: i18n.message(m.starter_review_title, {}),
      prompt: i18n.message(m.starter_review_prompt, {}),
    },
    {
      icon: testTube,
      title: i18n.message(m.starter_verify_title, {}),
      prompt: i18n.message(m.starter_verify_prompt, {}),
    },
    {
      icon: lightbulb,
      title: i18n.message(m.starter_plan_title, {}),
      prompt: i18n.message(m.starter_plan_prompt, {}),
    },
  ];
  return (
    <View class="min-h-80 items-center justify-center gap-5 py-10">
      <View class="w-11 h-11 rounded-xl bg-selected flex items-center justify-center">
        <Icon source={bot} size={21} class="text-accent" />
      </View>
      <View class="items-center gap-1">
        <Text class="text-xl font-semibold">
          {i18n.message(m.empty_workspace_title, {
            workspace: workspaceName(props.workspace),
          })}
        </Text>
        <Text class="max-w-xl text-sm text-muted text-center whitespace-normal">
          {i18n.message(m.empty_detail, {})}
        </Text>
      </View>
      <View class="w-full max-w-3xl flex flex-row gap-3">
        {prompts().map((item) => (
          <Button
            variant="outline"
            class="h-auto min-w-0 flex-1 items-start justify-start p-4"
            onClick={() => props.choosePrompt(item.prompt)}
          >
            <Icon source={item.icon} size={16} class="text-accent" />
            <Text class="text-sm whitespace-normal">{item.title}</Text>
          </Button>
        ))}
      </View>
    </View>
  );
}

function workspaceName(path: string): string {
  const normalized = path.replace(/[\\/]+$/, "");
  return normalized.split(/[\\/]/).at(-1) || path;
}
