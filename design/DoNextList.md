# DoNextList

**Written 2026-08-18.** Everything actionable this session turned up, in the
order I'd do it. Each item says what it is, why it matters, roughly how big, and
where the evidence lives. Nothing here is gated; nothing here has been reviewed.

Companion artifacts, all committed:

| file | what |
| --- | --- |
| `design/agent-reports/pathological-matched-pair-roundtrip.md` | PRIORITY 1, the wsh/tr pair, topology reasoning |
| `design/agent-reports/wallet-policy-recon-*.md` (5) | the recon round |
| `design/agent-reports/wallet-policy-pin-regime-differential.md` | two miniscript pins, measured |
| `design/DRAFT_round_trip_journey_definition.md` | what a round-trip journey is |
| `design/Preliminary-Brainstorm-Arbitrary-Tr-Wsh-Wallet-Backup.md` | the parked feature |

---

## DO NEXT — ordered

### 1. ~~Fix `md encode --from-policy --context tap`~~ — **DONE 2026-08-18**

**Shipped as `descriptor-mnemonic` `cf139508`.** `render_tr_template` in
`md-cli` now renders taptrees with #953's corrected algorithm instead of
`Descriptor::to_string()`. All six policies below compile and re-parse; the
1-of-5 now yields `tr(@4,{{pk(@3),pk(@2)},{pk(@1),pk(@0)}})`. Gates: full
workspace **781 passed / 0 failed**, clippy `-D warnings` clean, default
(feature-off) build clean, `Cargo.lock` unchanged.

**Reviewed and folded.** Opus exec review persisted at `e58f02f7`, fold at
`e2288ddf`: **GREEN — 0 Critical, 0 Important, 5 Nits**, all five addressed.
The reviewer rendered **all 23,714 proper binary taptree shapes (1–11 leaves)**
against an independent oracle plus both BIP-341 depth-128 spines, byte-identical
throughout, and measured non-regression: of 2,056 shapes, only **9 were
parseable under the old path and the new renderer reproduces all 9 exactly**
while repairing the other 2,047.

<details><summary>original entry</summary>

A **shipped command fails on ordinary input**: a plain 1-of-5 taproot wallet
cannot be compiled. Root cause `crates/md-cli/src/compile.rs:95-100`, which
round-trips the compiled descriptor through `desc.to_string()` — the pre-#953
`Display`, correct only for right-spine caterpillar trees.

**The fix is small and needs no dependency change.** The bug is only in the
*tree* formatter; the internal key and each leaf render fine, and
`TapTree::leaves()` is public and yields each leaf **with its depth**. So port
#953's corrected algorithm (explicit child-count stack, close a subtree once it
has emitted both children) into `md-cli` and stop calling `Display` for the `Tr`
case. ~20 lines. No reverse `Descriptor → md AST` converter needed. When
upstream eventually ships #953 the local formatter becomes redundant, never
wrong.

Order: (a) failing test — `thresh(1,pk(@0),…,pk(@4))` under `--context tap`;
(b) the formatter; (c) re-run the six-policy table below; (d) item 2.

**Bounded fix, not a cycle.** `md-cli` only, no codec behaviour change — it
turns a hard error into a correct result. TDD inline, one independent review
over the diff.

```
or(pk(@0),pk(@1))                              OK    tr(@0,pk(@1))
thresh(1,pk(@0),pk(@1),pk(@2))                 OK    tr(@0,{pk(@1),pk(@2)})
or(4@pk(@0),1@or(pk(@1),pk(@2)))               OK    tr(@0,{pk(@1),pk(@2)})
or(1@or(pk(@0),pk(@1)),1@or(pk(@2),pk(@3)))    OK    tr(@0,{pk(@1),{pk(@2),pk(@3)}})
or(pk(@0),or(pk(@1),or(pk(@2),pk(@3))))        FAIL
thresh(1,pk(@0),pk(@1),pk(@2),pk(@3),pk(@4))   FAIL
```

</details>

### 2. ~~Pin the caterpillar rule with a direct test~~ — **DONE 2026-08-18**

Same commit. Two tests: `render_tr_template_pins_every_topology_class` asserts
exact strings for all four leaf-depth classes (single leaf `[0]`, flat pair
`[1,1]`, **decreasing** `[2,2,1]`, **balanced** `[2,2,2,2]`), and
`upstream_display_is_still_broken_delete_local_renderer_when_this_fails` pins
the gap's exact shape — **when that test fails it is good news**, meaning the
pin moved past #953 and the local renderer can be deleted.

The exact-string form earned its keep immediately: it caught that the compiler
promotes a *different* key to the internal position and orders leaves
differently than encode→decode output suggests. My predicted strings were
wrong; the structure was right.

<details><summary>original entry</summary>

> The pre-#953 formatter is correct **exactly when the leaf-depth sequence never
> decreases**. Traced: `[1,2,2] → {A,{B,C}}` ✓, `[2,2,2,2] → {{A,B,C,D}}` ✗,
> `[2,2,1] → {{A,B,C}}` ✗.

This rule is **derived from reading the vendored formatter and matched against
six CLI observations — it has never been tested directly.** It is load-bearing
twice over: for item 1's fix, and as the justification for the device's
depth-≥2 EXPERIMENTAL gate. A per-topology unit test is cheap. Do not let it stay
a hypothesis.

</details>

### 3. ~~R1 — the `v:` wrapper-chain renderer bug~~ — **DONE + REVIEWED + FOLDED 2026-08-18**

**Shipped as `descriptor-mnemonic` `285b9fc9`.** `Tag::Verify` now joins the
wrapper-chain dispatch and its standalone arm is removed, so `vj:` renders as
`vj:` rather than the unparseable `v:j:`. Gates: **782 passed / 0 failed**,
clippy clean, default build clean.

**Review found an Important — against my test, not the fix.** The property
test asserted `render == input` first, making its re-parse clause unreachable
as a failure; my claim that it "closes the class whatever the shape" was false.
Folded in `407cab4b`: the corpus is now **generated** (every chain of length
1-3 over `c s a d j n v`, two bases, five type-position embeddings — 3990
candidates, 93 accepted) and the property is a **fixpoint**, so the re-parse is
genuinely load-bearing. Mutation-verified: restoring the old arm now fails with
*"render emitted a template that does NOT re-parse"* — that clause firing, not
the equality assert. Gates after fold: **784 passed / 0 failed**.

The renderer itself came back clean: the reviewer diffed old-vs-new over **485
encoder-accepted `v`-bearing shapes** — 439 changed, all 439 old outputs
unparseable, and all 485 new renders re-encode to a **byte-identical md1 wire**
(same-tree, not merely same-parses).

Why it survived at *two* independent sites, both now closed: md-codec's frozen
KAT had `snj:` as its only chain case (no `v`), and md-cli's fifteen
hand-written round-trip tests all put `v:` directly on a `pk`, where the
shorthand path renders it correctly by accident — a corpus assembled
shape-by-shape only ever covers the shapes somebody thought of, which is why
the replacement generates its corpus instead of listing it.

**Part 2 of the original item was NOT done, deliberately.** A *runtime* output
contract inside md-codec — returning `Err` when the render doesn't re-parse —
cannot live there as things stand: the renderer emits `@N` **placeholder**
templates, which are not miniscript-parseable at all without md-cli's
`substitute_synthetic`. The property therefore lives at the md-cli layer, where
parsing is possible. Doing it inside md-codec needs a placeholder-aware
validator that does not exist. **Recorded rather than silently skipped.**

<details><summary>original entry</summary>

`crates/md-codec/src/render.rs:150-163` gives `Tag::Verify` its own arm instead of joining
`render_wrapper_chain` (`crates/md-codec/src/render.rs:358`, dispatch at `:217` covers only
`c/s/a/d/j/n`), so `vj:` emits as `v:j:` — **a string rust-miniscript's own
parser rejects**. Emitted by **two shipped binaries** (`md`, and the toolkit via
`crates/mnemonic-toolkit/src/cmd/inspect.rs:325`/`:458`).

Three parts, and the third closes the class:

1. fix the arm;
2. **give the renderer an output contract** — `RenderError` has one variant and
   it describes malformed *input*, so the function literally cannot report that
   it emitted garbage. Add a variant; re-parse and return `Err` when the output
   doesn't round-trip;
3. **replace the snapshot with a round-trip property** over the whole corpus.
   The frozen 14-entry KAT missed `v:` because its only chain case is `snj:` — a
   snapshot blesses whatever the code did, bug included.

Put the re-parse property at test time (free, no dependency); gate any runtime
self-check behind the existing `derive` feature.

</details>

### 4. Decide the default derivation path for arbitrary miniscript — **RULED; implementation still gated**

md has **no** canonical origin for these shapes and says so at encode time
("no canonical default derivation path"). `crates/md-codec/src/canonical_origin.rs:13-76` covers
`pkh→44'`, `wpkh→84'`, `tr` key-path-only→`86'`, `wsh(multi|sortedmulti)→48'/0'/0'/2'`,
`sh(wsh(…))→48'/0'/0'/1'`.

Verified: **BIP-48 defines only `1'` and `2'`** — no taproot, no miniscript.
`48'/…/3'` for taproot multisig is a de facto convention, still unratified.
`m/84'/0'/0'/2'` is **wrong** — under BIP-84 level 4 *is* the change field.

**Candidate A (interop-leaning):** `wsh(<miniscript>) → m/48'/0'/0'/2'`,
`tr(<taptree>) → m/48'/0'/0'/3'`. Reuses the meaning BIP-48 already assigns to
level 4 (script type), and matches what md assigns to `wsh(multi|sortedmulti)`.
Risk: overloading BIP-48 invites **false recognition** — a wallet seeing
`48'/…/2'` assumes plain multisig and may confidently show wrong information.

**Candidate B (unambiguity-leaning, user proposal 2026-08-18):**
`m/27'/0'/0'/2'/8'` — reads as `bg002h`, the operator's bitcointalk handle.

**Its two stated purposes, in the operator's words: to underscore the arbitrary
nature of the path, and to be recognisable to the operator.** Both are real
design arguments and neither is decoration.

*On arbitrariness.* For arbitrary miniscript **there is no standard path**. So
`48'/0'/0'/2'` actively *implies* a standard exists and that the wallet is plain
multisig — it is a claim, and a false one. A visibly arbitrary path signals the
truth: *this path means nothing on its own; you need the descriptor.* Purpose
`27'` is unregistered, so no wallet can falsely recognise it either. This is the
"fails honestly" property extended from machines to humans.

*On recognisability.* The SH2 **displays origin paths**. An operator who can
recognise their own path at a glance has a cheap personal checksum against a
swapped, corrupted or substituted card — on a device whose entire purpose is
letting a human verify things by eye. That is an operational property, not
vanity.

*Three things to settle before adopting it:*

1. ~~It is depth 5~~ — **RESOLVED 2026-08-18 by going to depth 4:
   `m/270'/0'/2'/8'`**, which still reads `bg002h` (`270`→"bg0", then `0`,`2`,
   `8`→"h").

   Measured at `crates/md-cli/src/parse/keys.rs:67-77`: the check reads the
   **xpub's own serialized depth byte** and requires an **exact** match —
   `SingleSig => 3`, `MultiSig => 4`, compared with `!=`, not `>=`. So the
   original depth-5 form could never have bound keys, and depth 4 satisfies
   `MultiSig` directly. Path *values* are not inspected, so `270'` is
   unremarkable to it.

   **No normative md change is needed**, and this item is now **independent of
   item 5** (an earlier note said they had to be decided together — that is
   superseded).

   Residual, for whoever writes the spec: depth follows the **shape**, not the
   script type. `tr()` with script leaves classifies as `MultiSig` (depth 4),
   but key-path-only `tr(@0)` classifies as `SingleSig` (depth 3). Do not write
   "always depth 4".
2. **One path for both `tr` and `wsh` means key reuse across two wallets** —
   and this is in direct tension with the recognisability goal. The same seed
   yields the same pubkeys in both; different scripts give different addresses,
   but spending from both links them on-chain. Three ways out, and the choice is
   the operator's because it trades a real privacy property against the
   mnemonic:

   ~~open~~ — **RULED 2026-08-18 by the operator, revised same day.** Level 4
   **is** the script type, and the constellation defines its values:

   | wallet | path |
   | --- | --- |
   | **`tr`** keys | `m/270028'/0'/0'/0'` |
   | **`wsh`** keys | `m/270028'/0'/0'/1'` |

   `m / 270028' / coin' / account' / script'` — purpose `270028'`, then coin,
   then account, then script type where **`0'` = tr** and **`1'` = wsh**.

   **Why the six-digit purpose replaced `270'`.** Two reasons, and the second is
   the load-bearing one:

   1. `270028` carries the whole handle in a **single component** (`27`→bg,
      `00`, `2`, `8`→h = `bg002h`), which frees levels 2–4 to *mean* something
      instead of spelling something. The script level is now pure semantics.
   2. **It makes collision structurally impossible rather than merely
      unlikely.** BIP-43's convention is *purpose N ↔ BIP N*, so `270'` was
      always nominally claimable by a future BIP-270 (see item 3 below). A
      six-digit purpose sits outside any plausible BIP number permanently.

   It also deliberately does **not** reuse BIP-48's `1'`/`2'` script values, so
   nothing can mistake the layout for BIP-48.

   **Hard ceiling, measured — md1 is stricter than BIP-32.** A path component
   must fit md1's varint single-extension range:

   ```
   m/2147483647'/0'/0'/0'   md: codec error: varint value 2147483647
                                 exceeds single-extension range (max 2^29 - 1)
   ```

   BIP-32 allows hardened indices to 2^31−1; **md1 caps them at 2^29−1 =
   536,870,911**. `270028` is comfortably inside. Any future "make it uglier"
   proposal must stay under ~537 million.

   **Depth check:** `m/270028'/0'/0'` alone is **depth 3**, which would classify
   as `SingleSig`. The fourth (script) level is what makes it depth 4 and
   therefore valid for arbitrary-miniscript shapes.

   This picks the BIP-48-shaped layout over "script type not in the path", and
   the reason is sound: putting the script type *in* the path means an operator
   reading it on the SH2 can tell `tr` from `wsh` **at a glance**, which serves
   the same recognisability goal as the mnemonic itself. Key sets are disjoint
   by construction, so the key-reuse concern is closed.

   With the purpose field carrying the mnemonic, both wallets now read
   identically up to the script level — so the earlier wrinkle (the mnemonic
   attaching only to the `tr` variant) is gone.
3. ~~`27'` unchecked for collisions~~ — **CHECKED 2026-08-18 for `270'`: no
   collision.**

   `bitcoin/bips` `README.mediawiki` has **no BIP 270** — the index jumps from
   199 to 300, so the entire 260–280 range is unassigned (highest assigned:
   451). SLIP reservations are 10001–19999, which does not touch 270. The
   `BIP-270` that surfaces in search results lives in the **moneybutton/bips**
   fork (BSV "Simplified Payment Protocol"), a different ecosystem, and it
   **defines no derivation paths** — so no key-space overlap exists even if one
   counts it.

   Two residual risks, both stated rather than dismissed:
   - BIP-43's convention is *purpose N ↔ BIP N*, so `270'` is nominally
     claimable by a future BIP-270. Low: 260–280 has stayed empty while
     assignment ran past 450.
   - A real collision would need another wallet to use purpose `270'` **and**
     the same coin/account/script levels **and** the same seed. Effectively nil.

**This is normative** — the origin feeds the wire TLV and therefore both wallet
ids, so changing it later moves every id. Rust-primary, needs vectors.

### 5. Make the pathological pair actually round-trip — **decision, then S/M**

Both halves pass structurally; **both fail functionally** because the fixture's
11 xpubs are bip84 **depth 3** and a multisig script context demands depth 4. So
**no address has ever been derived for this wallet by any tool** — the structural
reason the journey has no address-verify step.

1. regenerate the 11 xpubs at bip48 depth 4 from the same committed seeds —
   smallest, but moves the committed `backup-strings.txt` and both wallet ids;
2. widen R4 so a declared origin depth is accepted — leaves the fixture alone,
   changes normative admission;
3. accept template-only forever — then the one journey exercising timelocks, a
   hashlock and unsorted `multi` can never carry a functional assertion.

Unresearched and it decides between 1 and 2: **is md right to demand depth 4?**
That is an external-protocol question nobody has checked.

### 6. R4 then R3 — the conformance-vector export — **M**

R4 (`--path` on `md address`/`md verify`) is a **prerequisite of** R3, not a
sibling: without it the non-canonical shapes are unreachable via `--template`.
Reproduced live this session.

R3 must emit per vector: template string, per-`@N` xpubs + fingerprints,
canonical descriptor string, scriptPubKey hex, `addresses[chain][0..N]`, both
wallet ids, `Md1EncodingId`, md1 chunks. **13 of 15 `test_vectors::MANIFEST`
entries carry `keys: &[]`.**

**Spec constraint, name it explicitly:** the exporter must **not** call
`Descriptor::to_string()`. Both descriptor-string renderers are defective in
different ways and both corruptions land on exactly the shapes the vectors are
for.

### 7. ~~F-210 — the journey generator~~ — **DONE + REVIEWED + FOLDED 2026-08-18** (I-1 spun out as item 9)

**Shipped as `mnemonic-engrave` `b822e4a`.** Both journeys regenerate:

| | non-zero exits, fresh run | committed |
| --- | --- | --- |
| pathological | **3** | 3 |
| operator | **1** | 1 |

All six intermediates now land, including `manifest.json` (11 KB, 25 plates) and
`sysw-public.bin`, which were only ever blocked by the cascade from the
unwritten files. The three pathological refusals are the **designed** ones and
reproduce exactly — including OBSTACLE 1, `mk`'s wire-format version mismatch
on a chunked md1.

Mechanism: a `runcap <outfile> <keep-regex> <cmd…>` helper. The regex is a
**required** argument because `md encode` prints `chunk-set-id: 0x…` on stdout
beside the md1 lines, and `MD1S` slurps the whole file — capturing raw stdout
would have swapped "file missing" for "file subtly wrong". Also: the `MD1S`
read moved from sixteen lines *before* its producer to just after it, and the
stale `me-preview` 0.5.1 sidecar was rebuilt to 0.6.0 the way CI does it.

**Evidence the artifact carried its own defect**, worth keeping: in the
committed pathological transcript, step 7's `mk encode` prints `mk1qpdw8zpq…`
while the `mk decode` on the very next line consumes `mk1qpghz4pq…` — a
different string, from a stale file. They now agree.

**Left undone deliberately:** the committed `.txt` transcripts were NOT
re-recorded. The remaining diff is tool versions, absolute paths (the committed
run came from a scratchpad, not the repo), and the mk1 strings themselves, which
changed between mk 0.12.1 and 0.13.0 — a real behavioural drift, and exactly
what a regenerable journey exists to surface. Deciding which artifact is
canonical is a separate call.

<details><summary>original entry</summary>

Four intermediates have never had a writer in any committed version;
`design/journeys/transcript_pathological.sh:18` reads `out/md1.txt` sixteen lines before the only
command that could produce it. Plus a stale `me-preview` 0.5.1 against `me`
0.6.0.

Not on the critical path for the parked feature — but **a new journey built on
this generator inherits the same defect**, so fix it before writing one.

</details>

### 8. Doc gates — **DECIDED 2026-08-18, and the citation gate is now WIRED (not to CI).**

**Call (operator stand-in, `design/agent-reports/decision-item8-item9.md`):**
`plan-cite-check.sh` runs from **`scripts/push-master.sh`** as a blocking,
**changed-docs-only** step. CI is the wrong home. `plan-build-gate.sh` stays a
fold-time local tool.

**Why not CI — a semantic objection, not a cost one, and it corrects my
earlier reasoning.** I declined the "check the siblings out in CI" option on
speed. The real problem is that CI would resolve citations against sibling
**origin HEAD**, while authors write docs against **local** sibling state,
often in the same session as unpushed sibling work. Push-ordering races would
then red the gate on *correct* docs — and by this repo's own measured words, a
gate that reds on correct work trains the reader to ignore it exactly as fast
as one that is always green. The local roots are, by construction, the state
the doc was written against. (All five siblings are public, so the cheap-clone
version was available and was declined on merit.)

**Two rulings I would not have reached:**
- **`design/agent-reports/` is excluded permanently**, and the exclusion is
  named in the gate's output. Reports are persisted verbatim and never edited,
  so a red gate on a report would demand a *forbidden edit*. A dangling
  citation inside a report is information about the review, not a defect.
- **`plan-build-gate.sh` stays out** because it is not generic — it is
  hardcoded to one plan's files (`src/seal/*.rs`, `tests/seal_cli.rs`), and
  the standing rule is that each repeatedly-folded plan commits its *own*
  extractor. Revisit trigger: a fold ever reaching `master` uncompiled.

**Traded away:** whole-corpus verification (the 201 + 633 corpus converges one
doc at a time as docs are touched, not in a campaign), and coverage for pushes
made outside the ritual — mitigated because a direct push already prints a
bypass line, which this project treats as a failure.

**Status: wired.** Its first execution is the push that introduces it.

<details><summary>the investigation that led here</summary>

**Stopped rather than forced, per the standing "never skip jobs — if making
them run turns CI red, stop and report" rule.** Two independent blockers, the
first structural:

1. **`plan-cite-check.sh` cannot run in CI as written.** It resolves citations
   against **absolute paths to sibling checkouts on a developer machine**. CI
   checks out this repo (into `$GITHUB_WORKSPACE`) plus the `seedhammer`
   submodule at `third_party/seedhammer` — so `/scratch/code/shibboleth/…`
   does not exist, `descriptor-mnemonic` / `mnemonic-toolkit` / `mnemonic-key`
   / `mnemonic-secret` are not checked out at all, and the gate would report
   **100% DANGLING regardless of citation quality**. A gate that is always red
   trains people to ignore it exactly as fast as one that is always green.
2. **The corpus is not clean.** 201 top-level design docs and 633 agent
   reports; a 12-doc sample found **13 dangling citations across 7 of the 12**.
   So even a working gate would be red on day one.

**What a real adoption would need**, and it is a decision rather than a patch:
either check the sibling repos out in the workflow (slow, and pins their
versions), or accept that cross-repo citations are developer-machine-only and
gate just the in-repo ones. Then scope it to **CHANGED docs only** — green by
construction, and it matches the project's own "a fold is authorship and
re-earns the gate" rule better than a whole-corpus sweep would.

**Done in the meantime** (commit `d0005da` and its follow-up): the gate itself
was repaired and is now genuinely useful locally — multi-repo roots,
ambiguity reported instead of silently guessing the wrong repo's file,
repo-qualified citations understood, and this repo's root located via `git`
rather than hardcoded. It then caught real defects in **this session's own
docs**, which is the strongest argument for eventually gating it:

| doc | before | after |
| --- | --- | --- |
| `DoNextList.md` | 0 / 8 | **12 / 12** |
| `DRAFT_round_trip_journey_definition.md` | 0 / 1 | **1 / 1** |
| `Preliminary-Brainstorm-Arbitrary-Tr-Wsh…md` | 7 / 18 | **23 / 23** |
| `pathological-matched-pair-roundtrip.md` | 6 / 7 | **7 / 7** |
| `wallet-policy-pin-regime-differential.md` | 2 / 7 | **10 / 10** |

Most of the gap was bare filenames — `render.rs` or `tag.rs` with a line number and no repo —
which are ambiguous in a constellation. One was a genuine error — a citation to
a citation to line 30 of `mnemonic-toolkit/Cargo.toml`, a file with 29 lines.

<details><summary>original entry</summary>

`scripts/plan-build-gate.sh` and `scripts/plan-cite-check.sh` exist and are
documented as gates, but **nothing in CI invokes them**. CI runs
`test (rust + go)`, two build jobs, and a tag-gated release job — nothing
docs-shaped. So design docs currently have **no automated gate at all**, and the
green check on a docs-only push proves only that untouched code still builds.

---

</details>

</details>

### 9. ~~The engraved `backup-strings.txt` has NO producer~~ — **DONE + REVIEWED + FOLDED 2026-08-18**

**Shipped**: `c6c6943` (fix) → `54e0d2b` (review) → `9992762` (fold). Both
journeys now assemble `out/backup-strings.txt` from their own md1 chunks and
freshly-encoded key cards and engrave *that*; the tracked fixtures are deleted.

**Key material independently confirmed unchanged** — the funds-sensitive half.
The reviewer decoded all **23 cards** (12 operator + 11 pathological) old-vs-new:
field-identical on xpub / fingerprint / path / stub, **0 differences**, and both
md1 sets byte-identical. Steel cut from the old fixtures stays valid.

**Review found 3 Importants, all mine, all folded:**
- **I-2, the serious one:** neither encode loop checked that `mk encode`
  *succeeded*, so a one-character typo in a key header silently dropped that
  cosigner from the **engraved** bundle at exit 0 — demonstrated as 23 plates
  instead of 25, with 10 of 20 pathological captions then naming the **wrong
  master**. I had replaced a *stale* bundle with a silently *short* one. Now
  guarded three ways (exit status, 2-lines-per-card, total count before
  engraving) and mutation-verified.
- **I-1:** a heading rename left the PDF builder on the old name, blanking a
  block as an empty `<pre>` — silent, because `S.get` returns its default. Fixed
  the name *and* the class: all ten lookups now raise and list the available
  headings.
- **I-3:** the builder mixed a committed transcript with a live bundle. Now
  guarded — and the guard caught **my own wrong assumption first**: I required
  every bundle card to appear in the transcript, but the transcript only quotes
  the first. The invariant runs the other way.

**Four more no-producer instances fixed along the way:** both PDF builders read
an `out/transcript.txt` nothing writes (and the *pathological* builder read the
**operator's** transcript); `build_pdf.py` required a never-committed
`keys.json`; both builders wrote the same `out/journey.html` and clobbered each
other; the two journeys shared `out/` and destroyed each other's artifacts.

**Still not regenerable, and the PDFs are deliberately untouched:**
`design/journeys/shots/` has **zero tracked files** — the screenshots these
documents embed exist nowhere, so a rebuild yields HTML with missing-image
placeholders. The **data** layer is fixed; the **screenshot** layer needs the
emulator re-walked. That is the remaining sense in which F-210 holds.

<details><summary>the original finding</summary>

**Call (operator stand-in):** both journeys **generate** `out/backup-strings.txt`
from their own inputs and encodes, `me bundle` engraves *that*, the tracked
`inputs*/backup-strings.txt` fixtures are **deleted**, and transcripts + PDFs are
re-recorded — landing **before** the round-trip audit dispatches.

**Why (a) and not the safer-looking options.** (b) regenerate-and-pin is the
current state with a newer timestamp. (c) a consistency check is **red today**,
so it forces a regeneration anyway *and* still leaves a file nothing produces —
and its one virtue, catching print/engrave divergence, is delivered
**structurally** by (a): same run, same tool, same inputs, so divergence becomes
impossible rather than merely detected. It is also exactly what
`DRAFT_round_trip_journey_definition.md` §5 already codifies — this file is an
intermediate wearing an input's clothes; the journeys' true origins are the
seeds, xpubs and policy.

**What answers the caution about touching engraving fixtures:** *nothing
key-material changes.* The new strings are re-encodings of the same xpubs,
field-identical under `mk decode`, so any steel already cut from the old fixture
stays valid and decodable. The superseded generations stay in git history and
should be named in the commit message.

**Feasibility checked, not assumed.** Operator fixture is 1 md1 + 12×2 mk1 = 25
lines and every cosigner xpub carries its origin in a header comment. The
**pathological key files carry no origin headers** — that is the one named prep
step, with values extractable mechanically via `mk decode` of the current
fixture.

**Still risk-set** (it changes what gets engraved): one implementer, one
independent execution review over the diff — the F-210 pattern.

**Why I have not done it while you slept:** it deletes tracked fixtures and
re-records PDFs, which is a large diff whose correctness is not checkable at a
glance. Decided and ready; yours to start.

<details><summary>the finding</summary>

Raised as **I-1** in `design/agent-reports/f210-journey-capture-exec-review.md`.
It is F-210's own defect class, one layer up, and F-210's fix did not touch it.

`inputs*/backup-strings.txt` is what `me bundle` **engraves**, and **nothing in
the repo produces it**. It is now stale against `mk 0.13.0`, so the journey
prints one card and engraves a different one for the same key:

| journey | step 2 prints | step 4 engraves |
| --- | --- | --- |
| operator | `mk1qpj6vvpq…` | `mk1qpmn4upq…` |
| pathological | `mk1qp30napq…` | `mk1qp0jgzpq…` |

**Both decode to identical fields, so `me bundle` exits 0 and nothing
complains.** Three generations of mk1 are now in play across inputs, committed
transcripts, and today's output.

**Not fixed unattended, deliberately.** Regenerating the fixture changes *what
gets engraved* and moves committed artifacts — that is the operator's call, not
a maintenance edit. Options: (a) generate `backup-strings.txt` from the journey
itself, so the engraved cards are the ones just printed; (b) regenerate it once
and pin it, accepting that it drifts again on the next `mk` release; (c) add a
consistency check that fails when the two disagree, leaving the fixture alone.
**(a) is the only one that removes the defect class rather than re-detecting
it.**

Same review also noted the committed **operator** transcript carries a second
instance of the stale-file evidence (`mk encode` prints `mk1qpf7f8pq…`,
`mk inspect` consumes `mk1qpmn4up…`) — my F-210 commit cited only the
pathological one.

</details>

---

## DECISIONS NEEDED — blocking, cheap to give

1. **Round-trip definition, 3 open items** (`DRAFT_round_trip_journey_definition.md` §8):
   does the audit inventory journeys that *exist* or enumerate those that
   *should* exist and mark each present/absent (the latter finds holes, since a
   per-repo sweep is blind to gaps *between* repos); may a generative journey
   start from a fixed test seed; are passphrase/network/account dimensions or
   separate journeys.
2. **Constellation audit fanout** — ~7–8 read-only inventory agents by repo, then
   synthesis by path single-author. **Not consented yet**, and should not start
   before #1 is ruled or the agents measure eight different things.
3. **Derivation path** (item 4) and **fixture keys** (item 5).
4. **R2** (`l:`/`u:` normalization) and **R5** (`sortedmulti_a` — a wire tag that
   renders but can neither be encoded by the CLI nor derived): rule inside the
   next cycle, or file out?

---

## BLOCKED / KNOWN-BAD — do not re-derive

- **`md-cli` does not compile against `ff4732e` or `95fdd1c5`** — two PR #915
  breaks (`WshInner` unresolved, `ShInner::SortedMulti` missing) at
  `crates/md-cli/src/parse/template.rs:945` and `:931`. So **any** plan that
  starts "bump the miniscript pin" pays this first. The recorded spike calling
  the bump "build-clean" was true for the *toolkit* only.
- **PR #953 is merged but in no release through 13.1.0.** The device's depth-≥2
  EXPERIMENTAL gate **stays**; its premise was re-confirmed, not weakened.
- **The two pin regimes produce no measured behavioural difference** — 461/461
  identical across 13.0.0, `95fdd1c5`, `ff4732e`. But it is a **weak green**:
  13 of 15 vectors carry no keys, so keyed derivation is barely exercised. R3
  is what turns it strong.

## PARKED

**The arbitrary `tr()`/`wsh()` Wallet Policy cycle** —
`Preliminary-Brainstorm-Arbitrary-Tr-Wsh-Wallet-Backup.md`. Six user decisions
already made, five open questions, no gates passed. Resume after the
constellation audit and round-trip journey work.

## LOOSE ENDS

- `tagPkh` (0x04, descriptor `pkh()`) and `tagPkH` (0x0B, miniscript `pk_h`)
  differ only in the case of one letter, in a funds-critical codec.
- A **balanced** `tr` variant of the pathological wallet — a deliberately
  defect-seeking fixture — does not exist. Per item 1 it cannot currently be
  produced via `--from-policy`; the template route should still reach it.
- `md inspect` prints `wallet-policy-mode: true` alongside "keyless descriptor
  template (no keys)" in some cases — contradictory advisory, worth fixing
  before that text mirrors onto a device.
