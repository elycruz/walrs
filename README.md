# wal-rs (work-in-progress)

An experimental Web Application Library, for Rust - The project is in research and development stage so please do not use it for production.

## (currently) in development:

- `wal_inputfilter` - A set of `Input` validation structs used to validate primitive values as they pertain to web applications.
- `wal_acl` - An access control list structure.
- `wal_graph` - A collection of basic graph structures to use in `wal_acl` and `wal_navigation`, etc.
- `wal_navigation` - A collection of structs to use to compose web page link graphs.  Can also be integrated with `wal_acl`.  This structure is overall useful in scenarios where page access needs to be controlled from the application level.

## Development

### Code coverage with `cargo-llvm-cov`

1.  Install `cargo-llvm-cov`: `cargo install cargo-llvm-cov`
2.  Run tests with coverage and generate HTML report: `sh ./test.sh` or the command directly: `cargo llvm-cov --html --workspace --branch`
3.  Open the generated report at `target/llvm-cov/html/index.html` in your web browser.

Cargo llvm-cov reference: https://github.com/taiki-e/cargo-llvm-cov

### Code coverage with `grcov`

Note: branch and functions tracking is not supported with this method (currently).

1. Install `llvm-tools`: `$ rustup component add llvm-tools-preview`
2. Install grcov: `cargo install grcov`.
3. Run `sh ./grcov-coverage.sh` (builds project with instrumentation and runs tests).
4. Run the coverage "index.html" file (target/coverage/html/index.html) in the browser.

Reference: https://github.com/mozilla/grcov?tab=readme-ov-file#how-to-get-grcov

## License:

MIT 3.0 + Apache 2.0
