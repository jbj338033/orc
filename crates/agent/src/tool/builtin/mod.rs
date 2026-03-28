mod ask_user;
mod bash;
mod edit;
mod glob;
mod grep;
mod read;
mod web_fetch;
mod write;

pub use ask_user::AskUser;
pub use bash::BashExec;
pub use edit::EditFile;
pub use glob::GlobSearch;
pub use grep::Grep;
pub use read::ReadFile;
pub use web_fetch::WebFetch;
pub use write::WriteFile;

use std::sync::Arc;

use super::ToolRegistry;

pub fn register_all(registry: &mut ToolRegistry) {
    registry.register(Arc::new(ReadFile));
    registry.register(Arc::new(WriteFile));
    registry.register(Arc::new(EditFile));
    registry.register(Arc::new(BashExec));
    registry.register(Arc::new(Grep));
    registry.register(Arc::new(GlobSearch));
    registry.register(Arc::new(WebFetch));
    registry.register(Arc::new(AskUser));
}
