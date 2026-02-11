# Flux Language

Flux is a functional, columnar-first language that targets WebAssembly (WASM).

## Features

- **Pipeline Operator (`|>`)**: Chain operations in a readable, left-to-right flow
- **Structural Labels (`#label`)**: Metadata and annotations on types and functions
- **Strong Type System**: Built-in types including `int`, `float`, `bool`, `string`, and comprehensive temporal types
- **Temporal Types**: First-class support for `Date`, `Time`, `DateTime`, `Timestamp`, and `Duration`
- **WASM Target**: Compile directly to WebAssembly for portable execution
- **LSP Support**: Full language server with hover, completion, and diagnostics

## Project Structure

This is a Rust workspace containing multiple crates:

- `flux-syntax`: Lexer, parser, and AST definitions
- `flux-errors`: Error handling with beautiful diagnostics (using miette)
- `flux-sema`: Semantic analysis, type checking, and VFS (Virtual File System)
- `flux-wasm`: WebAssembly code generation and wasmtime integration
- `flux-lsp`: Language Server Protocol implementation

## Getting Started

### Prerequisites

- Rust 1.70+ (install from [rustup.rs](https://rustup.rs))
- Node.js 18+ (for VS Code extension)

### Building

```bash
# Build all crates
cargo build

# Build in release mode
cargo build --release

# Run tests
cargo test

# Run tests with snapshot updates
cargo test -- --test-threads=1
```

### Running the LSP Server

```bash
# Build the LSP server
cargo build --release --bin flux-lsp

# The binary will be at: target/release/flux-lsp
```

### VS Code Extension

The VS Code extension provides syntax highlighting and LSP integration:

```bash
cd editors/vscode

# Install dependencies
npm install

# Compile TypeScript
npm run compile

# Run tests
npm test
```

To use the extension in VS Code:
1. Open the `editors/vscode` folder in VS Code
2. Press F5 to launch the extension in a new VS Code window
3. Create a `.flux` file to see syntax highlighting
4. Set `FLUX_LSP_PATH` environment variable to the flux-lsp binary path for full LSP features

## Language Syntax

### Function Definition

```flux
fn add(x: int, y: int) -> int {
    x + y
}

export fn plan(ctx) -> Project {
    ctx
}
```

### Pipeline Operations

```flux
fn process(value: int) -> int {
    value |> filter(#active) |> sum
}
```

### Imports

```flux
import { Date, Time, filter, map } from "std"
```

### Basic Types

```flux
fn example_types() {
    let x: int = 42
    let y: float = 3.14
    let flag: bool = true
    let name: string = "Flux"
}
```

### Labels

```flux
fn tagged() {
    #primary
    #secondary_label
}
```

### Temporal Types

Flux includes comprehensive temporal types for robust time handling:

```flux
// Calendar date only (YYYY-MM-DD)
fn get_birth_date() -> Date {
    return Date(1990, 6, 15)
}

// Time of day only (HH:mm:ss)
fn get_meeting_time() -> Time {
    return Time(14, 30, 0)
}

// Date + time + timezone (always with timezone)
fn schedule_event(local_time: DateTime) -> DateTime {
    return local_time
}

// Absolute UTC time for events/logs
fn log_event() -> Timestamp {
    return now()
}

// Unified duration supporting all time units
fn calculate_elapsed(start: Timestamp, end: Timestamp) -> Duration {
    return end - start
}
```

**Temporal Type Selection Guide:**
- Use `Date` for dates without time-of-day (birthdays, anniversaries)
- Use `Time` for time-of-day without date (schedules, recurring times)
- Use `DateTime` for user-facing times with timezone (event scheduling, display)
- Use `Timestamp` for absolute time points (event logs, causality, ordering)
- Use `Duration` for time intervals (elapsed time, time arithmetic)

## Testing

The project includes comprehensive testing at multiple levels:

### Unit Tests

Each crate has unit tests. Run with:
```bash
cargo test
```

### Snapshot Tests

The `flux-syntax` crate uses `insta` for snapshot testing:
```bash
cd crates/flux-syntax
cargo test
cargo insta review  # Review snapshot changes
```

### Integration Tests (WASM)

The `flux-wasm` crate includes integration tests that compile Flux code to WASM and execute it with wasmtime:
```bash
cd crates/flux-wasm
cargo test
```

## Architecture

### Virtual File System (VFS)

The VFS in `flux-sema` manages both disk-based files and in-memory "unsaved" buffers from the editor. This allows the LSP to work with unsaved changes without hitting the disk.

### Standard Library

The standard library is embedded as virtual files in the compiler, allowing imports like `import { Date, Time } from "std"` to work even without physical files.

### Symbol Bridge

The Symbol Bridge connects LSP queries to semantic information. When hovering over a variable, the LSP queries the flux-sema crate to find the specific Span and Type for that coordinate.

## Development

### Adding New Features

1. Update the lexer in `flux-syntax/src/lexer.rs` for new tokens
2. Update the parser in `flux-syntax/src/parser.rs` for new syntax
3. Add snapshot tests in `flux-syntax/tests/`
4. Update WASM codegen in `flux-wasm/src/codegen.rs`
5. Add integration tests in `flux-wasm/tests/`

### Error Messages

Flux uses `miette` for beautiful, helpful error messages with code snippets and hints. Errors are defined in `flux-errors/src/lib.rs` and can be automatically converted to LSP Diagnostic objects.

## License

MIT License - see LICENSE file for details

## Contributing

Contributions are welcome! Please ensure:
- All tests pass (`cargo test`)
- New features include tests
- Code follows Rust conventions (`cargo fmt`, `cargo clippy`)
