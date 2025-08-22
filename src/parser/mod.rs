pub use instruction::{Dest, Instruction, Instructions, Source, Variable};

mod error;
mod instruction;
mod lexer;
mod parse;
pub(crate) mod token;

pub fn parse<'a>(input: &'a str) -> error::Result<Instructions> {
    let tokens = lexer::lex(input)?;
    parse::parse(tokens)
}
