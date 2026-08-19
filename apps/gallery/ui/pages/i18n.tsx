import { Button, Text, View } from "@wabou/ui";
import { createLocale } from "@wabou/ui/i18n";
import { createSignal } from "solid-js";
import "virtual:wabou-stylesheet";

import * as m from "../paraglide/messages.js";
import type { Locale } from "../paraglide/runtime.js";
import { Preview } from "../preview";
import { PropertyRow } from "./showcase";

const i18n = createLocale<Locale>("en");

export function I18nPage() {
  const [count, setCount] = createSignal(1);

  return (
    <View class="flex flex-col gap-5">
      <Preview title="Paraglide compiled messages">
        <View class="w-full max-w-xl p-6 flex flex-col gap-4 rounded-xl border border-subtle bg-surface-muted">
          <Text class="text-xl font-semibold">
            {i18n.message(m.gallery_i18n_title, {})}
          </Text>
          <Text class="text-secondary">
            {i18n.message(m.gallery_i18n_greeting, { name: "Wabou" })}
          </Text>
          <Text class="text-secondary">
            {i18n.message(m.gallery_i18n_count, { count: count() })}
          </Text>
          <View class="flex flex-wrap gap-2">
            <Button
              variant={i18n.locale() === "en" ? "default" : "outline"}
              onClick={() => i18n.set("en")}
            >
              English
            </Button>
            <Button
              variant={i18n.locale() === "zh" ? "default" : "outline"}
              onClick={() => i18n.set("zh")}
            >
              中文
            </Button>
            <Button variant="secondary" onClick={() => setCount((n) => n + 1)}>
              +1
            </Button>
          </View>
        </View>
      </Preview>
      <PropertyRow
        name="Runtime model"
        value="Solid locale signal + precompiled typed message functions"
      />
      <PropertyRow
        name="Web globals"
        value="No URL, cookie, localStorage, or document locale strategy"
      />
    </View>
  );
}
