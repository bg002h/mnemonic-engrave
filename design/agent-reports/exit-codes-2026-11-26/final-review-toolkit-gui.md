# Final review: constellation exit codes in mnemonic-toolkit and mnemonic-gui (2026-11-26)

Independent review (Fable), after the folded Opus rounds. Normative text:
`SPEC_constellation_exit_codes.md` rev 5.

**Reviewed.**
- mnemonic-toolkit, branch `fx/mnemonic-toolkit` at `a0ac6ad0`: `e51607b8`,
  `71dc68ac`, `f8b8d19c`, `a32c3bd2`, `2a3ce852` (classification and conflict
  resolution only), `5931f7a7`, `1cfa8950`, `a0ac6ad0`.
- mnemonic-gui, branch `fx/mnemonic-gui` at `3419465`: `a5fa42d`, `691f9ce`,
  `2545d0c`, `2168783`.

**Method.** I rebuilt `mnemonic` at HEAD and ran it side by side with the
sibling builds in `/home/user/fx/*/target/debug` (`md`, `mk`, `ms`, `mt`,
`me`, `mr`, `mr-sitting`), on one bip84 bundle of the `abandon … about`
phrase, clean and with one substitution. The GUI was reviewed by reading the
source and tests. I ran no test suite and did not build the GUI. Every
"measured" below is a run of those binaries.

**Verdict: NOT GREEN. 0 Critical, 4 Important, 14 Minor, 6 Nit.**
No case was found where a code or badge says "fine" on a wrong wallet with
current binaries. The Important items are three places where the same input
gets different codes, or the manual gives codes the binary does not, and one
GUI detection path that fails in the reassuring direction.

---

## Critical

None.

## Important

### I1. toolkit manual: `45-mr.md` says it follows E105, but about 25 passages still give the old codes

- **Where:** `docs/manual/src/40-cli-reference/45-mr.md`. The table at
  `:53-80` is right. The note that the passages below were not yet renumbered
  was removed in `a0ac6ad0`, and the passages were not changed.
- **Measured against `mr` `3e2a50c5e`:**

  | line | manual says | binary |
  |---|---|---|
  | `:5553` argument parser | usage, exit 2 | 64 |
  | `:5372`, `:5441` `mr root` / `mr child` | "2 usage" | 64 (`--letters A`, a stray argument) |
  | `:5404` an A in `--letters` | parse error (exit 2) | 64 |
  | `:7554` a value after `--root` | usage, exit 2 | 8 |
  | `:7562` a passphrase on the command line | usage, exit 2 | 8 |
  | `:7005`, `:7566` `--root-first-card` required | usage, exit 2 | 8 |
  | `:7558` `--root-label-set-id` without `--root -` | usage, exit 2 | 64 |
  | `:7088` `--grown` not `none`, `:7054`/`:7075` `--root-letters` | usage, exit 2 | 2, but an invalid input, not usage |
  | `:7840` ST-RR-01, file cannot be read | exit 1 | 32 |
  | `:7844` ST-RR-02, not a hand-back | exit 1 | 2 |

  Not measured, same pattern: `:5524`, `:5557`, `:5559`, `:5612`, `:5621`,
  `:7198`, `:7426`, `:7439`, `:7570`, `:7611`, `:7848` (ST-RR-03, another
  network: 8 by the mr report), and `:4419` (a key of the other network,
  "exit 1"; the table's own row 8 says a network mismatch is 8).
- **The chapter contradicts itself:** `:3848`-`:3850` give 8 for
  `--root-first-card` and for a value after `--root`; `:7005`, `:7554` and
  `:7566` give "usage, exit 2" for the same refusals.
- **Why Important:** this is the operator's reference for the wallet-creation
  sitting. A script or person following it reads 8 ("refused: a secret on the
  command line") as "invalid input" and 32 as "a check said no".
  `impl-toolkit.md` says only the lines the mr report listed were changed and
  the rest were "assumed to be mismatches, still 1"; that assumption is false
  for the lines above.
- **Fix:** walk every `exit N` in `45-mr.md` against the mr binary (the
  `MR_EXIT_RECORD` table the mr implementer built, `/home/user/fx/mr-exit-record*.tsv`,
  already has the pairs). Until done, put the caveat note back under the table.
- **Same defect, smaller, in `46-mr-sitting.md`:** `:309` "ST-PR-04 to ST-PR-07
  and ST-PR-12 are exit 2" (the chapter's own table: an existing file is 8, a
  missing `--dir` 32; measured an unknown case 64), and `:515` "ST-G-02 … (exit
  2)" (measured 64).

### I2. toolkit: a `--slot` xpub of the other network is 2 in `bundle` and `verify-bundle`, 8 in `export-wallet`, `convert`, `mk` and `md`

- **Where:** `crates/mnemonic-toolkit/src/synthesize.rs:643-653` and
  `:898-908` return `ToolkitError::CosignerSpec` ("xpub network Main does not
  match --network testnet"), which `error.rs` classes INVALID.
- **Measured, the same mainnet xpub with `--network testnet --template bip84`:**
  - `mnemonic bundle … --slot @0.xpub=…` → **2**
  - `mnemonic verify-bundle … --slot @0.xpub=…` → **2**
  - `mnemonic export-wallet … --slot @0.xpub=…` → **8** ("network mismatch: …")
  - `mnemonic convert --from xpub=… --to address --network testnet` → **8**
  - `mk address … --network testnet` → **8**
- **Rule:** §3 test 3 (network mismatch) comes before test 6; decision 1; §7f
  "Refusals move to 8: … network mismatch". `41-mnemonic.md:1402` documents 8
  for export-wallet; no manual line or test pins the 2.
- **Fix:** return `NetworkMismatch` (or `Refused`) at both sites and add one
  row per verb to a test. Also check `restore`/`import-wallet` slot paths for
  the same `CosignerSpec` text.
- **Seen in passing, not an exit-code defect:** `mnemonic bundle --network
  testnet --descriptor "wpkh([fp/84'/0'/0']xpub…/<0;1>/*)"` exits 0 and writes
  a "testnet" bundle around a mainnet key. That belongs to the network rule's
  unbuilt units; it is worth confirming it is on that list.

### I3. toolkit: input that is not UTF-8 is 32 on most read paths and 2 on others; every sibling says 2

- **Where:** `error.rs:697` classes `Io(InvalidData)` as INVALID, with the
  comment "a read whose bytes are not UTF-8 is a malformed input". But about
  15 read sites wrap the same error as `IoFailure(format!(…))`, which is 32:
  `passphrase_input.rs:632`, `slot_input.rs:241`, `parse.rs:24`,
  `cmd/convert.rs:813`, `cmd/compare_cost.rs:122`, `cmd/build_descriptor.rs:252`,
  `cmd/bundle.rs:359` and `:1639`, `cmd/verify_bundle.rs:1568`,
  `cmd/export_wallet.rs:888`, `wallet_import/json_envelope.rs:467`,
  `cmd/xpub_search/{seed_intake.rs:103,114, descriptor_intake.rs:122,
  address_of_xpub.rs:145, passphrase_search.rs:69}`.
- **Measured with `printf '\xff\xfe\n'`:**
  - `mnemonic repair --mk1 -`, `inspect --md1 -` → 2
  - `mnemonic bundle … --slot @0.phrase=-`, `convert --from phrase=-`,
    `restore --from phrase=-` → **32**
  - `mnemonic bundle --descriptor-file bad.txt` → **32**
  - `md decode -`, `md decode --in bad.txt`, `mk decode -`, `ms repair --ms1 -` → 2
- **Rule:** §3 row 2 ("a malformed value the tool reads"); `42-md.md:698` and
  `44-mk-cli.md` state 2 for non-UTF-8 input. 32 tells the operator to fix
  the environment when the input is wrong.
- **Fix:** one helper that maps a read error to `BadInput` when its kind is
  `InvalidData` and to `IoFailure` otherwise, used at every site above; a
  table test with one non-UTF-8 row per verb.
- **Related (R5, message agrees with code):** the 2 path prints `error: I/O
  error: stream did not contain valid UTF-8`. Say "the input is not UTF-8
  text", as `mk` does.

### I4. mnemonic-gui: a CLI whose legend has drifted is read as an *old* CLI, where 5 is green and 1, 2 and 8 are uncoloured

- **Where:** `src/cli_version.rs` `speaks_bits` / `probe_scheme`: any `--help`
  that answers without the exact legend at its end is `CodeScheme::Legacy`.
  `src/exit_badge.rs` `legacy`: 5 → green "Repair Applied (BCH correction
  succeeded: re-engrave the damaged plate)" unless the command is `md
  descriptor`; every code but 4 and 5 → plain `exit: N`.
- **Scenario.** The legend lives in separate copies today (toolkit
  `cli.rs:45-50`, md `exit.rs`, mt `exit.rs`, mr-codec `exit_codes.rs`,
  mnemonic-io-lib, and the GUI's `exit_code.rs:44`), each pinned only by its
  own repo's test. A spec revision that changes one word, or one copy edited,
  makes the GUI treat that renumbered CLI as Legacy with no error. Then:
  - `mnemonic verify-bundle` with a mismatch row and a `partial` result exits
    5 (1 + 4; `verify_bundle.rs:4251` sums them) and is drawn **green,
    "Repair Applied"**;
  - a mismatch (1), an invalid card (2) and a refusal (8) are drawn in the
    default colour, not red.
  `impl-gui.md` calls this direction "safe". It is not: Legacy is the more
  reassuring reading.
- **Not fooled today (checked):** all eight binaries end `--help`, `-h` and
  `help` with the legend, re-wrapping is normalised, a hung or failing
  `--help` is Undecoded, and no pre-renumber build has the text.
- **Fix (small):** make the answer three-way on content. Exact legend at the
  end → Bits. No paragraph beginning `Exit status:` anywhere → Legacy. A
  paragraph beginning `Exit status:` that is not the legend, or the legend
  not at the end → Undecoded, with that reason. Add the two missing tests
  (legend present but not last; legend with one word changed). Independently,
  restrict Legacy's green to `repair` (and the toolkit's auto-repair verbs),
  since no old CLI gave 5 elsewhere except `md descriptor`.

---

## Minor

### Toolkit code

- **M1. `mnemonic repair` with several kinds prints one group's corrected
  strings and then exits 2 for another.** Measured: `repair --mk1 <damaged>
  --mk1 <c1> --md1 <c0> --md1 <uncorrectable> --md1 <c2>` prints the mk1
  report and both corrected mk1 strings, then `error: repair: chunk 1 has too
  many errors`, exit **2**. R1 allows either an atomic failure (nothing
  printed) or the sum of what was printed (18); this is neither, and
  `41-mnemonic.md` says a rejected group "fails the WHOLE call". Order
  matters too: with the bad card first nothing is printed. Fix: decide every
  group before printing any (`cmd/repair.rs` loop), or sum the bits.
- **M2. R2 was hoisted only in `seed-xor` and `convert` (prior I6, folded in
  part).** Measured: `echo garbage | mnemonic ms-shares split --from phrase=-
  --threshold 1 --shares 3` → 2, with a valid phrase → 64 (`friendly.rs:163`);
  `--threshold 3 --shares 2` the same; `derive-child --application bip39
  --length 13` → 2 with a bad phrase, 64 with a good one (`error.rs:888`). The
  secret is read on a run that the command line already dooms. GUI manual
  `4f-ms-shares-split.md:94,100` states 64 without the condition.
- **M3. `restore --from xpub=…` is 64; the same kind of error is 2 elsewhere.**
  `cmd/restore.rs:453,936,1553` ("not a seed source for restore") is `Usage`.
  §3's worked table puts "`convert` from xpub to xprv" and "`export-wallet`
  given a secret slot" at 2 (a wrong kind of input for the verb), and the
  toolkit follows it there. GUI manual `4d-restore.md:653` documents the 64.
  Pick one; 2 matches the spec's table.
- **M4. The R13 table for `BundleMismatch` (prior M1) has no end-to-end
  trigger,** as the implementer notes. Left as is; listed so it is not lost.

### Toolkit manuals

- **M5. `41-mnemonic.md:3196-3198`:** the seedqr verbs "refuse a secret
  supplied inline … at exit 2". Measured 8 (`seedqr decode --from seedqr=…`,
  `seedqr encode --from "phrase=…"`).
- **M6. `42-md.md:184`, `--allow-invalid-miniscript`: "refused, exit 1".**
  §7a names this refusal: it moves to 8. The flag is not in the fx md build
  (md PR 76), so nothing could be measured; the line must not ship saying 1.
- **M7. `mk repair`'s incomplete-set message is no longer "UNVERIFIED".**
  Measured: `warning: corrected chunk(s) of an incomplete chunk set …: the
  correction is verifiable when the rest of the set is reassembled`. The
  manual still calls it an `UNVERIFIED` advisory at `44-mk-cli.md:264`, `:278`,
  `:320`, `:339`, `:363` and manual-gui `70-mk/79-repair.md:61` (whose own
  "Version scoping" paragraph at `:82-88` says the opposite). Decision 7 asked
  for exactly this wording change.
- **M8. manual-gui `30-tour/32-run-and-output.md`, "A non-zero-exit
  example":** the code (64) is right, the transcript is not. `mk inspect` with
  no argument prints `error: expected at least one mk1 string (positional, via
  --in FILE, or via stdin with '-')`, not clap's "required arguments were not
  provided" block.
- **M9. Appendix J, "Per binary":** the `mr` row omits 18 and 22, which
  `45-mr.md:70-71` states; the `mr-sitting` row omits 101.
- **M10. Spec §6's network-rule changes are neither done nor recorded.**
  `design/SPEC_network_rule.md:143` still says 8 "is reserved … and
  unreachable", `:146` "a CLI that exits 8", `:400-401` key the GUI rows on
  exit 8; `scripts/spec-network-rule-gate.sh` check 5 is unchanged. The
  implementer mentions the gate under "gates not run" but filed no FOLLOWUP.

### mnemonic-gui

- **M11. Green for 16 alone covers a correction that is not yet verified.**
  Under decision 7 a lone corrected chunk of a larger mk1 or md1 set exits 16,
  and the CLI's stderr says it "could alias to a different card" until the
  set is reassembled. The badge is green and says "re-engrave the damaged
  plate" (`exit_badge.rs` `bits_badge`). The code cannot tell the two cases
  apart, so this is a ruling for Brian: either 16 alone is not green, or its
  text says "verified, or verifiable when the whole set is read: see the
  notes". This is the one place the safety lens lands on a current binary.
- **M12. The scheme is probed on one file and the run may execute another.**
  `exit_scheme_cached` probes the path `path_detect::detect` finds on `PATH`;
  `run_plan` and the G1 `--version` probe spawn `argv[0]` by name. On Windows
  Rust looks in the application's directory before `PATH`, so a CLI shipped
  beside `mnemonic-gui.exe` is run while the one on `PATH` is probed. Old
  beside new then reads old `mk verify` 4 as amber, or new 5 as green
  (I4's table). Fix: probe with the same `Command::new(bin)` lookup as the
  run, or spawn the resolved path for both.
- **M13. Legacy still shows old `mnemonic` 4 as amber VERIFY-ME for every
  verb.** In the old scheme `mnemonic verify-bundle`, `restore` and
  `xpub-search` exited 4 for a mismatch (spec §1). Pre-existing, and every
  current pin is Legacy, so it is what users see until the pin bump. If the
  GUI ships before the CLIs, add `mnemonic verify-bundle|restore|xpub-search`
  4 → red to `legacy`, as was done for `mk`/`ms verify`.
- **M14. Test gaps.** No GUI test for "legend present but not last" or a
  one-word-different legend (I4). In the toolkit, the sibling-parity rows run
  only when `MD_BIN`/`MS_BIN`/`MK_BIN` are set, and no CI job sets them for
  `cargo test` (the expected values are hard-coded, so the toolkit side is
  still pinned; it is sibling drift that goes unseen).

## Nit

- **N1.** `44-mk-cli.md:621-622`: an empty `| Code | Meaning |` table above
  the intro sentence, the twin of the stray header fixed in `45-mr.md`.
- **N2.** `44-mk-cli.md`'s general table lists 22 but not 18; its repair
  table has both.
- **N3.** Appendix J, question 5 drops the spec's "a plan it should belong
  to" and the sentence that a limit, cap, deadline or delta is not a match.
- **N4.** Appendix J and the md/mk "When stdout fails" paragraphs give "2
  becomes 34" as an example. Measured: a verb that fails at 2 writes nothing
  and stays 2 with `>&-` or `/dev/full` in all four tools; 34 needs a failing
  verb that prints (`verify-bundle` mismatch → 34, measured). Use that
  example.
- **N5.** `mnemonic-gui --help > /dev/full` exits 0 (`main.rs` ignores the
  print error); R8 would make it 32.
- **N6.** FOLLOWUPS `ms-manual-exit-codes` (`:6622`) and `md-manual-exit-codes`
  (`:6632`) look resolved by this work and are still open.

---

## Leftover minors (prior reviews: should have been folded, or still open)

**Toolkit review (`exit-codes-impl-review-2026-11-26.md`):**
- **I6, partly folded:** see M2 above.
- **N1, partly folded:** stale old-code comments remain at `error.rs:356`
  ("exit 5"), `cmd/inspect.rs:101,126` ("exit 5") and `:249` ("exit 3"),
  `cmd/convert.rs:839,1116,1143` ("exit 5"), `wallet_import/bsms.rs:88` and
  `friendly.rs:242` ("FutureFormat (exit 3)").
- **N2, not folded:** `cli.rs` test `root_help_ends_with_the_exit_code_legend`
  compares with `ends_with` on a wrapped literal; mnemonic-io-lib's legend is
  one line, so the switch to the shared crate breaks it. Normalise whitespace.
  This is also I4's root: the copies are not one constant.
- **N5, not filed:** two mk1 chunks with different `chunk_set_id`s that cover
  every index exit 0 in `mk` and `mnemonic` (N8). No FOLLOWUP or spec line
  found.
- **M1:** class-table test only (M4 above).
- **M3:** R6 adds 16 to a failing run (measured: `mk verify` mismatch on a
  corrected card 17, `mk address` network refusal 24, `mnemonic verify-bundle`
  18 and 19). The tools agree with each other, but R1's "a verb that fails
  atomically reports only its failure bits" still reads the other way. It
  needs one spec sentence, not code.
- **"Gaps in 45-mr and 46-mr-sitting (not counted as defects)":** now counted,
  I1.
- **M11's twin:** N1 above.
- **Open items the implementer listed and nobody closed:** `42-md.md:184` (M6);
  bundle row-19 inline-versus-slot path mismatch 1 against the slot
  fingerprint-versus-annotation check 2; manual-gui `verify-examples` fails on
  17 transcripts called "unrelated" (argv-secret refusals are among them, so
  this should be looked at once, not assumed); manual-gui lint phases 4, 5 and
  9-11 not passing.

**GUI review (`exit-codes-impl-review-2026-11-26.md`):**
- **M3:** 16 alone stayed green with the action added to the text; see M11.
- **I1's condition:** "each CLI should keep a test that its root help ends
  with the legend". I confirmed by running that all eight do end with it; I
  did not confirm each repo has the test (mk, toolkit and mr do).
- **N3:** `run_bytes.py` counts a failing run with stdout as "ok"; still only
  a note for the re-measurement at the pin bump.
- **Not run by anyone:** the tutorial snapshots, `schema_mirror` and the
  secret-channel re-measurement against the renumbered binaries.

---

## Checked and found consistent (no finding)

- **Repair parity, measured, `mk`/`md`/`ms` against `mnemonic`:** clean
  complete set 0; corrected complete set 16; corrected lone chunk and
  incomplete set 16; clean lone chunk 0; a repeated chunk 2 with nothing
  printed (md); single-string md1 20; ms1 20; ms1 on argv 8 in both.
- **Empty stdin and a blank line:** 2 in `mk`, `md`, `ms repair` and every
  `mnemonic` reader tried.
- **Stdout failure:** a succeeding verb with `>&-` or `/dev/full` is 32 in
  all eight binaries; a corrected repair is 48 in `mk`, `md` and `mnemonic`;
  a usage error stays 64; an argv refusal stays 8; `| head` keeps the code.
- **R7:** `mnemonic inspect --md1 <damaged>` is 2 with `MNEMONIC_FORCE_TTY`
  0 and 1 (the GUI sets it to 1).
- **R6:** `inspect --mk1`, `restore --cosigner` and `verify-bundle` on a
  damaged mk1 add 16 and print the note; the verify-bundle JSON result is
  still `ok`.
- **`md` chapter spot checks:** partial codes, `--verify-against` 0 and 2,
  `--chain 7` refused at 2 before any verdict, `--in` missing 32, `gen-man`
  output directory 32, no verb 64.
- **`ms` chapter:** `inspect` and `verify` on a bad checksum 2; clean 0.
- **Per-verb `--help` in `mnemonic`:** every code it quotes is a new one.
- **GUI:** the badge mapping for Bits, 64, 101, 130, abnormal codes and
  signals reads correctly; the scheme is stamped before the run and after the
  G1 gate; only definite answers are cached; the README and tooltips carry no
  version-floor text.
