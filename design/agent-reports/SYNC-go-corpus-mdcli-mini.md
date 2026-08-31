# SYNC — Go vendor corpus sync, mdcli-mini lockstep

Agent report. Worktree `/scratch/code/shibboleth/seedhammer-corpus-sync`
(isolated, via `git worktree add`), branch `mdcli-corpus-sync`, off the fork's
`master` at `5f02773` (merge f440/modal-back). Not pushed anywhere — per
brief, this leaves a branch only.

**Outcome: sync complete, committed, all scoped Go tests green (122/122),
gate independently verified twice (once by the controller's push, once by
this agent against origin/main's actual committed content).**

---

## 1. Trigger and background

`descriptor-mnemonic`'s `mdcli-mini` cycle (P2, N1 admission taxonomy) ruled
three of the corpus's multi-key vectors BIP-388-forbidden: each bound the
same `@i` placeholder to two use sites in one descriptor. `md-codec`'s
`test_vectors.rs` MANIFEST replaced them with distinct-placeholder
equivalents in the same shapes, and `md vectors --out
crates/md-codec/tests/vectors` regenerated exactly 15 committed files (3
vectors x 5 files). Source: `IMPL-mdcli-mini-P2.md` §2/§5, commit
`8a71594a` ("P2.5-6: blast-radius dispositions, the two flipped rows, stale
comments").

Per the Rust-primary rule, the Go port is strictly downstream and must
re-vendor byte-for-byte, never re-derive.

## 2. File mapping — all 15, byte-for-byte

Vendoring point found via
`crates/md-cli/tests/corpus_origin_consistency.rs:12` ("vendored into the Go
port and compared byte for byte"). The fork mirrors the Rust side's flat
`<name>.<ext>` naming with no path translation:

| Rust source (`descriptor-mnemonic/crates/md-codec/tests/vectors/`) | Go vendor twin (`seedhammer/md/testdata/vectors/`) |
| --- | --- |
| `keyed_tr_sortedmulti_a.template` | `keyed_tr_sortedmulti_a.template` |
| `keyed_tr_sortedmulti_a.bytes.hex` | `keyed_tr_sortedmulti_a.bytes.hex` |
| `keyed_tr_sortedmulti_a.phrase.txt` | `keyed_tr_sortedmulti_a.phrase.txt` |
| `keyed_tr_sortedmulti_a.descriptor.json` | `keyed_tr_sortedmulti_a.descriptor.json` |
| `keyed_tr_sortedmulti_a.conformance.json` | `keyed_tr_sortedmulti_a.conformance.json` |
| `keyed_tr_multi_a.template` | `keyed_tr_multi_a.template` |
| `keyed_tr_multi_a.bytes.hex` | `keyed_tr_multi_a.bytes.hex` |
| `keyed_tr_multi_a.phrase.txt` | `keyed_tr_multi_a.phrase.txt` |
| `keyed_tr_multi_a.descriptor.json` | `keyed_tr_multi_a.descriptor.json` |
| `keyed_tr_multi_a.conformance.json` | `keyed_tr_multi_a.conformance.json` |
| `keyed_wsh_timelock_hashlock.template` | `keyed_wsh_timelock_hashlock.template` |
| `keyed_wsh_timelock_hashlock.bytes.hex` | `keyed_wsh_timelock_hashlock.bytes.hex` |
| `keyed_wsh_timelock_hashlock.phrase.txt` | `keyed_wsh_timelock_hashlock.phrase.txt` |
| `keyed_wsh_timelock_hashlock.descriptor.json` | `keyed_wsh_timelock_hashlock.descriptor.json` |
| `keyed_wsh_timelock_hashlock.conformance.json` | `keyed_wsh_timelock_hashlock.conformance.json` |

Both directories hold 131 (Rust) / 149 (Go, which additionally vendors an
older 10-vector set plus template-only fixtures) files respectively; no
additions or orphans on either side from this sync — exactly the 15 named
files changed, confirmed by `cmp` before and after (`0/15` matched pre-sync,
`15/15` matched post-sync).

## 3. Origin gate — evidence

Hard gate per the brief: the corpus is authoritative only once pushed.

- Initial fetch (before the push landed): `origin/main` =
  `d3676fb1d43c9b71ddab5799e933718914d8b4dc`; `bdb031a4` NOT yet an ancestor.
  Polled every 60s via a background monitor for ~13 minutes.
- Controller notified the push had landed. **Independently re-verified
  rather than trusted:**
  - `git -C descriptor-mnemonic fetch origin` fresh, then
    `git rev-parse origin/main` → `bdb031a4cb54a9f57510af98db81386c360e9b70`
    exactly (origin/main's tip IS `bdb031a4`).
  - `git merge-base --is-ancestor bdb031a4 origin/main` → true.
  - `git log 8a71594a..origin/main --oneline -- crates/md-codec/tests/vectors/`
    → empty (no change to the corpus path between the regenerating commit and
    origin/main).
  - Local `descriptor-mnemonic` main had moved on to `39470c0d` (11 commits
    past `bdb031a4`, unrelated post-ship riders — followups/report commits).
    `git log bdb031a4..HEAD --oneline -- crates/md-codec/tests/vectors/` →
    empty, confirming the corpus was untouched by those riders too.
  - Final, strongest check: diffed the 15 synced files not against the local
    working tree but against `origin/main`'s actual committed blobs via
    `git show origin/main:crates/md-codec/tests/vectors/<name>.<ext> | cmp -`
    for all 15 — **15/15 matched**.

Gate satisfied and independently confirmed at the byte level against
`origin/main`, not merely against a local checkout that could have drifted.

## 4. Provenance pin

Updated: `seedhammer/md/testdata/vectors/../README.md` (i.e.
`md/testdata/README.md`), following the existing dated-section convention
(precedent: the F-217 re-vendor section, commit `0e180f6`). New section
`### REGENERATED 2026-08-31 -- BIP-388-forbidden repeated placeholders
replaced (mdcli-mini N1)`, citing source `descriptor-mnemonic 8a71594a`,
describing each vector's shape change, and stating explicitly that this is a
byte-for-byte re-vendor with the N1 taxonomy itself NOT ported (§6 below).
The file's top-level pin (`Commit: 5a0a4f41 …`) is untouched, consistent with
how the F-217 section was added without touching that unrelated top pin —
it scopes the original 10-vector unkeyed corpus, not the `keyed_*` set.

## 5. Test results

Scoped to the `md` package only (not `./gui/`'s 886 tests, per the brief —
`go` toolchain found at `/home/bcg/.local/go/bin/go`, `go1.26.4`).

- **Baseline, before sync:** `go test ./md/` → `ok seedhammer.com/md`, 122
  `--- PASS`, 0 `--- FAIL` (includes `TestKeyedConformanceAgreesWithRust` and
  `TestPolicyShapeDescribesRealCards`, both fully green on the OLD vector
  content).
- **After sync:** `go test ./md/` → `ok seedhammer.com/md`, 122 `--- PASS`, 0
  `--- FAIL`. Identical count.
- **No expectation adjustments were needed.** Checked specifically:
  - `TestKeyedConformanceAgreesWithRust` (`md/conformance_keyed_test.go`)
    discovers vectors via `filepath.Glob("testdata/vectors/keyed_*.conformance.json")`
    — no hand-maintained name list to update.
  - `TestPolicyShapeDescribesRealCards` (`md/policy_shape_test.go:49`) is the
    **only** site anywhere in the fork (`.go` files, whole repo) that names
    `keyed_tr_sortedmulti_a` literally: `{vector: "keyed_tr_sortedmulti_a",
    keyPath: KeyPathSpendable, branches: 1, wantK: 2, wantN: 2}`. The
    regenerated vector keeps the same shape (`sortedmulti_a(2, @1, @2)`, still
    2-of-2 in one leaf — only the keys/origins changed from a repeated `@0` to
    distinct `@1`/`@2`), so the row holds unchanged and passed as-is.
  - `keyed_tr_multi_a` and `keyed_wsh_timelock_hashlock` have no hardcoded
    literal references anywhere in the fork's `.go` files outside the vendored
    JSON/hex/txt data itself.
  - No other package (`gui`, `sysw`, `seal`, `oracle`, `address`, …) names any
    of the three vector strings.

## 6. NOT ported — the N1 admission taxonomy (obligation recorded)

Per the brief, the N1 admission taxonomy — the new Rust-side refusal that
made `md encode`/`--template` refuse the repeated-placeholder shapes
(`R-N1a`/`R-N1c`/`R-N1d`, `crates/md-cli/src/parse/reuse.rs`,
`CliError::Unsupported`) — is **NOT** ported in this sync. This port has no
`md encode`/template-mint path at all, so there is nothing on the Go side to
converge for the refusal's *behavior*; only the corpus's byte *content*
crossed, which is what this sync did.

**No FOLLOWUPS/ledger file exists in the `seedhammer` fork** (checked:
`find . -iname "FOLLOWUPS*" -o -iname "*ledger*"` → nothing under version
control). Recording the obligation here for the controller to file in
`mnemonic-engrave`'s `design/FOLLOWUPS.md` or equivalent:

> **Obligation:** if/when the Go port grows its own template-side admission
> logic (an encoder, or a `--template` mint path), it must independently
> arrive at the same repeated-placeholder refusal the Rust primary now has
> (R-N1a/R-N1c/R-N1d), per the Rust-primary-leads rule. Not scoped or
> scheduled by this sync — this sync only moved the corpus's byte content,
> which required no code change on the Go side because the port cannot mint
> the forbidden shape either way.

## 7. Commit

Branch `mdcli-corpus-sync` in worktree `/scratch/code/shibboleth/seedhammer-corpus-sync`,
**not pushed** (per brief — the fork has its own merge/flash conventions).

```
8e564c5e8cd4fb25db76dc797a0fe17e05267331
md: re-vendor keyed_tr_sortedmulti_a/keyed_tr_multi_a/keyed_wsh_timelock_hashlock -- N1 repeated-placeholder replacement (mdcli-mini lockstep)
```

16 files changed (15 vectors + `README.md`), +188/−73. Tree clean after
commit (`git status --short` empty). Staged explicitly by path, not `git add
-A`.

## 8. Scope note

This report was written by the same agent that did the sync (not a
separate reviewer) — per the brief's ask, it documents what was done and
found, not an independent review. No adversarial/independent review was
requested or run for this mechanical, glob-scoped, fully-green sync; flag to
the controller if one is wanted before this branch is merged in the fork.
