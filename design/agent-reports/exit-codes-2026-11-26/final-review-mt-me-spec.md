# Final independent review: `mt`, `me` / mnemonic-io-lib, and the exit-code spec

Reviewer: Fable (independent; wrote none of this code), 2026-11-26.
Normative: `design/SPEC_constellation_exit_codes.md` revision 5, read whole.

Reviewed:
- **mt**: `/home/user/fx/mnemonic-transaction`, `git diff ded423f^..HEAD` (HEAD `2b02417`).
- **me, mnemonic-io-lib**: `/home/user/fx/mnemonic-engrave`, commits 712ae21, 2a0747a, 8ff5711,
  72273ea, 82d2bb3, 236c576, cf88af4, f0a9654, 27234b9.
- Both copies of `SPEC_mt_v0_1.md`; all eight `impl-*.md` reports; both prior reviews.

**Verdict: 0 Critical, 2 Important, 11 Minor, 6 Nit.** The safety lenses are clean. Both
Importants are things nobody ran or re-read after the fold: a gate script that is red, and the
two `SPEC_mt` copies that no longer agree.

## What I ran

Binaries built from both HEADs (`CARGO_PROFILE_*_DEBUG=0`, `CARGO_INCREMENTAL=0`). No whole suite.

| probe | result |
| --- | --- |
| `cargo test -p mt-cli --test exit_codes` | 19 passed |
| `scripts/check-refusal-coverage.sh` (mt) | OK |
| root `--help` and `-h` of all eight binaries against the spec's R11 block | all end with the legend (`ms` and `mnemonic` re-wrapped by clap) |
| `mt decode` on a clean set / one-error set | 0 / 16, same 445 bytes of hex |
| `mt verify`, `inspect`, `decode --quiet`, `decode --json` on the one-error set | 16 each |
| `mt decode` one-error set, `--bitcoin-cli /nonexistent/x` | 48, hex printed |
| `mt decode --bitcoin-cli bitcoin-cli` (and `=bitcoin-cli`), none on PATH | 32 |
| default `bitcoin-cli` on PATH that exits 1, or prints garbage | 0, with the warning (as ruled) |
| **`mt verify --bitcoin-cli /nonexistent/x`** | **0** (see mt M2) |
| **named helper that exits 5 on every call**, `inspect` / `encode` | **0** (see mt M1) |
| `me` with an ms1 or `tx:` on argv: 10 shapes (bad flag, clap error, after `--`, as a flag's value, uppercase, `bundle`, `sysw pack`, `sysw show`) | 8 every time, token never echoed |
| `me --hex` / `me bundle` with ms1 on stdin | 2, nothing emitted |
| `me` gate scripts, `--cli` mode: admission-canonical, inspection-fail-open, output-integrity, checklist-partial-keys | PASS |
| **`scripts/fix-me-descriptor-input-strict-gate.sh --cli`** | **FAIL: 57 of 66 rows** (see I1) |
| `fix-me-cli-arg-contract-gate.sh --cli` | not runnable here (needs the seedhammer submodule) |

Not run by anyone, including me: mt's `scripts/mutate-refusals.sh` (93 rebuilds) and me's
arg-contract gate. Both were edited or affected by this work. CI must run them before merge.

## Critical

None.

## Important

### I1. (me) A gate script still asserts the old codes and is red: 57 of 66 rows

- **Where:** `scripts/fix-me-descriptor-input-strict-gate.sh:37-51`, which reads
  `design/evidence/fix-me-descriptor-input-strict/cases.txt` (column 3 is the wanted exit code:
  51 rows want 3, 5 want 2, 1 wants 4).
- **Measured:** `GATE_LOG_DIR=<scratch> bash scripts/fix-me-descriptor-input-strict-gate.sh --cli`
  gives `gate --cli: FAIL`. Examples: `i2-control.json[none]: exit 8, want 2` (`--as` omitted);
  `i2-escaped.json[descriptor]: exit 2, want 3`.
- **Why it was missed:** spec §7e says "three gate scripts", the implementer moved four, and the
  prior review checked only the edited ones. This one was never listed and never run.
- **Fix:** remap column 3 of `cases.txt` by `Row::class` (old 3 becomes 2, or 8 for `MixedNetwork`;
  old 2 for `--as` omitted becomes 8; old 4 becomes 2), then run the gate's `--cli` mode. Before
  merge, run the `--cli` mode of every `scripts/fix-me-*-gate.sh`; five of the six that have one
  take under a minute.

### I2. (mt, engrave) The two `SPEC_mt_v0_1.md` copies disagree on exit-code text, and the normative sentence that differs is wrong

- **Disagreement.** Engrave `27234b9` (16:53) is later than mt's re-sync `2b02417` (16:37). The
  engrave copy has two things mt's lacks:
  - (b1), engrave `:3408-3411`: the paragraph that a **default** `bitcoin-cli` found on `PATH`
    that fails leaves the exit status alone. mt's copy stops at `:3531`.
  - §10 item 10, engrave `:3463-3466`: "a check that came out no … is 1; … a bug is 101. The
    list is a summary". mt's copy (`:3596-3597`) ends at "a failed write adds 32."
- **The normative sentence is false.** Engrave `:3463-3464` says a NO is "`mt verify` against a
  node that proves an input spent". `mt verify` never asks a node: the same document's §1.1 says
  so, the code says so (`main.rs`, "§1.1: verify NEVER asks a node"), and I measured
  `mt verify --bitcoin-cli /nonexistent/x` at 0. The verb that refuses a spent input is
  `mt encode` (§8.5, and §6a for a contradicted value); `verify`'s NO is a `--transaction`
  mismatch. The error is copied from the exit-code spec's worked table (rev-6 item 1).
- **Fix:** in engrave, change the sentence to "(`mt verify --transaction` naming another
  transaction; `mt encode` when the node proves an input spent or contradicts a value)". Then
  re-sync mt's copy for both passages.

## Minor

### mt

- **M1. A named helper that exits 5 or 89 on any call counts as an answer, so it exits 0.**
  (Prior N2, not folded; the later ruling makes it a defect.)
  - Where: `crates/mt-cli/src/node.rs:176`.
  - Measured: a named helper that exits 5 on everything gives `inspect` rc 0 with "no node
    reachable". With `gettxout` exiting 5, `encode` prints "the node did not answer every
    question" and exits 0.
  - The ruling says a named helper that fails is 32. Core returns 5 only from
    `getrawtransaction` and 89 only from an old Core's `getindexinfo`.
  - Fix: pass the call name into `call()` and exempt 5 for `getrawtransaction` and 89 for
    `getindexinfo` only. Add two `exit_codes.rs` rows.
- **M2. `mt verify --bitcoin-cli PATH` is inert, and its help says a missing path adds 32.**
  - Where: `ReadArgs`, `main.rs:128-133`, shared by `decode`, `verify` and `inspect`.
  - Measured: 0 for a missing named path on `verify`.
  - Fix: either say on `verify` that the flag is ignored there, or refuse it as usage (64), as
    `--transaction` is refused on `decode`.
- **M3. Exit 32 for a named helper prints no line that names the helper.**
  - `mt decode --quiet --in set.txt --bitcoin-cli /nonexistent/x` prints only the generic "no
    bitcoind reachable" warning and exits 32. A typo and a node that is down look the same.
  - R5 asks for a message that agrees with the code.
  - Fix: one stderr line from `helper_failed()`, printed once, for example
    `mt: --bitcoin-cli <path>: could not be started: <os error>` (or "exited N", or "gave a
    reply mt could not read").
- **M4. `note_corrections` claims more than the oracle proves.**
  - `main.rs:1334-1338` says "the set's content id is the oracle … so the correction is verified".
    `content_id_guard`'s own doc (`main.rs:2403-2407`) says the id does not cover witness data,
    "most of the payload".
  - Funds are not at risk: the txid, and so the outputs, re-derive, and a miscorrected signature
    cannot broadcast.
  - Fix: make the comment say what is verified (the transaction's identity). See rev-6 item 7.
- **M5. Two rulings have no test row.**
  - A default `bitcoin-cli` that is found on `PATH` and fails (0 with the warning). Only the
    absent default is pinned (`the_default_bitcoin_cli_absent_is_offline`).
  - `--bitcoin-cli bitcoin-cli` counting as named. It is the only way to say "the node is
    required", so it deserves a row and a help sentence.
- **M6. Owed items are in reports, not in `design/FOLLOWUPS.md`.** (Prior M5 asked for this.)
  - Replacing `src/exit.rs` with mnemonic-io-lib 0.1.4 once published.
  - The offline-switch question.
  - The `SPEC_mt` re-sync (I2).
  - The unfolded nits N3, N4, N5.

### me and mnemonic-io-lib

- **M7. `SyswError::Wire(Iterations)` is 64 for a container that was read, not only for the flag.**
  - Where: `crates/me-cli/src/sysw/mod.rs:141`; raised by `wire::Header::parse` at `wire.rs:175`.
  - `sysw::open` and `sysw::region_image` are public API (the Refugium app pads through
    `region_image`). A caller that classifies a container whose header carries an out-of-range
    iteration count gets USAGE, "nothing ran", for a bad input.
  - The `me` binary is not affected: `sysw show` returns `EXIT_INVALID` directly, and `pack`
    range-checks the flag before reading.
  - Fix: keep 64 for the setting (`sysw/mod.rs:615`) and give the parsed-header case 2, by a
    separate variant or by mapping at the `parse` call sites.
- **M8. The ruling "tampered is 2" is not what the code can do.**
  - `SyswError::OpenFailed` and `seal::OpenError::Authentication` are 1 for any AEAD failure.
    AEAD cannot tell a wrong passphrase from altered ciphertext, and the code's doc says so.
  - As built: an authentication failure is 1; a missing passphrase or a malformed container is 2;
    a sealing-side failure is 101.
  - This is safe (nothing is released either way) and matches the toolkit's rule. The ruling's
    wording should be corrected (rev-6 item 12).
- **M9. The library's API is minimal, but six binaries already needed more.**
  - The six local copies differ: `Bits::of(u8)` (md, mk), `from_code` (mr), `add(code, bit)`
    (mk, mr), `with_io_failure(code)` (ms, toolkit), an absorbing `Exit` (mt), and `LEGEND`
    against `HELP_LEGEND`.
  - Each re-implements "add a bit, but 64 and 101 never combine" (R8). That rule is mechanism,
    not policy, and a mistake in it produces 96.
  - Suggestion, before the crates.io publish: add `exit::add(code: u8, bits: Bits) -> u8`
    (returns `code` unchanged when it is above 63) and `Bits::from_code(u8) -> Option<Bits>`.
    Neither hoists classification. Otherwise 0.1.4 lands and each consumer keeps a shim.
- **M10. The arg-contract gate has not been executed since its rows were edited** (A3 to A10).
  It needs the submodule. Run it in CI before merge and quote the result.

### Both

- **M11. R11's second sentence is not implemented.** "Per-verb help states only the bits that
  verb can set."
  - mt: verbs state only the `--bitcoin-cli` 32 line (prior N4).
  - me: no verb states any bit, except the `--expect` sentence.
  - Either add one line per verb, or drop the sentence from the spec (rev-6 item 19).

## Nit

- **N1 (mt).** The coverage gate still passes `assert_ne!(rc, Some(INVALID))` (any
  `Some(CLASS)` counts), and `refused` in `ASSERTING_HELPERS` matches any call of that name.
  No current row does either.
- **N2 (mt).** About 26 `decode`, `encode` and `inspect` test invocations set neither `PATH` nor
  `--bitcoin-cli` (`decode_verify.rs:63`, `encode.rs:43`, and others). They predate this work.
  On a machine with a `bitcoin-cli` they ask a real node about fixture txids. Making `mt()` set
  `PATH=/nonexistent` by default would close it.
- **N3 (mt).** `mt verify` prints `OK — …` and exits 16. Decision 5 intends this, but R5 says a
  message must agree with its code; rev-6 item 8 should say this pairing is allowed.
- **N4 (me).** `SyswError::PassphraseMismatch` now means "a sealed container and no passphrase
  given". The name says something else.
- **N5 (me).** FOLLOWUPS `me-nightly-clippy-*` are addressed by `cf88af4` and still open.
- **N6 (spec).** The status line still reads "Nothing is implemented yet".

## Coordinator rulings

- **mt, named `--bitcoin-cli` is 32; absent default is 0; default found but failing is 0 with a
  warning.** Safe. Every failure direction over-warns: a script using `&&` or the `rc & ~16`
  test never broadcasts on 32. Two notes:
  - There is now no offline switch when `bitcoin-cli` is on `PATH`. That is not a funds risk,
    but `bitcoin-cli` may be configured for a remote node, so `mt` sends the engraved
    transaction's txids there with no opt-out short of editing `PATH`. An `--offline` flag is
    worth a decision.
  - The opposite need is met but undocumented: `--bitcoin-cli bitcoin-cli` makes the node
    required (32 on failure). The help should say so.
- **me, wrong passphrase 1, tampered or missing 2, sealing-side failure 101.** Safe, with the
  correction in M8: tampered ciphertext is 1, not 2.

## Safety lenses, checked and clean

- **The `rc & ~16` one-liner** (`SPEC_mt` §1.1a, mt README:41).
  - It masks only bit 16, so 17, 18, 32, 48, 64, 101 and 130 do not broadcast. `mt` never
    returns 20.
  - `decode` writes hex only after `content_id_guard` and `decode_tx` pass
    (`main.rs:1806-1817`), in the plain, `--quiet` and `--json` forms.
  - The old `&&` one-liner now stops on a corrected set, which is the safe direction.
- **me bundle and sysw admission.** Unchanged. The `bundle.rs` and `lib.rs` diffs add only
  `exit_code()` and tests. ms1 and mt1 are still refused, at 2.
- **argv guard.** It runs before clap and before the UTF-8 refusal in every shape I tried, and
  never echoes the token.
- **io-lib seam.** Constants, `Bits` and the legend only. No error type, no classification.
- **Stale codes in docs and help.** None found in the READMEs, CHANGELOGs or `--help` of either
  tool. The remaining "exit 1/2/4" strings are historical comments.

## Leftover minors: prior findings not folded that should be

| prior finding | status | my view |
| --- | --- | --- |
| mt N2: exit 5/89 exempt on every call | not folded | fold; now M1 above |
| mt N4: per-verb help bits | not folded | fold or change R11; M11 |
| mt N5: discarded damaged duplicate exits 0 | not folded | needs a ruling; rev-6 item 9 |
| mt M5: file an owed FOLLOWUP | not filed in mt | fold; M6 |
| mt N3: closed stdin is 2 | not folded | leave; rev-6 item 20 |
| me N2: a non-UTF-8 token with a secret prefix loses the purge advice | not folded | leave; nothing is echoed |
| me N6: a sidecar render failure caused by the input is 32 | conforming | rev-6 item 21 |
| me N5: SPEC_mt item 10 lists 1 | folded | folded with a false sentence; I2 |
| me M5: FOLLOWUPS | filed in engrave | nothing filed on the mt side; M6 |

## Spec revision 6: defects the implementations found

For each: the question, the reading implemented, whether implementations agree, and a proposed
sentence. I did not edit the spec.

1. **§3 worked table names the wrong verb.** "`mt verify --bitcoin-cli …` on an input the node
   proves spent | 1". `mt verify` never asks a node.
   - Implemented: `mt encode` is 1 for a spent input or a contradicted value; `mt verify
     --transaction` mismatch is 1.
   - Proposed: "`mt encode` on an input the node proves spent | 1".

2. **R9: what "named" and "fails" mean.** (mt Q1, review I1 and I2, the ruling.)
   - Implemented in mt: named means given on the command line. Missing, unspawnable, non-zero,
     broken pipe or unusable reply is 32. The default absent is 0; the default found but
     failing is 0 with a warning.
   - me agrees: `ME_PREVIEW_BIN` set and missing is 32; an unnamed absent sidecar is 0.
   - Proposed: "A helper is named when the operator gave its path by flag or environment
     variable, even if the path equals the default. A named helper that is missing, cannot be
     started, exits with a failure, or gives a reply the tool cannot use sets 32. An unnamed
     optional helper that is absent or failing is a warning and sets nothing. A reply that is
     the helper's own answer (a null, a not-found) is not a failure."
   - Also correct §7d ("no node at all stays 0") and §8 to say that naming a missing path is
     not an offline switch.

3. **No offline switch for `mt` when `bitcoin-cli` is on `PATH`.** (mt fold, new question.)
   - Open. Proposed: a decision for Brian on `mt --offline`, recorded in §7d.

4. **R1: does 16 survive a later failure?** (mk Q11, mt Q2.)
   - mt keeps it: 17, 18, 34. mk keeps it: 18, 24. `repair` drops it when the set is
     uncorrectable or a complete set fails reassembly (2).
   - R1's text says an atomic failure reports only its failure bits, so implementations
     disagree with the letter.
   - Proposed: "Once a verb has used a corrected value, bit 16 stays set in whatever status the
     run ends with, including a refusal. The exception is `repair` when it prints nothing."

5. **Decision 6: a complete set that fails reassembly.** (mr Q7, md Q5, mk Q1.)
   - md and mk: 2, nothing printed. mr: 2, but the strings are printed. Disagreement.
   - Proposed: one sentence saying which, and that a chunked set of one takes the complete-set
     rule while an unchunked corrected string that is no card prints at 22.

6. **Decision 6 and 7: groups with different set ids.** (mk Q10, toolkit N5.)
   - Two mk1 chunks with different `chunk_set_id`s that together cover every index exit 0 (16
     if corrected) in `mk` and `mnemonic`. §7b's "ends today's 0 on strings `mk decode`
     rejects" does not hold there.
   - Proposed: "Each set id forms its own group. Groups that are each incomplete are not a
     failure, and the tool says that the strings belong to different sets."

7. **§7d: "the set-level content-id is the oracle".** (This review, mt M4.)
   - The id proves the transaction's identity, not its witness bytes. mt returns 16, never 20.
   - Proposed: "For `mt`, 16 means the reassembled transaction re-derives its id. Witness bytes
     are not covered; a miscorrection there yields a transaction that cannot be broadcast, not
     a different one."

8. **R5 against decision 5: `OK` printed with exit 16.**
   - mt and mk print their OK line and exit 16.
   - Proposed: "A verb whose only bit is 16 may print its success line, and must name the
     correction beside it."

9. **R6: what counts as used.** (mt Q3, Q4, review N5.)
   - mt: a transliterated HRP character is not a correction. A damaged duplicate that is
     discarded in favour of a clean copy is 0, although mt prints "re-cut it".
   - Proposed: rule the duplicate case. Either "a discarded damaged copy does not set 16", or
     set 16 whenever the tool tells the operator to re-engrave.

10. **Test 1 "a conflict" against input values that contradict each other.** (md Q1; mk Q6;
    mr Q3; toolkit round 2.)
    - md and mk: 2 for a fingerprint that disagrees with the xpub. mr: a fingerprint or binding
      mismatch is 1. The toolkit: a path mismatch is 1 and a fingerprint against an annotation
      is 2.
    - Disagreement.
    - Proposed: "An input that disagrees with itself, or two parts of one input that disagree,
      is 2. Bit 1 is for a comparison between the input and a separate reference the operator
      gave. A flag conflict judged without reading values is 64."

11. **The setting and input split has no list.** (mr Q2, md Q7, mk Q4 and Q5, ms Q4.)
    - Each tool made its own table. They agree on the principle.
    - Proposed: an appendix listing each tool's input flags, with the rule "`@env:` with a
      malformed name is 64, unset is 32, a non-UTF-8 value is 2".

12. **A wrong passphrase or password.** (me Q9 and M1, toolkit M5.)
    - me and the toolkit agree: a failure proven by an authentication tag is 1; a decrypt with
      no tag, a missing passphrase or a malformed container is 2; a failure while sealing with
      a key the tool derived is 101.
    - Proposed: that sentence, plus "the tool cannot tell a wrong passphrase from altered
      ciphertext, and its message says both".

13. **A flag the input makes necessary.** (me Q1: `--as` omitted.)
    - me: 8. It cannot be judged from the command line, and a flag resolves it.
    - Proposed: add to test 5, "or the tool declines to choose and a flag would choose".

14. **A missing argument against empty input.** (me Q4: `sysw hash` with no records.)
    - me: 64. Proposed: add the case to the §3 worked table.

15. **§7h contradicts decision 9, and the lock.** (mr Q1, Q4.)
    - A refused mainnet sentence is a typed stop, so mr exits 130; §7h says 8.
    - A held lock is 8 in mr (a guardrail); §7h lists "the lock" under 32.
    - Proposed: correct both lines in §7h, or rule the lock as 32.

16. **R14 covers `exit_code` but not `kind`.** (mk Q8, ms Q5, toolkit Q8.)
    - The three tools invented different names for one class: `IoError` (mk), `Io` (ms),
      `IoFailure` (toolkit). `Refused` and `Internal` agree.
    - Proposed: R14 names the `kind` string for each class.

17. **R11 and GUI detection.** (gui new Q, toolkit Q9 and N2.)
    - mnemonic-gui decides that a binary speaks the bit codes if its root `--help` ends with
      the legend, comparing with whitespace normalised.
    - All eight binaries satisfy this (measured).
    - Proposed: "The legend is the last text of the root `--help`, compared with whitespace
      normalised. Each binary has a test for it. Consumers may detect the scheme this way."
    - §7j and §9's "version floor on every run" should be replaced by this.

18. **R12: the colour of 16 alone.** (gui Q5.) The GUI shows green with "re-engrave the damaged
    plate". Proposed: say green or amber.

19. **R11: per-verb help.** Not implemented in mt or me. Proposed: keep it and list it as owed,
    or delete the sentence.

20. **R8 and closed standard streams.**
    - A closed stdout is detected on Linux, Android, macOS and the BSDs only (mt, me, ms).
      Windows still exits 0.
    - A closed stdin reads as empty input, 2 (mt, me).
    - Proposed: state both.

21. **R9 and §7e: a helper failure the input caused.** (me review N6.) A sidecar render failure
    is 32 even when the input is the cause. Proposed: one clause accepting that.

22. **R2 costs a content-dependent referral.** (me Q11.) `me --in wallet.txt` with no output
    mode is 64 before the tool can say "this is a descriptor". me added a static hint.
    Proposed: note in R2 that advice needing the input cannot precede a usage error.

23. **Decision 6 and `--out`.** (ms Q1.) At 22, ms prints the corrected string and does not
    write `--out`. Proposed: "A string that is no card is shown on the terminal and never
    written to `--out`."

24. **R5: `inspect` of a share that would not combine.** (ms Q2.) ms: 2. Proposed: add
    "or would not combine" to R5.

25. **Decision 4's order did not happen.** The library is unpublished and six binaries carry
    local copies with different APIs (me M9). Proposed: §9 step 1 records the interim state,
    and names the two helpers the library should gain before the publish.

26. **Decision 1 and N10 on `me`.** `MixedNetwork` sets 8 in `me`, and no N10 line exists yet,
    so a script cannot tell it from an argv refusal. Proposed: list it under the unbuilt
    network-rule units in §6.

27. **§7e's count.** "Three gate scripts" is five in practice (I1). Proposed: name them.
