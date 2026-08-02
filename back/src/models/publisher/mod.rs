pub mod manager;
pub mod mastodon;
pub mod matrix;
pub mod sse;
pub mod template;
pub mod telegram;
pub mod types;
pub mod x;

// No re-exports needed at this level; all consumers import
// from the specific submodules (manager::, sse::, types::, etc.)