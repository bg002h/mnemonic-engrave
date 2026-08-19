# Preliminary brainstorm — arbitrary `tr()` / `wsh()` miniscript wallet backup bundles

**STATUS: TENTATIVE / PARKED, 2026-08-18.** This is a brainstorm snapshot, not a
spec and not a plan. It has passed **no R0 review**, carries **0 gates**, and
**nothing should be implemented from it.** It exists so a later session does not
re-derive what this one established.

**Parked by user direction** in favour of first auditing the m\* constellation
utilities and planning round-trip journeys constellation-wide. Resume only after
that work has framed what a round-trip actually is.

Supersedes nothing. Companion to `design/HANDOFF_arbitrary_tr_wsh.md`, which
remains accurate except for the corrections in §5 below.

---

## 1. What the user asked for

Let a user create and engrave arbitrary `wsh()` and `tr()`-wrapped miniscript
wallet backup bundles via the m\* constellation tools, accepting the limits of
what rust-miniscript supports (particularly nested taproot trees). Deployed as a
program or nested in an existing one. Data input via **payload**, **typed on the
SH2**, or **NFC**. The device should **display the wallet descriptor** and
**derive the first several addresses** so the operator can confirm the wallet is
correct.

## 2. Decisions the user made during the brainstorm

These are rulings, not proposals. Carry them forward.

| # | Decision |
| --- | --- |
| D1 | **Consume this cycle, compose later.** The device consumes a host-built policy; an on-device miniscript composer is a later cycle. F-150 #1 (the blank-screen dead-end in `buildMultisigPolicyFlow`) stays a **separate defect fix** and must not be folded in. |
| D2 | **Proof = derived addresses + wallet id.** A structural summary is useful; the full descriptor text is available **on demand**. This puts address derivation on the consent path, not beside it. |
| D3 | **Accept either input shape.** A full-policy md1 (carries xpubs) derives addresses immediately; a keyless template md1 shows its summary and **gates addresses on gathering N `mk1` key cards**. Skipping the gather proceeds to consent **without** address proof. |
| D4 | **Engraving a keyless template is a valid goal**, and a template has its own unique id. |
| D5 | **A new 10th navigable program, "Wallet Policy"** — not a rename of Multisig, not an in-place extension. |
| D6 | **Show receive AND change, fewer of each** (≈0..2 per chain) rather than five receive-only. Change is where a policy mismatch silently loses funds, so proving both chains derive beats proving one chain five times. |

### Consequence of D2 + D4 that must reach the screen

The wallet id serving as proof is **mode-dependent**: `WalletDescriptorTemplateId`
(key-stable) for a keyless template, `WalletPolicyId` (key-dependent) for a full
policy. They hash differently for the same wallet, so the consent screen **must
name which id it is showing** or an operator comparing against a coordinator gets
a false mismatch. Confirmed by recon with a worked example: the template id is
identical across keyless/keyed/re-keyed (`b02b4403…`) while the policy id differs
in all three.

## 3. The invariant this brainstorm produced

Arrived at from the user's question *"if it is malformed, shouldn't we emit an
error and stop?"*, and it is the most durable thing here:

> **No renderer in this cycle, Rust or Go, may return a string it has not
> verified it can parse back.**

Rationale. `descriptor_to_template` returns `Result`, but `RenderError`
(`crates/md-codec/src/render.rs:35-39`) has exactly one variant, `MalformedTree`, describing a
malformed **input** tree. The function cannot say *"I produced a string that
isn't valid miniscript"* — and its only test, `render_template_snapshot.rs`,
contains zero `from_str`, so it never re-parses. A snapshot test blesses whatever
the code did, bug included.

Note the invariant does **not** mean erroring on `vj:` — that is a legitimate
shape rendered wrongly, and refusing it would reject a valid wallet. The error
belongs one level up, as the output contract the renderer never had.

**This transfers to the device, where it matters most.** The Go renderer cannot
re-parse through rust-miniscript to check itself, so the SH2 needs its own output
contract: if it cannot render faithfully it must **refuse to show a rendering**
rather than show a broken one. A malformed descriptor on a consent screen,
seconds before someone commits steel, is strictly worse than an honest refusal.

## 4. What recon established

Five agent reports, committed verbatim at `355c6b7`, plus one controller-run
differential at `83aa8ed`. All in `design/agent-reports/`:

- `wallet-policy-recon-miniscript-depth.md`
- `wallet-policy-recon-go-derivation-inventory.md`
- `wallet-policy-recon-transport.md`
- `wallet-policy-recon-rust-primary.md`
- `wallet-policy-recon-f210-journey.md`
- `wallet-policy-pin-regime-differential.md`

### 4.1 The codec is not the gap, and neither is the host

`md/md.go` implements the full miniscript tag set and arbitrary `tr()`/`wsh()`
**already engrave and verify** as template-only md1 (shipped 2026-06-21, fork
`f924556`). On the host, `md encode --from-policy` compiles a policy into a
template, and `derive.rs` (v0.32) delegates address rendering to rust-miniscript
so arbitrary shapes already derive. **Every gap is device-side.**

### 4.2 Rust is not ready to port against — the ordered blockers

- **R1 — the `v:` wrapper-chain break.** `crates/md-codec/src/render.rs:150-163` gives `Tag::Verify`
  its own arm instead of joining `render_wrapper_chain` (`crates/md-codec/src/render.rs:358`, whose
  dispatch at `:217` covers only `Check | Swap | Alt | DupIf | NonZero |
  ZeroNotEqual`). So `vj:` emits as `v:j:` — **a string rust-miniscript's own
  parser rejects**. The 14-entry frozen KAT misses it because its only chain case
  is `snj:`. Emitted by **two shipped binaries**: `md` and the toolkit both call
  `md_codec::descriptor_to_template` (`crates/mnemonic-toolkit/src/cmd/inspect.rs:325`, `:458`).
- **R2 — `l:`/`u:` normalization.** Rust renders `or_i(0,X)`/`or_i(X,0)`;
  rust-miniscript's canonical Display renders `l:X`/`u:X`. Both re-parse, so it
  is a divergence in *the text the operator compares against their coordinator*,
  not a break. Pick one, spec it, pin it.
- **R3 — machine-readable conformance-vector export. The real blocker.** Needed
  per vector: template string, per-`@N` xpubs + fingerprints, canonical
  descriptor string, scriptPubKey hex, `addresses[chain][0..N]`, both wallet ids,
  `Md1EncodingId`, md1 chunks. **13 of 15 `test_vectors::MANIFEST` entries carry
  `keys: &[]`** (`crates/md-codec/src/test_vectors.rs:68-117`). Without this the Go side has nothing
  to conform to.
- **R4 — `--path` on `md address` and `md verify`.** Both lack it; `md encode`
  has it. Consequence: exactly the non-canonical shapes this feature is about are
  unreachable via `--template`. **R4 is a prerequisite of R3**, not a sibling.
- **R5 — close or fence `sortedmulti_a`.** A first-class wire tag (`crates/md-codec/src/tag.rs:109`)
  that renders but **cannot be encoded by the CLI and cannot be derived**
  (`crates/md-codec/src/to_miniscript.rs:581-586`). Rust can render one and cannot price one.

**The trap R3 must avoid, and it belongs in the spec as a named constraint:** the
exporter must **not** call `Descriptor::to_string()`. Both available
descriptor-string renderers are defective in different ways — rust-miniscript's
`Display` (pre-#953, wrong for depth-≥2 taptrees) and md-codec's `render.rs` (the
`v:` bug) — and both corruptions land precisely on the shapes this feature
exists to support. Vectors generated naively would actively vouch for the wrong
answer.

### 4.3 The device side is cheaper than feared — "wiring, not primitive-building"

A real `tinygo build -target pico-plus2` measured **1.41 MB flash / 62 KB static
RAM — 8.4% / 11.8% of budget**. Every primitive is already linked: secp256k1,
all four BIP-340/341 tagged hashes, the taproot tweak, HMAC-SHA512, HASH160,
bech32/bech32m, base58, a general Script builder. `ComputeTaprootKeyNoScript` is
a one-line wrapper over the fully general `ComputeTaprootOutputKey(internalKey,
scriptRoot)`; passing a real Merkle root is the entire taproot delta.

Real costs: (a) **nothing in the fork walks the parsed miniscript AST to emit
Script** — `Decode` is package `md`'s only export and `Template` is a flat
summary, so the emitter must live *inside* `md`, making it a normative
codec change and R0-gated; (b) `multi_a`/`sortedmulti_a` need a CHECKSIGADD
builder — the only 2 of 36 tags genuinely blocked.

### 4.4 Most of the transport work already exists

`bundleGatherFlow` (`gui/bundle_flow.go:153`) is **already** a mixed-transport,
N-card gather. `supplyMultisigPolicyFlow` calls that same gatherer and then
discards everything but one md1 (`gui/multisig.go:105`). Typed entry exists too —
`validateMStar` (`gui/codex32_polish.go:259-281`) already recognises md/mk HRPs on
the full bech32 alphabet — but all four call sites are titled for secrets and none
is wired into the gather loop.

**The measurement that reshapes the typed path:** a real 2-of-3 full-policy md1 is
**478 characters / 6 chunks**; its template is **28 characters / 1 chunk**. Nobody
will type 478 characters on a touch panel, so "typed on SH2" is realistically the
*template* path with key cards arriving by NFC or payload — which fits D4.

### 4.5 F-210 is due this cycle but not on its critical path

Even perfectly regenerated, the pathological journey has **zero** emulator
interaction with the md1 gather / descriptor / address-verify flow — its only
device steps are seed typing and plate cutting — and its own miniscript shape is
excluded by the shipped `#10b D2` subset. **A new journey is needed regardless.**

Fix it first anyway, for a different reason: the root cause is **capture
plumbing** (four intermediates have never had a writer in any committed version;
`design/journeys/transcript_pathological.sh:18` reads `out/md1.txt` sixteen lines before the only
command that could produce it), plus a stale `me-preview` 0.5.1 against `me`
0.6.0. A new journey built on that same generator inherits the same defect.
~20–30 line refactor.

## 5. Corrections this session made to existing records

- **The tag set is 36, not 37.** `design/HANDOFF_arbitrary_tr_wsh.md:15` and
  `FOLLOWUPS.md` both say 37; that figure came from a name-grep that swept up a
  local variable at `md/md.go:563` (`tagRaw, err := r.read(5)`). Measured:
  `grep -cE '^\s+tag[A-Za-z0-9]+\s+tag = 0x' md/md.go` → **36**.
- **The depth-≥2 EXPERIMENTAL gate STAYS.** It was tempting to call it stale
  (PR #953 is merged), and that was wrong. #953 is in **no released version
  through 13.1.0**, verified by `git merge-base --is-ancestor` against all three
  tags, and the bug was reproduced live. The gate's premise has been
  independently re-confirmed, not weakened. It is about **off-device
  recoverability**, not the wire codec — `design/SPEC_seedhammer_template_engrave.md:36`
  says so explicitly.
- **Advancing the miniscript pin is not free.** `mnemonic-toolkit/Cargo.toml:20-22`
  records a spike finding the bump to `ff4732e` "build-clean + regression-free" —
  true for the toolkit, **false for `md-cli`**, which fails with two PR #915
  errors (`WshInner` unresolved, `ShInner::SortedMulti` missing) at
  `crates/md-cli/src/parse/template.rs:945` and `:931`.
- **Naming hazard, unfiled:** `tagPkh` (0x04, descriptor `pkh()`) and `tagPkH`
  (0x0B, miniscript `pk_h`) differ only in the case of one letter, in a
  funds-critical codec.

## 6. Open questions — NOT decided

1. **R2 and R5 — ruled inside the cycle, or filed out?**
2. **Advance the pin to `ff4732e` and lift the depth-≥2 gate this cycle?** Now
   priced: needs the `md-cli` port first. Tracked as
   `taproot-coverage-cycle-on-miniscript-gt-13-1-0`. Controller's tentative
   read was to **defer** and leave the gate untouched — the one part of the
   user's ask consciously left unshipped.
3. **Does the device ever show a *concrete* descriptor?** `render.rs` renders
   **templates only** (`@{idx}`, `crates/md-codec/src/render.rs:552`); concrete-key rendering does
   not exist in md-codec at all. Three xpubs at 111 chars each will not fit the
   panel, so the device likely shows template + per-slot fingerprints — but
   nobody has ruled, and off-device recovery *does* need a concrete descriptor.
4. **The two pin regimes** — document the divergence, or reconcile them? The
   differential found no covered divergence, but it is a **weak green**: the
   corpus barely exercises keyed derivation, which is the risk surface.
5. **Phase split.** Last shape discussed, unapproved: R1 standalone → P1 (R4
   then R3) → P2 (`md` emitter) → P3 (derivation) → P4 (Wallet Policy program)
   → P5 (new journey, actually run), with F-210's generator fix before P5.

## 7. Process notes for whoever resumes

- This is **risk-set work** on three counts — normative codec behaviour, funds /
  keys / addresses, and spanning repos — so it needs the R0 gate to 0C/0I before
  any code, with reports persisted by the agents themselves.
- **A plan may not close while any of its own gates has never been run.** The new
  journey must be *executed*, not merely specified. That was the defect that cost
  the multisig cycle six rounds.
- Two gates that earned their keep and apply here:
  `CITE_FORK_ROOT=… ./scripts/plan-cite-check.sh <doc>` and
  `./scripts/fold-propagation-check.sh <artifact> '<superseded phrasing>'`.
