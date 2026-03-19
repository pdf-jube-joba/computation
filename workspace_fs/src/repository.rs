use anyhow::{Context, Result, anyhow, bail};
use async_trait::async_trait;
use camino::{Utf8Path, Utf8PathBuf};
use tokio::fs;

use crate::{
    config::RepositoryConfig,
    info::{PathInfo, PathInfoKind},
    path::WorkspacePath,
};

// .repo 以外を書き換えるインターフェースの提供
#[async_trait]
pub trait Repository: Send + Sync {
    async fn list_directory(&self, path: &WorkspacePath) -> Result<Vec<String>>;
    async fn path_info(&self, path: &WorkspacePath) -> Result<PathInfo>;
    async fn create_directory(&self, path: &WorkspacePath) -> Result<()>;
    async fn delete_directory(&self, path: &WorkspacePath) -> Result<()>;
    async fn read_file(&self, path: &WorkspacePath) -> Result<Vec<u8>>;
    async fn create_text_file(&self, path: &WorkspacePath, content: &str) -> Result<()>;
    async fn write_text_file(&self, path: &WorkspacePath, content: &str) -> Result<()>;
    async fn delete_file(&self, path: &WorkspacePath) -> Result<()>;
}

pub struct FsRepository {
    repository_root: Utf8PathBuf,
    mounts: Vec<MountedDirectory>,
}

struct MountedDirectory {
    alias: WorkspacePath,
    source: WorkspacePath,
}

impl FsRepository {
    pub fn open(path: &Utf8Path, config: &RepositoryConfig) -> Result<Self> {
        let canonical = std::fs::canonicalize(path)
            .with_context(|| format!("failed to resolve repository path: {}", path))?;
        let repository_root = Utf8PathBuf::from_path_buf(canonical)
            .map_err(|_| anyhow!("repository path must be UTF-8"))?;

        if !repository_root.is_dir() {
            bail!("repository path must be a directory");
        }

        Ok(Self {
            repository_root,
            mounts: Self::mounts_from_config(config),
        })
    }

    pub fn repository_root(&self) -> &Utf8Path {
        &self.repository_root
    }

    fn mounts_from_config(config: &RepositoryConfig) -> Vec<MountedDirectory> {
        config
            .plugin
            .iter()
            .filter_map(|plugin| {
                plugin.mount.as_ref().map(|mount| MountedDirectory {
                    alias: WorkspacePath::from_path_str(mount.trim_start_matches('/'))
                        .expect("validated mount alias should parse"),
                    source: WorkspacePath::from_path_str(&format!(".repo/{}/generated", plugin.name))
                        .expect("generated plugin path should parse"),
                })
            })
            .collect()
    }

    fn resolve_path(&self, requested_path: &WorkspacePath) -> Result<Utf8PathBuf> {
        if let Some(resolved) = self.resolve_mounted_path(requested_path) {
            return Ok(resolved);
        }
        self.ensure_not_reserved_path(requested_path)?;
        Ok(requested_path.join_to(&self.repository_root))
    }

    fn ensure_parent_directory_exists(&self, path: &Utf8Path) -> Result<()> {
        let Some(parent) = path.parent() else {
            bail!("parent directory not found");
        };

        if !parent.exists() {
            bail!("parent directory not found");
        }

        if !parent.is_dir() {
            bail!("parent path is not a directory");
        }

        Ok(())
    }

    fn resolve_mounted_path(&self, requested_path: &WorkspacePath) -> Option<Utf8PathBuf> {
        let mount = self
            .mounts
            .iter()
            .find(|mount| requested_path.starts_with(&mount.alias))?;
        let relative = requested_path.strip_prefix(&mount.alias)?;
        let mut resolved = mount.source.join_to(&self.repository_root);
        if !relative.is_empty() {
            resolved.push(relative);
        }
        Some(resolved)
    }

    fn ensure_not_reserved_path(&self, requested_path: &WorkspacePath) -> Result<()> {
        if requested_path.is_reserved() {
            bail!("reserved path");
        }
        Ok(())
    }

    fn read_directory_entries(&self, directory: &Utf8Path) -> Result<Vec<String>> {
        let mut entries = Vec::new();
        for dir_entry in std::fs::read_dir(directory.as_std_path())? {
            let dir_entry = dir_entry?;
            let path = dir_entry.path();
            let utf8 = Utf8PathBuf::from_path_buf(path).map_err(|_| anyhow!("non-UTF-8 path"))?;

            let mut entry = utf8
                .file_name()
                .ok_or_else(|| anyhow!("invalid directory entry"))?
                .to_owned();

            if utf8.is_dir() {
                entry.push('/');
            }

            entries.push(entry);
        }

        entries.sort();
        Ok(entries)
    }
}

#[async_trait]
impl Repository for FsRepository {
    async fn list_directory(&self, path: &WorkspacePath) -> Result<Vec<String>> {
        let directory = self.resolve_path(path)?;
        if !directory.is_dir() {
            bail!("not a directory");
        }

        self.read_directory_entries(&directory)
    }

    async fn path_info(&self, path: &WorkspacePath) -> Result<PathInfo> {
        let resolved = self.resolve_path(path)?;
        let metadata = fs::metadata(resolved.as_std_path())
            .await
            .context("failed to read metadata")?;
        let kind = if metadata.is_dir() {
            PathInfoKind::Directory
        } else {
            PathInfoKind::File
        };
        let size = match kind {
            PathInfoKind::File => Some(metadata.len()),
            PathInfoKind::Directory => None,
        };

        Ok(PathInfo::new(
            path.as_str(),
            kind,
            size,
            metadata.modified().ok(),
            metadata.permissions().readonly(),
        ))
    }

    async fn create_directory(&self, path: &WorkspacePath) -> Result<()> {
        let resolved = self.resolve_path(path)?;

        if resolved.exists() {
            bail!("directory already exists");
        }

        self.ensure_parent_directory_exists(&resolved)?;

        fs::create_dir(resolved.as_std_path())
            .await
            .context("failed to create directory")?;
        Ok(())
    }

    async fn delete_directory(&self, path: &WorkspacePath) -> Result<()> {
        let resolved = self.resolve_path(path)?;

        if !resolved.exists() {
            bail!("directory not found");
        }

        if !resolved.is_dir() {
            bail!("path is not a directory");
        }

        if std::fs::read_dir(resolved.as_std_path())?.next().is_some() {
            bail!("directory is not empty");
        }

        fs::remove_dir(resolved.as_std_path())
            .await
            .context("failed to delete directory")?;
        Ok(())
    }

    async fn read_file(&self, path: &WorkspacePath) -> Result<Vec<u8>> {
        let resolved = self.resolve_path(path)?;
        fs::read(resolved.as_std_path())
            .await
            .context("failed to read file")
    }

    async fn create_text_file(&self, path: &WorkspacePath, content: &str) -> Result<()> {
        let resolved = self.resolve_path(path)?;

        if resolved.exists() {
            bail!("file already exists");
        }

        self.ensure_parent_directory_exists(&resolved)?;

        fs::write(resolved.as_std_path(), content)
            .await
            .context("failed to create file")?;
        Ok(())
    }

    async fn write_text_file(&self, path: &WorkspacePath, content: &str) -> Result<()> {
        let resolved = self.resolve_path(path)?;

        if !resolved.exists() {
            bail!("file not found");
        }

        if resolved.is_dir() {
            bail!("path is a directory");
        }

        fs::write(resolved.as_std_path(), content)
            .await
            .context("failed to write file")?;
        Ok(())
    }

    async fn delete_file(&self, path: &WorkspacePath) -> Result<()> {
        let resolved = self.resolve_path(path)?;

        if !resolved.exists() {
            bail!("file not found");
        }

        if resolved.is_dir() {
            bail!("path is a directory");
        }

        fs::remove_file(resolved.as_std_path())
            .await
            .context("failed to delete file")?;
        Ok(())
    }
}
