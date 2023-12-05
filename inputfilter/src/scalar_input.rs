use std::borrow::Cow;
use std::fmt::{Debug, Display, Formatter};

use crate::types::{Filter, InputConstraints, Validator, ViolationMessage};
use crate::{ConstraintViolation, ScalarValue, ValidationErrTuple, value_missing_msg, ValueMissingCallback, WithName};

pub type NumMissingViolationCallback<T> = dyn Fn(&NumberInput<T>, Option<T>) -> ViolationMessage + Send + Sync;

pub fn range_underflow_msg<T: ScalarValue>(rules: &NumberInput<T>, x: Option<T>) -> String {
    format!(
        "`{:}` is less than minimum `{:}`.",
        x.unwrap(),
        &rules.min.unwrap()
    )
}

pub fn range_overflow_msg<T: ScalarValue>(rules: &NumberInput<T>, x: Option<T>) -> String {
    format!(
        "`{:}` is greater than maximum `{:}`.",
        x.unwrap(),
        &rules.max.unwrap()
    )
}

pub fn num_not_equal_msg<T: ScalarValue>(
    rules: &NumberInput<T>,
    x: Option<T>,
) -> String {
    format!(
        "`{:}` is not equal to `{:}`.",
        x.unwrap(),
        &rules.equal.unwrap()
    )
}

pub fn num_missing_msg<T: ScalarValue>(_: &NumberInput<T>, _: Option<T>) -> String {
    "Value is missing.".to_string()
}

#[derive(Builder, Clone)]
#[builder(setter(strip_option))]
pub struct NumberInput<'a, T: ScalarValue> {
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
    pub equal: Option<T>,

    #[builder(default = "false")]
    pub required: bool,

    #[builder(default = "None")]
    pub default_value: Option<T>,

    #[builder(default = "None")]
    pub validators: Option<Vec<&'a Validator<T>>>,

    #[builder(default = "None")]
    pub filters: Option<Vec<&'a Filter<Option<T>>>>,

    #[builder(default = "&range_underflow_msg")]
    pub range_underflow: &'a (dyn Fn(&NumberInput<'a, T>, Option<T>) -> String + Send + Sync),

    #[builder(default = "&range_overflow_msg")]
    pub range_overflow: &'a (dyn Fn(&NumberInput<'a, T>, Option<T>) -> String + Send + Sync),

    #[builder(default = "&num_not_equal_msg")]
    pub not_equal: &'a (dyn Fn(&NumberInput<'a, T>, Option<T>) -> String + Send + Sync),

    #[builder(default = "&value_missing_msg")]
    pub value_missing: &'a ValueMissingCallback,
}

impl<'a, T> NumberInput<'a, T>
    where T: ScalarValue
{
    pub fn new(name: Option<&'a str>) -> Self {
        NumberInput {
            break_on_failure: false,
            name,
            min: None,
            max: None,
            equal: None,
            required: false,
            default_value: None,
            validators: None,
            filters: None,
            range_underflow: &(range_underflow_msg),
            range_overflow: &(range_overflow_msg),
            not_equal: &(num_not_equal_msg),
            value_missing: &value_missing_msg,
        }
    }

    fn _validate_against_self(&self, value: T) -> Result<(), Vec<ValidationErrTuple>> {
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

        if errs.is_empty() { Ok(()) } else { Err(errs) }
    }

    fn _validate_against_validators(&self, value: T, validators: Option<&[&'a Validator<T>]>) -> Result<(), Vec<ValidationErrTuple>> {
        validators.map(|vs| {

            // If not break on failure then capture all validation errors.
            if !self.break_on_failure {
                return vs.iter().fold(
                    Vec::<ValidationErrTuple>::new(),
                    |mut agg, f| match (f)(value) {
                        Err(mut message_tuples) => {
                            agg.append(message_tuples.as_mut());
                            agg
                        }
                        _ => agg,
                    });
            }

            // Else break on, and capture, first failure.
            let mut agg = Vec::<ValidationErrTuple>::new();
            for f in vs.iter() {
                if let Err(mut message_tuples) = (f)(value) {
                    agg.append(message_tuples.as_mut());
                    break;
                }
            }
            agg
        })
            .and_then(|messages| if messages.is_empty() { None } else { Some(messages) })
            .map_or(Ok(()), Err)
    }
}

impl<'a, 'b, T: 'b> InputConstraints<'a, 'b, T, T> for NumberInput<'a, T>
    where T: ScalarValue + Copy {
    fn validate(&self, value: Option<T>) ->  Result<(), Vec<ValidationErrTuple>> {
        match value {
            None => {
                if self.required {
                    Err(vec![(
                        ConstraintViolation::ValueMissing,
                        (&self.value_missing)(self),
                    )])
                } else {
                    Ok(())
                }
            }
            // Else if value is populated validate it
            Some(v) => match self._validate_against_self(v) {
                Ok(_) => self._validate_against_validators(v, self.validators.as_deref()),
                Err(messages1) => if self.break_on_failure {
                    Err(messages1)
                } else {
                    match self._validate_against_validators(v, self.validators.as_deref()) {
                        Ok(_) => Ok(()),
                        Err(mut messages2) => {
                            let mut agg = messages1;
                            agg.append(messages2.as_mut());
                            Err(agg)
                        }
                    }
                }
            },
        }
    }

    fn validate1(&self, value: Option<T>) -> Result<(), Vec<ViolationMessage>> {
        match self.validate(value) {
            // If errors, extract messages and return them
            Err(messages) =>
                Err(messages.into_iter().map(|(_, message)| message).collect()),
            Ok(_) => Ok(()),
        }
    }

    fn filter(&self, value: Option<T>) -> Option<T> {
        let v = match value {
            None => self.default_value.map(|x| x.into()),
            Some(x) => Some(x)
        };

        match self.filters.as_deref() {
            None => v,
            Some(fs) => fs.iter().fold(v, |agg, f| f(agg)),
        }
    }

    // @todo consolidate these (`validate_and_filter*`), into just `filter*` (
    //      since we really don't want to use filtered values without them being valid/etc.)
    fn validate_and_filter(&self, x: Option<T>) -> Result<Option<T>, Vec<ValidationErrTuple>> {
        self.validate(x).map(|_| self.filter(x))
    }

    fn validate_and_filter1(&self, x: Option<T>) -> Result<Option<T>, Vec<ViolationMessage>> {
        match self.validate_and_filter(x) {
            Err(messages) =>
                Err(messages.into_iter().map(|(_, message)| message).collect()),
            Ok(filtered) => Ok(filtered),
        }
    }
}

impl<'a, T: ScalarValue> WithName<'a> for NumberInput<'a, T> {
    fn get_name(&self) -> Option<Cow<'a, str>> {
        self.name.map(|xs| Cow::Borrowed(xs))
    }
}

impl<T: ScalarValue> Default for NumberInput<'_, T> {
    fn default() -> Self {
        Self::new(None)
    }
}

impl<T: ScalarValue> Display for NumberInput<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ScalarInput {{ name: {}, required: {}, validators: {}, filters: {} }}",
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

impl<T: ScalarValue> Debug for NumberInput<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self)
    }
}

#[cfg(test)]
mod test {}
