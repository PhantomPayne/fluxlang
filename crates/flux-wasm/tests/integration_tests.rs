use flux_wasm::compile_to_wasm;
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
