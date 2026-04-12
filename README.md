# walrs

An experimental Web Application Library for Rust. The project is in research and development stage — please do not use it for production.

## Usage

Add the root crate to get access to all sub-crates:

```toml
[dependencies]
walrs = "0.1"
```

Then import any sub-crate as a module:

```rust
use walrs::inputfilter::{Field, FieldBuilder};
use walrs::validation::Rule;
use walrs::filter::FilterOp;
```

### Feature flags

All sub-crates are enabled by default. Disable features you don't need to reduce compile times:

```toml
[dependencies]
walrs = { version = "0.1", default-features = false, features = ["inputfilter", "validation", "filter"] }
```

Available features: `acl`, `digraph`, `filter`, `form`, `graph`, `inputfilter`, `navigation`, `rbac`, `validation`.

You can also depend on individual sub-crates directly (e.g., `walrs_inputfilter = "0.1"`).

## Sub-crates

| Crate | Description |
|---|---|
| `walrs_acl` | Access control list structure |
| `walrs_digraph` | Directed graph structures |
| `walrs_filter` | Input value transformation/sanitization filters |
| `walrs_form` | Form elements and structure for web frameworks |
| `walrs_graph` | Undirected graph structures |
| `walrs_inputfilter` | Field-level validation and filtering for form processing |
| `walrs_navigation` | Web page link graph / navigation structures |
| `walrs_rbac` | Role-Based Access Control |
| `walrs_validation` | Composable validation rules |

## Development

### Code coverage with `cargo-llvm-cov`

1.  Install `cargo-llvm-cov`: `cargo install cargo-llvm-cov`
2.  Run tests with coverage and generate HTML report: `sh ./llvm-cov-all.sh` or the command directly: `cargo llvm-cov --html --workspace --branch`
3.  Open the generated report at `target/llvm-cov/html/index.html` in your web browser.

Cargo llvm-cov reference: https://github.com/taiki-e/cargo-llvm-cov

### Code coverage with `grcov`

Note: branch and functions tracking is not supported with this method (currently).

1. Install `llvm-tools`: `$ rustup component add llvm-tools-preview`
2. Install grcov: `cargo install grcov`.
3. Run `sh ./grcov-all.sh` (builds project with instrumentation and runs tests).
4. Run the coverage "index.html" file (target/coverage/html/index.html) in the browser.

Reference: https://github.com/mozilla/grcov?tab=readme-ov-file#how-to-get-grcov

## License

[Elastic License 2.0 (ELv2)](./LICENSE)

Free to use, modify, and distribute for personal or internal use. Commercial
resale and managed-service offerings require explicit permission from the
copyright holder. See [NOTICE](./NOTICE) for AI-assisted development disclosure.
