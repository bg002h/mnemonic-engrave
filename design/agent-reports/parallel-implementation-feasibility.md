# Parallel implementation of the multisig-build-repair plan — feasibility

Scope: one question — can `IMPLEMENTATION_PLAN_multisig_build_repair.md` S0–S6 be
built by multiple coding agents at once? Evidence measured against
`/scratch/code/shibboleth/seedhammer` at `a46a9ce` on 2026-08-14.

## VERDICT

**No — and the reason is structural, not procedural, so no set of rules fixes
it.** Five of the six software stages (S1–S5) all edit the *same 160-line
function*, `buildMultisigPolicyFlow` at `gui/multisig_build.go:39-198`; three
edit the same `assembleBuildPolicy` at `:464-511`. They are not merely
co-located, they are a supersede-chain: S5 **deletes** code S2 writes, S2's first
test **is** S1's discovery promoted to a regression, and the plan states outright
that S4's gate proof is synthetic until S5 rewires it. Every stage's gate is the
*same* single-session, ~4-minute emulator walk driven by one shared script. "Every
time I've tried it before it went poorly" is consistent with this repo's shape,
not with bad luck. Keep the one-implementer rule; it is right for this plan. The
only parallel-safe work is the S0 oracle slice already in flight (`address/`,
`md/testdata/`) and design-doc edits in the other repo.

## WHY — the structure of this plan

**One file carries five stages.** `gui/multisig_build.go` is 606 lines. Measured
touch points:

| stage | what it edits in that file |
| --- | --- |
| S1 | `syswOffer` `:54`, `buildCosignerCards` `:61` and `:254-272` |
| S2 | the D-1 fix, the gather title (D-4), the interim foreign-origin refusal, the interim duplicate-key check |
| S3 | one of the `TYPED-ONLY` comments (`:67`) |
| S4 | `buildPolicyParams` `:338-344`, `multisigSelfSlotChoices` `:324`, the slot-source model, the gate, per-seed passphrase, the `seedID` registry replacing the `defer` scrub at `:75-79` |
| S5 | `SelfSlot int` `:342` → a set, `cosignerFromCard` `:437-458`, `assembleBuildPolicy` `:464-511`, the engrave tail `:162-168` |

Narrowing further: `buildMultisigPolicyFlow` (`:39-198`) is edited by **all five**.
`assembleBuildPolicy` (`:464-511`) is edited by **S2, S4 and S5**.

**Later stages consume and delete earlier stages' code** — causation, not
co-location:

- S2's test 1 *is* S1's D-1 reproduction promoted to a regression test; S1 must
  run first to establish whether D-1 exists on the payload path at all.
- S5 deletes S2's code: *"Remove S2's interim foreign-origin refusal, which this
  stage supersedes."* S4's `TestGateRefusesDuplicateKeyAcrossFinalSlots`
  likewise supersedes S2's interim duplicate check.
- S5's test 8 re-runs S4's `TestGateDerivesAtTheCardsOwnOrigin` fixture through
  the post-rewire flow, because S2's interim refusal makes divergent input
  unreachable during S4 — *"every S4 gate test is necessarily synthetic… if it
  is not re-proven here, S4's proof expires silently."*
- S4 and S5 co-own the scrub registry; S1 hands a half-edited screen to S2
  (*"Title fixed in S2 with the rest of D-4"*).

**The gates contend for unshareable resources.** `cmd/emu/build.sh` writes
`emu.wasm` to a fixed path in the source tree; `walk_trace_a.js` records that the
browser caches it and you must "serve on a fresh port"; a run is ~4 minutes.
Every stage's gate is that walk, and the 13 KB walk script every stage extends is
itself a merge target — the acceptance criterion is a shared mutable file. Beside
it, `cmd/emu/sysw_cards_payload.bin` (978 bytes, digest pinned in Go and asserted
by the walk) is a single binary blob whose inventory comment enumerates which
record each of S1–S5 needs; two agents regenerating it cannot merge. One Go
module (`seedhammer.com`) means one shared baseline: a red package blocks
every stage.

**The collision already happened — serially, in one day.** The plan measured 9
`TYPED-ONLY` occurrences on 2026-08-13 and set S3's gate to
`grep -rn TYPED-ONLY --include='*.go'` returns 0. On 2026-08-14 S0's own
`cmd/emu/embed_confinement_test.go` (commit `3009f22`) added a **10th** — a
comment citing the phrase as the archetype of a stale hand-maintained list.
Measured now: 10 (`gui/multisig.go` ×4, `gui/bip85.go` ×2, `gui/singlesig.go` ×2,
`gui/multisig_build.go` ×1, `cmd/emu/…` ×1). S3's gate is already unsatisfiable
without editing S0's file. One agent, one day, and the stages still collided.

## WHY IT WENT POORLY BEFORE

**The one-implementer rule was bought with a specific failure — 2026-06-20.**
`design/seedhammer-own-code-fix-orchestration-plan.md:3` dispatched Track A and
Track B as two concurrent implementers in separate worktrees ("concurrent where
DEFINITELY safe") to fix 8 own-code findings in the Go fork. **Isolation held —
nothing was clobbered.** What failed is what isolation cannot cover: the fixes
were Go-only, and a retroactive audit (`design/FOLLOWUPS.md:3468`) found **1 of 8
was RUST-ALSO-AFFECTED** — an `ms1` zeroize gap left unpatched in the primary
Rust toolkit while the Go convergence shipped. Two conventions landed the same
day: `193a6b9` "Rust-primary rule" and `86b65b5` "keep implementation tight,
favor 1 agent (user directive)" (both 2026-06-20, `git log`-verified).
**STRUCTURAL** — the defect lived in a cross-repo invariant no agent owned, and
splitting into tracks is what left it unowned.

**Structural — no rule fixes these:**

1. **Invariants nobody owns.** Parallel tracks partition the *work* and silently
   partition *responsibility* for anything spanning tracks.
2. **Shared hot file.** The mild failure is a textual conflict; the severe one is
   two individually-correct edits merging *cleanly* into a wrong flow — no tool
   reports it, and only the emulator walk would.
3. **Single-session gate.** A queued agent is a serial agent with extra
   bookkeeping.
4. **Supersede-chains.** Parallel authorship races the deleter against the author.
5. **One module, one baseline, first assembled at merge.** Even *serial*
   dual-implementation work still drifted: F-120 (`design/FOLLOWUPS.md:2112`) has
   Go's `codex32.New` admitting 48–93/125–127 chars against Rust's 77 cap, under
   an explicit "Rust first, then Go" order.

**Procedural — fixable:**

6. **Poisoned baseline.** `design/FOLLOWUPS.md` ~3686 (2026-08-11): `gofmt -l` was
   non-empty and *"not fixed immediately only because two implementers were still
   holding `gui/` worktrees"* — consequence, *"'gofmt is clean' cannot be used as
   a gate by any future agent."* Parallelism did not create the dirt; it made
   cleanup unschedulable and disabled a gate for everyone after.
7. **Sandbox scoping.** F-97, `design/FOLLOWUPS.md` ~1368: an implementer filed a
   follow-up purely because `mnemonic-engrave/design/` was read-only to it —
   *"A follow-up filed because of a sandbox boundary is not a deferral."*

## THE PARALLEL-SAFE SUBSET

Exactly one slice, and it is already assigned:

- **S0-D6/D7 — published-BIP vectors + `address_test.go` provenance.** Files:
  `address/address.go`, `address/bip_vectors_test.go`, `address/testdata/**`,
  optionally `bip380/bip380_test.go`. Touches no `gui/` and no `cmd/emu` file.
  **Caveat:** `gui/` imports `seedhammer.com/address` (`gui/gui.go:23`,
  `gui/multisig_restore.go:4`), so it is disjoint only while the package's
  *exported API* is unchanged. In flight with the controller — do not reassign.
- **S0-D8 — the md vendored re-pin.** `md/testdata/**` plus its README. Data and
  docs; measured zero byte drift 0.36 → 0.42. Disjoint from D6/D7 and from `gui/`.
- **Design-doc edits** in `mnemonic-engrave/design/` — different repo, no code.

**Merge order:** D6/D7 and D8 are mutually independent (either order), and **both
must land on `main` green before any `gui/` stage opens**, because S2, S3 and S5
gates cite them.

**Not parallel-safe, including the tempting one.** The spec calls P2/S3
*"Smallest stage; independent of the others"* — true of its intent, false of its
files. S3 edits `gui/multisig_build.go` (S1/S2/S4/S5's file),
`gui/multisig_restore.go` (also S5's), plus `gui/bundle.go`, `gui/md1_inspect.go`,
`gui/multisig.go`, `gui/bip85.go`, `gui/singlesig.go` and now
`cmd/emu/embed_confinement_test.go` — the **widest-touching stage in the plan**.
S6 is one machine and one flash cycle: serial by physics.

## GUIDELINES — safe multi-agent code writing

Adopt permanently. Each is yes/no checkable before dispatch.

1. **Compute the file-touch matrix from the TREE before dispatching, not from the
   plan.** `git grep` each task's named symbols. Any file under two agents: do
   not dispatch concurrently.
2. **Disjointness is at FUNCTION granularity, not file.** Record the function
   list, not just paths.
3. **Never parallelize tasks where one's output is another's input, or where a
   later task deletes an earlier one's interim code.** Grep the plan for
   "supersede", "remove … interim", "promoted to a regression".
4. **One named owner per unshareable resource.** The emulator session,
   `cmd/emu/emu.wasm`, `walk_trace_a.js`, any `*_payload.bin`, the hardware.
5. **Never let two agents regenerate the same binary artifact.** Blobs do not
   merge. One agent regenerates for both, before either starts.
6. **Every writing agent gets its own `git worktree` AND a declared file
   allowlist.** Two writers on one branch, never. At merge, `git diff
   --name-only` must be a subset of the allowlist.
7. **Prove the baseline is green and CLEAN before dispatch; name the commands in
   the brief.** Here: `export PATH="/nix/var/nix/profiles/default/bin:$PATH";
   nix develop --command go test ./...`, plus `gofmt -l` and `go vet ./...`
   (6 known `testing.ArtifactDir requires go1.26` findings are the baseline).
   A dirty baseline is a blocking finding, not a caveat.
8. **Fix baseline dirt BEFORE fan-out, never during.** Concurrent worktrees make
   whole-tree cleanups unschedulable and disable the gate for everyone after.
9. **Fix the merge order and name the integrator at dispatch, in writing.**
   Agents never merge each other's work.
10. **The union is a new artifact and re-earns every gate.** Run the full suite
    and the emulator walk on the *merged* tree. Green-in-each-worktree is not
    green.
11. **Before splitting into tracks, write down the invariants that SPAN tracks
    and name ONE owner for each** — cross-repo (Rust-primary ↔ Go-port), wire
    format, scrub discipline, shared helpers. This is the 2026-06-20 failure:
    isolation was correct and the unowned invariant still broke. No owner, no
    split.
12. **Scope every agent to all repos it must write.** Otherwise it files phantom
    follow-ups against its own sandbox boundary (F-97).
13. **A whole-tree `grep` gate is a shared resource.** Scope it to the agent's
    allowlist, or make it the integrator's check.
14. **Fan out read-only work freely — unchanged.** Recon, review and audits have
    no write set and no merge order; the audit that caught the 2026-06-20 defect
    was itself two parallel agents.

## TRIPWIRES — stop and serialize when

- Two candidate agents' file lists intersect at all — even one comment.
- Two candidates edit the same function, struct, or flow, in any file.
- A later task's plan text says it supersedes, removes, or re-proves an earlier
  task's work.
- Any gate needs the emulator, the browser, the wasm build, a `*.bin` blob, or
  the hardware — and more than one agent needs it.
- The acceptance criterion is a whole-tree `grep`, `gofmt -l`, or `go vet ./...`.
- `gofmt -l` or the test suite is not clean at HEAD right now.
- The work is in the CLAUDE.md risk set (funds, seeds, keys, normative codec
  behavior, irreversible actions) and the parallel gain is speed only.
- You cannot name the integrator and the merge order in one sentence.

## WHAT I DID NOT CHECK

- I did not run the emulator walk, build the wasm, or run `go test ./...`. The
  ~4-minute walk duration is `walk_trace_a.js`'s own comment, not a timing I took.
- I did not audit the plan or spec for correctness. Stage content and the S0
  deliverable list are taken as given; I read them for dependency structure only.
- I did not review the in-flight S0-D6 diff for quality — only for which files it
  occupies.
- I did not verify the plan's S1–S5 file lists are *complete*. I confirmed the
  files it names resolve in the tree, so the true overlap can only be larger.
- Historical evidence is limited to what is written down. Undocumented episodes
  are unrepresented; the cited entries are the documented minimum, not the full
  history.
