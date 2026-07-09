use assert_cmd::Command;

const MD1_VALID: &str = "md1yqpqqxqq8xtwhw4xwn4qh";
const MS1: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";
const MK1_A: &str = "mk1qpzg69pqqsq3zg3ngj4thnxaq5zg3vs7zqsrqqdt4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4vp3kx98j76m4mjlwphf";
const MK1_B: &str =
    "mk1qpzg69ppsnz4v7cjv3qfjhf76k4t5pt96u0psdrqfqvll8qh7h5athg837pmkf3dpug2mmjtfel6x";

/// The crate version the sidecar must match (env at compile time).
const CRATE_VERSION: &str = env!("CARGO_PKG_VERSION");

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

// A3/F4: interior separators ('-' or whitespace) in an otherwise-valid md1 are
// stripped by md-codec before BCH but engraved verbatim by convert() — refuse
// fail-closed. Convert path AND single-line bundle path both exit 4; the bundle
// path must not echo the input body (composes with the Step 3 canary).
#[test]
fn convert_refuses_interior_separator_md1_exit_4() {
    for bad in ["md1yqpqq-xqq8xtwhw4xwn4qh", "md1yqpqq xqq8xtwhw4xwn4qh"] {
        Command::cargo_bin("me")
            .unwrap()
            .arg("--stdout")
            .write_stdin(bad)
            .assert()
            .code(4);
    }
}

#[test]
fn bundle_refuses_interior_separator_md1_exit_4_no_leak() {
    for bad in ["md1yqpqq-xqq8xtwhw4xwn4qh", "md1yqpqq xqq8xtwhw4xwn4qh"] {
        let assert = Command::cargo_bin("me")
            .unwrap()
            .arg("bundle")
            .write_stdin(bad)
            .assert()
            .code(4);
        let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
        assert!(
            !stderr.contains(bad),
            "bundle leaked non-canonical md1 body: {stderr}"
        );
    }
}

// A1/F1: an ms1 secret with a 1-typo HRP (`msx1…`) dodges the exact-HRP ms1
// refusal (classified as an unknown HRP) — the error MUST NOT echo the intact
// codex32 secret body to stderr (shell scrollback / 2>logfile / CI logs).
#[test]
fn bundle_msx1_mangled_hrp_does_not_leak_secret_body() {
    const MSX1: &str = "msx10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";
    // Everything after the mangled `msx1` HRP is the intact secret codex32 body.
    const SECRET_BODY: &str = "0entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";
    let assert = Command::cargo_bin("me")
        .unwrap()
        .arg("bundle")
        .write_stdin(MSX1)
        .assert()
        .code(4); // classify failure → invalid/integrity exit
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(
        !stderr.contains(SECRET_BODY),
        "leaked ms1 secret body to stderr: {stderr}"
    );
    assert!(
        !stderr.contains(MSX1),
        "leaked full mangled input line to stderr: {stderr}"
    );
}

// A1/F1: a corrupted (non-pristine) mk1 must not have its full string echoed to
// stderr on the bundle error path (the convert path was hardened; bundle regressed).
#[test]
fn bundle_corrupted_mk1_does_not_leak_full_string() {
    let mut bad = MK1_B.to_string();
    let last = bad.pop().unwrap();
    bad.push(if last == 'q' { 'p' } else { 'q' });
    let assert = Command::cargo_bin("me")
        .unwrap()
        .arg("bundle")
        .write_stdin(bad.clone())
        .assert()
        .code(4);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(
        !stderr.contains(&bad),
        "leaked corrupted mk1 full string to stderr: {stderr}"
    );
}

// B3 (F16): ms1 must be refused across case-folding, whitespace padding, a bad
// checksum (refusal is HRP-only — NO decode of the secret payload), and at every
// bundle line position. Exit 3 / RefusedSecret on BOTH convert and bundle; the
// secret body is NEVER echoed to stderr (regression insurance for the Step 3 A1
// redaction). "No decode" is asserted via the error TYPE (exit 3 = RefusedSecret,
// not exit 4 = a validate/decode error), not timing.
#[test]
fn ms1_refusal_table() {
    fn run(args: &[&str], stdin: &str) -> (i32, String) {
        let mut cmd = Command::cargo_bin("me").unwrap();
        for a in args {
            cmd.arg(a);
        }
        let assert = cmd.write_stdin(stdin.to_string()).assert();
        let out = assert.get_output();
        (
            out.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&out.stderr).to_string(),
        )
    }

    const BODY: &str = "0entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";
    let lc = format!("ms1{BODY}");
    let uc = lc.to_uppercase();
    let uc_body = BODY.to_uppercase();
    let mixed = format!("Ms1{BODY}");
    let padded = format!("  \t{lc}\n ");
    // Bad checksum: flip the last body char. Still refused (HRP-only pre-scan),
    // proving refusal precedes any BCH decode of the secret.
    let mut bad = lc.clone();
    let last = bad.pop().unwrap();
    bad.push(if last == 'q' { 'p' } else { 'q' });
    let bad_marker = bad["ms1".len()..].to_string();

    // (label, input, secret marker that must NOT appear in stderr)
    let single: Vec<(&str, String, String)> = vec![
        ("lowercase", lc.clone(), BODY.to_string()),
        ("uppercase", uc, uc_body),
        ("mixed-case", mixed, BODY.to_string()),
        ("whitespace-padded", padded, BODY.to_string()),
        ("bad-checksum", bad, bad_marker),
    ];

    for (label, input, marker) in &single {
        for mode in [["--stdout"].as_slice(), ["bundle"].as_slice()] {
            let (code, stderr) = run(mode, input);
            assert_eq!(
                code, 3,
                "{label} via {mode:?}: expected exit 3 (RefusedSecret); stderr={stderr}"
            );
            assert!(
                stderr.contains("CODEX32"),
                "{label} via {mode:?}: refusal message missing: {stderr}"
            );
            assert!(
                !stderr.contains(marker.as_str()),
                "{label} via {mode:?}: leaked secret body: {stderr}"
            );
        }
    }

    // ms1 at first / middle / last bundle line, surrounded by valid public lines.
    let positions = [
        ("first", format!("{lc}\n{MD1_VALID}\n{MK1_B}")),
        ("middle", format!("{MD1_VALID}\n{lc}\n{MK1_B}")),
        ("last", format!("{MD1_VALID}\n{MK1_B}\n{lc}")),
    ];
    for (label, input) in &positions {
        let (code, stderr) = run(&["bundle"], input);
        assert_eq!(code, 3, "bundle ms1 {label}: expected exit 3; stderr={stderr}");
        assert!(stderr.contains("CODEX32"), "bundle ms1 {label}: {stderr}");
        assert!(
            !stderr.contains(BODY),
            "bundle ms1 {label}: leaked secret body: {stderr}"
        );
    }
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

// ---------------------------------------------------------------------------
// Task 9: `me bundle --preview <DIR>` wiring (hermetic — fake `me-preview`).
//
// These tests stand up a tiny shell-script `me-preview` in a temp dir and put
// that dir FIRST on PATH. They never build the real Go sidecar (that's the
// Task 10 cross-lang test). Unix-only because the fake is a /bin/sh script;
// the `me` test binary lives in target/debug, which has no `me-preview`, so
// PATH-only discovery is deterministic.
// ---------------------------------------------------------------------------

#[cfg(unix)]
mod preview {
    use super::*;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::{Path, PathBuf};

    fn unique_dir(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!(
            "me-bundle-preview-{tag}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&d).unwrap();
        d
    }

    /// Write an executable fake `me-preview` into `dir`.
    /// - `--version` echoes `me-preview <version_line>`.
    /// - `render` writes a stub SVG to `--out` and echoes `mode text`.
    fn write_fake(dir: &Path, version_line: &str) {
        let path = dir.join("me-preview");
        let script = format!(
            "#!/bin/sh\n\
             if [ \"$1\" = \"--version\" ]; then\n\
             \techo 'me-preview {version_line}'\n\
             \texit 0\n\
             fi\n\
             if [ \"$1\" = \"render\" ]; then\n\
             \tout=\"\"\n\
             \twhile [ \"$#\" -gt 0 ]; do\n\
             \t\tif [ \"$1\" = \"--out\" ]; then out=\"$2\"; fi\n\
             \t\tshift\n\
             \tdone\n\
             \tcat > /dev/null\n\
             \tprintf '<svg/>' > \"$out\"\n\
             \techo 'mode text'\n\
             \texit 0\n\
             fi\n\
             exit 1\n"
        );
        fs::write(&path, script).unwrap();
        let mut perms = fs::metadata(&path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&path, perms).unwrap();
    }

    /// Like `write_fake` but `render` fails non-zero (e.g. a string that fits no
    /// plate). `--version` still matches, so we exercise the RENDER-failure path.
    fn write_fake_render_fail(dir: &Path, version_line: &str) {
        let path = dir.join("me-preview");
        let script = format!(
            "#!/bin/sh\n\
             if [ \"$1\" = \"--version\" ]; then\n\
             \techo 'me-preview {version_line}'\n\
             \texit 0\n\
             fi\n\
             if [ \"$1\" = \"render\" ]; then\n\
             \techo 'string fits no plate' >&2\n\
             \texit 1\n\
             fi\n\
             exit 1\n"
        );
        fs::write(&path, script).unwrap();
        let mut perms = fs::metadata(&path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&path, perms).unwrap();
    }

    fn input() -> String {
        format!("{MD1_VALID}\n{MK1_A}\n{MK1_B}\n")
    }

    // Spec §6: a sidecar RENDER failure (string fits no plate) → exit 4 (invalid
    // input), NOT 2. Version matches; only the render step fails.
    #[test]
    fn render_failure_exit_4() {
        let bindir = unique_dir("renderfail-bin");
        write_fake_render_fail(&bindir, CRATE_VERSION);
        let outdir = unique_dir("renderfail-out");
        Command::cargo_bin("me")
            .unwrap()
            .env("PATH", &bindir)
            .arg("bundle")
            .arg("--preview")
            .arg(&outdir)
            .write_stdin(input())
            .assert()
            .code(4);
    }

    #[test]
    fn matched_version_renders_and_sets_preview_exit_0() {
        let bindir = unique_dir("match-bin");
        write_fake(&bindir, CRATE_VERSION);
        let outdir = unique_dir("match-out");

        let assert = Command::cargo_bin("me")
            .unwrap()
            .env("PATH", &bindir) // only the fake is discoverable
            .arg("bundle")
            .arg("--preview")
            .arg(&outdir)
            .write_stdin(input())
            .assert()
            .success();

        let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
        let v: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
        // 4 plates: md1, mk1-chunk, mk1-chunk, ms1. Public ones get a preview;
        // ms1 must NOT.
        let plates = v["plates"].as_array().unwrap();
        assert_eq!(plates.len(), 4);
        for p in plates {
            if p["kind"] == "ms1" {
                assert!(
                    p.get("preview").is_none(),
                    "ms1 must never be rendered: {p}"
                );
            } else {
                let prev = p["preview"].as_str().expect("public plate has preview");
                assert!(prev.ends_with(".svg"), "svg path expected: {prev}");
                assert!(Path::new(prev).is_file(), "preview file written: {prev}");
            }
        }
        // Exactly 3 svg files (md1 + 2 mk1; not ms1).
        let svgs = fs::read_dir(&outdir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|x| x == "svg").unwrap_or(false))
            .count();
        assert_eq!(svgs, 3, "one svg per public plate, none for ms1");

        fs::remove_dir_all(&bindir).ok();
        fs::remove_dir_all(&outdir).ok();
    }

    #[test]
    fn png_flag_renders_png() {
        let bindir = unique_dir("png-bin");
        write_fake(&bindir, CRATE_VERSION);
        let outdir = unique_dir("png-out");

        let assert = Command::cargo_bin("me")
            .unwrap()
            .env("PATH", &bindir)
            .arg("bundle")
            .arg("--preview")
            .arg(&outdir)
            .arg("--png")
            .write_stdin(input())
            .assert()
            .success();
        let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
        let v: serde_json::Value = serde_json::from_str(&stdout).unwrap();
        let first = v["plates"][0]["preview"].as_str().unwrap();
        assert!(first.ends_with(".png"), "png path expected: {first}");

        fs::remove_dir_all(&bindir).ok();
        fs::remove_dir_all(&outdir).ok();
    }

    // A1/F8: rendering into a dir that already holds a foreign `plate-*` artifact
    // (e.g. a higher-index plate from a prior run) must refuse fail-closed with
    // exit 2, render nothing, and never delete the pre-existing file. Version is
    // matched so control reaches the dirty-dir scan past the locate/version gates.
    #[test]
    fn dirty_preview_dir_refused_exit_2() {
        let bindir = unique_dir("dirty-bin");
        write_fake(&bindir, CRATE_VERSION);
        let outdir = unique_dir("dirty-out");
        // A stale plate from a "prior run" with more plates than this one.
        fs::write(outdir.join("plate-9.svg"), "stale").unwrap();

        let assert = Command::cargo_bin("me")
            .unwrap()
            .env("PATH", &bindir)
            .arg("bundle")
            .arg("--preview")
            .arg(&outdir)
            .write_stdin(input())
            .assert()
            .code(2);
        let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
        assert!(
            stderr.contains(&outdir.display().to_string()),
            "refusal must name the dir: {stderr}"
        );
        // No render happened (plate-1 not written) and the stale file survives.
        assert!(
            !outdir.join("plate-1.svg").is_file(),
            "must not render into a dirty dir"
        );
        assert!(
            outdir.join("plate-9.svg").is_file(),
            "must not delete the pre-existing foreign file"
        );

        fs::remove_dir_all(&bindir).ok();
        fs::remove_dir_all(&outdir).ok();
    }

    #[test]
    fn mismatched_version_exit_2() {
        let bindir = unique_dir("mismatch-bin");
        write_fake(&bindir, "0.0.0-not-the-crate-version");
        let outdir = unique_dir("mismatch-out");

        Command::cargo_bin("me")
            .unwrap()
            .env("PATH", &bindir)
            .arg("bundle")
            .arg("--preview")
            .arg(&outdir)
            .write_stdin(input())
            .assert()
            .code(2)
            .stderr(predicates::str::contains("version"));

        fs::remove_dir_all(&bindir).ok();
        fs::remove_dir_all(&outdir).ok();
    }

    #[test]
    fn absent_sidecar_degrades_exit_0_with_note_and_manifest() {
        // An empty bin dir on PATH (no me-preview) -> locate_sidecar() == None.
        let bindir = unique_dir("absent-bin");
        let outdir = unique_dir("absent-out");

        let assert = Command::cargo_bin("me")
            .unwrap()
            .env("PATH", &bindir)
            .arg("bundle")
            .arg("--preview")
            .arg(&outdir)
            .write_stdin(input())
            .assert()
            .success(); // graceful degrade -> exit 0
        let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
        assert!(
            stderr.contains("preview skipped"),
            "expected skip note: {stderr}"
        );
        // Manifest still emitted on stdout, with NO preview keys.
        let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
        let v: serde_json::Value = serde_json::from_str(&stdout).expect("manifest still emitted");
        for p in v["plates"].as_array().unwrap() {
            assert!(
                p.get("preview").is_none(),
                "no previews when sidecar absent: {p}"
            );
        }

        fs::remove_dir_all(&bindir).ok();
        fs::remove_dir_all(&outdir).ok();
    }

    #[test]
    fn no_preview_flag_is_byte_for_byte_phase_a() {
        // With a fake present on PATH but WITHOUT --preview, output must match
        // Phase A exactly (no preview keys, no sidecar invocation).
        let bindir = unique_dir("noflag-bin");
        write_fake(&bindir, CRATE_VERSION);

        let assert = Command::cargo_bin("me")
            .unwrap()
            .env("PATH", &bindir)
            .arg("bundle")
            .write_stdin(input())
            .assert()
            .success();
        let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
        let mut v: serde_json::Value = serde_json::from_str(&stdout).unwrap();
        v["version"] = serde_json::Value::String("x.y.z".into());
        let golden = include_str!("vectors/bundle-md1-mk1.json");
        let expected: serde_json::Value = serde_json::from_str(golden).unwrap();
        assert_eq!(v, expected, "no --preview must be byte-for-byte Phase A");

        fs::remove_dir_all(&bindir).ok();
    }

    #[test]
    fn unwritable_preview_dir_exit_2() {
        // --preview pointing at a non-existent / unwritable dir: the matched
        // sidecar's render fails -> exit 2.
        let bindir = unique_dir("unwritable-bin");
        write_fake(&bindir, CRATE_VERSION);
        let missing = unique_dir("unwritable-parent").join("does-not-exist");
        // `missing` parent exists but the dir itself does not -> render's --out
        // path is in a missing dir; the fake's `> "$out"` fails -> non-zero.

        Command::cargo_bin("me")
            .unwrap()
            .env("PATH", &bindir)
            .arg("bundle")
            .arg("--preview")
            .arg(&missing)
            .write_stdin(input())
            .assert()
            .code(2);

        fs::remove_dir_all(&bindir).ok();
    }
}

// ---------------------------------------------------------------------------
// A3 (F10): restrictive permissions on written artifacts, plus the I2
// truncate-semantics regression guard. Unix-only (mode bits are POSIX; on
// Windows the write path is a cfg-guarded no-op).
// ---------------------------------------------------------------------------

#[cfg(unix)]
mod perms {
    use super::*;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;

    fn unique_dir(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!(
            "me-perms-{tag}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&d).unwrap();
        d
    }

    // After `me --out f.ndef`, the NDEF file must be owner-only (no group/other).
    #[test]
    fn ndef_out_file_is_owner_only() {
        let dir = unique_dir("ndef");
        let out = dir.join("wallet.ndef");
        Command::cargo_bin("me")
            .unwrap()
            .arg("--out")
            .arg(&out)
            .write_stdin(MD1_VALID)
            .assert()
            .success();
        let mode = fs::metadata(&out).unwrap().permissions().mode();
        assert_eq!(mode & 0o077, 0, "NDEF --out must be owner-only, got {mode:o}");
        fs::remove_dir_all(&dir).ok();
    }

    // After `me bundle --manifest m.json`, the manifest must be owner-only.
    #[test]
    fn manifest_file_is_owner_only() {
        let dir = unique_dir("manifest");
        let out = dir.join("m.json");
        Command::cargo_bin("me")
            .unwrap()
            .arg("bundle")
            .arg("--manifest")
            .arg(&out)
            .write_stdin(format!("{MD1_VALID}\n{MK1_A}\n{MK1_B}\n"))
            .assert()
            .success();
        let mode = fs::metadata(&out).unwrap().permissions().mode();
        assert_eq!(mode & 0o077, 0, "manifest must be owner-only, got {mode:o}");
        fs::remove_dir_all(&dir).ok();
    }

    // I2 regression guard: overwriting a large manifest with a smaller one to the
    // same path must leave no trailing stale bytes (write_private preserves
    // fs::write's truncate). Assert byte-identity with a fresh write of the same
    // small bundle, and that the result is still valid JSON.
    #[test]
    fn manifest_overwrite_shrink_no_trailing_bytes() {
        let dir = unique_dir("shrink");
        let path = dir.join("m.json");
        // Large: md1 + 2 mk1 chunks -> 4 plates.
        Command::cargo_bin("me")
            .unwrap()
            .arg("bundle")
            .arg("--manifest")
            .arg(&path)
            .write_stdin(format!("{MD1_VALID}\n{MK1_A}\n{MK1_B}\n"))
            .assert()
            .success();
        let large_len = fs::metadata(&path).unwrap().len();
        // Smaller, overwritten onto the same path: md1 alone -> md1 + ms1 reminder.
        Command::cargo_bin("me")
            .unwrap()
            .arg("bundle")
            .arg("--manifest")
            .arg(&path)
            .write_stdin(MD1_VALID)
            .assert()
            .success();
        let overwritten = fs::read(&path).unwrap();
        // Fresh write of the same small bundle for a byte-for-byte oracle.
        let fresh = dir.join("fresh.json");
        Command::cargo_bin("me")
            .unwrap()
            .arg("bundle")
            .arg("--manifest")
            .arg(&fresh)
            .write_stdin(MD1_VALID)
            .assert()
            .success();
        let fresh_bytes = fs::read(&fresh).unwrap();
        assert!(
            (overwritten.len() as u64) < large_len,
            "small manifest should be shorter than the large one it overwrote \
             (small={}, large={large_len})",
            overwritten.len()
        );
        assert_eq!(
            overwritten, fresh_bytes,
            "overwrite left trailing stale bytes (missing truncate)"
        );
        serde_json::from_slice::<serde_json::Value>(&overwritten)
            .expect("overwritten manifest must be valid JSON (no trailing bytes)");
        fs::remove_dir_all(&dir).ok();
    }
}
