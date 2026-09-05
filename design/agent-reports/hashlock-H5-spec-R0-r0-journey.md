# H5 spec — R0 round 0, JOURNEY WALK (opus)

**Artifact:** `design/SPEC_hashlock_H5_device_polish.md` at engrave master `f6dd437`.
**Ground:** fork main `b9a9a30` (worktree `/scratch/code/shibboleth/.tmp/h5-lens-journey`,
Go 1.26.7 at `/scratch/code/shibboleth/.toolchain/go`); parent spec `SPEC_hashlock_H2_device.md`;
host `me` built from engrave master (`me 0.8.1`); toolkit manual at `46b40bbb`.
**Lens:** one question — walk the three moments this spec changes plus the controller's
run of the changed walk, and find where the spec is SILENT at a step whose wrong outcome
is worse than telling the operator nothing.

**Counts: 0 Critical / 7 Important / 5 Minor / 2 Nit.**

---

## A note on what I measured, and one side effect to know about

Every number below came from a command in this session, not from a doc comment.

`/scratch/code/shibboleth/mnemonic-engrave/target/release/me` was a **stale `me 0.7.0`**
binary from Aug 31, not the `me 0.8.1` the brief expected. I rebuilt it
(`cargo build --release -p mnemonic-engrave --bin me`, 13.32 s) so Moment 3 could be
walked against the real host. **That path now holds 0.8.1.** Nothing else was written
outside my worktree; no commits; the two scratch probe files I added under
`gui/` in the worktree were deleted and the worktree removed.

---

## Moment 1 — HOLD on the confirm modal → the new reconcile screen (§1)

| # | In hand | What the device does (as specified) | What ELSE they might do | Class |
|---|---|---|---|---|
| 1.1 | phrase + method on paper (the confirm modal's "Write down this phrase and the method now"), the digest on the glowing modal | HOLD assigns `st.list.Paths[idx].Hash` and draws `showError("Hash lock", …)` with `hash <first8>..<last8>` / `method: <m>` / "Write this digest beside…" | — | — |
| 1.2 | the reconcile screen up | it waits for a dismiss; `hashlockPhraseRoute` then `return hashlockAssigned` | **dismiss without writing the digest.** Recoverable? **YES** — the consent screen (`composerConsentFlow`, `gui/composer_flow.go:98`, unavoidable before `composerEngraveStep` at `:101`) prints `  hash <first8>..<last8>` per path, in the *identical* form. Verified: `composerDigestShort` and `hashlockFirst8Last8` return the same string (probe log: both `"00010203..1c1d1e1f"`). The spec never says so | documentation only → **M-4** |
| 1.3 | paper: phrase, method, digest | — | **they wrote the phrase with a stray trailing space.** The one signal H2 §4.5 designed for this — `chars: <n>` — is on the confirm modal, which is gone, and §1's reconcile body drops it | **I-2** |
| 1.4 | at the host | — | run `ms hashlock --hashlock-phrase-stdin < phrase.txt`; it prints `hash:3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12` (manual `43-ms.md:367-368`). The device wrote `3cf5d421..b70a4c12`. The screen does not say to compare the ends; the method strings (`hardened`/`sha256`) *are* the host's `--method` values verbatim, so that half maps cleanly | documentation only → **N-2** |
| 1.5 | two digests that **disagree** | **nothing.** §1's body ends at "check the digest matches" | fund it anyway; retry on the host; re-run the phrase route. The plates are already cut and the engraved digest is the wallet's — the phrase they wrote does not open it | **I-1** |
| 1.6 | wanting to re-enter the hash editor | `Path N hash` re-opens; rows are the payload's `hash:` records + `Type a hashlock phrase` + `Type 64 hex` + `No hash lock` (`composerHashRows`, `gui/composer_hash.go:157-176`) | the phrase route *does* let them retype, and re-HOLDing replaces `p.Hash`. The **old** hash is nowhere on that screen — only on the consent screen and the path row's bare `+ hash` (`composerPathLine`, `gui/composer_state.go:257-281`) | not our concern (replacement is the remedy) |

## Moment 2 — Done's §8h banner under §2's per-digest provenance

| # | Composition | `composerAnyPathByPhrase` | Banner drawn | Right? | Class |
|---|---|---|---|---|---|
| 2.1 | three hashlocks, all phrase-set | true | phrase form | yes | — |
| 2.2 | ditto, one phrase path **removed and re-added from a payload row** carrying a different digest | true (the other two still match) | phrase form | yes — and this is exactly what §2 buys over `hashByPhrase` | — |
| 2.3 | **all three** replaced by payload rows | false (`phraseDigests` is stale but nothing matches) | plain form | yes — §2.1's "a value set cannot go stale" holds | — |
| 2.4 | a phrase digest **re-typed as 64 hex** on another path | true | phrase form | yes, per §2.4 | — |
| 2.5 | every hash from payload rows, all made on the host with `ms hashlock --hashlock-phrase-stdin` | false | plain form: "Back the preimage up separately" | the host **does** hand them a preimage plate string (`--out FILE`, manual `43-ms.md:337`), so the noun exists | — |
| 2.6 | **MIXED: path 1 phrase-set, path 2 from a payload row** | true | phrase form: *"Back up the phrase and its method, **or** the preimage plate, separately."* | **NO.** Path 1 needs the phrase; path 2 needs the plate. "or" reads as a choice | **I-3** |
| 2.7 | two paths, two **different** phrases | true | same phrase form — "the phrase … its method", singular | undercounts; the confirm modal's other-path line said "back up **every** phrase" and Done retracts it | **I-3** |
| 2.8 | `composerState` reuse across compositions | n/a | — | fresh per composition (`gui/composer_flow.go:34`: `st := &composerState{…}`), so no digest leaks between wallets | — |

## Moment 3 — the unlock refusal's next-step sentence (§5)

| # | In hand | What the device does | What ELSE / what the host does | Class |
|---|---|---|---|---|
| 3.1 | a sealed payload whose encrypted record 1 is a preimage plate | ~31 s KDF, then `unlockNotPermittedBody`: "Record 1 is a hashlock preimage, not a seed. This payload cannot be unlocked here. Nothing was opened." + §5's new "Remove that record on the host and seal the payload again." | — | — |
| 3.2 | that sentence | — | **Is `me seal` the verb?** Yes: `me seal` exists (`me --help`: "seal  Encrypt a payload for delivery to SeedHammer II flash") | — |
| 3.3 | — | — | **Does the host refuse to seal a plate anyway (H1b)?** Yes, verified: `printf '%s\n' ms10hash… \| me seal --seal-secret --out /tmp/h5a.uf2` → `me: this record is a hashlock PREIMAGE plate (kind 0x03), not a seed record; this container cannot place one yet…`. That does **not** make §5's sentence false — it says *remove, then seal*, and the host accepts the payload once the record is gone. The refused payload can only have come from `me ≤ 0.8.0` or a hand-built container | not a finding (recorded as verified) |
| 3.4 | "**Remove that record**" and "Record 1" | the screen never says how records are numbered | `seal.RecordNotPermittedError.Index` is `// 0-based, as \`me\` counts records` (`seal/record.go:69`), and `me`'s own messages append "(records count from 0)" **13 times**. `grep -rn "records count from\|count from 0" gui/ seal/` on the fork: **zero hits**. A 1-based reading deletes the record *above* the preimage — in the fixture, the operator's seed (`gui/unlock_preimage_test.go:40`: `[]string{d.Secret[0], guiPreimagePlate}`) | **I-4** |
| 3.5 | a fresh UF2 | — | `me seal` **generates a new passphrase** every run ("The passphrase is GENERATED and printed to STDERR"), so the one they just typed stops working; and they must re-flash. Neither is said | **M-5** |
| 3.6 | a non-preimage refusal (descriptor / address / debug command / a secret in the public section) | the same shared `unlockNotPermittedBody` gains the same sentence | "remove" is the right remedy for those, and in any case `me seal` refuses all of them first (verified: descriptor → uppercase-byte refusal; address → `unrecognised record: unrecognized HRP 'bc'`; `debug:` → `not a bech32 string`; a seed on `--plaintext` → the argv guard) | not our concern |

## Moment 4 — the controller runs the changed walk (§4)

| # | The controller has | What §4 says | What happens | Class |
|---|---|---|---|---|
| 4.1 | a branch build, fresh port, playwright | §4.1: `window.shComposerPathHashes()` in "`cmd/emu`, js build only; no production code path" | the state is `st`, a **local in `gui.composerFlow`**. `cmd/emu` cannot reach it. The sanctioned seam is an untagged package var in `gui` (`passphraseWidgetHook`, `gui/passphrase_flow.go:26`, "nil in production"), and `gui` has **no** `//go:build js` production file (`grep -rln "go:build js" gui/` → only `chain_class_walk_test.go`) | **I-6** |
| 4.2 | the confirm modal up | §4.2: read the hashes **before** the hold | **readable — yes.** The walk already calls into Go while that modal is up (`walk_hashlock_phrase.js:318`, `waitFor(…)` → `shScreen`), and `composerConfirmScreen` parks in `ctx.Frame` each iteration, yielding to the JS event loop | — |
| 4.3 | the same modal | §4.2's placement | "BEFORE the hold" is not pinned to *while the modal is up*. A read placed at the top of trial 4 would pass the §4.5 mutation. §4.5's required outcome ("must FAIL on §4.2's pre-hold assertion") is what corrects it, so it is self-healing — but only if the controller checks *which* assertion failed | documentation only → **M-1** company; called out under I-7 |
| 4.4 | `shTargets()` | §4.3: the phrase row is chosen **by LABEL** from `shTargets` | **impossible today.** `frameTargets` returns `[]image.Rectangle` (`cmd/emu/screen.go:92-119`) and `shTargets` maps it to `{x,y,w,h,cx,cy}` (`screen_js.go:65-77`) — **no text, no tag**, and `screen.go:95-98` explains why tags are deliberately dropped ("a tag is a live pointer into GUI state and this struct outlives the frame") | **I-5** |
| 4.5 | the mutated build | §4.5: HOLD assignment moved before the confirm → the walk must fail | it does: with the assignment before `composerConfirmScreen`, the pre-hold read (modal up) sees non-null. Trials 1-3 also leave the hash assigned after their Backs, so the failure is over-determined | — |
| 4.6 | the stored-vs-displayed assertion | §4.2's other half; §6's row for §4 is only "the two walk runs of §4.5" | **no mutation is required for it**, and §6's own heading is "each with the mutation that must fail it". A hook that returned the *displayed* digest's source instead of `*st.list.Paths[idx].Hash` makes the assertion a tautology and F-485's second defect ships open under a green walk | **I-7** |
| 4.7 | the branch build | §1 rewrites the reconcile body to "run ms hashlock with **them** on the host" | `walk_hashlock_phrase.js:318` waits for the literal `"run ms hashlock with this phrase"`. §4 does not list it | **M-1** |

---

## Findings

### I-1 — the reconcile screen tells the operator to check, and never says what a mismatch means or what to do

§1's body ends at *"…run ms hashlock with them on the host and check the digest matches."*
That is the last word the device gives them. The reconciliation happens **after the plates
are cut** ("Before you fund this wallet"), and at that point the engraved digest *is* the
wallet: a mismatch means the phrase on their paper does not open the path, and the only
remedies are to find the phrase they actually typed or to build and re-cut the wallet.
Nothing on the device says either.

The consequence is documented — but in the *host manual*, in another repo:
`mnemonic-toolkit/docs/manual/src/40-cli-reference/43-ms.md:507-510`, *"a phrase that does
not is a path you could not have spent, discovered now rather than at spend time."* Even
that says what the mismatch **means**, never what to **do**, and it is not on the paper
beside the digest.

Wrong outcome vs. silence: an operator who sees two different-looking strings and shrugs
funds a path that can never be spent. That is worse than the current silence, and worse
than being told.

Fit is not the constraint. Measured on the real gate, `errorScreenBody` at `sh2DisplaySize`:

```
H5 §1 reconcile, longest (hardened):        159 chars drawn in full, headroom 360 (margin 80)
H5 §1 reconcile + mismatch sentence:        205 chars drawn in full, headroom 320 (margin 80)
```

**SUGGESTION.** Add one sentence to §1 item 1's normative body, after "check the digest
matches": *"If they differ, do not fund this wallet: build it again."* Measured headroom
320, four times the 80-character margin. Add it to §1.3's fit row and to §6's §1 mutation
(drop the sentence → the flow test fails).

### I-2 — `chars: <n>` is on the screen that disappears, not the screen that now carries the write-down

H2 §4.5 states what `chars: <n>` is for, in its own words: *"the phrase's byte count — the
one signal that shows a stray space when the operator later reconciles against the host
card's `phrase_chars`"*. §1 moves the write-down instruction to the reconcile screen and
prints only `hash` and `method: <m>` there. The field designed for the reconciliation
moment is absent from the screen H5 built for the reconciliation moment, and the modal
that had it has been dismissed.

`ms hashlock`'s stderr card prints `phrase_chars`, so the host half of the comparison
still exists — the device half is what the operator can no longer write down. When the
digests disagree (I-1's branch), the char count is the one field that separates "I typed
a trailing space" from "I wrote the phrase down wrong".

Cost: `chars: <n>` is at most 11 characters against 360 of measured headroom.

**SUGGESTION.** §1 item 1's second line becomes `method: <m>   chars: <n>` — the same
spelling the confirm modal already uses (`composer_copy.go:410`), so the two screens
carry the identical line. Extend §1.3's fit row to `method: hardened   chars: 100` and
§4.2's walk assertion to the char count, which it already asserts on the modal
(`walk_hashlock_phrase.js:247`).

### I-3 — §8h's phrase form says "**or** the preimage plate" on exactly the wallets that need both

§2 makes the phrase form fire precisely when *some current path's* hash came from a
phrase. On a mixed wallet that is the correct predicate and the wrong sentence.
Constructed and run (worktree probe, `gui` package, `composerTwoPathList` with path 1's
digest phrase-set and path 2's taken from a payload row):

```
EVERY PATH HASHED = true
BANNER:
HASH ON EVERY PATH
Every way to spend this wallet needs a hashlock preimage. It is not on this device and
not on these plates. Back up the phrase and its method, or the preimage plate, separately.
```

Path 1 is spendable only with the phrase; path 2 only with the plate. The operator is told
they may back up either. An operator who keeps the phrase and discards the plate loses
path 2 permanently — and §8h is the *last* thing they read before engraving.

The same sentence undercounts two different phrases on two paths ("the phrase … its
method", singular) — and it retracts the confirm modal's own `composerCopyHashlockOtherPath`
line, which is deliberately count-free precisely because a hard-coded number was wrong
("two phrases" was wrong on any wallet with three hashlocks; `composer_copy.go:452-459`).

This is inherited from H2 §4.7, not introduced by H5 — but §2 is the section that
re-derives when the form fires, it is the only stage that will touch this predicate, and
the fit budget is there (the phrase form measures 160 chars drawn, headroom 378).

**SUGGESTION.** §2 gains a normative item: the phrase form's last sentence becomes
*"Back up every phrase and its method, and every preimage plate, separately."* — "every"
and "and", not "the" and "or". Add a copy-table row and a §6 mutation (restore "or" →
the mixed-wallet test fails).

### I-4 — §5 makes an unlabelled 0-based index actionable

Before §5, "Record 1" was a label. After it, it is an instruction: *remove that record*.
The index is 0-based (`seal/record.go:69`, `Index int // 0-based, as \`me\` counts records`)
and the device says so **nowhere**:

```
$ grep -rn "records count from\|counting from\|count from 0" gui/ seal/ --include=*.go | grep -v _test
(no output)
$ grep -c "records count from 0" crates/me-cli/src/main.rs
13
```

The host says it thirteen times; the screen that now tells the operator to act on the
number says it zero. A 1-based reading removes the record *above* the preimage. In the
package's own fixture that record is a seed
(`gui/unlock_preimage_test.go:40`: `sealBlobForTest(t, d.Public, []string{d.Secret[0], guiPreimagePlate}, …)`,
and the screen reads "Record 1"). Deleting a seed line from a records file the operator
may hold no other copy of is a loss the previous wording could not cause.

I did **not** raise this to Critical: it needs both a 1-based misreading and a records
file that is the only copy, and `me`'s own docs state the base. It is Important because
§5 is what converts a harmless ambiguity into an instruction, and the fix is free —
measured with a two-digit index and the longest noun:

```
H5 §5 refusal + 0-based note: 153 chars drawn in full, headroom 397 chars (margin 80)
H5 §5 refusal (as specified):  134 chars drawn in full, headroom 418 chars (margin 80)
```

**SUGGESTION.** §5's added sentence becomes *"Remove that record (records count from 0)
on the host and seal the payload again."* — `me`'s own phrasing, so the two surfaces read
alike. Measured headroom 397.

### I-5 — §4.3 requires a label lookup the emulator API cannot perform

> §4.3: *"The phrase row is chosen by LABEL (`Type a hashlock phrase`) from `shTargets`,
> never by index."*

`shTargets` carries no text:

```go
// cmd/emu/screen.go:92
func frameTargets(d *op.Drawer, bounds image.Rectangle) []image.Rectangle {
```
```go
// cmd/emu/screen_js.go:69-73
out = append(out, map[string]any{
    "x": r.Min.X, "y": r.Min.Y, "w": r.Dx(), "h": r.Dy(),
    "cx": (r.Min.X + r.Max.X) / 2, "cy": (r.Min.Y + r.Max.Y) / 2,
})
```

and `screen.go:95-98` says the tag was dropped on purpose: *"a tag is a live pointer into
GUI state and this struct outlives the frame, so keeping one would hold a screen's widgets
alive for as long as the emulator runs."* `walk_hashlock_phrase.js`'s `chooseRow(i, expect,
label)` uses `label` only in the **error message** (`:165-183`); the tap is `targets[i]`.

So §4.3 as written is unimplementable, and an implementer who "does it" will either
extend the emulator API (a change §4 does not scope, against a documented refusal) or
rename the parameter and call it done.

**SUGGESTION.** Pick one and say which:
(a) extend `screenRecorder` to record each target's **text** at frame time — text is
already extracted synchronously (`screen.go:125-131`), so no tag is retained — and have
`shTargets` return `{…, text}`; §4.3 then means what it says; or
(b) drop the label requirement and pin the index instead: before tapping, assert that the
frame carries the no-payload lead **and** all three fixed labels, and that
`shTargets().length === 3`. That catches a displaced row without a new API.

### I-6 — §4.1's "`cmd/emu`, js build only; no production code path" is not achievable

The hashes live in `st`, a local of `gui.composerFlow` (`gui/composer_flow.go:34`).
`cmd/emu` has no handle on it, and `gui` carries no `//go:build js` production file:

```
$ grep -rln "go:build js\|go:build.*wasm" gui/
gui/chain_class_walk_test.go
```

The established seam for exactly this is an **untagged package var in `gui`** —
`var passphraseWidgetHook func(name string, w any)` (`gui/passphrase_flow.go:21-28`),
documented as *"a test-only seam … nil in production; mirrors bip85SeedHook, the sanctioned
in-file test seam"*. That is production code: it compiles into the firmware and adds a
nil-check. Calling it "no production code path" is the claim H5 will be judged against
when someone greps the firmware for new globals, and it sends the implementer to the wrong
package.

**SUGGESTION.** §4.1 names the seam explicitly: a `composerStateHook`-style package var in
`gui` on the `passphraseWidgetHook` model (nil in production, set by `composerFlow` for the
composition's lifetime and cleared on exit), with `cmd/emu`'s `//go:build js` glue
installing `window.shComposerPathHashes()` over it. Replace "no production code path" with
what is true: "one nil package var in `gui`, on the sanctioned `passphraseWidgetHook`
model; no behaviour in production". State the firmware-size consequence, since §6 already
requires the delta.

### I-7 — the stored-vs-displayed assertion ships with no mutation, and it is the half a slip makes vacuous

§4's "Today" names **two** defects the walk cannot catch: the hash assigned before the
hold, and *the stored digest differing from the displayed one*. §4.5 mandates one mutation
run, for the first. §6's row for §4 is only "the two walk runs of §4.5". So the second
assertion is never shown to be able to fail — under a section headed "Tests (each with the
mutation that must fail it)".

It is not a theoretical gap. `shComposerPathHashes()` has two plausible implementations:
reading `*st.list.Paths[i].Hash` (correct) or formatting the digest the route already has
in hand. The second makes §4.2's comparison a tautology — the walk asserts the displayed
token against the displayed token, reports green, and F-485's second defect stays open with
a passing gate over it. Nothing in §4, §6 or §7 would notice.

Related, from the same section: §4.2 says "reads the hashes BEFORE the hold" without
pinning *where*. A read at the top of trial 4 (before the phrase screen) is null in the
mutated build too, so the §4.5 mutation run would pass. §4.5's stated requirement is what
corrects it — but only if the controller records **which** assertion failed, which §4.5
says and §7's "Both walk runs recorded" does not.

**SUGGESTION.** §4.5 gains a **third** run: a build whose stored hash is perturbed after
the confirm body is built (e.g. `d[0] ^= 1` before `st.list.Paths[idx].Hash = &d`); the
walk must FAIL on the stored-equals-displayed assertion, and specifically not on the
pre-hold one. §6's §4 row becomes "the three walk runs of §4.5". §7 requires each run to
be recorded **with the assertion that failed**, not just pass/fail.

### M-1 — §1's copy change silently falsifies a walk literal §4 does not list

`cmd/emu/walk_hashlock_phrase.js:318` is `await waitFor("run ms hashlock with this phrase", 20000)`.
§1's body says "run ms hashlock with **them** on the host". §4's five normative items do
not mention it, so an implementer working from §4 alone leaves a walk that times out on
the branch build — loud, but it means the first §4.5 run fails for the wrong reason.

**SUGGESTION.** Add to §4: the reconcile `waitFor` literal is re-pointed at §1's new text,
and the walk asserts the `hash` token rather than a prose fragment (§4.2 already requires
the token, so the prose match can simply go).

### M-2 — the toolkit manual quotes the screen §1 replaces, and nothing schedules the update

`mnemonic-toolkit/docs/manual/src/40-cli-reference/43-ms.md:501-502` quotes the current
one-sentence reconcile screen verbatim, and `:505-508` tells the operator *"its first and
last eight characters are what the **confirm** screen showed"* — under §1 they are also
what the reconcile screen shows, which is the whole point of F-487. The chapter states
"Screens are quoted from that firmware" (`:398`). H5 §0's out-of-scope list names F-483,
F-489 and the idle-timer note, not this.

**SUGGESTION.** §0 or §7 states that `43-ms.md`'s hashlock-on-the-SeedHammer section is
re-quoted in the same cycle (a toolkit-side docs commit), or files it as a follow-up with
an owning phase. Leaving it unnamed is how the H2 §14 sentences nearly survived into H3.

### M-3 — §2.3 deletes a function two tests mutate; §6 names only one of them

`composerHashByPhraseSync` has two call sites (`gui/composer_hash.go:237`, the `No hash
lock` arm; `gui/composer_shape.go:356`, the Remove arm) and **two** tests naming it as
their mutation target: `TestRemovePathReSyncsHashByPhrase` (`composer_hashlock_test.go:1016`)
and the `No hash lock` assertion at `:717-721` (*"MUTATION: delete the
composerHashByPhraseSync call in composerHashEdit's noneRow arm -> this fails"*). §6
converts the first and is silent about the second, which becomes uncompilable when
`hashByPhrase` is removed.

**SUGGESTION.** §6's §2 bullet says the `:717-721` assertion is **deleted** (§2.1's
value-set invariant makes it unnecessary: with no path hashed, §8h does not draw at all),
so the implementer does not invent a replacement for it.

### M-4 — the reconcile screen does not say the digest is repeated at consent

Verified above: the consent screen prints the same `first8..last8` token for every path,
and it is unavoidable before engraving. An operator who dismisses the reconcile screen
before writing has not lost the digest — but has no way to know that, and the plausible
reaction is to re-run the phrase route (a wasted ~10 s hardened derivation, and a second
chance to mistype).

**SUGGESTION.** Documentation only — one clause in §1's rationale paragraph noting that
`composerDigestShort` (`gui/composer_consent.go:61-64`) is byte-identical to
`hashlockFirst8Last8` (`gui/composer_hashlock.go:131-134`), so the consent screen is the
recovery. No copy change: the reconcile body has no room to spare for a "you can see it
again later", which would also weaken the instruction.

### M-5 — "seal the payload again" changes the unlock passphrase, and the screen does not say so

`me seal --help`: *"The passphrase is GENERATED and printed to STDERR — write it down and
store it apart from the machine. There is deliberately no way to supply your own."* The
operator has just spent ~31 s typing the old passphrase to reach this refusal. After
re-sealing, that passphrase is dead and the new UF2 must also be flashed. Neither step is
on the screen.

**SUGGESTION.** Documentation only, or one clause if §5's fit allows (headroom 397 with the
0-based note of I-4 already added): *"…and seal the payload again — the new one has a new
passphrase."* Judge against I-4, which has the stronger claim on the same budget.

### N-1 — the comment §1 rewrites carries a headroom number that is off by 79

`composerCopyHashlockReconcile`'s doc comment (`gui/composer_copy.go:441`) says the
placement *"keeps the confirm modal's measured headroom (**186**) intact"*. Measured today
on the same gate:

```
$ go test ./gui/ -run TestConfirmScreensThisBlockTouchesAreDrawnInFull -v
the hashlock confirm modal, longest variant (H2 §4.5): 336 chars drawn in full, headroom 107 chars (margin 80)
```

H5 §1.2 uses the correct 107. The stale 186 sits in the function §1 rewrites.

**SUGGESTION.** §1 item 1 says the doc comment's headroom figure is re-measured and
restated at the same time (or the number simply dropped — the fit table is the record).

### N-2 — the screen says "check the digest matches" without saying the ends are what match

The device writes `hash  3cf5d421..b70a4c12`; the host prints
`hash:3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12`. The two are
visibly different objects, and the `hash `/`hash:` near-collision does not help. The
manual explains it (`43-ms.md:505-507`); the screen does not.

**SUGGESTION.** Documentation only — the manual already covers it, and the reconcile body
should spend its remaining budget on I-1 and I-2 instead. Recorded so a later round does
not re-derive it as new.

---

## What I checked and did NOT find

- **§1's method strings map to the host's flag verbatim.** `hashlockMethod.String()` returns
  `sha256` / `hardened`; `ms hashlock --method hardened|sha256` (manual `43-ms.md:317, 335`).
  No translation is asked of the operator. Good design; no finding.
- **§5's verb is real and the host is consistent.** `me seal` exists, and at 0.8.1 it refuses
  a preimage plate with the same words the device uses — so "remove, then seal" is a path the
  host supports, and re-sealing *with* the record fails loudly rather than silently.
- **§2's value set cannot leak between wallets.** `composerState` is constructed per
  composition (`gui/composer_flow.go:34`).
- **The emulator hook is readable while a modal is up.** The existing walk already calls into
  Go at exactly that moment (`walk_hashlock_phrase.js:318`).
- **§4.5's mutation does fail the walk**, and over-determinedly: with the assignment moved
  before the confirm, trials 1-3's Backs also leave the hash assigned, so §4.6's Back
  contract breaks too.
- **The non-preimage arms of `unlockNotPermittedBody` are unreachable from a payload `me`
  0.8.1 built** (descriptor, address, debug command and a plaintext seed are all refused on
  the host, verified by four invocations), so §5's sentence landing on the shared function is
  not the hazard it first looks like.

## Method note

Two scratch test files were added under the worktree's `gui/` to measure the proposed
bodies against the real `assertModalBodyFits` gate and to construct the mixed-wallet
banner, then deleted with the worktree. Every quoted number above is from a command run
in this session; nothing was measured by reading a doc comment.
