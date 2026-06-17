//! Discovery, version-check, and spawning of the `me-preview` Go sidecar
//! (Phase B). The sidecar renders public plates to SVG/PNG; `me` never passes
//! secret material to it (ms1 is excluded by the caller). No rendering happens
//! here — this module only locates the binary, confirms it matches our version,
//! and shells out per plate.

use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// The sidecar binary's base name (no extension).
const SIDECAR_STEM: &str = "me-preview";

/// Why a sidecar render failed. The caller maps these to the CLI's exit codes;
/// a *missing* sidecar is NOT an error here (it degrades gracefully upstream).
#[derive(Debug)]
pub enum PreviewError {
    /// Could not spawn / talk to the sidecar process.
    Spawn(io::Error),
    /// The sidecar ran but exited non-zero (e.g. a string that fits no plate).
    Render { code: Option<i32>, stderr: String },
    /// The sidecar's `--version` output could not be parsed.
    VersionParse(String),
}

impl std::fmt::Display for PreviewError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PreviewError::Spawn(e) => write!(f, "cannot run me-preview: {e}"),
            PreviewError::Render { code, stderr } => {
                let c = code
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "signal".into());
                write!(f, "me-preview render failed (exit {c}): {}", stderr.trim())
            }
            PreviewError::VersionParse(s) => {
                write!(f, "cannot parse me-preview --version output: {s:?}")
            }
        }
    }
}
impl std::error::Error for PreviewError {}

/// The file name of the sidecar for the current platform (adds `.exe` on Windows).
fn sidecar_filename() -> String {
    if cfg!(windows) {
        format!("{SIDECAR_STEM}.exe")
    } else {
        SIDECAR_STEM.to_string()
    }
}

/// Locate the `me-preview` sidecar.
///
/// Discovery order (first hit wins):
///   1. The directory containing the current executable (release archives ship
///      `me` and `me-preview` side by side, so this is the trusted/default path).
///   2. Each entry on `$PATH`.
///
/// Returns `None` if neither finds an existing file — the caller then degrades
/// gracefully (prints a "preview skipped" note and exits 0).
pub fn locate_sidecar() -> Option<PathBuf> {
    let name = sidecar_filename();

    // 1) Next to the current executable.
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let cand = dir.join(&name);
            if cand.is_file() {
                return Some(cand);
            }
        }
    }

    // 2) On $PATH.
    if let Some(path) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path) {
            let cand = dir.join(&name);
            if cand.is_file() {
                return Some(cand);
            }
        }
    }

    None
}

/// Run `<sidecar> --version` and return the parsed semver string.
///
/// The sidecar prints `me-preview <version>` to stdout; we strip the `me-preview`
/// prefix and trim. An empty version (a plain `go build` with no `-ldflags`)
/// round-trips as `""`, which the caller treats as a mismatch against the crate
/// version.
pub fn sidecar_version(path: &Path) -> io::Result<String> {
    let out = Command::new(path).arg("--version").output()?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    // Find the first non-empty line that starts with the expected prefix.
    for line in stdout.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("me-preview") {
            return Ok(rest.trim().to_string());
        }
    }
    Err(io::Error::new(
        io::ErrorKind::InvalidData,
        format!("unexpected --version output: {stdout:?}"),
    ))
}

/// Render one plate via the sidecar.
///
/// Spawns `<sidecar> render --format <svg|png> --out <dir>/plate-<idx>.<ext>`,
/// pipes `string` to the sidecar's stdin, and (on success) returns the written
/// output path as a `String`. The sidecar prints `mode <m>` to stdout, which we
/// do not need here — the caller cares only about the path it set in the manifest.
///
/// `idx` is the 1-based plate number used in the file name. `png` selects PNG
/// output; otherwise SVG.
pub fn render_plate(
    sidecar: &Path,
    string: &str,
    dir: &Path,
    idx: usize,
    png: bool,
) -> Result<String, PreviewError> {
    let ext = if png { "png" } else { "svg" };
    let format = if png { "png" } else { "svg" };
    let out_path = dir.join(format!("plate-{idx}.{ext}"));

    let mut child = Command::new(sidecar)
        .arg("render")
        .arg("--format")
        .arg(format)
        .arg("--out")
        .arg(out_path.as_os_str())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(PreviewError::Spawn)?;

    // Pipe the public string to the sidecar's stdin, then close it so the
    // sidecar's io.ReadAll returns. Drop the handle (close) before waiting to
    // avoid a deadlock if the sidecar's stdout/stderr buffers fill.
    //
    // A `BrokenPipe` here means the sidecar exited before consuming stdin (e.g.
    // it bailed on a bad flag). That is NOT our failure to report — fall through
    // to `wait_with_output` so the child's real exit status + stderr surface as
    // a `Render` error instead of masking it with the write's EPIPE.
    {
        let mut stdin = child
            .stdin
            .take()
            .expect("stdin was requested via Stdio::piped()");
        match stdin.write_all(string.as_bytes()) {
            Ok(()) => {}
            Err(e) if e.kind() == io::ErrorKind::BrokenPipe => {}
            Err(e) => return Err(PreviewError::Spawn(e)),
        }
        // stdin dropped here -> EOF on the sidecar's stdin.
    }

    let out = child.wait_with_output().map_err(PreviewError::Spawn)?;
    if !out.status.success() {
        return Err(PreviewError::Render {
            code: out.status.code(),
            stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        });
    }

    Ok(out_path.to_string_lossy().into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsStr;
    use std::fs;

    /// True if `p`'s file name is exactly the platform sidecar name.
    fn has_sidecar_name(p: &Path) -> bool {
        p.file_name() == Some(OsStr::new(sidecar_filename().as_str()))
    }

    /// Retry a closure while it fails with `ETXTBSY` ("Text file busy").
    ///
    /// Writing then immediately exec'ing a script can race with the kernel's
    /// view of the file across parallel test threads — `execve` returns ETXTBSY
    /// (os error 26). This is a *test-harness* artifact (production `me-preview`
    /// is an already-installed binary, never one we just wrote), so a short
    /// bounded retry keeps the hermetic fakes reliable under `cargo test`'s
    /// parallel runner.
    fn retry_txtbsy<T, E, F>(mut f: F) -> Result<T, E>
    where
        F: FnMut() -> Result<T, E>,
        E: IsTxtBusy,
    {
        let mut last = f();
        // Up to ~5s of retries: ETXTBSY is transient (it clears once every write
        // fd to the executable is closed), so a bounded backoff always wins.
        for _ in 0..500 {
            match last {
                Err(ref e) if e.is_txt_busy() => {
                    std::thread::sleep(std::time::Duration::from_millis(10));
                    last = f();
                }
                other => return other,
            }
        }
        last
    }

    trait IsTxtBusy {
        fn is_txt_busy(&self) -> bool;
    }
    impl IsTxtBusy for io::Error {
        fn is_txt_busy(&self) -> bool {
            self.kind() == io::ErrorKind::ExecutableFileBusy
        }
    }
    impl IsTxtBusy for PreviewError {
        fn is_txt_busy(&self) -> bool {
            matches!(self, PreviewError::Spawn(e) if e.kind() == io::ErrorKind::ExecutableFileBusy)
        }
    }

    /// Build a tiny executable shell script that mimics the sidecar's
    /// `--version` and `render` contract, placed at `dir/me-preview`.
    /// `version_line` is echoed verbatim for `--version`.
    fn write_fake_sidecar(dir: &Path, version_line: &str) -> PathBuf {
        let path = dir.join(sidecar_filename());
        let script = format!(
            "#!/bin/sh\n\
             if [ \"$1\" = \"--version\" ]; then\n\
             \techo '{version_line}'\n\
             \texit 0\n\
             fi\n\
             if [ \"$1\" = \"render\" ]; then\n\
             \tout=\"\"\n\
             \twhile [ \"$#\" -gt 0 ]; do\n\
             \t\tif [ \"$1\" = \"--out\" ]; then out=\"$2\"; fi\n\
             \t\tshift\n\
             \tdone\n\
             \tcat > \"$out\"\n\
             \techo 'mode text'\n\
             \texit 0\n\
             fi\n\
             exit 1\n"
        );
        fs::write(&path, script).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&path).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&path, perms).unwrap();
        }
        path
    }

    fn tmpdir(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!(
            "me-preview-test-{tag}-{}-{:?}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn sidecar_filename_has_platform_suffix() {
        let n = sidecar_filename();
        if cfg!(windows) {
            assert_eq!(n, "me-preview.exe");
        } else {
            assert_eq!(n, "me-preview");
        }
    }

    #[test]
    fn has_sidecar_name_matches_only_the_stem() {
        let dir = tmpdir("name");
        let p = write_fake_sidecar(&dir, "me-preview 9.9.9");
        assert!(has_sidecar_name(&p));
        assert!(!has_sidecar_name(&dir.join("me-other")));
        fs::remove_dir_all(&dir).ok();
    }

    #[cfg(unix)]
    #[test]
    fn sidecar_version_parses_prefix() {
        let dir = tmpdir("ver");
        let p = write_fake_sidecar(&dir, "me-preview 1.2.3");
        let v = retry_txtbsy(|| sidecar_version(&p)).unwrap();
        assert_eq!(v, "1.2.3");
        fs::remove_dir_all(&dir).ok();
    }

    #[cfg(unix)]
    #[test]
    fn sidecar_version_empty_when_unset() {
        // A plain `go build` (no -ldflags) prints `me-preview ` with empty ver.
        let dir = tmpdir("ver-empty");
        let p = write_fake_sidecar(&dir, "me-preview ");
        let v = retry_txtbsy(|| sidecar_version(&p)).unwrap();
        assert_eq!(v, "");
        fs::remove_dir_all(&dir).ok();
    }

    #[cfg(unix)]
    #[test]
    fn render_plate_writes_file_and_returns_path() {
        let dir = tmpdir("render");
        let p = write_fake_sidecar(&dir, "me-preview 0.3.0");
        let out_dir = dir.join("out");
        fs::create_dir_all(&out_dir).unwrap();
        let got = retry_txtbsy(|| render_plate(&p, "md1yqpqqxqq8xtwhw4xwn4qh", &out_dir, 2, false))
            .unwrap();
        let want = out_dir.join("plate-2.svg");
        assert_eq!(got, want.to_string_lossy());
        assert!(want.is_file(), "render must write the --out file");
        // The fake `cat > out` writes the piped string verbatim.
        let body = fs::read_to_string(&want).unwrap();
        assert_eq!(body, "md1yqpqqxqq8xtwhw4xwn4qh");
        fs::remove_dir_all(&dir).ok();
    }

    #[cfg(unix)]
    #[test]
    fn render_plate_png_uses_png_extension() {
        let dir = tmpdir("render-png");
        let p = write_fake_sidecar(&dir, "me-preview 0.3.0");
        let out_dir = dir.join("out");
        fs::create_dir_all(&out_dir).unwrap();
        let got = retry_txtbsy(|| render_plate(&p, "md1x", &out_dir, 1, true)).unwrap();
        assert!(got.ends_with("plate-1.png"), "png ext expected: {got}");
        fs::remove_dir_all(&dir).ok();
    }

    #[cfg(unix)]
    #[test]
    fn render_plate_propagates_nonzero_exit() {
        // A sidecar that always exits 1 on render (drop the render branch).
        let dir = tmpdir("render-fail");
        let path = dir.join(sidecar_filename());
        fs::write(&path, "#!/bin/sh\nexit 1\n").unwrap();
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&path, perms).unwrap();
        let err = retry_txtbsy(|| render_plate(&path, "md1x", &dir, 1, false)).unwrap_err();
        assert!(
            matches!(err, PreviewError::Render { .. }),
            "expected Render, got {err:?}"
        );
        fs::remove_dir_all(&dir).ok();
    }
}
