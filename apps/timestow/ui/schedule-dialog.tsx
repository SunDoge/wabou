import {
  Button,
  Dialog,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  Icon,
  Select,
  Switch,
  Text,
  View,
} from "@wabou/ui";
import clock from "lucide-static/icons/clock-3.svg?raw";
import { createEffect, createSignal, Show } from "solid-js";
import type { BackupProfile } from "./api";
import {
  BACKUP_SCHEDULE_OPTIONS,
  type BackupScheduleInterval,
  backupScheduleIntervalLabel,
  isBackupScheduleInterval,
} from "./backup-schedule";
import { useTimestowSession } from "./session";

function formatScheduleTime(value?: string): string {
  if (!value) return "Not scheduled";
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? "Not scheduled" : date.toLocaleString();
}

export function BackupScheduleDialog(props: {
  profile: BackupProfile;
  disabled?: boolean;
  defaultOpen?: boolean;
}) {
  const session = useTimestowSession();
  const [enabled, setEnabled] = createSignal(false);
  const [intervalMinutes, setIntervalMinutes] =
    createSignal<BackupScheduleInterval>(1440);
  const [saving, setSaving] = createSignal(false);
  const [error, setError] = createSignal<string>();

  createEffect(
    () => ({ id: props.profile.id, schedule: props.profile.schedule }),
    ({ schedule }) => {
      setEnabled(schedule?.enabled ?? false);
      setIntervalMinutes(schedule?.intervalMinutes ?? 1440);
      setError(undefined);
    },
  );

  async function save(close: () => void): Promise<void> {
    if (saving()) return;
    setSaving(true);
    setError(undefined);
    try {
      await session.updateSchedule(
        props.profile.id,
        enabled(),
        intervalMinutes(),
      );
      close();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setSaving(false);
    }
  }

  return (
    <Dialog
      aria-label="Backup schedule"
      defaultOpen={props.defaultOpen}
      trigger={(trigger) => (
        <Button
          {...trigger}
          aria-label={
            props.profile.schedule?.enabled
              ? `Backup schedule: ${backupScheduleIntervalLabel(props.profile.schedule.intervalMinutes)}`
              : "Schedule"
          }
          variant="outline"
          disabled={props.disabled}
        >
          <Icon source={clock} size={14} />
          {props.profile.schedule?.enabled
            ? backupScheduleIntervalLabel(
                props.profile.schedule.intervalMinutes,
              )
            : "Schedule"}
        </Button>
      )}
    >
      {(dialog) => (
        <View class="min-w-0 flex flex-col gap-5">
          <DialogHeader>
            <DialogTitle>Scheduled backups</DialogTitle>
            <DialogDescription>
              Back up this profile automatically while Timestow is running and
              the repository is unlocked.
            </DialogDescription>
          </DialogHeader>

          <View class="flex flex-col gap-4">
            <View class="rounded-lg border border-subtle bg-surface-muted p-4">
              <Switch
                label="Run backups automatically"
                checked={enabled()}
                disabled={props.disabled}
                onCheckedChange={setEnabled}
              />
            </View>
            <View class="flex flex-col gap-1.5">
              <Text class="text-sm font-medium">Frequency</Text>
              <Select
                aria-label="Backup frequency"
                options={BACKUP_SCHEDULE_OPTIONS}
                value={String(intervalMinutes())}
                disabled={!enabled() || props.disabled}
                onValueChange={(value) => {
                  const interval = Number(value);
                  if (isBackupScheduleInterval(interval)) {
                    setIntervalMinutes(interval);
                  }
                }}
              />
            </View>
            <View class="grid grid-cols-2 gap-3 rounded-lg border border-subtle p-4">
              <View class="min-w-0 flex flex-col gap-1">
                <Text class="text-xs font-medium text-muted">Next backup</Text>
                <Text class="truncate text-sm">
                  {enabled()
                    ? formatScheduleTime(props.profile.schedule?.nextRunAt)
                    : "Automatic backups off"}
                </Text>
              </View>
              <View class="min-w-0 flex flex-col gap-1">
                <Text class="text-xs font-medium text-muted">Last backup</Text>
                <Text class="truncate text-sm">
                  {formatScheduleTime(props.profile.schedule?.lastRunAt)}
                </Text>
              </View>
            </View>
            <Show when={props.profile.schedule?.lastError}>
              {(message) => (
                <View class="rounded-md border border-danger bg-danger-surface px-3 py-2">
                  <Text class="whitespace-normal text-sm text-danger-primary">
                    Last automatic backup failed: {message()}
                  </Text>
                </View>
              )}
            </Show>
            <Show when={error()}>
              {(message) => (
                <Text role="alert" class="text-sm text-danger-primary">
                  {message()}
                </Text>
              )}
            </Show>
          </View>

          <DialogFooter>
            <Button variant="outline" onClick={dialog.close}>
              Cancel
            </Button>
            <Button
              loading={saving()}
              loadingLabel="Saving…"
              onClick={() => void save(dialog.close)}
            >
              Save schedule
            </Button>
          </DialogFooter>
        </View>
      )}
    </Dialog>
  );
}
