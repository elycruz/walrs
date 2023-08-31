#![feature(fn_traits)]
#![feature(unboxed_closures)]

#[macro_use]
extern crate derive_builder;

pub mod types;
pub mod validator;
pub mod input;

pub use types::*;
pub use validator::*;
pub use input::*;

// @todo Add 'Builder' for `wal_inputfilter` structs.

