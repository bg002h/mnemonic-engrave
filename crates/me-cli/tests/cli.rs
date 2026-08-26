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
        assert_eq!(
            code, 3,
            "bundle ms1 {label}: expected exit 3; stderr={stderr}"
        );
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
// Checklist card identity (PLAN_key_index_legibility §2).
//
// An operator cutting 34 plates cannot otherwise tell whose key is on the one
// in front of them. The two forms below are the ones that are NOT the happy
// path, so they are the ones a regression would silently take away.
//
// All four cards here carry the same policy stub as MD1_VALID (generated with
// `mk encode --from-md1 md1yqpqqxqq8xtwhw4xwn4qh`), which is what lets them
// join that md1 in one bundle.
// ---------------------------------------------------------------------------

/// A `--privacy-preserving` card has NO master fingerprint by design. The label
/// must say so and name the path instead — never fabricate a fingerprint, and
/// never fall back to the anonymous `[unidentified]` form when a path is known.
#[test]
fn checklist_names_privacy_preserving_card_by_path() {
    const P1: &str = "mk1qprx4gpqqqq52a6af5zsfz9jrcw099ckhwsv0jskp2rsal4egz4ep5859p875x67p5s3wem7sgluxl3d2a3syx3m7halwmgkz8syklk577aq";
    const P2: &str = "mk1qprx4gppxlg0x6vnl4rdcjgnpya7k5edv487ph7e30f8tpwunu53n25fsq7a95v5u6dycrf";

    let assert = Command::cargo_bin("me")
        .unwrap()
        .arg("bundle")
        .write_stdin(format!("{MD1_VALID}\n{P1}\n{P2}\n"))
        .assert()
        .success();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();

    assert!(
        stderr.contains("mk1 [path 48'/0'/0'/2', no fingerprint] chunk 1/2"),
        "{stderr}"
    );
    assert!(
        stderr.contains("mk1 [path 48'/0'/0'/2', no fingerprint] chunk 2/2"),
        "{stderr}"
    );
    // One card cannot be ambiguous with itself: its two chunks trivially share
    // an origin, and suffixing them would be the per-plate scan this test also
    // exists to rule out.
    assert!(!stderr.contains(" set 0x"), "{stderr}");
}

/// Two DIFFERENT cards sharing `(fingerprint, path)` render an identical
/// bracket, so the chunk-set id is appended to keep them apart. Without it,
/// plates 2..5 below would read as four chunks of one 2-chunk card.
#[test]
fn checklist_disambiguates_two_cards_sharing_an_origin() {
    const A1: &str = "mk1qp7vlepqqsq52a6afk4thnxaq5zg3vs7rnefw94m5rru59s2su80aw2q4wgdpapgfl4pkhsdyytkwl5z8lphut2hvvpp5kk2m8f6m4kvfc0p";
    const A2: &str =
        "mk1qp7vlepp806lhaeh6reknylagmwyjycf8044xtt9flsdlkvt6f6cthyl98enl9mes92usqz2hd8vy";
    const B1: &str = "mk1qp9kskpqqsq52a6afk4thnxaq5zg3vs78m0n74uevrz28llsnr4qya3jx00arhdt3p75feg52qpln6pv7sunhq8kxupyjh78u6atxdq73a9d";
    const B2: &str =
        "mk1qp9kskppp99zcyrd98kjcu8vgdtu6gt04upy3z0n8ek4aj5kk9satjt7uvk6zejjdmyuez3cwd3jk";

    let assert = Command::cargo_bin("me")
        .unwrap()
        .arg("bundle")
        .write_stdin(format!("{MD1_VALID}\n{A1}\n{A2}\n{B1}\n{B2}\n"))
        .assert()
        .success();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();

    for line in [
        "mk1 [aabbccdd/48'/0'/0'/2'] chunk 1/2 set 0x2da16",
        "mk1 [aabbccdd/48'/0'/0'/2'] chunk 2/2 set 0x2da16",
        "mk1 [aabbccdd/48'/0'/0'/2'] chunk 1/2 set 0xf33f9",
        "mk1 [aabbccdd/48'/0'/0'/2'] chunk 2/2 set 0xf33f9",
    ] {
        assert!(stderr.contains(line), "missing {line:?} in:\n{stderr}");
    }
    // The suffix is the ONLY thing distinguishing the two cards, so an
    // unsuffixed line means the collision scan failed to fire.
    assert!(
        !stderr.contains("chunk 1/2  \u{2192}"),
        "an unsuffixed chunk line survived:\n{stderr}"
    );
}

// ---------------------------------------------------------------------------
// Task 9: `me bundle --preview <DIR>` wiring (hermetic — fake `me-preview`).
//
// These tests stand up a tiny shell-script `me-preview` in a temp dir and point
// `me` at it via the explicit `ME_PREVIEW_BIN` opt-in (F11: discovery is
// co-located-only + this env var; `$PATH` is no longer searched). They never build
// the real Go sidecar (that's the Task 10 cross-lang test). Unix-only because the
// fake is a /bin/sh script; the `me` test binary lives in target/debug, which has
// no co-located `me-preview`, so discovery is deterministic (only ME_PREVIEW_BIN,
// where set, or the absence path).
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
    /// - `render` writes a format-appropriate signature stub to `--out` (`<svg/>`
    ///   for svg, the 8-byte PNG magic for png) and echoes `mode text`, so its
    ///   output clears `render_plate`'s F9 signature gate under both formats.
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
             \tfmt=\"\"\n\
             \twhile [ \"$#\" -gt 0 ]; do\n\
             \t\tif [ \"$1\" = \"--out\" ]; then out=\"$2\"; fi\n\
             \t\tif [ \"$1\" = \"--format\" ]; then fmt=\"$2\"; fi\n\
             \t\tshift\n\
             \tdone\n\
             \tcat > /dev/null\n\
             \tif [ \"$fmt\" = \"png\" ]; then\n\
             \t\tprintf '\\211PNG\\r\\n\\032\\n' > \"$out\"\n\
             \telse\n\
             \t\tprintf '<svg/>' > \"$out\"\n\
             \tfi\n\
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

    /// Like `write_fake` but `render` exits 0 while writing a 0-byte `--out` file
    /// (a sidecar that "succeeds" but produced nothing). `--version` still matches.
    fn write_fake_empty_output(dir: &Path, version_line: &str) {
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
             \t: > \"$out\"\n\
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

    fn input() -> String {
        format!("{MD1_VALID}\n{MK1_A}\n{MK1_B}\n")
    }

    // Spec §A2/F9: a sidecar that exits 0 but writes a 0-byte --out file must NOT
    // yield a recorded preview. render_plate errs (EmptyOutput) -> wire_previews
    // maps it to exit 4 and records no preview path.
    #[test]
    fn empty_sidecar_output_exit_4() {
        let bindir = unique_dir("empty-bin");
        write_fake_empty_output(&bindir, CRATE_VERSION);
        let outdir = unique_dir("empty-out");
        Command::cargo_bin("me")
            .unwrap()
            .env("ME_PREVIEW_BIN", bindir.join("me-preview"))
            .arg("bundle")
            .arg("--preview")
            .arg(&outdir)
            .write_stdin(input())
            .assert()
            .code(4);
        fs::remove_dir_all(&bindir).ok();
        fs::remove_dir_all(&outdir).ok();
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
            .env("ME_PREVIEW_BIN", bindir.join("me-preview"))
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
            .env("ME_PREVIEW_BIN", bindir.join("me-preview")) // only the vouched fake is discoverable
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
            .env("ME_PREVIEW_BIN", bindir.join("me-preview"))
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
            .env("ME_PREVIEW_BIN", bindir.join("me-preview"))
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
            .env("ME_PREVIEW_BIN", bindir.join("me-preview"))
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
        // No ME_PREVIEW_BIN opt-in and no co-located sidecar (the `me` test binary
        // in target/debug has none) -> locate_sidecar() == None -> graceful degrade.
        // env_remove guards against an ambient ME_PREVIEW_BIN in the runner's env.
        let bindir = unique_dir("absent-bin");
        let outdir = unique_dir("absent-out");

        let assert = Command::cargo_bin("me")
            .unwrap()
            .env("PATH", &bindir)
            .env_remove("ME_PREVIEW_BIN")
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

    // F11 (behavioral closure lock): a version-MATCHED `me-preview` planted ONLY on
    // $PATH — with NO co-located sidecar and NO ME_PREVIEW_BIN opt-in — must be
    // IGNORED. Co-located-only discovery no longer walks $PATH, so `me` degrades
    // gracefully (exit 0, "preview skipped", no preview keys) instead of piping the
    // public payload to an ambient $PATH binary. Red before D1 (the old $PATH arm
    // finds + runs the fake -> previews present); green after.
    #[test]
    fn planted_path_sidecar_ignored() {
        let bindir = unique_dir("planted-bin");
        write_fake(&bindir, CRATE_VERSION); // a valid fake, reachable ONLY via $PATH
        let outdir = unique_dir("planted-out");

        let assert = Command::cargo_bin("me")
            .unwrap()
            .env("PATH", &bindir) // the fake is discoverable only on $PATH
            .env_remove("ME_PREVIEW_BIN") // and there is no explicit opt-in
            .arg("bundle")
            .arg("--preview")
            .arg(&outdir)
            .write_stdin(input())
            .assert()
            .success(); // co-located-only + no opt-in -> graceful degrade, exit 0
        let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
        assert!(
            stderr.contains("preview skipped"),
            "a $PATH-only sidecar must be ignored -> skip note: {stderr}"
        );
        // The $PATH fake never ran: manifest carries no preview keys and nothing was
        // written into the output dir.
        let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
        let v: serde_json::Value = serde_json::from_str(&stdout).expect("manifest still emitted");
        for p in v["plates"].as_array().unwrap() {
            assert!(
                p.get("preview").is_none(),
                "a $PATH-only sidecar must not render: {p}"
            );
        }
        assert!(
            !outdir.join("plate-1.svg").is_file(),
            "a $PATH-only sidecar must write nothing"
        );

        fs::remove_dir_all(&bindir).ok();
        fs::remove_dir_all(&outdir).ok();
    }

    // F11/D2: an explicit ME_PREVIEW_BIN that names a path which does not exist is a
    // fail-loud usage error (EXIT_USAGE 2) with a distinct message naming the env var
    // and the path — NOT a silent graceful-degrade and NOT a fall-back to co-located
    // discovery. The user vouched for a specific binary that isn't there.
    #[test]
    fn set_but_missing_me_preview_bin_exit_2() {
        let outdir = unique_dir("missing-bin-out");
        let missing = unique_dir("missing-bin-parent").join("no-such-me-preview");

        let assert = Command::cargo_bin("me")
            .unwrap()
            .env("ME_PREVIEW_BIN", &missing)
            .arg("bundle")
            .arg("--preview")
            .arg(&outdir)
            .write_stdin(input())
            .assert()
            .code(2);
        let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
        assert!(
            stderr.contains("ME_PREVIEW_BIN"),
            "error must name the env var: {stderr}"
        );
        assert!(
            stderr.contains(&missing.display().to_string()),
            "error must name the missing path: {stderr}"
        );
        // It must NOT have silently degraded (no skip note) or rendered.
        assert!(
            !stderr.contains("preview skipped"),
            "set-but-missing must fail loud, not degrade: {stderr}"
        );

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
        // --preview pointing at a non-existent dir: `wire_previews`' `!dir.is_dir()`
        // gate refuses it with EXIT_USAGE(2) before any render is attempted.
        let bindir = unique_dir("unwritable-bin");
        write_fake(&bindir, CRATE_VERSION);
        let missing = unique_dir("unwritable-parent").join("does-not-exist");

        Command::cargo_bin("me")
            .unwrap()
            .env("ME_PREVIEW_BIN", bindir.join("me-preview"))
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
        assert_eq!(
            mode & 0o077,
            0,
            "NDEF --out must be owner-only, got {mode:o}"
        );
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

/// GRAFT 2 — THE EXIT-CODE VOCABULARY IS ONE VOCABULARY.
///
/// `me` has three failure codes and they mean different things:
///
/// | code | meaning | the operator's next move |
/// | --- | --- | --- |
/// | 2 | **usage** — the invocation is wrong; nothing was read, nothing judged | fix the command line |
/// | 3 | **policy refusal** — understood, admissible-looking, and refused on purpose | this tool will never do that |
/// | 4 | **invalid** — the INPUT is not what it must be | fix the input |
///
/// The distinction is load-bearing for scripts: a `2` says try again, a `3`
/// says stop, a `4` says the artifact is wrong. A subcommand that spells the
/// same situation with a different digit makes all three unusable.
///
/// This is a TABLE rather than a set of scattered `.code()` calls precisely
/// because the defect it exists to catch is a DISAGREEMENT between two
/// subcommands, which no per-subcommand test can see.
#[test]
fn the_exit_code_vocabulary_is_one_vocabulary() {
    // A transaction whose one input carries NEITHER scriptSig NOR witness --
    // the stripped form of the pinned "even" vector, same txid as the honest
    // one. Refused as inadmissible input, not as usage.
    const STRIPPED: &str = "02000000017c8da925af70e49a12b0cea7b639df5037c87b7fa61f262b86ac32c47aa3ba1a0000000000fdffffff02404b4c0000000000160014c1de0dd435d1d4ad97ed1f51d63f91c800cc4eab3ea1b92901000000160014751097c299d6354fbb2c5a84512dd708f2902f5e60000000";
    const SEED: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

    let dir = tempfile::tempdir().unwrap();
    let notacontainer = dir.path().join("nope.bin");
    std::fs::write(&notacontainer, b"this is not a systemwide container at all").unwrap();
    let uf2 = dir.path().join("o.uf2");

    struct Case {
        code: i32,
        args: Vec<String>,
        /// `String`, not `&'static str`: the bearer cases have to arrive on a
        /// PRIVATE channel now (G-P3.5), so their stdin is built from a const.
        stdin: String,
        why: &'static str,
    }
    let s = |v: &[&str]| v.iter().map(|x| x.to_string()).collect::<Vec<_>>();
    let cases = vec![
        // ── 2, USAGE ────────────────────────────────────────────────────────
        Case { code: 2, args: s(&["--stdout"]), stdin: "".to_string(),
               why: "no input at all" },
        Case { code: 2, args: s(&[]), stdin: MD1_VALID.to_string(),
               why: "a valid string with no output mode chosen" },
        Case { code: 2, args: s(&["sysw", "pack", "--no-passphrase"]), stdin: "".to_string(),
               why: "no records on argv and no --in" },
        Case { code: 2, args: s(&["sysw", "wipe", "--fill", "sideways"]), stdin: "".to_string(),
               why: "a --fill value that does not exist" },
        Case { code: 2, args: s(&["sysw", "show", "/nonexistent/payload.bin"]), stdin: "".to_string(),
               why: "a file that is not there" },
        Case { code: 2, args: s(&["tx"]), stdin: "".to_string(),
               why: "`me tx` with nothing piped in" },
        // THE ONE THAT DISAGREED. `me seal --iterations 5` exits 2 and
        // `me sysw pack --iterations 5` exited 4, for the same typo on the
        // same flag with the same range in the same message. Both are usage:
        // no input has been read and nothing about the operator's DATA is
        // wrong. Fixed on the sysw side, because `seal` is the shipped one.
        Case { code: 2,
               args: s(&["seal", "--iterations", "5", "--out"]),
               stdin: "".to_string(), why: "seal: --iterations below the floor" },
        Case { code: 2,
               args: s(&["sysw", "pack", "--iterations", "5", "--passphrase-words", "12", SEED]),
               stdin: "".to_string(), why: "sysw pack: --iterations below the floor" },
        Case { code: 2,
               args: s(&["sysw", "pack", "--iterations", "2000001", "--passphrase-words", "12", SEED]),
               stdin: "".to_string(), why: "sysw pack: --iterations above the ceiling" },
        // ── 3, POLICY REFUSAL ───────────────────────────────────────────────
        Case { code: 3, args: s(&["--stdout"]), stdin: MS1.to_string(),
               why: "an ms1 secret: understood, well-formed, and never engraved" },
        // R2 / G-P3.5. Understood, well-formed, and refused on purpose: a raw
        // signed transaction is BEARER and argv is a public channel. Not
        // usage -- the command line is spelled correctly -- and not invalid
        // input, because the same record on --in or stdin packs fine.
        Case { code: 3,
               args: s(&["sysw", "pack", "--no-passphrase", &format!("tx:{STRIPPED}")]),
               stdin: String::new(), why: "a tx: record on argv" },
        // ── 4, INVALID INPUT ────────────────────────────────────────────────
        Case { code: 4, args: s(&["--stdout"]), stdin: "md1notavalidstring".to_string(),
               why: "a string that does not decode" },
        Case { code: 4, args: s(&["sysw", "show"]), stdin: "".to_string(),
               why: "a file that is not a container" },
        Case { code: 4, args: s(&["sysw", "pack", "--no-passphrase", "not a record"]), stdin: "".to_string(),
               why: "a record this container cannot place" },
        // ON STDIN, not argv: argv is refused at 3 before admission is even
        // considered (the case above), so the unsigned-input refusal can only
        // be reached through a private channel.
        Case { code: 4, args: s(&["sysw", "pack", "--no-passphrase"]),
               stdin: format!("tx:{STRIPPED}\n"), why: "a transaction with an unsigned input" },
        Case { code: 4, args: s(&["tx"]), stdin: "abababab".to_string(),
               why: "`me tx` over bytes that are not a transaction" },
    ];

    for c in &cases {
        let mut cmd = Command::cargo_bin("me").unwrap();
        cmd.args(&c.args);
        // The two cases needing a path argument get it here rather than in the
        // table, so the temp dir does not have to outlive a &'static str.
        if c.args.last().map(String::as_str) == Some("--out") {
            cmd.arg(&uf2).arg("text:6869");
        }
        if c.args == ["sysw", "show"] {
            cmd.arg(&notacontainer);
        }
        cmd.write_stdin(c.stdin.clone());
        let out = cmd.output().unwrap();
        assert_eq!(
            out.status.code(),
            Some(c.code),
            "`me {}` ({}) exited {:?}, want {}\nstderr: {}",
            c.args.join(" "),
            c.why,
            out.status.code(),
            c.code,
            String::from_utf8_lossy(&out.stderr)
        );
        // A failure must not put anything on stdout: the pipeline downstream
        // reads stdout, and a half-written artifact is worse than none.
        assert!(
            out.stdout.is_empty(),
            "`me {}` wrote {} bytes to stdout while failing",
            c.args.join(" "),
            out.stdout.len()
        );
    }

    // ...and all three codes are actually exercised, so the table cannot rot
    // into a one-code test without anyone noticing.
    for want in [2, 3, 4] {
        assert!(
            cases.iter().any(|c| c.code == want),
            "the vocabulary test no longer covers exit {want}"
        );
    }
}
