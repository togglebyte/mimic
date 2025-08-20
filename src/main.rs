use std::env::args;

use parser::parse;
use ui::compile;

mod parser;
mod ui;

fn help() {
    println!(
        "
Usage
-----

run:            mimic <file path>
print syntaxes: mimic --syntax
print themes:   mimic --themes

example: mimic code.echo

For more information see https://github.com/togglebyte/mimic
"
    );
}

fn main() -> anyhow::Result<()> {
    let mut args = args().skip(1);

    let Some(arg) = args.next() else {
        help();
        return Ok(());
    };

    ui::setup_paths::ensure_exists()?;

    if arg == "--syntax" {
        ui::print_syntaxes();
        return Ok(());
    }

    if arg == "--themes" {
        ui::print_themes();
        return Ok(());
    }

    let echo = std::fs::read_to_string(arg)?;
    let instructions = parse(&echo)?;
    let instructions = compile(instructions)?;
    ui::run(instructions)?;
    Ok(())
}
