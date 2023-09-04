use std::borrow::Cow;
use ammonia;

pub struct StripTags<'a> {
  pub ammonia: Option<ammonia::Builder<'a>>,
}

impl<'a> StripTags<'a> {
  pub fn new() -> Self {
    Self {
      ammonia: None,
    }
  }

  pub fn filter<'b>(&self, input: Cow<'b, str>) -> Cow<'b, str> {
    match self.ammonia {
      None => Cow::Owned(
        ammonia::Builder::default().clean(&input).to_string()
      ),
      Some(ref sanitizer) => Cow::Owned(
        sanitizer.clean(&input).to_string()
      ),
    }
  }
}

impl<'a, 'b> FnOnce<(Cow<'b, str>, )> for StripTags<'a> {
  type Output = Cow<'b, str>;

  extern "rust-call" fn call_once(self, args: (Cow<'b, str>, )) -> Self::Output {
    self.filter(args.0)
  }
}

impl<'a, 'b> FnMut<(Cow<'b, str>, )> for StripTags<'a> {
  extern "rust-call" fn call_mut(&mut self, args: (Cow<'b, str>, )) -> Self::Output {
    self.filter(args.0)
  }
}

impl<'a, 'b> Fn<(Cow<'b, str>, )> for StripTags<'a> {
  extern "rust-call" fn call(&self, args: (Cow<'b, str>, )) -> Self::Output {
    self.filter(args.0)
  }
}
