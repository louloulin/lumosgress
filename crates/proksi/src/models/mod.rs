pub mod tenant;
pub mod message;

// Re-export message types for easy access
pub use message::{MsgProxy, MsgRoute}; 