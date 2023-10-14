use std::fmt::Debug;
use std::str::FromStr;

pub trait Symbol: Clone + Debug + PartialEq + FromStr + From<String> {
  fn id(&self) -> String;
}
