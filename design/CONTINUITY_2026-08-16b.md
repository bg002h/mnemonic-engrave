# Continuity — 2026-08-16b: S5 is GREEN, MERGED and PUSHED. Everything is safe on a remote.

Supersedes `CONTINUITY_2026-08-16.md`. Read this one.

## THE THING THAT CHANGED

The predecessor's first line was **"NOTHING IS PUSHED"**, with the whole stage
living in one worktree that a single `rm -rf` would have destroyed. That is over.
**S5 passed its gate at 0C/0I, merged, and both repos are pushed and in sync.**

## STATE

| repo | branch | head | vs origin |
| --- | --- | --- | --- |
| `mnemonic-engrave` | `master` | `ccdea9a` | **0 0 — in sync** |
| fork `/scratch/code/shibboleth/seedhammer` | `main` | `b8a23bf` | **0 0 — in sync** |
| fork worktree `/scratch/code/shibboleth/wt-s5` | `s5-multislot` | `2b9a128` | merged into `main` |

`b8a23bf` is the `--no-ff` merge of `s5-multislot`. Its tree hash is **identical**
to the gated head `2b9a128` (`7664ca98a920c627a5fe5abcac0fff9c4f2d4691` both), so
the gate's result carries to the merge commit rather than being assumed to.

`master` was pushed through the `ci/staging` ritual. `test (rust + go)` on
`ccdea9a` came back **completed / success** from run `31957955186`, and the
subsequent `git push origin master` printed **no bypass message** — the check was
SATISFIED, not bypassed. `ci/staging` is deleted (confirmed with a positive
control, since an empty `ls-remote` is also what a broken query looks like).

**The `wt-s5` worktree is now redundant** — everything in it is merged and
pushed. It is safe to remove, but nothing depends on removing it.

## HOW S5 CLOSED

Four gates and three folds. Every reviewer report was persisted **verbatim in its
own commit** before the response to it was written, so `git diff <report>..<fold>`
shows exactly what changed in response to what.

| round | result |
| --- | --- |
| 0 | RED — 3 Critical, 14 Important (C-1..C-3, I-1..I-14) |
| 1 | RED — 0 Critical, 5 Important (B1..B5) |
| 2 | RED — 0 Critical, 1 Important (test-only) |
| closure | **GREEN — 0 Critical, 0 Important** |

Reports: `design/agent-reports/s5-whole-diff-*.md`, `s5-fold-rereview-*.md`,
`s5-rereview-round2*.md`, `s5-round2-closure-check.md`,
`s5-i8-seed-residency-decision.md`.

## WHAT THE GATE ACTUALLY BOUGHT — the number to remember

**9 of round 0's 17 blocking findings were reproduced by mutating the tree and
watching a green suite stay green.** The build gate saw none of them. All of them
were in code that had already closed a stage green.

The three Criticals:

1. **"Verify Incomplete: Checked N of the M key plates" asserted a comparison that
   never ran.** `len(legs)` counted slots re-derived from a *typed seed*, not
   plates compared, and the only comparator call site was unreachable from that
   branch. A foreign mk1 passed.
2. **SPEC 4.3's multi-account notice was dead in the shipped flow** — the gate
   grouped by `SeedID` while the flow mints one per held slot. The operator would
   read two labels that look like two independent secrets; lose the one ms1 and
   **both** their keys are gone, because mk1 plates are public keys and cannot
   sign.
3. **The SUPPLY path labelled a passphrase build "Full (seed + keys)"** and its
   restore document never mentioned a passphrase. S5 built the truthful label for
   exactly this harm and wired it to the BUILD path only.

## FIVE LESSONS THIS CYCLE PAID FOR

1. **A reviewer reproduces the DEFECT, not the REMEDY.** Three prescribed fixes
   failed on contact with the code, and **I-8's would have introduced a Critical**
   — it said to scrub seeds at `buildSelfKeys`, but `buildEngraveTail` re-reads
   every registered seed afterwards, and a zeroed `bip39.Mnemonic` reads back as
   "abandon abandon…" under checksum-free PBKDF2, silently deriving real keys from
   the wrong wallet. Resolve every prescription against the call graph before
   implementing it.
2. **Folds fail by incomplete propagation.** The round-0 fold re-keyed the gate on
   `MasterFP` for C-2 and left the sibling `passphraseFacts` keyed on registry
   position — it diagnosed the identity mistake and re-committed it three files
   away. Grep the superseded phrasing; reading the diff does not find this.
3. **A dedupe must only ever fail SAFE.** Grouping the restore document on
   `MasterFP` would merge two unrelated seeds on a 4-byte collision and **drop a
   required passphrase**, where the same collision in the gate only adds a
   spurious notice. Same rule, opposite directions, two different answers.
4. **A gate that has never executed is a hypothesis.** Round 2's only finding was
   an ms1 rejection arm that no test had ever driven; reverting it restored a real
   dead end while the package stayed green.
5. **A gate you cannot read is not a gate.** `gofmt` appeared to report 1 file and
   `go vet` 41 findings; both were nix's `Git tree is dirty` warning bleeding from
   stderr into a `2>&1` capture. Separate the streams. Judge on true exit codes.

## OPEN FOLLOW-UPS

**F-198 is a CRITICAL and it gates S6.** The **single-sig** flow takes a
passphrase into derivation (`gui/singlesig.go:64-72, :90`), hard-codes
`"Full (seed + keys)"` (`:80`), and calls a `restoreDocFlow` that has **no
passphrase parameter at all** (`gui/singlesig_restore.go:119`). So a "Full"
single-sig engrave with a passphrase cuts ms1 — the words only — and hands the
operator a document saying the set is complete. Permanently unspendable.
**Pre-existing** (already in `main` from `b100425`; S5 only added a title
argument), which is why it did not gate the merge — but S6 flashes firmware an
operator engraves real backups with, and this path is reachable from the front
door. Both the round-0 review and the fold declined to assert the harm because
neither had checked; the answer cost one grep.

| id | severity | owning phase |
| --- | --- | --- |
| **F-198** | **Critical** | **S6 — before the hardware cycle** |
| F-197 | Important | S6 |
| F-199 | Important | S6 |
| F-196 | Important | the spec (a model change, earns its own R0) |
| F-200, F-201 | Minor | ownerless residue |

## WHAT IS NEXT — S6, hardware

One flash cycle via `~/bin/sh/sh2-flash` (**never** `picotool` by hand — the build
output is unsigned and a laptop port cannot boot the machine). At least one build
must be divergent-origin, multi-slot **and** multi-master, and master B's mnemonic
must restore from its engraved ms1 plate.

**Burn F-198 down first.** It is a Critical on a path the hardware cycle will
exercise, and the rule is that an item scheduled to a phase is not deferrable past
it.

S5 is a milestone, not the destination: the north star remains **arbitrary wallet
policies on the SH2**, with arbitrary `wsh`/`tr` miniscript in phase 2.

## TOOLCHAIN

    export PATH="/nix/var/nix/profiles/default/bin:$PATH"
    cd /scratch/code/shibboleth/seedhammer
    nix develop --command go test ./... -count=1
    nix develop --command ./cmd/emu/build.sh   # go test does NOT compile the emulator
    nix develop --command ./scripts/oracle-live.sh

Bare `go` does not exist on PATH — only inside `nix develop --command`; a
"command not found" proves nothing. `go vet` needs a **COLD** `GOCACHE` or it
prints nothing, exits 0 and proves nothing; **exit 1 with 40 test-only findings IS
the clean baseline**. `oracle-live.sh` must run inside `nix develop` too.

Emulator: serve `cmd/emu` on a **FRESH port** (the browser caches `emu.wasm`) and
prove the rebuild **by its byte size**. `md` is a shell alias for `mkdir -p` —
invoke pinned binaries by **absolute path** (`~/.cargo/bin/{md,mk,ms}`). md's
**template** syntax puts the origin AFTER the placeholder and **rejects `h`
notation**. `gh` needs `--repo bg002h/<name>`; `head_sha` queries need the full
40-char SHA, and judge **per-job** conclusions, not run-level status.

## PUSHING

    git push origin master:refs/heads/ci/staging
    gh run watch <id> --repo bg002h/mnemonic-engrave
    git push origin master          # no bypass message = satisfied
    git push origin --delete ci/staging

Required contexts differ per repo — `mnemonic-engrave` → `test (rust + go)`. The
fork's `main` is **unprotected**: plain push, no ritual.
**`enforce_admins: false` is the operator's deliberate hatch — never propose
flipping it.**
