# wal_inputfilter

A set of input validation structs used to validate primitive values as they pertain to web applications.

## Members

- `input`: Contains `Input` struct and some types for composing validators, attributes, and filters for value validation and filtering.
- ~~`constraints` - Contains constraint structs.~~ - Replaced by `validators` and `input` module/structs.
  - ~~`ScalarConstraints` - Validates scalar values.~~
  - ~~`StringConstraints` - Validates string/string slice values.~~
- `validators`
  - `EqualityValidator` - Validates values against a stored right-hand-side value.
  - `LengthValidator` - Validates slice's length.
  - `NumberValidator` - Validates numeric values.
  - `PatternValidator` - Validates values against a regular expression.
  - `RangeValidator` - Validates scalar value's range.
- `filters`
  - `SlugFilter` - Filters values to valid "slug" values.
  - `StripTagsFilter` - Filters out HTML tags, and/or, attributes.
  - `XmlEntitiesFilter` - Encodes XML entities.

## Usage:

See tests.

## License:

MIT 3.0 + Apache 2.0
