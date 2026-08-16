# Continuity — 2026-08-16: S5's mint has RUN and is green. Nothing is pushed.

Supersedes `CONTINUITY_2026-08-15d.md`. Read this one.

## THE TWO THINGS THAT MATTER BEFORE ANYTHING ELSE

1. **NOTHING IS PUSHED.** `mnemonic-engrave` `master` is **16 commits ahead** of
   `origin/master`. The fork branch **`s5-multislot` (10 commits) has NEVER been
   pushed** and exists only in the worktree `/scratch/code/shibboleth/wt-s5`.
   A single `rm -rf` of that worktree loses the entire stage.
2. **The built-policy mint has now EXECUTED and is GREEN.** That was S5's
   deferred gate — "a gate that has never run is a hypothesis". It ran.

## STATE

| repo | branch | head | unpushed |
| --- | --- | --- | --- |
| `mnemonic-engrave` | `master` | `288efe1` | **16** |
| fork worktree `/scratch/code/shibboleth/wt-s5` | `s5-multislot` | `7da66bd` | **10, never pushed** |
| fork `/scratch/code/shibboleth/seedhammer` | `main` | `84a4f4a` | 0 |

Gate at `7da66bd`, measured unpiped on true exit codes:

    go test ./... -count=1     exit 0 · 51 ok / 0 FAIL
    gofmt -l ./                exit 0 · empty
    go vet ./...               exit 1 · 40 findings · 0 outside _test.go  (COLD GOCACHE)
    ./scripts/oracle-live.sh   PASS · 7 discovered, 7 ran

`go vet` exiting **1** with 40 findings **is** the clean baseline, and `GOCACHE`
must be **cold** or vet reports exit 0 with no output and proves nothing.

## WHAT S5 DELIVERED

Every block below is built, gated and independently reviewed to 0C/0I.

- **Model** — `SelfSlot int` → `SelfSlots []int`; `cosignerFromCard` carries the
  card's origin; `OriginDivergent` when origins differ; S2's interim
  foreign-origin refusal removed with §4.1's duplicate check still running first;
  `derivedSlotOrigin` template-aware per §0.1a (`sh(wsh)` → `1'`).
- **Engrave tail** — one mk1 per held slot at that slot's own origin, one ms1 per
  distinct seed, emitted all-ms1s → all-mk1s → md1 for the oracle's
  consecutive-run contract.
- **Picker** — multi-select, so the flow can finally express Trace B.
- **Verify** — per-leg bijection; obligation carries the **engraved slot set AND
  the engraved md1**; `expectedSlots ∩ allUserSlots(seed)`.
- **Supply path** — engraves a plate per matched slot (F-188, operator ruling),
  byte-identical plates deduped to one and announced before the first cut.
- **Screens** — keys on the review, EXPERIMENTAL warning rewritten, DESTROY not
  discard, passphrase disclosure, interruption story, F-182, and F-185's
  **drawn-frame class check**.
- **Walk + mint** — `cmd/emu/walk_trace_b.js` (17 plates, `presented:0`) and
  `oracle/gaterecords/S5-trace-b.*`, record `06888d28…`, expect `adf13d2f…`.
  **17/17 artifacts byte-identical** to the pinned primary — 2 ms1, 7 mk1, 8 md1,
  full string equality.

## THE FOUR CRITICALS THIS CYCLE FOUND, AND WHAT EACH TEACHES

Recorded because every one was found by a *different lens*, not by looking harder.

1. **The ms1 dedupe keyed on `SeedID`** while the flow registers one entry **per
   held slot** — so Trace B minted 3 seed plates for 2 seeds, numbered "2 of 3".
   Now keyed on the **ms1 string**. *A dedupe must only ever fail SAFE: the
   master fingerprint was rejected because a 4-byte collision would DROP a
   plate.*
2. **The guarding fixture modelled a registry the flow cannot produce**, which is
   why a green suite AND a clean 9-mutation false-PASS hunt both missed (1).
3. **The verify's obligation carried slot indices but not the POLICY.** Present
   another wallet's plates → "Verify OK" with the just-cut plate never read.
   Same class as a rejected plate-COUNT design: *cardinality is not identity.*
4. **Two emulator walks were broken and CI could not see it.** Three of the four
   breakages were introduced by **S4, a stage that closed green**.

## THE PROCESS LESSON, in one line

**An unpinned fix is indistinguishable from an inert one.** Two fixes reached
commits this cycle while being provably inert — deleting them left the suite
green. Every mechanism must now be **mutation-checked at FLOW level**, with a
printed marker proving the deleted line *ran*. Helper-level proof is not proof.

## WHAT IS LEFT

**S5 is not closed.** Remaining, in order:

1. **A final independent adversarial review over the WHOLE S5 diff** (`main..s5-multislot`, 10 commits). Not yet run. This is the mandatory post-implementation gate and the last thing before merge.
2. **Merge `s5-multislot` → fork `main`**, then push. **Fork `main` is unprotected — plain push, no staging.**
3. **Push `mnemonic-engrave` `master`** via the `ci/staging` dance below.
4. **S6 — hardware.** One flash cycle via `~/bin/sh/sh2-flash` (never `picotool`).
   At least one build must be divergent-origin, multi-slot **and** multi-master,
   and master B's mnemonic must restore from its engraved ms1 plate.

**Open follow-ups filed this cycle:** F-188 (ruled, done), **F-189** (retired
APIs with no callers), **F-190** (fixed), **F-191** (passphrase misattributed as
"seed is not a cosigner"), **F-192**–**F-195** (F-185 sweep, two xpub renderings,
review page-break, watch-only silence on the absent seed).

## THE NORTH STAR, restated by the operator 2026-08-16

**Arbitrary wallet policies on the SH2.** That is *wider than this plan* — the
multisig-build-repair plan delivers k-of-n sorted-multisig and puts arbitrary
wsh/tr miniscript in **phase 2** (§1, §4). S5 is a milestone, not the
destination.

**Standing directive:** proceed autonomously and unceasingly; **route blocking
questions to a fable agent** rather than stopping. Be **permissive on input,
expressive on output, and loud about every assumption** — refusal wins only where
a wrong assumption would be *invisible in every artifact the operator keeps*.

## TOOLCHAIN

    export PATH="/nix/var/nix/profiles/default/bin:$PATH"
    cd /scratch/code/shibboleth/wt-s5
    nix develop --command go test ./... -count=1
    nix develop --command ./cmd/emu/build.sh   # go test does NOT compile the emulator
    ./scripts/oracle-live.sh

Emulator: serve `cmd/emu` on a **FRESH port** (the browser caches `emu.wasm`) and
prove the rebuild **by its byte size**. `md` is a shell alias for `mkdir -p` —
invoke pinned binaries by **absolute path** (`~/.cargo/bin/{md,mk,ms}`). md's
**template** syntax puts the origin AFTER the placeholder and **rejects `h`
notation**. `gh` needs `--repo bg002h/<name>`; `head_sha` queries need the full
40-char SHA.

## PUSHING

    git push origin master:refs/heads/ci/staging
    gh run watch <id> --repo bg002h/mnemonic-engrave
    git push origin master          # no bypass message = satisfied
    git push origin --delete ci/staging

Required contexts differ per repo — `mnemonic-engrave` → `test (rust + go)`.
Copying a sibling's block waits forever on a check that never reports.
**`enforce_admins: false` is the operator's deliberate hatch — never propose
flipping it.**
