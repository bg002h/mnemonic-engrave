# R0 round 2 — `IMPLEMENTATION_PLAN_descriptor_input_S1_S3.md` @ `05f8fea`

**Target:** the plan at `05f8fea` (167 lines), **plus the two companion
commits that are new normative authorship**: `9297a29` (spec conjunct 7a +
gate clause 8 + manifest 36/87/70) and `949d18e` (F-424).
**Question, as briefed:** did the fold fix each of r1's nineteen findings, and
did the rewrite or the post-GREEN amendment introduce defects. Proportional
re-review, not a fresh audit.
**Taken as settled, not re-derived:** r1's 20-row verified-TRUE table; the
spec's pre-amendment GREEN; F-418/F-417/F-422 and the overnight mandate; the
citation gates.
**Reviewer:** independent context, read-only on all three repos. Nothing
modified, committed or pushed in any repo. The fork tree was read via
`git archive main | tar -x` into the session scratchpad — no worktree added, no
ref written.
**Repos read:** `mnemonic-engrave` @ `05f8fea`, `descriptor-mnemonic` @
`6864f377`, `seedhammer` fork @ `main` = `d402f18` (checkout is
`ship/tx-engraving` @ `0b656d7`), plus the vendored registry `md-codec` 0.42.0.
**Tools run:** `git`, `diff -rq`, `grep`, `gh api`,
`descriptor-mnemonic/target/debug/md` (4 encode probes),
`go run` against the fork's `nonstandard`/`address` packages (4 parse probes,
Go 1.25.10), `scripts/plan-staleness-check.sh`.

---

## Counts

| severity | r1 disposition | NEW this round |
| --- | :-: | :-: |
| **Critical** | 2 → both mechanism-owned | **2** |
| **Important** | 7 → all 7 FIXED | **3** |
| Minor | 7 → all 7 FIXED | 7 |
| Nit | 3 → 2 FIXED, 1 open | 3 |

**NOT GREEN — 2 Critical / 3 Important, all five introduced by the fold or
by the post-GREEN spec amendment.** The rewrite is a genuinely good fold:
eighteen of r1's nineteen findings are closed, most of them by the exact
remedy, and I re-traced each constructed failure rather than checking for
the presence of words. What did not survive is the amendment itself —
conjunct 7a is the right idea landed in the wrong three places.

**All five new blocking findings are in the ungated class** (r1's N3: the
plan and the spec carry zero fenced code blocks, so
`scripts/plan-build-gate.sh` vouches for nothing). Four of the five were
found by *running something* — `md encode`, `nonstandard.OutputDescriptor`,
`gh api`. None was findable by re-reading.

---

# Critical

## NEW-C1 — §4.7's `multi` clause enumerates "conjuncts (2–7)", which does **not** include 7a, so the three `multi` twins bypass the impossible-wallet checks — and `multi` exists ONLY on the md1 path, the one path that reaches the published crate that lacks them

**What the amendment did.** `9297a29` inserted conjunct **7a** into §4.7's
predicate list (line 683) and left conjunct 1's scoping sentence untouched.
That sentence, at **line 656**, is normative and is an explicit enumeration:

```
All other conjuncts (2–7) apply to `multi` identically.
```

`7a` is not a member of `2–7`. Read as written, a `multi` form is subject to
conjuncts 2 through 7 and **not** to 7a.

**Why that is the worst possible place for the hole.** Conjunct 1 admits the
three `multi` twins on the **`--as md1` path ONLY** — under `--as descriptor`
the shape conjunct stays at the seven forms. So the class the enumeration
exempts from 7a is exactly the class that exists *only* on the path that calls
`md_codec::encode` — the published 0.42.0 crate whose missing validators are
the entire reason 7a was written. C1's fix is absent from C1's own attack
surface.

**Constructed failure.**

```
me sysw pack --as md1 \
 'wsh(multi(2,[dc567276/48h/0h/0h/2h]xpub6DkFA…r6KFrf/<0;1>/*,
              [dc567276/48h/0h/0h/2h]xpub6Dzhy…MBXd6Vk/<0;1>/*))'
```

Conjunct 1 ✓ (`multi` twin, md1 path). Conjuncts 2–7 all hold — threshold
`1≤2≤2`, `n=2≤20` under `wsh`, `xpub` versions, one network, both origins
non-empty, use-site `<0;1>/*` in the closed set. Conjunct 7a **is not applied,
by the line-656 enumeration**. `me` admits, builds the
`md_codec::encode::Descriptor`, and the published crate encodes it without
complaint. Plate cut for a wallet that cannot exist.

**Measured, just now, that the primary refuses the identical policy:**

```
$ md encode 'wsh(multi(2,@0/<0;1>/*,@1/<0;1>/*))' --path "m/48'/0'/0'/2'" \
    --key @0=<A> --key @1=<B> --fingerprint @0=dc567276 --fingerprint @1=dc567276
md: codec error: @0 and @1 declare the same key origin ([dc567276/48'/0'/0'/2'])
but different xpubs; one origin identifies exactly one key, so this card
describes a wallet that cannot exist
```

So the tree refuses it, the Go port refuses it, and `me` — the only encoder
pointed at steel — would not.

**And no vector row would catch it.** §7's new clause 8 specifies two rows: *"a
colliding-origin descriptor … and a duplicate-key-slot descriptor"* — both
`sortedmulti`-shaped, both `host_admits=false`. There is **no `multi` twin row
under clause 8**, and `md1_admits` is the axis that would have to carry it. The
gate for the fix has no cell for the case the fix misses.

**The plan does not close it either, and cannot be relied on to.** P1.1 reads
*"the seven shapes + the `multi` md1-path twins + conjuncts 2–7 **+ conjunct
7a's two impossible-wallet checks**"* — the same `2–7` phrase, with 7a appended
as a separate item of ambiguous scope. An implementer resolving the ambiguity
against the spec gets line 656's enumeration, which is unambiguous and wrong.

**What must be decided (fix not prescribed).** Either line 656's range covers
7a explicitly, or the spec states why `multi` is exempt — and clause 8 gains
the `multi`-twin row that makes whichever answer executable. Note this is
strictly the *convergence* direction: the primary already refuses it, so
extending 7a to `multi` is not leading.

---

## NEW-C2 — conjunct 7a's refusal has **no §6 row**, so clause 8's REQUIRED `refusal_row` field has no legal value, its text is never asserted, and P2.4's `row-test count == 34` pin makes adding the missing test RED

**What §7 requires of clause 8's rows.** The gate-field schema (read in full):

> `refusal_row` (on the `descriptor-refusal` and `multi-record` outcomes
> only: **a slug naming the §6 row whose text the test asserts** — the
> slug-to-text binding lives with §11 item 4's per-row text tests)

Clause 8 sets `outcome: descriptor-refusal` on both new rows. `refusal_row` is
therefore required on both.

**Measured.** §6's table is **34 data rows** (`awk` over lines 1286–1400: 36
table lines − header − separator), *unchanged by the amendment*. I read all 34:
**none** describes an origin contradiction, a duplicate key slot, or anything
about key identity. `grep -n "7a"` over the whole spec returns exactly three
hits — the conjunct itself (683) and clause 8's two mentions (1611, 1614).
There is no §6 row to name.

**Constructed failure, path A (P0.1 blocks).** P0.1 is the plan's *first* task
and authors the vector file "per spec §7 AS AMENDED". At clause 8's two rows it
must emit `refusal_row: "<slug>"`. Every legal value names a §6 row; none
exists. The task cannot be executed as written.

**Constructed failure, path B (the cheap repair, which is worse).** The author
invents a slug — `origin-contradiction` — and the Rust test binds it to
whatever string the implementation happens to print. §11 item 4 is satisfied
vacuously, because item 4 quantifies over *"every refusal in §6"* and this
refusal is not in §6. The result is a refusal whose operator text is chosen by
the implementer, in a section whose first line is *"A refusal that does not say
why is the defect this constellation has been punished for most"* and whose two
binding rules — **W5** (verdict first, no internal identifiers) and **"every
one of them names a next action"** — bind only rows that are in the table.

**The amendment's own prescription violates both of those rules.** It says
`me` enforces the checks *"with the tree's own refusal wording"*. The tree's
wording is (measured, `error.rs:399`):

```
@0 and @1 declare the same key origin ([dc567276/48'/0'/0'/2']) but different
xpubs; one origin identifies exactly one key, so this card describes a wallet
that cannot exist
```

An operator who typed a **descriptor** has no `@0`/`@1` placeholders and no
"card" — those are md1 authoring concepts — and the message **names no next
action**. Verbatim adoption ships a §6-class refusal that fails two of §6's
own normative rules. (`DuplicateKeySlots`' text — *"one of them holds two of
the seats"* — has the same two problems.)

**And the plan's own countability mechanism enforces the omission.** P2.4:
*"all 34 rows … The test file asserts its own row-test count == 34."* An
implementer who does the right thing — adds the 35th row and its text test —
reds the count assertion. The plan pins the defect in place.

**What must be decided.** §6 gains a row (or two) for conjunct 7a with
operator-language text naming a next action; §11 item 4's count and P2.4's
`== 34` pin move to 35 (or 36); clause 8's `refusal_row` slugs then have
referents. Every number that moves is one the fold already had to touch — this
is the cascade the amendment stopped one site short of.

---

# Important

## NEW-I1 — conjunct 7a's duplicate-slot half is worded `(xpub, ORIGIN)`; the Rust primary's F-218 rule is `(xpub, USE-SITE)`, and the difference is a wallet both the primary and the device carry. "Convergence with the Rust primary, not leading" is false as written

**What the amendment says (line 683ff):**

> …and **no two placeholder slots may carry the same xpub at the same
> origin** (duplicate key slots). Both checks are the F-217/F-218 refusals
> the Rust-primary `md-codec` enforces ON ENCODE … This is **convergence
> with the Rust primary, not leading**.

**What the primary actually enforces** (`crates/md-codec/src/validate.rs:361`,
whole function read):

```rust
if xa == xb && a.use_site_path == b.use_site_path {
    return Err(Error::DuplicateKeySlots { … });
}
```

and its doc comment states the choice as load-bearing, twice:

> THE COMPARISON IS `(xpub, use_site_path)`, and each half is load-bearing:
> … The use-site, because the same xpub at two different multipath branches
> derives a different child at every index — `<0;1>` and `<2;3>` over one key
> **are two different wallets, not a duplicate. Measured, not assumed.**

**Constructed failure — a legitimate wallet the spec's wording refuses.**

```
wsh(sortedmulti(2,[dc567276/48h/0h/0h/2h]xpubX/<0;1>/*,
                  [dc567276/48h/0h/0h/2h]xpubX/<2;3>/*))
```

Same xpub, same declared origin, different use-sites. Walk §4.7: shape ✓,
threshold ✓, key count ✓, versions ✓, network ✓, origins ✓, conjunct 7 ✓
(`<2;3>/*` is `<i;i+1>/*`, an admitted member of the closed set). Then
conjunct 7a **as worded** — same xpub, same origin — refuses it.

Measured on both sides that this is a real, distinct, carried wallet:

```
$ md encode 'wsh(sortedmulti(2,@0/<0;1>/*,@1/<2;3>/*))' … --key @0=X --key @1=X
chunk-set-id: 0xbc4ce                      # the PRIMARY encodes it clean

$ nonstandard.OutputDescriptor(<the descriptor above>)
PARSE-ACCEPT  canonical is a fixed point
addr0 = bc1qsa6qqvkypr9v8ve5z54t8yjtn99ws48w8umyyzyhxy8n6p4msemslqyh88
```

— versus `bc1qf4jpv99wj36eqez9fzxrzww6sdy97uw5gmgp38t6trqpk5lre8qsv3ttqz` for
the `<0;1>/*` pair. Different addresses; a different wallet, exactly as the
doc comment says.

So implemented as worded, `me` refuses what the Rust primary encodes and the
device derives — and emits the primary's *"one of them holds two of the seats"*
message about a policy that has two genuinely different seats. That is a host
narrowing the primary has not made, which the Rust-primary rule requires to
land in Rust first with a test vector. The amendment's "convergence, not
leading" claim does not cover the delta it actually creates, and P1.1 inherits
the error verbatim (*"conjunct 7a's two impossible-wallet checks … with the
tree's refusal wording — convergence with the Rust primary"*).

Fix is two words in the spec, but it must be *decided*: `(xpub, use-site)` is
the primary's rule and the one the refusal text describes.

## NEW-I2 — P2.4 asserts §6's texts "verbatim, all 34 rows", but §5.3's NORMATIVE window substitution **rewrites two of those rows in exactly this build**; nothing in the plan mentions substitution

**The rule (§5.3, NORMATIVE, read in full):**

> In a build where the descriptor path has not shipped, every remedy in this
> section and in §6 that ROUTES TO the descriptor path — naming the flag or
> otherwise (semantic, not lexical) — replaces that clause with: *"the
> scannable-plate path is not in this build — keep the export file; it packs
> when the device update ships."* **No refusal points the operator at a flag
> that refuses in the current build.**

The S1+S3 build is precisely "a build where the descriptor path has not
shipped" — P2.1 ships `--as descriptor` as the window refusal, and §6 has a
row literally named *"`--as descriptor` in a build where its path has not
shipped"*.

**Measured: two §6 rows carry the annotation `Window substitution per §5.3`** —
line 1369 (`/0/*` under `--as md1`) and line 1387 (`<0;1>` without wildcard
under `--as md1`). Both quote a remedy ending *"Use `--as descriptor`, which
carries `/0/*` exactly."*

**Measured: the plan never says the word.** `grep -n "substitut"` over the plan
returns **0 hits**. P2.4 reads *"§6's refusal texts, **verbatim**, one named
test per row, all 34 rows"*.

**Constructed failure.** The implementer writes 34 text tests from §6's quoted
strings. The `/0/*` test asserts *"Use `--as descriptor`, which carries `/0/*`
exactly."* Correct S3 behaviour prints the substituted clause instead, so the
test reds **against correct code**. The cheapest repair is to make the code
print the §6 quote — shipping a refusal that routes the operator to a flag
which refuses in the same build, which is the walk-W11 defect the substitution
rule exists to prevent, now enforced by a green test.

One clause fixes it: name the two substituted rows and say their tests assert
the substituted text.

## NEW-I3 — clause 8 was inserted **into the middle of clause 7's sentence**; as the normative list now reads, the `[` alone row has no verdict, the `xpub…/` row belongs to clause 8, and the clause tally is 37, not 36

**Measured, verbatim from `9297a29`'s diff and confirmed in the file
(lines 1608–1620):**

```
  7. the **edge tokens** — a base58check token with a valid checksum and
     a 77-byte payload (gate CLOSED, `record-refusal`, exit 4); `[` alone
  8. **the impossible-wallet pair (conjunct 7a, PLAN-r1's C1)** — …
     `md1_admits=false`;
     (gate OPEN, `descriptor-refusal`, the unparseable-file row carrying
     branch 4's error, exit 3); and `xpub…/` with a trailing slash (gate
     OPEN, …).
```

The insertion split clause 7 between `` `[` alone `` and its parenthetical
verdict. This is a NORMATIVE list, and **P0.1 authors the 36 gate rows from
it**.

**Constructed failure.** The author enumerates rows per clause, as the manifest
requires. Clause 7 yields two rows, one of which (`` `[` alone ``) has **no
stated `gate_open`, no `outcome`, no `exit_code`** — the three fields the row
schema marks REQUIRED on every `gate` row. Clause 8 appears to yield four. The
tally becomes 15+6+2+4+1+3+**2**+**4** = **37**, against a manifest that says
36 and a floor computed from 87 slots. The floor is a *minimum*, so nothing
reds — the file is simply wrong in a way no assertion in P0.2 can see, and the
`` `[` alone `` row's verdict is whatever the author guesses.

Repair is to move four lines. Recorded as Important rather than Minor because
the corrupted text is normative, is the direct input to the plan's first task,
and destroys one row's entire specification.

---

# Minor

**NEW-M1 — the second `conjuncts 2–7` range (§5.4, line 1172) is also stale;
there the accidental answer happens to be right.** The tier rule reads *"a
wallet that passes conjuncts 2–7 AND whose shape at least one `--as` path
admits gets the FULL block"*. A 7a-failing wallet passes 2–7, so it gets FULL:
wallet-id, address 0, compare prompt, then the refusal. That is defensible —
its addresses genuinely derive (measured above) and §5.4's PARTIAL tier is for
the *underivable* class — but it is accidental, not decided. State it, since
the sentence enumerates rather than generalises.

**NEW-M2 — the plan's recorded baseline predates the amendment it is built on,
so the staleness gate is aimed at the wrong tree.** The plan header still reads
*"FINAL GREEN at `b949d18`"* and records `mnemonic-engrave b949d18` as its
staleness baseline. Conjunct 7a and 36/87/70 arrive at `9297a29`, two commits
later. Measured: `scripts/plan-staleness-check.sh <plan> . b949d18` →
`unchanged: 0 ; DRIFTED: 0 ; not in this repo: 0` — a clean result against a
spec that contains no conjunct 7a. The plan schedules this check at three
phase gates (the M4 fold), so the wrong baseline propagates. Separately, the
spec's own status header still reads *"Status: GREEN — re-closed at round 19"*
with no mention of a post-GREEN normative amendment; a reader of the spec
cannot tell that §4.7 changed after its gate.

**NEW-M3 — the fork push mechanics name a ritual the fork does not have, for a
protection rule it does not have.** P0.3/P3.4: *"Push: the fork's own ritual
with `REQUIRED_CONTEXT=tests`"*. Measured: the fork carries **no**
`push-via-staging.sh` (`scripts/` on `main` is `oracle-live.sh`,
`test-32bit.sh`); `gh api repos/bg002h/seedhammer/branches/main/protection` →
**404 "Branch not protected"**; and `.github/workflows/test.yml` triggers on a
bare `push:`, so any branch push already runs the job. The staging ritual
exists to make a SHA earn a *required context* before a protected-branch push
— there is nothing here for it to satisfy, and a plain
`git push -u origin seam/descriptor-vectors` both suffices and runs `tests`.
(The `REQUIRED_CONTEXT=tests` value itself is **correct** if the script is
borrowed: the job id is `tests` with no `name:` key, so the context string is
`tests`.)

**NEW-M4 — the Go `wallet_id` route named in P0.3 covers a subset of rows, and
the plan does not say which rows carry the column.** The symbols are real
(verified on `main`): `EncodeMultisig` `md/encode_multisig.go:112`,
`WalletPolicyIdChunks` `md/walletpolicyid.go:138`, `WalletPolicyIDStub` :125.
But `EncodeMultisigRequest` has **no use-site field** — `EncodeMultisig`
hard-codes `useSite = <0;1>/*` (`md/encode_multisig.go:166–169`) — and there is
no single-sig arm in the named route (`EncodeSingleSig` is a separate function
with a different signature). So the Go half of the F-212 gate is computable
only for multisig rows at the device-default use-site. P0.1's *"`wallet_id`
computed over the (a′)-materialised policy"* happens to agree with that for
childless rows, which is lucky rather than stated. Name the rows that carry
`wallet_id`, or the standing cross-language gate narrows to whatever the author
fills in.

**NEW-M5 — `address_0` cannot witness clause 8's colliding-origin row; it is
byte-identical to a clean control.** Measured through the device's own
`address.Receive`:

| input | addr0 |
| --- | --- |
| colliding origin (`dc567276` on both keys) | `bc1qf4jpv99wj36eqez9fzxrzww6sdy97uw5gmgp38t6trqpk5lre8qsv3ttqz` |
| control (`dc567276` + `f245ae38`, same two xpubs) | `bc1qf4jpv99wj36eqez9fzxrzww6sdy97uw5gmgp38t6trqpk5lre8qsv3ttqz` |

Identical — which empirically confirms the primary's own doc comment (*"every
address check passes either way"*) and is the same shape as R0 r5's `address_1`
lesson on the `multi` row. P0.1 should not reach for `address_0` as clause 8's
discriminator; the refusal is the only witness.

**NEW-M6 — P1.0 schedules the fable consult unconditionally, where the mandate
conditions it.** The mandate reads *"fable consult substitutes for the operator
**only if it gates**"*, and adds *"Funds-risk decisions a consult cannot
legitimately settle → park."* P1.0 states neither test. F-413 does appear to
gate (owning phase *"before S1 closes"*, and phase-owned items are not
deferrable past their phase), so the consult is probably licensed — but the
plan should say so in the sentence that spends it, and should apply the
funds-risk park test, since `ypub` normalisation changes which wallet an
operator is handed.

**NEW-M7 — the fork branch is left unmerged by design, and nothing owns the
residual.** P3.4 correctly refuses to merge into another cycle's repo. The
consequence is that after the night, the cross-language seam gate — the whole
point of §7 — lives on an unmerged branch and runs in neither repo's default
CI. P3.3's reconciliation list does not include it and no follow-up owns it.
One FOLLOWUPS entry with an owning phase closes it.

---

# Nit

**NEW-N1** — `7a.` is not a valid ordered-list marker and it is placed
*before* item `7.`. Markdown renders it as a plain paragraph and restarts the
list numbering at 7. Cosmetic, but §4.7's list is the artifact reviewers count
conjuncts in.

**NEW-N2** — r1's N3 is unchanged: `grep -c '^```'` on the plan → **0**, so
`scripts/plan-build-gate.sh` remains a no-op and vouches for nothing. Worth
restating because all five of this round's blocking findings are in that
ungated class, and four came from executing something.

**NEW-N3** — P2's build-order line (*"P2.2 → P2.3 → P2.1's window text →
P2.4"*) assigns a slot to P2.1's *window text* but leaves P2.1's `--as` flag
surface unplaced, and P2.2 cannot be exercised end to end until the flag
exists. Harmless to a careful reader; one clause makes it a sequence.

---

# Disposition of r1's nineteen findings

| r1 | verdict | how re-traced |
| --- | :-: | --- |
| C1 (published `md-codec` lacks F-217/F-218) | **MECHANISM OWNED**, defective in three places | Registry-vs-tree diff re-run independently: 8 files differ; registry `validate.rs` has **7** `^pub fn` vs the tree's **10**; registry `encode.rs` calls only 3 validators (lines 103/105/109), tree calls 5 (adds 118/120); `Cargo.lock:547–550` still resolves the registry crate. Conjunct 7a + clause 8 + F-424 land the fix — but see **NEW-C1** (multi excluded), **NEW-C2** (no §6 row), **NEW-I1** (wrong F-218 rule) |
| C2 (bin-internal module unreachable from `tests/`) | **FIXED** | `lib.rs` read whole: 8 `pub mod` + one `pub use`; `main.rs` declares no modules of its own and already does `use mnemonic_engrave::…`. `#[doc(hidden)] pub mod` is public for linkage, hidden only from rustdoc, so `mnemonic_engrave::descriptor::…` resolves from an integration test. Precedent shape matches: `codex32_seam.rs:59` calls `mnemonic_engrave::sysw::classify` |
| I1 (`two permitted overlaps`) | **FIXED** | `grep -c "two permitted overlaps"` on the plan → **0**. P0.2 now asserts 17 overlap slots = 15 second-tags + 2 third-tags, plus `covers` distinct within a row — both halves of §7's sentence |
| I2 (`wallet_id` dropped Go-side) | **FIXED** (+ NEW-M4) | Route named and every symbol verified to exist on `main`: `EncodeMultisig` :112, `WalletPolicyIdChunks` :138, `WalletPolicyIDStub` :125. `WalletPolicyId(d *descriptor)` is *not* callable externally; the plan names the two that are |
| I3 (fork mechanics) | **FIXED** (+ NEW-M3) | `main` = `d402f18` ✓ exactly the plan's baseline; `sysw/codex32_seam_test.go` + `sysw/testdata/codex32_seam_vectors.json` both present on `main` ✓; `seam/descriptor-vectors` does not exist (no collision) ✓; the checkout really is `ship/tx-engraving` @ `0b656d7` with 4 worktrees live, so "never that checkout" is the right call; job name `tests` ✓ |
| I4 (F-413 carried to ship) | **FIXED** (+ NEW-M6) | P1.0 discharges it at the head of P1, which is before S1 closes — the owning phase in `FOLLOWUPS.md:14470` (*"descriptor-input cycle, before S1 closes"*, still `#ruling-needed`). P3.3 now records it as discharged at P1.0 rather than swept |
| I5 (§5.3(b) label warning) | **FIXED** | P2.3 carries the verbatim text and a test that J1's `Name: sh` fixture fires it |
| I6 (S3-vs-S2 row split) | **FIXED** (+ NEW-I2) | Both `--as descriptor`-mentioning rows spot-derived reachable in an S1+S3 build: §5.1 line ~930 — *"the §4.7 admission refusal PRECEDES the window refusal … a `multi` form under `--as descriptor` gets conjunct 1's permanent refusal instead, in every build"*; and the window row is what P2.1 builds. §6 is still **34** data rows (machine-counted), so "all 34" is arithmetically right today |
| I7 (uncountable per-row assertions) | **FIXED** | P0.2 now rejects unknown row keys **and** asserts per-column assertion counts against expected totals — either alone kills the `addres_0` mutant; both is belt and braces |
| M1 (delivery clause) | **FIXED** | P0.1 carries both r20-M2's `--in` clause and r19's LF-separated record-stream note |
| M2 (`#[ignore]` grep in two phases) | **FIXED** | P0.2 now defers to P2; P2's gate owns it |
| M3 (P2.1 unbuildable first) | **FIXED** (+ NEW-N3) | Explicit build-order line at the head of P2 |
| M4 (no re-validation step) | **FIXED** (+ NEW-M2) | Staleness re-check named at the P0, P1 and P2 gates |
| M5 (F-422 mischaracterised) | **FIXED** | P3.3: *"F-422 → standing decision, no owner"* |
| M6 (§5.3 citing clause) | **FIXED** | P2.4 names it explicitly |
| M7 (fable per-review) | **FIXED** | P2 review: *"opus — fable only per the mandate's 15-round count trigger, not per-review"* |
| N1 (`ClassMDMK`) | **FIXED** | P2.2 now writes `Class::MdMk` |
| N2 (GREEN one fold apart) | **ADDRESSED** | Header now reads *"20 rounds + the 15-finding walk"* and cites the r20 closure. Superseded by NEW-M2, which is the same class one commit further on |
| N3 (build gate is a no-op) | **OPEN** | Still 0 fenced blocks. Carried as NEW-N2 |

---

# Verified TRUE this round — do not re-derive in round 3

| # | claim | how checked | verdict |
| --- | --- | --- | :-: |
| 1 | The registry/tree `md-codec` divergence is real and unchanged | `diff -rq` re-run: 8 files differ; registry `validate.rs` 7 `^pub fn` vs tree 10; registry `encode.rs` lacks the 118/120 calls | ✓ C1's premise |
| 2 | `me` still links the registry crate | `Cargo.lock:547–550`, checksum `336f2c0c…` | ✓ |
| 3 | The tree refuses the colliding-origin **sortedmulti** | `md encode` → `OriginKeyContradiction` text, verbatim as quoted in 7a | ✓ |
| 4 | The tree refuses the **true** duplicate slot (same xpub, same use-site) | `md encode` → *"carry the same key at the same use-site … one of them holds two of the seats"* | ✓ |
| 5 | The tree **accepts** same xpub at different use-sites | `md encode` → `chunk-set-id: 0xbc4ce` | ✓ NEW-I1's counterexample |
| 6 | The tree refuses the colliding-origin **multi twin** too | `md encode 'wsh(multi(2,…))'` → same `OriginKeyContradiction` | ✓ NEW-C1's convergence direction |
| 7 | The device ACCEPTS all three impossible/near-impossible inputs | `nonstandard.OutputDescriptor` probe, Go 1.25.10: all PARSE-ACCEPT, canonical a fixed point, `address.Receive(…,0)` returns with `err=<nil>` | ✓ so clause 8's rows are `device_admits=true`, `host_admits=false`, no `canonical` required |
| 8 | Clause 8's stated column values are derivable | host false ✓ (7a refuses on both paths), md1 false ✓, device true ✓ (row 7), gate OPEN ✓ (T1/T3 both trigger on a `(`-bearing xpub line), exit 3 ✓ | ✓ except `refusal_row` — NEW-C2 |
| 9 | Manifest arithmetic, all three sites | tag table `gate`=36 ✓; minima sum 4+15+14+1+5+3+3+6+36 = **87** ✓; 87−17 = **70** ✓; clause list says "eight adversarial clauses" ✓ | ✓ internally exact (the *text* of clause 7/8 is the problem — NEW-I3) |
| 10 | No stale 68/85/34 manifest numbers survive anywhere in the spec | grepped `68`/`85`/`34` over all 1872 lines; every hit is unrelated (line refs, a 34-byte redeemScript, 6840 bytes) | ✓ the fold propagated completely |
| 11 | The plan's own numbers are internally consistent | 70 ×2, 87 ×2, 36 ×2, 34 ×3 — no 68/85 anywhere | ✓ (34 must become 35 under NEW-C2) |
| 12 | §6 is 34 data rows and none covers key identity | `awk` count = 36 table lines − 2; all 34 read | ✓ NEW-C2's premise |
| 13 | Window substitution binds this build and covers exactly two §6 rows | §5.3's NORMATIVE paragraph read whole; rows at 1369 and 1387 carry the annotation | ✓ NEW-I2's premise |
| 14 | F-424 is filed with an owning phase and the right tags | `FOLLOWUPS.md:14637` — *"owning phase: **next md-codec publish, operator-gated**"*, `#md-codec #funds #publish`, and it states the interim host mirror | ✓ good entry |
| 15 | The fork's required-context value is right | `.github/workflows/test.yml` job id `tests`, no `name:` key → context string `tests` | ✓ (though nothing requires it — NEW-M3) |
| 16 | `plan-staleness-check.sh` runs clean | `0 / 0 / 0` against both `b949d18` and `05f8fea` | ✓ clean, but see NEW-M2 for what that clean means |
| 17 | The overnight mandate's boundaries are respected by the plan | no tags, releases, publishes or on-device actions anywhere; pushes in scope; P3.4's refusal to merge the fork is conservative and consistent | ✓ |
| 18 | The `#[ignore]`-then-un-ignore staging still holds after the rewrite | P0.2 tags host assertions, each naming its removing phase; P1 gate un-ignores all but the enumerated md1 set; P2 gate greps zero | ✓ single owner now |

---

# Verdict

**NOT GREEN — 2 Critical / 3 Important.** The plan fold itself is close to
clean: eighteen of nineteen findings closed, each by a remedy that survives
re-tracing, and the three r1 findings I was asked to re-verify against the
world (C2's module reachability, I3's fork mechanics, I2's Go symbols) all
check out against the real trees.

**Every blocking finding this round comes from the spec amendment, and they
share one shape:** conjunct 7a was written into §4.7's list and clause 8's
gate bullet, and the four *other* places that had to move were not — conjunct
1's `2–7` enumeration (NEW-C1), §6's refusal table (NEW-C2), the F-218 rule it
claims to mirror (NEW-I1), and clause 7's sentence it was pasted through
(NEW-I3). This is the standing "a diff falsifies text it never touches" class,
and it is worth noting that the amendment landed **post-GREEN with no review
round of its own** — the plan's R0 loop is the only gate it has ever passed
through, which is why round 2's budget went to it rather than to the plan.

The remedies are all small and local. What round 3 should verify is not the
plan's spine — that is sound and was sound at r1 — but whether the amendment's
propagation is now complete: the two conjunct-range sentences, the §6 row and
its count cascade (34 → 35 in §11 item 4, P2.4's pin, and P3.1), the
`(xpub, use-site)` wording, and clause 7/8's restored boundary.
