use anyhow::{bail, Result};
use std::{fs, path::{Component, Path, PathBuf}};

#[derive(Debug, Clone)]
pub struct Workspace { root: PathBuf }

impl Workspace {
    pub fn new(root: impl AsRef<Path>) -> Result<Self> {
        fs::create_dir_all(root.as_ref())?;
        Ok(Self { root: root.as_ref().canonicalize()? })
    }
    pub fn resolve(&self, input: &str) -> Result<PathBuf> {
        if input.is_empty() || input.contains('\0') { bail!("invalid path"); }
        let mut relative = PathBuf::new();
        for part in Path::new(input).components() {
            match part {
                Component::CurDir => {},
                Component::Normal(v) => relative.push(v),
                Component::ParentDir | Component::RootDir | Component::Prefix(_) => bail!("path escapes workspace"),
            }
        }
        Ok(self.root.join(relative))
    }
    pub fn read(&self, path: &str) -> Result<String> { Ok(fs::read_to_string(self.resolve(path)?)?) }
    pub fn write(&self, path: &str, body: &str) -> Result<()> {
        let target = self.resolve(path)?;
        if let Some(parent) = target.parent() { fs::create_dir_all(parent)?; }
        fs::write(target, body)?;
        Ok(())
    }
}
