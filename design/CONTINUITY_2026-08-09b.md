# Continuity — 2026-08-09b

Supersedes `CONTINUITY_2026-08-09.md`, which was written before implementation
began and is now only useful for the R0 history.

## One line

**B2b is implemented (Tasks 1–7 and 9), flashed, and running on the real machine.
Task 8 found a CRITICAL: after §10.2.4's wipe fires, re-entering the sealed
payload HANGS the device. The wipe itself works perfectly. Nothing is pushed.**

**Update, end of 2026-08-09:** Task 9 (F-105, the in-flight passphrase) is
implemented at **`749fce7`**, and a **diagnostic firmware is on the machine** —
`v0.0.0-ge969839` from branch `b2b-heapprobe`, which draws a heap readout on the
start-screen version line. **The next hardware action is three heap readings, not
a re-test of Task 8.**

| repo | HEAD | state |
| --- | --- | --- |
| `mnemonic-engrave` | `33df670` | **85 commits** unpushed |
| `seedhammer-b2b` (worktree, branch `b2b`) | `484ceb9` | **8 commits** ahead of `a01b666`; one untracked test file from the in-flight audit |
| `seedhammer` (main checkout) | `a01b666` | untouched, clean |

Pushing goes via `ci/staging` — see `CLAUDE.md`. **Do not push or tag; the phase
has an open Critical.**

## The Critical — the post-wipe hang

**Recorded in full in `design/HARDWARE_RESULT_2026-08-09_phaseB2b.md`.** Read that
first; this is the summary.

### Symptom

After a wipe, pressing checkmark on Sealed Payload renders the button's **pressed
style** — so the click was processed and a frame drawn — and then **everything
stops**: no redraw, no touch anywhere, **no screensaver at 3:00 or 3:30**, dead
past 4:00. Version line stays correct on screen. Deterministic.

The screensaver, the §10.2.4 timer and touch handling share one goroutine, so all
three stopping is **one fact**: the flow stopped calling `ctx.Frame`.

### Narrowed by experiment — the wipe is required

| sequence | result |
| --- | --- |
| enter → exit Sealed Payload, repeatedly | works |
| full unlock → **normal** exit → re-enter | works |
| full unlock → **wipe** → re-enter | **HANGS** |

This **exonerates the NFC-shutdown deadlock** as primary (that defer runs on every
program entry, many times, without hanging) and shows a single 64 KiB alloc/free
cycle is not sufficient.

### Leading cause

A normal exit **reuses** the `Context`. A wipe **abandons** it and allocates a
fresh one, stranding the old `ctx.B` (grown through the seed screen), `a.warnBuf`
(grown during the 30 s warning) and `a.mask`. Re-entry then needs **64 KiB
contiguous** for `XIPReader.Read`, under `-gc precise`, which is **non-moving**,
on a Cortex-M33 with **no MMU** — so contiguous means physically contiguous and
nothing can defragment.

**This is B2b's own design decision.** The plan says *"a fresh `Context`, not a
scrubbed one … a wipe is rare enough that the allocation is irrelevant."* Round 0
accepted it. It weighed the allocation's cost and never the garbage it strands.

**The interaction worth remembering:** F-79 deliberately drops the blob *early*
(`clear(blob); blob = nil`) so 64 KiB of ciphertext is not live during the
engrave — good residency. That opens the hole **mid-session**, which the following
3½ minutes then fragment. Two individually-correct decisions combining into a
Critical.

### The number that reframes the fix

`seal/wire.go:192` records the format's own arithmetic: the largest admissible
payload is **52 + 8191 + 8191 + 16 = 16,450 bytes**. `XIPReader.Read` allocates
**65,536** unconditionally — ~4× more than anything legal can contain. The
contiguous demand is the flash **region size** used as an **allocation size**.

### Three fixes, written up in `design/DESIGN_b2b_payload_read_allocation.md`

- **A** — static 64 KiB buffer at boot. Removes the demand; reserves 64 KiB
  forever; cuts against F-79.
- **B** — reuse the `Context` across sessions. Smallest; treats the trigger, not
  the class.
- **C — bounded read (RECOMMENDED)** — `hasMagic`, copy the fixed 52-byte header,
  `ParseHeader` to **validate** the lengths, then allocate exactly
  `HeaderLen + pub_len + ct_len + TagLen` ≤ 16,450.

**The constraint an R0 reviewer must attack:** `read_tinygo.go` requires nothing
consult the attacker-controlled lengths before `ParseHeader` validates them.
Reading them early reintroduces the `int(uint32)` **reinterpretation** overflow
that `unlock_key.go:44-58` documents with a measured `GOARCH=386` panic.

### The measurement not yet taken

`runtimeStats.Dump` already calls `runtime.ReadMemStats` and logs
`mem/allocs/total` — gated behind `debug`, to a console this build does not have.
**Nobody has ever measured this device's live heap.** The plan was a throwaway
diagnostic branch drawing heap-in-use **on the start screen** (where the operator
stands before pressing checkmark), compared fresh-boot vs post-wipe.

**Known limitation:** `MemStats` reports total free, **not the largest contiguous
run**. If the cause is fragmentation the readout may show plenty free while the
allocation still fails — and it cannot be probed, because a failed allocation is
`runtimePanic("out of memory")`, not a `nil`. Either outcome still picks a fix:
higher post-wipe usage → B; similar usage → fragmentation → **C**, which stops
caring.

**Deferred by the operator 2026-08-09** in favour of writing this document.

## What passed on hardware, and is real

- **Boot** on machine power; signature accepted against the slot-1 key.
- **F-100 / SPEC §11.5 CLOSED** — the payload survived a firmware reflash. 9 pager
  dots (B1 baseline: 8 absent, 9 present). First time §11.5 has ever been run.
- **8.1 PASSES on every observation** — warning at **3:00**, wipe at **exactly
  3:30**, transition **instantaneous with no blank screen** (twice). That last one
  is the whole design validated: the flow *unwound*, it did not reboot.
- **8.1a** — the unlock takes **40.2 s ± 1.0** at 300,000 iterations, corroborated
  independently by the device's own on-screen estimate of **40 s**.

## Open questions and corrections carried forward

- **§7.1 remains OPEN.** I briefly "falsified" it and changed `me seal`'s default
  to 230,000 — **wrong, and reverted (`7c4a7b4`)**. §7.1's 9,715 it/s is a
  `cmd/kdfbench` **derivation** figure (time inside `d.Step`); the stopwatch
  measures **wall**, which adds ~600 full-panel repaints. `unlock_kdf.go:217-229`
  warns about exactly this conflation and names the 1.54× precedent. **The
  in-situ *derivation* rate still needs the `derived` log line off the machine.**
- **F-93's severity was understated** — that recalculation **stands**, because
  parking is a wall-clock phenomenon. At ~7,463 it/s wall the threshold is
  1,343,284 iterations, so **34.6%** of the legal range would have hung pre-Task-5
  firmware, not the recorded 13.2%.
- **§7.1's "300,000 → 30.9 s"** is derivation time. The operator experiences
  ~40 s. Needs an operator-approved amendment; the CLI and spec do **not** disagree
  (the revert restored 300,000).
- **Task 8.4's §6.6 hash comparison cannot be satisfied** with vector F — it has
  zero public records, so `me seal` printed no hash. Dropped, not fudged.

## Follow-ups filed today

- **F-100 CLOSED** — reflash preservation, closed on hardware.
- **F-99 CLOSED** — §10.2.4's warn@3:00/wipe@3:30 ambiguity, operator-approved.
- **F-101** — `mutation-run.py` is not crash-safe: killed mid-row it leaves a
  **mutant** in the worktree. Hit three times; each left `armed := true`, which
  permanently arms the wipe. Its pre-flight cleanliness check is the only reason
  this surfaced as a refusal rather than a false green.
- **F-102** — `me seal` takes seed material on **argv** while every other
  subcommand reads stdin. Measured: `/proc` has no `hidepid`; `fish_history` is
  world-readable. Binds before a tag, not before Task 8.
- **F-103** — **the protective screen film silently disables the wipe**, the
  screensaver and every idle behaviour, by generating continuous touch events. The
  idle clock keys on **any event, not effective input**. The film **ships on the
  device**. Predicted verbatim in the preflight's accepted-risk list as "a free
  bench check" that was never run — and the first hardware step hit it.

## Two procedure defects of the same class, both found by running things

Both are "a bare command name resolved to the wrong artifact":

1. **`sh2-flash` with no `SH2_REPO`** builds `/scratch/code/shibboleth/seedhammer`
   — the phase's **parent**. Caught by the preflight, fixed in Task 8 before
   flashing, with a pass condition on the `== Build ==` header.
2. **`me` on `PATH` is v0.3.0 and has no `seal` subcommand.** The repo is v0.4.0.
   Found by running it. **Not yet folded into the plan.**

## In flight when this was written

- **A fable wipe-inventory agent** — auditing what §10.2.4's wipe actually zeroes
  vs what survives, across all three arms, and ruling on whether F-88 / F-90 /
  F-94's deferral to B2c is safe. It will write
  `design/agent-reports/…-wipe-inventory.md` itself and has left
  `gui/wipe_inventory_audit_test.go` untracked in the worktree.

## Task 9 — DONE in code, blocked on hardware

`749fce7` on `b2b`. §10.2.4 amended with **rows 4 and 5** (`c7dbfc7`), rows
**appended** so existing row references stay valid.

**Its R0 review found a Critical in my first draft** and is worth reading before
touching it: arming across the KDF is **unsurvivable**, because `Run`'s warning
branch parks the flow for the whole 30 s window, so a derivation reaching 3:00
can never finish. That is **34.6% of §6.2's legal iteration range, permanently
un-openable**. The seam is therefore the **keyboard alone**, closing before the
derivation — row 5 *is* the bracket's scope, not a flag on `armed()`, which is
untouched.

Verified by me: green criterion clean, and the C1 mutation (never uninstall the
bracket) fails with *"ctx.wipe is non-nil on a KDF progress frame … arming here
would freeze the derivation under the warning and make it unopenable"*.

Device budget after Task 9: **1313928 flash / 60584 ram** — RAM still flat across
every task in the phase.

**9.5 (hardware) is BLOCKED** until the post-wipe re-entry Critical is closed.

## The next hardware session — do this first

The diagnostic build `v0.0.0-ge969839` is already flashed. **Three readings of the
start-screen version line:**

1. **fresh boot**, before touching anything
2. **after a full unlock and a NORMAL exit** (the control)
3. **after a wipe**, standing on the carousel where you would press checkmark

Reads `heap <inuse>/<total>K free <free>K a<live allocs>`.

- **post-wipe free is LOW** → plain exhaustion; the wipe is not freeing what we
  assume → fix **B** (reuse the `Context`)
- **post-wipe free is HIGH and it still hangs** → **fragmentation proven by
  elimination** → fix **C** (bounded read, ≤16,450 instead of 65,536)

Passphrase for the payload on the device:
`mosquito neither reopen morning canoe find tiny brand resist satisfy gun ball`

**Then reflash `b2b` before any further Task 8 work** — the diagnostic branch must
never be what gets tested or merged.

## What is owed, in order

1. **The wipe-inventory report** — read and fold.
2. **Pick a fix** (A/B/C) and take it through the **R0 loop**. It is a normative
   change to §6.2-adjacent read logic on a funds path.
3. **Re-flash and re-run Task 8 from 8.1**, then 8.2, 8.3, 8.4 — none of which
   have been attempted, because all need a working post-wipe re-entry.
4. Whole-diff review to 0C/0I, then merge.
5. **The release-tag checklist** at the end of the B2b plan — the one place that
   list lives. Includes B2a-ii's Task 9, F-85, F-92, F-98, F-102 and the
   `ci/staging` push.

## The lesson worth carrying

Four R0 rounds, a four-lens pre-hardware preflight **with a dedicated brick/hang
lens**, and 16/16 mutation kills did not find this. The host reproduction agent
then found **why**: `testPlatform.NextChunk` returns no framebuffer, so `Run`'s
drawer had **zero hit targets** — meaning **no `Run`-level test could ever land a
click**. The session-restart path had never been exercised with real input by
anything. That hole is now closed (`484ceb9`), and it is the most transferable
thing this phase produced.
