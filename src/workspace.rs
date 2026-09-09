use anyhow::{bail, Result};
use std::fs;
use std::path::{Component, Path, PathBuf};

#[derive(Clone, Debug)]
pub struct Workspace {
    pub files: PathBuf,
}

impl Workspace {
    pub fn new(root: impl AsRef<Path>) -> Result<Self> {
        let files = root.as_ref().join("files");
        fs::create_dir_all(&files)?;
        let files = files.canonicalize()?;
        Ok(Self { files })
    }

    /// Join `path` under the workspace. Rejects `..`, absolute paths, and prefixes.
    pub fn resolve(&self, path: &str) -> Result<PathBuf> {
        if path.is_empty() || path.contains('\0') {
            bail!("invalid path");
        }
        let mut out = PathBuf::new();
        for c in Path::new(path).components() {
            match c {
                Component::CurDir => {}
                Component::Normal(p) => out.push(p),
                Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                    bail!("Path escapes workspace: {path}");
                }
            }
        }
        let joined = self.files.join(out);
        if !joined.starts_with(&self.files) {
            bail!("Path escapes workspace: {path}");
        }
        Ok(joined)
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
