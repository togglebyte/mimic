use std::path::PathBuf;
use std::time::Duration;

use anathema::geometry::{Pos, Size};

use super::markers::Markers;
use crate::parser::Variable;

#[derive(Debug)]
pub enum Instruction {
    // Relative jump
    Jump(Pos),
    JumpToMarker(String),
    Select(Size),

    // -----------------------------------------------------------------------------
    //   - Modifying instructions -
    // -----------------------------------------------------------------------------
    // * Require new highlighting
    // * If the `content` contains a newline then offset all the subsequent markers
    LoadTypeBuffer(String),
    LoadCommandBuffer(String),
    ClearCommandBuffer,
    ClearCommandWait,
    CommandClearTimeout(Duration),
    // Inserts all the content at once, unlike Type which types the content out
    // character by character
    Insert(String),
    // Remove all character in the highlighted range of the editor, or
    // if no selection exists: remove the character under the cursor
    Delete,
    Wait(Duration),
    Speed(Duration),
    LinePause(Duration),

    FindInCurrentLine {
        needle: String,
        end_of_word: bool,
        count: usize,
    },

    SetTitle(String),
    SetExtension(String),
    SetJitter(u64),
    SetTheme(String),
    ShowLineNumbers(bool),
    AddMarkers {
        row: usize,
        markers: Markers,
    },
    LoadAudio(PathBuf),
    Popup(String),
    ClosePopup,
    Clear,

    WriteBuffer(PathBuf),
    SetVariable(String, Variable)
}
