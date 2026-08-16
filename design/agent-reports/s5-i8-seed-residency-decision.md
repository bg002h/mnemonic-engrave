# S5 / I-8 — Operator decision: seed residency during a multi-plate engrave

Decision requested by the S5 whole-diff review (I-8,
`design/agent-reports/s5-whole-diff-review-round0.md:639`). Scope: this one
question only. Repo read at `/scratch/code/shibboleth/wt-s5` @ 7da66bd.

## DECISION: (b) ACCEPT AND DOCUMENT

## Reasoning, in terms of this device's threat model

**1. On a full build — the 12-plate walk-away case the finding is about — the
registry is not the marginal copy of the secret, and scrubbing it buys nothing.**
Step (9) of the flow (`buildEngraveTail`, `gui/multisig_build_tail.go:107`) calls
`deriveMultisigLeg(seed.Mnemonic, ...)` for every held slot, and in full mode
that mints `b.MS1 = codex32.EncodeMS1(entropy)` (`gui/multisig_derive.go:64-71`)
— the seed entropy re-encoded as an immutable Go string. Those ms1 strings go
into `cardsOut` and are held by `bundleEngrave` for the entire hours-long engrave,
because they are the thing being cut. They cannot be zeroed (Go strings), and the
same words are accumulating ON THE PLATES in the tray. An attacker who reaches an
unattended mid-build machine reads the steel with their eyes; SRAM extraction
from a live, seized RP2350 is not the cheap path and not the realistic one.
Scrubbing the registry early changes nothing about what that seizure yields.

**2. The only build shape an early scrub would protect is watch-only, against an
attacker class this device does not realistically face.** In watch-only mode
`cardsOut` is public-only, so a scrub after the tail would genuinely empty RAM of
seed words during the engrave. The attacker that defeats is one who seizes a
powered, air-gapped, single-purpose device mid-engrave AND can extract live SRAM
— while being unable to have simply watched the operator type the seed into the
same device minutes earlier. The device's whole security premise is physical
custody plus power-off at the end (which clears SRAM); within that premise,
minutes-vs-hours of RAM residency is not a boundary an adversary crosses.

**3. Option (a) as specified by the reviewer is not implementable, which measures
its risk.** The reviewer's scrub point — "as soon as its key is derived in
`buildSelfKeys`" — is *before* the last read of the mnemonics: step (9) re-reads
every registered seed (derived and `both` slots alike) at
`gui/multisig_build_tail.go:103-107`. A scrub there breaks every engrave. A
*corrected* (a) exists (scrub immediately after `buildEngraveTail` returns,
~`gui/multisig_build.go:356`; nothing reads `reg.seeds[].Mnemonic` after that —
verify at step (10) re-derives from a RE-TYPED seed by design, and `bundleEngrave`
takes only `cardsOut`). But it converts the file's stated invariant — "the
registry holds the seeds for the flow's lifetime; ONE scrub site, deferred at
creation" (`gui/multisig_build_slots.go:153-160`) — into "held until step 9",
and any future consumer added after that point fails in the worst class this
codebase knows: BIP-39 seed derivation is PBKDF2 over the sentence with **no
checksum check**, so a zeroed `bip39.Mnemonic` reads back as "abandon abandon…"
and *silently derives real keys from the all-abandon wallet*. (`deriveMultisigLeg`
happens to gate `m.Valid()`; `deriveAccountXpub` callers do not all.) Weigh the
two failure modes: (b)'s is *exposure* to a physical attacker who already owns
the steel; a mis-sequenced or future-eroded (a)'s is *silent wrong keys on a
funds path*. On this device, (b)'s failure mode is strictly the cheaper one —
and this cycle has just produced three Criticals, which is exactly when a
control-flow change on the funds path, whose reference specification was already
wrong once, should not be the pick.

**4. What the walk-away case actually needs is the truth, said to the operator.**
The new exposure at 12 plates is real, but its dominant form is cut seed plates
sitting in an unattended machine — no scrub touches that. The remedy that
addresses the actual threat is the ruling: tell the operator the machine holds
every entered seed, and the plates themselves, until the build ends.

## Required text changes (both go in this diff)

### 1. Replacement for the justification comment, `gui/multisig_build_census.go:58-63`

Replace the "WHY NOT AN IDLE LIMIT." paragraph with:

```
// WHY NOT AN IDLE LIMIT. A timer that scrubs and exits mid-build would fire on
// the operator reading this very document, and would throw away a build that
// costs hours to redo. S5 multiplied what the registry holds -- up to one seed
// per held slot, across distinct masters, for a build that grew to a dozen
// plates over hours -- and the re-decision filed to S5 is now made: STILL NO
// BOUND, on the new premise, stated honestly. On a full build the registry is
// not the marginal copy: the same entropy is in the ms1 engrave strings
// (immutable Go strings, unscrubbable) for the whole engrave, and it
// accumulates on the plates in the tray -- whoever reaches an unattended
// machine reads the steel, not the SRAM. An earlier scrub would protect only
// watch-only builds, against a live-SRAM-extraction attacker this air-gapped,
// physically-custodied device does not defend against anyway, and it would
// break the one-site scrub invariant: BIP-39 derivation is checksum-free
// PBKDF2, so a future read-after-scrub silently derives the all-"abandon"
// wallet. The walk-away exposure is answered where it lives: the ruling below
// tells the operator the machine holds every entered seed, and the plates
// themselves, until the build ends.
```

### 2. Replacement for the operator-facing "Seed handling" ruling in `buildPlateInventoryLines` (`gui/multisig_build_census.go:75-77`)

Replace the current string with:

```go
lines = append(lines, "Seed handling: this build does not time out. Every seed "+
    "you entered -- this build can hold several -- stays in device memory until "+
    "the build ends, and on a full build the words are also on the plates as "+
    "they are cut. Do not leave a mid-build machine unattended: the plates are "+
    "the secret. Power the device off when you are done.")
```

The stale "holds exactly one seed" premise (comment) and the singular "A seed you
entered" (ruling) must not survive the fold; both replacements above are written
to be dropped in verbatim.
