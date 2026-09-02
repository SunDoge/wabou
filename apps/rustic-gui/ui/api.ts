import {
  bindCapability,
  type Host,
  type NativeCapability,
  useHost,
} from "@wabou/ui";

export interface AppStatus {
  connected: boolean;
  repositoryPath?: string;
  sources: string[];
}

export interface SnapshotEntry {
  id: string;
  time: string;
  hostname: string;
  paths: string[];
  filesNew: number;
  filesChanged: number;
}

export interface FileEntry {
  name: string;
  path: string;
  kind: "directory" | "file" | "symlink" | "special";
  size: number;
  modified?: string;
}

interface RusticCapability extends NativeCapability {
  status(): AppStatus | PromiseLike<AppStatus>;
  createRepository(request: {
    path: string;
    password: string;
  }): AppStatus | PromiseLike<AppStatus>;
  openRepository(request: {
    path: string;
    password: string;
  }): AppStatus | PromiseLike<AppStatus>;
  setSources(request: {
    sources: string[];
  }): AppStatus | PromiseLike<AppStatus>;
  runBackup():
    | { snapshot: SnapshotEntry }
    | PromiseLike<{ snapshot: SnapshotEntry }>;
  listSnapshots(): SnapshotEntry[] | PromiseLike<SnapshotEntry[]>;
  listFiles(request: {
    snapshotId: string;
    path: string;
  }): FileEntry[] | PromiseLike<FileEntry[]>;
}

interface RusticHost extends Host {
  rustic: RusticCapability;
}

export function useRusticApi(): RusticCapability {
  return bindCapability(useHost<RusticHost>().rustic, {
    name: "rustic",
    version: 1,
  });
}
