import {
  BaseRootRoute,
  Button,
  createDataRouter,
  createMemoryHistory,
  Dynamic,
  Input,
  mount,
  Portal,
  View,
} from "@wabou/ui";
import * as Wabou from "@wabou/ui";
import { createLocale } from "@wabou/ui/i18n";
import { Button as PrimitiveButton } from "@wabou/ui/primitives";
import { defineWabouConfig } from "@wabou/vite";
import { createSignal } from "solid-js";

const [enabled, setEnabled] = createSignal(false);
const locale = createLocale("en" as "en" | "zh");
const localized = locale.message(
  (input: { name: string }, options?: { locale?: "en" | "zh" }) =>
    `${options?.locale}:${input.name}`,
  { name: "Wabou" },
);
const rootRoute = new BaseRootRoute();

// The public JSX registry is intentionally native and finite, rather than the
// complete browser element catalogue.
const structuralHostNode = <view class="flex" />;
const plainTextInput = <Input />;
// @ts-expect-error links are components/capabilities, not implicit Web anchors
const unsupportedAnchor = <a href="https://example.com">Unsupported</a>;
// @ts-expect-error secrets require the dedicated PasswordInput + SecretStore
const insecureGenericInput = <input type="password" />;
// @ts-expect-error browser tooltip attributes have no native Wabou behavior
const unsupportedTitle = <view title="Browser tooltip" />;
// @ts-expect-error structural host nodes do not acquire browser navigation
const unsupportedHref = <view href="https://example.com" />;
// @ts-expect-error image semantics use an explicit accessible label, not Web alt fallback
const unsupportedAlt = <img alt="Preview" />;
// @ts-expect-error primitives expose a finite native contract too
const unsupportedPrimitiveHref = <View href="https://example.com" />;
// @ts-expect-error headless controls do not forward arbitrary DOM attributes
const unsupportedButtonHref = <PrimitiveButton href="https://example.com" />;
// @ts-expect-error portal containers expose only native host properties
const unsupportedPortalHref = <Portal href="https://example.com" />;
const dynamicNativeView = <Dynamic component="view" class="flex" />;
// @ts-expect-error Dynamic string targets use the same finite native registry
const unsupportedDynamicAnchor = <Dynamic component="a" />;
const NamedView = (props: { name: string }) => <view>{props.name}</view>;
const dynamicComponent = <Dynamic component={NamedView} name="Wabou" />;
// @ts-expect-error Dynamic preserves function-component props
const invalidDynamicComponent = <Dynamic component={NamedView} />;
// @ts-expect-error inline styles expose only properties implemented by Style IR
const unsupportedStyle = <view style={{ filter: "blur(4px)" }} />;
// @ts-expect-error numeric native effects are a private framework ABI
const privateEffectDispatcher = Wabou.dispatchEffect;
// @ts-expect-error protocol writers are available only from @wabou/core/renderer
const privateProtocolWriter = Wabou.writer;
// @ts-expect-error generated opcodes are available only from explicit protocol/renderer subpaths
const privateOpcodeTable = Wabou.OP;
void [
  structuralHostNode,
  plainTextInput,
  unsupportedAnchor,
  insecureGenericInput,
  unsupportedTitle,
  unsupportedHref,
  unsupportedAlt,
  unsupportedPrimitiveHref,
  unsupportedButtonHref,
  unsupportedPortalHref,
  dynamicNativeView,
  unsupportedDynamicAnchor,
  dynamicComponent,
  invalidDynamicComponent,
  unsupportedStyle,
  privateEffectDispatcher,
  privateProtocolWriter,
  privateOpcodeTable,
  localized,
];

createDataRouter({
  routeTree: rootRoute,
  history: createMemoryHistory(),
  context: {},
});

mount(() => (
  <Button onClick={() => setEnabled((value) => !value)}>
    {enabled() ? "Enabled" : "Disabled"}
  </Button>
));

export default defineWabouConfig({});
