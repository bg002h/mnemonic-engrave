# PLAN — wallet-file export for the reasonably complex wallet

**Status:** all five planning reports landed and folded. R0 on Phase 1 has run
twice — round 1 **1C/4I/2M/1N**, round 2 **0C/4I/1M/2N**, round 3 **0C/2I/1M/0N**,
round 4 **0C/2I/1M/0N**. All RED; this document is the fold of all four. Phase 2 is DELETED. **No code until a review
returns 0C/0I.**

**Operator ask (2026-08-22):** output Nunchuk, Sparrow and Bitcoin Core
**watch-only and hot** wallets via the m* utilities, and the same on SeedHammer
II.

---

## 0. The correction this plan is built on

I briefed five agents with a "settled fact" that was **false**:

> *"no export surface exists anywhere in ms/md/mk/me today"*

True as written and wrong as asserted. I checked four CLIs and wrote *anywhere*,
forgetting the constellation's fifth repo. **`mnemonic export-wallet` has
shipped in `mnemonic-toolkit` since v0.97.0** — 11 formats, watch-only by
definition, ~4k lines of emitters. This plan therefore EXTENDS a shipped
surface; it does not build one.

Everything in §1 was measured by running the tools, not read from a report.

---

## 1. The gaps, measured

Against this wallet's own concrete descriptors
(`design/journeys/out/rcw/{tr,wsh}/descriptor.txt`):

| what | result | exit |
| --- | --- | --- |
| `export-wallet --descriptor <wsh> --format bitcoin-core` | **emits, and Core then REFUSES it at import** — 2694 bytes of shape-perfect JSON with a valid checksum that `importdescriptors` rejects per-entry (R0/C1) | 0 |
| `export-wallet --descriptor <tr> --format bitcoin-core` | **refused** — *"All spend paths must require a signature"* | 2 |
| `export-wallet --descriptor <wsh> --format sparrow` | **refused** — *"requires --template; descriptor passthrough is not supported"* | 1 |
| `--format nunchuk` | **does not exist** (11 formats, no nunchuk) | — |
| hot-wallet export, any format | **does not exist** — watch-only by definition | — |

Four gaps, in the order they should be closed:

### G1 — `export-wallet` has no `--allow`, so the tr form cannot export

The tier-4 keyless spend path trips `rust-miniscript`'s `sanity_check()` at
`cmd/export_wallet.rs:524` (`MsDescriptor::from_str`, the strict parse).

**The capability already exists and is not wired here.**
`descriptor_builder/gate.rs` has `AllowSet` with a `sigless_branch` field
mapping to `ExtParams::top_unsafe`, and `build-descriptor` exposes all five
rules as `--allow <RULE>` including `sigless-branch`. `md encode` solves the
same problem with `--experimental`.

So G1 is *parity*, not invention. **But see §1a: it does not unblock import
into any of the three targets** — they enforce the same rule. It unblocks our
own emission, which is worth having and is not the same thing.

> I initially recorded `sigless-branch` as "deliberately excluded" from
> `--allow`. That was false and came from my own `grep -A6` truncating a
> five-item list. It is exposed. Same class of error as reading an exit code
> through a pipe, which I also did in this cycle.

### G2 — ~~no Nunchuk format~~ **CLOSED: do not add one**

`PLAN_export_nunchuk.md` settles it. `--format descriptor` and `--format bsms`
already emit exactly Nunchuk's two import shapes, verified by execution
including a byte-exact 4-line BSMS record. **No `nunchuk` emitter should be
built.**

Nunchuk still refuses this wallet — same keyless tier 4, via libnunchuk's
`IsSane()`/`NeedsSignature()` — plus a second tr-specific blocker: it cannot
represent a fixed raw NUMS internal key and re-renders a per-index unspendable
xpub, changing every address.

**Phase 2 is deleted.** The gap was in the wallet, not the constellation.

### G3 — no hot-wallet export anywhere

`export-wallet` is watch-only *by definition* — `validate_watch_only` rejects
phrase/entropy/xprv/wif at slot resolution.

The CLI-surface report's ruling, which this plan adopts: **hot export never
lives in `md`, and never as a flag on the watch-only surface.** If built at all
it is a distinct subcommand (`mnemonic export-signer`) with secret slots,
`--output` required, `0600` + `create_new`, an always-on advisory, and no
interactive confirmation.

**This is the largest and least-safe item. It is deliberately last.**

### G4 — SeedHammer II has no wallet-file output

The one-output claim is true at the platform interface (`gui/gui.go:3385-3415`)
but the useful finding is that the transport is **~90% built**: the Type 4 tag
emulator already implements `READ_BINARY` and transmits over RF, serving a
hardcoded 2-byte `emptyFile` with 8 KB advertised capacity
(`nfc/type4/type4.go:103-106, 241-242`). An NFC share is ~200-400 LOC with no
new dependency, ranked above a paged-display QR route.

---

## 1a. THE FINDING THAT REFRAMES ALL OF THIS

**Every one of the three named wallet apps refuses this wallet, and all three
refuse it for the same reason: tier 4 has no key.**

Not miniscript complexity. Not the hashlocks. Not the mixed timelock flavours.
Not `multi_a`, not `or_i`, not the NUMS internal key. The single sigless spend
path.

Measured directly against **Bitcoin Core 25.0** (`getdescriptorinfo`, a running
node, positive controls first):

| probe | result |
| --- | --- |
| `wsh(and_v(v:sha256(H1),pk(K)))` — hashlock AND a key | **accepts** |
| `wsh(and_v(v:sha256(H1),multi(3,…)))` — our tier 1 | **accepts** |
| `wsh(and_v(v:after(1383520),sha256(H3)))` — tier 4 as-is | **refuses** |
| `wsh(and_v(v:after(…),and_v(v:sha256(H3),pk(K))))` — tier 4 **with a key** | **accepts** |
| `wsh(or_i(keyed, KEYLESS))` | **refuses** |
| `wsh(or_i(keyed, keyed))` | **accepts** |

This independently reproduces the minimal pair in `PLAN_export_nunchuk.md`,
which found the same thing through libnunchuk's `IsSane()`/`NeedsSignature()`.
Sparrow's refusal has a different mechanism (no miniscript engine at all) but
the same practical effect.

**Consequence for this plan: G1 does not achieve the operator's goal.** Adding
`--allow` to `export-wallet` lets US emit a file. It does not make any target
import it, because the targets run the same rule we would be relaxing. G1 is
still worth doing — parity, and it unblocks `--format descriptor`/`bsms` output
for inspection — but it must not be described as "enables export to Core".

**The change that WOULD achieve the goal is a wallet-design change: give tier 4
a key.** That is the operator's call, not mine, and it is a different wallet.

### 1b. Taproot has a SECOND, independent blocker

The tr form refuses **even with tier 4 keyed**. Isolated on the same node:

| tapleaf contents | Core 25 |
| --- | --- |
| `pk(K)` | accepts |
| `multi_a(3,…)` | accepts |
| `and_v(v:pk(K),older(144))` — any miniscript | **refuses** |
| `and_v(v:sha256(H),pk(K))` | **refuses** |

**Core 25 supports miniscript in `wsh()` but not in tapleaves** — a leaf may
hold `pk` or `multi_a` and nothing else. So the tr form is unreachable for Core
25 regardless of what we do to tier 4.

**SUPERSEDED — and my claim was version-local.** `PLAN_export_bitcoin_core.md`
landed with a six-binary matrix (v25, 26, 27, 28, 29, 31.1) and pins the actual
floors:

| capability | first Core version |
| --- | --- |
| `wsh` miniscript, watch | v24 |
| `wsh` miniscript, sign | v25 |
| **`tr` miniscript** | **v26** |
| **multipath `<0;1>`** | **v29+** — absent even from v28; PR #22838, milestone 29.0, undocumented in its release notes |

So my "Core cannot do tapleaf miniscript" was true of **v25 only** and would not
have held at v26. One version is not a floor.

### 1d. It does not matter, and that is the real answer

**No Core version through v31.1 can load this wallet** — either wrapper, watch
or hot. The keyless tier 4 trips *"witnesses without signature exist"*, and in
Core that rule is **non-waivable**. Unlike our side, there is no flag.

The one thing that DOES work: an **`addr()`-list export**, verified. You can
watch this wallet in Core; you cannot describe it to Core.

Two traps worth carrying forward, both verified:

- `getdescriptorinfo` **silently collapses multipath**, so round-tripping a
  `<0;1>` descriptor through it loses the change descriptor — still true on
  v31.1. Anyone "normalising" that way loses half the wallet with no error.
- A hot export must use **account-level xprvs**; master xprvs with hardened
  paths trip a Core duplicate-key false positive (`PubkeyProvider::operator<`,
  reproduced on v29 and v31.1).

### 1c. How nearly I got this wrong

Worth recording, because the method matters more than the result:

- My first run tested a **mainnet** descriptor against a **regtest** node. Core
  said `"is not valid"` for every key and I was one step from reporting "Core
  rejects our export". The tell was that my **control also failed** — a
  known-good xpub straight from `ms derive` was rejected identically. A control
  that fails with the subject means the harness is broken, not the subject.
- Before that I read `[exit 0]` on a refusal because I took `$?` through a
  `head` pipe (real codes: 0/2/1), and recorded `sigless-branch` as "excluded
  from `--allow`" because `grep -A6` truncated a five-item list at four.

Three wrong readings in one afternoon, all from a filter between me and the
output. See `negatives-inherit-the-search-scope`.

---

## 2. What is NOT a gap

**Sparrow.** `PLAN_export_sparrow.md` finds Sparrow has no miniscript engine at
all (its `Miniscript` class is a 59-line regex shim; miniscript is open feature
request #1700, absent through v2.5.3). The tr form is rejected loudly. **The wsh
form is silently imported as a `sortedmulti` 3-of-6 P2WSH with wrong
addresses.**

Our emitter refuses descriptor passthrough, so the constellation **cannot**
produce that file. That is the right outcome — but note it is **incidental**:
the emitter refuses because it only accepts templates, not because anyone
reasoned about Sparrow's miniscript gap. A future change that adds descriptor
passthrough to the Sparrow emitter would silently create a funds-safety defect.

**Action: none in code; one regression test pinning the refusal, so the
incidental safety becomes deliberate.**

---

## 3. Phases

Each phase ends with a fable review before the next begins, per the operator's
instruction. **No phase may start: the Phase 1 gate is RED** (rounds 1-4:
1C/4I, 0C/4I, 0C/2I, 0C/2I).

### Phase 1 — G1, `--allow` parity on `export-wallet`

**R0 round 1: 1C / 4I / 2M / 1N — gate RED. This section is the fold.**
(`design/agent-reports/R0_export_phase1.md`)

**This changes ADMISSION**, risk-set item (c), so it takes the R0 gate: 0C/0I
before code.

#### What the review changed

**C1 — the purpose sentence was falsified by measurement.** §1's table says
wsh → bitcoin-core *"works"*. That is **export-side only**. Measured on a live
Core v25: `importdescriptors` returns per-entry `success: false`,
*"is not sane: witnesses without signature exist"*, on the toolkit's **own**
exported JSON. So for `bitcoin-core`, `--allow` converts an export-time refusal
into an **import-time** refusal — and no Phase-1 test as listed would ever
notice, because every test stopped at emission.

*Fold:* §1's row is re-worded to **"emits; Core refuses at import"**, and Phase 1
gains an explicit non-goal — see the constraint below. The acceptance criterion
for anything claiming Core compatibility is an **import**, not an emit.

**I3 — "relaxing the sanity rule" is partly fiction on this surface.** On
`build-descriptor` the rule is a genuine surface invariant checked on every
artifact. On `export-wallet` it is a **tr-only upstream parser quirk** that the
surface never enforced for other wrappers. Worse: the test I listed
(*"the wsh form is unaffected with no flag"*) would have **pinned that silent
wsh hole as correct**.

*Fold:* Phase 1 must first **rule** whether `--allow` here means *uniform
policy* (enforce on every wrapper, then waive) or *quirk passthrough* (waive
only where the parser happens to object). These are different products. The plan
picks **uniform policy**, because a flag whose reach depends on an upstream
parser's shape is not a flag anyone can reason about — and that decision makes
the wsh hole a **defect to close**, not a behaviour to pin.

**I1 — one parse site named, at least three exist.** `export_wallet.rs:524` is
not the only strict parse on the target path.

*Fold:* the implementer **enumerates every strict parse site on the
`--descriptor` path first and lists them in the PR**, then states where the
relaxation lives (one chokepoint vs per-site). Divergence here is the single
largest "two implementers build different things" risk the review found.

**I2 — `emit_allow_notes` cannot be reused as written.** The enum and printer
are reusable and drift-tested (`CliAllow::kind()` ↔ `DiagnosticKind`). The
**fired-detection** behind it is Segwitv0/wsh-typed and **cannot run on tr
leaves** — exactly the shape Phase 1 unlocks.

*Fold:* the plan no longer says "reuse `emit_allow_notes`". It says: reuse the
**vocabulary and the printer**; the **fired-detection for tr leaves is new
code** and is Phase 1's real work. N1 applies — both pieces are currently
private to `cmd/build_descriptor.rs` and need a shared home.

**I4 — the missing test is the one that guards over-admission.** My list tested
that the flag *permits*; none tested that it permits **only what was asked**. A
granularity lapse admitting rules the user never waived is invisible to every
test I named.

*Fold:* added — (a) requesting one rule must not admit any other, asserted
per-rule; (b) a keyless-leaf vector through the **transforming** emitters
(`bip388`), which are reached post-relaxation and had no such vector.

**M1 — does the override leave a trace in the artifact?** Undecided in round 1.
*Fold:* **no trace in the artifact.** The emitters are passthrough by design and
a comment field would be format-specific and silently dropped by most targets.
The trace lives in stderr and in the operator's records. Stated so it is a
decision, not an omission.

**M2 — the warning speaks the wrong act.** Build's wording is *"don't author
this"*; export needs *"understand what watching this means"*.
*Fold:* export gets its own wording. Same vocabulary, different sentence.

**N1 — "reuse" needs a home.** `CliAllow`, `allow_set()` and
`emit_allow_notes` are all private to `cmd/build_descriptor.rs`.
*Fold:* Phase 1 lifts the **vocabulary and printer** into a shared module before
wiring the second caller — not after, because "make it pub" applied twice is how
two commands end up with two rule vocabularies that drift. The new tr-leaf
fired-detection (I2) lands beside them.

**Naming — settled, no finding.** `--allow` is right: same binary as
`build-descriptor`, same five-value vocabulary. `md encode --experimental` is a
different binary and a single-axis flag; importing its name would import
semantics that do not match a five-valued rule set. The cross-constellation
inconsistency is real and is **md's to reconcile**, not a gate item here.

#### Round 2 fold — the breaking change I introduced and did not state

**R0 round 2: 0C / 4I / 1M / 2N.** (`R0_export_phase1_round2.md`)

**F-1 — Phase 1 IS A BREAKING CHANGE, and round 1 told me so in the sentence I
dropped.** Round 1's option (a) carried a rider verbatim: *"then the wsh keyless
form starts requiring the flag too (a behavior change to state loudly,
Rust-first per Open Q4)."* I adopted the option and lost the rider. Stated now,
plainly:

> **`export-wallet --descriptor <wsh> --format bitcoin-core` exits 0 today and
> will REFUSE without `--allow sigless-branch` once Phase 1 lands.** §1's table
> describes **pre-Phase-1** behaviour. This is a behaviour change to a surface
> shipped since v0.97.0 and it needs a release note, not a footnote.

**The enforced rule set, which round 1 left singular and I left unscoped.**
`rust-miniscript` runs **no** sane rule on Wsh/Sh/Bare at `from_str`, so
"uniform policy" was itself an unstated fork. Taking it now:

- **(b) `sigless-branch` only, enforced uniformly.** Chosen.
- Not (a) all five, which would be true `build-descriptor` parity but would
  start refusing every currently-exportable wsh/sh descriptor that is
  malleable, repeats a key, mixes timelocks or exceeds resource limits. Four
  new refusals nobody asked for on a shipped tool, each deserving its own
  evidence and its own decision.

The five-value vocabulary is still **shared** (I2/N1). On this surface the other
four are simply never enforced, so requesting one always produces the existing
*"requested but did not fire"* note — no new concept, and the plan says so
explicitly rather than leaving a flag that silently does nothing.

**Open Q4 is now in scope, not adjacent to it.** Uniform policy *is* touching
the tr/wsh sanity asymmetry Q4 flags as potentially normative, so Q4's
obligation binds: **Rust-first with vectors pinning the new refusals.** I
deleted the Rust-primary bullet from Phase 1 in the round-1 diff; it is restored
and strengthened.

**F-2 — the plan must make the two decisions, not the implementer's PR.**
Round 1 said the plan decides where relaxation lives; my fold deferred it to a
PR and dropped the sole-gate caveat. Decided: **a single admission gate at
intake.** Every downstream re-parse then becomes a lenient parse of an
already-admitted string. The implementer still enumerates the sites file+function
(that is discovery, not a decision), and the
**sole-gate / `--template` invariant is an acceptance bullet**: the intake gate
must be the only admission point, and `--template` inputs must not route around
it.

**F-3 — my acceptance contradicted my own ruling.** I ruled the never-silent
contract extends to every enforced wrapper, then wrote acceptance pinning
fired-detection to **tr leaves only**. Corrected: **fired-detection per enforced
wrapper — per-leaf for tr, top-level for wsh/sh.** `to_ext_params` joins the
pieces moving to the shared home, and the module is named:
`descriptor_builder::allow` (beside `gate.rs`, which already owns `AllowSet`).

**F-4 — I presented two added tests as closing I4 while deleting the baselines.**
Restored to acceptance, per wrapper and with the format named: flagless refusal
on **tr and wsh**; export-with-flag; the fired warning; the requested-not-fired
note. And the sub-part I skipped is ruled: **`--allow` on
`--template` / `--slot` / `--from-import-json` produces the did-not-fire note**
rather than refusing — those paths do not reach the descriptor gate, so there is
nothing to waive and a refusal would be a lie about why.

**F-5 — "Phase 1 may start now" survived the RED gate it sat beside.** Removed.

**N-2 — the import criterion needs a harness.** C1's acceptance ("an import, not
an emit") binds later phases; naming how: a regtest `bitcoin-cli
importdescriptors` against a pinned Core version, asserting per-entry `success`.
Available on this box and already used to produce the round-1 evidence.

**N-3 — the no-trace rationale contradicted my own I4 text.** M1 said "emitters
are passthrough by design", which is not true of the transforming emitters I4
names (`bip388`). Narrowed: **no trace in the artifact** stands as the ruling,
but on the ground that a trace would be format-specific and silently dropped by
most targets — not on a passthrough claim that is false for some of them.

#### Round 3 fold — two rulings that did not compose

**R0 round 3: 0C / 2I / 1M / 0N.** (`R0_export_phase1_round3.md`) Both
Importants are NEW and both are mine: I answered F-2 and F-4 independently and
never checked that the two answers describe the same surface.

**R3-1 — I decided two incompatible gate topologies.** F-2's answer says the
intake gate is the **ONLY** admission point; F-4's says
`--template`/`--slot`/`--from-import-json` **do not reach** the descriptor gate.
Against a surface with **three** descriptor intakes those cannot both hold, and
under one reading a sigless wsh envelope **exits 0 via `--from-import-json` with
no flag** — which would falsify "the wsh hole closed rather than pinned" in the
same document that claims it.

*Ruling — topology (B), the one consistent with this plan's own acceptance:*
**the admission gate runs on the canonical descriptor where all three arms
converge, before `EmitInputs`, honouring the `AllowSet` uniformly.**

That makes F-4's answer a **prediction** rather than a rule, and splits it:

- `--template` / `--slot` — a builder-produced descriptor cannot carry a sigless
  branch, so those paths only ever emit the note. True, and now stated as a
  consequence rather than an exemption.
- **`--from-import-json` — CORRECTED.** An envelope descriptor **can** be
  sigless. It is gated and waivable **exactly like `--descriptor`**. My round-2
  text put it in the exempt list; that was wrong and is the hole R3-1 found.

**R3-2 — the note I reused asserts a check that never ran.** `(b)` enforces
`sigless-branch` only, and I pinned the *existing* did-not-fire note — which
literally prints *"(the policy passes that rule without it)"*. For the four
rules that never run under (b), that sentence is **false**: the descriptor was
not checked, so nothing "passes". My own (b) ruling made the reused note lie.

*Ruling:* (b) stands and note-not-refusal stands. **The "passes that rule"
parenthetical may only ever be printed by a rule that actually ran.** Two
export-side wordings, hung on the existing export-wording acceptance bullet:

- **unenforced rule** — *"note: `--allow <rule>` has no effect on
  `export-wallet` — only `sigless-branch` is enforced here; the descriptor was
  NOT checked against `<rule>`"*
- **ungated path** — *"note: `--allow` does not apply to this path — no
  descriptor admission gate runs on `--template`/`--slot`"*

Two distinct reasons no longer collide on one sentence, and (b) becomes as
honest in the tool as it now is in the plan.

**R3-3 (Minor) — "format named" named no format.** The baseline tests use
**`--format bitcoin-core`**: it is the default, and it is the format with
measured evidence on both sides of the air gap.

#### Round 4 fold — the composition lens, run on my own rulings

**R0 round 4: 0C / 2I / 1M / 0N.** (`R0_export_phase1_round4.md`) I asked round
4 to check the rulings **pairwise** rather than in isolation, because that is
what round 3 caught. It found two more, both mine, both from the same habit.

**R4-1 — my two note wordings do not compose with my own topology ruling.**
Under topology (B) the gate is uniform, so **there is no ungated path** — and my
new *"no descriptor admission gate runs on `--template`/`--slot`"* note is
therefore **false under the very ruling I wrote it beside**.

*Fix: delete a wording, not add one.* The note matrix loses its arm dimension
entirely:

- `--allow sigless-branch`, **any arm** → the fired warning, or the did-not-fire
  note. True everywhere *as a consequence of the uniform gate*, including
  `--template`/`--slot`.
- `--allow <other>`, **any arm** → the unenforced-rule wording. Already
  arm-independent and already true everywhere.

**R4-2 — "where all three arms converge, before `EmitInputs`" is not one
place.** The arms build `EmitInputs` at **two** sites in **two** functions, and
the shared pre-`EmitInputs` boundary I gestured at also serves **`restore`'s**
two production constructors. Machine-run at toolkit tip `5f88071c`:
`restore --md1 --format bitcoin-core` on this wallet's sigless wsh **emits
flagless at exit 0 today**, 2694 bytes. So my one phrase admits two compliant
placements that are *observably different products*, one of which silently
breaks a shipped, waiver-less surface I never intended to touch.

*Ruling, stated as a mechanism against real code rather than a locus:*

> **One gate helper, invoked at `export-wallet`'s two `EmitInputs` construction
> sites (`run` and `run_from_import_json`), on each arm's canonical descriptor,
> honouring the `AllowSet`.** Other `EmitInputs` builders —
> `cmd/restore.rs:2496` and `:2801` — are **explicitly out of scope. Phase 1
> makes no behaviour change to `restore`.** If that door should be ruled on, it
> is its own decision with its own release note.

And the consequence my topology implied but never stated: **the `--descriptor`
intake parse at `:524` becomes LENIENT**, so a tr form can reach the gate at
all. An implementer who leaves it strict fails the export-with-flag baseline, so
this is machine-caught — but it should not have to be discovered.

**R4-3 (Minor) — an acceptance bullet that cannot be satisfied.** The
sigless-envelope bullet is unsatisfiable for tr, which refuses regardless of the
flag. Scoped to **wsh**.

#### The constraint that survives all of this

**No help text, doc, commit message or release note may say `--allow` "enables
export to Core / Nunchuk / Sparrow".** It enables **emission**. Measured: Core
refuses the result at import on every version through v31.1, and the rule is
non-waivable there. Phase 1 is worth doing on parity, inspection and archival
merits — and on those alone.

#### Acceptance

- Every strict parse site on **all three arms** enumerated file+function and
  listed in the PR, **annotated with which arm each serves** — that annotation
  is what makes "ONLY admission point" assertable rather than asserted.
- **One gate helper at `export-wallet`'s two `EmitInputs` construction sites**
  (`run`, `run_from_import_json`), on each arm's canonical descriptor, honouring
  the `AllowSet`; asserted to be the ONLY admission point with no arm routing
  around it. **`cmd/restore.rs:2496`/`:2801` are out of scope — no behaviour
  change to `restore`.**
- **The `--descriptor` intake parse at `:524` becomes lenient**, so a tr form
  reaches the gate at all.
- Uniform enforcement of **`sigless-branch` only**; the wsh hole closed rather
  than pinned; the other four rules explicitly not enforced on this surface.
- **Baseline tests, per wrapper, `--format bitcoin-core`:** flagless refusal on
  tr AND wsh; export-with-flag; the fired warning; the requested-not-fired note.
- **`--from-import-json` gated like `--descriptor`** — a sigless **wsh**
  envelope refuses without the flag and exports with it. This is the hole round 3
  found. (Scoped to wsh: tr refuses regardless of the flag, so a tr bullet would
  be unsatisfiable.)
- Per-rule over-admission tests — requesting one rule admits no other.
- Keyless-leaf vector through the transforming emitters (`bip388`).
- **Fired-detection per enforced wrapper** — per-leaf for tr, top-level for
  wsh/sh — implemented and tested. New code, not reuse.
- Vocabulary, printer and `to_ext_params` moved to `descriptor_builder::allow`
  **before** the second caller is wired.
- Export-specific warning wording (not build's "don't author this").
- **Rust-first vectors pinning the new refusals**, per Open Q4 — this phase
  touches the asymmetry Q4 flags as potentially normative.
- **A release-note line for the behaviour change.**
- `--allow sigless-branch` on `--template`/`--slot` emits the **did-not-fire
  note** — a consequence of the uniform gate, asserted as such; `--allow <other>`
  emits the unenforced-rule note. Same wordings as `--descriptor`, **because no
  arm is ungated**.
- **The "passes that rule" parenthetical is never printed by a rule that did
  not run**, asserted per unenforced rule.
- Re-review to 0C/0I before any of it merges.

### Phase 2 — **DELETED**

Nunchuk needs no emitter; `descriptor` and `bsms` already emit its two import
shapes. What remains is one regression test pinning that, folded into Phase 3.

### Phase 3 — Sparrow refusal pinned as deliberate

One test asserting `--format sparrow --descriptor` refuses, whose comment
states WHY: Sparrow would misimport this shape with wrong addresses.

### Phase 4 — G3, hot export — **gated on an explicit operator go-ahead**

Not because it is hard, but because it writes spendable key material to disk and
the operator asked for it in one clause of a long request. It deserves its own
confirmation before anyone builds it.

### Phase 5 — G4, SH2 NFC share — separate cycle

Firmware work with its own risk profile. Not in this plan's scope beyond
recording that the route exists and is cheaper than assumed.

---

## 4. Open questions

1. Does Nunchuk import BSMS? (PENDING)
2. Which Bitcoin Core version watches vs solves each wrapper? (PENDING) —
   `export-wallet` already has `--bitcoin-core-version`, so the answer is a
   parameter, not new plumbing.
3. Should `--allow` on `export-wallet` be named `--allow` (matching
   `build-descriptor`) or `--experimental` (matching `md encode`)? The two
   surfaces disagree today and this plan picks `--allow` for locality. Worth one
   line of operator input if they care.
4. A measured tr/wsh sanity-check asymmetry and `md`'s depth-0 xpub
   re-serialization are flagged in the CLI-surface report as *potentially
   normative*; if either is touched, it is Rust-first with vectors.
