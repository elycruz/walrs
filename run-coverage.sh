rm -rf target/.profraw target/coverage & \
CARGO_INCREMENTAL=0 RUSTFLAGS='-Cinstrument-coverage' LLVM_PROFILE_FILE='target/.profraw/cargo-test-%p-%m.profraw' cargo test --workspace -- --test-threads 32 && \
grcov . --binary-path ./target/debug/deps/ -s . -t html --branch --ignore-not-existing --ignore '../*' --ignore "/*" -o ./target/coverage/html && \
echo "Process completed successfully.\nCoverage generated to ./target/coverage/html\n"