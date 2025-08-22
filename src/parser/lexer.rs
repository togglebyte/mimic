use std::iter::Peekable;
use std::str::Chars;

use unicode_width::UnicodeWidthChar;

use super::error::{Error, Result};
use super::token::{Span, Token, Tokens};

pub fn lex<'a>(code: &'a str) -> Result<Tokens<'a>> {
    Lexer::new(code).lex()
}

struct Lexer<'src> {
    source: &'src str,
    input: Peekable<Chars<'src>>,
    tokens: Vec<Token>,
    spans: Vec<Span>,
    current_span: Span,
    next_span: Span,
}

impl<'src> Lexer<'src> {
    pub fn new(source: &'src str) -> Self {
        Self {
            source,
            input: source.chars().peekable(),
            tokens: vec![],
            spans: vec![],
            current_span: Span::INITIAL,
            next_span: Span::INITIAL,
        }
    }

    fn consume_char(&mut self) {
        let c = self
            .input
            .next()
            .expect("every character has to be checked before consume is called");

        if c == '\n' {
            self.next_span.line += 1;
            self.next_span.col = 1;
        } else {
            self.next_span.col += c.width().unwrap_or(0) as u16;
        }
    }

    fn single_char_token(&mut self, token: Token) {
        if token == Token::Newline {
            self.next_span.line += 1;
            self.next_span.col = 1;
        } else {
            self.next_span.col += 1;
        }
        self.push_token(token);
    }

    fn multi_char_token(&mut self, token: Token) {
        _ = self.input.next();
        self.next_span.col += 2;
        self.push_token(token);
    }

    fn lex(mut self) -> Result<Tokens<'src>> {
        loop {
            let Some(c) = self.input.next() else { break };

            match c {
                // -----------------------------------------------------------------------------
                //   - Multi char tokens-
                // -----------------------------------------------------------------------------
                '/' if Some('/') == self.input.peek().copied() => self.comment(),
                '@' if Some('@') == self.input.peek().copied() => self.multi_char_token(Token::AtAt),

                // -----------------------------------------------------------------------------
                //   - Single char tokens -
                // -----------------------------------------------------------------------------
                '\n' => self.single_char_token(Token::Newline),
                '=' => self.single_char_token(Token::Equal),
                '@' => self.single_char_token(Token::At),
                '!' => self.single_char_token(Token::Bang),

                '-' | '0'..='9' => self.int(c)?,
                'a'..='z' | 'A'..='Z' => self.ident(c)?,
                '"' | '\'' => self.string(c)?,
                _ => self.whitespace(),
            }
        }

        self.push_token(Token::Eof);

        let tokens = Tokens::new(self.source, self.tokens, self.spans);
        Ok(tokens)
    }

    fn string(&mut self, quote: char) -> Result<()> {
        let mut buffer = String::new();
        let mut escaping = false;

        loop {
            match self.input.peek() {
                Some('\\') if !escaping => {
                    if let Some('"' | '\\') = self.input.peek() {
                        escaping = true;
                    } else {
                        escaping = false;
                    }
                }
                Some('n') if escaping => {
                    buffer.push('\n');
                    escaping = false;
                }
                Some(c @ '\\') if escaping => {
                    buffer.push(*c);
                    escaping = false;
                }
                Some(c) if escaping && *c == quote => {
                    buffer.push(*c);
                    escaping = false;
                }
                // Closing quote
                Some(c) if *c == quote => {
                    self.consume_char();
                    break;
                }
                Some(c) => buffer.push(*c),
                None => return Error::unterminated_string(self.next_span, self.source),
            }

            self.consume_char();
        }

        self.push_token(Token::Str(buffer));
        Ok(())
    }

    fn ident(&mut self, initial: char) -> Result<()> {
        let mut buffer = String::from(initial);

        loop {
            match self.input.peek() {
                Some(c @ ('a'..='z' | 'A'..='Z' | '0'..'9' | '_' | '-')) => {
                    buffer.push(*c);
                    self.consume_char();
                }
                Some(_) | None => break,
            }
        }

        let token = match buffer.as_str() {
            "as" => Token::As,
            "audio" => Token::Audio,
            "clear" => Token::Clear,
            "closepopup" | "close_popup" => Token::ClosePopup,
            "command" => Token::Command,
            "command_clear_timeout" => Token::CommandClear,
            "delete" => Token::Delete,
            "extension" => Token::SetExtension,
            "false" => Token::Bool(false),
            "find" => Token::Find,
            "finde" => Token::FindEnd,
            "goto" => Token::Goto,
            "insert" => Token::Insert,
            "jitter" => Token::Jitter,
            "linepause" | "line_pause" => Token::LinePause,
            "load" => Token::Load,
            "nonl" => Token::NoNewline,
            "numbers" => Token::ShowLineNumbers,
            "popup" => Token::Popup,
            "replace" => Token::Replace,
            "select" => Token::Select,
            "set" => Token::SetVariable,
            "speed" => Token::Speed,
            "theme" => Token::Theme,
            "title" => Token::SetTitle,
            "true" => Token::Bool(true),
            "type" => Token::Type,
            "typenl" => Token::TypeNl,
            "wait" | "sleep" => Token::Wait,
            "write" => Token::WriteBuffer,
            _ => Token::Ident(buffer),
        };
        self.push_token(token);
        Ok(())
    }

    fn int(&mut self, c: char) -> Result<()> {
        let mut buffer = String::from(c);
        loop {
            match self.input.peek() {
                Some(c @ '0'..='9') => {
                    buffer.push(*c);
                    self.consume_char();
                }
                Some(_) | None => break,
            }
        }

        let int = match buffer.parse() {
            Ok(int) => int,
            Err(_) => return Error::invalid_int(self.next_span, self.source),
        };

        let token = Token::Int(int);
        self.push_token(token);
        Ok(())
    }

    fn push_token(&mut self, token: Token) {
        self.current_span.token = self.tokens.len() as u32;
        self.spans.push(self.current_span);
        self.current_span = self.next_span;
        self.tokens.push(token);
    }

    fn comment(&mut self) {
        // Consume the last '\'
        self.consume_char();

        while let Some(&c) = self.input.peek() {
            self.consume_char();
            if c == '\n' {
                break;
            }
        }

        self.push_token(Token::Comment);
    }

    fn whitespace(&mut self) {
        loop {
            match self.input.peek() {
                Some(c) if c.is_ascii_whitespace() => {
                    self.consume_char();
                    break;
                }
                Some(_) | None => break,
            }
        }
        self.push_token(Token::Whitespace);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn ident(s: &str) -> Token {
        Token::Ident(s.into())
    }

    fn string(s: &str) -> Token {
        Token::Str(s.into())
    }

    fn int(i: i64) -> Token {
        Token::Int(i)
    }

    macro_rules! token_fn {
        ($name:ident, $variant:ident) => {
            fn $name() -> Token {
                Token::$variant
            }
        };
    }

    token_fn!(load, Load);
    token_fn!(goto, Goto);
    token_fn!(whitespace, Whitespace);
    token_fn!(comment, Comment);
    token_fn!(equal, Equal);
    token_fn!(nl, Newline);
    token_fn!(eof, Eof);

    fn lex_tokens(input: &str) -> Vec<Token> {
        lex(input).unwrap().take_tokens()
    }

    #[test]
    fn lex_load() {
        let input = "load \"hello\"\n=";
        let tokens = lex_tokens(input);

        let expected = vec![load(), whitespace(), string("hello"), nl(), equal(), eof()];
        assert_eq!(tokens, expected);
    }

    #[test]
    fn lex_goto() {
        let input = "goto 1 2";
        let tokens = lex_tokens(input);

        let expected = vec![goto(), whitespace(), int(1), whitespace(), int(2), eof()];
        assert_eq!(tokens, expected);
    }

    #[test]
    fn lext_string_with_nl() {
        let input = "\"string\n\"";
        let tokens = lex_tokens(input);

        let expected = vec![string("string\n"), eof()];
        assert_eq!(tokens, expected);
    }

    #[test]
    fn lex_negative_int() {
        let input = "-123";
        let tokens = lex_tokens(input);

        let expected = vec![int(-123), eof()];
        assert_eq!(tokens, expected);
    }

    #[test]
    fn lex_ident() {
        let input = "change1";
        let tokens = lex_tokens(input);

        let expected = vec![ident("change1"), eof()];
        assert_eq!(tokens, expected);
    }

    #[test]
    fn lext_comments() {
        let input = "// comment";
        let tokens = lex_tokens(input);

        let expected = vec![comment(), eof()];
        assert_eq!(tokens, expected);
    }

    #[test]
    fn span_for_comments() {
        let input = "// comment";
        let (start, end) = lex(input).unwrap().spans();

        assert_eq!(
            Span {
                token: 0,
                line: 1,
                col: 1
            },
            start
        );
        assert_eq!(
            Span {
                token: 1,
                line: 1,
                col: 10,
            },
            end
        );
    }
}
