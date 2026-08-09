# Continuity — 2026-08-08 (b)

Supersedes `CONTINUITY_2026-08-08.md` for the **encrypted payload delivery**
feature. That doc handed off "B2a needs plan + R0 gate". Since then **B2a was
planned, gated through three R0 rounds to GREEN, and split into B2a-i and
B2a-ii**, the follow-up register was reconciled, and **B2a-i was implemented,
reviewed and merged**. B2a-ii implementation is in flight.

Its §5 carry-forward and §6 how-to-work still stand except where corrected below.

---

## 1. STATE

```
mnemonic-engrave  master  a320ba9+  NOT pushed -- 12+ commits ahead of origin
seedhammer        main    a01b666   NOT pushed -- B2a-i AND B2a-ii MERGED
```

**Nothing has been pushed this cycle, in either repo.** Push `master` via
`ci/staging` per `CLAUDE.md`, or the required check is bypassed rather than
satisfied.

| | state |
| --- | --- |
| Plan A — host `me seal` / `me hash` | shipped |
| Plan B Phase A — device headless core | shipped |
| Plan B Phase B1 — the unsealed path with UI | shipped, hardware-verified |
| **Plan B Phase B2a-i** — the headless substrate | **MERGED to `main` at `421dca8`** (unpushed) |
| **Plan B Phase B2a-ii** — unlock + the secret session | **MERGED at `a01b666`** (unpushed) |
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
   **25 failing tests each**, measured unfiltered on the real branch. *(The plan
   first said 14, which was a `-run`-filtered count stated as if it were the
   whole suite. A number that does not reproduce is worse than no number — say
   which command produced it.)*
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

## 8. B2a-i — DONE, and what its review found

Merged at `421dca8` (4 commits, 16 files, +848/−85). Post-merge `main` is green:
the sanctioned two setup failures, `seal` and `gui` passing, TinyGo
**1285664 flash / 60544 ram** — unchanged, because nothing calls the new code
until B2a-ii wires it.

**Whole-diff review: 0C / 1I / 2M / 2N**, persisted at
`agent-reports/…-i-whole-diff-round0.md`, folded in `7b8b8f0`. Plan fidelity was
confirmed **mechanically** — every whole-file Go block byte-identical to the plan.

Three things from it worth carrying:

- **The Important was a one-directional test.** `Probe()`'s false branch had no
  coverage: the mutant `return true` left BOTH packages green, unpinning §10.1's
  normative "absent → invisible". `return false` *was* killed, which is exactly
  why it read as covered. **A one-directional kill is not a kill.** Fixed by
  extending `read_test.go`'s absent-payload table so `Probe` and `Read` can never
  disagree about "present", and re-mutating to prove it now fails.
- **One finding was DECLINED with evidence**, and the discipline matters: it
  claimed the §3d fold deleted `DeriveKey`'s fail-closed `nil`. Measured against
  the old body — `crypto/pbkdf2.Key` returns `err=<nil>` at iterations −1 and 0,
  so the old body returned a key too. No regression; the proposed "restoration"
  would have been a new divergence from the stdlib the differential pins.
- **`-size full` cannot measure F-79.** The 65,536 bytes were a heap allocation,
  not static RAM. What measures the fix is `reads == 0` at startup.

## 8a. IN FLIGHT — B2a-ii implementation

Worktree `/scratch/code/shibboleth/seedhammer-wt-b2aii`, branch
`feat/encrypted-payload-b2a-ii`, from `main` @ `421dca8`. One implementer, TDD,
Tasks 4–8. **Task 9 is the hardware pass and is operator-run**, not agent-run; it
closes §7.1's in-situ RP2350B KDF measurement.

**Mandatory before merge:** the independent adversarial whole-diff review. This
is the phase where secrets are resident, so the wipe lifecycle is where that
review's budget belongs.

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

---

## 11. STOPPED AT 1% USAGE — how to resume (2026-08-08)

**Nothing is lost and nothing is half-applied.** Both repos are committed and
clean; the only thing abandoned was an in-flight review's *findings*.

```
mnemonic-engrave  master  8ef7a6d   NOT pushed
seedhammer        main    421dca8   NOT pushed -- B2a-i merged
seedhammer        feat/encrypted-payload-b2a-ii  3db3bfe  -- B2a-ii, UNREVIEWED
                  worktree /scratch/code/shibboleth/seedhammer-wt-b2aii, clean
```

### The one thing outstanding

**B2a-ii is implemented and verified green, but NOT reviewed, and MUST NOT be
merged until it is.** This is the phase where decrypted seed material is
resident, with no §10.2.4 backstop. Verified locally: exactly two sanctioned
`[setup failed]`, `seal` 12.6s / `gui` 16.1s green, vet and gofmt clean, TinyGo
1307232 flash / 60584 ram (+21568 over baseline — the first phase whose code is
actually reachable).

### Resume the review with

```
Workflow({scriptPath: "/home/bcg/.claude/projects/-scratch-code-shibboleth-mnemonic-engrave/e91c4717-45e0-4990-855e-052bfd823fc8/workflows/scripts/b2a-ii-whole-diff-review-wf_ae4a9fd9-bea.js",
          resumeFromRunId: "wf_ae4a9fd9-bea"})
```

Completed lenses replay from cache; only unfinished ones re-run. **It was stopped
deliberately at 1% usage, not by a failure.** If budget is tight, a single-lens
review satisfies the mandatory-review rule — the fan-out was a quality choice,
not a requirement.

### Then, in order

1. Persist the review verbatim in its own commit; fold in a second.
2. Merge B2a-ii to `main` (`--no-ff`, as B2a-i was).
3. **Task 9 — the hardware pass, operator-run.** Closes §7.1's in-situ RP2350B
   KDF rate, still owed before release.
4. B2b — §10.2.4's idle wipe. §9 above carries its verified findings forward.
5. Push BOTH repos via `ci/staging` per `CLAUDE.md`.

### Carried from B2a-ii's implementation, unreviewed but measured

- 29 of 30 mutants killed. The survivor is the one the plan predicts
  (`clear(blob)`/`blob = nil` is not test-observable) and instructs recording
  rather than papering over.
- Mutation testing found a **one-directional test**: `SecretsResident` ignoring
  `IsSecret` survived a version that only wiped the cards. Code right, test wrong
  — the same class as B2a-i's `Probe`.
- `%` renders as zero pixels in `Styles.progress` — filed as **F-86**, owned by
  F-78's font cycle.
- Four declared deviations, none reviewed: commit sequencing of §5d's fragment;
  `click` rather than `press` in §6c (`Clickable.Next` needs press-then-release);
  §6d's plate-list row moved to Task 7; the mutation runner left uncommitted.


---

## 12. B2a-ii — MERGED 2026-08-09, and what its review cost and bought

22 commits, 26 files, +5992/−140, merged at `a01b666`. Post-merge `main` green:
the sanctioned two setup failures, everything else ok; vet, gofmt, `GOARCH=386`
and `-race` clean; TinyGo **1310184 flash / 60584 ram** (+24520 over B2a-i —
the first phase whose code the image actually reaches).

**The review found two Criticals, and BOTH were in the plan as well as the code.**
R0 had cleared that plan over three rounds.

1. **A full copy of the seed lived through the whole cut.** `bip39.Parse`'s
   `[]Word` was zeroed only on a `defer`, i.e. after `Engrave`. Neither
   `p.Wipe()` nor `SecretsResident()` could reach a local, so §10.2.4's
   predicate would have read *false* while the seed was live.
2. **The KDF progress loop asked for its frame one frame too late.** `Run` reads
   `ctx.Wakeup` *before* `Reset`, so a `WakeupAt` after `ctx.Frame` governs the
   NEXT frame — and frame 1 inherited a three-minute idle deadline. The
   derivation parked at 500/300,000 behind the screensaver: worse than the
   blocking `pbkdf2.Key` it replaced, and it would have logged ~1,400 it/s into
   §7.1's closing measurement instead of ~9,715. **An earlier reviewer had
   explicitly cleared this ordering** — it reasoned about frames 2..N and missed
   frame 1. Two reviewers disagreed; the source settled it.

**The dominant class was not wrong code — it was tests that could not fail.**
58 mutants applied, **20 survived**, including six wipe deletions (among them
`defer clear(plaintext)`, the decrypted record container) and a cancelled unlock
falling through to the plate list. All now pinned and mutation-checked.

**What the completeness critic added that six lenses did not.** Asking *"what did
nobody look at"* rather than *"is this correct"*: three changed files had **zero**
mentions across ten prior reports, and two held Importants. It also measured and
**retracted two of its own hypotheses**, and ran four modalities nobody had.

**Method notes worth keeping.** Every lens wrote its own report to disk before
returning — the first fan-out died at 1% usage and took five lenses' work with
it. Every gating finding got a refute-by-default pass. Two findings were
**declined after measurement** showed their premise false, and one recorded
remedy (F-92's) was **corrected after the experiment showed it does not work**.

## 13. WHAT IS OWED NEXT

1. **Task 9 — the hardware pass. Operator-run, outstanding.** Closes §7.1's
   in-situ RP2350B rate. Note the log line now reports derivation time separately
   from wall time, which the parking defect would otherwise have corrupted.
2. **B2b** — §10.2.4's residency wipe, and it inherits three constraints filed
   during this review, all of which bite before a line of it is written:
   **F-89** (the timer must UNWIND flows, not just call `p.Wipe()`, *and*
   `SecretsResident()`'s contract is wrong on the `ms1` arm — the funds-relevant
   half), **F-93** (the saver still parks a spec-legal derivation above
   1,748,700 iterations, 13.2% of the legal range), **F-90** (the `ms1` arm is
   the under-examined one and it is the DEFAULT arm — six of seven vectors).
3. **Before the tag:** F-85 (§2.2 must name the during-engrave residency) and
   F-92 (`tinygo test` — see its corrected entry; it is a `flake.nix` + `go:embed`
   job, not a build-tag edit).
4. **Push both repos via `ci/staging`.** Neither has been pushed this cycle.
