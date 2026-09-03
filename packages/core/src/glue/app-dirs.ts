import { dispatchEffect, effectOps } from "./effects";

/** Native, absolute roots owned by the current application. */
export interface AppDirectories {
  readonly configDir: string;
  readonly dataDir: string;
  readonly localDataDir: string;
  readonly cacheDir: string;
  readonly logDir: string;
  readonly resourceDir: string;
  readonly tempDir: string;
}

let resolved: Promise<AppDirectories> | undefined;

function resolve(): Promise<AppDirectories> {
  return (resolved ??= dispatchEffect<AppDirectories>(
    effectOps.appDirsResolve,
  ));
}

/** Resolve app-private native roots, caching the host result for this runtime. */
export const appDirs = Object.freeze({
  resolve,
  config: () => resolve().then((paths) => paths.configDir),
  data: () => resolve().then((paths) => paths.dataDir),
  localData: () => resolve().then((paths) => paths.localDataDir),
  cache: () => resolve().then((paths) => paths.cacheDir),
  log: () => resolve().then((paths) => paths.logDir),
  resource: () => resolve().then((paths) => paths.resourceDir),
  temp: () => resolve().then((paths) => paths.tempDir),
});
