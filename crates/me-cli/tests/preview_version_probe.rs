//! bh-me-I19: `me bundle --preview` and `scripts/build-preview.sh` trust the
//! sidecar's `--version` probe only when it exits 0 and its stdout is the one
//! documented line `me-preview <SEMVER>` (SPEC_me_bundle_phaseB_preview.md
//! §4.1). The stub sidecar logs every invocation, so a test can assert that no
//! render starts after a refused probe.
#![cfg(unix)]

use assert_cmd::Command;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

const CRATE_VERSION: &str = env!("CARGO_PKG_VERSION");
const MD1_VALID: &str = "md1yqpqqxqq8xtwhw4xwn4qh";
const MK1_A: &str = "mk1qpzg69pqqsq3zg3ngj4thnxaq5zg3vs7zqsrqqdt4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4vp3kx98j76m4mjlwphf";
const MK1_B: &str =
    "mk1qpzg69ppsnz4v7cjv3qfjhf76k4t5pt96u0psdrqfqvll8qh7h5athg837pmkf3dpug2mmjtfel6x";

/// A stub `me-preview`: `--version` prints `stdout` verbatim (printf %s, so the
/// caller writes every newline), writes `stderr` to stderr and exits `code`;
/// `render` writes `<svg/>` to `--out` and exits 0. Every call is logged.
fn write_stub(dir: &Path, stdout: &str, stderr: &str, code: i32) -> (PathBuf, PathBuf) {
    let log = dir.join("log");
    let p = dir.join("me-preview");
    let script = format!(
        "#!/bin/sh\necho \"$@\" >> '{log}'\n\
         if [ \"$1\" = \"--version\" ]; then printf '%s' '{stdout}'; printf '%s' '{stderr}' >&2; exit {code}; fi\n\
         while [ $# -gt 0 ]; do if [ \"$1\" = \"--out\" ]; then o=\"$2\"; fi; shift; done\n\
         cat > /dev/null; printf '<svg/>' > \"$o\"; exit 0\n",
        log = log.display(),
    );
    std::fs::write(&p, script).unwrap();
    std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755)).unwrap();
    (p, log)
}

fn bundle_with(stub: &Path) -> std::process::Output {
    let out = tempfile::tempdir().unwrap();
    Command::cargo_bin("me")
        .unwrap()
        .env("ME_PREVIEW_BIN", stub)
        .args(["bundle", "--preview"])
        .arg(out.path())
        .write_stdin(format!("{MD1_VALID}\n{MK1_A}\n{MK1_B}\n"))
        .output()
        .unwrap()
}

fn renders(log: &Path) -> bool {
    std::fs::read_to_string(log)
        .unwrap_or_default()
        .lines()
        .any(|l| l.starts_with("render"))
}

fn stderr(o: &std::process::Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

/// Refused at exit 32 (a named helper that failed), the given stderr line
/// printed, and no render started.
fn assert_refused(stdout: &str, err: &str, code: i32, want_line: &str) {
    let d = tempfile::tempdir().unwrap();
    let (stub, log) = write_stub(d.path(), stdout, err, code);
    let o = bundle_with(&stub);
    assert_eq!(
        o.status.code(),
        Some(32),
        "probe {stdout:?} exit {code} must be refused: {o:?}"
    );
    assert!(
        stderr(&o).lines().any(|l| l == want_line),
        "want stderr line {want_line:?}, got {:?}",
        stderr(&o)
    );
    assert!(!renders(&log), "a render started after a refused probe");
}

#[test]
fn version_probe_nonzero_exit_is_refused_before_any_render() {
    assert_refused(
        &format!("me-preview {CRATE_VERSION}\n"),
        "",
        7,
        "me: cannot determine me-preview version: me-preview --version failed (exit 7)",
    );
}

#[test]
fn version_probe_failure_text_carries_its_stderr() {
    assert_refused(
        &format!("me-preview {CRATE_VERSION}\n"),
        "  probe broke\n",
        7,
        "me: cannot determine me-preview version: me-preview --version failed (exit 7): probe broke",
    );
}

#[test]
fn version_probe_without_separator_is_refused() {
    assert_refused(
        &format!("me-preview{CRATE_VERSION}\n"),
        "",
        0,
        &format!(
            "me: cannot determine me-preview version: unexpected --version output: \"me-preview{CRATE_VERSION}\\n\""
        ),
    );
}

#[test]
fn version_probe_with_leading_space_is_refused() {
    assert_refused(
        &format!(" me-preview {CRATE_VERSION}\n"),
        "",
        0,
        &format!(
            "me: cannot determine me-preview version: unexpected --version output: \" me-preview {CRATE_VERSION}\\n\""
        ),
    );
}

#[test]
fn version_probe_with_a_second_line_is_refused() {
    assert_refused(
        &format!("me-preview {CRATE_VERSION}\nextra\n"),
        "",
        0,
        &format!(
            "me: cannot determine me-preview version: unexpected --version output: \"me-preview {CRATE_VERSION}\\nextra\\n\""
        ),
    );
}

#[test]
fn version_probe_success_renders_and_mismatch_refuses() {
    let d = tempfile::tempdir().unwrap();
    let (stub, log) = write_stub(d.path(), &format!("me-preview {CRATE_VERSION}\n"), "", 0);
    let o = bundle_with(&stub);
    assert!(o.status.success(), "a correct probe must render: {o:?}");
    assert!(renders(&log));
    // stderr noise on a successful probe is diagnostics, not part of the line.
    let d2 = tempfile::tempdir().unwrap();
    let (stub2, log2) = write_stub(
        d2.path(),
        &format!("me-preview {CRATE_VERSION}\n"),
        "note\n",
        0,
    );
    assert!(bundle_with(&stub2).status.success());
    assert!(renders(&log2));
    // A genuine version mismatch keeps its existing refusal.
    let d3 = tempfile::tempdir().unwrap();
    let (stub3, log3) = write_stub(d3.path(), "me-preview 0.12.0\n", "", 0);
    let o3 = bundle_with(&stub3);
    assert_eq!(o3.status.code(), Some(32));
    assert!(stderr(&o3).contains(&format!(
        "me: me-preview version mismatch: sidecar is \"0.12.0\", expected \"{CRATE_VERSION}\""
    )));
    assert!(!renders(&log3));
}

/// A disposable tree holding a copy of scripts/build-preview.sh and a
/// crates/me-cli/Cargo.toml carrying the crate version.
fn script_tree() -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    let r = root.path();
    std::fs::create_dir_all(r.join("scripts")).unwrap();
    std::fs::create_dir_all(r.join("crates/me-cli")).unwrap();
    std::fs::create_dir_all(r.join("target/release")).unwrap();
    std::fs::create_dir_all(r.join("preview")).unwrap();
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/build-preview.sh");
    std::fs::copy(&src, r.join("scripts/build-preview.sh")).unwrap();
    std::fs::write(
        r.join("crates/me-cli/Cargo.toml"),
        format!("[package]\nversion = \"{CRATE_VERSION}\"\n"),
    )
    .unwrap();
    root
}

fn run_check(r: &Path, stdout: &str, err: &str, code: i32) -> std::process::Output {
    let (stub, _log) = write_stub(r, stdout, err, code);
    std::fs::rename(&stub, r.join("target/release/me-preview")).unwrap();
    std::process::Command::new("bash")
        .arg(r.join("scripts/build-preview.sh"))
        .arg("--check")
        .output()
        .unwrap()
}

fn failed_line(r: &Path) -> String {
    format!(
        "FAILED: {} --version exited 7 — run without --check\n",
        r.join("target/release/me-preview").display()
    )
}

#[test]
fn build_preview_check_refuses_failed_probe() {
    let t = script_tree();
    let r = t.path();
    let o = run_check(r, &format!("me-preview {CRATE_VERSION}\n"), "", 7);
    assert_eq!(
        o.status.code(),
        Some(1),
        "--check approved a failed probe: {o:?}"
    );
    assert_eq!(String::from_utf8_lossy(&o.stdout), failed_line(r));
}

#[test]
fn build_preview_check_passes_probe_stderr_through() {
    let t = script_tree();
    let r = t.path();
    let o = run_check(
        r,
        &format!("me-preview {CRATE_VERSION}\n"),
        "probe broke\n",
        7,
    );
    assert_eq!(o.status.code(), Some(1), "{o:?}");
    assert_eq!(String::from_utf8_lossy(&o.stdout), failed_line(r));
    assert_eq!(
        stderr(&o),
        "probe broke\n",
        "the probe's stderr must reach the terminal"
    );
}

#[test]
fn build_preview_check_ignores_stderr_on_a_good_probe() {
    let t = script_tree();
    let o = run_check(
        t.path(),
        &format!("me-preview {CRATE_VERSION}\n"),
        "note\n",
        0,
    );
    assert_eq!(o.status.code(), Some(0), "{o:?}");
    assert_eq!(
        String::from_utf8_lossy(&o.stdout),
        format!("ok: me-preview {CRATE_VERSION} matches the crate\n")
    );
}

#[test]
fn build_preview_check_reports_drift() {
    let t = script_tree();
    let o = run_check(t.path(), "me-preview 0.12.0\n", "", 0);
    assert_eq!(o.status.code(), Some(1), "{o:?}");
    assert_eq!(
        String::from_utf8_lossy(&o.stdout),
        format!("DRIFT: sidecar reports 'me-preview 0.12.0', crate is {CRATE_VERSION} — run without --check\n")
    );
}

/// The build path: a fake `go` writes a stub sidecar whose probe exits 7; the
/// script must stop with its FATAL line, not exit silently with the probe's code.
#[test]
fn build_preview_build_refuses_failed_probe() {
    let t = script_tree();
    let r = t.path();
    let (stub, _log) = write_stub(r, &format!("me-preview {CRATE_VERSION}\n"), "", 7);
    let fake_go = r.join("fake-go");
    std::fs::write(
        &fake_go,
        format!(
            "#!/bin/sh\nwhile [ $# -gt 0 ]; do if [ \"$1\" = \"-o\" ]; then o=\"$2\"; fi; shift; done\n\
             cp '{}' \"$o\"\n",
            stub.display()
        ),
    )
    .unwrap();
    std::fs::set_permissions(&fake_go, std::fs::Permissions::from_mode(0o755)).unwrap();
    let o = std::process::Command::new("bash")
        .arg(r.join("scripts/build-preview.sh"))
        .env("GO", &fake_go)
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(1), "{o:?}");
    assert_eq!(
        stderr(&o),
        "FATAL: built sidecar's --version exited 7\n",
        "the build path must name the failed probe"
    );
}
