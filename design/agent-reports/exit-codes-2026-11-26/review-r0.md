# R0 adversarial review: SPEC_constellation_exit_codes.md (DRAFT, 2026-11-26)

Reviewer: independent, read-only. Spec: `/home/user/mnemonic-engrave/design/SPEC_constellation_exit_codes.md`
("S:n" = spec line n). Evidence: the six survey reports in `/home/user/wt/ec/`,
the related specs, and about 95 fresh runs of the built binaries (stdin
`</dev/null`, scratch `/home/user/wt/ec/tmp-review/`, surveyors' fixtures read
only). No repository was edited.

Verdict: **not GREEN: 2 Critical, 16 Important.** The arithmetic of the scheme
is sound and the survey transcription is mostly accurate. What fails is (a) the
class boundaries, which the spec itself applies inconsistently, (b) one D26
meaning that does not survive summation, and (c) several rulings reversed
without being listed in §6 or offered as decisions.

---

## Lens 1. Soundness of the scheme

Arithmetic, checked: six bits give 63 distinct sums; 64, 101 and 128+N are
disjoint from 1..63 and from each other; 126/127 are untouched. That part holds.

### C1 (Critical). D26's "verified vs unverified correction" does not survive summation: 20 is ambiguous

S:146-147 encode D26's two meanings as 16 (verified) and 20 = 16+4 (unverified).
But bits are per run and D26 is per correction. Bit 4 is also set by things
that have nothing to do with the correction (S:112: "a partial decode, an
incomplete set, a signature accepted leniently"). So a run with a **verified**
correction plus an unrelated bit 4 also exits 20:

- a chunked dead-origin md1 with one substituted character: the set
  reassembles (16, verified now) and the card partial-decodes (4). Under R6
  `mnemonic inspect` / `md decode` exit 20, which S:147 defines as
  "VERIFY-ME: correction with no self-oracle";
- `verify-bundle` "partial" (4, S:346) on a bundle one of whose chunked cards
  was corrected and reassembled (16);
- any batch verb (R1, S:152-153): input A corrected and verified, input B
  partial.

The same holds for 22 (S:386-388). The owner's requirement "any combination
sums uniquely" is met for bit *sets*, but the D26 meaning is a *conjunction*
of two bits, and the conjunction is not unique. It errs on the safe side
(over-warns), but a consumer can no longer tell "this correction has no oracle"
from "verified correction, unrelated caveat", which is the one distinction D26
exists to carry. Options: give "correction without an oracle" its own bit
instead of 16+4 (it needs a seventh bit, which reopens D1), or keep 16+4 and
state the over-warning as an accepted loss. Either way it is a decision.

### C2 (Critical). REFUSED (8) versus INVALID (2) is not decidable, and R3 misstates §6i

R3 (S:157-162) says the split "is the taxonomy of uniformity spec §6i". It is
not, and the spec contradicts itself with it:

1. **Same failure, two codes inside this spec.** §6i lists "an `ms1` string
   handed to a transaction tool" among the ten *correctness* refusals
   (uniformity spec §6i, lines 1337-1343). S:302-303 sends those ten to 2 in
   `mt`. S:113 puts "secret material where public material is expected" in
   REFUSED, and S:323 sends `me`'s ms1 refusal (measured today: `me --hex --in
   ms.txt` -> 3) to 8; S:250, S:273 send an xprv-as-xpub to 8 in `md` and `mk`.
   So ms1 into `mt` is 2 and ms1 into `me` is 8.
2. **The stated test contradicts the stated example.** S:113 defines REFUSED as
   "a reason a different channel, network or explicit `--allow-` flag could
   clear". R3 (S:160) gives "an unsigned transaction" as the type case of
   INVALID. `me sysw pack --allow-unsigned-inputs` exists
   (`mnemonic-engrave/crates/me-cli/src/main.rs:302`, `:1957-1961`): by the
   definition it is 8, by R3 it is 2.
3. **§5 uses 8 for refusals that fail the definition.** Measured today:
   `mnemonic compare-cost --descriptor "wpkh(...)"` -> 3, "unsupported wrapper
   ... not a miniscript-wrapping question"; `mnemonic convert --from xpub=...
   --to xprv` -> 2, "cryptographically unrecoverable". No channel, network or
   flag clears either, yet S:360, S:364 send both to 8. Same for mr's
   "published-vector root" and seating refusals (S:382), and md's seating
   refusals are not classified at all.
4. **§6i's posture is "who can observe the machine"** (argv, world-readable
   stdout, terminal gate) and it says "No behaviour follows from it". Network
   mismatch, output-overwrites-input, secret-where-public and
   `--experimental`/`--md-only` gates are none of those. Conversely
   `mr-sitting`'s "no RAM-only folder / not on tmpfs" *is* posture by §6i's
   definition and S:398 sends it to 32.

Two implementers will classify these differently; the spec's own authors did.
The scheme needs an operational test per class (a decision procedure, in
order), and §5 re-derived from it.

### I6 (Important). R2 is wrong about its own example, and collides with the normative argv-guard ordering

- S:155-156 and S:255-256 class `md descriptor ... --chain 7` as USAGE (64,
  "the command line was wrong and nothing ran", S:116). Measured:
  `md descriptor C1 C2 C3 --chain 7` -> 1, `md: codec error: chain index 7 out
  of range; use-site multipath alt-count is 2`; `--chain 2` -> 1 the same way;
  `--chain 1` -> 0. The limit is a property of the **card**, known only after
  decoding. It cannot be checked "before work", and if the card is also
  invalid or corrected the run has other bits to report, which 64 forbids. The
  real defect (verdict printed before a later failure, md.md C14) is an
  ordering bug; the class is INVALID-for-this-request. `me` already has the
  right test in its source: "A flag value out of range is USAGE, not invalid:
  no input has been read at the point it is caught"
  (`me-cli/src/main.rs:493-494`). The spec should adopt that as the definition
  of USAGE.
- Uniformity spec §6d: the argv guard "runs on raw `std::env::args()` BEFORE
  any argument parser ... This is NORMATIVE". Measured: `ms encode
  --nosuchflag --hex 00..00` -> 1, the refusal wins over the usage error. R2
  ("a usage error exits 64 before any output") and "64 never combines" give no
  precedence between a REFUSED argv secret and a USAGE error on the same line.
  State it: argv refusal (8) outranks 64.

### I7 (Important). USAGE vs INVALID for a bad *value* is undefined, and an owner ruling sits on that boundary

S:111 puts "not a valid ... value for this request" in INVALID; S:255, S:339,
S:375 send some program-checked values to 64. Unclassified by the spec: md
`--fingerprint @0=zz` / bad template (1 today), mk `--origin-fingerprint zz`
(64), ms bad K/N and bad BIP-39 phrase (1), `@env:` variable unset (1 in ms and
mnemonic), `mr create` with no template (1), `mr root --from coins` with zero
flips (2). For `mr` this is not free: the mr spec's E45 closed list and the
owner ruling "Keep 1" (Brian, decision card, 2026-10-06;
`mnemonic-refugium/design/SPEC_refugium_mr1.md:27508-27551`) rule that
malformed hot-family values are refusals, not usage errors, and E52 rules which
missing inputs are 1 and which 2. S:375-378 renumbers across that ruling
without mentioning it (see I11).

### I8 (Important). UNVERIFIED (4) versus "warning, exit 0" is undefined; so is R5's report/classify split

- S:112: "output was produced, but it rests on something not verified".
  Measured: `mnemonic restore --from phrase=- --template bip84` -> 0 with
  `UNVERIFIED: no --expect-fingerprint/--expect-xpub supplied` on stderr. By
  the definition that is bit 4, which makes every reference-less restore
  non-zero; the spec is silent. Same question for `mt` with no node (measured
  0, "could confirm NOTHING about it against the chain"; R9 says warning,
  S:181-183), mr "verify: passed, but ...", md `encode`'s "card will
  partial-decode" warning, mk's stale chunk-set-id warning.
- R5 (S:165-168): `inspect` must set INVALID, but "a verb whose job is to
  classify" exits 0. `ms inspect`'s job is arguably to classify (it prints
  `FAIL: would NOT decode v0.1` with a reason; measured 0). `mt inspect`
  STATUS DEAD is 0 today as "a report, not a verdict" (mt-me.md:100): NO, or
  0? No rule separates the two kinds of verb.

### I9 (Important). There is no class for "some other failure", so a stray 1 reads "a check said no"

Unclassifiable today-outcomes: internal serialize failures (ms -> 1,
mk-ms.md:136), the caught BIP-322 unwind (mnemonic -> 1, toolkit.md:113),
"not Linux" (mr-sitting 2), session lock held, picotool packer self-check,
`ExitCode::FAILURE`, and Rust's default exit 1 when `main` returns `Err`
(mnemonic-gui's eframe error, gui.md:40). Under the scheme, any unmigrated or
unforeseen error path that yields 1 is read by a script or the GUI as "verify
mismatch". Either a rule ("anything unclassified is 101, never 1") or a bit is
needed; today the spec has neither.

### I2 (Important). Atomic repair under R1: 2 or 18?

R1 (S:152): "A run sets every bit that applies." D26's atomic-fail: "the WHOLE
call exits 2 ... NO partial corrected output". Measured: `md repair C1 C2BAD
md1qqqq...` -> 2, nothing on stdout (chunk 1 was correctable, chunk 2 was not).
REPAIRED is "a checksum correction was applied to some input" (S:114): 2 or
2+16? Also unspecified: `mk repair` `SetReassemblyMismatch` (2 today: corrected
but the set does not reassemble), and md's "single card corrected at an
unsupported wire version" (5 today, md.md:88).

### I15 (Important). "A batch verb sums over all its inputs" versus fail-fast

Refusing verbs are atomic and stop at the first failure. To report the full
sum a run must keep reading inputs after it has decided to fail (including
after an argv-secret refusal). Say whether the sum is "bits observed up to the
stop" or "all inputs evaluated"; the two give different codes for one input set.

### M11, M12 (Minor)

- M11: every bit test needs the `< 64` guard first (130 & 2 != 0 reads
  "invalid"). The script at S:127-137 has it; the R11 legend (S:191-195) and
  R12 do not say it. mnemonic-gui also runs on Windows, where abnormal exits
  are large NTSTATUS values; the decoder must range-check 1..63.
- M12: S:71 "seven bits at most, with every sum below 126" is wrong arithmetic
  (seven bits reach 127; six reach 63).

---

## Lens 2. Faithfulness

Checked about 60 cells against the reports and re-ran about 95 commands. **All
file:line pointers I opened are right** (md `main.rs:930-961`; mk
`error.rs:135`; ms `error.rs:60`; mt `main.rs:297-378`; me `main.rs:495-500`;
toolkit `error.rs:641-702`, `out_sink.rs:89`, `cli.rs:173`; gui
`app_window.rs:1316-1356`, `schema/ms.rs:649`, `tutorial/manifest.rs:104`,
`schema/mnemonic.rs:2358`; wallet `mr_panel.rs:240-246`, `create/mod.rs:4031`,
`create/sitting.rs:1181`; `rehearse.sh:60`; pin `25051a06`).

Re-run and confirmed (today's code): md verify-against NOT spend-equal 5, with
`--chain 7` 1 after the verdict; `md decode OK >&-` 0; md repair single string
5 / chunked 5 / mixed-uncorrectable 2; `md decode ''` 1 vs empty `--in` 2;
`md gen-man --out /proc/...` 2; `md compose --wrapper bogus` 1; partial decode
4; `mk repair N12` 0 vs `mk decode N12` 3; `mk decode` one-error 0; mk repair
complete 5 / incomplete 5; mk verify mismatch 4; mk empty 64, missing file 64,
multisig `address` 64; `ms inspect` future/wrong-HRP 0 with FAIL; `ms verify`
future 3 with "OK"; ms repair 4; ms empty 1 (repair 2); ms argv secret 1;
ms verify mismatch 4; mt `--json --quiet` 1, non-UTF-8 64, clap 2, corrected
verify/decode 0, empty 1, failing `--bitcoin-cli` 0; me non-UTF-8 101, bad md1
4, ms1 3, empty 2 (`sysw show` 4), missing file 2, `--expect` 4; mr repair 5
on the not-a-card vector, 0 on the clean not-a-card string, decode 1 / repair
2 on garbage, usage 2, `create` 1; mr-sitting usage 2, missing plan 2;
mnemonic inspect bad md1 1 piped / 5 with `MNEMONIC_FORCE_TTY=1`, `inspect
--mk1` one-error 0, repair 5 / 4 / 4, restore mismatch 4, `--allow-mismatch`
0, argv guard on `--from bogus=1` 2, `--import-json-index 5` 1, empty 1
(`build-descriptor` 2), clap 64, compare-cost 3, `verify-bundle` mismatch to
`/dev/full` stays 4.

Wrong or unsupported:

- **I16 (Important). The pin counts are not on one basis and understate mr.**
  S:225-226 defines "Pins" as assertions "on an exact non-zero code".
  - `mt` "about 130" (S:295) is the survey's count **including ~82 on 0**;
    non-zero is about 36 (mt-me.md:186-188).
  - `me` "about 110" (S:314) includes 22 on 0 (mt-me.md:194-196).
  - `mr` "about 1,400 status assertions, most of them on 0" (S:369): the
    survey's literal counts are 0 ≈ 2,500, **1 ≈ 1,700, 2 ≈ 230**, 5 = 7
    (mr.md:142), and it says "changing 1 or 2 touches thousands of assertions
    across ~170 test files plus ~1,100 TRACE/spec lines" (mr.md:148). The
    spec's phrase hides that mr alone is roughly two thirds of the whole
    renumber.
  - `mk` "about 86" and `ms` "about 139" (S:258, S:278) are not in the survey;
    its rows sum to about 93 and 146 (mk-ms.md:319-324).
- **M1.** S:38-39: "Inside `mt`, `me`, `mnemonic` and `mr`, flag clashes that
  the program checks itself use yet another code". Not `me`: its
  program-decided usage errors are 2, the same as clap's (mt-me.md:102;
  re-run). And S:36 "Each binary uses one code everywhere" is contradicted by
  S:209 itself (mt 2 / 1 / 64) and by md `compose` values (1).
- **M2.** S:61-62: "`mt` and `me` document no codes at all". They have no
  manual chapter and no `--help` codes, but the READMEs and me's source table
  do document 1, 2, 3 (mt-me.md:165-174). S:60-61 lists `mnemonic` among
  "per-tool tables"; the survey says whole-binary tables exist only for `mk`,
  `mr`, `mr-sitting` (toolkit.md:392-393).
- **M3.** S:57-58: "4 means 'verify mismatch' or 'unverified repair' in `ms`,
  `mk` and `mnemonic`". `mk`'s 4 is only verify mismatch (mk-ms.md:105-106);
  md's 4 (partial decode) and me's other 4s (`--expect`, sidecar render) are
  omitted.
- **M4.** S:179-180 and S:497-498 name `md`, `mk`, `ms` for "exit 0 with stdout
  closed". It is every binary: `mr` (mr.md:45; re-run 0), `mnemonic`
  (toolkit.md:116-117; re-run 0), and, not in any survey, **`mt decode --in
  set.txt >&-` -> 0 and `me --hex --in md.txt >&-` -> 0** (measured for this
  review).
- **M5.** S:230-232 "Clap errors and `CliError::BadArg` go to 64" contradicts
  S:244-254: `BadArg` also carries empty input (stays 2), missing files (32),
  keyless card (2), `--out` names the input (8), output directories (32)
  (md.md:78-85).
- **M6.** S:18-19: the reports "are kept verbatim under
  `design/agent-reports/exit-codes-2026-11-26/`". That directory does not
  exist; they are in `/home/user/wt/ec/`.
- **M8.** S:450-451: Create's path call "becomes 8". Nothing in gui.md says
  what kind of refusal that code 1 is (it could as well be 2).
- **M13.** S:297-298 "returns SUCCESS/FAILURE only": the same range also
  returns clap's code and 64. S:302-303 and S:310 conflict: "an input absent
  from the UTXO set" is one of §6i's ten (so 2) and also "a node verdict that
  a check failed" (so 1).
- **N1.** R6 (S:170-171) says `mt` and `mk` "correct silently"; both print a
  stderr note (re-run: "CORRECTION APPLIED. 1 chunk needed repair"). Only the
  exit code is silent. The truly silent case is `mnemonic inspect --mk1`.
- **N2.** S:262 "Invalid stays at 1 → 2" is garbled.

---

## Lens 3. Rulings

### I1 (Important). §5b applies D26 against D26, and against the spec's own table

S:146 (correctly): `REPAIR_APPLIED` = "correction verified now, **or
verifiable by reassembly**" -> 16. S:265-266: an `mk` **incomplete set** moves
"from 5 to 20". But D26's text names exactly that case as the
"verifiable-by-reassembly later" branch of REPAIR_APPLIED: "(an `mk1`
single-plate chunk)" (`42-md.md:700-705`), and "this binary's own
incomplete-group case" (`44-mk-cli.md:278-281`). `mk repair`'s 5 conforms to
D26 today (re-run: 5 with the UNVERIFIED advisory); the toolkit's 4 for the
same input is a documented second reading. The spec silently picks the
toolkit's reading and deletes D26's second branch while claiming "its meanings
carry over and only the numbers move" (S:140-141). That is a change to D26 and
belongs in §2 as a decision and in §6.

### I3 (Important). R6 reverses a fixed rule of the mt spec and breaks its normative one-liner; not in §6

`SPEC_mt_v0_1.md:3649-3660`: "`0 = every check passed` is fixed"; "Exit codes
are the exception that should not wait: §1.1a's documented pipeline now depends
on a non-zero exit". `:819-836`: `decode` "exits non-zero otherwise", and the
printed one-liner `mt decode ... > tx.hex && ...` "is specification". Under R6
plus S:308-309 a plate with one corrected character, whose content id
re-derives (every check passed, hex written; re-run: 0 today), exits 16, and
the specified pipeline stops. The mt spec is owned by mnemonic-engrave, the
same repo as this draft.

### I5 (Important). The network rule: more is reversed than §6 says

S:463-465 says "exits 8 stays true" and 8 "is widened". Not listed:

- N9 (`SPEC_network_rule.md:140-143`): "`ms` and `mt` never refuse on the
  network; 8 is reserved for them and unreachable". Under S:289 and S:305 both
  reach 8 routinely (argv secret).
- N10 (`:145-147`): "a CLI that exits 8 prints one extra line" (the machine
  line). §4 rows (`:400-401`): "mnemonic-gui, any form, CLI exit 8 -> red
  banner"; "exit 8 with the machine line's class `soft` -> Override button".
  Both are keyed on the whole number 8, which R12 forbids (S:196-197), and
  with 8 widened the banner would fire on every argv refusal. The GUI must key
  on the machine line instead, which is only printed when
  `MNEMONIC_NETWORK_JSON=1`.
- "Exits 8" is exact in N5/N7/N9, in `vectors.tsv`'s `exit` column (0 or 8),
  in `nr0_exit_code_8_is_network_mismatch`, in mk's JSON envelope
  (`exit_code 8`, N5) and in the manual row text. Under R1 + R6 a network
  refusal on a run that also corrected a card exits 24. Say whether a refusal
  short-circuits the other bits.
- §8 of the network rule: "`PromotionTestnetKey` stays 3" (deliberately not a
  network mismatch); S:323 moves all of `EXIT_REFUSED` to 8.
- `scripts/spec-network-rule-gate.sh:35-40` greps every repo to keep 8 unused.
- S:464-465 "'nothing was written' is kept for every refusal" is a new
  requirement, not a kept one: `me`'s destination refusal happens after a
  successful convert, and `--manifest`-is-a-plate after rendering
  (mt-me.md:122-126).
- Sequencing: the network rule is ruled but unbuilt (toolkit.md:221-224). Its
  units add `NETWORK_MISMATCH = 8`, tests and manual rows that this spec then
  rewrites. Which lands first is a decision.

### I11 (Important). Other rulings and normative texts reversed without a line in §6

- **§6f's one change is undone.** "`mk`'s invalid-artifact 2 becomes 1 ... the
  only code this cycle changes" shipped (mk
  `tests/exit_code_invalid_artifact.rs`). S:262 moves it back to 2, and moves
  md/ms/mt/mr to it. S:466 says only "was done".
- **mr's normative spec.** E45 ("0 / 2 usage (closed list) / 1 any other
  refusal"), plan decision D9, the "Keep 1" ruling (I7), and "The exit code is
  the checksum's either way" (`45-mr.md:2103`; mr CLAUDE.md: "its exit code
  stays the checksum's even when the corrected strings are no card").
  S:386-389 reverses the last one, and S:505-506 lists the specified
  behaviour under "Defects ... behaviour bugs". It is a ruled behaviour with
  its own corpus vector, not a bug. Also inconsistent: S:268-269 makes `mk
  repair` decode-check a checksum-clean card, but `mr repair` on a clean
  not-a-card string (re-run: 0 plus a warning) is left alone.
- **ms's "locked" table** (`SPEC_ms_cli_v0_1.md:440-468`, with its rationale
  for 1 vs 2 vs 3) and ms's deliberate clap -> 64 carve-out.
- **me ruling T6** (write failure = 1, `main.rs:499`) and the
  one-vocabulary table test.
- **toolkit**: "exit 4 is the toolkit's project-wide VERIFY-ME class"
  including a bundle/seed mismatch (`app_window.rs:1319-1328`), which S:342
  splits into NO; `tests/cli_p3_argv_refusal.rs:35` ("2, not 3 and not 64").
- **md** `--verify-against` 5, the deliberate "reserved-5 precedent"
  (`main.rs:609-612`).
- **refugium-wallet** `SPEC_ui_refugium_wallet.md:1432-1442` (§4.1.1), the
  normative mapping of `outcome.code()`.
- **`restore --allow-mismatch`**: help says "loud banner, exit 0"
  (toolkit.md:213); R4 makes it 1 (see I4).

mnemonic-io-lib `exit.rs`: the quotation is fair (`exit.rs:13`, "A CONSTANT IS
A MAPPING"), and D4 addresses the seam. Missing from D4: the module's other
stated reason, that the usage split "is ruled out of scope" (`exit.rs:9-10`),
which D2 reverses; and that `md-cli`, `mk-cli`, `ms-cli`, the toolkit, `mt-cli`
and `me-cli` all already depend on the crate (so only mr needs a copy), with
`mt` on the crates.io release (0.1.3), so step 1 of §7 is a publish plus six
pin and `vendor/` moves.

---

## Lens 4. Consequences not listed in §5 / §7

### I3 again: R6 turns "success with a caveat" non-zero on reading verbs

Today a non-zero success exists only on `repair` and md partial decode. R6
extends it to `mk decode/verify/inspect/address/derive`, `mt
decode/verify/inspect` and `mnemonic inspect --mk1` (all re-run 0 today).
Every `cmd && next`, `set -e` script, `make` recipe and `xargs` call that
reads a plate with one worn character starts failing, including mt's specified
one-liner and mr's printed "it must exit 0" procedure lines where they run
`mk`/`ms`. The spec's argument (the code should not hide a damaged plate) is
sound; the cost is that 0 no longer means "you may use the output". **This
must be a numbered decision**, with the alternatives (a `--strict` flag;
REPAIRED only on `repair` and stderr/JSON elsewhere; or accept and rewrite
every documented pipeline to test `rc & ~16`).

### I4 (Important). R4 is much wider than the one example, and makes the code say less

- Re-run: `restore --expect-fingerprint deadbeef` -> 4, 0 bytes on stdout;
  with `--allow-mismatch` -> 0, 402 bytes. Under R4 both are 1, so the code no
  longer says whether output was produced, and a script that passed the flag
  on purpose now fails under `set -e`.
- The override vocabulary is not `--allow-`: `--accept-coin-type-mismatch`
  (ruled name, N7), about fifteen `--accept-*` flags in mr
  (`--accept-stub-mismatch`, `--accept-tip`, `--accept-seed-language`,
  `--accept-late-unlock`, ...), `--experimental`, `--md-only`, `--seal-secret`,
  `--pack-preimage`, `--allow-weak`, `--allow-unsigned-inputs`. For each one
  the spec must say whether what it overrides is a REFUSED (flag gives 0) or a
  fact (flag still gives 1). N7 already rules exit 0 for the network override.
- The owner has said he is unsure about an `--allow-` prefix. S:113 and R4
  write that prefix into the definition of a class.

### I12 (Important). Consumers and gates the spec does not mention

- **JSON wire shape.** `mk` and `ms` error envelopes carry `exit_code`
  (mk-ms.md:50, 343), `mnemonic`'s auto-repair envelope carries
  `"exit_code":5` (toolkit.md:148-149), and N5 fixes mk's envelope at
  `exit_code 8`. The GUI's `--json` consumers have no drift gate (toolkit
  CLAUDE.md).
- **mr corpus.** 46 vector files carry `"exit": N` (mr.md:145; I counted 46
  `"exit"` lines under `mr-codec/tests/vectors/`), the constants are
  duplicated in `mr_codec::repair::EXIT_*`, and code 2 switches the
  "usage-quiet" trailer (mr.md C12).
- **Printed text.** `mr --help` and mr's printed advice say "must exit 0",
  "exits 1", "Refused (exit 2)" (mr.md:133-137); these are under mr's
  quote-allowlist, printed-command and insta-snapshot gates, and the plan is
  frozen at M15. The toolkit pin `ci/toolkit-manual.rev` must move too.
- **Scripts.** mnemonic-key `scripts/` (about 30 `rc=$?` rows and four gate
  scripts, mk-ms.md:343-347); toolkit `prove-envelope-consistency.sh` (refusal
  must be 1), `verify-examples.sh`, `root-shaped-test.sh`; mr
  `spec-gate-sitting-kit-day.sh`, `golden/diff-wrapper.py`,
  `create-key-kind-fix-gate.sh`; descriptor-mnemonic's gate script;
  mnemonic-engrave `policy-differential.py`. §5 names only me's three gates
  and `rehearse.sh`.
- **Cross-repo tests.** mr tests run `mk`/`ms` (`jf8_pr64.rs`,
  `jf8_b_advice.rs`), me runs `ms` (`ms_remedy_runs.rs`), refugium-view has
  about 170 test lines on `code()` (gui.md:179-183).
- **Scope.** `me-preview` (Go, 0/1), `gui_render`, `gen_ap2_grind` and
  mr-gui's child codes (3, 1) are binaries in these repos; S:5-8 neither
  includes nor excludes them. `mt` and `me` have no manual chapter for
  per-verb codes to go in.

### I13 (Important). The cutover in §7 cannot be enforced, and every old code is a valid new sum

- mnemonic-gui runs whatever binary is first on `$PATH` and checks a minimum
  version only before secret-bearing runs (gui.md:271-273). "Step 3 does not
  trail" (S:485-486) is therefore not something a release order can deliver.
- Old codes read through the new table are all plausible and wrong: old 5
  (`md` NOT spend-equal, or repair applied) = "no + unverified"; old 4 from
  `mk verify` = "unverified, output produced"; old 3 = "no + invalid"; old 2
  usage = "invalid"; old 1 anything = "a check said no". A GUI or script needs
  a positive signal of which table a binary speaks (a version floor for every
  run, or a capability line), and the spec should say so.
- S:437-438: "`0 | 4` as 'has output' move to 'bit 4 or 16 set'" is wrong as
  written (0 has output; 22 and 6 may not). "Was the output written?" is not
  derivable from the bits at all (see I4). Consumers ask exactly that
  question, so the spec needs a rule.

### I14 (Important). R7 silently decides how auto-repair behaves

Re-run: with `MNEMONIC_FORCE_TTY=1`, `mnemonic inspect` on a one-error card
prints "# Repair report" and the corrected card *instead of* the inspect
output, and exits 5. R7 (S:173-176) says only that `isatty` must not decide.
If auto-repair is then on by default, a piped `inspect`/`convert`/
`verify-bundle` puts a repair report on stdout where the consumer expects the
verb's output; if off, the GUI (which sets the variable on every spawn,
`runner.rs:106-136`) loses the feature. S:354-355 ("an auto-repaired card that
the verb then used") implies a third behaviour, repair-and-continue, that does
not exist today. One of the three must be chosen explicitly.

### I10 (Important). R9 contradicts §5d

R9 (S:183): "A helper that was asked for and failed sets 32." S:311-312: "An
absent or failing `bitcoin-cli` stays 0". Re-run: `mt inspect --bitcoin-cli
./cli-fail.sh` (explicitly asked for, exits 1) -> 0. Likewise `me bundle
--preview DIR` asks for a preview; with no sidecar it is 0 (S:330), and with a
sidecar and a missing preview directory it is 2 today and unclassified here.
"Asked for" needs a definition.

### Minor

- **M9.** D4's copy "with a test that compares it to the table in §3"
  (S:102-103): mr's CI has no checkout of mnemonic-engrave. The network rule's
  precedent is a vendored file pinned by sha256, and "each repo carries its
  own copy of the helper (no shared crate)" (`SPEC_network_rule.md:600`).
- **M10.** R12 (S:197-198) lists GUI codes 0, 64, 32, 101. Not placed: mr-gui's
  1 for a crash after first frame, its release `panic = "abort"` (SIGABRT,
  which S:218 does note), mnemonic-gui's deliberate 130 for SIGTERM
  (gui.md:43).
- **M14.** Unplaced outcomes: `ms verify` printing "OK: valid future format"
  while exiting INVALID (S:283-284); mr-sitting "holder stopped" kept as NO
  (S:400) though no check ran, "output path exists", "hand-back not this
  plan's"; `build-descriptor` gate diagnostics (2); `Bip388VerifyDistinctness`
  and `DescriptorReparseFailed` (4 today: NO or INVALID?).
- **M15.** S:366-367 relies on CLI-03 (`--json-out` created 0600), which is a
  proposal, not built; today the tool warns "inherits umask".
- **N4.** R10: a killed process does not "exit 128+N"; the shell reports it so
  (Rust sees no code; Python sees -N).
- **N5.** S:476 "no existing card holders" has no bearing on exit codes.

---

## Lens 5. Decisions

D1 to D4 are the right questions and D2, D3, D4 are framed fairly. D1 is
framed too narrowly: it presents "own bit for network" as the only
alternative and omits that the N10 machine line already distinguishes a
network refusal, and it does not show Brian the N9/N10/GUI-banner consequences
(I5). D2 should state the churn both ways (five binaries move to 64; CLI-14
leaned the other way because "changing three is less churn").

Decided silently and should be decisions:

1. **R6**: non-zero on reading verbs when a correction was applied (I3).
2. **R4**: overrides keep NO; and the list of flags it binds (I4).
3. **D26 narrowing**: incomplete set -> 20, and what 20 means when bit 4 has
   another cause (C1, I1).
4. **The class tests**: REFUSED vs INVALID, USAGE vs INVALID, UNVERIFIED vs
   warning (C2, I7, I8), including whether mr's "Keep 1" ruling is reopened.
5. **R7**: the auto-repair default (I14).
6. **A fallback** for unclassified failures (I9).
7. **mr scope, cost and order**: mr is about two thirds of the pinned
   assertions, has a frozen plan and a normative spec with about 900 TRACE
   lines; and whether this lands before or with the network-rule build (I5,
   I16).
8. **R5**: `inspect` exits non-zero on a string that would not decode.
9. **The bit assignment itself** (1 = NO, 2 = INVALID): it reuses every number
   in use today with a new meaning; that is the root of I13.

---

## Summary

- Critical: 2 (C1 the 16+4 conjunction is ambiguous; C2 REFUSED/INVALID undecidable and §6i misstated).
- Important: 16 (I1-I16: D26 misapplied to mk; R6 vs mt spec; R4 scope; network-rule fallout; R2; class boundaries; unlisted reversals; missed consumers; rollout; auto-repair; R9; costs).
- Minor: 14 (M1-M6, M8-M15; M7 was folded into I16); Nit: 4 (N1, N2, N4, N5).
- Faithfulness: about 60 cells checked, about 95 runs; every file:line opened is right; errors are in §1 prose, the pin counts and the closed-stdout list.
- Arithmetic of the scheme (sums 1..63, 64, 101, 128+N) is sound; the defects are in class definitions and in rulings reversed without being listed.
- Not GREEN. Fold C1, C2 and the nine silent decisions before a second round; the survey evidence does not need redoing.
