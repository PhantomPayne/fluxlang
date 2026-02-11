use miette::{IntoDiagnostic, Result};
use std::env;
use std::fs;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        return Ok(());
    }

    match args[1].as_str() {
        "parse" => {
            if args.len() < 3 {
                eprintln!("Usage: flux parse <file.flux>");
                return Ok(());
            }
            parse_file(&args[2])?;
        }
        "compile" => {
            if args.len() < 3 {
                eprintln!("Usage: flux compile <file.flux> [output.wasm] [--core]");
                return Ok(());
            }
            let output = if args.len() > 3 && !args[3].starts_with("--") {
                &args[3]
            } else {
                "output.wasm"
            };
            let use_core = args.contains(&"--core".to_string());
            compile_file(&args[2], output, use_core)?;
        }
        "check" => {
            if args.len() < 3 {
                eprintln!("Usage: flux check <file.flux>");
                return Ok(());
            }
            check_file(&args[2])?;
        }
        "--version" | "-v" => {
            println!("flux 0.1.0");
        }
        "--help" | "-h" => {
            print_usage();
        }
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            print_usage();
        }
    }

    Ok(())
}

fn print_usage() {
    println!(
        r#"Flux - A functional, columnar-first language

Usage:
    flux <command> [options]

Commands:
    parse <file.flux>              Parse and display AST
    compile <file.flux> [out.wasm] Compile to WebAssembly Component
            [--core]               Use --core for legacy core module format
    check <file.flux>              Check syntax without compilation
    --version, -v                  Show version
    --help, -h                     Show this help

Examples:
    flux parse examples/plan.flux
    flux compile examples/plan.flux output.wasm
    flux compile examples/plan.flux output.wasm --core
    flux check examples/plan.flux
"#
    );
}

fn parse_file(path: &str) -> Result<()> {
    let content = fs::read_to_string(path).into_diagnostic()?;

    match flux_syntax::parse(&content) {
        Ok(ast) => {
            println!("✓ Successfully parsed {}", path);
            println!("\nAST:");
            println!("{:#?}", ast);
            Ok(())
        }
        Err(e) => {
            eprintln!("✗ Parse error:");
            Err(e).into_diagnostic()
        }
    }
}

fn compile_file(input_path: &str, output_path: &str, use_core: bool) -> Result<()> {
    let content = fs::read_to_string(input_path).into_diagnostic()?;

    let result = if use_core {
        flux_wasm::compile_to_wasm(&content)
    } else {
        flux_wasm::compile_to_component(&content)
    };

    match result {
        Ok(wasm) => {
            fs::write(output_path, &wasm).into_diagnostic()?;
            let format = if use_core {
                "core module"
            } else {
                "component"
            };
            println!(
                "✓ Successfully compiled {} to {} ({})",
                input_path, output_path, format
            );
            println!("  WASM size: {} bytes", wasm.len());
            Ok(())
        }
        Err(e) => {
            eprintln!("✗ Compilation error:");
            Err(e).into_diagnostic()
        }
    }
}

fn check_file(path: &str) -> Result<()> {
    let content = fs::read_to_string(path).into_diagnostic()?;

    match flux_syntax::parse(&content) {
        Ok(ast) => {
            println!("✓ {} is valid", path);
            println!("  {} items found", ast.items.len());

            // List functions
            for item in &ast.items {
                match item {
                    flux_syntax::Item::Function(func) => {
                        let export_marker = if func.is_export { "export " } else { "" };
                        println!("  - {}fn {}", export_marker, func.name);
                    }
                    flux_syntax::Item::Import(import) => {
                        println!(
                            "  - import {{ {} }} from \"{}\"",
                            import.items.join(", "),
                            import.module
                        );
                    }
                }
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("✗ {} contains errors:", path);
            Err(e).into_diagnostic()
        }
    }
}
