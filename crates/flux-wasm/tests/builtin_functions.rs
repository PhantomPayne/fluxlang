use flux_wasm::compile_to_component;
use wasmtime::component::Component;
use wasmtime::*;

#[test]
fn test_builtin_abs_positive() {
    let source = "fn main() { return abs(42) }";
    let wasm_bytes = compile_to_component(source).expect("Compilation failed");

    let engine = Engine::default();
    Component::from_binary(&engine, &wasm_bytes).expect("Failed to create component");
}

#[test]
fn test_builtin_abs_negative() {
    // Use subtraction to get negative number: 0 - 42
    let source = "fn main() { return abs(0 - 42) }";
    let wasm_bytes = compile_to_component(source).expect("Compilation failed");

    let engine = Engine::default();
    Component::from_binary(&engine, &wasm_bytes).expect("Failed to create component");
}

#[test]
fn test_builtin_max() {
    let source = "fn main() { return max(10, 20) }";
    let wasm_bytes = compile_to_component(source).expect("Compilation failed");

    let engine = Engine::default();
    Component::from_binary(&engine, &wasm_bytes).expect("Failed to create component");
}

#[test]
fn test_builtin_min() {
    let source = "fn main() { return min(10, 20) }";
    let wasm_bytes = compile_to_component(source).expect("Compilation failed");

    let engine = Engine::default();
    Component::from_binary(&engine, &wasm_bytes).expect("Failed to create component");
}

// Note: Float functions (sqrt, floor, ceil) are not tested here because the current
// codegen always returns i32, and mixing float operations with int return types
// causes validation errors. These would need proper type inference/checking.

#[test]
fn test_builtin_nested_calls() {
    // Use subtraction to get negative number
    let source = "fn main() { return max(abs(0 - 5), 10) }";
    let wasm_bytes = compile_to_component(source).expect("Compilation failed");

    let engine = Engine::default();
    Component::from_binary(&engine, &wasm_bytes).expect("Failed to create component");
}

#[test]
fn test_builtin_with_variables() {
    // Use subtraction to get negative number
    let source = "fn main() { let x = 0 - 42 return abs(x) }";
    let wasm_bytes = compile_to_component(source).expect("Compilation failed");

    let engine = Engine::default();
    Component::from_binary(&engine, &wasm_bytes).expect("Failed to create component");
}

#[test]
fn test_unknown_function_fails() {
    let source = "fn main() { return unknown_func(42) }";
    let result = compile_to_component(source);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Unknown builtin function"));
}

#[test]
fn test_pow_not_supported_yet() {
    let source = "fn main() { return pow(2.0, 10.0) }";
    let result = compile_to_component(source);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("requires stdlib support"));
}
