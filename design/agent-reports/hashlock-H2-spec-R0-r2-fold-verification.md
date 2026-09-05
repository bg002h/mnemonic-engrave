# R0 round 2 — fold verification on `design/SPEC_hashlock_H2_device.md`

Reviewer: independent (sonnet, narrowly scoped). Fold under test: `c06a760`,
over `60a86f6`, responding to `hashlock-H2-spec-R0-r1-fold-verification.md`
(`040e85f`: 5/5 C and 10/11 I fixed, I-5 partial, two new Importants NF-A and
NF-B). Read-only on all three repos (`mnemonic-engrave`; `seedhammer` at
`c4a64fc`; `mnemonic-secret` at `cd0a60f`, confirmed by `git rev-parse HEAD` —
the checkout is currently at `e96676c` but the cited file's content at
`cd0a60f` was read via `git show cd0a60f:<path>`, not the working tree, so the
citation is against the pinned commit as required); nothing committed or
modified in any of them; no worktrees; no sub-agents; no `.jsonl` read. One
throwaway Go program (two files) was compiled and run under
`/scratch/code/shibboleth/.tmp/nf-a-check/` (outside all three repos) to
replicate `seal.NormalisePassphrase` and `format::strip_display_separators`
against the exact corpus strings.

**ONE QUESTION: does the r2 fold fix NF-A, NF-B and the I-5 partial — with
every new claim about the fork true — and introduce no contradiction of its
own?**
**Answer: NF-A, NF-B and I-5 are all genuinely fixed and every new citation
checked is true. One new Important surfaced: a leftover in §3, outside the
fold's own edited lines, still names the confirm-modal action `CONTINUE` —
directly contradicting §4.5's `HOLD`, which this same fold introduced, and
falsifying the commit message's unqualified "M-4 FIXED."** Gate: **NOT GREEN.**

---

## Verify (executed, per brief item)

### 1. NF-A

Corpus strings, read directly from ms `cd0a60f`
(`crates/ms-codec/tests/vectors/hashlock-v0.8.json`, parsed with `git show
cd0a60f:... | python3 -m json.tool`-style extraction): confirmed the exact
three strings are corpus `derivation` rows: `"Correct Horse Battery Staple"`,
`"  a  b "`, `"correct-horse,battery staple"`.

`seal.NormalisePassphrase` read at `seedhammer` `c4a64fc`, `seal/open.go:76-78`:

```go
func NormalisePassphrase(s string) string {
	return strings.ToLower(strings.Join(strings.Fields(s), " "))
}
```

Throwaway Go run (`/scratch/code/shibboleth/.tmp/nf-a-check/main.go`):

```
input="correct horse battery staple" normalised="correct horse battery staple" fixed_point=true
input="Correct Horse Battery Staple" normalised="correct horse battery staple" fixed_point=false
input="  a  b "                      normalised="a b"                          fixed_point=false
input="correct-horse,battery staple" normalised="correct-horse,battery staple" fixed_point=true
```

This matches §2 and §7.1 exactly: §2 says `NormalisePassphrase` "changes
exactly two corpus rows: `Correct Horse Battery Staple` (case) and `  a  b `
... those two are the witnesses against it. The third row,
`correct-horse,battery staple`, is a fixed point of that normaliser." Confirmed
true.

`format::strip_display_separators`, read at `mnemonic-secret` `cd0a60f`
(`crates/ms-cli/src/format.rs:12-14,35`): `is_display_separator(c) =
c.is_whitespace() || c == '-' || c == ','`; `strip_display_separators` filters
those out. Throwaway Go replica (`sep.go`, same directory):

```
input="correct-horse,battery staple" stripped="correcthorsebatterystaple" changed=true
```

So the separators row (contains `-` and `,`) **does** change under
`strip_display_separators` — it is a real witness for a strip-separators
mutation. §7.1's mutation bullet now reads: "**strip display separators from
the phrase before deriving → the `correct-horse,battery staple` row fails**" —
true, and it names a mutation the row can actually catch, unlike round 0's
`NormalisePassphrase` mis-attribution. **NF-A: FIXED.**

### 2. NF-B

Read `gui/modal_fits_test.go` at `c4a64fc` directly; confirmed by line number:

```
$ grep -n "^func assertModalBodyFits\|^func modalHeadroom\|^func firstModalFrame\|^func normalizeDrawn\|^const modalBodyMargin" gui/modal_fits_test.go
51:const modalBodyMargin = 80
60:func normalizeDrawn(s string) string {
140:func firstModalFrame(t *testing.T, ui func(*Context)) string {
182:func modalHeadroom(t *testing.T, r modalRenderer, body string) int {
201:func assertModalBodyFits(t *testing.T, what string, r modalRenderer, body string) {
```

All five citations (`:51`, `:60` (spec cites `:60-71`, matching the exact
function body), `:140`, `:182`, `:201`) are exact. Read the function bodies:
`assertModalBodyFits` renders the SPECIFIC body via `firstModalFrame`, requires
it draws in full, then calls `modalHeadroom` (binary search over appended
filler) and requires `head >= modalBodyMargin` (80). No capacity constant
exists anywhere in `gui/` outside this margin (`grep -rn "capacity"
--include=*.go gui/` returns unrelated hits — plate/QR/backup capacities, none
for modal fit). `grep -n "588\|composer_copy_test" gui/modal_fits_test.go
gui/composer_copy_test.go` returns exactly one hit, the historical-measurement
comment at `modal_fits_test.go:32` ("both modal shapes drew 588 normalized
characters in full") — `composer_copy_test.go` contains neither string.

The spec (`grep -n "composer_copy_test\|588" design/SPEC_hashlock_H2_device.md`)
now cites only `gui/modal_fits_test.go` and frames "588" correctly as "one
historical measurement of unrelated filler, not a budget" — the false
`composer_copy_test.go` / "capacity 588" citation from round 0 is gone
everywhere in the file. §4's preamble, §4.5, §7.2 and §10 all describe the
mechanism the same way (per-body render + headroom, margin 80, no capacity
constant) — consistent with each other and with the source. **NF-B: FIXED.**

§4.5's drop order: "first shorten the reuse block to the brainstorm's two
sentences ..., then move the reconciliation line into the phrase-route §8h at
Done (§4.7). The backup line and the relation line are never dropped." Two
concrete steps, each naming what to edit and where it goes; the never-dropped
lines are named explicitly. Well-formed.

### 3. I-5

`gui/composer_hash.go:27-28` at `c4a64fc`, read directly:

```
// THE COMPOSER NEVER DERIVES, STORES OR ENGRAVES A PREIMAGE this cycle
// (§14). It takes a digest and puts it in a script.
```

§1 item 5 gives the exact replacement text, in the same two-sentence
all-caps-then-lowercase shape: *"THE COMPOSER DERIVES A PREIMAGE IN RAM FOR ONE
SCREEN (H2) AND NEVER STORES, SHOWS OR ENGRAVES IT. It puts a digest in a
script."*

`SPEC_wallet_policy_composer.md:386`, read directly: still "The composer never
derives, stores or engraves a preimage this cycle (§14)." — unchanged, exactly
as the H2 spec's opening paragraph and §1 item 5 now say (future tense: "the
composer spec's two sentences are H3's ... and stay as they are until then").

`grep -n "folded\|fold\b" design/SPEC_hashlock_H2_device.md` — every hit that
refers to the composer-spec sentences is future/conditional ("folded by H3,
not here"; "to be folded when H2 re-vendors" — an unrelated tag comment). No
present-tense "are folded" claim remains anywhere in the file. **I-5: FIXED.**

### 4. M-4 / M-2

`gui/composer_shape.go:77`:
```go
func composerConfirmScreen(ctx *Context, th *Colors, title, body string) bool {
```
`gui/composer_copy.go:32-33`:
```go
func composerConfirmBody(body string) string {
	return body + "\n\nHold button to confirm."
}
```
Both citations exact by line number and signature. §4.5 states: "the operator
HOLDS to confirm and presses Back to decline" and "**HOLD** (the confirm
gesture) sets `st.list.Paths[idx].Hash`... **Back** returns to the method pick
with the phrase intact." Consistent with §4.6 (Back contract, no mention of
CONTINUE) and §7.2 (no mention of CONTINUE).

§4.4 (M-2): "driven by `hashlock.DeriveHardened` — never `unlockDerive` and
never a `seal.Header` (§3)" — this repeats §3's own forbid clause
verbatim-in-substance ("`unlockDerive` and `seal.Header` are NOT used"). **M-2:
FIXED.**

**But `grep -n "CONTINUE" design/SPEC_hashlock_H2_device.md` returns a
survivor:**

```
149:is dropped after CONTINUE or Back** (L7; L15: no scrub beyond that). The digest
```

This is §3's sentence "The preimage lives on the stack for the derivation and
the confirm modal and is dropped after **CONTINUE** or Back" — present
unchanged since the original draft (`bfd042e`, then line 103; `60a86f6`, then
line 136; confirmed by `git show <rev>:design/SPEC_hashlock_H2_device.md | grep
CONTINUE` at each). The r1 fold rewrote §4.5's own "**CONTINUE** sets..." to
"**HOLD** (the confirm gesture) sets...", and the fold commit's message claims
unqualified "M-4 FIXED: ... HOLD, not CONTINUE" — but §3, three sentences
before §4, still names the confirm-modal exit action CONTINUE. The actual
surface, `ConfirmWarningScreen` (`gui/composer_shape.go:74-90`), has no
CONTINUE control at all — only hold-to-confirm and Back-to-decline — so §3's
phrasing does not merely use stale terminology, it names a control that does
not exist in the mechanism §4.5 (this same fold) correctly describes. A hostile
implementer reading §3 in isolation, or grepping the document for "how does the
operator confirm", finds two different answers.

**M-4: PARTIAL, not fully fixed — see NF-D below.**

### 5. New contradictions

Re-read the whole scoped set (opening paragraph, §1 item 5, §2, §4 preamble,
§4.4, §4.5, §7.1, §7.2, §10's new rows, "R0 round 1 folded here") as a hostile
implementer. The only contradiction found is the §3/§4.5 CONTINUE-vs-HOLD
mismatch above (NF-D). Everything else checked cross-consistent: §4.6 and §7.2
never mention CONTINUE; §4.7 is unaffected by and doesn't conflict with §4.5's
drop order (a conditional fallback, not yet triggered); §10's three new/changed
rows (fit gate, confirm surface, `NormalisePassphrase`) match source exactly;
the "R0 round 1 folded here" paragraph accurately maps NF-A/NF-B/NF-C/M-4/M-2
to the sections that changed.

> **NF-D (Important, new).** §3's "is dropped after CONTINUE or Back" was not
> updated when §4.5 was renamed HOLD in this same fold. It contradicts §4.5's
> normative statement of the confirm gesture and names a UI control
> (`CONTINUE`) that does not exist on `ConfirmWarningScreen`. One-clause fix:
> change "CONTINUE" to "HOLD" at line 149 (or reword to "HOLD or Back", per
> §4.5's own phrasing).

---

## Closing counts

| item | verdict |
| --- | --- |
| NF-A (false mutation claim) | FIXED |
| NF-B (wrong fit-gate citation) | FIXED |
| I-5 (composer-spec record scope claim) | FIXED |
| M-2 (§4.4 repeats forbid) | FIXED |
| M-4 (HOLD not CONTINUE, everywhere) | PARTIAL — §4.5 fixed; §3 leftover (NF-D) |
| New Important findings this round | 1 (NF-D) |
| New contradictions beyond NF-D | 0 |

**GATE: NOT GREEN.** One Important blocks: NF-D, a one-clause leftover in §3
("CONTINUE" → "HOLD") that the r2 fold's own M-4 edit did not reach. NF-A,
NF-B and I-5 — the three items this round was dispatched to verify — are all
genuinely fixed with every new citation confirmed true against the fork and ms
sources at the pinned revisions.
