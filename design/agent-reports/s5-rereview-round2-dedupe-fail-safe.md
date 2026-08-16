# S5 round-2 re-review — LENS: CAN B1'S DEDUPE FAIL UNSAFE?

**Artifact:** `git diff 830aaf7..s5-multislot` on `/scratch/code/shibboleth/wt-s5`
(HEAD `6088487092f21a77df543a3c83c5b9ff482cd609`, tree clean, **never written to**).
**The one question:** is there any input for which the restore document **omits,
misattributes, or under-states** a required spending factor, now that
`seedRegistry.passphraseFacts()` groups on the derivation unit `(Mnemonic,
Passphrase)`?

---

# VERDICT: **GREEN on this lens — 0 Critical, 0 Important**

| | count |
| --- | --- |
| Critical | **0** |
| Important | **0** |
| Minor | 1 |
| Nit | 0 |

**No input was found that omits, misattributes or under-states a passphrase.**
Ten registry shapes were built and pushed through the **production** chain
(`reg.passphraseFacts()` → `buildPassphraseInventoryLines`) and the rendered
document read out verbatim for each; two further candidate defects (a
page-overflowing merged line, and the merged group's fingerprint being taken
from the first member) were constructed, measured, and **refuted**. The one
Minor is a missing regression test on a fail-safe property, not a defect in
shipped behaviour, and is stated with the 12-line test that closes it.

---

# How it was answered

Everything below was **executed**, in a `cp -a` copy at
`…/scratchpad/r2lens`, verified byte-identical to the frozen tree before any
test ran:

```
diff -q /scratch/code/shibboleth/wt-s5/gui/multisig_build_slots.go \
        …/scratchpad/r2lens/gui/multisig_build_slots.go   → IDENTICAL
grep -c "slices.Equal" …/r2lens/gui/multisig_build_slots.go → 1
```

**Method note, because it nearly produced a false Critical.** The session
scratchpad is shared between agents and already contained a `wt-s5-copy`
directory from the implementer's fold work, so `cp -a wt-s5 …/wt-s5-copy`
nested the tree as `wt-s5-copy/wt-s5` and my first matrix ran against the
**pre-fold** sources. It reported "the pure B1 cell is NOT merged" — i.e. B1
entirely unfixed. The `diff -q` above is what caught it. Every measurement in
this report is from the re-run against the verified-identical tree.

## The comparison itself, resolved against the code

`gui/multisig_build_slots.go:298`:

```go
if g.passphrase == s.Passphrase && slices.Equal(g.mnemonic, s.Mnemonic) {
```

* `bip39.Mnemonic` is `[]Word`, `Word` is `int` (`bip39/bip39.go:22-24`), so
  `slices.Equal` is an element-wise value comparison — not slice identity, not a
  shared-backing-array accident. Confirmed by running it both ways: two
  independently-parsed copies of `fixtureMasterA` merge, and two `reg.add` calls
  handed the **same slice** also merge to one fact (`TestLensAliasing`,
  `same array=true`, `facts: [{Label:your seed for @0 and your seed for @1 …}]`).
* `Passphrase` is compared as a raw Go string — byte-exact, no trimming, no
  normalisation. **That is the correct comparison here, and I measured why
  rather than assuming it:** this tree's BIP-39 does not NFKD-normalise either,
  so byte-different passphrases are genuinely different wallets. `"café"` in NFC
  and NFD derive `2f918343` and `75902abe`; `"alpha"` and `"alpha "` derive
  `8aaa4f4b` and `f29a9dab`. A byte-exact key therefore agrees with the
  derivation on exactly the inputs that matter.
* The merged group's `MasterFP` is taken from the **first** member
  (`:306`, `masterFP: s.MasterFP`). That cannot misattribute: `reg.add` and
  `reg.bindPassphrase` have exactly **one production call site each**
  (`gui/multisig_build.go:618` and `:634`, both `&chaincfg.MainNetParams` —
  grepped, no other non-test caller), so equal `(Mnemonic, Passphrase)` implies
  equal `MasterFP` deterministically. Measured in every merged cell below: the
  group fingerprint equalled both members'.

## The premise B1's fix rests on, re-checked

The fold's comment asserts "the registry holds one entry per HELD SLOT". That is
true and there is no stale-entry path: `gui/multisig_build.go:229-235` is
`for _, slot := range p.SelfSlots { id, ok := buildSeedForSlot(...); if !ok { return } }`
— a Back **leaves the whole flow** rather than re-prompting, so no slot can be
registered twice with different words. `n ∈ {2,3,4,5}` (`multisigNChoices`,
`gui/multisig_build.go:803`), so the registry holds at most 5 entries.

---

# The matrix — 10 shapes, production chain, output read out

Run: `go test ./gui/ -run 'TestLensDedupeMatrix|TestLensAliasing' -v -count=1`
→ **PASS**, `ok seedhammer.com/gui 0.033s`.

| # | registry shape | facts | what the document says | correct? |
| --- | --- | --- | --- | --- |
| 1 | two DIFFERENT masters, SAME passphrase `alpha` | 2 | `Needs a passphrase: your seed for @0 (…8aaa4f4b)` + `@1 (…0fc4c8a7)` | **yes** — not merged; two secrets, two lines, two fingerprints |
| 2 | one master, `alpha` @0 / `beta` @1 | 2 | both named, fps `8aaa4f4b` / `d70ed067` | **yes** |
| 3 | `alpha` vs `alpha ` (trailing space) | 2 | both named, fps `8aaa4f4b` / `f29a9dab` | **yes** — different derivation units, said so |
| 4 | `café` NFC vs NFD | 2 | both named, fps `2f918343` / `75902abe` | **yes** — this bip39 does not normalise, so they *are* two wallets |
| 5 | same master, bare @0 + `alpha` @1 | 2 | `Needs a passphrase: … @1 (…8aaa4f4b)` **and** `Needs NO passphrase: … @0 (…73c5da0a)` | **yes** — the passphrased/bare split is structurally unmergeable |
| 6 | **pure B1**: one master, `alpha`, @0 and @1 | **1** | `A BIP-39 passphrase WAS used …` + `Without it, these plates do not reach the money …` (the `len(seeds) < 2` arm) | **yes** — one secret, one passphrase, nothing to disambiguate |
| 7 | empty registry (watch-only shape) | 0 | `No BIP-39 passphrase was used …` | unchanged by the fold; **unreachable** from the build path (`SelfSlots` cannot be empty, `gui/multisig_build.go:1008-1012`) |
| 8 | one master at @0..@4 + a bare master @5 | 2 | one merged line naming all five slots against `8aaa4f4b`, plus `Needs NO passphrase: … @5 (…b8688df1)` | **yes** |
| 9 | after `reg.scrub()` | 1 | `No BIP-39 passphrase was used` | not reachable — `passphraseFacts()` is called at `gui/multisig_build.go:479`, inside the flow; `scrub` is the deferred exit. Pre-fold code said the same thing post-scrub, so the merge adds no exposure |
| 10 | 12-word vs 24-word, same passphrase | 2 | both named | **yes** |

**The `len(seeds) < 2` composition, which is where the fold said a paper fix
would have failed, is correct** (row 6): after merging, `len(seeds) == 1` means
*exactly one secret exists*, because a group requires byte-identical passphrase
and so can never absorb a bare seed into a passphrased one. The singular arm's
two lines are then true with no attribution lost. The merged `Label` is
discarded on that arm — correctly, since there is nothing to tell apart — and it
*is* rendered on the mixed arm (row 8, and the fold's own
`TestRestoreDocMergesOneSeedHeldAtTwoSlots`).

---

# Candidates constructed and REFUTED

**R2-A — "the merged line overflows a restore-doc page and its tail is
unreachable." REFUTED, measured.** `restoreDocScreen`
(`gui/singlesig_restore.go:137-165`) always draws the line at `start` (the
`i > start` guard at `:158`) and then pages forward by `shown`, so a line taller
than the content box would have its tail drawn below `contentBottom` and be
permanently unreachable — the merge is the first thing in this file that makes a
single line longer. So I replicated the screen's own geometry and measured
`widget.Labelw` heights on the SH2 display:

```
display=(480,320) lineWidth=464 contentTop=52 contentBottom=276 page=224
pre-fold single-slot passphrase line          chars=175 height= 77  FITS
post-fold merged 4-slot line (worst RENDERED) chars=232 height= 95  FITS
post-fold merged 5-slot line                  chars=250 height= 95  FITS
CONTROL: pre-existing "Seed handling:" line   chars=328 height=113  FITS
```

Worst case is 95 px against a 224 px page, and the branch already ships a
**113 px** line from before this fold, so the merge does not approach the limit.
The worst *rendered* merge is 4 labels, not 5: five held slots from one master
collapse to `len(seeds) == 1` and take the singular arm, which draws no label.

**R2-B — "the group's fingerprint comes from the first member and could belong
to the wrong pair." REFUTED** — one `add` site, one `bindPassphrase` site, both
mainnet; equal pairs give equal fingerprints. Measured in every merged cell.

**R2-C — "a Back re-enters a slot and leaves a stale registry entry whose
passphrase is then printed against a seed not in the wallet." REFUTED** —
`gui/multisig_build.go:231-233` returns from the flow on `!ok`; there is no
re-prompt path.

**R2-D — "the seam `var multisigVerifyFn` (B4) could leak a stub between
tests." REFUTED** — restored via `t.Cleanup`
(`gui/multisig_engrave_tail_walk_test.go:104,114`) and `grep -rn "t.Parallel()"
gui/` returns **nothing**, so no test in the package runs concurrently with it.

---

# Minor — recorded, does NOT gate

**M-1 (Minor) — the dedupe KEY, which is the whole fail-safe argument, is pinned
by no test, and the fold's stated reason for that is wrong.**

`gui/multisig_build_slots.go:298`. The fold's log
(`s5-fold-rereview-fold-round1.md:71-75`) says:

> **NOT PINNED, stated rather than implied.** The CHOICE of key is a fail-safe
> argument, not a behaviour any test here can distinguish. […] exhibiting one
> costs 2^32 work. A reviewer should read the comment, not look for a red test.

That is true only for a registry built through `reg.add`. `seedRegistry.seeds`
is a plain field of an in-package struct, so a test can **state** the collision
instead of finding it. Measured, both directions:

* Mutation — change `:298` to `if g.masterFP == s.MasterFP {` (the exact change
  the comment forbids, because it merges in the funds-losing direction). Full
  package: `go test ./gui/ -count=1` → **EXIT=0**, `ok seedhammer.com/gui
  250.510s`. **Nothing goes red.**
* A 12-line test turns it red. Written and run: green on the real tree, and
  under the mutation

  ```
  --- FAIL: TestLensFingerprintCollisionDoesNotDropAPassphrase (0.00s)
      two UNRELATED masters sharing a 4-byte fingerprint collapsed to 1 fact(s);
      a required passphrase just left the restore document:
      [{Label:your seed for @0 and your seed for @1 MasterFP:3735928559 Uses:true}]
  ```

  The test body, verbatim, as run:

  ```go
  func TestFingerprintCollisionDoesNotDropAPassphrase(t *testing.T) {
      a, _ := bip39.ParseMnemonic(fixtureMasterA)
      b, _ := bip39.ParseMnemonic(fixtureMasterB)
      const collided = uint32(0xdeadbeef)
      reg := &seedRegistry{seeds: []registeredSeed{
          {Label: "your seed for @0", Mnemonic: a, Passphrase: "alpha", MasterFP: collided},
          {Label: "your seed for @1", Mnemonic: b, Passphrase: "alpha", MasterFP: collided},
      }}
      if facts := reg.passphraseFacts(); len(facts) != 2 {
          t.Fatalf("two UNRELATED masters sharing a 4-byte fingerprint collapsed "+
              "to %d fact(s); a required passphrase just left the restore "+
              "document: %+v", len(facts), facts)
      }
  }
  ```

  (Errors elided here for width; the run used `t.Fatal` on both parses.)

**Why this is Minor and not Important, stated plainly so the call can be
disputed:** the shipped behaviour is correct on every reachable input — the
merge cannot fail unsafe today, and a 4-byte collision is not an input an
operator can supply. What is missing is a regression test against a future
"unification" of the two sites, which the comment at `:272-280` already argues
against in prose. It differs from round 1's B4/B5, which were Important because
their mutations restored **operator-reachable** wrong behaviour (a one-shot
offer; an inert abort). This one does not gate the merge. If it is landed, land
it as a test-only commit; **no production change is warranted** — the key is
correct as written.

---

# Facts treated as SETTLED and not re-derived

The five gates and their numbers; `go vet` exit 1 / 40 / 0-outside-`_test.go`;
R-1 refuted; I-8 ruled (b); `gui/singlesig.go` out of scope (F-197, F-198);
F-199, F-200, F-201 filed; the gate record needed no re-mint; the fold's eight
mutations were run and RED. None of the above was re-checked, and no finding
here depends on any of it.

# Commands run for this review

```
git diff --stat 830aaf7..s5-multislot                       (3 commits, 8 files)
diff -q <frozen>/gui/multisig_build_slots.go <copy>/…       IDENTICAL
go test ./gui/ -run 'TestLensDedupeMatrix|TestLensAliasing' -v -count=1   PASS
go test ./gui/ -run 'TestLensRestoreDocLineFits' -v -count=1              PASS
go test ./gui/ -run 'TestLensFingerprintCollisionDoesNotDropAPassphrase' -v -count=1
                                                            PASS on the real tree
  … same test under the g.masterFP mutation                  FAIL (1 fact)
go test ./gui/ -count=1   under the g.masterFP mutation      EXIT=0, ok 250.510s
grep -rn "reg\.add(|\.bindPassphrase(" gui/*.go | grep -v _test.go   2 sites, both mainnet
grep -rn "t.Parallel()" gui/                                 no hits
```

The frozen worktree was read only; every mutation was applied in the copy and
reverted (`diff -q` back to the frozen file confirms `RESTORED`).

---

*Round 2, dedupe fail-safe lens. 10 shapes executed, 4 candidates refuted,
0 Critical, 0 Important, 1 Minor. Gate on this lens: **GREEN**.*
