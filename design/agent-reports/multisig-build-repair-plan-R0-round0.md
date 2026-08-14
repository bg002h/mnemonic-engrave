# R0 round 0 — IMPLEMENTATION_PLAN_multisig_build_repair.md

Reviewer: independent architect (fable), 2026-08-13. Plan reviewed at
`/scratch/code/shibboleth/mnemonic-engrave/design/IMPLEMENTATION_PLAN_multisig_build_repair.md`;
source at `/scratch/code/shibboleth/seedhammer` @ `a10d007`; primaries at
`/scratch/code/shibboleth/{descriptor-mnemonic,mnemonic-key,mnemonic-secret}`.
Settled inputs honoured: spec GREEN 0C/0I not re-reviewed; round-1 traces and
R-3 stand; operator rulings (phase-1 multisig, NFC out, S4 before S5) treated
as fixed. Every version, pin, and CLI surface cited below was read from the
named file this session, not from a report.

## Verdict

**RED — 1 Critical, 3 Important, 5 Minor.** The plan's byte-identity spine is
right for the md1 and structurally absent for the ms1; for the mk1 it mandates
a comparison that no current host tool can perform as written. All four
blocking findings are bounded plan edits — no spec change is required for any
of them.

---

## The byte-identity table — the headline question, answered per artifact

| artifact | which stage proves byte-identity? | against what oracle? | is the oracle the PRIMARY, or a stale snapshot? | gap? |
| --- | --- | --- | --- | --- |
| **md1** (descriptor) | **S2** ("an md1 the host accepts byte for byte"), re-proven by every later stage's §4.5 walk | "**the host**" — never named. Implementable via the primary toolchain: `md encode` (md-codec 0.42.0) or `me` (pins `md-codec = "0.42"`, `crates/me-cli/Cargo.toml:22`), with `ms derive` supplying the account xpub from the typed seed | **UNPINNED.** Nothing in the plan stops the walk script from "comparing" against the fork's own vendored vectors — `seedhammer/md/testdata/README.md` pins them to **md-codec v0.36.0 (c85cd49)** while the primary is **0.42.0**. That is byte-identity of the Go port against a snapshot of itself — F-127's exact shape | **I2** (pin the oracle, print its version in the gate output); **M1** ("accepts" ≠ "is byte-identical to what the host produces") |
| **mk1** (key card, one per held slot) | **S5** ("the §4.5 byte comparison extends to every mk1") | **None exists as written.** The primary mk-codec draws a **fresh random 20-bit `chunk_set_id` from the CSPRNG per encode** (`mk-codec/src/string_layer/pipeline.rs:34-43`) and `mk encode` exposes no override (`mk-cli/src/cmd/encode.rs` — no such arg); the fork derives the id **deterministically from the bytecode** (`seedhammer/mk/encode.go:32-35`). Two honest runs can never be string-identical | Fork's mk wire pin is "**family_token mk-codec 0.2**" (`seedhammer/mk/mk.go:4-5`); primary is **0.4.2**. Changelog says V1–V18 byte-identical across the jump, with V19 (depth-0/no-path — the very shape S5 test 6 exercises) added at 0.4.x | **I1** — the comparison plane must be ruled now: primary-decode acceptance + `canonical_payload_bytes` equality (the API mk-codec 0.4.1 added for exactly this) |
| **ms1** (seed card, full mode) | **NO STAGE.** S5's gate checks **presence**; `TestFullModeEngravesMs1ForEveryMaster` checks **cardinality** ("engraves both ms1s") | None named — yet the oracle is **one command**: `ms encode --hex <entropy>` (ms-cli, `main.rs:73-77`), deterministic on both sides (device: `codex32.EncodeMS1`, fixed "entr"/threshold-0/share-'s' recipe, `msencode.go`; primary: ms-codec 0.7.0, same recipe, `encode.rs`) | Oracle available at the current primary; simply unused | **C1** — a wrong-master ms1 is invisible to every gate in the plan |

**Where does the plan compare something weaker than bytes?** Three places:
(1) **ms1: presence** — a defect, not defensible scoping; see C1. (2) **S2's
gate wording** — "an md1 the host *accepts* byte for byte" is acceptance
(decode succeeds), which is strictly weaker than §4.5's own rule (byte-equality
with what the host *produces* for the same inputs); see M1. (3) **S6** compares
restore-at-an-external-coordinator, which is semantic rather than
byte-comparison — that one is correct and deliberate: it is the independent
oracle byte-identity cannot supply (below).

**"Presence" — defect or defensible scoping? Defect.** The scoping would be
defensible only if ms1 content were carried by some other gate or if the
comparison were expensive. Neither holds: the encoding recipe is deterministic
and identical on both sides, the host command exists today, and the defect
class presence cannot see is precisely the one S5 exists to close (C2 scenario
2 — the unspendable multi-master backup). A gate specified as presence in the
exact stage that touches the master→ms1 mapping is a gate designed to miss its
own stage's failure mode.

---

## What byte-identity buys — and what it cannot

**What it buys.** The safety property of an engraved backup is that an
implementation *other than the one that cut it* can read it back, decades
later, with the constellation's full validation, repair (BCH correction), and
derivation machinery behind it. Byte-identity against the current primary
proves the steel is a citizen of that ecosystem — every corpus vector, BIP
alignment, and repair-path guarantee the Rust side carries transfers to the
plate. It also makes drift *visible*: pinned to the current primary, the S2
gate is itself the machine check that the fork's 0.36-era encode behaviour
still matches 0.42 (the changelogs claim byte-stability across 0.37–0.42 for
existing vectors; that claim is currently **unverified by any machine check**,
and this plan's gates — correctly anchored — would be the first).

**What it cannot buy.** Two implementations can agree on a defect; the
constellation's own record proves the host is not ground truth (F-127, F-128,
F-130, F-140 were all host-side). Byte-identity against a wrong oracle
launders the defect into a "verified" artifact. The plan already carries the
right complement: **S6 restores every build at an external coordinator** — an
implementation outside the constellation entirely — with at least one
divergent/multi-slot/multi-master build mandated. Byte-identity (device ↔
constellation) plus semantic restore (constellation ↔ external coordinator) is
the correct two-oracle structure; either alone is insufficient. Two
consequences the plan should state: (a) a byte divergence found at a gate is
**adjudicated under the Rust-primary rule**, not auto-resolved by making Go
match — if the Rust side is the wrong one, it is fixed there first, with a
test vector (folded into I2's fix); (b) S6's external-coordinator check covers
the descriptor; the multi-master ms1 plates should get their own readback (M3).

---

## Findings

### C1 (Critical) — no gate in the plan can see a wrong-master ms1; the byte comparison must reach the ms1

**Where.** S5 gate ("…and to **ms1 presence**"); S5 test 5 ("engraves both
ms1s, or refuses with a named reason").

**Failure scenario.** The tail refactor is exactly the risky edit: today
`deriveMultisigLeg(m bip39.Mnemonic, …)` takes ONE mnemonic and full mode
encodes `m.Entropy()` (`gui/multisig_derive.go:32,64-71`). The natural
implementation of "ms1 per distinct master" iterates the master set but the
entropy extraction keeps referring to the flow's original single-mnemonic
variable — a captured-variable bug one refactor away. Result: Trace B in full
mode engraves **two** ms1 plates, both carrying master A's entropy, one
labelled as B's. Presence: 2/2, pass. Cardinality test: pass. §4.5 walk:
pass. S6's coordinator restore checks the *descriptor*, not the seed plates:
pass. Ship. Master B's holder loses their electronic copy; the "Full (seed +
keys)" steel cannot reconstitute B; two accessible legs against k=3 — **funds
unspendable, from a green-gated backup**. This is C2 scenario 2, the named
reason S5 exists, passing every gate S5 specifies.

**Why it is the plan's defect.** The spec's "ms1 presence" (§4.1a item 4,
§4.5) is a floor, not a ceiling — byte comparison satisfies presence a
fortiori, so the plan can fix this without touching the GREEN spec. The
operator's criterion for this gate is byte-identity per artifact; the plan
implements it for one artifact of three.

**Fix (bounded).** Two edits to S5: (1) the gate becomes "the §4.5 comparison
extends to every mk1 and to **every ms1, byte for byte**: each engraved ms1
must equal `ms encode --hex <that master's entropy>` from the current primary"
— the command exists, is deterministic on both sides, and the mapping
master→ms1 is exactly what it pins; (2) `TestFullModeEngravesMs1ForEveryMaster`
asserts the mapping by decoding each ms1 and comparing entropy to its declared
master (and is mutation-checked with the captured-variable mutation above:
both ms1s from A must turn the test red).

### I1 (Important) — the mk1 "byte comparison" has no implementable oracle as written; rule the comparison plane now

**Where.** S5 gate; §4.5's "every mk1" extension.

**The mechanism fact.** Primary mk1 string encoding is **non-deterministic by
design**: a fresh 20-bit `chunk_set_id` from `getrandom` per encode
(`mk-codec/src/string_layer/pipeline.rs:34-43`), no CLI override
(`mk-cli/src/cmd/encode.rs`). The fork's encoder derives the id from the
bytecode ("never from randomness", `mk/encode.go:32-35`). Literal
string-equality device↔host therefore fails on every honest run.

**Failure scenario.** The implementer discovers this mid-S5, with the gate
red for a reason that is not a defect. The likely on-the-spot remedies are
both bad: hand-roll an id-masking normalization inside the walk script
(unreviewed custom slicing of a wire format, on the load-bearing comparison of
the whole safety case), or quietly downgrade to "the host decodes it" —
acceptance, the weaker relation M1 flags — with nobody having ruled that
downgrade. Either way the comparison the operator asked for is improvised
under schedule pressure at the exact moment it matters.

**Fix (bounded).** One paragraph in S5 ruling the plane, before any code:
a device mk1 passes iff (a) the **current primary** `mk decode`/`mk inspect`
accepts the chunk strings (string-layer and BCH validity judged by the
primary), and (b) its canonical payload bytes equal those of the expected
card — `KeyCard::canonical_payload_bytes`, the chunk-set-id-independent form
mk-codec 0.4.1 added for precisely this comparison ("incl.
cross-`chunk_set_id` bytecode-determinism" KATs). State explicitly that the
id is excluded *because the primary randomizes it by design*, so the exclusion
is a ruled property of the format, not a test-time convenience. (Optionally
file, not build: a `--chunk-set-id` flag on `mk encode` would restore full
string identity; that is a host-side change owned by its own cycle.)

### I2 (Important) — "the host" is never named or version-pinned; the gate can be satisfied by a stale snapshot

**Where.** S2 gate, S5 gate, §4.5 as inherited; §5's blind-spot list (which
names cite-rot but not oracle-rot).

**The drift is already on disk.** The fork's md parity vectors are pinned to
**md-codec v0.36.0** (`md/testdata/README.md`) against a primary at
**0.42.0**; its mk wire pin is "**mk-codec 0.2**" (`mk/mk.go:5`) against
0.4.2; interior goldens sit at 0.37.0 (`md/template_strip_test.go:12`,
`md/template_id_test.go:78`). The changelogs *claim* byte-stability for
existing vectors across the gap — a claim no machine has checked, and this
project's rule is that such claims get machine-checked, not trusted. F-127 is
the constellation's own record of what a vendored 0.34 against a 0.42 primary
cost.

**Failure scenario.** The walk script's author needs "the host" and reaches
for what is nearest: the vendored `md/testdata` vectors, or whatever `md`
binary is on PATH (possibly a months-old release). Every gate goes green.
What has been proven is that the Go port agrees with a snapshot of the Go
port's own provenance — the operator's criterion ("compare with mnemonic
constellation output") unmet, undetectably, for the whole cycle.

**Fix (bounded).** Three sentences in the plan: (1) the oracle for every
comparison is the **current primary release toolchain** — md-cli/md-codec
0.42.x (or `me`, which pins it), mk-cli/mk-codec 0.4.2, ms-cli/ms-codec 0.7.0
— and the walk script **prints the oracle versions into the gate output** so a
stale oracle is visible in every gate record; comparing against vendored fork
testdata explicitly does not satisfy any gate. (2) The walk script records the
full input tuple (template, n, k, slot order, fp choice, per-slot origins,
seeds) so "same inputs" is reproducible rather than remembered. (3) A byte
divergence found at any gate is adjudicated Rust-first per the standing rule
(if the primary is wrong, it is fixed there first with a test vector; the Go
fix is the convergence port) — and the vendored-vector re-pin (0.36→current,
mk 0.2→0.4 including V19) is filed as a follow-up owned by the stage that
finds drift, or by S6 if none does.

### I3 (Important) — the seed↔key gate is never re-proven after S5 rewires the origin plumbing it derives against

**Where.** S4 tests (all provable only synthetically before S5) vs S5's test
list (no gate test).

**The structural fact.** S2 adds an interim refusal of any card whose
declared origin differs from the shared origin (S2 test 3), and S5 removes it
("Remove S2's interim foreign-origin refusal"). So during S4 — the stage that
builds and proves the gate — a divergent-origin input **cannot reach the gate
through the real flow**; every S4 gate test is necessarily a synthetic
slot-set test, and nothing in the plan says its fixtures must include a
non-shared declared origin. S5 then changes exactly what the gate derives
against (`cosignerFromCard` stops discarding origins; the slot model gains
divergence) and its six named tests contain no gate case.

**Failure scenario.** S4's gate implementation, tested only against
shared-origin fixtures, derives at the flow's origin rather than the card's
declared origin — the precise binding the settled M-B ruling exists to
prevent. All S4 tests pass (their fixtures make the two origins coincide). S5
lands, origins diverge, no S5 test touches the gate, both stages close green.
Now a `both` slot holding the operator's genuinely-theirs card at
`m/48'/0'/1'/2'` FAILS LOUDLY on an honest input (fail-closed but
feature-dead for the flagship multi-account shape) — or, with the inverse
bug, the gate silently stops firing for divergent slots and the operator's
requirement is unmet with green gates: round 0's I1 resurfacing one stage
later.

**Fix (bounded).** Two test-list edits: (1) S4 adds
`TestGateDerivesAtDeclaredOriginNotFlowOrigin` — a `both` slot whose card
declares a non-shared origin, key genuinely derived there: must PROCEED (and
the same fixture with a key derived at the shared origin must FAIL, naming
the slot) — mutation-checked like the other rows; (2) S5's list adds a gate
regression row — re-run the S4 gate tests through the assembled post-rewire
flow (or a named `TestGateSurvivesOriginRewire`). One sentence in S4 noting
its gate tests are synthetic until S5 by construction, so the reader knows
the flow-level proof lands one stage later by design.

### Minors (recorded; none gates)

- **M1 — S2's gate says "accepts byte for byte".** Acceptance (host decode
  succeeds) is weaker than §4.5's production-comparison (host *builds* the
  same md1 from the same inputs; bytes equal). The plan means the latter —
  say it, in §4.5's own words. One sentence.
- **M2 — S5 test 5 asserts a disjunction** ("engraves both ms1s, or refuses
  with a named reason"). A test that passes on either arm cannot fail
  meaningfully and cannot be mutation-checked. The spec ruled the disjunction
  (either arm is compliant); the *plan* must pick the arm before tests are
  written — its own Trace B text already leans "both ms1s". Pick it, and let
  the test assert that arm only. (C1's fix subsumes the content assertion.)
- **M3 — S6 never reads back an ms1.** "All three restore correctly at an
  external coordinator" exercises the descriptor. The multi-master build's
  ms1 plates — the artifact class C1 shows is least-gated — should get one
  readback (restore master B's mnemonic from its plate) in the same flash
  cycle. One line in S6 item 3.
- **M4 — the S4-before-S5 conditional is settled.** The operator ruled it
  ("Agreed. Safety first."). Replace "If the operator rules otherwise, swap"
  with the recorded ruling, so a future reader doesn't reopen a closed
  question.
- **M5 — depth-0 mk1 sits on the pin seam.** S5 test 6's depth-0 card is the
  V19 shape the primary added in the 0.4.x line ("completing v0.4.0's missed
  roll") while the fork pins 0.2-era wire. When I2's re-pin happens, include
  V19 so the refusal test's premise (the fork can decode the card far enough
  to see `Path == "m"`) is anchored to the same vintage as the primary.

---

## The four standard concerns, answered

1. **Stage independence and gating.** Sound, with I3's caveat. S1's either/or
   gate correctly resolves the old I2 paradox; S2 owns D-1's fix and the
   named-if-absent branch; S3 is freestanding; S4 closes on synthetic proof
   only (acceptable once stated — I3); S5's assembly+tail unity correctly
   enforces the C2 constraint; S6 confirms, never discovers. No stage depends
   on a later one; no stage closes green with a defect it owns still open —
   except S5 under C1, which is the Critical.
2. **Test sufficiency / can each test fail.** S1–S4's mutation mandates are
   real and well-aimed (S1 test 2, S4's every-failing-row rule, S2's
   calibrated raster floor). S5 is the weak stage: test 5 is a disjunction
   (M2) with no mutation mandate, and the gate has no S5 coverage at all
   (I3). With C1/I3/M2 folded, S5's suite proves what the stage claims.
3. **Can a user do the thing.** Preserved. Trace A is deliverable at S2 close
   (S2's gate is Trace A end-to-end, matching round 1's P1-close placement);
   Trace B at S5 close (S5's gate is Trace B, and S6 item 3 rehearses that
   exact shape on hardware). The stage table's map keeps a missing stage
   visible, as intended.
4. **Rust-primary rule.** Not crossed: every change is `gui/`-side or a sysw
   accessor; no wire format, identity/stub algorithm, validation, or
   admission changes. Two adjacencies worth the sentences I2 adds: drift
   adjudication direction (Rust-first), and the pre-existing fork/primary
   divergence on mk1 chunk-set-id derivation (deterministic vs random — a
   canonical-form choice within the format, not a wire violation, but it is
   the reason I1's comparison plane must be ruled explicitly).

## Gate disposition

**RED 1C/3I.** All four blocking findings are plan-text edits — roughly a
dozen sentences and four test-list rows; none requires touching the GREEN
spec, and none reopens an operator ruling. Re-review after the fold should
scope to: C1's ms1 byte-gate wording, I1's ruled comparison plane, I2's oracle
pin, and I3's two test rows — the stage structure, traces, and the md1 spine
are settled and need no re-derivation.
