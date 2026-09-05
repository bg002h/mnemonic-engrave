# H5 plan R0 round 0 — journey walk (opus)

**Artifact:** `design/IMPLEMENTATION_PLAN_hashlock_H5_device_polish.md` at engrave
master `0c2b13e`. Spec `design/SPEC_hashlock_H5_device_polish.md` (R0 GREEN
`e03d8e7`). Fork baseline `b9a9a30`.

**Method.** The gated tree `/scratch/code/shibboleth/.tmp/h5-gate` was copied to
`/scratch/code/shibboleth/.tmp/h5-journey` and driven on the `gui` touch harness
(`runComposerHashEdit` / `sessionHarness`, `sh2DisplaySize`) at each moment H5
changes, plus `composerShapeFlow` under `runUI` for the Done row, `md.Compose` +
`encodePayload` for what reaches the plates, and `seal.AdmitSection` for the
unlock refusal. The controller's three emulator runs were walked by applying the
plan's exact mutation strings to `gui/composer_hashlock.go` and re-running a Go
stand-in for `walk_hashlock_phrase.js` with the same four trials in the same
assertion order. The gated tree was never modified (verified with `diff -rq`);
every scratch test was removed afterwards. No browser was available, so the
walk's JavaScript itself was read rather than executed; the assertion ORDER was
executed in Go.

**Counts: 0 Critical / 2 Important / 5 Minor / 2 Nit.**

---

## Moment A — HOLD to the reconcile screen, and back

Driven end to end: `Which hash?` → phrase row → §8i rule → phrase screen → type
`correct horse battery staple` → `Which method?` → Hardened → confirm modal →
hold → reconcile screen → Back.

| step | in hand | device does | what else | class |
| --- | --- | --- | --- | --- |
| A0 `Path 1 hash` | an empty region, no payload | `No hash record in the payload. Type a phrase below, or make one with ms hashlock on the host.` + 3 rows | pick `Type 64 hex` / `No hash lock` — no reconcile screen, correct: nothing was derived here | not our concern |
| A1 §8i rule | — | `The hash must be SHA-256 of a 32-byte value…` | Back → returns to `Which hash?` | not our concern |
| A2 phrase screen | 28 chars typed | lead `This screen does that hashing for you. Use a phrase you have never used anywhere else.`, `28/100`, 28 masked chars | tap `show`; type >100 → OK refuses | not our concern |
| A3 method pick | the phrase | `Hardened (about 10 s)` / `SHA-256` | Back → phrase kept | not our concern |
| A4 confirm modal | phrase + method | `hash 3cf5d421..b70a4c12` / `method: hardened chars: 28` / **`Write down this phrase, the method and this digest now. They are not on this device and not on your plates. Without both, this path can never be spent.`** / reuse line / `Hold button to confirm.` | **the operator reads three items then "both"; and the digest IS on the plates** | **finding I-1** |
| A5 reconcile screen | phrase, method, digest written down | `hash 3cf5d421..b70a4c12` / `method: hardened chars: 28` / `Before you fund this wallet, run ms hashlock … and check the digest matches. If they differ, do not fund this wallet: build it again.` | act now (before plates are cut) vs. "before you fund" — minutes and steel apart | **finding M-2** |
| A6 Back on A5 | — | dismisses exactly as the checkmark does; `composerHashEdit` returned `true`, path holds `3cf5d421…b70a4c12`, phrase set has 1 entry | an operator reacting to "build it again" may press Back expecting an undo; there is none, and the manual's Back table has no row for this screen | **finding M-5** |

Verified sound: the reconcile screen is reached for a hardened trial with a token
(`3cf5d421..b70a4c12`) no other row of the file produces; `chars: 28` is present;
the header is spelled identically on both screens; the fit gate logs 186 drawn /
339 headroom for the longest variant, reproduced independently here.

## Moment B — Done's banner on four wallets

`composerCopyHashEveryPathFor` driven for each composition, and the banner drawn
through the real `Done` row of `composerShapeFlow`.

| wallet | everyHashed | byPhrase | banner drawn | class |
| --- | --- | --- | --- | --- |
| all-phrase, 2 paths, 2 phrase digests | true | true | phrase form | `every preimage plate` names an artifact this wallet has none of → **M-4** |
| mixed: 1 phrase path, 1 payload-row path | true | true | phrase form (`every phrase and its method, and every preimage plate`) | correct — this is F-480's own case, fixed |
| the phrase path removed, survivor re-hashed as 64 hex | true | false | plain form | correct |
| the same digest re-typed as 64 hex | true | true | phrase form | correct — the phrase was derived here and the backup burden is unchanged |
| **2 paths, two DIFFERENT hex/payload digests, no phrase** | true | false | plain form: `Back the preimage up separately.` | singular, for two preimages → **finding I-2** |

Reachability checked rather than assumed: `md.ValidatePathList` requires
`anyKeyed`, so an all-keyless hashed composition is refused at Done
(`Every wallet needs at least one path with a key.`) — the plan's fixtures are
keyless and therefore assert on §8h's *guard* only. §8h **is** reachable: a path
that is both keyed and hashed validates, `Hashlock` is offered on a keyed path,
and driving `composerShapeFlow` → `Done` on `{1 key + hash, hash only}` drew the
phrase form. Not a finding; recorded because nothing in the plan drives §8h
through the flow.

## Moment C — the phrase screen's lead and readout at sh2 size

| step | in hand | device does | what else | class |
| --- | --- | --- | --- | --- |
| C1 blank screen | nothing typed | lead wraps in the band `left=8 width=411`, measures `(407,44)` = 2 lines of 23 px; `0/100`; no ink in any nav rectangle | the mutation (panel-wide wrap) puts ink at `(431,52)` inside button `(427,44)-(480,97)` and measures 440 px | fixed, gated |
| C2 3 chars | `abc` | `***`, `3/100`, `MaxHeight=209 grid=(340,182) gap=8 → budget 19 px, one line 19 px` | budget is exactly one line — no slack, which the plan states as the intent | not our concern |
| C3 60 chars | 60 typed | 56 asterisks drawn (tail-clamped); `show` reveals the last 34 characters only | an operator proof-reading a long phrase cannot see its head | documented (manual §"The readout, and how much it shows"); pre-existing, not H5's |

§3.3's fallback copy is not used and the plan says so with the measured line
count, which the gate re-logs on every run. F-481 does not regress: 19 ≥ 19.

## Moment D — the unlock refusal

| step | in hand | device does | what else | class |
| --- | --- | --- | --- | --- |
| D1 | a sealed payload with a preimage plate at record 1 | after the ~31 s KDF: `Record 1 is a hashlock preimage, not a seed. This payload cannot be unlocked here. Nothing was opened. Remove that record (records count from 0) on the host and seal the payload again.` | try another on-device route to engrave the plate — every route refuses (`unlockEngraveCodex32`, `codex32_polish.go`), so "remove it" does not send the operator away from a working path | verified sound |
| D2 | a payload with TWO preimage plates | refuses the FIRST offender only; `AdmitSection` returns on the first refusal | remove it, re-seal, retry → another ~31 s and another refusal, and **the index has shifted** | **finding M-3** |
| D3 | the longest noun at a two-digit index | `Record 13 is not a format this machine reads. … Remove that record (records count from 0) …`, 153 chars, headroom 397 | — | fits |

The `(records count from 0)` clause is verified load-bearing: `seal/record.go:69`
declares `Index` 0-based and the fixture's record 0 is a seed.

## Moment E — the controller's three emulator runs (Task 5 Step 12)

**The mutation needles are exact and unique.** In the gated tree,
`grep -cP '^\t\t\th := hashlock\.Digest\(&x\)$' gui/composer_hashlock.go` = 1
(line 64, and the only `hashlock.Digest` call in non-test code), and
`grep -cP '^\t\t\t\td := h$'` = 1 (line 68). Both plan strings apply verbatim.

**The claimed failing assertions are the ones that fire, and no earlier one.**
Executed by replaying the walk's four trials and its assertion order in Go:

| run | applied | result |
| --- | --- | --- |
| (a) unmutated | — | W1–W8 all pass; stored = `3cf5d421…b70a4c12`; reconcile screen carries the token, `chars: 28`, `If they differ` |
| (b) `h := hashlock.Digest(&x); st.list.Paths[idx].Hash = &h` | trials 1–3 store their digests too | **W1–W4 still pass** (the confirm modal is byte-identical: `hashlockOtherPathLine` skips `idx`, the payload is empty, and `composerHashRows` does not depend on whether the path already holds a hash) and the run fails at the pre-hold read, exactly as claimed |
| (c) `d := h; d[0] ^= 1` | — | pre-hold read still `null`; fails at the stored-versus-displayed assertion (`stored=3df5d421…`, corpus `3cf5d421…`) and at no earlier one |

Run (c) also demonstrates *why* the assertion is needed: the reconcile screen
under (c) still draws `3cf5d421..b70a4c12`, because `showError` renders `h` and
the policy holds `d`. The operator's own reconciliation cannot see this class;
only the walk can. The design is right.

**Could a run pass for the wrong reason?** Two ways, one closed and one open:

- *Closed.* A stale `emu.wasm` cannot silently skip the new assertions: `run()`
  checks all seven `sh*` globals up front and `pathHashes()` throws both when
  `shComposerPathHashes` is missing and when it returns `null`.
- *Open.* Nothing in Step 12 reverts a mutation between runs → **finding M-1**.

---

## Findings

### I-1 — the write-down edit makes the next sentence false about the digest, and leaves "both" with a three-item antecedent

**Where.** Task 2 Step 4, `gui/composer_copy.go`, `composerCopyHashlockConfirm`;
spec §1.2; propagated to H2 §4.5 by Task 6 Step 1 and to the toolkit manual by
Task 6 Step 4.

The body now reads, drawn (Moment A4, verbatim from the harness):

```
Write down this phrase, the method and this digest now. They are not on this
device and not on your plates. Without both, this path can never be spent.
```

**(a) "not on your plates" is false of the digest.** The digest is compiled into
the descriptor as `sha256(H)` (`md/compose.go:403`), the wire encoder writes all
32 bytes (`md/encode.go:218-222`), and the md1 the composer engraves carries them
verbatim. Constructed in `package md` on the gated tree's own code:

```
wire payload (43 bytes): 2012d84212a001824cd2dd3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12
the FULL 64-hex digest appears verbatim in the engraved payload: true
engraved as: md1fyjztqspqztvyyy4qqxpye5ka8n6aggw2725u36uauyr88y0k86z9lcz md1fyjztqsw5qzrxaf75whnt49ugvxaszear0jms5nqjdf3qfyhfjdta5
```

Before H5 the sentence's subject was "this phrase and the method", and it was
true. H5 adds "and this digest" to the list without narrowing the "They" that
follows, so the screen whose job is to define the backup burden now asserts that
the one item which *is* recoverable from the plates is not on them. The sibling
body H5 edits in the same leg gets this right — §8h's "It is not on this device
and not on these plates" refers to the *preimage*, which is correct.

**(b) "Without both" now has three candidate antecedents.** The spec reasons that
"'both' keeps meaning the phrase and the method". That is the author's reading.
An operator reading top-down meets three items and then "both", and the
nearest-pair reading ("the method and this digest") makes **the phrase look
optional** — on the one screen that exists to stop an unspendable path. The
shipped H2 text had no such ambiguity because its list had exactly two items.

**SUGGESTION.** Scope the second sentence instead of leaving "They" and "both"
pointing at a three-item list. The spec rejected the obvious repair on fit, and
that measurement reproduces here — but the constraint is a *line* budget, not a
character budget, and shorter scoped forms clear it. Measured with the gated
tree's own `assertModalBodyFits` renderer at `sh2DisplaySize`, longest variant
(`hardened`, `chars: 100`, both relation lines), margin 80:

```
H2 shipped (pre-H5)              drawn=true (336/336) headroom=107  PASSES=true
H5 as planned                    drawn=true (347/347) headroom=107  PASSES=true
"…The phrase and method are not on this device and not on your plates…"
                                 drawn=true (361/361) headroom= 64  PASSES=false
"…Without the phrase and method, …" (the spec's rejected repair)
                                 drawn=true (361/361) headroom= 64  PASSES=false
"…The first two are not on this device or your plates. Without both, …"
                                 drawn=true (348/348) headroom=107  PASSES=true
"…The phrase and method are not on this device. Without both, …"
                                 drawn=true (343/343) headroom=107  PASSES=true
```

The fifth row keeps the operator's ruled first sentence **byte-identical**, makes
the claim true of exactly the two things it is true of, and gives "both" a single
unambiguous antecedent, at the same headroom (107) as the planned text. It is
offered as a proof that a gate-passing repair exists, not as the wording to
adopt; the ruled-remedy owner should choose it.

### I-2 — §8h's PLAIN form keeps the singular undercount H5 removes from the phrase form

**Where.** `gui/composer_copy.go`, `composerCopyHashEveryPath()` — unchanged by
the plan; spec §2 item 5 changes only `composerCopyHashEveryPathPhrase`.

H5 §2 item 5 changes the phrase form from "Back up **the** phrase and its method,
**or** the preimage plate, separately" to "Back up **every** phrase and its
method, **and every** preimage plate, separately", on the stated ground that a
choice is *"an undercount at the one screen whose job is to say what spending
needs"*. Its sibling has the identical defect and is left standing. Constructed
on a composition that validates and reaches Done:

```
paths: {1 key + hash ab..ab, hash only 5a..5a}   everyHashed=true byPhrase=false
banner: "HASH ON EVERY PATH\nEvery way to spend this wallet needs the preimage of
a hash. It is not on this device and not on these plates. Back the preimage up
separately."
```

Two paths, two different digests, two different preimages the operator must hold
— and the screen names one. The wallet shape is not exotic: it is the mixed
wallet of finding I-3 with the phrase path replaced by a second plate, and both
H5 §2's own reasoning and the phrase form's new "every … and every" say it needs
counting.

This is in scope: H5 §2 item 5 is *the* fold that makes the two forms inconsistent
with each other, and `composerCopyHashEveryPathFor` — which chooses between them
— is rewritten by Task 1 Step 8.

**SUGGESTION.** Give the plain form the same plural, and add a row + mutation
beside `TestComposerMixedWalletBannerNamesEveryPhraseAndEveryPlate`. Measured on
`errorScreenBody` at `sh2DisplaySize`:

```
shipped plain form     drawn=true (131/131) headroom=397 PASSES=true
"Back up every preimage separately."
                       drawn=true (133/133) headroom=397 PASSES=true
```

Headroom is unchanged at 397. If the operator rules it out of scope, file it with
an owning phase rather than leaving the two forms disagreeing.

### M-1 — Step 12 never reverts a mutation, and run (c) then fails at run (b)'s assertion

**Where.** Task 5 Step 12. The table gives two edits to `gui/composer_hashlock.go`
and says each is "rebuilt with `./cmd/emu/build.sh`"; there is no step that
restores the file between runs, and no clean-tree check.

With (b) still applied when (c) is added, the walk throws at the **pre-hold**
assertion and never reaches the stored-versus-displayed one. Constructed:

```
both (b) and (c) applied
W5 PRE-HOLD stored = 3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12
W5 FAILED (`the path ALREADY holds a hash while the confirm modal is up`)
W6 POST-HOLD stored = 3df5d421…   (the JS walk never gets here: it throws at W5)
```

Spec §4.5(c) exists precisely because *"without (c) that assertion is never shown
able to fail"* (journey I-7). A contaminated run (c) would be recorded as a FAIL
— satisfying spec §7's "all three walk runs recorded" — while leaving that
assertion exactly as unproven as before.

The plan does state the discriminator ("must FAIL … **and at NO earlier one**"),
which is why this is Minor and not Important: a controller who reads the assertion
name catches it. It is still a procedure whose only guard is attention.

**SUGGESTION.** Add to Step 12: `git checkout -- gui/composer_hashlock.go &&
git diff --quiet gui/composer_hashlock.go` before each mutation and after run (c),
with `./cmd/emu/build.sh` re-run after the final restore. One line, and it makes
"at NO earlier one" enforced rather than observed.

### M-2 — the reconcile screen names the funding threshold; the plates are cut first

**Where.** Task 2 Step 4, `composerCopyHashlockReconcile`; spec §1.1.

The screen says *"Before you fund this wallet, run ms hashlock … If they differ,
do not fund this wallet: build it again."* It is drawn inside
`composerShapeFlow`, which is followed in the same `composerFlow` by the stub
screen, seating, and engraving — roughly 21 minutes per plate. A divergence
found after the plates are cut costs every plate, because the digest is *in* the
engraved md1: recomposing the same policy with one bit changed in the digest
produces different chunks (`md1fyjztqs…` vs `md1fgltpqs…`, measured). "Before you
fund" is the funds-safety-correct deadline, but the operator is standing at the
cheapest moment to act and the screen points them at a later one.

**SUGGESTION.** Name the nearer threshold. The reconcile body has 339 characters
of headroom, so it is free — measured:

```
H5 as planned                     drawn=true (186/186) headroom=339 PASSES=true
"Before you cut plates, run ms hashlock …"
                                  drawn=true (181/181) headroom=339 PASSES=true
"Before you cut plates or fund this wallet, … the plates carry this digest."
                                  drawn=true (202/202) headroom=339 PASSES=true
```

Classified as a **default/copy** change, not a refusal: nothing should block, and
the operator may reasonably prefer the single funding threshold. Record the
decision either way, because the plates-carry-the-digest fact is the same one
that makes I-1 wrong.

### M-3 — the unlock refusal is singular, one record at a time, and the index shifts between rounds

**Where.** Task 4 Step 2, `unlockNotPermittedBody`; spec §5.

`AdmitSection` returns on the first refused record (`seal/record.go:319-325`), so
a payload carrying more than one preimage plate is refused once per plate, each
round costing the ~31 s KDF and a host re-seal. The device's own H5 copy
contemplates the plural case: §8h's phrase form now says "every preimage **plate**".
Constructed on `[seed, plate, seed, plate]`, the operator following the
instruction each round:

```
round 1: 4 record(s) -> refusal names Record 1 (Preimage=true)
round 2: 3 record(s) -> refusal names Record 2 (Preimage=true)
round 3: admitted
```

The index moves. The screen tells the operator the number is 0-based — which is
what makes them trust it — and says nothing about it being valid only for the
payload in front of them. An operator applying round 2's `Record 2` to their
*original* listing deletes original record 2, a seed.

**SUGGESTION.** Either say there may be more than one (`Remove that record — and
any others like it — (records count from 0) on the host and seal the payload
again.`), or have the refusal name every offending index rather than the first.
The second is a `seal` change and larger; the first fits inside 397 characters of
headroom. Documentation-only is also defensible; file it with an owning phase if
so.

### M-4 — the phrase form names "every preimage plate" on an all-phrase wallet that has none

**Where.** `composerCopyHashEveryPathPhrase`, spec §2 item 5.

Measured: an all-phrase wallet (2 paths, 2 phrase digests) draws the phrase form,
whose last sentence asks the operator to back up "every preimage plate". There is
no plate. The reverse case — a hex-typed hash, which has no *plate* either — is
named the same way. The wording is right for the mixed wallet it was written for
and vacuous or slightly wrong for the two pure cases.

**SUGGESTION.** No change is clearly better than the current text: the sentence
overcounts, which is the safe direction, and a form that counted would need three
variants. Record the decision (a sentence in `composer_copy.go` beside the "every
… and every" comment) so the next reader does not re-open it. Nothing gates.

### M-5 — the manual's Back table gains no row for the reconcile screen, and its lead sentence is false there

**Where.** Task 6 Step 4 edits `docs/manual/src/40-cli-reference/43-ms.md` at
`:482-483` and `:501-502` (both citations verified exact at toolkit `46b40bb`) but
leaves the `#### What Back does` table at `:514-525`.

That table opens *"Every Back inside the route moves one step back within it and
keeps the phrase. Only Back at the phrase screen leaves."* Verified on the
harness: Back at the reconcile screen **dismisses**, `composerHashEdit` returns
`true`, and the hash stays assigned — it moves no step back and it is not the
phrase screen. The device behaviour is correct and deliberate (F-440: `back`
dismisses exactly as `ok` does on every dismiss-only modal, and `ErrorScreen.Layout`
returns one boolean with nothing to skip). The manual simply does not cover it,
and H5 is the change that gives the operator a reason to press Back there — the
screen now ends with *"do not fund this wallet: build it again"*, and the reflex
after reading that is to look for an undo.

**SUGGESTION.** In Task 6 Step 4, add a row —
`| the reconcile screen | the spend-path list | dropped; the hash is already assigned |`
— and qualify the lead sentence with "inside the route, before the hold". Same
commit, same `make lint`.

### N-1 — the widened `ok`-shape guard logs a claim it does not check

**Where.** Task 5 Step 6, `cmd/emu/needle_test.go`.

`okSetRe` matches a bare `true|false` right-hand side and logs
`"%s sets \`ok\` to %s after its last assertion, so it restates nothing (H5 §4.4)"`,
then `continue`s. Nothing checks that the assignment is after the last assertion.
A walk with `out.ok = true;` near the top now clears the guard with a
positive-sounding log line, where before H5 the same file reported INCONCLUSIVE
(a `t.Errorf`). The widening is necessary — the old regex could not read the
assignment shape at all, which is the pre-existing RED Task 5 fixes — but the new
arm's message asserts the one property §4.4 introduces without measuring it.

**SUGGESTION.** Either soften the log to what is checked ("sets `ok` to a
constant, so it restates nothing"), or add the cheap positional check: the index
of the `.ok =` match must exceed the index of the last `throw`/`must(`/`await` in
the file. Non-blocking.

### N-2 — nothing tells the operator where the host's character count is

The reconcile screen's `chars: <n>` does have a host counterpart — verified in
`mnemonic-secret` at `504ff46`: `ms hashlock` prints `phrase_chars` in JSON
(`crates/ms-cli/src/cmd/hashlock.rs:329-331`) and `phrase: N characters` on the
engraving card (`:351`). The plan's code comment ("the host card's `phrase_chars`")
is therefore true. But the manual passage Task 6 Step 4 leaves in place quotes only
the `hash:` line from stdout and says "its first and last eight characters are what
the confirm screen showed" — the count is never mentioned, so the field the
reconcile screen newly carries has no documented counterpart.

**SUGGESTION.** One clause in the same manual edit: the card on stderr also prints
`phrase: 28 characters`, which is what `chars:` reconciles against.

---

## What was checked and found sound

- The reconcile screen's operand, method, char count and mismatch sentence are all
  drawn at `sh2DisplaySize` in the hardened trial, with a token no other row of the
  file produces; 186 drawn / 339 headroom reproduced independently.
- The header-equality gate is a real gate: `normalizeDrawn` strips whitespace, so
  no frame assertion in the package can see the `method: %s   chars: %d` spacing —
  the plan says this and it is true.
- `composerAnyPathByPhrase` gives the right answer on all four wallets the brief
  names, plus the two-paths-one-digest and re-typed-as-hex cases, and the plain
  form is chosen for the removed-and-re-hashed wallet.
- §8h is reachable through the real `Done` row on a composition that validates.
- The unlock refusal's "remove that record" does not divert the operator from a
  working device route: every on-device route to engrave a preimage plate refuses.
- Both plan mutation strings are exact and unique in the gated tree, and each fails
  at the assertion the plan names and at no earlier one.
- All eleven of the plan's new tests pass on a copy of the gated tree, and the
  geometry gates log `band left=8 width=411; lead (407,44) = 2 line(s)` and
  `readout budget 19 px; one line is 19 px`.
