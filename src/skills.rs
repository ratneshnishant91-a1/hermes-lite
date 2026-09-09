use anyhow::{bail, Result};
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Skills {
    root: PathBuf,
}

impl Skills {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn catalog_text(&self) -> String {
        let mut lines = vec!["Available skills:".to_string()];
        if let Ok(entries) = fs::read_dir(&self.root) {
            for e in entries.flatten() {
                let skill = e.path().join("SKILL.md");
                if skill.exists() {
                    let name = e.file_name().to_string_lossy().into_owned();
                    let desc = fs::read_to_string(&skill)
                        .ok()
                        .and_then(|t| {
                            t.lines()
                                .find(|l| l.to_lowercase().starts_with("description:"))
                                .map(|l| l.splitn(2, ':').nth(1).unwrap_or("").trim().to_string())
                        })
                        .unwrap_or_else(|| "No description.".into());
                    lines.push(format!("- {name}: {desc}"));
                }
            }
        }
        if lines.len() == 1 {
            lines.push("- none yet (self-learning will create them)".into());
        }
        lines.push("Load a skill with skill_load before following it.".into());
        lines.join("\n")
    }

    pub fn load(&self, name: &str) -> Result<String> {
        let path = self.root.join(name).join("SKILL.md");
        if !path.exists() {
            bail!("Unknown skill: {name}");
        }
        Ok(fs::read_to_string(path)?)
    }

    pub fn write_skill(&self, name: &str, body: &str) -> Result<()> {
        let dir = self.root.join(name);
        fs::create_dir_all(&dir)?;
        fs::write(dir.join("SKILL.md"), body)?;
        Ok(())
    }
}
