# Contributing to Flux

Thank you for your interest in contributing to Flux! This document provides guidelines for contributing to the project.

## Code of Conduct

This project adheres to a [Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code.

## Getting Started

### Prerequisites

- Rust 1.93.0 or later (the project uses `rust-toolchain.toml` to pin the version)
- For VS Code extension development: Node.js 18+ and npm

### Setting Up Your Development Environment

1. **Clone the repository:**
   ```bash
   git clone https://github.com/PhantomPayne/fluxlang.git
   cd fluxlang
   ```

2. **Build the project:**
   ```bash
   cargo build
   ```

3. **Run tests:**
   ```bash
   cargo test --all
   ```

## Development Workflow

### Building

```bash
# Build all crates
cargo build

# Build in release mode
cargo build --release

# Build specific crate
cargo build -p flux-syntax
```

### Testing

```bash
# Run all tests
cargo test --all

# Run tests for a specific crate
cargo test -p flux-syntax
cargo test -p flux-sema
cargo test -p flux-wasm

# Run specific test
cargo test test_name
```

#### Snapshot Testing

The parser uses [Insta](https://insta.rs) for snapshot testing. After making changes to the parser:

1. **Run tests to generate new snapshots:**
   ```bash
   cargo test -p flux-syntax
   ```

2. **Review snapshot changes:**
   ```bash
   cargo insta review
   ```
   
   Or accept all changes:
   ```bash
   INSTA_UPDATE=always cargo test -p flux-syntax
   ```

3. **Commit both code changes and updated snapshots**

### Formatting

Before committing, ensure your code is properly formatted:

```bash
# Check formatting
cargo fmt --all --check

# Apply formatting
cargo fmt --all
```

### Linting

Run Clippy to catch common mistakes and improve code quality:

```bash
# Run clippy (CI enforces -D warnings)
cargo clippy --all-targets -- -D warnings

# Fix automatically fixable issues
cargo clippy --all-targets --fix
```

### Running the CLI

The project includes a command-line interface (`flux`) with several commands:

```bash
# Build the CLI
cargo build --release --bin flux

# Parse a Flux program and display AST
./target/release/flux parse examples/simple.flux

# Check a Flux program for syntax errors
./target/release/flux check examples/simple.flux

# Compile to WASM component
./target/release/flux compile examples/simple.flux output.wasm
```

### Running the LSP Server

The Language Server Protocol (LSP) implementation provides IDE support:

```bash
# Build the LSP server
cargo build --release --bin flux-lsp

# The LSP server is used by the VS Code extension
# See editors/vscode/README.md for extension development
```

#### VS Code Extension Development

1. **Install dependencies:**
   ```bash
   cd editors/vscode
   npm install
   ```

2. **Compile TypeScript:**
   ```bash
   npm run compile
   ```

3. **Open in VS Code and press F5** to launch the extension in a new VS Code window

4. **Create or open a `.flux` file** to test syntax highlighting and LSP features

## Pull Request Guidelines

### Before Submitting a PR

1. **Ensure all tests pass:**
   ```bash
   cargo test --all
   ```

2. **Format your code:**
   ```bash
   cargo fmt --all
   ```

3. **Run Clippy with no warnings:**
   ```bash
   cargo clippy --all-targets -- -D warnings
   ```

4. **Build successfully:**
   ```bash
   cargo build --all-targets
   ```

### PR Requirements

- **Clear description**: Explain what changes you made and why
- **Tests**: Add tests for new features or bug fixes
- **Documentation**: Update documentation if you change public APIs
- **Commit messages**: Write clear, descriptive commit messages
- **One feature per PR**: Keep PRs focused on a single feature or fix

### Code Quality Standards

- All tests must pass
- Code must be formatted with `rustfmt`
- No Clippy warnings (CI enforces `-D warnings`)
- New features should include tests
- Public APIs should be documented

## Adding New Features

When adding a new language feature, follow this workflow:

1. **Lexer**: Add tokens in `crates/flux-syntax/src/lexer.rs`
2. **AST**: Add node types in `crates/flux-syntax/src/ast.rs`
3. **Parser**: Add parsing logic in `crates/flux-syntax/src/parser.rs`
4. **Tests**: Add snapshot tests in `crates/flux-syntax/tests/`
5. **Semantic Analysis**: Update `crates/flux-sema/` if needed
6. **Code Generation**: Update `crates/flux-wasm/src/codegen.rs`
7. **Integration Tests**: Add tests in `crates/flux-wasm/tests/`

## Project Structure

```
fluxlang/
├── crates/
│   ├── flux-syntax/    # Lexer, parser, AST
│   ├── flux-errors/    # Error types and diagnostics
│   ├── flux-sema/      # Semantic analysis, type checking
│   ├── flux-wasm/      # WASM codegen and runtime
│   ├── flux-lsp/       # Language Server Protocol
│   └── flux-cli/       # Command-line interface
├── editors/
│   └── vscode/         # VS Code extension
├── examples/           # Example .flux programs
└── README.md
```

## Reporting Bugs

If you find a bug:

1. **Search existing issues** to see if it's already reported
2. **Create a new issue** with:
   - A clear, descriptive title
   - Steps to reproduce
   - Expected vs actual behavior
   - Your environment (OS, Rust version)
   - Minimal code example if applicable

## Security Vulnerabilities

Please see [SECURITY.md](SECURITY.md) for information on reporting security vulnerabilities.

## Questions?

If you have questions about contributing, feel free to:

- Open an issue for discussion
- Reach out to the maintainers

## License

By contributing to Flux, you agree that your contributions will be licensed under the MIT License.
