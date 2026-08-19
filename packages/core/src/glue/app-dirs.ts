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

/** Resolve all app-private roots once and reuse the same native result. */
export function resolveAppDirectories(): Promise<AppDirectories> {
  return (resolved ??= dispatchEffect<AppDirectories>(
    effectOps.appDirsResolve,
  ));
}

export const appDirs = Object.freeze({
  resolve: resolveAppDirectories,
  config: () => resolveAppDirectories().then((paths) => paths.configDir),
  data: () => resolveAppDirectories().then((paths) => paths.dataDir),
  localData: () => resolveAppDirectories().then((paths) => paths.localDataDir),
  cache: () => resolveAppDirectories().then((paths) => paths.cacheDir),
  log: () => resolveAppDirectories().then((paths) => paths.logDir),
  resource: () => resolveAppDirectories().then((paths) => paths.resourceDir),
  temp: () => resolveAppDirectories().then((paths) => paths.tempDir),
});

// Tauri-compatible names ease migration while the grouped object keeps imports compact.
export const appConfigDir = appDirs.config;
export const appDataDir = appDirs.data;
export const appLocalDataDir = appDirs.localData;
export const appCacheDir = appDirs.cache;
export const appLogDir = appDirs.log;
export const resourceDir = appDirs.resource;
export const tempDir = appDirs.temp;
