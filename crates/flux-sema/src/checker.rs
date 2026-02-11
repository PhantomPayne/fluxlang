use flux_errors::FluxError;
use flux_syntax::{Expr, Function, Item, SourceFile};
use crate::SymbolTable;
use std::collections::HashSet;

/// Check a source file for semantic errors
pub fn check_semantics(ast: &SourceFile, symbol_table: &SymbolTable, file_id: crate::FileId) -> Vec<FluxError> {
    let mut checker = SemanticChecker::new(symbol_table, file_id);
    checker.check_source_file(ast);
    checker.errors
}

struct SemanticChecker<'a> {
    #[allow(dead_code)]
    symbol_table: &'a SymbolTable,
    #[allow(dead_code)]
    file_id: crate::FileId,
    errors: Vec<FluxError>,
    defined_names: HashSet<String>,
}

impl<'a> SemanticChecker<'a> {
    fn new(symbol_table: &'a SymbolTable, file_id: crate::FileId) -> Self {
        // Collect all defined symbols from the symbol table
        let symbols = symbol_table.get_symbols(file_id);
        let defined_names = symbols.iter().map(|s| s.name.clone()).collect();

        Self {
            symbol_table,
            file_id,
            errors: Vec::new(),
            defined_names,
        }
    }

    fn check_source_file(&mut self, source_file: &SourceFile) {
        for item in &source_file.items {
            match item {
                Item::Function(func) => self.check_function(func),
                Item::Import(_) => {
                    // Imports don't need semantic checking for now
                }
            }
        }
    }

    fn check_function(&mut self, func: &Function) {
        // Add parameters to the scope for this function
        let mut local_scope = self.defined_names.clone();
        for param in &func.params {
            local_scope.insert(param.name.clone());
        }

        // Check the function body with the local scope
        self.check_expr_with_scope(&func.body, &local_scope);
    }

    fn check_expr_with_scope(&mut self, expr: &Expr, scope: &HashSet<String>) {
        match expr {
            Expr::Var { name, span } => {
                if !scope.contains(name) {
                    self.errors.push(FluxError::UnknownIdentifier {
                        name: name.clone(),
                        span: span.to_source_span(),
                    });
                }
            }
            Expr::Call { func, args, .. } => {
                // Check the function expression
                self.check_expr_with_scope(func, scope);
                
                // Check all arguments
                for arg in args {
                    self.check_expr_with_scope(arg, scope);
                }
            }
            Expr::Binary { left, right, .. } => {
                self.check_expr_with_scope(left, scope);
                self.check_expr_with_scope(right, scope);
            }
            Expr::Pipeline { left, right, .. } => {
                self.check_expr_with_scope(left, scope);
                self.check_expr_with_scope(right, scope);
            }
            Expr::Let { name, value, body, .. } => {
                // Check the value expression with current scope
                self.check_expr_with_scope(value, scope);
                
                // Create a new scope with the let-bound variable
                let mut new_scope = scope.clone();
                new_scope.insert(name.clone());
                
                // Check the body with the extended scope
                self.check_expr_with_scope(body, &new_scope);
            }
            Expr::If { cond, then_branch, else_branch, .. } => {
                self.check_expr_with_scope(cond, scope);
                self.check_expr_with_scope(then_branch, scope);
                if let Some(else_branch) = else_branch {
                    self.check_expr_with_scope(else_branch, scope);
                }
            }
            Expr::Block { stmts, .. } => {
                // Check each statement in the block
                for stmt in stmts {
                    self.check_expr_with_scope(stmt, scope);
                }
            }
            // Literals don't need checking
            Expr::Int { .. } | Expr::String { .. } | Expr::Label { .. } => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flux_syntax::parse;
    use crate::{SymbolBridge, FileId};

    #[test]
    fn test_undefined_variable_detected() {
        let source = r#"
            fn test() -> int {
                unknown_var
            }
        "#;

        let ast = parse(source).unwrap();
        let file_id = FileId(1);
        
        let symbol_bridge = SymbolBridge::new();
        symbol_bridge.analyze_file(file_id, &ast);
        
        let symbol_table = symbol_bridge.symbol_table();
        let errors = check_semantics(&ast, symbol_table, file_id);

        assert_eq!(errors.len(), 1);
        match &errors[0] {
            FluxError::UnknownIdentifier { name, .. } => {
                assert_eq!(name, "unknown_var");
            }
            _ => panic!("Expected UnknownIdentifier error"),
        }
    }

    #[test]
    fn test_defined_parameter_not_error() {
        let source = r#"
            fn test(data: int) -> int {
                data
            }
        "#;

        let ast = parse(source).unwrap();
        let file_id = FileId(1);
        
        let symbol_bridge = SymbolBridge::new();
        symbol_bridge.analyze_file(file_id, &ast);
        
        let symbol_table = symbol_bridge.symbol_table();
        let errors = check_semantics(&ast, symbol_table, file_id);

        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_multiple_undefined_variables() {
        let source = r#"
            fn test() -> int {
                unknown1 + unknown2
            }
        "#;

        let ast = parse(source).unwrap();
        let file_id = FileId(1);
        
        let symbol_bridge = SymbolBridge::new();
        symbol_bridge.analyze_file(file_id, &ast);
        
        let symbol_table = symbol_bridge.symbol_table();
        let errors = check_semantics(&ast, symbol_table, file_id);

        assert_eq!(errors.len(), 2);
    }

    #[test]
    fn test_let_binding_creates_scope() {
        let source = r#"
            fn test() -> int {
                let x = 5 x
            }
        "#;

        let ast = parse(source).unwrap();
        let file_id = FileId(1);
        
        let symbol_bridge = SymbolBridge::new();
        symbol_bridge.analyze_file(file_id, &ast);
        
        let symbol_table = symbol_bridge.symbol_table();
        let errors = check_semantics(&ast, symbol_table, file_id);

        assert_eq!(errors.len(), 0);
    }
}
