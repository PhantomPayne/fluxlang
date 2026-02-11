use flux_errors::{FluxError, Result};
use flux_syntax::{Expr, SourceFile, Type};
use std::collections::HashMap;
use wasm_encoder::{
    CodeSection, ExportKind, ExportSection, Function, FunctionSection, Instruction, Module,
    TypeSection, ValType,
};
use wit_component::ComponentEncoder;

/// Local variable context for tracking variable indices
struct LocalContext {
    /// Maps variable names to local indices
    locals: HashMap<String, u32>,
    /// Next available local index
    next_index: u32,
}

impl LocalContext {
    fn new() -> Self {
        Self {
            locals: HashMap::new(),
            next_index: 0,
        }
    }

    /// Add a parameter as a local (parameters come first)
    fn add_param(&mut self, name: &str) -> u32 {
        let idx = self.next_index;
        self.locals.insert(name.to_string(), idx);
        self.next_index += 1;
        idx
    }

    /// Add a local variable (allocated after parameters)
    fn add_local(&mut self, name: &str) -> u32 {
        let idx = self.next_index;
        self.locals.insert(name.to_string(), idx);
        self.next_index += 1;
        idx
    }

    /// Get the index of a local variable
    fn get(&self, name: &str) -> Option<u32> {
        self.locals.get(name).copied()
    }
}

/// WASM code generator for Flux
pub struct WasmCodegen {}

impl WasmCodegen {
    pub fn new() -> Self {
        Self {}
    }

    /// Compile a Flux source file to a WASM component
    pub fn compile_component(&mut self, ast: &SourceFile) -> Result<Vec<u8>> {
        // Generate the core module
        let core_wasm = self.compile_core_module(ast)?;

        // Create a component encoder
        let encoder = ComponentEncoder::default()
            .module(&core_wasm)
            .map_err(|e| FluxError::WasmError {
                message: format!("Failed to encode component: {}", e),
            })?
            .validate(true)
            .encode()
            .map_err(|e| FluxError::WasmError {
                message: format!("Failed to finalize component: {}", e),
            })?;

        Ok(encoder)
    }

    /// Compile the core WASM module
    fn compile_core_module(&mut self, ast: &SourceFile) -> Result<Vec<u8>> {
        let mut module = Module::new();

        // Create type section
        let mut types = TypeSection::new();
        // Function type: () -> i32
        types.ty().function(vec![], vec![ValType::I32]);
        module.section(&types);

        // Create function section
        let mut functions = FunctionSection::new();
        functions.function(0); // Function 0 uses type 0
        module.section(&functions);

        // Create export section
        let mut exports = ExportSection::new();
        exports.export("main", ExportKind::Func, 0);
        module.section(&exports);

        // Create code section
        let mut codes = CodeSection::new();

        // Generate code for the first function's body
        if let Some(flux_syntax::Item::Function(first_func)) = ast.items.first() {
            let mut locals_ctx = LocalContext::new();

            // Add parameters as locals
            for param in &first_func.params {
                locals_ctx.add_param(&param.name);
            }

            // Count additional locals needed for let bindings
            let additional_locals = self.count_let_bindings(&first_func.body);

            let func_locals = if additional_locals > 0 {
                vec![(additional_locals, ValType::I32)]
            } else {
                vec![]
            };

            let mut func = Function::new(func_locals);
            self.compile_expr_with_locals(&first_func.body, &mut locals_ctx, &mut func)?;
            func.instruction(&Instruction::End);
            codes.function(&func);
        } else {
            // Default: return 42
            let mut func = Function::new(vec![]);
            func.instruction(&Instruction::I32Const(42));
            func.instruction(&Instruction::End);
            codes.function(&func);
        }

        module.section(&codes);
        Ok(module.finish())
    }

    /// Count let bindings to determine how many locals we need
    fn count_let_bindings(&self, expr: &Expr) -> u32 {
        match expr {
            Expr::Let { value, body, .. } => {
                1 + self.count_let_bindings(value) + self.count_let_bindings(body)
            }
            Expr::Binary { left, right, .. } => {
                self.count_let_bindings(left) + self.count_let_bindings(right)
            }
            Expr::Call { func, args, .. } => {
                let mut count = self.count_let_bindings(func);
                for arg in args {
                    count += self.count_let_bindings(arg);
                }
                count
            }
            Expr::Block { stmts, .. } => stmts.iter().map(|s| self.count_let_bindings(s)).sum(),
            Expr::Return { value, .. } => self.count_let_bindings(value),
            _ => 0,
        }
    }

    /// Compile an expression with local variable context
    fn compile_expr_with_locals(
        &mut self,
        expr: &Expr,
        locals: &mut LocalContext,
        func: &mut Function,
    ) -> Result<()> {
        match expr {
            Expr::Int { value, .. } => {
                func.instruction(&Instruction::I32Const(*value as i32));
            }
            Expr::Float { value, .. } => {
                func.instruction(&Instruction::F64Const(*value));
            }
            Expr::Bool { value, .. } => {
                func.instruction(&Instruction::I32Const(if *value { 1 } else { 0 }));
            }
            Expr::String { .. } => {
                // Strings are not yet fully supported - return placeholder
                func.instruction(&Instruction::I32Const(0));
            }
            Expr::Var { name, .. } => {
                let local_idx = locals.get(name).ok_or_else(|| FluxError::WasmError {
                    message: format!("Undefined variable: {}", name),
                })?;
                func.instruction(&Instruction::LocalGet(local_idx));
            }
            Expr::Binary {
                op, left, right, ..
            } => {
                self.compile_expr_with_locals(left, locals, func)?;
                self.compile_expr_with_locals(right, locals, func)?;
                match op {
                    flux_syntax::BinOp::Add => {
                        func.instruction(&Instruction::I32Add);
                    }
                    flux_syntax::BinOp::Sub => {
                        func.instruction(&Instruction::I32Sub);
                    }
                    flux_syntax::BinOp::Mul => {
                        func.instruction(&Instruction::I32Mul);
                    }
                    flux_syntax::BinOp::Div => {
                        func.instruction(&Instruction::I32DivS);
                    }
                }
            }
            Expr::Let {
                name, value, body, ..
            } => {
                // Compile the value
                self.compile_expr_with_locals(value, locals, func)?;

                // Allocate a local and store
                let local_idx = locals.add_local(name);
                func.instruction(&Instruction::LocalSet(local_idx));

                // Compile the body
                self.compile_expr_with_locals(body, locals, func)?;
            }
            Expr::Return { value, .. } => {
                self.compile_expr_with_locals(value, locals, func)?;
                func.instruction(&Instruction::Return);
            }
            Expr::Block { stmts, .. } => {
                if let Some(last) = stmts.last() {
                    self.compile_expr_with_locals(last, locals, func)?;
                } else {
                    func.instruction(&Instruction::I32Const(0));
                }
            }
            Expr::Call { func: func_expr, args, .. } => {
                // For now, support only simple function calls with Var as the function name
                if let Expr::Var { name, .. } = func_expr.as_ref() {
                    self.compile_builtin_call(name, args, locals, func)?;
                } else {
                    return Err(FluxError::WasmError {
                        message: "Only direct function calls are supported (e.g., abs(x))".to_string(),
                    });
                }
            }
        }
        Ok(())
    }

    /// Compile a builtin function call
    fn compile_builtin_call(
        &mut self,
        name: &str,
        args: &[Expr],
        locals: &mut LocalContext,
        func: &mut Function,
    ) -> Result<()> {
        match name {
            // Integer math functions
            "abs" if args.len() == 1 => {
                // abs(x) implemented as: (x >= 0) ? x : (0 - x)
                self.compile_expr_with_locals(&args[0], locals, func)?;
                // Duplicate x on stack
                func.instruction(&Instruction::I32Const(0));
                self.compile_expr_with_locals(&args[0], locals, func)?;
                func.instruction(&Instruction::I32Sub);
                // Stack now has: [x, 0-x]
                self.compile_expr_with_locals(&args[0], locals, func)?;
                func.instruction(&Instruction::I32Const(0));
                func.instruction(&Instruction::I32GeS);
                // Stack: [x, 0-x, x>=0]
                func.instruction(&Instruction::Select);
                // Select returns first value if condition is true, else second
            }
            "max" if args.len() == 2 => {
                // max(a,b) = (a > b) ? a : b
                self.compile_expr_with_locals(&args[0], locals, func)?;
                self.compile_expr_with_locals(&args[1], locals, func)?;
                // Stack: [a, b]
                // Duplicate both for comparison
                self.compile_expr_with_locals(&args[0], locals, func)?;
                self.compile_expr_with_locals(&args[1], locals, func)?;
                func.instruction(&Instruction::I32GtS);
                // Stack: [a, b, a>b]
                func.instruction(&Instruction::Select);
            }
            "min" if args.len() == 2 => {
                // min(a,b) = (a < b) ? a : b
                self.compile_expr_with_locals(&args[0], locals, func)?;
                self.compile_expr_with_locals(&args[1], locals, func)?;
                // Stack: [a, b]
                self.compile_expr_with_locals(&args[0], locals, func)?;
                self.compile_expr_with_locals(&args[1], locals, func)?;
                func.instruction(&Instruction::I32LtS);
                // Stack: [a, b, a<b]
                func.instruction(&Instruction::Select);
            }
            // Float math functions
            "sqrt" if args.len() == 1 => {
                self.compile_expr_with_locals(&args[0], locals, func)?;
                func.instruction(&Instruction::F64Sqrt);
            }
            "floor" if args.len() == 1 => {
                self.compile_expr_with_locals(&args[0], locals, func)?;
                func.instruction(&Instruction::F64Floor);
            }
            "ceil" if args.len() == 1 => {
                self.compile_expr_with_locals(&args[0], locals, func)?;
                func.instruction(&Instruction::F64Ceil);
            }
            "pow" if args.len() == 2 => {
                // pow is not a single WASM instruction, so this is a placeholder
                // In a real implementation, we'd either:
                // 1. Import a math library function
                // 2. Implement pow using a loop
                // 3. Use WASM SIMD extensions if available
                // For now, we'll return an error
                return Err(FluxError::WasmError {
                    message: format!(
                        "Function '{}' requires stdlib support (not yet available as intrinsic)",
                        name
                    ),
                });
            }
            _ => {
                return Err(FluxError::WasmError {
                    message: format!("Unknown builtin function: '{}'", name),
                });
            }
        }
        Ok(())
    }

    /// Map a Flux type to the corresponding WIT type name
    ///
    /// This helper will be used when implementing the full WIT adapter layer
    /// for binding Flux functions to component exports with proper type mapping.
    /// Currently preserved as documentation of the type mapping strategy.
    #[allow(dead_code)]
    fn flux_type_to_wit_name(&self, ty: &Type) -> &'static str {
        match ty {
            Type::Int(_) => "s64",
            Type::Float(_) => "f64",
            Type::Bool(_) => "bool",
            Type::String(_) => "string",
            Type::Named { .. } => "named",
        }
    }
}

impl Default for WasmCodegen {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to compile Flux source to a WASM component
pub fn compile_to_component(source: &str) -> Result<Vec<u8>> {
    let ast = flux_syntax::parse(source)?;
    let mut codegen = WasmCodegen::new();
    codegen.compile_component(&ast)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_simple_function() {
        let source = "fn main() { return 42 }";
        let result = compile_to_component(source);
        assert!(result.is_ok());
        let wasm = result.unwrap();
        assert!(!wasm.is_empty());
    }

    #[test]
    fn test_compile_addition() {
        let source = "fn main() { return 10 + 32 }";
        let result = compile_to_component(source);
        assert!(result.is_ok());
    }
}
