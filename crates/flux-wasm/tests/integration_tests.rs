use flux_wasm::{compile_to_component, compile_to_wasm};
use wasmtime::component::Component;
use wasmtime::*;

#[test]
fn test_wasm_execution_simple_value() {
    let source = "fn main() { 42 }";
    let wasm_bytes = compile_to_wasm(source).expect("Compilation failed");

    // Create wasmtime engine and module
    let engine = Engine::default();
    let module = Module::new(&engine, &wasm_bytes).expect("Failed to create module");

    // Create store and instance
    let mut store = Store::new(&engine, ());
    let instance = Instance::new(&mut store, &module, &[]).expect("Failed to create instance");

    // Get the exported function
    let main = instance
        .get_typed_func::<(), i32>(&mut store, "main")
        .expect("Failed to get main function");

    // Call the function
    let result = main.call(&mut store, ()).expect("Function call failed");

    assert_eq!(result, 42);
}

#[test]
fn test_wasm_execution_addition() {
    let source = "fn main() { 10 + 32 }";
    let wasm_bytes = compile_to_wasm(source).expect("Compilation failed");

    let engine = Engine::default();
    let module = Module::new(&engine, &wasm_bytes).expect("Failed to create module");

    let mut store = Store::new(&engine, ());
    let instance = Instance::new(&mut store, &module, &[]).expect("Failed to create instance");

    let main = instance
        .get_typed_func::<(), i32>(&mut store, "main")
        .expect("Failed to get main function");

    let result = main.call(&mut store, ()).expect("Function call failed");

    assert_eq!(result, 42);
}

#[test]
fn test_wasm_execution_subtraction() {
    let source = "fn main() { 50 - 8 }";
    let wasm_bytes = compile_to_wasm(source).expect("Compilation failed");

    let engine = Engine::default();
    let module = Module::new(&engine, &wasm_bytes).expect("Failed to create module");

    let mut store = Store::new(&engine, ());
    let instance = Instance::new(&mut store, &module, &[]).expect("Failed to create instance");

    let main = instance
        .get_typed_func::<(), i32>(&mut store, "main")
        .expect("Failed to get main function");

    let result = main.call(&mut store, ()).expect("Function call failed");

    assert_eq!(result, 42);
}

#[test]
fn test_wasm_execution_multiplication() {
    let source = "fn main() { 6 * 7 }";
    let wasm_bytes = compile_to_wasm(source).expect("Compilation failed");

    let engine = Engine::default();
    let module = Module::new(&engine, &wasm_bytes).expect("Failed to create module");

    let mut store = Store::new(&engine, ());
    let instance = Instance::new(&mut store, &module, &[]).expect("Failed to create instance");

    let main = instance
        .get_typed_func::<(), i32>(&mut store, "main")
        .expect("Failed to get main function");

    let result = main.call(&mut store, ()).expect("Function call failed");

    assert_eq!(result, 42);
}

#[test]
fn test_wasm_execution_complex_expression() {
    let source = "fn main() { (10 + 2) * 3 + 6 }";
    let wasm_bytes = compile_to_wasm(source).expect("Compilation failed");

    let engine = Engine::default();
    let module = Module::new(&engine, &wasm_bytes).expect("Failed to create module");

    let mut store = Store::new(&engine, ());
    let instance = Instance::new(&mut store, &module, &[]).expect("Failed to create instance");

    let main = instance
        .get_typed_func::<(), i32>(&mut store, "main")
        .expect("Failed to get main function");

    let result = main.call(&mut store, ()).expect("Function call failed");

    assert_eq!(result, 42);
}

#[test]
fn test_wasm_execution_bool_true() {
    let source = "fn main() { true }";
    let wasm_bytes = compile_to_wasm(source).expect("Compilation failed");

    let engine = Engine::default();
    let module = Module::new(&engine, &wasm_bytes).expect("Failed to create module");

    let mut store = Store::new(&engine, ());
    let instance = Instance::new(&mut store, &module, &[]).expect("Failed to create instance");

    let main = instance
        .get_typed_func::<(), i32>(&mut store, "main")
        .expect("Failed to get main function");

    let result = main.call(&mut store, ()).expect("Function call failed");

    assert_eq!(result, 1);
}

#[test]
fn test_wasm_execution_bool_false() {
    let source = "fn main() { false }";
    let wasm_bytes = compile_to_wasm(source).expect("Compilation failed");

    let engine = Engine::default();
    let module = Module::new(&engine, &wasm_bytes).expect("Failed to create module");

    let mut store = Store::new(&engine, ());
    let instance = Instance::new(&mut store, &module, &[]).expect("Failed to create instance");

    let main = instance
        .get_typed_func::<(), i32>(&mut store, "main")
        .expect("Failed to get main function");

    let result = main.call(&mut store, ()).expect("Function call failed");

    assert_eq!(result, 0);
}

// Component Model Tests

#[test]
fn test_component_compilation_simple() {
    // Test that we can compile to a component successfully
    let source = "fn main() { 42 }";
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
    let source = "fn main() { 10 + 32 }";
    let component_bytes = compile_to_component(source).expect("Component compilation failed");

    let engine = Engine::default();
    Component::from_binary(&engine, &component_bytes).expect("Failed to create component");
}

#[test]
fn test_component_vs_module_size() {
    // Components should be larger than core modules due to metadata
    let source = "fn main() { 42 }";

    let module_bytes = compile_to_wasm(source).expect("Module compilation failed");
    let component_bytes = compile_to_component(source).expect("Component compilation failed");

    // Component should be larger than raw module
    assert!(component_bytes.len() > module_bytes.len());
}
