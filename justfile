# RustAG task runner. Run `just` to list recipes.

set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

default:
    @just --list

# --- Rust ---------------------------------------------------------------

# Build the whole workspace (debug).
build:
    cargo build --workspace

# Build optimized release binaries.
release:
    cargo build --workspace --release

# Run the CLI (pass args after `--`, e.g. `just dev -- status`).
dev *ARGS:
    cargo run -p rustag-cli -- {{ARGS}}

# Run all tests. Skips network/mainnet tests by default (see `test-all`).
test:
    cargo test --workspace

# Run every test, including the ones that hit mainnet RPC.
test-all:
    cargo test --workspace -- --include-ignored

# Lint: clippy with warnings denied + format check.
lint:
    cargo clippy --workspace --all-targets -- -D warnings
    cargo fmt --all --check

# Auto-format.
fmt:
    cargo fmt --all

# Full CI gate, mirrors section 11 of the spec.
ci: lint test

# --- TypeScript ---------------------------------------------------------

# Install JS workspace dependencies.
js-install:
    pnpm install

# Build SDK + dashboard.
js-build:
    pnpm -r build

# Type-check all TS packages.
js-check:
    pnpm -r typecheck

# Run the dashboard dev server.
dashboard:
    pnpm --filter dashboard dev

# --- Misc ---------------------------------------------------------------

# Remove build artifacts and local stagenet state.
clean:
    cargo clean
    -Remove-Item -Recurse -Force .rustag -ErrorAction SilentlyContinue
