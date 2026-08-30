use std::process::{Command, Output};

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_which-key-wayland"))
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn help_is_available_from_the_binary() {
    let output = run(&["--help"]);

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Usage: which-key-wayland [COMMAND]"));
    assert!(stdout.contains("show"));
    assert!(stdout.contains("reload"));
    assert!(stdout.contains("quit"));
}

#[test]
fn version_is_available_from_the_binary() {
    let output = run(&["--version"]);

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap().trim(),
        concat!("which-key-wayland ", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn invalid_show_key_is_rejected_before_startup() {
    let output = run(&["show", "Ctrl++a"]);

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("invalid value 'Ctrl++a'"));
}

#[test]
fn extra_show_argument_is_rejected_before_startup() {
    let output = run(&["show", "a", "b"]);

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("unexpected argument 'b'"));
}
