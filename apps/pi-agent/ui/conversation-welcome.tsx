import { Button, Icon, Text, View } from "@wabou/ui";
import bot from "lucide-static/icons/bot.svg?raw";
import fileSearch from "lucide-static/icons/file-search.svg?raw";
import lightbulb from "lucide-static/icons/lightbulb.svg?raw";
import wrench from "lucide-static/icons/wrench.svg?raw";
import { i18n, m } from "./i18n";

export function ConversationWelcome(props: {
  choosePrompt: (prompt: string) => void;
}) {
  const prompts = () => [
    {
      icon: fileSearch,
      title: i18n.message(m.starter_explain_title, {}),
      prompt: i18n.message(m.starter_explain_prompt, {}),
    },
    {
      icon: wrench,
      title: i18n.message(m.starter_fix_title, {}),
      prompt: i18n.message(m.starter_fix_prompt, {}),
    },
    {
      icon: lightbulb,
      title: i18n.message(m.starter_plan_title, {}),
      prompt: i18n.message(m.starter_plan_prompt, {}),
    },
  ];
  return (
    <View class="min-h-72 items-center justify-center gap-4 py-8">
      <View class="w-11 h-11 rounded-xl bg-selected flex items-center justify-center">
        <Icon source={bot} size={21} class="text-accent" />
      </View>
      <View class="items-center gap-1">
        <Text class="text-lg font-semibold">
          {i18n.message(m.empty_title, {})}
        </Text>
        <Text class="max-w-xl text-sm text-muted text-center whitespace-normal">
          {i18n.message(m.empty_detail, {})}
        </Text>
      </View>
      <View class="w-full max-w-2xl flex flex-row gap-3">
        {prompts().map((item) => (
          <Button
            variant="outline"
            class="h-auto min-w-0 flex-1 items-start justify-start p-3"
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
