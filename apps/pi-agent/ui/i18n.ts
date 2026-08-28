import { createLocale } from "@wabou/ui/i18n";
import * as messages from "./paraglide/messages.js";
import type { Locale } from "./paraglide/runtime.js";

export const i18n = createLocale<Locale>("en");
export const m = messages;
