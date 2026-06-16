use assert_cmd::Command;

const MD1_VALID: &str = "md1yqpqqxqq8xtwhw4xwn4qh";
const MS1: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";

#[test]
fn md1_hex_to_stdout() {
    let mut cmd = Command::cargo_bin("me").unwrap();
    let out = cmd.arg("--hex").write_stdin(MD1_VALID).assert().success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    // TLV NDEF starts 0x03; record header 0xD1; ends 0xFE.
    assert!(stdout.trim().starts_with("03"));
    assert!(stdout.trim().ends_with("fe"));
}

#[test]
fn ms1_is_refused_with_exit_3() {
    Command::cargo_bin("me")
        .unwrap()
        .arg("--stdout")
        .write_stdin(MS1)
        .assert()
        .code(3)
        .stderr(predicates::str::contains("CODEX32"));
}

#[test]
fn missing_output_mode_is_usage_error() {
    Command::cargo_bin("me")
        .unwrap()
        .write_stdin(MD1_VALID)
        .assert()
        .code(2);
}
