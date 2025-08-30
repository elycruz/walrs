CARGO_INCREMENTAL=0 RUSTFLAGS='-Cinstrument-coverage' LLVM_PROFILE_FILE='target/.profraw/cargo-test-%p-%m.profraw' \
cargo test --workspace && \
sh ./scripts/build-coverage.sh
