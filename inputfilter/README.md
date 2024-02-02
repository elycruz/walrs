# wal_inputfilter

A set of input validation structs used to validate primitive values as they pertain to web applications.

## Members

- `constraints` - Contains constraint structs.
  - `ScalarConstraints` - Validates scalar values.
  - `StringConstraints` - Validates string/string slice values.
- `validators`
  - `NumberValidator` - Validates numeric values.
  - `PatternValidator` - Validates values against a regular expression.
  - `EqualityValidator` - Validates values against a stored right-hand-side value.
- `filters`
  - `SlugFilter` - Filters value to valid "slug" values.
  - `StripTagsFilter` - Filters values against a regular expression.
  - `XmlEntitiesFilter` - Filters values against a stored right-hand-side

## Usage:

See tests.

## License:

MIT 3.0 + Apache 2.0
