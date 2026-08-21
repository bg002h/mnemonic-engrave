# Staged plan — arbitrary `tr()`/`wsh()`, and on to concrete descriptors

**STATUS: DRAFT FOR APPROVAL, 2026-08-20.** No R0 review, no gates, nothing may
be implemented from it. Written because rulings D7 and D8 enlarged the scope past
what one cycle can hold, and a staged plan is the operator's own call ("we may
need a staged plan for this").

Supersedes the unapproved phase sketch in
`Preliminary-Brainstorm-Arbitrary-Tr-Wsh-Wallet-Backup.md` §6 Q5. Every fact
below carries a `file:line` or a measured value taken **today**, against the
working tree, not quoted from the brainstorm.

---

## What is already done, so no stage re-derives it

| | evidence |
| --- | --- |
| Arbitrary `tr()`/`wsh()` **engrave and verify** as template-only md1 | shipped 2026-06-21, fork `f924556` |
| **R1** — `v:` wrapper-chain break | FIXED, `descriptor-mnemonic` `285b9fc9` |
| **R4** — `--path` on `md address`/`md verify` | DONE, `a785c3b5`, round-trip verified |
| **F-210** — all three operator journeys regenerate | closed 2026-08-20 |
| Device flash/RAM headroom | measured **1.41 MB / 62 KB = 8.4% / 11.8%** |

**The framing trap, which has caught two sessions:** "the SH2 can't do
miniscript" is false, and the *host* is not the gap either. **Every remaining gap
is device-side or vector-side.**

---

## Stage 0 — lift the miniscript pin, port `md-cli` (ruling D7) — **DONE 2026-08-20**

| repo | commit | gate |
| --- | --- | --- |
| `descriptor-mnemonic` | `5b4d20ad` | 758/758 nextest, fmt + clippy clean |
| `mnemonic-toolkit` | `5f88071c` | 3893/3893, all 3 required contexts green |
| `seedhammer` (fork) | `2b84de4` | 887/887 gui + every other package, wasm vet |

What landed beyond the port itself, none of it foreseen when the stage was
written:

- **`render_descriptor` is deleted** and `d.to_string()` is correct again. The
  previous cycle's tripwire decided it: it passes at `13.0.0` and FIRES at
  `ff4732e`. It was **inverted, not deleted** — it now asserts upstream still
  nests correctly, because addresses do not go through `Display`, so a
  regression would produce right addresses and an unparseable descriptor
  silently, exactly as before.
- **The property tests reach nested taptrees for the first time.**
  `t_tr_tree`'s depth-2 arm was capped *because* of the Display bug, so the
  shapes this whole feature is about were excluded from every property run.
  Restored, unbalanced on purpose, and `typed_generator_reaches_depth2_taptrees`
  proves it fires — 29 in 400 samples — because a restored arm that never fires
  looks exactly like a restored one.
- **Two more depth gates came down with it:** the toolkit's
  `ensure_taptree_depth_le_one` (now `ensure_taptree_wellformed`, keeping only
  its malformed-tree branch), and its two refusal cells flipped to
  reconstruction.
- **The device's warning was false and is rewritten.** It claimed the shipped
  toolkit cannot reconstruct the taptree and that recovery awaited an unreleased
  rust-miniscript. The caveat is now a MINIMUM VERSION (`md 0.13+ / toolkit
  0.97+`), and its test asserts the superseded phrases are *absent*.
- **An EXPERIMENTAL docs appendix retired.** CI caught it: the examples golden
  pinned two errors that are now successes, and regenerating alone would have
  shipped a document whose prose contradicted the output printed beneath it.
- **`at_derivation_index` deprecation handled with both branches** in
  `derive.rs` — `derive_at_index` for wildcards, `into_definite` otherwise —
  rather than assuming md1 can never mint a wildcard-free descriptor.

**R5 did not close for free**, as the fold predicted: `sortedmulti_a`'s
unencodability is a standards boundary, not an upstream gap.

### Original stage text (kept — the measurements are still the evidence)

**Measured today, by patching the pin to `ff4732e` and building.** Exactly two
errors, both in **one file**:

```
error[E0432]: unresolved import `miniscript::descriptor::WshInner`
  --> crates/md-cli/src/parse/template.rs:945
error[E0599]: no variant `SortedMulti` for enum `ShInner`
  --> crates/md-cli/src/parse/template.rs:931
```

**PR #915 is a refactor, not a rename**, and that is the whole cost of this
stage. In `ff4732e`:

- `ShInner` is now `{ Wsh, Wpkh, Ms }` — the `SortedMulti` variant is **gone**
  (rust-miniscript `ff4732e`, `descriptor/sh.rs` lines 40-47);
- `WshInner` **does not exist**;
- `sortedmulti` is now a miniscript **`Terminal::SortedMulti`**
  (rust-miniscript `ff4732e`, `miniscript/astelem.rs` lines 155-158) — it moved from the descriptor layer
  into the AST.

So the port is:

1. delete the `ShInner::SortedMulti` arm (`crates/md-cli/src/parse/template.rs:931`);
2. rewrite `walk_wsh_inner` (same file, lines 941-953) — with `WshInner` gone it becomes a
   direct walk of the inner miniscript;
3. **add `Terminal::SortedMulti` and `Terminal::SortedMultiA` arms to
   `walk_miniscript_node`** — it handles `Terminal::Multi` (line 976) and
   `Terminal::MultiA` (line 982) and neither sorted form, because until now they
   could not arrive that way.

**The normative question this stage MUST answer, and must not answer by
accident.** `md-codec` asserts today:

> `Tag::SortedMulti must be the sole child of wsh/sh; cannot appear as a
> miniscript leaf`

(`crates/md-codec/src/to_miniscript.rs:575-577`. The attribution sits outside the
quote deliberately: `plan-glyph-check.sh` reads a blockquote as an operator
string, and an em dash in one is undrawable on the device font. Keeping the quote
to the string itself is what makes that check mean something.)

Upstream has just made it exactly that: a leaf. Two options, and they differ on
the wire:

**RULED: HOLD (a).** Not a preference — the standards settle it. Opus recon
persisted at `design/agent-reports/sortedmulti-leaf-bip-recon.md`; every claim
below was re-measured independently before this fold.

| source | verbatim |
| --- | --- |
| BIP-388 l.138 | `sortedmulti(k,KEY_1,KEY_2,...,KEY_n)` **(inside `sh` or `wsh` only)** |
| BIP-383 l.37 | "`multi()` and `sortedmulti()` expressions can be used as a top level expression, or inside of either a `sh()` or `wsh()` descriptor." |
| BIP-379 (Miniscript) | **zero** occurrences of `sortedmulti` — it is not a Miniscript fragment at all |
| BIP-386 l.118 | lists `multi_a()`/`sortedmulti_a()` as expressions **of BIP-387**, a sibling category to the Miniscript fragments |

BIP-388 is the decisive one, because md1 templates are BIP-388-shaped: it says
*only*, in parentheses, in the normative list.

**PR #915 is evidence AGAINST nesting, not for it.** It is self-described as an
internal cleanup rather than a conformance fix, and its own body notes that
`sortedmulti` "cannot be decoded into from Script" — which is exactly the
property that disqualifies it as a Miniscript fragment. At `ff4732e` upstream
dispatches `"sortedmulti"` through its generic recursive expression parser with
no depth guard, so **rust-miniscript is now more permissive than the standard**.
That is the trap this stage must not follow.

**Our codec already implements the rule, and already implements the asymmetry**
(`crates/md-codec/src/to_miniscript.rs:575-586`): `Tag::SortedMulti` refuses with
"must be the sole child of wsh/sh", while `Tag::SortedMultiA` refuses with "must
be a tap-leaf root child" — which is correct, because `sortedmulti_a` legitimately
appears at arbitrary *taptree* depth under `tr()`, as a **sibling** of miniscript
expressions rather than nested inside one. So Stage 0 changes no admission rule.

**But one REASON goes stale at the new pin, and it must be updated in the same
commit.** The `SortedMultiA` message ends "rust-miniscript v13 has no
`Terminal::SortedMultiA` fragment" — true today, **false at `ff4732e`**. Left
alone it tells a future reader the restriction is an implementation limitation,
inviting them to "fix" it by relaxing a rule the BIPs impose. The restriction is
normative; the message must say so.

**Consequence for R5, correcting an earlier guess in this plan:** `sortedmulti_a`
does **not** close for free when the pin moves. Its unencodability is a standards
boundary, not an upstream gap, so R5 is a documentation-and-fencing task rather
than a feature that arrives with `ff4732e`.

**Then, and only then, the depth gate — and the evidence already exists and has
been RUN (2026-08-20).**

The previous cycle left a self-disarming tripwire inside
`crates/md-codec/tests/address_derivation.rs` (the `assert_ne!` closing
`nested_taptree_renders_with_nesting_intact`, not a separate test as the comment
above `render_descriptor` implies). It asserts upstream's `Display` still
DISAGREES with our corrected renderer. Measured both ways:

| pin | tripwire | meaning |
| --- | --- | --- |
| crates.io `13.0.0` (today) | **passes** | upstream still mangles the tree; `render_descriptor` is load-bearing |
| git `ff4732e` (spiked) | **fires** | *"upstream Display now agrees ... #953 has landed in the pinned release; delete render_descriptor and use to_string()"* |

(The assertion's own wording uses an em dash, which `plan-glyph-check.sh` cannot
draw; it is quoted here in italics rather than a code span so the check is not
answering a question about device copy with a line of Rust.)

So the depth-≥2 gate's premise is now falsified BY MEASUREMENT rather than by
argument, and `render_descriptor` can be deleted in favour of `to_string()` in
the same stage. Note the md-codec crate must be tested alone (`-p md-codec`)
while `md-cli` is still broken by the port.

The original reasoning, unchanged: `design/SPEC_seedhammer_template_engrave.md:36` says the gate is about
**off-device recoverability**, so lifting it requires showing depth-≥2 taptrees
now round-trip through a *released-or-pinned* renderer — which is precisely what
#953 changes. Re-run the reproduction that re-confirmed the gate on 2026-08-18
and require it to now pass.

**Watch for:** the two pin regimes (open question #4). `mnemonic-toolkit` already
patches miniscript to git `95fdd1c` while `descriptor-mnemonic` uses crates.io
`13.0.0` unpatched — measured today. This stage should land them on **one** rev
or state in writing why not.

**Exit:** `md-cli` builds and its suite is green on `ff4732e`; the toolkit is on
the same rev; the depth gate is lifted with a re-run reproduction as evidence;
R5 (`sortedmulti_a`) is fenced and documented rather than closed — see the
ruling above.

---

## Stage 1 — R3, keyed conformance vectors (the real blocker)

**Measured today: 15 of 15 `test_vectors::MANIFEST` entries carry `keys: &[]`.**
Not the 13 of 15 recorded. There is **no keyed vector at all**, so the Go side
has nothing to conform to and every later stage would be guessing.

Per vector, export: template string, per-`@N` xpubs + fingerprints, canonical
descriptor string, scriptPubKey hex, `addresses[chain][0..N]`, both wallet ids
(`WalletDescriptorTemplateId` **and** `WalletPolicyId`), `Md1EncodingId`, md1
chunks.

**The named trap, and it is not optional:** the exporter must **not** call
`Descriptor::to_string()`. Both descriptor renderers are defective in different
ways on exactly the shapes this feature targets, so naive vectors would vouch for
the wrong answer. Vectors must be built from the round-trip that R4 established:
encode → md1 → decode → derive, cross-checked against an independent path.

**Exit:** every MANIFEST entry keyed; a Go-readable export; and a conformance
runner that fails when the two implementations disagree.

---

## Stage 2 — device-side display/expand — **DONE 2026-08-20** (`seedhammer` `eceed57`)

The cheap tier landed: `md.PolicyShapeChunks` walks the decoded tree and the
consent screen renders spend-path structure instead of "Cannot fully display
on-device."

    Key-path: A KEY CAN SPEND ALONE
    Spend paths: 3 (tree depth 2)
      1: 1 key(s), custom
      2: 1 key(s), custom
      3: 2-of-3 +timelock

**C3's objection is answered structurally, not argued away.** It refused a
summary because summarizing one leaf hides the key-path and the other leaves —
so `PolicyShape.Complete` is a field: the walk understands every node and
describes every spend path, or the screen shows the old wording and nothing from
the summary. Three refusals to invent are pinned by tests: the key-path line is
never omitted, `K/N` are set only for a genuine threshold over keys (an
`and_v(v:pk,older)` branch reads "1 key(s), custom", never "1-of-1"), and an
unknown tag sets `Complete=false`.

**Still open, and now the whole of the remaining display work:** the full text
renderer (`seedhammer-broad-miniscript-renderer`). The summary earns the device
the right to say *something* about a complex policy; it does not render one.

### Original stage text

`classifyPolicy` (`seedhammer/md/md.go:1266`) returns `PolicyComplex` for
anything outside an enumerated shape list, so a complex policy degrades to the
honest-minimal consent screen. That copy is **deliberate and correct** for what
the device can prove today — it is the thing a renderer *earns* the right to
replace, not a bug to delete.

Take the cheaper tier first, as the filed follow-up says: a **structural
summary** walked from the already-decoded tree (threshold structure, per-branch
k-of-N, timelock/hashlock presence, leaf count, taptree depth) with **no** text
render.

**The output contract is the §3 invariant, and on-device it is the whole point:**
if the device cannot render faithfully it must **refuse to show a rendering**
rather than show a broken one. A malformed descriptor on a consent screen,
seconds before someone commits steel, is strictly worse than an honest refusal.

---

## Stage 3 — device-side address derivation

**RULED 2026-08-20 (operator, on a fable recommendation): R5 is the FIRST work
item of this stage, and R2 defers to S6.**

**R5 — DONE 2026-08-20** (`descriptor-mnemonic` `75032c2f`, `seedhammer`
`b79e2c9`). `sortedmulti_a` at a taproot leaf now derives; nested inside a
fragment it still refuses, by the standard. The conversion went into
`tree_to_taptree`'s leaf path exactly as ruled, leaving the generic converter's
nested refusal byte-for-byte.

Proven semantically rather than by "it returned an address": `sortedmulti_a` is
order-INVARIANT and `multi_a` is not, and the sorted address equals `multi_a`
with the keys already in sorted order — which pins *which* order the sort
produces and rules out a reverse or a no-op. One known-bad self-test cell flipped
to a conversion; its `wsh` sibling still refuses, which is what proves admission
did not widen into segwit v0. A keyed conformance vector carries the shape to Go,
where both wallet ids agreed with no port change.

**Original framing:** the CLI ADMITTED a card it could not
verify: `md encode` produces a valid `tr(@0,sortedmulti_a(...))` card (that half
started working when PR #915 was ported in S0), while `md address` refuses it.
The project's proof model is "derived addresses + wallet id", so an engraved card
whose address cannot be derived is a backup nobody can verify with our own tools.
It leads this stage because **S3 IS address derivation ported from Rust** — if
Rust still refuses, S3 either ships the hole onto the device or lets the Go port
lead, and both break Rust-primary.

Taken *inside* S3 rather than before it, deliberately: it is risk-set normative
work whose vectors and review gate make it a mini-cycle rather than a quick fix,
so it should carry the stage's gate rather than a bolted-on one.

**Where the fix goes, corrected.** NOT the `Tag::SortedMultiA` arm in the generic
node→Terminal converter — that arm's *nested* refusal is a standards rule
(BIP-386/387 put `sortedmulti_a` in BIP-387's category, a sibling of the
Miniscript fragments) and must stay byte-for-byte. The legal position is the
tap-leaf root ONLY, so the conversion belongs in `tree_to_taptree`'s leaf path,
before it delegates. Upstream now has `Terminal::SortedMultiA` (PR #910, in the
pinned rev), which is what makes this writable at all.

**R2 — `l:`/`u:` normalization → S6, with an acceptance shape.** It has no
operator-visible surface today: the device shows a structural summary, not
descriptor text, and concrete text comes from rust-miniscript's `Display` — the
same renderer Sparrow and Nunchuk sit on — so the coordinator comparison already
matches byte-for-byte where it counts. The only cross-form exposure is our
template render (long `or_i(0,X)`) vs concrete text (`l:X`), and nothing puts
those in one view until S6 exists. **Acceptance:** S6's vectors assert
byte-equality between our emitted concrete descriptors and coordinator output; if
the template/concrete pair turns out to be visible together, normalize the
template renderer then — Rust first, vectors, then the Go port. Not dropped:
"two renderings of one policy read as a mismatch to exactly the careful operator"
is a real honesty concern, it simply has no consumer yet.

### Progress — two capabilities landed 2026-08-20

| what | commits | cross-checked vs Rust |
| --- | --- | --- |
| **taproot script-path addresses** | `338e8c8` | 24 addresses (tr-with-leaf, sortedmulti_a, both depth-2 chiralities) |
| **segwit-v0 witness scripts** (`wsh` miniscript) | `40318b8` | 18 addresses across 3 vectors |

The `wsh` fragment set is the pathological journey's OWN wallet — `or_i`,
`and_v`, `v:`, `after`, `older`, `sha256`, `multi` — so the target is a real
policy this repo already engraves rather than an invented one.

**Layering settled, and it is not what the stage text predicted.** The stage
assumed the emitter must live inside `md` and that this makes it "a normative
codec change". Emission does live in `md` (it owns the AST), but it does **no key
derivation** — the caller passes derived keys, because the use-site path belongs
to the address layer and applying it twice, or not at all, is a wrong address.
`address` still does not import `md`.

**Mutation testing drove three fixture fixes**, each a case where a test passed
that should not have:

- *ignore leaf depth* passed, because the only depth-2 vector was left-heavy —
  fixed by adding a right-spine mirror (`b8663056`);
- *emit `n` before `k`* passed, because both multis were `k == n` — fixed by
  making them 2-of-3 and 1-of-2 (`e30224ef`);
- *drop `sortedmulti`'s BIP-67 sort* passed, because the gate ran one vector —
  fixed by running every vendored `keyed_wsh_*`.

Also worth carrying forward: two mutation runs reported "not caught" when the
mutant had simply **failed to compile**. A build failure prints no FAIL lines, so
a mutation harness must verify the mutant built before believing a zero.

**Still unsupported, and refused rather than approximated:** tap leaves and `wsh`
fragments outside the emitted set (`thresh`, `and_b`, `or_b`/`or_c`/`or_d`, the
`s:`/`a:`/`d:`/`n:`/`j:` wrappers), which return `ErrTapLeafUnsupported` /
`ErrScriptUnsupported`.

### Original stage text

`seedhammer/address/address.go:97,130` derives `SortedMulti` and `Singlesig`
only; everything else is a typed `errUnsupported`. Until this grows, **no
receive/change address can be shown for a complex shape** — which is what D2 made
the proof of correctness, so Stage 3 is what makes D2 true.

Cheaper than feared: every primitive is already linked, and
`ComputeTaprootKeyNoScript` is a one-line wrapper over the fully general
`ComputeTaprootOutputKey(internalKey, scriptRoot)` — **passing a real Merkle root
is the entire taproot delta**. The real cost is that nothing in the fork walks
the parsed miniscript AST to emit Script, and `Decode` is package `md`'s only
export — so the emitter must live *inside* `md`, making it a normative codec
change and R0-gated. `multi_a`/`sortedmulti_a` need a CHECKSIGADD builder: **2 of
36 tags** genuinely blocked.

Per D6: show **receive and change, ≈0..2 each**, not five receive-only. Change is
where a policy mismatch silently loses funds.

---

## Stage 4 — the Wallet Policy program (D5) — **DONE 2026-08-20**

A new navigable program, not a rename of Multisig and not an in-place extension.
Most transport already exists: `bundleGatherFlow`
(`seedhammer/gui/bundle_flow.go:153`) is already a mixed-transport N-card gather,
and `supplyMultisigPolicyFlow` calls it then discards everything but one md1
(`gui/multisig.go:105`).

**Measured, and it decides the typed path:** a real 2-of-3 full-policy md1 is
**478 characters / 6 chunks**; its template is **28 characters / 1 chunk**.
Nobody types 478 characters on a touch panel, so "typed on SH2" is realistically
the *template* path with keys arriving by NFC or payload — which is exactly D4.

The consent screen **must name which wallet id it shows** (D2 + D4): the template
id is key-stable and the policy id is not, so an operator comparing against a
coordinator gets a false mismatch otherwise.

### Progress — the WIRING landed 2026-08-20 (`seedhammer` `d028d44`)

**What the stage actually needed first, and it was not the program.** Stage 3's
two capabilities existed only as package APIs. Every complex policy still reached
`gatheredDescriptorFlow`'s `expandUnsupported` branch and read *"Complex policy —
display only"* — a sentence that had become false for exactly the shapes an
operator most needs to check.

That is closed. `gui/policy_address.go` routes a decoded md1 to the taproot or
wsh deriver, `gatheredDescriptorFlow` asks before it asserts, and the addresses
sit on Button2 where `DescriptorScreen.Confirm` already puts them for simple
shapes. The screen is the same one — `descriptorAddressFlow`'s body became
`addressListFlow`, parameterised by its address source — so the
measure-and-advance paging rule exists once, and so does the use-site translation
(`expandedKeysToBip380`, shared with the flat route).

The consent requirement above is **met**: the screen labels the id it shows
("Policy id: …"), because an unlabelled 32-hex string is ambiguous between the
key-stable template id and the key-dependent policy id.

**Gated from the device's side**, not the library's: all 13 keyed conformance
vectors reach an address (4 flat, 9 complex), every index of every chain equal to
Rust byte for byte, and the operator test taps the button *through the drawer*
and reads the address off the rendered screen.

Two fixture gaps that mutation found, both fixed rather than noted:

| survivor | why nothing caught it | fix |
| --- | --- | --- |
| reverse a tap leaf's key indices | the corpus's only multi-key leaf was `sortedmulti_a`, which **sorts** — written order was unobservable in every input the suite owned | `keyed_tr_multi_a`, added Rust-first (`descriptor-mnemonic` `97d39e4b`) |
| delete the derive probe | every vector derives | `gap_tr_leaf_and_v` — a timelocked tap leaf Rust derives and this port refuses (F-214), pinned by shape |

### The program landed 2026-08-20 (`seedhammer` `09c5f14`) — **STAGE 4 CLOSES**

The 10th navigable program, inserted mid-enum beside the other engrave programs
so `bip85Derive` stays the bound `lastNav()` returns.

**It is its own program because neither existing one answers D2.** Engrave Bundle
can already gather and engrave a supplied md1 — its review reads "N cards
verified", which says the chunks reassembled and nothing about *which wallet*.
Engrave Multisig proves more but demands a seed and cuts cosigner plates: its
question is "am I in this policy", not "is this the right policy".

Transport is reused verbatim (`bundleGatherFlowResume`, `bundleEngrave`), so a
card enters through the same `offer()`, deduplication and validation. What is new
is the consent surface between them:

- **addresses are LINES on the consent screen**, not a side trip — D2 says the
  derivation belongs *on* the consent path, so an operator who taps straight
  through still passes them. Receive **and** change, two each (D6).
- **the id is named by the CODEC**, not by the caller: `md.FormAwareIdChunks`
  returns the id authoritative for the card's form *and* which kind it is, so the
  wrong label is unwritable. Same `isWalletPolicy` discriminant as the existing
  `FormAwareStub`, so a card's stub and its displayed id cannot disagree.
- **the absence of addresses is always stated**, and the two reasons are
  distinguishable — keyless-on-purpose (D4) vs. F-214's capability gap. An empty
  block reads as "this screen has no addresses", which is the observation that
  should stop an operator.
- **no seed class.** `progWalletPolicy` admits `ClassDescriptor` + `ClassMDMK`
  only, enforced in the admission table rather than by the flow not asking.

The 11 lockstep failures the enum comment predicts all fired and were updated
rather than loosened; `cmd/emu`'s decoy-needle pin went 2 → 3.

**Two mutation survivors, both in the TESTS:** the per-chain address count looped
on `addrProofPerChain` itself, so halving it moved both sides together (now a
literal, with the constant pinned once as a claim); and selecting the program
needs the physical Button3, because the carousel's right-arrow `Clip` covers the
whole right edge and a nav-slot hit test there resolves to the arrow.

**Carried out of the stage, both filed rather than absorbed:** F-216 (D3's
mk1-gated addresses for a keyless template — the slot-mapping rule is a decision,
and a wrong mapping shows a wrong address as *proof*) and F-215 (the
template-engrave shape guard is stale in both halves).

---

## Stage 5 — a new journey, actually executed

The pathological journey has **zero** emulator interaction with the md1 gather /
descriptor / address-verify flow, so a new journey is needed regardless of
F-210. F-210 is closed, so the generator it would be built on is now sound, and
`capture_operator.py` / `shots_operator.js` are the worked pattern.

**A plan may not close while any of its own gates has never been run.** This
journey must be *executed*, not merely specified — that was the defect that cost
the multisig cycle six rounds.

---

## Stage 5 — a new journey — **DONE 2026-08-20** (`mnemonic-engrave` `4d45a82`)

`SeedHammer-II-wallet-policy-journey.pdf`, and it is **executed**, not specified:
8 md1 chunks over emulated NFC into the Wallet Policy program, and the consent
screen's wallet id and four addresses compared against host-derived Rust output.
The capture **refuses to finish** on a mismatch, and the negative control was run
(corrupt one character of one address → *"the device's proof does not match the
host's"*).

Three things it measured that no reading would have: the gather tally counts
**cards, not chunks** (a per-chunk wait hangs forever on chunk 1 of 8); NFC
records carry **no spaces** (codex32's five-character grouping is for a human
reading a plate); and the **fingerprints are part of the wallet identity**.

It also found **F-154** on the framebuffer — the tenth program's carousel dot
drawn over the version text, exactly as F-154 predicted in August — and, through
a reader's two questions about it, **F-217**.

---

## Stage 6 — concrete descriptors (ruling D8)

**F-217 PREEMPTED THIS STAGE**, per the operator-proxy ruling in
`design/agent-reports/RULING_f217_vs_stage6_ordering.md`, and is now **closed**:
the corpus every remaining sub-stage would author vectors against was 9-of-9
contradictory, and 6c's whole purpose is to export descriptors verbatim into
files real coordinators import.

| sub-stage | state |
| --- | --- |
| **6a** concrete renderer | **DONE** (`descriptor-mnemonic` `c907240e`) — and the plan's premise was stale, see below |
| **6b** QR-series transport | **contract + ruling OPEN** — needs an operator decision on wire shape and on what may cross a display channel. Its round-trip vectors were gated on F-217(3), which is now done |
| **6c** named formats (BSMS / Nunchuk / Sparrow) | **UNBLOCKED, not started** — three features, each needing conformance vectors |
| **6d** engraving a concrete descriptor | **DEFERRED out of this cycle** by the ruling: unmeasured sizing plus an irreversible medium while content rules were still moving |

### 6a — the premise was stale, and measuring it first shrank the stage

This section said *"`render.rs` renders **templates only** (`@{idx}`) —
concrete-key rendering does not exist in md-codec at all"*, and ordered 6a around
writing a renderer. **It existed.** `md_codec::to_miniscript` has built concrete
descriptors all along — it is what every derived address goes through and what
the conformance corpus stores per chain — and both
`to_miniscript_descriptor` and `to_miniscript_descriptor_multipath` were already
public. What was missing was a **surface**: no caller but the maintainer vector
exporter.

So 6a is a command, not a renderer. `md descriptor` emits the concrete string
with real xpubs, key origins and its BIP-380 checksum; multipath (`<0;1>`) by
default because that is the form a coordinator wants; gated against the corpus —
every keyed vector, every chain, byte-identical.

Same lesson as [[departure-sections-need-a-run-check]], and the second instance
in this stage: the other was my own claim that the CLI could not express per-key
origins (F-217), which was equally wrong and equally one command away.

---

## Stages 6+ — original text (superseded above)

**Not in the current cycle**, and deliberately last: every stage above is a
prerequisite. Recorded now because D8 decides what the renderer is eventually
*for*, and because `render.rs` renders **templates only** (`@{idx}`) —
**concrete-key rendering does not exist in md-codec at all**.

- **6a — a concrete renderer, Rust-primary.** New normative surface. The §3
  invariant binds hardest here: a template that renders wrongly is a wrong
  *shape*; a concrete descriptor that renders wrongly is a wrong *wallet*.
- **6b — QR series transport.** Multi-part framing (BBQr, UR, or indexed chunks)
  is a wire decision with its own round-trip contract, and a partially-scanned
  series must be **detectably** partial. Note the device already refuses to emit
  ms1 over NFC — a QR series is a *display* channel and needs its own ruling on
  what may cross it.
- **6c — named backup formats: BSMS (BIP-129), Nunchuk, Sparrow.** Three
  features, not one: BSMS has normative text of its own; the other two are vendor
  formats. Each needs conformance vectors, and the Rust-primary rule applies to
  all three.
- **6d — engraving a concrete descriptor.** Sizing unknown: a concrete descriptor
  is far longer than a template, so plate count and legibility need measuring
  before this is a plan rather than an intention.

---

## Order, and why

```
S0 pin + port ──► S1 keyed vectors ──► S2 display ──► S3 derivation ──► S4 program ──► S5 journey ──► S6 concrete
   (unblocks         (nothing            (the cheap      (makes D2         (D5)          (executed)      (D8)
    depth≥2)          conforms            tier first)     true)
                      without it)
```

S0 first because D7 asks for it and it is the smallest measured piece. S1 next
because **every device-side stage is unverifiable without it**. S2 before S3
because a summary needs no addresses, while addresses without a summary prove a
wallet the operator cannot read.

## What is NOT priced

S2, S3 and every part of S6. S0 and S1 are measured; the rest are shaped. **No
stage below S2 should be treated as estimated** until its own recon runs.

## Process

Risk-set on three counts — normative codec behaviour, funds/keys/addresses, and
spanning repos. **R0 to 0C/0I before code, per stage.** Reports persist to
`design/agent-reports/`, written by the agent. Rust-primary: normative behaviour
lands in `descriptor-mnemonic` first, with vectors, then ports to Go.
