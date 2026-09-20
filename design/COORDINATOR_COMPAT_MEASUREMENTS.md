# Coordinator-compatibility — controller measurements

Recorded during the 2026-09-20 brainstorm so the design rests on measurement
rather than recollection. Companion to
`design/DESIGN_coordinator_compatibility.md`.

## The Core 25 -> 31.1 boundary is ONE capability, not a scatter

Ruling 1 requires the boundary release to be measured. Computed from
`design/IMPORTABILITY_composer_shapes.md`, the 56-shape matrix:

**15 shapes discriminate Core 25.0 from Core 31.1, and every one of them is
`tr`.** Core 25 refuses each with either `Miniscript expressions can only be
used in wsh` or a `multi_a(...)` parse failure; Core 31.1 accepts all 15.

    preset-simple-timelocked-inheritance-tr   preset-kofn-recovery-tr
    preset-tiered-recovery-tr                 preset-hashlock-gated-tr
    preset-decaying-multisig-tr               hashlock-gated-tr-hash160
    mixed-lock-bases-tr                       same-seed-two-paths-tr
    X09-tr-1of1-2of3older                     X10-tr-1of1-tworec
    X18-tr-inherit-older5                     X19-tr-kofn-nums-older5
    X20-tr-hashlock-known                     X23-tr-1of1-1of1older-h-spelling
    X26-tr-2of3-1of1-unlocked-plus-rec

**Zero `wsh`, `sh` or `sh(wsh)` shapes differ between the two releases.**

### What this buys the design

The boundary is a single capability — **tapscript miniscript** — so measuring
it does not mean re-running 15 shapes against 6 releases. It means running
**one representative `tr` policy with a tapscript-miniscript leaf** against
each candidate release until the answer flips. One descriptor, one
`getdescriptorinfo` call per release.

It also says something about the rule, not just the number: the Core RuleSet's
discriminator should be *"does this policy put miniscript under `tr`"*, with
the version span attached to that capability. A future coordinator that gains
the same capability at some release is then the same rule with a different
span, which is what ruling 6 asked for.

### What is NOT yet measured — the open task

The boundary release itself. Locally only Core 25 is installed
(`/usr/bin/bitcoind` and `/usr/local/bin/bitcoind` both report
`Bitcoin Satellite version v0.2.4`, which is the matrix's "Core 25.0"); the
31.1 build the review used is no longer on the box. Measuring the boundary
needs releases 26 through 31 fetched from bitcoincore.org and the single
representative descriptor run against each.

**Do not infer it from release notes and write a number into the design.**
Ruling 1 says measured, and a version printed on a consent screen that turns
out to be wrong is exactly the false positive this whole feature is built to
avoid — it would tell an operator on Core 26 that their taproot policy
imports when it does not.

Until it is measured, the Core RuleSet cannot claim a lower bound above 25,
and the honest span for a tapscript-miniscript policy is "refused on 25.0,
accepted on 31.1, boundary unmeasured" -- which under ruling 3 means the
verdict is `Unproven` and the row stays silent rather than naming a version.
