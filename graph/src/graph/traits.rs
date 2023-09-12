use std::borrow::Cow;
use std::fmt::Debug;
use std::str::FromStr;

pub trait Symbol: Clone + Debug + PartialEq + FromStr + From<&'static str> {
  fn id(&self) -> Cow<str>;
}
