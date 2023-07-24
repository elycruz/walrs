# `inputfilter` package design

## Inspiration

- https://docs.laminas.dev/laminas-inputfilter/

## Questions

- Do function references need to be wrapped in `Arc<...>` to be shared across threads safely?  Yes.

## FAQs

- What are the pros and cons of accepting `Cow<T>`, vs, `&T`, vs `T` in validator functions (note all userland, and lib. land, validators will have to match chosen type)?
  - For now we'll think about/implement the design using `&T`, for validators.  If any roadblocks are reached we'll adjust to them as required. 
- Is it ok to not wrap `Input.validators` in an `Arc<>`?  Seems `Arc` makes all internal members, of target value, accessible in "atomic reference" context, hence probably not requiring us to wrap internal members, since the purpose of using Arc is to allow the parent struct to be reused across multiple threads.
