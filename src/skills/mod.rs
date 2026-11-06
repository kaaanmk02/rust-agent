
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Skill {
    pub name: String,
    pub content: String,
    pub path: PathBuf,
}

pub fn load_skills(dir: &Path) -> Vec<Skill> {
    let mut out = Vec::new();
    if !dir.exists() {
        return out;
    }
    if let Ok(entries) = fs::read_dir(dir) {
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                let skill_md = p.join("SKILL.md");
                if skill_md.exists() {
                    if let Ok(content) = fs::read_to_string(&skill_md) {
                        out.push(Skill {
                            name: p.file_name().unwrap_or_default().to_string_lossy().into(),
                            content,
                            path: skill_md,
                        });
                    }
                }
            }
        }
    }
    out
}
