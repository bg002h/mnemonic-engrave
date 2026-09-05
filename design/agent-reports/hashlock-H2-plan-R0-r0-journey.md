# H2 device plan — R0 round 0, JOURNEY lens

**Artifact:** `design/IMPLEMENTATION_PLAN_hashlock_H2_device.md` at engrave `02abee6`
(unchanged at `5685258`, verified by `git diff`).
**Spec:** `design/SPEC_hashlock_H2_device.md` (GREEN `55ee7a4`).
**Trees read:** fork `/scratch/code/shibboleth/seedhammer` main `c4a64fc`; gated tree
`/scratch/code/shibboleth/.tmp/h2-gate` (read-only); ms `/scratch/code/shibboleth/mnemonic-secret`
(`504ff46`, working tree carries `cd0a60f`'s hashlock sources).
**Question asked:** walk the operator's journeys through the device THE PLAN BUILDS and
find the moments where the plan is silent.

**Counts: 0 Critical / 5 Important / 3 Minor / 2 Nit.**

Nothing in the plan derives a digest the host would not for the same intent — I looked
for that first and could not construct one. Every refusal fires in the host's order, the
phrase bytes reach the KDF untouched, the salt is a 14-byte slice, and the two `method`
words the device prints are byte-identical to the host's `--method` value-enum names. The
findings are all of the other shape: a moment the plan does not answer.

---

## Journey 1 — the happy path with a real wallet

Operator ran `ms hashlock --hashlock-phrase-stdin --method sha256` on the host for
`correct horse battery staple`, and is now building the same policy on the SH2.

| step | in hand | what the device does (plan's code) | what else they might do | class |
| --- | --- | --- | --- | --- |
| reach `Which hash?` | a host card + a policy with one key-less path | `composerHashRows` (Task 3): payload rows, `Type a hashlock phrase`, `Type 64 hex`, `No hash lock`; with 0 payload digests the LEAD is replaced by `composerCopyHashlockNoPayloadLead()` | load the `hash:` record instead → the payload row route, unchanged | default |
| tap the phrase row | — | `taking` is true → `showError(…, composerCopyHashRule())`, the §8i modal, then the phrase route | read §8i and back out | **I-5** |
| the phrase screen | 28 characters to type | `hashlockPhraseFlow`: `NewPassphraseKeyboard`, lead, `n/100`, OK applies `hashlock.ValidatePhrase` | type any of `0x20..0x7E` | **not our concern — every one is reachable** |
| type `-` `,` `"` `\` `~` `^` `` ` `` `|` space | a diceware phrase with separators | all four pages plus the function-row space key cover **95 of 95** printable ASCII exactly (`ppPageLower/Upper/Symbols/Symbols2` + `{r:' ', label:"space"}`; enumerated, no gaps, no extras) | a character the host accepts and the keyboard cannot produce | **none exists** |
| watch the counter | 28 typed | `%d/%d` from `len(kbd.Fragment)` and `hashlock.PhraseMaxChars`; unclamped | type 101 | see journey 3; **M-3** on whether the counter is still visible |
| pick `SHA-256` | — | `composerPickScreen` two rows; then the brainwallet modal (HOLD), then instant derivation | pick `Hardened` | default |
| read the confirm | `b867db87..edbc96cb` | `hash  b867db87..edbc96cb` / `method: sha256   chars: 28` / backup + reuse lines / `Hold button to confirm.` | compare to the host card | **I-3** (nothing asks them to) |
| HOLD | — | `Paths[idx].Hash = &d`, `st.hashByPhrase = true`, return | Back | §4.6, correct |

**Host display form, measured** (`crates/ms-cli/src/cmd/hashlock.rs:291-360`): stdout is
`hash:<64 hex>`; the stderr engraving card prints `digest:          <64 hex>`,
`method:          preimage = SHA-256(phrase)` (or the full PBKDF2 line naming
`salt = "ms-hashlock-v1", iterations = 100000, dkLen = 32`), and
`phrase:          28 characters -- …`. The device's `method: sha256` / `method: hardened`
match `Method::{Sha256,Hardened}`'s `ValueEnum` spellings exactly, so the two cards
reconcile word for word; only the digest is elided on the device, in the same
`first8..last8` form `composerHashRow` already uses for the payload rows. No divergence.

**The drop order at the longest phrase:** the plan measured and executed both of §4.5's
steps, but on the wrong screen — **I-2**.

---

## Journey 2 — the hardened method

| step | in hand | what the device does | what else | class |
| --- | --- | --- | --- | --- |
| pick `Hardened (about 10 s)` with a 28-char phrase | — | `len(phrase) < 20` is false → **no modal**, straight to `hashlockDeriveFlow` | a 19-char phrase → the 72-days modal | default (spec §4.3's threshold) |
| the first ~51 ms | — | **no frame is drawn**; the previous screen stands until the first `progress` call | — | not our concern |
| the countdown | — | `0%` and `About 10 seconds left.` — the zero-state lead `Deriving. This takes about 10 seconds.` is **never rendered** | — | **M-2** |
| press Back mid-derivation | — | `abandoned = true` → `(x,false)` → `continue` → method pick, **phrase intact** | — | §4.6, correct |
| a second `Deriving` on a second path | the same phrase, retyped | derives again, ~10 s, and assigns a second digest with **no comparison to path 1's** | mistype it | **I-1** |
| Back at the confirm, re-enter | phrase intact through `initial` | `hashlockPhraseFlow(ctx, th, phrase)` seeds `kbd.Fragment` | — | correct |

`seal.NewDeriver` + `Step(500)` totals exactly `Iterations` (`done` starts at 1 for U_1;
`Step` runs `total-done` more), and `DeriveHardened` was gate-proven against the corpus
constants. No divergence.

---

## Journey 3 — mistakes

| the operator types | host | device (plan) | class |
| --- | --- | --- | --- |
| a trailing space | **keeps it** — `strip_one_trailing_newline` strips one `\r?\n` and nothing else; its own test asserts `" abc "` is unchanged, *"spaces are bytes"* | keeps it; `chars: n` shows it | **no divergence** |
| 101 characters | `TooLong{chars}` | keyboard accepts the 101st key, counter reads `101/100`, OK refuses with *"A hashlock phrase is at most 100 characters."* | default, as spec'd — refuse at OK, not at the key |
| 64 hex characters | `Hex64` | `ErrHex64` → *"Use the Type 64 hex row."* (order: shape, then cap, then hex — matches `validate_phrase`) | correct |
| an ms1 plate, grouped or not | `looks_like_ms1` | `IsMS1Shaped` — trim, lowercase, strip separators, `len >= 48`, `ms1` prefix, bech32 charset; `MIN_MS1_LEN = 48` and the charset literal both verified identical | correct (**N-1** on the separator set) |
| nothing, then OK | `Empty` | `ErrEmpty` → *"Type a hashlock phrase, or press Back."* | correct |
| a different CASE from the host | different digest, silently | different digest, silently — and `chars:` is **identical** (28 either way), and with no payload record there is **no relation line** | **I-3** |
| the wrong method | a different digest | a different digest; the method is on no plate and in no `composerState` field | documentation only (`§8h` says to write it down) |

---

## Journey 4 — the two-hash payload

| step | in hand | device | what else | class |
| --- | --- | --- | --- | --- |
| `Which hash?` with two `hash:` records | 5 rows | `hash 1 …`, `hash 2 …`, phrase, hex, none | — | correct |
| type a phrase whose digest is record 2 | — | `rel = composerCopyHashlockRelation(1)` → *"matches hash 2 in the payload"*; numbering matches `composerHashRow(i+1, …)` | — | correct |
| edit path 1 back to `No hash lock` | — | `Hash = nil`; **`st.hashByPhrase` stays true** | start over from a preset | **M-1** |
| Back at `Which hash?` on a NEW path | — | `false` → `composerAddPath` deletes the path (`composer_shape.go:269`) — spec §4.6, tested | — | correct |
| Back at `Which hash?` on an EXISTING path | — | `false` → `composerPathEdit` case 2's `composerApplyShapeEdit` sees no signature change → nothing happens | — | correct |
| Back from `Type 64 hex` | — | **behaviour CHANGE**: returns to `Which hash?` instead of deleting the path | — | **I-4** |

---

## Journey 5 — power and memory

- `seal.NewDeriver` allocates once (`hmac.New`); `Step` allocates nothing — `d.mac.Sum(d.u[:0])` appends into `u`'s own backing array. Nothing per iteration reaches the 16 kb stack or the precise GC.
- The derive callback allocates per frame (`fmt.Sprintf`, `widget.Label*`, `layoutNavigation`, `op.Layer`) at ~200 frames over 10 s — the same shape `unlockDerive` already ships at ~600 frames over 30 s. Not a finding.
- `DeriveHardened` carries `defer d.Wipe()`; the phrase itself is a `[]byte` off `kbd.Fragment`, an unwipeable Go string, for the life of the route. **Secret-handling — never gates** (operator ruling 2026-08-27). Recorded, not filed.
- The confirm modal's longest body: measured, but on the wrong surface — **I-2**.
- The phrase screen's own geometry: unmeasured — **M-3**.

---

## Journey 6 — the reader side

An engraved preimage plate presented to the typed or NFC door still hits H0's guards
unchanged (`gui/codex32_polish.go:232`, `gui/unlock_session.go:197`, `gui/scan.go:89`,
`sysw/classify.go:126`), and H2 adds `codex32.DecodeMS1Preimage` with no screen caller.
The operator sees *"This record is a hashlock preimage, not a seed. It is not engraved as
one."* — true, and a dead end: it names no route, where §2 rule 3's phrase-screen refusal
does. Spec §6 states this explicitly and defers the manual to H3 — **N-2**, recorded, not
a change this stage owes.

---

## Findings

### I-1 — Two paths, two different phrases, and the device never says so
**Plan §Task 4 Step 3, `hashlockPhraseRoute`'s relation block.**

The relation line's comparison set is `payload` — `rows.digests`, i.e. `composerPayloadDigests(ctx.sysw)`, the payload's `hash:` records and nothing else. It is never compared against the digests **already assigned to other paths of the policy being built**, and `st` is in scope on that exact line.

The wsh policy the composer is for can carry a hashlock on more than one spend path, and `md.ValidatePathList` (`md/compose.go:299-340`) imposes **no** rule that two `Hash` values be equal — I read every clause; it checks thresholds, slot counts, lock ranges, the key-less/tr rule and the legacy shape, nothing else. So:

1. Operator sets path 1 by phrase. Confirm shows `b867db87..edbc96cb`, `chars: 28`, no relation line (no payload records).
2. Operator sets path 2 by phrase and mistypes one character of the same length. Confirm shows a *different* digest, `chars: 28`, no relation line, no warning.
3. Both confirms say **"One phrase per policy."** At Done, §8h says **"Back up the phrase and its method"** — singular, both in the shipped text and in the plan's new `composerCopyHashEveryPathPhrase`.
4. The operator writes down one phrase. Path 2 is unspendable by them.

Every screen asserts a single phrase; nothing checks it, and the one same-length typo the copy is least able to catch is the one that costs a path. `chars:` cannot see it, the relation line does not look there, and the two confirm screens are separated by a whole path-building flow.

I considered Critical. It sits just under this brief's bar: no digest is wrong, nothing is deleted, and the operator did get exactly what they typed. A funds-safety lens may rate it higher, and I would not argue.

**SUGGESTION.** Widen the comparison set at the same line, before the payload loop:

```go
for j, p := range st.list.Paths {
    if j != idx && p.Hash != nil && *p.Hash == h { rel = fmt.Sprintf("same hash as path %d", j+1); break }
}
```

and, when at least one other path already carries a *different* phrase-set hash, say so instead — `this policy will need TWO phrases`. One loop, one copy body, one row in each of the two copy gates.

### I-2 — Three of the five fit-gate rows are measured on the wrong screen
**Plan §Task 4 Step 1 (the `modal_fits_test.go` block and the drop-order note).**

`TestModalsThisBlockTouchesAreDrawnInFull` has exactly one `assertModalBodyFits` call and it hard-codes the renderer: `assertModalBodyFits(t, tc.what, errorScreenBody, tc.body)` — **gated tree `gui/modal_fits_test.go:359`**, unchanged by this plan. `errorScreenBody` renders `showError`. But three of the five rows the plan adds are drawn by `composerConfirmScreen` → `ConfirmWarningScreen` (`gui/composer_shape.go:77-91`), whose renderer is `confirmWarningBody`, defined in the same file at `:121`:

- `"the hashlock confirm modal, longest variant (H2 §4.5)"` — `composerConfirmScreen(ctx, th, "Hash lock", …)`
- `"the hashlock hardened warning (H2 §4.3)"` — `composerConfirmScreen(ctx, th, "Hardened", …)`
- `"the hashlock sha256 warning (H2 §4.3)"` — `composerConfirmScreen(ctx, th, "SHA-256", …)`

The fork's convention is unambiguous and already applied at four sites: `gui/composer_shape_test.go:192`, `gui/composer_discard_test.go:74`, `gui/composer_selfcheck_test.go:176`, `gui/multisig_build_prose_test.go:98` — every `composerConfirmScreen` body goes through `confirmWarningBody`. This plan is the exception.

Three consequences, in ascending order:

1. The quoted headroom is a number from a screen the operator never sees. `ConfirmWarningScreen` carries `Icon: assets.IconHammer` and a hold control that `showError` does not, and the §4.5 body is not filler: four explicit `\n`s, plus `\n\nHold button to confirm.` appended by `composerConfirmBody`, plus an 18-character `b867db87..edbc96cb` token the spec itself flags as unwrappable. The file's own "both modal shapes drew 588" note was measured with 5-to-7-letter filler, which is exactly the input that hides a line-break difference.
2. The two warning rows pass the **raw** copy (`composerCopyHashlockHardenedWarning()`), while the code draws `composerConfirmBody(composerCopyHashlockHardenedWarning())` — 24 characters more than what is measured.
3. **§4.5's drop order was executed on those numbers.** The plan's Step 0/1/2 table (484 cut → 384 drawn/headroom 64 → 290 drawn/headroom 186) is what removed the reconciliation line from the confirm modal. A decision that permanently changed what the operator is told rests on a measurement of the wrong surface — and I-3 is what it cost.

**SUGGESTION.** Give the table a `renderer` column (or split the three confirm bodies into their own `t.Run` with `confirmWarningBody`), wrap the two warning bodies in `composerConfirmBody` as the code does, then **re-run §4.5's drop order from step 0** against the new numbers. If the unshortened body now fits with 80 characters to spare, the reconciliation line goes back where the spec first put it and I-3 closes with it.

### I-3 — The reconciliation line is unreachable for any policy with one un-hashed path
**Plan §Task 4 Step 1 (`composerCopyHashEveryPathPhrase`) and §Task 4 Step 3 (`composerCopyHashEveryPathFor`).**

The line *"Before you fund this wallet, run ms hashlock with this phrase and method on the host and check the digest matches."* now lives **only** in `composerCopyHashEveryPathPhrase`, and §8h fires under `composerEveryPathHashed(st.list)` — which requires **every** path to carry a hash (`gui/composer_state.go:239-249`: any `p.Hash == nil` returns false).

So the ordinary shape — a 2-of-3 key path plus one hashlocked path — never sees it. The confirm modal no longer carries it either. For that operator the device says nothing, anywhere, about checking the digest against the host.

The sharpest case is the one journey 3 reaches: `Correct Horse Battery Staple` typed where the host used `correct horse battery staple`. Both read `chars: 28`. With no `hash:` record loaded there is no relation line. The digest itself is the only signal that anything diverged, and the one sentence that would have made the operator look at it has been moved behind a predicate that is false for their policy.

**SUGGESTION.** Either (a) restore the line to the confirm modal after I-2's re-measurement, or (b) if it must stay at Done, put it behind `st.hashByPhrase` rather than `composerEveryPathHashed` — a separate `showError` at the same site, fired whenever any hash was set by phrase. (b) is the smaller change and does not depend on I-2's outcome.

### I-4 — `Type 64 hex`'s Back changes behaviour, no test drives it, and the plan says one does
**Plan §Task 3 Step 2, the note at plan line 805.**

Verbatim: *"The test in Step 1 does not cover it; Task 4's harness tests do (Back from hex entry at creation keeps the path)."*

No test in this plan drives `composerHexEntry` at all. Grepping the plan for `Type 64 hex` finds four hits: the row-label assertion in Step 1, the label in `composerHashRows`, the refusal string, and a needle inside `TestHashlockPhraseRefusalsOnScreen` (which types 64 hex *into the phrase screen*). None of them enters the hex route, and none presses Back there.

The change itself is an improvement — the shipped `composerHexEntry`'s `false` propagates out and deletes the path at creation; under the new switch it `continue`s to `Which hash?`. But it is a live behaviour change to a shipped, funds-relevant route, shipped on a claim of coverage that does not exist. This is the class the project rule names: *a plan may not close while any of its own gates has never been run.*

**SUGGESTION.** Either add the six-line harness test (`tapRow(hexRow, 3)`, `tapNav(Button1)`, assert `len(st.list.Paths) == 1` and back at `Which hash?`), or delete the sentence and file the gap. Do not leave the claim standing.

### I-5 — The §8i rule modal now fires in front of a route it was written to warn against
**Plan §Task 3 Step 2 (`taking` predicate), fork `gui/composer_copy.go` `composerCopyHashRule`.**

`composerCopyHashRule()` reads, verbatim:

> "The hash must be SHA-256 of a 32-byte value. A passphrase must be hashed to 32 bytes first, then hashed again. **A hash of the passphrase itself can never be spent.**"

It was authored for the world the fork's own header comment states (`gui/composer_hash.go:27-28`, which this stage rewrites): *"THE COMPOSER NEVER DERIVES, STORES OR ENGRAVES A PREIMAGE this cycle."* Its audience was an operator about to paste a hex digest.

Under this plan it is the **first** thing shown when the operator taps `Type a hashlock phrase`, and three taps later they are offered a method row labelled `SHA-256`. The device is correct — the route computes `H = SHA-256(SHA-256(phrase))`, two hashes, a 32-byte preimage — but it never says so at either screen. The SHA-256 warning modal explains the *brainwallet* risk and says nothing about the 32-byte rule the operator was just told about. An operator who reads §8i literally has good reason to believe the row they are looking at is the one the device just told them can never be spent.

The likely reaction (retreat to Hardened) is harmless, which is why this is not Critical; the possible reaction (abandon the phrase route, or distrust the digest they just set) is worse than saying nothing on that screen.

**SUGGESTION.** One clause, on the phrase row only — either a second sentence in the §8i modal when `sel == rows.phraseRow` (*"This route does that for you: it hashes your phrase to a 32-byte preimage, then hashes that."*), or the same sentence at the head of `composerCopyHashlockPhraseLead()`. The latter costs no new gate row beyond the one that body already has.

### M-1 — `st.hashByPhrase` is set and never cleared
**Plan §Task 4 Step 3 (`composer_state.go` field, `composerCopyHashEveryPathFor`).**

The field means, per §4.7, *"at least one hash was set by phrase"*. As written it means *"at least one hash was ever set by phrase in this composition"*. It is not cleared when that path's hash is later set to `No hash lock`, replaced from a payload row or by hex, when the path is removed, or when `composerStartStep`'s preset replaces `st.list` wholesale (`gui/composer_flow.go:165` — `composerApplyShapeEdit(st, func() { st.list = next })`, which touches `assigned` and nothing else).

The wrong outcome is mild: the phrase form is a superset of the shipped one ("the phrase and its method, **or** the preimage plate"), so an operator who no longer has a phrase is told to back up something they do not have, alongside the thing they do. Worth noting that `composerState`'s own doc comment is a paragraph explaining why no memo lives there (C16, index-keyed confirms); this one is not index-keyed, so it does not repeat that bug, but it is the first sticky flag in the struct.

**SUGGESTION.** Derive it instead of storing it — a `hashByPhrase [N]bool` parallel to paths is index-keyed and repeats C16, so the cheaper honest fix is to clear the flag in the `noneRow` and `hexRow` arms and in `composerStartStep`'s replace branch, and say in the field comment that it is a high-water mark.

### M-2 — The `Deriving` zero-state lead can never render
**Plan §Task 4 Step 3 (`hashlockDeriveFlow`).**

```go
lead := composerCopyHashlockDerivingLead()
if elapsed := time.Since(start); done > 0 && elapsed > 0 { lead = fmt.Sprintf("About %d seconds left.", …) }
```

`seal.NewDeriver` sets `d.done = 1` before returning (it performs U_1), and `Deriver.Done()`'s own doc says so: *"Done counts iterations already applied, including the U_1 that NewDeriver performed."* `DeriveHardened` calls `progress` only after a `Step(500)` that returned false, so the **first** callback arrives with `done == 501`. `done > 0 && elapsed > 0` is therefore true on the first frame and every frame after it: `composerCopyHashlockDerivingLead()` is never drawn.

Spec §3 and §4.4 require that lead specifically, and the spec's own fold record credits it to fidelity M-6 / journey M-3 — it exists because the sealed payload's "about 30 seconds" is calibrated for a different iteration count. Both copy gates pass because they call the function, not the screen; `h.mustReach("Deriving")` matches the *title*.

The shipped `unlockKDFLead`'s `done <= 0` arm is unreachable for the same reason, so this is a pre-existing pattern rather than a new mistake — but the fork at least has that one as a pure function with its own unit test (`gui/unlock_kdf_test.go:521`), where this one is inline and untestable.

**SUGGESTION.** Either drop the dead branch and admit the screen opens on `About N seconds left.` (a spec amendment), or gate on `done <= 1` and hoist the lead into a pure `hashlockDerivingLead(done, total int, elapsed time.Duration) string` with the fork's existing unit-test shape.

### M-3 — The phrase screen adds a band to the one layout whose overflow was a measured defect, and nothing measures it
**Plan §Task 4 Step 3 (`hashlockPhraseFlow`'s layout block).**

`hashlockPhraseFlow` reproduces `passphraseEntryFlow`'s layout and inserts a lead band ahead of the counter:

```go
leadBand, content := content.CutTop(leadSz.Y)
counterBand, content := content.CutTop(cntsz.Y)
kbd.MaxHeight = content.Dy()
```

`MaxHeight` clamps only the **readout** (`passphrase_keyboard.go:454-473`); the grid height is fixed, and the block is bottom-aligned with `content.S(kbdsz)`, so if the remaining band is shorter than grid + gap the block grows upward and `op.Layer` draws it on top of the counter — the exact defect `passphrase_flow.go:120-138` records having measured on the real 480×320 panel, where "from ~70 characters the counter was hidden in exactly the revealed state a user proof-reads in".

`TestPassphraseEntryFitsPanel` (`gui/passphrase_flow_test.go:1331`) binds `passphraseEntryFlow` alone. The plan adds no equivalent for the new screen; its own checks are `h.mustReach("28/100")` and `h.mustReach("101/100")` — and the fork's comment at `passphrase_flow.go:128` says in terms why that cannot see it: *"ExtractText collects runes regardless of occlusion, so no text-based test could see it."*

I could not run a measurement without writing into the gated tree, so I am not asserting it overflows — my arithmetic says it probably does not, with roughly two lines of readout tail left instead of three. The finding is that the plan does not know either, on the one screen in this package with a written history of exactly this failure and an existing test shape to copy.

**SUGGESTION.** One test, modelled line for line on `TestPassphraseEntryFitsPanel`, at n = 70/90/100/101 revealed: assert `kbd.MaxHeight > 0`, `kbdsz.Y <= kbd.MaxHeight`, a 10-rune run of the readout, and the counter string. It also pins the lead, which nothing else does.

### N-1 — `IsMS1Shaped` strips a narrower separator set than the host
Go strips `' '`, `'\t'`, `'\n'`, `'\r'`, `'-'`, `','`. Rust's `format::is_display_separator` is `c.is_whitespace() || c == '-' || c == ','`, and `char::is_whitespace` additionally covers `0x0B` and `0x0C`. Unreachable in practice: `ValidatePhrase` runs the printable-ASCII rule first, and the keyboard cannot produce either byte. But `IsMS1Shaped` is exported and its doc comment calls itself *"the host's looks_like_ms1"*. A one-line comment naming the two bytes and why they cannot arrive would keep the claim true.

### N-2 — The preimage-plate journey is asymmetric, by the spec's own ruling
H0's door refusal (*"This record is a hashlock preimage, not a seed. It is not engraved as one."*, `gui/codex32_polish.go:233`, `gui/unlock_session.go:201`) names what the plate is and no route out; §2 rule 3's phrase-screen refusal names one. Spec §6 states this and defers the manual to H3. Recorded so H3 cannot miss it; no change asked of this plan.

---

## What I checked and did not file

- **Every printable-ASCII character is typeable.** Enumerated the four pages plus the function-row space key: exactly 95 of 95 in `0x20..=0x7E`, no gaps, no extras. `-`, `,`, `"`, `\`, `~`, `^`, `` ` ``, `|` and space are all present. No host/device divergence to document.
- **The host does not trim.** `strip_one_trailing_newline` removes one `\r?\n` and nothing else, and its own unit test asserts `" abc "` survives, *"spaces are bytes"*. Device matches.
- **The rule order matches** `validate_phrase`: empty → printable → ms1-shape → cap → 64-hex, with the shape test before the cap.
- **`MIN_MS1_LEN = 48`** and `BECH32_CHARSET = "qpzry9x8gf2tvdw0s3jn54khce6mua7l"` are byte-identical to the Go literals.
- **The method words match the host's `--method` value-enum** (`hardened` / `sha256`, default hardened), so device and card reconcile without a mapping.
- **`first8..last8`** is the same elision `composerHashRow` already uses, so the confirm line and the `Which hash?` rows compare directly.
- **`composerPickScreenMaxRows = 24`** bounds hit areas per *page*; `composerPickScreen` pages, so a payload with many `hash:` records paginates rather than truncating. Not a finding.
- **The §8i modal re-fires** on each re-entry through the loop — that is "as today", per §4.7.
- **`seal.NewDeriver` allocates once**, `Step` allocates nothing; the 16 kb stack and precise GC see the same load `unlockDerive` already ships.
- **Phrase lifetime / `Wipe`:** the phrase lives as an unwipeable Go string in `kbd.Fragment` for the route's life. Secret-handling, never gates (operator ruling 2026-08-27). Recorded here so the measurement is not lost.
