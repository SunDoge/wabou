import { expect, test } from "bun:test";
import { runExperiment } from "./index";

test("TanStack Router Core drives native-style memory navigation", async () => {
  expect(await runExperiment()).toEqual({
    compatible: true,
    diagnostics: [
      {
        routeId: "__root__",
        status: "success",
        params: { projectId: "alpha" },
        loaderData: undefined,
      },
      {
        routeId: "/projects/$projectId",
        status: "success",
        params: { projectId: "alpha" },
        loaderData: { id: "alpha", page: 2, key: "project:alpha" },
      },
    ],
    initial: {
      pathname: "/projects/alpha",
      search: { page: 2 },
      params: { projectId: "alpha" },
      loaderData: { id: "alpha", page: 2, key: "project:alpha" },
    },
    navigated: {
      pathname: "/settings",
      loaderData: { section: "general" },
    },
    restored: {
      pathname: "/projects/alpha",
      canGoBack: false,
    },
  });
});
