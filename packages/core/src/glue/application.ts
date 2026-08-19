import { dispatchFireAndForget, effectOps } from "./effects";

export interface Application {
  /** Terminate the full native application, including tray-resident windows. */
  exit(): void;
}

export const application: Application = Object.freeze({
  exit: () => dispatchFireAndForget(effectOps.applicationExit),
});
