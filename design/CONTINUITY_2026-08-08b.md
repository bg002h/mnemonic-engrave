# Continuity — 2026-08-08 (b)

Supersedes `CONTINUITY_2026-08-08.md` for the **encrypted payload delivery**
feature. That doc handed off "B2a needs plan + R0 gate". Since then **B2a was
planned, gated through three R0 rounds to GREEN, and split into B2a-i and
B2a-ii**, the follow-up register was reconciled, and **B2a-i implementation is
in flight**.

Its §5 carry-forward and §6 how-to-work still stand except where corrected below.

---

## 1. STATE

```
mnemonic-engrave  master  3daf929   NOT pushed -- 9 commits ahead of origin
seedhammer        main    78949e7   unchanged; B2a-i is on a worktree branch
```

**Nothing has been pushed this cycle.** `mnemonic-engrave` carries nine design
and record commits. Push via `ci/staging` per `CLAUDE.md` when ready.

| | state |
| --- | --- |
| Plan A — host `me seal` / `me hash` | shipped |
| Plan B Phase A — device headless core | shipped |
| Plan B Phase B1 — the unsealed path with UI | shipped, hardware-verified |
| **Plan B Phase B2a-i** — the headless substrate | **GREEN plan; implementation IN FLIGHT** |
| **Plan B Phase B2a-ii** — unlock + the secret session | GREEN plan; not started |
| Plan B Phase B2b — residency wipe + residue | not started |

## 2. THE B2a SPLIT — decided 2026-08-08

The nine tasks split at a **property**, not a task count:

> **Nothing in B2a-i can decrypt.** No key reaches a cipher, no AEAD is opened,
> no secret record is ever resident.

That is the same property that made B1 cheap to review, and it means a reviewer
of B2a-i never reasons about residency while a reviewer of B2a-ii reasons about
nothing else.

| | tasks | contents |
| --- | --- | --- |
| **B2a-i** | 1–3 | F-77 grouping, F-79 retention, the chunked KDF engine |
| **B2a-ii** | 4–9 | words, progress, AEAD open, retry, §10.2.2 session, plate list, mutation rows, hardware |

**Task numbers are unchanged across the split**, so every finding in the R0
reports still resolves against a task number. `Opener.UnlockWithKey` stayed in
B2a-ii deliberately: a phase that could decrypt without §10.2.2's lifecycle would
leave seed material resident with nothing managing it.

**B2a-ii is not separable further**, and **B2b is still owed** — §10.2.4's idle
wipe. Do not tag a release when B2a-ii merges.

## 3. THE PLANS AND THEIR GATES

- `design/IMPLEMENTATION_PLAN_encrypted_payload_deviceB_phaseB2a_i.md`
- `design/IMPLEMENTATION_PLAN_encrypted_payload_deviceB_phaseB2a_ii.md`

| review | verdict | report |
| --- | --- | --- |
| R0 round 0 (opus) | 1C / 4I / 6M / 3N | `agent-reports/…phaseB2a-R0-round0.md` |
| R0 round 1 (opus) | 0C / 2I / 4M / 3N | `…-R0-round1.md` |
| R0 round 2 (sonnet) | 0C / 1I / 0M / 2N — GREEN | `…-R0-round2.md` |
| §3d scoped (opus) | 0C / 3I / 2M / 1N — GREEN | `…-i-section3d-review.md` |

**Every round is persisted verbatim in its own commit with the fold in a
second**, so `git diff <report>..<fold>` shows what changed in response and
nothing else.

**Running B2a-ii's build gate needs a fork with B2a-i merged:**
`scripts/plan-build-gate-go.sh <plan> <fork-with-B2a-i>`. Against the unmodified
fork tier 1 fails, and that is not a defect — its Go calls `seal.NewDeriver` and
reads labels B2a-i creates. Two traps met while proving it: the gate's scratch
dir is **rebuilt every invocation**, and `nix develop` prints
`warning: Git tree … is dirty` on an uncommitted root, which the gate reads as a
build failure because it treats any output as one.

Expected cite-gate failures: `seal.Deriver`, `seal.NewDeriver` (B2a-i) and
additionally `seal.IsSecret` (B2a-ii) — all symbols the plans create.

## 4. WHAT THE R0 LOOP ACTUALLY CAUGHT — read this before authoring anything

**The Critical was an "interpretation" section, and its justification was false
rather than debatable.** The plan held a decrypted seed across a ~21-minute
engrave, justified by "the retry prompt needs the record". It does not:
`newEngraverJob` holds `plate.Spline` (`gui/engraver.go:64`) and the loop
iterates `e.spline` (`:170`) — nothing reads the record after the plate is built.
Worse, the design did not satisfy §10.2.2 anyway, because **Back while running
does not return from `Engrave`** (`gui/gui.go:2651-2656` calls `Stop()` and keeps
rendering), so the seed stayed resident on the abort-mid-plate path the spec
singles out. The compliant design — `clear(rec)` when the plate exists — was
*simpler* than the thing the section justified.

**But do not conclude such sections are a smell.** B1's plan carried one and its
reviewer explicitly cleared it. The difference is that B1's behavioural claim had
been **run against the code**. Flagging a departure for the reviewer is not
verifying it. *(Recorded as memory `departure-sections-need-a-run-check`.)*

**Folds authored most of the rest, three phases running**, and specifically the
part nobody asked for: a residency claim true only for single-secret payloads, a
shipped comment stating the pre-fix price, a misquoted call site. Round 2 then
caught the pattern one level down — round 1's finding named **two** locations and
the fold corrected one, leaving the false sentence in the copy that ships into
the fork as a source comment. **When a finding's WHERE lists more than one
location, the fix has more than one edit; `grep` for the old wording before
calling it folded.** A gate cannot help: a comment is not code.

**Two Importants were tests that could not fail** — a fixture that never reached
the code path it was named for, and a mutation table naming a KDF counter that
the phase had moved out of the path.

## 5. WHAT EXECUTION FOUND THAT COMPILING DID NOT

The build gate passes on `go build` + `go vet`. Running the plan's own Go found
four things the gate could not:

1. **The chunked PBKDF2 reproduces six vector keys byte-for-byte** at step sizes
   1 … 2²⁰ — Task 3 verified, not proposed.
2. **A claim in the plan was false**: "no Phase A test needs editing". Exactly one
   does — `TestEncryptedRecordsCarryNoGrouping`, which exists to pin the F-77 gap.
3. **A mutation check was not a mutation.** `Step(iterations-1)` is an
   *equivalent* implementation, because `NewDeriver` performs iteration 1 before
   any `Step`. Shipping it would have had an implementer report a surviving
   mutant against correct code. Real mutants: `Step(iterations-2)`, `Step(1)` —
   14 failing tests each.
4. **A zero-value `Deriver` returned a valid all-zero AES key** and `Wipe()`
   panicked on its nil `mac`. Found by the §3d review, fixed, mutation-checked.

Measured numbers worth not re-deriving: host `DeriveKey` at 100,000 iterations is
**13.1 ms** (300,000 → 35.5 ms); the chunked deriver covers 100,000 in **10.2 ms
across exactly 200 `Step` calls, drawing 199 frames** (so a gui test must pump
≥256, not the house idiom's 32); the `gui` suite is ~12 s.

## 6. THE ACCEPTED LIMITATION — F-83

**Operator, 2026-08-08: "the one honest gap is unavoidable."**

The plate cannot be wiped until the engrave finishes. It *is* the geometry being
cut and must be resident while the needle moves; a `[]byte` plate pipeline would
**relocate** the secret, not remove it, because the spline still encodes it.

It is therefore an **accepted limitation, not a follow-up** — a register whose
value is that every open item is real cannot carry one that will never be
actioned. What is true, stated once: during a secret engrave the seed is
recoverable from SRAM by an attacker with physical access and an SWD probe
(§2.2 item 9). §10.2.2's wipe removes the **record**, the only copy that outlives
the plate, and that is the whole of what it claims.

**F-85 is what this owes**: SPEC §2.2 does not name during-engrave residency.
Owning phase **before the release tag** — the SPEC is GREEN, so it is an
amendment with its own gate and must not ride an implementation commit.

## 7. FOLLOW-UP REGISTER — reconciled 2026-08-08

23 open items. Moved below `## Resolved`: **F-73, F-74** (closed in their
headings but still in the open list — the record defect the previous continuity
asked to fold in). Retargeted: **F-77 → B2a-i Task 1** (gating), **F-79 → B2a-i
Task 2**, **F-76 → after B2b**. **F-80 amended** — the previous continuity said
"two B2 items"; there are **three**, and the 2026-08-08 decision sends the
"already cut" marks and Back-is-Lock to B2a-ii while the `layoutMainPager` pixel
pin stays out (it needs F-78's rasterising check).

Filed: **F-85** (the §2.2 amendment), **F-84** (`SeedScreen.NoEdit`, implemented
in B2a-ii Task 6 but recorded because it touches a screen the NFC scan path
shares), **F-82** (`seal.Deriver`/`DeriveKey` have no Rust counterpart, and why
the Rust-primary rule does not bind them). **Withdrawn: F-81** — it described a
residency window created by the design R0 round 0 rejected.

## 8. IN FLIGHT — B2a-i implementation

Worktree `/scratch/code/shibboleth/seedhammer-wt-b2ai`, branch
`feat/encrypted-payload-b2a-i`, from `main` @ `78949e7`. One implementer, TDD,
one commit per task, nothing pushed.

**Not yet done, and mandatory before merge:** the independent adversarial
whole-diff execution review. R0 covered plan correctness; that review is what
catches implementation-introduced regressions TDD misses.

## 9. WHAT COMES AFTER

1. B2a-i whole-diff review → merge.
2. B2a-ii: implement (its plan is GREEN), whole-diff review, merge.
3. B2b: §10.2.4's idle wipe. **Carry forward, verified — do not rediscover:**
   there is **no last-physical-input accessor reachable from a flow**.
   `a.idle.start` is a field of an anonymous struct local to `Run`'s closure
   (`gui/gui.go:2884-2891`), `Context` has no such field, and `Event`
   (`gui/event.go:105-109`) carries no timestamp. A flow-local reconstruction is
   **lossy**: `EventRouter.Reset` (`gui/event.go:281-294`) discards every event no
   filter claimed, so a press on an unbound button resets the screensaver's timer
   invisibly to the flow — a flow-local timer therefore drifts *early* and can
   fire while the operator is present. And the screensaver does **not** unwind the
   flow: `gui/gui.go:2954-2959` `continue`s without calling `yield()`, so a flow
   stays blocked inside `ctx.Frame` with its stack, and its secret, live.
   `seal.Payload.SecretsResident()` ships in B2a-ii as the predicate to key on —
   note it goes false only when the **last** secret's plate is built, not the
   first, so a 2-of-3 holds records for ~63 min per §10.2.2's cost paragraph.
4. Then: tag a release (F-85 first), F-65, F-66, and the residue tracks.

## 10. CORRECTED CITATIONS — do not re-derive

Seven inherited citations had drifted and are corrected in both plans' own tables.
The load-bearing ones: `idleTimeout` is **`gui/gui.go:2879`** (not 2801);
`AppendEvents` is **`platform_sh2.go:369`**; `wipeBytes` is **defined** at
`gui/slip39_polish.go:342` (`passphrase_flow.go:605` is a call site);
`NewKeyboard` is **`gui/gui.go:983`**; the `unlockPayload` dispatch is
**`gui/gui.go:1595`**, its title case **`:1791`**; pass 3's `SectionPublic` gate
is **`seal/record.go:214`**; `editBtn.Clicked` is **`gui/gui.go:2331`**.
