import { expect, test } from "bun:test";
import { mount } from "@wabou/solid-renderer";
import { createComponent, createEffect, type JSX } from "solid-js";
import { isServer } from "solid-js/web";
import {
  createMemoryHistory,
  MemoryRouter,
  Route,
  useHistory,
  useLocation,
  useNavigate,
  useParams,
} from ".";

test.skipIf(isServer)(
  "MemoryRouter publishes reactive location and route parameters",
  () => {
    const history = createMemoryHistory({ initialEntries: ["/story/1"] });
    const seen: string[] = [];
    let mounts = 0;
    let navigate: ReturnType<typeof useNavigate> | undefined;

    function Story() {
      mounts += 1;
      const location = useLocation();
      const params = useParams<{ id: string }>();
      navigate = useNavigate();
      expect(useHistory()).toBe(history);
      createEffect(() => seen.push(`${location.pathname}:${params.id}`));
      return null;
    }

    const dispose = mount(() =>
      createComponent(MemoryRouter, {
        history,
        get children() {
          return createComponent(Route, {
            path: "/story/:id",
            component: Story,
          });
        },
      }),
    );
    navigate?.("/story/2", { state: { selected: true } });
    history.back();
    dispose();

    expect(seen).toEqual(["/story/1:1", "/story/2:2", "/story/1:1"]);
    expect(mounts).toBe(1);
    expect(history.get().value).toBe("/story/1");
  },
);

test("router hooks reject calls outside MemoryRouter", () => {
  expect(() => useNavigate()).toThrow(
    "Wabou router hooks must be used inside <MemoryRouter>",
  );
});

test.skipIf(isServer)(
  "switches leaf routes without remounting the root layout",
  () => {
    const history = createMemoryHistory();
    const rendered: string[] = [];
    let rootMounts = 0;
    let navigate: ReturnType<typeof useNavigate> | undefined;

    function Root(props: { children?: JSX.Element }) {
      rootMounts += 1;
      navigate = useNavigate();
      return props.children;
    }
    function Home() {
      rendered.push("home");
      return null;
    }
    function Story() {
      rendered.push("story");
      return null;
    }

    const dispose = mount(() =>
      createComponent(MemoryRouter, {
        history,
        root: Root,
        get children() {
          return [
            createComponent(Route, { path: "/", component: Home }),
            createComponent(Route, { path: "/story/:id", component: Story }),
          ];
        },
      }),
    );
    navigate?.("/story/7");
    navigate?.(-1);
    dispose();

    expect(rendered).toEqual(["home", "story", "home"]);
    expect(rootMounts).toBe(1);
  },
);
