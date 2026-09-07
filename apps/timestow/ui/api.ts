import {
  bindCapability,
  type Host,
  type NativeCapability,
  useHost,
} from "@wabou/ui";
import type { BackupSchedule } from "./backup-schedule";

export const FILE_PAGE_SIZE = 250;

export interface BackupProfile {
  id: string;
  name: string;
  repositoryPath: string;
  sources: string[];
  schedule?: BackupSchedule;
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
  parentId?: string;
  label: string;
  description?: string;
  tags: string[];
  deleteProtected: boolean;
}

export interface FileEntry {
  name: string;
  path: string;
  kind: "directory" | "file" | "symlink" | "special";
  size: number;
  modified?: string;
}

export interface FileListing {
  entries: FileEntry[];
  total: number;
  offset: number;
  hasMore: boolean;
}

export type SnapshotDiffChange =
  | "added"
  | "removed"
  | "modified"
  | "metadata"
  | "typeChanged";

export interface SnapshotDiffEntry {
  name: string;
  path: string;
  kind: FileEntry["kind"];
  change: SnapshotDiffChange;
  previousSize?: number;
  currentSize?: number;
  previousModified?: string;
  currentModified?: string;
}

export interface SnapshotDiff {
  entries: SnapshotDiffEntry[];
  summary: {
    added: number;
    removed: number;
    modified: number;
    metadata: number;
    typeChanged: number;
  };
  /** Exact when complete; a lower bound when `truncated` is true. */
  totalEntries: number;
  truncated: boolean;
}

export interface RestorePlanSummary {
  restoreSize: number;
  matchedSize: number;
  filesToRestore: number;
  filesToModify: number;
  filesUnchanged: number;
  directoriesToRestore: number;
  directoriesToModify: number;
}

export interface RestoreResult {
  destination: string;
  plan: RestorePlanSummary;
}

export interface RusticCapability extends NativeCapability {
  status(): RuntimeStatus | PromiseLike<RuntimeStatus>;
  createProfile(request: {
    id: string;
    name: string;
    path: string;
    passwordSlot: string;
    sources: string[];
  }): RuntimeStatus | PromiseLike<RuntimeStatus>;
  openProfile(request: {
    id: string;
    name: string;
    path: string;
    passwordSlot: string;
    sources: string[];
  }): RuntimeStatus | PromiseLike<RuntimeStatus>;
  selectProfile(request: {
    profileId: string;
  }): RuntimeStatus | PromiseLike<RuntimeStatus>;
  forgetProfile(request: {
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
    offset?: number;
    limit?: number;
  }): FileListing | PromiseLike<FileListing>;
  searchFiles(request: {
    profileId: string;
    snapshotId: string;
    query: string;
    limit?: number;
  }): FileEntry[] | PromiseLike<FileEntry[]>;
  diffSnapshots(request: {
    profileId: string;
    baseSnapshotId: string;
    snapshotId: string;
    path: string;
    includeMetadata?: boolean;
    limit?: number;
  }): SnapshotDiff | PromiseLike<SnapshotDiff>;
  updateSnapshot(request: {
    profileId: string;
    snapshotId: string;
    label: string;
    description: string;
    tags: string[];
    deleteProtected: boolean;
  }): SnapshotEntry | PromiseLike<SnapshotEntry>;
  previewRestore(request: {
    profileId: string;
    snapshotId: string;
    path: string;
    destination: string;
  }): RestorePlanSummary | PromiseLike<RestorePlanSummary>;
  restorePath(request: {
    profileId: string;
    snapshotId: string;
    path: string;
    destination: string;
  }): RestoreResult | PromiseLike<RestoreResult>;
  previewPath(request: {
    profileId: string;
    snapshotId: string;
    path: string;
  }): RestoreResult | PromiseLike<RestoreResult>;
  openPath(request: { path: string }): void | PromiseLike<void>;
}

interface RusticHost extends Host {
  rustic: RusticCapability;
}

export function useRusticApi(): RusticCapability {
  return bindCapability(useHost<RusticHost>().rustic, {
    name: "rustic",
    version: 8,
  });
}
