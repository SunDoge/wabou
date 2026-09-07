import type { KvValue } from "@wabou/ui";

export const BACKUP_SCHEDULE_INTERVALS = [60, 360, 720, 1440, 10080] as const;

export type BackupScheduleInterval = (typeof BACKUP_SCHEDULE_INTERVALS)[number];

export interface BackupSchedule {
  enabled: boolean;
  intervalMinutes: BackupScheduleInterval;
  nextRunAt?: string;
  lastRunAt?: string;
  lastError?: string;
}

export const BACKUP_SCHEDULE_OPTIONS = [
  { value: "60", label: "Every hour" },
  { value: "360", label: "Every 6 hours" },
  { value: "720", label: "Every 12 hours" },
  { value: "1440", label: "Every day" },
  { value: "10080", label: "Every week" },
] as const;

export function backupScheduleIntervalLabel(
  intervalMinutes: BackupScheduleInterval,
): string {
  return (
    BACKUP_SCHEDULE_OPTIONS.find(
      (option) => Number(option.value) === intervalMinutes,
    )?.label ?? `${intervalMinutes} minutes`
  );
}

export function isBackupScheduleInterval(
  value: number,
): value is BackupScheduleInterval {
  return BACKUP_SCHEDULE_INTERVALS.some((interval) => interval === value);
}

function isKvRecord(
  value: KvValue | undefined,
): value is { readonly [key: string]: KvValue } {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

export function backupScheduleFromValue(
  value: KvValue | undefined,
): BackupSchedule | undefined {
  if (!isKvRecord(value)) return undefined;
  const intervalMinutes = value.intervalMinutes;
  if (
    typeof value.enabled !== "boolean" ||
    typeof intervalMinutes !== "number" ||
    !isBackupScheduleInterval(intervalMinutes)
  ) {
    throw new TypeError("invalid persisted backup schedule");
  }
  const optionalString = (name: string): string | undefined => {
    const candidate = value[name];
    if (candidate === undefined) return undefined;
    if (typeof candidate !== "string") {
      throw new TypeError(`invalid persisted backup schedule ${name}`);
    }
    return candidate;
  };
  return {
    enabled: value.enabled,
    intervalMinutes,
    nextRunAt: optionalString("nextRunAt"),
    lastRunAt: optionalString("lastRunAt"),
    lastError: optionalString("lastError"),
  };
}

export function backupScheduleValue(schedule: BackupSchedule): KvValue {
  return {
    enabled: schedule.enabled,
    intervalMinutes: schedule.intervalMinutes,
    ...(schedule.nextRunAt ? { nextRunAt: schedule.nextRunAt } : {}),
    ...(schedule.lastRunAt ? { lastRunAt: schedule.lastRunAt } : {}),
    ...(schedule.lastError ? { lastError: schedule.lastError } : {}),
  };
}

export function nextBackupTime(
  intervalMinutes: BackupScheduleInterval,
  now = Date.now(),
): string {
  return new Date(now + intervalMinutes * 60_000).toISOString();
}

export function scheduleDueAt(schedule: BackupSchedule): number | undefined {
  if (!schedule.enabled || !schedule.nextRunAt) return undefined;
  const timestamp = Date.parse(schedule.nextRunAt);
  return Number.isFinite(timestamp) ? timestamp : undefined;
}

export function createBackupSchedule(
  enabled: boolean,
  intervalMinutes: BackupScheduleInterval,
  previous?: BackupSchedule,
  now = Date.now(),
): BackupSchedule {
  if (!enabled) {
    return {
      enabled: false,
      intervalMinutes,
      lastRunAt: previous?.lastRunAt,
      lastError: previous?.lastError,
    };
  }
  const preserveNext =
    previous?.enabled === true &&
    previous.intervalMinutes === intervalMinutes &&
    scheduleDueAt(previous) !== undefined;
  return {
    enabled: true,
    intervalMinutes,
    nextRunAt: preserveNext
      ? previous.nextRunAt
      : nextBackupTime(intervalMinutes, now),
    lastRunAt: previous?.lastRunAt,
    lastError: previous?.lastError,
  };
}

export function advanceBackupSchedule(
  schedule: BackupSchedule,
  result: { completedAt: number; error?: string },
): BackupSchedule {
  return {
    ...schedule,
    nextRunAt: schedule.enabled
      ? nextBackupTime(schedule.intervalMinutes, result.completedAt)
      : undefined,
    lastRunAt: result.error
      ? schedule.lastRunAt
      : new Date(result.completedAt).toISOString(),
    lastError: result.error,
  };
}
