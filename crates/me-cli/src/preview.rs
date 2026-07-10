//! Discovery, version-check, and spawning of the `me-preview` Go sidecar
//! (Phase B). The sidecar renders public plates to SVG/PNG; `me` never passes
//! secret material to it (ms1 is excluded by the caller). No rendering happens
//! here — this module only locates the binary, confirms it matches our version,
//! and shells out per plate.

use std::io::{self, Read, Write};
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
    /// The sidecar exited 0 but the `--out` file is missing, empty, or lacks the
    /// expected SVG/PNG signature — i.e. it produced no usable render (F9).
    EmptyOutput { path: String, reason: String },
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
            PreviewError::EmptyOutput { path, reason } => {
                write!(f, "me-preview produced no usable render at {path}: {reason}")
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

/// Pure discovery-precedence over already-known candidate paths (first hit wins):
///   1. `explicit` — an operator-vouched path (e.g. from `ME_PREVIEW_BIN`). Its
///      existence is the caller's responsibility; it is returned verbatim.
///   2. `exe_dir` — the directory of the current executable, where release
///      archives ship `me` and `me-preview` side by side (the trusted layout).
///
/// It consults NO `$PATH` and reads NO environment, so an attacker-writable `$PATH`
/// entry can never be auto-selected. Returns `None` when neither yields a file.
fn locate_in(exe_dir: Option<&Path>, explicit: Option<&Path>) -> Option<PathBuf> {
    // 1) An explicit opt-in wins (existence already confirmed by the caller).
    if let Some(p) = explicit {
        return Some(p.to_path_buf());
    }
    // 2) Next to the current executable.
    if let Some(dir) = exe_dir {
        let cand = dir.join(sidecar_filename());
        if cand.is_file() {
            return Some(cand);
        }
    }
    None
}

/// Locate the `me-preview` sidecar for the running `me`.
///
/// Thin wrapper over the pure [`locate_in`]: it supplies the current executable's
/// directory as the co-located search dir and forwards `explicit` (an
/// operator-vouched path, e.g. from `ME_PREVIEW_BIN`, whose existence the caller
/// has already checked).
///
/// Discovery is co-located-only: `$PATH` is deliberately NOT searched. Auto-reaching
/// for the first `me-preview` on an ambient, possibly attacker-writable `$PATH` would
/// hand it the public md1/mk1 payload and let it write into the preview dir (F11), so
/// the only auto-discovered location is the trusted exe-adjacent one. A non-standard
/// install is served by the explicit `ME_PREVIEW_BIN` opt-in instead. Returns `None`
/// when no sidecar is found — the caller degrades gracefully (prints a "preview
/// skipped" note, exits 0).
pub fn locate_sidecar(explicit: Option<&Path>) -> Option<PathBuf> {
    let exe = std::env::current_exe().ok();
    let exe_dir = exe.as_deref().and_then(Path::parent);
    locate_in(exe_dir, explicit)
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

    // F9: exit 0 is necessary but not sufficient — a sidecar that wrote nothing or
    // garbage would otherwise yield a "recorded-valid" preview. Confirm the --out
    // file is a non-empty regular file whose leading bytes carry the format
    // signature (bounded prefix read — never the whole ~150 KB render).
    validate_render_output(&out_path, png)?;

    Ok(out_path.to_string_lossy().into_owned())
}

/// The SVG/PNG signature gate for `render_plate` (F9). Errs with `EmptyOutput` if
/// `out_path` is missing, empty, not a regular file, or lacks the expected
/// signature for the requested format. Reads only a bounded 512-byte prefix into
/// a fixed buffer.
fn validate_render_output(out_path: &Path, png: bool) -> Result<(), PreviewError> {
    let empty = |reason: String| PreviewError::EmptyOutput {
        path: out_path.to_string_lossy().into_owned(),
        reason,
    };
    let meta =
        std::fs::metadata(out_path).map_err(|e| empty(format!("cannot stat output: {e}")))?;
    if !meta.is_file() {
        return Err(empty("output is not a regular file".into()));
    }
    if meta.len() == 0 {
        return Err(empty("output is empty".into()));
    }
    let mut buf = [0u8; 512];
    let n = {
        let mut f = std::fs::File::open(out_path)
            .map_err(|e| empty(format!("cannot reopen output: {e}")))?;
        f.read(&mut buf)
            .map_err(|e| empty(format!("cannot read output: {e}")))?
    };
    let prefix = &buf[..n];
    let ok = if png {
        // PNG magic: 89 50 4E 47 0D 0A 1A 0A.
        prefix.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A])
    } else {
        // SVG: after trimming leading ASCII whitespace, begins with `<svg` or the
        // XML prolog `<?xml`. (A leading UTF-8 BOM would defeat this, but the
        // pinned sidecar emits none — render_svg.go writes `<svg` at byte 0.)
        let t = trim_leading_ascii_ws(prefix);
        t.starts_with(b"<svg") || t.starts_with(b"<?xml")
    };
    if ok {
        Ok(())
    } else {
        let fmt = if png { "png" } else { "svg" };
        Err(empty(format!("output is not a valid {fmt} render")))
    }
}

/// Trim leading ASCII whitespace from a byte slice (no allocation).
fn trim_leading_ascii_ws(b: &[u8]) -> &[u8] {
    let mut i = 0;
    while i < b.len() && b[i].is_ascii_whitespace() {
        i += 1;
    }
    &b[i..]
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
    ///
    /// `render` drains stdin then writes a format-appropriate signature stub to
    /// `--out` — `<svg/>` for `--format svg`, the 8-byte PNG magic for
    /// `--format png` — so its output passes `render_plate`'s F9 signature gate.
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
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&path).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&path, perms).unwrap();
        }
        path
    }

    /// Build a fake sidecar whose `render` drains stdin then runs `render_body`
    /// (a shell snippet with `$out` bound to the `--out` path) and exits 0. Used
    /// to simulate a sidecar that "succeeds" while writing empty/garbage output.
    fn write_fake_sidecar_render(dir: &Path, version_line: &str, render_body: &str) -> PathBuf {
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
             \tcat > /dev/null\n\
             \t{render_body}\n\
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
        // The migrated fake writes a format-appropriate signature stub (not the
        // raw string). The file must be a non-empty SVG that clears the F9 gate.
        let body = fs::read_to_string(&want).unwrap();
        assert!(!body.is_empty(), "render output must be non-empty");
        assert!(
            body.starts_with("<svg"),
            "expected an SVG signature, got: {body:?}"
        );
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

    // F9: a sidecar that exits 0 but writes a 0-byte --out file produced no usable
    // render -> EmptyOutput (not a silently-recorded valid preview).
    #[cfg(unix)]
    #[test]
    fn render_plate_rejects_empty_output() {
        let dir = tmpdir("render-empty");
        let p = write_fake_sidecar_render(&dir, "me-preview 0.3.0", ": > \"$out\"");
        let out_dir = dir.join("out");
        fs::create_dir_all(&out_dir).unwrap();
        let err = retry_txtbsy(|| render_plate(&p, "md1x", &out_dir, 1, false)).unwrap_err();
        assert!(
            matches!(err, PreviewError::EmptyOutput { .. }),
            "expected EmptyOutput, got {err:?}"
        );
        fs::remove_dir_all(&dir).ok();
    }

    // F9: a sidecar that exits 0 writing bytes with no SVG/PNG signature -> EmptyOutput.
    #[cfg(unix)]
    #[test]
    fn render_plate_rejects_garbage_output() {
        let dir = tmpdir("render-garbage");
        let p = write_fake_sidecar_render(&dir, "me-preview 0.3.0", "printf 'garbage' > \"$out\"");
        let out_dir = dir.join("out");
        fs::create_dir_all(&out_dir).unwrap();
        let err = retry_txtbsy(|| render_plate(&p, "md1x", &out_dir, 1, false)).unwrap_err();
        assert!(
            matches!(err, PreviewError::EmptyOutput { .. }),
            "expected EmptyOutput, got {err:?}"
        );
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

    // --- D1 (F11): the pure discovery-precedence helper `locate_in`. It consults
    // NO `$PATH` and reads NO env — the `ME_PREVIEW_BIN` read + existence check live
    // in the wrapper (`wire_previews`). Its only inputs are the two candidate paths.

    /// Touch an empty file named exactly `me-preview` in `dir`; return its path.
    fn touch_co_located(dir: &Path) -> PathBuf {
        let p = dir.join(sidecar_filename());
        fs::write(&p, b"").unwrap();
        p
    }

    // (b) explicit None + exe_dir holds the co-located sidecar -> exe-adjacent hit
    // (the co-located happy path, otherwise uncovered by integration tests).
    #[test]
    fn locate_in_exe_adjacent_hit() {
        let dir = tmpdir("locate-exe");
        let want = touch_co_located(&dir);
        assert_eq!(locate_in(Some(&dir), None), Some(want));
        fs::remove_dir_all(&dir).ok();
    }

    // (c) explicit None + exe_dir present but no file -> None. `$PATH` is never
    // consulted, so a planted `$PATH` binary can never turn this into `Some`.
    #[test]
    fn locate_in_exe_adjacent_miss_is_none() {
        let dir = tmpdir("locate-miss");
        assert_eq!(locate_in(Some(&dir), None), None);
        fs::remove_dir_all(&dir).ok();
    }

    // (c) explicit None + exe_dir None -> None.
    #[test]
    fn locate_in_no_exe_dir_is_none() {
        assert_eq!(locate_in(None, None), None);
    }

    // (a) explicit set wins over a present exe-adjacent sidecar (precedence). The
    // explicit path need not even share the sidecar name — the user vouched for it.
    #[test]
    fn locate_in_explicit_wins_over_exe_adjacent() {
        let exe_dir = tmpdir("locate-prec-exe");
        let _co = touch_co_located(&exe_dir); // a co-located sidecar ALSO present
        let explicit_dir = tmpdir("locate-prec-explicit");
        let explicit = explicit_dir.join("custom-me-preview");
        fs::write(&explicit, b"").unwrap();
        assert_eq!(
            locate_in(Some(&exe_dir), Some(&explicit)),
            Some(explicit.clone())
        );
        fs::remove_dir_all(&exe_dir).ok();
        fs::remove_dir_all(&explicit_dir).ok();
    }

    // (d) explicit is returned verbatim WITHOUT a re-check — existence is the
    // wrapper's responsibility — so it wins even when exe_dir has no sidecar.
    #[test]
    fn locate_in_explicit_returned_verbatim() {
        let explicit_dir = tmpdir("locate-verbatim");
        let explicit = explicit_dir.join("some-me-preview");
        fs::write(&explicit, b"").unwrap();
        let empty = tmpdir("locate-verbatim-empty");
        assert_eq!(
            locate_in(Some(&empty), Some(&explicit)),
            Some(explicit.clone())
        );
        fs::remove_dir_all(&explicit_dir).ok();
        fs::remove_dir_all(&empty).ok();
    }
}
