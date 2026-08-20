use std::{
    collections::{BTreeMap, HashMap},
    fs,
    path::{Path, PathBuf},
};

use jiff::{Span, Timestamp};
use serde::{Deserialize, Serialize};

// Keep a complete 52-week grid. A multiple of seven also makes the UI's
// week columns stable without padding synthetic days at either edge.
const HISTORY_DAYS: usize = 364;

#[derive(Debug, Default, Deserialize, Serialize)]
struct PersistedActivity {
    days: BTreeMap<String, u64>,
    #[serde(default)]
    downloaded_total: u64,
    #[serde(default)]
    uploaded_total: u64,
}

/// Persistent transfer accounting derived from per-task completed-byte deltas.
pub struct ActivityLog {
    path: PathBuf,
    days: BTreeMap<String, u64>,
    transferred_by_id: HashMap<String, (u64, u64)>,
    downloaded_total: u64,
    uploaded_total: u64,
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
            transferred_by_id: HashMap::new(),
            downloaded_total: persisted.downloaded_total,
            uploaded_total: persisted.uploaded_total,
            baselined: false,
            dirty: false,
            dirty_ticks: 0,
        }
    }

    pub fn observe<'a>(&mut self, tasks: impl IntoIterator<Item = (&'a str, u64, u64)>) {
        let next: HashMap<String, (u64, u64)> = tasks
            .into_iter()
            .map(|(id, downloaded, uploaded)| (id.to_owned(), (downloaded, uploaded)))
            .collect();
        if self.baselined {
            let (downloaded, uploaded) = next
                .iter()
                .map(|(id, &(completed, uploaded))| {
                    self.transferred_by_id.get(id).map_or(
                        (0, 0),
                        |&(previous_completed, previous_uploaded)| {
                            (
                                completed.saturating_sub(previous_completed),
                                uploaded.saturating_sub(previous_uploaded),
                            )
                        },
                    )
                })
                .fold((0_u64, 0_u64), |(download_total, upload_total), next| {
                    (
                        download_total.saturating_add(next.0),
                        upload_total.saturating_add(next.1),
                    )
                });
            if downloaded > 0 {
                *self.days.entry(today()).or_default() += downloaded;
            }
            self.downloaded_total = self.downloaded_total.saturating_add(downloaded);
            self.uploaded_total = self.uploaded_total.saturating_add(uploaded);
            self.dirty |= downloaded > 0 || uploaded > 0;
        }
        self.baselined = true;
        self.transferred_by_id = next;
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

    pub fn downloaded_total(&self) -> u64 {
        self.downloaded_total
    }

    pub fn uploaded_total(&self) -> u64 {
        self.uploaded_total
    }

    fn save(&mut self) -> Result<(), String> {
        let parent = self
            .path
            .parent()
            .ok_or_else(|| "activity path has no parent".to_owned())?;
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        let source = serde_json::to_vec_pretty(&PersistedActivity {
            days: self.days.clone(),
            downloaded_total: self.downloaded_total,
            uploaded_total: self.uploaded_total,
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
            log.observe([("a", 100, 20), ("b", 50, 5)]);
            assert_eq!(log.downloaded_today(), 0);
            log.observe([("a", 140, 32), ("b", 55, 8)]);
            assert_eq!(log.downloaded_today(), 45);
            assert_eq!(log.downloaded_total(), 45);
            assert_eq!(log.uploaded_total(), 15);
        }
        let restored = ActivityLog::load(&root);
        assert_eq!(restored.downloaded_today(), 45);
        assert_eq!(restored.downloaded_total(), 45);
        assert_eq!(restored.uploaded_total(), 15);
        assert_eq!(restored.recent().len(), HISTORY_DAYS);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn loads_legacy_day_only_activity_files() {
        let root =
            std::env::temp_dir().join(format!("motrix-legacy-activity-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        fs::write(
            root.join("activity.json"),
            format!(r#"{{"days":{{"{}":123}}}}"#, today()),
        )
        .unwrap();

        let log = ActivityLog::load(&root);
        assert_eq!(log.downloaded_today(), 123);
        assert_eq!(log.downloaded_total(), 0);
        assert_eq!(log.uploaded_total(), 0);
        let _ = fs::remove_dir_all(root);
    }
}
