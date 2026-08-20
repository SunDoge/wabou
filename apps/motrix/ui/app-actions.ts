import { createComponent, createContext, type JSX, useContext } from "solid-js";

export interface AppActions {
  openAddTask(): void;
}

const AppActionsContext = createContext<AppActions>();

export function AppActionsProvider(props: {
  value: AppActions;
  children?: JSX.Element;
}): JSX.Element {
  return createComponent(AppActionsContext, {
    value: props.value,
    get children() {
      return props.children;
    },
  });
}

export function useAppActions(): AppActions {
  const actions = useContext(AppActionsContext);
  if (!actions) throw new Error("useAppActions must be used inside AppShell");
  return actions;
}
