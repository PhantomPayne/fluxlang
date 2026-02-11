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

impl FluxError {
    /// Convert FluxError to an LSP Diagnostic
    #[cfg(feature = "lsp")]
    pub fn to_lsp_diagnostic(&self, content: &str) -> tower_lsp::lsp_types::Diagnostic {
        use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Range};

        let (span, message, code) = match self {
            FluxError::Syntax { message, span } => (span, message.clone(), "flux::syntax"),
            FluxError::TypeError { message, span } => (span, message.clone(), "flux::type_error"),
            FluxError::Semantic { message, span } => (span, message.clone(), "flux::semantic"),
            FluxError::UnknownIdentifier { name, span } => {
                (span, format!("Unknown identifier: {}", name), "flux::unknown_identifier")
            }
            FluxError::WasmError { message } => {
                // WASM errors don't have spans, so we return a diagnostic at position 0
                return Diagnostic {
                    range: Range {
                        start: tower_lsp::lsp_types::Position { line: 0, character: 0 },
                        end: tower_lsp::lsp_types::Position { line: 0, character: 0 },
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: Some(tower_lsp::lsp_types::NumberOrString::String("flux::wasm".to_string())),
                    message: message.clone(),
                    ..Default::default()
                };
            }
        };

        let range = span_to_lsp_range(span, content);

        Diagnostic {
            range,
            severity: Some(DiagnosticSeverity::ERROR),
            code: Some(tower_lsp::lsp_types::NumberOrString::String(code.to_string())),
            message,
            ..Default::default()
        }
    }
}

/// Convert a SourceSpan to an LSP Range
#[cfg(feature = "lsp")]
fn span_to_lsp_range(span: &SourceSpan, content: &str) -> tower_lsp::lsp_types::Range {
    use tower_lsp::lsp_types::{Position, Range};

    let start_offset = span.offset();
    let end_offset = start_offset + span.len();

    let (start_line, start_char) = offset_to_position(content, start_offset);
    let (end_line, end_char) = offset_to_position(content, end_offset);

    Range {
        start: Position { line: start_line as u32, character: start_char as u32 },
        end: Position { line: end_line as u32, character: end_char as u32 },
    }
}

/// Convert a byte offset to (line, character) position
#[cfg(feature = "lsp")]
fn offset_to_position(content: &str, offset: usize) -> (usize, usize) {
    let mut line = 0;
    let mut character = 0;
    let mut current_offset = 0;

    for c in content.chars() {
        if current_offset >= offset {
            break;
        }
        
        if c == '\n' {
            line += 1;
            character = 0;
        } else {
            // LSP uses UTF-16 code units for character positions
            // Count UTF-16 code units for this character
            character += c.len_utf16();
        }
        
        current_offset += c.len_utf8();
    }

    (line, character)
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
        SourceSpan::new(self.start.into(), (self.end - self.start).into())
    }
}
