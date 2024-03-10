use crate::{Filter, InputConstraints, InputValue};

// #[derive(Clone)]
// #[builder(setter(strip_option))]
pub struct Input<'a, 'b, T, FT> where T: InputValue {
    // pub name: Option<String>,
    // pub value: Option<T>,
    // pub default_value: Option<T>,
    // #[builder(default = "None")]
    pub constraints: Option<Box<dyn InputConstraints<'a, 'b, T, FT>>>,

    // #[builder(default = "None")]
    pub filters: Option<Vec<&'a Filter<Option<T>>>>,
}

impl<'a, 'b, T: InputValue, FT> Input<'a, 'b, T, FT> {
    pub fn new() -> Self {
        Input {
            // name: None,
            // value: None,
            constraints: None,
            // default_value: None,
            filters: None,
        }
    }

    /// Filters value against contained filters.
    ///
    /// ```rust
    /// use walrs_inputfilter::{
    ///   Input,
    ///   // InputBuilder,
    ///   // InputConstraints2,
    ///   ScalarConstraintsBuilder,
    /// };
    ///
    /// // Setup input constraints
    /// let mut usize_input = Input::<usize, usize>::new();
    ///
    ///   /*usize_input.constraints = Some(
    ///     Box::new(ScalarConstraintsBuilder::<usize>::default()
    ///       .min(0)
    ///       .max(10)
    ///       .build()
    ///       .unwrap())
    ///   );*/
    ///
    ///   usize_input.filters = Some(vec![&|x: Option<usize>| x.map(|_x| _x * 2usize)]);
    ///
    /// let test_cases = [
    ///   (&usize_input, None, None),
    ///   (&usize_input, Some(0), Some(0)),
    ///   (&usize_input, Some(2), Some(4)),
    ///   (&usize_input, Some(4), Some(8)),
    /// ];
    ///
    /// // Run test cases
    /// for (i, (input, value, expected_rslt)) in test_cases.into_iter().enumerate() {
    ///   println!("Case {}: `(usize_input.filter)({:?}) == {:?}`", i + 1, value.clone(), expected_rslt.clone());
    ///   assert_eq!(input.filter(value), expected_rslt);
    /// }
    /// ```
    ///
    pub fn filter(&self, value: Option<T>) -> Option<T> {
        match self.filters.as_deref() {
            None => value,
            Some(fs) => fs.iter().fold(value, |agg, f| f(agg)),
        }
    }
}
#[cfg(test)]
mod test {
    use std::borrow::Cow;
    use std::error::Error;
    use crate::{range_overflow_msg, ScalarConstraints, ScalarConstraintsBuilder, StringConstraintsBuilder};
    use crate::ViolationEnum::StepMismatch;
    use super::*;

    #[test]
    fn test_new() -> Result<(), Box<dyn Error>> {
        let _ = Input::<&str, Cow<str>>::new();
        let _ = Input::<char, char>::new();
        let _ = Input::<usize, usize>::new();
        let _ = Input::<bool, bool>::new();

        let mut float_percent = Input::<usize, usize>::new();
        float_percent.constraints = Some(Box::new(ScalarConstraintsBuilder::<usize>::default()
            .min(0)
            .max(100)
            .validators(vec![
                &|x| if x != 0 && x % 5 != 0 {
                    Err(vec![(StepMismatch, format!("{} is not divisible by 5", x))])
                } else {
                    Ok(())
                },
            ])
            .build()?
        ));

        assert_eq!(float_percent.constraints.as_deref().unwrap().validate(Some(5)), Ok(()));
        assert_eq!(float_percent.constraints.as_deref().unwrap().validate(Some(101)),
                   Err(vec![
                       // range_overflow_msg(
                       //     float_percent.constraints.as_deref().unwrap()
                       //         .downcast_ref::<ScalarConstraints<usize>>().unwrap(), 
                       //     101usize
                       // ), 
                       "`101` is greater than maximum `100`.".to_string(),
                       "101 is not divisible by 5".to_string()
                   ])
        );
        assert_eq!(float_percent.constraints.as_deref().unwrap().validate(Some(26)),
                   Err(vec!["26 is not divisible by 5".to_string()]));

        let mut str_input = Input::<&str, Cow<str>>::new();
        str_input.constraints = Some(Box::new(StringConstraintsBuilder::default()
            .max_length(4)
            .build()?
        ));

        assert_eq!(str_input.constraints.as_deref().unwrap()
                       .validate(Some("aeiou")),
                   Err(vec![
                       "Value length `5` is greater than allowed maximum `4`.".to_string(),
                   ])
        );

        Ok(())
    }
}
