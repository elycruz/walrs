use crate::{Filter, InputConstraints, InputValue,
            Validator, ViolationEnum, ViolationMessage, ViolationTuple};

type ValueMissingCallback<T, FT> = dyn Fn(&Input<T, FT>) -> ViolationMessage + Send + Sync;

pub fn value_missing_msg_getter<T: InputValue, FT>(_: &Input<T, FT>) -> ViolationMessage {
    "Value is missing".to_string()
}

#[derive(Builder, Clone)]
#[builder(setter(strip_option))]
pub struct Input<'a, 'b, T, FT>
    where T: InputValue,
{
    #[builder(default = "None")]
    pub name: Option<&'a str>,

    #[builder(default = "None")]
    pub default_value: Option<T>,

    #[builder(default = "false")]
    pub break_on_failure: bool,

    #[builder(default = "None")]
    pub min: Option<T>,

    #[builder(default = "None")]
    pub max: Option<T>,

    #[builder(default = "false")]
    pub required: bool,

    #[builder(default = "None")]
    pub validators: Option<Vec<&'a Validator<T>>>,

    #[builder(default = "None")]
    pub filters: Option<Vec<&'a Filter<Option<FT>>>>,

    #[builder(default = "&range_underflow_msg_getter")]
    pub range_underflow_msg: &'a (dyn Fn(&Input<'a, 'b, T, FT>, T) -> String + Send + Sync),

    #[builder(default = "&range_overflow_msg_getter")]
    pub range_overflow_msg: &'a (dyn Fn(&Input<'a, 'b, T, FT>, T) -> String + Send + Sync),

    #[builder(default = "&value_missing_msg_getter")]
    pub value_missing_msg: &'a (dyn Fn(&Input<'a, 'b, T, FT>) -> ViolationMessage + Send + Sync)
}

impl<'a, 'b, T: InputValue, FT> Input<'a, 'b, T, FT> {
    pub fn new() -> Self {
        Input {
            name: None,
            default_value: None,
            break_on_failure: false,
            min: None,
            max: None,
            required: false,
            validators: None,
            filters: None,
            range_underflow_msg: &(range_underflow_msg_getter),
            range_overflow_msg: &(range_overflow_msg_getter),
            value_missing_msg: &value_missing_msg_getter,
        }
    }

    fn validate(&self, value: Option<T>) -> Result<(), Vec<ViolationMessage>> {
        match self.validate_detailed(value) {
            // If errors, extract messages and return them
            Err(messages) => Err(messages.into_iter()
                .map(|(_, message)| message).collect()),
            Ok(_) => Ok(()),
        }
    }

    fn validate_detailed(&self, value: Option<T>) -> Result<(), Vec<ViolationTuple>> {
        match value {
            None => if self.required {
                Err(vec![(
                    ViolationEnum::ValueMissing,
                    (self.value_missing_msg)(self),
                )])
            } else {
                Ok(())
            },
            // Else if value is populated validate it
            Some(v) =>  self._validate_against_validators(v)
        }
    }

    /// Filters value against contained filters.
    pub fn filter(&self, value: Option<FT>) -> Option<FT> {
        match self.filters.as_deref() {
            None => value,
            Some(fs) => fs.iter().fold(value, |agg, f| f(agg)),
        }
    }

    fn _validate_against_validators(&self, value: T) -> Result<(), Vec<ViolationTuple>> {
        self
            .validators
            .as_deref()
            .map(|vs|
                // If not break on failure then capture all validation errors.
                if !self.break_on_failure {
                    return vs
                        .iter()
                        .fold(Vec::<ViolationTuple>::new(), |mut agg, f| {
                            match f(value) {
                                Err(mut message_tuples) => {
                                    agg.append(message_tuples.as_mut());
                                    agg
                                }
                                _ => agg,
                            }
                        });
                } else {
                    // Else break on, and capture, first failure.
                    // ----
                    let mut agg = Vec::<ViolationTuple>::new();
                    for f in vs.iter() {
                        if let Err(mut message_tuples) = f(value) {
                            agg.append(message_tuples.as_mut());
                            break;
                        }
                    }
                    agg
                })
            .and_then(|messages| {
                if messages.is_empty() {
                    None
                } else {
                    Some(messages)
                }
            })
            .map_or(Ok(()), Err)
    }
}

/// Returns generic range underflow message.
///
/// ```rust
/// use walrs_inputfilter::{InputBuilder, range_underflow_msg_getter};
///
/// let input = InputBuilder::<usize, usize>::default()
///   .min(1)
///   .build()
///   .unwrap();
///
/// assert_eq!(range_underflow_msg_getter(&input, 0), "`0` is less than minimum `1`.");
/// ```
pub fn range_underflow_msg_getter<T: InputValue, FT>(rules: &Input<T, FT>, x: T) -> String {
    format!(
        "`{:}` is less than minimum `{:}`.",
        x,
        &rules.min.unwrap()
    )
}

/// Returns generic range overflow message.
///
/// ```rust
/// use walrs_inputfilter::{InputBuilder, range_overflow_msg_getter};
///
/// let input = InputBuilder::<usize, usize>::default()
///   .max(10)
///   .build()
///   .unwrap();
///
/// assert_eq!(range_overflow_msg_getter(&input, 100), "`100` is greater than maximum `10`.");
/// ```
pub fn range_overflow_msg_getter<T: InputValue, FT>(rules: &Input<T, FT>, x: T) -> String {
    format!(
        "`{:}` is greater than maximum `{:}`.",
        x,
        &rules.max.unwrap()
    )
}

#[cfg(test)]
mod test {
    use std::borrow::Cow;
    use std::error::Error;
    // use crate::{InputBuilder, StringConstraintsBuilder};
    // use crate::ViolationEnum::StepMismatch;
    use super::*;

    #[test]
    fn test_new() -> Result<(), Box<dyn Error>> {
        let _ = Input::<&str, Cow<str>>::new();
        let _ = Input::<char, char>::new();
        let _ = Input::<usize, usize>::new();
        let _ = Input::<bool, bool>::new();

        let _ = Input::<usize, usize>::new();
        // float_percent.constraints = Some(Box::new(InputBuilder::<usize>::default()
        //     .min(0)
        //     .max(100)
        //     .validators(vec![
        //         &|x| if x != 0 && x % 5 != 0 {
        //             Err(vec![(StepMismatch, format!("{} is not divisible by 5", x))])
        //         } else {
        //             Ok(())
        //         },
        //     ])
        //     .build()?
        // ));
        //
        // assert_eq!(float_percent.validate(Some(5)), Ok(()));
        // assert_eq!(float_percent.validate(Some(101)),
        //            Err(vec![
        //                // range_overflow_msg(
        //                //     float_percent.constraints.as_deref().unwrap()
        //                //         .downcast_ref::<Input<usize>>().unwrap(),
        //                //     101usize
        //                // ),
        //                "`101` is greater than maximum `100`.".to_string(),
        //                "101 is not divisible by 5".to_string(),
        //            ]));
        // assert_eq!(float_percent.validate(Some(26)),
        //            Err(vec!["26 is not divisible by 5".to_string()]));
        //
        let _ = Input::<&str, Cow<str>>::new();
        // str_input.constraints = Some(Box::new(StringConstraintsBuilder::default()
        //     .max_length(4)
        //     .build()?
        // ));
        //
        // assert_eq!(str_input.validate(Some("aeiou")),
        //            Err(vec![
        //                "Value length `5` is greater than allowed maximum `4`.".to_string(),
        //            ]));

        Ok(())
    }
}
