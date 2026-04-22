# Spike: Evaluate `deunicode` for `RemoveDiacritics` Filter

**Issue:** #237
**Parent:** #235
**Date:** 2026-04-21
**Status:** Recommend Defer (lean-Approve behind a feature flag once a concrete consumer lands)

## Motivation

A `RemoveDiacritics` / `Unidecode`-style sanitizer comes up in three concrete user
scenarios:

1. **Pre-slug normalization.** `Slug { max_length }` already lowercases and collapses
   non-ASCII runs, but it leans on `char::is_alphanumeric`, which keeps `é`, `ñ`, `ü`
   as-is. A user who wants `"Café Münchner"` to slugify to `"cafe-munchner"` (not
   `"café-münchner"`) today has to pre-process the input themselves.
2. **Fuzzy / accent-insensitive search keys.** Building search indexes where the query
   `"naive"` should match the stored value `"naïve"` requires normalizing both sides
   to an accent-free form.
3. **ASCII-only storage / legacy sinks.** Filenames, email local-parts, legacy
   databases, punycode pre-processing, etc., where downstream systems reject
   non-ASCII bytes.

The question this spike answers: **should `walrs_filter` ship a built-in filter for
(1) and (2), and which crate — if any — should back it?**

## Options Evaluated

### Option A — `deunicode`

- Crate: <https://crates.io/crates/deunicode> — v1.6.2 (released 2025-04-27).
- Repo: <https://github.com/kornelski/deunicode>.
- License: **BSD-3-Clause**.
- Transitive deps: **0** runtime deps (pure lookup tables).
- Data tables: ~450 KB in-memory (~160 KB gzipped) covering ~75 K codepoints
  mapped to ~245 K ASCII characters. `no_std`-compatible.
- Compile-time cost: small — no proc-macros, no build script of significance.
  Adds a static table to the binary; single crate to compile.
- Binary-size cost: ~175 KB added to a release binary (per upstream README).
- Runtime cost: single-pass `char` → `&'static str` table lookup. `O(n)` in the
  number of input chars; cache-friendly for Latin-script hot paths. No worst-case
  pathological input class.
- Fidelity (diacritic-removal cases):
  - `café → cafe`, `naïve → naive`, `étude → etude`, `Æneid → AEneid` ✓
  - Non-Latin scripts are **transliterated**, not stripped:
    `北亰 → "Bei Jing"`, `げんまい茶 → "genmaiCha"`, emoji
    `🦄☣ → "unicorn biohazard"`.
  - That is a **superset** of "remove diacritics" — it is full Unicode-to-ASCII
    transliteration. The upstream README explicitly warns it uses a "one-size-fits-all
    1:1 mapping" and cannot handle language-specific or context-dependent romanization
    (e.g. Han characters resolve to a single Mandarin reading, illegible to
    Japanese readers).
- Maintenance signals: ~3.19 M downloads/month, used by 1,278 crates (94 direct),
  last release 2025-04-27 (~12 months ago as of 2026-04-21). Author is
  [kornelski](https://github.com/kornelski) (Kornel, prolific and reliable
  Rust-ecosystem maintainer — `lodepng-rust`, `cargo-deb`, `imgref`, etc.). Healthy.

### Option B — `unicode-normalization` + diacritic stripping

- Crate: <https://crates.io/crates/unicode-normalization> — v0.1.25 (released
  2025-10-30).
- Repo: <https://github.com/unicode-rs/unicode-normalization> (official
  `unicode-rs` org).
- License: **MIT OR Apache-2.0** (and the Unicode license for the embedded data).
- Transitive deps: 1 direct (`tinyvec`), shallow graph.
- Data tables: ~710 KB of compiled Unicode data (decomposition tables).
- Compile-time cost: moderate — larger tables than `deunicode`, one extra crate
  (`tinyvec`) to compile.
- Binary-size cost: larger than `deunicode` for this narrow use-case (full NFD
  data), but many downstream crates already transitively depend on it, so the
  *marginal* cost in a non-trivial workspace is often near zero.
- Runtime cost: `.nfd().filter(|c| !is_combining_mark(c)).collect()` — two-pass
  over decomposed chars, allocates a `String`. Slightly slower than `deunicode`'s
  direct lookup for pure-Latin input, but still `O(n)`.
- Fidelity (diacritic-removal cases):
  - `café → cafe`, `naïve → naive`, `étude → etude` ✓ (NFD splits `é` into
    `e` + U+0301 combining acute, filter drops the combining mark).
  - **Does not handle** pre-composed letters with no combining-mark decomposition:
    - `Æ` (U+00C6) has no NFD decomposition to `AE`; it stays as-is.
      `Æneid → Æneid` (a subsequent ASCII-only filter would drop it entirely).
    - `ß → ß` (unchanged; `Ss` transliteration requires separate logic).
    - `ø → ø`, `đ → đ`, `ł → ł` likewise survive NFD.
  - CJK, emoji, symbols are untouched (no transliteration).
  - **This is the "strictest" option** — it only removes combining diacritic marks
    from letters that decompose. It does NOT produce ASCII output.
- Maintenance signals: ~19.7 M downloads/month, used by 15,801 crates (559
  direct), last release 2025-10-30 (~6 months ago). Maintained by `unicode-rs`.
  Excellent health.

### Option C — `any_ascii`

- Crate: <https://crates.io/crates/any_ascii> — v0.3.3 (released 2025-06-29).
- Repo: <https://github.com/anyascii/anyascii>.
- License: **ISC**.
- Transitive deps: **0** runtime deps.
- Data tables: ~300 KB (smaller than `deunicode`) covering 124 K of 155 K
  Unicode codepoints.
- Compile-time cost: small. MSRV 1.42.
- Binary-size cost: smaller than `deunicode` per upstream ("often has a smaller
  file size"). Noticeably fewer LoC (~813).
- Runtime cost: table lookup, same order as `deunicode`.
- Fidelity (diacritic-removal cases):
  - `café → cafe`, `naïve → naive` ✓
  - Non-Latin: Cyrillic `Борис → "Boris"`, Mandarin `深圳 → "ShenZhen"`,
    emoji `👑 → ":crown:"`, symbols `☆ ♯ → "* #"`.
  - Upstream claims **better results than Unidecode/`deunicode`** on many edge
    cases and broader coverage (124 K vs 75 K codepoints).
  - Unknown characters map to empty strings (silently dropped).
  - Missing ~30 K rare CJK plus cuneiform.
- Maintenance signals: ~328 K downloads/month, 203 reverse deps (16 direct) —
  an order of magnitude less used than `deunicode`. Last release 2025-06-29
  (~10 months ago). Multi-language project (Rust is one of 12 bindings), so
  Rust-port churn can lag the parent project.

### Option D — Inline NFD + combining-mark filter (roll our own)

- Uses `unicode-normalization` only (same as Option B) and a one-line predicate.
- **Not a separate option in practice** — it *is* Option B. There is no
  meaningful way to build "strip diacritics" without either a full Unicode
  decomposition table or a transliteration table, both of which dwarf anything
  we'd hand-maintain.
- Rolling our own transliteration table (Option A/C equivalent from scratch) is
  out of scope: hundreds of KB of hand-curated data, with ongoing Unicode
  revision maintenance. **Dismissed.**

## Comparison Table

| Criterion | `deunicode` | `unicode-normalization` + filter | `any_ascii` |
|---|---|---|---|
| Output guarantee | Pure ASCII | Original script, minus combining marks | Pure ASCII |
| `café → cafe` | ✓ | ✓ | ✓ |
| `naïve → naive` | ✓ | ✓ | ✓ |
| `Æneid` | `AEneid` | `Æneid` (unchanged) | `AEneid` |
| `ß` | `ss` | `ß` (unchanged) | `ss` |
| `ø / đ / ł` | `o / d / l` | unchanged | `o / d / l` |
| Han `北京` | `Bei Jing` | `北京` (unchanged) | `BeiJing` |
| Emoji `🦄` | `unicorn` | `🦄` (unchanged) | `:unicorn_face:` |
| Runtime deps | 0 | 1 (`tinyvec`) | 0 |
| Embedded data | ~450 KB | ~710 KB | ~300 KB |
| Binary cost | ~175 KB | shared with existing users | smaller |
| License | BSD-3-Clause | MIT/Apache-2.0 (+ Unicode) | ISC |
| Last release | 2025-04-27 | 2025-10-30 | 2025-06-29 |
| Ecosystem usage | 1,278 crates | 15,801 crates | 203 crates |
| Surprise factor | Moderate (transliterates non-Latin) | Low (narrow, predictable) | Moderate (transliterates non-Latin) |

## Recommendation

**Recommend Defer.** If/when this filter is actually needed by a consumer, lean
toward **Approve with Option B (`unicode-normalization` + combining-mark filter)
behind a non-default feature flag**, with an optional `Transliterate` variant
later backed by Option A (`deunicode`) if a user asks for full ASCII folding.

### Rationale

1. **No concrete consumer today.** Issue #237 is a spike, #235 is the parent.
   Neither cites an in-tree caller. Adding a ~300–700 KB data-table dependency
   and a new public `FilterOp` variant ahead of demand contradicts the
   "Simplicity First" working principle.
2. **The three options do meaningfully different things.** Option B is the
   literal interpretation of "remove diacritics" — narrow, predictable, and
   round-trips non-Latin scripts unchanged. Options A and C do
   *Unicode-to-ASCII transliteration*, which is a different feature most users
   conflate with "remove diacritics" until they discover `北京 → "Bei Jing"` or
   `🦄 → "unicorn"` happening inside their form pipeline. Shipping the wrong
   one (or both, unlabelled) is a minor foot-gun.
3. **When we do ship it, the narrow, surprise-free option is the right default.**
   `unicode-normalization` is already by far the most widely-used of the three
   in the Rust ecosystem and is the canonical implementation of UAX #15. Its
   licensing (MIT/Apache) matches the walrs norm. A `RemoveDiacritics` filter
   backed by it will do exactly what its name says — no more, no less.
4. **Transliteration belongs under a different name.** If a follow-up asks for
   "make it ASCII, whatever it takes", a distinct `FilterOp::Transliterate`
   variant — backed by `deunicode` (more conservative, more widely deployed)
   or `any_ascii` (slightly better coverage, looser in the ecosystem) — would
   be the right shape. That's a separate, later ticket.
5. **Feature-gate either way.** Both crates add non-trivial binary size.
   `walrs_filter` currently has 0 such data-table deps. A `diacritics` feature
   flag keeps the baseline unchanged for users who don't opt in.

### Why not pick one now?

This spike is cheap to defer and expensive to get wrong: flipping
`FilterOp::RemoveDiacritics` from "strip combining marks" semantics
(Option B) to "transliterate to ASCII" semantics (Option A/C) later would be
a breaking behavior change, and the other direction is equally surprising.
Without a user driving requirements, defer.

## If Approved: Implementation Sketch

If the follow-up ticket proceeds with Option B (the recommended default), the
rough shape in `crates/filter/`:

**`Cargo.toml`**

```toml
[features]
default = ["validation"]
diacritics = ["dep:unicode-normalization"]
# ... existing features

[dependencies]
unicode-normalization = { version = "0.1", optional = true }
```

**`src/filter_op.rs`** — new variant (Cow-aware, gated):

```rust
/// Remove Unicode combining diacritic marks (NFD-decompose, drop combining marks,
/// recompose).
///
/// `café → cafe`, `naïve → naive`, `étude → etude`.
///
/// Does **not** transliterate non-Latin scripts: `北京` stays `北京`, `🦄` stays
/// `🦄`. For full Unicode→ASCII folding, see `Transliterate` (follow-up).
///
/// Letters with no combining-mark decomposition (`Æ`, `ß`, `ø`, `đ`, `ł`) are
/// preserved as-is. Pair with `Slug` / `AllowChars` if pure ASCII output is
/// required.
#[cfg(feature = "diacritics")]
RemoveDiacritics,
```

And in the `apply`/`filter` match arms:

```rust
#[cfg(feature = "diacritics")]
FilterOp::RemoveDiacritics => {
    use unicode_normalization::{UnicodeNormalization, char::is_combining_mark};
    // Allocation-free fast path: if input has no chars that would change
    // under NFD (all-ASCII), return Cow::Borrowed.
    if value.is_ascii() {
        return Cow::Borrowed(value);
    }
    Cow::Owned(value.nfd().filter(|c| !is_combining_mark(*c)).collect())
}
```

Mirror the existing `FilterOp::Slug` / `FilterOp::StripTags` conventions for
`Debug`, `PartialEq`, serde `#[cfg(feature = "diacritics")]` gating, and the
`FilterOp<Value>` adapter.

## If Approved: Follow-up Ticket Spec

The follow-up implementation ticket should contain:

- **Title:** `feat(walrs_filter): add RemoveDiacritics filter (feature-gated)`
- **Parent:** #235
- **Predecessor:** #237 (this spike)
- **Scope:**
  - Add `diacritics` feature to `crates/filter/Cargo.toml`, gating an optional
    `unicode-normalization` dep.
  - Add `FilterOp::RemoveDiacritics` variant for both `FilterOp<String>` and
    the `FilterOp<Value>` string-adapter path, behind `#[cfg(feature = "diacritics")]`.
  - Extend `Debug`, `PartialEq`, serde, and the `TryFilterOp` paths (if
    applicable) to cover the new variant.
  - ASCII fast-path: return `Cow::Borrowed` when input is pure ASCII.
  - Unit tests covering: empty string, all-ASCII (borrow), Latin+diacritics,
    combining marks, non-Latin (unchanged), mixed input, idempotency.
  - Criterion benchmark entry under `benches/filter_benchmarks.rs` for a
    representative Latin-with-diacritics string.
  - README + feature-flag table updates in `crates/filter/README.md` and the
    workspace `README.md`.
  - Coverage over 80% as per `CLAUDE.md`.
- **Out of scope:**
  - Full Unicode→ASCII transliteration (file a separate ticket if/when a
    consumer needs it; `deunicode` is the suggested backing crate).
  - Language-specific romanization (Pinyin, Hepburn, etc.).

## References

- Parent ticket: #235
- Spike ticket: #237
- `deunicode` — <https://crates.io/crates/deunicode> (v1.6.2, 2025-04-27)
- `deunicode` repo — <https://github.com/kornelski/deunicode>
- `unicode-normalization` — <https://crates.io/crates/unicode-normalization> (v0.1.25, 2025-10-30)
- `unicode-normalization` repo — <https://github.com/unicode-rs/unicode-normalization>
- `any_ascii` — <https://crates.io/crates/any_ascii> (v0.3.3, 2025-06-29)
- `any_ascii` repo — <https://github.com/anyascii/anyascii>
- UAX #15 (Unicode Normalization Forms) — <https://www.unicode.org/reports/tr15/>
