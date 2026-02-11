// Example Flux program demonstrating key features

import { Table, filter, map, reduce } from "std"

// Simple function with type annotations
fn add(x: int, y: int) -> int {
    x + y
}

// Pipeline operator example
fn process(data: Table<int>) -> int {
    data 
    |> filter(#active) 
    |> map(double) 
    |> reduce(add, 0)
}

// Helper function
fn double(x: int) -> int {
    x * 2
}

// Plan skeleton - entry point for Flux programs
export fn plan(ctx) -> Project {
    ctx
}
