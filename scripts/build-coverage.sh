rm -rf ./target/coverage/html && \
CARGO_INCREMENTAL=0 \
RUSTFLAGS='-Cinstrument-coverage -Zprofile -Ccodegen-units=1 -Copt-level=0 -Clink-dead-code -Coverflow-checks=off -Zpanic_abort_tests -Cpanic=abort' \
LLVM_PROFILE_FILE='target/.profraw/cargo-test-%p-%m.profraw' \
grcov . --binary-path ./target/debug/deps/ -s . -t html --branch --ignore-not-existing \
--ignore '_recycler' --ignore 'src/**/*' --ignore "target/debug/build/**/*" -o ./target/coverage/html \
--excl-line "unreachable!" \
 && echo "Process completed successfully.\nCoverage generated to ./target/coverage/html\n"