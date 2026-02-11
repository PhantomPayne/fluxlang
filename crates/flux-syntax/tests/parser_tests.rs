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

#[test]
fn test_parse_temporal_types_date() {
    let input = r#"fn get_birth_date() -> Date { 0 }"#;
    let result = parse(input);
    insta::assert_debug_snapshot!(result);
}

#[test]
fn test_parse_temporal_types_time() {
    let input = r#"fn get_meeting_time() -> Time { 0 }"#;
    let result = parse(input);
    insta::assert_debug_snapshot!(result);
}

#[test]
fn test_parse_temporal_types_datetime() {
    let input = r#"fn schedule_event(local_time: DateTime) -> DateTime { local_time }"#;
    let result = parse(input);
    insta::assert_debug_snapshot!(result);
}

#[test]
fn test_parse_temporal_types_timestamp() {
    let input = r#"fn log_event() -> Timestamp { 0 }"#;
    let result = parse(input);
    insta::assert_debug_snapshot!(result);
}

#[test]
fn test_parse_temporal_types_duration() {
    let input = r#"fn calculate_elapsed(start: Timestamp, end: Timestamp) -> Duration { 0 }"#;
    let result = parse(input);
    insta::assert_debug_snapshot!(result);
}

#[test]
fn test_parse_temporal_types_table() {
    let input = r#"fn get_dates(dates: Table<Date>) -> Table<Timestamp> { dates }"#;
    let result = parse(input);
    insta::assert_debug_snapshot!(result);
}
