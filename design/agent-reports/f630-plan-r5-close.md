# R5 — did the r4 fold land, and does the end state close the class NOW?

**Reviewer:** independent of the fold's author.
**Artifact:** `design/IMPLEMENTATION_PLAN_F630_xpub_header_sync.md` at mnemonic-engrave `7b032747`.
**Trees:** fork `/scratch/code/shibboleth/seedhammer` main `95716e97`; primary
`/scratch/code/shibboleth/descriptor-mnemonic` main `b2c5d693`. Go 1.26.7.
**Method:** Question B was answered by EXECUTION. The plan's end state was built in a
throwaway `git worktree` of the fork at `95716e97` — T2's record + phrase pins, D6's
anti-drift rework, D5b's card fix, the record reads routed through pin-preferring loaders
in BOTH packages, T3's three edits with the **real widened re-vendor** against `b2c5d693`,
and T1's gate implementing D1 + **D1′** + **D1″** + D2a/D2b/D2c + D3 + D5g **including its
membership assertion**. **Twenty-one** mutations were then applied — r4's ten plus eleven
new ones aimed at D1′ — each asserted to have APPLIED before its verdict was taken, each
re-pinned the way the vendor script re-pins, and each run against
`go test ./md/ ./sysw/ ./mk/ ./address/ ./bip380/` plus the FULL `./gui/` package via
`scripts/gui-shard-test.sh ./gui/ 24` (1374 tests, partition asserted exhaustive, ~21s).

**A methodological correction to r4's battery, applied here.** r4's descriptor mutations
changed the body without recomputing the BIP-380 checksum. At the end state D1″ exists, so
every such mutation would report CAUGHT *by the checksum clause* and the other clauses would
never be exercised. The primary always emits a correct checksum, so **every body mutation
below was re-checksummed** (BIP-380 descriptor checksum re-implemented and validated against
90 of 90 untouched chain descriptors before use). M4 is the sole deliberate exception.

**Counts: 0 Critical, 2 Important, 5 Minor, 1 Nit.**
**A-verdict tally: 7 FIXED / 0 PARTIAL / 0 NOT-FIXED (C-1, C-2, I-1, M-1, M-2, M-3, N-1).**
**B: YES — the F-630 import class is closed. Three things still slip through, none of them
that class; one of the three (I-2) is a specification gap that must be closed in the text.**

---

# A. Did the r4 fold land?

| r4 finding | verdict | the plan text that settles it, and the measurement |
| --- | --- | --- |
| **C-1** the residue section claims a coverage that does not exist (`TestEveryKeyedVectorReachesAnAddress` "catches" suffix and key order) | **FIXED** | The claim is retracted by name — "*false, since that test's record type carries no `Descriptor` field at all (r4 C-1)*" — and the gap is closed rather than merely recorded: D1′ catches both. Re-measured: `policyAddrVector` (`gui/policy_address_test.go:18-24`) still carries only `Name`/`Template`/`Chains{Addresses}`. Suffix (M2) and key order (M3) both CAUGHT at the end state. |
| **C-2** the residue list omits the quorum and the script type, both funds-relevant, both silent | **FIXED** | Both are now in D1′'s mutation table and both are closed, not recorded. Measured: M6 `multi(2,…)→multi(3,…)` CAUGHT; M7 `wsh(…)→sh(wsh(…))` CAUGHT. |
| **I-1** D5g's tier is selected by file existence and nothing pins its membership | **FIXED** | D5g: "*the gate enumerates `md/testdata/forkbuilt/*.conformance.json` and **fails unless that set is exactly the three F-529 names***". Reproduced r4's escape hatch end to end (M17): import the 0.44.0 defect for `keyed_wsh_multi_2of3` → gate reds 45/1; copy that defective record into `forkbuilt/` → **the membership assertion fires** (`got [4 names] want [3 names]`), where r4 measured the whole 1374-test suite going green. |
| **M-1** the checksum gap is cheaper to close than the plan implies | **FIXED** | D1″ added. The export compiles (`bip380` does not import `md`; `go build ./bip380/` ok) and the clause fires: M4 `#uruj67xy → #qqqqqqqq` → `chain 0: BIP-380 checksum "qqqqqqqq" is not valid for the descriptor body`. |
| **M-2** D5e's synthetic known-bad must be a `*_test.go` written into live package dirs | **FIXED by deletion** | D5c cuts the scanner. **Checked that nothing still depends on it:** `D5d`/`D5e`/`D5f` have **0** occurrences in the plan; the 4 remaining "scanner" mentions and the single "token scan" / "known-bad" mentions are all inside D5c's own deletion rationale (lines 257-267, 324) or reference it (T2.3 line 392, "*there is no scanner to satisfy (D5c)*"); `TestVectorRecordBoundaryHoldsInBothPackages` — the scanner's self-test, named by r3 M-4 — has **0** occurrences and is correctly absent from T2's gate regex. D5b survives re-purposed as the *pairing rule*, carried in a comment at the top of each fixtures file. Nothing dangles. |
| **M-3** T1's acceptance count changes once T2 lands, and the plan does not say so | **FIXED** | T1 now records it: "*After T2 the same run against the stale corpus reads 5 pass / 41 fail*". Measured at the end state, all three of the plan's numbers reproduce exactly: T1 (stale corpus, no pins) **2 pass / 44 fail**; post-T2 stale **5 pass / 41 fail**; end state **46 pass / 0 fail**. (See M-1 below for the two implementation choices those numbers silently depend on.) |
| **N-1** D6's recorded template ids are 8-hex prefixes, not the ids | **FIXED** | D6 now quotes all 32 hex characters. Re-measured against `b2c5d693`: `keyed_tr_multi_a 8c1c05666abdf6b29df9c2056c0cb49e`, `keyed_tr_sortedmulti_a 09903620dbcf4e059300f23391079052`, `keyed_wsh_timelock_hashlock 71ff3b7424b35d410c16b6d992aa437e` — all three agree byte for byte. |

## Independently re-measured while building the end state

- **T3's numbers.** The widened script printed `vendored 246 files, 50 vectors, primary b2c5d6938432`. `composeVectorNames` counted **with `ast`** (per the plan's own instruction): **50**. The naive `grep` the plan warns about returned 34. Diff expectation held exactly: **44** conformance records changed, of which **3** wholesale and **41** descriptor-only, **88** chain-descriptor lines moved, **0** files added or removed.
- **D6 was load-bearing.** Before the rework, `go test ./md/` at real T2+T3 state failed on both existing pins (`pin has 4 chunks, vendored has 6`) — the failure D6 predicts, reproduced.
- **D5a's three sites were red.** Before routing, `TestEveryKeyedVectorReachesAnAddress` (3 subtests), `TestTaprootScriptPathMatchesRust` (2) and `TestWshWitnessScriptHashesToRustsAddress` all failed; all green after routing.
- **T4's surface.** `go vet ./...` exits 1 with exactly **10** diagnostics, every one `testing.ArtifactDir requires go1.26 or later`, none other. `gofmt -l .` returns exactly the five-file baseline. Both held at the end state.

---

# B. Does the end state close the class? — **YES.**

## The class itself is closed

**M8 is the F-630 shape, exactly**, and r4 measured it SILENT. At this end state it is CAUGHT.

```
M8 APPLIED: 92 of 92 chain descriptors now carry a shifted suffix
  sample chain0: …T3sWa2bPW/7/*))#8nyxjvd4
  address0 UNCHANGED: bc1q8z8kvwnpeqy79hfkggrtfm26hkgq2tu708a86tcwtm5gy5wrc85s99fe24
F630 gate: 46 vectors, PASS 3, FAIL 43
```

Every one of r4's seven silent shapes is now caught, and both of its catches still catch.

## The battery — 21 mutations, CAUGHT/SILENT and by which clause

| # | mutation (re-checksummed unless noted) | verdict | clause |
| --- | --- | --- | --- |
| **M1** | bracket account re-point `/48'/0'/1'/2'` → `/48'/0'/9'/2'`, xpub untouched | **CAUGHT** | D3 *and* D1′ |
| **M2** | derivation suffix `/0/*` → `/7/*`, one chain of one vector | **CAUGHT** | D1′ |
| **M3** | swap two `multi()` key positions | **CAUGHT** | D1′ |
| **M4** | wrong BIP-380 checksum `#uruj67xy` → `#qqqqqqqq` (*not* re-checksummed, by design) | **CAUGHT** | D1″ |
| **M5** | `chains["0"].descriptor` ↔ `chains["1"].descriptor` (both checksums asserted still valid) | **CAUGHT** | D1′ |
| **M6** | quorum `multi(2,…)` → `multi(3,…)` | **CAUGHT** | D1′ |
| **M7** | script type `wsh(…)` → `sh(wsh(…))` | **CAUGHT** | D1′ |
| **M8** | the corpus-wide suffix regression, **92 of 92** chain descriptors, addresses and ids untouched | **CAUGHT** | D1′ (43 of 46 red) |
| **M9** | drop one key from a rendered `multi(2,…)` | **CAUGHT** | D1′ *and* D2b ("Go slot @2 carries an xpub that appears in NO chain descriptor") |
| **M10** | import the original 0.44.0 header defect (the stale `95716e97` record; its checksum is self-consistent) | **CAUGHT** | D1 (`depth 0, origin has 4 components`). D1′ silent, as the plan's table claims. |
| **M11** | `template` mutated ALONE (`multi(2,` → `multi(3,`) | **CAUGHT by D1′** — and **SILENT across the entire pre-existing surface** (see I-1) | D1′ only |
| **M12** | M8's suffix shift applied to **both** `descriptor` (92/92) and `template` (46/46), card unmoved | **SILENT** | — (but unreachable from the primary; see I-1) |
| **M13** | whole-policy substitution: card, descriptor, ids **and** addresses all moved to `sortedmulti`; only `template` left saying `multi` | **CAUGHT** | **D1′ alone.** Every other clause green — the ids and addresses agree because the card moved with them. |
| **M14** | an **xprv** in place of the xpub, forged to carry the rendered xpub's exact header bytes | **SILENT** | — (see M-3) |
| **M15** | hardening `'` → `h` inside the origin bracket | **SILENT** under a normalising D1′; **CAUGHT** under a strict one | D1′ (see M-2) |
| **M16** | every bracket fingerprint → `deadbeef`, corpus-wide (92/92) | **CAUGHT** | D2c (`bracket fingerprint deadbeef, Go says 73c5da0a`). D1′ blind by design. |
| **M17** | r4 I-1's escape hatch: import a header regression, then **pin** the regressed record | **CAUGHT** | D5g membership |
| **M18** | a *third* staleness inside a pinned record (depth 0 → depth 7) | **CAUGHT** | D5g legacy arm (`neither the legacy shape nor correct`) |
| **M19** | suffix regression **inside a pinned record** | **CAUGHT** with D1′ on the pinned tier; **SILENT** under D5g as written | see I-2 |
| **M20** | a pinned record that becomes **correct** (the primary re-shipped it) | **CAUGHT** | D5g legacy arm — the second direction the plan promises |
| **M21** | move the origin **out of the bracket into the suffix** — the reduction is byte-identical to the template | **CAUGHT** | D1 *and* D3. D1′ blind, which is exactly why both are required. |

## Attacking D1′ — the four probes

### 1. Can a descriptor be wrong in a way that still reduces to the template?

The reduction substitutes only `[fp/origin]xkey` → `@N/origin` and compares **everything else
verbatim** — mean 495 rendered characters per chain descriptor, max 1909, including whole
miniscript fragments (`wsh(thresh(2,pk(@0/…),s:pk(@1/…),s:pk(@2/…)))`). That makes its blind
spots enumerable, and I enumerated them by construction:

1. **the origin bracket's fingerprint** — discarded by the reduction; **D2c** covers it (M16).
2. **the xkey's header bytes** — discarded (slots are matched on the 65 bytes); **D1** covers it (M10, M21).
3. **the xkey's TYPE** — `xprv` vs `xpub`. **NOT covered by anything. M14 is SILENT** (see M-3).
4. **the checksum** — stripped; **D1″** covers it (M4).
5. **hardening spelling** — blind if D1′ normalises, caught if it does not; unspecified (M-2).
6. **slot identity when two slots carry identical key material** — "the slot whose 65 bytes match"
   is not single-valued then. Measured: **0 of 46** vendored records and **0 of 46** keyed records
   in the primary at `b2c5d693` have two slots sharing an xpub, so it is latent, not live (M-5).

I also constructed the case the probe names — *an origin containing a `/`* — as M21: the origin
moved out of the bracket and into the suffix, which reduces to a string **byte-identical to the
template** while describing a completely different key. It is caught by D1 and D3, which is the
plan's "neither alone closes the class" claim demonstrated rather than asserted.

### 2. Does D1′ hold for the pinned tier?

**The plan does not say, and the omission costs coverage — see I-2.** D5g enumerates the pinned
tier's clauses as "*the legacy header shape … plus full D2a, D2b and D2c*"; D1′ and D1″ are absent
from an enumeration whose form reads as exhaustive. Measured both ways:

- **D1′ applied to every record, pinned included: 46 of 46 pass.** The three pinned records
  satisfy it — including `tr(@0/48'/0'/0'/2'/<0;1>/*,multi_a(2,@0/…,@1/…))`, whose slot `@0`
  appears twice. So there is no reason to relax it.
- **D1′ restricted to the vendored tier (the plan as written): M19 is SILENT** — a suffix
  regression in a pinned record passes the whole 1374-test suite.

### 3. Is `template` itself trustworthy?

**The plan's stated reason is false; the true reason is different and stronger. See I-1.**

Executed: with the F-630 gate excluded, M11 (`template` mutated alone) leaves `./md/ ./sysw/ ./mk/
./address/ ./bip380/` **and all 1374 `./gui/` tests** green. Nothing in the fork asserts
`rec.template` — the only two readers are the struct field in `md/conformance_keyed_test.go` and
`gui/policy_address_test.go:218-219`, where it appears **inside a `t.Fatalf` message**.
`wallet_descriptor_template_id` is an independent JSON field; the existing assertion compares a
Go-computed id **from the card** against **that field**, never against the template string.

But the *primary* settles it favourably. `crates/md-cli/src/cmd/vectors.rs:142` writes
`json!(v.template)` — the vector's hand-authored `&'static str` (`crates/md-codec/src/test_vectors.rs:108`)
— while the card, both ids, the addresses **and** the rendered `descriptor` are all derived from
`parse_template(v.template, …)` at `vectors.rs:54-101`. `template` is the INPUT; everything else
is OUTPUT. So D1′ compares the primary's source of truth against the primary's rendering, which
is the right shape. **M13 proves what that buys:** a whole-policy substitution in which the card,
the descriptor, both ids and the addresses all move consistently is caught **by D1′ and nothing
else**. And **M12** — the only construction in which `template` and `descriptor` move together —
requires the card to stay behind, a corpus state the primary's generator cannot emit.

### 4. D1″'s export

`func ValidChecksum(s, c string) bool { return validChecksum(s, c) }` in `bip380/checksum.go`
compiles; `bip380` imports no `md`, so no cycle. The clause fires (M4). Side effect the plan's
T4 does not account for: this is a **non-test Go file**. Measured — firmware size is byte-identical
with and without the export (`1622896 code / 32188 data / 31148 bss | 1655084 flash / 63336 ram`,
same tree, one file differing), so T4's check does not false-alarm; its stated *reason* is wrong
(M-4).

## What still slips through

| shape | reachable from the primary? | disposition |
| --- | --- | --- |
| an **xprv** where an xpub belongs (M14) | **No** — the md1 wire carries only 65 bytes of public material, so `to_miniscript` cannot render a private key | **M-3**, secret-handling class → non-gating follow-up per the 2026-08-27 ruling |
| **hardening spelling** `'`→`h` (M15) | Yes, and cosmetically harmless (BIP-380 treats them as equivalent) | **M-2**, closed for free by specifying a strict compare |
| a body regression **inside a pinned record** (M19) | No — the gate reads the pin, so the primary cannot reach it; a fork-side edit can | **I-2**, one clause |

None of these is the F-630 class. **B is YES.**

---

## I-1 (Important) — D1′'s stated justification names a binding that does not exist

Plan, D1′:

> "It is also cross-language rather than self-consistent, because `template` is already bound to
> the card by the existing `wallet_descriptor_template_id` assertion."

**Measured false.** `wallet_descriptor_template_id` is a separate JSON field. The assertion in
`md/conformance_keyed_test.go` computes the id from the **card** and compares it to that **field**;
it never touches `rec.template`. Mutating `template` alone (M11) leaves
`./md/ ./sysw/ ./mk/ ./address/ ./bip380/` and **all 1374 `./gui/` tests** green with the F-630
gate excluded. The fork's only other reader of `template` is a `t.Fatalf` message
(`gui/policy_address_test.go:218-219`).

**State the wrong outcome.** This sentence is the whole answer to "is D1′ cross-language or is it
comparing two fields of one vendored file?" — the question a reviewer of the *next* cycle will ask,
and the question that decides whether D1′ survives a future proportionality cut. A reader who
checks the claim finds it false, and the honest conclusion from that check is "D1′ is
self-consistency" — which would retire the one clause that catches M13, M8, M2, M3, M5, M6, M7
and M9.

**Remedy, and it makes the plan stronger, not weaker.** Replace the clause with the measured truth:
`template` is the primary's hand-authored `&'static str` input literal
(`crates/md-codec/src/test_vectors.rs:108`), and the card, both ids, the addresses and the rendered
`descriptor` are **all derived from it** by `parse_template` (`crates/md-cli/src/cmd/vectors.rs:54-101`,
`142`). D1′ therefore compares the primary's source of truth against the primary's rendering — an
input↔output check, not a field↔field one. Cite M13 as the evidence: card, descriptor, ids and
addresses all moved consistently and **only D1′ noticed**.

## I-2 (Important) — D5g's pinned-tier enumeration omits D1′, and reading it as written costs the coverage

Plan, D5g:

> "- **pinned tier** — the legacy header shape is asserted *exactly* (depth 0, child 0, parent fp 0
>   under a non-empty origin), plus full D2a, D2b and D2c."

That list names three of the seven clauses and omits D1′ and D1″. It is written in the form of an
exhaustive enumeration, immediately after a bullet that says the vendored tier gets "full D1" — so
an implementer reads it as "the pinned tier gets the legacy header arm and the D2 family, and
nothing else".

**Reproduced, both ways.** Built the plan-as-written variant (D1′ skipped on the pinned tier) and
re-ran M19 — a `/0/*` → `/7/*` suffix regression inside `md/testdata/forkbuilt/keyed_tr_multi_a.conformance.json`:

```
PLAN-AS-WRITTEN variant built (D1' skipped on the pinned tier)
    F630 gate: 46 vectors, PASS 46, FAIL 0        ← clean end state, so the variant is not vacuous
M19 RE-APPLIED to 2 pinned chain descriptors
VERDICT[M19-planaswritten]: SILENT (whole suite green)
```

With D1′ applied to every record it is CAUGHT (`chain 0: the descriptor does NOT reduce to its
template`), **and all 46 still pass** — the three pinned records satisfy D1′ today, including
`keyed_tr_multi_a`'s `tr(@0/…,multi_a(2,@0/…,@1/…))` with its repeated `@0`.

**State the wrong outcome.** The pinned tier is not incidental: it is the three fork-maintained
fixtures that exist *because* the primary dropped the policies, so the fork is their only
custodian and no upstream re-vendor will ever correct a defect in them. Under the plan as written,
their rendered descriptors are checked for key material, bracket fingerprint, bracket path and a
legacy header shape — and for **nothing at all** about the script, the quorum, the operand order,
the chain suffix or the checksum. A fork-side edit to a pin, or a mis-copied pin during T2, is
invisible.

**Remedy, one clause.** Say that the pinned tier differs from the vendored tier in **exactly one**
respect — the header arm — and that D1′, D1″, D2a, D2b, D2c and D3 apply unchanged. Record the
measurement (46 of 46 pass with D1′ on every record) so the implementer does not have to discover
it.

## M-1 (Minor) — T1's acceptance is observable only under two choices the plan leaves open

T1's "*44 of 46 fail, 2 pass*" and M-3's "*5 pass / 41 fail*" both reproduce exactly — but each
depends on a choice D5g does not make.

**(a) The membership assertion's severity.** At genuine T1 state (stale corpus, no pinned records)
the pinned set is empty, so the membership assertion fires. As a `t.Fatalf` the gate aborts before
examining any vector and the implementer sees this and nothing else:

```
    the PINNED TIER's membership has drifted:
      got  []
      want [keyed_tr_multi_a keyed_tr_sortedmulti_a keyed_wsh_timelock_hashlock]
--- FAIL: TestF630DescriptorGate (0.00s)
```

T1's stated acceptance is then unobservable, and the shortest path to observing it is to weaken the
assertion — undoing r4 I-1's remedy. As a `t.Errorf` (which is the shape D5f used, and which D5g
explicitly invokes: "*the same shape as the count assertion the cut scanner's exemption marker used
to carry*") the vectors still run and the count prints. Say `t.Errorf`.

**(b) The tier SELECTOR.** D5g adds a membership *assertion* but does not say whether the *selector*
remains file-existence. It matters, measured at T1 state:

| selector | T1 (stale corpus, no pinned records) |
| --- | --- |
| file existence (`forkbuilt/<name>.conformance.json` exists) — the strict reading | **2 pass / 44 fail** ✓ the plan's number |
| the asserted name list | **5 pass / 41 fail** ✗ |

Both agree at the end state (the assertion forces them to), so only T1's acceptance can tell them
apart — which is precisely the number an implementer is told to check. One sentence: the selector
stays file-existence; the name list is the assertion, not the selector.

## M-2 (Minor) — D1′'s hardening normalisation is unspecified, and the two readings differ

D3 says "*every path comparison normalises before comparing*". D1′ is a whole-string compare, not a
path comparison, and the plan does not say which side of that line it falls on. Measured, both
variants are **exact on 46 of 46**:

| D1′ variant | clean end state | M15 (`48'` → `48h` in the bracket) |
| --- | --- | --- |
| normalising | 46/46 pass | **SILENT** |
| strict (verbatim compare) | 46/46 pass | **CAUGHT** |

Since strict costs nothing and catches strictly more, say so: "D1′ compares verbatim — hardening is
**not** normalised here, unlike D3, because both the record's `template` and its `descriptor` spell
hardening with `'` and a change to that spelling is itself a rendering change worth seeing."

## M-3 (Minor, non-gating) — an xprv passes every clause of the gate

**Severity note: this is a secret-handling defect and is therefore never Critical and never
Important (operator ruling 2026-08-27). Logged for future optimisation; it does not hold the gate.**

Forged an `xprv` carrying the rendered xpub's exact header bytes (`0488ade4` ‖ depth 4 ‖ parent fp 0
‖ child `80000002` ‖ the same chain code ‖ `00`‖privkey), substituted it for slot 0's key in both
chain descriptors of `keyed_wsh_multi_2of3`, and re-checksummed:

```
M14 APPLIED to 2 chain descriptors: slot 0's xpub replaced by an XPRV carrying the same 65 bytes
   wsh(multi(2,[73c5da0a/48'/0'/0'/2']xprv9zYYzzUWBwGGkPAK9JKPZ5xysKvvH1Lpd4YR9VWjWuJhPNBGkWJ…
VERDICT[M14]: SILENT (whole suite green)
```

Every clause passes: D1 reads the same depth/child/parent-fp; D2a and D2b compare the 65 bytes
(chain code ‖ compressed pubkey), which are identical for an xprv and its xpub; D1′ discards the
key entirely; D1″ validates the recomputed checksum; D2c and D3 read the bracket. Not reachable
from the primary (the md1 wire carries no private material, so `to_miniscript` cannot render one),
so the exposure is a hand-edit, a mis-merge, or a future primary feature — and the consequence is a
private key vendored into the repository with the gate that exists to police descriptors reporting
`ok`. One line closes it: assert the decoded 4-byte version is the xpub version, not just that it
base58-decodes. D1's own wording ("*the xpub base58-decodes with a valid checksum*") already implies
the assertion it does not make.

## M-4 (Minor) — T4's firmware-size rationale contradicts D1″

T4:

> "Every file this plan edits is a `_test.go`, a `testdata/` fixture, or the vendor script:
> **zero non-test Go files**, so flash and RAM are invariant by construction. If the size moves, a
> non-test file was edited and the plan's 'no normative Go behaviour changes' clause has been broken."

D1″ edits `bip380/checksum.go`, a non-test Go file — which the plan's own *What this plan does NOT
cover* section acknowledges ("*and one identifier exported from `bip380` for D1″*"). The two
sentences cannot both be true, and T4's is the one an implementer runs.

Measured, so the gate itself is fine: the size is **byte-identical** with and without the export —
`1622896 code / 32188 data / 31148 bss | 1655084 flash / 63336 ram` both ways (same worktree,
`bip380/checksum.go` the only differing file, TinyGo `-target pico-plus2 -stack-size 16kb -gc precise
-opt 2 -scheduler tasks`). The invariant holds because the exported wrapper is unreferenced from
`cmd/controller` and the linker drops it — which is a *reason*, not a *construction*. Restate T4:
one non-test Go file is edited, it is unreferenced from the firmware, the size is therefore expected
to be invariant, and here are the two measured numbers.

## M-5 (Minor) — "the slot whose 65 bytes match" is not single-valued

D1′ says "*replace every `[fp/origin]xpub` with `@N/origin`, where `N` is the slot whose 65 bytes
match*". When two slots carry the same key material there is no such `N`, and the plan does not say
what happens. A map-based lookup silently takes one of them, which produces a **false RED** wherever
the template names the other — an implementer debugging a red gate against a corpus the primary just
shipped.

Measured: **0 of 46** vendored records and **0 of 46** keyed records in the primary at `b2c5d693`
carry two slots with the same xpub, so this is latent rather than live. It is reachable, though —
the fork's own F-531 repeated-seat shape puts one key at two seats. Remedy: match on
`(65 bytes, origin path)` and fail loudly if the match is not unique, so an ambiguity announces
itself instead of resolving arbitrarily.

## N-1 (Nit) — the three vendored records under pinned names get no descriptor coverage at all

Because the gate reads the pin for the three F-529 names, a regression in
`md/testdata/vectors/keyed_tr_multi_a.conformance.json` (as distinct from the `forkbuilt/` copy) is
invisible to it — visible in M8's tally, where 43 of 46 reddened and the three pinned ones stayed
green precisely because their vendored copies are never read. D6's anti-drift check looks at the
card and the `wallet_descriptor_template_id`, not at descriptors, so it does not cover them either.
This is arguably correct by design — the vendored copies describe the reuse-**free** policy the
primary replaced these with, and no fork consumer reads them — but it is the one place where "the
corpus stops diverging silently" is not literally true, and one sentence in D5g would say so.

---

## Out of scope, per the brief

Earlier rounds' findings were not re-verified; style and wording were not reviewed; no new features
were proposed; the scanner was not argued back in. **No tracked file in either repo was modified.**
The end state was built in a throwaway `git worktree` at `/scratch/code/shibboleth/.tmp/sh-f630r5`,
removed with `git worktree remove --force` and pruned; `git status --porcelain` is clean in the fork,
in descriptor-mnemonic and in mnemonic-engrave, and `git worktree list` no longer carries it.
