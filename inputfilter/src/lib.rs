#![feature(fn_traits)]
#![feature(unboxed_closures)]

#[macro_use]
extern crate derive_builder;

pub mod types;
pub mod validator;
pub mod input;
pub mod filter;
pub mod string_input;

pub use types::*;
pub use validator::*;
pub use input::*;
pub use filter::*;
pub use string_input::*;

// @todo Add 'Builder' for `wal_inputfilter` structs.

