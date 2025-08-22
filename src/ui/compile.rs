use std::time::Duration;

use anathema::geometry::Size;
use unicode_width::UnicodeWidthStr;

pub use super::context::Context;
use super::error::{Error, Result};
use super::instructions::Instruction;
use crate::parser::{Dest, Source};

pub fn compile(parsed_instructions: crate::parser::Instructions) -> Result<Vec<Instruction>> {
    let mut context = Context::new();
    let mut instructions = vec![];

    for inst in parsed_instructions {
        match inst {
            crate::parser::Instruction::Load(path, key) => {
                let content = std::fs::read_to_string(&path).map_err(|_| Error::Import(path))?;
                context.set(key, content);
            }
            crate::parser::Instruction::Find { needle, count } => instructions.push(Instruction::FindInCurrentLine {
                needle,
                end_of_word: false,
                count,
            }),
            crate::parser::Instruction::FindEnd { needle, count } => {
                instructions.push(Instruction::FindInCurrentLine {
                    needle,
                    end_of_word: true,
                    count,
                })
            }
            crate::parser::Instruction::Goto(dest) => {
                let inst = match dest {
                    Dest::Relative { row, col } => Instruction::Jump((col, row).into()),
                    Dest::Marker(name) => Instruction::JumpToMarker(name),
                };
                instructions.push(inst);
            }
            crate::parser::Instruction::Select { width, height } => {
                instructions.push(Instruction::Select(Size::new(width, height)))
            }
            crate::parser::Instruction::Delete => instructions.push(Instruction::Delete),
            crate::parser::Instruction::Type {
                source,
                trim_trailing_newline,
                prefix_newline,
            } => {
                let mut content = match source {
                    Source::Str(content) => content,
                    Source::Ident(key) => context.load(key)?,
                };

                if trim_trailing_newline && content.ends_with('\n') {
                    _ = content.pop();
                }

                if prefix_newline {
                    instructions.push(Instruction::Insert("\n".into()));
                }
                instructions.push(Instruction::LoadTypeBuffer(content));
            }
            crate::parser::Instruction::Command(source) => {
                let cmd = match source {
                    Source::Str(cmd) => cmd,
                    Source::Ident(key) => context.load(key)?,
                };
                instructions.push(Instruction::LoadCommandBuffer(cmd));
                instructions.push(Instruction::ClearCommandWait);
                instructions.push(Instruction::ClearCommandBuffer);
            }
            crate::parser::Instruction::Insert(source) => {
                let inst = match source {
                    Source::Str(content) => Instruction::Insert(content),
                    Source::Ident(key) => {
                        let content = context.load(key)?;
                        Instruction::Insert(content)
                    }
                };
                instructions.push(inst);
            }
            crate::parser::Instruction::Replace { src, replacement } => {
                let width = src.width() as u16;
                instructions.push(Instruction::FindInCurrentLine {
                    needle: src,
                    end_of_word: false,
                    count: 1,
                });
                instructions.push(Instruction::Select(Size::new(width, 1)));
                instructions.push(Instruction::Delete);
                let inst = match replacement {
                    Source::Str(content) => Instruction::LoadTypeBuffer(content),
                    Source::Ident(key) => {
                        let content = context.load(key)?;
                        Instruction::LoadTypeBuffer(content)
                    }
                };
                instructions.push(inst);
            }
            crate::parser::Instruction::Wait(seconds) => {
                instructions.push(Instruction::Wait(Duration::from_secs(seconds)))
            }
            crate::parser::Instruction::Speed(instructions_per_second) => {
                let ips = instructions_per_second as f64;
                let micros = (1000_000.0 / ips) as u64;
                instructions.push(Instruction::Speed(Duration::from_micros(micros)))
            }
            crate::parser::Instruction::LinePause(millis) => {
                instructions.push(Instruction::LinePause(Duration::from_millis(millis)))
            }
            crate::parser::Instruction::SetTitle(title) => instructions.push(Instruction::SetTitle(title)),
            crate::parser::Instruction::SetExtension(ext) => instructions.push(Instruction::SetExtension(ext)),
            crate::parser::Instruction::ShowLineNumbers(show) => instructions.push(Instruction::ShowLineNumbers(show)),
            crate::parser::Instruction::Jitter(jitter) => instructions.push(Instruction::SetJitter(jitter)),
            crate::parser::Instruction::SetTheme(theme) => instructions.push(Instruction::SetTheme(theme)),
            crate::parser::Instruction::LoadAudio(path) => instructions.push(Instruction::LoadAudio(path)),
            crate::parser::Instruction::Clear => instructions.push(Instruction::Clear),
            crate::parser::Instruction::Popup(Source::Str(msg)) => instructions.push(Instruction::Popup(msg)),
            crate::parser::Instruction::Popup(Source::Ident(ident)) => {
                let msg = context.load(ident)?;
                instructions.push(Instruction::Popup(msg))
            }
            crate::parser::Instruction::ClosePopup => instructions.push(Instruction::ClosePopup),
            crate::parser::Instruction::WriteBuffer(path) => instructions.push(Instruction::WriteBuffer(path)),
            crate::parser::Instruction::CommandClearTimeout(timeout) => {
                instructions.push(Instruction::CommandClearTimeout(Duration::from_millis(timeout)))
            }
        }
    }

    Ok(instructions)
}

#[cfg(test)]
mod test {}
