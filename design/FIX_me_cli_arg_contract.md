# FIX: options the operator typed are honoured or refused, never dropped, and printed commands re-run as printed (bh-me-I3, bh-me-I11, bh-me-I18)

- Source: bughunt 2026-11-19 keys **bh-me-I3**, **bh-me-I11** and **bh-me-I18**, all graded **Important**
  (coordinator `agent-reports/bughunt-1119/grade-me.md:66, :74, :80`; `design/agent-reports/bug-hunt-2026-11-19.md`
  §I3, §I11, §I18). Architect FIX table row #50, **small** track, one Opus review round. Not class B.
- Ruling **T32** (decided, architect-bughunt-1119-questions.md:74): *honour converter options given before the
  subcommand when the subcommand has no conflicting value, refuse when it does; me-preview refuses a stray operand
  with its usage line.* I11 needs no ruling: the consultant's fix (one shell argument per path) is the only reading
  of "the printed command fails as printed".
- Base: master `e09d538dcd0b552cdc58fe7ff228e2eca5f7f64e`. Author: Claude Opus 5.5 (claude-opus-5-5), 2026-11-22.
- **Two STOPs** (§7): STOP-1 (a converter option before a subcommand that has no option of that name) and
  STOP-2 (a printed path that is not UTF-8 or carries a control character or a backslash). The document is written with each
  STOP's recommendation in place; rows that depend on a STOP say so. Four new holder-visible texts are
  **PROPOSED (architect confirms)** (§4).
- **Files the build changes** (for build ordering against other me rows): `crates/me-cli/src/main.rs`,
  `crates/me-cli/tests/cli_arg_contract.rs` (new), `preview/main.go`, `preview/operand_test.go` (new),
  `design/SPEC_systemwide_payloads.md` (§5.6), `design/SPEC_me_bundle_phaseB_preview.md` (§4.1),
  `crates/me-cli/CHANGELOG.md`. This document and the gate script `scripts/fix-me-cli-arg-contract-gate.sh` are
  already committed on this branch.
- **Toolkit manual lockstep: none.** `mnemonic-toolkit` master `docs/manual/src/40-cli-reference/` has no `me`
  page; its only `me` mentions (`43-ms.md:89, :411, :438-446`) are `me sysw pack` invocations whose text and
  behaviour this FIX does not change.

## 1. The defects (measured on master e09d538d)

`me` is `target/debug/me` built `--locked` from the base; `me-preview` is `go build` of `preview/` with the
`third_party/seedhammer` submodule at `7b6f2fbb`. All records are `text:` free-text records or the signet md1
test cards of `tests/seal_cli.rs:4-6`. Files: `given` holds `text:6162`, `other` holds `text:7172`.

### 1.1 I3: a converter option before a subcommand is silently dropped

`Cli` (`crates/me-cli/src/main.rs:20-48`) declares the converter's seven options (`--in`, `--out`, `--stdout`,
`--hex`, `--base64`, `--echo`, `--allow-world-readable`) on the root parser. `run()` (`main.rs:675-722`)
dispatches every subcommand from its own fields and never reads the root ones.

| command | master result |
| --- | --- |
| `printf 'text:6364\n' \| me --in given sysw pack --no-passphrase --out r1.bin` | exit 0; `r1.bin` is byte-equal to the `text:6364` container: stdin was read, `given` was not |
| `me --out missing.bin sysw pack --no-passphrase text:6869 >r2.stdout` | exit 0; `missing.bin` absent; the 61-byte container went to stdout |
| `me --in given sysw pack --in other --no-passphrase --out r3.bin` | exit 0; `r3.bin` carries `text:7172` (the second value won, silently) |
| `me --hex sysw pack --no-passphrase --out r4.bin text:6162` | exit 0; `--hex` ignored |
| `me --out r5.uf2 seal --plaintext MD1 --plaintext MD1B --plaintext MD1C </dev/null` | exit 2, clap: `the following required arguments were not provided: --out <OUT>` (the typed `--out` was not seen) |
| `me --in given bundle --in other` | exit 4 on `other`'s content (`unrecognized HRP 'text:7'`) |
| `umask 022; me --allow-world-readable sysw pack --no-passphrase text:6162 >r12.bin` | exit 2, the F-244 world-readable stdout refusal: the operator's own override was dropped |

### 1.2 I11: the printed re-print command is not one shell argument per path

`print_pack_card` (`main.rs:2085-2091`) prints `re-print it with: me sysw show {}` with `Path::display()`. The seal
card (`main.rs:1059-1062`) prints `load:  picotool load --verify {}   (machine in BOOTSEL)` the same way. An audit
of every `eprintln!`/`println!`/`format!` in `crates/me-cli/src` that interpolates a path into a command finds
these two and no other (the remaining command texts, `main.rs:808, 1364-1368, 1422-1426`, carry fixed names or
placeholders).

| `--out` | printed | running it (`sh`) |
| --- | --- | --- |
| `my copy.bin` | `me sysw show my copy.bin` | exit 2, `error: unexpected argument 'copy.bin' found` |
| `it's.bin` | `me sysw show it's.bin` | exit 2, shell: `unexpected EOF while looking for matching '''` |
| `-dash.bin` | `me sysw show -dash.bin` | exit 2, `error: unexpected argument '-d' found` |
| `my copy.uf2` (seal) | `load:  picotool load --verify my copy.uf2   (machine in BOOTSEL)` | the path is 2 shell words |

### 1.3 I18: me-preview ignores everything from the first operand on

`runRender` (`preview/main.go:42-50`) calls `fs.Parse` and never checks `fs.NArg()`; Go's `flag` stops at the
first non-flag.

| command (stdin: MD1) | master result |
| --- | --- |
| `me-preview render --format svg --out - second-card --mode qr` | exit 0, stderr `mode text+qr`, 301,185-byte SVG |
| `me-preview render --format svg --out - second-card` | exit 0, stderr `mode text+qr`, 301,185 bytes (the same image) |
| `me-preview render --format svg --out - --mode qr` (control) | exit 0, stderr `mode qr`, 240,326 bytes |

## 2. Decisions

- **D1 (I3, T32) honour.** When a subcommand is selected and one of the seven root options was given on the
  command line before it, and the leaf subcommand (`bundle`, `seal`, `hash`, `sysw pack`, `sysw wipe`,
  `sysw show`) declares an option with the **same long name**, the run behaves exactly as if the option had been
  given after the leaf: every existing rule of the leaf applies to it unchanged (for example `sysw pack`'s
  `pass records on argv OR via --in, not both` refusal, and `seal`'s requirement of `--out`). Matching is by long name (`seal`'s `--in` is the field `in_path`).
- **D2 (I3, T32) conflict.** If the leaf was also given that option: a **flag** (`--allow-world-readable`) given
  in both places is honoured once (the values cannot differ). A **path** option given in both places with
  byte-identical values (`OsStr` equality, no canonicalisation) is honoured once; with any other values it is
  refused with T-CONFLICT, exit 2. Aliases (`./a` and `a`) count as different values: no path is resolved.
- **D3 (STOP-1 recommendation) no counterpart.** A root option whose long name the leaf does not declare is
  refused with T-NOTAKE, exit 2. Same name only: `--out` before `bundle` is not mapped to `--manifest`.
  The same-name matrix at the base:

  | leaf | `--in` | `--out` | `--allow-world-readable` | `--stdout` `--hex` `--base64` `--echo` |
  | --- | --- | --- | --- | --- |
  | `bundle` | yes | no | no | no |
  | `seal` | yes (`in_path`) | yes (required) | no | no |
  | `hash` | no | no | no | no |
  | `sysw pack` | yes | yes | yes | no |
  | `sysw wipe` | no | yes | no | no |
  | `sysw show` | no | no | no | no |

- **D4 order.** The check runs after `Cli::parse()` and before any dispatch, so a refusal reads no input and
  writes no file. It walks the root options **in command-line order** and stops at the first that is refused (one
  stderr line). The pre-parse `argv_secret_guard` (`main.rs:669-672`) is unchanged and still runs first: its
  `--allow-argv-secret` surface test reads argv[1..] literally, so `me --out f seal --allow-argv-secret <ms1>`
  stays exit 3 with the guard's text (row A9). clap's own root `conflicts_with_all` errors (for example
  `--stdout --out`) and `--help`/`--version`/the `help` subcommand are unchanged and come first.
- **D5 seal's required `--out`.** With `--out` in neither place the output stays byte-for-byte the present clap
  text (row A10). With `--out` only before `seal`, the seal runs (row A6). The mechanism is the builder's.
- **D6 (I11) one shell word per path.** Both printed commands render the path with one rule, `shell_word`:
  1. (STOP-2 recommendation) a path that is not valid UTF-8, or contains a C0 control character (U+0000-U+001F),
     U+007F or a backslash, is not rendered: the line prints the placeholder `<the file you just wrote>` instead.
  2. A path whose first character is `-` gets `./` prepended (it is relative; no `--` is needed, so picotool's
     parser is not relied on).
  3. If every character is in `A-Z a-z 0-9 _ . / , : @ + -`, the path prints as is (every path a test or doc
     prints today keeps its exact text: tempfile paths and plain names are in this set).
  4. Otherwise the path is wrapped in single quotes, each `'` written as `'\''`. `=`, `%` and `~` are outside the
     set on purpose (zsh `=cmd` expansion, tilde expansion).

  The rule was checked on this host: `printf '[%s]' 'it'\''s.bin' './-dash.bin' 'my copy.bin'` prints
  `[it's.bin][./-dash.bin][my copy.bin]` under fish, zsh and bash.
- **D7 (I18, T32).** After `fs.Parse`, if `fs.NArg() > 0`, `runRender` prints T-OPERAND naming the first leftover
  argument, then the existing usage line, and returns 1, before it reads stdin or opens `--out`. me-preview's usage
  exit code is 1 (`preview/main.go:21-27`); `me bundle --preview` never passes an operand
  (`SPEC_me_bundle_phaseB_preview.md` §5), so no `me` behaviour changes.

## 3. Spec delta

- `design/SPEC_systemwide_payloads.md` §5.6, after the paragraph that ends "transcribed correctly the first
  time." (line 950), add:
  - **Options before a subcommand (bh-me-I3, ruling T32, 2026-11-22).** The converter's options (`--in`, `--out`,
    `--stdout`, `--hex`, `--base64`, `--echo`, `--allow-world-readable`) may be typed before a subcommand. Each is
    honoured as if given after the subcommand when the subcommand has an option of the same name; a path option
    also given after the subcommand with a different value is refused, exit 2; an option the subcommand does not
    have is refused, exit 2. Nothing is read or written before these checks. The texts are FIX_me_cli_arg_contract
    §4.
  - **Printed commands (bh-me-I11, 2026-11-22).** A path printed inside a command (`re-print it with: me sysw show
    …`, and `seal`'s `load:  picotool load --verify …`) is one shell word: plain paths print as is, a leading `-`
    gets `./`, anything else is single-quoted, and a path that is not UTF-8 or carries a control character or a backslash prints
    the placeholder `<the file you just wrote>`.
- `design/SPEC_me_bundle_phaseB_preview.md` §4.1, append to the `me-preview render` bullet: "An operand after the
  flags (anything `flag` leaves unparsed) is refused before stdin is read: `me-preview: unexpected operand "<arg>"`,
  then the usage line, exit 1 (bh-me-I18, ruling T32, 2026-11-22)."

## 4. Printed texts and exit codes

All on stderr. Every text marked PROPOSED is new (architect confirms).

| id | text | exit | when |
| --- | --- | --- | --- |
| T-CONFLICT (PROPOSED) | `me: --<opt> was given before '<leaf>' and again after it, with a different value; give it once, after '<leaf>'` | 2 | D2 |
| T-NOTAKE (PROPOSED; STOP-1) | `me: --<opt> is a converter option (me with no subcommand); '<leaf>' does not take it` | 2 | D3 |
| T-SHOW (PROPOSED form) | `          re-print it with: me sysw show <shell_word(path)>` | 0 | D6; the prefix is unchanged |
| T-LOAD (PROPOSED form) | `load:  picotool load --verify <shell_word(path)>   (machine in BOOTSEL)` | 0 | D6; the frame is unchanged |
| T-OPERAND (PROPOSED) | `me-preview: unexpected operand "<arg>"` (Go `%q`), then the line `usage: me-preview [--version] \| render --format <svg\|png> --out <FILE\|-> [--mode <text+qr\|text\|qr>]` (existing, unchanged) | 1 | D7 |

`<leaf>` is one of `bundle`, `seal`, `hash`, `sysw pack`, `sysw wipe`, `sysw show`. `<opt>` is the long name
without its value. With STOP-2's recommendation, T-SHOW's placeholder line is the existing text
`re-print it with: me sysw show <the file you just wrote>` (`main.rs:2089`), and T-LOAD's is
`load:  picotool load --verify <the file you just wrote>   (machine in BOOTSEL)` (new, PROPOSED). No other exit
code or text changes.

## 5. RED rows

Every row is a row of `scripts/fix-me-cli-arg-contract-gate.sh --cli` (section A), which asserts the post-fix
expectation; the commands are the ones in that script, run in a fresh directory under `target/`. Measured on the
base: `gate --cli: FAIL`, with exactly the RED rows below failing and the controls passing (A4 is an honour row
that master happens to satisfy, because the second value wins).

| row | test (TRACE name) | expected after the fix | master |
| --- | --- | --- | --- |
| A1 | `bh_me_i3_root_in_honoured_by_sysw_pack` | exit 0; container = `text:6162`'s | exit 0, `text:6364`'s (RED) |
| A2 | `bh_me_i3_root_out_honoured_by_sysw_pack` | exit 0; `a2.bin` = `text:6869`'s; stdout 0 bytes | file absent, stdout 61 bytes (RED) |
| A3 | `bh_me_i3_root_in_conflict_refused_by_sysw_pack` | exit 2; stderr is exactly T-CONFLICT(`in`, `sysw pack`); no `a3.bin` | exit 0 (RED) |
| A4 | `bh_me_i3_root_in_same_value_honoured` | exit 0; container = `text:6162`'s | passes |
| A5 | `bh_me_i3_root_converter_option_refused_by_sysw_pack` (STOP-1) | exit 2; stderr is exactly T-NOTAKE(`hex`, `sysw pack`); no `a5.bin` | exit 0 (RED) |
| A5b | `bh_me_i3_first_misplaced_option_is_named` (STOP-1) | `me --echo --in given bundle --in other`: exit 2; stderr is exactly T-NOTAKE(`echo`, `bundle`) | exit 4 (RED) |
| A5c | same test (P-2: command-line order) | `me --in given --echo bundle --in other`: exit 2; stderr is exactly T-CONFLICT(`in`, `bundle`) | exit 4 (RED) |
| A6 | `bh_me_i3_root_out_honoured_by_seal` | exit 0; `a6.uf2` exists; stderr line `load:  picotool load --verify a6.uf2   (machine in BOOTSEL)` | exit 2, clap missing `--out` (RED) |
| A6b | `bh_me_i3_root_in_honoured_by_seal` | `me --in recs.txt seal --out a6b.uf2 </dev/null` (recs.txt = the three md1 test cards, one per line): exit 0; `a6b.uf2` exists; stderr line `load:  picotool load --verify a6b.uf2   (machine in BOOTSEL)` | exit 2, `me: nothing to seal` (RED) |
| A6c | `bh_me_i3_root_out_honoured_by_sysw_wipe` | `me --out a6c.bin sysw wipe --fill zeros`: exit 0; `a6c.bin` byte-equal to `me sysw wipe --fill zeros --out a6c.ref`; stdout 0 bytes | exit 0, `a6c.bin` absent, 65,536 bytes on stdout (RED) |
| A7 | `bh_me_i3_root_in_conflict_refused_by_bundle` | exit 2; stderr is exactly T-CONFLICT(`in`, `bundle`) | exit 4 (RED) |
| A8 | `bh_me_i3_root_allow_world_readable_honoured_by_sysw_pack` | umask 022, stdout to a file: exit 0; stdout = `text:6162`'s container | exit 2, F-244 refusal (RED) |
| A9 | `bh_me_i3_controls_guard_seal_text_converter_unchanged` (control) | `me --out a9.uf2 seal --allow-argv-secret <MS1>`: exit 3, first line starts `me: argument 5 on ARGV`; no file | passes |
| A10 | same test (control) | `me seal --plaintext MD1 </dev/null`: exit 2, stderr byte-equal to the clap text in the gate script | passes |
| A11 | same test (control) | `me --in md1.txt --hex`: exit 0, 150 hex characters | passes |
| A12 | `bh_me_i11_printed_show_command_runs_as_printed` | for `my copy.bin`, `it's.bin`, `-dash.bin` the printed tail is `me sysw show 'my copy.bin'`, `me sysw show 'it'\''s.bin'`, `me sysw show ./-dash.bin`; `sh -c` of it (with `me` on PATH, same cwd) exits 0 and its `digest:` line equals pack's | printed unquoted; running it exits 2 (RED) |
| A13 | `bh_me_i11_printed_show_plain_path_unchanged` (control) | `--out plain.bin` prints `me sysw show plain.bin` | passes |
| A14 | `bh_me_i11_printed_picotool_path_is_one_shell_word` | `seal --out='my copy.uf2'`: line is `load:  picotool load --verify 'my copy.uf2'   (machine in BOOTSEL)`; the path part is 1 `sh` word | unquoted, 2 words (RED) |
| A15 | `TestBhMeI18StrayOperandRefused` | `render --format svg --out a15.svg second-card`: exit 1; stderr is exactly T-OPERAND(`second-card`) + usage; no `a15.svg`; stdout empty | exit 0 (RED) |
| A16 | `TestBhMeI18StrayOperandBeforeModeRefused` | `… --out a16.svg second-card --mode qr`: the same two lines, exit 1, no file | exit 0, `mode text+qr` (RED) |
| A17 | `TestBhMeI18ModeQrControl` (control) | `… --out a17.svg --mode qr`: exit 0; stdout `mode qr`; file non-empty | passes |

The base also showed each I11 printed command failing when run (§1.2 table), with the author's probe run before
the gate existed.

## 6. Tests

No existing test changes. Checked: no test in `crates/me-cli/tests/` passes a root option before a subcommand
(grep of every `.args([...])` carrying one of the seven options and a subcommand name finds them only after the
subcommand); the tests that read the pack card (`sysw_cli.rs:2826, :2844`) assert `contains("re-print it with")`
on tempdir paths, which D6.3 prints unchanged; `seal_cli.rs:360` anchors on the `load:` banner only.

New tests, each named exactly as in §5 (the gate counts each PASS once):

- `crates/me-cli/tests/cli_arg_contract.rs` (new): the fifteen Rust tests of §5. Each runs the built binary in its own
  tempdir as the current directory, so the printed paths are the relative names the rows pin.
  `bh_me_i3_controls_guard_seal_text_converter_unchanged` holds A9, A10 and A11.
  `bh_me_i11_printed_show_command_runs_as_printed` runs the printed tail through `sh -c` with the directory of
  `env!("CARGO_BIN_EXE_me")` first on `PATH` (the binary is named `me`). A1 feeds stdin; the refusal rows assert
  the whole of stderr, not `contains`.
- `preview/operand_test.go` (new): the three Go rows through `run(args, stdin, stdout, stderr)`. The two refusal
  rows pass a stdin reader that records whether it was read and assert it was not, and an `--out` path in
  `t.TempDir()` that must not exist afterwards.
- `crates/me-cli/CHANGELOG.md` `[Unreleased]` / `### Fixed`: one entry for the three keys (wording is the
  builder's; it states that a converter option before a subcommand is honoured or refused, that printed commands
  quote their path, that me-preview refuses a stray operand, and that exit codes elsewhere are unchanged).

## 7. STOPs and the points this document decides (PROPOSED, architect confirms)

- **STOP-1 (D3) a converter option the subcommand does not have.** T32 decides "honour when no conflicting value,
  refuse when conflicting", which presumes the subcommand has the option. Cases: `--stdout`, `--hex`,
  `--base64`, `--echo` before any subcommand; `--out` or `--allow-world-readable` before `bundle`; anything
  before `hash` or `sysw show`. Options: (a) refuse with T-NOTAKE, exit 2, same-name matching only;
  (b) map by meaning where one exists (`--out` -> `bundle --manifest`, `--stdout` -> `sysw pack` with no
  `--out`) and refuse the rest; (c) proceed with a warning. **Recommend (a)**: (c) keeps the silent-drop class the
  key reports (a warning is easy to miss in a long pack card), and (b) guesses meanings the operator never stated
  (`--out` is the NDEF file for the converter, `--manifest` is JSON). Rows A5, A5b and D3 assume (a).
- **STOP-2 (D6.1) a path that cannot be printed as a shell word.** A non-UTF-8 path prints lossily through
  `Path::display()` today (U+FFFD), so the printed command names a different file; a control character
  (newline, ESC) splits the line or reaches the terminal. Options: (a) print the placeholder
  `<the file you just wrote>` (the existing `main.rs:2089` text; T-LOAD gets the same placeholder);
  (b) ANSI-C quoting `$'…'` (bash and zsh only; fish and POSIX `sh` read it differently); (c) keep the lossy text.
  **Recommend (a)**: it never prints a wrong command and reuses a text the operator already sees for stdout.
  No row covers it (a non-UTF-8 name needs an OS-specific fixture); the builder adds a unit test of `shell_word`
  for one non-UTF-8 `OsStr` (`#[cfg(unix)]`, `OsStrExt::from_bytes`), one path with `\n` and one with a
  backslash (`a\\b.bin`; fish reads `\\` inside single quotes as one backslash). Ruled (a) with the backslash
  extension (architect-bh-batch5, 2026-11-22).
- **P-1 (D2) equal values are not a conflict.** T32 refuses a *conflicting* value; byte-identical paths do not
  conflict, so `--in a … --in a` is honoured (A4). Paths are compared as bytes, never resolved.
- **P-2 (D4) first offender only.** One refusal line, for the first refused root option in command-line order.
- **P-3 (D6.2) `./` rather than `--`.** The consultant suggested an option terminator; `./` works for both printed
  commands without relying on how picotool parses `--`, and prints one fewer token.
- **P-4 texts.** T-CONFLICT, T-NOTAKE, T-OPERAND and the quoted forms of T-SHOW/T-LOAD (§4).

## 8. Gate and records

- Gate: `bash scripts/fix-me-cli-arg-contract-gate.sh` (A: the CLI rows; B: the whole nextest suite once with each
  named Rust test passing once; C: me-preview `go test ./...` with each named Go test passing once, and
  `gofmt -l preview` empty; D: doc-tests, `cargo fmt --check`, both CI clippy runs on 1.85.0). The script's header
  states what it does not cover. `--cli` runs section A only.
- FOLLOWUPS: this FIX closes no existing line (grep of `design/FOLLOWUPS.md` for the three keys, the clap
  setting, the quoting and the operand check finds none). The lines to append after the merge are in the author's
  report, in the coordinator's FOLLOWUPS format.
