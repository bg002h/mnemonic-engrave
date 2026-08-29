# R0 round 1 — `IMPLEMENTATION_PLAN_descriptor_input_S1_S3.md` @ `ea9f1ac`

**Target:** `design/IMPLEMENTATION_PLAN_descriptor_input_S1_S3.md`, 145 lines,
committed at `ea9f1ac`. Tree clean at dispatch.
**Question, as briefed:** is this plan sound to build from — will executing it as
written produce the spec's behaviour with the gates actually able to fail?
**Taken as settled, not re-derived:** the spec's GREEN at `b949d18`; F-418,
F-417, F-422's status quo, the overnight mandate and stop-rules; the
vector-file-first structure; the citation gate's clean result.
**Reviewer:** independent context, read-only on all three repos. Nothing
modified, nothing committed, nothing pushed. This report is the only file
written.
**Repos read:** `mnemonic-engrave` @ `ea9f1ac`, `descriptor-mnemonic` @
`6864f377`, `seedhammer` fork @ `0b656d7` (branch `ship/tx-engraving`), plus
the vendored registry copy of `md-codec` 0.42.0.
**Tools run:** `git`, `diff`, `grep`, `./target/debug/me` (0.7.0),
`descriptor-mnemonic/target/debug/md`, `scripts/plan-staleness-check.sh`.

---

## Counts

| severity | count |
| --- | :-: |
| **Critical** | **2** |
| **Important** | **7** |
| Minor | 7 |
| Nit | 3 |

**NOT GREEN.** Both Criticals are constructible today and neither is named
anywhere in the plan, the spec's §9 residual list, or `FOLLOWUPS.md`.

---

# Critical

## C1 — `me` links a **published** `md-codec` 0.42.0 that is NOT the tree's, and the two encode-side refusals it is missing are exactly the impossible-wallet class; P2.2 mints on it and P2's gate cannot fail on it

**What the plan says.** P2.2: *"The md1 build path: `md_codec::encode::Descriptor`
constructed in-process (§5.3 …), `encode_md1_string`/`split`, records packed as
`ClassMDMK`."* P2 gate: *"`md decode` round-trips … and whose address 0 equals
the Go derivation."* Nothing in the plan, its Out-of-scope section, or its
stop-rules mentions which `md-codec` this is.

**Measured.** `crates/me-cli/Cargo.toml` declares `md-codec = "0.42"`;
`Cargo.lock:547–550` resolves it to
`registry+https://github.com/rust-lang/crates.io-index`, checksum
`336f2c0c…`. `diff -rq` of that registry source against
`descriptor-mnemonic/crates/md-codec/src` (tree also version `0.42.0`) reports
**eight differing files**: `derive.rs`, `encode.rs`, `error.rs`, `render.rs`,
`test_vectors.rs`, `to_miniscript.rs`, `use_site_path.rs`, `validate.rs`.

The material difference:

```
$ grep -n "validate_" <registry>/md-codec-0.42.0/src/encode.rs
103:    crate::validate::validate_placeholder_usage(&d.tree, d.n)?;
105:    crate::validate::validate_multipath_consistency(&d.use_site_path, overrides)?;
109:            crate::validate::validate_tap_script_tree(t)?;

$ grep -n "validate_" descriptor-mnemonic/crates/md-codec/src/encode.rs
103:    crate::validate::validate_placeholder_usage(&d.tree, d.n)?;
105:    crate::validate::validate_multipath_consistency(&d.use_site_path, overrides)?;
109:            crate::validate::validate_tap_script_tree(t)?;
118:    crate::validate::validate_origin_key_consistency(d)?;      <-- F-217
120:    crate::validate::validate_no_duplicate_key_slots(d)?;      <-- F-218
```

The published crate does not merely skip the calls — the functions and their
error variants **do not exist in it**. Full `pub fn` inventory of
`validate.rs`: the registry copy has 10 items, the tree has 16; absent from the
published crate are `validate_origin_key_consistency`,
`validate_no_duplicate_key_slots` and `validate_relative_timelocks`, and
`error.rs` lacks the `OriginKeyContradiction` and `DuplicateKeySlots` variants
entirely (`diff -u error.rs` shows them as a pure `+54` block).

**Constructed failure.** A plain BIP-380 input — cascade branch 2, childless, so
§5.3(a′) materialises `<0;1>/*`:

```
wsh(sortedmulti(2,
  [dc567276/48h/0h/0h/2h]xpub6DiYrfRwNnjeX…Uhpan,
  [dc567276/48h/0h/0h/2h]xpub6DnT4E1fT8Vxu…Y39Ge))
```

Two different xpubs declaring one `(fingerprint, origin path)`. Walk §4.7's
predicate conjunct by conjunct: shape ✓ (`wsh(sortedmulti)`), threshold ✓
(`1 ≤ 2 ≤ 2`), key count ✓ (`2 ≤ 20` under `wsh`), version bytes ✓ (`xpub`),
network ✓ (one), origins ✓ (both keys carry a non-empty origin path), use-site ✓
(absent — an admitted member of conjunct 7's closed set). **Every conjunct
holds, so `me` admits it and `--as md1` carries it.**

The tree's encoder refuses the identical policy — run just now:

```
$ md encode 'wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*))' --path "m/48'/0'/0'/2'" \
    --key @0=<A> --key @1=<B> --fingerprint @0=dc567276 --fingerprint @1=dc567276
md: codec error: @0 and @1 declare the same key origin ([dc567276/48'/0'/0'/2'])
but different xpubs; one origin identifies exactly one key, so this card
describes a wallet that cannot exist
```

(Control with distinct fingerprints encodes clean, `chunk-set-id: 0x16d62`.)
The Go port enforces it too — `md/encode_multisig.go:102,275`
(`ErrOriginKeyContradiction`) and `md/expand.go:257` (`DuplicateKeySlots`), with
`md/origin_key_contradiction_test.go` and `md/duplicate_key_slots_test.go`
standing over them. So under this plan **`me sysw pack --as md1` becomes the
only encoder in the constellation without the refusal**, and it is the one
pointed at steel.

**And P2's gate cannot fail on it.** `grep -rn` for the two validators across
the tree's whole `md-codec/src` returns **only** `encode.rs:118` and
`encode.rs:120` — the decode path never calls them, so `md decode` reads the
card back happily. The tree's own doc comment on the error states why the
address check is blind: *"addresses derive from the xpubs a card CARRIES, not
from the origin it declares, so every address check passes either way and the
failure surfaces only when someone asks a signer to find the key."* P2's gate is
`md decode` round-trip + address-0 equality. Both pass. The plate is cut.

**Why nothing else catches it.** §4.7's seven conjuncts contain no
origin-consistency and no duplicate-slot clause (read in full, lines 627–736) —
by design, since the check lived in the codec. §7's required-row list has no such
row, so no vector fires. §9 residual 4 named the risk and explicitly left it
open: *"**Not checked:** whether the published 0.42.0 tarball is byte-identical
to the tree's `md-codec`, only that it exports the same names with the same
signature."* It is not, and a name-and-signature check was never going to see
this — the divergence is a missing call and two missing functions, not a changed
one. A `FOLLOWUPS.md` sweep for `md-codec`, `0.42`, `publish`, `tarball`,
`F-217`, `F-218`, `origin_key`, `duplicate_key` finds **no entry owning this
gap**; F-217 is recorded CLOSED (line 8391) with a resolution table asserting the
check is *"on the ENCODE path in both languages"* — true of the tree and the Go
port, false of the crate `me` links.

**What the plan must do (a fix is not prescribed — reproduce the defect first).**
The decision belongs in the plan because it collides with a hard boundary: the
mandate forbids crate publishes, so the obvious remedy (publish 0.43.0, bump) is
out of scope tonight. The plan must own the choice explicitly — a temporary path
dependency, a host-side conjunct 8 mirroring both validators before
`encode_md1_string`, a vector row that would fail if the check is absent, or a
recorded park with the funds-risk stated. Any of those is a plan task; silence is
not. Note the constellation rule this touches: an origin-consistency check added
host-side in `me` would be normative behaviour, and the Rust primary already has
it — so mirroring it is convergence, not leading.

---

## C2 — P1.1's `descriptor/` module is **bin-internal**, which puts the `host_admits` column out of reach of `tests/descriptor_seam.rs`, and no CLI surface in an S1+S3 build can express that predicate

**What the plan says.** P1.1: *"New module `crates/me-cli/src/descriptor/`
(**bin-internal**, no new deps …)"*. P0.2: *"Rust harness
`crates/me-cli/tests/descriptor_seam.rs`"* — an integration test. P1.1 again:
*"TDD: un-ignore the P0 host-column tests FIRST, watch red, then build to
green."* P1 gate: *"all P0-ignored host assertions un-ignored and green."*

**Measured.** `crates/me-cli/src/main.rs` (2612 lines) declares **no** modules of
its own — `grep -n "^mod \|^pub mod "` returns exactly one hit, `mod tests` at
line 2453. Every module in the crate lives in `src/lib.rs`
(`pub mod bundle/classify/manifest/ndef/preview/seal/sysw/validate`). A Rust
integration test links the **library** target, so a module declared in `main.rs`
is unreachable from `tests/`.

The precedent §7 instructs the plan to follow *"exactly, not approximately"*
proves the shape: `crates/me-cli/tests/codex32_seam.rs:60` calls
`mnemonic_engrave::sysw::classify(s)` — a **lib-public** function — to assert its
host column.

**Constructed failure, path A (compile).** The implementer writes
`mod descriptor;` in `main.rs`. P1.1 says un-ignore the host-column tests first.
`tests/descriptor_seam.rs` then needs `mnemonic_engrave::descriptor::…`, which
does not exist: `error[E0433]: failed to resolve: could not find 'descriptor' in
'mnemonic_engrave'`. Not a TDD red — a build that never runs the assertion.

**Constructed failure, path B (the dangerous repair).** The implementer reroutes
the host column through `assert_cmd` on the `me` binary instead. But
`host_admits` is defined by §7 as *"§5.2's classification predicate: `me` would
pack this input as a `Descriptor` record"* — explicitly **not** "`me`'s cascade
parses it" and **not** "some `--as` succeeds". In an S1+S3 build no invocation
can produce a `Descriptor` record: P2.1 ships `--as descriptor` as the §5.1
window **refusal**, and §11 item 1 (the only surface that would show one) is
parked with S2 by F-418. Measured today: `me sysw pack '<admitted descriptor>'`
→ `rc=4`, *"Descriptors and addresses are not yet classifiable here"*; `me sysw
pack --as md1 …` → `error: unexpected argument '--as' found`. So every
CLI-observable outcome for an admitted descriptor and for a `--as descriptor`
refusal is the **same window refusal**, and a host column asserted through the
CLI returns the same answer for `host_admits=true` and `host_admits=false` rows
alike. That is a gate that cannot fail — over the invariant
`host_admits ⇒ device_admits(canonical)`, which is the entire reason §7 exists.

**Why the plan reached for bin-internal is legible and does not dissolve the
finding.** `mnemonic-engrave` is a published crate (`Cargo.toml`: `repository`,
`keywords`, `categories`; version 0.7.0), so `pub mod descriptor` widens a
public API. The plan needs to say which way it resolves that — a
`#[doc(hidden)] pub mod`, a `pub(crate)` module plus a lib-public predicate
function shaped like `sysw::classify`, or an explicit API widening — and the
host column's callable surface has to be named. "Bin-internal" plus "integration
test asserts the host column" cannot both hold.

---

# Important

## I1 — P0.2 resurrects `two permitted overlaps`, a phrasing the spec's own stale-phrase sweep swept to zero; asserted literally it fails the conformant file, and the natural repair removes the only thing stopping the row floor being counted around

P0.2: *"asserts the manifest arithmetic (tag minima, 85 slots, 68-row floor,
`covers`+`md1_admits` present on every row, **the two permitted overlaps
only**)"*.

**Measured.** `grep -c "two permitted overlaps" design/SPEC_descriptor_input.md`
→ **0**. It is one of the phrasings the r20 closure's stale-phrase sweep
verified dead (*"0 hits on every superseded phrasing"*, its Part 2 table). The
spec's live rule is at line 1608: the fifteen §4.5 rows carry `gate` as **a
second tag; two of the fifteen carry a third**, the original overlap pair. That
is **15 + 2 = 17 overlap slots**, which is exactly the number the floor rests
on: 85 − 17 = 68.

**Constructed failure.** The implementer writes P0.2 as stated — assert at most
two rows carry more than one tag. The conformant file authored by P0.1 has
**fifteen** such rows, so the harness reds at 15, immediately, with the file
correct. The cheapest repair is to delete the overlap assertion. Now the floor
is satisfiable by retagging: drop a physical row, add a second tag to another,
and the tag minima and the 85-slot sum still hold. That is verbatim the defect
R0 r7's NEW-M1 filed and the floor was introduced to close (*"so a dropped row
cannot be counted around by retagging or by duplicate tags"*).

Remedy is one clause: assert 17 overlap slots, distributed as 15 second-tags on
the §4.5 rows plus 2 third-tags on the named pair — and assert `covers` entries
are distinct within a row, which is the other half of §7's sentence and is also
missing from P0.2.

## I2 — §7's `wallet_id` column is "asserted by BOTH suites"; P0.3's Go-side list drops it, which deletes the one gate built for the F-212 class

§7: *"Rows may also carry **`wallet_id`** — the WalletPolicyId fingerprint (walk
W10): asserted by BOTH suites, each computing it from its own implementation —
the F-212 class (a cross-language identity divergence no per-repo test can see)
made into a standing gate."*

P0.3 enumerates what the Go test asserts: *"`device_admits` via
`nonstandard.OutputDescriptor` on the input, requirement 4's fixed point on
`canonical`, `address_0/1` via `address.Receive` where present, `device_probe`
rows never fed to the panicking function"*, plus the named `sysw_class` skip.
**`wallet_id` is absent**, and P0.3 is the only task that could place it. P2's
gate mentions "wallet-id … assertions all live" — Rust-side, inside the seam
harness.

**Constructed failure.** The Rust suite computes `wallet_id` via
`md_codec::identity::compute_wallet_policy_id`; the Go suite never computes it.
A divergence of exactly the F-212 shape — the two languages disagreeing on a
`WalletPolicyId` while every other assertion in both suites passes — ships green,
which is the failure mode the column was created to make impossible. The repo's
own memory records the precedent: *"Go/Rust computed different WalletPolicyIds;
887/887 fork tests passed either way."*

**It is implementable, so this is a dropped task and not an impossibility.** The
fork carries `md/walletpolicyid.go` with `WalletPolicyIDStub(d)`,
`md/template_id.go` with `WalletPolicyIDStubChunks(strs)`, and
`md/encode_multisig.go`'s `EncodeMultisig` to get from a descriptor to the md1
form. The plan should either name that route in P0.3 or state, with its reason,
that the Go half of the `wallet_id` gate is parked — and to which phase.

## I3 — the fork-side task has no repo boundary, no branch, and no push ritual; its stated baseline is not the tree an implementer would find checked out, and the precedent it is told to copy is not on that branch

P0.3 says *"copy the file byte-identically to
`seedhammer/nonstandard/testdata/descriptor_seam_vectors.json`; add
`nonstandard/descriptor_seam_test.go`"*. The plan's baseline line records
*"seedhammer fork `d402f18`"*. P3.4 covers pushing with
*"`scripts/push-via-staging.sh`"* and no mention of a second repo.

**Measured, in `/scratch/code/shibboleth/seedhammer`:**

- HEAD is `0b656d7`, on branch **`ship/tx-engraving`** — an in-flight ship branch
  for the tx-engraving cycle.
- `git merge-base --is-ancestor d402f18 HEAD` → **false**. The plan's baseline is
  not reachable from the checkout. `d402f18` is the tip of `main` (`git branch -v`:
  `main  d402f18  sysw: the device half of the codex32 seam gate`), and
  `git rev-list --count d402f18..HEAD` → 0 — the two lines have diverged, not
  advanced.
- `git ls-files | grep seam` on the checked-out branch returns only
  `gui/unlock_mnemonic_seam.go`. `git ls-tree -r --name-only d402f18 | grep seam`
  returns `sysw/codex32_seam_test.go` **and** `sysw/testdata/codex32_seam_vectors.json`.
  **The precedent §7 tells P0.3 to copy exactly does not exist on the branch that
  is checked out.**
- `scripts/push-via-staging.sh:16` defaults
  `REQUIRED_CONTEXT="test (rust + go)"` — `mnemonic-engrave`'s job name. The
  fork's `.github/workflows/test.yml` declares `jobs: tests:`. The script's own
  failure path is `FATAL: timed out waiting for the required context` after 120
  polls.

**Constructed failure.** The implementer, working in the fork checkout as the
plan directs, writes `nonstandard/descriptor_seam_test.go` and commits. It lands
on `ship/tx-engraving` — a second writer on someone else's in-flight branch,
against the standing parallel-isolation rule — and the test it was told to model
on `sysw/codex32_seam_test.go` is not there to read. If instead they switch to
`main`, the descriptor work now sits on a branch that has none of
`ship/tx-engraving`'s tree, and P3.4's push script cannot gate it.

**One clause fixes the mechanics** (which fork branch, worktree or not, and the
fork's own push path), but it has to be *decided*, because the fork's default
branch and its checked-out branch are different lines of work and the plan's
baseline names the one nobody is standing on.

## I4 — F-413 is `#ruling-needed` with owning phase "before S1 closes"; the plan carries it to a ship-time sweep without obtaining a ruling or re-owning it

The r20 closure's leaves-open list names it as one of *"Two operator rulings due
before S1 closes (both `#ruling-needed`, owning phase stated in FOLLOWUPS)"*.
`FOLLOWUPS.md` line 14470 records F-413's owning phase verbatim as
**`descriptor-input cycle, before S1 closes`**, still tagged `#ruling-needed`.

The plan's only handling is P3.3 — *"F-413 → spec-as-written noted"* — inside the
ship-phase reconciliation sweep, and one line in Out of scope. That is **after**
S1 and S3 have both closed.

The sibling ruling in the same closure bullet, F-422, has since been ruled
(INTERIM RULING 2026-08-28, status quo, *"Owning phase changed accordingly:
**none — standing decision**"*). F-413 has not: the overnight continuity records
a *build default* (*"build spec-as-written (`ypub` refused, executable
remedy)"*), which decides what to code, not whether the ruling is discharged.

**Constructed failure.** S1 and S3 close green. P3.3's sweep runs at ship and
writes "spec-as-written noted", leaving F-413's entry reading `#ruling-needed`,
owning phase *before S1 closes*. The repo's grep-based reconciliation — the whole
reason owning phases are recorded — then reports an overdue item at ship, and a
user-visible refusal (`ypub` refused rather than normalised) ships in the S3
window build without the ruling its own filing says is due first. The mandate's
own escape is unused: *"fable consult substitutes for the operator only if it
gates"* — the plan never asks whether it gates.

Remedy: move F-413's discharge into P0 or P1 (build-as-specified plus an explicit
re-own of the FOLLOWUPS entry to a later phase, or the consult), so the sweep at
P3.3 reconciles rather than discovers.

## I5 — §5.3(b)'s label warning has no task and no test, and journey J1's own fixture fires it

§5.3(b) is inside §5, which is headed **NORMATIVE**, and carries a verbatim
quoted text:

```
me: warning: the label "Test Multisig 2-of-3" is not carried by any record
    format and will not appear on the device. Nothing else is lost.
```

§5.4 places it: *"§5.3(b)'s label warning, where it applies, follows the
block."* It applies on **both** paths (*"The label is dropped, by both
paths"*), so it is S3 behaviour, not S2's.

**No plan task produces it.** P2.3 enumerates the identification block's lines —
wallet-id, address 0 + compare prompt, watch-only, the (a′) annotation — and stops
there. P2.4 is scoped to *"§6's refusal texts"*, and the label warning is not a
§6 row (it is a warning, not a refusal). P2's gate does not mention it.

**Constructed failure.** J1 — the plan's own P2 integration journey — is *"the
fork's own `sh` fixture, 14 lines, saved as `wallet.txt`"* (walk header). That
fixture's first header is `Name: sh` (`nonstandard/parse_test.go`, the `sh`
case). So the first command of the plan's own acceptance journey is specified to
emit a warning that nothing in the plan builds and nothing in the plan asserts.
It ships missing, and the S3 release silently drops a wallet label with no
notice — the exact class §5.3(b) was written to prevent.

## I6 — the S3-vs-S2 split of §6's rows is never enumerated, and the window refusal — the front door of this release — falls through the gap between P2.1, P2.4 and P3.1

§11 item 4 requires *"Every refusal in §6 has a test that reaches it and asserts
the *text*, not just the exit code"*, splitting them: *"The `--as
descriptor`-only rows among them are S2's (F-418); the rest bind S3."* The plan
never says which rows those are. Machine-counted: §6's table has **34 data
rows** (`sed -n '1270,1385p' … | grep -c '^| '` → 36, minus header and
separator).

The plan uses "enumerated" three times for sets it does not enumerate: P1's gate
(*"except the `--as md1`-execution rows (P2's, enumerated)"*), P2.4 (*"§11 item
4's S3-bound rows"*), P3.1 (*"the `--as descriptor` rows recorded as S2-parked"*).

**Constructed failure.** The implementer applies the only mechanical reading
available — park any §6 row whose text mentions `--as descriptor`. Two rows
match: `wsh(multi(…))` under `--as descriptor`, and **`--as descriptor` in a
build where its path has not shipped**. Both are in fact fully reachable in the
S1+S3 build the plan is producing: §5.1 rules that a `multi` form under explicit
`--as descriptor` gets conjunct 1's **permanent** refusal *in every build*, and
the window row is precisely what P2.1 builds. So P3.1 parks as "S2's" the row
that P2.1 shipped, and the window refusal's verbatim text — which §5.1 calls
*"the front door of the S3 release"* and which the walk records as the **first
command of both walked journeys** — ends up with no owned text test. P2.4's other
clause does not catch it either: it covers *"the five-case item-5 matrix"*, and
the window's two variants are §11 item 5's **sibling**, a sixth requirement
stated after the five.

Remedy: enumerate the split once, by row, in the plan — 34 rows is a table, not a
judgement call — and name the window row's test explicitly.

## I7 — nothing in the plan makes the per-row value assertions **countable**, so a mistyped field in a hand-authored 68-row file deletes its own gate and every stated gate still passes

§7 is unambiguous that the reading must be total: *"EVERY such field a row
carries is asserted (R0 r6's NEW-I1: r5's fold added the values to the `multi`
row and left this paragraph naming `address_0` alone, so the `multi` →
`sortedmulti` mutant still passed every stated assertion — **a gate that cannot
fail, twice**)."*

The plan's mechanisms are: P0.2's manifest arithmetic (`covers` and `md1_admits`
presence, tag minima, floor, sha256), §7 requirement 5's non-vacuity, and P2's
grep for zero `#[ignore]`. **None of them can see an unevaluated row.** The value
fields are optional per row, so any implementation reads them as "where
present"; `#[ignore]` is a whole-function attribute, so the grep proves a
function runs, not that its loop body reached row 41.

**Constructed failure.** P0.1 authors 68 rows by hand. One row's `address_0` is
typed `addres_0`. Re-pin the sha256 (P0.2 requires it anyway), and: the manifest
arithmetic is untouched (`covers` unchanged), non-vacuity is untouched (booleans
unchanged), the row floor is met, zero `#[ignore]` remain, both suites are green
— and that row's address assertion no longer exists. §7's row schema does not
forbid unknown keys, so nothing rejects the typo either.

Remedy is cheap and belongs in P0.2/P0.3: count assertions performed per column
and assert the counts against a manifest of expected per-column totals, or
reject unknown row keys outright. Either turns "every field a row carries is
asserted" from a sentence into something a suite holds.

---

# Minor

**M1 — r20-M2's blanket delivery clause is absent; measured, only one row
actually diverges, but clauses 5/6's multi-line inputs still have no stated
delivery.** The closure prescribed *"One clause in the plan ('gate rows deliver
via `--in`') closes it"*; the plan has none. Measured on `me 0.7.0`, argv
delivery: `tx: zz` → **rc=3** (the bearer guard preempts, as r20 found, and §7
clause 2 already pins `--in` for that row); `pass: hunter (2)` → rc=4;
`text: my wallet (2 of 3)` → rc=4; `seed: abandon abandon abandoz` → rc=4; a
mistyped bare mnemonic → rc=4. So the other five clause-2/3 rows are safe on
argv and the exposure is smaller than the closure feared. What remains
unstated: clause 5's mnemonic-then-descriptor split and clause 6's three
records-plus-bare-key files are multi-line, and r19's carried note (*"per-line
scope over a multi-operand argv invocation means the LF-separated record
stream — one clause in the plan"*) is also missing.

**M2 — the `#[ignore]` grep gate is assigned to two different phases.** P0.2:
*"P3's exit gate greps for zero remaining"*. P2 gate: *"ZERO `#[ignore]`
remaining in the seam harness (grep-gated)"*. Pick one; P2 is the right one,
since P2 is where the last surface lands.

**M3 — P2.1's stated acceptance cannot close in P2.1.** It claims the carriage
rule, both window variants and admission-precedes-window. The variant choice is
decided by md1-representability (§5.3(a)/(a″)) → P2.2; the carriage rule needs
to know whether `--as md1` carries the input → P2.2; and §5.1 emits the window
refusal *after* the §5.4 identification block → P2.3. Not a gate break (the
gate is at phase end), but the sub-task order as written is unbuildable and will
mislead a TDD sequence.

**M4 — the plan states no per-phase re-validation step.** The standing rule is
explicit — *"A PLAN'S GREEN EXPIRES … the order is: prior phase closes green →
re-validate the next plan → dispatch"*. The plan supplies its half (baseline
revs, and `plan-staleness-check.sh` runs clean: `unchanged: 0 ; DRIFTED: 0`)
but never schedules the check. One line per phase gate.

**M5 — P3.3 mischaracterises F-422.** The plan lists it among *"F-414/F-420/F-422
→ confirmed parked with owners"*. F-422's entry reads *"Owning phase changed
accordingly: **none — standing decision**"* — it has no owner and is not parked
work; it is a decision record. F-414 (*post descriptor-input cycle*) and F-420
(*with or after descriptor-input S1*) are correctly characterised.

**M6 — P2's gate does not carry §5.3's citing clause.** §7 requires that where
`md1_admits` is false on an otherwise-admitted row, the Rust test asserts the
md1 path refuses **citing §5.3(a)/(a″)** — *"a refusal for an unrelated cause
must not satisfy the assertion; this is what turns §5.3(a)/(a″) from prose into
a gate"* (R0 r4's NEW-M2). P2's gate says only *"the md1-splits/gate/address/
wallet-id/read-back assertions all live"*. It inherits correctly by reference to
§7, but this is the one clause in §7 whose whole purpose is stopping a
false-pass, and it is worth naming where the gate is stated.

**M7 — "fable per the standing triggers" in P2's review conflicts with the closed
carve-out.** `CLAUDE.md` (2026-08-16) states *"fable is NOT a reviewer tier at
all … opus is the top of the ladder, including for the final pre-irreversible
review. Do not propose fable for a gate, a final review, or a pre-flash
check."* The overnight mandate reinstates fable only on a **count** trigger
(*"at 15 opus reviews the reviewer tier SWITCHES TO FABLE"*), which the plan
records correctly in its own stop-rules section. P2.2's parenthetical *"opus
(fable per the standing triggers)"* reads as a per-review trigger and can route
the mandatory post-implementation review to fable at round 1.

---

# Nit

**N1** — P2.2 writes `ClassMDMK`, the Go spelling. The Rust variant is
`Class::MdMk` (`crates/me-cli/src/sysw/record.rs:51`); §5.3 says *"records of
class `MdMk`"*. `ClassMDMK` is correct only in the sentence about the fork's
`classifyConstellation`.

**N2** — the plan's header cites the closure report for the spec's GREEN, but
the two are one fold apart: the closure targets `5e3c16b` and checks
`wc -l` = 1843, while the spec at `b949d18` is **1850** lines and already carries
r20-M1's fold (gate min 34, 85 slots, 68 floor, the `deadbeef` row in §7 clause
3, and r20-M2's `--in` note inline in clause 2). Recorded so round 2 does not
re-derive the arithmetic against the older number.

**N3** — the plan contains **zero fenced code blocks**
(`grep -c '^```'` → 0), so `scripts/plan-build-gate.sh` is a no-op on it and
vouches for nothing. The standing rule asks that a review brief state what the
gate does and does not cover; this plan's executable content is entirely
*commands and file paths*, which is the ungated class the constellation measured
at 5 of 22 false. C1, I3 and I6 are all in that class.

---

# Plan claims verified TRUE — do not re-derive in round 2

| # | claim | how checked | verdict |
| --- | --- | --- | :-: |
| 1 | P0.1's manifest: 9 tags, 85 tag-slots, ≥68 rows, `gate` 34 | summed against §7's table (line ~1650): 4+15+14+1+5+3+3+6+34 = 85; 85−17 = 68 | ✓ exact |
| 2 | P0.1's per-tag minima, all nine | compared value-by-value to §7's manifest table | ✓ exact match |
| 3 | §7's gate clauses sum to 34 | 15+6+2+4+1+3+3 = 34 (clause 3 carries the mnemonic **and** the `deadbeef` row) | ✓ |
| 4 | r20-M1 is folded into the spec, not left for the plan | §7 clause 3 carries `deadbeef: xpub…`; manifest reads 34/85/68 | ✓ spec-side done |
| 5 | r20-M2 is folded into the spec for its own row | §7 clause 2: *"`tx: zz` (delivered via `--in` …)"* | ✓ (see M1 for the residue) |
| 6 | P1.1's *"no new deps"* | `crates/me-cli/Cargo.toml` `[dependencies]` read whole — no `miniscript`, no `bitcoin` | ✓ |
| 7 | P1.1's *"the four normative refusals"* for BlueWallet | §4.2's NORMATIVE paragraph names exactly four: no `Format:`, zero cosigner lines, empty origin path, fingerprint ≠ 8 hex | ✓ |
| 8 | P0.3's `package nonstandard_test` is import-cycle-proof and legal | `nonstandard/parse_test.go` is `package nonstandard` (internal) — the two coexist; `address` imports only `bip380`, and an external test package may import importers of `nonstandard` | ✓ |
| 9 | P0.3's Go assertions are implementable | `nonstandard.OutputDescriptor` at `nonstandard/parse.go:36`; `address.Receive`/`address.Supported` at `address/address.go:24,28` | ✓ |
| 10 | P0.1's authoring route for `wallet_id` exists | `md inspect --json` emits `wallet_policy_id` (`crates/md-cli/src/cmd/inspect.rs:64`) | ✓ |
| 11 | `identity.rs` and `canonicalize.rs` are byte-identical between the published and tree `md-codec` | `diff -rq` — neither appears in the differing set | ✓ (so `wallet_id` agrees across the seam; C1 is confined to `encode`'s validators) |
| 12 | §11 item 2's *"the JSON exemplar non-`/0/*`"* note is real | the fork's JSON fixture is `wsh(sortedmulti(2,…/0/*,…/0/*))#hfwurrvt` (`nonstandard/parse_test.go:22`) | ✓ |
| 13 | J2's bare-`zpub` fixture is reproducible from the repo | `nonstandard/parse_test.go` carries `[4bbaa801/84'/0'/0']zpub6qpFgGWoG7bKm…` → `wpkh([4bbaa801/84'/0'/0']xpub6C9j4wAxxkWN4…)`, and a bare `zpub…` promoting with fingerprint `00000000` | ✓ |
| 14 | J1's BlueWallet fixture is reproducible from the repo | the `sh` case in `nonstandard/parse_test.go`, 14 lines, `Name: sh`, `Format: P2WSH`, `Derivation: m/48'/0'/0'/2'`, three childless xpubs | ✓ (and it is what fires I5) |
| 15 | P3.4's *"docs-only fast path"* exists | `.github/workflows/release.yml:73–101,170–172` — a real `docs_only` detection gating every test step, with a verdict step that reports the required context green | ✓ |
| 16 | `scripts/plan-staleness-check.sh` runs clean on this plan | `… design/IMPLEMENTATION_PLAN_… . b949d18` → `unchanged: 0 ; DRIFTED: 0 ; not in this repo: 0` | ✓ |
| 17 | `--as` is not yet a flag, and a descriptor is exit 4 today | `me sysw pack --as md1 …` → `error: unexpected argument '--as' found`; an admitted `wsh(sortedmulti(…))` on argv → rc=4 | ✓ baseline |
| 18 | F-416, F-419, F-420, F-421, F-418, F-417 owning phases as the plan characterises them | read verbatim from `FOLLOWUPS.md` lines 14502, 14511, 14522, 14536, 14569, 14586 | ✓ (F-413 and F-422 are the exceptions — I4 and M5) |
| 19 | the ignore-then-unignore pattern is sound *in principle* | `#[ignore]` tests are skipped, not failed, by both `cargo test` and `cargo nextest run`, so P0 can be green with the host column dark, and P1/P2 un-ignore incrementally | ✓ as a pattern (its holes are C2, I7, M2) |
| 20 | the plan does not violate the mandate's hard boundaries | no tags, releases, publishes or on-device actions appear anywhere in it; pushes are in scope per the mandate | ✓ (but see I3 — the fork push is unspecified, and C1's most natural remedy *would* need a publish) |

---

# Verdict

**NOT GREEN — 2 Critical / 7 Important.** The plan's spine is right: the
vector-file-first ordering is sound, the manifest arithmetic is exact against
the spec, the phase boundaries respect F-418, and the fixtures the walk journeys
need are all really in the repos. What it is missing is a codec it does not
control (C1), a module boundary that makes its own primary gate callable (C2),
and — across the Importants — the specific clauses that turn §7's paragraphs
into assertions that can fail.

Two of the seven Importants (I1, I7) and both Criticals share one shape: a gate
stated in prose that, executed as written, either fails on correct input or
cannot fail on wrong input. That is the class this cycle's own spec history was
rewritten twice to close, so it is worth spending round 2's budget on the
mechanism of each gate rather than on fresh scope.
