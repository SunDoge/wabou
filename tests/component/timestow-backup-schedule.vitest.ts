import { expect, test } from "vitest";
import {
  advanceBackupSchedule,
  backupScheduleFromValue,
  createBackupSchedule,
  nextBackupTime,
  scheduleDueAt,
} from "../../apps/timestow/ui/backup-schedule";

test("creates a durable schedule and preserves its due time when unchanged", () => {
  const now = Date.parse("2026-09-04T08:00:00.000Z");
  const schedule = createBackupSchedule(true, 360, undefined, now);

  expect(schedule.nextRunAt).toBe("2026-09-04T14:00:00.000Z");
  expect(scheduleDueAt(schedule)).toBe(now + 360 * 60_000);
  expect(createBackupSchedule(true, 360, schedule, now + 1_000)).toEqual(
    schedule,
  );
  expect(createBackupSchedule(false, 360, schedule, now).nextRunAt).toBe(
    undefined,
  );
});

test("advances successful and failed schedules without inventing success", () => {
  const schedule = createBackupSchedule(
    true,
    60,
    undefined,
    Date.parse("2026-09-04T08:00:00.000Z"),
  );
  const completedAt = Date.parse("2026-09-04T09:00:00.000Z");
  const succeeded = advanceBackupSchedule(schedule, { completedAt });
  expect(succeeded.lastRunAt).toBe("2026-09-04T09:00:00.000Z");
  expect(succeeded.nextRunAt).toBe(nextBackupTime(60, completedAt));

  const failed = advanceBackupSchedule(succeeded, {
    completedAt: completedAt + 60 * 60_000,
    error: "disk full",
  });
  expect(failed.lastRunAt).toBe(succeeded.lastRunAt);
  expect(failed.lastError).toBe("disk full");
  expect(failed.nextRunAt).toBe("2026-09-04T11:00:00.000Z");
});

test("rejects malformed persisted schedules", () => {
  expect(() =>
    backupScheduleFromValue({ enabled: true, intervalMinutes: 15 }),
  ).toThrow("invalid persisted backup schedule");
});
