//! Load and save the store to `~/.local/share/t2d2/tasks.json`.
//!
//! The public API uses the standard XDG location. Internally, the work goes
//! through `load_from` / `save_to` which take an explicit path — that seam
//! exists so tests can use a temp file.

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::state::{Store, Task};

const DEBOUNCE_MS: u64 = 150;

#[derive(Debug, Serialize, Deserialize)]
struct Persisted {
    tasks: Vec<Task>,
}

/// The default storage location, per XDG.
pub fn storage_path() -> PathBuf {
    let base = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("t2d2").join("tasks.json")
}

pub fn load() -> io::Result<Store> {
    load_from(&storage_path())
}

pub fn save(store: &Store) -> io::Result<()> {
    save_to(&storage_path(), store)
}

/// Test seam: load from an arbitrary path.
pub fn load_from(path: &Path) -> io::Result<Store> {
    let raw = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Store::empty()),
        Err(e) => return Err(e),
    };
    match serde_json::from_str::<Persisted>(&raw) {
        Ok(p) => {
            let next_id = p.tasks.iter().map(|t| t.id).max().unwrap_or(0) + 1;
            Ok(Store {
                tasks: p.tasks,
                undo: Vec::new(),
                cursor: 0,
                next_id,
            })
        }
        Err(e) => {
            eprintln!("t2d2: failed to parse {}: {e}", path.display());
            Ok(Store::empty())
        }
    }
}

/// Test seam: save to an arbitrary path, creating parent dirs.
pub fn save_to(path: &Path, store: &Store) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    let persisted = Persisted {
        tasks: store.tasks.clone(),
    };
    let json = serde_json::to_string_pretty(&persisted)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    let mut tmp = path.to_path_buf();
    tmp.set_extension("json.tmp");
    {
        let mut f = fs::File::create(&tmp)?;
        f.write_all(json.as_bytes())?;
        f.sync_all()?;
    }
    fs::rename(&tmp, path)?;
    Ok(())
}

/// Synchronous flush. Used at quit and on SIGINT.
pub fn flush_save(store: &Store) -> io::Result<()> {
    save(store)
}

/// Debounced save. Returns the duration until the save fires.
pub fn schedule_save(store: &Store) -> Duration {
    let _ = save(store);
    Duration::from_millis(DEBOUNCE_MS)
}