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
    <View class="min-h-80 items-center justify-center gap-4 py-10">
      <View class="w-10 h-10 rounded-xl bg-control flex items-center justify-center">
        <Icon source={bot} size={19} class="text-secondary" />
      </View>
      <View class="items-center gap-1">
        <Text class="text-lg font-semibold text-primary">
          {i18n.message(m.empty_workspace_title, {
            workspace: workspaceName(props.workspace),
          })}
        </Text>
        <Text class="max-w-xl text-sm text-muted text-center whitespace-normal">
          {i18n.message(m.empty_detail, {})}
        </Text>
      </View>
      <View class="w-full max-w-2xl flex flex-row gap-2">
        {prompts().map((item) => (
          <Button
            variant="outline"
            class="h-auto min-w-0 flex-1 items-start justify-start border-subtle bg-transparent p-3 shadow-none"
            onClick={() => props.choosePrompt(item.prompt)}
          >
            <Icon source={item.icon} size={15} class="text-muted" />
            <Text class="text-sm text-secondary whitespace-normal">
              {item.title}
            </Text>
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
