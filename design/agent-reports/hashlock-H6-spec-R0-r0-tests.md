# REPORT — H6 spec R0 round 0: tests + citations (sonnet, independent)

Reviewing `design/SPEC_hashlock_H6_preimage_plates.md` at engrave master `a0f832d0`
(current HEAD `f6c696d3` carries the file unchanged — `git diff a0f832d0 HEAD --
design/SPEC_hashlock_H6_preimage_plates.md` is empty). Revisions used: fork
`fb0dd04` (`/scratch/code/shibboleth/seedhammer`), engrave `a0f832d0`
(`crates/me-cli`), mnemonic-secret `504ff46` (`crates/ms-cli`, `crates/ms-codec`).
All three repos read-only; all execution in a detached worktree,
`/scratch/code/shibboleth/.tmp/h6-lens-tests` (`git worktree add --detach ...
fb0dd04`), Go `/scratch/code/shibboleth/.toolchain/go/bin/go`, removed as this
report's final housekeeping step. No sub-agents used. No `.jsonl` read.

**Scope note.** Per the settled-facts list, I reviewed the SPEC against the fork,
engrave and ms trees — not the operator's brainstorm rulings themselves, which
are out of scope for this round.

---

## 1. Citation table

**Mechanical pass.** All `` `path:line` `` and `` `path:line-line` `` citations
were extracted by regex (116 unique forms after dedup) and checked against the
actual file at the correct repo/revision with a throwaway script
(`/tmp/check_citations.sh`): does the file exist there, and does every cited
line number (or the top of a comma list) fall inside the file's actual line
count. **Result: 116/116 resolve — every citation names a file that exists at
its revision with a line number in range.** (This does not by itself prove the
cited lines say what the spec claims — see the content spot-checks below, which
is where the real risk lives.)

**Content spot-checks.** Beyond the range check, I read the cited lines for a
representative, risk-weighted sample spanning every major area of the spec and
compared the actual text/behavior against the spec's claim. All of the
following were **confirmed true, verbatim or in substance**:

| claim | cite | verdict |
| --- | --- | --- |
| phrase dropped on return; HOLD assigns digest, notes it | `gui/composer_hashlock.go:17-20,43-86,69,70` | TRUE — read in full; line 69 assigns `st.list.Paths[idx].Hash`, line 70 is `composerNotePhraseDigest` |
| `IsPreimage`'s header and its "refusal costs a re-encode; a wrong cut exposes a spend secret" trade | `codex32/mspayload.go:63-93`, `:78-92`, `:94-101` | TRUE — read in full; line ranges match the doc comment and function exactly |
| `Split()` returns `(id, threshold, idx)` for the proposed `IsPreimagePlate` | `codex32/codex32.go:394-401` | TRUE |
| reserved-prefix rule + `key:`/`hash:`/`now:` prefixes | `sysw/record.go:14-21`, `sysw/composer_records.go:24-31` | TRUE — record.go states the general rule, composer_records.go defines the three prefixes; spec combines them correctly |
| `IsSecret()` — `Mnemonic⎮Codex32Secret⎮Passphrase` | `sysw/record.go:60-66` | TRUE, exact function body |
| `unhexLower`: even-length lowercase hex | `sysw/composer_records.go:82-97` | TRUE, exact function body |
| `now:` idiom cuts on the FIRST comma | `sysw/composer_records.go:136-160` | TRUE — `strings.Cut(text, ",")`, Go's first-occurrence cut |
| `ClassKey/ClassHash/ClassNow` admitted **only** at `progWalletPolicy` | `gui/sysw_admit.go:32,64-73` | TRUE — read the whole `admitted` map; the three classes appear in no other program's row |
| `decide_sealing` seals iff some record class is secret; `--no-passphrase` at :244 | `crates/me-cli/src/main.rs:2353-2410`, `:244` | TRUE, exact logic and flag |
| `preimage_plate` does not consult the id (excludes id/kind mismatch, else tests `unshared && len==33 && data[0]==0x03`) | `crates/me-cli/src/seal/record.rs:287-320` | TRUE — matches D9's characterization exactly, including the `id_kind_mismatch` exclusion |
| `RecordError::PreimagePlate`'s shipped text (the em dash is pre-existing, not new) | `crates/me-cli/src/seal/record.rs:130-136` | TRUE — verbatim match; confirms §8.1.1's inherited em dash predates H6 |
| `ValidatePhrase` order: empty, ASCII, ms1-shape, cap, 64-hex | `hashlock/hashlock.go:92-111` | TRUE, exact order |
| `validate_phrase` same order (Rust) | `crates/ms-cli/src/hashlock_phrase.rs:118` | TRUE — the doc comment states the identical order |
| `Salt/Iterations/PreimageLen` = `ms-hashlock-v1`/100000/32 | `hashlock/hashlock.go:21,24,27` | TRUE |
| F-132 (plate half of the follow-up) | `design/FOLLOWUPS.md:4298` | TRUE, title matches |
| F-483 (phrase in `kbd.Fragment`) | `design/FOLLOWUPS.md:16003` | TRUE, matches verbatim |
| corpus commit `c05074f1…`, sha256 `5b3960ca…b312`, 47 rows | `sysw/testdata/record_class_vectors.provenance.json` | TRUE — **independently recomputed** `sha256sum` of the vendored file at `fb0dd04` reproduces the pin byte for byte; `json.load(...)` on the engrave-side corpus counts exactly 47 rows; commit `c05074f1` exists in the engrave repo with the message "...(47 rows)" |
| `!tinygo` composition-state seam is a real, existing mechanism | `gui/composer_state_hook.go` | TRUE — file exists, `//go:build !tinygo` |

No false citation was found anywhere in this sample. Two numeric errors and one
ASCII-rule violation were found by **measurement**, not by mis-citation — see
§2.

---

## 2. Measured numbers vs. the spec's

All measured with throwaway probes in the detached worktree (`h6probe`,
`h6probe2`, `h6probe3` under `.../gui/`, plus a `gui` package test file for the
modal-fit numbers), calling the fork's OWN functions (`qr.Encode` at `qr.L`,
`backup.CharsPerLine`/`LinesPerPlate`, `assertModalBodyFits`/`errorScreenBody`/
`confirmWarningBody`), not reimplementations.

### 2.1 QR module counts / version thresholds (§7.1) — EXACT MATCH

```
hardened,100-char (worst case)           bytes=194 modules(dim)=53 version=9
sha256,100-char                          bytes=135 modules(dim)=45 version=7
hardened,28-char anchor phrase           bytes=122 modules(dim)=41 version=6
sha256,1-char                            bytes=36  modules(dim)=29 version=3
ECC-L thresholds: ≥1→21(v1) ≥18→25 ≥33→29 ≥54→33 ≥79→37(v5) ≥107→41(v6)
                  ≥135→45(v7) ≥155→49(v8) ≥193→53(v9) ≥231→57(v10)
```
Every one of these numbers is what §7.1's table and threshold list state,
digit for digit. Method line length measured at 73 characters (hardened),
matching §6.5/§8.6's "73 characters" claim exactly.

### 2.2 Alignment-pattern centres (§7.2) — EXACT MATCH, empirically derived

Rather than trust the ISO/IEC 18004 standard from memory, I scanned the actual
encoder's `qr.Code.Black(x,y)` bitmap for 5×5 alignment rings at every
candidate centre, for real encoded payloads that land at v6/v7/v8/v9:

```
v6 (dim=41): 1 ring at [34 34]
v7 (dim=45): 6 rings at [[6 22] [22 6] [22 22] [22 38] [38 22] [38 38]]
v8 (dim=49): 6 rings at [[6 24] [24 6] [24 24] [24 42] [42 24] [42 42]]
v9 (dim=53): 6 rings at [[6 26] [26 6] [26 26] [26 46] [26 46] [46 46]]
```
This is a byte-for-byte match against §7.2's table, derived from the actual
fork encoder rather than from a textbook — exactly what the spec claims §11.3's
future test will do.

### 2.3 Plate geometry (§6.5) — chars/line and lines/plate EXACT; one **advance** value WRONG

Called `backup.CharsPerLine`/`backup.LinesPerPlate` with `sh2.Params()` and
`constant.Font` directly:

```
Millimeter=6400 StrokeWidth=1920      (matches §6.5's opening line, exact)
W advance=600 Metrics.Ascent=800 Metrics.Height=900   (matches §6.3's citation, exact)

rung    chars/line  lines/plate  advance(measured)   spec's advance
6.0 mm  19          13           4.0000 mm           4.000 mm   MATCH
5.0 mm  23          15           3.3333 mm            3.435 mm   **MISMATCH**
4.4 mm  26          17           2.9333 mm            2.933 mm   MATCH
3.8 mm  31          20           2.5333 mm            2.533 mm   MATCH
3.4 mm  34          23           2.2666 mm            2.267 mm   MATCH
3.0 mm  39          26           2.0000 mm            2.000 mm   MATCH
```
`chars/line` and `lines/plate` reproduce EXACTLY at all six rungs. The
**advance** column reproduces exactly at five of six rungs and fails at 5.0 mm:
the spec (and its companion author's report, `hashlock-H6-spec-author-report.md`
§1.3, same table, same wrong number) states **3.435 mm**; the fork's own
`fixedCharWidth` formula (`W_advance × fontSize_units / Metrics.Height`,
integer-truncated, exactly as `CharsPerLine` computes it) gives **3.3333 mm**
(21,333 units), which is also the only value consistent with the otherwise
perfectly linear pattern every other rung satisfies (`advance = 2.0 mm ×
rung/3.0`; 5.0×2/3 = 3.3333, not 3.435). See §4, I-1.

**This does not corrupt any downstream conclusion.** The worst-case fit table's
"phrase, no QR" row at 5.0 mm states a total of 80.0 mm; 80.0 / 3.3333 = 24.0
exactly (a whole line count), while 80.0 / 3.435 = 23.28 (not a whole line
count) — so the downstream arithmetic was actually done with the CORRECT
3.3333 mm value; only the displayed cell in the primitives table is wrong, in
both documents that carry it.

Band-holds character counts (§6.3's "32/25/16 chars" claim) were independently
recomputed from `409,600 / advance(units)` and confirmed exact at all three
cited rungs (32 at 3.0 mm, 25 at 3.8 mm [409600/16213=25.26→25], 16 at 6.0 mm).

### 2.4 Modal body fits (§8) — 8 of 9 EXACT reproductions; 1 fails to reproduce

Ran the fork's own `assertModalBodyFits` against `errorScreenBody`/
`confirmWarningBody` with the spec's literal §8 strings:

| body | renderer | spec's drawn/headroom | measured |
| --- | --- | --- | --- |
| §8.3 not-on-any-path notice | showError | 107 / 455 | **107 / 455 — EXACT** |
| §8.4 abort, preimage arm alone | showError | 288 / 244 | 291 / 244 (headroom exact; drawn off by 3 — my reconstruction guessed `cardIdx`/`cardTotal`/`label`, which the spec does not pin; not a spec defect) |
| §8.4 abort, LONGEST (preimage+seed) | showError | 411 / 121 | 414 / 121 (headroom exact, same note) |
| §9 ms1-in-free-text, errorScreenBody | showError | 184 / 378 | **184 / 378 — EXACT** |
| §9 ms1-in-free-text, as shipped | confirm | 204 / 302 | **204 / 302 — EXACT** |
| §8.5 QR toggle warning | confirm | 126 / 378 | **126 / 378 — EXACT** |
| §10.1 §8h plain form | showError | 153 / 378 | **153 / 378 — EXACT** |
| §10.1 §8h phrase form | showError | 159 / 378 | **159 / 378 — EXACT** |
| §8.1.2 id-is-not-hash refusal | showError | 165 / 397 | **FAILS — see below** |

**§8.1.2 does not reproduce, and the reason is a real defect.** Feeding the
spec's own literal text (which contains `SPEC_ms_hashlock §1 rule 2` — a
Unicode section-sign, "§", U+00A7) through `errorScreenBody` produces a
**near-blank frame**: `the modal drew only 5004 ink pixels (floor 6000) --
this frame is near blank`. Substituting ASCII "section 1" for "§1" (nothing
else changed) renders cleanly: 242 drawn / 320 headroom. This isolates the "§"
character as the cause — the device font/renderer does not carry that glyph,
and something in the render path treats the resulting frame as content-free
rather than silently dropping the one character. §8's own opening line is
**"ASCII only. Every DEVICE body below was measured..."**, and this is the one
body that violates it while also claiming (falsely, as measured) to have been
run through the device measurement harness "so it can be surfaced later
without a re-measure." See §4, I-2.

(The other five §8 items not independently re-run — §8.1.1's shipped/inherited
text, §8.2.1-4 host warnings, and §10.2 — are host-only stderr text with "no
panel budget" per the spec's own rule, so `assertModalBodyFits` does not apply
to them; I confirmed §8.1.1's text is the genuinely pre-existing shipped
string, em dash included, in §1's citation check above.)

### 2.5 Corpus provenance (§3.1) — EXACT, machine-verified

`sha256sum` of `crates/me-cli/testdata/record_class_vectors.json` at engrave
`a0f832d0` = `5b3960cad7f924f6f1e7f19ef49599814733cee4874d0f5eb48c28af4cd8b312`,
matching the spec's citation and the fork's stored provenance pin
(`sysw/testdata/record_class_vectors.provenance.json`) byte for byte; row count
(`json.load` → `len() == 47`) matches both the spec's "47 rows" and the pin's
`"vectors": 47`.

---

## 3. Per-test table (§11)

For every test bullet I checked (a) whether the cited hooks/functions the test
would need actually exist (or are a direct, buildable extension of an existing
pattern — this is a pre-implementation spec, so most targets don't exist yet)
and (b) whether the stated mutation is specific enough to distinguish
pass/fail. Representative rows, weighted toward what I could verify against
real code:

| test | can it fail on what it names? | how I know |
| --- | --- | --- |
| §11.1 `phrase:` classification, cut-on-FIRST-comma mutation | YES | `now:` already uses `strings.Cut` (first-occurrence) as its precedent (`sysw/composer_records.go:146`, read in full); a `phrase:` parser built the same way and mutated to `strings.LastIndex`-style cutting would visibly misparse any comma-bearing phrase row — a real, checkable divergence |
| §11.1 id rule: kind-0x03/id≠hash refused, MUTATION drop id test from `preimage_plate_admissible` | YES — this is the funds-relevant row, correctly flagged as such | `Split()` (confirmed exact signature) gives the id for free; `IsPreimage`/`preimage_plate`'s existing "does not consult the id" behavior is confirmed true today (§1), so a predicate that narrows by id and is then mutated to drop that check reproduces exactly the collision the spec's own header comment already documents ("a 0x03 single under any other id... not a seed either way") |
| §11.1 `decide_sealing`, MUTATION make the class non-secret | YES | `decide_sealing`'s actual logic (confirmed §1) is a direct function of `classify(r).is_secret()`; flipping `IsSecret()` for the new classes is a one-line mutation with an observable "NOT SEALED" branch already coded and printed today |
| §11.2 QR text corpus, MUTATION trailing newline / line-order swap | YES | §8.6's text is a plain three-line join with **no trailing newline** — a mutation adding one, or swapping line order, is mechanically distinguishable from the pinned corpus rows by a straight byte compare |
| §11.3 alignment table derived-vs-hardcoded | YES | directly reproduced in §2.2 above using the real encoder; a wrong centre is caught by literally the same bitmap-scan technique the test proposes |
| **§11.3 constant-time: "two payloads of the same version emit the SAME move count, and `findPath` returns no error", for v6..v9** | **NO, not as scoped** | see §4, I-3: `constantTimeQRModules` (`engrave/engrave.go:349`, read in full) only handles dims 21/25/29/33/37 and returns **0** for anything else; `ConstantQR`'s own capacity check (`len(modules) > nmod`, read in full at `:483-486`) means **every** v6-v9 encode errors with `"too many dims %d QR modules..."` today, regardless of alignment-table or bound fixes, because `nmod` is 0. This test cannot pass until an UNSCOPED task (a fuzzing campaign per new version, on the pattern already documented for v5's own "18.5M executions, 32 min" derivation) is done — and §7.2's four "normative consequences" never name that task |
| §11.4 fit gate: worst-case body refuses above 3.0 mm, MUTATION add one char to method line | PARTIALLY — mechanism sound, numbers not independently re-derivable pre-implementation | `backup.Hashlock` and its layout function do not exist yet, so I could not render the actual worst-case body; I confirmed the PRIMITIVES it depends on (chars/line, lines/plate, `MaxTitleLen`, the 409,600-unit band cap) are correct except the one advance value in §2.3, which — as shown — is NOT the value the downstream fit numbers actually used. The auto-fit-and-refuse pattern (`EngraveText`'s refusal, `backup/backup.go:388-400`) is a real, existing mechanism the new type can follow |
| §11.5 `Which hash?` by LABEL, MUTATION reintroduce index-keyed default | YES | `composerHashRows`/`composerHashEdit`'s label-keyed dispatch and PANIC-on-default (cited, H2 §5 r2 C-4 precedent) is exactly the mechanism a label-vs-index regression would break; this is a proven pattern already shipped for the existing two row classes |
| §11.5 `hashlockHeld` nil-map / scrub via `!tinygo` seam | YES | `gui/composer_state_hook.go` exists with `//go:build !tinygo` (confirmed); this is a real, already-used mechanism (H5's own walk uses the same seam per the fork's `fb0dd04` merge message), not an invented hook |
| §11.6 whole gates (nextest, gui shards, gofmt, vet, firmware size) | Standard, executable; not independently re-run here (would require a full build) | tooling (`cargo nextest`, `scripts/gui-shard-test.sh`) is the project's own established gate, unrelated to H6-specific content |
| §11.7 walk arm, asserts the SCREEN not a hook | YES, mechanism sound | `cmd/emu/walk_hashlock_phrase.js` exists (confirmed non-empty, 481 lines at `fb0dd04`) and H5 §4.1's screen-only doctrine is the same discipline the existing file already follows |

---

## 4. Findings

### I-1 — The plate-geometry "advance" value at the 5.0 mm rung is wrong in both the spec and its companion report

`design/SPEC_hashlock_H6_preimage_plates.md` §6.5's rung table states **3.435
mm** for the 5.0 mm rung's per-character advance; `design/agent-reports/
hashlock-H6-spec-author-report.md` §1.3 carries the identical wrong value.
Measured directly against the fork's own `backup.CharsPerLine`/
`LinesPerPlate` and the same `fixedCharWidth` formula those functions use
(`W advance(600) × fontSize_units / Metrics.Height(900)`, integer-truncated),
the correct value is **3.3333 mm** (21,333 units) — the only value consistent
with the linear pattern (`advance = 2.0 mm × rung/3.0`) every other one of the
six rungs satisfies exactly, and the only value consistent with the
downstream fit table's own "phrase, no QR, 5.0 mm = 80.0 mm" total (80.0 /
3.3333 = 24.0 exactly; 80.0 / 3.435 = 23.28, not a whole line count) — i.e.
the wrong number was NOT what produced any of the spec's accept/refuse
verdicts, which used the right one. **Impact: bounded to the display cell**
(no downstream geometric conclusion changes), but it directly falsifies the
spec's central methodology claim ("every number in this spec is a
measurement... taken on the scratch gate tree") for this one cell, in both
of the documents that repeat it. Fix: correct the two table cells to 3.3333
mm (or "3.333 mm" to match the other rows' 3-decimal style).

### I-2 — §8.1.2's claimed device-rendering measurement does not reproduce; the text violates §8's own ASCII-only rule

§8.1.2's literal body contains a Unicode section sign ("§" in "SPEC_ms_hashlock
§1 rule 2"). Run through the fork's actual `errorScreenBody`/
`assertModalBodyFits` harness (the same harness the spec cites as having
produced "165 characters drawn, headroom 397"), this body renders as a
**near-blank frame** (5,004 ink pixels, under the 6,000-pixel non-blank
floor) rather than drawing 165 characters — confirmed by isolating the cause
(substituting ASCII "section 1" for "§1" renders cleanly, 242 drawn / 320
headroom). §8's own governing rule is "ASCII only," stated immediately before
the sentence that describes how every device body below was measured; §8.1.2
is the one body in that section that violates it, and the specific measured
numbers it claims (165/397) are therefore not obtainable from the text as
printed. **Impact is currently contained** — the spec itself says "its
device-side twin is not a modal" today, so nothing draws this text on-device
yet — but the spec explicitly measured it "so it can be surfaced later
without a re-measure," and that claim is false: a re-measure would be
required, and would fail as shown. Fix: either drop the "§1" citation from
the user-facing string (say "SPEC_ms_hashlock rule 2" or "section 1"), or
drop the "measured... so it can be surfaced later" claim until it is
re-measured against an ASCII-clean string.

### I-3 — The v6-v9 QR raise is unimplementable as scoped: `constantTimeQRModules` is never listed as needing new entries, but the raise cannot function without them

§7.2's four "normative consequences" (the alignment-pattern change, `v7-v9`
needing the six-ring layout, `ConstantQR`'s bound becoming `dim > 53`, and "the
constant-time argument is re-earned... via a test") never state that
`constantTimeQRModules` (`engrave/engrave.go:349`) must gain new entries for
dims 41/45/49/53 (v6-v9). It is cited once, in the same breath as
`constantTimeStartEnd` and `findPath`, as a function that merely "takes
`dim`." Read in full, `constantTimeQRModules` today handles only
21/25/29/33/37 and returns **0** for any other `dim`; `ConstantQR`'s own
capacity check (`nmod := constantTimeQRModules(dim)`, then
`if len(modules) > nmod { return nil, fmt.Errorf("too many dims...") }`,
read in full) means that with `nmod=0`, **every** v6-v9 encode will error
immediately with `"too many dims %d QR modules for constant time engraving"`
— even after the alignment table (§7.2 item 2) and the size bound (§7.2 item
3) are both correctly implemented. So §11.3's own test bullet ("for each of
v6..v9, two different payloads of the same version emit the SAME move count,
and `findPath` returns no error") **cannot pass today and is not made
passable by anything §7.2 lists as a deliverable** — it requires populating
`constantTimeQRModules` for four new dims, and the existing v5 entry's own
comment records that this kind of number is derived by a **fuzzing campaign**
("18.5M executions, 32 min... converged with 24.9 min of quiet" for v5
alone) — a materially sized, unscoped, unbudgeted piece of work that an
implementer following §7.2 literally would not know to do until the build
already fails. **This is the single most consequential finding in this
review**: it is exactly the kind of thing the whole "encoder raise" exists to
deliver (a phrase's QR must engrave in constant time to avoid leaking it
through toolpath timing), and the plan is silently incomplete on it. Fix:
add a fifth normative bullet to §7.2 naming the `constantTimeQRModules`
extension explicitly, and either budget the four fuzzing runs as their own
task or state the acceptance bound each one needs to clear (module count,
not moves, so the numbers are on a different axis than what's already
tabulated).

### M-1 — The §8.4 abort-arm reconstruction could not be pinned exactly (not a spec defect)

My own probe's "alone"/"longest" abort-arm bodies drew 291/414 characters
against the spec's claimed 288/411 (headroom identical at 244/121 in both
cases). The 3-character gap is consistent across both variants and across two
different guessed `cardIdx`/`cardTotal` pairs, which points to a small,
constant difference in how I reconstructed `bundleAbortWarningText`'s
unspecified caller context (`cardIdx`, `cardTotal`, `label` are not pinned by
the spec text), not a defect in the spec's own claim. Recorded as a residual
gap in what this round could independently confirm, not as a finding against
the spec.

### N-1 — `constantTimeQRModules`'s "return a low number to force error" comment is exactly what fires here, which is worth noting positively

The function's own comment ("Not supported, return a low number to force
error") shows the 0-return for I-3 is a deliberate fail-safe, not an
oversight in the EXISTING code — the gap is only that the H6 spec doesn't
say it must be un-triggered for v6-v9. Noted so a fold does not read I-3 as
"the fork's QR code is buggy."

---

## Closing counts

**3 Important (I-1, I-2, I-3), 1 Minor (M-1), 1 Nit (N-1), 0 Critical.**

I-3 is the one I would push on hardest before this spec leaves R0: it is a
missing deliverable on the funds/security-relevant half of the stage (the
constant-time property exists specifically so a hashlock phrase's QR does not
leak through engrave-toolpath timing), and it is invisible to a citation
check because the cited line (`engrave/engrave.go:349`) is real and the
function really does take `dim` — the citation is true, the omission is what
it doesn't ask that function to also do.
