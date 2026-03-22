mod bash;
mod file_edit;
mod file_read;
mod file_write;
mod grep;
mod r#glob;
mod traits;

pub use traits::*;

use std::collections::BTreeMap;

pub struct ToolRegistry {
    tools: BTreeMap<String, Box<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            tools: BTreeMap::new(),
        };
        registry.register(Box::new(bash::BashTool));
        registry.register(Box::new(file_read::FileReadTool));
        registry.register(Box::new(file_write::FileWriteTool));
        registry.register(Box::new(file_edit::FileEditTool));
        registry.register(Box::new(grep::GrepTool));
        registry.register(Box::new(r#glob::GlobTool));
        registry
    }

    fn register(&mut self, tool: Box<dyn Tool>) {
        let def = tool.definition();
        self.tools.insert(def.name.clone(), tool);
    }

    pub fn get(&self, name: &str) -> Option<&dyn Tool> {
        self.tools.get(name).map(|t| t.as_ref())
    }

    pub fn definitions(&self) -> Vec<ToolDefinition> {
        self.tools.values().map(|t| t.definition()).collect()
    }
}
