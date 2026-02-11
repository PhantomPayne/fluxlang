# Flux Language

Flux is a functional language that targets WebAssembly Components. This is currently a **minimal boilerplate** implementation with a small but complete feature set.

## Current Status

This is a clean, minimal implementation focused on core functionality. Features are fully implemented and type-safe, not placeholders.

## Supported Features

### Types
- `int` - 64-bit signed integers (i64)
- `float` - 64-bit floating point (f64)
- `bool` - Boolean values
- `string` - String literals (basic support)

### Expressions
- **Literals**: `42`, `3.14`, `true`, `false`, `"hello"`
- **Variables**: `x`, `my_var`
- **Binary operators**: `+`, `-`, `*`, `/` (type-checked, no mixing int and float)
- **Let bindings**: `let x = 42 return x + 10`
- **Blocks**: `{ let x = 1 return x }`
- **Return statements**: `return expr` (explicit returns required)

### Declarations
- **Functions**: `fn name(param: type) -> type { return expr }`
- **Function parameters**: Must have type annotations
- **Return types**: Must be explicitly declared
- **Exported functions**: `export fn name() -> type { return expr }`

### Type System
- **Type inference**: For let bindings
- **Type checking**: Binary operations enforce same numeric type
- **Type errors**: Clear error messages for type mismatches

## Examples

### Simple Function
```flux
fn main() -> int {
    return (5 + 3) * 2
}
```

### Let Bindings
```flux
fn calculate() -> int {
    let x = 10
    let y = 32
    return x + y
}
```

### Type Error (won't compile)
```flux
fn bad_add(x: int, y: float) -> float {
    return x + y  // ERROR: Cannot mix int and float
}
```

## Project Structure

This is a Rust workspace containing multiple crates:

- `flux-syntax`: Lexer, parser, and AST definitions
- `flux-errors`: Error handling with beautiful diagnostics (using miette)
- `flux-sema`: Semantic analysis, type checking, symbol table, and VFS
- `flux-wasm`: WebAssembly Component code generation
- `flux-lsp`: Language Server Protocol implementation
- `flux-cli`: Command-line interface for parsing, checking, and compiling

## Getting Started

### Prerequisites

- Rust 1.70+ (install from [rustup.rs](https://rustup.rs))

### Building

```bash
# Build all crates
cargo build

# Build in release mode
cargo build --release

# Run tests
cargo test

# Format code
cargo fmt --all

# Check for warnings
cargo clippy --all-targets -- -D warnings
```

## Using the CLI

```bash
# Build the CLI
cargo build --release --bin flux

# Check a Flux program
./target/release/flux check examples/simple.flux

# Compile to WASM component
./target/release/flux compile examples/simple.flux output.wasm

# Parse and display AST
./target/release/flux parse examples/simple.flux
```

## WebAssembly Component Model

Flux compiles to the [WebAssembly Component Model](https://github.com/WebAssembly/component-model), providing:

- **Type Safety**: Components with well-defined interfaces
- **Interoperability**: Components can be used from any language
- **Standards-based**: Uses WIT (WebAssembly Interface Types)

The compiled components can be executed with:
```bash
wasmtime output.wasm
```

## Type Checking

Flux enforces type safety at compile time:

```flux
// ✓ Valid: same types
fn add_ints(x: int, y: int) -> int {
    return x + y
}

// ✗ Type error: cannot mix int and float
fn bad_add(x: int, y: float) -> float {
    return x + y  // ERROR
}

// ✓ Valid: let binding with type inference
fn with_let() -> int {
    let x = 10      // inferred as int
    let y = 32      // inferred as int
    return x + y    // valid: both int
}
```

## Testing

### Run All Tests
```bash
cargo test
```

### Test Individual Crates
```bash
cargo test -p flux-syntax
cargo test -p flux-sema
cargo test -p flux-wasm
```

### Snapshot Tests
```bash
# Update snapshots after parser changes
INSTA_UPDATE=always cargo test -p flux-syntax
```

## Architecture

### Compiler Pipeline
1. **Lexer** (`flux-syntax`): Source → Tokens
2. **Parser** (`flux-syntax`): Tokens → AST
3. **Type Checker** (`flux-sema`): AST → Type-checked AST + Errors
4. **Code Generator** (`flux-wasm`): AST → WASM Component

### Error Handling
All errors use `miette` for beautiful, helpful error messages with:
- Source code snippets
- Error locations highlighted
- Helpful suggestions

Example error output:
```
✗ Type error: Cannot apply Add to int and float.
  Both operands must be the same numeric type.
  ╭─[example.flux:3:12]
3 │     return x + y
  ·            ─────┬────
  ·                 ╰─── here
  ╰────
```

## Language Server (LSP)

Basic LSP support is available:

```bash
# Build the LSP server
cargo build --release --bin flux-lsp

# The binary will be at: target/release/flux-lsp
```

Features:
- Syntax error diagnostics
- Basic parsing support

## What's NOT Included

To keep this a clean, minimal boilerplate, the following are **not** implemented:

- Temporal types (Date, Time, DateTime, Timestamp, Duration)
- Pipeline operator (`|>`)
- Labels (`#label`)
- If expressions
- Comparison operators (`<`, `>`)
- Import statements
- Function calls (placeholder error)
- Standard library

These can be added incrementally as needed.

## Development

### Adding New Features

1. **Lexer**: Add tokens in `crates/flux-syntax/src/lexer.rs`
2. **AST**: Add node types in `crates/flux-syntax/src/ast.rs`
3. **Parser**: Add parsing logic in `crates/flux-syntax/src/parser.rs`
4. **Type Checking**: Update `crates/flux-sema/src/types.rs`
5. **Code Generation**: Update `crates/flux-wasm/src/codegen.rs`
6. **Tests**: Add tests at each level

### Code Quality

Before committing:
```bash
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test
```

## License

MIT License - see LICENSE file for details

## Contributing

Contributions are welcome! Please ensure:
- All tests pass (`cargo test`)
- Code is formatted (`cargo fmt`)
- No clippy warnings (`cargo clippy`)
- New features include tests
