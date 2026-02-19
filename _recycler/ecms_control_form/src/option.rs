use crate::traits::FormControlValue;
use serde::{Deserialize, Serialize};

const OPTION_INVALID_MSG: &str =
  "Expected either `label`, or `value` to be set;  Found 'only' `None`";

#[derive(Serialize, Deserialize, Debug, Builder, Clone)]
#[builder(build_fn(validate = "Self::validate"), setter(into, strip_option))]
pub struct OptionControl<Value>
where
  Value: FormControlValue,
{
  #[builder(setter(custom, strip_option), default = "None")]
  pub value: Option<Value>,

  #[builder(setter(custom, strip_option), default = "None")]
  pub label: Option<Value>,

  #[builder(default = "false")]
  pub disabled: bool,

  #[builder(default = "false")]
  pub selected: bool,

  #[builder(setter(into), default = "None")]
  pub options: Option<Vec<Box<OptionControl<Value>>>>,
}

impl<Value> OptionControlBuilder<Value>
where
  Value: FormControlValue,
{
  /// Sets `label`, and `value`, if it's not set.
  pub fn label(&mut self, label: Value) -> &mut Self {
    if self.value.is_none() {
      self.value = Some(Some(label.clone()));
    }
    self.label = Some(Some(label));
    self
  }

  /// Sets `value`, and `label`, if it's not set.
  pub fn value(&mut self, value: Value) -> &mut Self {
    if self.label.is_none() {
      self.label = Some(Some(value.clone()));
    }
    self.value = Some(Some(value));
    self
  }

  /// Ensures at least one of, `value`, and/or `label` is set (e.g., not a `None`).
  fn validate(&self) -> Result<(), String> {
    let is_option_ctrl = self.options.is_none();
    if (is_option_ctrl && self.value.is_none() && self.label.is_none())
      || (!is_option_ctrl && self.label.is_none())
    {
      Err(OPTION_INVALID_MSG.to_string())
    } else {
      Ok(())
    }
  }
}

impl<Value> OptionControl<Value>
where
  Value: FormControlValue,
{
  pub fn new() -> Self {
    OptionControl {
      value: None,
      label: None,
      disabled: false,
      selected: false,
      options: None,
    }
  }

  /// Sets value, and label (if its not set and incoming value is `Some), to incoming value.
  pub fn set_value(&mut self, value: Option<Value>) {
    if self.label.is_none() && value.is_some() {
      self.label = value.clone();
    }
    self.value = value;
  }

  /// Sets label, and value (if it's not set and incoming value is `Some), to incoming value.
  pub fn set_label(&mut self, label: Option<Value>) {
    if self.value.is_none() && label.is_some() {
      self.value = label.clone();
    }
    self.label = label;
  }

  /// Sets `disabled` state on control, additionally, if control contains child `options`,
  /// `set_disabled()` gets called on each html option ctrl in child options (recursively).
  pub fn set_disabled(&mut self, disabled: bool) {
    self.disabled = disabled;
    if let Some(ops) = self.options.as_deref_mut() {
      ops.iter_mut().for_each(|o| {
        o.set_disabled(disabled);
      });
    }
  }

  /// Returns a reference to control's `value`, and or `label` (if `value` is not set), property.
  pub fn get_value(&self) -> Option<&Value> {
    self.value.as_ref().map_or(self.label.as_ref(), |v| Some(v))
  }
}

/**
 * Convenience methods
 * ----------------------- */

/// Static method for marking options in a slice as "not selected".
pub fn deselect_all<Value: FormControlValue>(options: &mut [OptionControl<Value>]) {
  options.iter_mut().for_each(|o| {
    o.selected = false;
  });
}

/// Selects the matching options and returns a boolean indicating
/// whether matching options were found or not;  Deselects non-matching options.
pub fn select_matching<Value: FormControlValue>(
  options: &mut [OptionControl<Value>],
  value: Option<&Value>,
) -> bool {
  if value.is_none() {
    deselect_all(options);
    return false;
  }
  value.map_or(false, |v| {
    options.iter_mut().fold(false, |match_found, o| {
      let _matching_op_found = o.value.as_ref().map_or(false, |o_v| o_v == v);
      o.selected = _matching_op_found;
      if !match_found && _matching_op_found {
        true
      } else {
        match_found
      }
    })
  })
}

pub fn select_matching_multi<'a, Value: FormControlValue>(
  options: &mut [OptionControl<Value>],
  values: &[&Value],
) -> bool {
  if values.len() == 0 || options.len() == 0 {
    return false;
  }
  values.iter().fold(false, |match_found_1, v| {
    options.iter_mut().fold(match_found_1, |match_found, o| {
      let _matching_op_found = o.value.as_ref().map_or(false, |o_v| o_v == *v);
      o.selected = _matching_op_found;
      if !match_found && _matching_op_found {
        true
      } else {
        match_found
      }
    })
  })
}

// @todo Should support nested descendant options search.
pub fn is_value_in_options<Value: FormControlValue>(
  options: Option<&[Box<OptionControl<Value>>]>,
  v: Option<&Value>,
) -> bool {
  options.map_or(false, |ops| {
    if ops.len() == 0 {
      false
    } else {
      ops.iter().any(|o| {
        return o.value.as_ref().zip(v).map_or(false, |(a, b)| a == b)
          || is_value_in_options(o.options.as_deref(), v);
      })
    }
  })
}

#[cfg(test)]
pub mod html_option_ctrl_tests {
  use crate::constants::{DISABLED, SELECTED};
  use crate::option::{
    is_value_in_options, OptionControl, OptionControlBuilder, OPTION_INVALID_MSG,
  };
  use crate::traits::FormControlValue;
  use std::error::Error;

  const VOWELS: &str = "aeiou";
  const VOWELS_LEN: usize = VOWELS.len();

  pub fn triangular_num(n: usize) -> usize {
    n * (n + 1) / 2
  }

  // Assigns each 'char', as a `String` to generated options' values
  pub fn options_from_str_provider<'a>(value_seed: &'a str) -> Vec<Box<OptionControl<&'a str>>> {
    let mut ops: Vec<Box<OptionControl<&'a str>>> = vec![];

    for (i, _) in value_seed.char_indices() {
      ops.push(Box::new(
        OptionControlBuilder::default()
          .value(&value_seed[i..i + 1])
          .label(&value_seed[i..i + 1])
          .build()
          .unwrap(),
      ));
    }
    ops
  }

  pub fn assert_default_option_ctrl<T>(ctrl: &OptionControl<T>)
  where
    T: FormControlValue,
  {
    assert_eq!(ctrl.value, None);
    assert_eq!(ctrl.label, None);
    assert_eq!(ctrl.disabled, false);
    assert_eq!(ctrl.selected, false);
    assert!(ctrl.options.is_none());
  }

  /// Checks if `optgroup` (`OptionCtrl` containing `options`), and all it's child options (checked recursively) are "disabled".
  pub fn optgroup_is_disabled<T>(optgroup: &OptionControl<T>, disabled: bool) -> bool
  where
    T: FormControlValue,
  {
    optgroup.disabled == disabled
      && optgroup.options.as_deref().map_or(disabled, |os| {
        os.iter().all(|o| optgroup_is_disabled(o, disabled))
      })
  }

  #[test]
  fn test_option_ctrl_new() {
    let ctrl: OptionControl<usize> = OptionControl::new();
    assert_default_option_ctrl(&ctrl);
  }

  #[test]
  fn test_option_ctrl_builder() -> Result<(), Box<dyn Error>> {
    // Test invalid option setting
    // ----
    println!("(0) With no Value, and/or, Label");
    let invalid_option: Result<OptionControl<usize>, String> =
      OptionControlBuilder::default().build();

    match invalid_option {
      Ok(option) => panic!(
        "Builder should return an `Err(String)` when both \
        `value` and `label` are `None`;  Received {:?}",
        option
      ),
      Err(msg) => assert_eq!(msg, OPTION_INVALID_MSG.to_string()),
    }

    // Test other structure cases
    // ----
    let test_cases: Vec<(
      &str,
      OptionControl<usize>,
      (Option<usize>, Option<usize>, bool, bool),
    )> = vec![
      (
        "With Value and Label (1)",
        OptionControlBuilder::default()
          .value(99) // Should populate `label` as well
          .build()?,
        (Some(99), Some(99), !DISABLED, !SELECTED),
      ),
      (
        "With Value and Label (2)",
        OptionControlBuilder::default()
          .label(99) // Should populate `value`, as well
          .build()?,
        (Some(99), Some(99), !DISABLED, !SELECTED),
      ),
      (
        "With Value, Label, and selected",
        OptionControlBuilder::default()
          .value(99) // Should populate `value`, as well
          .selected(true)
          .build()?,
        (Some(99), Some(99), !DISABLED, SELECTED),
      ),
    ];

    for (i, (test_name, option, (value, label, disabled, selected))) in
      test_cases.iter().enumerate()
    {
      println!("({}) {} Test", i + 1, test_name);

      assert_eq!(option.value, *value, "invalid value");
      assert_eq!(option.label, *label, "invalid label");
      assert_eq!(option.disabled, *disabled, "invalid disabled");
      assert_eq!(option.selected, *selected, "invalid selected");
    }

    println!("Test invalid builder configuration");
    // Should return `Err(String)`
    match OptionControlBuilder::default().build() as Result<OptionControl<&str>, String> {
      Ok(rslt) => panic!(
        "Expected builder 'build()' call to produce an `Err(String)`;  \
        Received {:?}",
        rslt
      ),
      Err(_) => (),
    }

    println!("Test invalid builder configuration, for 'optgroup' variant");
    // Should return `Err(String)`
    match OptionControlBuilder::default()
      .options(options_from_str_provider(VOWELS))
      .build() as Result<OptionControl<&str>, String>
    {
      Ok(rslt) => panic!(
        "Expected builder 'build()' call to produce an `Err(String)`;  \
        Received {:?}",
        rslt
      ),
      Err(_) => (),
    }

    Ok(())
  }

  #[test]
  fn test_option_ctrl_label_and_value_setters() {
    let mut ctrl1: OptionControl<usize> =
      OptionControlBuilder::default().value(98).build().unwrap();

    // Test base result
    assert_eq!(ctrl1.value.unwrap(), 98);
    assert_eq!(ctrl1.label.unwrap(), 98);
    assert_eq!(*ctrl1.get_value().unwrap(), 98, "Expected `value`'s value");

    for i in [99usize, 100usize] {
      // If `even` num., then `label` should be left untouched
      if i % 3 == 0 {
        ctrl1.set_value(Some(i));
        assert_eq!(ctrl1.value.unwrap(), i);
        assert_eq!(*ctrl1.get_value().unwrap(), i, "Expected `value`'s value");
        assert_eq!(ctrl1.label.unwrap(), 98);
      }
      // Else `value` should be left untouched
      else {
        ctrl1.set_label(Some(i));
        assert_eq!(ctrl1.value.unwrap(), 99);
        assert_eq!(*ctrl1.get_value().unwrap(), 99, "Expected `value`'s value");
        assert_eq!(ctrl1.label.unwrap(), i);
      }
    }

    // Test unsetting fields
    ctrl1.set_value(None);

    // Only `value` field should be unset
    assert_eq!(ctrl1.value, None);
    assert_eq!(
      *ctrl1.get_value().unwrap(),
      100,
      "`get_value()` should always return either `value`\
     (first), or `label`'s value otherwise"
    ); //
    assert_eq!(ctrl1.label.unwrap(), 100);

    ctrl1.set_label(None);

    // Both `label`, and `value`, fields should be unset/`None`
    assert_eq!(ctrl1.value, None);
    assert_eq!(ctrl1.get_value(), None);
    assert_eq!(ctrl1.label, None);
  }

  #[test]
  fn test_option_ctrl_get_value() {
    let mut ctrl: OptionControl<usize> = OptionControl::new();
    assert_eq!(ctrl.get_value(), None.as_ref());
    assert_eq!(ctrl.get_value(), ctrl.value.as_ref());
    assert_eq!(ctrl.get_value(), ctrl.label.as_ref());

    // When `value` is not set
    ctrl.label = Some(99);
    assert_eq!(
      ctrl.get_value(),
      ctrl.label.as_ref(),
      "When `value` is not set, should return `label`'s value"
    );

    // When `value` is set
    ctrl.value = Some(100);
    assert_eq!(
      ctrl.get_value(),
      ctrl.value.as_ref(),
      "When `value` is set, should return `value`'s value"
    );

    // When `value` is unset
    ctrl.value = None;
    assert_eq!(ctrl.label, Some(99));
    assert_eq!(
      ctrl.get_value(),
      ctrl.label.as_ref(),
      "When `value` is 'unset', should return `label`'s value"
    );

    // When neither, `value`, or `label`, is set
    ctrl.label = None;
    assert_eq!(
      ctrl.get_value(),
      None,
      "When neither, `label`, and/or, `value`, is set, should return `None`"
    );
  }

  #[test]
  fn test_option_ctrl_set_disabled() {
    let mut ctrl: OptionControl<&str> = OptionControl::new();
    assert_default_option_ctrl(&ctrl);

    ctrl.set_disabled(true);
    assert!(ctrl.disabled);

    ctrl.set_disabled(false);
    assert_eq!(ctrl.disabled, false);

    // Create child options
    ctrl.options = Some(options_from_str_provider(VOWELS));

    // Ensure all options are "not" disabled, by default
    match ctrl.options.as_deref() {
      Some(ops) => assert!(
        ops.iter().all(|o| !o.disabled),
        "Options should not be disabled by default"
      ),
      None => panic!("Expect 'Vec<Box<OptionCtrl>>' found 'None'"),
    }

    ctrl.set_disabled(true);
    assert_eq!(ctrl.disabled, true);

    assert!(
      optgroup_is_disabled(&ctrl, true),
      "Expected 'optgroup' to be `disabled`"
    );
  }

  #[test]
  fn test_option_ctrl_is_value_in_options() {
    let bag = "abc";

    fn _new_option(value: String) -> OptionControl<String> {
      OptionControlBuilder::default()
        .value(value)
        .build()
        .unwrap()
    }

    fn _gen_options_for_option(
      o: &OptionControl<String>,
      slice: &str,
      value_suffix: usize,
    ) -> Option<Vec<Box<OptionControl<String>>>> {
      if slice.is_empty() {
        return None;
      }

      let out = slice
        .chars()
        .enumerate()
        .map(|(j, c2)| {
          let mut op = _new_option(format!("{}{}", c2.to_string(), value_suffix));
          let j2 = j + 1;
          if j2 <= slice.len() {
            op.options = _gen_options_for_option(&op, &slice[j2..], value_suffix + 1);
          }
          Box::new(op)
        })
        .collect();

      Some(out)
    }

    // Populate test control
    let options: Vec<Box<OptionControl<String>>> = bag
      .chars()
      .enumerate()
      .map(|(i, c)| {
        let mut option = _new_option(c.to_string());
        option.options = _gen_options_for_option(&option, &bag[i + 1..], 0);
        Box::new(option)
      })
      .collect();

    println!("{:#?}", &options);

    // @todo Upgrade the following case to be an assertion method to be reused.
    // Extract values we expect to find in `options`
    bag.chars().enumerate().for_each(|(i, c)| {
      let plain_c = c.to_string();

      // Truthy case
      assert!(is_value_in_options(Some(options.as_ref()), Some(&plain_c)));

      // Sub lists should not contain plain character
      assert_eq!(
        is_value_in_options(options[i].options.as_deref(), Some(&plain_c)),
        false
      );

      // Sub lists should not contain non-existent values
      assert_eq!(
        is_value_in_options(
          options[i].options.as_deref(),
          Some(&format!("{}{}", plain_c, bag.len() - 1))
        ),
        false
      );
    });

    // Falsy cases
    "efg".chars().enumerate().for_each(|(i, c)| {
      let plain_c = c.to_string();

      // Sub lists should not contain non-existing values
      assert_eq!(
        is_value_in_options(options[i].options.as_deref(), Some(&plain_c)),
        false
      );
    });
  }

  #[test]
  fn test_option_ctrl_as_opt_group() -> Result<(), Box<dyn Error>> {
    let mut opt_group =
      VOWELS
        .char_indices()
        .rfold(None as Option<OptionControl<&str>>, |right, (i, _)| {
          let mut left: OptionControl<&str> = OptionControlBuilder::default()
            .label(&VOWELS[i..i + 1])
            .build()
            .unwrap();

          left.options = match right {
            Some(right) => {
              let mut options: Vec<Box<OptionControl<&str>>> = vec![Box::new(right)];

              if i + 1 < VOWELS_LEN - 1 {
                let mut options_tail: Vec<Box<OptionControl<&str>>> = (&VOWELS[i + 1..])
                  .char_indices()
                  .map(|(i2, _)| {
                    Box::new(
                      OptionControlBuilder::default()
                        .label(&VOWELS[i + i2 + 1..i + i2 + 2])
                        .build()
                        .unwrap(),
                    )
                  })
                  .collect();

                options.append(&mut options_tail);
              }

              Some(options)
            }
            _ => None,
          };

          Some(left)
        });

    fn assert_nested_optgroup_structure(_og: &OptionControl<&str>) {
      _og.options.as_ref().map(|ops| {
        println!(
          "Asserting first childs are opt_groups, in opt_group {:?}",
          &_og.label
        );
        if ops[0].value.unwrap() != "u" {
          assert!(
            ops[0].options.is_some(),
            "Expected first control in list to be an 'opt_group'"
          );
        }
        assert_nested_optgroup_structure(&ops[0])
      });
    }

    fn assert_optgroup_disabled_state(_og: &OptionControl<&str>, disabled: bool) -> bool {
      println!(
        "Asserting disabled state `{}` for optgroup {:?}",
        disabled, _og.label
      );
      _og.options.as_deref().map_or(true, |ops| {
        ops.iter().all(|o| {
          let o_matches = o.disabled == disabled;
          let child_options_match = assert_optgroup_disabled_state(&o, disabled);
          assert!(o_matches, "Option {:?} is invalid", &o);
          assert!(child_options_match, "Some descendant options are invalid");
          o_matches && child_options_match
        })
      })
    }

    println!("Asserting generated 'base' 'test' optgroup structure");
    opt_group.as_ref().map(assert_nested_optgroup_structure);

    println!("Asserting disabled state `false` for optgroup structure");
    opt_group
      .as_ref()
      .map(|_og| assert_optgroup_disabled_state(_og, false));

    println!("Testing `set_disabled(true)` on first nested option");
    opt_group.as_mut().map(|og| {
      og.options.as_deref_mut().map(|ops| {
        ops[0].set_disabled(true); // Expect setter here to trigger recursive `disabled` property updates
        assert_optgroup_disabled_state(&ops[0], true);
      })
    });

    // println!("\nserde 'Deserialize' test:\n{}", serde_json::to_string_pretty(&opt_group)?);

    println!("Testing `set_disabled(false)` on first nested option");
    opt_group.as_mut().map(|og| {
      og.options.as_deref_mut().map(|ops| {
        ops[0].set_disabled(false); // Expect setter here to trigger recursive `disabled` property updates
        assert_optgroup_disabled_state(&ops[0], false);
      })
    });

    // println!("{:#?}", opt_group);

    // Test serde 'Deserialize' trait
    println!(
      "\nserde 'Deserialize' test:\n{}",
      serde_json::to_string_pretty(&opt_group)?
    );
    println!("End of serde 'Deserialize' test");

    Ok(())
  }
}
