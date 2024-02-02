pub mod scalar;
pub mod string;
pub mod traits;

pub use scalar::*;
pub use string::*;
pub use traits::*;

pub fn value_missing_msg() -> String {
    "Value missing".to_string()
}
