# Hashlock H2 — post-implementation review, lens: INTERRUPTION AND STATE

Fork branch `hashlock-h2` @ **17b3979**, reviewed in a detached worktree at
`/scratch/code/shibboleth/.tmp/h2-wf-lens-interruption` (removed after this
report; nothing committed, nothing pushed). Go 1.26.7 at
`/scratch/code/shibboleth/.toolchain/go`.

**Question answered:** what the branch's device does when the operator interrupts
the phrase route — Back, the screensaver/idle timeout, the session ending — at
every step, and whether composer state is exactly what `SPEC_hashlock_H2_device.md`
§4.6 says, whether any `Hash` is assigned before HOLD, whether the phrase bytes
are wiped on exit, and whether `Deriving` can be re-entered cleanly.

**Counts: 0 Critical / 1 Important / 3 Minor / 1 Nit.**

Ten new harness cases were written (7 top-level tests, 5 of them subtests of one
table) and all pass; four of them were mutation-controlled. The full package is
green with them: `RESULT: ok -- all 1231 tests ran across 24 shards` (was 1229
before the two readout tests, 1222 at 17b3979 per the dispatch brief). The tests
are attached verbatim as
`design/agent-reports/hashlock-H2-post-impl-lens-interruption.tests.go.txt`
(drop it into `gui/composer_hashlock_interruption_test.go` to re-run).

---

## Findings

### I-1 — the phrase screen draws NO readout, so every interruption that PRESERVES the phrase gives the operator nothing to check it against

`hashlockPhraseFlow` (`gui/composer_hashlock.go:141-186`) renders the four-page
keyboard, a lead, an `n/100` counter and the nav — and **the keyboard's readout
is clamped to zero characters**, in both the masked and the revealed state. The
operator's only feedback while typing a hashlock phrase is a character count.

This is an interruption finding, not a cosmetic one. §4.6 preserves the typed
phrase across four of the five interruptions on this route (Back from the method
pick, a declined method modal, Back during the derivation, Back from the confirm
modal), and Run's screensaver preserves it across the fifth. At every one of
those resumptions the operator can see only *how many* bytes survived, never
*which*. §4.2's own sentence — *"The keyboard's reveal (`show`) key is inherited
as-is"* — is not true on this screen: the key toggles its cap from `show` to
`hide` and reveals nothing, because there is nothing being drawn to reveal.

**Mechanism, measured.** `PassphraseKeyboard.Layout` clamps the readout to
`MaxHeight - grid.Y - readoutGap` and binary-searches leading runes away until
it fits (`gui/passphrase_keyboard.go:448-472`). `hashlockPhraseFlow` cuts a
**lead band** out of the content that `passphraseEntryFlow` does not
(`composer_hashlock.go:169`, before its `kbd.MaxHeight` at `:175`; `passphrase_flow.go` cuts only the counter band at `:129` before its own `:137`), and that band is what
takes the budget below one line.

```
$ go test ./gui/ -run 'TestHashlockPhraseScreenReadoutBudget' -count=1 -v
=== RUN   TestHashlockPhraseScreenReadoutBudget
    composer_hashlock_interruption_test.go:660: panel (480,320), content 268 px; lead band 44, counter band 23, grid (340,182), one readout line 19
    composer_hashlock_interruption_test.go:662: hashlockPhraseFlow readout budget = 11 px; passphraseEntryFlow = 55 px
    composer_hashlock_interruption_test.go:672: PINNED DEFECT: 11 px of readout budget against a 19 px line
--- PASS: TestHashlockPhraseScreenReadoutBudget (0.00s)
```

**The lead copy is the cause, and it is a departure from §4.2.** §4.2 states the
lead as *"Use a phrase you have never used anywhere else."*
`composerCopyHashlockPhraseLead()` returns *"This screen does that hashing for
you. Use a phrase you have never used anywhere else."* — one extra sentence,
which wraps the lead onto a second line. Measured across three variants on the
real 480x320 panel:

```
lead band 44 px, readout budget 11 px (need 19) -- "This screen does that hashing for you. Use a phrase you have never used anywhere else."
lead band 23 px, readout budget 32 px (need 19) -- "Use a phrase you have never used anywhere else."
lead band 23 px, readout budget 32 px (need 19) -- ""
```

With §4.2's lead **exactly as specified**, the readout draws. The added sentence
is what silently deleted it.

**Reproduction on the touch harness** (the frames are `ExtractText` output, which
collects runes regardless of occlusion — `passphrase_flow.go:126` — so an empty
extraction means the readout was never *drawn*, not that it was covered):

```
$ go test ./gui/ -run 'TestHashlockPhraseScreenDrawsNoReadout' -count=1 -v
=== RUN   TestHashlockPhraseScreenDrawsNoReadout
    composer_hashlock_interruption_test.go:613: PINNED DEFECT: 4 characters are in kbd.Fragment and the frame draws no '****': "qwertyuiopasdfghjklzxcvbnmABCspaceshowThisscreendoesthathashingforyou.Useaphraseyouhaveneverusedanywhereelse.4/100Hashlockphrase"
    composer_hashlock_interruption_test.go:633: PINNED DEFECT: revealed=true, the cap reads `hide`, and the frame still draws nothing: "qwertyuiopasdfghjklzxcvbnmABCspacehideThisscreendoesthathashingforyou.Useaphraseyouhaveneverusedanywhereelse.4/100Hashlockphrase"
--- PASS: TestHashlockPhraseScreenDrawsNoReadout (0.01s)
```

The control that proves the keyboard is not at fault is
`gui/passphrase_keyboard_test.go:137` (`TestPassphraseMaskReveal`), which asserts
`****` in a rendered frame of the same widget with no clamp.

**Why Important and not Critical.** The wrong outcome is a mistyped phrase whose
digest gets held-to-confirm. §4.5's reconciliation line (*"Before you fund this
wallet, run ms hashlock with this phrase and method on the host and check the
digest matches"*) catches exactly that before funds move, and the `n/100` counter
still reports the true length — so this is a real defect and a missing case, not
a divergence path or lost funds. Both tests are written as PINS: they fail loudly
if the readout starts drawing, with instructions to invert them.

*Spec lines:* §4.2 (*"lead (journey I-2): 'Use a phrase you have never used
anywhere else.'"*; *"The keyboard's reveal (`show`) key is inherited as-is"*).

---

### M-1 — `st.hashByPhrase` survives "Remove path", so §8h names a phrase the composition does not have

`composerHashByPhraseSync`'s own godoc (`gui/composer_hash.go:177-180`) calls
itself the sync for *"the one event after which no phrase-set hash can still be
in the composition"*. It has exactly **one** call site:

```
$ grep -rn "composerHashByPhraseSync" --include=*.go .
gui/composer_hash.go:192:func composerHashByPhraseSync(st *composerState) {
gui/composer_hash.go:237:			composerHashByPhraseSync(st)      # the `No hash lock` row
```

`composerPathEdit`'s Remove arm splices the path away without it
(`gui/composer_shape.go:353`). Remove the only phrase-hashed path, add another
whose hash came from `Type 64 hex` or a payload row, and §8h at Done
(`gui/composer_shape.go:442`, `composerCopyHashEveryPathFor` at
`composer_copy.go:465-470`) draws the **phrase** form:

```
$ go test ./gui/ -run 'TestHashlockByPhraseFlagSurvivesRemovePath' -count=1 -v
    composer_hashlock_interruption_test.go:566: PINNED DEFECT (Minor, safe direction): §8h at Done draws the PHRASE form for a composition holding no phrase-set hash: "HASH ON EVERY PATH\nEvery way to spend this wallet needs a hashlock preimage. It is not on this device and not on these plates. Back up the phrase and its method, or the preimage plate, separately."
--- PASS: TestHashlockByPhraseFlagSurvivesRemovePath (2.03s)
```

Minor because the direction is the safe one the fold argued for
(`composer_hash.go:185-189`: *"the operator is told to back up one artifact too
many, never one too few"*) — but that comment reasons only about a hash being
*replaced*, never about the path being *removed*, and the godoc's "one event"
claim is inaccurate as written. Owning phase: **H3**, alongside the per-path
provenance follow-up the same comment already files.

### M-2 — nothing on the phrase route wipes, clears or scrubs the phrase bytes on any exit (secret handling — non-gating by the 2026-08-27 ruling)

```
$ grep -n "Scrub\|clear(\|Wipe\|wipeGuard" gui/composer_hashlock.go
(none)
```

`hashlockPhraseRoute`'s `phrase []byte` (`composer_hashlock.go:44`) and the
keyboard's `Fragment` string are both left intact on every return — the Back to
`Which hash?`, the assignment, and the `ctx.Done` unwind. The sibling flow for
the other secret this device types installs the opposite discipline
(`gui/unlock_kdf.go:135-144`), with its reason stated:

> F-107 (R0 round 0, I1): §8's twelve-word passphrase is rendered HERE, outside
> unlockSecretSession's bracket, and on the give-up routes nothing scrubbed at
> all. The passphrase opens the payload, so its glyphs are as sensitive as the
> seed's.

**A coupling worth recording:** today no *glyph* of the phrase reaches `ctx.B` at
all — precisely because of I-1, the readout draws nothing. Fixing I-1 opens the
`op.Glyph`-into-`args` residue path that `op.Buffer.Scrub`'s own doc describes
(*"after Reset the plaintext comes back verbatim and in order from the backing
array"*), so a readout fix without a `defer ctx.B.Scrub()` would convert this
Minor into a live buffer-residue leak. `op.Buffer.Residue()` is the seam that
measures it.

### M-3 — the route holds a secret with §10.2.4's timer disarmed, and arming it would cost the composition (secret handling — non-gating)

`ctx.wipe` is installed only by `unlock_session.go:88-89` and by
`unlock_kdf.go:136` (`wipeGuard{subject: wipeWarningSubjectPassphrase}` —
§10.2.4 row 4, the in-flight passphrase). `composerFlow` is reached from
`gui/wallet_policy.go:56`, outside both, so `ctx.wipe.armed()` reads false for
the whole phrase route. An operator who types a hashlock phrase and walks away
gets the ordinary screensaver and no wipe: the phrase stays in RAM until the
session ends.

Recorded, not prescribed. The tension is real in both directions — the phrase is
the same class of secret §10.2.4 row 4 covers, but arming the timer inside the
composer would let a walk-away **destroy the whole composition** (Run's armed
branch draws the warning, `continue`s without returning control, then sets
`wiping` and unwinds — `run_flow.go:381-386`), which is a worse outcome than the
one it prevents. Naming which of those the spec intends is an operator/spec
decision, not an implementation defect.

### N-1 — Back at the §8i rule modal opens the phrase keyboard instead of returning to `Which hash?`

`composerHashEdit` shows the 32-byte rule with `showError`
(`composer_hash.go:214`), and `ErrorScreen.Layout` binds Button1 and Button3 to
the *same* single dismissal (`gui/gui.go:400-404`) — deliberate, F-440, across
all 143 `showError` sites. So an operator who reads the rule and presses Back
lands on the keyboard rather than back on the row list, and pays one extra tap to
leave. Not worth a change: the wrong outcome is strictly better than telling the
operator nothing, and changing it here would fork a contract the whole firmware
shares.

---

## PASSes — what the branch gets right, each driven on the harness

Everything below was *executed*, not read. `st` is the `composerState` after the
interruption; "spec" is `SPEC_hashlock_H2_device.md`.

| # | Interruption | Result |
|---|---|---|
| P-1 | Back at the phrase screen, then **re-enter the row** | **PASS.** The re-entered screen reads `0/100`, never `28/100`. §4.2: *"Back returns to `Which hash?` and drops the phrase."* `TestHashlockBackAtThePhraseScreenActuallyDropsThePhrase`. The branch's own `TestHashlockBackContractKeepsThePath` backs out and stops, so the word "drops" was asserted nowhere. |
| P-2 | Back mid-countdown, then **re-enter `Deriving`** | **PASS.** Nothing assigned by the abandoned run; the second run yields the corpus hardened digest `3cf5d421..b70a4c12` with `chars: 28` and `method: hardened`, one path, `hashByPhrase` set only after the hold. §4.4: *"Back during the derivation abandons it and nothing is assigned."* `TestHashlockAbandonedDerivationReEntersCleanly`. Abandon point logged: `"2%About0secondsleft.Deriving"`. |
| P-3 | Back from the method pick, then **EDIT** the restored phrase | **PASS.** 29 typed, Back, one backspace, SHA-256 → the 28-character anchor's digest `b867db87..edbc96cb`. §4.6's `initial` restore hands back the live `kbd.Fragment`, not a display echo. `TestHashlockRestoredPhraseIsEditable`. |
| P-4 | Back at the **reconcile screen**, after the hold | **PASS.** `composerHashEdit` returns true, the path survives, the held digest and `hashByPhrase` survive. `TestHashlockReconcileScreenBackKeepsTheAssignedHash`. |
| P-5 | The **screensaver** crossing the phrase screen | **PASS.** The saver activates (43 `Dirty` calls — the phrase screen correctly does *not* `KeepAwake`, unlike `Deriving`), a touch un-parks it, and the flow resumes on the phrase screen still reading `28/100`; Back then returns `(nil, false)`. `TestHashlockPhraseScreenSurvivesTheScreensaver`. Frames: 3 total, 2 before the saver, 1 after. This is the only screen of the route besides `Deriving` that any test drives through Run's real idle loop. |
| P-6 | The **session ending** (`ctx.Done`) at five points | **PASS at all five.** Phrase screen, method pick, mid-derivation, the confirm modal before the hold, and the reconcile screen after it. The flow unwinds at every one; nothing is assigned at the four pre-HOLD points; the held digest survives at the fifth. `TestHashlockRouteUnwindsWhenTheSessionEnds` (5 subtests). This is the closest in-process analogue of §4.4's *"A power loss ends the composition"*. |
| P-7 | **No `Hash` before HOLD**, structurally | **PASS.** Exactly one assignment on the phrase route, inside the confirm arm: `gui/composer_hashlock.go:69` `st.list.Paths[idx].Hash = &d`, guarded by `if composerConfirmScreen(...)`. The other three `.Hash =` sites are the payload row, the hex row and `No hash lock` (`composer_hash.go:219,233,236`) — the shipped routes, not this one. |
| P-8 | §4.6's *"`composerHashEdit` returns `false` ONLY for Back at `Which hash?`"* | **PASS, machine-checked.** `awk 'NR>=202 && NR<=243 && /return false/' gui/composer_hash.go` → exactly one hit, line 208. |
| P-9 | The abandoned KDF's own state | **PASS.** `hashlock.DeriveHardened` carries `defer d.Wipe()` (`hashlock/hashlock.go:59`) so the `seal.Deriver` is wiped on the abandoned path too, and the abandoned return is the zero `[32]byte` (`return x, false` before any `copy`). |
| P-10 | A cancelled route does not discard seats | **PASS.** `composerApplyShapeEdit` (`gui/composer_discard.go:144-153`) discards only when `composerShapeSignature` moves; a route that assigns nothing leaves the list byte-identical, so no seat assignment is lost by backing out. |

## Mutation controls — the new tests can fail

Each mutation was applied to the worktree, run, and reverted with
`git checkout`; the tree was verified clean afterwards (`git status --short`
shows only the untracked test file).

| Mutation | Result |
|---|---|
| `kbd.Fragment = string(initial)` → `_ = initial` (`composer_hashlock.go:143`) | `TestHashlockRestoredPhraseIsEditable`: *"the restored phrase screen does not read 29/100"*; `TestHashlockPhraseScreenSurvivesTheScreensaver`: *"the first frame after the screensaver reads ... 0/100 ... not 28/100"*. P-1 correctly stayed green (it asserts a drop). |
| the route caches the typed phrase across a Back to `Which hash?` | `TestHashlockBackAtThePhraseScreenActuallyDropsThePhrase`: *"the phrase survived a Back to `Which hash?`: the re-entered screen reads 28/100"*. |
| `abandoned` hoisted to a package-level var (leaks across calls) | `TestHashlockAbandonedDerivationReEntersCleanly`: *"never reached \"Write down this phrase\"; last frame \"HashlockmethodWhichmethod?Hardened(about10s)SHA-256\""*. |
| `ErrorScreen.Layout` Back unbound (F-440 reverted) | `TestHashlockReconcileScreenBackKeepsTheAssignedHash`: *"composerAddPath never returned after 256 frames"*. |
| `hashlockPhraseFlow`'s `for !ctx.Done` → `for {` | `TestHashlockRouteUnwindsWhenTheSessionEnds/at_the_phrase_screen`: *"the flow did not unwind when the session ended: a screen of the phrase route is still drawing frames after ctx.Done"*. |

## A harness trap the next reviewer should not fall into

`h.ctx.Done = true` between frames **does nothing** under `runUITouch`. Its
`FrameCallback` is `ctx.Done = ctx.Done || !yield(content)`, and Go evaluates the
left operand before the call, so the assignment writes back the value `Done` had
at the start of the suspended frame and silently clobbers an externally-set
`true`. Measured: twelve consecutive `Which hash?` frames drawn with
`h.ctx.Done` set, `done=false` on every one — which reads exactly like a flow
that refuses to unwind. The real seam is the iterator's `stop()`
(`runComposerHashEditStoppable` in the attached file), and it is wrapped in a
15-second watchdog because a screen that ignores `ctx.Done` makes `stop()` block
forever — a `go test` timeout instead of a failure.

## Coverage this lens did NOT reach

- **The real panel.** Everything here is the host harness at `sh2DisplaySize`
  (480x320, `gui_test.go:405`). I-1 in particular deserves one photograph from
  the flashed device before it is called closed — text extraction cannot see
  clipping, and it could not see this readout either way.
- **The idle path at the method pick and the confirm modal.** P-5 drives Run's
  real idle loop over the phrase screen only. `ConfirmDelay.Progress` re-signals
  `ctx.Wakeup` every iteration of a press-and-hold, which `deadlinePlatform`'s
  own comment (`gui/run_harness_test.go:108-118`) says would stop a synctest bubble's
  clock — so a held confirm under a deadline platform needs harness work first.
- **NFC arriving mid-route.** `rows.digests` is captured once per
  `composerHashEdit` iteration and the relation line is computed from that
  snapshot; whether `ctx.sysw` can change under a running flow was not
  established.
- **Copy fidelity, the fit gate, the corpus lockstep, the codec and seal halves.**
  Out of lens by the brief; the concurrent whole-diff review owns them. The one
  copy fact reported here (§4.2's lead) is reported because it is I-1's cause,
  not as a copy review.
