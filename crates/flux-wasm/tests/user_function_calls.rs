use flux_wasm::compile_to_component;
use wasmtime::component::Component;
use wasmtime::*;

#[test]
fn test_user_defined_function_call() {
    let source = r#"
        fn add_ten(x) { return x + 10 }
        fn main() { return add_ten(5) }
    "#;
    let wasm_bytes = compile_to_component(source).expect("Compilation failed");

    let engine = Engine::default();
    Component::from_binary(&engine, &wasm_bytes).expect("Failed to create component");
}

#[test]
fn test_multiple_user_defined_functions() {
    let source = r#"
        fn double(x) { return x * 2 }
        fn triple(x) { return x * 3 }
        fn main() { return double(5) + triple(3) }
    "#;
    let wasm_bytes = compile_to_component(source).expect("Compilation failed");

    let engine = Engine::default();
    Component::from_binary(&engine, &wasm_bytes).expect("Failed to create component");
}

#[test]
fn test_nested_user_function_calls() {
    let source = r#"
        fn add_five(x) { return x + 5 }
        fn double(x) { return x * 2 }
        fn main() { return double(add_five(10)) }
    "#;
    let wasm_bytes = compile_to_component(source).expect("Compilation failed");

    let engine = Engine::default();
    Component::from_binary(&engine, &wasm_bytes).expect("Failed to create component");
}

#[test]
fn test_builtin_and_user_function_mix() {
    let source = r#"
        fn add_five(x) { return x + 5 }
        fn main() { return max(add_five(10), 20) }
    "#;
    let wasm_bytes = compile_to_component(source).expect("Compilation failed");

    let engine = Engine::default();
    Component::from_binary(&engine, &wasm_bytes).expect("Failed to create component");
}

#[test]
fn test_wrong_argument_count_builtin() {
    let source = "fn main() { return abs(1, 2) }";
    let result = compile_to_component(source);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("expects 1 argument"));
}

#[test]
fn test_wrong_argument_count_user_function() {
    let source = r#"
        fn add_five(x) { return x + 5 }
        fn main() { return add_five(1, 2) }
    "#;
    let result = compile_to_component(source);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("expects 1 argument"));
}
