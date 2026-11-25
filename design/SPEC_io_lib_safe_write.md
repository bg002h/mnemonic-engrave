# SPEC: mnemonic-io-lib safe write (bughunt-1119 row 7, ruling T5)

Author: Claude Opus 5.5 (claude-opus-5-5), 2026-11-22, as implementer-author. Status: DRAFT for the Opus spec
review. Every point marked **PROPOSED** is the author's decision and is the review's to accept or overturn.

Heads read (default branches, fetched 2026-11-22):

| repo | head | io-lib pin |
| --- | --- | --- |
| mnemonic-engrave (me, this repo) | `7caf903a` | path dep, crate at 0.1.1 |
| descriptor-mnemonic (md) | `1307aeaf` | registry `"0.1.0"`, lock 0.1.0, vendored |
| mnemonic-key (mk) | `b6b8286` | registry `"0.1.0"`, lock 0.1.0, vendored |
| mnemonic-toolkit (tk) | `dbac472f` | registry `"0.1.0"`, lock 0.1.0, vendored |
| mnemonic-secret (ms) | `423f173` | git rev `d8fec678` (0.1.1), vendored |
| mnemonic-transaction (mt) | `4d1461c` | git rev `d8fec678` (0.1.1), not vendored |

crates.io carries `mnemonic-io-lib` 0.1.0 only (API read 2026-11-22); the tag `mnemonic-io-lib-v0.1.1` exists and
was never published.

## 1. The defect and the ruling

`write_private` (crates/mnemonic-io-lib/src/write.rs:47-70) opens the target, sets 0600 on the handle, `set_len(0)`
and then `write_all`. A write that fails after the truncation (ENOSPC, quota, EFBIG) leaves the old file cut to a
fragment, and a hard-linked target is rewritten under every name. Measured on `7caf903a` (Linux, non-root): a
4096-byte write over a 15-byte file under `ulimit -f 1` returns `File too large (os error 27)` and leaves 512 bytes of
the NEW data at the target; a killed run (SIGXFSZ) leaves the same; a write over one name of a hard-linked pair
changes both names.

Ruling T5 (architect, `coordinator/agent-reports/architect-bughunt-1119-questions.md` T5): fix once, here; write a
sibling temp file and rename it over the target; keep the old file byte-identical on any error; add `same_file` for
the alias rows. Findings closed or unblocked: md-C4, mk-I1, mk-I6, ms-I1, ms-I13, mt-C3, mt-I4, me-I5, me-I10,
tk-I3 (bughunt-1119 row 7).

## 2. API (all in module `mnemonic_io_lib::write`; no root re-export, as today)

1. `pub fn write_private(path: &std::path::Path, bytes: &[u8]) -> std::io::Result<()>`. Signature unchanged, so
   every call site compiles unchanged.
2. `pub enum Endpoint<'a> { Path(&'a std::path::Path), Stdin, Stdout }`, deriving `Debug, Clone, Copy`.
   **PROPOSED** (name and the `Stdin` variant; `Stdin` covers `--out X < X`).
3. `pub fn same_file(out: Endpoint<'_>, inputs: &[Endpoint<'_>]) -> Option<usize>`: the index of the FIRST entry of
   `inputs` that is the same file as `out`, or `None`. **PROPOSED** (returns the index so a caller can name the
   input in its refusal; never an error, see section 5).
4. The crate's `[dependencies]` table stays empty (std only). `tempfile` stays the only dev-dependency.
5. The module doc and `crates/mnemonic-io-lib/src/lib.rs:73` gain one sentence each saying the module also answers "is this output one of my
   inputs". `channel.rs` doc lines that describe `write_private` as "tightening the OPEN file" are updated to say the
   new file is created 0600 and renamed over the old one.

**Error type and texts.** `std::io::Error`, unchanged. **The library authors no error text**: every error
`write_private` returns is the `std::io::Error` of the system call that failed, unmodified, so its `Display` is the
OS text (Linux examples: `Permission denied (os error 13)`, `File too large (os error 27)`, `No space left on device
(os error 28)`, `File exists (os error 17)`, `Device or resource busy (os error 16)`, `Invalid cross-device link (os
error 18)`). Each consumer's existing wrapper (section 9) puts it in its own sentence and exit code, as today.
**PROPOSED.** The only library-made errors are the test-only injected ones (section 8, rows W13), whose text is
`injected at <Step>`.

## 3. `write_private`: the decided algorithm

Terms. `ATOMIC` = the temp-and-rename path below. `DIRECT` = today's body verbatim (open write+create, 0600 at
create on Unix, `set_permissions(0600)` on the open handle on Unix, `set_len(0)` if the handle is a regular file,
`write_all`).

1. **Classify** with `std::fs::metadata(path)` (follows symlinks):
   - **Stdio alias (Unix), checked first.** If `path` as given names a descriptor of this process (section 3a:
     `/dev/stdin`, `/dev/stdout`, `/dev/stderr`, `/dev/fd/N` or `/proc/self/fd/N` and its variants): ALIAS. So is a path whose canonical
     parent joined with its name is one of those, or whose last component's symlink chain reaches one.
     Any other `/proc/` path is a name like any other (section 3a).
   - `Err` of kind `NotFound`: mode ABSENT.
   - any other `Err`: DIRECT (the open then reports its own error, as today).
   - `Ok` and not `is_file()` (FIFO, char or block device, socket, directory, a pipe or tty behind `/dev/stdout`):
     DIRECT.
   - `Ok` and `is_file()`: mode REPLACE.
2. **Resolve the final name** by walking the last component's symlink chain: while `symlink_metadata(p)` says
   symlink, `p = read_link(p)` if absolute, else `parent(p).join(read_link(p))`. At most 40 hops; a 41st hop, an
   error from `read_link` or from `symlink_metadata`, gives DIRECT. A hop whose path is a stdio alias (step 1) gives
   DIRECT. Directory components are not canonicalised: the temp goes in the directory
   that holds the final name. If the final name has no `file_name()` (for example `x/..`): DIRECT.
3. **Check the final name still matches the classification.** REPLACE: `metadata(final)` must be `Ok`, a regular
   file, and on Unix have the same `(dev, ino)` as step 1; otherwise DIRECT (this sends any race to today's path). ABSENT: `symlink_metadata(final)` must be `NotFound`; otherwise
   DIRECT.
4. **Create the temp** in `dir = final.parent()` (`.` when empty) named `<name>.<pid>.<n>.tmp`, where `<name>` is
   the final file name when it is at most 200 bytes long (`as_encoded_bytes().len()`), else the literal
   `mnemonic-io-lib`; `<pid>` is `std::process::id()` in decimal; `<n>` runs 0 to 99 within this call. Open with
   `write(true).create_new(true)` and, on Unix, mode 0600. On `AlreadyExists` try the next `<n>`; after `n = 99`
   return that last `AlreadyExists` error. Any other error is returned at once (nothing was created, the old file
   was not opened: this is where an unwritable directory fails closed). **PROPOSED** (visible, not dot-hidden, so a
   crash leftover sits next to the file the holder looks for; the counter is per call, not process-wide, so tests
   are deterministic).
5. **Preflight (REPLACE only).** Open `final` with `write(true)` only (no create, no truncate) and, on Unix,
   `set_permissions(0600)` on that handle; drop it. On error: remove the temp and return the error unchanged. This
   keeps today's two refusals exactly: a read-only target (`Permission denied`) and a target owned by someone else
   (`Operation not permitted` from the chmod). The old bytes are never touched; on Unix the old file's mode is 0600
   from here on even if a later step fails (today's code also chmodded it before writing). **PROPOSED** (without
   the preflight, the rename would silently replace a file the holder made read-only, a promise change).
6. On Unix, `set_permissions(0600)` on the temp handle (umask cannot loosen or tighten the result; parity with
   F-244).
7. `write_all(bytes)`, then `sync_all()`, then drop (close) the handle. On Apple targets a `sync_all` error with `raw_os_error() == Some(45)` (ENOTSUP) is followed by a plain `fsync(2)`, whose result is step 7's result (section 6, macOS).
8. `std::fs::rename(temp, final)`. No copy fallback: a rename that fails (a bind-mounted target, `EXDEV`, `EBUSY`,
   Windows sharing violation) is a refusal. **Rename across filesystems = refuse.**
9. On any error in steps 6 to 8 (and step 5, above): `remove_file(temp)` (its own error ignored), then return the step's error. The old
   file is byte-identical.
10. On Unix, after a successful rename: open `dir` read-only and `sync_all()` it. **Its error is ignored and
    `write_private` returns `Ok`.** **PROPOSED**: after the rename the new bytes are the file; reporting failure here
    would tell the holder the old file survived when it did not, which breaks the ruling's "on any error the old
    file is byte-identical". Some filesystems refuse a directory fsync (`EINVAL`), and that must not fail a write.
11. Return `Ok(())`.

Not done, on purpose: no cleanup of stale temps from earlier runs (another live process may own one; pids repeat);
no preservation of the old file's owner, group, ACLs or xattrs (the new file is the writer's, 0600, as the F-244
tightening already intends); no `sync_all` in DIRECT (today has none).

## 3a. ALIAS (0.1.3; architect-io-lib-append-1124, narrowed by architect-io-lib-proc-alias-1124)

An alias names a stream the process already holds: it is written into that stream, never truncated.
The aliases (Unix) name a descriptor N of THIS process: `/dev/stdin|stdout|stderr` (0, 1, 2), `/dev/fd/N`, and
`/proc/W/fd/N` with W = `self`, `thread-self`, the own pid, `self/task/T` or `<own pid>/task/T`; N and T are ASCII digits.
The own pid is the one this `/proc` numbers (`readlink /proc/self`), not `getpid()`: they differ in a pid namespace that kept its parent's `/proc`.
Except for `/dev/stdin|stdout|stderr`, the name must exist (`symlink_metadata`): `/dev/fd/01` or a closed N is not an alias.
A name whose alias is found only through its canonical parent must exist as given too: `/dev/fd/1/.` is ENOTDIR to the kernel and is not an alias.
Every N is duplicated (`F_DUPFD_CLOEXEC`): the write lands at the stream's own offset and keeps its O_APPEND;
it first flushes std's stdout, under its lock (an error is returned only for N = 1). A descriptor not open for writing (`fcntl(F_GETFL)` access mode `O_RDONLY`, as `/dev/stdin < f`) is refused with EBADF before its file's mode is touched. Then 0600 on the handle and `write_all`: no `set_len`, no temp, no sync.
A failed write can leave part of the bytes in the stream at its offset (after the earlier contents when it appends).
Any other `/proc/` path (`/proc/self/cwd/x`, `/proc/self/root/...`, `/proc/<another pid>/...`) is not an alias:
it is classified by steps 1 to 3 like any other name, so a regular file there is replaced ATOMIC, never appended to.

## 4. Every case, decided

| case | path | result for the holder |
| --- | --- | --- |
| absent target, parent exists | ATOMIC | new file, 0600; no other entry left |
| absent target, parent missing | ATOMIC, step 4 fails | `No such file or directory (os error 2)`, as today |
| existing regular file, writable | ATOMIC | replaced, 0600; a failure at any step keeps the old bytes |
| existing regular file, mode 0400 | preflight (step 5) refuses, temp removed | `Permission denied (os error 13)`, old kept, as today |
| existing regular file owned by another user | preflight (step 5) refuses (chmod), temp removed | `Operation not permitted (os error 1)`, as today |
| shrinking overwrite | ATOMIC | exact new bytes, no tail (as today) |
| symlink to a regular file | ATOMIC on the final name | target replaced; the link stays a link to it |
| dangling symlink | ATOMIC (ABSENT) on the final name | target created 0600 in the target's directory; link kept (today: same) |
| symlink loop | step 1 `Err` (ELOOP) | DIRECT, which reports `Too many levels of symbolic links (os error 40)` |
| hard-linked target | ATOMIC | replaced at the given name only; the other name keeps the old bytes (CHANGE, ruled) |
| directory without write permission, writable file in it | step 4 fails | `Permission denied (os error 13)`; old file untouched, bytes and mode (CHANGE, ruled) |
| ENOSPC / quota / EFBIG during the write | step 7 fails | the OS error; temp removed; old kept (the fix) |
| failure at fsync of the data | step 7 fails | same (on Apple, ENOTSUP from `F_FULLFSYNC` first retries with a plain `fsync`; only that one's failure refuses) |
| run killed mid-write | none | old kept; one 0600 leftover `<name>.<pid>.<n>.tmp` beside it (new, named) |
| rename refused (bind-mounted target, cross-device, busy, Windows sharing) | step 8 fails | the OS error (a bind-mounted target: `Device or resource busy (os error 16)`, measured); temp removed; old kept |
| directory fsync fails after the rename | step 10 | ignored; `Ok` |
| FIFO | DIRECT | written through, as today |
| char device (`/dev/null`, a tty) | DIRECT | as today: `/dev/null` as non-root refuses with `Operation not permitted (os error 1)` (measured); see section 11 |
| `/dev/stdout` when stdout is a pipe or tty | ALIAS | written into descriptor 1 |
| `/dev/stdout` (or `/dev/fd/N`, `/proc/self/fd/N`, or a link to one) when stdout is a regular file | ALIAS | written into the stream at its offset; same inode, never truncated, 0600 (CHANGE in 0.1.3; 0.1.2 truncated it) |
| `/dev/stdout` when stdout is a deleted file | ALIAS | written into descriptor 1 |
| `/dev/fd/N` (N > 2) on a regular file (`3>f`, `3>>f`) | ALIAS | written into descriptor N at its offset, as the shell left it; never reopened |
| `/proc/self/cwd/x`, `/proc/self/root/...` or another process's `/proc/<pid>/...` onto a regular file | ATOMIC (a file name, not an alias) | replaced: exactly the new bytes, 0600 (CHANGE from the 0.1.3 draft bfa9a0a2, which appended) |
| a descriptor name the kernel lacks (`/dev/fd/01`, a closed N) | not an alias | the ATOMIC path's error, `No such file or directory (os error 2)` (measured) |
| the path `-` | ATOMIC on a file named `-` | the library gives `-` no meaning; callers route `-` to stdout before calling (consumer FIX rows check theirs) |
| file name over 200 bytes | ATOMIC, temp named `mnemonic-io-lib.<pid>.<n>.tmp` | written (a suffixed temp name would exceed NAME_MAX 255) |
| 100 taken temp names | step 4 fails | `File exists (os error 17)`; old kept |
| empty `bytes` | ATOMIC | an empty 0600 file |

## 5. `same_file` semantics

Each endpoint is reduced to an identity:

- `Path(p)`: `metadata(p)` (follows symlinks). Regular file: on Unix `Id(dev, ino)`; on Windows `Canon(canonicalize(p))`
  (an error gives `Unknown`). Not a regular file: `NonRegular`. `NotFound`: `Planned(c)`, where `c` is
  `canonicalize(dir).join(name)` for the final name of section 3 step 2 (`dir` empty means `.`; a walk that gives up,
  a missing `file_name`, or a `canonicalize` error gives `Unknown`). Any other error: `Unknown`.
- `Stdin` / `Stdout` on Unix: the metadata of fd 0 / fd 1 (a duplicated `OwnedFd` from `as_fd().try_clone_to_owned()`,
  turned into a `File`; std only). Regular: `Id(dev, ino)`; otherwise `NonRegular`; an error (fd closed): `Unknown`.
  On Windows: `Unknown`.

Two identities match only as `Id == Id`, `Canon == Canon` or `Planned == Planned` (byte-equal paths). `NonRegular`
and `Unknown` never match anything, so a terminal or pipe shared by stdin and stdout is never "the same file", and a
lookup that fails never causes a refusal (the write or read that follows reports it). `same_file` returns the index of
the first input matching `out`, else `None`; an `out` that is `NonRegular` or `Unknown` gives `None`.

Limits, documented in the doc comment: two `Planned` paths that differ only in letter case on a case-insensitive
filesystem do not match; on Windows a hard link and a redirected stdin or stdout are not detected (std does not
expose a stable file id there). **PROPOSED.**

## 6. Windows (CI builds `me` for Windows; md and mk run their tests on a Windows leg)

Same algorithm with these differences: no mode bits (steps 5 and 6 skip the chmod; the preflight still opens for
write, so a read-only-attribute file is refused as today); no directory fsync (step 10 skipped; std cannot open a
directory handle without non-std flags); no `(dev, ino)` check in step 3 (the final name only has to be a regular
file); `std::fs::rename` replaces an existing file, and fails if another process holds the file open without
delete sharing, which is a refusal with the old file kept. `same_file` per section 5. **PROPOSED.**

**macOS (Apple targets).** std's `sync_all` is `fcntl(F_FULLFSYNC)` with no fallback, and SMB and some FUSE volumes refuse it with ENOTSUP (45). On that error step 7 calls `fsync(2)` itself (declared `extern "C"` against libSystem, which std already links; no dependency, one `unsafe` call, retried on `EINTR`) and uses its result, as Go (go#64215) and SQLite do. Any other `sync_all` error refuses as on every platform. So `--out` to an SMB share keeps working and still flushes before the rename. **RULED** (architect-me41-s1-i1).

## 7. Holder-visible changes, in one list

1. A failed `--out` write keeps the previous file byte-identical (the fix; md-C4, mk-I1, ms-I13, mt-C3, me-I5, tk-I3).
2. A hard-linked `--out` target is replaced at that name only (ruled).
3. An existing file in a directory the holder cannot write is now refused, `Permission denied` (ruled).
4. A run killed mid-write leaves `<name>.<pid>.<n>.tmp`, 0600, beside the target (new).
5. The replaced file is a new file owned by the writer (today: the same inode, chmodded). A root run (sudo) over
   another user's file leaves it root-owned 0600, so that user can no longer read it; today the owner is kept.
6. A target that is itself a mount point (one file bind-mounted, for example `docker run -v "$PWD/card.txt:/out/card.txt"`)
   is refused with `Device or resource busy (os error 16)`; today it is overwritten in place. The old file is kept.
   Write to a path inside a mounted directory instead.
No wrapper text changes; the OS error inside it is new for items 3 and 6 (and an absent parent directory `x/` now
reads `Not a directory (os error 20)` where today it reads `Is a directory (os error 21)`).
Exit codes are unchanged.

## 8. Test rows (TRACE)

All rows live in `crates/mnemonic-io-lib/src/write.rs` `mod tests` (the fault seam is private). The module
attribute becomes `#[cfg(test)]` (today `#[cfg(all(test, unix))]`); `use std::os::unix::fs::PermissionsExt`, `mode_of`
and the two existing rows get `#[cfg(unix)]`. Rows marked (unix) are `#[cfg(unix)]`; the rest compile on every
platform. Child rows re-execute the test binary
(`std::env::current_exe()`, args `--exact write::tests::child --nocapture`) with `MNEMONIC_IO_LIB_CHILD=<row id>`
(`W1`, `W2`, `W10`, `W17`, `W19`-`W25`, `W28`-`W30`, `S7`, `S8`, `S11`) and `MNEMONIC_IO_LIB_TARGET=<path>` (unused by S8). `fn child()` is a `#[test]`
that returns at once when `MNEMONIC_IO_LIB_CHILD` is unset. The child reports on STDERR one line
`child-result: <value>`, where `<value>` is `Ok`, `Some(<i>)`, `None` or `Err raw_os_error=<n>`; after an `Err` it
calls `std::process::exit(3)`. The parent gives the child piped stdin and stderr, and the stdout the row names (piped
unless the row says otherwise; W17 and S7 give a `File`). A pipe is not subject to RLIMIT_FSIZE, so W1's report
survives the limit (a child whose stderr is a regular file over the size limit dies writing its own harness output,
measured, exit 101). The ulimit is applied through
`sh -c` (`trap '' XFSZ; ulimit -f 1; exec "$0" ...` for W1, the same without the trap for W2); the child writes 4096
bytes of `N` over a 15-byte old file `OLD-VALID-CARD\n`.

RED is two commits. RED-1 adds W1-W4 and the GREEN guards W5-W11 and W17 (all compile against 7caf903a): W1-W4 fail as
"measured today", W5-W11 and W17 pass. RED-2 adds S1-S10: the build fails with E0425 on `same_file`; record that output.
W12-W16 and W18 land in the GREEN commit, with the seam and the temp-name function they call. Measured results on
`7caf903a` are given.

| row | kind | setup | expected after the fix | measured today |
| --- | --- | --- | --- | --- |
| W1 `a_failed_write_leaves_the_old_file_byte_identical` (unix) | RED | EFBIG child | child exits 3 after reporting `child-result: Err raw_os_error=27`; target bytes are the old 15; directory holds only the target | 512 bytes of `N`; RED |
| W2 `a_killed_run_leaves_the_old_file_and_a_named_owner_only_leftover` (unix) | RED | SIGXFSZ child | child killed by signal 25; target bytes are the old 15; exactly one other entry, matching `card.txt.<digits>.<digits>.tmp`, mode 0600, 512 bytes, all `N` | target 512 bytes of `N`, no other entry; RED |
| W3 `a_hard_linked_target_is_replaced_at_that_name_only` (unix) | RED | `a` = `old`, `b` hard link to `a`; write `new` to `a` | `a` = `new`, `b` = `old`, nlink of each 1 | both `new`, nlink 2; RED |
| W4 `an_unwritable_directory_fails_closed_before_the_old_file_is_touched` (unix) | RED | dir 0500 holding `card.txt` 0644 `old`; precondition: creating a probe file in the dir fails, else the row panics `precondition: the directory is still writable (running as root?)` | `Err` with `raw_os_error()` 13; bytes `old`; mode still 0644; dir entries unchanged; dir restored to 0700 at the end | `Ok`, bytes `new`; RED |
| W5 `an_absent_target_is_created_owner_only_with_no_leftover` | GREEN guard | empty dir | `Ok`; bytes; mode 0600 (unix); dir holds only the target | GREEN |
| `an_existing_world_readable_target_is_tightened_not_inherited` | GREEN guard, existing | unchanged | unchanged | GREEN |
| `a_shrinking_overwrite_leaves_no_stale_tail` | GREEN guard, existing | unchanged | unchanged | GREEN |
| W6 `a_read_only_target_is_refused_and_kept` (unix) | GREEN guard | `card.txt` 0400 `old` | `Err` `raw_os_error()` 13; bytes `old`; dir holds only the target | GREEN (os 13, `old`) |
| W7 `a_symlink_is_written_through_and_stays_a_link` (unix) | GREEN guard | dir A: `link` -> `../B/real` (relative); dir B: `real` = `old` | `real` = `new`, mode 0600; `link` still a symlink with the same text; A holds only `link`, B only `real` | GREEN |
| W8 `a_dangling_symlink_creates_its_target_owner_only` (unix) | GREEN guard | `link` -> `missing` | `Ok`; `missing` exists, 0600, bytes; `link` still a symlink; dir holds `link` and `missing` only | GREEN |
| W9 `a_fifo_target_is_written_directly` (unix) | GREEN guard | `mkfifo` (std has none; `std::process::Command::new("mkfifo")`), reader thread | reader receives exactly the bytes; dir holds only the FIFO | GREEN |
| W10 `dev_stdout_on_a_pipe_is_written_directly` (unix) | GREEN guard | child with piped stdout calls `write_private("/dev/stdout", b"via-dev-stdout")` | child stdout contains `via-dev-stdout`; child reports `child-result: Ok` | GREEN |
| W11 `a_250_byte_file_name_is_written` | GREEN guard | target name of 250 `a` bytes | `Ok`; bytes; dir holds only the target | GREEN |
| W17 `dev_stdout_on_a_regular_file_is_written_directly` (unix) | GREEN guard | child with stdout = `File` of `f` (holding `OLD`, mode 0644) calls `write_private("/dev/stdout", b"via")` | `f` keeps its inode number, contains `via` (libtest's own stdout lines may precede; never truncated), mode 0600; dir holds only `f` | GREEN |
| W19 `dev_fd_1_on_a_regular_file_is_written_directly` (unix) | new | child with stdout = `File` of `f` (holding `OLD`, mode 0644) calls `write_private("/dev/fd/1", b"via")` | `f` keeps its inode number and contains `via`; the child reports `child-result: Ok` | n/a |
| W20 `dev_stdout_appended_to_a_file_keeps_its_earlier_lines` (unix) | RED | stdout = APPEND File of `f` (`EARLIER\n`, 0644); `write_private("/dev/stdout", b"NEW\n")` | `f` starts `EARLIER\n`, contains `NEW\n`, 0600; dir holds only `f` | `NEW` only; RED |
| W21 `dev_fd_1_appended_to_a_file_keeps_its_earlier_lines` (unix) | RED | stdout = APPEND File of `f` (`EARLIER\n`, 0644); `write_private("/dev/fd/1", b"NEW\n")` | `f` starts `EARLIER\n`, contains `NEW\n`; dir holds only `f` | `NEW` only; RED |
| W22 `dev_stdout_keeps_the_stream_order_in_a_redirected_file` (unix) | RED | stdout = `File::create(f)`; child `print!`s `before-the-write|`, calls `write_private("/dev/stdout", b"via-dev-stdout\n")`, then `println!`s `after-the-write` | `f` holds all three, in that order; child reports `child-result: Ok` | order lost; RED |
| W23 `a_link_to_dev_stdout_appended_to_a_file_keeps_its_earlier_lines` (unix) | RED | as W20 with the path a symlink `l` -> `/dev/stdout` | `f` starts `EARLIER\n`, contains `NEW\n`; `l` still a symlink; dir holds `f` and `l` only | `NEW` only; RED |
| W24 `a_name_under_a_link_to_dev_fd_appended_to_a_file_keeps_its_earlier_lines` (unix) | RED | as W20 with the path `fd/1`, `fd` a symlink -> `/dev/fd` | `f` starts `EARLIER\n`, contains `NEW\n`; dir holds `f` and `fd` only | `NEW` only; RED |
| W25 `a_proc_self_cwd_path_onto_a_file_is_replaced_atomically` (linux) | RED | `f` = `OLD-VALID-CARD\n` (0644); child sets its cwd to `f`'s directory and calls `write_private("/proc/self/cwd/f", b"NEW\n")` | `child-result: Ok`; `f` is exactly `NEW\n`, 0600, a new inode; dir holds only `f` | `OLD-VALID-CARD\nNEW\n`; RED |
| W26 `a_proc_self_root_path_onto_a_file_is_replaced_atomically` (linux) | RED | `f` as W25; `write_private("/proc/self/root" + f, b"NEW\n")` | `Ok`; `f` is exactly `NEW\n`, 0600, a new inode; dir holds only `f` | `OLD-VALID-CARD\nNEW\n`; RED |
| W27 `a_link_to_a_proc_self_root_path_is_written_through_and_stays_a_link` (linux) | RED | `f` as W25; `l` a symlink -> the W26 path; `write_private(l, b"NEW\n")` | `Ok`; `f` is exactly `NEW\n`, 0600, a new inode; `l` still a symlink; dir holds `f` and `l` only | `OLD-VALID-CARD\nNEW\n`; RED |
| W28 `dev_fd_3_keeps_the_stream_order_in_a_redirected_file` (unix) | RED | `f` = `EARLIER\n` (0644); `sh -c '{ echo a >&3; child; echo b >&3; } 3>f'`, child calls `write_private("/dev/fd/3", b"NEW\n")` | `child-result: Ok`; `f` is exactly `a\nNEW\nb\n`, 0600 | `a\nb\nW\n`; RED |
| W29 `a_name_under_a_link_to_proc_thread_self_fd_appended_to_a_file_keeps_its_earlier_lines` (linux) | GREEN guard | as W24 with `fd` a symlink -> `/proc/thread-self/fd` | `f` starts `EARLIER\n`, contains `NEW\n`; dir holds `f` and `fd` only | passes |
| W30 `a_descriptor_name_the_kernel_lacks_is_refused` (unix) | RED | child with piped stdout calls `write_private("/dev/fd/01", b"NEW\n")` | `child-result: Err raw_os_error=2`; child stdout does not contain `NEW` | `Ok`, `NEW` on stdout; RED |
| W31 `a_read_only_descriptor_is_refused_before_its_mode_changes` (unix) | RED | `f` = `OLD-VALID-CARD\n` (0644); `sh -c '"$0" ... 3<"$1"'`, child calls `write_private("/dev/fd/3", b"NEW\n")` | `child-result: Err raw_os_error=9`; `f`'s bytes unchanged and its mode still 0644 | `f` mode 0600 (`left: 384`, `right: 420`); RED |
| W32 `a_name_under_a_link_to_dev_fd_in_a_pid_namespace_keeps_its_earlier_lines` (linux) | RED | `fd` a symlink -> `/dev/fd`; `f` = `EARLIER\n`; `unshare -Urpf --kill-child sh -c '"$0" ... >>"$1"'` (a new pid namespace that keeps the outer `/proc`), child calls `write_private(<dir>/fd/1, b"NEW\n")`; prints `W32 SKIP: unshare -Urpf unavailable` and passes where user namespaces are off (the gate prints a SKIP line and does not count it as a passed row) | `child-result: Ok`; `f` starts `EARLIER\n` and contains `NEW\n` | `f` is exactly `NEW\n`; RED |
| W33 `dev_fd_3_onto_stdouts_file_keeps_the_stream_order` (unix) | RED | `sh -c '"$0" ... >"$1" 3>&1'`; child prints `before-the-write|` (no newline), calls `write_private("/dev/fd/3", b"NEW\n")`, prints `after-the-write` | `child-result: Ok`; `f` contains `before-the-write|NEW\nafter-the-write` | `NEW` before `before-the-write|`; RED |
| W34 `a_descriptor_name_with_a_trailing_dot_is_refused` (unix) | RED | child with piped stdout calls `write_private("/dev/fd/1/.", b"NEW\n")` | `child-result: Err raw_os_error=20`; child stdout does not contain `NEW` | `Ok`, `NEW` on stdout; RED |
| W35 `another_processs_descriptor_path_is_not_an_alias` (linux) | GREEN guard | `f` = `OLD-VALID-CARD\n`; `g` open for writing in the test; a child `sh` holds `f` open on the same descriptor number N (`exec N>>f`); `write_private("/proc/<child pid>/fd/N", b"NEW\n")` | `Ok`; `f` is exactly `NEW\n` (replaced ATOMIC); `g` is empty | passes; under the plant "`who != me` arm removed" `g` holds `NEW\n` |
| W36 `an_alias_write_leaves_the_callers_descriptor_open` (unix) | GREEN guard | the test opens `f` for writing as descriptor N; for each spelling of N (`/dev/fd/N`; on Linux also `/proc/self/fd/N`, `/proc/thread-self/fd/N`, `/proc/<pid>/fd/N`, `/proc/self/task/T/fd/N`, `/proc/<pid>/task/T/fd/N`) calls `write_private(spelling, spelling bytes)` | each `Ok`; the test's handle is still valid after each; `f` holds all writes in order | passes; under the plant "`from_raw_fd(N)` instead of a dup" the process aborts (`IO Safety violation`) |
| W12 `the_temp_name_is_the_target_name_or_the_fallback` | new | calls the private temp-name function | 200-byte name -> `<name>.7.3.tmp` for pid 7, n 3; 201-byte name -> `mnemonic-io-lib.7.3.tmp` | n/a (new function) |
| W13 `a_failure_at_each_step_removes_the_temp_and_keeps_the_old_file` | new | old `card.txt` = `old`; for each step of the platform (unix: `CreateTemp`, `Preflight`, `Chmod`, `Write`, `Sync`, `Rename`; other: `CreateTemp`, `Preflight`, `Write`, `Sync`, `Rename`) inject an error | `Err` whose `Display` is `injected at <Step>`; bytes `old`; dir holds only the target | n/a (new branch) |
| W14 `a_directory_sync_failure_after_the_rename_is_not_reported` (unix) | new | inject at `DirSync` | `Ok`; bytes new; dir holds only the target | n/a |
| W15 `a_stale_leftover_is_skipped_and_left_alone` (unix) | new | pre-create `card.txt.<own pid>.0.tmp` = `stale` | `Ok`; target new; the stale file still holds `stale` | n/a |
| W16 `the_temp_name_search_stops_after_100_taken_names` (unix) | new | pre-create `card.txt.<own pid>.<n>.tmp` for n 0 to 99 | `Err` kind `AlreadyExists`; target `old`; the 100 files unchanged | n/a |
| W18 `enotsup_from_the_full_sync_falls_back_to_fsync_on_apple_only` | new | calls the private `full_sync_or_plain` with injected results, then `sync_temp` on a local temp file | apple + 45: the plain result stands (Ok, then os 5), called twice; apple + os 5 / os 22: returned, no call; apple + Ok: Ok, no call; not apple + 45: os 45, no call; `sync_temp` Ok | n/a (new function) |
| S1 `the_same_path_twice_is_the_same_file` | RED (compile) | existing file | `Some(0)` | `same_file` absent: E0425 |
| S2 `a_hard_link_is_the_same_file` (unix) | RED (compile) | `a`, `b` hard link | `same_file(Path(a), [Path(b)])` = `Some(0)` | E0425 |
| S3 `a_symlink_is_the_same_file` | RED (compile; symlink creation cfg(unix)) | `link` -> `real` | `Some(0)` | E0425 |
| S4 `two_files_with_equal_bytes_are_not_the_same` | RED (compile) | two files, same bytes | `None` | E0425 |
| S5 `an_absent_output_matches_an_absent_input_at_the_same_resolved_path` | RED (compile) | `dir/./x` vs `[dir/y, dir/x]` | `Some(1)`; vs `[dir/y]` `None` | E0425 |
| S6 `the_first_matching_input_is_reported` | RED (compile) | inputs `[other, alias1, alias2]` of out | `Some(1)` | E0425 |
| S7 `stdout_redirected_to_the_output_file_is_the_same_file` (unix) | RED (compile) | child with stdout = `File` of `f` | child reports `child-result: Some(0)` for `same_file(Path(f), [Stdout])` | E0425 |
| S8 `stdin_and_stdout_on_pipes_never_match` (unix) | RED (compile) | child with piped stdin and stdout | child reports `child-result: None` for `same_file(Stdout, [Stdin])` | E0425 |
| S9 `a_non_regular_path_never_matches` (unix) | RED (compile) | `/dev/null` vs `[/dev/null]` (metadata only, never written) | `None` | E0425 |
| S10 `an_unreadable_lookup_never_matches` (unix) | RED (compile) | `d/x` with `d` at mode 0 vs `[d/x]`; same root precondition as W4 | `None` | E0425 |
| S11 `stdin_redirected_from_the_output_file_is_the_same_file` (unix) | new | child with stdin = `File` opened on `f` | child reports `child-result: Some(0)` for `same_file(Path(f), [Stdin])` | n/a |

Fault seam (test-only, **PROPOSED**): a `#[cfg(test)]` thread-local `Cell<Option<Step>>`, with a private
`enum Step { CreateTemp, Preflight, Chmod, Write, Sync, Rename, DirSync }` (`Chmod` is step 6, the temp's chmod); at the start of each step, if the cell names it,
the step returns `io::Error::other(format!("injected at {step:?}"))` instead of running. Release builds carry none of
it.

No row writes to `/dev/null` or any device: a root run would chmod it (today's DIRECT does).

Gate (me repo): `cargo nextest run --locked -p mnemonic-io-lib` plus the full me CI test job and
`scripts/lint-gate.sh`; Windows and macOS run no row of this spec (io-lib's unit tests run only in me's ubuntu job); md's and mk's
Windows/macOS legs exercise `write_private` only through their own `--out` CLI tests, after their pin bump.

## 9. Consumer table

| repo | call sites (default head) | wrapper text today (unchanged by this spec) | pin after | then unblocks |
| --- | --- | --- | --- | --- |
| me | `crates/me-cli/src/main.rs:824` (`run`, NDEF `--out`), `:899` (`run_bundle_cli`, manifest), `:1023` (`run_seal_cli`, UF2), `:3318` (`emit`) | `me: cannot write <path>: <e>` exit `EXIT_USAGE` (824, 899, 1023); `<path>: <e>` (3318) | path dep (no bump) | #13 FIX-me-output-integrity (me-I10 through `same_file`; me-I5 is closed by this spec's W1 to W4; me-I4 is T6) |
| md | `descriptor-mnemonic` `crates/md-cli/src/cmd/mod.rs:109-112` `write_artifact` | `--out <path>: <e>` (`CliError::BadArg`) | `"0.1.3"` | #9 FIX_bh1119_md_out_writes (md-C4; I1, I7 are T6/T7). md adopts `write_private` only; `tests/cli_p3_decline.rs` needs no change |
| mk | `mnemonic-key` `crates/mk-cli/src/cmd/encode.rs:436` | `--out <path>: <e>` (`CliError::UsageError`) | `"0.1.3"` | #10 F-B output writes (mk-I1; mk-I6 through `same_file`) |
| ms | `mnemonic-secret` `crates/ms-cli/src/out.rs:85-88` `write_artifact` (callers `crates/ms-cli/src/cmd/encode.rs:273`, `crates/ms-cli/src/cmd/hashlock.rs:426`, `crates/ms-cli/src/cmd/repair.rs:170`) | `failed to write --out <path>: <e>` (`CliError::BadInput`) | git rev -> registry `"0.1.3"` (ruling T25) | #11 FIX_bughunt1119_ms_out_writes (ms-I13; ms-I1 through `same_file`). `write_artifact_create_new` (`crates/ms-cli/src/out.rs:46`) is ms's own and not touched |
| mt | `mnemonic-transaction` `crates/mt-cli/src/main.rs:848` | refusal `cannot write <path>` / `The file could not be created or written: <e>. Nothing was engraved and nothing was left behind.` | git rev -> registry `"0.1.3"` | #12 FIX-B output writes (mt-C3; mt-I4 through `same_file`; mt-C2 is T6). Its "nothing was left behind" becomes true for every reported error. #12 revisits the remedy ("Check the directory exists and is writable, then re-run.", `crates/mt-cli/src/main.rs:859-860`): it names the directory only, and is wrong for item 6 of section 7 |
| tk | none today (`argv_guard.rs` uses `remedy` only); bare `std::fs::write` at `crates/mnemonic-toolkit/src/cmd/seedqr.rs:309,337` and the other sites listed on line 18 of `coordinator/agent-reports/bughunt-1119/grade-tk.md` | n/a | `"0.1.3"` | #14 FIX-tk-secret-file-writes (tk-I3, tk-C1 per T38) adopts `write::write_private`; `tests/p3_declined_crate_items.rs` and `tests/p3_io_lib_pin.rs` change with it |

Toolkit-manual sentences that describe the old mechanism, each pin-bump PR updates these lines in the same PR (lockstep rule), as well as `42-md.md:219` and the source comments
that describe the open-file chmod and truncate (md-cli `cmd/mod.rs:105-108`, mt-cli `main.rs:844-847`):
`docs/manual/src/40-cli-reference/42-md.md:30` ("Overwrites an existing file and tightens its mode"),
`docs/manual/src/40-cli-reference/43-ms.md:44-45` ("set on the open file ... it overwrites and truncates"), `docs/manual/src/40-cli-reference/44-mk-cli.md:42` ("overwrites an
existing file"). This spec changes no manual text itself (no printed text changes).

Outside the table: mnemonic-refugium (`6e2072e72`) has no `write_private` and no io-lib dependency in its crates today
(the md grader's "mr 1" hit is a review report under `design/agent-reports/`); out of scope.

## 10. Release and pin-bump order

1. me PR (full track, this spec's plan): the rows of section 8, the implementation, `crates/mnemonic-io-lib/Cargo.toml`
   version 0.1.1 -> 0.1.3 (0.1.2 is never published) and the `Cargo.lock` entry with it. Merges to master through `ci/staging`.
2. Release run (coordinator): tag `mnemonic-io-lib-v0.1.3` on the merge commit and `cargo publish -p
   mnemonic-io-lib` (Brian 2026-11-22 "Go for any release"). 0.1.1 and 0.1.2 are skipped on crates.io; 0.1.3 carries them.
3. me #13 can land with step 1 or after it (path dep); it needs no release.
4. After 0.1.3 is on crates.io, each consumer's bughunt FIX PR makes the pin bump its FIRST commit
   (**PROPOSED**: one PR per repo, not a separate bump PR): md, mk, tk set `mnemonic-io-lib = "0.1.3"` (the
   requirement must name 0.1.3 so a lock cannot fall back to 0.1.0), `cargo update -p mnemonic-io-lib --precise
   0.1.3`, refresh `vendor/` and pass `ci/repro/vendor-freshness.sh`; ms and mt replace the git dependency with
   `mnemonic-io-lib = "0.1.3"` and run `cargo update -p mnemonic-io-lib` (no `--precise`: with the git entry still in
   the lock, `--precise 0.1.3` fails `revspec '0.1.3' not found`); the lock must then show `version = "0.1.3"` with
   the crates.io source. ms also refreshes `vendor/`, deletes the
   `mnemonic-io-lib|https://github.com/bg002h/mnemonic-engrave|crates/mnemonic-io-lib` line from
   `ci/repro/vendor-git-sources.txt`, and passes `ci/repro/vendor-freshness.sh`. ms also edits `.github/workflows/man-release.yml`: delete the `pins` job and `needs: pins`; delete `git_source_url` and `git_source_rev` from the `repro` job (an empty or absent pair makes the toolkit workflow skip its fourth `[source]` stanza); in both musl build steps delete the `IO_LIB_REV` and `IO_LIB_SRC` lines and the three `--config "source.\"${IO_LIB_SRC}\"..."` lines (and the `-e IO_LIB_REV="$IO_LIB_REV" \` line in the x86_64 `docker run` step), keeping the `source.crates-io.replace-with` and `vendored-sources` configs. `grep -n 'mnemonic-engrave\|IO_LIB\|pins' .github/workflows/*.yml` must then print no ms workflow line other than comments. Order among them is
   free; the master table order is #9 md, #10 mk, #11 ms, #12 mt, #14 tk.

## 11. Out of scope, and proposed FOLLOWUPS lines (listed, not appended)

- `--out /dev/null` refuses for a non-root holder (`Operation not permitted`, measured), and a root run chmods
  `/dev/null` to 0600, because DIRECT chmods any handle. Kept unchanged here by the ruling ("non-regular keeps
  today's direct write"). Proposed line (Minor): the DIRECT path should skip the chmod for a non-regular handle.
- ms `write_artifact_create_new` is not atomic on a failed write either (a create-new file is cut, but no old file
  exists); deferred-minor M1 already tracks the create-new twin.
