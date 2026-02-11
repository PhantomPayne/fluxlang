use flux_errors::{FluxError, Result};
use flux_syntax::{Expr, Item, SourceFile, Type};
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

/// Signature of a builtin function
#[allow(dead_code)]
struct BuiltinSignature {
    /// Number of parameters
    param_count: usize,
    /// Expected parameter types (for validation when type checking is available)
    param_types: Vec<ValType>,
    /// Return type
    return_type: ValType,
}

/// Registry of builtin functions
struct BuiltinRegistry {
    signatures: HashMap<String, BuiltinSignature>,
}

impl BuiltinRegistry {
    fn new() -> Self {
        let mut signatures = HashMap::new();

        // Integer math functions
        signatures.insert(
            "abs".to_string(),
            BuiltinSignature {
                param_count: 1,
                param_types: vec![ValType::I32],
                return_type: ValType::I32,
            },
        );
        signatures.insert(
            "max".to_string(),
            BuiltinSignature {
                param_count: 2,
                param_types: vec![ValType::I32, ValType::I32],
                return_type: ValType::I32,
            },
        );
        signatures.insert(
            "min".to_string(),
            BuiltinSignature {
                param_count: 2,
                param_types: vec![ValType::I32, ValType::I32],
                return_type: ValType::I32,
            },
        );

        // Float math functions
        signatures.insert(
            "sqrt".to_string(),
            BuiltinSignature {
                param_count: 1,
                param_types: vec![ValType::F64],
                return_type: ValType::F64,
            },
        );
        signatures.insert(
            "floor".to_string(),
            BuiltinSignature {
                param_count: 1,
                param_types: vec![ValType::F64],
                return_type: ValType::F64,
            },
        );
        signatures.insert(
            "ceil".to_string(),
            BuiltinSignature {
                param_count: 1,
                param_types: vec![ValType::F64],
                return_type: ValType::F64,
            },
        );
        signatures.insert(
            "pow".to_string(),
            BuiltinSignature {
                param_count: 2,
                param_types: vec![ValType::F64, ValType::F64],
                return_type: ValType::F64,
            },
        );

        Self { signatures }
    }

    fn get(&self, name: &str) -> Option<&BuiltinSignature> {
        self.signatures.get(name)
    }

    fn is_builtin(&self, name: &str) -> bool {
        self.signatures.contains_key(name)
    }
}

/// Information about a user-defined function
struct UserFunctionInfo {
    /// WASM function index
    wasm_index: u32,
    /// Number of parameters
    param_count: usize,
}

/// WASM code generator for Flux
pub struct WasmCodegen {
    /// Registry of builtin functions
    builtin_registry: BuiltinRegistry,
    /// Map of user-defined functions
    user_functions: HashMap<String, UserFunctionInfo>,
}

impl WasmCodegen {
    pub fn new() -> Self {
        Self {
            builtin_registry: BuiltinRegistry::new(),
            user_functions: HashMap::new(),
        }
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

        // First pass: collect all user-defined functions and assign WASM indices
        let mut wasm_func_index = 0u32;
        let mut function_signatures = Vec::new(); // Track signatures for each function

        for item in &ast.items {
            let Item::Function(func) = item;
            self.user_functions.insert(
                func.name.clone(),
                UserFunctionInfo {
                    wasm_index: wasm_func_index,
                    param_count: func.params.len(),
                },
            );
            // For now, all params are i32 and all returns are i32
            function_signatures.push(func.params.len());
            wasm_func_index += 1;
        }

        // Create type section - generate unique type signatures as needed
        let mut types = TypeSection::new();
        let mut type_indices: HashMap<usize, u32> = HashMap::new();

        if wasm_func_index > 0 {
            // Generate unique type signatures for user-defined functions
            for &param_count in &function_signatures {
                if !type_indices.contains_key(&param_count) {
                    let params = vec![ValType::I32; param_count];
                    let type_idx = type_indices.len() as u32;
                    types.ty().function(params, vec![ValType::I32]);
                    type_indices.insert(param_count, type_idx);
                }
            }
        } else {
            // No user functions - add a default () -> i32 signature
            types.ty().function(vec![], vec![ValType::I32]);
            type_indices.insert(0, 0);
        }
        module.section(&types);

        // Create function section - declare all functions with proper type indices
        let mut functions = FunctionSection::new();
        if wasm_func_index > 0 {
            for &param_count in &function_signatures {
                let type_idx = type_indices[&param_count];
                functions.function(type_idx);
            }
        } else {
            // No user functions - add default function
            functions.function(0);
        }
        module.section(&functions);

        // Create export section - export main if it exists
        let mut exports = ExportSection::new();
        if let Some(main_info) = self.user_functions.get("main") {
            exports.export("main", ExportKind::Func, main_info.wasm_index);
        }
        module.section(&exports);

        // Create code section - generate code for all functions
        let mut codes = CodeSection::new();

        if wasm_func_index > 0 {
            // Generate code for user-defined functions
            for item in &ast.items {
                let Item::Function(func) = item;
                let mut locals_ctx = LocalContext::new();

                // Add parameters as locals
                for param in &func.params {
                    locals_ctx.add_param(&param.name);
                }

                // Count additional locals needed for let bindings
                let additional_locals = self.count_let_bindings(&func.body);

                let func_locals = if additional_locals > 0 {
                    vec![(additional_locals, ValType::I32)]
                } else {
                    vec![]
                };

                let mut wasm_func = Function::new(func_locals);
                self.compile_expr_with_locals(&func.body, &mut locals_ctx, &mut wasm_func)?;
                wasm_func.instruction(&Instruction::End);
                codes.function(&wasm_func);
            }
        } else {
            // No functions defined - add a default function that returns 42
            let mut wasm_func = Function::new(vec![]);
            wasm_func.instruction(&Instruction::I32Const(42));
            wasm_func.instruction(&Instruction::End);
            codes.function(&wasm_func);
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
            Expr::Call {
                func: func_expr,
                args,
                ..
            } => {
                // Support only simple function calls with Var as the function name
                if let Expr::Var { name, .. } = func_expr.as_ref() {
                    // Check if it's a builtin function
                    if self.builtin_registry.is_builtin(name) {
                        self.compile_builtin_call(name, args, locals, func)?;
                    } else if self.user_functions.contains_key(name) {
                        // User-defined function call
                        self.compile_user_function_call(name, args, locals, func)?;
                    } else {
                        return Err(FluxError::WasmError {
                            message: format!("Unknown function: '{}'", name),
                        });
                    }
                } else {
                    return Err(FluxError::WasmError {
                        message: "Only direct function calls are supported (e.g., abs(x))"
                            .to_string(),
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
        // Get the builtin signature and validate argument count
        let signature = self
            .builtin_registry
            .get(name)
            .ok_or_else(|| FluxError::WasmError {
                message: format!("Unknown builtin function: '{}'", name),
            })?;

        // Validate argument count
        if args.len() != signature.param_count {
            return Err(FluxError::WasmError {
                message: format!(
                    "Function '{}' expects {} argument(s), but {} were provided",
                    name,
                    signature.param_count,
                    args.len()
                ),
            });
        }

        // Compile the function based on its name
        // This dispatches to the actual implementation
        match name {
            "abs" => self.compile_abs(args, locals, func),
            "max" => self.compile_max(args, locals, func),
            "min" => self.compile_min(args, locals, func),
            "sqrt" => self.compile_sqrt(args, locals, func),
            "floor" => self.compile_floor(args, locals, func),
            "ceil" => self.compile_ceil(args, locals, func),
            "pow" => Err(FluxError::WasmError {
                message: format!(
                    "Function '{}' requires stdlib support (not yet available as intrinsic)",
                    name
                ),
            }),
            _ => Err(FluxError::WasmError {
                message: format!("Builtin function '{}' is not implemented", name),
            }),
        }
    }

    /// Compile a user-defined function call
    fn compile_user_function_call(
        &mut self,
        name: &str,
        args: &[Expr],
        locals: &mut LocalContext,
        func: &mut Function,
    ) -> Result<()> {
        // Get function info and extract what we need before the loop
        let (wasm_index, param_count) = {
            let func_info = self
                .user_functions
                .get(name)
                .ok_or_else(|| FluxError::WasmError {
                    message: format!("Unknown function: '{}'", name),
                })?;
            (func_info.wasm_index, func_info.param_count)
        };

        // Validate argument count
        if args.len() != param_count {
            return Err(FluxError::WasmError {
                message: format!(
                    "Function '{}' expects {} argument(s), but {} were provided",
                    name,
                    param_count,
                    args.len()
                ),
            });
        }

        // Compile all arguments (they'll be pushed onto the stack)
        for arg in args {
            self.compile_expr_with_locals(arg, locals, func)?;
        }

        // Emit a call instruction to the user-defined function
        func.instruction(&Instruction::Call(wasm_index));

        Ok(())
    }

    // Individual builtin implementations

    fn compile_abs(
        &mut self,
        args: &[Expr],
        locals: &mut LocalContext,
        func: &mut Function,
    ) -> Result<()> {
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
        Ok(())
    }

    fn compile_max(
        &mut self,
        args: &[Expr],
        locals: &mut LocalContext,
        func: &mut Function,
    ) -> Result<()> {
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
        Ok(())
    }

    fn compile_min(
        &mut self,
        args: &[Expr],
        locals: &mut LocalContext,
        func: &mut Function,
    ) -> Result<()> {
        // min(a,b) = (a < b) ? a : b
        self.compile_expr_with_locals(&args[0], locals, func)?;
        self.compile_expr_with_locals(&args[1], locals, func)?;
        // Stack: [a, b]
        self.compile_expr_with_locals(&args[0], locals, func)?;
        self.compile_expr_with_locals(&args[1], locals, func)?;
        func.instruction(&Instruction::I32LtS);
        // Stack: [a, b, a<b]
        func.instruction(&Instruction::Select);
        Ok(())
    }

    fn compile_sqrt(
        &mut self,
        args: &[Expr],
        locals: &mut LocalContext,
        func: &mut Function,
    ) -> Result<()> {
        self.compile_expr_with_locals(&args[0], locals, func)?;
        func.instruction(&Instruction::F64Sqrt);
        Ok(())
    }

    fn compile_floor(
        &mut self,
        args: &[Expr],
        locals: &mut LocalContext,
        func: &mut Function,
    ) -> Result<()> {
        self.compile_expr_with_locals(&args[0], locals, func)?;
        func.instruction(&Instruction::F64Floor);
        Ok(())
    }

    fn compile_ceil(
        &mut self,
        args: &[Expr],
        locals: &mut LocalContext,
        func: &mut Function,
    ) -> Result<()> {
        self.compile_expr_with_locals(&args[0], locals, func)?;
        func.instruction(&Instruction::F64Ceil);
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
