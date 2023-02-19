# <https://github.com/casey/just#quick-start>
# <https://just.systems/man/en/>

set dotenv-load := true

# Execute formatters and linters
lint:
    pre-commit run --all-files
    cargo clippy

# Print help about 'just'
help:
    @ just --list
    @ echo "For more help: https://just.systems/man/en/chapter_22.html"
    @ echo "Environment variables to be set or put in an .env file: DIST_DIR, DEVBOARD"

# Build and open project documentation
doc:
    cargo doc --open --lib

# vim: set ft=make :
