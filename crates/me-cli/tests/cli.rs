use assert_cmd::Command;

const MD1_VALID: &str = "md1yqpqqxqq8xtwhw4xwn4qh";
const MS1: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";
const MK1_A: &str = "mk1qpzg69pqqsq3zg3ngj4thnxaq5zg3vs7zqsrqqdt4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4vp3kx98j76m4mjlwphf";
const MK1_B: &str =
    "mk1qpzg69ppsnz4v7cjv3qfjhf76k4t5pt96u0psdrqfqvll8qh7h5athg837pmkf3dpug2mmjtfel6x";

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

#[test]
fn echo_prints_validated_string_to_stderr() {
    let assert = Command::cargo_bin("me")
        .unwrap()
        .args(["--hex", "--echo"])
        .write_stdin(MD1_VALID)
        .assert()
        .success();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(stderr.contains("validated md1:"), "stderr: {stderr}");
    assert!(stderr.contains(MD1_VALID), "stderr: {stderr}");
    // stdout stays binary/encoded NDEF only: the echo must never bleed onto it.
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(
        !stdout.contains("validated"),
        "echo leaked to stdout: {stdout}"
    );
    assert!(
        !stdout.contains(MD1_VALID),
        "input leaked to stdout: {stdout}"
    );
}

#[test]
fn no_echo_by_default() {
    let assert = Command::cargo_bin("me")
        .unwrap()
        .args(["--hex"])
        .write_stdin(MD1_VALID)
        .assert()
        .success();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(!stderr.contains("validated"), "unexpected echo: {stderr}");
}

#[test]
fn bundle_emits_manifest_json_on_stdout() {
    let assert = Command::cargo_bin("me")
        .unwrap()
        .arg("bundle")
        .write_stdin(format!("{MD1_VALID}\n{MK1_A}\n{MK1_B}\n"))
        .assert()
        .success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON on stdout");
    assert_eq!(v["wallet_plates"], 4);
    assert_eq!(v["sets"][0]["chunk_set_id"], "0x12345");
    // checklist must NOT be on stdout
    assert!(!stdout.contains("TYPE ON DEVICE"));
}

#[test]
fn bundle_checklist_on_stderr() {
    let assert = Command::cargo_bin("me")
        .unwrap()
        .arg("bundle")
        .write_stdin(format!("{MD1_VALID}\n{MK1_A}\n{MK1_B}\n"))
        .assert()
        .success();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(stderr.contains("TYPE ON DEVICE"), "{stderr}");
}

#[test]
fn bundle_ms1_refused_exit_3() {
    Command::cargo_bin("me")
        .unwrap()
        .arg("bundle")
        .write_stdin(MS1)
        .assert()
        .code(3)
        .stderr(predicates::str::contains("CODEX32"));
}

#[test]
fn bundle_dropped_chunk_exit_4_no_stdout() {
    let assert = Command::cargo_bin("me")
        .unwrap()
        .arg("bundle")
        .write_stdin(MK1_A) // total=2, only 1
        .assert()
        .code(4);
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(stdout.trim().is_empty(), "no manifest on failure: {stdout}");
}

#[test]
fn existing_converter_still_works_without_subcommand() {
    Command::cargo_bin("me")
        .unwrap()
        .arg("--hex")
        .write_stdin(MD1_VALID)
        .assert()
        .success();
}

#[test]
fn bundle_manifest_golden() {
    let assert = Command::cargo_bin("me")
        .unwrap()
        .arg("bundle")
        .write_stdin(format!("{MD1_VALID}\n{MK1_A}\n{MK1_B}\n"))
        .assert()
        .success();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let mut v: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    // Normalize the version so a routine bump doesn't break the golden (spec m-4).
    v["version"] = serde_json::Value::String("x.y.z".into());
    let golden = include_str!("vectors/bundle-md1-mk1.json");
    let expected: serde_json::Value = serde_json::from_str(golden).unwrap();
    assert_eq!(v, expected);
}
