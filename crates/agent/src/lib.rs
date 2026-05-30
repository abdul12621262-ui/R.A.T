//! Session agent: pairing, capture loop, input, file/shell handlers.

pub mod fs_ops;
pub mod session;

pub use session::{AgentConfig, AgentEvent, AgentHandle, Role};
