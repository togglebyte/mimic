use std::fmt::{Display, Formatter};

#[derive(Debug, Default, PartialEq)]
pub enum Token {
    // Single char tokens
    Newline,
    Bang,
    Equal,
    At,
    AtAt,

    // Multi char tokens
    As,
    Audio,
    Delete,
    Bool(bool),
    Int(i64),
    Str(String),
    Ident(String),
    Comment,
    Whitespace,
    NoNewline,

    // Actions
    Clear,
    ClosePopup,
    Find,
    FindEnd,
    Goto,
    Insert,
    Jitter,
    LinePause,
    Load,
    Popup,
    Replace,
    Select,
    SetExtension,
    SetTitle,
    ShowLineNumbers,
    Speed,
    Theme,
    Type,
    TypeNl,
    Wait,

    // Eof
    Eof,

    // Consumed (used as a place holder)
    #[default]
    Consumed,
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::At => write!(f, "@"),
            Token::AtAt => write!(f, "@@"),
            Token::Equal => write!(f, "="),
            Token::Bang => write!(f, "!"),
            Token::Newline => write!(f, "<nl>"),

            Token::As => write!(f, "as"),
            Token::Delete => write!(f, "delete"),
            Token::Ident(s) => write!(f, "{s}"),
            Token::Int(int) => write!(f, "{int}"),
            Token::NoNewline => write!(f, "no newline"),
            Token::Str(s) => write!(f, "\"{s}\""),
            Token::Bool(b) => write!(f, "{b}"),

            Token::Audio => write!(f, "audio"),
            Token::Clear => write!(f, "clear"),
            Token::ClosePopup => write!(f, "close popup"),
            Token::Find => write!(f, "find"),
            Token::FindEnd => write!(f, "findend"),
            Token::Goto => write!(f, "goto"),
            Token::Insert => write!(f, "insert"),
            Token::Jitter => write!(f, "jitter"),
            Token::LinePause => write!(f, "line pause"),
            Token::Load => write!(f, "load"),
            Token::Popup => write!(f, "popup"),
            Token::Replace => write!(f, "change"),
            Token::Select => write!(f, "select"),
            Token::SetExtension => write!(f, "set extenion"),
            Token::SetTitle => write!(f, "set title"),
            Token::ShowLineNumbers => write!(f, "show line numbers"),
            Token::Speed => write!(f, "speed"),
            Token::Theme => write!(f, "theme"),
            Token::Type => write!(f, "type"),
            Token::TypeNl => write!(f, "typenl"),
            Token::Wait => write!(f, "wait"),

            Token::Eof => write!(f, "EOF"),

            Token::Consumed => write!(f, "<consumed>"),
            Token::Comment => write!(f, "<comment>"),
            Token::Whitespace => write!(f, "<whitespace>"),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Span {
    pub token: u32,
    pub line: u16,
    pub col: u16,
}

impl Span {
    pub const INITIAL: Self = Self {
        token: 0,
        line: 1,
        col: 1,
    };
}

#[derive(Debug)]
pub struct Tokens<'src> {
    pub(crate) source: &'src str,
    index: usize,
    tokens: Vec<Token>,
    spans: Vec<Span>,
}

impl<'src> Tokens<'src> {
    pub(crate) fn new(source: &'src str, tokens: Vec<Token>, spans: Vec<Span>) -> Self {
        Self {
            source,
            index: 0,
            tokens,
            spans,
        }
    }

    pub(crate) fn current(&self) -> &Token {
        &self.tokens[self.index]
    }

    fn skip_pointelss_tokens(&mut self) {
        while let Token::Whitespace | Token::Comment = self.tokens[self.index] {
            self.index += 1;
        }
    }

    pub(crate) fn consume(&mut self) {
        if self.index == self.tokens.len() - 1 {
            return;
        }

        self.index += 1;
        self.skip_pointelss_tokens();
    }

    pub(crate) fn consume_if(&mut self, token: Token) -> bool {
        self.skip_pointelss_tokens();

        if token.eq(self.current()) {
            self.consume();
            true
        } else {
            false
        }
    }

    pub(crate) fn take(&mut self) -> Token {
        self.skip_pointelss_tokens();

        let token = std::mem::take(&mut self.tokens[self.index]);
        self.consume();
        token
    }

    pub(crate) fn spans(&self) -> (Span, Span) {
        let start = self.spans[self.index];
        let end = if self.index + 1 == self.tokens.len() { start } else { self.spans[self.index + 1] };

        (start, end)
    }

    #[cfg(test)]
    pub fn take_tokens(self) -> Vec<Token> {
        self.tokens
    }
}
