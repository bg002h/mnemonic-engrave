# Walk-harness false-pass review — `shToolpath.strings()` census

Reviewer: independent adversarial review (sonnet), repo `/scratch/code/shibboleth/seedhammer`,
diff `10286e4..HEAD` (5 commits, branch `main`). Mutation/build work done in the
isolated worktree at
`/tmp/claude-1000/-scratch-code-shibboleth-mnemonic-engrave/3985bd41-08d3-42b8-a967-1493b588d215/scratchpad/wt-falsepass`
(same commit, `740888d`), per the coordinator's isolation correction. Worktree
and main repo both verified clean (`git status --short`, empty) before and
after.

## THE ONE QUESTION

Can `shToolpath.strings()` report a PASS when it should not?

**Answer: yes.** One Critical finding: a real, funds-relevant engrave (an ms1
codex32 secret/share) can be cut and accepted and never appear in the census —
it silently lands in `unattributed`, indistinguishable from an ordinary seed
plate. No Important findings beyond this. No over-reporting defect found.

---

## CRITICAL — ms1 (codex32) secrets bypass the census at three call sites, contradicting the census's own documented contract

**What the code claims**, in four separate doc comments this diff adds:

- `cmd/emu/toolpath_js.go:28` — "strings holds the md1/mk1/ms1 whose plates
  were cut AND accepted, in cut order."
- `cmd/emu/engraved.go:6` — "the gate is a BYTE COMPARISON -- the md1/mk1/ms1
  strings that came out must be the ones the inputs require."
- `gui/engraved_hook.go:3` — "The engraved-text seam: which md1/mk1/ms1
  string each finished plate carried."
- `gui/engraved_hook.go:47` — "which md1/mk1/ms1 string each engraved plate
  carried."

**What the code does**: only `gui/gui.go:2299` ever sets `Plate.id` to a
non-zero value (confirmed by grep — it is the *only* assignment site in the
non-test tree), and that happens inside `validateMdmk` (`gui/gui.go:2257`).
Every `Plate` built any other way keeps `id`'s zero value, and
`EngraveScreen.Engrave`'s accept branch (`gui/gui.go:2952`,
`notifyPlateEngraved(ctx.Platform, s.id)`) then routes it to
`engravedRecorder.unknown` (`cmd/emu/engraved.go:71-75`), not to `strings`.

I enumerated every production caller of `NewEngraveScreen` (13 sites,
`grep -rn "NewEngraveScreen(" gui/*.go | grep -v _test.go`) and classified
each by whether its `Plate` can carry a non-zero id:

| Call site | Source | In census? |
|---|---|---|
| `gui/gui.go:2362` (`mdmkFlow`) | `validateMdmk` | **yes** |
| `gui/bundle_flow.go:318` (`bundleEngrave`) | `validateMdmk` (per-plate, `gui/bundle_flow.go:298`) | **yes** — covers md1/mk1 **and** ms1 cards from the T6a-2 derive-and-engrave flagship flow (`gui/singlesig_engrave.go:20-45`, `gui/multisig_engrave.go`) |
| `gui/derive_xpub.go:472` (`multiPlateEngrave`) | `validateMdmk` (`gui/derive_xpub.go:453`) | **yes** (mk1 xpub cards) |
| `gui/unlock_platelist.go:233` (`unlockEngraveFlow`) | `validateMdmk` (`gui/unlock_platelist.go:222`) | **yes** (public md1/mk1 payload records) |
| `gui/gui.go:2409` (`backupWalletFlow`) | `engraveSeed` | no — correctly excluded (BIP-39 seed, not a constellation string) |
| `gui/gui.go:2432` (`backupSeedStringFlow`) | `toPlate` direct | **no — BUG when this is an ms1 secret** (see below); correctly excluded when it's a genuine seed-string backup |
| `gui/gui.go:2448` (`descriptorFlow`) | `toPlate` direct | no — correctly excluded (bip380 descriptor, not md1/mk1/ms1 text) |
| `gui/passphrase_flow.go:699` | `toPlate` direct | no — correctly excluded (passphrase) |
| `gui/slip39_polish.go:507` | `toPlate` direct | no — correctly excluded (SLIP-39 share, different format) |
| `gui/freetext_flow.go:1604` | `toPlate` direct | no — correctly excluded (arbitrary free text) |
| `gui/bip85.go:341` | `engraveBip85Child` | no — correctly excluded (derived BIP-39 child seed) |
| `gui/unlock_session.go:221` (`unlockEngraveCodex32`) | `toPlate` direct | **no — BUG**, ms1 secret from an unlocked payload |
| `gui/unlock_session.go:332` (`unlockEngraveMnemonic`) | `engraveSeed` | no — correctly excluded (bare mnemonic) |

**The three call sites that build an ms1 plate the way `backupSeedStringFlow`
expects — and are therefore invisible to the census — are:**

1. `gui/codex32_polish.go:230-234` — `engraveCodex32`'s `codex32Engrave`
   case, reached from the ordinary top-level scan/typed-menu flow
   (`gui/gui.go:2226-2227`, `case codex32.String: return
   engraveCodex32(ctx, th, scan)`, itself reached from `gui.go:1847`'s
   `engraveObjectFlow(ctx, th, obj)` where `obj` can be a directly scanned or
   manually-typed ("M\*1 STRING" menu row, per `gui/gui.go:2464-2466`) ms1
   secret or share.
2. `gui/unlock_session.go:186-227` (`unlockEngraveCodex32`) — reached when
   unlocking a payload whose secret record is `seal.ClassCodex32Secret`
   (`gui/unlock_session.go:165-166`). This is the "Load Payload" journey's
   secret-engrave leg.

Both build the plate via `backup.EngraveSeedString` / `toPlate` directly
(never through `validateMdmk`), so `Plate.id` is always `0`.

**Only the T6a-2 derive-and-engrave flagship flow correctly wires ms1** (it
routes through `bundleEngrave`/`validateMdmk`, format-agnostic per its own
comment at `gui/singlesig_engrave.go:14-15`). The standalone
scan/type-then-engrave path and the payload-unlock secret-engrave path do
not. This is an inconsistency, not a blanket "ms1 is out of scope" design —
which is exactly what makes it a false-pass hazard: a walk that trusts the
documented "strings holds ms1" contract, or that only exercises the
recovery/unlock paths, gets silent, content-blind confirmation.

**Concrete failure mode**: the census's stated purpose is a byte comparison —
proof that the *correct* secret was cut, not merely that *some* plate was
cut. For these two paths, a walk that correctly recovers and engraves an ms1
secret, and a walk that (due to a bug elsewhere, e.g. in
`recoverCodex32Flow`'s share interpolation) engraves the *wrong* secret,
produce **identical** census output: `strings` unchanged, `unattributed`
incremented by one. The gate cannot tell them apart. That is precisely
"under-reporting that reads as success": the walk that cut the wrong content
still looks complete.

It also weakens the `announced`/`unattributed` empty-vs-broken distinction
documented at `cmd/emu/engraved.go:107-110` ("Unattributed counts finished
plates that never came from validateMdmk -- seeds, passphrases, free text
... NOT an error"): that enumeration silently also covers a mis-cut ms1
secret, which is very much a thing a walk should be able to catch and is not
"NOT an error" in the same sense.

### Evidence (executed, not argued)

Added a temporary test to `gui/engraved_hook_test.go` in the isolated
worktree, driving `engraveCodex32` to full completion (confirm screen →
Button3 "Engrave" → the same press/hold/accept sequence the existing
`TestEngraveScreenReportsTheStringItEngraved` uses) with the fixture
`ms10testsxxxxxxxxxxxxxxxxxxxxxxxxxx4nzvca9cmczlw` (the same unshared-secret
fixture already used by `TestConfirmCodex32Unshared`,
`gui/codex32_polish_test.go:129-147`, and matching `gui/bundle_test.go`'s
`ms1Fixture`). Ran it with:

```
export PATH="/nix/var/nix/profiles/default/bin:$PATH"
nix develop --command go test ./gui/ -run TestScratchCodex32NeverEntersCensus -v
```

Result:

```
=== RUN   TestScratchCodex32NeverEntersCensus
    engraved_hook_test.go:66: p.engraved = []string(nil), p.unknown = 1, p.candidates = map[uint64]string{}
--- PASS: TestScratchCodex32NeverEntersCensus (4.31s)
```

The engrave completed and was accepted (test would `t.Fatal` otherwise —
it didn't). `p.candidates` is empty, proving `PlateText`/`notifyPlateText`
was never called for this string at all (not merely "announced but not
matched") — confirming `validateMdmk` is genuinely never on this path, not a
subtler id-mismatch. `p.unknown == 1` confirms the accepted plate silently
became an "unattributed" count instead of a recorded string.

The test file was reverted immediately after
(`git checkout -- gui/engraved_hook_test.go`); `git status --short` in the
worktree is empty.

### Severity

**Critical.** This is exactly the mechanism the census exists to prevent
(byte-comparison content proof for secret-bearing plates, stated explicitly
in `cmd/emu/engraved.go`'s own top comment), it is reachable through ordinary
top-level UI flows (not a contrived edge case), it concerns the most
funds-sensitive artifact type this project handles (an unshared or shared
ms1 secret), and it is asserted as covered in four separate doc comments
that this diff itself writes. It also explains why no test caught it: the
new test files (`gui/engraved_hook_test.go`, `cmd/emu/engraved_test.go`)
exercise only md1/mk1 via `validateMdmk` directly or via a hand-built
`EngraveScreen`; nothing drives `engraveCodex32` or `unlockEngraveCodex32`
end-to-end.

### Suggested direction (not prescribing the fix)

Either (a) route `backupSeedStringFlow` through the same id-announcement
mechanism when its `SeedString.Seed` originated from a codex32 ms1 (would
need a source-tagged variant, since the same function legitimately serves
non-attributable BIP-39 seed-string backups too), or (b) narrow the doc
comments' claim from "md1/mk1/ms1" to what is actually wired, and treat the
gap as a tracked follow-up rather than a silent one. (a) is more consistent
with the stated design intent; (b) is the honest minimum if (a) is deferred.

---

## Other hunt categories — clean

**1. Over-reporting** (validated-then-abandoned, aborted mid-cut, id reuse,
non-constellation plate mis-attributed): no defect found.

- `plateTextSeq` (`gui/gui.go:610`) is a monotonic, never-reset,
  never-recycled `uint64`, pre-incremented before assignment
  (`gui/gui.go:2297-2299`), so id `0` is never issued by `validateMdmk` and
  stays a reliable "not from validateMdmk" sentinel. Grep confirms it is the
  only place `.id` is ever set to non-zero.
- `notifyPlateEngraved` fires exactly once per `EngraveScreen.Engrave` call,
  only from the explicit `case engraveDone: if selectBtn.Clicked(ctx)`
  branch (`gui/gui.go:2944-2953`), and the function returns immediately
  after — no path re-enters and double-fires for the same id.
- `engraveJob`'s state machine (`gui/engraver.go:157-182`) only reaches
  `engraveDone` on a genuine `err == nil` completion; a failed
  (`engraveFailed`) or stopped (`engraveStopped`) job never reaches the
  accept branch.
- The emu side asserts the interface shape at compile time
  (`var _ gui.EngravedAware = (*platform)(nil)`, `cmd/emu/platform.go`),
  closing the "method signature silently drifts and the hook goes dead"
  failure class this project has been bitten by before (per the comment
  immediately above it).

**2. Under-reporting elsewhere**: covered by the Critical finding above; no
other bypass found. All four call sites that legitimately produce md1/mk1
text (`mdmkFlow`, `bundleEngrave`, `multiPlateEngrave`, `unlockEngraveFlow`)
were updated in this diff to route through `validateMdmk`'s new
`Platform`-taking signature — confirmed by grepping every `validateMdmk(`
call site (5 total: the definition plus exactly these 4 callers).

**3. Empty-vs-broken**: `cmd/emu/engraved_test.go`'s
`TestEngravedCensusJSONDistinguishesEmptyFromBroken` exercises exactly this
(a fresh recorder reports `announced=0`; a wired-but-idle one reports
`announced=3, unattributed=1, strings=[]`), with the `"strings":[]` vs `null`
serialization check. This works as documented, modulo the ms1 case above
(which degrades, but does not destroy, the distinction — `unattributed`
still correctly signals "the hook is alive," just not "which secret was
cut").

**4. Do the new tests actually assert?** `gui/engraved_hook_test.go` and
`cmd/emu/engraved_test.go` are well-built: every test that could pass
vacuously carries an explicit positive control (e.g.
`TestAnUnannouncedPlateIsIgnored` and
`TestEngravedRecorderIgnoresUnannouncedPlates` both re-engrave the
*announced* id afterward and assert it *does* land, so "nothing is recorded
at all" cannot masquerade as "unannounced plates are correctly ignored").
`TestEngraveScreenReportsTheStringItEngraved` explicitly pumps frames until
the screen has re-entered its `engraveDone` switch arm before asserting
"not yet recorded," with a comment noting this was measured against the
`engraveDone`-instead-of-accept mutant. No assertions found that cannot
fail. The only gap is coverage, not weak assertions: no test drives
`engraveCodex32` or `unlockEngraveCodex32` to completion, which is exactly
the gap that hid the Critical above.

---

## Scope notes

- No fresh audit performed; every finding above is specific to code this
  diff (`10286e4..HEAD`) added or changed.
- §10.2.2 residency and the structural (`_tinygo.go` pairing) guard were
  explicitly out of scope per the brief and not reviewed here
  (`gui/tinygo_split_test.go` confirmed present and clearly labeled as that
  guard, not touched further).
- The four already-killed mutations (delete `notifyPlateText`; delete
  `notifyPlateEngraved`; report at `engraveDone`; recycle ids) were not
  repeated.
- `go test ./...`, `go vet ./...`, `GOOS=js GOARCH=wasm go vet ./cmd/emu/`,
  and the TinyGo build were taken as settled per the brief and not re-run.

## Repo state

Both trees verified clean via `git status --short` (empty output) before
and after this review:
- `/scratch/code/shibboleth/seedhammer` (read-only for this review)
- `/tmp/claude-1000/-scratch-code-shibboleth-mnemonic-engrave/3985bd41-08d3-42b8-a967-1493b588d215/scratchpad/wt-falsepass`
  (mutation/build/test worktree)
