#![feature(fn_traits)]
#![feature(unboxed_closures)]
#![feature(associated_type_defaults)]

#[macro_use]
extern crate derive_builder;

pub mod traits;
pub mod validator;
pub mod filter;
pub mod constraints;

pub use traits::*;
pub use validator::*;
pub use filter::*;
pub use constraints::*;
