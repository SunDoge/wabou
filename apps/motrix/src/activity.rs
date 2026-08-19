use std::{
    collections::{BTreeMap, HashMap},
    fs,
    path::{Path, PathBuf},
};

use jiff::{Span, Timestamp};
use serde::{Deserialize, Serialize};

const HISTORY_DAYS: usize = 84;

#[derive(Debug, Default, Deserialize, Serialize)]
struct PersistedActivity {
    days: BTreeMap<String, u64>,
}

/// Persistent transfer accounting derived from per-task completed-byte deltas.
pub struct ActivityLog {
    path: PathBuf,
    days: BTreeMap<String, u64>,
    completed_by_gid: HashMap<String, u64>,
    baselined: bool,
    dirty: bool,
    dirty_ticks: u8,
}

impl ActivityLog {
    pub fn load(config_dir: &Path) -> Self {
        let path = config_dir.join("activity.json");
        let persisted = fs::read_to_string(&path)
            .ok()
            .and_then(|source| serde_json::from_str::<PersistedActivity>(&source).ok())
            .unwrap_or_default();
        Self {
            path,
            days: persisted.days,
            completed_by_gid: HashMap::new(),
            baselined: false,
            dirty: false,
            dirty_ticks: 0,
        }
    }

    pub fn observe<'a>(&mut self, tasks: impl IntoIterator<Item = (&'a str, u64)>) {
        let next: HashMap<String, u64> = tasks
            .into_iter()
            .map(|(gid, completed)| (gid.to_owned(), completed))
            .collect();
        if self.baselined {
            let downloaded = next
                .iter()
                .map(|(gid, completed)| {
                    self.completed_by_gid
                        .get(gid)
                        .map_or(0, |previous| completed.saturating_sub(*previous))
                })
                .sum::<u64>();
            if downloaded > 0 {
                *self.days.entry(today()).or_default() += downloaded;
                self.dirty = true;
            }
        }
        self.baselined = true;
        self.completed_by_gid = next;
        self.dirty_ticks = self.dirty_ticks.saturating_add(1);
        if self.dirty && self.dirty_ticks >= 10 {
            let _ = self.save();
        }
    }

    pub fn recent(&self) -> Vec<u64> {
        let today = local_date();
        (0..HISTORY_DAYS)
            .rev()
            .map(|offset| {
                let date = today
                    .checked_sub(Span::new().days(offset as i64))
                    .unwrap_or(today);
                self.days.get(&date.to_string()).copied().unwrap_or(0)
            })
            .collect()
    }

    pub fn downloaded_today(&self) -> u64 {
        self.days.get(&today()).copied().unwrap_or(0)
    }

    fn save(&mut self) -> Result<(), String> {
        let parent = self
            .path
            .parent()
            .ok_or_else(|| "activity path has no parent".to_owned())?;
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        let source = serde_json::to_vec_pretty(&PersistedActivity {
            days: self.days.clone(),
        })
        .map_err(|error| error.to_string())?;
        fs::write(&self.path, source).map_err(|error| error.to_string())?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&self.path, fs::Permissions::from_mode(0o600))
                .map_err(|error| error.to_string())?;
        }
        self.dirty = false;
        self.dirty_ticks = 0;
        Ok(())
    }
}

impl Drop for ActivityLog {
    fn drop(&mut self) {
        if self.dirty {
            let _ = self.save();
        }
    }
}

fn local_date() -> jiff::civil::Date {
    Timestamp::now()
        .to_zoned(jiff::tz::TimeZone::system())
        .date()
}

fn today() -> String {
    local_date().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_only_deltas_after_the_initial_baseline_and_persists() {
        let root = std::env::temp_dir().join(format!("motrix-activity-{}", uuid::Uuid::new_v4()));
        {
            let mut log = ActivityLog::load(&root);
            log.observe([("a", 100), ("b", 50)]);
            assert_eq!(log.downloaded_today(), 0);
            log.observe([("a", 140), ("b", 55)]);
            assert_eq!(log.downloaded_today(), 45);
        }
        let restored = ActivityLog::load(&root);
        assert_eq!(restored.downloaded_today(), 45);
        assert_eq!(restored.recent().len(), HISTORY_DAYS);
        let _ = fs::remove_dir_all(root);
    }
}
