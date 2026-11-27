# Exit-code survey: `mt` and `me` (surveyor: mt-me)

Heads surveyed: mnemonic-transaction `c0ac240` (mt 0.2.0), mnemonic-engrave `c667773` (me 0.13.0).
Binaries: `/home/user/wt/target/debug/{mt,me}`. All runs from `/home/user/wt/ec/tmp-mt-me/`, umask 077,
stdin `</dev/null` unless shown. `B=/home/user/wt/target/debug`. Paths below are relative to
`/home/user/wt/`. "M" = measured, "S" = source only, "I" = inferred.

Inputs (all published vectors / throwaways): `tx.hex` + `set.txt` = vector "even" of
`mnemonic-transaction/crates/mt-codec/src/test_vectors/mt1_v1.json` (6 strings); `set1err.txt` = one
substituted character in string 1; `setbad.txt` = 16 substituted characters; `missing.txt` = string 1
dropped; `other.hex` = another corpus vector; `md.txt` = `md1yqpqqxqq8xtwhw4xwn4qh`; `mdbad.txt` = same
with last char changed; `ms.txt` = `ms10entrsqqqq…cj9sxraq34v7f` (all-zero test entropy, me-cli
`tests/cli.rs:4`); `bundle.txt` = md1 + the two mk1 chunks of `tests/vectors/bundle-md1-mk1.json`;
`garbage.txt` = "garbage"; `empty.txt` = 0 bytes.

Shared constants (asked): **mnemonic-io-lib publishes NO exit-code constant, by design**
(`mnemonic-engrave/crates/mnemonic-io-lib/src/exit.rs:1-33`, `lib.rs:33-36`: "No exit-code integer,
and no binary→code mapping"). It exports refusal *decisions* (`write_block`/`WriteBlock`,
`no_records_guard`, `same_file`), and each binary maps them itself. `EXIT_REFUSED` etc. are private
consts of **me-cli only** (`crates/me-cli/src/main.rs:495-500`; `EXIT_WRITE` in
`crates/me-cli/src/stdout_out.rs:18`). `mt` (dep `mnemonic-io-lib = "0.1.3"` from crates.io,
`mt-cli/Cargo.toml:25`) uses the lib's decisions but has no named exit constants at all. The only
`process::exit(3)` in the lib (`write.rs:764`) is a test child helper, not API.

---------------------------------------------------------------------------------------------------

## A. Exit-code inventory

### mt (`mnemonic-transaction/crates/mt-cli/src/main.rs`)

Mechanism: `fn main() -> std::process::ExitCode` (main.rs:297). No error enum with a code mapping:
every verb returns `Result<(), Refusal>` and `run()` (main.rs:371-378) maps `Ok → 0`, **any `Err` → 1**.
Clap: `Cli::try_parse_from` (main.rs:345); on error the code is clap's own `e.exit_code()`
(main.rs:357: 0 for help/version, 2 for usage), not `Error::exit()`. Every final code passes through
`out::exit_code` (`src/out.rs:91-99`), which turns **0 into 1** if a stdout/`--out` write failed (not
BrokenPipe) and keeps a non-zero code. No `process::exit`, no `catch_unwind`, no panic hook; no
`panic=` profile setting (unwind).

| code | meaning | verbs | decided at | evidence |
|---|---|---|---|---|
| 0 | every check passed (incl. with WARNINGs, with corrections applied, with no node) | all | main.rs:373 | M `$B/mt encode --in tx.hex` → 0; `mt decode --in set.txt` → 0; `mt verify --in set.txt` → 0; `mt inspect --in set.txt --bitcoin-cli /nonexistent/bitcoin-cli` → 0 |
| 0 | help / version | all | main.rs:357 (clap 0) | M `mt --help` → 0; `mt -V` → 0; `mt help` → 0; `mt encode --help` → 0 |
| 1 | any `Refusal` (94 rows in `tests/refusals.toml`): invalid artifact, verify mismatch, no input, I/O read error, policy refusal, flag-combination refusal, node-proved refusal | all | main.rs:374-376 | M (see B) |
| 1 | argv guard: transaction / mt1 on the command line (§8.2f), before clap | all | main.rs:340-343 (`ExitCode::FAILURE`) | M `mt encode $(cat tx.hex)` → 1; `mt verify $(head -1 set.txt)` → 1 |
| 1 | output write failed (stdout or `--out`), incl. `--help`/`--version` | all | out.rs:91-99 | M `mt encode --in tx.hex >/dev/full` → 1; `mt --help >/dev/full` → 1; `mt encode --in tx.hex 1</dev/null` → 1 (EBADF); `mt encode --in tx.hex --out /nonexistent/dir/o.txt` → 1 |
| 2 | clap usage error (unknown flag/verb, bad value type, no subcommand) | all | main.rs:357 | M `mt --nosuch` → 2; `mt nosuchverb` → 2; `mt encode --group-size x --in tx.hex` → 2; `mt` (no args, prints help to stderr) → 2 |
| 64 | an argv element is not UTF-8 (T22/I24), before anything else | all | main.rs:300-305 | M `mt verify $'\xff\xfe'` → 64 (message is in REFUSED form, "§10.10, argument 2 is not valid UTF-8") |
| 101 | panic (Rust default; main does not catch) | — | none | S; tests assert `!= Some(101)` in 12 places |

### me (`mnemonic-engrave/crates/me-cli/src/main.rs`)

Mechanism: `fn main() { std::process::exit(run()); }` (main.rs:675-677); `run() -> i32`. Named private
consts (main.rs:495-500, with the doc table at :481-494): `EXIT_OK=0`, `EXIT_USAGE=2`,
`EXIT_REFUSED=3`, `EXIT_INVALID=4`, `EXIT_WRITE=1` (`stdout_out::EXIT_WRITE`, stdout_out.rs:18).
Sub-mappings: `BundleError::exit_code()` (`src/bundle.rs:61-67`: Empty→2, RefusedSecret→3, else 4);
`SealError::Iterations → 2, else 4` (main.rs:1278-1281); `PreviewError::{Render,EmptyOutput} → 4, else 2`
(main.rs:1533-1540); `read_records` returns `(msg, code)` pairs (main.rs:3513-3749).
Clap: `Cli::try_parse()` then `clap_exit()` (main.rs:867-884) returns clap's `e.exit_code()` (0/2) after
printing through the checked writer. **One `clap::Error::exit()` site**: main.rs:830 (`seal` with no
`--out`, re-parse to get clap's own wording; exits 2). Two direct `process::exit` sites: main.rs:676
and stdout_out.rs:49 (failed stdout write → 1 immediately, mid-verb). No `catch_unwind`/panic hook.

| code | meaning | verbs | decided at | evidence |
|---|---|---|---|---|
| 0 | success (incl. warnings; preview skipped because no sidecar) | all | EXIT_OK | M `$B/me --hex --in md.txt` → 0; `me bundle --in bundle.txt` → 0; `me seal --out s.uf2 --plaintext md1yqpqqxqq8xtwhw4xwn4qh` → 0; `me hash --unsealed md1…` → 0; `me sysw pack --out p.bin --no-passphrase --in md.txt` → 0; `me sysw show p.bin` → 0; `me sysw wipe --out w.bin` → 0 |
| 0 | help / version | all | main.rs:883 | M `me --help` → 0; `me --version` → 0; `me help bundle` → 0 |
| 1 | output write failed: stdout (incl. help text) or `--out`/`--manifest` file | all | stdout_out.rs:49; `write_out` main.rs:1656-1671 | M `me --hex --in md.txt >/dev/full` → 1; `me --help >/dev/full` → 1; `me --hex --in md.txt 1</dev/null` → 1; `me --out /nonexistent/d/o --in md.txt` → 1; `me bundle --in bundle.txt --manifest /nonexistent/d/m.json` → 1; `me seal --out /nonexistent/d/s.uf2 --plaintext md1…` → 1; `me sysw pack --out /nonexistent/d/p.bin …` → 1; `me sysw wipe --out /nonexistent/d/w.bin` → 1 |
| 2 | "usage" — in practice a wide bucket: clap errors; no/empty input; **input file unreadable**; no output mode; out-of-range flag value; **world-readable / terminal stdout refusal**; sidecar problems; `--manifest` aliasing a plate | all | EXIT_USAGE (≈45 sites) | M (see B) |
| 3 | policy refusal: secret/bearer on argv; ms1 to the converter or bundle; seed to `seal` without `--seal-secret`; `sysw pack --as` document refused | all | EXIT_REFUSED: main.rs:895, 1021, 1265, 1943, 2060, 3621; bundle.rs:65 | M `me --hex --in ms.txt` → 3; `me --hex ms10…` → 3; `me --nosuchflag ms10…` → 3; `me bundle --in bundle-ms.txt` → 3; `me seal --out s2.uf2 --in ms.txt` → 3; `me sysw pack --out p2.bin --no-passphrase ms10…` → 3; `me sysw pack … tx:<hex>` → 3; `me sysw pack … --as descriptor --in md.txt` → 3 |
| 4 | invalid input artifact / unmet `--expect` / sidecar render failure | all | EXIT_INVALID (≈22 sites); bundle.rs:66 | M `me --hex --in mdbad.txt` → 4; `me --hex --in garbage.txt` → 4; `me bundle --in bundle-incomplete.txt` → 4; `me hash --unsealed md1…q` (bad) → 4; `me seal --out s3.uf2 --plaintext <bad md1>` → 4; `me sysw pack … --in garbage.txt` → 4; `me sysw show garbage.txt` → 4; `me sysw pack … --expect cosigner --in md.txt` → 4 |
| 101 | panic (not caught) | all | none | **M `me --hex $'\xff\xfe'` → 101** (`std::env::args()` panics on non-UTF-8 argv at main.rs:893/899/901; "called `Result::unwrap()` on an `Err` value: "\xFF\xFE""). `mt` returns 64 for the same input. |

### me-preview Go sidecar, as seen by `me` (`mnemonic-engrave/preview/main.go`)

Sidecar itself: `os.Exit(run(...))` (main.go:19); returns only **0 or 1** (main.go:28-129; 1 for every
failure incl. bad args). Not built in `target/debug`, so the real sidecar was not run; `me`'s mapping
was measured with throwaway shell stubs via `ME_PREVIEW_BIN`.

| situation | `me bundle --preview` code | decided at | evidence |
|---|---|---|---|
| no sidecar beside `me`, `ME_PREVIEW_BIN` unset | **0**, note "preview skipped (install me-preview)", manifest still emitted | main.rs:1443-1449 | M `me bundle --in bundle.txt --preview pv1` → 0 |
| `ME_PREVIEW_BIN` set, file missing | 2 | main.rs:1428-1436 | M `ME_PREVIEW_BIN=/nonexistent/me-preview me bundle --in bundle.txt --preview pv1` → 2 |
| sidecar `--version` mismatch | 2 | main.rs:1454-1473 | M stub printing `me-preview 0.0.1` → 2 |
| sidecar `--version` exits non-zero (stub exit 7) / not executable | 2 | main.rs:1474-1477 | M → 2 ("me-preview --version failed (exit 7)"); non-exec file → 2 |
| preview dir missing / holds `plate-*` files / unreadable | 2 | main.rs:1481-1512 | M `--preview /nonexistent/pv` (with a sidecar) → 2; dir with `plate-9.svg` → 2. NB with NO sidecar the same missing dir → **0** (skip happens first) |
| render exits non-zero (stub exit 1) | **4** | main.rs:1534 | M → 4 ("me-preview render failed (exit 1)") |
| render exits 0 but no/garbage output | 4 | main.rs:1538 | M → 4 |
| sidecar cannot be spawned during render | 2 | main.rs:1539 | S |
| `--manifest` is one of the plate files | 2 | main.rs:1127-1160 | M → 2 |

The sidecar's own code is never propagated; it appears only inside the stderr text.

---------------------------------------------------------------------------------------------------

## B. Class matrix

| # | class | mt | me |
|---|---|---|---|
| 1 | success | 0 | 0 |
| 2 | "answer is no" | **1** — `mt verify --in set.txt --transaction other.hex` → 1 (§1.1 REFUSED). `inspect` reporting STATUS DEAD/UNKNOWN is **0** (S: tests/inspect.rs:415; it is a report, not a verdict). | no verify verb. Nearest: `sysw pack --expect <kind>` not met → **4** (M) |
| 3 | invalid artifact | **1** on every verb. M: `mt verify --in setbad.txt` → 1; `decode`/`inspect` same → 1; `mt verify --in wronghrp.txt` → 1; `mt verify --in garbage.txt` → 1; `mt encode --in garbage.txt` → 1; `mt verify --in missing.txt` (incomplete set) → 1 | **4** on every verb (M, table above); `sysw show trunc.bin` → 4 (after printing 41 B of header to stdout) |
| 4 | usage error | clap → **2** (M). But flag-combination errors the program decides are **1**: `mt verify --json --quiet --in set.txt` → 1; `mt inspect --transaction tx.hex --in set.txt` → 1 (19 "§10.10" refusal rows). Non-UTF-8 argv → **64** | **2** for clap and for program-decided ones: `me --nosuch` → 2; `me nosuchverb` → 2; `me --hex --base64 …` → 2; `me --in md.txt` (no output mode) → 2; `me hash md1…` (neither --sealed/--unsealed) → 2; `me seal --plaintext md1…` (no --out, via `Error::exit()`) → 2; `me seal … --iterations 5` → 2; `me sysw pack … --iterations 5` → 2; `--expect nonsense` → 2; `sysw wipe --fill bogus` → 2; `me --hex bundle …` → 2; `me sysw` → 2. Non-UTF-8 argv → **101 (panic)** |
| 5 | no / empty input | **1**: `mt verify </dev/null` → 1; `mt decode </dev/null` → 1; `mt encode </dev/null` → 1; `mt inspect </dev/null` → 1; `mt verify --in empty.txt` → 1; `mt encode --in empty.txt` → 1 | **2**: `me --hex </dev/null` → 2; `me --hex --in empty.txt` → 2; `me bundle </dev/null` → 2; `me bundle --in empty.txt` → 2; `me seal --out s2.uf2 </dev/null` → 2; `me hash --unsealed` → 2; `me sysw pack --out p2.bin --no-passphrase </dev/null` → 2; `… --in empty.txt` → 2; `… -` → 2. Exception: `me sysw show empty.txt` → **4** (TooShort(0)) |
| 6 | policy / safety refusal | **1** for all: argv material (M above); world-readable stdout `mt encode --in tx.hex > wr.txt` (0644) → 1 (§8.2h; `--allow-world-readable` → 0); same file `mt encode --in same.hex --out same.hex` → 1 (§8.2i). `mt decode` to a 0644 stdout only WARNS → 0 (M) | secret/bearer/content refusals → **3** (M above). **Destination refusals → 2**: `me --hex --in md.txt > wr.txt` (0644) → 2; `me sysw pack --no-passphrase --in md.txt > wr.txt` → 2; terminal stdout → 2 (S, main.rs:1702-1709; pinned by tests/terminal_destination.rs). `--manifest` = plate file → 2 |
| 7 | I/O error | read: **1** (`mt verify --in /nonexistent/x` → 1; `mt encode --in /nonexistent/x` → 1). write: **1** (M above) | read: **2** (`me --hex --in /nonexistent/x` → 2; `me bundle --in /nonexistent/x` → 2; `me seal --out s3.uf2 --in /nonexistent/x` → 2; `me sysw pack … --in /nonexistent/x` → 2; `me sysw show /nonexistent/x` → 2). write: **1** (M above) |
| 8 | repair | No `repair` verb. Correction is implicit in decode/verify/inspect: ≤4 substitutions per string are corrected and the verb returns **0**, with "CORRECTION APPLIED. 1 chunk needed repair" on stderr only (M `mt verify --in set1err.txt` → 0; `decode` → 0 and still emits hex; `inspect` → 0). The set-level content id re-derivation is the oracle. Uncorrectable → **1** (M `setbad.txt`). So mt has no 5/4/2 D26 codes. | No repair; `me` applies no correction at all ("no correction is applied -- the string travels verbatim"); a bad checksum is **4** |
| 9 | external dependency failure | bitcoin-cli absent, failing (stub `exit 1`) or silent (stub `exit 0`): **0** with a WARNING / STATUS UNKNOWN. M: `mt inspect --in set.txt --bitcoin-cli ./cli-fail.sh` → 0; `… ./cli-null.sh` → 0; `mt verify … --bitcoin-cli ./cli-fail.sh` → 0; `mt encode --in tx.hex --bitcoin-cli ./cli-fail.sh` → 0. A node that *answers* can cause a refusal → 1 (S: §8.5 spent input with confirmed parent, §6a value disagrees with chain; tests/refusals.rs, node_tristate.rs). | sidecar: absent → 0; bad env path / version / spawn → 2; render failure → 4 (table above) |
| 10 | partial success / warnings-only | **0** (M: nLockTime WARNING on encode → 0; no-node WARNING → 0; correction applied → 0) | **0** (M: bundle chunk-set-id warning → 0; `sysw pack --in ms.txt --no-passphrase` weak-protection WARNING → 0; `--passphrase-ask` ignored on a non-secret payload → 0; `--echo`, long-input warning → 0) |
| 11 | internal error / panic | 101, not caught (S; none reproduced) | 101, not caught; **reproduced with non-UTF-8 argv** |
| 12 | other | help/version 0; no-args 2; closed pipe: verb's own code (M `mt encode --in tx.hex \| true` → PIPESTATUS 0); SIGINT etc.: default signal death (I) | help/version 0; `me` with no args and empty stdin → 2 ("no input given"); `me sysw` → 2; closed pipe → verb's own code (M `me --hex --in md.txt \| true` → 0) |

---------------------------------------------------------------------------------------------------

## C. Collisions and surprises

1. **mt has one failure code for everything it decides (1).** Verify mismatch, corrupt set, empty stdin,
   missing input file, unsafe file mode, argv leak, flag-combination error, full disk and "the node
   proved this input spent" are indistinguishable by code. Only clap (2) and non-UTF-8 argv (64) differ.
2. **mt usage errors return three codes**: clap 2; program-decided flag conflicts 1 (`--json --quiet`,
   `--transaction` on `inspect`, §10.10); non-UTF-8 argv 64. The 64 contradicts the "mt uses 2" row of
   `cc/shared.md` CLI-14 for that one path (and is the only sysexits-style code in mt).
3. **me's 2 means at least five things**: clap usage; no input; **unreadable input file** (I/O);
   **world-readable/terminal stdout** (a safety refusal, while secret-on-argv is 3); and sidecar
   environment failures. The const's own doc (main.rs:483) says "Nothing was read and nothing about the
   DATA was judged", which is not true of the destination refusal after a successful convert, nor of
   `--manifest`-is-a-plate after rendering.
4. **Same class, different code across the two binaries**: invalid artifact mt 1 / me 4; no input
   mt 1 / me 2; argv secret mt 1 / me 3; world-readable stdout mt 1 / me 2; unreadable input mt 1 / me 2;
   non-UTF-8 argv mt 64 / me 101. Only write failure (1), clap usage (2) and success agree.
5. **me read error (2) vs write error (1)**: two I/O failures, two codes.
6. **me empty input is 2 on every verb except `sysw show`** (empty file → 4).
7. **me sidecar**: a failed render is 4 ("invalid input"), though a crash of the sidecar for any reason
   also lands there; a missing preview directory is 2 with a sidecar and 0 without one.
8. **me panics (101) on non-UTF-8 argv** — before the argv secret guard runs. Not documented anywhere.
9. **`me seal --allow-argv-secret ms10…` → 3** (the flag is accepted, then the missing `--seal-secret`
   refuses): two different refusals share 3, fine, but the override alone does not reach 0.
10. **mt "repair applied" is silent in the exit code** (0), unlike D26's 5/4 in md/mk/ms/mnemonic. The
   uniformity spec's table marks mt "n/a" for repair; the behaviour exists, inside decode/verify/inspect.
11. **Docs vs. behaviour**:
   - `mnemonic-engrave/design/SPEC_constellation_cli_uniformity.md:1047-1054` §6f table, `me` row:
     "4 = unplaceable record; 2 = terminal refusal; 3 = policy refusal (`EXIT_REFUSED`)" — accurate but
     omits 1 (write failure, added later by ruling T6); `mt` row "invalid artifact 1, repair n/a" matches.
   - Same file line 35: "`me` and `mt` refuse at exit 2 (F-244, F-252)" for world-readable stdout —
     **mt measures 1**, me 2. (Line 41-46 of the same file does say mt returns only 1 there.)
   - `cc/shared.md:488`: "`mt decode -` 1" on empty stdin — consistent with measured `mt decode` → 1.
   - `mnemonic-transaction/design/SPEC_mt_v0_1.md:3649-3660`: "Still unspecified, and deliberately: exit
     codes beyond 0. `0 = every check passed` is fixed … the rest of the code space is
     implementation's." So mt's 1/2/64 are unspecified by its own spec, apart from §10.10 "(exit 1)" at
     :3618-3620 and write failure "exits 1" at :2623, :2711.
   - mnemonic-io-lib `exit.rs:7-9` records "`mt` 2, `me` 2" for clap usage and "`md` 1, `mk` 2, `ms` 1,
     `mnemonic` 2" for an invalid artifact (the mk figure is stale per CLI-14; not my binaries).

---------------------------------------------------------------------------------------------------

## D. Documented codes

`--help`: neither binary documents exit codes. `mt {encode,decode,verify,inspect} --help`: 0 mentions.
`me`: only `me sysw pack --help` mentions "exit 0" in passing (the `--expect` rationale).

Toolkit manual `mnemonic-toolkit/docs/manual/src/40-cli-reference/`: chapters 41-46 cover mnemonic, md,
ms, mk, mr, mr-sitting. **There is no `mt` or `me` chapter**, and no exit code for either is stated
anywhere under `docs/manual/src` (grep).

Repo docs / source docs:
- `mnemonic-engrave/README.md:132` and `crates/me-cli/README.md:22`: "ms1 is refused (exit 3)".
- `mnemonic-engrave/README.md:136`: output-write failure line and exit 1 (same sentence as mt's README).
- `mnemonic-engrave/README.md:152`: "A set-but-missing `ME_PREVIEW_BIN` is a hard error (exit 2). A
  `--manifest` that is one of the plate previews is refused (exit 2)."
- `mnemonic-engrave/crates/me-cli/src/main.rs:481-500`: the 2/3/4 vocabulary table and consts;
  `stdout_out.rs:1-18`: write failure 1; `bundle.rs:61-67`; `main.rs:1529-1540` (sidecar 4 vs 2).
- `mnemonic-engrave/design/SPEC_constellation_cli_uniformity.md` §6f, lines 1017-1075 (table at
  1047-1054); also lines 29, 35, 41-46, 166-173, 691, 971, 1194.
- `mnemonic-engrave/crates/mnemonic-io-lib/src/exit.rs:1-33`, `lib.rs:17-36`, `fd.rs:7-8`.
- `mnemonic-transaction/README.md:26`: "`mt: cannot write the output: <error>` and exits 1".
- `mnemonic-transaction/design/SPEC_mt_v0_1.md`: :819-820 (decode exits non-zero unless every check
  passes), :2623, :2667-2689 (decode warns, exit 0), :2709-2712, :3618-3620 (§10.10 exit 1),
  :3649-3660 (codes beyond 0 unspecified).
- `mnemonic-transaction/crates/mt-cli/src/main.rs:299` (64), :349 (clap 2), :369 (0 = every check
  passed); `src/out.rs:8-14`; `tests/refusals.toml:556` (64).
- `cc/shared.md:479-500` CLI-14.

---------------------------------------------------------------------------------------------------

## E. Tests that pin exit codes

mt (`mnemonic-transaction/crates/mt-cli/tests/`, 21 files): ~130 assertions on an exact integer
(`Some(0)` ≈82, `Some(1)` ≈34, `Some(2)` 1, 64 pinned by `bh1119_mt_i24_non_utf8_argument_is_refused_exit_64`,
`!= Some(101)` 12) plus ~177 `.success()`/`.failure()`/`status.success()` checks that only see
zero/non-zero. Heaviest: `bughunt.rs` (58 status lines), `decode_verify.rs` (46), `refusals.rs` (33),
`prefix_corrected.rs` (30), `encode.rs` (20), `verdict_notice.rs` (18), `tx_record.rs` (16),
`inspect.rs` (14), `bh1119_mt_output_writes.rs` (13). Renumbering mt's single 1 into several codes
touches ~34 exact pins; the ~177 boolean checks would not notice.

me (`mnemonic-engrave/crates/me-cli/tests/`, 20 files with exact pins): ~110 exact integers
(`Some(0)` 22, `Some(1)` 28, `Some(2)`/`.code(2)` 22, `Some(3)`/`.code(3)` 11, `Some(4)`/`.code(4)` 18,
101 ×2, plus `EXIT_*` names) and ~236 boolean checks. Heaviest: `bh1119_me_output_integrity.rs` (31),
`sysw_cli.rs` (25), `cli.rs` (18; includes the table test `the_exit_code_vocabulary_is_one_vocabulary`,
cli.rs:1111), `argv_secret_guard.rs` (11), `descriptor_refusals.rs` (10), `preview_version_probe.rs`
(9), `dash_stdin.rs` (9), `bughunt.rs` (9), `exit_digits.rs` (8; written specifically to pin 2 vs 3 vs
1 at five sites). Unit test `bundle.rs:696 exit_codes_match_spec` pins 2/3/4.
Go sidecar: `preview/*_test.go` test `run()`'s 0/1 return (not counted).

---------------------------------------------------------------------------------------------------

## F. Consumers that read the exit code

mt:
- `mnemonic-transaction/scripts/live-smoke-test.sh:74,81,86,91` — `"$MT" <verb> … || fail_step <verb> $?`
  (zero/non-zero only; the code is printed, not branched on).
- `mnemonic-transaction/scripts/journeys.sh:43-227` — runs `target/debug/mt`, zero/non-zero + stderr text.
- `mnemonic-transaction/scripts/mutate-refusals.sh`, `check-refusal-coverage.sh` — via the test suite.
- `mnemonic-transaction/.github/workflows/release.yml:222` and
  `mnemonic-engrave/scripts/release-workflows/emit.py:43` — `"$BIN" --help >/dev/null` smoke (0 expected).
- `mnemonic-engrave/scripts/gen-tx-journey.sh:50` — runs `mt` to generate a journey (non-zero aborts).
- The documented operator pipeline `mt decode … > tx.hex && …` (SPEC_mt_v0_1.md:836) and
  `mt encode --qr --in tx.hex | me sysw pack --out p.bin` (me-cli `src/sysw/advice.rs:14`) depend only on
  zero/non-zero; in a pipe mt's code is lost unless `pipefail` (the uniformity spec's §6g point).
- No other binary spawns `mt`. mnemonic-gui's `pinned-upstream.toml` pins mnemonic/md/ms/mk only.

me:
- `mnemonic-engrave/scripts/fix-me-cli-arg-contract-gate.sh:64-109` — branches on exact `rc = 0` / `rc = 2`.
- `mnemonic-engrave/scripts/fix-me-admission-canonical-gate.sh:52-77` — exact `rc = 4` / `rc = 0`.
- `mnemonic-engrave/scripts/fix-me-checklist-partial-keys-gate.sh:47-48` — `rc = 0`.
- Other gates that run `me` (zero/non-zero or via nextest): `fix-me-output-integrity-gate.sh`,
  `fix-me-descriptor-input-strict-gate.sh`, `fix-me-inspection-fail-open-gate.sh`,
  `fix-sec-canonical-keeps-xpub-depth-gate.sh`, `spec-s1-gate.sh`, `check-sysw-mr-vectors.sh`,
  `gen-sysw-mr-vectors.py`, `policy-generate.py`, `demo/sh2/build-payload.sh`.
- `me` → `me-preview`: `crates/me-cli/src/preview.rs:113` (`--version` must exit 0) and :206 (render
  non-zero → `PreviewError::Render { code }`); release workflow checks `me-preview --version` output
  (`.github/workflows/*.yml:410-412`).
- No GUI or other binary spawns `me` (mnemonic-gui pins do not include it; refugium-wallet: none found).

Not done / limits: the real `me-preview` was not built or run (stubs only); terminal-stdout refusal (2)
needs a pty and is source + test only; node-answer refusals in mt (§8.5, §6a) are source + test only
(no bitcoind); signal exits not measured.
