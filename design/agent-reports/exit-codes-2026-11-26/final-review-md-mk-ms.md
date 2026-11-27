# Final independent review: constellation exit codes in md, mk and ms (2026-11-26)

Reviewer: Fable 5.2 (did not write or fold any of the code). No source was edited.

**Reviewed.** Against `SPEC_constellation_exit_codes.md` rev 5:
- md: `git diff d269ede^..HEAD` on `fx/descriptor-mnemonic` (head ded2455)
- mk: `git diff 7212cf6^..HEAD` on `fx/mnemonic-key` (head 3623baa)
- ms: `git diff 844888b^..HEAD` on `fx/mnemonic-secret` (head e49dbc7)

**What I ran.** I rebuilt `md`, `mk` and `ms` at those heads and probed them by hand, about 400 runs: every repair shape, empty, missing and non-UTF-8 input on every reading verb, stdout full, closed and EPIPE, network, argv secrets, `--out`, JSON modes. I ran no test suites. Every "measured" below is a run of the built binary; "from source" marks the few that are not.

**Verdict: NOT GREEN. 0 Critical, 3 Important, 9 Minor, 6 Nit.**

The three binaries agree on almost every situation in the table. The two code defects are both in `repair`, one in mk and one in ms. Each is a place where one repo's review fold fixed a problem and the same problem was left in a sibling.

---

## Important

### I-1 (mk). `repair` reports a duplicated, over-full or inconsistent chunk set as "incomplete": exit 0 or 16, with a false "verifiable later" message. md exits 2.

- **Where:** `crates/mk-cli/src/cmd/repair.rs:238-279` (`group_is_complete_and_consistent`) and `:347-357` (the `!group_is_complete_and_consistent` branch in `classify_mk1_set`).
- **Cause.** The function returns `false` for five different things: too few chunks, more members than `total_chunks`, a repeated index, an index out of range, and members that disagree on `total_chunks`. The caller treats every `false` as decision 7's incomplete set.
- **Measured** (V1 is a 2-chunk card, chunks A and B; A' is A with one character flipped):

| command | mk | `mk decode`, same input | md, same shape |
|---|---|---|---|
| `mk repair A A` | 0, both printed | 2 "duplicate chunk_index 0" | 2, nothing printed |
| `mk repair A B B` | 0, all printed | 2 "received 3 chunks, header declares 2" | 2, nothing printed |
| `mk repair A' B B` | **16**, printed, "the correction is verifiable when the rest of the set is reassembled" | 2 | 2 |
| `mk repair A' A` | **16**, same message | 2 | 2 |

- **Why it matters.**
  - In `A' B B` every chunk of the card is present, so the reassembly check (the cross-chunk hash, which the source calls "THE FUNDS FIX") could run. It is skipped, and the user is told a later reassembly will verify the correction.
  - A miscorrection that keeps its header would be caught as `SetReassemblyMismatch` (2) for `A' B`. With one plate entered twice it gets 16 and is printed. This follows from the code; I did not construct a 5-error alias.
  - This is the defect md's review found as I-1 and M-6, and md folded it (`short_set_of_distinct_chunks`). mk's review did not look for it.
- **Fix.** Split the predicate three ways: complete, short with distinct in-range indices and one `total_chunks`, and malformed. Only the short case takes the decision-7 branch. The malformed case exits 2 with nothing printed, as md does, with the reason on stderr. Add rows for `A A`, `A B B`, `A' B B` and a group whose members disagree on `total_chunks`.

### I-2 (ms). `repair` of a damaged share plate exits 22 and says the corrected share "is NOT an ms1 card"; `--out` is not written.

- **Where:** `crates/ms-cli/src/cmd/repair.rs:157-165`. Any decode error after a successful correction becomes `not_a_card`, including `IsShareNotSingleString`.
- **Measured** (a share from `ms split -k 2 -n 3`, one character flipped):
  ```
  $ ms repair --ms1 - < share_damaged
  ms12qzp5qm94an4cyj5qj9lnqpu02mtakk97qe9ehymckert7z       (the correct share)
  repair: correction UNVERIFIED ...
  repair: the corrected string is NOT an ms1 card this version decodes: this is one share of a K-of-N set ...
  exit 22        --json: "not_a_card":{"kind":"IsShareNotSingleString",...}
  ```
  The clean share gives exit 2 with nothing printed. `ms inspect` of the same clean share exits 0, and its source calls a share "a first-class read, NOT a malformed v0.1 single-string" (`cmd/inspect.rs:63-68`).
- **Why it matters.** A share plate is the one multi-plate artifact ms makes, and a damaged one is what `repair` is for. The corrected share is right, but bit 2 says "fix or replace the input", the text says it is not a card, and `--out` refuses to file it. An operator could discard a share that would combine. Before this diff the run was 2 with nothing printed, so it is not a regression in safety, but the new 22 path was never tried on a share and no test covers it.
- **Fix.** Treat `IsShareNotSingleString` after BCH as a valid string of the format:
  - corrected: 20 (no oracle), written to `--out`, with a message that names it a share;
  - clean: 0, echoed.
  
  Keep 22 for strings that are neither a card nor a share (unknown tag, reserved prefix). Add rows for both. If the coordinator prefers `repair` to stay single-string-only, the message must at least stop saying "NOT an ms1 card" for a share.

### I-3 (md, docs only). `CHANGELOG.md` has no entry for the cutover, and its Unreleased section states the old codes.

- **Where:** `CHANGELOG.md` `## md-cli [Unreleased]`. It is untouched by all eight commits (`git diff --stat d269ede^..HEAD -- CHANGELOG.md` is empty).
- **What is missing.** mk has a "BREAKING: exit codes follow the constellation exit-code spec" entry and ms a "Changed (breaking: exit codes)" entry. md has neither, and nothing for the closed-stdout fix (d269ede).
- **What is stale** in the same unreleased section:
  - `:25`: an `--out` write error "exits 1 (was ... exit 2)". It is 32.
  - `:31`: an output that names an input "exits 2". It is 8.
  - `:37`: a second `--key` for one slot, "two different values exit 1 ... the same value given twice exits 2". Both are 2, and the `MISMATCH:` prefix is gone.
- **Why Important.** The change is breaking for scripts (`rehearse.sh` expected 5) and for the GUI badge, and the changelog is where a consumer looks. It is one commit of prose.
- **Fix.** Add the BREAKING entry, modelled on mk's, and correct the three bullets.

---

## Minor

### M-1 (mk). `repair` exits 0 or 16 on a card every other verb calls invalid.

- **Where:** `cmd/repair.rs:360-374`. The `Ok(card)` arms never call `card.validate()`.
- **Measured** with `LEGACY_I8` from `tests/keycard_validation.rs` (a depth-0 key with a nonzero parent fingerprint): `decode`, `inspect` and `derive` print it and exit 2, `verify` and `address` exit 2, `repair` exits 0, and 16 when corrected.
- Spec §7b says decision 6 "ends today's 0 on strings `mk decode` rejects".
- **Fix.** Add INVALID through `warn_if_invalid(&card)` in both `Ok` arms. The strings are still printed, giving 2 or 18.

### M-2 (mk against md). A corrected lone string whose chunk header is malformed.

- **mk, measured:** N6 and N9 clean exit 2 **with the string printed on stdout**. N6 with one flip exits 18, printed.
- **md, from source** (`cmd/repair.rs:154-158` and the final `Err(e)` arm): a chunk-flagged string that fails `short_set_of_distinct_chunks` or reassembly exits 2 with nothing printed, corrected or not.
- Three things differ:
  - printed or not for a clean non-chunk (mk prints an invalid string at 2; md and ms print nothing for a clean non-card);
  - 18 against 2 for a corrected one;
  - mk's 18 has no bit 4, though nothing verified that correction. Decision 6 says a correction with no oracle is 22.
- **Fix.** One coordinator ruling for all three repos. My suggestion: a clean non-chunk is 2 with nothing printed, and a corrected one is printed at 22.

### M-3 (md, mk). `repair --json` says nothing about "not a card" or "incomplete".

- ms gained `verdict` and `not_a_card` in its fold (ms review M2). md and mk did not.
- **Measured:**
  - `mk repair --json` on a corrected N7 exits 22, and on a corrected N6 exits 18. Both print the ordinary success envelope.
  - `md repair --json` at 20 and at 16 for an incomplete set also prints the plain envelope. By the same code path, so does md's 22.
- A consumer reading stdout alone sees a repaired card (R5).
- md also prints no JSON error envelope on failure (stdout empty, text on stderr), while mk and ms print one on stdout.
- **Fix.** Give md and mk the same additive field ms has, plus an `incomplete` marker. Or record in FOLLOWUPS that the three `RepairJson` shapes now differ, since the "byte-matches `mnemonic repair --json`" comments in all three are no longer true for ms.

### M-4 (mk, ms). R11: per-verb help does not state the bits the verb can set.

- md's M-3 was folded for `md repair` only.
- `mk repair --help` says only "exit 16 = a correction was applied". It omits 22, 18, 2, 32 and 64.
- `mk decode`, `verify`, `inspect`, `address` and `derive` help say nothing about exit status. So decision 5's new behaviour (16 on a corrected plate, 17 or 18 on a later failure) is not in any mk help text.
- `ms inspect --help` does not say a string that would not decode exits 2.
- `ms repair --help` omits 8, 32 and 64.
- **Fix.** One "Exit status:" line per verb.

### M-5 (mk, ms). Unreleased changelog bullets still state old codes.

- **mk** `crates/mk-cli/CHANGELOG.md`, in the same `[Unreleased]` section as the BREAKING entry:
  - `:66-67`, `:73`, `:80-83`: "exit 64" for refusals that are now 2, or 1 for the keyed-policy contradictions;
  - `:68`: "invalid (exit 1)", now 2;
  - `:85`: an output that is an input is now 8.
- **ms** `CHANGELOG.md:47`: "`ms repair` README gives exit 4 (VERIFY-ME candidate), not 5". It is 20.
- **Fix.** Append "(N since the exit-code change above)", as ms already does at `:37` and `:39`.

### M-6 (mk, ms). No cross-repo FOLLOWUP for the manual lockstep.

- md filed `md-exit-codes-manual-lockstep` after its review's M-7.
- mk's `design/FOLLOWUPS.md` is untouched by the diff. ms's has no such entry.
- The manual lines for both exist only in `/home/user/fx/impl-mk.md` and `impl-ms.md`, which are not in any repo.
- **Fix.** File `mk-exit-codes-manual-lockstep` and `ms-exit-codes-manual-lockstep` with those line lists, unless the coordinator tracks them centrally.

### M-7 (mk). An xprv given as `--xpub` is echoed in full, and under `--json` it goes to stdout.

- **Where:** `crates/mk-cli/src/slip132.rs:129` (`invalid xpub {s:?}: {e}`). The text predates this diff; the diff only changed its class.
- **Measured:**
  - `mk encode --xpub xprv9s21...` prints `error: invalid xpub "xprv9s21...": unknown version` on stderr.
  - With `--json` the same text is in the envelope on stdout.
  - `mk verify --xpub xprv...` does the same.
- md prints only the version bytes for the same mistake. mk has no argv guard, so 2 is the right class (§3's table).
- **Fix.** Do not echo the value. Name the flag and the version bytes, as md does.

### M-8 (md, mk). The closed-stdout gap on Windows is documented only in ms.

- All three use the same Unix-only `.init_array` probe.
- ms states the gap in its CHANGELOG and `SPEC_ms_cli_v0_1.md:459` (its review's M4).
- md and mk make R8's claim with no platform caveat. On Windows a closed stdout still exits 0 with the output lost (from source: the `cfg` lists in `md-cli/src/out.rs` and `mk-cli/src/out.rs`).
- **Fix.** Add the same sentence to md's README and mk's CHANGELOG, and one FOLLOWUP covering the three.

### M-9 (md, ms). Tests that assert only "failed".

- A count of `.failure()` with no `.code(` within three lines:
  - md, 17: `cmd_compile.rs` 5, `cli_compose.rs` 5, `keyless_hashlock_reaches_a_descriptor.rs` 5, and two more;
  - ms, 4: `cli_combine.rs` 2, and two more;
  - mk, 0.
- These pass on any non-zero status, including 101 and 64, so they cannot catch a class regression.
- md's `cli-compiler` suites are also not built by CI (implementer's report).
- **Fix.** Pin the class in each.

---

## Nit

- **N-1 (md).** `every_variant_maps_to_its_class` (`src/error.rs`) has no guard that every variant has a row. Its comment now says a row "is added here by hand". mk and ms both parse their own source to enforce it. The class itself is forced by the exhaustive `exit_code()` and `codec_class()` matches, so this is test honesty only.
- **N-2 (mk).** `exit_code()` maps `Codec(_) | MdCodec(_)` to 2 as a blanket. For `MdCodec` that includes the three md-codec variants md itself classes as 101 (`NetworkRequiredForUnspendable`, `ChunkSetEmpty`, `Codex32EncodeError`). They are unreachable from `--from-md1` today, but this is the R13 blanket pattern, and a new `mk_codec::Error` variant lands at 2 with JSON kind `"Unknown"`.
- **N-3 (all).** The three local "mirrors of mnemonic-io-lib" already differ:
  - module `exit` (md, mk) against `exit_code` (ms);
  - `LEGEND` against `HELP_LEGEND`;
  - `Bits::NONE` against `Bits::empty()`;
  - `contains(u8)` against `contains(Bits)`;
  - `add()` against `with_io_failure()`;
  - 130 is `STOPPED` in mk, `OPERATOR_STOP` in ms and absent in md.
  
  The JSON `kind` words differ too: mk has `InvalidInput`, `IoError`, `UsageError`; ms has `BadInput`, `Io`, `Usage`. Settle the names when the shared crate is published; the GUI will key on the kinds.
- **N-4 (ms).** Empty-input messages do not say "empty". `ms decode` prints "string length 0 not in v0.1 set [50, ...]" and `ms repair` prints `wrong HRP: got ""`. md and mk both name the empty input.
- **N-5 (md).** ms's legend is hard-wrapped; md and mk print one line. It is the same text after whitespace folding.
- **N-6 (spec).** §7a lists `--allow-invalid-miniscript` among md's overrides, but md has no such flag (md review N-7). It still needs the spec owner.

---

## Coordinator rulings: is any unsafe?

1. **A chunked set of one follows the complete-set rule; a non-chunked corrected string that fails is printed at 22.** Safe.
   - Measured: md's chunked-of-one gives 0 clean and 16 corrected, verified by the content id.
   - One exception in md (from source, `cmd/repair.rs:186`): a corrected single string at an unsupported wire version is printed at 22 whatever its chunk flag ("ruling 7"). So a chunked-of-one at a newer version is 22, not 2. That errs toward warning. Worth one line in the spec.
2. **16 stays set on a later failure.** Safe. It can only add a warning. Measured in mk: 17, 18 and 24, and the JSON `exit_code` carries the same sum.
3. **No input named is 64; an empty named input is 2.** Safe. ms differs in what counts as "named": see the table.
4. **An invalid card in decode, inspect, address and derive is 2.** Safe as a code. Note that `mk address --address-type p2wpkh` on such a card still prints addresses at exit 2 (measured). The ruling covers that, but a script that ignores the status gets addresses from a card `verify` rejects. `mk repair` does not follow the ruling (M-1).

---

## Consistency table (measured unless marked)

| situation | md | mk | ms | agree? |
|---|---|---|---|---|
| repair, clean complete card | 0 | 0 | 0 | yes |
| repair, corrected single string (not chunked) that decodes | 20 | cannot occur | 20 | yes |
| repair, corrected chunk set that reassembles | 16 | 16 | n/a | yes |
| repair, lone chunk or short set, clean / corrected | 0 / 16 | 0 / 16 | n/a | yes |
| repair, share plate, clean / corrected | n/a | n/a | 2 silent / **22 "not a card"** | **I-2** |
| repair, duplicated or over-full set, clean / corrected | 2 / 2 | **0 / 16** | n/a | **NO (I-1)** |
| repair, chunks of two different cards | 2 (`ChunkSetInconsistent`) | 0 (two groups; N8 also 0) | n/a | NO: md repairs one card per call, mk batches |
| repair, uncorrectable | 2 silent | 2 silent | 2 silent | yes |
| repair, clean single string that is no card | 2 silent | 2 silent (N7) | 2 silent | yes |
| repair, corrected single string that is no card | 22 printed | 22 printed (N7) | 22 printed, `--out` not written | yes |
| repair, string with a malformed chunk header, clean / corrected | 2 silent / 2 silent (source) | **2 printed / 18 printed** | n/a | **NO (M-2)** |
| repair, complete set that does not decode | 2 silent | 2 silent (N12) | n/a | yes |
| repair, card that fails key validation | n/a | **0 / 16** (other verbs 2) | n/a | **M-1** |
| repair `--json` at 22, 18 or incomplete | plain envelope | plain envelope | `not_a_card` field | NO (M-3) |
| empty stdin through `-`; empty `--in` file | 2 | 2 | 2 | yes |
| no input named, stdin at `/dev/null` | 64 (clap) | 64 | **2**: an omitted positional means stdin (`repair` is 64) | by design; say so in the spec or manual |
| empty-string argument `''` | 2 | 2 | n/a | yes |
| non-UTF-8 stdin; non-UTF-8 `--in` file | 2 / 2 | 2 / 2 | 2 / 2 | yes |
| non-UTF-8 argv | 64 | 64 | 64 (own guard, value not shown) | yes |
| missing `--in`; `--in` is a directory; stdin is a directory | 32 | 32 | 32 | yes |
| missing `--keys`, `--from-md1-set`, `--from-mk1-file` | 32 | 32 | n/a | yes |
| unknown flag, unknown verb, no verb | 64 | 64 | 64 | yes |
| `--help`, `--version` | 0 | 0 | 0 | yes |
| usage check against a missing input file (R2) | 64 first | 64 first | 64 first | yes |
| network mismatch | 8 (`--key` tpub on mainnet, and the reverse) | 8 (`address --network`) | no network input | yes |
| N10 machine line under `MNEMONIC_NETWORK_JSON=1` | not printed | not printed | n/a | network-rule units not built (spec §9 step 2) |
| xprv given as a key | 2, version bytes only | 2, **value echoed** (M-7) | n/a | code yes, text no |
| a secret (ms1, phrase) on argv | 2, no guard, wrong HRP | 2, no guard | 8 before clap; 8 with a bad flag too | as spec R2: only ms has the guard |
| stdout full, or closed with `>&-` | +32 (32, 36, 48, 52) | +32 (32, 48) | +32 (32, 52) | yes |
| stdout failure on a failing verb that printed nothing | 2 kept | 2 kept; JSON error envelope 34 | 2 kept; JSON error envelope 34 | yes |
| EPIPE | verb's code | verb's code | verb's code | yes |
| `--help` to a full stdout; a usage error with stdout closed | 32; 64 | 32; 64 | 32; 64 | yes |
| `--out` is an input | 8 | 8 | **no refusal**: `encode --in F --out F` overwrites F, 0; `repair --in F --out F` 20 | NO, but not from this diff |
| `--out` is stdout while the verb prints | 8 | 8 | 8 | yes |
| `--out` exists | overwrites, 0 | overwrites, 0 | overwrites, 0; `split` and `hashlock --random` refuse, 8 | yes |
| `--out` unwritable; `gen-man --out` unwritable | 32 | 32 | 32 | yes |
| inspect, string that fails checksum or parse | 2, nothing | 2, nothing | 2, nothing | yes |
| inspect, parses but would not decode | 2, nothing | 2, nothing | 2 with the full report (decision 8) | yes, per decision 8 |
| decode or inspect, one-character damage | 2 (no auto-correct) | 16, printed | 2 (no auto-correct) | codes yes; only mk corrects in read verbs |
| decode, lone chunk or share | 2 | 2 | 2 | yes |
| verify mismatch; on a corrected card | 1 | 1; 17 | 1 | yes |
| bad setting value (`--count abc`, `-k abc`) | 64 | 64 | 64 | yes |
| bad input value (fingerprint `zz`) | 2 | 2 | n/a | yes |

---

## Leftover minors from the prior reviews

Not folded, or only partly, and not explicitly deferred.

**md** (review of 2026-11-26; the fold commit b79bfa5 claims N-1 to N-6):
- **N-3, not folded.** The table test still cannot detect a variant with no row; only a comment was added (N-1 above).
- **N-4, half folded.** The `md repair` row in the README table is right. The paragraph at `crates/md-cli/README.md:171-178` ("**v0.6.0 limitation — chunked-form only** ... non-chunked single-string md1 ... is rejected with a wire-format error") is still there and contradicts the 20 row beside it.
- **N-7.** The spec defect (`--allow-invalid-miniscript`) is not recorded anywhere I can find (N-6 above).
- **M-5.** Folded in e178792. The commit says each step-1 row set was run; I did not re-run the gates, because their step 2 is the whole suite.
- I-1, I-2, M-1, M-2, M-3, M-6, M-7, N-1, N-2, N-5 and N-6 are folded, and I re-measured each.

**mk:**
- **M3 (N8: two chunks with different set ids exit 0).** Left as spec question 10 and still open. It is the same weakness as I-1 above, grouping decides the verdict, and it should be ruled with it.
- **M6.** Ruled (16 stays set).
- I1, I2, M1, M2, M4, M5, N1, N2 and N4 are folded and re-measured.
- New here: M-1, M-4, M-5, M-6 and M-7 above were not raised before.

**ms:**
- **M4 (Windows closed stdout).** Documented in the CHANGELOG and SPEC, not fixed, and there is no FOLLOWUP entry. md and mk have neither (M-8 above).
- **N1, part.**
  - `argv_guard.rs`, `ms-codec/README.md` and `SPEC_ms_derive.md` are fixed.
  - The open FOLLOWUPS entries at `design/FOLLOWUPS.md:411, :438, :456, :932, :1014, :1035` still say "exit 1" or "exit 3" for behaviour that is now 2.
  - `CHANGELOG.md:47` is M-5 above.
- I1, M1, M2, M3, N2, N4 and N5 are folded and re-measured, including the `@env:` order, `--account` range, `not_a_card`, and the unknown language byte at 2.

---

## Notes for the coordinator

- **Shared scratchpad.** The session scratchpad is shared with another agent. My first probes wrote `env.sh`, `lib.sh` and small fixture files at its top level before I noticed. The other agent's `env.sh` later replaced mine and is intact now. I then moved to `scratchpad/fr3/`.
- **Stray file.** One probe run with an unset variable created `/err.txt` (65 bytes, in the filesystem root). The safety check refused my `rm`, so it is still there and should be deleted by a person.
