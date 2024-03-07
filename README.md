# wal-rs (work-in-progress)

An experimental Web Application Library, for Rust - The project is in research and development stage so please do not use it for production.

## (currently) in development:

- `wal_inputfilter` - A set of `Input` validation structs used to validate primitive values as they pertain to web applications.
- `wal_acl` - An access control list structure.
- `wal_graph` - A collection of basic graph structures to use in `wal_acl` and `wal_navigation`, etc.
- `wal_navigation` - A collection of structs to use to compose web page link graphs.  Can also be integrated with `wal_acl`.  This structure is overall useful in scenarios where page access needs to be controlled from the application level.

## Development

### Notes:

### Code coverage with `grcov`

1. Install `llvm-tools`: `$ rustup component add llvm-tools-preview`
2. Install grcov: `cargo install grcov`.
2. Build library with instrumentation:
```
CARGO_INCREMENTAL=0 RUSTFLAGS='-Cinstrument-coverage' LLVM_PROFILE_FILE='.profraw/cargo-test-%p-%m.profraw' cargo test
```
3. Run grcov: 
```bash
grcov . --binary-path ./target/debug/deps/ -s . -t html --branch --ignore-not-existing --ignore '../*' --ignore "/*" -o target/coverage/html
```

Reference: https://github.com/mozilla/grcov?tab=readme-ov-file#how-to-get-grcov

## License:

MIT 3.0 + Apache 2.0
