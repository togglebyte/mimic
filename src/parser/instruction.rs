use std::path::PathBuf;

#[derive(Debug, PartialEq)]
pub enum Dest {
    Relative { row: i32, col: i32 },
    Marker(String),
}

impl From<(i32, i32)> for Dest {
    fn from((row, col): (i32, i32)) -> Self {
        Self::Relative { row, col }
    }
}

impl From<&str> for Dest {
    fn from(dest: &str) -> Self {
        Self::Marker(dest.into())
    }
}

#[derive(Debug, PartialEq)]
pub enum Source {
    Str(String),
    Ident(String),
}

#[derive(Debug, PartialEq)]
pub enum Variable {
    Bool(bool),
    Str(String),
    Int(i64),
}

#[derive(Debug, PartialEq)]
pub enum Instruction {
    Load(PathBuf, String),
    WriteBuffer(PathBuf),
    Find { needle: String, count: usize },
    FindEnd { needle: String, count: usize },
    Goto(Dest),
    Type {
        source: Source,
        trim_trailing_newline: bool,
        prefix_newline: bool,
    },
    Command(Source),
    CommandClearTimeout(u64),
    Insert(Source),
    Jitter(u64),
    Delete,

    /// This instructions requires that the cursor is placed on the
    /// same line as the src.
    ///
    /// ```text
    /// replace "hello", "good bye"
    ///
    /// text:
    /// hello
    /// hello <- cursor pos -> good bye
    /// hello
    /// ```
    Replace {
        src: String,
        replacement: Source,
    },
    Select {
        width: u16,
        height: u16,
    },
    SetTitle(String),
    SetTheme(String),
    SetExtension(String),
    ShowLineNumbers(bool),
    LinePause(u64),
    Speed(u64),
    LoadAudio(PathBuf),
    Popup(Source),
    ClosePopup,
    Clear,
    Wait(u64),

    SetVariable(String, Variable),
}

#[derive(Debug)]
pub struct Instructions {
    inner: Vec<Instruction>,
}

impl Instructions {
    pub fn new(inner: Vec<Instruction>) -> Self {
        Self { inner }
    }

    #[cfg(test)]
    pub fn take_instructions(self) -> Vec<Instruction> {
        self.inner
    }
}

impl IntoIterator for Instructions {
    type IntoIter = <Vec<Instruction> as IntoIterator>::IntoIter;
    type Item = Instruction;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}
