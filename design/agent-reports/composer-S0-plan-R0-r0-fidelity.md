# Composer Stage 0 plan — R0 round 0, lens: PLAN-TO-SPEC FIDELITY AND LOWERING CORRECTNESS

**Artifact:** `design/IMPLEMENTATION_PLAN_composer_S0_md_compose.md` at `mnemonic-engrave` `3a799fa`.
**Spec:** `design/SPEC_wallet_policy_composer.md` (R0 CLOSED at `49d2dae` + `0b56ed4`); rulings
`design/BRAINSTORM_wallet_policy_composer.md` §2/§3.12; staging `design/STAGED_PLAN_wallet_policy_composer.md`.
**Target repo:** `/scratch/code/shibboleth/descriptor-mnemonic`. Plan baseline `3b0944fb`; **repo HEAD is now
`480e54fe`** (one docs-only commit past the baseline — see M-5).

**Scope.** One question only: does the plan, executed exactly as written, produce the lowering the spec
defines, and nothing else? Findings are built from CONSTRUCTED inputs run against the plan's own code.
Not reviewed: the spec itself; compile/clippy/test-pass state inside the build gate's coverage; Stage 1-4
material (§6, §7, §8, §9, §12 items 2-3, 5-6, 8-11).

**What I ran (read-only on every repo; all writes confined to my own scratch copy).**
`scripts/plan-build-gate-md.sh` into a private `TMPDIR`, reproducing the plan's stated result (48 tests,
47 pass, the pinned `MANIFEST lacks` red). Then, inside that scratch copy only: three new probe test files
constructing ~30 path lists; two mutation experiments; a §5b sanity sweep over all 28 family templates.
Against the real repo I ran only the already-built `target/release/md` (0.14.0) — read-only.

---

## VERDICT: 0C / 4I / 5M / 3N

The lowering itself is faithful. Every §5 row, §4c band, §4e refusal and §4f origin rule I could construct
an input for produced exactly what the spec requires — including the boundaries the brief named
(two unlocked single keys, a locked head, a sole leaf that exists only after internal-key extraction, a
keyless head, 8 paths, 32 slots, `@10` vs `@1` in `template_with_origins`, a declared origin colliding with
a default). **No Critical.** All four Importants are in the ACCEPTANCE apparatus — the vector tag list, the
presets, and the reach of the §5b cross-check — not in the lowering.

---

## I-1 — The §12 item 1 "required tag" list omits two required members, so the coverage gate cannot fail when their coverage disappears

**Plan location:** lines 1787-1794, Task 5 Step 1 (`every_tag_appears_in_at_least_two_vectors`), and the
`fp:` tag definition in the `family()` doc comment at lines 1636-1642.

**Spec clause — §12 item 1:**

> Required tags include: all four wrappers; path counts 1, 2, 3, 4 and the 32-slot maximum; taptree spine
> shapes m in {0, 1, 2, 3, 7}; the extracted internal key first-listed AND not first-listed with ≥ 4 paths;
> NUMS; the five lock encodings; hash present; sorted and unsorted; keyless wsh; **the three fingerprint
> cases (declared, one seed at two slots in one path, one seed across two paths); unseated-slot origins per
> wrapper.**

The plan's required list ends `"unsorted", "keyless-wsh", "fp:one-seed", "fp:distinct",`. Against the spec:

- **The three fingerprint cases become two tags.** `fp:one-seed` is defined as "one master fingerprint on
  every slot", which collapses the spec's *second* case (one seed at two slots in ONE path) and *third*
  case (one seed across two paths) into a single tag. Neither is separately named, so neither is separately
  assertable. `fp:none` (unkeyed) is a fourth tag the spec does not ask for and does not substitute for
  either missing case.
- **`unseated-slot origins per wrapper` has no tag at all.** Every family template does carry §4f default
  origins, for all four wrappers — but nothing names that, so the required-list loop cannot check it.

**Constructed input and measured result.** Delete the single family entry
`("keyed_compose_wsh_sole_sortedmulti", pl(Wrapper::Wsh, vec![k(2, 3)]), …)` — the only vector in which one
seed fills two (in fact three) slots of ONE path — and re-run the file:

```
$ cargo test --locked -p md-codec --test compose_vectors
test result: FAILED. 5 passed; 1 failed
failures:
    every_compose_vector_in_the_manifest_is_exactly_what_compose_renders   <- the PINNED red only
```

`every_tag_appears_in_at_least_two_vectors` **passed**. The spec's second fingerprint case vanished from
the corpus and the gate said nothing. Every other tag that vector carried (`paths:1`, `sorted`,
`head:bare-multi`, `w:wsh`, `lock:none`, `ik:none`, `fp:one-seed`) still has ≥ 2 carriers.

**Severity: Important.** The test's own comment says it checks "The spec's required tag list, every member
present (spec §12 item 1)"; two members of that list are absent. §12 item 1 is a Stage 0 deliverable
(`STAGED_PLAN` S0: "Delivers (spec §5, §10 items 1 and 3, §12 items 1, 4, 7)"), and the tag mechanism is
the *only* thing standing between the vector family and silent coverage loss for the Go port in S2.

---

## I-2 — §10 item 3's "expected templates" for the presets is a Stage 0 deliverable and is not delivered; the preset tests cannot fail on a wrong archetype

**Plan location:** Task 7, lines 2237-2310 (`presets.rs`) and its tests at lines 2258-2299
(`presets_compose_and_carry_the_documented_shapes`, `presets_refuse_parameters_the_grammar_refuses`);
coverage map at line 2438 (`§4d presets → Task 7`).

**Spec clause — §10 item 3:** "The five presets as **Concrete policies + expected templates** (C2)."
`STAGED_PLAN` S0 names §10 item 3 explicitly among S0's deliverables. The plan's self-review maps only
§4d to Task 7 and never mentions §10 item 3, in either the delivered or the deferred list.

Task 7 produces `PathList`s. No preset's LOWERED TEMPLATE is pinned anywhere (`plain_multisig` is compared
against another `compose` call, not against a literal), and no Concrete policy exists for the §5b lift leg
the brainstorm's §3.7 correction says the archetype goldens carry over as
("They carry over as PRESETS (the same spend conditions) and as validity/lift vectors").

**Constructed input and measured result.** Two independent mutations of `presets.rs`, both of which change
the archetype a preset produces, and neither of which any test notices:

1. `kofn_recovery`: recovery tier `ks(1, 1)` → `ks(2, 2)`.
   `presets::kofn_recovery(Wrapper::Tr, 2, 3, 52560)` now lowers to a 2-of-2 recovery tier. The test asserts
   `p.paths[0].keys` and `p.paths[1].lock` and never looks at `p.paths[1].keys`.
2. `tiered_recovery`: move the lock from tier 2 to tier 1 —
   `vec![ks(k1,n1), tier2_locked]` → `vec![tier1_locked, ks(k2,n2)]`. This inverts the archetype: the
   PRIMARY quorum becomes timelocked and the RECOVERY quorum becomes immediately spendable. The test's only
   assertion on this preset is `assert_eq!(p.paths.len(), 2)`.

```
$ cargo test --locked -p md-codec --test compose_lowering        # both mutations applied
test result: ok. 38 passed; 0 failed; 0 ignored
```

A single pinned expected template per preset — which is exactly what §10 item 3 asks for — fails on both.

**Severity: Important.** A spec requirement in Stage 0's declared scope is not implemented, and its absence
is measurable: the preset half of Task 7 has no test that can fail on a wrong spend-condition shape,
including one that inverts which tier is timelocked.

---

## I-3 — `presets::decaying_multisig` cannot express its archetype: both quorum tiers are forced to the same k-of-n

**Plan location:** line 2370,
`pub fn decaying_multisig(wrapper: Wrapper, k: u8, n: u8, older1: u32, older2: u32, after_height: u32)`,
whose body builds `t1 = ks(k, n)` and `t2 = ks(k, n)`.

**Spec clause — §4d:**

> The five toolkit archetypes — simple-timelocked-inheritance, kofn-recovery, tiered-recovery,
> hashlock-gated, **decaying-multisig** — are offered as one-tap presets that POPULATE a path list the
> operator then edits (§9 item 5) … They are **the same spend conditions as `mnemonic build-descriptor`'s
> goldens** but NOT byte-identical to them.

The toolkit's archetype (`mnemonic-toolkit/crates/mnemonic-toolkit/src/descriptor_builder/archetype.rs:121`)
is summarised in its own registry as:

> "k-of-n multisig that **decays to a smaller recovery quorum** and finally a single key as timelocks expire"

and takes `--threshold`/`--key` **and** `--recovery-threshold`/`--recovery-key` as separate required
parameters (`:121-133`), with a gate at `:301-340` refusing tiers that do not unlock progressively later.

**Constructed input and measured result.** The plan's own test call:

```
presets::decaying_multisig(Wrapper::Wsh, 2, 2, 1000, 2000, 4_000_000)
→ wsh(or_i(and_v(v:multi(2,@0/<0;1>/*,@1/<0;1>/*),older(1000)),
           or_i(and_v(v:multi(2,@2/<0;1>/*,@3/<0;1>/*),older(2000)),
                and_v(v:pkh(@4/<0;1>/*),after(4000000)))))
```

Tier 2 is 2-of-2, identical in threshold and size to tier 1. There is **no parameter** on the function that
could make the recovery quorum smaller, so the archetype's defining property — the decay — is
unreachable. Contrast the plan's own `tiered_recovery`, which correctly takes `(k1, n1, k2, n2)`; the
omission looks like a slip rather than a decision, and nothing in the plan records a decision to reduce it.

Second, weaker half of the same slip: nothing enforces `older2 > older1`.
`presets::decaying_multisig(Wrapper::Wsh, 2, 2, 2000, 1000, 4_000_000)` composes successfully and yields
`older(2000)` on tier 1 and `older(1000)` on tier 2 — tier 2 opens first, so tier 1 is dead weight. The
toolkit refuses this ("decaying-multisig requires --recovery-older > --older: tiers must unlock
progressively later"). §4d does not itself require the check, so this half is informational.

**Severity: Important.** One of the five archetypes §4d names cannot be produced by the preset that carries
its name.

---

## I-4 — the §5b cross-check is never run over the vector family; three hand-picked shapes stand in for 28

**Plan location:** Task 4, lines 1399-1580 (`compose_crosscheck.rs`) — four tests over exactly three path
lists (`two_path(Wsh)`, `two_path(Tr)`, and one keyless wsh list). Task 5's `family()` (line 1643) is never
reached by any cross-check assertion; `compose_vectors.rs` compares text only.

**Spec clauses.** §5b: "**For every composable list** the emitted template: parses in its context; passes
`sanity_check` (keyless wsh paths via `ExtParams::top_unsafe()` …); survives `md encode` → `md decode`
byte-identically (C1); and, for every family WITH a key in every path, `lift()`s to the same semantic
policy as `md compile` …". §12 item 1, per vector: "… **The §5b cross-check holds**; the Go builder
reproduces every template, every CHUNK and every address byte for byte".

**Constructed input and measured result.** I wired the §5b sanity leg to the family myself (substituting 32
distinct xpubs derived from the journey base key into each `template_with_origins` string, then
`Descriptor::<DescriptorPublicKey>::from_str` + `sanity_check()`):

```
keyed_compose_wsh_sole_sortedmulti …………… from_str OK  sanity=Ok(())
… (20 more keyed vectors) ……………………………… from_str OK  sanity=Ok(())
compose_wsh_keyless_hash_path ……………………… from_str OK  sanity=Err("All spend paths must require a signature")
compose_wsh_keyless_hash_only ……………………… from_str OK  sanity=Err("All spend paths must require a signature")
compose_wsh_eight_paths / compose_tr_seven_leaves / compose_wsh_thirty_two_slots /
compose_tr_thirty_two_slots ……………………………… from_str OK  sanity=Ok(())
```

I also derived a live address for all 22 keyed vectors through the real `md address --template` (every one
succeeded, `bc1q…`/`bc1p…`/`3…` as appropriate) and encoded all 28 through the real `md encode` (28 ok).

So the requirement **holds today** — every family template passes the leg, and the two exceptions are
exactly the keyless-wsh carve-out §5b names. The defect is that nothing in the plan keeps it holding: a
vector added in S2 to close a Go-side gap, or a lowering change that makes a leaf sigless, passes every
test the plan writes. Under this project's closure rule a gate that is never run over its stated input is
a hypothesis, and the vector family is the cross-language contract the Go port is measured against.

**Severity: Important.** §12 item 1's cross-check clause is a Stage 0 deliverable
(`STAGED_PLAN` S0 names "the §5b cross-check") and is implemented for three shapes rather than for the
family the same acceptance item defines. Remedy is one loop; a responder who disagrees has my measurement
above to downgrade on.

---

## M-1 — the `main.rs` fragment names an enum that does not exist (`Commands`; the enum is `Command`)

**Plan location:** line 2187 ("Add to `crates/md-cli/src/main.rs`, in the **`Commands`** enum directly
after the `Compile { .. }` variant"), line 2209 ("directly after the **`Commands::Compile { .. }`** arm"),
and line 2212 (`Commands::Compose { wrapper, paths, experimental, json } => {`).

**Measured:** `crates/md-cli/src/main.rs:95-96` is `#[derive(Debug, Subcommand)] enum Command {`, and every
dispatch arm reads `Command::…` (e.g. `:946` `Command::Compile {`). `grep -rn "Commands" crates/md-cli/src/`
returns only an unrelated `Emit::Commands` variant in `cmd/decompose.rs`.

This fragment is explicitly outside the build gate's reach ("NOT covered … the `main.rs` clap variant and
dispatch arm"), so it reaches the implementer unchecked, and with it the plan's own claim that Task 6 wires
the subcommand.

**Severity: Minor.** It cannot produce a wrong result — it fails to compile the instant it is pasted.

---

## M-2 — the presets' lock refusals name the wrong path

**Plan location:** line 2321, `fn blocks(b: u32) -> Result<Lock, ComposeError>`, which returns
`ComposeError::LockOutOfRange { path: 0, why: … }` unconditionally, and is called for tier-2 and tier-3
locks by `simple_timelocked_inheritance`, `kofn_recovery`, `tiered_recovery`, `hashlock_gated` and
`decaying_multisig`.

**Spec clause — §11:** "Every refusal in §4e, §6a, §6b, §6c, §7d and §7g **names what to do instead** and
prints no encoding." A refusal that names the wrong tier sends the operator to edit a path that is fine.

**Constructed input and measured result:**

```
presets::decaying_multisig(Wrapper::Wsh, 2, 2, 1000, 70_000, 4_000_000)
  → "path 1: older in blocks needs 1..=65535"      (the bad lock is on path 2)
presets::tiered_recovery(Wrapper::Wsh, 2, 2, 2, 3, 70_000)
  → "path 1: older in blocks needs 1..=65535"      (the bad lock is on path 2)
```

The plan's own test only asserts `presets::kofn_recovery(Wrapper::Wsh, 2, 3, 70_000).is_err()`, so it
cannot see this. (`compose::validate`'s own `LockOutOfRange` path index is correct — probe:
`tr [1of1, lock-only]` → `LockOnlyPath { path: 1 }` → "path 2 …". Only the presets helper is wrong.)

**Severity: Minor.** Wrong text in a correct refusal.

---

## M-3 — the "renderer is the authority" step lets a NORMATIVE expectation be rewritten to match the implementation

**Plan location:** line 1181 (Task 2 Step 4): "If a rendered string differs from an expectation ONLY in a
spelling the renderer owns (e.g. how divergent origins are inlined), the renderer is the authority: fix the
test string, note it in the commit, and carry the corrected spelling into the spec's §5 vectors." And line
1382 (Task 3 Step 4): "Same renderer-authority rule as Task 2 **for any string mismatch**."

**Spec clause — §5:** "It is defined in Rust first (§10) and ported to Go; **the two must produce
byte-identical templates for every composable list**." The plan's own file header repeats it: "Every
expected template string below is the FIXED spelling the Go port must reproduce byte for byte; a change
here is a normative change."

Task 2's qualifier ("ONLY in a spelling the renderer owns") is the right rule; Task 3's restatement widens
it to "any string mismatch", and Task 3 is where a mismatch would come from the TREE (spine associativity,
`multi_a` vs `sortedmulti_a`, internal-key extraction), not from the renderer. I could not construct an
input that exercises this — the plan's expected strings were generated from the lowering and all pass — so
it stays Minor by the brief's own rule.

**Severity: Minor.** No constructed input; a process instruction that would ratify a wrong tree shape if
one ever appeared.

---

## M-4 — the plan's coverage map contradicts `STAGED_PLAN` on §12 item 4

**Plan location:** line 2438, self-review item 1: "… §12 items 1 and 7 (device half is Stage 2) → Task 5
tagged coverage and the lock tests. **Not in this stage by design:** … §12 items **2-6**, 8-11 (Stages 2-4)."

`STAGED_PLAN_wallet_policy_composer.md` S0 says: "**Delivers (spec §5, §10 items 1 and 3, §12 items 1, 4,
7)**". The plan therefore defers an item the staging document assigns to it.

In fact the Stage-0-reachable half of §12 item 4 IS covered by Tasks 1-3 — I checked each: every §4e
refusal, the §4c bands in and out per kind (including `older(0x400000)` via
`Lock::OlderUnits(0).operand().is_err()`), the 33rd slot, and the §8v one-fingerprint case. So this is a
bookkeeping contradiction, not a hole. But it means a reader of the plan cannot tell that §12 item 4 was
considered, and a future re-validation would look for it in Stage 2.

**Severity: Minor.**

---

## M-5 — a repo follow-up whose owning phase is *this stage* has no task in the plan

**Repo location:** `descriptor-mnemonic` `design/FOLLOWUPS.md`, entry
`md-encode-keyless-template-sigless-path-not-gated`, filed at `480e54fe` — one commit past the plan's
recorded baseline `3b0944fb`, by this cycle's own author. Its owning phase, verbatim:

> owning phase: **next md-cli patch, alongside `md compose` (composer Stage 0), which gates it correctly**

**Measured on the repo's built `target/release/md` 0.14.0:**

```
$ md encode "wsh(or_i(pkh(@0/48'/0'/0'/2'/<0;1>/*),sha256(a8…a8)))"   # no --experimental
exit=0
md1yq fdsss j5qqc ye9hd g4z52 … (stderr: group size / separator / "keyless descriptor template")
```

`md compose` will refuse that same shape without `--experimental` (Task 6), then print an artifact that
`md encode` accepts with no flag and no warning — the EXPERIMENTAL gate is exactly one command deep.
Nothing in Tasks 1-8 touches `md encode`.

Per the constellation rule "Phase-owned or gating → burn down in/before that phase closes green", this is
S0's to carry or to re-schedule explicitly. It is a staleness item, not an authoring defect: the follow-up
postdates the plan commit.

**Severity: Minor.**

---

## N-1 — a test named for a property it never checks

Line 1798, `keyed_compose_vectors_carry_four_journey_keys_or_fewer`, asserts `!v.keys.is_empty()` and
`v.keys.len() == v.fingerprints.len()`. It never asserts `v.keys.len() <= 4`. (The bound does hold — I
checked every keyed family entry: max 4 slots — so nothing is wrong today.)

## N-2 — `unsorted` is silently discarded where sorted was not legal

`md compose --wrapper wsh --path 2of3,unsorted --path 1of1,older=10` and the same command without
`unsorted` produce **byte-identical** output, with no note on either stream (measured through the library:
`wsh [2of3 unsorted, 1of1 locked]` → `wsh(or_d(multi(2,…),and_v(v:pkh(@3),older(10))))`, `experimental` is
`[]`). Suppressing the §8b confirm there is correct per §5a ("the §8b confirm fires ONLY where sorted was
legal and declined, never on a lowering-forced `multi`"), but the DSL accepts and drops a request the
operator typed. Documentation-only under the journey-walk classification.

## N-3 — a Unix-time `after` typed without the `t` suffix refuses with a message that never names the suffix

`--path 1of1,after=1893456000` parses as `Lock::AfterHeight(1_893_456_000)` and refuses with
"after height needs 1..=499999999" (line 2130 in `parse_path`, and `Lock::operand`'s wording). The DSL's
own remedy — `after=1893456000t` — appears nowhere in the message. §11's "names what to do instead" is a
device/refusal rule, and this is a host DSL, so it is a Nit.

---

## Attacks tried that found nothing

Each of these is a construction I ran against the plan's assembled code (or the real `md`) and that came
back matching the spec.

**§5 wsh row.**
- `or_d` vs `or_i` head dispatch: `[2of3, 1of1+older]` → `or_d(multi(2,…),…)`; `[1of1, 1of1+older]` →
  `or_i(pkh(@0),…)`; `[2of3+older, 1of1]` → `or_i(and_v(v:multi…),pkh)`; `[2of3+hash, 1of1]` → `or_i`.
  `is_bare_multi` requires n ≥ 2, unlocked, unhashed — exactly the spec's predicate.
- Bare-multi head with an unsorted flag still takes `or_d` (`[2of3 unsorted, 1of1+older]`) — correct, the
  spec's `or_d` predicate does not mention sortedness and a non-sole multi is `multi` regardless.
- `or_d(multi, multi)`: `[2of3, 2of3]` → `wsh(or_d(multi(2,@0,@1,@2),multi(2,@3,@4,@5)))`, sane.
- Right associativity and last-path-alone at 8 paths: 7 `or_i`, terminal `and_v(v:pkh(@7),older(107))`.
- A keyless path as the HEAD: `[keyless(H,after), 1of1]` → `or_i(and_v(v:sha256(H),after(N)),pkh(@0))`;
  `[keyless(H), 2of3]` → `or_i(sha256(H),multi(2,…))` — the bare-`sha256(H)` form §5 requires.
- Two keyless paths in one list: both marked `KeylessPath`, `or_i` nesting correct.

**§5 tr row.**
- Internal key = first-listed bare single, at every position: index 0, index 1, index 7 of 8 — each time
  extracted, numbered `@0`, and removed from the leaf set.
- Only the FIRST bare single is extracted: `[2of2, 1of1, 1of1]` → `tr(@0,{multi_a(2,@1,@2),pk(@3)})`.
- Spine depths: m=1 bare leaf; m=2 `{P1,P2}`; m=3 `{P1,{P2,P3}}`; m=4 `{P1,{P2,{P3,P4}}}`; m=7 and m=8
  — every one matches depth `min(j, m−1)`.
- m=0 → `tr(@0/<0;1>/*)`, no tree, `is_nums=false`.
- NUMS is byte-identical across spec §5, plan, and `md-codec/src/nums.rs`
  (`50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0`).
- **The brief's named case — a sole leaf that exists only after extraction:** `tr [2of3 sorted, 1of1]` →
  `tr(@0,sortedmulti_a(2,@1,@2,@3))` with `internal_key_path = Some(1)`. Correct: the leaf is sole, so
  sorted is legal. `tr [1of1, 2of3 unsorted]` → `multi_a` + `Experimental::UnsortedKeys(1)`. Correct.
- Sorted NOT legal once m ≥ 2: `tr [1of1, 2of2 unsorted, 1of1+older]` → `multi_a`, `experimental` empty —
  the "never on a lowering-forced `multi`" clause of §5a holds.
- `pkh` in wsh / `pk` in tr (C17) on every single-key path I built.

**Conjunct order (§5 "inside a path").** keys→hash→lock in every combination, dropping absent parts:
`and_v(v:multi,and_v(v:sha256,after))`, `and_v(v:pkh,older)`, `and_v(v:pkh,sha256)`,
`and_v(v:sha256,older)`, bare `sha256`.

**§4c bands (`Lock::operand`).** In-band boundaries 1/65535 blocks, 1/65535 units, 1/499,999,999 height,
500,000,000/2,147,483,647 time — all accepted with the right tag and operand
(`OlderUnits(1)` → `0x0040_0001`). Out-of-band 0 blocks, 0 units (= `older(0x400000)`, the filed md defect),
0 and 500,000,000 height, 499,999,999 and 2,147,483,648 time — all refused. The `u16` types make
`older` ≥ 65536 unrepresentable, so the band is total.

**§4e refusals.** Empty list, 9 paths, no keyed path, lock-only path, keyless under `tr`, `k=0`, `k>n`,
`n=10`, 33 slots (admits exactly 32), `sh`/`sh(wsh)` with n=1 / two paths / a locked path / an unsorted
path. I specifically checked whether the plan refuses a LEGAL input by rejecting `sh(2-of-3 unsorted)`,
since §4e's row text alone ("anything other than ONE unlocked, unhashed path whose key set has n ≥ 2")
would admit it — but §4a ("a `sortedmulti`") and §4e's own note ("the legacy wrappers are sorted-only, so
the §8b confirm is never offered under them") settle it the plan's way. Not a finding.

**§4f origins.**
- Ascending default accounts under each wrapper's script type: wsh/sh `2'`, sh(wsh) `1'`, tr `3'`.
- One slot → `PathDeclPaths::Shared`, several → `Divergent`, indexed by EMITTED slot.
- **The brief's named case — a default colliding with a declaration:** `compose_with(wsh 2-of-2,
  [None, Some(m/48'/0'/0'/2' + fp)])` → the unseated slot takes account **1**, not 0, and the declared slot
  keeps account 0. `taken` is seeded from every declaration before the fill loop, so a default can never
  collide with a declaration; the pairwise invariant then runs over the resolved list regardless of origin.
- The invariant: neither fingerprinted → refused; one fingerprinted → refused; two distinct → admitted
  (`Shared`); two identical fingerprints at one origin → refused.
- An arbitrary declared origin (`m/7`, unhardened, one component) is carried verbatim, per §4f's
  "carries the origin the record or card DECLARES, verbatim".

**Numbering and the wire.** `canonicalize_placeholder_indices` is the identity on every shape I built
(NUMS tr, extracted-ik tr, 8-path wsh, 32-slot wsh and tr, keyless wsh). `encode_payload`, `split` and
`reassemble` round-trip byte-identically for all of them.

**`template_with_origins`' `@1` vs `@10` claim.** Verified at n = 32 in both wrappers: every one of
`@0`..`@31` received its own origin and no `@1x/` was corrupted by the `@1/` pass.

**The whole family through the real toolchain.** All 28 `template_with_origins` strings encode through
`md encode` (exit 0); all 22 keyed ones derive a mainnet address through `md address --template`; all 28
pass the §5b sanity leg (with the two documented keyless exceptions); `md encode → md decode` returns the
origin-less renderer form, which is what the plan's `compose_output_round_trips_through_encode_and_decode`
assumes. `md encode`'s unbroken md1 goes to **stdout** and the grouped form to stderr, so the plan's
`lines().filter(|l| l.starts_with("md1") && !l.contains(' '))` filter is correct.

**Tag arithmetic.** I hand-counted every tag in `family()` against the two-vector rule: all ≥ 2 except
`spine:0`, which is correctly declared `SINGULAR` with the spec's own justification ("m = 0 is one unlocked
single key and nothing else"). Vector count 28, keyed count 22 — both match the plan's stated expectations.

**Presets (the four not named in I-3).** `plain_multisig`, `simple_timelocked_inheritance`,
`kofn_recovery`, `tiered_recovery` and `hashlock_gated` all reproduce their toolkit archetype's SPEND
CONDITIONS (which path is locked, which keys are fresh), including `hashlock_gated`'s `andor(pk(A),
sha256(H), and_v(v:pk(B), older(N)))` becoming `[1-of-1 + hash, 1-of-1 + older]`. Their differing FRAGMENT
spellings (`or_i(pkh…)` where the toolkit wrote `or_d(pk…)`, `or_d` where it wrote `or_i`,
`and_v(v:multi…)` where it wrote `thresh`) are exactly what §4d and C21/C23 require. Every preset composes,
and every preset refuses a legacy wrapper it does not fit.

**DSL.** `2of3`/`keyless` heads; `older=N` / `older=Nu` / `after=H` / `after=Tt` / `sha256=<64 lc hex>` /
`unsorted`; double lock, double hash, `unsorted` on a keyless path, uppercase hex, `older=65536`,
`older=4194305` — each refused with a message naming the band. No spelling I found lets an input past the
grammar that the library would then admit.

---

## What I ran

```
# read-only, mnemonic-engrave
git log --oneline -1; wc -l design/{IMPLEMENTATION_PLAN_composer_S0_md_compose,SPEC_wallet_policy_composer,
    BRAINSTORM_wallet_policy_composer,STAGED_PLAN_wallet_policy_composer}.md
sed/grep over the plan, the spec (§4, §5, §8, §10, §11, §12), the brainstorm (§2, §3.2, §3.7, §3.8)
    and the staged plan

# read-only, descriptor-mnemonic (repo HEAD 480e54fe)
git log -3 --format='%H %s'; git show 480e54fe
sed/grep over crates/md-codec/src/{validate,to_miniscript,test_vectors,tree,nums}.rs,
    crates/md-cli/src/{main.rs,cmd/vectors.rs,cmd/encode.rs,parse/template.rs}
./target/release/md --version | encode | decode | address --template     (existing binary, no build)

# read-only, mnemonic-toolkit
sed/grep over crates/mnemonic-toolkit/src/descriptor_builder/archetype.rs

# PRIVATE scratch copy only (TMPDIR=…/scratchpad/lensA, CARGO_TARGET_DIR=…/.plan-gate-target-lensA)
bash scripts/plan-build-gate-md.sh design/IMPLEMENTATION_PLAN_composer_S0_md_compose.md
cargo test --locked -p md-codec --test compose_probe        -- --nocapture   # ~20 constructed lists
cargo test --locked -p md-codec --test compose_probe2       -- --nocapture   # or_d chains + presets
cargo test --locked -p md-codec --test compose_sanity_probe -- --nocapture   # §5b sanity over 28 templates
cargo test --locked -p md-codec --test compose_vectors      print_family --nocapture
cargo test --locked -p md-codec --test compose_vectors      # mutation: family entry deleted
cargo test --locked -p md-codec --test compose_lowering     # mutation: two presets rewritten
```

No file in `mnemonic-engrave`, `descriptor-mnemonic` or `mnemonic-toolkit` was modified. `/tmp/plan-build-gate-md`
was not touched. No `.jsonl` file was read.
