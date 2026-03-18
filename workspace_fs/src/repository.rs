use std::path::PathBuf;

use anyhow::{Context, Result, anyhow, bail};
use async_trait::async_trait;
use camino::{Utf8Path, Utf8PathBuf};
use tokio::fs;

// .repo 以外を書き換えるインターフェースの提供
#[async_trait]
pub trait Repository: Send + Sync {
    fn repository_root(&self) -> &Utf8Path;
    async fn list_directory(&self, path: &str) -> Result<Vec<String>>;
    async fn create_directory(&self, path: &str) -> Result<()>;
    async fn delete_directory(&self, path: &str) -> Result<()>;
    async fn read_file(&self, path: &str) -> Result<Vec<u8>>;
    async fn create_text_file(&self, path: &str, content: &str) -> Result<()>;
    async fn write_text_file(&self, path: &str, content: &str) -> Result<()>;
    async fn delete_file(&self, path: &str) -> Result<()>;
}

pub struct FsRepository {
    repository_root: Utf8PathBuf,
}

impl FsRepository {
    pub fn open(argument: String) -> Result<Self> {
        let path = PathBuf::from(argument);
        let canonical = std::fs::canonicalize(&path)
            .with_context(|| format!("failed to resolve repository path: {}", path.display()))?;
        let repository_root = Utf8PathBuf::from_path_buf(canonical)
            .map_err(|_| anyhow!("repository path must be UTF-8"))?;

        if !repository_root.is_dir() {
            bail!("repository path must be a directory");
        }

        Ok(Self { repository_root })
    }

    fn resolve_repository_path(&self, requested_path: &str) -> Result<Utf8PathBuf> {
        Ok(self.repository_root.join(requested_path.trim()))
    }

    fn resolve_directory_path(&self, requested_path: &str) -> Result<Utf8PathBuf> {
        if requested_path.is_empty() {
            return Ok(self.repository_root.clone());
        }

        Ok(self
            .repository_root
            .join(requested_path.trim().trim_end_matches('/')))
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
}

#[async_trait]
impl Repository for FsRepository {
    fn repository_root(&self) -> &Utf8Path {
        &self.repository_root
    }

    async fn list_directory(&self, path: &str) -> Result<Vec<String>> {
        let directory = self.resolve_directory_path(path)?;
        if !directory.is_dir() {
            bail!("not a directory");
        }

        let mut entries = Vec::new();
        for dir_entry in std::fs::read_dir(directory.as_std_path())? {
            let dir_entry = dir_entry?;
            let path = dir_entry.path();
            let utf8 = Utf8PathBuf::from_path_buf(path).map_err(|_| anyhow!("non-UTF-8 path"))?;
            utf8.strip_prefix(&self.repository_root)?;

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

    async fn create_directory(&self, path: &str) -> Result<()> {
        let resolved = self.resolve_directory_path(path)?;

        if resolved.exists() {
            bail!("directory already exists");
        }

        self.ensure_parent_directory_exists(&resolved)?;

        fs::create_dir(resolved.as_std_path())
            .await
            .context("failed to create directory")?;
        Ok(())
    }

    async fn delete_directory(&self, path: &str) -> Result<()> {
        let resolved = self.resolve_directory_path(path)?;

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

    async fn read_file(&self, path: &str) -> Result<Vec<u8>> {
        let resolved = self.resolve_repository_path(path)?;
        let content = fs::read(resolved.as_std_path())
            .await
            .context("failed to read file")?;
        Ok(content)
    }

    async fn create_text_file(&self, path: &str, content: &str) -> Result<()> {
        let resolved = self.resolve_repository_path(path)?;

        if resolved.exists() {
            bail!("file already exists");
        }

        self.ensure_parent_directory_exists(&resolved)?;

        fs::write(resolved.as_std_path(), content)
            .await
            .context("failed to create file")?;
        Ok(())
    }

    async fn write_text_file(&self, path: &str, content: &str) -> Result<()> {
        let resolved = self.resolve_repository_path(path)?;

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

    async fn delete_file(&self, path: &str) -> Result<()> {
        let resolved = self.resolve_repository_path(path)?;

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
