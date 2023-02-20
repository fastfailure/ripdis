# <https://github.com/casey/just#quick-start>
# <https://just.systems/man/en/>

# set dotenv-load := true

# Execute formatters and linters
lint:
    cargo fmt
    pre-commit run --all-files
    cargo clippy

# Print help about 'just'
help:
    @ just --list
    @ echo "More help: https://just.systems/man/en/chapter_22.html"

# vim: set ft=make :
