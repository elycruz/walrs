use std::fmt::Debug;
use std::str::FromStr;

pub trait Symbol: Clone + Debug + PartialEq + FromStr + From<String> {
  // @todo fn name should be a less generic name;  E.g., `get_symbol_id`, etc.
  fn id(&self) -> String;
}
