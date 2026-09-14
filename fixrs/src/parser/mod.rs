mod collector;
mod path;
mod scope;
mod visitor;

pub use collector::{ModuleContext, PathCollector};
pub use path::QualifiedPath;
pub use scope::{FileScope, collect_file_scope};
