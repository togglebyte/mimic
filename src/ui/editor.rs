use std::collections::VecDeque;
use std::io::Write;
use std::time::Duration;

use anathema::component::*;
use anathema::default_widgets::{Canvas, CanvasBuffer};
use anathema::geometry::{LocalPos, Pos, Region, Size};
use anathema::widgets::query::Elements;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use super::audio::AudioShell;
use super::document::Document;
use super::instructions::Instruction;
use super::markers::generate;
use super::random::Random;
use super::syntax::{Highlighter, InactiveScratch};
use super::textbuffer::TextBuffer;

// -----------------------------------------------------------------------------
//   - Frame timer -
// -----------------------------------------------------------------------------
struct Timer {
    frame_time: Duration,
    accumulator: Duration,
    wait: Duration,
    jitter: Duration,
    jitter_ms: u64,
    rand: Random,
}

impl Timer {
    pub fn new(frame_time: Duration) -> Self {
        Self {
            frame_time,
            accumulator: Duration::ZERO,
            wait: Duration::ZERO,
            jitter: Duration::ZERO,
            jitter_ms: 20,
            rand: Random::new(),
        }
    }

    fn apply_jitter(&mut self) {
        self.wait += self.jitter;
        self.jitter = Duration::from_millis(self.rand.next(self.jitter_ms));
    }

    fn tick(&mut self, mut dt: Duration) -> usize {
        if !self.wait.is_zero() {
            match self.wait.checked_sub(dt) {
                Some(wait) => {
                    self.wait = wait;
                    return 0;
                }
                None => {
                    self.wait = Duration::ZERO;
                    dt -= self.wait;
                }
            }
        } else {
            self.apply_jitter();
        }

        self.accumulator += dt;

        let mut count = 0;
        while self.accumulator >= self.frame_time {
            self.accumulator = self.accumulator.saturating_sub(self.frame_time);
            count += 1;
        }

        count
    }

    fn wait(&mut self, wait: Duration) {
        self.wait = wait;
    }
}

// -----------------------------------------------------------------------------
//   - Render action -
// -----------------------------------------------------------------------------
enum RenderAction {
    Render,
    Skip,
    NextFrame,
}

// -----------------------------------------------------------------------------
//   - State -
// -----------------------------------------------------------------------------
#[derive(Debug, State, Default)]
pub struct DocState {
    screen_cursor_x: Value<i32>,
    screen_cursor_y: Value<i32>,
    offset_x: Value<i32>,
    offset_y: Value<i32>,
    cursor_x: Value<i32>,
    cursor_y: Value<i32>,
    height: Value<u16>,
    title: Value<String>,
    error: Value<String>,
    debug: Value<String>,
    show_line_numbers: Value<bool>,
    popup: Value<String>,
    command_buffer: Value<String>,
    show_cursor: Value<bool>,
    ctx: Value<Map<Box<dyn State>>>,
}

impl DocState {
    pub fn new() -> Self {
        Self {
            show_cursor: true.into(),
            ..Default::default()
        }
    }
}

// -----------------------------------------------------------------------------
//   - Visual rang -
// -----------------------------------------------------------------------------
#[derive(Debug)]
struct VisualRange {
    region: Region,
}

impl VisualRange {
    fn new(pos: Pos, size: Size) -> Self {
        Self {
            region: Region::from((pos, size)),
        }
    }
}

trait OptVisualRange {
    fn contains(&self, pos: Pos) -> bool;
}

impl OptVisualRange for Option<VisualRange> {
    fn contains(&self, pos: Pos) -> bool {
        match self {
            None => false,
            Some(range) => range.region.contains(pos),
        }
    }
}

// -----------------------------------------------------------------------------
//   - Virtual editor -
// -----------------------------------------------------------------------------
pub struct Editor {
    doc: Document,
    cursor: Pos,
    offset: Pos,
    selected_range: Option<VisualRange>,
    instructions: VecDeque<Instruction>,
    type_buffer: TextBuffer,
    type_command_buffer: TextBuffer,
    highlighter: Highlighter,
    buffer: CanvasBuffer,
    lines: InactiveScratch,
    line_pause: Duration,
    extension: String,
    theme: String,
    audio: AudioShell,
    frame_timer: Timer,
    size: Size,
    command_clear_timeout: Duration,
}

impl Editor {
    pub fn new(instructions: Vec<Instruction>, highlighter: Highlighter, frame_time: Duration) -> Self {
        Self {
            doc: Document::new(String::new()),
            cursor: Pos::ZERO,
            offset: Pos::ZERO,
            selected_range: None,
            instructions: instructions.into(),
            type_buffer: TextBuffer::new(),
            type_command_buffer: TextBuffer::new(),
            highlighter,
            buffer: CanvasBuffer::default(),
            lines: InactiveScratch::new(),
            line_pause: Duration::ZERO,
            extension: "txt".into(),
            theme: String::from("togglebit"),
            audio: AudioShell::new(),
            frame_timer: Timer::new(frame_time),
            size: Size::ZERO,
            command_clear_timeout: Duration::from_secs(1),
        }
    }

    fn error(&mut self, state: &mut DocState, msg: impl Into<String>) {
        self.instructions.clear();
        state.error.set(msg.into());
    }

    fn apply(&mut self, state: &mut DocState) -> RenderAction {
        if let Some(s) = self.type_command_buffer.next() {
            state.command_buffer.to_mut().push_str(s);
            return RenderAction::Render;
        }

        // If we have something to type then do that.
        // otherwise load the next instruction
        if let Some(s) = self.type_buffer.next() {
            self.doc.insert_str(self.cursor, s);

            self.audio.play(s);

            if s == "\n" {
                self.cursor.x = 0;
                self.cursor.y += 1;

                if self.line_pause > Duration::ZERO {
                    self.frame_timer.wait(self.line_pause);
                    return RenderAction::NextFrame;
                }
            } else {
                self.cursor.x += s.width() as i32;
            }

            return RenderAction::Render;
        }

        let instruction = self.instructions.pop_front();
        match instruction {
            None => return RenderAction::Skip,
            Some(instruction) => {
                match instruction {
                    Instruction::LoadCommandBuffer(content) => {
                        state.show_cursor.set(false);
                        self.type_command_buffer.push(content);
                    }
                    Instruction::LoadTypeBuffer(content) => {
                        // Make markers and all that what what
                        let (content, markers) = generate(content);
                        self.type_buffer.push(content);

                        if let Some(markers) = markers {
                            self.instructions.push_front(Instruction::AddMarkers {
                                row: self.cursor.y as usize,
                                markers,
                            });
                        }
                    }
                    Instruction::Insert(content) => {
                        let (content, markers) = generate(content);
                        self.cursor.x = 0;
                        self.doc.insert_str(self.cursor, &content);
                        if let Some(markers) = markers {
                            self.instructions.push_front(Instruction::AddMarkers {
                                row: self.cursor.y as usize,
                                markers,
                            });
                        }
                    }
                    Instruction::AddMarkers { row, markers } => self.doc.add_markers(row, markers),
                    Instruction::Jump(pos) => {
                        self.cursor += pos;
                        // Don't move the cursor past zero
                        self.cursor.x = self.cursor.x.max(0);
                        self.cursor.y = self.cursor.y.max(0);
                    }
                    Instruction::JumpToMarker(name) => {
                        let Some(row) = self.doc.lookup_marker(&name).map(|m| m.row) else {
                            self.error(state, format!("marker \"{name}\" does not exist"));
                            return RenderAction::Render;
                        };
                        self.cursor.y = row as i32;
                        self.cursor.x = 0;
                    }
                    Instruction::Select(size) if size == Size::ZERO => return RenderAction::Render,
                    Instruction::Select(size) => {
                        let visual_range = VisualRange::new(self.cursor, size);
                        self.cursor = visual_range.region.to - Pos::new(1, 1);
                        self.selected_range = Some(visual_range);
                    }
                    Instruction::Delete => match self.selected_range.take() {
                        Some(range) => {
                            self.cursor = range.region.from;
                            self.doc.delete(range.region);
                        }
                        None => self.doc.delete(Region::from((self.cursor, Size::new(1, 1)))),
                    },
                    Instruction::Wait(dur) => {
                        self.frame_timer.wait(dur);
                        return RenderAction::NextFrame;
                    }
                    Instruction::Speed(dur) => self.frame_timer.frame_time = dur,
                    Instruction::FindInCurrentLine {
                        needle,
                        end_of_word,
                        count,
                    } => {
                        if needle.is_empty() {
                            return RenderAction::Render;
                        }
                        let Some(x) = self.doc.find(self.cursor, &needle, count) else { return RenderAction::Render };
                        self.cursor.x = x as i32;
                        if end_of_word {
                            self.cursor.x += needle.width() as i32 - 1;
                        }
                    }
                    Instruction::LinePause(duration) => self.line_pause = duration,
                    Instruction::SetTitle(title) => state.title.set(title),
                    Instruction::SetJitter(jitter) => self.frame_timer.jitter_ms = jitter,
                    Instruction::ShowLineNumbers(show) => state.show_line_numbers.set(show),
                    Instruction::Clear => {
                        self.doc.clear();
                        self.offset = Pos::ZERO;
                        self.cursor = Pos::ZERO;
                    }
                    Instruction::SetExtension(ext) => self.extension = ext,
                    Instruction::SetTheme(theme) => self.theme = theme,
                    Instruction::LoadAudio(path) => {
                        if let Err(e) = self.audio.load(path) {
                            self.error(state, e.to_string());
                        }
                    }
                    Instruction::Popup(message) => state.popup.set(message),
                    Instruction::ClosePopup => state.popup.set(String::new()),
                    Instruction::WriteBuffer(path_buf) if path_buf.exists() => {
                        self.error(state, format!("can't write to {path_buf:?}, file already exists"));
                    }
                    Instruction::WriteBuffer(path_buf) => match std::fs::File::create(&path_buf) {
                        Err(e) => self.error(state, format!("failed to create {path_buf:?} : {e}")),
                        Ok(mut file) => {
                            if let Err(e) = file.write_all(self.doc.text().as_bytes()) {
                                self.error(state, format!("failed to write {path_buf:?} : {e}"));
                            }
                        }
                    },
                    Instruction::ClearCommandBuffer => {
                        state.command_buffer.to_mut().clear();
                        state.show_cursor.set(true);
                    }
                    Instruction::CommandClearTimeout(duration) => self.command_clear_timeout = duration,
                    Instruction::ClearCommandWait => self
                        .instructions
                        .push_front(Instruction::Wait(self.command_clear_timeout)),
                    Instruction::SetVariable(name, variable) => {
                        let value: Box<dyn State> = match variable {
                            crate::parser::Variable::Bool(var) => Box::new(var),
                            crate::parser::Variable::Str(var) => Box::new(var),
                            crate::parser::Variable::Int(var) => Box::new(var),
                        };
                        state.ctx.to_mut().insert(name, value);
                    }
                }
            }
        }

        RenderAction::Render
    }

    fn update_cursor(&mut self, state: &mut DocState) {
        static PADDING: i32 = 7;

        let height = self.size.height as i32 - 1 - PADDING;
        let width = self.size.width as i32 - 1;

        let y = self.cursor.y + self.offset.y;
        if y > height {
            self.offset.y = height - self.cursor.y;
        } else if y < 0 {
            self.offset.y -= self.cursor.y + self.offset.y;
        }

        let x = self.cursor.x + self.offset.x;
        if x > width {
            self.offset.x = width - self.cursor.x;
        } else if x < 0 {
            self.offset.x -= self.cursor.x + self.offset.x;
        }

        state.screen_cursor_x.set(self.cursor.x + self.offset.x);
        state.screen_cursor_y.set(self.cursor.y + self.offset.y);
        state.cursor_x.set(self.cursor.x);
        state.cursor_y.set(self.cursor.y);
        state.offset_x.set(self.offset.x);
        state.offset_y.set(self.offset.y);
    }

    fn draw(&mut self, mut elements: Elements<'_, '_, '_>, state: &mut DocState) {
        elements.by_tag("canvas").first(|el, _| {
            let canvas = el.to::<Canvas>();
            canvas.clear();

            let mut y = self.offset.y;

            // re-highlight the content
            let scratch = unsafe { self.lines.activate(self.doc.text()) };
            let res = scratch.with(|lines, code| {
                self.highlighter.highlight(&self.theme, code, &self.extension, lines)?;

                let skip = (y < 0).then_some(y.abs() as usize).unwrap_or(0);
                y = 0;
                for spans in lines.iter().skip(skip) {
                    let mut x = self.offset.x;
                    for span in spans {
                        for c in span.src.chars() {
                            if x >= 0 {
                                let pos: LocalPos = (x, y).into();
                                let mut style = span.style();
                                // if we have a selected range
                                // then set the background of the style to red
                                // but only if the pos is inside the selected range
                                if self.selected_range.contains(pos.into()) {
                                    style.bg = Some(Color::Red);
                                }
                                canvas.put(c, style, pos);
                            }
                            x += c.width().unwrap_or(0) as i32;
                        }
                    }

                    y += 1;
                }

                Ok::<_, super::error::Error>(())
            });

            if let Err(e) = res {
                self.error(state, e.to_string());
            }
        });
    }
}

impl Component for Editor {
    type Message = Instruction;
    type State = DocState;

    fn on_key(&mut self, key: KeyEvent, _: &mut Self::State, _: Children<'_, '_>, _: Context<'_, '_, Self::State>) {
        match key.code {
            // KeyCode::Char('h') => self.instructions.push_back(Instruction::Jump(Pos::new(-1, 0))),
            // KeyCode::Char('j') => self.instructions.push_back(Instruction::Jump(Pos::new(0, 1))),
            // KeyCode::Char('k') => self.instructions.push_back(Instruction::Jump(Pos::new(0, -1))),
            // KeyCode::Char('l') => self.instructions.push_back(Instruction::Jump(Pos::new(1, 0))),
            // KeyCode::Char('d') => self.instructions.push_back(Instruction::Jump(Pos::new(0, 9))),
            _ => {}
        }
    }

    fn on_tick(
        &mut self,
        state: &mut Self::State,
        mut children: Children<'_, '_>,
        _: Context<'_, '_, Self::State>,
        dt: Duration,
    ) {
        if self.size == Size::ZERO {
            let Some(size) = children.elements().by_tag("canvas").first(|el, _| el.size()) else { return };
            self.size = size;
        }

        state.height.set(self.size.height);

        let mut count = self.frame_timer.tick(dt);
        let mut render = false;

        while count > 0 {
            count -= 1;
            match self.apply(state) {
                RenderAction::Render => render = true,
                RenderAction::Skip => (),
                RenderAction::NextFrame => {
                    render = true;
                    break;
                }
            }
        }

        if render {
            self.update_cursor(state);
            self.draw(children.elements(), state);
        }
    }

    fn on_mount(&mut self, _: &mut Self::State, mut children: Children<'_, '_>, _: Context<'_, '_, Self::State>) {
        children
            .elements()
            .by_tag("canvas")
            .first(|el, _| el.to::<Canvas>().restore_buffer(&mut self.buffer))
            .unwrap();
    }

    fn on_unmount(&mut self, _: &mut Self::State, mut children: Children<'_, '_>, _: Context<'_, '_, Self::State>) {
        self.buffer = children
            .elements()
            .by_tag("canvas")
            .first(|el, _| el.to::<Canvas>().take_buffer())
            .unwrap();
    }

    fn on_resize(&mut self, state: &mut Self::State, mut children: Children<'_, '_>, _: Context<'_, '_, Self::State>) {
        if let Some(size) = children.elements().by_tag("canvas").first(|el, _| el.size()) {
            self.size = size;
            state.height.set(size.height);
        }
    }
}
