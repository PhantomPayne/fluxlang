# Copilot Instructions for Fluxlang

## Project Overview

Flux is a functional, columnar-first language targeting WebAssembly. This is a Rust workspace with six crates, a VS Code extension, and example programs.

## Repository Layout

```
crates/
  flux-syntax/   # Lexer (logos), parser, AST — snapshot tests via insta
  flux-errors/   # Error types using miette + thiserror
  flux-sema/     # Semantic analysis, type checking, VFS, symbol table
  flux-wasm/     # WASM codegen (wasm-encoder) + runtime (wasmtime)
  flux-lsp/      # Language Server Protocol server (tower-lsp) — binary: flux-lsp
  flux-cli/      # CLI tool (parse/check/compile commands) — binary: flux
editors/
  vscode/        # VS Code extension (TypeScript, TextMate grammar)
examples/        # .flux example programs
```

## Build & Test Commands

```bash
# Build everything
cargo build

# Run all tests
cargo test --all

# Format check (CI enforces this)
cargo fmt --all --check

# Lint (CI enforces -D warnings)
cargo clippy --all-targets -- -D warnings

# Run a specific crate's tests
cargo test -p flux-syntax
cargo test -p flux-wasm

# Review insta snapshots after changes to the parser
cargo insta review
```

## Coding Conventions

- **Rust edition 2021**, resolver v2.
- Run `cargo fmt` before committing — CI rejects unformatted code.
- Run `cargo clippy -- -D warnings` before committing — CI treats all warnings as errors.
- Every new parser feature needs snapshot tests in `crates/flux-syntax/tests/`.
- Every new WASM codegen feature needs integration tests in `crates/flux-wasm/tests/`.
- Use `miette::Diagnostic` for user-facing errors; define them in `flux-errors`.

## Adding a New Language Feature

1. Add tokens in `crates/flux-syntax/src/lexer.rs`
2. Update the parser in `crates/flux-syntax/src/parser.rs`
3. Add snapshot tests in `crates/flux-syntax/tests/`
4. Add semantic analysis in `crates/flux-sema/` if needed
5. Update WASM codegen in `crates/flux-wasm/src/codegen.rs`
6. Add integration tests in `crates/flux-wasm/tests/`

## Key Dependencies

| Crate | Purpose |
|-------|---------|
| logos | Lexer generator |
| rowan | Lossless syntax trees |
| insta | Snapshot testing |
| miette | Diagnostic error reporting |
| thiserror | Derive Error trait |
| wasm-encoder | WASM binary encoding |
| wasmtime | WASM runtime for tests |
| tower-lsp | LSP server framework |
| tokio | Async runtime |
| dashmap | Concurrent hash map (VFS) |

## Testing Strategy

- **Unit tests**: Inline `#[cfg(test)]` modules in most crates.
- **Snapshot tests**: `crates/flux-syntax/tests/` using `insta`. Run `cargo insta review` to accept changes.
- **Integration tests**: `crates/flux-wasm/tests/integration_tests.rs` compiles Flux → WASM and executes with wasmtime.
- **VS Code extension**: `editors/vscode/src/test/client-test.js` (requires Node.js 18+).

## CI

GitHub Actions runs on every push to `main` and on pull requests:
- `cargo fmt --all --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test --all`
- `cargo build --all-targets`

All four checks must pass before merging.
