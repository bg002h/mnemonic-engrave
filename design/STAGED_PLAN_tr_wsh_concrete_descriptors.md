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

## Stage 0 — lift the miniscript pin, port `md-cli` (ruling D7)

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

## Stage 2 — device-side display/expand

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

## Stage 4 — the Wallet Policy program (D5)

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

## Stages 6+ — concrete descriptors (ruling D8)

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
