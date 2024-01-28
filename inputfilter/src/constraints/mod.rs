pub mod scalar_input;
pub mod string_input;
pub mod traits;

pub use scalar_input::*;
pub use string_input::*;
pub use traits::*;

pub fn value_missing_msg() -> String {
    "Value missing".to_string()
}
