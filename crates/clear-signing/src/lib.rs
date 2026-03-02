#![no_std]
extern crate alloc;

pub mod clear_call;
pub mod display;
pub mod fields;
pub mod format;
pub mod reference;
pub mod registry;
pub mod resolver;
pub mod sol;

pub type Result<T> = anyhow::Result<T>;

pub trait ResultExt<T, E> {
    fn err_ctx(self, msg: &str) -> Result<T>;
}

impl<T, E: core::fmt::Display> ResultExt<T, E> for core::result::Result<T, E> {
    fn err_ctx(self, msg: &str) -> Result<T> {
        self.map_err(|e| anyhow::anyhow!("{}: {}", msg, e))
    }
}
