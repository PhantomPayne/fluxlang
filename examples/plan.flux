// Example Flux program demonstrating key features

// Simple function with type annotations
fn add(x: int, y: int) -> int {
    x + y
}

// Function with bool and float
fn calculate(flag: bool, value: float) -> int {
    42
}

// Temporal type example
fn get_current_time() -> Timestamp {
    0
}

// Pipeline operator example
fn process(value: int) -> int {
    value 
    |> filter(#active) 
    |> map(double) 
    |> reduce(add, 0)
}

// Helper functions
fn double(x: int) -> int {
    x * 2
}

fn filter(predicate) {
    // native implementation
    0
}

fn map(f) {
    // native implementation
    0
}

fn reduce(f, init) {
    // native implementation
    0
}

// Plan skeleton - entry point for Flux programs
export fn plan(ctx) -> Project {
    ctx
}
