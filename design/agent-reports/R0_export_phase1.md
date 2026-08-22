# R0 gate review — PLAN_wallet_file_export.md, Phase 1 only (`--allow` on `export-wallet`)

- **Reviewed:** 2026-08-22, adversarial R0, scope strictly Phase 1 (G1).
- **Verdict: 1 Critical / 4 Important / 2 Minor / 1 Nit — the gate does not pass.**
- Everything below marked *measured* was executed during this review, not recalled.
  New evidence beyond the brief's machine-verified list: a live Bitcoin Core v25.0
  (`bitcoind` on this box) imported/parsed the actual export artifacts; the pinned
  rust-miniscript source (`~/.cargo/git/checkouts/rust-miniscript-ce5fa57e8900265e/ff4732e`)
  was read at the exact gate; both exports were re-run with stdout/stderr separated
  (the controller's list did not state stderr contents, and a finding hinges on them).

---

## C1 (Critical) — Bitcoin Core refuses every descriptor with a keyless spend path; Phase 1's purpose sentence is falsified by measurement, and no Phase-1 gate would notice

**Plan text attacked:** §G1 — *"So G1 is parity, not invention: the smallest change
that unblocks the wallet the operator actually asked about"*; §1 table row 1 —
*"works — 2694 bytes of `importdescriptors` JSON, valid checksum"*; Phase 1 test
list (no importability gate of any kind).

**Measured.** Against Bitcoin Core v25.0 on this box (mainnet chain, networkless,
scratch datadir):

1. `importdescriptors` fed the toolkit's **actual exported wsh JSON, verbatim**
   (the plan's "works" row): **both entries `success: false`**, error:

   > `... is not sane: witnesses without signature exist`

   `getdescriptorinfo` on the same descriptor: identical refusal (error -5). The
   JSON envelope itself is well-formed — Core parsed it and refused each
   descriptor on miniscript sanity. So the "works" row is **export-side only**;
   the import that gives the operator a Core watch-only wallet has never been run
   and fails.
2. The tr receive form (multipath split to `/0/*`): Core v25 —
   `'and_v(v:sha256(...),multi_a(3,...))' is not a valid descriptor function`
   (tap-leaf miniscript arrived in Core v26).

**Why it is wrong.** The tier-4 branch `and_v(v:after(1383520),sha256(H))` has no
signature in its witness. Core's descriptor parser hard-requires miniscript
sanity — including "every spend path needs a signature" — and `importdescriptors`
exposes **no override** analogous to `--allow`. That refusal is Core policy, not
a version accident: it is the same rule for wsh (measured above on the wrapper
that already exports today) and per-leaf for tapscript on v26+ (inference from
Core's shared miniscript sanity path — flagged as inference; the plan offers no
counter-evidence, and the burden is on the plan to name one Core version that
accepts it). Consequence: **for the `bitcoin-core` format, `--allow` converts a
refusal at export time into a refusal at import time.** Phase 1's tests (exit 0,
warning fired) all pass, the phase closes GREEN, and the operator receives a
file Bitcoin Core rejects with error -5 — for the tr form AND, already today,
for the wsh form nobody has ever imported. This is the "can a user do the thing"
class: the gap G1 claims to close is not in the exporter.

Adjacent, same evidence: the emitter's `--bitcoin-core-version` parameter is
accepted and unused (`wallet_export/bitcoin_core.rs:46`, `_bitcoin_core_version`),
so nothing would even version-gate a tr export labeled for the default `25`.

**What would fix it** (not authoritative — the defect is reproduced above, the
remedy is a design choice): Phase 1 must state what the unlocked export is FOR
and gate on it. Options: (a) scope Phase 1's `--allow` to formats whose consumer
accepts the output (`--format descriptor` passthrough is the only one with no
downstream validator; bitcoin-core is demonstrably not one) and say so in the
refusal/warning; (b) hold G1 until `PLAN_export_bitcoin_core.md` lands, since
that PENDING report is exactly where "which Core accepts what" belongs — the
plan currently lets Phase 1 run ahead of the fact that falsifies its framing;
(c) at minimum, add a measured import verdict (a real `importdescriptors` run)
to Phase 1's acceptance, so the phase cannot close on export-side exit codes
alone. Also correct §1 row 1 ("works") to "exports; Core refuses the import —
measured", or the table keeps vouching for an artifact its consumer rejects.

---

## I1 (Important) — "Route the parse at `export_wallet.rs:524`" names one strict parse site of at least three on the target path; the relaxation architecture is unspecified

**Plan text attacked:** Phase 1 — *"Route the `--descriptor` parse at
`export_wallet.rs:524` through the relaxed path when any allowance is
requested"*.

**Why it is wrong.** The intake parse at :524 produces `canonical =
d.to_string()`, which preserves the sigless leaf — and that string is then
**strictly re-parsed** downstream, re-firing the identical `FromStr` gate each
time:

- `cmd/export_wallet.rs:640` — script-type derive, runs on every `--descriptor`
  invocation (`"export-wallet script-type derive: {e}"`);
- `wallet_export/bitcoin_core.rs:48` — the bitcoin-core emitter's own re-parse
  (`"export-wallet re-parse: {e}"`);
- plus per-format re-parses reachable once admission is relaxed:
  `wallet_export/pipeline.rs:175` (bip388 transform), `bsms.rs:105`,
  `green.rs:52`.

Relax :524 alone and the tr run dies at :640 with the same error; relax :640 and
it dies in the emitter. The plan's TDD test ("tr form exports") would eventually
force an implementer through all of them, but the plan specifies a one-line
change where the real decision is architectural: **either** thread the allow
state to every downstream parse (conditional relaxation everywhere), **or** make
intake at :524 the single admission gate and convert downstream re-parses to
lenient re-parses of an already-admitted string. Those are different designs
with different blast radius (a lenient emitter parse is safe only while intake
remains the sole gate — and pipeline.rs:28 also serves the `--template` path).
Two implementers will build different things. The plan must name the sites and
pick the architecture.

---

## I2 (Important) — "Reuse `emit_allow_notes`" cannot be done as written: the fired-set computation it consumes is Segwitv0/wsh-only and does not exist for the tr shape Phase 1 unlocks

**Plan text attacked:** Phase 1 — *"`build-descriptor` already has the
never-silent surface (`emit_allow_notes`): an unmissable stderr warning for
every allowed rule that actually FIRED, plus a note for each requested allowance
that did not. Reuse it; do not reimplement a quieter one."*

**Why it is wrong.** `emit_allow_notes(requested, fired, stderr)` is a printer;
the load-bearing input is `fired: &[DiagnosticKind]`, produced by the
build-descriptor gate pipeline (`validated.allowed_fired`), which runs
`Miniscript::<DescriptorPublicKey, Segwitv0>::from_str_ext` over a rendered
`wsh(M)` (gate.rs:171-180; build-descriptor is a wsh(M)-only surface, its own
doc: *"The concrete `wsh(M)#checksum`"* at build_descriptor.rs:245). A tr
descriptor's leaves are `Miniscript<_, Tap>` — a different type in a different
correctness context; the existing gate machinery cannot run on them. The only
existing tr relaxation precedent, `md parse_template_ext`
(descriptor-mnemonic/crates/md-cli/src/parse/template.rs:2296), computes **no**
fired set — `md encode --experimental` warns unconditionally on the flag
(encode.rs:64-75). So the fired/did-not-fire distinction the plan mandates
requires **new Tap-context per-leaf analysis** (run each leaf's
`ext_check` against the sane baseline, partition failures by rule) that the plan
neither acknowledges nor specs. Left as written, one implementer ships the
md-style unconditional warning (violating the plan's own "note for each
requested allowance that did not [fire]"), another writes the new analysis.
Specify the detection: per-leaf `ext_check(&ExtParams::sane())`-vs-relaxed
partitioning, and where it lives (it is neither `gate.rs` as-is nor
`emit_allow_notes`).

---

## I3 (Important) — Phase 1 builds `--allow` semantics on a tr-only upstream parser quirk, and its own test pins the silent wsh hole as correct

**Plan text attacked:** §G1 — *"The tier-4 keyless spend path trips
rust-miniscript's `sanity_check()` at `cmd/export_wallet.rs:524`"* (framed as
the surface's sanity rule); Phase 1 test — *"the wsh form is unaffected with no
flag"*; Phase 1 — *"The warning is not optional."*

**Why it is wrong.** In the pinned miniscript (rev ff4732e),
`Descriptor::from_str` runs the sanity gate **only for the `Tr` variant**, under
an upstream comment that reads verbatim: *"FIXME preserve weird/broken behavior
from 12.x. See … issues/734"* (src/descriptor/mod.rs:1138-1148). Wsh/Sh/Bare get
no check at all. Measured on the shipped surface: the **same** keyless branch
(`and_v(v:after(1383520),sha256(4743d7…)))` appears in both rcw forms, and

- tr → exit 2, `"All spend paths must require a signature"`;
- wsh → **exit 0, stderr = the watch-only note only (62 bytes), zero warning**.

So the "sanity rule" `--allow` relaxes is not a rule of the export surface; it
is a wrapper-dependent parser quirk that upstream itself calls broken and tracks
for removal. Phase-1 consequences, all inside scope:

1. The flag's advertised meaning — "a keyless spend path cannot be exported
   without explicit consent plus a loud warning" — is true for exactly one
   wrapper. The never-silent story the plan sells is vacuous for the wsh form
   the same command exports today.
2. The test *"the wsh form is unaffected with no flag"* converts that hole from
   an accident into pinned, asserted behavior — the same "incidental → let's at
   least not bless it" trap the plan itself diagnoses for Sparrow in §2, handled
   oppositely here.
3. If the miniscript pin ever advances past upstream's #734 fix, the tr gate
   vanishes from `from_str`: the plan's "refuses without it" test goes red (good
   — keep that test), but the specified mechanism ("route the parse through the
   relaxed path") becomes a wrapper around a check that no longer exists, and
   the surface silently converges on exporting sigless-anything flagless. The
   plan's Open Q4 knows the asymmetry exists ("potentially normative … if
   touched, Rust-first with vectors") but does not connect that **Phase 1
   touches it** — it defines flag semantics on top of it.

**What would fix it:** decide, in the plan, which of the two coherent designs
Phase 1 builds: (a) `--allow` as an export-surface **policy** — an explicit
admission check owned by export-wallet (per-leaf for tr, top-level for wsh/sh),
run uniformly across wrappers, so the flag means the same thing everywhere and
survives pin advances; then the wsh keyless form starts requiring the flag too
(a behavior change to state loudly, Rust-first per Open Q4); or (b) `--allow`
as tr-only parity with the upstream quirk — then the plan must say so, must not
claim a never-silent surface, and must file the wsh hole as a follow-up with an
owning phase instead of pinning it as correct. The current text specifies
neither and reads as (a) while testing (b).

---

## I4 (Important) — Phase 1's test list omits the one test that guards against over-admission, and leaves the flag's reach unspecified

**Plan text attacked:** Phase 1 — *"Tests: tr form exports with `--allow
sigless-branch` and refuses without it; the warning appears; a rule requested
but not fired says so; the wsh form is unaffected with no flag."*

**Why it is wrong / incomplete:**

1. **No granularity test.** The relaxed parse has no `Descriptor`-level API in
   the pinned miniscript (no `Descriptor::from_str_ext`; only bare
   `Miniscript::from_str_ext`); the implementer must parse via
   `expression::Tree` + `FromTree` — which skips **all five** sane rules — and
   then re-apply every non-waived rule per leaf, exactly as md does
   (template.rs:2310-2330). If that re-application is forgotten, `--allow
   sigless-branch` silently also admits malleable / repeated-key /
   mixed-timelock / over-limit tr descriptors the user never waived. Nothing in
   the plan's test list can fail on that mutation. Required test: a tr
   descriptor violating a *non-requested* rule still refuses under `--allow
   sigless-branch`, naming the unwaived rule.
2. **No format named.** "tr form exports" — against which `--format`? The flag
   is command-wide; once admission relaxes, the descriptor reaches
   passthrough-capable emitters with different downstream fates (bitcoin-core:
   Core refuses, see C1; `descriptor`: no validator at all; `bip388`: a
   *transforming* emitter, pipeline.rs:175, whose behavior on a keyless leaf
   has no vector). Name the format(s) under test and pin the transforming ones
   or exclude them.
3. **`--allow` on non-`--descriptor` paths unspecified.** What does `--allow`
   do with `--template`/`--slot` or `--from-import-json`? (Presumably
   did-not-fire notes; possibly a refusal as meaningless.) Two implementers
   will differ; one line settles it.

---

## M1 (Minor) — No decision on whether the override leaves a trace in the artifact

`CliAllow`'s own doc records the build-descriptor decision: *"The emitted
spec/document records NO allowance."* That was deliberate there. The export
artifact is a file handed to a third-party wallet; the plan is silent on
whether it carries any record that a sanity override produced it (Core JSON
cannot; `--format descriptor` cannot; stderr is the only trace, exactly as md's
comment laments: *"the operator's memory and this line are the only trace"*).
Probably the right answer is the same "no record" — but say it, or it will be
re-litigated at implementation time. (Q2's "does something need to travel WITH
the file": for bitcoin-core the question is mooted by C1 — Core refuses the
file. For formats with no downstream validator it remains open.)

## M2 (Minor) — The reused warning text speaks the authoring act, not the exporting act

`emit_allow_notes`' fired text: *"This descriptor failed miniscript's
funds-safety analysis; you have accepted that risk after review."* True but
under-informative at export time, where the operator did not author the
descriptor and the operative fact is downstream: the wallet being watched has a
spend path anyone with the preimage can exercise after the timelock — balances
the watch-only wallet displays are not guaranteed to be the operator's to keep.
md's text names its consequence concretely ("whoever learns its preimage can
spend it alone"); the export warning should name this one. One sentence appended
for the export call-site; keep the shared FIRED/did-not-fire machinery.

## N1 (Nit) — "Reuse" needs a home: the pieces are private

`allow_set()` and `emit_allow_notes()` are private `fn`s in
`cmd/build_descriptor.rs`; `AllowSet::to_ext_params` is private in
`descriptor_builder/gate.rs` (line 61). Trivial visibility/move work, but name
the destination module in the plan so "reuse" does not decay into copy-paste of
the enum and printer.

---

## Direct answers to the brief's six questions

1. **Is `--allow` on export the same act as on build? No — in three ways.**
   (i) Consent object: build waives a rule on something the operator is
   creating; export waives it on a pre-existing wallet, and the warning's job
   shifts from "don't author this" to "understand what watching this means"
   (M2). (ii) On build the rule is a genuine surface invariant (the gate checks
   every artifact); on export it is a tr-only upstream parser quirk the surface
   never enforced for other wrappers (I3) — so "relaxing the sanity rule" is
   partly fiction. (iii) Build's output feeds the constellation's own flow;
   export's output feeds third-party validators that re-judge it — and the
   named one refuses it outright (C1). The plan's parity framing ("G1 is
   parity, not invention") holds for the flag surface, not for the act.
2. **What does Core do? Refuses. Measured.** `importdescriptors` returns
   per-entry `success: false`, *"is not sane: witnesses without signature
   exist"*, on the toolkit's own exported wsh JSON; no Core-side override
   exists. v25 additionally cannot parse the tr form at all. A stderr warning
   at export time is therefore not the question for bitcoin-core — the file is
   dead on arrival (C1). For validator-less formats, see M1.
3. **Is reusing `CliAllow` right? The enum and printer, yes; the implication
   that the machinery exists, no.** One rule vocabulary across the binary is
   correct and drift-tested (`CliAllow::kind()` ↔ `DiagnosticKind`). But the
   fired-detection behind `emit_allow_notes` is Segwitv0/wsh-typed and cannot
   run on tr leaves — the part that matters is new code (I2). Coupling risk is
   low (shared enum + printer, separate detection); the plan should just draw
   that boundary explicitly.
4. **Naming: `--allow` is right.** Locality wins: same binary as
   build-descriptor, same five-value vocabulary, same note surface. `md
   --experimental` is a different binary and a semantically different flag
   (single-axis, top_unsafe only, warning unconditional on the flag) — adopting
   its name here would import a name whose semantics don't match a five-valued
   rule set. The cross-constellation inconsistency is real but is md's to
   reconcile; not a gate item. No finding.
5. **Wrong-export paths found: none on bitcoin-core — refused, not wrong.** The
   emitter is passthrough (canonical string verbatim; multipath split is
   structural via `into_single_descriptors`, no re-render), and Core
   re-validates and fail-closes (measured). The two residual wrong-output
   channels are (a) a granularity lapse admitting rules the user never waived —
   no plan test can catch it (I4.1); and (b) transforming emitters (`bip388`)
   reached post-relaxation with no keyless-leaf vector (I4.2). Both are
   closable with named tests.
6. **Two-implementers divergence points:** where relaxation lives across the
   ≥3 strict parse sites (I1); fired-detection semantics — md-style
   unconditional vs build-style fired/not-fired (I2); uniform-policy vs
   tr-only-quirk admission (I3); format under test, `--allow` on non-descriptor
   paths, transforming emitters (I4).

---

## Appendix — evidence commands (all run 2026-08-22)

- Exports re-run with streams separated:
  `mnemonic export-wallet --descriptor <rcw wsh> --format bitcoin-core` → exit 0,
  stderr 62 bytes (watch-only note only); `<rcw tr>` → exit 2,
  `error: export-wallet --descriptor: All spend paths must require a signature`.
- Core: Bitcoin Satellite v0.2.4 = Core v25.0, mainnet chain, networkless.
  `createwallet r0gate` (blank, no private keys) → `importdescriptors <exported
  wsh JSON verbatim>` → both entries `success: false`, `... is not sane:
  witnesses without signature exist`. `getdescriptorinfo` on the receive-side
  descriptor: error -5, same text. tr receive form (`<0;1>`→`0`, checksum
  stripped): error -5, `'and_v(...)' is not a valid descriptor function`.
- Pinned miniscript read at
  `~/.cargo/git/checkouts/rust-miniscript-ce5fa57e8900265e/ff4732e/src/descriptor/mod.rs:1136-1151`
  (Tr-only sanity gate + `FIXME preserve weird/broken behavior from 12.x`,
  issue #734); no `Descriptor::from_str_ext` exists (relaxed parse is
  `Miniscript::from_str_ext`/`from_str_insane` only, miniscript/mod.rs:847,856).
- Toolkit strict-parse inventory: `cmd/export_wallet.rs:524, 640`;
  `wallet_export/bitcoin_core.rs:48`; `wallet_export/pipeline.rs:28, 175`;
  `wallet_export/bsms.rs:105`; `wallet_export/green.rs:52`.
- Reuse surfaces read: `cmd/build_descriptor.rs:108-210` (`CliAllow`,
  `allow_set`, `emit_allow_notes`), `descriptor_builder/gate.rs:47-63,155-185`
  (AllowSet→ExtParams, Segwitv0 gate), md
  `parse/template.rs:2278-2330` + `cmd/encode.rs:64-75` (the tr relaxation
  precedent and its unconditional warning).
