use std::process::Command;

/// Helper: write content to a temp file, run mdsh in-place, return the result.
fn mdsh_in_place(name: &str, content: &[u8]) -> Vec<u8> {
    let dir = std::env::temp_dir();
    let path = dir.join(name);
    std::fs::write(&path, content).unwrap();

    let bin = env!("CARGO_BIN_EXE_mdsh");
    let status = Command::new(bin)
        .arg("--input")
        .arg(&path)
        .status()
        .expect("failed to run mdsh");
    assert!(status.success(), "mdsh exited with {status}");

    std::fs::read(&path).unwrap()
}

#[test]
fn no_mdsh_tags_with_trailing_newline_is_unchanged() {
    let input = b"# No mdsh tags\n\nlast line\n";
    let result = mdsh_in_place("mdsh_test_with_nl.md", input);
    assert_eq!(result, input);
}

#[test]
fn no_mdsh_tags_without_trailing_newline_is_unchanged() {
    let input = b"# No mdsh tags\n\nlast line";
    let result = mdsh_in_place("mdsh_test_without_nl.md", input);
    assert_eq!(result, input);
}
