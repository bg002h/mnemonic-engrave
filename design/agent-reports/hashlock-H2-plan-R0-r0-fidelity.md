# R0 round 0 — fidelity + design lens on `IMPLEMENTATION_PLAN_hashlock_H2_device.md`

**Artifact:** `design/IMPLEMENTATION_PLAN_hashlock_H2_device.md` at engrave master `02abee6`
**Spec:** `design/SPEC_hashlock_H2_device.md` (R0 GREEN at `55ee7a4`)
**Fork:** `/scratch/code/shibboleth/seedhammer` main `c4a64fc` (read-only)
**Corpus:** ms `crates/ms-codec/tests/vectors/hashlock-v0.8.json` at `cd0a60f`
**Lens:** fidelity + design — *if an implementer follows this plan literally, does the device
derive the SAME digest the host derives for the same typed bytes and method, with every screen,
Back edge and copy line the spec requires — and does the plan claim nothing false about the fork
code and APIs it cites?*

**Counts: 0 Critical / 6 Important / 7 Minor / 4 Nit.**

---

## What I verified clean (stated so a later round does not re-derive it)

These are the questions the brief named, answered in the plan's favour, by measurement:

1. **Spec §2 ↔ `ValidatePhrase` / `IsMS1Shaped` — NO DIVERGENCE.** I read the host's
   `validate_phrase` (`crates/ms-cli/src/hashlock_phrase.rs:118-142`) and the plan's Go side by
   side. Rule order is identical: empty → printable `0x20..=0x7e` → `looks_like_ms1` → cap →
   64-hex. The host's cap is `s.len()` (BYTES) and so is Go's `len(phrase)`; the host's 64-hex
   test is `s.len() == 64 && hex::decode(s).is_ok()` and Go's is `len==64 && isHex` over
   `0-9a-fA-F` — identical for a 64-length input.
   * **The wider strip set is unreachable, so it cannot diverge.** Rust strips ALL Unicode
     whitespace + `-` + `,` (`format.rs:12-14`); Go strips space/tab/newline/CR + `-` + `,`. Rule
     2 runs FIRST and admits only `0x20..=0x7E`, in which the only whitespace is `0x20`. The two
     sets therefore agree on every input `IsMS1Shaped` can ever see. Same argument retires
     `strings.ToLower` vs `to_ascii_lowercase` and `len()`-in-bytes vs chars.
   * **Rule ordering between 3/4/5 is immaterial for 64-hex, and I can state why:** a 64-hex
     string cannot be ms1-shaped, because `m` and `s` are not hex digits so it can never start
     `ms1`; and 64 <= 100 so the cap never fires first. The order that DOES matter — shape before
     cap — is correct, and is pinned by the corpus's 112-character grouped-plate row.
   * **`ValidatePhrase` accepts exactly the rows the host accepts.** I walked all 15 `refusals`
     rows by hand: `''`→ErrEmpty; `café`(0xc3)/`0xff`/`a\tb`/`0x617f`→ErrNotPrintableASCII;
     ` ~` and `beef`→accepted; both 64-hex cases→ErrHex64; the five plate spellings→ErrMS1Shaped
     (the 75-char plate is all-bech32 after `ms1`, `grouped(plate,2)` is 75+37 = 112 chars as the
     corpus says); the 101-char row→ErrTooLong. The plan's placeholder switch matches the corpus's
     five placeholder strings byte for byte, and a miss would fail loudly (the literal placeholder
     is accepted, `want` is `ErrMS1Shaped`).
2. **Spec §3 ↔ `DeriveHardened` — the iteration arithmetic is RIGHT.** `NewDeriver`
   (`seal/pbkdf2.go:85-104`) computes `U_1` and sets `done = 1`; `Step(n)` runs
   `for i := 0; i < n && d.done < d.total; i++`. So `Step(Iterations)` performs **`Iterations`
   PRF evaluations in total** (U_1 plus `Iterations-1`) — correct PBKDF2 with c = 100000, not
   off by one. `DeriveHardened`'s `for !d.Step(500)` reaches exactly the same `done == total`
   and therefore the same key; the plan's own `TestDerivationRowsLockstep` asserts
   `DeriveHardened == PreimageHardened` on all 11 rows. `Key()` returns a fresh 32-byte copy
   (`acc [32]byte`). `defer d.Wipe()` is registered before any return in BOTH functions, so it
   runs on the progress-false (Back) path too.
3. **The corpus values are mine, not the plan's.** I recomputed the anchor row independently:
   `sha256_x = SHA256(phrase)`, `sha256_h = SHA256(sha256_x)`,
   `hardened_x = pbkdf2_hmac('sha256', phrase, b'ms-hashlock-v1', 100000, 32)`,
   `hardened_h = SHA256(hardened_x)` — all four match the corpus. `b867db87..edbc96cb` is the
   anchor `sha256_h`; `3cf5d421..b70a4c12` is its `hardened_h`; `chars: 28` is its `phrase_chars`.
   §7.5's assertions are correct.
4. **Spec §4.6 ↔ the loop — every Back edge is right, and `false` really is returned once.**
   Full trace in the table below. `composerHashEdit` returns `false` at exactly one site
   (`if !ok` after `composerPickScreen`); every other arm returns `true`, `continue`s, or panics.
   `Hash` and `hashByPhrase` are assigned only inside `if composerConfirmScreen(...)`, i.e. only
   after HOLD. The phrase is dropped on exactly one edge (Back at the phrase screen), because the
   route returns before `phrase` is reused, and a re-pick calls the route afresh with `phrase = nil`.
5. **Copy conforms to the spec, including the drop order, and the drop order was genuinely
   needed — I reproduced the gate's numbers.** Normalising the spec's §4.5 body the way
   `normalizeDrawn` does gives **504** characters (gate: 484 of 504 drawn, CUT); after drop-order
   step 1 (reuse block → the brainstorm's two sentences) **384** (gate: 384 drawn, headroom 64 <
   margin 80 → still failing); after step 2 (reconciliation line → §8h) **290** (gate: 290,
   headroom 186); the §8h phrase form is **254** (gate: 254, headroom 262). Every figure the plan
   folds is independently reproducible. The backup line and the relation line survive, as §4.5
   requires. Every other new string matches the spec's text verbatim (phrase lead, both method
   modals, deriving lead, all three §2 refusal messages, the no-payload lead, the §8h phrase form).
6. **Fork citations resolve.** `gui/composer_hash.go:27-28` is exactly the sentence §1 item 5
   replaces; `composer_shape.go:269` is the `st.list.Paths = st.list.Paths[:idx]` after
   `composerHashEdit`; `:443` is the `composerCopyHashEveryPath()` call; `:250` is the
   `ComposeTr` key-less refusal; `md/compose.go:32`'s `ComposeTr` is iota 0;
   `modal_fits_test.go:51` is `modalBodyMargin = 80`; `composerPickScreenMaxRows = 24`;
   `hookPPWidget` is production code (`passphrase_flow.go:28`), not a test-only symbol;
   `kbd.Fragment = prior` is the idiom `ftTextEntryFlow` already uses; the keyboard appends with
   `// NO ToUpper — case preserved` (`passphrase_keyboard.go:264`);
   `SPEC_wallet_policy_composer.md:386` is the §14 sentence H3 folds. The firmware baseline
   `1,583,132 / 62,800` for `c4a64fc` is corroborated by the H0 implementation record, and the
   deltas (+12,104 / +56) are arithmetically correct.
7. **`float64` in the countdown is not the truncation bug `unlock_kdf.go` documents.**
   `unlockKDFLead` multiplies before dividing to avoid `elapsed/done` truncating to 0 ns; the
   plan uses `float64`, which cannot truncate and cannot overflow. Correct by a different route.

---

## The Back-edge table (§4.6)

| edge | spec §4.6 says | the plan does | verdict |
| --- | --- | --- | --- |
| Back from the confirm modal | → method pick, phrase intact, nothing assigned | `composerConfirmScreen` false → falls out of the `if`, inner `pick` loop iterates → `hashlockMethodPick`; `phrase` untouched; `Hash` never written | OK |
| Back from a declined method modal | → method pick, phrase intact | `hashlockMethodWarning` false → `continue` (inner loop) | OK |
| Back from the method pick | → phrase screen, phrase intact via `initial` | `break pick` → outer loop → `hashlockPhraseFlow(ctx, th, phrase)`; `kbd.Fragment = string(initial)` | OK |
| Back from the phrase screen | → `Which hash?`, phrase DROPPED | `return hashlockBackToWhichHash`; `composerHashEdit` `continue`s; a re-pick re-enters with `phrase = nil` | OK |
| Back during the derivation | → method pick, phrase intact, nothing assigned | progress callback returns false → `hashlockDeriveFlow` false → `continue` | OK |
| Back at `Which hash?` | the ONLY `false` from `composerHashEdit`; deletes the path at creation | `if !ok { return false }`, single site; `composer_shape.go:269` splices | OK |
| Back from `Type 64 hex` entry | (covered by "false ONLY at `Which hash?`") → must return to `Which hash?`, path intact | `continue` — correct, and a deliberate behaviour change from the shipped `return false` | OK in code, **untested — see I-4** |

---

## Findings

### I-1 — The C-4 regression this whole task exists to prevent has no test that can fail on it, and the plan's stated mutation for it is false

**Where:** Task 3 Step 1 (`TestWhichHashRowsAreLabelKeyed`), against spec §7.3 and §5.

`TestWhichHashRowsAreLabelKeyed` calls `composerHashRows(s)` and asserts label placement and the
three indices. It never calls `composerHashEdit`, so it cannot observe the DISPATCH at all.

**Counterexample.** Leave `composerHashRows` exactly as the plan writes it and mutate only the
switch inside `composerHashEdit` back to index arithmetic:

```go
switch {
case sel < len(rows.digests):        // payload
case sel == len(rows.digests):       // now the PHRASE row
    ...
default:                             // now catches BOTH `Type 64 hex` and `No hash lock`
    st.list.Paths[idx].Hash = nil
}
```

This is r2 C-4 restored verbatim — tapping `Type 64 hex` silently clears the lock. Every test in
the plan still passes: `TestWhichHashRowsAreLabelKeyed` inspects only the row set, and all seven
`TestHashlock*` tests tap only the phrase row, the method rows and Back. So the plan's own comment
— *"MUTATION: restore the index arithmetic with the new row inserted -> `Type 64 hex` lands in the
clearing arm and this fails"* — is **false**. Spec §7.3 asks for exactly the missing test:
*"Every `Which hash?` row by label with 0, 1, 2 payload digests: each row does what its label says;
`Type 64 hex` never clears the lock (the C-4 regression test); the §8i modal fires for the three
taking rows and not for `No hash lock`."* None of those four clauses is driven behaviourally.

**SUGGESTION.** Add a behavioural test in `gui/composer_hash_test.go` that drives
`composerHashEdit` (or `composerAddPath`) once per row label with 1 and 2 payload digests: a
payload row assigns that digest; `Type 64 hex` reaches hex entry and, on Back, leaves `Hash`
UNCHANGED (not nil); `No hash lock` sets `Hash = nil` and shows no §8i modal. Then re-word the
mutation note to name the switch mutation above, which that test does catch.

---

### I-2 — Moving the reconciliation line behind `composerEveryPathHashed` silently removes it from the commonest policy shape, and the plan does not say so

**Where:** Task 4 Step 1, `composerCopyHashEveryPathPhrase` / `composerCopyHashEveryPathFor`;
spec §4.5 drop order step 2, §4.7.

In the spec the reconciliation line — *"Before you fund this wallet, run ms hashlock with this
phrase and method on the host and check the digest matches."* — sits in the §4.5 confirm modal,
which fires for **every phrase-set hash**. The plan (correctly following §4.5's drop order, and
correctly, because the line does not fit — 384 normalised chars, headroom 64 < 80) moves it into
the §8h phrase form. But §8h at `composer_shape.go:443` is guarded by
`composerEveryPathHashed(st.list)`, which returns false the moment ANY path has `Hash == nil`
(`gui/composer_state.go:239-249`).

**Counterexample.** The standard hashlock vault: path 1 = a 2-of-3 multisig (keys, no hash),
path 2 = key-less, hash set by phrase. `composerEveryPathHashed` is false, §8h never fires, and the
operator is **never told to cross-check the device's digest against `ms hashlock` on the host** —
in a stage whose entire purpose is that the two agree. The backup line survives (it is in the
confirm modal, unconditional), so journey C-1 is intact; it is journey C-2's second half that is
lost, for the majority shape. The plan presents the move as free ("the spec is unchanged; §4.5
names this drop order itself") and never states the audience change.

**SUGGESTION.** Keep the line unconditional on the phrase route by giving it its own surface
rather than sharing §8h's gate: after HOLD assigns the digest, one
`showError(ctx, th, "Hash lock", composerCopyHashlockReconcile())` — 94 normalised characters on
an error screen measured at ~550 capacity, so it fits with enormous headroom and costs the confirm
modal nothing. Failing that, state the loss explicitly in the plan and in §4.7, so H3's manual
chapter carries the instruction instead.

---

### I-3 — Three ConfirmWarningScreen bodies are fit-measured on the ERROR screen, two of them unwrapped; the gate over-reports by ~70 characters and cannot fail on the surface the operator sees

**Where:** Task 4 Step 1's five `TestModalsThisBlockTouchesAreDrawnInFull` rows.

That test's loop hard-codes the renderer: `assertModalBodyFits(t, tc.what, errorScreenBody, tc.body)`
(`gui/modal_fits_test.go:337`). But the fork has TWO renderers, and the convention is unambiguous —
`confirmWarningBody` is used for `composerConfirmBody(...)` bodies at three existing call sites
(`composer_shape_test.go:192`, `composer_discard_test.go:74`, `composer_selfcheck_test.go:176`).
Three of the plan's five new rows are ConfirmWarningScreen bodies: the hardened warning, the
SHA-256 warning and the §4.5 confirm modal all reach the screen through
`composerConfirmScreen(...)`.

**Measured counterexample** (I ran the fork's existing suite, `-run` scoped, read-only):

```
composer_hash_test.go:46   the §8i rule (errorScreenBody):        132 drawn, headroom 418  -> capacity 550
modal_fits_test.go:337     ms1 reminder (errorScreenBody):         71 drawn, headroom 476  -> capacity 547
modal_fits_test.go:337     incomplete-card, reader only:           52 drawn, headroom 513  -> capacity 565
composer_shape_test.go:192 §8a key-less confirm (confirmWarning): 166 drawn, headroom 339  -> capacity 505
composer_shape_test.go:192 §8b unsorted confirm (confirmWarning): 173 drawn, headroom 339  -> capacity 512
```

**The ConfirmWarningScreen holds ~45-60 normalised characters LESS than the error screen.** On top
of that, the two method-warning rows are tabled UNWRAPPED — `composerCopyHashlockHardenedWarning()`
rather than `composerConfirmBody(...)` — so they are measured 20 characters short of what is drawn
(169 vs 189 as drawn; 206 vs 226 as drawn), while the confirm row right beside them IS wrapped.
Combined, the gate over-reports headroom by roughly 70 characters for the two warnings and by
~50 for the confirm body.

Nothing is cut today by my estimate (the shortened confirm body is 290 and the SHA-256 warning 226,
both far inside ~450-505), so this is not a Critical. It is Important because this is the gate that
justified deleting two safety sentences from a funds-critical modal, and it cannot fail on the
geometry that modal actually has — the "a test that cannot fail on what it names" class.

**SUGGESTION.** Do not put these three in `TestModalsThisBlockTouchesAreDrawnInFull` (whose loop
you would have to change). Add them as direct calls in `gui/composer_hashlock_test.go`, following
`composer_shape_test.go:189-193`:

```go
for _, tc := range []struct{ what, body string }{
    {"the hashlock hardened warning (H2 §4.3)", composerConfirmBody(composerCopyHashlockHardenedWarning())},
    {"the hashlock sha256 warning (H2 §4.3)",   composerConfirmBody(composerCopyHashlockSHA256Warning())},
    {"the hashlock confirm modal, longest variant (H2 §4.5)",
        composerConfirmBody(composerCopyHashlockConfirm("b867db87..edbc96cb", "hardened", 100,
            composerCopyHashlockRelation(-1)))},
} {
    assertModalBodyFits(t, tc.what, confirmWarningBody, tc.body)
}
```

Leave the ms1 refusal and the §8h phrase form where they are — those two really are `showError`
bodies. Re-run and re-record the headroom numbers in the plan; the ones folded from the gate are
`errorScreenBody`'s and should be labelled as such until they are re-measured.

---

### I-4 — `Type 64 hex`'s Back behaviour change ships with no test, and the plan claims a test that does not exist

**Where:** Task 3 Step 2's note.

The plan states: *"Note the behaviour change for `Type 64 hex`'s Back … The test in Step 1 does not
cover it; **Task 4's harness tests do (Back from hex entry at creation keeps the path)**."*

**Counterexample: no such test exists.** Task 4 creates seven tests
(`TestHashlockPhraseRouteSetsTheCorpusDigest`, `…DoesNotNormalise`, `…BackContractKeepsThePath`,
`…DeclineThenHardenedTypesOnce`, `…PhraseRefusalsOnScreen`, `…MethodModalsFireOnCondition`,
`…ConfirmRelationLine`); none of them taps the hex row — the string `Type 64 hex` does not appear in
`gui/composer_hashlock_test.go` at all. Nor does any pre-existing test cover it: the only fork tests
that touch `composerHexEntry` (`gui/composer_gates_test.go:906, 951`) call it DIRECTLY, never through
`composerHashEdit`, so they cannot see the propagation change.

The change is real and funds-adjacent in the same way §4.6 says the phrase route's would be: today
a Back out of hex entry at path creation deletes the path and the EXPERIMENTAL key-less consent
with it; after this plan it must not. It ships untested, and the plan's claim to the contrary is
the reason nobody will notice.

**SUGGESTION.** Fold this into I-1's behavioural row test: enter `composerAddPath`, tap
`Type 64 hex`, Back out of hex entry, and assert `len(st.list.Paths) == 1` and `Hash == nil` (the
path survives and is still hashless), then that `Which hash?` is on screen again. Then delete the
false sentence.

---

### I-5 — The relation line's no-match branch is never driven through the flow; a mutation that always says "matches hash 1" passes every test

**Where:** Task 4 Step 3's match search; `TestHashlockConfirmRelationLine`; spec §4.5, §7.2,
journey C-2.

Spec §7.2 requires *"The confirm modal's relation line with 0, 1 and 2 payload records (matching
and not)."* The plan drives exactly one of those: two records, matching, asserting
`matches hash 1 in the payload`.

**Counterexample.** Mutate the search so the match index is never -1:

```go
match := 0                                  // was: match := -1
for i, d := range payload { if d == h { match = i; break } }
```

`TestHashlockConfirmRelationLine` still reaches `matches hash 1 in the payload` and passes. Every
other test loads no payload records, so the line is absent and nothing asserts on it. The copy
table pins the two STRINGS (`composerCopyHashlockRelation(0)` and, inside the confirm-body row,
`(-1)`), but nothing pins the SELECTION. Under that mutation the device tells an operator whose
device digest does NOT match the `hash:` record they packed with `ms hashlock` that it DOES — the
precise false-agreement signal the relation line was added (journey C-2) to prevent, and the one
signal the operator has before the H4 walk.

**SUGGESTION.** Parameterise `TestHashlockConfirmRelationLine` over three cases: (a) 2 records,
the second matching → `matches hash 2 in the payload` (this also pins the 1-based index against an
off-by-one, which the current single case cannot, since `matches hash 1` is what an index-0 bug
also prints); (b) 2 records, neither matching → `no hash: record in the payload has this digest`;
(c) 0 records → assert the confirm frame contains neither `matches hash` nor `no hash: record`.

---

### I-6 — §7.4's two strongest cases are missing, and the assertion that replaces one of them is near-vacuous

**Where:** Task 2 Step 1 (`TestDecodeMS1PreimageIsShapeExact`); Task 1 Step 2
(`TestKindRowPreimageDigest`); spec §7.1 and §7.4.

Spec §7.4 lists seven cases. The plan drives five. The two it does not drive are the two that
compare a VALUE against a cross-language constant:

* *"the corpus's `kind[0].ms1` → its `preimage_hex`"* — the plan decodes the plate and then asserts
  only `if x[0] == 0 && x[31] == 0 { t.Fatalf("preimage looks zero") }`. **Counterexample:** mutate
  the decoder's slice from `d[1:]` to `d[:32]` (a one-character off-by-one that returns the kind
  byte plus the first 31 preimage bytes). The result is `03abab…ab`, `x[0] = 0x03 != 0`, so the
  test PASSES — and every caller downstream gets a preimage shifted by one byte. The corpus gives
  the exact constant (`abababab…ab`) and the plan does not use it.
* *"the acceptance record's plate (`ms10hashsq0p7jaf…`, `ms-hashlock-H1-acceptance.md`) → the
  corpus anchor row's `hardened_x`"* — absent entirely. This is the only case that ties the fork's
  decoder to an artifact the host actually produced.

Separately, `TestKindRowPreimageDigest` in Task 1 does not test what its name says. Its whole body
is `x := mustHex(t, c.Kind[0].PreimageHex); if h := Digest(&x); h == x { t.Fatalf("Digest is the
identity") }` — it asserts that SHA-256 is not the identity function. The corpus carries
`kind[0].digest = 9a2db2e23f1504cd056606553ac049c5e718e8f9ce9233876df1a7a1821af885`, which I
verified is `SHA256(0xab * 32)`, and the plan's `corpus` struct does not even declare the field.
**Counterexample:** mutate `Digest` to `sha256(sha256(x))` — `h != x` still holds, the test passes.

Also in this class, at Nit weight: the negative case uses an ad-hoc hard-coded
`ms10entrsqqqq…` whose payload is short, rather than the corpus's `kind[0].entr32_pair_ms1`
(X = 0xab*32 under `Tag::ENTR`) that spec §7.1 names. The corpus row is strictly stronger: it is
33 bytes with the SAME data and only the prefix byte differing, so it is the only input for which
the prefix check is the sole discriminator.

**SUGGESTION.** Give `codex32/mspayload_test.go` the corpus (or, since `codex32` should not
depend on `hashlock`'s testdata, pass the three constants as named literals with the corpus row
they come from in a comment): assert `hex.EncodeToString(x[:]) == kind[0].preimage_hex`; use
`kind[0].entr32_pair_ms1` for the entr case; add the acceptance-record plate → anchor `hardened_x`
case. In `hashlock_test.go`, add `Digest string \`json:"digest"\`` to the `Kind` struct and assert
`Digest(&x) == mustHex(t, c.Kind[0].Digest)`.

---

### M-1 — The zero-state lead "Deriving. This takes about 10 seconds." can never be drawn

**Where:** Task 4 Step 3, `hashlockDeriveFlow`; spec §3, §4.4 (fidelity M-6, journey M-3).

The frame is drawn only inside `DeriveHardened`'s progress callback, which fires only AFTER
`d.Step(500)` — so on the first (and every) drawn frame `done >= 501` and `elapsed > 0`, and the
guard `if elapsed := time.Since(start); done > 0 && elapsed > 0` always takes the
`"About %d seconds left."` branch. `composerCopyHashlockDerivingLead()` is computed and
immediately overwritten on every frame; it reaches no screen. (The shipped `unlockDerive` has the
same shape, so this is faithful mirroring rather than a transcription error — but the spec presents
the lead as a visible screen element and it is not one, and the plan gives it a `composerCopyTable`
row that documents text the operator cannot see.)

**SUGGESTION.** Either draw it for real — hoist one frame before `DeriveHardened` with
`done = 0` — or record in the plan that it is the estimator's unreachable fallback, and drop its
copy-table row's implication that it is a screen.

### M-2 — `hashByPhrase` is never reset, so §8h's phrase form is wider than §4.7's predicate

**Where:** Task 4 Step 3 / `gui/composer_state.go`.

§4.7's predicate is *"every path is hashed **and at least one hash was set by phrase**"*. The plan
sets `st.hashByPhrase = true` on assignment and never clears it, and `composerState` is built once
per composition (`gui/composer_flow.go:34`). **Counterexample:** set path 1's hash by phrase, then
re-enter `Hash lock` and choose the payload row `hash 1` instead. No hash is now phrase-set, but at
Done the phrase form still fires and tells the operator to back up a phrase and reconcile it. The
field's own doc comment ("records that AT LEAST ONE path's hash was set through the phrase route")
is therefore not true of what the field holds.

The dangerous direction — a phrase-set hash with `hashByPhrase` false — is not reachable, so this
is a copy-accuracy defect, not a safety one.

**SUGGESTION.** Either recompute the predicate from the paths (record the method per path, e.g.
`Paths[idx]`-parallel `hashSource []bool` in `composerState`, cleared wherever `Hash` is
reassigned in `composerHashEdit`), or reword the field and its comment to what it is —
"a phrase was used somewhere in this composition" — and say in §4.7 that the form is deliberately
sticky.

### M-3 — Two bodies §4 and §7.2 name are absent from the fit table

Spec §4 requires *"the phrase screen's lead and refusals, both method modals, the confirm modal in
its longest variant, **the `Which hash?` no-payload lead**, the phrase-route §8h"* in the fit
table, and §7.2 repeats "the no-payload lead" by name. The plan tables five bodies and neither of
those two is among them (measured: the no-payload lead is 75 normalised characters, the phrase
lead 39). Consequence is low — `composerPickScreen` draws its lead as a wrapping body row rather
than in the 44 px band, and the phrase screen's lead band is self-sizing — but the requirement is
unmet and the plan is silent. **SUGGESTION:** add both, or state in the plan why neither is a
modal and what does bound them instead (e.g. an assertion on the drawn frame in
`TestWhichHashRowsAreLabelKeyed`, which today only greps the STRING — precisely what
`modal_fits_test.go`'s own header says a string assertion cannot see).

### M-4 — The anchor's digests and the `kind[0].ms1` plate are hard-coded in `gui`/`codex32` with no pin

`hashlockAnchorHardH`, `hashlockAnchorSHA_H` and the 75-character plate are literals in
`gui/composer_hashlock_test.go` and `codex32/mspayload_test.go` (the plate twice). I verified all
three are correct today. But §7.1's whole design is that values come from the pinned corpus, and
these do not: a corpus re-vendor that changed them would red `hashlock`'s sha pin and leave these
silently stale. **SUGGESTION:** derive them via `loadHashlockCorpusForGUI` (which the file already
has) rather than as constants, or add a one-line test asserting each literal equals its corpus row.

### M-5 — The lockstep array's "BOTH directions" clause is never driven

`lockstep[3]` says *"the fork's pin test drives these rows in BOTH directions (encode and
decode)"*, and spec §7.1 repeats it ("in BOTH directions where it says so"). The plan decodes
only; nothing encodes `[0x03][preimage]` back to `kind[0].ms1`. `TestLockstepListIsTheOneWeDrive`
asserts `len(Lockstep) == 4` — it counts the clauses without driving the fourth.
**SUGGESTION:** one assertion using the `NewSeed` call the plan already makes for the 17-byte
case: `NewSeed("ms", 0, "hash", 's', append([]byte{0x03}, x[:]...)).String() == kind[0].ms1`.

### M-6 — `d.Key()`'s documented nil is copied without a length check

Both `PreimageHardened` and `DeriveHardened` do `copy(out[:], d.Key())`. `Key()` returns **nil**
for a dead, zero-value or incomplete deriver, and `seal/pbkdf2.go` goes out of its way to say why
that matters: *"an all-zero key would be worse than none -- it is a VALID AES key and hides the
fault."* Here the equivalent is worse: a nil leaves `x` as 32 zero bytes, `Digest` yields
`SHA256(0x00 * 32)`, and the device would engrave a hashlock whose preimage is public. It is not
reachable as written (`NewDeriver` clamps `total >= 1`, `Step` returns true only at
`done >= total`) and CI's corpus comparison would catch it, so this is Minor — but the guard is
two lines and the package's own contract asks for it.
**SUGGESTION:** `k := d.Key(); if len(k) != 32 { return x, false }` in `DeriveHardened`, and the
same shape (returning a bool, or panicking at the pure function) in `PreimageHardened`.

### M-7 — Records the plan changes but does not reconcile

* `cmd/emu/sysw_composer_payload.go:8-16` states *"in the emulator the composer's key sources are
  empty, its hash-lock picker has **nothing to offer but `Type 64 hex`**"*. Task 3 makes that
  false — the picker gains `Type a hashlock phrase` for every payload, including that blob.
  Task 6's records list does not mention it.
* `design/FOLLOWUPS.md` carries **F-466 `on-device-preimage-entry`** ("the operator asks to type
  the passphrase/preimage on the SH2 and have the device double-hash it into the hashlock"), which
  is the item this stage closes, and an entry at :15576 whose last sentence — *"Screen hint:
  `Which hash?` with no `hash:` record loaded gains a lead line naming the host route"* — is
  exactly Task 3's no-payload lead. Task 6 files new H3 pointers but reconciles neither.
  **SUGGESTION:** add both to Task 6, and fix the emu comment in Task 3's commit.

### N-1 — `hashlock.Salt` is an exported mutable `var []byte`

`var Salt = []byte("ms-hashlock-v1")`. Any package could append to or overwrite it and every
digest would diverge silently at runtime (CI would catch it; the device would not). The
mutation table depends on it being a `var`, so this is a genuine trade-off, not an oversight —
worth one comment saying so. Consider returning a copy from a `salt()` helper, or a
`const saltString` with `[]byte(saltString)` at each call site.

### N-2 — The counter counts bytes where the keyboard's readout counts runes

`widget.Labelf(..., "%d/%d", len(kbd.Fragment), hashlock.PhraseMaxChars)` vs the keyboard's own
`utf8.RuneCountInString(k.Fragment)` for the masked readout. Identical for printable ASCII, which
is all the keyboard emits; noted only so a future non-ASCII keyboard does not inherit a silent
disagreement between the counter and the mask.

### N-3 — §7.5's "digests" plural vs one digest

Spec §7.5 asks the mixed-case run to show *"the corpus's mixed-case digests, not the anchor's"*;
Task 5 Step 1 walks it under SHA-256 only, so one digest. Adequate (a case fold changes both), but
the plan and the spec should read the same.

### N-4 — Duplicate payload digests make the relation line arbitrary

If a payload carries two identical `hash:` records, the search `break`s at the first, so the modal
says `matches hash 1` while `hash 2` is equally a match. Harmless, worth a comment.

---

## Notes on scope

* I did not re-derive the build gate's compile fixes; per the brief I checked only whether a gated
  fix changed a block's MEANING. Two did, both legitimately and both disclosed: the §4.5 drop
  order (fixes 3 and 4 — verified above by reproducing 504 / 384 / 290 / 254 myself) and the
  `hashHex` → `hashlockHashHex` rename (verified: `gui/seal_fixture_test.go:172` declares
  `func hashHex(h [16]byte) string`, so the round-0 name really was a redeclaration).
* `scripts/h2-plan-blocks-vs-tree.sh`'s 25/25 PASS means the plan's blocks ARE the gated tree; every
  finding above is therefore a finding about the gated tree too, not a transcription slip.
* Task 5 Step 1 (the emulator walk) remains prose, and the plan says so plainly. Its four
  assertions are correct against the corpus (I recomputed all of them). It is the stage's only
  un-gated executable artifact and, per spec §8, the acceptance until the operator walks it —
  so it should be written and RUN before the post-implementation review, not after.
* `default: panic(...)` in `composerHashEdit` is spec §5's own ruling and is unreachable
  (`composerPickScreen` bounds `sel`), so I did not raise the fork's "on a device a panic is a
  brick" doctrine against it. Recorded here only so the next round does not re-open it.
