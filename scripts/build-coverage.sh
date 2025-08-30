rm -rf ./target/coverage/html && \
CARGO_INCREMENTAL=0 RUSTFLAGS='-Cinstrument-coverage' LLVM_PROFILE_FILE='target/.profraw/cargo-test-%p-%m.profraw' \
grcov . --binary-path ./target/debug/deps/ -s . -t html --branch --ignore-not-existing \
--ignore '_recycler' --ignore 'src/**/*' --ignore "target/debug/build/**/*" -o ./target/coverage/html \
 && echo "Process completed successfully.\nCoverage generated to ./target/coverage/html\n"