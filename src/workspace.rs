use anyhow::{bail, Result};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct Workspace {
    pub files: PathBuf,
}

impl Workspace {
    pub fn new(root: impl AsRef<Path>) -> Result<Self> {
        let files = root.as_ref().join("files");
        fs::create_dir_all(&files)?;
        Ok(Self {
            files: files.canonicalize().unwrap_or(files),
        })
    }

    pub fn resolve(&self, path: &str) -> Result<PathBuf> {
        let target = self.files.join(path);
        let resolved = if target.exists() {
            target.canonicalize()?
        } else {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            target
        };
        let root = self.files.to_string_lossy();
        if !resolved.starts_with(&*root) && !target.starts_with(&self.files) {
            bail!("Path escapes workspace: {path}");
        }
        Ok(resolved)
    }

    pub fn read(&self, path: &str) -> Result<String> {
        Ok(fs::read_to_string(self.resolve(path)?)?)
    }

    pub fn write(&self, path: &str, content: &str) -> Result<String> {
        let target = self.resolve(path)?;
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&target, content)?;
        Ok(format!("Wrote {path}"))
    }
}
