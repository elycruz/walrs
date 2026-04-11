# Fuzz Testing Strategy for `walrs`

## Question

What is the best approach for integrating fuzz testing into the walrs workspace? Should it only run in CI/CD, be triggerable locally, or both? What tools exist in the Rust ecosystem that we can automate into our CI/CD process?

---

## Current State

- **No fuzzing infrastructure** exists (no `fuzz/` directories, no `proptest`/`arbitrary`/`cargo-fuzz` dependencies).
- CI uses **nightly Rust** (`dtolnay/rust-toolchain@master`) with `cargo build` + `cargo test` only.
- Coverage tooling (grcov, llvm-cov) is mature but **local-only** — not wired into CI.
- Tests are **assertion-based** (manual examples), not property-based.
- 7 of 9 crates have tests; benchmarks present in several.

---

## Rust Fuzz Testing Ecosystem

### Coverage-Guided Fuzzers

| Tool | Engine | Notes |
|------|--------|-------|
| **cargo-fuzz** | libFuzzer (LLVM) | Most popular, requires nightly, first-class Cargo integration |
| **afl.rs** | AFL++ | Alternative engine, good for diversity; also needs nightly |
| **bolero** | Abstracts libFuzzer, AFL, + property testing | Write one harness, run under multiple engines or as a unit test on stable |

### Property-Based / Structured Fuzzing

| Tool | Notes |
|------|-------|
| **proptest** | QuickCheck-style, works on stable, great for structured inputs |
| **arbitrary** | Derive macro for generating structured types from raw bytes; pairs with cargo-fuzz |
| **quickcheck** | Simpler alternative to proptest |

### CI/CD Services

| Service | Notes |
|---------|-------|
| **OSS-Fuzz** (Google) | Free for open-source; runs cargo-fuzz harnesses 24/7 on Google infra |
| **ClusterFuzzLite** | Lighter OSS-Fuzz variant designed for GitHub Actions CI |
| **cargo-fuzz in GH Actions** | Run short (~60–120s) fuzz sessions per PR as a smoke test |

---

## Recommended Tool Stack

| Layer | Tool | Rationale |
|-------|------|-----------|
| **Coverage-guided fuzzing** | `cargo-fuzz` (libFuzzer) | Best Rust integration; nightly already used in CI |
| **Structured input generation** | `arbitrary` (derive) | Generate valid `Rule`, `FilterOp`, `Value` variants from raw bytes |
| **Property-based testing** | `proptest` | Complement for graph/acl/rbac invariant checking; works on stable |
| **CI smoke runs (per-PR)** | `cargo fuzz run <target> -- -max_total_time=120` | Short runs to catch regressions |
| **Deep fuzzing (scheduled)** | ClusterFuzzLite or scheduled GH Actions | Nightly/weekly multi-hour runs |

---

## Fuzz Target Priority

### 🔴 Critical — walrs_validation

- **Why:** Complex parsing logic for email, URL, IP, hostname, date formats; regex `Pattern` compilation; rule composition trees (`All`, `Any`, `Not`, `When`).
- **Key types:** `Rule<T>`, `Value` (dynamic enum with 8 variants), `Validate<T>` / `ValidateRef<T>`.
- **Targets:** `Rule::Email`, `Rule::Url`, `Rule::Uri`, `Rule::Ip`, `Rule::Hostname`, `Rule::Date`, `Rule::DateRange`, `Rule::Pattern`, composed rule trees.

### 🔴 High — walrs_filter

- **Why:** HTML sanitization via Ammonia (`StripTagsFilter`), regex-based slug generation (`SlugFilter`), chained transformations.
- **Key types:** `FilterOp<T>`, `TryFilterOp<T>`, `Filter` trait.
- **Targets:** `StripTagsFilter::apply`, `SlugFilter::apply`, chained `FilterOp::Chain` sequences.

### 🔴 High — walrs_inputfilter

- **Why:** Combines `FilterOp` + `Rule` validation on untrusted form input; fallible filter pipelines.
- **Key types:** `Field<T>`, `FieldFilter`, `FormViolations`.
- **Targets:** `Field::validate`, `FieldFilter::validate`, combined filter+rule pipelines.

### 🟡 Medium — walrs_form

- **Why:** `FormData` path-based access, JSON deserialization of form structures.
- **Key types:** `Form`, `FormData`, `Element` (polymorphic enum).
- **Targets:** `FormData` path parsing, element deserialization.

### 🟡 Medium — walrs_acl / walrs_rbac

- **Why:** JSON deserialization into role inheritance graphs, permission queries.
- **Key types:** `Acl`, `AclBuilder`, `AclData`, `Rbac`, `RbacBuilder`.
- **Targets:** `AclData::try_from`, `RbacData` deserialization, permission resolution.

### 🟢 Low — walrs_digraph / walrs_graph

- **Why:** Internal data structures with less user-input surface.
- **Targets:** Graph algorithm invariants (cycle detection, topological sort) — better suited for `proptest`.

### 🟢 Low — walrs_navigation

- **Why:** Tree construction from JSON/YAML.
- **Targets:** `Container` deserialization, hierarchical path parsing.

---

## Local vs. CI/CD — Both

### Local Development

```bash
# Run a specific fuzz target for 60 seconds
cargo fuzz run <target> -- -max_total_time=60

# Run with a specific corpus
cargo fuzz run <target> fuzz/corpus/<target>/ -- -max_total_time=60

# List available targets
cargo fuzz list
```

- Developers run fuzz targets during development to explore new code paths.
- Corpus stored in `fuzz/corpus/` and committed to git so findings persist and are shared.

### CI — Per-PR Smoke Runs

```yaml
# In .github/workflows/fuzz.yml
- name: Fuzz smoke test
  run: |
    cargo install cargo-fuzz
    for target in $(cargo fuzz list); do
      cargo fuzz run "$target" -- -max_total_time=120
    done
```

- Short runs (60–120s) per target on every PR.
- Catches regressions introduced by the PR against the committed corpus.

### CI — Scheduled Deep Runs (Nightly/Weekly)

```yaml
on:
  schedule:
    - cron: '0 3 * * 1'  # Weekly on Mondays at 3 AM UTC

jobs:
  fuzz:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        target: [validation_rules, filter_ops, inputfilter_fields, ...]
    steps:
      - run: cargo fuzz run ${{ matrix.target }} -- -max_total_time=3600
      # Optionally commit new corpus entries via bot PR
```

- Longer runs (1+ hours per target) explore deeper state spaces.
- New corpus entries can be committed automatically via a bot PR.

### Optional — OSS-Fuzz / ClusterFuzzLite

- Free continuous fuzzing for open-source projects on Google infrastructure.
- ClusterFuzzLite is the lighter variant designed for GitHub Actions.
- Runs harnesses 24/7 and files issues for discovered crashes.

---

## Key Considerations

1. **Edition 2024:** The workspace uses Rust edition 2024. `cargo-fuzz` and libFuzzer work fine with nightly (already used in CI). Verify compatibility for stable-channel property tests via `proptest`/`bolero`.

2. **Corpus management:** Commit corpus files to git. This ensures coverage-guided fuzzing builds on previous discoveries across all environments (local, CI, scheduled).

3. **`arbitrary` derive:** Implementing `Arbitrary` for key types (`Rule`, `FilterOp`, `Value`, `AclData`) enables structured fuzzing that generates valid variant combinations rather than random bytes.

4. **Incremental adoption:** Start with the highest-value targets (validation parsers, HTML sanitization) and expand to other crates over time.

---

## Decision

**TBD** — To be decided after review.
