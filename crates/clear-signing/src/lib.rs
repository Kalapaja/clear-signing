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

use alloc::vec::Vec;
pub use fields::{ClearCall, DisplayField, Direction, Label};
pub use display::{Display, Field, Entry, Labels};
pub use resolver::Message;
pub use registry::Registry;
pub use sol::SolFunction;
use crate::clear_call::parse_message;

pub type Result<T> = anyhow::Result<T>;

pub(crate) trait ResultExt<T, E> {
    fn err_ctx(self, msg: &str) -> Result<T>;
}

impl<T, E: core::fmt::Display> ResultExt<T, E> for core::result::Result<T, E> {
    fn err_ctx(self, msg: &str) -> Result<T> {
        self.map_err(|e| anyhow::anyhow!("{}: {}", msg, e))
    }
}

pub fn parse_clear_call(
    message: Message,
    displays: Vec<Display>,
    registry: &dyn Registry,
) -> Result<ClearCall> {
    parse_message(&displays, &message, registry, 0)
}
