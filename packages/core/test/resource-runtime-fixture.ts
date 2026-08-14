import "@wabou/core";

import { createElement, mount, spread, useHost } from "@wabou/solid-renderer";
import {
  createComponent,
  createMemo,
  Errored,
  type JSX,
  Loading,
} from "solid-js";

declare module "@wabou/solid-renderer" {
  interface HostCapabilities {
    readonly promiseTest: {
      resolve(): Promise<string>;
      reject(): Promise<string>;
    };
  }
}

function host(
  tag: "view" | "text",
  props: { children?: JSX.Element },
): JSX.Element {
  const node = createElement(tag);
  spread(node, props, false);
  return node as unknown as JSX.Element;
}

const View = (props: { children?: JSX.Element }) => host("view", props);
const Text = (props: { children?: JSX.Element }) => host("text", props);

function ResourceFixture() {
  const native = useHost();
  const resolved = createMemo(() => native.promiseTest.resolve());
  const rejected = createMemo(() => native.promiseTest.reject());

  const success = createComponent(Loading, {
    fallback: createComponent(Text, { children: "success pending" }),
    get children() {
      return createComponent(Text, {
        get children() {
          return `success ${resolved()}`;
        },
      });
    },
  });
  const failure = createComponent(Errored, {
    fallback: (error) =>
      createComponent(Text, {
        children: `caught ${(error() as Error).message}`,
      }),
    get children() {
      return createComponent(Loading, {
        fallback: createComponent(Text, { children: "failure pending" }),
        get children() {
          return createComponent(Text, {
            get children() {
              return `failure ${rejected()}`;
            },
          });
        },
      });
    },
  });

  return createComponent(View, { children: [success, failure] });
}

mount(() => createComponent(ResourceFixture, {}));
