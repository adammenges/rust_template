//! All per-user paths and the exclusive process lease are owned here.
use crate::identity::STORAGE_NAME;
use anyhow::{Context, Result};
use fs2::FileExt;
use std::{
    fs::{self, File, OpenOptions},
    path::PathBuf,
};

#[derive(Clone, Debug)]
pub struct AppPaths {
    pub root: PathBuf,
}
impl AppPaths {
    pub fn discover(override_path: Option<PathBuf>) -> Result<Self> {
        let root = match override_path {
            Some(path) => path,
            None => dirs::data_local_dir()
                .context("No per-user application data directory")?
                .join(STORAGE_NAME),
        };
        let mut builder = fs::DirBuilder::new();
        builder.recursive(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700); // Private from creation; no transient public directory.
        }
        builder
            .create(&root)
            .context("Create application directory")?;
        if !fs::symlink_metadata(&root)?.file_type().is_dir() {
            anyhow::bail!("Application data directory must be a real directory, not a symlink");
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if fs::metadata(&root)?.permissions().mode() & 0o077 != 0 {
                anyhow::bail!(
                    "Application data directory must be private (mode 0700): {}. Choose a dedicated --data-dir.",
                    root.display()
                );
            }
        }
        Ok(Self { root })
    }
    pub fn settings(&self) -> PathBuf {
        self.root.join("settings.json")
    }
    pub fn state(&self) -> PathBuf {
        self.root.join("state.json")
    }
    pub fn acquire(&self) -> Result<InstanceLease> {
        let mut options = OpenOptions::new();
        options.create(true).read(true).write(true).truncate(false);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let path = self.root.join("instance.lock");
        if let Ok(metadata) = fs::symlink_metadata(&path)
            && !metadata.file_type().is_file()
        {
            anyhow::bail!("Instance lock must be a regular file, not a symlink or device");
        }
        let file = options.open(path)?;
        file.try_lock_exclusive()
            .context("Another instance owns this data directory; close it or use --data-dir")?;
        Ok(InstanceLease { _file: file })
    }
}
pub struct InstanceLease {
    _file: File,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn a_second_writer_is_rejected_until_the_owner_exits() {
        let dir = tempfile::tempdir().unwrap();
        let paths = AppPaths::discover(Some(dir.path().join("owned"))).unwrap();
        let owner = paths.acquire().unwrap();
        assert!(paths.acquire().is_err());
        drop(owner);
        assert!(paths.acquire().is_ok());
    }
    #[test]
    fn symlinked_root_and_lock_are_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let owned = AppPaths::discover(Some(dir.path().join("owned"))).unwrap();
        let alias = dir.path().join("alias");
        std::os::unix::fs::symlink(&owned.root, &alias).unwrap();
        assert!(AppPaths::discover(Some(alias)).is_err());
        std::os::unix::fs::symlink(
            dir.path().join("elsewhere"),
            owned.root.join("instance.lock"),
        )
        .unwrap();
        assert!(owned.acquire().is_err());
        assert!(!dir.path().join("elsewhere").exists());
    }
    #[test]
    fn an_existing_shared_directory_is_not_repermissioned() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o755)).unwrap();
        assert!(AppPaths::discover(Some(dir.path().into())).is_err());
        assert_eq!(
            fs::metadata(dir.path()).unwrap().permissions().mode() & 0o777,
            0o755
        );
    }
}
