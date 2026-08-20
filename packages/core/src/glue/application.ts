import { dispatchFireAndForget, effectOps } from "./effects";

export interface Application {
  /** Terminate the full native application, including tray-resident windows. */
  exit(): void;
  /** Gracefully stop the application and launch the same executable again. */
  relaunch(): void;
}

export const application: Application = Object.freeze({
  exit: () => dispatchFireAndForget(effectOps.applicationExit),
  relaunch: () => dispatchFireAndForget(effectOps.applicationRelaunch),
});
