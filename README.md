# Mimic

Script and playback of text with syntax highlighting.

[Mimic.mp4](https://github.com/user-attachments/assets/28010dd5-edc7-4a32-ac5c-eab1f6a82fd8)

## Example

Create a `example.echo` file and add the following code:
```rust
// example.echo
load "src/main.rs" as main
extension "rs"
speed 20
jitter 20
line_pause 300

type main
```
Then run it with:
```bash
$ mimic example.echo
```

## Syntax

To add syntax highlighting for a language currently not included:
Copy the directory into your equivalent of `~/.config/mimic/syntax/<lang>`.

## Changing the UI

Mimic was made with [Anathema](https://crates.io/crates/anathema) and the
templates are installed by default the first time the program is run.

The templates are located in the following directories depending on your
platform: 

* Lin: `~/.config/mimic/templates`
* Win: `C:\Users\<user>\AppData\Roaming\mimic\templates`
* Mac: `/Users/<user>/Library/Application Support/mimic/templates`

### Example

To add support for `TOML` copy the `TOML` directory from this repository into `~/.config/mimic/syntax/TOML`.

## Markers

Markers are used as jump-to points in the code.
A marker represents a line in the code.

Given the following code:
```rust
fn main() {
    // @marker
    println!("hello world");
}
```

A `goto marker` instruction would place the cursor at the beginning of the
`println` macro:

```rust
fn main() {
->    println!("hello world");
}
```

## Commands

## Load

Load a file into memory

Syntax: `load <filepath> as <ident>`

## Delete

Delete selected region and place the cursor at the start of the region.

Syntax: `delete`

## Goto

Move the cursor to a marker if a marker named is given, or to a position
relative to the current cursor. The position is given as `row` and `col` offset.

This means `goto 0 0` keeps the cursor in its current position, 
where as `goto 1 0` moves the cursor down one row but keeps it on the same column.

Note that `goto <marker>` will go to the line where the marker was inserted, 
and first column, regardless of what the column is before the `goto`.

Syntax: `goto <marker>|<row> <col>`

## Insert

Insert either a string or content from memory.

Syntax: `insert <marker>|<string>` or `insert <string>`

## Select

Select the text from the current cursor position given a width and a height.

Syntax: `select <width> <height>`
            
## Type

Type out the given text in the editor.

Syntax: `type <ident>|<string>`

## TypeNl

Type the given text in the editor, unlike the `type` command this will insert a
newline character and move the cursor into the new empty line and start the
typing.
This has a more natural appearance when inserting new lines into existing code.

Syntax: `typenl <ident>|<string>`
or optionally to remove the final trailing newline character:
Syntax: `typenl <ident>|<string> nonl`

## Wait / Sleep

Wait N seconds before loading the next command.
`sleep` is an alias for `wait`

Syntax: `wait <seconds>`

## Speed

This value is given in number of characters per second.
Note that `line_pause` will be respected between characters if a newline is
written.

Syntax: `speed <integer>`
Default: `20`

## Line pause

Set the speed for which to wait after each newline char is typed

Syntax: `line_pause|linepause <milliseconds>`
Default: `0`
            
## Replace

Selects, deletes and replaces the text.

Syntax: `replace <string> <ident>|<string>`

## Numbers

Show / hide line numbers

Syntax: `numbers <true|false>`
Default: `false`

## Clear

Clear the screen

Syntax: `clear`

## Extension

Set the file extension for the syntax highlighter

Syntax: `extension "rs"`
Default: `"txt"`

## Jitter

Pad the frame time with some jitter, making for a more natural appearance of typing.

Syntax: `jitter 25`
Default: `20`

## Theme

Set the theme.
To see a list of themes run `mimic --themes`.

Syntax: `theme <string>`

## Audio

Load a directory with audio files for typing sounds.

Each key will be mapped to an audio file with the same name.
E.g `a` -> `a.mp3`, `_` -> `_.mp3`
If no filename is found it will fallback to `default.mp3`.
If `default.mp3` is missing an error will be raised.

Syntax: `audio <filepath>`

## Popup

Show a popup message

Syntax: `popup <string>|<ident>`

## Close popup

Close a popup message

Syntax: `close_popup|closepopup`

## Find

Move the cursor to a specific word on the same line as the cursor is on.
A second argument decides the nth instance to find.

Syntax: `find <string> <int>?`

## Finde

Same as `find` but places the cursor at the end of the result instead of the
start.

Syntax: `finde <string> <int>?`

## Write

Write the buffer to disk. 
This will fail if the file already exists.

Syntax: `write <path>`.

## Command

Simulate typing a command to the command line in the editor

Syntax: `command <string>|<ident>`

## Command clear timeout

Clear the command line after N milliseconds

Syntax: `command_clear_timeout <milliseconds>`

## Set

Set a context value in the template

Syntax: `set <ident> <string>|<bool>|<int>`

**Note** This instruction is only relevant in the context of the template.

This can be accessed in the template as `state.ctx.<ident>`.

For more information on how this works see [Anathema](https://crates.io/crates/anathema).

## Include

Include another echo file. 
This file will have its own context (meaning given `load "x" as x` `x` is only relevant in the included echo file).

Syntax: `include "other.echo"`
