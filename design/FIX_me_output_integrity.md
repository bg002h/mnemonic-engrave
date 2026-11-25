# FIX_me_output_integrity: a failed write is never success, and the manifest never lands on a plate preview (bh-me-I4, bh-me-I10; both Important)

- Source: bughunt 2026-11-19 keys **bh-me-I4** and **bh-me-I10** (coordinator
  `agent-reports/bughunt-1119/grade-me.md:67,73`; triage t1 and t3). FIX table row #13
  (`agent-reports/architect-bughunt-1119-questions.md:125`), **small** track, one Opus review round. Not class B
  (no funds path). bh-me-I5 was row #7 (io-lib safe write) and is closed; it is not in this document.
- Final grades: **bh-me-I4 Important** (grader I; T2/P6: exit 0 with no artifact is the class graded I at md-I1,
  mt-C2, tk-I2, sh-I5, sh-I6), **bh-me-I10 Important** (grader I; fails open, one unusual input).
- Rulings used, all decided: **T6** (failed-write contract, `architect-bughunt-1119-questions.md:42`, names me-I4),
  **T5** (`same_file` gets callers, names me-I10, `:41`), **Q2** of the file-output convention (output names an
  input or another output: refuse, naming both, "write to a new name"; `memory/file-output-convention-rulings-2026-11-23.md`),
  **R5 as amended** by `agent-reports/architect-io-lib-append-1124.md` §2 (state the disk only when a second look
  confirms it; nothing about the disk for a device or a standard-stream name). Batch overrides
  `briefs/bh1119-batch-1124-overrides.md` (io-lib 0.1.3). Printed shapes follow descriptor-mnemonic
  `design/FIX_bh1119_md_out_writes.md` @5aab2405 (row #9) and mnemonic-transaction
  `design/FIX_bh1119_mt_output_writes.md` @b39efe44 (row #12).
- Base: mnemonic-engrave master `44492264`. Branch `fix/me-output-integrity`. Crate `mnemonic-engrave` (me-cli)
  only. No new flag, type visible to a holder, or command; no wire change. Author: Claude Opus 5.5
  (claude-opus-5-5), 2026-11-24.
- Every new holder-visible text is **PROPOSED (architect confirms)** unless quoted from T6.
- Toolchain: `export PATH=$HOME/.cargo/bin:$PATH`; the CI pin is 1.85.0 (`release.yml` `RUST_TOOLCHAIN`; the repo
  has no `rust-toolchain.toml`), so `cargo +1.85.0`; `--locked`; every cargo call with `CARGO_BUILD_JOBS=6`;
  nextest `--build-jobs 6 -j 6`; never `--release`.
- **io-lib.** me-cli takes mnemonic-io-lib as a PATH dependency (`crates/me-cli/Cargo.toml:28`), so there is no
  pin to change and no registry `cargo search` step. Master carries 0.1.2 (e09d538d content). One row,
  **o7**, needs 0.1.3 (the stdio alias is written into the open stream, never truncated): it is GREEN only once
  `build/io-lib-0.1.3` (head bfa9a0a2 when this was written) is merged to master. Every other row needs only
  master. The builder cuts its branch from master after that merge; if it is not merged, the builder does every
  step, shows o7 RED, and STOPs before claiming o7 GREEN.

## 1. The defects (RED evidence, run by the author on the base 44492264)

Base binary `target/debug/me` from `cargo +1.85.0 build --locked -p mnemonic-engrave --bin me`. Every run under
`umask 077` (otherwise a stdout redirect into a new file is refused by F-244 at exit 2, a different reason) with
stderr on a pipe (a size limit on the stderr file makes the shell fail for the wrong reason; triage t1 note (a)).
`FSIZE0` = `bash -c "ulimit -f 0; trap '' XFSZ; ..."` (a zero file-size limit with the signal ignored, so the
write returns EFBIG; this stands in for a full disk or quota). `MD1` = `md1yqpqqxqq8xtwhw4xwn4qh`; `m.txt` holds
MD1 and a newline; `b.txt` holds MD1, `MK1_A`, `MK1_B` (triage t3's constants, `bh_triage_i10_i11.rs:9-12`), one
per line; `c.bin` = the output of `me sysw pack --no-passphrase text:6869 --out c.bin`. The fake sidecar for
`--preview` rows is triage t3's `write_fake` (`ME_PREVIEW_BIN` set; it answers `--version` with `me-preview 0.13.0`
and writes `<svg/>` or a PNG magic to `--out`).

- **bh-me-I4, a write the destination refused is reported as success.** `FSIZE0 me sysw pack --no-passphrase
  text:6869 > sink.bin`: exit 0, sink.bin 0 bytes, stderr ends with the full card `digest:   c679 6b68 b993 bc10
  793a 3de2 8b3d 46e0` / `re-print it with: me sysw show <the file you just wrote>`. The same with `> /dev/full`
  and with a read-only stdout (`1</dev/null`). The converter's raw branch: `me --in m.txt --stdout > /dev/full`
  exits 0 with empty stderr. Cause: `emit` (`crates/me-cli/src/main.rs:3539-3543`) and the converter's raw branch
  (`:1038`) call `write_all` on std's line-buffered stdout and never `flush`; the bytes (no newline, or a tail
  after the last newline) sit in the buffer, and std drops the flush error at exit. A read-only stdout answers
  EBADF, which std's stdout reports as a successful write.
- **The T6 gap around it (same cause class, same sinks).** Every other stdout sink uses `println!`, which panics:
  `me --in m.txt --hex > /dev/full`, `--base64`, `me bundle --in b.txt > /dev/full`, `me sysw show c.bin >
  /dev/full`, `me hash --unsealed MD1 > /dev/full` each exit **101** with `thread 'main' panicked at
  library/std/src/io/stdio.rs:1123:9:` / `failed printing to stdout: No space left on device (os error 28)`. A
  closed pipe does the same: `me sysw show c.bin | head -c0` exits 101 with `failed printing to stdout: Broken
  pipe (os error 32)`. `me --help > /dev/full` and `me --version > /dev/full` exit 0 (clap drops the error).
  `me --in m.txt --hex 1</dev/null` exits 0. `me sysw wipe > /dev/full` exits **2** with `me: stdout: No space
  left on device (os error 28)` (64 KiB overflows the buffer, so the error surfaces), where T6 rules exit 1 and
  `me: cannot write the output: <error>`.
- **The `--out` failure texts (T6 + R5).** Every `--out`-style write error exits **2** with one of two texts and
  no disk state: `me sysw pack ... --out nodir/x.bin` gives `me: nodir/x.bin: No such file or directory (os error
  2)` (`emit`, `:3540`); the converter `--out`, `me bundle --manifest` and `me seal --out` give `me: cannot write
  nodir/x.ndef: No such file or directory (os error 2)` (`:1018`, `:1093`, `:1217`). The old file is already kept
  on master (0.1.2 temp + rename): `FSIZE0 me sysw pack ... --out F.bin` over `OLD BACKUP\n` exits 2 with `me:
  F.bin: File too large (os error 27)` and F.bin still `OLD BACKUP\n`.
- **The stdio alias (architect io-lib-append-1124 F1, on me master today).** `FSIZE0 me sysw pack --no-passphrase
  text:6869 --out /dev/stdout >> s.txt` with s.txt = `OLD BACKUP\n` (11 bytes): exit 2, `me: /dev/stdout: File too
  large (os error 27)`, and **s.txt is 0 bytes**. With build/io-lib-0.1.3's `write.rs` swapped in (scratch,
  reverted): exit 2, the same old text, s.txt keeps its 11 bytes; without the limit the 61-byte container is
  appended (72 bytes).
- **bh-me-I10, the manifest overwrites a plate preview.** `me bundle --in b.txt --preview p --manifest
  p/plate-1.svg` (p empty): exit 0, stderr `me: rendered plate 1 → p/plate-1.svg` then `me: wrote manifest to
  p/plate-1.svg`, and p/plate-1.svg now starts `{` (the manifest JSON). The same with `--png` and
  `p/plate-1.png`; with `./p/./plate-1.svg`; with `--manifest L`, L a dangling symlink to `p/plate-2.svg`
  (plate-2.svg is the JSON); with `--manifest q/plate-3.svg`, q a symlink to p. With no sidecar (previews skipped)
  the manifest is written as `p9/plate-1.svg` and the next `--preview p9` run refuses the directory. Cause:
  `run_bundle_cli` (`:1048-1102`) renders (`wire_previews`, `:1078-1082`; paths `DIR/plate-<n>.<ext>`,
  `preview.rs:158`) and then writes the manifest (`:1091-1099`) without comparing the two.
- Already refused on the base (controls): a manifest hard-linked as `p/plate-1.svg` (F8's dirty-directory scan,
  exit 2), and `me bundle --preview p > p/plate-1.svg` (the shell creates the file first; F8 refuses, exit 2).

## 2. Spec delta (behaviour)

**S1 One stdout writer (T6; bh-me-I4).** Every byte `me` writes to stdout goes through one writer that writes,
then flushes, and checks both results: the converter's `--stdout` (raw), `--hex` and `--base64` output; `me
bundle`'s manifest JSON (no `--manifest`); `me hash`'s line; `me sysw show`'s header, identity, record and mt-set
lines; `emit`'s container or image (`sysw pack`, `sysw wipe` without `--out`); clap's `--help`, `<verb> --help`,
`help <verb>` and `--version` output.
- **S1a** The first failed write ends the verb there: no later stdout or stderr output of that verb is printed
  (no pack card, no `me bundle` checklist, no `sysw show` digest), and it is reported once.
- **S1b** A closed pipe (`ErrorKind::BrokenPipe`) prints nothing more and the exit code is the verb's own (0 for
  a verb that succeeded). No panic text.
- **S1c** Any other error prints, as the last stderr line, `me: cannot write the output: <error>` (`<error>` is
  the `std::io::Error` Display, for example `No space left on device (os error 28)`) and the exit code is **1**.
  A verb that already failed keeps its own code (me verbs write stdout only on success, so this arises only in
  rule). No panic text.
- **S1d** A stdout whose access mode is read-only (`fcntl(1, F_GETFL) & O_ACCMODE` neither `O_WRONLY` nor
  `O_RDWR`) is error EBADF at the first stdout write: `me: cannot write the output: Bad file descriptor (os error
  9)`, exit 1. A verb that writes nothing to stdout never checks. A closed fd 1 (`>&-`) is not detected (fcntl
  fails; md's `md-cli/src/out.rs` `read_only` precedent). Implementation note: `libc` is already in
  `Cargo.lock` (0.2.186); the builder adds it as a direct unix dependency of me-cli (that changes the package's dependency list in `Cargo.lock`: commit the regenerated `Cargo.lock`; no new crate version), or reads the same flag
  another way; the behaviour is what is specified.
- **S1e** stderr is unchanged by this FIX: `eprintln!` still panics if stderr cannot be written (out of scope;
  section 7 files it).
- **S1f** clap usage errors keep clap's stderr text and exit 2. Help and version text is byte-identical to today's
  when the write succeeds.
- The F-244 world-readable and F-253 terminal refusals run before any write, unchanged.

**W1 `--out` writer (T5).** `me --out`, `me bundle --manifest`, `me seal --out`, `me sysw pack --out` and `me sysw
wipe --out` keep calling `mnemonic_io_lib::write::write_private` and add no write logic. What the holder gets is
io-lib's (0.1.3, `crates/mnemonic-io-lib/src/write.rs` on build/io-lib-0.1.3): an absent path or an existing
regular file (symlinks followed) is written to a sibling temporary and renamed over it, so a failed write leaves an
existing file's bytes as they were and creates nothing at a new name; a FIFO or device is written directly; a
standard-stream alias (`/dev/stdin`/`stdout`/`stderr`, `/dev/fd/N`, `/proc/{self,thread-self,<own pid>}/fd/N`, or a symlink into one; any other `/proc/` path is an ordinary file name) is written into the open
stream at its own offset and never truncated, so a failed write can leave part of the output after the earlier
contents.

**W2 `--out` failure text (T6 + R5 amended).** Before the write `me` takes `std::fs::metadata(path)` (symlinks
followed): a regular file keeps (dev, ino, len, mtime); `NotFound` (including a dangling symlink) is "absent";
anything else is "other". A path that names a standard stream is always "other": the path as typed, or its
canonical parent joined with its name, starts with `/dev/` or `/proc/`, or a hop of its symlink chain
(`std::fs::read_link`, repeated) does. On a write error `me` takes the same metadata again, prints exactly one
line as the last stderr line, and exits **1**:
- regular before, and the same (dev, ino, len, mtime) after: `me: cannot write the output: <path>: <error>. <path> is unchanged.`
- absent before, and still `NotFound` after: `me: cannot write the output: <path>: <error>. Nothing was written.`
- anything else: `me: cannot write the output: <path>: <error>`

`<path>` is the path as typed (`Path::display()`). "Unchanged" is about the bytes: io-lib may already have set an
existing file to 0600. The verb stops there (S1a): no `me: wrote ...` line, no pack card, no checklist, no
passphrase. These replace `me: <path>: <error>` (`emit`) and `me: cannot write <path>: <error>` (converter,
bundle, seal), both exit 2.

**P1 The manifest is never a plate preview (bh-me-I10; Q2, T5 `same_file`).** `me bundle --preview DIR --manifest
M`:
- **P1a** The planned plates are the paths the render loop would write: `DIR/plate-<n>.svg` (`.png` under
  `--png`) for each plate it renders (a plate that is not `ms1` and carries a string), built the same way as
  `preview::render_plate` builds them (one helper for both).
- **P1b** After `run_bundle` succeeds (its refusals keep their precedence) and before `wire_previews` (so before
  `ME_PREVIEW_BIN` is checked, the sidecar is located or version-probed, the directory is checked or scanned, or
  anything is rendered or written), `me` calls `mnemonic_io_lib::write::same_file(Endpoint::Path(M), &planned)`.
  Identity is io-lib's: a regular file by (dev, ino) (so a hard link matches), an absent path by its canonical
  directory plus its name after following M's symlink chain (so `./p/./plate-1.svg`, a dangling symlink to a
  planned plate, and a directory symlink to DIR all match).
- **P1c** A match refuses: the P1 text (section 3) is the last stderr line and the only line this check prints; lines printed before the check stay (with `b.txt`, MK1_A's `warning: this key card's stamped chunk-set id (12345) was not derived from its content, ...`), naming M as typed and the first
  matching planned plate `P` as `me: rendered plate <n> → <P>` would print it; exit **2**; stdout empty; nothing
  rendered; nothing written; no checklist.
- **P1d** The check runs whenever `--preview` and `--manifest` are both given, also when no sidecar is installed
  (the paths are planned by the flags, not by the install; the manifest would otherwise sit at a plate name that
  the next `--preview` run refuses as a dirty directory).
- **P1e** A manifest elsewhere in DIR, including at a plate-shaped name no plate takes (`p/plate-9.svg` for a
  3-plate bundle), is not refused (unchanged). A hard link of an existing M at a planned plate name now gets the P1
  text instead of F8's dirty-directory text (both exit 2).
- **P1f** After `wire_previews` returns with plates rendered, and before the manifest is serialised or written, `me` calls `same_file(Endpoint::Path(M), &rendered)` a second time. `rendered` is the `plate.preview` path of every plate just written. Those files now exist, so identity is (dev, ino) on Unix and the canonical path on Windows. That also catches a name that differs only in letter case on a case-insensitive filesystem, and an alias that appears during rendering. A match prints the P1 text as the last stderr line and exits 2. No manifest is written, and the rendered plates stay (their `rendered plate` lines were printed). No checklist follows.
- Without `--manifest` the manifest goes to stdout; `> DIR/plate-1.svg` is already refused by F8 (the shell creates
  the file first) and is unchanged.

## 3. Texts (exact)

- **T6 line (decided by T6, not PROPOSED):** `me: cannot write the output: <error>`.
- **W2 lines (PROPOSED, architect confirms; T6's prefix with R5's path and state, the md #9 and mt #12 shape):**
  - `me: cannot write the output: <path>: <error>. <path> is unchanged.`
  - `me: cannot write the output: <path>: <error>. Nothing was written.`
  - `me: cannot write the output: <path>: <error>`
- **P1 refusal (PROPOSED, architect confirms):**
  `me: --manifest <M> is the same file as the plate preview <P>; write the manifest to a new name`
- **Spec, `design/SPEC_systemwide_payloads.md` §5.6, a new bullet after "Printed commands (bh-me-I11, ...)" (PROPOSED):**
  `- **Failed writes (bh-me-I4, ruling T6, 2026-11-24).** This holds for every \`me\` command, not only \`sysw\`.
  Every stdout write is flushed and checked. A reader that closes the pipe early only stops the output; the exit
  code is the command's own. Any other stdout write error (a full disk, a read-only stdout), from a command or
  from \`--help\`/\`--version\`, prints \`me: cannot write the output: <error>\` as the last stderr line and exits 1;
  nothing the command would print after it is printed, so no digest card follows a failed write. An \`--out\` (or
  \`--manifest\`) write that fails prints \`me: cannot write the output: <path>: <error>\` and exits 1. \`me\` then
  looks at the path again: if a regular file was there and is still the same file with the same size and
  modification time, the line ends \`. <path> is unchanged.\`; if nothing was there and nothing is there now, it
  ends \`. Nothing was written.\`; otherwise (a device, or a standard-stream name such as \`/dev/stdout\`, which is
  written into the open stream) it says nothing about the disk. These replace exit 2 for a failed write.`
- **Spec, `design/SPEC_me_bundle_phaseB_preview.md` §6, a new bullet at the end (PROPOSED):**
  `- **The manifest is never a plate preview (bh-me-I10, 2026-11-24).** With \`--preview DIR\` and \`--manifest M\`,
  M that is the same file as a plate preview this run would write (the same path, a symlink to it, a path through a
  symlinked directory, or a hard link) is refused (before anything is rendered or written when the names show it; otherwise after the plates are rendered and before the manifest is written), also when no sidecar is
  installed: \`me: --manifest <M> is the same file as the plate preview <P>; write the manifest to a new name\`,
  exit 2.`
- **README (PROPOSED)**, after the sentence ending "all human-readable text goes to **stderr**." (`README.md:136`):
  `If an output cannot be written (a full disk), \`me\` prints \`me: cannot write the output: ...\` and exits 1,
  and an existing \`--out\` file is left as it was; a reader that closes the pipe early only stops the output.`
  And after "A set-but-missing `ME_PREVIEW_BIN` is a hard error (exit 2)." (`README.md:152`): `A \`--manifest\` that
  is one of the plate previews is refused (exit 2).`
- **CHANGELOG `crates/me-cli/CHANGELOG.md` `## [Unreleased]` / `### Fixed`, PROPOSED (two bullets):**
  - `**A write the destination refused is no longer reported as success** (bughunt-1119 bh-me-I4, Important;
    ruling T6). \`me sysw pack\` and the converter's \`--stdout\` exited 0, with the pack card and its digest, when
    stdout was a full disk or a read-only descriptor and nothing reached it; \`--hex\`, \`--base64\`, \`me bundle\`,
    \`me hash\` and \`me sysw show\` panicked (exit 101), also on a closed pipe. Every stdout write is now flushed
    and checked: a closed pipe only stops the output, any other error prints \`me: cannot write the output:
    <error>\` and exits 1, and no card follows. A failed \`--out\`/\`--manifest\` write now prints \`me: cannot write
    the output: <path>: <error>\` and says whether the file is unchanged, and exits 1 (was exit 2). Through
    mnemonic-io-lib 0.1.3, \`--out /dev/stdout\` writes into the open stream and never empties the file behind it.`
  - `**\`me bundle --manifest\` can no longer overwrite a plate preview** (bughunt-1119 bh-me-I10, Important).
    A manifest path that is one of the \`--preview\` plates (also through a symlink or a hard link) is refused
    with exit 2 before anything is rendered or written; it used to replace the rendered plate at exit 0.`
- **Toolkit manual:** none. `mnemonic-toolkit` master `docs/manual/src/40-cli-reference/` has no `me` page (41-46
  are mnemonic, md, ms, mk-cli, mr, mr-sitting); its `me sysw pack` mentions (43-ms.md) are about input and are
  unchanged.
- **Help:** no change (no flag help states the failure behaviour).

## 4. TRACE rows (RED on the base; GREEN after the build)

New file `crates/me-cli/tests/bh1119_me_output_integrity.rs` (`#![cfg(unix)]`), one test function per row, named
exactly as the row. Each row runs the printed command through `sh -c`/`bash -c` in a `tempfile::tempdir()` with
`umask 077`, stderr captured on a pipe, stdin `/dev/null` unless stated. `/dev/full` rows skip off Linux. `LAST` =
the last stderr line. `NOSPC` = `me: cannot write the output: No space left on device (os error 28)`. `EBADF` =
`me: cannot write the output: Bad file descriptor (os error 9)`. `CARD` = a stderr line starting `digest:`.
`PANIC` = any stderr line containing `panicked`. Base results were seen by the author on 44492264 (section 1);
fixed results are the rules of section 2 (no prototype was built, except o7's io-lib half).

| row | setup; command | base (seen) | fixed |
|---|---|---|---|
| w1_pack_stdout_rejected_by_fsize_limit_is_not_success | `FSIZE0 me sysw pack --no-passphrase text:6869 > sink.bin` | rc 0, sink 0 B, CARD present | rc 1, LAST == `me: cannot write the output: File too large (os error 27)`, no CARD, no `re-print it with` |
| w2_pack_stdout_full_exits_1 | `me sysw pack --no-passphrase text:6869 > /dev/full` | rc 0, CARD present | rc 1, LAST == NOSPC, no CARD |
| w3_pack_read_only_stdout_exits_1 | `me sysw pack --no-passphrase text:6869 1</dev/null` | rc 0, CARD present | rc 1, LAST == EBADF, no CARD |
| w4_converter_raw_stdout_full_exits_1 | `me --in m.txt --stdout > /dev/full` | rc 0, stderr empty | rc 1, stderr exactly NOSPC |
| w5_converter_hex_full_exits_1 | `me --in m.txt --hex > /dev/full` | rc 101, PANIC | rc 1, stderr exactly NOSPC |
| w6_converter_base64_full_exits_1 | `me --in m.txt --base64 > /dev/full` | rc 101, PANIC | rc 1, stderr exactly NOSPC |
| w7_converter_hex_read_only_stdout_exits_1 | `me --in m.txt --hex 1</dev/null` | rc 0 | rc 1, stderr exactly EBADF |
| w8_bundle_stdout_full_exits_1_without_checklist | `me bundle --in b.txt > /dev/full` | rc 101, PANIC | rc 1, LAST == NOSPC, no PANIC, no line containing `backup needs` |
| w9_show_stdout_full_exits_1 | `me sysw show c.bin > /dev/full` | rc 101, PANIC | rc 1, stderr exactly NOSPC (no CARD) |
| w10_hash_stdout_full_exits_1 | `me hash --unsealed MD1 > /dev/full` | rc 101, PANIC | rc 1, stderr exactly NOSPC |
| w11_wipe_stdout_full_exits_1 | `me sysw wipe > /dev/full` | rc 2, `me: stdout: No space left on device (os error 28)` | rc 1, stderr exactly NOSPC |
| w12_help_stdout_full_exits_1 | `me --help > /dev/full` | rc 0 | rc 1, stderr exactly NOSPC |
| w13_version_stdout_full_exits_1 | `me --version > /dev/full` | rc 0 | rc 1, stderr exactly NOSPC |
| w14_show_closed_pipe_keeps_exit_0 | `(sleep 0.5; exec me sysw show c.bin) \| true`, rc of the left side | rc 101, PANIC `Broken pipe (os error 32)` | rc 0, no PANIC, no `cannot write` |
| w15_converter_raw_closed_pipe_keeps_exit_0 | `(sleep 0.5; exec me --in m.txt --stdout) \| true`, rc of the left side | rc 0, stderr empty | same (control, GREEN on base) |
| w16_pack_stdout_success_is_unchanged | `me sysw pack --no-passphrase text:6869 > ok.bin` | rc 0, ok.bin 61 B == c.bin, CARD | same (control, GREEN on base) |
| o1_pack_failed_overwrite_says_unchanged | F.bin = `OLD BACKUP\n`; `FSIZE0 me sysw pack --no-passphrase text:6869 --out F.bin` | rc 2, `me: F.bin: File too large (os error 27)`, F.bin intact | rc 1, LAST == `me: cannot write the output: F.bin: File too large (os error 27). F.bin is unchanged.`, F.bin == `OLD BACKUP\n`, no `*.tmp` in the dir, no CARD |
| o2_pack_missing_dir_says_nothing_was_written | `me sysw pack --no-passphrase text:6869 --out nodir/x.bin` | rc 2, `me: nodir/x.bin: No such file or directory (os error 2)` | rc 1, LAST == `me: cannot write the output: nodir/x.bin: No such file or directory (os error 2). Nothing was written.`, no CARD |
| o3_pack_failed_new_file_leaves_nothing | N.bin absent; `FSIZE0 me sysw pack --no-passphrase text:6869 --out N.bin` | rc 2, `me: N.bin: File too large (os error 27)`, N.bin absent | rc 1, LAST == `me: cannot write the output: N.bin: File too large (os error 27). Nothing was written.`, the dir holds no `N.bin*` |
| o4_pack_read_only_dir_says_unchanged | ro/F = `OLD\n`, chmod 0500 ro; `me sysw pack --no-passphrase text:6869 --out ro/F` (skip when euid is 0) | rc 2, `me: ro/F: Permission denied (os error 13)`, ro/F intact | rc 1, LAST == `me: cannot write the output: ro/F: Permission denied (os error 13). ro/F is unchanged.`, ro/F == `OLD\n` |
| o5_converter_out_missing_dir | `me --in m.txt --out nodir/x.ndef` | rc 2, `me: cannot write nodir/x.ndef: No such file or directory (os error 2)` | rc 1, stderr exactly `me: cannot write the output: nodir/x.ndef: No such file or directory (os error 2). Nothing was written.` |
| o6_bundle_manifest_missing_dir | `me bundle --in b.txt --manifest nodir/x.json` | rc 2, LAST `me: cannot write nodir/x.json: No such file or directory (os error 2)` | rc 1, LAST == `me: cannot write the output: nodir/x.json: No such file or directory (os error 2). Nothing was written.`, stdout empty, no line containing `backup needs` |
| o7_stdio_alias_onto_a_file_names_no_state | s.txt = `OLD BACKUP\n` (11 B); `FSIZE0 me sysw pack --no-passphrase text:6869 --out /dev/stdout >> s.txt` | rc 2, `me: /dev/stdout: File too large (os error 27)`, **s.txt 0 B** | rc 1, LAST == `me: cannot write the output: /dev/stdout: File too large (os error 27)` (no state sentence), s.txt == `OLD BACKUP\n`. **Needs io-lib 0.1.3** |
| o8_seal_out_missing_dir | `me seal md1fv9wjpqpqpm6jzzqqvqpdqnf4ztqq4gy99tzyzyzdv7xh9vpdwu3t7dhhesk2tl3 --out nodir/x.uf2` | rc 2, `me: cannot write nodir/x.uf2: No such file or directory (os error 2)` | rc 1, LAST == `me: cannot write the output: nodir/x.uf2: No such file or directory (os error 2). Nothing was written.`, no `wrote` line |
| o9_pack_out_device_names_no_state | `me sysw pack --no-passphrase text:6869 --out /dev/full` | rc 2, `me: /dev/full: Operation not permitted (os error 1)` | rc 1, LAST starts `me: cannot write the output: /dev/full: `, ends `)`, no `unchanged`, no `Nothing was written` |
| p1_manifest_is_svg_plate_refused | p empty; `me bundle --in b.txt --preview p --manifest p/plate-1.svg` | rc 0, p/plate-1.svg starts `{` | rc 2, LAST == `me: --manifest p/plate-1.svg is the same file as the plate preview p/plate-1.svg; write the manifest to a new name`, and no stderr line contains `rendered plate`, `wrote manifest`, `preview skipped` or `backup needs`, stdout empty, p empty |
| p2_manifest_is_png_plate_refused | same with `--png` and `--manifest p/plate-1.png` | rc 0, p/plate-1.png starts `{` | rc 2, LAST == the P1 text with M = P = `p/plate-1.png`, and no stderr line contains `rendered plate`, `wrote manifest`, `preview skipped` or `backup needs`, p empty |
| p3_manifest_dot_slash_spelling_refused | `--manifest ./p/./plate-1.svg` | rc 0, p/plate-1.svg starts `{` | rc 2, LAST == the P1 text with M = `./p/./plate-1.svg`, P = `p/plate-1.svg`, and no stderr line contains `rendered plate`, `wrote manifest`, `preview skipped` or `backup needs`, p empty |
| p4_manifest_symlink_to_plate_refused | `ln -s p/plate-2.svg L`; `--manifest L` | rc 0, p/plate-2.svg starts `{` | rc 2, LAST == the P1 text with M = `L`, P = `p/plate-2.svg`, and no stderr line contains `rendered plate`, `wrote manifest`, `preview skipped` or `backup needs`, p empty, L still a dangling symlink |
| p5_manifest_through_dir_symlink_refused | `ln -s p q`; `--manifest q/plate-3.svg` | rc 0, p/plate-3.svg starts `{` | rc 2, LAST == the P1 text with M = `q/plate-3.svg`, P = `p/plate-3.svg`, and no stderr line contains `rendered plate`, `wrote manifest`, `preview skipped` or `backup needs`, p empty |
| p6_manifest_hard_link_at_plate_name_refused | m.json = `X`; `ln m.json p/plate-1.svg`; `--manifest m.json` | rc 2, `me: preview directory p already contains plate artifacts (e.g. plate-1.svg); use an empty/clean directory` | rc 2, LAST == the P1 text with M = `m.json`, P = `p/plate-1.svg`, and no stderr line contains `rendered plate`, `wrote manifest`, `preview skipped` or `backup needs`, m.json == `X` |
| p7_manifest_on_plate_refused_without_sidecar | `ME_PREVIEW_BIN` unset, no sidecar beside `me`; p9 empty; `--preview p9 --manifest p9/plate-1.svg` | rc 0, `me: preview skipped (install me-preview)`, p9/plate-1.svg starts `{` | rc 2, LAST == the P1 text with M = P = `p9/plate-1.svg`, and no stderr line contains `rendered plate`, `wrote manifest`, `preview skipped` or `backup needs`, p9 empty |
| p8_distinct_manifest_still_written | `--preview p --manifest m.json` | rc 0, 3 SVG plates, m.json is the JSON | same (control, GREEN on base) |
| p9_unplanned_plate_name_still_written | `--preview p --manifest p/plate-9.svg` | rc 0, p/plate-1..3.svg are `<svg/>`, p/plate-9.svg is the JSON | same (control, GREEN on base; P1e) |
| p10_manifest_alias_made_during_render_refused | the fake sidecar also runs `if [ -n "$FAKE_ALIAS" ] && [ ! -e "$FAKE_ALIAS" ]; then ln -s "$PWD/$out" "$FAKE_ALIAS"; fi` after writing `--out`; `FAKE_ALIAS=M me bundle --in b.txt --preview p --manifest M` | rc 0, p/plate-1.svg starts `{` | rc 2, LAST == `me: --manifest M is the same file as the plate preview p/plate-1.svg; write the manifest to a new name`, p/plate-1.svg == `<svg/>`, stdout empty, no line containing `wrote manifest` or `backup needs` (stands in for the case-folding spelling, which CI cannot reach: CI tests run on Linux only) |

`FSIZE0`, `MD1`, `m.txt`, `b.txt`, `c.bin` and the fake sidecar are as in section 1. The `p` rows set
`ME_PREVIEW_BIN` to the fake sidecar except p7; a p7 run where a real `me-preview` sits beside the test's `me`
binary skips.

RED expectation for the builder: w1-w14, o1-o9 and p1-p7 fail on the base; w15, w16, p8 and p9 pass on the base and
after. o7 also fails on master with the rest of this FIX applied until io-lib 0.1.3 is merged (s.txt is emptied).
Plant for W2's alias rule: a build of W2 that skips the standard-stream test prints `/dev/stdout is unchanged.` on
o7 (architect io-lib-append-1124 F2); the builder shows that plant failing o7.

Coverage map (rule -> rows): S1 raw and emit flush -> w1, w2, w4, w16; S1 println sinks -> w5, w6, w8, w9, w10;
S1 emit large image -> w11; S1 clap -> w12, w13; S1a stop (no card, no checklist) -> w1, w2, w3, w8, w9; S1b ->
w14, w15; S1c text and exit -> w1-w13; S1d -> w3, w7; S1f -> the existing clap-usage tests (unchanged); W1 old
file kept -> o1, o4; W1 absent stays absent -> o3; W1 alias into the stream -> o7; W2 regular -> o1, o4; W2 absent
-> o2, o3, o5, o6, o8; W2 other -> o7, o9; P1a planned set -> p1, p2, p9; P1b identities -> p3 (spelling), p4
(symlink), p5 (directory symlink), p6 (hard link); P1c text, exit, nothing written -> p1-p7; P1d -> p7; P1e -> p6,
p9; unchanged success -> w16, p8.

## 5. Tests that change

- `crates/me-cli/tests/exit_digits.rs` `a_failed_write_is_usage_two` (`:139-165`): `emit`'s write failure is now
  exit 1 (T6), so the row is renamed **`a_failed_write_is_exit_one`** and asserts code 1 and the stderr line
  `me: cannot write the output: <out>: No such file or directory (os error 2). Nothing was written.` (<out> as
  passed), keeping `assert!(!out.exists())`. The module header table's `emit`, write failure row (`:13`) and the
  F-265 text that calls it a usage error gain one sentence: `Since FIX_me_output_integrity (T6) a failed write is
  exit 1, not 2; the row now pins that digit.` The control `the_vocabulary_still_has_more_than_one_code_in_it` is
  unchanged.
- No other test asserts a write-failure text or exit code (`grep -rn "cannot write\|me: stdout\|os error"
  crates/me-cli/tests` finds only the above and a test-side BrokenPipe in `dash_stdin.rs:56`). The builder runs
  the full suite and lists any other test the change moves.
- The triage tests `bh_me_i4_raw_stdout_rejected_by_destination_is_not_success` (t1) and `i10_*`
  (`bh_triage_i10_i11.rs`, t3) are superseded by w1 and p1-p3 and are not copied in.

## 6. Points this document decides (PROPOSED where holder-visible)

1. S1a: a failed stdout write ends the verb (no card, checklist or digest after it), matching me's existing rule
   that the pack card prints only after `emit` returns OK. (mt #12 S1c keeps the remaining stderr; me does not,
   because me's later stderr lines are success claims.)
2. P1 exit code 2: me's usage code, the same as F8's dirty-directory refusal and the T32 path-option conflicts.
3. P1b order: after `run_bundle` (bundle refusals first), before anything about the sidecar.
4. P1d: refused also with no sidecar installed.
5. P1e: only planned plate paths are refused; `p/plate-9.svg` stays allowed.
6. S1e: stderr failure is out of scope (section 7).
7. W2 "Nothing was written." on `me bundle --preview DIR --manifest M` refers to M; plates already rendered stay
   (their `rendered plate` lines were printed). The architect may want a different sentence there.

## 7. FOLLOWUPS

Closes nothing in `design/FOLLOWUPS.md` (bh-me-I4 and I10 are not filed there; the coordinator's bughunt records
close them). Proposed lines (direct-push after the merge, not in the PR) are in the author's report
`coordinator/agent-reports/fix-doc-bh-me13.md`: me stderr write failure (S1e), and `--manifest`/`--out` that names
an input file (Q2, not in this row's keys).

## 8. Gate line

`bash scripts/fix-me-output-integrity-gate.sh` (the builder commits it with its coordinator catalog row; shape of
`scripts/fix-me-inspection-fail-open-gate.sh`; every cargo call with `CARGO_BUILD_JOBS=6`): step A builds `me`
(`--locked`) and runs w1, w2, w4, w5, w11, w12, w14, o1, o2, o7, p1, p4 and p7 as printed commands against the
built binary (exit code, stdout, the exact stderr line (the last stderr line for p1, p4 and p7) and the file bytes compared to the fixed column; FSIZE0 by
`ulimit`); step B the full nextest suite once to a log (Go on PATH, `ME_REQUIRE_GO=1`, as CI;
`cargo nextest run --locked --workspace --all-targets --no-fail-fast --build-jobs 6 -j 6`), each named row PASS
exactly once and no FAIL line; step C doc-tests, `cargo fmt --all --check`, `bash ci/code-docs-check.sh` and both
clippy runs of the CI `test` job on 1.85.0. Shown RED once on the base. Not covered: macOS/Windows legs (the
`/dev/full`, `ulimit` and fcntl rows are Linux/unix), the device side (unchanged), the release workflow (CI only),
the WORDING_REVIEW rows for the W2 and P1 texts.

## 9. Overlaps

- `main.rs` `emit`, the converter and `run_bundle_cli` were last touched by FIX_me_cli_arg_contract and
  FIX_me_recovery_checklist_partial_keys, both merged; no other open me row (FIX table #27, #28, #50, #51, #52 are
  merged, as is #6) touches these lines.
- io-lib 0.1.3 (build/io-lib-0.1.3) is the only dependency, for o7 alone.
- me-preview (Go) writes the plate files itself; its write errors are row #16's class (sh/Go), not this FIX.
