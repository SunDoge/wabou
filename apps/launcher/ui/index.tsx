import "@wabou/ui";
import "virtual:wabou-stylesheet";
import {
  ColorThemeProvider,
  ComponentsProvider,
  mount,
  useHost,
  useWindow,
} from "@wabou/ui";
import { createSignal } from "solid-js";
import type { LauncherEntry } from "./generated/rust-bindings";
import { LauncherSurface } from "./launcher";

function App() {
  const api = useHost().launcher;
  const window = useWindow();
  const [entries, setEntries] = createSignal<LauncherEntry[]>([]);
  const [loading, setLoading] = createSignal(true);
  const [error, setError] = createSignal<unknown>();
  const [busyId, setBusyId] = createSignal<string>();

  const load = async () => {
    setLoading(true);
    setError(undefined);
    try {
      setEntries(await api.list());
    } catch (nextError) {
      setError(nextError);
    } finally {
      setLoading(false);
    }
  };
  const launch = async (id: string) => {
    setBusyId(id);
    setError(undefined);
    try {
      await api.launch({ id });
      window.close();
    } catch (nextError) {
      setError(nextError);
    } finally {
      setBusyId(undefined);
    }
  };

  void load();

  return (
    <ColorThemeProvider theme="dark" transition={false}>
      <ComponentsProvider theme="dark">
        <LauncherSurface
          entries={entries()}
          loading={loading()}
          error={error()}
          busyId={busyId()}
          onLaunch={(id) => void launch(id)}
          onDismiss={() => window.close()}
          onRetry={() => void load()}
        />
      </ComponentsProvider>
    </ColorThemeProvider>
  );
}

mount(() => <App />);
