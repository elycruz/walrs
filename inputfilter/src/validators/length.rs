use crate::{ValidateValue, ValidationResult, ViolationEnum, ViolationMessage};

pub type StrLenValidatorCallback<'a, 'b> = dyn Fn(&StrLenValidator<'a, 'b>, &'b str) -> ViolationMessage + Send + Sync;

#[derive(Builder, Clone)]
#[builder(pattern = "owned", setter(strip_option))]
pub struct StrLenValidator<'a, 'b> {
    #[builder(default = "false")]
    pub break_on_failure: bool,

    #[builder(default = "None")]
    pub min_length: Option<usize>,

    #[builder(default = "None")]
    pub max_length: Option<usize>,

    #[builder(default = "&str_len_too_short_msg")]
    pub too_short_msg: &'a StrLenValidatorCallback<'a, 'b>,

    #[builder(default = "&str_len_too_long_msg")]
    pub too_long_msg: &'a StrLenValidatorCallback<'a, 'b>,
}

impl <'a, 'b> StrLenValidator<'a, 'b> {
    pub fn new() -> Self {
        StrLenValidatorBuilder::default().build().unwrap()
    }
}

impl<'a, 'b> ValidateValue<&'b str> for StrLenValidator<'a, 'b> {
    fn validate(&self, value: &'b str) -> ValidationResult {
        let mut errs = vec![];

        if let Some(min_length) = self.min_length {
            if value.len() < min_length {
                errs.push((
                    ViolationEnum::TooShort,
                    (self.too_short_msg)(self, value),
                ));

                if self.break_on_failure { return Err(errs); }
            }
        }

        if let Some(max_length) = self.max_length {
            if value.len() > max_length {
                errs.push((
                    ViolationEnum::TooLong,
                    (self.too_long_msg)(self, value),
                ));

                if self.break_on_failure { return Err(errs); }
            }
        }

        if errs.is_empty() { Ok(()) } else { Err(errs) }
    }
}

impl<'a, 'b> FnMut<(&'b str, )> for StrLenValidator<'a, 'b> {
  extern "rust-call" fn call_mut(&mut self, args: (&'b str, )) -> Self::Output {
    self.validate(args.0)
  }
}

impl<'a, 'b> Fn<(&'b str, )> for StrLenValidator<'a, 'b> {
  extern "rust-call" fn call(&self, args: (&'b str, )) -> Self::Output {
    self.validate(args.0)
  }
}

impl<'a, 'b> FnOnce<(&'b str,)> for StrLenValidator<'a, 'b> {
  type Output = ValidationResult;

  extern "rust-call" fn call_once(self, args: (&'b str,)) -> Self::Output {
    self.validate(args.0)
  }
}

impl<'a, 'b> Default for StrLenValidator<'a, 'b> {
    fn default() -> Self {
        StrLenValidator::new()
    }
}

pub fn str_len_too_short_msg(rules: &StrLenValidator, xs: &str) -> String {
    format!(
        "Value length `{:}` is less than allowed minimum `{:}`.",
        xs,
        &rules.min_length.unwrap_or(0)
    )
}

pub fn str_len_too_long_msg(rules: &StrLenValidator, xs: &str) -> String {
    format!(
        "Value length `{:}` is greater than allowed maximum `{:}`.",
        xs,
        &rules.max_length.unwrap_or(0)
    )
}
