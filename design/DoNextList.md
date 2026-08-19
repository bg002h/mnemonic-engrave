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

Not yet reviewed by anyone but its author — an independent pass over the diff
is still owed.

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

### 3. ~~R1 — the `v:` wrapper-chain renderer bug~~ — **DONE 2026-08-18, review in flight**

**Shipped as `descriptor-mnemonic` `285b9fc9`.** `Tag::Verify` now joins the
wrapper-chain dispatch and its standalone arm is removed, so `vj:` renders as
`vj:` rather than the unparseable `v:j:`. Gates: **782 passed / 0 failed**,
clippy clean, default build clean.

**Mutation-verified, not assumed:** restoring the old standalone arm turns BOTH
the new md-cli property test and the md-codec KAT red.

Why it survived at *two* independent sites, both now closed: md-codec's frozen
KAT had `snj:` as its only chain case (no `v`), and md-cli's fifteen
hand-written round-trip tests all put `v:` directly on a `pk`, where the
shorthand path renders it correctly by accident. The md-cli side now has a
**table-driven property** instead of a sixteenth hand-written case, asserting
`parse(t) → render == t` **and that the render re-parses** — the second clause
closes the class for shapes nobody has thought of.

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

### 7. ~~F-210 — the journey generator~~ — **DONE 2026-08-18, review in flight**

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

### 8. Wire the doc gates into CI — **XS**

`scripts/plan-build-gate.sh` and `scripts/plan-cite-check.sh` exist and are
documented as gates, but **nothing in CI invokes them**. CI runs
`test (rust + go)`, two build jobs, and a tag-gated release job — nothing
docs-shaped. So design docs currently have **no automated gate at all**, and the
green check on a docs-only push proves only that untouched code still builds.

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
