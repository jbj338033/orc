use std::collections::HashMap;
use std::sync::Arc;

use orc_core::provider::ToolDef;

use super::traits::Tool;
use crate::runtime::ToolResult;

pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register(&mut self, tool: Arc<dyn Tool>) {
        let name = tool.definition().name;
        self.tools.insert(name, tool);
    }

    pub fn get(&self, name: &str) -> Option<&Arc<dyn Tool>> {
        self.tools.get(name)
    }

    pub fn definitions(&self) -> Vec<ToolDef> {
        self.tools.values().map(|t| t.definition()).collect()
    }

    pub fn not_found(name: &str) -> ToolResult {
        ToolResult {
            id: String::new(),
            output: format!("tool not found: {name}"),
            is_error: true,
        }
    }
}
