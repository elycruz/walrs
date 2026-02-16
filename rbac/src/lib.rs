#![cfg_attr(not(feature = "std"), no_std)]
#![allow(unused_imports)]

#[cfg(not(feature = "std"))]
extern crate alloc;

extern crate core;

// Re-export common types for internal use
mod prelude {
  #[cfg(feature = "std")]
  pub use std::string::String;
  #[cfg(feature = "std")]
  pub use std::vec::Vec;
  #[cfg(feature = "std")]
  pub use std::format;
  #[cfg(feature = "std")]
  pub use std::vec;
  #[cfg(feature = "std")]
  pub use std::string::ToString;

  #[cfg(not(feature = "std"))]
  pub use alloc::string::String;
  #[cfg(not(feature = "std"))]
  pub use alloc::vec::Vec;
  #[cfg(not(feature = "std"))]
  pub use alloc::format;
  #[cfg(not(feature = "std"))]
  pub use alloc::vec;
  #[cfg(not(feature = "std"))]
  pub use alloc::string::ToString;
}

pub mod rbac;
pub mod role;
pub mod rbac_builder;
pub mod rbac_data;
pub mod error;

#[cfg(feature = "wasm")]
pub mod wasm;

#[cfg(feature = "wasm")]
pub use wasm::*;

pub use rbac::Rbac;
pub use role::Role;
pub use rbac_builder::RbacBuilder;
pub use rbac_data::RbacData;
pub use error::{RbacError, Result};
