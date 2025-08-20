use std::time::Duration;

use anathema::prelude::*;
pub use compile::compile;

use self::editor::Editor;
use self::instructions::Instruction;
use self::syntax::Highlighter;
use error::Result;

mod audio;
mod compile;
mod context;
mod document;
mod editor;
mod error;
mod instructions;
mod markers;
mod random;
pub(crate) mod syntax;
mod textbuffer;

pub mod setup_paths {
    use std::io::Write;
    use std::path::PathBuf;

    use super::error::{Error, Result};

    static INDEX: &[u8] = include_bytes!("../templates/index.aml");
    static STATUS: &[u8] = include_bytes!("../templates/status.aml");
    static ERROR: &[u8] = include_bytes!("../templates/error.aml");
    static POPUP: &[u8] = include_bytes!("../templates/popup.aml");
    static THEME: &[u8] = include_bytes!("../themes/togglebit.tmTheme");

    fn mimic_root() -> PathBuf {
        dirs::config_dir().unwrap().join("mimic")
    }

    pub fn theme_root() -> PathBuf {
        mimic_root().join("themes")
    }

    pub fn syntax_root() -> PathBuf {
        mimic_root().join("syntax")
    }

    // Ensure that templates and syntax files exists
    pub fn ensure_exists() -> Result<PathBuf> {
        let template_dir = mimic_root().join("templates");
        let syntax_dir = syntax_root();
        let theme_dir = theme_root();

        if template_dir.is_dir() {
            return Ok(template_dir);
        }

        if template_dir.is_file() {
            panic!("you have in all your wisdom made a file where a template directory should be");
        }

        _ = std::fs::create_dir_all(&template_dir);
        _ = std::fs::create_dir_all(&syntax_dir);
        _ = std::fs::create_dir_all(&theme_dir);

        for (path, content) in [
            ("index.aml", INDEX),
            ("status.aml", STATUS),
            ("error.aml", ERROR),
            ("popup.aml", POPUP),
        ] {
            let path = template_dir.join(path);
            let mut file = std::fs::File::create(&path).map_err(|_| Error::FilePath(path))?;
            file.write_all(content).expect("did you run out of disk space?");
        }

        let path = theme_root().join("togglebit.tmTheme");
        let mut file = std::fs::File::create(&path).map_err(|_| Error::FilePath(path))?;
        file.write_all(THEME).expect("did you run out of disk space?");

        Ok(mimic_root())
    }
}

pub fn print_syntaxes() {
    let highlighter = Highlighter::new();
    highlighter.print_syntaxes();
}

pub fn print_themes() {
    let highlighter = Highlighter::new();
    highlighter.print_themes();
}

pub fn run(instructions: Vec<Instruction>) -> Result<()> {
    let highlighter = Highlighter::new();
    let editor = Editor::new(instructions, highlighter, Duration::from_millis(70));

    let doc = Document::new("@index");

    let mut backend = TuiBackend::builder()
        .enable_alt_screen()
        .enable_raw_mode()
        .hide_cursor()
        .finish()
        .unwrap();
    backend.finalize();

    let mut builder = Runtime::builder(doc, &backend);

    let template_root = dirs::config_dir().unwrap().join("mimic").join("templates");

    builder.component("index", template_root.join("index.aml"), editor, Default::default())?;
    builder.template("status", template_root.join("status.aml"))?;
    builder.template("error", template_root.join("error.aml"))?;
    builder.template("popup", template_root.join("popup.aml"))?;
    let res = builder.finish(&mut backend, |runtime, backend| runtime.run(backend));

    match res {
        Ok(()) | Err(anathema::runtime::Error::Stop) => {}
        Err(e) => return Err(e.into()),
    }

    Ok(())
}
