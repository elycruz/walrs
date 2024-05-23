#![allow(unused)]

use std::borrow::Cow;
use std::fmt::Display;
use std::collections::HashMap;
use serde::de::Error;
use walrs_inputfilter::{Input, InputBuilder, InputConstraints, InputValue, ValidationResult, Validator, ViolationMessage, ViolationTuple};
use walrs_inputfilter::ViolationEnum::{RangeOverflow, RangeUnderflow};

/// Here we're setting up an application context example, for testing walrs_inputfilter types

type ModelValidationResult = Result<(), HashMap<&'static str, Vec<ViolationMessage>>>;

// Fixes error in issue #56105 (see the issue on rust github for more)
trait Shush56105<T: ?Sized>: Fn(&T) -> ValidationResult {}
impl<T: ?Sized, F: ?Sized> Shush56105<T> for F
    where
        F: Fn(&T) -> ValidationResult
{}

type ValidatorRef<T> = dyn Shush56105<T, Output=ValidationResult>;

/// Faux data model, for validation tests
struct SomeModel {
    name: String,
    id: usize,
}

/// Filter rules
struct InputRules<'a, 'b> {
    name: Input<'a, &'b str, Cow<'b, str>>,
    id: Input<'a, usize, usize>
}

impl SomeModel {
    fn validate(&self, rules: &InputRules) -> ModelValidationResult {
        // Validate values
        // ----
        let mut msgs: HashMap<&str, Vec<String>> = HashMap::new();

        if let Err(err_msgs) = rules.name.validate(Some(self.name.as_str())) {
            msgs.insert("name", err_msgs);
        }

        if let Err (err_msgs) = rules.id.validate(Some(self.id)) {
            msgs.insert("id", err_msgs);
        }

        if !msgs.is_empty() {
            Err(msgs)
        } else {
            Ok(())
        }
    }
}

fn app_runtime_ctx(rules: &InputRules, data: String) {
    let model = SomeModel { name: data, id: 0 };
    let _ = model.validate(&rules);
}

#[test]
fn test_type_constructors() -> Result<(), Box<dyn Error>> {
    let name: Input<&str, Cow<str>> = Input::new();
    let id: Input<usize, usize> = InputBuilder::<usize, usize>::default()
        .validators(vec![
            &|n| if n == 0 { Err(vec![(RangeUnderflow, "range-underflow error".to_string())]) }
            else if n > 10 { Err(vec![(RangeOverflow, "range-overflow error".to_string())]) }
            else { Ok(()) }
        ])
        .build()?;

    let rules = InputRules {
        name,
        id
    };

    app_runtime_ctx(&rules, "Hello".to_string());

    Ok(())
}