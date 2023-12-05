use std::borrow::Cow;
use std::fmt::{Debug, Display, Formatter};

use crate::types::{Filter, InputConstraints, Validator, ViolationMessage};
use crate::{ConstraintViolation, NumberValue, ValidationErrTuple};

pub type NumMissingViolationCallback<T: NumberValue> = dyn Fn(&NumberInput<T>, Option<T>) -> ViolationMessage + Send + Sync;

pub fn range_underflow_msg<T: NumberValue>(rules: &NumberInput<T>, x: Option<T>) -> String {
    format!(
        "`{:}` is less than minimum `{:}`.",
        x.unwrap(),
        &rules.min.unwrap()
    )
}

pub fn range_overflow_msg<T: NumberValue>(rules: &NumberInput<T>, x: Option<T>) -> String {
    format!(
        "`{:}` is greater than maximum `{:}`.",
        x.unwrap(),
        &rules.max.unwrap()
    )
}

pub fn step_mismatch_msg<T: NumberValue>(
    rules: &NumberInput<T>,
    x: Option<T>,
) -> String {
    format!(
        "`{:}` is not divisible by `{:}`.",
        x.unwrap(),
        &rules.step.unwrap()
    )
}

pub fn num_not_equal_msg<T: NumberValue>(
    rules: &NumberInput<T>,
    x: Option<T>,
) -> String {
    format!(
        "`{:}` is not equal to `{:}`.",
        x.unwrap(),
        &rules.equal.unwrap()
    )
}

pub fn num_missing_msg<T: NumberValue>(_: &NumberInput<T>, _: Option<T>) -> String {
    "Value is missing.".to_string()
}

#[derive(Builder, Clone)]
#[builder(setter(strip_option))]
pub struct NumberInput<'a, T: NumberValue> {
    #[builder(default = "true")]
    pub break_on_failure: bool,

    /// @todo This should be an `Option<Cow<'a, str>>`, for compatibility.
    #[builder(setter(into), default = "None")]
    pub name: Option<&'a str>,

    #[builder(default = "None")]
    pub min: Option<T>,

    #[builder(default = "None")]
    pub max: Option<T>,

    #[builder(default = "None")]
    pub step: Option<T>,

    #[builder(default = "None")]
    pub equal: Option<T>,

    #[builder(default = "false")]
    pub required: bool,

    #[builder(default = "None")]
    pub validators: Option<Vec<&'a Validator<T>>>,

    // @todo Add support for `io_validators` (e.g., validators that return futures).

    #[builder(default = "None")]
    pub filters: Option<Vec<&'a Filter<T>>>,

    #[builder(default = "&range_underflow_msg")]
    pub range_underflow: &'a (dyn Fn(&NumberInput<'a, T>, Option<T>) -> String + Send + Sync),

    #[builder(default = "&range_overflow_msg")]
    pub range_overflow: &'a (dyn Fn(&NumberInput<'a, T>, Option<T>) -> String + Send + Sync),

    #[builder(default = "&step_mismatch_msg")]
    pub step_mismatch: &'a (dyn Fn(&NumberInput<'a, T>, Option<T>) -> String + Send + Sync),

    #[builder(default = "&num_not_equal_msg")]
    pub not_equal: &'a (dyn Fn(&NumberInput<'a, T>, Option<T>) -> String + Send + Sync),

    #[builder(default = "&num_missing_msg")]
    pub value_missing: &'a (dyn Fn(&NumberInput<'a, T>, Option<T>) -> String + Send + Sync),
}

impl<'a, T> NumberInput<'a, T>
    where T: NumberValue
{
    pub fn new(name: Option<&'a str>) -> Self {
        NumberInput {
            break_on_failure: false,
            name,
            min: None,
            max: None,
            equal: None,
            required: false,
            validators: None,
            filters: None,
            range_underflow: &(range_underflow_msg),
            range_overflow: &(range_overflow_msg),
            not_equal: &(num_not_equal_msg),
            value_missing: &num_missing_msg,
            step: None,
            step_mismatch: &(step_mismatch_msg),
        }
    }
    fn validate_custom(&self, value: T) -> Result<(), Vec<ValidationErrTuple>> {
        let mut errs = vec![];

        // Test lower bound
        if let Some(min) = self.min {
            if value < min {
                errs.push((
                    ConstraintViolation::RangeUnderflow,
                    (&self.range_underflow)(self, Some(value)),
                ));

                if self.break_on_failure { return Err(errs); }
            }
        }

        // Test upper bound
        if let Some(max) = self.max {
            if value > max {
                errs.push((
                    ConstraintViolation::TooLong,
                    (&self.range_overflow)(self, Some(value)),
                ));

                if self.break_on_failure { return Err(errs); }
            }
        }

        // Test equality
        if let Some(equal) = self.equal {
            if value != equal {
                errs.push((
                    ConstraintViolation::NotEqual,
                    (&self.not_equal)(self, Some(value)),
                ));

                if self.break_on_failure { return Err(errs); }
            }
        }

        // Test Step
        if let Some(step) = self.step {
            if step != Default::default() && value % step != Default::default() {
                errs.push((
                    ConstraintViolation::StepMismatch,
                    (&self.step_mismatch)(self, Some(value))
                ));

                if self.break_on_failure { return Err(errs); }
            }
        }

        if errs.is_empty() { Ok(()) } else { Err(errs) }
    }

}

impl<'a, 'b, T: 'b> InputConstraints<'a, 'b, T, T> for NumberInput<'a, T>
    where T: NumberValue
{
    fn validate(&self, value: Option<T>) -> Result<(), Vec<ValidationErrTuple>> {
        todo!()
    }

    fn validate1(&self, value: Option<T>) -> Result<(), Vec<ViolationMessage>> {
        todo!()
    }

    fn filter(&self, value: T) -> T {
        todo!()
    }

    fn validate_and_filter(&self, x: Option<T>) -> Result<Option<T>, Vec<ValidationErrTuple>> {
        self.validate(x).map(|_| self.filter(x))
    }

    fn validate_and_filter1(&self, x: Option<T>) -> Result<Option<T>, Vec<ViolationMessage>> {
        todo!()
    }
}

impl<T: NumberValue> Default for NumberInput<'_, T> {
    fn default() -> Self {
        Self::new(None)
    }
}

impl<T: NumberValue> Display for NumberInput<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "StrInput {{ name: {}, required: {}, validators: {}, filters: {} }}",
            self.name.unwrap_or("None"),
            self.required,
            self
                .validators
                .as_deref()
                .map(|vs| format!("Some([Validator; {}])", vs.len()))
                .unwrap_or("None".to_string()),
            self
                .filters
                .as_deref()
                .map(|fs| format!("Some([Filter; {}])", fs.len()))
                .unwrap_or("None".to_string()),
        )
    }
}

impl<T: NumberValue> Debug for NumberInput<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self)
    }
}

#[cfg(test)]
mod test {}
