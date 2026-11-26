# Independent review: `me` / mnemonic-io-lib exit-code implementation

Reviewer: independent adversarial review (did not write the code), 2026-11-26.
Worktree `/home/user/fx/mnemonic-engrave`, branch `fx/mnemonic-engrave`, HEAD `cf88af4`.
Commits reviewed: 712ae21 (closed stdout), 2a0747a (non-UTF-8 argv), 8ff5711 (io-lib exit
constants), 236c576 (me renumber), cf88af4 (clippy/fmt), 72273ea and 82d2bb3 (SPEC_mt, uniformity
spec). Normative: `design/SPEC_constellation_exit_codes.md` rev 5.

Ran (debug=0, incremental=0, -j 4): `cargo build -p mnemonic-engrave`; `cargo test -p
mnemonic-engrave --test closed_stdio --test non_utf8_argv --test exit_digits --test
argv_secret_guard` (19 pass); `cargo test -p mnemonic-io-lib --lib exit` (4 pass) and its doc test
(pass). I also ran the binary by hand on the cases below. I did not run the whole suite.

## Verdict

**0 Critical, 1 Important, 6 Minor, 8 Nit.** The scheme is implemented faithfully. The numbers come
from the library. The legend matches the spec byte for byte: I extracted the spec's R11 block and
compared it with the last line of `me --help`. The argv guard still runs before clap and before
the UTF-8 refusal, and neither echoes the token. Usage is always 64 and never combined. The one
real defect is a setting that is still judged after the input is read (I1).

## Important

### I1. `sysw pack --passphrase-words N` out of range is judged after the input is read, after the `sealing:` line, and its exit depends on the input (R2, F-246)

- **Where:** `crates/me-cli/src/main.rs:2291-2306`, where `sysw::passphrase::generate(n)` returns
  `GenError::WordCount`, which maps to `EXIT_USAGE`. Also `decide_sealing`, `main.rs:3081-3092`,
  which ignores the flag on an unsealed payload.
- **What happens (measured):**
  - `me sysw pack --in s.txt --passphrase-words 1 --out p.bin`, where s.txt holds an ms1, prints
    the `sealing:  SEALED — … opens only with the passphrase you chose with --passphrase-words`
    paragraph, then `me: WordCount(1)` (a Debug format), and exits **64**.
  - With a public-only payload, the same flag value prints "--passphrase-words is IGNORED here" and
    exits **0**.
  - So one bad setting gives 0 or 64 depending on the input's content, after the input was read.
- **Spec conflict:**
  - Decision 2 and test 1 say a setting is judged "without reading any input", and the flag's
    own help states the range (2..=24).
  - R2 says a usage error "exits 64 before any input is read or any output is written".
  - The commit message and impl-me.md both say "every usage check now runs before input is read
    (R2)". That is true for `--iterations`, `--expect` and `--as`, but not for this flag.
  - It also breaks the F-246 rule the same function cites: nothing that describes a container
    prints before every gate that can abort.
- **Fix:**
  - Range-check `passphrase_words` beside the `--iterations` check at `main.rs:1999-2006`, before
    `read_records`, with a Display message (`--passphrase-words 1 is outside 2..=24`), exit 64.
  - Or give clap a `value_parser` with that range (clap errors are already 64).
  - Add a vocabulary row in `tests/cli.rs` with a public-only stdin, so the "0 when unsealed" path
    is pinned too.

## Minor

### M1. `SyswError::Crypto` / `PassphraseMismatch` and `SealError::Passphrase` classify as 101 context-free on public library types; for an *open* with a user passphrase that is wrong (spec question 9)

- **Where:** `crates/me-cli/src/sysw/mod.rs:157`, `crates/me-cli/src/seal/mod.rs:112`.
- **Answer to the question "a wrong passphrase is user input; is 101 right?":** for the `me`
  binary as shipped, yes, but only by reachability.
  - No `me` verb opens a container. `sysw::open` has no caller in `main.rs`; it is called only
    from tests.
  - `SyswError::Crypto` is reachable from the CLI only through `seal_with` (`sysw/mod.rs:807`), an
    AEAD failure while sealing with a key `me` derived itself. That is a bug, so 101.
  - `PassphraseMismatch` is raised only by `open(…, None)` on a sealed container
    (`sysw/mod.rs:876`). Its doc ("a sealing/passphrase mismatch" in `pack`) describes a path that
    does not exist.
  - `SealError::Passphrase` is reachable from the CLI only with the generated passphrase.
- **Failure scenario:**
  - `exit_code()` is a `pub` method on types exported by the `mnemonic_engrave` library, and
    `sysw::open(blob, Some(user_pass))` is public API.
  - A caller that classifies an open failure gets 101 ("report it: a bug") for a mistyped
    passphrase or a tampered container.
  - By §3, a wrong passphrase on a well-formed container is test 4 (a comparison against "a
    phrase" came out no, so 1). A tampered ciphertext or a missing passphrase is test 6 (2).
  - The same applies to the v2 seal path (`seal/mod.rs:599-603`), where `SealError::Passphrase`
    is raised for operator-typed words while the sibling `SealError::Password` is 2.
- **Fix:** give the open side its own variants and class them by §3. For example:
  - `OpenFailed`: 1, or 2 if the reviewers prefer, since AEAD cannot tell a wrong phrase from
    tampering;
  - `PassphraseRequired`: 2.
  - Keep the seal-side `Crypto` at 101.
  - At minimum, correct the doc comments and state that `exit_code()` classifies the `pack` and
    `seal` paths only.

### M2. Uniformity spec line 35 now says `mt` refused at exit 2 "when measured"; the exit-code spec records that it measured 1

- **Where:** `design/SPEC_constellation_cli_uniformity.md:35-36` (72273ea).
- The new text reads "the exact disposal `me` and `mt` refuse (F-244, F-252): at exit 2 when
  measured, and at exit 8".
- `SPEC_constellation_exit_codes.md` §6, under "Uniformity spec §6f", says: *"Its line 35 says
  `mt` refuses a world-readable stdout at 2; it measures 1."* The mt-me survey (row 6) measured
  `mt encode … > wr.txt` at 1.
- The edit keeps the false half of the sentence the exit spec flagged.
- **Fix:** "at exit 2 (`me`) and 1 (`mt`) when measured, and at exit 8 (REFUSED) since …".

### M3. The CHANGELOG's was→now table misstates several old codes, which a migrating script relies on

- **Where:** `crates/me-cli/CHANGELOG.md`, Unreleased "Changed" table (236c576). Checked against
  `236c576^:crates/me-cli/src/main.rs`:
  - "4 (`--as` omitted, mixed network, …) → 8":
    - `--as` omitted (`Outcome::AsDecides`) was **2** (`EXIT_USAGE`);
    - `MixedNetwork`, like every descriptor `Row` from the gate or `--as`, was **3**
      (`EXIT_REFUSED`).
  - "4 (record, descriptor or container refusal) → 2": descriptor rows were **3**. Only
    `MultiRecord` was 4.
  - "2 (argument not UTF-8) → 64": the released behaviour was a **101** panic. The 2 existed only
    in unreleased 2a0747a.
  - Missing rows:
    - `me seal` without `--seal-secret` on seed material, 3 → 8;
    - a sidecar render failure, 4 → 32 (only "1 / 2" is listed);
    - `--expect` value, `--iterations` and `--pw-form` are listed, but `me seal` has no
      `--pw-form` flag (`SealError::PwForm` is unreachable from the CLI). That cell is harmless but
      names a flag that does not exist.
- **Fix:** correct the "was" column. The old codes are what a script greps for.

### M4. The R13 "table tests" sample variants rather than tabulating each one

- **Where:**
  - `seal/mod.rs:747-763`: `Record`, `RecordAt`, `Container`, `Titles` and `Crypto` are absent.
  - `sysw/mod.rs:903-948`: absent are `Wire::{TooShort, UnknownVersion, UnknownKdf, UnknownAead}`,
    `Unclassifiable::{NotATransaction, Bip93OutsideTheProfile, PreimagePlate, Composer, Mr}` and
    `MrSet`.
  - `lib.rs:105-113`: one of four `ConvertError` arms.
  - `bundle.rs:709-729`: `Classify`, `Validate`, `Md1Undecodable`, `Md1HeaderRead` and
    `SetIncomplete*` are absent.
  - `descriptor/refusal.rs:1214-1234`: `every_row_maps_to_its_exit_class` restates the
    implementation's own `AsOmitted | MixedNetwork => REFUSED, _ => INVALID`, so a row moved in
    both places is not caught. It is still useful as a per-row walk of `Row::ALL`.
- **Why it matters:** R13 says "each error variant, and each refusal row, maps to its §3 class".
  - The exhaustive `match` guarantees that every variant is classified.
  - Only a table guarantees that it is classified *as intended*.
  - Example: moving `Titles` or `Unclassifiable::Mr` to 101 or 8 compiles and passes every test.
- **Fix:** list every variant with its expected code, using a literal expected value per row
  rather than a mirrored predicate.

### M5. Owed cross-repo items are not filed in FOLLOWUPS

- impl-me.md lists them, but `design/FOLLOWUPS.md` is unchanged by these commits:
  - the fork's `seamVectorsSHA256` re-pin and re-vendored `nonstandard/testdata/`;
    `scripts/fix-sec-canonical-keeps-xpub-depth-gate.sh:129` now requires `ec4b6e21…` in the
    fork's test and fails until then;
  - the mnemonic-io-lib 0.1.4 crates.io publish (§9 step 1);
  - `demo/sh2/rehearse.sh:60`, which now pins `md repair` at 20 and fails against today's md (5)
    until md's renumber lands;
  - `gen.py` not re-run. I checked the hand remap: rows.py's `G(...)` values and the JSON's
    `exit_code` cells agree, with no 3/4 left in either.
- **Fix:** file them as `owed` items, with a Companion line where the other repo is named, per
  the repo's cross-repo convention.

### M6. Two new tests under-assert

- `tests/closed_stdio.rs` `closed_stdout_out_dev_stdout_fails` asserts only `rc != 0` and no
  panic. Measured: 32 with `me: cannot write the output: /dev/stdout: Bad file descriptor (os
  error 9)`. A regression to, say, 8 or 101 would pass. Pin 32 and the message.
- `tests/non_utf8_argv.rs` `argv_secret_guard_still_runs_before_the_utf8_refusal` asserts only
  code 8. Add the guard's own wording ("Refused BEFORE the command line was parsed") and assert
  that stderr carries neither the ms1 nor the raw bytes, as `is_guard_refusal` does in
  `argv_secret_guard.rs`.

## Nit

- **N1. Output mode before the F-421 referral (the behaviour change asked about): not a regression
  worth reverting.**
  - R2 requires that `me --in wallet.txt` with no mode exits 64 without reading the file, so the
    content-dependent referral cannot come first.
  - The cost is one extra round trip for an operator holding a wallet export.
  - Cheap mitigation that reads nothing: add a static line to the "choose an output mode" message
    (`main.rs:1001-1002`), e.g. "a wallet export or descriptor is packed with `me sysw pack --as
    <descriptor|md1>`".
- **N2. A non-UTF-8 token is refused at 64 even when it carries a secret or `tx:` prefix plus a
  stray byte.**
  - Examples: `me --hex "$MS1"$'\xff'` and `me sysw pack "tx:01"$'\xff'`, both measured at 64.
  - The lossy copy no longer classifies as secret, and the post-parse `argv_records_guard` never
    runs.
  - Nothing is echoed (good), but the operator loses the purge advice.
  - Optional: before the UTF-8 refusal, run the `tx:` prefix and secret-prefix test on the lossy
    copy, or add "if it was key material, purge your history" to the UTF-8 message.
- **N3. Stale comments and wording:**
  - `main.rs:2224`: "it exits 3 rather than 2". The argv bearer refusal and the terminal refusal
    are now both 8; only the wording tells them apart.
  - `crates/mnemonic-io-lib/src/records.rs:46-51`: "a shared IO library may publish no exit
    integer at all … `me` maps this refusal onto its own usage code". Both halves are now false;
    `me` maps it to INVALID (2).
  - `sysw pack --expect` help (§6g paragraph): "turns that into a refusal". It is now NO (1),
    "a check said no" (R5: the message agrees with the code).
- **N4. Fail-open on an empty collection.**
  - `main.rs` `Decision::Refused(rs)` and the `--expect` arm OR `class()` over a Vec into
    `Bits::OK`. An empty Vec would exit 0.
  - Both producers guard non-emptiness today (`as_flag.rs:113`, `unmet.is_empty()`).
  - A `debug_assert!(!bits.is_ok())`, or falling back to `EXIT_INTERNAL`, would keep a future
    producer from turning a refusal into success.
- **N5. Two gaps in the SPEC_mt §10 item 10 summary:**
  - It lists 64/2/8/32/16 but not 1 (`mt verify` against a node that proves an input spent, §3's
    worked table) or 101. A reader may take the list as complete.
  - (b1) covers "absent from PATH" and "named", but not a default `bitcoin-cli` found on PATH that
    then fails (node down). By R9 that is an unnamed optional helper, so a warning and 0. One
    clause would settle it.
- **N6. A sidecar render failure is 32 even when the input caused it.** The old comment cited "a
  string that fits no plate". §7e explicitly says 32, so it conforms; noted only because the
  operator guidance ("fix the environment") would be wrong for that cause. The bundle codecs bound
  string length, so this is likely unreachable.
- **N7. A write failure in R8 fail-fast mode exits 32 alone.**
  - `stdout_out::fail` exits at the first failed write. `sysw show` prints header lines before it
    can decide truncation (2), so a closed stdout gives 32, not 34.
  - That conforms to R1's fail-fast clause, and no verb writes stdout after its other bits are
    known. No change needed.
- **N8. `scripts/policy-differential.py` (named in §7e) needs no change.** It runs `md` and tests
  only `rc == 0` / `rc != 0`, so leaving it untouched is correct.

## Checked and found correct

- **R11 legend.**
  - `HELP_LEGEND` equals the spec's R11 block exactly; I compared the extracted spec text with the
    last line of `me --help` programmatically.
  - The legend appears on `me --help`, `me -h` and `me help`, and not on subcommand help.
- **io-lib API.**
  - The constants are `u8`: 0, 1, 2, 4, 8, 16, 32, 64, 101, 130.
  - `Bits` is a private-field newtype whose associated constants are the six bits only, so
    `to_exit_code` can never produce or combine 64, 101 or 130.
  - It provides `|`, `|=`, `contains` and `is_ok`.
  - No error type or classification is in the library; the doc states classification is the
    caller's. This respects the CLAUDE.md seam: mechanism only, no hoisted policy.
  - The legend is shared text, as the remedy text already is.
- **argv guard ordering.**
  - The pre-parse guard runs on a lossy copy of `args_os()`, before the UTF-8 refusal and before
    clap.
  - `me <ms1> $'\xff\xfe'` gives 8 with the guard text. `me --nosuchflag <ms1>` gives 8.
  - `sysw pack`'s `argv_records_guard` was extracted and now runs before the `--as` and `--expect`
    checks, the `--iterations` check and `read_records`. `read_records` still calls it for every
    caller.
  - The UTF-8 refusal never echoes the token; the tests check both the marker and the raw bytes.
- **Closed stdout (712ae21).**
  - The `.init_array` / `__mod_init_func` probe runs before std's `sanitize_standard_fds`.
  - It detects EBADF precisely (`F_GETFD == -1 && errno == EBADF`).
  - It plugs the fds in 0..2 order, so each plug lands on its own number, with fd 1 read-only.
  - It lives in the binary (`main.rs` `mod stdout_out`), not the library, so library users are
    unaffected.
  - Measured: `--hex >&-`, `--out /dev/stdout >&-` and `--out /dev/fd/1 >&-` all give 32 with
    EBADF.
  - `--out file >&-` gives 0 with the right bytes. `2>&-` gives 0. `<&-` gives 2 (empty input).
- **Spec questions 1-11 (impl-me.md).** I agree with all eleven:
  - (1) `--as` omitted is 8: not judgeable from argv, and a flag waives it.
  - (2) MixedNetwork is 8 by test 3.
  - (3) The representability rows are 2. `PromotionTestnetKey` is 2, which matches
    SPEC_network_rule.md:363: "not a network contradiction".
  - (4) `hash` with no records is 64 (a missing argument).
  - (5) `UnreadableVersion` is 2. §3 lists "a newer format" under INVALID, and it sums to 3 with an
    unmet expectation.
  - (6) ms1 to the converter or `bundle` is 2, per the worked table; ms1 on argv is 8.
  - (7) A read error is 2 for `InvalidData` and 32 otherwise.
  - (8) Prompt EOF is 2. rpassword 7.5.4 returns `UnexpectedEof` on Ctrl-D with nothing typed, and
    sends itself SIGINT on Ctrl-C, so the shell sees 130.
  - (9) 101 is right for the reachable paths; see M1 for the library caveat.
  - (10) An unnamed absent sidecar is 0.
  - (11) See N1.
- **Other classes.**
  - Unsigned tx, preimage carrier, digest-shaped phrase and `--seal-secret` are 8.
  - Destination refusals and a plate-artifact directory are 8.
  - Manifest serialisation is 101.
  - `sysw show` on an empty file is 2. A missing `--in` file is 32. clap's non-zero errors are 64.
- **Gate scripts and seam vectors.** The edited rows are consistent. The JSON's sha256 equals the
  pin in `tests/descriptor_seam.rs:46` and in the gate script. No script I could find still
  compares a current `me` with 3 or 4, except the env-gated RED replay in
  `check-sysw-mr-vectors.sh:92`, which is deliberate.
- **SPEC_mt edits.**
  - The §1.1a one-liner `[ $(( rc & ~16 )) -eq 0 ]` is correct and safe: it masks only the
    REPAIRED bit, so 20 (corrected and unverified), 64, 101 and 130 do not broadcast.
  - (b1) matches the R9 ruling, and §10 item 10 points at §3.
  - The uniformity §6f "superseded" note is accurate for `me` (but see M2).
- **R14.** `me` emits no JSON `exit_code` field, so R14 does not apply.
