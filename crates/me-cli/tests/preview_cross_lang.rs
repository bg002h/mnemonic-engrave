//! Cross-language preview round-trip (spec §8/§10): the REAL `me-preview` Go
//! sidecar, built from `preview/` with the crate version baked in via
//! `-ldflags -X main.version=…`, drives `me bundle --preview <dir>`. Each PUBLIC
//! plate (md1 + the mk1 chunks) must yield a non-empty SVG containing `<svg` and
//! a `<path`; the ms1 secret plate must NEVER be rendered.
//!
//! Auto-skips cleanly when the Go toolchain is unavailable (mirrors
//! `cross_lang.rs`) so the suite stays green everywhere; runs for real wherever
//! `go` is on PATH. The hermetic fake-sidecar tests in `cli.rs` already cover
//! the version-mismatch / graceful-degrade exit codes — this test stays focused
//! on the real binary round-trip.

use assert_cmd::Command;
use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;

// Phase A vectors (identical to `cli.rs`): md1 + a 2-chunk mk1 set. The bundle
// also synthesizes a trailing ms1 reminder plate, which must stay un-rendered.
const MD1_VALID: &str = "md1yqpqqxqq8xtwhw4xwn4qh";
const MK1_A: &str = "mk1qpzg69pqqsq3zg3ngj4thnxaq5zg3vs7zqsrqqdt4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4vp3kx98j76m4mjlwphf";
const MK1_B: &str =
    "mk1qpzg69ppsnz4v7cjv3qfjhf76k4t5pt96u0psdrqfqvll8qh7h5athg837pmkf3dpug2mmjtfel6x";

/// The crate version the sidecar must match (env at compile time): we bake this
/// exact string into the sidecar so `me-preview --version` == `me`'s version,
/// satisfying the lockstep check `me` performs before rendering.
const CRATE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// `true` iff the Go toolchain is callable.
fn go_available() -> bool {
    StdCommand::new("go").arg("version").output().is_ok()
}

/// A unique scratch directory under the system temp dir.
fn unique_dir(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!(
        "me-preview-xlang-{tag}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// The sidecar binary's platform file name.
fn sidecar_name() -> &'static str {
    if cfg!(windows) {
        "me-preview.exe"
    } else {
        "me-preview"
    }
}

/// Build the REAL sidecar from `preview/` into `dir`, with the crate version
/// baked in via `-ldflags`. Returns the directory holding the binary (so the
/// caller can put it on `me`'s discovery path).
fn build_real_sidecar(dir: &Path) {
    let preview_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../preview");
    let out = dir.join(sidecar_name());
    let status = StdCommand::new("go")
        .args(["build", "-ldflags"])
        .arg(format!("-X main.version={CRATE_VERSION}"))
        .arg("-o")
        .arg(&out)
        .arg(".")
        .current_dir(preview_dir)
        .status()
        .expect("spawn `go build` for me-preview");
    assert!(status.success(), "go build me-preview failed");
    assert!(out.is_file(), "me-preview not produced at {}", out.display());
}

#[test]
fn real_sidecar_renders_public_plates_only() {
    if !go_available() {
        eprintln!("skipping cross-language preview round-trip: `go` is not on PATH");
        return;
    }

    let bindir = unique_dir("bin");
    build_real_sidecar(&bindir);
    let outdir = unique_dir("out");

    // Discovery: `me` looks next to its own exe first, then on $PATH. The `me`
    // test binary lives in target/debug (no sidecar there), so prepend `bindir`
    // to PATH and let discovery find the real sidecar on $PATH.
    let path = match std::env::var_os("PATH") {
        Some(p) => {
            let mut paths = vec![bindir.clone()];
            paths.extend(std::env::split_paths(&p));
            std::env::join_paths(paths).unwrap()
        }
        None => bindir.clone().into_os_string(),
    };

    let assert = Command::cargo_bin("me")
        .unwrap()
        .env("PATH", &path)
        .arg("bundle")
        .arg("--preview")
        .arg(&outdir)
        .write_stdin(format!("{MD1_VALID}\n{MK1_A}\n{MK1_B}\n"))
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("valid manifest JSON");

    // 4 plates: md1, mk1-chunk, mk1-chunk, ms1. The 3 public ones get a populated
    // `preview` path; the ms1 secret plate must NOT.
    let plates = v["plates"].as_array().expect("plates array");
    assert_eq!(plates.len(), 4, "md1 + 2 mk1 chunks + ms1 reminder");

    let mut public_svgs = 0usize;
    for p in plates {
        if p["kind"] == "ms1" {
            assert!(
                p.get("preview").is_none(),
                "ms1 secret must never be rendered: {p}"
            );
            continue;
        }
        let prev = p["preview"]
            .as_str()
            .unwrap_or_else(|| panic!("public plate missing preview: {p}"));
        assert!(prev.ends_with(".svg"), "expected an .svg path: {prev}");

        let body = std::fs::read_to_string(prev).expect("preview file readable");
        assert!(!body.is_empty(), "preview SVG must be non-empty: {prev}");
        assert!(body.contains("<svg"), "missing <svg in {prev}");
        assert!(body.contains("<path"), "missing <path in {prev}");
        public_svgs += 1;
    }
    assert_eq!(public_svgs, 3, "one populated preview per public plate");

    // On disk: exactly 3 SVGs in the output dir, none for ms1.
    let svgs_on_disk = std::fs::read_dir(&outdir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "svg").unwrap_or(false))
        .count();
    assert_eq!(svgs_on_disk, 3, "one SVG file per public plate, none for ms1");

    std::fs::remove_dir_all(&bindir).ok();
    std::fs::remove_dir_all(&outdir).ok();
}
