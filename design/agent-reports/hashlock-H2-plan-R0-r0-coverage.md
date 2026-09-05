# R0 round 0 — spec-coverage + comprehension review

`design/IMPLEMENTATION_PLAN_hashlock_H2_device.md` at engrave `02abee6`, against
`design/SPEC_hashlock_H2_device.md` (GREEN `55ee7a4`), the fork at
`/scratch/code/shibboleth/seedhammer` main `c4a64fc`, mnemonic-secret at `cd0a60f`,
and the gated tree `/scratch/code/shibboleth/.tmp/h2-gate` (read only, not modified).
Sonnet tier, no sub-agents, read-only, no `.jsonl` read.

**Method.** Read the spec and the plan in full (both entire files, not excerpts).
Built the coverage matrix below directly from the spec's §2-§7/§10 sentences, not
from the plan's own Self-review. Grepped the plan for placeholder patterns.
Verified every `file:line` citation and every quoted Rust/Go identifier the spec
and the plan make, against the fork (`git -C seedhammer show c4a64fc:<path> | ...`)
and against mnemonic-secret at `cd0a60f`, by direct `git show`/`grep`, not by
trusting either document's own prose. Independently re-measured the firmware
baseline (`nix develop -c tinygo build -size short ...` against the fork's actual
`c4a64fc` checkout, output to `/dev/null`, tree left clean) rather than trusting
the plan's/gate's cited number. Independently re-derived the vendored corpus's
per-row facts (row count, which rows carry `-`/`,`, the anchor digests, the
`too-long` row's length) from the actual corpus JSON at `mnemonic-secret@cd0a60f`,
not from the plan's or gate's tables. Checked every Interfaces block against the
neighbouring task that consumes it, and every task's `git add` list against its
own `Files:` section.

---

## 1. Coverage matrix (built from the spec text, not the plan's Self-review)

| Spec clause | Plan location | Verdict |
| --- | --- | --- |
| §2 rule 1 non-empty | Task 1 `ValidatePhrase` (`ErrEmpty`); Task 1 `TestRefusalRowsMatchTheHost` row 0 | COVERED |
| §2 rule 2 printable ASCII 0x20-0x7E | Task 1 `ValidatePhrase` loop; refusal rows 1-4 (café, TAB, DEL, 0xFF) | COVERED |
| §2 rule 3 ms1-shaped, host's shape test, no checksum | Task 1 `IsMS1Shaped` (port of `looks_like_ms1`/`is_ms1_shaped`); refusal rows 9-13; Task 4 refusal copy | COVERED |
| §2 rule 3 precedes rule 4 (order) | `ValidatePhrase`'s literal statement order (empty→ascii→ms1-shaped→toolong→hex64); corpus's grouped-by-2/112-char row exercises exactly this ordering | COVERED, order verified against actual code |
| §2 rule 4 ≤100 chars, unclamped `n/100` counter | Task 1 `ErrTooLong`/`PhraseMaxChars`; Task 4 `hashlockPhraseFlow`'s `"%d/%d"` counter (not clamped — no min/cap on the format call); §7.2's 101/100 screen test | COVERED |
| §2 rule 5 exactly-64-hex refused | Task 1 `ErrHex64`/`isHex`; Task 4 refusal copy; §7.2 test | COVERED |
| §2 bytes verbatim, forbidden-mechanism rule | Task 1 `ValidatePhrase`/`IsMS1Shaped` (copy only inside the shape test); Task 4 `hashlockPhraseFlow` uses `kbd.Fragment` directly, no `TrimSpace/Fields/ToLower`; Task 1's mutation table exercises exactly this; Task 4's `TestHashlockPhraseRouteDoesNotNormalise` | COVERED |
| §2 `PhraseMaxChars` single source | Task 1 `PhraseMaxChars = 100` read by both `ValidatePhrase` and the Task 4 counter; `TestPhraseMaxCharsIsTheCap` | COVERED |
| §3 constants + both derivations + `Digest` | Task 1 `hashlock.go` (Salt/Iterations/PreimageLen, `PreimageHardened`, `PreimageSHA256`, `Digest`) | COVERED |
| §3 `DeriveHardened` signature, stepwise, never `unlockDerive`/`seal.Header` | Task 1 `DeriveHardened` on `seal.NewDeriver` with `Salt` as a `[]byte` slice; Task 4 `hashlockDeriveFlow` calls only `hashlock.DeriveHardened` | COVERED, and independently confirmed `seal.Header.Salt` is `[16]byte` (see citation table) |
| §3 preimage on stack only, dropped after HOLD/Back; digest is what's stored | Task 4 `hashlockPhraseRoute` (locals `phrase`, `x`; only `h := hashlock.Digest(&x)` is ever assigned into `st.list.Paths[idx].Hash`) | COVERED |
| §4.1 row order; no-payload lead (exact copy) | Task 3 `composerHashRows` (digests, phraseRow, hexRow, noneRow, in that order); `composerCopyHashlockNoPayloadLead()` — text verified word-for-word against spec | COVERED |
| §4.2 title/lead, `NewPassphraseKeyboard`, `hashlockPhraseFlow` signature, `initial`, Back drops phrase, OK applies §2 | Task 4 `hashlockPhraseFlow` | COVERED, signature matches spec's stated one exactly |
| §4.3 method pick rows, both warnings verbatim, hardened threshold <20, decline keeps phrase | Task 4 `hashlockMethodPick`/`hashlockMethodWarning`; copy verified word-for-word | COVERED |
| §4.4 Deriving title/lead/step, never `unlockDerive`/`seal.Header`, Back abandons | Task 4 `hashlockDeriveFlow` | COVERED |
| §4.5 confirm body order, relation line, backup line unconditional, drop order | Task 4 `composerCopyHashlockConfirm`/`composerCopyHashlockRelation`; both drop-order steps folded (build gate fixes 3-4) | COVERED, confirmed the folded text is the exact post-drop-order shape |
| §4.6 Back contract, loop, `false` only at `Which hash?`, tests via `composerAddPath` | Task 3 `composerHashEdit`'s loop; Task 4 `hashlockPhraseRoute`'s nested loops; `TestHashlockBackContractKeepsThePath` drives through `runComposerAddPath`→`composerAddPath` | COVERED |
| §4.7 §8h phrase-route form, only at Done, not repeated in confirm | `composerCopyHashEveryPathFor`/`composerCopyHashEveryPathPhrase`; `composer_shape.go:443` swap | COVERED |
| §5 label-keyed switch, no assigning `default`, §8i predicate | Task 3 `composerHashRowSet`/`composerHashRows`/`composerHashEdit`'s `panic` default | COVERED |
| §6 `DecodeMS1Preimage`, shape-exact, `DecodeMS1` unchanged | Task 2 | COVERED |
| §7.1 lockstep tests (sha pin, 11 derivation, 15 refusals, kind, cap) | Task 1 `hashlock_test.go` | COVERED, corpus shape independently re-measured (11/15/1/4) |
| §7.2/§7.3 screen + switch tests | Task 4 `composer_hashlock_test.go`; Task 3 `composer_hash_test.go` | COVERED |
| §7.4 decoder tests | Task 2 `mspayload_test.go` | COVERED |
| §7.5 emulator arm | Task 5 Step 1 — **prose only, no code block** | GAP, but self-disclosed (Self-review §2) and scheduled before the post-impl review (Task 6) — not silent |
| §7.6 firmware size | Task 5 Step 2 | COVERED, baseline independently re-measured (see citation table) |
| §8 acceptance (H4) | Task 6 (operator's walk, out of this plan's build) | COVERED (correctly out of build scope) |
| §9 out of scope | Nothing to build | N/A, correctly untouched |
| §1 item 5 (`gui/composer_hash.go:27-28` header rewrite) | Task 3 Step 2 | COVERED, verified the exact old/new text against the fork |

No spec MUST/NEVER/refuse/copy sentence in §2-§7 or §10 was found with no plan
step. The single gap (§7.5, the emulator walk) is prose-only but is named as
such by the plan's own Self-review and is explicitly scheduled as a controller
action before the post-implementation review (Task 6) — this is a disclosed,
scheduled gap, not a silent one, so it is Minor rather than Important.

---

## 2. Placeholder scan

`grep -inE "TBD|TODO|\blater\b|similar to Task|\bappropriate\b|handle edge|as needed|…"`
over the whole plan: every hit is either prose describing a PAST event ("the
second and later holds route to..."), a truncated quote inside a citation
("PreimageHardened … undefined"), or the disclosed §7.5 gap. One additional item
found by reading rather than grepping:

- **Task 3 Step 5's "one-line stub" is a step that describes without showing
  the code** — see I-1.

No other placeholder-shaped step found. `scripts/h2-plan-blocks-vs-tree.sh`'s own
"NOT COVERED" list (25 blocks checked, 0 FAIL) is accurate and matches what a
`file=`-headerless block actually is in this plan (7 bash recipes + all prose) —
spot-checked against the plan's own line numbers for those 7 (119, 540, 636, 853,
1957, 1975, 1993 — all confirmed bash/no-header blocks by direct inspection).

---

## 3. Citation table (representative; every citation below was checked directly
against the fork/mnemonic-secret, not against either document's own prose)

| Claim | True/False | Evidence |
| --- | --- | --- |
| `composerHashEdit` doc comment at :139, func at :140, ends :176 (plan's own citation) | TRUE | `git show c4a64fc:gui/composer_hash.go` line-numbered |
| header comment "THE COMPOSER NEVER DERIVES..." at :27-28 | TRUE | same file, lines 27-28 verbatim |
| `rows = append(rows, "Type 64 hex")` at :147; `composerPickScreen(...,"Which hash?",...)` at :149 | TRUE | same file |
| `default: st.list.Paths[idx].Hash = nil` — spec §10/§5 cite :172 | FALSE (off by one) — actual line is **:173** | same file; Minor, spec-level only, plan's own citation (:139-176 range) is correct |
| `composerHashRow`/:38, `composerPayloadDigests`/:47, `composerHexEntry`/:69 | TRUE | same file |
| `composerCopyHashRule`/:175, `composerCopyHashEveryPath`/:169-173 | TRUE | `gui/composer_copy.go` |
| `composerConfirmScreen`/:77, signature `(ctx, th, title, body string) bool` | TRUE | `gui/composer_shape.go` |
| `composerConfirmBody`/:32-33, "Hold button to confirm." | TRUE | `gui/composer_copy.go` |
| `composer_shape.go:269` — `false` from `composerHashEdit` deletes path | TRUE | verified surrounding code |
| `composer_shape.go:443` — `composerEveryPathHashed` guards `showError(...composerCopyHashEveryPath())` | TRUE | same |
| `composer_shape.go:250` — key-less refused under `tr`, exact refusal text | TRUE | `composerCopyRefuseKeylessTr()` = "This build will not put a key-less path in taproot..." |
| `composerPickScreen` sig at :259; `composerPickScreenMaxRows = 24` at :224 | TRUE | `gui/composer_paged.go` |
| Three keyboards at :76/:92/:112 | TRUE | `gui/passphrase_keyboard.go` |
| `passphraseEntryFlow` sig at :74 | TRUE | `gui/passphrase_flow.go` |
| `passphrase_flow_test.go:28-31` display-size rule | TRUE | verbatim match |
| `kdfStepIterations = 500` at :26; `unlockKDFLead` "about 30 seconds" at :219-221; `unlockDerive` sig at :242 | TRUE | `gui/unlock_kdf.go` |
| `seal.NewDeriver(passphrase, salt []byte, iterations int) *Deriver` at `seal/pbkdf2.go:85`; `Step/Done/Total/Key/Wipe` | TRUE | full function bodies read |
| `seal.Header.Salt` is `[16]byte` (`SaltLen=16`) — zero-pad risk claim | TRUE | `seal/wire.go:32,85` |
| `seal/open.go:76-78` `NormalisePassphrase` = `ToLower(Join(Fields(s)," "))` | TRUE | exact |
| `sysw/open.go:55`, `seal/open.go:231` — same normalise-then-derive shape | TRUE | both read |
| `codex32/mspayload.go:35` `DecodeMS1`, `:94` `IsPreimage`, `msPrefixPreimage=0x03` at :11 | TRUE | exact |
| Five `DecodeMS1` callers at the cited lines (ms1_decode.go:22, codex32_polish.go:106, singlesig_verify.go:185, multisig_verify.go:1237, bundle/verify.go:138) | TRUE (all 5) | each line is a `codex32.DecodeMS1(...)` call |
| `codex32/polish.go:82` `ParsePrefix`, `:71` `Fields.Unshared`; `codex32/codex32.go:279` `NewSeed` | TRUE | exact |
| `gui/composer_door_test.go:15` `composerSessionWith` | TRUE | exact |
| `gui/composer_copy_test.go:29` `composerCopyTable()` | TRUE | exact |
| Fork `composerCopy*` count at `c4a64fc` = 41; test's own literal is `!= 41` | TRUE (independently counted) | `grep -c "^func composerCopy"` = 41 |
| `gui/composer_gates_test.go`'s pre-fold pump target is literally `"Which hash?"` (fix 12's premise) | TRUE | line 674, exact |
| `md/compose.go:32` `ComposeTr` iota 0; `:167` `Hash *[32]byte` | TRUE | exact |
| `gui/composer_shape.go:223` `composerAddPath`'s `ChoiceScreen` precedes EXPERIMENTAL (fix 7's premise) | TRUE | exact |
| `gui/event.go:14-15` single global `pointer{pressedTag,pressed}`; `Events` reuses stale tag while pressed (fix 11's mechanism) | TRUE | full method body read |
| `gui/seal_fixture_test.go:172` pre-existing `hashHex(h [16]byte)` (fix 13's collision) | TRUE | exact |
| `gui/wipe_guard_test.go:18` `sessionHarness.hold` never releases | TRUE | full body read |
| `sh2DisplaySize = image.Pt(480,320)`; `plateHitPoints`, `sessionHarness{frame,drawer func...}`, `runUITouch` 3-return sig, `ppTagFor`/`typeRune` mirrored faithfully | TRUE (all) | exact |
| `layoutTitle`, `layoutNavigation`, `widget.Label/Labelf/Labelw`, `Styles{lead,subtitle,progress}`, `NavButton{Clickable,Style,Icon}`, `StyleSecondary/StylePrimary`, four `assets.Icon*`, `layout.Rectangle.CutTop/CutBottom/N/S/Center/Dy` | TRUE (all) | every signature and call-site argument order matches |
| `composerState.bound` exists as an anchor for the `hashByPhrase` field insertion | TRUE | `gui/composer_state.go:33` |
| ms-codec `HASHLOCK_SALT`/`ITERATIONS`/`DKLEN` at hashlock.rs:27/30/32; `HASHLOCK_PHRASE_MAX_CHARS=100` at hashlock_phrase.rs:24 | TRUE | `mnemonic-secret@cd0a60f` |
| `looks_like_ms1`/`is_ms1_shaped`/`strip_display_separators`/`MIN_MS1_LEN=48`/`BECH32_CHARSET` at argv_guard.rs:98-164 | TRUE | exact, including the Go port's byte-for-byte charset match |
| Corpus sha256 `a46c197a…1d30` | TRUE (independently computed) | `git show cd0a60f:...json \| sha256sum` |
| Corpus shape: 11 derivation / 15 refusals / 1 kind / 4 lockstep rows | TRUE (independently counted from JSON) | python3 json.load |
| Anchor row digests `3cf5d421..b70a4c12` (hardened), `b867db87..edbc96cb` (sha256) | TRUE (independently recomputed first8/last8) | corpus row 0 |
| "4 rows carry `-`/`,`" (gate's correction of the plan's own claim) | TRUE (independently re-derived) | dumped all 11 phrases, matched exactly: `correct-horse,battery staple`(28), `a-b,c`(5), the 64- and 65-char rows |
| "the sole too-long refusals row is 101 chars" (gate's correction) | TRUE (independently re-derived) | corpus refusals[14], len=101 |
| Plate `ms10hashsq...` (kind[0].ms1) length 75; grouped-by-2 → 112 chars | TRUE (independently computed) | python3 |
| Firmware baseline at `c4a64fc`: flash 1,583,132 / RAM 62,800 | TRUE (independently rebuilt) | `nix develop -c tinygo build -size short ...` run directly against the fork's own `c4a64fc` checkout (already HEAD, tree clean before and after); measured **1583132 / 62800**, exact match |
| Gated-tree delta arithmetic: 1,595,236−1,583,132=12,104; 62,856−62,800=56 | TRUE | arithmetic checked |
| `assertModalBodyFits`/`modalHeadroom` grouped under spec §10's `gui/modal_fits_test.go:51` citation | Loosely stated — actual defs are at :201 and :182, not :51 (only `modalBodyMargin` is at :51) | spec-level citation, not attached to specific lines for those two names so not a hard falsehood; Minor |
| Plan's own citation `modal_fits_test.go:301` for `TestModalsThisBlockTouchesAreDrawnInFull` | TRUE | exact |

---

## 4. Findings

### I-1 — Task 3 Step 5's "one-line stub" instruction under-specifies what compiles

Task 3 Step 2 writes `composerHashEdit`'s phrase-row arm as:

```go
case sel == rows.phraseRow:
    switch hashlockPhraseRoute(ctx, th, st, idx, rows.digests) {
    case hashlockAssigned:
        return true
    case hashlockBackToWhichHash:
        continue
    }
```

This switch references **both** `hashlockAssigned` and `hashlockBackToWhichHash`
as case values of `hashlockPhraseRoute`'s return type — for the package to
compile, both constants and the `hashlockOutcome` type they belong to must
already exist. Task 3 Step 5 then says: *"Task 4 supplies `hashlockPhraseRoute`;
until then add a one-line stub in `gui/composer_hashlock.go` returning
`hashlockBackToWhichHash` so this task compiles, and replace it in Task 4."*

That is not one line and not fully specified: it names only the ONE constant
the stub function returns, not the type declaration or the second constant
(`hashlockAssigned`) that `composer_hash.go`'s own switch (written earlier in
the SAME task) already requires to exist. No code block is shown for this stub
anywhere in the plan. A fresh implementer following the literal instruction
would hit an "undefined: hashlockAssigned" compile error not explained by this
step, and would have to reverse-engineer that the stub file must declare:

```go
package gui

type hashlockOutcome int

const (
    hashlockAssigned hashlockOutcome = iota
    hashlockBackToWhichHash
)

func hashlockPhraseRoute(ctx *Context, th *Colors, st *composerState, idx int, payload [][32]byte) hashlockOutcome {
    return hashlockBackToWhichHash
}
```

This is self-correcting via the TDD RED step (the compile error is the signal),
so nothing ships silently wrong — but it is exactly the "step that says what to
do without showing the code" class the placeholder scan is meant to catch, and
it is the one place in an otherwise extremely precisely-cited plan where a name
(`hashlockAssigned`) is used in one task before being shown defined anywhere.
Confirmed the build gate's own report is silent on the stub's exact contents
too (it only says "a Task-3 stub `hashlockPhraseRoute` (replaced in Task 4)"),
so this gap is not resolved by cross-referencing the gate report either.

### M-1 — Task 3's `Files:` header omits `gui/composer_hashlock.go`

Task 3's `Files:` section lists 5 files; Step 5 creates a 6th
(`gui/composer_hashlock.go`, the stub), and Step 6's `git add` list correctly
includes all 6. The `git add` list itself is complete (satisfies check #6);
only the informational `Files:` summary is out of sync with what the task
actually touches.

### M-2 — Spec §10/§5 cite `gui/composer_hash.go:172` for the clearing `default`; actual line is `:173`

The function spans :140-176 (doc comment :139). Spec §5 and §10 both write
":140-172" / ":172" for the same content the plan itself correctly cites as
":139-176". Does not affect the plan (which cites the range accurately); a
one-line drift in the spec, pre-existing and out of this plan's authorship.

### M-3 — Spec §10 loosely groups `assertModalBodyFits`/`modalHeadroom` under `gui/modal_fits_test.go:51`

Those two identifiers are actually defined at :201 and :182; only
`modalBodyMargin` (:51) and `normalizeDrawn` (:60-71, correctly given its own
line range) belong there. Not a hard falsehood since no specific line is
attached to the two names, but a reader searching at :51 for `assertModalBodyFits`
will not find it there. Spec-level, not plan-introduced.

### N-1 — Task 6 shows no `git add`/`commit` block for the `FOLLOWUPS.md` edit

Unlike Tasks 1-5, Task 6's bullet list (FOLLOWUPS.md, post-impl review, merge)
has no explicit commit step. This edits `design/FOLLOWUPS.md` in the
mnemonic-engrave repo (not the fork), a different context from every other
task's fork commits, and recording continuity notes is already covered by this
repo's standing conventions. Nit, not blocking.

---

## Closing counts

- Critical: 0
- Important: 1 (I-1)
- Minor: 2 (M-1, M-2 is spec-level/M-3 is spec-level — counting M-1 as the only
  plan-attributable Minor; M-2/M-3 are pre-existing spec citation drift, noted
  for completeness but not chargeable to this plan's authorship)
- Nit: 1 (N-1)

Every spec §2-§7/§10 normative sentence maps to a plan task/step (one disclosed,
scheduled exception: §7.5's emulator walk, prose-only by the plan's own
admission). Every `file:line` citation checked against the fork and against
mnemonic-secret came back true except the two pre-existing spec-level drifts
above (M-2, M-3). The firmware baseline (1,583,132/62,800 at `c4a64fc`) and the
vendored corpus's per-row facts (11/15/1/4 rows; the 4 hyphen/comma rows; the
single 101-char too-long row; the anchor digests) were independently
re-measured from source, not taken on the plan's or gate's word, and all
matched exactly. Interfaces blocks (Task 1→4, Task 3→4) and every task's
`git add` list against its own edits were checked and found consistent, with
the one exception at M-1 (a documentation-only omission, not a missing file).

**Recommendation: fold I-1 (show the stub's real content, or fold the type/enum
declaration into Task 3 Step 2's own code block instead of describing it in
prose), fold M-1 (add the stub file to Task 3's `Files:` header), then this
plan is ready for the next review lens.**
