use std::{io::Write, process::{Command, Stdio}};

pub fn bril2json(input: &str) -> String {
    bril_utils(input, "bril2json")
}
pub fn bril2txt(input: &str) -> String {
    bril_utils(input, "bril2txt")
}
fn bril_utils(input: &str, command: &str) -> String {
    assert!(command == "bril2json" || command == "bril2txt");
    let mut command = Command::new(command)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to execute command");
    if let Some(mut stdin) = command.stdin.take() {
        stdin
            .write_all(input.as_bytes())
            .expect("Failed to write to stdin");
    }
    let out = command
        .wait_with_output()
        .expect("Failed to wait on bril2json")
        .stdout;
    let out = String::from_utf8(out).expect("invalid string");
    out
}
