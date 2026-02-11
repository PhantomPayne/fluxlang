# Flux Standard Library Implementation

This document describes the current state of the standard library implementation for Fluxlang.

## What Has Been Implemented

### 1. Function Call Support in Codegen

The Flux compiler now supports function calls! Previously, `Expr::Call` nodes would cause a compilation error. The codegen has been updated to handle function calls properly.

Location: `crates/flux-wasm/src/codegen.rs`

#### Builtin Math Functions

The following builtin math functions are now available as intrinsics (compiled directly to WASM instructions):

- **`abs(x)`** - Returns the absolute value of an integer
  - Implementation: Uses WASM `select` instruction with conditional logic
  - Example: `abs(0 - 42)` returns `42`

- **`max(a, b)`** - Returns the maximum of two integers
  - Implementation: Uses WASM `select` with comparison
  - Example: `max(10, 20)` returns `20`

- **`min(a, b)`** - Returns the minimum of two integers
  - Implementation: Uses WASM `select` with comparison
  - Example: `min(10, 20)` returns `10`

#### Float Functions (Declared but Not Fully Functional)

The following float functions are declared in the WIT interface but not currently usable in the main codegen due to type system limitations:

- `sqrt(x)` - Square root (uses `F64Sqrt`)
- `floor(x)` - Floor function (uses `F64Floor`)
- `ceil(x)` - Ceiling function (uses `F64Ceil`)

These work at the WASM level but require proper type inference to be used in practice, as the current codegen assumes all functions return `i32`.

### 2. WIT Interface Definitions

Basic WIT interface definitions have been created for the standard library in `wit/`:

- **`stdlib-math.wit`** - Mathematical functions
- **`stdlib-io.wit`** - I/O functions (println, eprintln, etc.)
- **`stdlib-temporal.wit`** - Temporal/time functions (now, timestamps, durations)
- **`stdlib-world.wit`** - World definition that exports all interfaces

These serve as documentation and a foundation for future component-based stdlib implementation.

### 3. Comprehensive Test Suite

New test file: `crates/flux-wasm/tests/builtin_functions.rs`

Tests cover:
- Basic function calls (`abs`, `max`, `min`)
- Nested function calls (`max(abs(0 - 5), 10)`)
- Function calls with variables
- Error handling for unknown functions
- Error handling for not-yet-implemented functions

**Test Results**: All 20 tests passing (10 existing + 8 new + 2 error cases)

## Current Limitations

### 1. Type System

The current codegen always assumes functions return `i32`. This means:
- Float operations work but can't be used as return values
- Mixed int/float expressions cause validation errors
- Proper type inference is needed to support all function types

### 2. No True Stdlib Component Yet

The original goal of creating a separate WASM component for the stdlib proved more complex than initially estimated. The current implementation uses intrinsics (inline WASM code) instead.

Future work could:
- Create a proper stdlib component using cargo-component
- Implement composition with wasm-tools
- Add proper import/export mechanisms

### 3. Limited Function Set

Only basic math functions are implemented:
- `pow(x, y)` returns an error message saying it needs stdlib support
- I/O functions (println, etc.) are not yet implemented
- Temporal functions are not yet implemented

### 4. No Negative Literals

The parser doesn't support negative number literals like `-42`. Workaround: use subtraction like `0 - 42`.

## Examples

### Basic Function Call
```flux
fn main() {
    return abs(42)
}
```

### Nested Function Calls
```flux
fn main() {
    return max(abs(0 - 5), 10)
}
```

### Function Calls with Variables
```flux
fn main() {
    let x = 0 - 42
    return abs(x)
}
```

## Future Work

### Short Term
1. Add type inference to support float functions properly
2. Implement I/O builtin functions (println, etc.)
3. Add support for negative number literals in the parser

### Long Term
1. Create a proper stdlib WASM component
2. Implement composition pipeline with wasm-tools or wasm-compose
3. Add more stdlib functions (string operations, collections, etc.)
4. Implement `pow()` and other complex math functions
5. Add proper imports for WASI interfaces

## Testing

Run all tests:
```bash
cargo test --all
```

Run only builtin function tests:
```bash
cargo test -p flux-wasm test_builtin
```

## References

- WIT definitions: `wit/stdlib-*.wit`
- Codegen implementation: `crates/flux-wasm/src/codegen.rs` (lines 241-318)
- Tests: `crates/flux-wasm/tests/builtin_functions.rs`
