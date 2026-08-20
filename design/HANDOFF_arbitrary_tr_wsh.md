# Handoff — arbitrary `tr()` / `wsh()` wrapper support

**Written 2026-08-18, at the end of the S6b cycle, to survive a context clear.**
Start here; do not re-derive what is below. Every fact carries a `file:line` and
was measured against the working tree on the date above.

---

## ▶ THE ONE THING TO READ FIRST

**The codec is not the gap.** The obvious framing of this feature — "the SH2
can't do miniscript, let's add it" — **is wrong**, and the first hour of a cold
session will be wasted proving it if this paragraph is skipped.

`md/md.go` implements the **full miniscript tag set**, **36** tags:

> **The 37 this document used to say was wrong, and the way it was wrong is the
> point.** It came from a name-grep that swept up a local variable — `tagRaw, err
> := r.read(5)` at `md/md.go:563` — which is not a tag at all. Counting the
> const-block declarations instead gives 36. A grep for a naming convention is a
> hand-count wearing a tool's clothes.


```
tagAfter tagAlt tagAndB tagAndOr tagAndV tagCheck tagDupIf tagFalse tagHash160
tagHash256 tagMulti tagMultiA tagNonZero tagOlder tagOrB tagOrC tagOrD tagOrI
tagPkh tagPkH tagPkK tagRaw tagRawPkH tagRipemd160 tagSh tagSha256
tagSortedMulti tagSortedMultiA tagSwap tagTapTree tagThresh tagTr tagTrue
tagVerify tagWpkh tagWsh tagZeroNotEqual
```

Arbitrary miniscript and multi-leaf taptrees **already engrave and verify**
on-device, shipped 2026-06-21 as template-only md1 (see `constellation-template-
only-engraving` in `FOLLOWUPS.md`, marked SHIPPED, fork merge `f924556`).

---

## WHAT IS ACTUALLY MISSING

Two gaps, in different layers, and they are **separable** — they may even be
separate cycles.

### Gap 1 — on-device DISPLAY/EXPAND

`classifyPolicy` (`md/md.go:1266-1315`) returns `PolicyComplex, 0, 0` for
anything outside an enumerated shape list. It recognises exactly:

| shape | result |
| --- | --- |
| `wpkh(@N)`, `pkh(@N)` | `PolicySingle` |
| `tr(@N)` **key-path only** (`!b.isNums && b.tree == nil`) | `PolicySingle` |
| `wsh(multi/sortedmulti)` | `PolicyMulti`, k, N |
| `sh(wpkh)`, `sh(wsh(multi/sortedmulti))`, `sh(multi/sortedmulti)` | as above |
| **anything else — any `tr` with a script tree, any combinator** | `PolicyComplex` |

Consequence at `gui/template_engrave.go:65`: a complex shape still engraves, but
the consent screen degrades to the honest-minimal form —

```
COMPLEX POLICY (advanced)
Cannot fully display on-device.
Script: <family>   Key slots: N   Template-ID: <4 bytes>
VERIFY against your coordinator / …
```

That copy is **deliberate and correct** for what the device can currently prove
(spec §4.2 / C3: summarizing one tapscript leaf would omit the key-path and
other-leaf spend conditions and mislead the operator). Do not treat it as a bug
to delete — treat it as the thing a renderer would *earn* the right to replace.

### Gap 2 — ADDRESS DERIVATION

`address/address.go:95-155` derives only:

- `SortedMulti` → `P2SH`, `P2WSH`, `P2SH_P2WSH`
- `Singlesig` → `P2PKH`, `P2WPKH`, `P2SH_P2WPKH`, `P2TR` (key-path only,
  `ComputeTaprootKeyNoScript`)
- everything else → typed `errUnsupported`, e.g.
  `address: multisig script: Taproot (P2TR): unsupported descriptor`

So even with a renderer, **no receive/change address can be shown or verified
for a complex shape** until this layer grows. Anything that promises the
operator an address for a miniscript policy has to cross this first.

### The separate, narrower parser

`bip380/bip380.go:300-340` — the **descriptor-string** parser (coordinator
import, not the md1 path) accepts `wsh|sh|pkh|wpkh|tr`, `sh`-wrapping of
`wpkh|wsh`, and **exactly one** inner function:

```go
switch script2 {
case "sortedmulti":
    r.Type = SortedMulti
default:
    return nil, fmt.Errorf("bip380: unknown script type: %q", script2)
}
```

Not even unsorted `multi()`. **Note the asymmetry**: `tr(sortedmulti(...))`
*parses* here (the wrapped-script guard only constrains `sh`) and is caught
later by `address`'s typed error. Worth a look during brainstorm — it is a
parse/derive split, not obviously a defect, but nobody has ruled on it.

---

## ALREADY FILED — READ BEFORE PROPOSING A PLAN

Two follow-ups in `design/FOLLOWUPS.md` frame this work, with sizing and a
deliberate ordering. **A brainstorm that ignores them will re-invent them.**

1. **`seedhammer-template-engrave-policy-summary-display`** — the *intermediate*
   tier. A structural summary from walking the already-decoded tree (threshold
   structure, per-branch k-of-N, timelock/hashlock presence, leaf count, taptree
   depth) **without** a full `to_miniscript` text render. Explicitly called "the
   cheaper, higher-value first step."
2. **`seedhammer-broad-miniscript-renderer`** — the full tier. Port the
   `to_miniscript.rs` semantics so any admissible template renders on-screen.
   **L-sized.** Smallest high-value first step named in the entry:
   `tr(NUMS,multi_a)` + unsorted `multi` display + a `scriptForTemplate`
   `PolicyMulti` arm.

Both say **own full gated cycle when picked up**.

---

## THE CONSTRAINT THAT DECIDES WHERE THIS STARTS

**The Rust-primary rule** (`CLAUDE.md`, standing user directive). Any change to
**normative behavior** — wire format, identity/stub algorithms, validation,
admission — lands **first in the primary Rust repo, with test vectors**, and only
then is ported to Go. Render semantics are normative: two implementations that
disagree about what a policy *says* is exactly the class this rule exists for.

| | |
| --- | --- |
| Rust primary | `/scratch/code/shibboleth/descriptor-mnemonic` @ `89ab0f62` (`main`) |
| The reference renderer | `crates/md-codec/src/to_miniscript.rs`, **696 lines** |
| md-codec version | 0.42.0 |
| Go counterpart | `seedhammer/md/md.go` (1505 lines), `gui/md1_expand.go` (160) |

Bind **semantics, not lines**: the Go port deliberately omits `rust-miniscript`
for TinyGo, so a behavior-faithful reimplementation is compliant. This is why
the renderer follow-up is L-sized rather than "port a file."

---

## PROCESS THIS CYCLE WILL NEED

This is **risk-set work** on at least three counts (`CLAUDE.md`): it changes
**normative codec behavior** (admission/rendering), it touches **funds, keys and
addresses**, and it **spans repos** (Rust primary + fork + possibly the toolkit).

So: orchestrate and gate. **R0 to 0C/0I before any code.** Reports persist to
`design/agent-reports/` written by the agent itself. Reviewer tiering: sonnet for
mechanical/fold verification, opus for design-level adversarial; **never propose
fable** — the operator calls for it or it does not happen.

**Ultracode is ON for brainstorm/design/spec and OFF for implementation against a
green plan** — but turning it on **requires asking first**. It has not been asked
for this feature yet.

Two gates that earned their keep in S6b and apply directly here:

```sh
CITE_FORK_ROOT=/scratch/code/shibboleth/wt-<branch> ./scripts/plan-cite-check.sh <doc>
./scripts/fold-propagation-check.sh <artifact> '<superseded phrasing>'...
```

The first exists because the default root is the fork's `main`, and the script
checks only that a line is *in range*, never what is *on* it — so it prints `ok`
for citations to unmerged branch work. The second exists because folds fail by
incomplete propagation; in S6b it caught a fifth stale site an entire review
sweep had missed.

---

## OPEN QUESTIONS FOR THE BRAINSTORM

Not answers — the things that need ruling before a spec can be written.

0. **The pathological journey does not currently regenerate — F-210.** It is the
   only journey exercising timelocks and a hashlock, i.e. exactly this cycle's
   shapes, and F-210 is assigned to THIS cycle to fix before leaning on it.
   Measured 2026-08-18: 9 non-zero exits on a fresh run vs 1 in the committed
   transcript, six intermediates read that nothing writes, and `mk`/`ms`/`me`
   all moved under it.
1. **Which gap is this cycle?** Display-only, address-derivation-only, or both?
   They are separable and Gap 2 is the one that touches funds.
2. **Summary tier or full render?** The follow-ups deliberately order summary
   *before* renderer. Is that ordering still wanted, or is the summary tier now
   skippable?
3. **What may the device claim?** The current copy refuses to summarize a
   tapscript leaf because doing so omits key-path and sibling-leaf conditions.
   A renderer must either render *everything* or state precisely what it omits.
   This is the safety heart of the feature, not a UX detail.
4. **Does anything need to change in Rust first**, or does `to_miniscript.rs`
   already define the semantics completely enough to port against? If the answer
   is "already complete", say so explicitly with evidence — that is a finding,
   not an assumption.
5. **Screen budget.** Rendered miniscript is long; the SH2 panel is 480×320 with
   a 417px-wide body clip. S6b added touchable scroll arrows (`Warning`), so
   scrolling now exists — but "reachable by scrolling is not the same as read"
   was that cycle's ruling on funds-critical text.
6. **`tr(sortedmulti(...))`** parses in `bip380` and fails only at derivation.
   Intended, or tighten the parser?
7. **Does an unrenderable shape still need the EXPERIMENTAL warning** once it
   renders? Depth-≥2 taptrees currently carry one naming unreleased
   rust-miniscript >13.1.0 / PR #953.

---

## HOW TO SEE THE THING RUNNING

**The emulator is the firmware GUI compiled to wasm** — the same `gui` package,
a real 480x320 framebuffer. It is how complex-policy screens get exercised
without cutting steel.

```sh
sh /scratch/code/shibboleth/seedhammer/cmd/emu/build.sh     # -> emu.wasm
cd /scratch/code/shibboleth/seedhammer/cmd/emu
python3 -m http.server 8777 --bind 127.0.0.1                # open index.html
```

Rebuild before trusting it: the checked-in `emu.wasm` is a build artifact and
was four days stale when this was written (it predated the whole S6b cycle).
`build.sh` also refreshes `wasm_exec.js`, which MUST come from the same Go that
compiled the wasm — a mismatched pair fails at load with an opaque error.

**Do not trust the journey PDFs as a statement about today's build** — see F-210
above. The emulator itself is fine; the documents around it cannot currently be
regenerated.

---

## STATE AT HANDOFF (nothing open, nothing in flight)

| repo | branch | head |
| --- | --- | --- |
| fork `bg002h/seedhammer` | `main` | `5bfc118` — pushed, all CI green |
| `bg002h/mnemonic-engrave` | `master` | `b6b4c6a` — pushed, check SATISFIED |
| Rust primary `descriptor-mnemonic` | `main` | `89ab0f62` |

S6b is shipped and closed; see `CONTINUITY_2026-08-18.md`. Worktree `wt-s6b` is
merged and safe to remove. One trailing push-record commit is local and
unpushed, which is the normal tail.

**The hardware flash is DONE** (2026-08-18): `seedhammerii-v0.0.0-bg5bfc118.
signed.uf2`, sha256 `7fe6700b…7281258`, boots on machine power with
`bg5bfc118` on the version line. So the SH2 in front of you is running the S6b
tree, and any on-device check of this feature starts from that image.
