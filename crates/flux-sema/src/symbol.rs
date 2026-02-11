use crate::types::TypeInfo;
use crate::vfs::FileId;
use dashmap::DashMap;
use flux_errors::Span;

/// Symbol information for variables, functions, etc.
#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub ty: TypeInfo,
    pub span: Span,
    pub file_id: FileId,
    pub kind: SymbolKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SymbolKind {
    Function,
    Variable,
    Parameter,
    Type,
}

/// Symbol table for tracking symbols across files
pub struct SymbolTable {
    symbols: DashMap<FileId, Vec<Symbol>>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            symbols: DashMap::new(),
        }
    }

    pub fn insert(&self, file_id: FileId, symbol: Symbol) {
        self.symbols.entry(file_id).or_default().push(symbol);
    }

    pub fn get_symbols(&self, file_id: FileId) -> Vec<Symbol> {
        self.symbols
            .get(&file_id)
            .map(|entry| entry.clone())
            .unwrap_or_default()
    }

    /// Find symbol at a specific position (for hover/go-to-definition)
    pub fn find_symbol_at_position(&self, file_id: FileId, offset: usize) -> Option<Symbol> {
        let symbols = self.get_symbols(file_id);
        symbols
            .into_iter()
            .find(|sym| sym.span.start <= offset && offset <= sym.span.end)
    }

    pub fn clear(&self, file_id: FileId) {
        self.symbols.remove(&file_id);
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

/// Symbol bridge - connects LSP queries to semantic information
pub struct SymbolBridge {
    symbol_table: SymbolTable,
}

impl SymbolBridge {
    pub fn new() -> Self {
        Self {
            symbol_table: SymbolTable::new(),
        }
    }

    /// Analyze a file and populate symbol table
    pub fn analyze_file(&self, file_id: FileId, ast: &flux_syntax::SourceFile) {
        self.symbol_table.clear(file_id);

        for item in &ast.items {
            match item {
                flux_syntax::Item::Function(func) => {
                    // Build parameter types
                    let params: Vec<TypeInfo> = func
                        .params
                        .iter()
                        .map(|param| {
                            param
                                .ty
                                .as_ref()
                                .map_or(TypeInfo::Unknown, |ty| self.type_from_ast(ty))
                        })
                        .collect();

                    // Get return type
                    let ret = if let Some(ret_ty) = &func.return_type {
                        self.type_from_ast(ret_ty)
                    } else {
                        TypeInfo::Unknown
                    };

                    // Create function type
                    let ty = TypeInfo::Function {
                        params,
                        ret: Box::new(ret),
                    };

                    self.symbol_table.insert(
                        file_id,
                        Symbol {
                            name: func.name.clone(),
                            ty,
                            span: func.span,
                            file_id,
                            kind: SymbolKind::Function,
                        },
                    );
                }
            }
        }
    }

    /// Convert AST type to TypeInfo
    fn type_from_ast(&self, ty: &flux_syntax::Type) -> TypeInfo {
        match ty {
            flux_syntax::Type::Int(_) => TypeInfo::Int,
            flux_syntax::Type::String(_) => TypeInfo::String,
            flux_syntax::Type::Bool(_) => TypeInfo::Bool,
            flux_syntax::Type::Float(_) => TypeInfo::Float,
            flux_syntax::Type::Named { name, .. } => TypeInfo::Named { name: name.clone() },
        }
    }

    /// Find symbol at position (for LSP hover)
    pub fn symbol_at_position(&self, file_id: FileId, offset: usize) -> Option<Symbol> {
        self.symbol_table.find_symbol_at_position(file_id, offset)
    }

    /// Get access to the underlying symbol table for semantic checking
    pub fn symbol_table(&self) -> &SymbolTable {
        &self.symbol_table
    }
}

impl Default for SymbolBridge {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_table_insert_and_find() {
        let table = SymbolTable::new();
        let file_id = FileId(1);

        let symbol = Symbol {
            name: "test".to_string(),
            ty: TypeInfo::Int,
            span: Span::new(0, 4),
            file_id,
            kind: SymbolKind::Variable,
        };

        table.insert(file_id, symbol.clone());

        let found = table.find_symbol_at_position(file_id, 2);
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "test");
    }
}
