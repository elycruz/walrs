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
use walrs::fieldfilter::{Field, FieldBuilder};
use walrs::validation::Rule;
use walrs::filter::FilterOp;
```

### Feature flags

All sub-crates are enabled by default. Disable features you don't need to reduce compile times:

```toml
[dependencies]
walrs = { version = "0.1", default-features = false, features = ["fieldfilter", "validation", "filter"] }
```

Available features: `acl`, `digraph`, `filter`, `graph`, `fieldfilter`, `fieldfilter-derive`, `navigation`, `rbac`, `validation`.

`fieldfilter-derive` opts into the `#[derive(Fieldset)]` proc-macro — it implies `fieldfilter` and enables `walrs_fieldfilter`'s `derive` feature:

```toml
[dependencies]
walrs = { version = "0.1", default-features = false, features = ["fieldfilter-derive", "validation"] }
```

```rust
use walrs::fieldfilter::{DeriveFieldset, Fieldset};
```

You can also depend on individual sub-crates directly (e.g., `walrs_fieldfilter = "0.1"`).

## Sub-crates

| Crate | Description |
|---|---|
| `walrs_acl` | Access control list structure |
| `walrs_digraph` | Directed graph structures |
| `walrs_filter` | Input value transformation/sanitization filters |
| `walrs_graph` | Undirected graph structures |
| `walrs_fieldfilter` | Field-level validation and filtering for form processing |
| `walrs_fieldset_derive` | Proc-macro crate providing `#[derive(Fieldset)]`; consumed via `walrs_fieldfilter`'s `derive` feature |
| `walrs_navigation` | Web page link graph / navigation structures |
| `walrs_rbac` | Role-Based Access Control |
| `walrs_validation` | Composable validation rules |

### The typed path (`Fieldset`)

`walrs_fieldfilter` standardises on the typed `Fieldset` trait (or
`#[derive(Fieldset)]` via the `fieldfilter-derive` feature) — define a struct
describing your fields and get statically-checked validation and filtering.

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

### Fuzz testing with `cargo-fuzz`

The workspace includes [cargo-fuzz](https://github.com/rust-fuzz/cargo-fuzz) targets for crates that handle untrusted input.

#### Prerequisites

```bash
# cargo-fuzz requires nightly Rust
rustup install nightly

# Install cargo-fuzz
cargo install cargo-fuzz
```

#### Running fuzz targets locally

```bash
# List available targets for a crate
cd crates/validation
cargo fuzz list

# Run a specific target for 60 seconds
cargo fuzz run fuzz_email -- -max_total_time=60

# Run with a specific corpus directory
cargo fuzz run fuzz_email fuzz/corpus/fuzz_email/ -- -max_total_time=60
```

#### Fuzz target crates

| Crate | Targets | Focus |
|---|---|---|
| `walrs_validation` | `fuzz_email`, `fuzz_url`, `fuzz_ip`, `fuzz_hostname`, `fuzz_date`, `fuzz_rule_composition` | String parsers, rule composition |
| `walrs_filter` | `fuzz_strip_tags`, `fuzz_slug`, `fuzz_filter_op_string` | HTML sanitization, slug generation, filter chains |
| `walrs_fieldfilter` | `fuzz_field_string_clean` | Field validation pipelines |

#### CI integration

- **Per-PR smoke runs**: Every PR touching validation/filter/fieldfilter crates triggers 60-second fuzz runs per target.
- **Scheduled deep runs**: Weekly 30-minute runs per target on Mondays at 3 AM UTC (also manually triggerable).
- Crash artifacts are uploaded on failure for investigation.

## License

[Elastic License 2.0 (ELv2)](./LICENSE)

Free to use, modify, and distribute for personal or internal use. Commercial
resale and managed-service offerings require explicit permission from the
copyright holder. See [NOTICE](./NOTICE) for AI-assisted development disclosure.
