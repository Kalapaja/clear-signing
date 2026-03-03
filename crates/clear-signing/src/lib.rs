#![no_std]
extern crate alloc;

mod clear_call;
mod display;
mod fields;
mod format;
mod reference;
mod registry;
mod resolver;
mod sol;

pub use clear_call::parse_clear_call;

// Re-export core types
pub use fields::{ClearCall, DisplayField, Direction, Label};
pub use display::{Display, Field, Entry, Labels};
pub use resolver::Message;

// Re-export traits
pub use registry::Registry;

// Re-export commonly used sol types
pub use sol::SolFunction;

pub type Result<T> = anyhow::Result<T>;

pub(crate) trait ResultExt<T, E> {
    fn err_ctx(self, msg: &str) -> Result<T>;
}

impl<T, E: core::fmt::Display> ResultExt<T, E> for core::result::Result<T, E> {
    fn err_ctx(self, msg: &str) -> Result<T> {
        self.map_err(|e| anyhow::anyhow!("{}: {}", msg, e))
    }
}
