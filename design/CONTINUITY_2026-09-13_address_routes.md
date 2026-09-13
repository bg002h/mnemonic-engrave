# Continuity — F-531 and F-530, before the flash

**Written 2026-09-13 at the operator's direction.** Both must be fixed before
the device is flashed. Everything below is measured, not recalled.

## Where the repos are

| repo | branch | tip | state |
| --- | --- | --- | --- |
| `mnemonic-engrave` | `master` | `e1c67a0e` | pushed, check earned |
| `seedhammer` (fork) | `main` | `3bb9f91` | pushed |
| `descriptor-mnemonic` | `main` | `40c400de` | pushed, both contexts earned |

All three clean, nothing in flight, no agents running.

## F-531 — funds-critical, do this one first

Two address routes in one binary return DIFFERENT addresses for a repeated-slot
multisig. `expandedToDescriptor` projects the policy to **one key per slot**
while keeping `tpl.K`, so for `wsh(sortedmulti(1,@0,@0,@1))`:

| route | address |
| --- | --- |
| flat (`expandedToDescriptor` → `address.Receive`) | `bc1qvcrd8s7…hw9yuw` |
| the emitter | `bc1qxdqrua3…d7vp5p` |

The screen says **1-of-3** and shows a **1-of-2's** address. Found by the F-514
reviewer; full detail in `design/agent-reports/duplicate-key-warning-review.md`
under `C-2`.

**Decide before coding**: dropping a repeated slot changes the script and so the
address, so "project to one key per slot" and "keep K" cannot both stay. The
emitter's answer is the one the plates reconstruct, which makes it the candidate
for correct — **measure it against Bitcoin Core rather than assuming.** Core
25.0.0 is on this box at `/usr/local/bin` (reports `/Satoshi:25.0.0/`); boot a
throwaway regtest datadir under `/scratch/code/shibboleth/.tmp`, use
`getdescriptorinfo` / `deriveaddresses`, and delete the datadir after.

## F-530 — the address route still silent

`descriptorFlow` → `DescriptorScreen.Confirm` → `descriptorAddressFlow` shows
addresses with no duplicate-key warning. Three callers, and they do not share an
input: two have md1 chunks, one is a **scanned descriptor with none**. So it
needs the rule over a `*bip380.Descriptor`, not over chunks — which is the more
valuable version anyway, since it also catches a pasted descriptor.

Do not thread chunks into the two callers that have them: that closes two thirds
and leaves the third silent under a suite that then looks complete, which is
exactly how the first surface stayed hidden through a whole review round.

**The argument that this is a gap rather than a hole is already written down**,
in `scriptForTemplate` (`gui/md1_expand.go`), with
`TestScriptForTemplateAdmitsOnlyTwo` to fail the day it stops holding. Read it
first — F-531 may change it, because that argument rests on
`expandedToDescriptor` behaving as it currently does.

## What this session established that you will need

- `md.DuplicateKeySlot` (`md/duplicate_keys.go`) returns a `DuplicateKind`:
  `DuplicateRefusedByCore` (wsh/sh miniscript) or `DuplicateFewerKeys`
  (top-level multi/sortedmulti, and **every** taproot shape). Taproot is
  fewer-keys because **Core 25.0.0 has no tapscript miniscript at all** —
  `tr(A,and_v(…))` returns *"Miniscript expressions can only be used in wsh"* —
  so a tapleaf `multi_a` never reaches `CheckDuplicateKey`. That constant's doc
  says what would falsify it on a newer Core.
- The warning is on three surfaces: `composerConsentLinesFor`,
  `walletPolicyAddressLines`, `policyIDHeader`. One test asserts all three.
- **Assert against `composerCopyDuplicateKeys`, never a literal.** Three tests
  broke on copy edits this session because they hardcoded the words.
- The policy harness: `scripts/policy-generate.py --corpus` is the standing
  gate (67 vectors against `design/policy-corpus-baseline.json`), and
  `--edges` generates boundary-lock policies. `design/POLICY_HARNESS.md`
  explains all four programs.

## Gates to run

```sh
# fork
scripts/gui-shard-test.sh ./gui/ 24        # from mnemonic-engrave, run in the fork
go test $(go list ./... | grep -v "/gui$")
scripts/fork-vet-gate.sh
nix develop -c tinygo build -size short -o /dev/null -target pico-plus2 \
  -stack-size 16kb -gc precise -opt 2 -scheduler tasks ./cmd/controller

# derivation is untouched
python3 scripts/policy-generate.py --corpus
```

Last measured at fork `3bb9f91`: gui **1307/1307** across 24 shards (partition
verified exhaustive), all 75 non-gui packages, vet clean beyond the known TinyGo
gap, tinygo **1,650,652 flash / 63,304 ram**, corpus gate all 67 matching.

## Also open, not blocking the flash

F-529 (the fork's vendored corpus has diverged from the primary under identical
names — on exactly the three key-reuse vectors this work depends on; a re-vendor
would remove them), F-532 (the review's Minors; **M-7** is a sentence false at
k = 1, **M-4** is a test comparing a string against itself).

Operator-owned and unchanged: the flash, the H4 walk, ACCEPTANCE item 8.
