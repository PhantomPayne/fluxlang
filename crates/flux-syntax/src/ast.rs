use flux_errors::Span;

/// Root AST node for a Flux source file
#[derive(Debug, Clone, PartialEq)]
pub struct SourceFile {
    pub items: Vec<Item>,
    pub span: Span,
}

/// Top-level item in a source file
#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    Function(Function),
    Import(Import),
}

impl Item {
    pub fn span(&self) -> Span {
        match self {
            Item::Function(func) => func.span,
            Item::Import(import) => import.span,
        }
    }
}

/// Function definition
#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub is_export: bool,
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub body: Expr,
    pub labels: Vec<String>,
    pub span: Span,
}

/// Function parameter
#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub ty: Option<Type>,
    pub span: Span,
}

/// Import statement
#[derive(Debug, Clone, PartialEq)]
pub struct Import {
    pub items: Vec<String>,
    pub module: String,
    pub span: Span,
}

/// Type annotation
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int(Span),
    String(Span),
    Table { element: Box<Type>, span: Span },
    Named { name: String, span: Span },
}

impl Type {
    pub fn span(&self) -> Span {
        match self {
            Type::Int(s) | Type::String(s) => *s,
            Type::Table { span, .. } | Type::Named { span, .. } => *span,
        }
    }
}

/// Expression
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    // Literals
    Int {
        value: i64,
        span: Span,
    },
    String {
        value: String,
        span: Span,
    },
    Label {
        name: String,
        span: Span,
    },

    // Variables and identifiers
    Var {
        name: String,
        span: Span,
    },

    // Binary operations
    Binary {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
        span: Span,
    },

    // Pipeline operator
    Pipeline {
        left: Box<Expr>,
        right: Box<Expr>,
        span: Span,
    },

    // Function call
    Call {
        func: Box<Expr>,
        args: Vec<Expr>,
        span: Span,
    },

    // Let binding
    Let {
        name: String,
        value: Box<Expr>,
        body: Box<Expr>,
        span: Span,
    },

    // If expression
    If {
        cond: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Option<Box<Expr>>,
        span: Span,
    },

    // Block expression
    Block {
        stmts: Vec<Expr>,
        span: Span,
    },
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Int { span, .. }
            | Expr::String { span, .. }
            | Expr::Label { span, .. }
            | Expr::Var { span, .. }
            | Expr::Binary { span, .. }
            | Expr::Pipeline { span, .. }
            | Expr::Call { span, .. }
            | Expr::Let { span, .. }
            | Expr::If { span, .. }
            | Expr::Block { span, .. } => *span,
        }
    }
}

/// Binary operator
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Lt,
    Gt,
}
