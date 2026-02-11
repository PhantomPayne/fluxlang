use flux_wasm::compile_to_component;
use wasmtime::component::Component;
use wasmtime::*;

#[test]
fn test_wasm_execution_simple_value() {
    let source = "fn main() { return 42 }";
    let wasm_bytes = compile_to_component(source).expect("Compilation failed");

    // Verify it's a valid component by loading it with wasmtime
    let engine = Engine::default();
    Component::from_binary(&engine, &wasm_bytes).expect("Failed to create component");

    // Component bytes were generated
    assert!(!wasm_bytes.is_empty());
}

#[test]
fn test_wasm_execution_addition() {
    let source = "fn main() { return 10 + 32 }";
    let wasm_bytes = compile_to_component(source).expect("Compilation failed");

    let engine = Engine::default();
    Component::from_binary(&engine, &wasm_bytes).expect("Failed to create component");
}

#[test]
fn test_wasm_execution_subtraction() {
    let source = "fn main() { return 50 - 8 }";
    let wasm_bytes = compile_to_component(source).expect("Compilation failed");

    let engine = Engine::default();
    Component::from_binary(&engine, &wasm_bytes).expect("Failed to create component");
}

#[test]
fn test_wasm_execution_multiplication() {
    let source = "fn main() { return 6 * 7 }";
    let wasm_bytes = compile_to_component(source).expect("Compilation failed");

    let engine = Engine::default();
    Component::from_binary(&engine, &wasm_bytes).expect("Failed to create component");
}

#[test]
fn test_wasm_execution_complex_expression() {
    let source = "fn main() { return (10 + 2) * 3 + 6 }";
    let wasm_bytes = compile_to_component(source).expect("Compilation failed");

    let engine = Engine::default();
    Component::from_binary(&engine, &wasm_bytes).expect("Failed to create component");
}

#[test]
fn test_wasm_execution_bool_true() {
    let source = "fn main() { return true }";
    let wasm_bytes = compile_to_component(source).expect("Compilation failed");

    let engine = Engine::default();
    Component::from_binary(&engine, &wasm_bytes).expect("Failed to create component");
}

#[test]
fn test_wasm_execution_bool_false() {
    let source = "fn main() { return false }";
    let wasm_bytes = compile_to_component(source).expect("Compilation failed");

    let engine = Engine::default();
    Component::from_binary(&engine, &wasm_bytes).expect("Failed to create component");
}

#[test]
fn test_wasm_execution_with_let() {
    let source = "fn main() { let x = 10 let y = 32 return x + y }";
    let wasm_bytes = compile_to_component(source).expect("Compilation failed");

    let engine = Engine::default();
    Component::from_binary(&engine, &wasm_bytes).expect("Failed to create component");
}

// Component Model Tests

#[test]
fn test_component_compilation_simple() {
    // Test that we can compile to a component successfully
    let source = "fn main() { return 42 }";
    let component_bytes = compile_to_component(source).expect("Component compilation failed");

    // Verify it's a valid component by loading it with wasmtime
    let engine = Engine::default();
    Component::from_binary(&engine, &component_bytes).expect("Failed to create component");

    // Verify component bytes were generated
    assert!(!component_bytes.is_empty());
}

#[test]
fn test_component_compilation_with_operations() {
    // Test component compilation with arithmetic operations
    let source = "fn main() { return 10 + 32 }";
    let component_bytes = compile_to_component(source).expect("Component compilation failed");

    let engine = Engine::default();
    Component::from_binary(&engine, &component_bytes).expect("Failed to create component");
}
