use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub prompt: String,
    pub allowed_tools: Option<Vec<String>>,
    pub disable_model_invocation: bool,
    pub context: SkillContext,
    pub triggers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum SkillContext {
    #[default]
    Inline,
    Fork,
}
