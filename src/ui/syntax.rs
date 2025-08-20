use anathema::state::Color;
use syntect::easy::HighlightLines;
use syntect::highlighting::{FontStyle, Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

use super::error::{Error, Result};
use super::setup_paths::{syntax_root, theme_root};

// -----------------------------------------------------------------------------
//   - Scratch buffer -
// -----------------------------------------------------------------------------
pub struct InactiveScratch {
    lines: Lines<'static>,
}

impl InactiveScratch {
    pub unsafe fn activate<'a>(&mut self, code: &'a str) -> ActiveScratch<'a> {
        ActiveScratch {
            lines: unsafe { std::mem::transmute(&mut self.lines) },
            code,
        }
    }

    pub(crate) fn new() -> Self {
        Self { lines: Lines::new() }
    }
}

pub struct ActiveScratch<'a> {
    lines: &'a mut Lines<'a>,
    code: &'a str,
}

impl ActiveScratch<'_> {
    pub fn with<F, U>(self, mut f: F) -> U
    where
        for<'a> F: FnMut(&mut Lines<'a>, &'a str) -> U,
    {
        f(self.lines, self.code)
    }
}

impl Drop for ActiveScratch<'_> {
    fn drop(&mut self) {
        self.lines.reset();
    }
}

// -----------------------------------------------------------------------------
//   - Lines buffer -
// -----------------------------------------------------------------------------
#[derive(Debug, Default)]
pub struct Lines<'a> {
    spans: Vec<Span<'a>>,
    lines: Vec<usize>,
}

impl<'a> Lines<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    fn reset(&mut self) {
        self.spans.clear();
        self.lines.clear();
    }

    fn newline(&mut self) {
        self.lines.push(self.spans.len());
    }

    fn push_span(&mut self, span: Span<'a>) {
        self.spans.push(span);
    }

    pub fn iter(&self) -> impl Iterator<Item = &[Span<'_>]> {
        let mut index = 0;
        let mut prev = 0;
        std::iter::from_fn(move || {
            if index == self.lines.len() {
                return None;
            }

            let range = prev..self.lines[index];
            prev = self.lines[index];
            index += 1;
            let spans = &self.spans[range];
            Some(spans)
        })
    }
}

// -----------------------------------------------------------------------------
//   - Span -
// -----------------------------------------------------------------------------
#[derive(Debug)]
pub struct Span<'a> {
    pub style: Style,
    pub src: &'a str,
}

impl Span<'_> {
    pub(crate) fn style(&self) -> anathema::widgets::Style {
        let mut style = anathema::widgets::Style::new();

        let fg = self.style.foreground;
        style.fg = Some(Color::Rgb(fg.r, fg.g, fg.b));
        style.set_bold(self.style.font_style.contains(FontStyle::BOLD));
        style.set_italic(self.style.font_style.contains(FontStyle::ITALIC));

        style
    }
}

// -----------------------------------------------------------------------------
//   - Highligher -
// -----------------------------------------------------------------------------
pub struct Highlighter {
    set: SyntaxSet,
    theme_set: ThemeSet,
}

impl Highlighter {
    pub fn new() -> Self {
        #[cfg(not(debug_assertions))]
        let set = SyntaxSet::load_defaults_newlines();
        #[cfg(debug_assertions)]
        let set = SyntaxSet::new();
        let mut builder = set.into_builder();

        // Try to load syntaxes from a config dir
        let syntaxes = syntax_root();
        _ = builder.add_from_folder(syntaxes, true);

        builder.add_plain_text_syntax();
        let set = builder.build();

        let mut theme_set = ThemeSet::load_defaults();
        theme_set
            .add_from_folder(theme_root())
            .expect("the theme directory should be created the first time the program is run");

        Self { set, theme_set }
    }

    pub fn highlight<'a>(&self, theme_name: &str, src: &'a str, ext: &str, buffer: &mut Lines<'a>) -> Result<()> {
        buffer.reset();

        let syntax = self
            .set
            .find_syntax_by_extension(ext)
            .unwrap_or_else(|| self.set.find_syntax_plain_text());

        let theme = self
            .theme_set
            .themes
            .get(theme_name)
            .ok_or_else(|| Error::InvalidTheme(theme_name.into()))?;
        let mut h = HighlightLines::new(syntax, theme);

        for line in LinesWithEndings::from(src) {
            // LinesWithEndings enables use of newlines mode
            let spans = h.highlight_line(line, &self.set)?;
            for (style, src) in spans {
                buffer.push_span(Span { style, src });
            }
            buffer.newline();
        }

        Ok(())
    }

    pub(crate) fn print_syntaxes(&self) {
        for syntax in self.set.syntaxes() {
            println!("{}", syntax.name);
        }
    }

    pub(crate) fn print_themes(&self) {
        for name in self.theme_set.themes.keys() {
            println!("{name}");
        }
    }
}
