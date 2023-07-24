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

Other possible implementation:
```text
Input<T>

```

## FAQs

- What are the pros and cons of accepting `Cow<T>`, vs, `&T`, vs `T` in validator functions (note all userland, and lib. land, validators will have to match chosen type)?
  - For now we'll think about/implement the design using `&T`, for validators.  If any roadblocks are reached we'll adjust to them as required. 
- Is it ok to not wrap `Input.validators` in an `Arc<>`?  Seems `Arc` makes all internal members, of target value, accessible in "atomic reference" context, hence probably not requiring us to wrap internal members, since the purpose of using Arc is to allow the parent struct to be reused across multiple threads.
