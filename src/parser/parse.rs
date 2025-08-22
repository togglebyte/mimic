use super::instruction::{Dest, Instruction, Instructions, Source};
use super::token::{Token, Tokens};
use super::error::{Error, Result};

struct Parser<'src> {
    tokens: Tokens<'src>,
}

impl<'src> Parser<'src> {
    fn new(tokens: Tokens<'src>) -> Self {
        Self { tokens }
    }

    fn parse(&mut self) -> Result<Instructions> {
        let mut instructions = vec![];

        loop {
            match self.tokens.current() {
                Token::Newline | Token::Comment | Token::Whitespace => {
                    self.tokens.consume();
                    continue;
                }
                Token::Eof => break,
                _ => (),
            }

            let inst = self.next_instruction()?;
            instructions.push(inst);

            match self.tokens.take() {
                Token::Newline | Token::Comment | Token::Whitespace => continue,
                Token::Eof => break,
                token => {
                    return Error::unexpected_token(
                        "newline or end of file",
                        token,
                        self.tokens.spans(),
                        self.tokens.source,
                    );
                }
            }

            // there has to be either newline OR eof here
        }

        Ok(Instructions::new(instructions))
    }

    fn next_instruction(&mut self) -> Result<Instruction> {
        match self.tokens.take() {
            Token::Load => self.load(),
            Token::Goto => self.goto(),
            Token::Type => self.print(false),
            Token::TypeNl => self.print(true),
            Token::Insert => self.insert(),
            Token::Replace => self.change(),
            Token::Delete => self.delete(),
            Token::Speed => self.speed(),
            Token::Select => self.select(),
            Token::Find => self.find(),
            Token::FindEnd => self.find_end(),
            Token::LinePause => self.linepause(),
            Token::SetExtension => self.set_extension(),
            Token::SetTitle => self.set_title(),
            Token::ShowLineNumbers => self.numbers(),
            Token::Clear => self.clear(),
            Token::Jitter => self.jitter(),
            Token::Theme => self.theme(),
            Token::Audio => self.audio(),
            Token::Popup => self.popup(),
            Token::ClosePopup => self.closepopup(),
            Token::WriteBuffer => self.write_buffer(),
            Token::Command => self.command(),
            Token::CommandClear => self.command_clear(),
            Token::Wait => self.wait(),
            token => Error::invalid_instruction(token, self.tokens.spans(), self.tokens.source),
        }
    }

    fn load(&mut self) -> Result<Instruction> {
        match self.tokens.take() {
            Token::Str(path) => match self.tokens.take() {
                Token::As => match self.tokens.take() {
                    Token::Ident(key) => Ok(Instruction::Load(path.into(), key)),
                    token => return Error::invalid_arg("ident", token, self.tokens.spans(), self.tokens.source),
                },
                token => return Error::invalid_arg("as", token, self.tokens.spans(), self.tokens.source),
            },
            token => Error::invalid_arg("string", token, self.tokens.spans(), self.tokens.source),
        }
    }

    fn goto(&mut self) -> Result<Instruction> {
        // goto <ident>|<int> <int>
        // <ident>
        let instr = match self.tokens.take() {
            Token::Ident(ident) => Instruction::Goto(Dest::Marker(ident)),
            Token::Int(row) => match self.tokens.take() {
                Token::Int(col) => Instruction::Goto(Dest::Relative {
                    row: row as i32,
                    col: col as i32,
                }),
                token => return Error::invalid_arg("number", token, self.tokens.spans(), self.tokens.source),
            },
            token => return Error::invalid_arg("ident", token, self.tokens.spans(), self.tokens.source),
        };

        Ok(instr)
    }

    fn print(&mut self, prefix_newline: bool) -> Result<Instruction> {
        let source = match self.tokens.take() {
            Token::Str(s) => Source::Str(s),
            Token::Ident(ident) => Source::Ident(ident),
            token => return Error::invalid_arg("ident", token, self.tokens.spans(), self.tokens.source),
        };

        let trim_trailing_newline = self.tokens.consume_if(Token::NoNewline);
        Ok(Instruction::Type {
            source,
            trim_trailing_newline,
            prefix_newline,
        })
    }

    fn insert(&mut self) -> Result<Instruction> {
        match self.tokens.take() {
            Token::Str(s) => return Ok(Instruction::Insert(Source::Str(s))),
            Token::Ident(ident) => return Ok(Instruction::Insert(Source::Ident(ident))),
            token => return Error::invalid_arg("ident", token, self.tokens.spans(), self.tokens.source),
        }
    }

    fn change(&mut self) -> Result<Instruction> {
        // <string>
        let src = match self.tokens.take() {
            Token::Str(string) => string,
            token => return Error::invalid_arg("string", token, self.tokens.spans(), self.tokens.source),
        };

        // <string|ident>
        let replacement = match self.tokens.take() {
            Token::Str(string) => Source::Str(string),
            Token::Ident(ident) => Source::Ident(ident),
            token => return Error::invalid_arg("string or ident", token, self.tokens.spans(), self.tokens.source),
        };

        let instr = Instruction::Replace { src, replacement };
        Ok(instr)
    }

    fn delete(&mut self) -> Result<Instruction> {
        Ok(Instruction::Delete)
    }

    fn speed(&mut self) -> Result<Instruction> {
        // <int>
        let instr = match self.tokens.take() {
            Token::Int(speed) => Instruction::Speed(speed as u64),
            token => return Error::invalid_arg("int", token, self.tokens.spans(), self.tokens.source),
        };

        Ok(instr)
    }

    fn select(&mut self) -> Result<Instruction> {
        let instr = match self.tokens.take() {
            Token::Int(width) => match self.tokens.take() {
                Token::Int(height) => Instruction::Select {
                    width: width as u16,
                    height: height as u16,
                },
                token => return Error::invalid_arg("col", token, self.tokens.spans(), self.tokens.source),
            },
            token => return Error::invalid_arg("row", token, self.tokens.spans(), self.tokens.source),
        };

        Ok(instr)
    }

    fn find(&mut self) -> Result<Instruction> {
        let instr = match self.tokens.take() {
            Token::Str(needle) => {
                let count = match self.tokens.current() {
                    &Token::Int(count) => {
                        self.tokens.consume();
                        count
                    }
                    _ => 1,
                };
                Instruction::Find {
                    needle,
                    count: count as usize,
                }
            }
            token => return Error::invalid_arg("string", token, self.tokens.spans(), self.tokens.source),
        };

        Ok(instr)
    }

    fn find_end(&mut self) -> Result<Instruction> {
        let instr = match self.tokens.take() {
            Token::Str(needle) => {
                let count = match self.tokens.current() {
                    &Token::Int(count) => {
                        self.tokens.consume();
                        count
                    }
                    _ => 1,
                };
                Instruction::FindEnd {
                    needle,
                    count: count as usize,
                }
            }
            token => return Error::invalid_arg("string", token, self.tokens.spans(), self.tokens.source),
        };

        Ok(instr)
    }

    fn linepause(&mut self) -> Result<Instruction> {
        let instr = match self.tokens.take() {
            Token::Int(ms) => Instruction::LinePause(ms as u64),
            token => return Error::invalid_arg("int", token, self.tokens.spans(), self.tokens.source),
        };

        Ok(instr)
    }

    fn set_extension(&mut self) -> Result<Instruction> {
        let instr = match self.tokens.take() {
            Token::Str(ext) => Instruction::SetExtension(ext),
            token => return Error::invalid_arg("string", token, self.tokens.spans(), self.tokens.source),
        };

        Ok(instr)
    }

    fn set_title(&mut self) -> Result<Instruction> {
        let instr = match self.tokens.take() {
            Token::Str(title) => Instruction::SetTitle(title),
            token => return Error::invalid_arg("string", token, self.tokens.spans(), self.tokens.source),
        };

        Ok(instr)
    }

    fn numbers(&mut self) -> Result<Instruction> {
        let instr = match self.tokens.take() {
            Token::Bool(b) => Instruction::ShowLineNumbers(b),
            token => return Error::invalid_arg("boolean", token, self.tokens.spans(), self.tokens.source),
        };

        Ok(instr)
    }

    fn clear(&mut self) -> Result<Instruction> {
        Ok(Instruction::Clear)
    }

    fn jitter(&mut self) -> Result<Instruction> {
        let instr = match self.tokens.take() {
            Token::Int(jitter) => Instruction::Jitter(jitter as u64),
            token => return Error::invalid_arg("boolean", token, self.tokens.spans(), self.tokens.source),
        };

        Ok(instr)
    }

    fn theme(&mut self) -> Result<Instruction> {
        let instr = match self.tokens.take() {
            Token::Str(theme) => Instruction::SetTheme(theme),
            token => return Error::invalid_arg("boolean", token, self.tokens.spans(), self.tokens.source),
        };

        Ok(instr)
    }

    fn audio(&mut self) -> Result<Instruction> {
        let instr = match self.tokens.take() {
            Token::Str(path) => Instruction::LoadAudio(path.into()),
            token => return Error::invalid_arg("boolean", token, self.tokens.spans(), self.tokens.source),
        };

        Ok(instr)
    }

    fn popup(&mut self) -> Result<Instruction> {
        let instr = match self.tokens.take() {
            Token::Str(msg) => Instruction::Popup(Source::Str(msg)),
            Token::Ident(ident) => Instruction::Popup(Source::Ident(ident)),
            token => return Error::invalid_arg("string", token, self.tokens.spans(), self.tokens.source),
        };

        Ok(instr)
    }

    fn closepopup(&mut self) -> Result<Instruction> {
        Ok(Instruction::ClosePopup)
    }

    fn write_buffer(&mut self) -> Result<Instruction> {
        let instr = match self.tokens.take() {
            Token::Str(path) => Instruction::WriteBuffer(path.into()),
            token => return Error::invalid_arg("string", token, self.tokens.spans(), self.tokens.source),
        };

        Ok(instr)
    }

    fn command(&mut self) -> Result<Instruction> {
        let instr = match self.tokens.take() {
            Token::Str(cmd) => Instruction::Command(Source::Str(cmd)),
            Token::Ident(cmd) => Instruction::Command(Source::Ident(cmd)),
            token => return Error::invalid_arg("string", token, self.tokens.spans(), self.tokens.source),
        };

        Ok(instr)
    }

    fn command_clear(&mut self) -> Result<Instruction> {
        let instr = match self.tokens.take() {
            Token::Int(millis) => Instruction::CommandClearTimeout(millis as u64),
            token => return Error::invalid_arg("milliseconds", token, self.tokens.spans(), self.tokens.source),
        };

        Ok(instr)
    }

    fn wait(&mut self) -> Result<Instruction> {
        let instr = match self.tokens.take() {
            Token::Int(seconds) => Instruction::Wait(seconds as u64),
            token => return Error::invalid_arg("seconds", token, self.tokens.spans(), self.tokens.source),
        };

        Ok(instr)
    }
}

pub fn parse(tokens: Tokens<'_>) -> Result<Instructions> {
    Parser::new(tokens).parse()
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use super::*;
    use crate::lexer::lex;

    fn parse(input: &str) -> Result<Vec<Instruction>> {
        let tokens = lex(input)?;
        super::parse(tokens).map(|i| i.take_instructions())
    }

    fn parse_ok(input: &str) -> Vec<Instruction> {
        parse(input).unwrap()
    }

    // -----------------------------------------------------------------------------
    //   - Util functions -
    // -----------------------------------------------------------------------------
    fn load(path: impl Into<PathBuf>, key: impl Into<String>) -> Instruction {
        let path = path.into();
        let key = key.into();
        Instruction::Load(path, key)
    }

    fn goto(dest: impl Into<Dest>) -> Instruction {
        Instruction::Goto(dest.into())
    }

    fn print_str(s: &str) -> Instruction {
        Instruction::Type {
            source: Source::Str(s.into()),
            trim_trailing_newline: false,
            prefix_newline: false,
        }
    }

    fn print_ident(s: &str) -> Instruction {
        Instruction::Type {
            source: Source::Ident(s.into()),
            trim_trailing_newline: false,
            prefix_newline: false,
        }
    }

    fn replace_str(src: &str, s: &str) -> Instruction {
        let src = src.into();
        Instruction::Replace {
            src,
            replacement: Source::Str(s.into()),
        }
    }

    fn replace_ident(src: &str, s: &str) -> Instruction {
        let src = src.into();
        Instruction::Replace {
            src,
            replacement: Source::Ident(s.into()),
        }
    }

    fn wait(secs: u64) -> Instruction {
        Instruction::Wait(secs)
    }

    #[test]
    fn parse_load() {
        let output = parse_ok("load \"foo.rs\" as hoppy");
        let expected = vec![load("foo.rs", "hoppy")];
        assert_eq!(output, expected);
    }

    #[test]
    fn parse_goto() {
        let output = parse_ok("goto aaa");
        let expected = vec![goto("aaa")];
        assert_eq!(output, expected);

        let output = parse_ok("goto 1, 2");
        let expected = vec![goto((1, 2))];
        assert_eq!(output, expected);
    }

    #[test]
    fn parse_type() {
        let output = parse_ok("type \"a string\"");
        let expected = vec![print_str("a string")];
        assert_eq!(output, expected);

        let output = parse_ok("type aaa");
        let expected = vec![print_ident("aaa")];
        assert_eq!(output, expected);
    }

    #[test]
    fn parse_replace() {
        let output = parse_ok("replace \"a\" \"b\"");
        let expected = vec![replace_str("a", "b")];
        assert_eq!(output, expected);

        let output = parse_ok("replace \"a\" b");
        let expected = vec![replace_ident("a", "b")];
        assert_eq!(output, expected);
    }

    #[test]
    fn parse_wait() {
        let output = parse_ok("wait 123");
        let expected = vec![wait(123)];
        assert_eq!(output, expected);
    }

    #[test]
    fn parse_goto_negatives() {
        let output = parse_ok("goto -1 -2");
        let expected = vec![goto((-1, -2))];
        assert_eq!(output, expected);
    }

    #[test]
    fn multi_lines() {
        let output = parse_ok(
            "

        //
goto 1     2
        //
            wait 1
            // waffles
            wait 2
            // waffles
            ",
        );
        let expected = vec![goto((1, 2)), wait(1), wait(2)];
        assert_eq!(output, expected);
    }
}
