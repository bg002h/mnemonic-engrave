# R0 round 0 — FOLD summary

**Artifact folded:** `design/SPEC_constellation_cli_uniformity.md`
**Report folded:** `design/agent-reports/R0-cli-uniformity-spec-round0.md`
(4 Critical / 11 Important / 6 Minor / 2 Nit — 23 findings)
**Folded:** 2026-08-26.

**Counts: 22 FOLDED, 1 REJECTED, 0 DEFERRED.**

Every factual correction below was **re-measured during the fold against the
built binaries** before it was written into the spec. Where round 0's number and
the fold's number differ, the fold's is the one in the spec and the disagreement
is named at the point it occurs.

Binaries used:

```
/scratch/code/shibboleth/descriptor-mnemonic/target/debug/md
/scratch/code/shibboleth/mnemonic-key/target/debug/mk
/scratch/code/shibboleth/mnemonic-secret/target/debug/ms
/scratch/code/shibboleth/_work/p3b/mnemonic-transaction/target/debug/mt
/scratch/code/shibboleth/mnemonic-engrave/target/debug/me
/scratch/code/shibboleth/mnemonic-toolkit/target/debug/mnemonic
```

---

## Disposition — all 23

| ID | Disposition | Where it landed / why |
| --- | --- | --- |
| **C-1** | **FOLDED** | Operator ruling written into new §6g: `me sysw pack --expect <kinds>`, opt-in, keyed on KINDS not counts; an incomplete chunk set of a named kind REFUSES; the no-`--expect` gap is stated as an accepted limitation, with `pack`-runs-the-producers recorded as rejected. §2's retracted "one thing that did not need fixing" sentence removed and replaced with the single-producer/composition distinction. D6 added to §5. |
| **C-2** | **FOLDED** | §10 rewritten from what the tools consume. Reproduced both refusals: `md encode` on a concrete descriptor exits 1 (*template contains no @i placeholders*); `mk encode` on a bare xpub file exits 64 (*expected BIP-380 origin notation*), and without a policy binding exits 64 (*at least one of --policy-id-stub or --from-md1 is required*). The new criterion is two-stage because `md` and `mk` are **not** independent producers, names `--from-md1-set FILE` as its one piece of new surface with P3 owning it, and adds a **negative** half — the same pipeline with one producer refusing must FAIL. |
| **C-3** | **FOLDED (retraction)** | §6e's terminal-gate generalisation is **withdrawn**. Reproduced both halves: on a pty `me sysw pack` refuses at exit 2 on a *raw-binary-in-a-scrollback* rationale, and `mt encode --quiet --in tx.hex` prints all six strings and exits **0**, warning on the *opposite* condition. The gate stays scoped to `me`'s binary container; `mt`'s print-to-terminal is recorded as deliberate so nobody "fixes" it. The disk-copy counter-argument is stated. |
| **C-4** | **FOLDED** | §6d makes the pre-parser ordering **normative**, extends it to the override's own parse, and rules that admitted material never reaches clap as a positional. Clap's echo reproduced live: `mt encode --qr deadbeefcafe` → `error: invalid value 'deadbeefcafe' for '[-]'`, exit 2. Adds the sentence that "never echo" is a property of *where the check sits*, not of the wording. |
| **I-1** | **FOLDED** | §2's `mk` cell → **none, ever**; `md`'s → **chunked output ONLY**. §8 bullet 1 struck. P3's gate pinned to a **chunking** policy — measured, a keyed 2-of-2 emits one header plus four `md1` lines and fails `me sysw pack` at exit 4 **on record 0**, while an unchunked policy cannot exercise the defect at all. |
| **I-2** | **FOLDED, with its evidence corrected** | P1's gate becomes *suite passes with the diff enumerated and each edit justified by a named §6 ruling*; 236 re-counted and confirmed. §6b **rules explicitly that `mt` gains `--out`** — and the fold found the reason the ruling is safe: `mt`'s refusal cites section 3b for "no `--out`", and **section 3b does not say it** (it rules which *stream*, not whether a file channel exists). §6e restates F-246 narrowly to its real title (printing *secret material* early), because the broad form was a real change to `mt`. **Evidence correction:** round 0's reproduction passes `--quiet` and then shows the report lines; with `--quiet` those lines are suppressed (82 stderr lines, no `TX`/`CUT`/`PREFIX`) and the report appears only **without** it. Finding right, command wrong; noted in §6e. |
| **I-3** | **FOLDED, and NARROWED by measurement** | §6d gains a per-verb channel table for all eight `ms` verbs; P2 reordered. **The fold narrowed the finding:** round 0 reads as though the private channel is missing across `ms`; measured, `-` is documented on **7 of 8** verbs (`decode`/`verify`/`inspect`/`derive` read stdin even when the positional is omitted). **`combine` is the sole exception** — and the worst one, being the recovery path. §2's `-` row corrected for `mk` too (all five artifact verbs, plus `--keys -`) and for `md` (`repair` alone). P2's content changed from "`--in`/`-` on all eight verbs" to "`-` on `combine`, `--in` on all eight". |
| **I-4** | **FOLDED** | New §6f carries the table, measured cell by cell across five CLIs. D26's cross-CLI parity quoted from `md repair --help`, and `ms repair`'s reasoned divergence (4, not 5) recorded. **Repair codes FROZEN**; only `mk`'s invalid-artifact 2 → 1 changes; the clap 2-vs-64 split is recorded and explicitly not resolved. Two `mnemonic` cells marked **not measured** rather than guessed, and filling them is a P0 gate. |
| **I-5** | **FOLDED** | §7 P0 gains a distribution sub-step. Both mechanisms verified in `me-cli/Cargo.toml`: crates.io deps (`md-codec = "0.42"`, `mk-codec = "0.4"`, `ms-codec = "0.7"`) and a **git-rev pin** for `mt-codec` whose own comment says the pin exists to keep publishing deferred. The precedent claim is re-characterised as the *opposite* of what D5 needs. `write_private` confirmed at `crates/me-cli/src/main.rs:844`, a private fn in a **binary** crate. Versions re-measured. **Mixed states declared ACCEPTABLE** rather than left to be discovered. |
| **I-6** | **FOLDED** | §4's principle now carries D3's qualifier — *secret and bearer* material never arrives on argv — with the watch-only reasoning quoted from `mt`'s shipped refusal and from `md`/`mk` stderr. The absolute form is named as what would delete `md`'s and `mk`'s positionals. |
| **I-7** | **FOLDED** | New §6a scopes the stdout rule to **`encode` only**, with a measured per-verb output table. `decode` is **explicitly out of scope and named so**; `verify`/`inspect` declared report verbs and exempt. Added to §9. |
| **I-8** | **FOLDED** | §6b **retracts "uniform"**. `--json` is unbroken (verified: grouping flags genuinely ignored in all three) but not uniform — table shows three artifact key names, three schema conventions, and `md` alone pretty-printing. The exclusion now stands on cost, not on a false premise, and is filed to a later cycle. |
| **I-9** | **FOLDED** | §1's `mt` cell corrected **exit 3 → exit 1**, reproduced, with the source confirmation that only `SUCCESS`/`FAILURE` exist and no third code appears anywhere in `mnemonic-transaction/crates`. Fed into §6f as *`mt` has no distinguishable refusal code*. |
| **I-10** | **FOLDED, with the golden count corrected** | §2a names `mnemonic-engrave` as an affected repo; P2 carries the driver migration — **18 call sites across 7 `.sh` files**, re-counted; P4's gate would otherwise be unsatisfiable when reached. **Count correction:** round 0 gives 12 files carrying `chunk-set-id:` including generated HTML under `design/journeys/out/`; that directory is **not tracked** (`git ls-files design/journeys/out` → nothing), so those are self-regenerating build products. **7 tracked files** under `design/journeys/` carry the line; 28 across all of `design/`. P3 owns the 7. |
| **I-11** | **FOLDED, and WIDENED by measurement** | §2a takes `mnemonic-gui` into scope and P3 owns the mirror regeneration. **The fold found the drift wider:** `const SEPARATORS` in **four** schema files (round 0 named two), `default_value: Some("5")` at **eight** sites in those four (round 0 counted seven, missing `md.rs:77`). Drift gate's self-declared `mnemonic`-only scope quoted. **Premise correction:** I-11 says all four CLIs carry `gui-schema`; measured, **`mt` does not** — the published-schema set is `md`/`mk`/`ms`/`mnemonic`, i.e. exactly the set §6 rewrites minus the one tool that already behaves. |
| **M-1** | **FOLDED** | §6c gains a measured stderr table: `md` and `mk` emit **one `note:` line** and have no card; `ms` has a 4-line card (`--no-engraving-card` cuts it to 1); `mt` has a full report plus legend. **D4 therefore requires inventing a card for `md` and `mk`, and P3 owns its contents.** The consequence M-1 asked for is stated: after D4, `--no-engraving-card` or `2>/dev/null` yields no grouped form anywhere. |
| **M-2** | **FOLDED, cost restated** | §6c's table: `md`, `mk` **and** `ms` all round-trip their own hyphen-grouped output at exit 0 (each fed **its own** `--separator hyphen` output — an ad-hoc regrouping does not decode, which is why the first attempt looked like a negative). `comma` is offered by all of them and goes with `hyphen`. Cost restated as **two options across three measured CLIs and one unmeasured**, not one option in three. |
| **M-3** | **FOLDED** | §6c records the F-245 interaction as a plan reconciliation item, with the reproduction (a record with one trailing space packs at exit 0, no warning). |
| **M-4** | **REJECTED** | The argument was sound when written and **the fact changed under it**. `me sysw pack` now ships `--allow-argv-secret` as the override for `Class::is_argv_forbidden()` = `is_secret() \|\| is_bearer()` — a predicate whose union is deliberate, under a ruling dated 2026-08-26 asking for *"uniform behavior with secret bearing between ms1 and passwords and mt1"*, and whose own comment says the two differ in kind but are the same problem at argv. Renaming is therefore a rename of **shipped surface against a ruling that says the union is the point**, not a naming choice. Proof: `me sysw pack --help` lists `--allow-argv-secret` with that rationale; `git -C /scratch/code/shibboleth/mnemonic-engrave diff crates/me-cli/src/sysw/record.rs` shows `is_argv_forbidden`. Declined in §6d, with the reason, so a later reader does not re-file it. |
| **M-5** | **FOLDED** | §6b: *"stdout is used when `--out` is not given"*, with the ambiguity named — the input channel has no bearing on where output goes. |
| **M-6** | **FOLDED** | §8 bullet 2 **struck**. Reproduced: `--from-md1` is documented *Repeatable* and repeating it across a real 4-chunk `md1` set exits 0; the recorded failure was a **space-joined single value**, a usage error, not a capability gap. |
| **N-1** | **FOLDED** | §7 P0's distribution list gains it. Verified: `_work/p3b/mnemonic-transaction` is a **git worktree** whose common git dir resolves to `/scratch/code/shibboleth/mnemonic-transaction/.git`, and a separate checkout exists there. Two relative paths for one repo, one of them transient — a further argument against `path =`. |
| **N-2** | **FOLDED** | §8 gains a record-hygiene item. Verified: F-246, F-250, F-251, F-252, F-253 carry **no `CLOSED` marker** in `design/FOLLOWUPS.md` while F-244 does, and the behaviour is present in the binaries. Correct about the code, stale about the record; a sweep the plan does before scheduling against those numbers. |

---

## Found during the fold, and NOT in round 0

Five things. The first two change what P0 builds.

**1. The operator's C-1 ruling names two `Class` variants that do not exist.**
The ruling is phrased as keying `--expect` on classes `me` already computes,
naming `Class::Md` and `Class::Mk`. `me`'s enum
(`crates/me-cli/src/sysw/record.rs:44`) has a **single `MdMk` variant** covering
both, so `--expect descriptor,cosigner` keyed on `Class` alone **cannot tell a
descriptor card from a cosigner card**. The discriminant exists one level down:
`mdmk_unconfirmed` groups by `(hrp, chunk_set_id)` through
`seal::record::chunk_key` and switches on the HRP character (`'d'` → `md_codec`,
`'k'` → `mk_codec`). The decision is unaffected; the mechanism sentence is
corrected in §6g, and `mdmk_unconfirmed` is named as **already computing the
incomplete-set predicate** the ruling's third bullet needs — so `--expect`
escalates an existing walk rather than adding a second one to drift.

**2. §4's stated principle was a block quote with no source.** The sentence the
previous revision attributed to `SPEC_mt_v0_1` section 3b — ruling grouping
opt-in and the canonical artifact ungrouped — **appears nowhere in that
document**. Its nearest ancestor is a *proposal* in
`design/agent-reports/R6-lens-implementability.md:489`, which recommended adding
such a ruling and said either direction would do so long as one was chosen. So
the spec's whole principle was a paraphrase of a review agent's suggestion,
presented as a verbatim quotation of a normative spec — and round 0 verified the
principle without checking its provenance. §4 now carries the **real** sentence
from section 3b (*"stdout carries the artifact, stderr carries everything the
human must see"*) and states that the ungrouped-canonical rule is **this spec's
D4, not an inherited one**. This also resolved I-2's `--out` tension: section 3b
rules which stream, not whether a file channel exists.

**3. `me`'s shipped remedy text forward-references a channel that does not
exist.** `me sysw pack`'s widened argv refusal advises `ms encode --in seed.txt`
as the private channel for a secret class. Measured: `ms encode` has no `--in`
and exits **64**, `unexpected argument '--in' found`. **P2 owes that text, not
just the feature** — until P2 lands, `me` is advising a command that fails. In
§6h.

**4. The affected surface is a five-CLI system in three of the five decisions,
not one.** Round 0's I-4 brings `mnemonic` in for exit codes only. Measured,
`mnemonic bundle` also carries `--group-size` defaulting to **5**, `--separator`
accepting **space, hyphen, comma**, and `--no-engraving-card` — D4 and the
separator rule both reach it. It also ships **`--passphrase-stdin`**, which
together with `ms derive`'s is an existing in-constellation precedent for D3's
private channel that the spec previously cited only from `mt`. New §2a; title
updated to five CLIs.

**5. D3 is not merely decided — it has already shipped in `me`, on a branch
newer than the one this fold sits on.** The main checkout's working tree carries
`Class::is_argv_forbidden()` (`is_secret() || is_bearer()`),
`--allow-argv-secret`, and a remedy block implementing the operator's
teach-the-HOW instruction with per-shell purge commands. **A plan reading only
this branch would conclude, wrongly, that `me`'s gate is still bearer-only.**
Recorded in §5 so P0 **extracts** that code rather than re-deriving it.

---

## Also folded: an operator instruction that arrived mid-fold

**"A message that tells the operator to clean up must tell them HOW, with the
exact command, at the step doing the telling."** The phrasing shipped in `me`
today says WHAT and not HOW. New §6h makes it a spec rule, built on the
reference implementation and on three facts verified during the fold:

- **zsh's history builtin with `-d` does NOT delete.** On zsh 5.9.2 `-d` is a
  **display** flag that prints timestamps, and the builtin rejects the
  invocation outright. Advising it would report success while purging nothing.
- **Match on the COMMAND NAME, never on the secret** — anchoring the pattern on
  the material types it into history a second time. `sed -i '/me sysw pack/d'
  "$HISTFILE"` verified working.
- **fish needs a different command AND a different file.** Verified on fish
  4.8.1: `history delete [--exact | --prefix | --contains]` exists, its history
  lives at `$XDG_DATA_HOME/fish/fish_history` rather than `$HISTFILE`, and that
  file is a two-line-per-entry `- cmd:` / `when:` format — so a stream edit would
  strip the command and leave an orphaned timestamp. A generic paragraph is
  wrong for at least one shell.
- The **in-memory-history resurrection** half is marked **NOT YET VERIFIED** and
  no command is stated for it. P0 owes the measurement before it writes the
  sentence.

A related **taxonomy** is recorded in §6i as a taxonomy only, with **no
behaviour derived from it**: "refuse" is used for two different reasons here.
Measured in `mt`, `Refusal::new(` is constructed at **56** sites naming **12**
distinct `§8` sections, of which exactly **two** are environment posture (argv,
world-readable stdout) and ten are artifact correctness. An air-gap changes who
can see your machine; it does not make an unsigned transaction worth engraving.

A **declared-posture mechanism** for argv was proposed during the fold and
**declined by the operator** — *"We don't have to split hairs so finely. Refuse
argv with override is fine."* Recorded in §5 and §9 as rejected so it is not
revisited. D3 is unchanged from its original form.

---

## Gate output

Both gates are now **CLEAN on this file**, which is a change: the previous
revision reported 7 structural defects and 2 malformed table rows, all false
positives, and **documented them in §8a rather than removing them**. A permanent
documented FAIL teaches a reader to skim the gate, which is how a real finding
hides.

```
$ ./scripts/spec-structure-check.sh design/SPEC_constellation_cli_uniformity.md
  sections: 21 ; cross-refs checked: 15
  STRUCTURE OK

$ ./scripts/plan-table-check.sh design/SPEC_constellation_cli_uniformity.md
─── table rows checked: 58 ; malformed: 0
```

Before: `7 STRUCTURAL DEFECT(S)` and `malformed: 2`.

- The five duplicate-section reports are gone because section 6's subsections
  are now **lettered**, which the gate keys distinctly — the form its own
  comments show was intended. Real duplicates in section 6 are visible again.
  `SPEC_engrave_transaction.md` still trips the same class **21** times
  (re-measured) and is a candidate for the same fix.
- The two table-cell reports are gone because the two affected phase cells now
  **describe the pipeline in words instead of drawing it**, so no cell contains
  an escaped pipe and neither gate needs an exception. `grep -c '\\|'` on the
  file returns 0.
- §8a is rewritten to record all of this, and no longer documents a permanent
  FAIL.

**Two defects the fold introduced and then caught with the gate, recorded
because they are the class this project keeps hitting.** (a) Quoting
`md repair --help` verbatim dragged a **foreign section sigil** into the file;
the gate resolved it happily against *this* document's §1, producing a
false-clean cross-reference — the exact shape the gate's own comments warn
about. The clause is now elided and described. (b) The §2 table kept a stale
"stdin idiom" row that **contradicted** the `-` row corrected under I-3 in the
same table. Both were found by re-reading the rendered tables after the gates
went green, not by the gates.

## Not asserted

This fold does **not** claim the spec is now GREEN — that is round 1's call. It
claims that all 23 findings are dispositioned, that every factual correction was
re-measured rather than transcribed, and that the two machine gates pass.
