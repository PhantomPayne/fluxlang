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
pub struct TypeChecker {}

impl TypeChecker {
    pub fn new() -> Self {
        Self {}
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
            Expr::Var { name, span } => env.get(name).cloned().ok_or_else(|| {
                flux_errors::FluxError::UnknownIdentifier {
                    name: name.clone(),
                    span: span.to_source_span(),
                }
            }),
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
                name,
                value,
                body,
                ..
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

    /// Type check function calls (placeholder)
    fn check_call(
        &self,
        _func: &flux_syntax::Expr,
        _args: &[flux_syntax::Expr],
        _env: &TypeEnv,
        _span: flux_errors::Span,
    ) -> flux_errors::Result<TypeInfo> {
        // For now, assume function calls return Unknown
        // Full implementation would look up function type and check arguments
        Ok(TypeInfo::Unknown)
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
            value: 3.14,
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
}

