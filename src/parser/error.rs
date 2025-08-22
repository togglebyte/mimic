use std::fmt::{Display, Formatter};

use crate::parser::token::{Span, Token};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    start: Span,
    end: Span,
    source: String,
}

impl Error {
    fn err<T>(kind: ErrorKind, (start, end): (Span, Span), source: impl Into<String>) -> Result<T> {
        let error = Self {
            kind,
            start,
            end,
            source: source.into(),
        };
        Err(error)
    }

    // -----------------------------------------------------------------------------
    //   - Lex errors -
    // -----------------------------------------------------------------------------
    pub(crate) fn invalid_int<T>(span: Span, source: impl Into<String>) -> Result<T> {
        Self::err(ErrorKind::InvalidInteger, (span, span), source)
    }

    pub(crate) fn unterminated_string<T>(span: Span, source: impl Into<String>) -> Result<T> {
        Self::err(ErrorKind::UnterminatedString, (span, span), source)
    }

    // -----------------------------------------------------------------------------
    //   - Parse errors -
    // -----------------------------------------------------------------------------
    pub(crate) fn invalid_instruction<T>(token: Token, spans: (Span, Span), source: impl Into<String>) -> Result<T> {
        Self::err(ErrorKind::InvalidInstruction(token), spans, source)
    }

    pub(crate) fn invalid_include_path<T>(path: String, spans: (Span, Span), source: impl Into<String>) -> Result<T> {
        Self::err(ErrorKind::InvalidIncludePath(path), spans, source)
    }

    pub(crate) fn invalid_arg<T>(
        expected: &'static str,
        token: Token,
        spans: (Span, Span),
        source: impl Into<String>,
    ) -> Result<T> {
        Self::err(
            ErrorKind::InvalidArg {
                expected,
                found: token.to_string(),
            },
            spans,
            source,
        )
    }

    pub(crate) fn unexpected_token<T>(
        expected: &'static str,
        token: Token,
        spans: (Span, Span),
        source: impl Into<String>,
    ) -> Result<T> {
        Self::err(
            ErrorKind::UnexpectedToken {
                expected,
                found: token.to_string(),
            },
            spans,
            source,
        )
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        static MAX_LINES: u16 = 3;
        let from = self.start.line.saturating_sub(2) as usize;
        let to = MAX_LINES.max(self.end.line - self.start.line + 1) as usize;

        let lines = self.source.lines().enumerate().skip(from).take(to);

        let row_width = (from + to - 1).to_string().len();

        writeln!(f)?;
        for (no, line) in lines {
            let gutter = format!("{:>row_width$}: ", no + 1);
            writeln!(f, "{gutter}{line}")?;
            if no + 1 == self.start.line as usize {
                let indent = " ".repeat(gutter.len());
                writeln!(f, "{indent}^-- {}", self.kind)?;
            }
        }
        Ok(())
    }
}

impl std::error::Error for Error {
}

#[derive(Debug)]
pub enum ErrorKind {
    // Lex errors
    UnterminatedString,
    InvalidInteger,

    // Parse errors
    InvalidArg { expected: &'static str, found: String },
    InvalidInstruction(Token),
    UnexpectedToken { expected: &'static str, found: String },
    InvalidIncludePath(String),
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorKind::UnterminatedString => write!(f, "unterminated string"),
            ErrorKind::InvalidInteger => write!(f, "invalid integer"),
            ErrorKind::InvalidArg { expected, found } => write!(f, "expected `{expected}`, found `{found}`"),
            ErrorKind::InvalidInstruction(token) => write!(f, "invalid instruction: `{token}`"),
            ErrorKind::UnexpectedToken { expected, found } => {
                write!(f, "unexpected token, `{expected}`, found `{found}`")
            }
            ErrorKind::InvalidIncludePath(path) => write!(f, "invalid include path: `{path}`"),
        }
    }
}
