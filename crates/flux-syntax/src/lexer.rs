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
    #[token("return")]
    KwReturn,
    #[token("export")]
    KwExport,

    // Types
    #[token("int")]
    TyInt,
    #[token("string")]
    TyString,
    #[token("bool")]
    TyBool,
    #[token("float")]
    TyFloat,
    #[token("Project")]
    TyProject,

    // Operators
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
    #[regex(r"[0-9]+\.[0-9]+", priority = 3)]
    LitFloat,
    #[token("true")]
    LitTrue,
    #[token("false")]
    LitFalse,
    #[regex(r#""([^"\\]|\\["\\bnfrt]|u[a-fA-F0-9]{4})*""#)]
    LitString,

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

    while let Some(result) = lexer.next() {
        let kind = result.unwrap_or(TokenKind::Error);
        let text = lexer.slice().to_string();
        let span_range = lexer.span();
        let span = flux_errors::Span::new(span_range.start, span_range.end);

        tokens.push(Token { kind, text, span });
    }

    let end = input.len();
    tokens.push(Token {
        kind: TokenKind::Eof,
        text: String::new(),
        span: flux_errors::Span::new(end, end),
    });

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_keywords() {
        let input = "fn let return";
        let tokens = tokenize(input);
        assert_eq!(tokens[0].kind, TokenKind::KwFn);
        assert_eq!(tokens[1].kind, TokenKind::KwLet);
        assert_eq!(tokens[2].kind, TokenKind::KwReturn);
    }

    #[test]
    fn test_tokenize_bool_float_types() {
        let input = "bool float true false 3.14";
        let tokens = tokenize(input);
        assert_eq!(tokens[0].kind, TokenKind::TyBool);
        assert_eq!(tokens[1].kind, TokenKind::TyFloat);
        assert_eq!(tokens[2].kind, TokenKind::LitTrue);
        assert_eq!(tokens[3].kind, TokenKind::LitFalse);
        assert_eq!(tokens[4].kind, TokenKind::LitFloat);
    }
}
