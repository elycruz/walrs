# `inputfilter` package design

## Inspiration

- https://docs.laminas.dev/laminas-inputfilter/

## Questions

- Do function references need to be wrapped in `Arc<...>` to be shared across threads safely?  Yes.

## Scratch area

```text
// Pseudo code

type Message = String

Validator<T>
  validate(&self, value: Option<T>) -> Result<(), Message>

Input<T>
  validate(&self, value: Option<T>) -> Result<(), Error>,
  filter(value: Option<T>) -> Option<T>

Inputfilter<T>
  filter(&self, value: Option<T>) -> Result<Option<T>, Error> {
    calls validate
    calls filter
  }

Component Context
  validate_and_filter (&self, _inpt_filters: &InputFilter) -> Result<Self, HashMap<&str, Vec<String>>>,



```