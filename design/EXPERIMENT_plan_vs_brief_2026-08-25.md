# Experiment — a 2,060-line reviewed plan versus a one-paragraph brief

**Run 2026-08-25 at the operator's direction.** Two agents implemented the same
target in isolated worktrees. **Arm A** (opus) had the R0-GREEN spec, the P1
implementation plan (2,062 lines, ten review rounds) and all sixteen agent
reports. **Arm B** (fable) had the operator's paragraph and a 397-line extract of
the measured QR findings, with the entire design surface **removed from its
worktree** so the isolation was structural rather than a promise.

Target, verbatim and identical for both arms: *"modify mnemonic-engrave and
seedhammer firmware (and mt, if necessary) to support mt transaction data similar
to how md and mk strings are handled … engraving transactions via on-boot payload
detection program and carousel program … as many mt strings that can fit on a
plate … users should have the option of QR encoding, and this should encode raw
signed bitcoin transaction according to our qr encoding recon findings."*

## What each produced — measured, not reported

| | ARM A (plan-driven) | ARM B (brief-driven) |
| --- | --- | --- |
| `mnemonic-engrave` | 13 files, **4,472** insertions | 14 files, **1,315** |
| `seedhammer` | 22 files, **3,484** | 24 files, **2,281** |
| `mnemonic-transaction` | 3 files, **872** | **untouched** |
| **total** | **~8,828** | **~3,596** |
| Rust suite | **344 passed** (verified) | **328 passed** (verified) |
| Go | 51 packages ok | 927 gui tests + all packages |
| clippy | clean | clean (verified, 0 warnings) |

**Arm A wrote 2.5× the code.** It also changed `mt`; arm B judged the brief's
*"if necessary"* not met and left it alone.

## The wire formats are unrelated, which is the evidence of independence

| | A | B |
| --- | --- | --- |
| record | `MTX1` magic, version, form, **txid + wtxid**, flags byte, `body_len` — 75-byte framing | `tx:<hex>`, classified by **structural parse** |
| `MTX1` occurrences in B | — | **0** |
| `wtxid` occurrences | **66** | **0** |

B reached none of the plan's framing. It also reached the anti-smuggling
requirement **independently**, reasoning from the sibling rather than from review:

> *an `mt1` chunk set reassembles to bytes with **no semantic decoder of its
> own** — any complete set of BCH-valid strings "decodes", which is exactly the
> entropy-smuggling channel `[mdmk-decode]` closes for md/mk*

That is the channel the plan calls C3. **It took the plan until round 6 to bind
that requirement to a wiring site (W15); B derived it in one pass from `md`/`mk`.**

## THE DECISIVE TEST — witness stripping, run against both binaries

A signature-stripped segwit transaction **has the same txid** as the original.
Built from the corpus vector: 222 B → **113 B, txid identical**. Fed to both:

| input | ARM A | ARM B |
| --- | --- | --- |
| honest 222-byte record | packs, exit 0 | packs, exit 0 |
| stripped body, **real** wtxid carried | **exit 4 — `fails rule wtxid`** | n/a (no wtxid) |
| stripped body, **self-consistent** wtxid | **packs, exit 0** | n/a |
| stripped raw transaction | (rejected: wrong framing) | **packs at exit 0**, and `show` reports **the honest transaction's txid** |

**A catches one of the two stripping cases. B catches neither.** The case A
catches is the honest-bug/interop class — a Go port that strips while carrying
correct identifiers. The case A misses is the deliberate one, and the plan is
explicit that no carried field can catch it (*"nothing in the record can tell it
from an honest witness-free transaction"*).

**This is the clearest thing the plan bought**, and it cost a full review round
(r2-C1) and 32 bytes of framing to find.

## THE OTHER DECISIVE TEST — did anyone decode what they produced

| | A | B |
| --- | --- | --- |
| Structured Append encoder | 2 files | 1 file |
| test that **decodes** a produced symbol | **0** | **1 — `TestZxingMergesTheSetBackToTheTransaction`, PASS in 0.08s, verified** |

Arm A's own log: *"**I never decoded a symbol I produced** — the SA work is
encode-side only, and §4.2c's two gates are both unsatisfied."*

Arm B ran an independent mainstream decoder (`ZXingReader`) over its own symbol
set and got the exact transaction bytes back, in reverse scan order. It also
vendored a **1,027-byte payload packed by the Rust binary** into the firmware
repo and decoded it from Go — the cross-language seam that catches the F-212
class, where both sides pass their own tests while disagreeing.

**On the requirement the brief actually stated — "encode raw signed bitcoin
transaction" — arm B proved it works and arm A did not.**

## Arm A's own verdict on its guidance

> *"The plan earned its size, **unevenly**. ~600 of its 2,060 lines prevented at
> least three defects I would have shipped … The other ~1,400 lines are a
> review-process record inside a build document; **I read them and acted on
> none**."*

The three it credits: the **DISPLAY-order txid** (internal order would have made
R15 refuse every byte-perfect record), the **second identifier** (above), and the
**prefix-without-branch**. All three produce artifacts that pass every check and
are unrecoverable in steel.

Arm A also found **one gap the plan does not cover** — the Go section cap, which
would have made the device refuse containers the host now emits — and one rule
(§4.3 rule 2) it judged only half-implementable.

## What this says

**The plan's value was real, concentrated, and about 30% of its bulk.** Every
defect it prevented shares a shape: *an artifact that passes every check and is
worthless in metal*. That is exactly the class review is for and testing is not.

**The 70% that is fold history bought nothing an implementer used.** Arm A read
it and acted on none of it. The archaeology pass had already reached the same
conclusion from the other direction.

**And the brief-driven arm was better where it was told what to prove.** It got
the QR round-trip and the cross-language seam that the plan-driven arm, holding a
document that specifies both, did not build. Guidance is not the same as
pressure to demonstrate.

**Neither arm is shippable as-is.** A has no decode proof and four gates resting
on unrun hardware; B has no wtxid, a ~3.5 KB delivery ceiling, and no UI walk.
