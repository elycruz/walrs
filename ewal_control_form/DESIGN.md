# Ecms_Form package design

## Problem

Build a form control library that facilitates easy/organized management of form data, their validation, and control configurations (passing control configuration data to control views (html form controls etc.)), to-and-fro applications, for web applications.  The library should work for multi-threaded environments as well.  

## General/Frequently Asked Questions

What are the general data types we require? 

- Form
- Fieldset
- Form Control
- Form Control Value
- Form Control Constraints
- Number Constraints
- Text Constraints

What form controls would we require?

- Select
- RadioGroup
- Checkbox
- Button
- Input
- Textarea

Can it be efficient? We can use `Send + Sync + 'static` types to ensure our methods/callbacks are thread safe.  Additionally, the `Cow<'_, X>` type can be used to allow control structs to not require ownership of values/not have to worry too much about lifetimes of incoming values. 

## Definitions

- rendering/render contexts - When rendering json (when using serde, etc.), or when rendering content from handlebars templates, etc..

## Traits

- `trait FormControlValue: Clone + Debug + PartialEq {}` - Form control value - Value that gets validated by Constraint types.  Additionally, these are used in render contexts (Handlebars, serde, etc.).

- `trait InputConstraint<Value>` - Constraints type - Contains fields that control constriant validation, validation error messaging, for incoming values (`FormControlValue` types etc.), on `FormControls` or directly.  

- `trait FormControlConstraints<Value: FormControlValue>: Clone + Debug + InputConstraints<Value> {}` - Our form control constraints struct - Used to facilitate validation on form controls.

- `trait FormControl<'a, Value, ValueConstraints>
  where Value: 'a + FormControlValue,
  ValueConstraints: 'a + FormControlConstraints<Value>` - Our control type - Added to fieldset/form elements, as child controls, and also used in rendering contexts (Handlebars, serde, etc.).

### Notes:

- `Deref` + `DerefMut` are reserved for smart pointers so this constraint should only be used when requiring a smart pointer.

### HTMLSelectControl

#### Specs

##### `options`

@todo

##### `set_value()`

When `set_value()` is called with a `None` control's internal value property should be set, and `check_validity()` should be called, returning it's validity result.

##### `set_values()`

@todo

##### `check_validity()`

When `value` is set to `Some()`, and control's `required` state is set to `false`, and options are `Some()`, and set value is not in contained options, validity check should return `false`;  

When control's `options` are `None`, and `value` is a `Some()`, control's validity check should return `false`.

When control is not marked as `required` and `value` is `None` control's validity check should return `true`.

When control's `options`, and `value`, are `None`, and `required` is `false`, controls validity check should return `true`

When control's `options` are `None` controls validity check should return `false`.

### Todos:

- [x] `set_value()`
- [ ] `set_values()`
- [x] `check_validity()`
