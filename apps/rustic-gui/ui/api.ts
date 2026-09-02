import {
  bindCapability,
  type Host,
  type NativeCapability,
  useHost,
} from "@wabou/ui";

export interface BackupProfile {
  id: string;
  name: string;
  repositoryPath: string;
  sources: string[];
}

export interface RuntimeStatus {
  unlockedProfileIds: string[];
  activeProfileId?: string;
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
  status(): RuntimeStatus | PromiseLike<RuntimeStatus>;
  createProfile(request: {
    id: string;
    name: string;
    path: string;
    password: string;
    sources: string[];
  }): RuntimeStatus | PromiseLike<RuntimeStatus>;
  openProfile(request: {
    id: string;
    name: string;
    path: string;
    password: string;
    sources: string[];
  }): RuntimeStatus | PromiseLike<RuntimeStatus>;
  selectProfile(request: {
    profileId: string;
  }): RuntimeStatus | PromiseLike<RuntimeStatus>;
  setSources(request: {
    profileId: string;
    sources: string[];
  }): RuntimeStatus | PromiseLike<RuntimeStatus>;
  runBackup(request: {
    profileId: string;
  }): { snapshot: SnapshotEntry } | PromiseLike<{ snapshot: SnapshotEntry }>;
  listSnapshots(request: {
    profileId: string;
  }): SnapshotEntry[] | PromiseLike<SnapshotEntry[]>;
  listFiles(request: {
    profileId: string;
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
    version: 2,
  });
}
