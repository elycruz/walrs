# ECMS Form Controls Refactor

## `FormControl`

For our purposes we only need a generic form control that can validate any validatable form control;  E.g., html input, select, textarea, and/or button - Note not all of these are validatable on the browser side but can be validated on the server side.

Reference: 

- 
- https://html.spec.whatwg.org/multipage/input.html

### Validation properties:

See [MDN Constraint Validation](https://developer.mozilla.org/en-US/docs/Web/HTML/Constraint_validation) page for supported `type`, constraint violation, and description details.

- `pattern`
- `min`
- `max`
- `type`
- `required`
- `step`
- `minLength`
- `maxLength`

### Properties that control whether validation happens or not:

- `disabled` - Should be ignored on the server side;  Form controls on the browser side will not send values for controls that are `disabled`, so property doesn't matter on the server side, unless we want to control the element's 'enabled'/'disabled' state, on the browser side.
- `name` - On the browser side this one is required (for validation);  On the server side as well, as we cannot associate the error with an element unless we have the `name` for said element.
- `required` - If value isn't provided and `required` is `false` then validation should not occur.

### Properties that provide further control on how validation happens:

- `multiple` - For 'email', and 'URL', types allows multiple values to be entered.

### Our Implementation

#### Caveats

- Should allow setting all validation properties on control - All properties should be optional (`Option<...>`). 
- On serde serialize properties that are `None` should not show up in serialized output.
- Values that can be `Owned`/`Borrowed` should be stored in `Cow` enums.
- Validation should happen by way of calling a `call`, and/or, `validate`, method.
- Value to be validated should be passed directly to the `call`, and/or`, `validate` methods.
- Value to be validated should be able to be set (stored) on the control - Facilitates sending the control (for rendering) to view rendering layers;  Will require using `Cow` enums for non `Copy` values (will help avoid duplicating values in memory).
- Control should have a way to set/remove/check html attributes;  Values should be stored in a serde `Map` object (using serde `Json` values);  Set values, here, should be 'owned' values.
- Control should contain `validate`, and `validate_and_filter`, methods.

- @todo Figure out how we'll allow 

#### FAQs

- Are `&str` and `String` `PartialEq` and `PartialOrd`? Yes. [See here](https://doc.rust-lang.org/std/string/struct.String.html).

## Todos

1.  Gather common properties between native html form control elements.
   2.  Construct a `FormControl` struct (tentatively) that derives, `Builder`, serde `Serialize` and `Deserialize`, `Default`, and `Clone`.
3. @todo


