//! **Creating a file that a bearer artifact may be written into.**
//!
//! One function. It is here rather than in [`super::channel`] or [`super::fd`]
//! because both of those declare narrower contracts than an effectful write:
//! `channel` is *"classification ONLY"* and says in as many words that
//! `destination` **never touches a path**, and `fd` is *"what was measured about
//! stdout"* — the read side of one file descriptor. Widening either header to
//! admit a path-writer would cost more than a seventh module does.
//!
//! ## Why `0o600` here is mechanism, when `0o044` and `0o077` are not
//!
//! [`super::fd`] refuses to publish a *disqualifying* mask because `me` rules
//! `0o044` and `mt` rules `0o077`, and the crate does not get to settle which
//! of them is right. **That is a disagreement about somebody else's file.**
//!
//! The mode a tool creates *its own* output at is not that question, and the two
//! consumers do not disagree about it: a file this crate's callers create for a
//! bearer artifact is owner-only. So `0o600` is a constant here rather than a
//! parameter. If a third consumer ever needs a different creation mode, that is
//! the moment to add one — not before, and a parameter added speculatively would
//! be the first place a caller could weaken the mode by accident.
//!
//! ## F-244 — the half that is easy to leave out
//!
//! `OpenOptions::mode()` binds **on create only**. Open an existing file with it
//! and the mode on disk is untouched, so `me sysw pack --out stale.bin` over a
//! target already at `0644` left it at `0644` — and that is the case an operator
//! re-running a command actually hits, not an exotic one.
//!
//! The fix is the `set_permissions` call below, and it is deliberately made on
//! the **open file** rather than on the path. A path-based `set_permissions`
//! names the file a second time, and between the two calls the name can be made
//! to point somewhere else; a handle cannot be redirected once it is open.

/// Write `bytes` to `path`, creating or truncating it owner-only.
///
/// Truncation is load-bearing and not a stylistic echo of `std::fs::write`:
/// without it a **shrinking** overwrite — a smaller manifest written over a
/// larger one — leaves the tail of the old file in place, which for a JSON
/// artifact means trailing bytes after the closing brace. It is done by
/// `set_len(0)` after `set_permissions`, not by `truncate(true)` at open, so a
/// refused chmod leaves the existing file unchanged.
///
/// On non-Unix the mode calls compile out and the create/truncate semantics
/// remain. The threat model is POSIX: mode bits do not mean the same thing
/// elsewhere, and pretending otherwise would be worse than saying so.
pub fn write_private(path: &std::path::Path, bytes: &[u8]) -> std::io::Result<()> {
    use std::fs::OpenOptions;
    use std::io::Write;
    let mut opts = OpenOptions::new();
    opts.write(true).create(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
    }
    let mut f = opts.open(path)?;
    // F-244. `opts.mode()` above did nothing if `path` already existed, so the
    // mode is set a second time on the OPEN FILE -- see the module header for
    // why the handle and not the path.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        f.set_permissions(std::fs::Permissions::from_mode(0o600))?;
    }
    if f.metadata()?.is_file() {
        f.set_len(0)?;
    }
    f.write_all(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;
    use std::path::{Path, PathBuf};
    use std::process::{Command, Output, Stdio};

    #[cfg(unix)]
    fn mode_of(p: &std::path::Path) -> u32 {
        std::fs::metadata(p).unwrap().permissions().mode() & 0o777
    }

    /// Sorted entry names of `d`.
    fn names(d: &Path) -> Vec<String> {
        let mut v: Vec<String> = std::fs::read_dir(d)
            .unwrap()
            .map(|e| e.unwrap().file_name().into_string().unwrap())
            .collect();
        v.sort();
        v
    }

    /// The child harness: the test binary re-executes itself with
    /// `--exact write::tests::child --nocapture`. `ulimit` is `None` (no `sh`),
    /// `Some(true)` (`sh`, SIGXFSZ ignored, `ulimit -f 1`) or `Some(false)`
    /// (`sh`, `ulimit -f 1`, signal left at its default).
    fn run_child(row: &str, target: &Path, stdout: Stdio, ulimit: Option<bool>) -> Output {
        let exe = std::env::current_exe().unwrap();
        let mut cmd = match ulimit {
            None => Command::new(&exe),
            Some(trap) => {
                let mut c = Command::new("sh");
                let script = if trap {
                    "trap '' XFSZ; ulimit -f 1; exec \"$0\" \"$@\""
                } else {
                    "ulimit -f 1; exec \"$0\" \"$@\""
                };
                c.arg("-c").arg(script).arg(&exe);
                c
            }
        };
        cmd.args(["--exact", "write::tests::child", "--nocapture"])
            .env("MNEMONIC_IO_LIB_CHILD", row)
            .env("MNEMONIC_IO_LIB_TARGET", target)
            .stdin(Stdio::piped())
            .stdout(stdout)
            .stderr(Stdio::piped());
        cmd.output().unwrap()
    }

    fn show(r: Option<usize>) -> String {
        match r {
            Some(i) => format!("Some({i})"),
            None => "None".to_string(),
        }
    }

    /// Not a row: the body of the child processes the rows above start. It
    /// returns at once when `MNEMONIC_IO_LIB_CHILD` is unset.
    #[test]
    fn child() {
        let Ok(row) = std::env::var("MNEMONIC_IO_LIB_CHILD") else {
            return;
        };
        let target = PathBuf::from(std::env::var_os("MNEMONIC_IO_LIB_TARGET").unwrap_or_default());
        let r: std::io::Result<String> = match row.as_str() {
            "W1" | "W2" => write_private(&target, &[b'N'; 4096]).map(|()| "Ok".to_string()),
            "W10" => write_private(Path::new("/dev/stdout"), b"via-dev-stdout")
                .map(|()| "Ok".to_string()),
            "W17" => write_private(Path::new("/dev/stdout"), b"via").map(|()| "Ok".to_string()),
            "S7" => Ok(show(same_file(
                Endpoint::Path(&target),
                &[Endpoint::Stdout],
            ))),
            "S8" => Ok(show(same_file(Endpoint::Stdout, &[Endpoint::Stdin]))),
            other => panic!("unknown child row {other}"),
        };
        match r {
            Ok(v) => eprintln!("child-result: {v}"),
            Err(e) => {
                match e.raw_os_error() {
                    Some(n) => eprintln!("child-result: Err raw_os_error={n}"),
                    None => eprintln!("child-result: Err raw_os_error=none"),
                }
                std::process::exit(3);
            }
        }
    }

    const OLD_CARD: &[u8] = b"OLD-VALID-CARD\n";

    /// W1: ENOSPC-class failure (EFBIG via `ulimit -f 1`) keeps the old file.
    #[cfg(unix)]
    #[test]
    fn a_failed_write_leaves_the_old_file_byte_identical() {
        let d = tempfile::tempdir().unwrap();
        let t = d.path().join("card.txt");
        std::fs::write(&t, OLD_CARD).unwrap();
        let out = run_child("W1", &t, Stdio::piped(), Some(true));
        let err = String::from_utf8_lossy(&out.stderr).to_string();
        assert!(
            err.contains("child-result: Err raw_os_error=27"),
            "child stderr: {err}"
        );
        assert_eq!(out.status.code(), Some(3), "child stderr: {err}");
        assert_eq!(
            std::fs::read(&t).unwrap(),
            OLD_CARD,
            "the old file must be byte-identical"
        );
        assert_eq!(names(d.path()), vec!["card.txt".to_string()]);
    }

    /// W2: a run killed mid-write (SIGXFSZ) keeps the old file and leaves one
    /// named owner-only temp.
    #[cfg(unix)]
    #[test]
    fn a_killed_run_leaves_the_old_file_and_a_named_owner_only_leftover() {
        use std::os::unix::process::ExitStatusExt;
        let d = tempfile::tempdir().unwrap();
        let t = d.path().join("card.txt");
        std::fs::write(&t, OLD_CARD).unwrap();
        let out = run_child("W2", &t, Stdio::piped(), Some(false));
        assert_eq!(out.status.signal(), Some(25), "status: {:?}", out.status);
        assert_eq!(std::fs::read(&t).unwrap(), OLD_CARD);
        let others: Vec<String> = names(d.path())
            .into_iter()
            .filter(|n| n != "card.txt")
            .collect();
        assert_eq!(others.len(), 1, "entries: {others:?}");
        let n = &others[0];
        let mid = n
            .strip_prefix("card.txt.")
            .and_then(|r| r.strip_suffix(".tmp"))
            .unwrap_or_else(|| panic!("leftover name {n}"));
        let parts: Vec<&str> = mid.split('.').collect();
        assert!(
            parts.len() == 2
                && parts
                    .iter()
                    .all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit())),
            "leftover name {n}"
        );
        let lp = d.path().join(n);
        assert_eq!(mode_of(&lp), 0o600);
        let body = std::fs::read(&lp).unwrap();
        assert_eq!(body.len(), 512);
        assert!(body.iter().all(|b| *b == b'N'));
    }

    /// W3: a hard-linked target is replaced at the given name only.
    #[cfg(unix)]
    #[test]
    fn a_hard_linked_target_is_replaced_at_that_name_only() {
        use std::os::unix::fs::MetadataExt;
        let d = tempfile::tempdir().unwrap();
        let a = d.path().join("a");
        let b = d.path().join("b");
        std::fs::write(&a, b"old").unwrap();
        std::fs::hard_link(&a, &b).unwrap();
        write_private(&a, b"new").unwrap();
        assert_eq!(std::fs::read(&a).unwrap(), b"new");
        assert_eq!(std::fs::read(&b).unwrap(), b"old");
        assert_eq!(std::fs::metadata(&a).unwrap().nlink(), 1);
        assert_eq!(std::fs::metadata(&b).unwrap().nlink(), 1);
    }

    /// W4: an unwritable directory fails closed before the old file is touched.
    #[cfg(unix)]
    #[test]
    fn an_unwritable_directory_fails_closed_before_the_old_file_is_touched() {
        let d = tempfile::tempdir().unwrap();
        let t = d.path().join("card.txt");
        std::fs::write(&t, b"old").unwrap();
        std::fs::set_permissions(&t, std::fs::Permissions::from_mode(0o644)).unwrap();
        std::fs::set_permissions(d.path(), std::fs::Permissions::from_mode(0o500)).unwrap();
        let probe = d.path().join("probe");
        if std::fs::File::create(&probe).is_ok() {
            std::fs::set_permissions(d.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
            panic!("precondition: the directory is still writable (running as root?)");
        }
        let before = names(d.path());
        let r = write_private(&t, b"new");
        let after = names(d.path());
        let mode = mode_of(&t);
        let bytes = std::fs::read(&t).unwrap();
        std::fs::set_permissions(d.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        let e = r.expect_err("a directory the holder cannot write must refuse");
        assert_eq!(e.raw_os_error(), Some(13));
        assert_eq!(bytes, b"old");
        assert_eq!(mode, 0o644);
        assert_eq!(before, after);
    }

    /// W5: an absent target is created owner-only with no leftover.
    #[test]
    fn an_absent_target_is_created_owner_only_with_no_leftover() {
        let d = tempfile::tempdir().unwrap();
        let t = d.path().join("card.txt");
        write_private(&t, b"bytes").unwrap();
        assert_eq!(std::fs::read(&t).unwrap(), b"bytes");
        #[cfg(unix)]
        assert_eq!(mode_of(&t), 0o600);
        assert_eq!(names(d.path()), vec!["card.txt".to_string()]);
    }

    /// W6: a read-only target is refused and kept.
    #[cfg(unix)]
    #[test]
    fn a_read_only_target_is_refused_and_kept() {
        let d = tempfile::tempdir().unwrap();
        let t = d.path().join("card.txt");
        std::fs::write(&t, b"old").unwrap();
        std::fs::set_permissions(&t, std::fs::Permissions::from_mode(0o400)).unwrap();
        let e = write_private(&t, b"new").expect_err("a read-only target must refuse");
        assert_eq!(e.raw_os_error(), Some(13));
        assert_eq!(std::fs::read(&t).unwrap(), b"old");
        assert_eq!(names(d.path()), vec!["card.txt".to_string()]);
    }

    /// W7: a symlink is written through and stays a link.
    #[cfg(unix)]
    #[test]
    fn a_symlink_is_written_through_and_stays_a_link() {
        let d = tempfile::tempdir().unwrap();
        let a = d.path().join("A");
        let b = d.path().join("B");
        std::fs::create_dir(&a).unwrap();
        std::fs::create_dir(&b).unwrap();
        std::fs::write(b.join("real"), b"old").unwrap();
        let link = a.join("link");
        std::os::unix::fs::symlink("../B/real", &link).unwrap();
        write_private(&link, b"new").unwrap();
        assert_eq!(std::fs::read(b.join("real")).unwrap(), b"new");
        assert_eq!(mode_of(&b.join("real")), 0o600);
        assert!(std::fs::symlink_metadata(&link)
            .unwrap()
            .file_type()
            .is_symlink());
        assert_eq!(
            std::fs::read_link(&link).unwrap(),
            PathBuf::from("../B/real")
        );
        assert_eq!(names(&a), vec!["link".to_string()]);
        assert_eq!(names(&b), vec!["real".to_string()]);
    }

    /// W8: a dangling symlink creates its target owner-only.
    #[cfg(unix)]
    #[test]
    fn a_dangling_symlink_creates_its_target_owner_only() {
        let d = tempfile::tempdir().unwrap();
        let link = d.path().join("link");
        std::os::unix::fs::symlink("missing", &link).unwrap();
        write_private(&link, b"bytes").unwrap();
        let m = d.path().join("missing");
        assert_eq!(std::fs::read(&m).unwrap(), b"bytes");
        assert_eq!(mode_of(&m), 0o600);
        assert!(std::fs::symlink_metadata(&link)
            .unwrap()
            .file_type()
            .is_symlink());
        assert_eq!(
            names(d.path()),
            vec!["link".to_string(), "missing".to_string()]
        );
    }

    /// W9: a FIFO target is written directly (run under `timeout 60`: a wrong
    /// classification can hang here).
    #[cfg(unix)]
    #[test]
    fn a_fifo_target_is_written_directly() {
        use std::io::Read;
        let d = tempfile::tempdir().unwrap();
        let t = d.path().join("pipe");
        let st = Command::new("mkfifo").arg(&t).status().unwrap();
        assert!(st.success(), "mkfifo failed");
        let rp = t.clone();
        let reader = std::thread::spawn(move || {
            let mut v = Vec::new();
            std::fs::File::open(rp)
                .unwrap()
                .read_to_end(&mut v)
                .unwrap();
            v
        });
        write_private(&t, b"fifo-bytes").unwrap();
        assert_eq!(reader.join().unwrap(), b"fifo-bytes");
        assert_eq!(names(d.path()), vec!["pipe".to_string()]);
    }

    /// W10: `/dev/stdout` on a pipe is written directly.
    #[cfg(unix)]
    #[test]
    fn dev_stdout_on_a_pipe_is_written_directly() {
        let d = tempfile::tempdir().unwrap();
        let out = run_child("W10", d.path(), Stdio::piped(), None);
        let so = String::from_utf8_lossy(&out.stdout).to_string();
        let se = String::from_utf8_lossy(&out.stderr).to_string();
        assert!(so.contains("via-dev-stdout"), "child stdout: {so}");
        assert!(se.contains("child-result: Ok"), "child stderr: {se}");
    }

    /// W11: a 250-byte file name is written.
    #[test]
    fn a_250_byte_file_name_is_written() {
        let d = tempfile::tempdir().unwrap();
        let name = "a".repeat(250);
        let t = d.path().join(&name);
        write_private(&t, b"bytes").unwrap();
        assert_eq!(std::fs::read(&t).unwrap(), b"bytes");
        assert_eq!(names(d.path()), vec![name]);
    }

    /// W17: `/dev/stdout` on a regular file is written directly (same inode).
    #[cfg(unix)]
    #[test]
    fn dev_stdout_on_a_regular_file_is_written_directly() {
        use std::os::unix::fs::MetadataExt;
        let d = tempfile::tempdir().unwrap();
        let f = d.path().join("f");
        std::fs::write(&f, b"OLD").unwrap();
        std::fs::set_permissions(&f, std::fs::Permissions::from_mode(0o644)).unwrap();
        let ino = std::fs::metadata(&f).unwrap().ino();
        let sf = std::fs::OpenOptions::new().write(true).open(&f).unwrap();
        let out = run_child("W17", &f, Stdio::from(sf), None);
        let se = String::from_utf8_lossy(&out.stderr).to_string();
        assert!(se.contains("child-result: Ok"), "child stderr: {se}");
        assert_eq!(std::fs::metadata(&f).unwrap().ino(), ino);
        assert!(std::fs::read(&f).unwrap().starts_with(b"via"));
        assert_eq!(mode_of(&f), 0o600);
        assert_eq!(names(d.path()), vec!["f".to_string()]);
    }

    /// **THE GATE, and the pre-existing target is the whole of it.**
    ///
    /// The fresh-file half passes under any implementation that sets a mode at
    /// all, so on its own it proves nothing. The `0644` half is what fails
    /// against `OpenOptions::mode()` alone — measured: without the
    /// `set_permissions` call this assertion reports `0o644` — which makes it a
    /// gate on F-244 rather than a restatement of the doc comment.
    ///
    /// It also pins the CONTENTS, because a function that tightened the mode and
    /// wrote nothing would satisfy a permissions-only test.
    #[cfg(unix)]
    #[test]
    fn an_existing_world_readable_target_is_tightened_not_inherited() {
        let d = tempfile::tempdir().unwrap();

        let fresh = d.path().join("fresh.bin");
        write_private(&fresh, b"new bytes").unwrap();
        assert_eq!(
            mode_of(&fresh),
            0o600,
            "a file this function creates is owner-only"
        );

        let stale = d.path().join("stale.bin");
        std::fs::write(&stale, b"old").unwrap();
        std::fs::set_permissions(&stale, std::fs::Permissions::from_mode(0o644)).unwrap();
        assert_eq!(
            mode_of(&stale),
            0o644,
            "the control: the target really is 0644 before the call"
        );

        write_private(&stale, b"new bytes").unwrap();
        assert_eq!(
            mode_of(&stale),
            0o600,
            "F-244: `0o600` binds on CREATE, so an implementation that only passes \
             it to OpenOptions leaves this at 0644 and reports success"
        );
        assert_eq!(
            std::fs::read(&stale).unwrap(),
            b"new bytes",
            "the new contents must be there -- tightening a file this function \
             failed to write would pass a mode-only assertion"
        );
    }

    /// A **shrinking** overwrite leaves no tail of the previous file.
    ///
    /// Dropping the `set_len(0)` is the tidy-up that looks harmless: the mode
    /// assertions above all still pass, and a smaller JSON manifest written over
    /// a larger one silently acquires trailing bytes after its closing brace.
    #[cfg(unix)]
    #[test]
    fn a_shrinking_overwrite_leaves_no_stale_tail() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("manifest.json");
        write_private(&p, b"{\"a\":1,\"padding\":\"xxxxxxxxxxxxxxxxxxxx\"}").unwrap();
        write_private(&p, b"{\"a\":1}").unwrap();
        assert_eq!(
            std::fs::read(&p).unwrap(),
            b"{\"a\":1}",
            "without set_len(0) the tail of the longer write survives"
        );
    }

    /// S1: the same path twice is the same file.
    #[test]
    fn the_same_path_twice_is_the_same_file() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("f");
        std::fs::write(&p, b"x").unwrap();
        assert_eq!(
            same_file(Endpoint::Path(&p), &[Endpoint::Path(&p)]),
            Some(0)
        );
    }

    /// S2: a hard link is the same file.
    #[cfg(unix)]
    #[test]
    fn a_hard_link_is_the_same_file() {
        let d = tempfile::tempdir().unwrap();
        let a = d.path().join("a");
        let b = d.path().join("b");
        std::fs::write(&a, b"x").unwrap();
        std::fs::hard_link(&a, &b).unwrap();
        assert_eq!(
            same_file(Endpoint::Path(&a), &[Endpoint::Path(&b)]),
            Some(0)
        );
    }

    /// S3: a symlink is the same file as its target.
    #[test]
    fn a_symlink_is_the_same_file() {
        let d = tempfile::tempdir().unwrap();
        let real = d.path().join("real");
        std::fs::write(&real, b"x").unwrap();
        #[cfg(unix)]
        {
            let link = d.path().join("link");
            std::os::unix::fs::symlink("real", &link).unwrap();
            assert_eq!(
                same_file(Endpoint::Path(&link), &[Endpoint::Path(&real)]),
                Some(0)
            );
        }
    }

    /// S4: two files with equal bytes are not the same file.
    #[test]
    fn two_files_with_equal_bytes_are_not_the_same() {
        let d = tempfile::tempdir().unwrap();
        let a = d.path().join("a");
        let b = d.path().join("b");
        std::fs::write(&a, b"same").unwrap();
        std::fs::write(&b, b"same").unwrap();
        assert_eq!(same_file(Endpoint::Path(&a), &[Endpoint::Path(&b)]), None);
    }

    /// S5: an absent output matches an absent input at the same resolved path.
    #[test]
    fn an_absent_output_matches_an_absent_input_at_the_same_resolved_path() {
        let d = tempfile::tempdir().unwrap();
        let dir = d.path().join("dir");
        std::fs::create_dir(&dir).unwrap();
        let out = dir.join(".").join("x");
        let y = dir.join("y");
        let x = dir.join("x");
        assert_eq!(
            same_file(
                Endpoint::Path(&out),
                &[Endpoint::Path(&y), Endpoint::Path(&x)]
            ),
            Some(1)
        );
        assert_eq!(same_file(Endpoint::Path(&out), &[Endpoint::Path(&y)]), None);
    }

    /// S6: the first matching input is reported.
    #[test]
    fn the_first_matching_input_is_reported() {
        let d = tempfile::tempdir().unwrap();
        let out = d.path().join("out");
        let other = d.path().join("other");
        std::fs::write(&out, b"x").unwrap();
        std::fs::write(&other, b"y").unwrap();
        assert_eq!(
            same_file(
                Endpoint::Path(&out),
                &[
                    Endpoint::Path(&other),
                    Endpoint::Path(&out),
                    Endpoint::Path(&out)
                ]
            ),
            Some(1)
        );
    }

    /// S7: stdout redirected to the output file is the same file.
    #[cfg(unix)]
    #[test]
    fn stdout_redirected_to_the_output_file_is_the_same_file() {
        let d = tempfile::tempdir().unwrap();
        let f = d.path().join("f");
        std::fs::write(&f, b"x").unwrap();
        let sf = std::fs::OpenOptions::new().write(true).open(&f).unwrap();
        let out = run_child("S7", &f, Stdio::from(sf), None);
        let se = String::from_utf8_lossy(&out.stderr).to_string();
        assert!(se.contains("child-result: Some(0)"), "child stderr: {se}");
    }

    /// S8: stdin and stdout on pipes never match.
    #[cfg(unix)]
    #[test]
    fn stdin_and_stdout_on_pipes_never_match() {
        let d = tempfile::tempdir().unwrap();
        let out = run_child("S8", d.path(), Stdio::piped(), None);
        let se = String::from_utf8_lossy(&out.stderr).to_string();
        assert!(se.contains("child-result: None"), "child stderr: {se}");
    }

    /// S9: a non-regular path never matches (metadata only, never written).
    #[cfg(unix)]
    #[test]
    fn a_non_regular_path_never_matches() {
        let n = Path::new("/dev/null");
        assert_eq!(same_file(Endpoint::Path(n), &[Endpoint::Path(n)]), None);
    }

    /// S10: an unreadable lookup never matches.
    #[cfg(unix)]
    #[test]
    fn an_unreadable_lookup_never_matches() {
        let d = tempfile::tempdir().unwrap();
        let dir = d.path().join("d");
        std::fs::create_dir(&dir).unwrap();
        let x = dir.join("x");
        std::fs::write(&x, b"x").unwrap();
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o000)).unwrap();
        if std::fs::metadata(&x).is_ok() {
            std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700)).unwrap();
            panic!("precondition: the directory is still readable (running as root?)");
        }
        let r = same_file(Endpoint::Path(&x), &[Endpoint::Path(&x)]);
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700)).unwrap();
        assert_eq!(r, None);
    }
}
