use std::collections::HashMap;
use std::fmt;

/// Type information for Flux types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeInfo {
    Int,
    String,
    Bool,
    Float,
    Named {
        name: String,
    },
    Function {
        params: Vec<TypeInfo>,
        ret: Box<TypeInfo>,
    },
    Unknown,
}

impl fmt::Display for TypeInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeInfo::Int => write!(f, "int"),
            TypeInfo::String => write!(f, "string"),
            TypeInfo::Bool => write!(f, "bool"),
            TypeInfo::Float => write!(f, "float"),
            TypeInfo::Named { name } => write!(f, "{}", name),
            TypeInfo::Function { params, ret } => {
                write!(f, "(")?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", param)?;
                }
                write!(f, ") -> {}", ret)
            }
            TypeInfo::Unknown => write!(f, "?"),
        }
    }
}

/// Type environment mapping variable names to their types
#[derive(Debug, Clone)]
pub struct TypeEnv {
    bindings: HashMap<String, TypeInfo>,
}

impl TypeEnv {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
        }
    }

    pub fn insert(&mut self, name: String, ty: TypeInfo) {
        self.bindings.insert(name, ty);
    }

    pub fn get(&self, name: &str) -> Option<&TypeInfo> {
        self.bindings.get(name)
    }
}

impl Default for TypeEnv {
    fn default() -> Self {
        Self::new()
    }
}

/// Type checker for Flux
pub struct TypeChecker {
    /// Function registry mapping function names to their types
    functions: HashMap<String, TypeInfo>,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
        }
    }

    /// Create a new type checker with a function registry
    pub fn with_functions(functions: HashMap<String, TypeInfo>) -> Self {
        Self { functions }
    }

    /// Build function registry from AST
    pub fn build_function_registry(ast: &flux_syntax::SourceFile) -> HashMap<String, TypeInfo> {
        let mut functions = HashMap::new();

        #[allow(irrefutable_let_patterns)]
        for item in &ast.items {
            if let flux_syntax::Item::Function(func) = item {
                let params = func
                    .params
                    .iter()
                    .map(|param| {
                        param
                            .ty
                            .as_ref()
                            .map_or(TypeInfo::Unknown, Self::type_from_ast)
                    })
                    .collect();

                let ret = func
                    .return_type
                    .as_ref()
                    .map_or(TypeInfo::Unknown, Self::type_from_ast);

                functions.insert(
                    func.name.clone(),
                    TypeInfo::Function {
                        params,
                        ret: Box::new(ret),
                    },
                );
            }
        }

        functions
    }

    /// Convert AST type to TypeInfo
    fn type_from_ast(ty: &flux_syntax::Type) -> TypeInfo {
        match ty {
            flux_syntax::Type::Int(_) => TypeInfo::Int,
            flux_syntax::Type::String(_) => TypeInfo::String,
            flux_syntax::Type::Bool(_) => TypeInfo::Bool,
            flux_syntax::Type::Float(_) => TypeInfo::Float,
            flux_syntax::Type::Named { name, .. } => TypeInfo::Named { name: name.clone() },
        }
    }

    /// Infer the type of an expression given an environment
    pub fn infer_expr(
        &self,
        expr: &flux_syntax::Expr,
        env: &TypeEnv,
    ) -> flux_errors::Result<TypeInfo> {
        use flux_syntax::Expr;
        match expr {
            Expr::Int { .. } => Ok(TypeInfo::Int),
            Expr::Float { .. } => Ok(TypeInfo::Float),
            Expr::Bool { .. } => Ok(TypeInfo::Bool),
            Expr::String { .. } => Ok(TypeInfo::String),
            Expr::Var { name, span } => {
                env.get(name)
                    .cloned()
                    .ok_or_else(|| flux_errors::FluxError::UnknownIdentifier {
                        name: name.clone(),
                        span: span.to_source_span(),
                    })
            }
            Expr::Binary {
                op,
                left,
                right,
                span,
            } => {
                let left_ty = self.infer_expr(left, env)?;
                let right_ty = self.infer_expr(right, env)?;
                self.check_binary_op(*op, left_ty, right_ty, *span)
            }
            Expr::Let {
                name, value, body, ..
            } => {
                let value_ty = self.infer_expr(value, env)?;
                let mut new_env = env.clone();
                new_env.insert(name.clone(), value_ty);
                self.infer_expr(body, &new_env)
            }
            Expr::Call { func, args, span } => self.check_call(func, args, env, *span),
            Expr::Block { stmts, .. } => self.infer_block(stmts, env),
            Expr::Return { value, .. } => self.infer_expr(value, env),
        }
    }

    /// Check binary operation types
    fn check_binary_op(
        &self,
        op: flux_syntax::BinOp,
        left: TypeInfo,
        right: TypeInfo,
        span: flux_errors::Span,
    ) -> flux_errors::Result<TypeInfo> {
        use flux_syntax::BinOp;
        match (op, &left, &right) {
            // Arithmetic ops: both operands must be same numeric type
            (BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div, TypeInfo::Int, TypeInfo::Int) => {
                Ok(TypeInfo::Int)
            }
            (
                BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div,
                TypeInfo::Float,
                TypeInfo::Float,
            ) => Ok(TypeInfo::Float),
            (BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div, _, _) => {
                Err(flux_errors::FluxError::TypeError {
                    message: format!(
                        "Cannot apply {:?} to {} and {}. Both operands must be the same numeric type.",
                        op, left, right
                    ),
                    span: span.to_source_span(),
                })
            }
        }
    }

    /// Type check function calls
    fn check_call(
        &self,
        func: &flux_syntax::Expr,
        args: &[flux_syntax::Expr],
        env: &TypeEnv,
        span: flux_errors::Span,
    ) -> flux_errors::Result<TypeInfo> {
        // Resolve function name
        let func_name = match func {
            flux_syntax::Expr::Var { name, .. } => name,
            _ => {
                return Err(flux_errors::FluxError::TypeError {
                    message: "Function calls must use a simple identifier".to_string(),
                    span: span.to_source_span(),
                });
            }
        };

        // Look up function type in registry
        let func_type = self.functions.get(func_name).ok_or_else(|| {
            flux_errors::FluxError::UnknownIdentifier {
                name: func_name.clone(),
                span: span.to_source_span(),
            }
        })?;

        // Extract parameters and return type
        let (params, ret) = match func_type {
            TypeInfo::Function { params, ret } => (params, ret),
            _ => {
                return Err(flux_errors::FluxError::TypeError {
                    message: format!("{} is not a function", func_name),
                    span: span.to_source_span(),
                });
            }
        };

        // Check arity
        if args.len() != params.len() {
            return Err(flux_errors::FluxError::TypeError {
                message: format!(
                    "Function {} expects {} argument(s), but {} were provided",
                    func_name,
                    params.len(),
                    args.len()
                ),
                span: span.to_source_span(),
            });
        }

        // Check argument types
        for (i, (arg, expected_type)) in args.iter().zip(params.iter()).enumerate() {
            let arg_type = self.infer_expr(arg, env)?;

            // Allow Unknown types to pass (for untyped parameters)
            if arg_type != *expected_type
                && arg_type != TypeInfo::Unknown
                && *expected_type != TypeInfo::Unknown
            {
                return Err(flux_errors::FluxError::TypeError {
                    message: format!(
                        "Argument {} to function {}: expected type {}, but got {}",
                        i + 1,
                        func_name,
                        expected_type,
                        arg_type
                    ),
                    span: arg.span().to_source_span(),
                });
            }
        }

        // Return the function's return type
        Ok((**ret).clone())
    }

    /// Type check blocks
    fn infer_block(
        &self,
        stmts: &[flux_syntax::Expr],
        env: &TypeEnv,
    ) -> flux_errors::Result<TypeInfo> {
        if let Some(last) = stmts.last() {
            self.infer_expr(last, env)
        } else {
            Ok(TypeInfo::Int) // Empty block returns default
        }
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_type_info_display() {
        // Test all TypeInfo variants for completeness
        assert_eq!(TypeInfo::Int.to_string(), "int");
        assert_eq!(TypeInfo::String.to_string(), "string");
        assert_eq!(TypeInfo::Bool.to_string(), "bool");
        assert_eq!(TypeInfo::Float.to_string(), "float");
        assert_eq!(
            TypeInfo::Named {
                name: "MyType".to_string()
            }
            .to_string(),
            "MyType"
        );
        assert_eq!(TypeInfo::Unknown.to_string(), "?");
    }

    #[test]
    fn test_function_with_bool_float() {
        let func_type = TypeInfo::Function {
            params: vec![TypeInfo::Bool, TypeInfo::Float],
            ret: Box::new(TypeInfo::Int),
        };
        assert_eq!(func_type.to_string(), "(bool, float) -> int");
    }

    #[test]
    fn test_type_env_insert_and_get() {
        let mut env = TypeEnv::new();
        env.insert("x".to_string(), TypeInfo::Int);
        assert_eq!(env.get("x"), Some(&TypeInfo::Int));
        assert_eq!(env.get("y"), None);
    }

    #[test]
    fn test_type_checker_infer_literals() {
        let checker = TypeChecker::new();
        let env = TypeEnv::new();

        let int_expr = flux_syntax::Expr::Int {
            value: 42,
            span: flux_errors::Span::new(0, 2),
        };
        assert_eq!(checker.infer_expr(&int_expr, &env).unwrap(), TypeInfo::Int);

        let float_expr = flux_syntax::Expr::Float {
            value: 1.414,
            span: flux_errors::Span::new(0, 4),
        };
        assert_eq!(
            checker.infer_expr(&float_expr, &env).unwrap(),
            TypeInfo::Float
        );

        let bool_expr = flux_syntax::Expr::Bool {
            value: true,
            span: flux_errors::Span::new(0, 4),
        };
        assert_eq!(
            checker.infer_expr(&bool_expr, &env).unwrap(),
            TypeInfo::Bool
        );
    }

    #[test]
    fn test_type_error_int_plus_float() {
        let checker = TypeChecker::new();
        let env = TypeEnv::new();

        let left = flux_syntax::Expr::Int {
            value: 10,
            span: flux_errors::Span::new(0, 2),
        };
        let right = flux_syntax::Expr::Float {
            value: 3.5,
            span: flux_errors::Span::new(5, 8),
        };
        let binary = flux_syntax::Expr::Binary {
            op: flux_syntax::BinOp::Add,
            left: Box::new(left),
            right: Box::new(right),
            span: flux_errors::Span::new(0, 8),
        };

        let result = checker.infer_expr(&binary, &env);
        assert!(result.is_err());
        match result.unwrap_err() {
            flux_errors::FluxError::TypeError { message, .. } => {
                assert!(message.contains("Cannot apply"));
            }
            _ => panic!("Expected TypeError"),
        }
    }

    #[test]
    fn test_type_check_valid_addition() {
        let checker = TypeChecker::new();
        let env = TypeEnv::new();

        let left = flux_syntax::Expr::Int {
            value: 10,
            span: flux_errors::Span::new(0, 2),
        };
        let right = flux_syntax::Expr::Int {
            value: 32,
            span: flux_errors::Span::new(5, 7),
        };
        let binary = flux_syntax::Expr::Binary {
            op: flux_syntax::BinOp::Add,
            left: Box::new(left),
            right: Box::new(right),
            span: flux_errors::Span::new(0, 7),
        };

        let result = checker.infer_expr(&binary, &env);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), TypeInfo::Int);
    }

    #[test]
    fn test_type_check_let_binding() {
        let checker = TypeChecker::new();
        let env = TypeEnv::new();

        let value = flux_syntax::Expr::Int {
            value: 42,
            span: flux_errors::Span::new(8, 10),
        };
        let body_var = flux_syntax::Expr::Var {
            name: "x".to_string(),
            span: flux_errors::Span::new(11, 12),
        };
        let let_expr = flux_syntax::Expr::Let {
            name: "x".to_string(),
            value: Box::new(value),
            body: Box::new(body_var),
            span: flux_errors::Span::new(0, 12),
        };

        let result = checker.infer_expr(&let_expr, &env);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), TypeInfo::Int);
    }

    #[test]
    fn test_build_function_registry() {
        let source = r#"
            fn add(x: int, y: int) -> int { return x + y }
            fn greet(name: string) -> string { return "Hello" }
        "#;
        let ast = flux_syntax::parse(source).unwrap();
        let registry = TypeChecker::build_function_registry(&ast);

        assert_eq!(registry.len(), 2);

        let add_type = registry.get("add").unwrap();
        assert!(matches!(add_type, TypeInfo::Function { .. }));
        if let TypeInfo::Function { params, ret } = add_type {
            assert_eq!(params.len(), 2);
            assert_eq!(params[0], TypeInfo::Int);
            assert_eq!(params[1], TypeInfo::Int);
            assert_eq!(**ret, TypeInfo::Int);
        }

        let greet_type = registry.get("greet").unwrap();
        if let TypeInfo::Function { params, ret } = greet_type {
            assert_eq!(params.len(), 1);
            assert_eq!(params[0], TypeInfo::String);
            assert_eq!(**ret, TypeInfo::String);
        }
    }

    #[test]
    fn test_function_call_valid() {
        let source = "fn add(x: int, y: int) -> int { return x + y }";
        let ast = flux_syntax::parse(source).unwrap();
        let registry = TypeChecker::build_function_registry(&ast);
        let checker = TypeChecker::with_functions(registry);
        let env = TypeEnv::new();

        // Build call: add(1, 2)
        let func = flux_syntax::Expr::Var {
            name: "add".to_string(),
            span: flux_errors::Span::new(0, 3),
        };
        let arg1 = flux_syntax::Expr::Int {
            value: 1,
            span: flux_errors::Span::new(4, 5),
        };
        let arg2 = flux_syntax::Expr::Int {
            value: 2,
            span: flux_errors::Span::new(7, 8),
        };
        let call = flux_syntax::Expr::Call {
            func: Box::new(func),
            args: vec![arg1, arg2],
            span: flux_errors::Span::new(0, 9),
        };

        let result = checker.infer_expr(&call, &env);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), TypeInfo::Int);
    }

    #[test]
    fn test_function_call_wrong_arity() {
        let source = "fn add(x: int, y: int) -> int { return x + y }";
        let ast = flux_syntax::parse(source).unwrap();
        let registry = TypeChecker::build_function_registry(&ast);
        let checker = TypeChecker::with_functions(registry);
        let env = TypeEnv::new();

        // Build call: add(1) - missing argument
        let func = flux_syntax::Expr::Var {
            name: "add".to_string(),
            span: flux_errors::Span::new(0, 3),
        };
        let arg1 = flux_syntax::Expr::Int {
            value: 1,
            span: flux_errors::Span::new(4, 5),
        };
        let call = flux_syntax::Expr::Call {
            func: Box::new(func),
            args: vec![arg1],
            span: flux_errors::Span::new(0, 6),
        };

        let result = checker.infer_expr(&call, &env);
        assert!(result.is_err());
        match result.unwrap_err() {
            flux_errors::FluxError::TypeError { message, .. } => {
                assert!(message.contains("expects 2 argument"));
                assert!(message.contains("but 1 were provided"));
            }
            _ => panic!("Expected TypeError for arity mismatch"),
        }
    }

    #[test]
    fn test_function_call_wrong_type() {
        let source = "fn add(x: int, y: int) -> int { return x + y }";
        let ast = flux_syntax::parse(source).unwrap();
        let registry = TypeChecker::build_function_registry(&ast);
        let checker = TypeChecker::with_functions(registry);
        let env = TypeEnv::new();

        // Build call: add(1, 3.14) - wrong type for second argument
        let func = flux_syntax::Expr::Var {
            name: "add".to_string(),
            span: flux_errors::Span::new(0, 3),
        };
        let arg1 = flux_syntax::Expr::Int {
            value: 1,
            span: flux_errors::Span::new(4, 5),
        };
        let arg2 = flux_syntax::Expr::Float {
            value: 3.5,
            span: flux_errors::Span::new(7, 11),
        };
        let call = flux_syntax::Expr::Call {
            func: Box::new(func),
            args: vec![arg1, arg2],
            span: flux_errors::Span::new(0, 12),
        };

        let result = checker.infer_expr(&call, &env);
        assert!(result.is_err());
        match result.unwrap_err() {
            flux_errors::FluxError::TypeError { message, .. } => {
                assert!(message.contains("expected type int"));
                assert!(message.contains("but got float"));
            }
            _ => panic!("Expected TypeError for type mismatch"),
        }
    }

    #[test]
    fn test_function_call_unknown_function() {
        let checker = TypeChecker::new();
        let env = TypeEnv::new();

        // Build call to unknown function: foo()
        let func = flux_syntax::Expr::Var {
            name: "foo".to_string(),
            span: flux_errors::Span::new(0, 3),
        };
        let call = flux_syntax::Expr::Call {
            func: Box::new(func),
            args: vec![],
            span: flux_errors::Span::new(0, 5),
        };

        let result = checker.infer_expr(&call, &env);
        assert!(result.is_err());
        match result.unwrap_err() {
            flux_errors::FluxError::UnknownIdentifier { name, .. } => {
                assert_eq!(name, "foo");
            }
            _ => panic!("Expected UnknownIdentifier error"),
        }
    }
}
