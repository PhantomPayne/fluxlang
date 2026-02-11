use logos::Logos;

#[derive(Debug, Clone, Copy, PartialEq, Logos)]
#[logos(skip r"[ \t\n\f]+")]
#[logos(skip r"//[^\n]*")]
pub enum TokenKind {
    // Keywords
    #[token("fn")]
    KwFn,
    #[token("let")]
    KwLet,
    #[token("if")]
    KwIf,
    #[token("else")]
    KwElse,
    #[token("return")]
    KwReturn,
    #[token("import")]
    KwImport,
    #[token("from")]
    KwFrom,
    #[token("export")]
    KwExport,

    // Types
    #[token("int")]
    TyInt,
    #[token("string")]
    TyString,
    #[token("Table")]
    TyTable,
    #[token("Project")]
    TyProject,

    // Operators
    #[token("|>")]
    OpPipe,
    #[token("->")]
    OpArrow,
    #[token("=")]
    OpEq,
    #[token("+")]
    OpPlus,
    #[token("-")]
    OpMinus,
    #[token("*")]
    OpStar,
    #[token("/")]
    OpSlash,
    #[token("<")]
    OpLt,
    #[token(">")]
    OpGt,

    // Delimiters
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token(",")]
    Comma,
    #[token(":")]
    Colon,
    #[token(";")]
    Semi,

    // Literals
    #[regex(r"[0-9]+", priority = 2)]
    LitInt,
    #[regex(r#""([^"\\]|\\["\\bnfrt]|u[a-fA-F0-9]{4})*""#)]
    LitString,
    #[regex(r"#[a-zA-Z_][a-zA-Z0-9_]*")]
    LitLabel,

    // Identifiers
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")]
    Ident,

    // Special
    Error,
    Eof,
}

impl TokenKind {
    pub fn is_trivia(&self) -> bool {
        matches!(self, TokenKind::Error)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub text: String,
    pub span: flux_errors::Span,
}

pub fn tokenize(input: &str) -> Vec<Token> {
    let mut lexer = TokenKind::lexer(input);
    let mut tokens = Vec::new();
    let mut offset = 0;

    while let Some(result) = lexer.next() {
        let kind = result.unwrap_or(TokenKind::Error);
        let text = lexer.slice().to_string();
        let len = text.len();
        let span = flux_errors::Span::new(offset, offset + len);
        
        tokens.push(Token { kind, text, span });
        offset += len;
    }

    tokens.push(Token {
        kind: TokenKind::Eof,
        text: String::new(),
        span: flux_errors::Span::new(offset, offset),
    });

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_keywords() {
        let input = "fn let if else return";
        let tokens = tokenize(input);
        assert_eq!(tokens[0].kind, TokenKind::KwFn);
        assert_eq!(tokens[1].kind, TokenKind::KwLet);
        assert_eq!(tokens[2].kind, TokenKind::KwIf);
        assert_eq!(tokens[3].kind, TokenKind::KwElse);
        assert_eq!(tokens[4].kind, TokenKind::KwReturn);
    }

    #[test]
    fn test_tokenize_pipe_operator() {
        let input = "x |> f |> g";
        let tokens = tokenize(input);
        assert_eq!(tokens[0].kind, TokenKind::Ident);
        assert_eq!(tokens[1].kind, TokenKind::OpPipe);
        assert_eq!(tokens[2].kind, TokenKind::Ident);
        assert_eq!(tokens[3].kind, TokenKind::OpPipe);
    }

    #[test]
    fn test_tokenize_label() {
        let input = "#primary #secondary_label";
        let tokens = tokenize(input);
        assert_eq!(tokens[0].kind, TokenKind::LitLabel);
        assert_eq!(tokens[0].text, "#primary");
        assert_eq!(tokens[1].kind, TokenKind::LitLabel);
        assert_eq!(tokens[1].text, "#secondary_label");
    }

    #[test]
    fn test_tokenize_table_type() {
        let input = "Table<int>";
        let tokens = tokenize(input);
        assert_eq!(tokens[0].kind, TokenKind::TyTable);
        assert_eq!(tokens[1].kind, TokenKind::OpLt);
        assert_eq!(tokens[2].kind, TokenKind::TyInt);
        assert_eq!(tokens[3].kind, TokenKind::OpGt);
    }
}
