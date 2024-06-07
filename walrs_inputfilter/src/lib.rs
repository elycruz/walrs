#![feature(fn_traits)]
#![feature(unboxed_closures)]
#![feature(associated_type_defaults)]

#[macro_use]
extern crate derive_builder;

pub mod traits;
pub mod validators;
pub mod filters;
pub mod input;
pub mod scalar;

pub use traits::*;
pub use validators::*;
pub use filters::*;
pub use input::*;
pub use scalar::*;