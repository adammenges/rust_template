//! Typed storage boundary. Missing files use defaults; malformed files are preserved.
use crate::{
    app_paths::AppPaths,
    domain::{RuntimeState, Settings},
};
use anyhow::{Context, Result, bail};
use serde::{Serialize, de::DeserializeOwned};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::Path,
};
const MAX_DOCUMENT_BYTES: u64 = 64 * 1024;

#[derive(Clone)]
pub struct Store {
    paths: AppPaths,
}
impl Store {
    pub fn new(paths: AppPaths) -> Self {
        Self { paths }
    }
    pub fn load_settings(&self) -> Result<Settings> {
        let value: Settings = read(&self.paths.settings())?;
        value.validate().map_err(anyhow::Error::msg)?;
        Ok(value)
    }
    pub fn load_state(&self) -> Result<RuntimeState> {
        read(&self.paths.state())
    }
    pub fn save_settings(&self, value: &Settings) -> Result<()> {
        value.validate().map_err(anyhow::Error::msg)?;
        atomic_write(&self.paths.settings(), value)
    }
    pub fn save_state(&self, value: &RuntimeState) -> Result<()> {
        atomic_write(&self.paths.state(), value)
    }
}
fn read<T: DeserializeOwned + Default>(path: &Path) -> Result<T> {
    // Reject FIFOs/devices before opening: opening a FIFO can indefinitely block startup.
    // A symlink (including a dangling one) is an error, never a missing document.
    match fs::symlink_metadata(path) {
        Ok(metadata) if !metadata.file_type().is_file() => bail!(
            "{} must be a regular file; preserved on disk",
            path.display()
        ),
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(T::default()),
        Err(e) => return Err(e).with_context(|| format!("Inspect {}", path.display())),
    }
    let file = File::open(path).with_context(|| format!("Read {}", path.display()))?;
    let mut bytes = Vec::new();
    file.take(MAX_DOCUMENT_BYTES + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_DOCUMENT_BYTES {
        bail!("{} exceeds 64 KiB", path.display());
    }
    serde_json::from_slice(&bytes)
        .with_context(|| format!("Invalid {}; preserved on disk", path.display()))
}
fn atomic_write<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let bytes = serde_json::to_vec_pretty(value)?;
    if bytes.len() as u64 + 1 > MAX_DOCUMENT_BYTES {
        bail!("Document exceeds 64 KiB including its trailing newline");
    }
    if let Ok(metadata) = fs::symlink_metadata(path)
        && !metadata.file_type().is_file()
    {
        bail!(
            "{} must be a regular file; preserved on disk",
            path.display()
        );
    }
    let parent = path.parent().context("Storage path has no parent")?;
    // One process lease and one writer: a fixed sibling is safe. No replacement until sync succeeds.
    let temporary = path.with_extension("json.pending");
    let result = (|| -> Result<()> {
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        // Leftover crash files are never treated as committed state.
        match fs::symlink_metadata(&temporary) {
            Ok(_) => fs::remove_file(&temporary).context("Remove stale pending file")?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error).context("Inspect pending file"),
        }
        let mut file = options.open(&temporary)?;
        file.write_all(&bytes)?;
        file.write_all(b"\n")?;
        file.sync_all()?;
        fs::rename(&temporary, path)?;
        // A failed directory sync is observable, but rename already committed the new file.
        File::open(parent)?
            .sync_all()
            .context("File replaced, but directory sync failed; restart to reload")?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result.with_context(|| format!("Save {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn round_trip_is_atomic_and_validation_preserves_previous_file() {
        let d = tempfile::tempdir().unwrap();
        let store = Store::new(AppPaths {
            root: d.path().into(),
        });
        assert_eq!(store.load_settings().unwrap(), Settings::default());
        let valid = Settings {
            workspace_name: "Research".into(),
            compact: true,
            ..Settings::default()
        };
        store.save_settings(&valid).unwrap();
        let before = fs::read(d.path().join("settings.json")).unwrap();
        let invalid = Settings {
            workspace_name: "".into(),
            ..valid.clone()
        };
        assert!(store.save_settings(&invalid).is_err());
        assert_eq!(before, fs::read(d.path().join("settings.json")).unwrap());
        assert_eq!(store.load_settings().unwrap(), valid);
        assert!(!d.path().join("settings.json.pending").exists());
    }
    #[test]
    fn malformed_and_oversized_files_are_not_defaults() {
        let d = tempfile::tempdir().unwrap();
        let store = Store::new(AppPaths {
            root: d.path().into(),
        });
        fs::write(store.paths.settings(), b"{broken").unwrap();
        assert!(store.load_settings().is_err());
        assert_eq!(fs::read(store.paths.settings()).unwrap(), b"{broken");
        fs::write(store.paths.settings(), vec![b' '; 65537]).unwrap();
        assert!(store.load_settings().is_err());
    }
    #[test]
    fn stale_crash_file_is_replaced_without_becoming_committed_state() {
        let d = tempfile::tempdir().unwrap();
        let store = Store::new(AppPaths {
            root: d.path().into(),
        });
        fs::write(
            d.path().join("settings.json.pending"),
            b"partial crash output",
        )
        .unwrap();
        assert_eq!(store.load_settings().unwrap(), Settings::default());
        store.save_settings(&Settings::default()).unwrap();
        assert!(!d.path().join("settings.json.pending").exists());
        assert_eq!(store.load_settings().unwrap(), Settings::default());
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            fs::metadata(store.paths.settings())
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }
    #[test]
    fn nonregular_documents_are_preserved_and_rejected() {
        let d = tempfile::tempdir().unwrap();
        let store = Store::new(AppPaths {
            root: d.path().into(),
        });
        fs::create_dir(store.paths.settings()).unwrap();
        assert!(store.load_settings().is_err());
        assert!(store.save_settings(&Settings::default()).is_err());
        fs::remove_dir(store.paths.settings()).unwrap();
        let outside = d.path().join("outside.json");
        fs::write(&outside, b"do not replace").unwrap();
        std::os::unix::fs::symlink(&outside, store.paths.settings()).unwrap();
        assert!(store.load_settings().is_err());
        assert!(store.save_settings(&Settings::default()).is_err());
        assert_eq!(fs::read(outside).unwrap(), b"do not replace");
        fs::remove_file(store.paths.settings()).unwrap();
        std::os::unix::fs::symlink(d.path().join("missing"), store.paths.settings()).unwrap();
        assert!(store.load_settings().is_err());
    }
    #[test]
    fn future_and_unknown_settings_are_not_silently_downgraded() {
        let d = tempfile::tempdir().unwrap();
        let store = Store::new(AppPaths {
            root: d.path().into(),
        });
        for document in [
            r#"{"schema_version":2}"#,
            r#"{"unknown":true}"#,
            r#"{"workspace_name":42}"#,
        ] {
            fs::write(store.paths.settings(), document).unwrap();
            assert!(store.load_settings().is_err());
            assert_eq!(
                fs::read_to_string(store.paths.settings()).unwrap(),
                document
            );
        }
    }
    #[test]
    fn fifo_document_is_rejected_without_opening_it() {
        let d = tempfile::tempdir().unwrap();
        let store = Store::new(AppPaths {
            root: d.path().into(),
        });
        assert!(
            std::process::Command::new("mkfifo")
                .arg(store.paths.settings())
                .status()
                .unwrap()
                .success()
        );
        assert!(store.load_settings().is_err());
        assert!(store.save_settings(&Settings::default()).is_err());
    }
    #[test]
    fn serialized_size_limit_includes_the_trailing_newline() {
        let d = tempfile::tempdir().unwrap();
        let path = d.path().join("bounded.json");
        let maximum = "a".repeat(MAX_DOCUMENT_BYTES as usize - 3); // Two quotes plus newline.
        atomic_write(&path, &maximum).unwrap();
        assert_eq!(fs::metadata(&path).unwrap().len(), MAX_DOCUMENT_BYTES);
        assert_eq!(read::<String>(&path).unwrap(), maximum);
        assert!(atomic_write(&path, &format!("{maximum}a")).is_err());
        assert_eq!(read::<String>(&path).unwrap(), maximum);
    }
    #[test]
    fn runtime_result_round_trips() {
        let d = tempfile::tempdir().unwrap();
        let store = Store::new(AppPaths {
            root: d.path().into(),
        });
        let state = RuntimeState {
            last_result: Some(crate::domain::JobResult {
                samples: 10,
                checksum: 42,
            }),
        };
        store.save_state(&state).unwrap();
        assert_eq!(store.load_state().unwrap(), state);
    }
}
