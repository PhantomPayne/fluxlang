use flux_syntax::parse;

#[test]
fn test_parse_simple_function() {
    let input = r#"fn add(x: int, y: int) -> int { return x + y }"#;
    let result = parse(input);
    insta::assert_debug_snapshot!(result);
}

#[test]
fn test_parse_bool_float_types() {
    let input = r#"fn process(flag: bool, value: float) -> int { return 42 }"#;
    let result = parse(input);
    insta::assert_debug_snapshot!(result);
}

#[test]
fn test_parse_plan_skeleton() {
    let input = r#"export fn plan(ctx) -> Project { return ctx }"#;
    let result = parse(input);
    insta::assert_debug_snapshot!(result);
}

#[test]
fn test_parse_nested_expressions() {
    let input = r#"fn calc() { return (1 + 2) * (3 + 4) }"#;
    let result = parse(input);
    insta::assert_debug_snapshot!(result);
}

#[test]
fn test_parse_bool_float_literals() {
    let input = r#"fn test() { return true } fn test2() { return false } fn test3() { return 3.14 }"#;
    let result = parse(input);
    insta::assert_debug_snapshot!(result);
}

#[test]
fn test_parse_let_binding() {
    let input = r#"fn test() -> int { let x = 10 return x + 32 }"#;
    let result = parse(input);
    insta::assert_debug_snapshot!(result);
}

#[test]
fn test_parse_function_call() {
    let input = r#"fn main() -> int { return add(1, 2) }"#;
    let result = parse(input);
    insta::assert_debug_snapshot!(result);
}

#[test]
fn test_parse_block() {
    let input = r#"fn test() -> int { { return 42 } }"#;
    let result = parse(input);
    insta::assert_debug_snapshot!(result);
}
