# <https://github.com/casey/just#quick-start>
# <https://just.systems/man/en/>

# Run formatters, linters, tests and get code coverage
test:
    cargo fmt
    # Fix spelling errors with 'codespell -wi3' or add exceptions to .codespellrc
    codespell
    cargo check --all-features
    cargo clippy --all-features -- --forbid unsafe_code
    cargo check --tests
    cargo clippy --tests
    pre-commit run --all-files
    cargo test
    cargo doc --no-deps --all-features
    cargo audit

coverage:
    cargo tarpaulin --skip-clean --target-dir target/coverage # to avoid recompilation

# Build and open project documentation
doc:
    cargo doc --open --lib

# Print help about 'just'
help:
    @ just --list
    @ echo "More help: https://just.systems/man/en/chapter_22.html"
