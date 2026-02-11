use crate::ast::*;
use crate::lexer::{tokenize, Token, TokenKind};
use flux_errors::{FluxError, Result, Span};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(input: &str) -> Self {
        Self {
            tokens: tokenize(input),
            pos: 0,
        }
    }

    fn current(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&self.tokens[self.tokens.len() - 1])
    }

    fn peek(&self, offset: usize) -> &Token {
        self.tokens
            .get(self.pos + offset)
            .unwrap_or(&self.tokens[self.tokens.len() - 1])
    }

    fn advance(&mut self) -> Token {
        let token = self.current().clone();
        if token.kind != TokenKind::Eof {
            self.pos += 1;
        }
        token
    }

    fn expect(&mut self, kind: TokenKind) -> Result<Token> {
        let token = self.current().clone();
        if token.kind == kind {
            self.advance();
            Ok(token)
        } else {
            Err(FluxError::Syntax {
                message: format!("Expected {:?}, found {:?}", kind, token.kind),
                span: token.span.to_source_span(),
            })
        }
    }

    pub fn parse(&mut self) -> Result<SourceFile> {
        let start = self.current().span.start;
        let mut items = Vec::new();

        while self.current().kind != TokenKind::Eof {
            items.push(self.parse_item()?);
        }

        let end = if items.is_empty() {
            start
        } else {
            items.last().unwrap().span().end
        };

        Ok(SourceFile {
            items,
            span: Span::new(start, end),
        })
    }

    fn parse_item(&mut self) -> Result<Item> {
        let is_export = if self.current().kind == TokenKind::KwExport {
            self.advance();
            true
        } else {
            false
        };

        match self.current().kind {
            TokenKind::KwFn => Ok(Item::Function(self.parse_function(is_export)?)),
            TokenKind::KwImport => {
                if is_export {
                    return Err(FluxError::Syntax {
                        message: "Cannot export import statement".to_string(),
                        span: self.current().span.to_source_span(),
                    });
                }
                Ok(Item::Import(self.parse_import()?))
            }
            _ => Err(FluxError::Syntax {
                message: format!("Expected item, found {:?}", self.current().kind),
                span: self.current().span.to_source_span(),
            }),
        }
    }

    fn parse_function(&mut self, is_export: bool) -> Result<Function> {
        let start = self.current().span.start;
        self.expect(TokenKind::KwFn)?;

        let name_token = self.expect(TokenKind::Ident)?;
        let name = name_token.text.clone();

        self.expect(TokenKind::LParen)?;
        let mut params = Vec::new();

        while self.current().kind != TokenKind::RParen {
            params.push(self.parse_param()?);
            if self.current().kind == TokenKind::Comma {
                self.advance();
            } else {
                break;
            }
        }

        self.expect(TokenKind::RParen)?;

        let return_type = if self.current().kind == TokenKind::OpArrow {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        let body = self.parse_expr()?;
        let end = body.span().end;

        Ok(Function {
            is_export,
            name,
            params,
            return_type,
            body,
            labels: Vec::new(),
            span: Span::new(start, end),
        })
    }

    fn parse_param(&mut self) -> Result<Param> {
        let start = self.current().span.start;
        let name_token = self.expect(TokenKind::Ident)?;
        let name = name_token.text.clone();

        let ty = if self.current().kind == TokenKind::Colon {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        let end = ty.as_ref().map(|t| t.span().end).unwrap_or(name_token.span.end);

        Ok(Param {
            name,
            ty,
            span: Span::new(start, end),
        })
    }

    fn parse_import(&mut self) -> Result<Import> {
        let start = self.current().span.start;
        self.expect(TokenKind::KwImport)?;

        self.expect(TokenKind::LBrace)?;
        let mut items = Vec::new();

        while self.current().kind != TokenKind::RBrace {
            let ident = self.expect(TokenKind::Ident)?;
            items.push(ident.text.clone());
            if self.current().kind == TokenKind::Comma {
                self.advance();
            } else {
                break;
            }
        }

        self.expect(TokenKind::RBrace)?;
        self.expect(TokenKind::KwFrom)?;

        let module_token = self.expect(TokenKind::LitString)?;
        let module = module_token.text.trim_matches('"').to_string();
        let end = module_token.span.end;

        Ok(Import {
            items,
            module,
            span: Span::new(start, end),
        })
    }

    fn parse_type(&mut self) -> Result<Type> {
        let token = self.current().clone();
        match token.kind {
            TokenKind::TyInt => {
                self.advance();
                Ok(Type::Int(token.span))
            }
            TokenKind::TyString => {
                self.advance();
                Ok(Type::String(token.span))
            }
            TokenKind::TyTable => {
                let start = token.span.start;
                self.advance();
                self.expect(TokenKind::OpLt)?;
                let element = Box::new(self.parse_type()?);
                let end_token = self.expect(TokenKind::OpGt)?;
                Ok(Type::Table {
                    element,
                    span: Span::new(start, end_token.span.end),
                })
            }
            TokenKind::Ident | TokenKind::TyProject => {
                let name = token.text.clone();
                self.advance();
                Ok(Type::Named {
                    name,
                    span: token.span,
                })
            }
            _ => Err(FluxError::Syntax {
                message: format!("Expected type, found {:?}", token.kind),
                span: token.span.to_source_span(),
            }),
        }
    }

    fn parse_expr(&mut self) -> Result<Expr> {
        self.parse_pipeline()
    }

    fn parse_pipeline(&mut self) -> Result<Expr> {
        let mut left = self.parse_let()?;

        while self.current().kind == TokenKind::OpPipe {
            let start = left.span().start;
            self.advance();
            let right = self.parse_let()?;
            let end = right.span().end;
            left = Expr::Pipeline {
                left: Box::new(left),
                right: Box::new(right),
                span: Span::new(start, end),
            };
        }

        Ok(left)
    }

    fn parse_let(&mut self) -> Result<Expr> {
        if self.current().kind == TokenKind::KwLet {
            let start = self.current().span.start;
            self.advance();

            let name_token = self.expect(TokenKind::Ident)?;
            let name = name_token.text.clone();

            self.expect(TokenKind::OpEq)?;
            let value = Box::new(self.parse_expr()?);

            let body = Box::new(self.parse_expr()?);
            let end = body.span().end;

            Ok(Expr::Let {
                name,
                value,
                body,
                span: Span::new(start, end),
            })
        } else {
            self.parse_if()
        }
    }

    fn parse_if(&mut self) -> Result<Expr> {
        if self.current().kind == TokenKind::KwIf {
            let start = self.current().span.start;
            self.advance();

            let cond = Box::new(self.parse_comparison()?);
            let then_branch = Box::new(self.parse_comparison()?);

            let else_branch = if self.current().kind == TokenKind::KwElse {
                self.advance();
                Some(Box::new(self.parse_comparison()?))
            } else {
                None
            };

            let end = else_branch
                .as_ref()
                .map(|e| e.span().end)
                .unwrap_or_else(|| then_branch.span().end);

            Ok(Expr::If {
                cond,
                then_branch,
                else_branch,
                span: Span::new(start, end),
            })
        } else {
            self.parse_comparison()
        }
    }

    fn parse_comparison(&mut self) -> Result<Expr> {
        let mut left = self.parse_additive()?;

        while matches!(self.current().kind, TokenKind::OpLt | TokenKind::OpGt) {
            let start = left.span().start;
            let op = match self.current().kind {
                TokenKind::OpLt => BinOp::Lt,
                TokenKind::OpGt => BinOp::Gt,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_additive()?;
            let end = right.span().end;

            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
                span: Span::new(start, end),
            };
        }

        Ok(left)
    }

    fn parse_additive(&mut self) -> Result<Expr> {
        let mut left = self.parse_multiplicative()?;

        while matches!(
            self.current().kind,
            TokenKind::OpPlus | TokenKind::OpMinus
        ) {
            let start = left.span().start;
            let op = match self.current().kind {
                TokenKind::OpPlus => BinOp::Add,
                TokenKind::OpMinus => BinOp::Sub,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_multiplicative()?;
            let end = right.span().end;

            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
                span: Span::new(start, end),
            };
        }

        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<Expr> {
        let mut left = self.parse_call()?;

        while matches!(self.current().kind, TokenKind::OpStar | TokenKind::OpSlash) {
            let start = left.span().start;
            let op = match self.current().kind {
                TokenKind::OpStar => BinOp::Mul,
                TokenKind::OpSlash => BinOp::Div,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_call()?;
            let end = right.span().end;

            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
                span: Span::new(start, end),
            };
        }

        Ok(left)
    }

    fn parse_call(&mut self) -> Result<Expr> {
        let mut expr = self.parse_primary()?;

        while self.current().kind == TokenKind::LParen {
            let start = expr.span().start;
            self.advance();

            let mut args = Vec::new();
            while self.current().kind != TokenKind::RParen {
                args.push(self.parse_expr()?);
                if self.current().kind == TokenKind::Comma {
                    self.advance();
                } else {
                    break;
                }
            }

            let end_token = self.expect(TokenKind::RParen)?;
            expr = Expr::Call {
                func: Box::new(expr),
                args,
                span: Span::new(start, end_token.span.end),
            };
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr> {
        let token = self.current().clone();
        match token.kind {
            TokenKind::LitInt => {
                self.advance();
                let value = token.text.parse().unwrap_or(0);
                Ok(Expr::Int {
                    value,
                    span: token.span,
                })
            }
            TokenKind::LitString => {
                self.advance();
                let value = token.text.trim_matches('"').to_string();
                Ok(Expr::String {
                    value,
                    span: token.span,
                })
            }
            TokenKind::LitLabel => {
                self.advance();
                Ok(Expr::Label {
                    name: token.text.clone(),
                    span: token.span,
                })
            }
            TokenKind::Ident => {
                self.advance();
                Ok(Expr::Var {
                    name: token.text.clone(),
                    span: token.span,
                })
            }
            TokenKind::LParen => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(TokenKind::RParen)?;
                Ok(expr)
            }
            TokenKind::LBrace => {
                let start = token.span.start;
                self.advance();
                let mut stmts = Vec::new();

                while self.current().kind != TokenKind::RBrace {
                    stmts.push(self.parse_expr()?);
                    if self.current().kind == TokenKind::Semi {
                        self.advance();
                    }
                }

                let end_token = self.expect(TokenKind::RBrace)?;
                Ok(Expr::Block {
                    stmts,
                    span: Span::new(start, end_token.span.end),
                })
            }
            _ => Err(FluxError::Syntax {
                message: format!("Unexpected token: {:?}", token.kind),
                span: token.span.to_source_span(),
            }),
        }
    }
}

pub fn parse(input: &str) -> Result<SourceFile> {
    let mut parser = Parser::new(input);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_function() {
        let input = "fn add(x: int, y: int) -> int { x + y }";
        let result = parse(input);
        assert!(result.is_ok());
        let ast = result.unwrap();
        assert_eq!(ast.items.len(), 1);
    }

    #[test]
    fn test_parse_pipeline() {
        let input = "fn test() { x |> f |> g }";
        let result = parse(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_label() {
        let input = "fn test() { #primary }";
        let result = parse(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_table_type() {
        let input = "fn test(data: Table<int>) { data }";
        let result = parse(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_plan_skeleton() {
        let input = "export fn plan(ctx) -> Project { ctx }";
        let result = parse(input);
        assert!(result.is_ok());
        let ast = result.unwrap();
        assert_eq!(ast.items.len(), 1);
        if let Item::Function(func) = &ast.items[0] {
            assert!(func.is_export);
            assert_eq!(func.name, "plan");
        }
    }
}
