# Continuity — 2026-08-18: **S6b IS SHIPPED AND FLASHED.** It boots as `bg5bfc118`.

Supersedes `CONTINUITY_2026-08-17.md`. Read this one.

---

## ▶ START HERE

**S6b is merged and pushed. There is nothing open on it.** Do not re-review it,
do not re-run its gates, do not re-litigate its decisions.

| repo | branch | head | state |
| --- | --- | --- | --- |
| fork `bg002h/seedhammer` | `main` | `5bfc118fb6524a2ab8722aa643ccfae853c9c99f` | pushed direct (unprotected); **all CI jobs green** |
| `bg002h/mnemonic-engrave` | `master` | `723a78fccad666e725b386ba009a77ef6c6c6ae3` + the trailing push record | pushed via `ci/staging`, check **SATISFIED** |
| worktree `wt-s6b` | `s6b-pre-flash` | merged | **safe to remove** |

**The hardware flash is DONE, same day.** `seedhammerii-v0.0.0-bg5bfc118.signed.uf2`
(sha256 `7fe6700b…7281258`) was built from fork `main` `5bfc118`, signed against
the burned OTP key, flashed with `picotool load --verify`, and **boots on machine
power with `bg5bfc118` on the version line** — so the bootrom accepted the
signature and the device is running this tree. What that does NOT yet prove is
that S6b's flows BEHAVE correctly on hardware; that is operator verification,
not a boot. Use `~/bin/sh/sh2-flash` — never `picotool` by hand; the
build output is unsigned and a laptop port cannot boot the machine. The boot key
has been burned since 2026-08-03 (slot 1, permanent; slot 0 recovery path
intact).

**The suite is fully green**, on both machines:

    scripts/gui-shard-test.sh ./gui/ 6 20m   869 tests, partition exhaustive,
                                             6/6 shards ok, ~138s wall
    CI `tests` job                           ok  seedhammer.com/gui  386.065s

---

## WHAT S6b SHIPPED

Twenty-one commits on the branch, 49 files, +5243/−245. The cycle's subject is
the gap between **what the machine does and what the machine says**.

| phase | shipped |
| --- | --- |
| **P1** | the verify tail. **F-199**: a correctable readback refusal returns `verifyIncomplete` — the verdict both engrave callers re-offer on — instead of terminal `verifyRefused`. Per-site, because three other sites share that verdict and must never loop. **F-206**: the ms1 pass clause stops being singular on a multi-seed multisig. **R-M** rewrites the `provedInnocent` arm to stop advising a skip. |
| **P2** | plate marking on the predicate *"the set contains a seed"* — a watch-only set is not marked (**R-A**). |
| **P3** | the passphrase plate, run preloaded (**R-C**). |
| **P4** | the restore document, stating what was engraved rather than denying passphrases exist. |
| **P5/P5b** | scroll arrows floating over the body's fade zones, per-direction (**R-I**). The SH2's first touchable scroll affordance. |
| **P6** | S6b's own modal-fit sweep. |
| **P7** | the firmware-wide modal sweep — F-192's real scope. |
| **P8/P9** | the whole-diff and failure-states folds. |

---

## WHAT THE REVIEWS COST AND BOUGHT

**Seven phase gates went green. Three adversarial lenses then found nine
things, and no phase gate could see any of them.**

| lens | found |
| --- | --- |
| **truth** — *does it state something false?* | **2 Criticals.** The preloaded passphrase was editable and never re-checked; a passphrase over 100 chars truncated silently. Both would engrave under the **full** passphrase's fingerprints, stamped `DERIVED`. |
| **failure-states** — *is it still true when interrupted?* | **3 Importants.** An aborted passphrase plate left unwarned secret steel while the document denied it existed; new advice named an action the flow forbade; the device blamed good plates while holding the fact that exonerated them. |
| **falsified-elsewhere** — *what did this diff make false somewhere else?* | **1 Important + 2 Minor**, plus a **fifth site the sweep itself missed** and `fold-propagation-check.sh` caught. |

**The third lens did not exist when the cycle started.** It was written after
mutation-checking a one-line test happened to print a failure message that P5
had falsified ten commits earlier. That is the cycle's lesson: the first two
lenses read what the diff **wrote**; nothing read what the diff **broke**, and
a diff falsifies text in files it never opens.

---

## THREE DECISIONS A FUTURE READER WILL TRIP OVER

**1. GATE 5.1b now PINS its gap instead of failing forever.**
`TestGate51bMaxScrollAgreesWithVisibility` used to assert `diverged == 0` and
fail on every run by design, citing spec §7's *"does not gate"*. But the fork's
CI runs `go test` on every push and skips nothing, so a permanently red test
**gates everything** — `main` red forever, next real failure invisible. It now
asserts the divergence is **exactly 22 values spanning `[239,260]`** and passes.
Mutation-proved both ways: it fails if the gap vanishes (fadeClip restored —
delete the gate) and if it drifts (a defect). The old shape could distinguish
neither. **Do not re-pin the constants to whatever a red run prints.**

**2. `plan-cite-check.sh` takes `CITE_FORK_ROOT`.** Its default root is the
fork's checked-out `main`, and it only checks a line is *in range*, never what
is *on* it — so for citations to unmerged branch work it printed `ok` while
resolving against a tree without that work. Set the override when a doc cites a
branch:

    CITE_FORK_ROOT=/scratch/code/shibboleth/wt-<branch> ./scripts/plan-cite-check.sh <doc>

**3. The S6a design docs carry dated SUPERSEDED blocks, not rewrites.** P9's
retry loop falsified four passages asserting `statusVerifiedOnRetry` is
unreachable — two of which read **"PASS"**, and one of which was a *test
placement directive* telling a future implementer to put retry coverage on
multisig. Each keeps its original text with a supersession block beneath.
Rewriting them would have fixed today's fact by falsifying the record of what
S6a's gate actually verified.

---

## STILL OPEN (none of it gates the flash)

- **F-205** and multisig marking → phase `key & password custody refinement`.
- **F-207** — `singleSigReadbackCards` drops a card of an unexpected kind.
- **F-208**'s post-flash items; the honest-geometry work restoring `fadeClip`'s
  real clip mask, which is what makes GATE 5.1b's pinned gap go to zero.
- The **wallet-type-mismatch** trigger of F3's false lead: re-picking a
  different wallet type at verify than the engrave used still reaches
  *"Check the engraved plates."* Recorded in `511f7f3`'s message as
  deliberately out of that fix's scope; needs purpose/script plumbed.
- One trailing **push record** commit, unpushed — the usual tail, since a push
  record can only be written after its own push.
