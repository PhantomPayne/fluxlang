use flux_syntax::parse;

#[test]
fn test_parse_simple_function() {
    let input = r#"fn add(x: int, y: int) -> int { x + y }"#;
    let result = parse(input);
    insta::assert_debug_snapshot!(result);
}

#[test]
fn test_parse_pipeline_operator() {
    let input = r#"fn process(data) { data |> filter |> map |> reduce }"#;
    let result = parse(input);
    insta::assert_debug_snapshot!(result);
}

#[test]
fn test_parse_label_literals() {
    let input = r#"fn tagged() { #primary }"#;
    let result = parse(input);
    insta::assert_debug_snapshot!(result);
}

#[test]
fn test_parse_table_type() {
    let input = r#"fn process(data: Table<int>) -> Table<string> { data }"#;
    let result = parse(input);
    insta::assert_debug_snapshot!(result);
}

#[test]
fn test_parse_plan_skeleton() {
    let input = r#"export fn plan(ctx) -> Project { ctx }"#;
    let result = parse(input);
    insta::assert_debug_snapshot!(result);
}

#[test]
fn test_parse_import_statement() {
    let input = r#"import { Table } from "std""#;
    let result = parse(input);
    insta::assert_debug_snapshot!(result);
}

#[test]
fn test_parse_complex_pipeline() {
    let input = r#"
fn analyze(data: Table<int>) -> int {
    data |> filter(#active) |> sum
}
"#;
    let result = parse(input);
    insta::assert_debug_snapshot!(result);
}

#[test]
fn test_parse_nested_expressions() {
    let input = r#"fn calc() { (1 + 2) * (3 + 4) }"#;
    let result = parse(input);
    insta::assert_debug_snapshot!(result);
}
