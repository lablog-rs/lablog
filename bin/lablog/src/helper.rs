use std::env;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::process::Command;
use store::errors::*;
use tempdir::TempDir;

pub fn string_from_editor(prepoluate: Option<&str>) -> Result<String> {
    let tmpdir = TempDir::new("lablog_tmp").unwrap();
    let tmppath = tmpdir.path().join("note.asciidoc");
    let editor = {
        match env::var("VISUAL") {
            Ok(editor) => editor,
            Err(_) => {
                match env::var("EDITOR") {
                    Ok(editor) => editor,
                    Err(_) => {
                        bail!("not editor set. either set $VISUAL OR $EDITOR environment variable")
                    }
                }
            }
        }
    };

    if let Some(content) = prepoluate {
        let mut file = File::create(tmppath.display().to_string()).chain_err(
            || "can not open tmp editor file to prepoluate with string",
        )?;

        file.write_all(content.as_bytes()).chain_err(
            || "can not prepoluate editor tmp file",
        )?;
    }

    let mut editor_command = Command::new(editor);
    editor_command.arg(tmppath.display().to_string());

    editor_command
        .spawn()
        .chain_err(|| "couldn not launch editor")?
        .wait()
        .chain_err(|| "problem while running editor")?;

    let mut string = String::new();
    let mut file = File::open(tmppath).chain_err(
        || "can not open tmppath for reading",
    )?;

    file.read_to_string(&mut string).chain_err(
        || "can not read tmpfile to string",
    )?;

    Ok(string)
}
