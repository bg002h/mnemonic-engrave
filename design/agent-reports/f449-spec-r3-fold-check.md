# F-449 spec r3 — fold verification (mechanical)

**Verdict: 4 findings partially addressed (I2, I4, I5, I6) / 0 new defects introduced.**

Reviewer: sonnet, mechanical fold-check only. No design merits reviewed.
Diff verified: `git diff a1136d97..9d909a30 -- design/SPEC_liana_unspendable_internal_key.md`
(175 insertions / 34 deletions, confirmed via `git diff --stat`).

Ground: `descriptor-mnemonic 6cbd49d8`, `seedhammer 7b6f2fb`, `mnemonic-engrave`
HEAD `9d909a30` (contains `49964db1` as an ancestor — confirmed via
`git merge-base --is-ancestor`). All three repos' checked-out commits match
what r3 cites. `md` built from descriptor-mnemonic HEAD, `go1.26.7` at
`/scratch/code/shibboleth/.toolchain/go` used for the Go probe.

---

## Checklist — journey r2 findings (2C/6I/4M)

| id | verdict | note |
| --- | --- | --- |
| C1 | ADDRESSED | New §0b states a two-conjunct FIRING PREDICATE (`internal_key_path == null` AND classifier-with-class-2-skipped `== ""`). **Mechanically reproduced against the real compiled `composerLianaOutsideModelClass`** (see Defect-check §3 below): fires on exactly `kofn-recovery`/`tiered-recovery`, not on `simple-timelocked-inheritance`/`plain-multisig`. Gated at §9 stage 4: "§0b's firing predicate exercised on all six `tr` presets, firing on exactly `kofn-recovery` and `tiered-recovery`" |
| C2 | ADDRESSED | New §3e paragraph "THIS RULING BINDS THE GO PORT IDENTICALLY" names all three Go `writeNode` call sites (verified below), the three operator-facing consumers, and the consequence. Scheduled at §9 stage 3 content (explicit site list + "(§3e)") and gated ("§8.5's Go identity leg"); owning-stage table row 5 says "1b (Rust) and 3 (Go) — deliberate" |
| I1 | ADDRESSED | New §0b PLACEMENT: screen sits between `composerShapeFlow` and the first `composerStubFlow`, with a fallback rule (return through `composerStubFlow` + changed-id banner) if placed later. No section still points an implementer at `composer_consent.go:261` as the *current* placement — both remaining occurrences are historical ("r1 pointed... which aims an implementer at the last screen") |
| I2 | **PARTIAL** | New §0b RESET states the reset rule (broader than asked: resets on *any* `composerShapeFlow` re-entry, not only edits that leave the firing set — a safe superset). But I2 also required "*that §8j's edit path carries it*" — `§8j` (a section of the separate `SPEC_wallet_policy_composer.md`) is never referenced anywhere in r3 |
| I3 | ADDRESSED | New §0b DEFAULT ROW (`Initial` = NUMS row) and COPY (names coordinators + "different wallets with different addresses", avoids "unspendable xpub" language) |
| I4 | **PARTIAL** | New §6a row 2 states the exact required ruling text (distinguish BCH-exceeded from unsupported-version; never discard a successful correction). But I4 explicitly required **"§8 gains a vector for it"** — no new §8 item, and this fix is not named in **any** §9 stage's content or gate column (unlike I5, which is at least scheduled — see below) |
| I5 | **PARTIAL** | New §6a row 1 states the required ruling (split `gatherIgnored`, name the version). Scheduled: §9 stage 3 content now reads "§6a's `gatherIgnored` split". But I5 also required "**it must be gated**" — stage 3's *gate* column ("§8 vectors in Go, including §8.4's `ParseChunkHeader`/`Decode` leg and §8.5's Go identity leg") never names this fix |
| I6 | **PARTIAL** | New §9 paragraph ("`me` is downstream...") + new stage 4a address the git-pin/scheduling half exactly, confirming "r2's downstream list named only `mnemonic-toolkit`" (verified true against r2 text). But I6's classification also required **"the single-string arm must not silently skip a decode that feeds a completeness claim"** (the `bundle.rs:371` fail-open defect) — grepped for `bundle.rs`, `key_slots`, `hashlock_kinds`, `keyless_template`, `decode_md1_string(s)`, `SetIncompleteMd`: **zero hits** anywhere in r3. Not addressed at all |
| M1 | ADDRESSED | New §7 bullet: `md1Summary` named as not switching on `KeyPath`, ties the `Policy id:` line to C2. Scheduled: stage 4 content "the print-site arms including `md1Summary`" |
| M2 | ADDRESSED | New §6a row 3 states the `WireVersionMismatch` Display fix. (Not independently scheduled to a stage/gate, but M2's own classification never asked for that — "documentation" only) |
| M3 | ADDRESSED | New "Scope of the engrave refusal" paragraph in §7a item 3 scopes the refusal to an unrecognised internal-key kind, explicitly preserving shipped D3/D4 behavior |
| M4 | ADDRESSED | New §8a "The last mile" states the runbook route (`md encode` → `md descriptor`, checksum `#xnta28tv`), explicitly "not a gate" as M4's classification required |

**Note on I1/I2/I3's gate asymmetry:** stage 4's *content* column names all five
§0b sub-requirements ("predicate, placement, reset, default row and copy"), but
stage 4's *gate* column only names a test for the **predicate**. Placement,
reset, default row and copy have no corresponding §9 gate item. I1 and I3 never
explicitly demanded a gate in their classification text (only I2 did, via §8j),
so only I2 is marked PARTIAL above — but this is the same shape of gap as I4/I5,
just not independently required by I1/I3's own wording.

## Checklist — r2 fold-check's 1 cosmetic defect

| item | verdict | note |
| --- | --- | --- |
| "exactly one owning stage" self-contradiction (item 2 double-assigned) | ADDRESSED | Heading changed to "an owning stage, and no item is left unowned... Two items are deliberately gated twice... that is a double gate, not a duplicate." Table rows for items 2 and 5 now carry explicit "— deliberate" annotations instead of silently duplicating |

---

## New defects introduced by this fold: none found

### 1. File:line citations — 9 new citations found (not 7), all 9 verified accurate

Diffing `+` lines against `\.(rs|go|toml|sh):[0-9]+` and excluding citations
already present in r2 (`gui/composer_consent.go:261` pre-existed) gives **9**
new citations, not the 7 the brief describes the author as having checked:

| citation | claim | verified |
| --- | --- | --- |
| `crates/me-cli/Cargo.toml:26` | `md-codec = "0.42"` | Exact — line 26 is `md-codec = "0.42"`; `Cargo.lock` confirms `version = "0.42.0"`, `source = "registry+...crates.io-index"` |
| `crates/md-codec/src/error.rs:33` | `WireVersionMismatch` Display hardcodes "expected 4" | Exact — line 33 is `#[error("wire-format version mismatch: got {got}, expected 4")]` |
| `crates/md-cli/src/cmd/repair.rs:88-96` | the correction-then-discard sequence, `return Ok(2)` | Exact — 88 is the `match decode_with_correction`, 94 is `return Ok(2)`, 96 is the closing `};` |
| `gui/composer_consent.go:216-229` | consent screen's id + mk1 stub lines | Exact — 216 `FormAwareIdChunks`, 220 `FormAwareStubChunks`, 228 the id/stub append, 229 blank line before the next block |
| `gui/composer_flow.go:97-118` | screen sits between `composerShapeFlow` and first `composerStubFlow` | True but loosely bound: `composerShapeFlow` is called at line 85, first `composerStubFlow` at line 129 (neither name appears in `composer_flow.go` at all under "func composerShapeFlow" — it's defined in `composer_shape.go:636`). Lines 97-118 fall entirely inside that 85-129 gap (`composerTemplateChunksFor` call + surrounding comment), so the citation is accurate but not the tightest possible pointer |
| `md/encode.go:417` | the wire-payload `writeNode` call | Exact — `if err := writeNode(&w, dc.tree, width); err != nil {` |
| `md/template_id.go:53` | `WalletDescriptorTemplateId`'s `writeNode` call | Exact — `if err := writeNode(&w, d.tree, width); err != nil {` |
| `md/walletpolicyid.go:42` | `WalletPolicyId`'s `writeNode` call | Exact — `if err := writeNode(&treeW, dc.tree, width); err != nil {` |
| `md/template_id.go:112` | the r3-corrected `FormAwareStub` dispatcher (journey report had cited `:116`) | **Correction verified right.** Line 112 is `func FormAwareStub(d *descriptor) ([4]byte, error) {`; line 114 calls `WalletPolicyIDStub` (the `if` branch), line 116 calls `WalletDescriptorTemplateIdStub` (the `else` branch, what the journey report cited). Both flavours are downstream of the dispatcher at 112 and both ultimately call the version-less `writeNode` (114→`WalletPolicyId`→`walletpolicyid.go:42`; 116→`WalletDescriptorTemplateId`→`template_id.go:53`). r3's "the exposure is slightly wider than reported" is accurate |

Zero errors among the 9. The brief's "checked 7, corrected one" undercounts by
2 (possibly a different accounting basis, e.g. excluding the Cargo.toml/error.rs
citations) but every citation that exists resolves true, so this is not a
defect — it is a stronger result than claimed, not a weaker one.

### 2. Internal consistency — no contradictions found

- **§0b's firing predicate is stated identically everywhere it appears.** §0
  only forward-references "see §0b, which specifies it completely" (no
  restated rule to drift). §0b states the two-conjunct predicate once. §9
  stage 4's gate restates only the *outcome* ("firing on exactly
  `kofn-recovery` and `tiered-recovery`"), not a second copy of the rule — no
  risk of divergence. Grepped for `"fired only when"` / `"fires only when"` /
  `"5 of 6 presets"`: the only hit is inside a quoted, explicitly-superseded r1
  sentence at line 44. No leftover inverted-rule prose.
- **No section still points an implementer at `composer_consent.go:261` as the
  place to install the predicate.** Both remaining occurrences (lines 35, 80)
  are historical/corrective framing ("r1 pointed twice at... which aims an
  implementer at the last screen before steel").
- **§9's stage table, the owning-stage table, and §8's numbered items 1-9
  agree.** Every item 1-9 appears in the owning-stage table exactly once,
  except items 2 and 5 which deliberately appear twice (both now labelled
  "— deliberate"). The main stage table's gate columns (1b: "1, 2, 5, 6, 7, 9";
  2: "§8.2 and §8.8"; 3: "§8.4's... leg and §8.5's Go identity leg"; 4:
  "§8.3's device leg") match the owning-stage table's assignments exactly for
  every item. §8's own item 5 text ("Rust leg: stage 1b. Go leg: stage 3")
  independently states the same split as the owning-stage table row for item 5.
- **§3e's Go paragraph, §9 stage 3's content, and §8 item 5's text all name the
  identical three Go sites** (`md/encode.go:417`, `md/template_id.go:53`,
  `md/walletpolicyid.go:42`) and the identical section reference (§3e). No
  drift between the three mentions.
- **Stage 4a does not conflict with §9's prose.** The new "`me` is downstream"
  paragraph explicitly says "Stage 4a owns it," and stage 4a's row content
  matches (`md-codec` unpin + §6a's chunked-arm message). Confirmed r3's claim
  "r2's downstream list named only `mnemonic-toolkit`" is literally true by
  reading `git show a1136d97:...` at that location — no `me` paragraph existed
  before this fold.

### 3. §0b's firing predicate, mechanically reproduced against real code (not just re-traced)

Rather than re-trust a manual trace, I built real chunked md1 for four `tr`
presets (`md compose --json` → template → `md encode --path bip48 --key
@i=<xpub> --force-chunked`, distinct xpubs already used in this repo's own
test suite), decoded each with the real `md.PolicyShapeChunks`, called the
**real, compiled** `composerLianaOutsideModelClass` (`gui/composer_consent.go:381`)
for the baseline class, and a byte-for-byte copy with only the class-2 check
deleted (exactly what "class 2 skipped for the new kind" specifies) for the
predicate's conjunct 2. `internal_key_path` (conjunct 1) taken from the
`md compose --json` values already reproduced in the checklist above.

```
$ go test -run TestZZF449R3PredicateReproduction -v ./gui
kofn-recovery                    KeyPath=1(NUMS) baseline-class="NUMS key path" skip2-class=""                conjunct1=true  conjunct2=true  FIRE=true  want=true
tiered-recovery                  KeyPath=1(NUMS) baseline-class="NUMS key path" skip2-class=""                conjunct1=true  conjunct2=true  FIRE=true  want=true
simple-timelocked-inheritance    KeyPath=2(Spendable) baseline-class=""        skip2-class=""                conjunct1=false conjunct2=true  FIRE=false want=false
plain-multisig                   KeyPath=1(NUMS) baseline-class="NUMS key path" skip2-class="no locked path"  conjunct1=true  conjunct2=false FIRE=false want=false
--- PASS: TestZZF449R3PredicateReproduction (0.00s)
```

All four match §0b's table exactly. `hashlock-gated` and `decaying-multisig`
were checked by reading `composer_consent.go:381-468` directly against their
`md compose --json` shapes (not probed): `hashlock-gated` has a hashlock on one
branch → `skip2-class = "a hash lock"` (non-empty, does not fire);
`decaying-multisig` has `after(800000)` on one branch → `skip2-class = "an
absolute lock"` (non-empty, does not fire) — both consistent with §0a's stated
classes and both correctly excluded.

**This is the exact defect class that got through last round** (C1: the rule
read literally was inverted). The r3 fix, read literally and executed against
real code, is not inverted and does not over-fire — reasoned end-to-end, not
re-asserted.

Probe file was written to `seedhammer/gui/zz_f449_r3_probe_test.go`, run, and
**deleted**; `git status` in the fork shows only pre-existing untracked files
(`gui/zz_scratch_probe_test.go`, `gui/zz_scratch_probe2_test.go`, not mine) and
`.claude/worktrees/`.

---

## What was NOT re-derived (per the brief)

The r2 fold-check's already-verified facts (baseline SHAs, `md 0.17.0`, the six
prior-round citations, the blast-radius grep counts, the RUN evidence for the
sortedmulti_a/synthetic-key findings) were taken as settled and not re-run —
this was a fold-vs-findings, new-citation, and predicate-reproduction check on
the r2→r3 diff only.
