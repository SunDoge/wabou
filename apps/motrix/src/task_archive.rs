use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use aria2_ws::{TaskOptions, response::Status};
use serde::{Deserialize, Serialize};

const ARCHIVE_PREFIX: &str = "archive:";

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ArchivedTask {
    pub id: String,
    pub engine_gid: String,
    pub name: String,
    pub source: String,
    pub dir: String,
    pub out: Option<String>,
    pub file_path: Option<String>,
    pub payload_paths: Vec<PathBuf>,
    #[serde(default = "default_file_count")]
    pub file_count: usize,
    pub total_length: u64,
    pub completed_length: u64,
    pub uploaded_length: u64,
    #[serde(default)]
    pub options: TaskOptions,
}

fn default_file_count() -> usize {
    1
}

impl ArchivedTask {
    pub fn from_seeding(
        status: &Status,
        source: String,
        payload_paths: Vec<PathBuf>,
        options: TaskOptions,
    ) -> Self {
        let file_path = status
            .files
            .first()
            .map(|file| file.path.clone())
            .filter(|path| !path.is_empty());
        let name = file_path
            .as_deref()
            .and_then(|path| Path::new(path).file_name())
            .and_then(|name| name.to_str())
            .filter(|name| !name.is_empty())
            .unwrap_or(&status.gid)
            .to_owned();
        let out = (status.files.len() == 1)
            .then(|| {
                file_path
                    .as_deref()
                    .and_then(|path| Path::new(path).file_name())
                    .and_then(|name| name.to_str())
                    .map(str::to_owned)
            })
            .flatten();
        Self {
            id: format!("{ARCHIVE_PREFIX}{}", uuid::Uuid::new_v4().simple()),
            engine_gid: status.gid.clone(),
            name,
            source,
            dir: status.dir.clone(),
            out,
            file_path,
            payload_paths,
            file_count: status.files.len(),
            total_length: status.total_length,
            completed_length: status.completed_length,
            uploaded_length: status.upload_length,
            options,
        }
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct PersistedArchive {
    tasks: BTreeMap<String, ArchivedTask>,
}

pub struct TaskArchive {
    path: PathBuf,
    tasks: BTreeMap<String, ArchivedTask>,
}

impl TaskArchive {
    pub fn load(config_dir: &Path) -> Result<Self, String> {
        let path = config_dir.join("task-archive.json");
        let tasks = match fs::read_to_string(&path) {
            Ok(source) => {
                serde_json::from_str::<PersistedArchive>(&source)
                    .map_err(|error| format!("cannot parse {}: {error}", path.display()))?
                    .tasks
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => BTreeMap::new(),
            Err(error) => return Err(format!("cannot read {}: {error}", path.display())),
        };
        Ok(Self { path, tasks })
    }

    pub fn all(&self) -> impl Iterator<Item = &ArchivedTask> {
        self.tasks.values()
    }

    pub fn get(&self, id: &str) -> Option<&ArchivedTask> {
        self.tasks.get(id)
    }

    pub fn contains_engine_gid(&self, gid: &str) -> bool {
        self.tasks.values().any(|task| task.engine_gid == gid)
    }

    pub fn insert(&mut self, task: ArchivedTask) -> Result<(), String> {
        let id = task.id.clone();
        self.tasks.insert(id.clone(), task);
        if let Err(error) = self.save() {
            self.tasks.remove(&id);
            return Err(error);
        }
        Ok(())
    }

    pub fn remove(&mut self, id: &str) -> Result<Option<ArchivedTask>, String> {
        let Some(task) = self.tasks.remove(id) else {
            return Ok(None);
        };
        if let Err(error) = self.save() {
            self.tasks.insert(id.to_owned(), task);
            return Err(error);
        }
        Ok(Some(task))
    }

    pub fn clear(&mut self) -> Result<(), String> {
        let previous = std::mem::take(&mut self.tasks);
        if let Err(error) = self.save() {
            self.tasks = previous;
            return Err(error);
        }
        Ok(())
    }

    fn save(&self) -> Result<(), String> {
        let parent = self
            .path
            .parent()
            .ok_or_else(|| "task archive path has no parent".to_owned())?;
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        let bytes = serde_json::to_vec_pretty(&PersistedArchive {
            tasks: self.tasks.clone(),
        })
        .map_err(|error| error.to_string())?;
        fs::write(&self.path, bytes).map_err(|error| error.to_string())?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&self.path, fs::Permissions::from_mode(0o600))
                .map_err(|error| error.to_string())?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn archived(id: &str) -> ArchivedTask {
        ArchivedTask {
            id: id.to_owned(),
            engine_gid: "engine-1".into(),
            name: "Linux.iso".into(),
            source: "magnet:?xt=urn:btih:abc".into(),
            dir: "/downloads".into(),
            out: Some("Linux.iso".into()),
            file_path: Some("/downloads/Linux.iso".into()),
            payload_paths: vec![PathBuf::from("/downloads/Linux.iso")],
            file_count: 1,
            total_length: 10,
            completed_length: 10,
            uploaded_length: 3,
            options: TaskOptions::default(),
        }
    }

    #[test]
    fn persists_archived_tasks_and_engine_identity() {
        let root =
            std::env::temp_dir().join(format!("motrix-task-archive-{}", uuid::Uuid::new_v4()));
        let mut archive = TaskArchive::load(&root).unwrap();
        archive.insert(archived("archive:test")).unwrap();
        let mut restored = TaskArchive::load(&root).unwrap();
        assert!(restored.contains_engine_gid("engine-1"));
        assert_eq!(restored.get("archive:test").unwrap().name, "Linux.iso");
        assert!(restored.remove("archive:test").unwrap().is_some());
        assert!(TaskArchive::load(&root).unwrap().all().next().is_none());
        let _ = fs::remove_dir_all(root);
    }
}
