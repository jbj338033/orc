use std::collections::HashMap;
use std::path::Path;

use super::parser::parse_skill_md;
use super::types::Skill;

pub struct SkillRegistry {
    skills: HashMap<String, Skill>,
}

impl SkillRegistry {
    pub fn new() -> Self {
        Self {
            skills: HashMap::new(),
        }
    }

    pub fn register(&mut self, skill: Skill) {
        self.skills.insert(skill.name.clone(), skill);
    }

    pub fn get(&self, name: &str) -> Option<&Skill> {
        self.skills.get(name)
    }

    pub fn find_by_trigger(&self, input: &str) -> Vec<&Skill> {
        let lower = input.to_lowercase();
        self.skills
            .values()
            .filter(|s| {
                s.triggers.iter().any(|t| lower.contains(&t.to_lowercase()))
                    || (!s.disable_model_invocation
                        && lower.contains(&s.name.to_lowercase()))
            })
            .collect()
    }

    pub fn list(&self) -> Vec<&Skill> {
        self.skills.values().collect()
    }

    pub fn load_dir(&mut self, dir: &Path) -> std::io::Result<usize> {
        let mut count = 0;

        if !dir.exists() {
            return Ok(0);
        }

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // look for SKILL.md inside directory
                let skill_file = path.join("SKILL.md");
                if skill_file.exists() {
                    let name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                        .to_string();
                    let content = std::fs::read_to_string(&skill_file)?;
                    if let Some(skill) = parse_skill_md(&name, &content) {
                        self.register(skill);
                        count += 1;
                    }
                }
            } else if path.extension().is_some_and(|ext| ext == "md") {
                let name = path
                    .file_stem()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                let content = std::fs::read_to_string(&path)?;
                if let Some(skill) = parse_skill_md(&name, &content) {
                    self.register(skill);
                    count += 1;
                }
            }
        }

        Ok(count)
    }
}
