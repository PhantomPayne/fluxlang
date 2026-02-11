use flux_errors::{FluxError, Result};
use flux_syntax::{Expr, SourceFile};
use wasm_encoder::{
    CodeSection, ExportKind, ExportSection, Function, FunctionSection, Instruction, Module,
    TypeSection, ValType,
};

/// WASM code generator for Flux
pub struct WasmCodegen {}

impl WasmCodegen {
    pub fn new() -> Self {
        Self {}
    }

    /// Compile a Flux source file to WASM
    pub fn compile(&mut self, ast: &SourceFile) -> Result<Vec<u8>> {
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
        let mut func = Function::new(vec![]);

        // Generate code for the first function's body (simplified)
        if let Some(flux_syntax::Item::Function(first_func)) = ast.items.first() {
            self.compile_expr(&first_func.body, &mut func)?;
        } else {
            // Default: return 42
            func.instruction(&Instruction::I32Const(42));
        }

        func.instruction(&Instruction::End);
        codes.function(&func);
        module.section(&codes);

        Ok(module.finish())
    }

    fn compile_expr(&mut self, expr: &Expr, func: &mut Function) -> Result<()> {
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
            Expr::Binary {
                op, left, right, ..
            } => {
                self.compile_expr(left, func)?;
                self.compile_expr(right, func)?;
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
                    _ => {
                        return Err(FluxError::WasmError {
                            message: "Unsupported binary operation".to_string(),
                        });
                    }
                }
            }
            Expr::Block { stmts, .. } => {
                if let Some(last) = stmts.last() {
                    self.compile_expr(last, func)?;
                } else {
                    func.instruction(&Instruction::I32Const(0));
                }
            }
            _ => {
                // Unsupported expression - return 0
                func.instruction(&Instruction::I32Const(0));
            }
        }
        Ok(())
    }
}

impl Default for WasmCodegen {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to compile Flux source to WASM
pub fn compile_to_wasm(source: &str) -> Result<Vec<u8>> {
    let ast = flux_syntax::parse(source)?;
    let mut codegen = WasmCodegen::new();
    codegen.compile(&ast)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_simple_function() {
        let source = "fn main() { 42 }";
        let result = compile_to_wasm(source);
        assert!(result.is_ok());
        let wasm = result.unwrap();
        assert!(!wasm.is_empty());
    }

    #[test]
    fn test_compile_addition() {
        let source = "fn main() { 10 + 32 }";
        let result = compile_to_wasm(source);
        assert!(result.is_ok());
    }
}
