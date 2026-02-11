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

        // Build function index map
        let mut func_index_map = HashMap::new();
        #[allow(irrefutable_let_patterns)]
        for (idx, item) in ast.items.iter().enumerate() {
            if let flux_syntax::Item::Function(func) = item {
                func_index_map.insert(func.name.clone(), idx as u32);
            }
        }

        // Create type section - register type signatures for all functions
        let mut types = TypeSection::new();
        let mut type_index_map: HashMap<String, u32> = HashMap::new();

        #[allow(irrefutable_let_patterns)]
        for item in &ast.items {
            if let flux_syntax::Item::Function(func) = item {
                let param_types: Vec<ValType> = func
                    .params
                    .iter()
                    .map(|param| {
                        param
                            .ty
                            .as_ref()
                            .map_or(ValType::I32, |ty| self.flux_type_to_valtype(ty))
                    })
                    .collect();

                let return_type = func
                    .return_type
                    .as_ref()
                    .map_or(ValType::I32, |ty| self.flux_type_to_valtype(ty));

                // Check if we already have this type signature
                let type_key = format!("{:?}->{:?}", param_types, return_type);
                let _type_idx = if let Some(&existing_idx) = type_index_map.get(&type_key) {
                    existing_idx
                } else {
                    let idx = types.len();
                    types.ty().function(param_types.clone(), vec![return_type]);
                    type_index_map.insert(type_key, idx);
                    idx
                };
            }
        }
        module.section(&types);

        // Create function section - register all functions
        let mut functions = FunctionSection::new();
        #[allow(irrefutable_let_patterns)]
        for item in &ast.items {
            if let flux_syntax::Item::Function(func) = item {
                let param_types: Vec<ValType> = func
                    .params
                    .iter()
                    .map(|param| {
                        param
                            .ty
                            .as_ref()
                            .map_or(ValType::I32, |ty| self.flux_type_to_valtype(ty))
                    })
                    .collect();

                let return_type = func
                    .return_type
                    .as_ref()
                    .map_or(ValType::I32, |ty| self.flux_type_to_valtype(ty));

                let type_key = format!("{:?}->{:?}", param_types, return_type);
                let type_idx = type_index_map[&type_key];
                functions.function(type_idx);
            }
        }
        module.section(&functions);

        // Create export section - export main function if it exists
        let mut exports = ExportSection::new();
        if let Some(&main_idx) = func_index_map.get("main") {
            exports.export("main", ExportKind::Func, main_idx);
        }
        module.section(&exports);

        // Create code section - compile all functions
        let mut codes = CodeSection::new();

        #[allow(irrefutable_let_patterns)]
        for item in &ast.items {
            if let flux_syntax::Item::Function(func_def) = item {
                let mut locals_ctx = LocalContext::new();

                // Add parameters as locals
                for param in &func_def.params {
                    locals_ctx.add_param(&param.name);
                }

                // Count additional locals needed for let bindings
                let additional_locals = self.count_let_bindings(&func_def.body);

                let func_locals = if additional_locals > 0 {
                    vec![(additional_locals, ValType::I32)]
                } else {
                    vec![]
                };

                let mut func = Function::new(func_locals);
                self.compile_expr_with_locals(
                    &func_def.body,
                    &mut locals_ctx,
                    &mut func,
                    &func_index_map,
                )?;
                func.instruction(&Instruction::End);
                codes.function(&func);
            }
        }

        module.section(&codes);
        Ok(module.finish())
    }

    /// Convert Flux type to WASM ValType
    fn flux_type_to_valtype(&self, ty: &Type) -> ValType {
        match ty {
            Type::Int(_) => ValType::I32,
            Type::Float(_) => ValType::F64,
            Type::Bool(_) => ValType::I32,
            // String and named types default to I32 for now
            Type::String(_) | Type::Named { .. } => ValType::I32,
        }
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
        func_index_map: &HashMap<String, u32>,
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
                self.compile_expr_with_locals(left, locals, func, func_index_map)?;
                self.compile_expr_with_locals(right, locals, func, func_index_map)?;
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
                self.compile_expr_with_locals(value, locals, func, func_index_map)?;

                // Allocate a local and store
                let local_idx = locals.add_local(name);
                func.instruction(&Instruction::LocalSet(local_idx));

                // Compile the body
                self.compile_expr_with_locals(body, locals, func, func_index_map)?;
            }
            Expr::Return { value, .. } => {
                self.compile_expr_with_locals(value, locals, func, func_index_map)?;
                func.instruction(&Instruction::Return);
            }
            Expr::Block { stmts, .. } => {
                if let Some(last) = stmts.last() {
                    self.compile_expr_with_locals(last, locals, func, func_index_map)?;
                } else {
                    func.instruction(&Instruction::I32Const(0));
                }
            }
            Expr::Call {
                func: call_func,
                args,
                ..
            } => {
                // Resolve function name
                let func_name = match call_func.as_ref() {
                    Expr::Var { name, .. } => name,
                    _ => {
                        return Err(FluxError::WasmError {
                            message: "Function calls must use a simple identifier".to_string(),
                        });
                    }
                };

                // Look up function index
                let func_idx =
                    func_index_map
                        .get(func_name)
                        .ok_or_else(|| FluxError::WasmError {
                            message: format!("Unknown function: {}", func_name),
                        })?;

                // Compile arguments (push them onto the stack)
                for arg in args {
                    self.compile_expr_with_locals(arg, locals, func, func_index_map)?;
                }

                // Emit call instruction
                func.instruction(&Instruction::Call(*func_idx));
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
