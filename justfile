# <https://github.com/casey/just#quick-start>
# <https://just.systems/man/en/>

reponame := "ripdis"
build_image := "rust:bullseye"

# Run formatters, linters and tests (not audit)
default: format spell check test outdated
    @ echo "Everything's good! 🍰"

# Scan for vulnerabilities
audit:
    cargo audit

# Automatic code formatting
format:
    cargo fmt

# Find common misspellings
spell:
    # Fix spelling errors with 'codespell -wi3' or add exceptions to .codespellrc
    codespell

# Lint
check:
    cargo check --tests --all-features
    cargo clippy --tests --all-features -- --forbid unsafe_code --deny warnings
    pre-commit run --all-files
    cargo doc --no-deps --all-features

# Run tests
test:
    cargo test

# Check for outdated dependencies (NB: NOT RUN IN CI)
outdated:
    cargo outdated --root-deps-only --exit-code 1 --exclude ceamprotodiane,adxl345_driver,ceamlibrabbitmq

# Check test coverage
coverage:
    cargo tarpaulin --skip-clean --target-dir target/coverage # to avoid recompilation

# Publish development version on 0.0.0.0:8080
publish-dev-armv7:
    podman run -i \
      -v "$(pwd)":/srv \
      -v {{reponame}}-cargo:/usr/local/cargo \
      --workdir /srv \
      -t {{build_image}} /bin/bash -c "./ci-tools/build-debug-armv7"
    sfz -p 8080 -b 0.0.0.0 target/armv7-unknown-linux-gnueabihf/debug/

update-all:
    devenv update
    cargo update
    podman pull {{build_image}}

# Build and open project documentation
doc:
    cargo doc --open --lib

# Print help about 'just'
help:
    @ just --list
    @ echo "More help: https://just.systems/man/en/chapter_22.html"
