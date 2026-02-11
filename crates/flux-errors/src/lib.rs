// thiserror derive generates code that triggers unused_assignments lint
#![allow(unused_assignments)]

use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

/// Main error type for Flux compiler errors
#[derive(Debug, Error, Diagnostic)]
pub enum FluxError {
    #[error("Syntax error: {message}")]
    #[diagnostic(code(flux::syntax))]
    Syntax {
        message: String,
        #[label("here")]
        span: SourceSpan,
    },

    #[error("Type error: {message}")]
    #[diagnostic(code(flux::type_error))]
    TypeError {
        message: String,
        #[label("here")]
        span: SourceSpan,
    },

    #[error("Semantic error: {message}")]
    #[diagnostic(code(flux::semantic))]
    Semantic {
        message: String,
        #[label("here")]
        span: SourceSpan,
    },

    #[error("WASM generation error: {message}")]
    #[diagnostic(code(flux::wasm))]
    WasmError { message: String },

    #[error("Unknown identifier: {name}")]
    #[diagnostic(code(flux::unknown_identifier))]
    UnknownIdentifier {
        name: String,
        #[label("unknown identifier")]
        span: SourceSpan,
    },
}

pub type Result<T> = std::result::Result<T, FluxError>;

/// Represents a position in source code
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn to_source_span(&self) -> SourceSpan {
        SourceSpan::new(self.start.into(), self.end - self.start)
    }
}
