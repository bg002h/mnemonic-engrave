# R0 architect review — `IMPLEMENTATION_PLAN_f449_stage2_compose.md`

**Verdict: NOT ready for implementation.**
**Counts: 3C / 7I / 4M / 2N.**

Reviewed against `design/SPEC_liana_unspendable_internal_key.md` (r5, binding),
`design/RECON_f449_stage2.md`, and `descriptor-mnemonic` main `25acb33c`
(clean tree, verified `git status --porcelain` empty before and after every
patch below; all patches reverted with `git checkout --`).

Everything below that says MEASURED was run on this box at `25acb33c` with
`target/debug/md` built from that tree, or with the prebuilt Liana v15.0
harness at `/scratch/code/shibboleth/.tmp/liana-harness-target/debug/liana-harness`.
I did not re-derive anything the brief listed as already machine-checked.

---

## Summary of the scope question (brief Q1)

Three of SPEC §9's four stage-2 items: **two are genuinely done, one is
half-done, and one stage-2-owned acceptance row is missing from the plan
entirely.**

| §9 / §8.9 item | recon says | measured |
| --- | --- | --- |
| `md compose --unspendable` | ABSENT | ABSENT — correct, this is the work |
| `md descriptor` kind 1 | DONE 1b | **DONE** — `crates/md-cli/tests/liana_kind1_cmd_descriptor.rs`, exactly 2 `#[test]`s |
| `UNSPENDABLE(liana)` substitution rule | DONE 1b | **DONE** — `md encode 'tr(UNSPENDABLE(liana),…)'` parses, RUN |
| JSON schema bump (§4a) | DONE 1b | **HALF DONE** — see I-4 |
| §8.9 row "`md repair`", stage **2** | not mentioned | **UNSCHEDULED** — see I-2 |

Nothing the plan schedules is already finished.

---

# CRITICAL

## C-1 — `--unspendable liana` on `plain-multisig` composes at exit 0 and emits a template `md encode` refuses, while `md descriptor` still renders a concrete fundable descriptor for it

The plan's central premise (`Architecture`, line 7) is that
`compose/tr.rs:47`'s `None => InternalKey::NumsPoint` is *"exactly one decision
site"* and that selecting `LianaUnspendable` there is *"the ONLY behavioural
line in the task"* (Task 1 Step 6). That is true of the codec. It is not true
of the CLI, because one of the six presets reaches that site with a tree SPEC
§6 row 1 refuses.

**Reachability, measured.** `compose/tr.rs:39` calls
`path_body(n, true, m == 1 && n.path.is_bare_multi())` — a **`sortedmulti_a`**
leaf whenever there is exactly one leaf and it is a bare multi. With
`presets::plain_multisig` (`compose/presets.rs:34-39`) that is a single
`ks(k,n)` path, which for `n >= 2` is not `is_bare_single`
(`compose/mod.rs:292`), so `ik` is `None` and the internal key is the NUMS
point today:

```
$ md compose --wrapper tr --preset plain-multisig,2of3
tr(50929b74…03ac0,sortedmulti_a(2,@0/48'/0'/0'/3'/<0;1>/*,…))     # exit 0, no --experimental needed
```

It is the only preset that does this (`simple-timelocked-inheritance`,
`kofn-recovery,1of1` extract a real key; the rest have >1 leaf), and it is also
reachable as `md compose --wrapper tr --path 2of3`.

**What stage 2 makes happen.** `--unspendable liana` turns that into
`tr(UNSPENDABLE(liana),sortedmulti_a(…))`. Then:

```
$ md encode "tr(UNSPENDABLE(liana),sortedmulti_a(2,@0/48'/0'/0'/3'/<0;1>/*,@1/…,@2/…))"
md: codec error: wire kind 1 (Liana unspendable internal key) with a sortedmulti_a
leaf is refused: …                                                      # REFUSED
```

**Compose's own read-back does not catch it.** `cmd/compose.rs:704` re-parses
its output with `crate::parse::template::parse_template_ext(...)` — the F-600
guard whose own comment says compose *"was emitting, at EXIT 0, templates `md
encode` refuses"*. But `validate_unspendable_shape` is called **only** from
`encode.rs:230`, inside `encode_payload` under `Admission::Enforce`
(`grep -rn validate_unspendable_shape crates/ | grep -v tests/` → one
non-test caller). `parse_template_ext` (`parse/template.rs:2784-2900`) never
calls it. So the read-back passes and compose exits 0.

**And it is worse than a wall at the next verb.** `md descriptor` uses the same
parse path, so it happily renders a concrete descriptor for the wallet md
refuses to encode:

```
$ md descriptor --template "tr(UNSPENDABLE(liana),sortedmulti_a(2,@0/…,@1/…,@2/…))" --key @0=… --key @1=… --key @2=…
tr(xpub661MyMwAqRbcFJqq8BkVqxydFaTyhmJhSXHrjtFV5LRQyE6n1aLdiRfWun4cF9Y1HDFt7mfNUuXby5RoUxJkWMoxBrzp47DTz4ueXNYLi1J/<0;1>/*,sortedmulti_a(2,…))#ctjy7seq
                                                                          # exit 0
```

That is a pasteable, fundable descriptor for a wallet **that has no md1
encoding at all** — money can go in and no plate can be minted. (Liana also
refuses it independently: no timelocked path, §0a class 3.)

The plan names none of this: not a refusal, not a warning, not a test, not a
line in Global Constraints. The brief's question — *"do any of the six
composer presets produce a `sortedmulti_a`? If so the plan must say what
happens"* — is answered yes, and the plan does not say.

**Remedy.** `md compose` must refuse `--unspendable liana` when the lowering
would produce a `sortedmulti_a` leaf, at compose time, naming the preset and
the reason, before anything is printed. Either call
`md_codec::validate::validate_unspendable_shape` on `composed.descriptor` in
`cmd/compose.rs` right after `compose()` (it is already `pub`), or refuse in
`lower_tr` with a `ComposeError`. Add the refusal to Task 1's tests with
`plain-multisig,2of3` as the fixture.

---

## C-2 — Task 1's regression floor compares the binary to itself, so it cannot fail for the property it names, and Step 9's prescribed mutation does not turn it red

Task 1 Step 1's test is named
`omitting_the_flag_is_byte_identical_to_the_previous_release`. It runs the SAME
binary twice and asserts `a == b`:

```rust
let (a, _, ca) = md(&["compose", …]);                              // default
let (b, _, cb) = md(&["compose", …, "--unspendable", "nums"]);     // explicit
assert_eq!(a, b, …);
```

Nothing in it is anchored to the previous release, to the NUMS hex, or to the
committed corpus. It asserts only that two spellings of the same request agree
— which they do under every mutation that moves them together.

**Constructible counterexample.** Have Step 7's string parser map
`"nums" => UnspendableKind::Liana` (a one-token slip; Step 7 says only *"Parse
it with an explicit error naming both values"*). Then:

- default → `Liana` → `UNSPENDABLE(liana)`;
- `--unspendable nums` → `Liana` → `UNSPENDABLE(liana)`; `a == b` → **PASS**;
- `--unspendable liana` → `Liana` → contains the marker, no NUMS hex, and the
  comma-tail is unchanged → Step 3's test also **PASSES**.

Both new tests are green and **every default `md compose --wrapper tr` on a
NUMS shape now silently emits wire version 8 kind 1** — a different wallet
(different `WalletPolicyId`, `WalletDescriptorTemplateId`, phrase and mk1 stub,
per §8.5) that no shipped device can read.

**The existing suite does not save it either.** The corpus floor
(`crates/md-codec/tests/compose_vectors.rs:17-52`) calls
`md_codec::compose::compose(list)` **directly**, so it pins the *codec*
default, not the CLI's flag parse. And no CLI-level compose test pins the NUMS
hex: `grep -c 50929b74 crates/md-cli/tests/cli_compose.rs
crates/md-cli/tests/cli_compose_preset.rs` → **0 and 0**.
`cli_compose_preset.rs:34` compares a preset against the equivalent `--path`
list, which flips with it.

**Step 9's mutation proof is wrong on its face.** Step 9 says: swap the arms so
`Nums` yields `LianaUnspendable`, and
`omitting_the_flag_is_byte_identical_to_the_previous_release` *"MUST go red"*.
It will not: under that swap the default and `--unspendable nums` both resolve
to `UnspendableKind::Nums` and both render the marker, so `a == b` still holds.
(Step 3's test does catch that particular swap — but Step 9 names the wrong
test, so the floor is never shown able to fail. `mutation-testing-finds-false-passes`,
`a-proof-in-a-transcript-is-not-a-gate`.)

**The Self-Review's admission is inadequate.** *"The one risk worth naming"*
says the floor *"is only as wide as the presets it lists"* and prescribes
widening it. Widening a test that compares a binary to itself gives six
self-comparisons instead of two. Breadth is not the defect; the **anchor** is.

**Remedy (both halves).**
1. Anchor the default arm to a fixed literal, not to a second run:
   `assert!(a.contains("50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0"))`
   and `assert!(!a.contains("UNSPENDABLE(liana)"))`, per preset.
2. Cover all six archetypes under `tr` — `plain-multisig` (after C-1),
   `simple-timelocked-inheritance`, `kofn-recovery`, `tiered-recovery`,
   `hashlock-gated`, `decaying-multisig` — plus one `--wrapper wsh` case
   proving the flag is inert there (see I-3). Note `decaying-multisig`'s
   spelling is `older1=`/`older2=`, not `older=` twice (RUN: `older= given twice`).
3. Rewrite Step 9's mutation to one this test catches — e.g. make the clap
   `default_value` `"liana"`, or map `"nums"` to `Liana` — and paste red/green.

---

## C-3 — Task 4 Step 3 authorises amending a GREEN spec on a premise I measured to be FALSE: Liana v15.0 **accepts** a nested taptree

Task 4 Step 3: *"if none is → record the enumeration as evidence and **amend
SPEC §8.1**, because it asks for something that does not exist."* The RECON
(§"F-640 — nested taptrees look UNOBTAINABLE") supplies the premise from two
refused constructions.

**The premise is false. MEASURED, with a control, in one run.**

Liana's own policy check (`/scratch/code/shibboleth/.tmp/fable-liana-src-v15/liana/src/descriptors/analysis.rs:583-608`)
does `tree.lift()` then `.normalized()` on the whole taptree before inspecting
it. `normalized()` **flattens nested ORs**, so `{A,{B,C}}` and a flat 3-leaf
tree lift to the same `thresh(1, A, B, C)`. Nesting is structurally invisible
to Liana's policy model; what it actually requires
(`analysis.rs:620-665`) is one non-timelocked primary path and ≥1 timelocked
recovery path with **distinct** timelocks.

Both of the RECON's refusals are explained by leaf *content*, not nesting:
`preset-decaying-multisig-tr` has every tier timelocked plus an `after()`
(SPEC §0a already records this as classes 5 and 7 — *no unlocked path*), and
the fresh `{multi_a(2,A,B),{and_v(older),and_v(older)}}` has two recovery paths
whose timelocks collide (`recovery_paths.contains_key(&timelock)` →
`IncompatibleDesc`).

I built the shape that varies content instead of arrangement — a nested
taptree with one unlocked primary and two **distinct** timelocks, which
**md's own composer produces**:

```
$ md compose --wrapper tr --path 2of3 --path 1of1,older=26280 --path 1of1,older=52560
tr(50929b74…,{multi_a(2,@0/…,@1/…,@2/…),{and_v(v:pk(@3/…),older(26280)),and_v(v:pk(@4/…),older(52560))}})
```

rendered it at kind 1 over vendored key material, and fed it to the harness
alongside a known-good control and the RECON's own refused nested case:

```
CONTROL-kofn-recovery                ok=True   recv0= bc1pj6davmeetfe2uutrjlgxcjytq50ggvyd82xtxx2eazm24hsdhlnqxvh4v4
PROBE-nested-two-distinct-recovery   ok=True   recv0= bc1p32ny335apyeq787eqe6r0wyt5hfjkkkmhtw68ce5kav4my7xtvestf3grq
CONTROL-nested-decaying-refused      ok=False  Descriptor is not compatible with a Liana spending policy.
```

Liana returned `is_taproot: true`, primary = the 2-of-3 multi (the unspendable
internal key correctly excluded), and **two recovery paths at `older` 26280 and
52560, inferred from a nested taptree**.

**Ruling on the brief's question.** "Amend the spec" is a legitimate move in
general — an acceptance item no input can satisfy is a spec defect, and the
project's own doctrine (*a refusal that cannot fire is not a guard*) supports
retiring one. It is the wrong move **here**, for three reasons:

1. **The premise is false.** §8.1 is satisfiable and now satisfied. The
   remedy is a vector and `F-640 CLOSED`, not an amendment.
2. **Four probes cannot establish non-existence.** Step 3's enumeration
   ("nested left, nested right, depth 3, a nested pair of recovery tiers")
   varies the *arrangement*, which `normalized()` erases, and holds the
   *content* fixed — so all four can fail for reasons that say nothing about
   nesting, and the plan converts that into "does not exist". Where a fact is
   readable from the source that produced the refusal, it must be read, not
   sampled (`naming-a-risk-is-not-managing-it`).
3. **Even a true unsatisfiability would not license this shape of amendment.**
   §8.1's nested item is the gate on the recipe's *traversal order*, and that
   half is already delivered — `crates/md-codec/tests/liana_unspendable.rs:102-140`
   pins recipe agreement across `kofn-recovery`/`tiered-recovery`/`decaying-multisig`
   over the same four keys (I confirmed the nested refused case's
   `expected_xpub` equals the flat cases' and equals its own descriptor's
   embedded internal key). Deleting the row wholesale would delete that gate
   too. An amendment would have to *narrow*, not remove — and would have to go
   through the spec's own review loop, in its own commit, after the evidence is
   persisted, not be folded by the same author in the same step as the
   measurement.

**Remedy.** Replace Step 3's branch with: probe by varying *leaf content* (one
unlocked primary, ≥2 distinct timelocks) rather than nesting arrangement;
vendor the accepted nested shape as a case; close F-640 with the evidence
file. Delete the "amend SPEC §8.1" branch. Cite `analysis.rs:583-608` in the
plan so the next reader does not re-sample a question the source answers.

---

# IMPORTANT

## I-1 — the plan spans TWO repositories and never says so; Task 5's follow-up work targets a file that does not contain the follow-ups

`Baseline revision` names only `descriptor-mnemonic 25acb33c`, and every path
is written repo-relative. Measured:

| path in the plan | actually in |
| --- | --- |
| `crates/md-codec/…`, `crates/md-cli/…`, `fuzz/` | descriptor-mnemonic |
| `scripts/phase-gate.sh` (Task 1 Step 10) | descriptor-mnemonic **only** |
| `scripts/followups-status.sh` (Task 5 Step 5) | mnemonic-engrave **only** |
| `scripts/plan-api-check.sh` (Self-Review) | mnemonic-engrave **only** |
| `scripts/liana-live-gate.sh` (Task 4, create) | harness is in mnemonic-engrave |
| `design/evidence/f449-stage2/` (Task 4) | mnemonic-engrave (`design/evidence/` exists only there) |
| `design/FOLLOWUPS.md` (Tasks 4, 5, 6) | **both repos have one** |
| `harnesses/liana` | mnemonic-engrave |

`descriptor-mnemonic/design/FOLLOWUPS.md` exists and contains **none** of
F-636 / F-638 / F-640 / F-641 (`grep` → empty); they live at
`mnemonic-engrave/design/FOLLOWUPS.md:19327, :19461, :19509, :19541`. An
implementer working in the repo the Baseline names will write Task 5 Step 5's
`Status: CLOSED` lines into the wrong file and `scripts/followups-status.sh`
will not exist. Four of six tasks touch mnemonic-engrave.

**Remedy.** State both baselines (mnemonic-engrave is at `49964db1`), and
prefix every path with its repo. The plan's own `File Structure` section is the
right place.

## I-2 — SPEC §8.9 assigns an acceptance row to **stage 2** that the plan schedules nowhere and does not list as deliberately absent

`SPEC …:724`:

> | §6a `md repair` | a v8 chunk with a correctable BCH error KEEPS the correction and reports an unsupported wire version, distinctly from the atomic-fail exit | **2** |

The defect is real and confirmed in source: `crates/md-cli/src/cmd/repair.rs:88-96`
returns `Ok(2)` on **any** `decode_with_correction` error and drops `details`
— the successful corrections — on the floor; exit 2 is the atomic-fail code
(`repair.rs:76-80` doc comment). §6a's own row calls this *"discarding the
successful correction"*.

The plan's Self-Review enumerates spec coverage (`§9 stage 2's four items… §6
row 3 → Task 2. §8.2 → Task 3. §8.8 → Task 4. §8.1 → Task 4 Step 3`) and
lists what is *"Deliberately absent, owned elsewhere"*. This row appears in
neither. §8.9's stated purpose is *"a rule with no gate is not a rule"*, and
this is the gate.

**Remedy.** Add it as a task (it is small: distinguish `WireVersionMismatch`
from BCH-capacity failure and keep `details` in the former case), or move the
row to a later stage **in the spec**, with the move recorded in Task 6's sweep.
Silence is the one option the §8.9 table forbids.

## I-3 — `--unspendable liana` is a silent no-op under `wsh`/`sh`/`sh-wsh`, and Task 2's trigger cannot fire there

`compose/lowering.rs:293-299`:

```rust
match list.wrapper {
    Wrapper::Tr => super::tr::lower_tr(list, declared),
    Wrapper::Wsh | Wrapper::Sh | Wrapper::ShWsh => { … }
}
```

Only `Wrapper::Tr` reaches `lower_tr`, so Task 2 Step 3's trigger
(`ik.is_some() && unspendable == Liana`, evaluated *inside* `lower_tr`) is never
evaluated for a non-`tr` wrapper. `md compose --wrapper wsh --preset
kofn-recovery,2of3,older=26280 --unspendable liana` will exit 0, emit a plain
`wsh(...)` (RUN, today, minus the flag), and say nothing. §6 row 4 refuses kind
1 under `sh`/`wsh` at encode; the composer would silently drop the request
instead.

**Remedy.** Refuse `--unspendable liana` with a non-`tr` wrapper in
`cmd/compose.rs` before `compose()` is called (a wrapper/flag contradiction,
not a lowering outcome), and add the case to Task 1's test list.

## I-4 — §4a's `md compose --json` third state is NOT shipped; the recon/plan mark the §4a JSON item done

SPEC §4a, *"Published contract change"*: *"`md decode --json`'s `is_nums: bool`
… **and `md compose --json`'s internal-key field** need a third state. This is
a breaking change to a published v1 schema and must be versioned as one."*

1b shipped the **decode** half: `format/json.rs:390-396` emits an
`unspendable_kind` alongside `is_nums`/`key_index`, and `SCHEMA` is
`"md-cli/2"` (`json.rs:19`). The **compose** half cannot have shipped in 1b —
compose could not produce kind 1 — and it has not:

```
$ md compose --wrapper tr --preset kofn-recovery,2of3,older=26280 --json
  … "internal_key_path": null, "schema": "md-cli/2", …
```

`cmd/compose.rs:735-743` emits exactly one internal-key field,
`internal_key_path` (`Option<usize>`), which is `null` for NUMS **and will be
`null` for kind 1**. A `--json` consumer cannot distinguish the two wallets
except by string-matching `UNSPENDABLE(liana)` inside `template` — which is
precisely the "structural pattern-match instead of the real check" weakening
§8.10 calls out. The recon's row and the plan's Self-Review both record this
item as done, so nobody is scheduled to notice.

**Remedy.** Add the field to Task 1 (e.g. `"unspendable_kind": "nums"|"liana"`,
or `null` when a real key is extracted), assert it in Task 2 Step 7's `--json`
test, and record in Task 6 whether `md-cli/2` covers it or a further bump is
owed.

## I-5 — Task ordering: Task 4 Step 3 adds an ACCEPT case to the corpus Task 3 has already pinned

Task 3 Step 1/2 pin *"each vendored case whose recorded Liana verdict is
ACCEPT"* and *"the same 24"* addresses. Measured at `25acb33c`:
`crates/md-codec/tests/fixtures/liana/cases.json` holds 8 cases, 4 with
`accepted: true`, each with 3 `liana_receive` + 3 `liana_change` = **24**. Both
legs already hold — I ran them: descriptor equality 4/4 byte-exact including
checksum, address equality **24/24** (via `md decompose --emit template` +
`--emit commands` → `md descriptor` / `md address --chain {0,1} --count 3`).

But Task 4 Step 3 says *"if ANY nested shape is accepted → **vector it**"*, and
C-3 shows one will be. Vectoring it makes 5 ACCEPT cases and 30 addresses,
after Task 3 has committed tests written for 4 and 24. The plan orders Task 3
before Task 4 and says nothing about the interaction — and
`scripts/vendor-liana-evidence.sh` (descriptor-mnemonic) regenerates
`cases.json` from mnemonic-engrave's `composer-fable-r0` JSONL only, so a
newly-probed case needs a new vendoring path that neither task defines.

**Remedy.** Run Task 4's probe before Task 3, or state that Task 3's tests
enumerate the corpus (never a literal count) and that Task 4 extends
`vendor-liana-evidence.sh` to carry a stage-2 evidence file.

## I-6 — `--unspendable liana` on `hashlock-gated` and `decaying-multisig` succeeds and hands the operator a wallet no coordinator imports and no shipped device reads

Both reach the decision site (measured: `md compose --wrapper tr --preset
hashlock-gated,sha256=<64hex>,older=100` and `--preset
decaying-multisig,2of3,1of2,older1=100,older2=200,after=800000` both emit the
NUMS internal key today), so `--unspendable liana` will apply. Unlike C-1
nothing refuses them: they encode at wire version 8 and render fine. And per
SPEC §0a they are Liana-unimportable for reasons no encoding change touches
(class 4 *a hash lock*; class 5 *an absolute lock* then 7 *no unlocked path*).

So the operator asks for Liana compatibility and receives a **strictly worse**
plate than the default: not importable by Liana, and unreadable by every
device before this cycle's firmware. That is worse than telling them nothing.
Note the device is scheduled to be more careful than the host here: §0b's
choice screen fires on exactly `kofn-recovery` and `tiered-recovery` and is
gated on all six presets (§8.9, stage 4) — the host has no counterpart, so the
two surfaces will disagree about which shapes are offered, with the host the
permissive one.

**Remedy.** Warn (do not refuse — the shape is legal) when `--unspendable
liana` is applied to a tree Liana's model cannot admit, reusing Task 2's
stderr vocabulary and naming the class. The predicate is cheap: a leaf carrying
a hash, or no unlocked leaf.

## I-7 — §8.8 is a REQUIRED acceptance item and Task 4 lets it skip, with nothing saying the stage may not close on a skipped run

Task 4 Step 1: *"A missing checkout must be a clear skip-with-reason, never a
silent pass."* Loud is right, but a loud skip still lets stage 2 close with
§8.8 — *"A live `harnesses/liana` install run at v15.0 — **REQUIRED, not a
bonus**. It is the only measurement of the install path for any unspendable
shape"* — never having executed. That is the exact shape
`closure-is-lens-closure` was written against: *"a plan may not close while any
of its own gates has never been run."* The Self-Review's coverage line says
"§8.8 → Task 4" as though scheduling it were satisfying it.

Two mechanical snags in the same step: the harness's Liana dependency is a
**Cargo path dependency**
(`mnemonic-engrave/harnesses/liana/Cargo.toml`: `liana = { path =
"../../../.tmp/fable-liana-src-v15/liana" }`), which no environment variable
can override — the script must rewrite that line or use a `[patch]`/`paths`
override, so "env override" is not implementable as written. And `README.md`
records that the path differs by tag (`<checkout>` at v8.0, `<checkout>/liana`
at v15.0), so the script must pin the tag it edits for.

**Remedy.** Add one sentence to Task 4: *"Stage 2 does not close on a skipped
§8.8 run; the committed evidence under `design/evidence/f449-stage2/` must come
from a non-skip run, and must name the Liana tag and commit SHA."* And replace
"env override" with the actual mechanism.

---

# MINOR

## M-1 — Task 1 Step 5's fragment fails Task 1 Step 10's own gate (MEASURED)

`UnspendableKind`'s two variants carry no doc comments. The workspace sets
`missing_docs = "warn"` (`descriptor-mnemonic/Cargo.toml:12`) and
`scripts/phase-gate.sh:32-33` runs
`cargo clippy --locked --all-targets --all-features -- -D warnings`. I applied
Steps 5 and 6 verbatim to a scratch copy of the tree and ran it:

```
error: missing documentation for a variant  --> crates/md-codec/src/compose/mod.rs:331:5   (Nums)
error: missing documentation for a variant  --> crates/md-codec/src/compose/mod.rs:332:5   (Liana)
error: could not compile `md-codec` (lib) due to 2 previous errors        # exit 101
```

(Both files restored; `git status --porcelain` empty.) One line each fixes it.
Worth recording separately: Step 5 **is** an anchored block, so
`plan-build-gate-md.sh` compiled it and reported green — its scratch crate does
not carry the workspace `[workspace.lints]` table. The Self-Review's
*"Build-gate coverage — stated, not assumed"* paragraph names only the
unanchored fragments as blind spots; lints are a second one and should be named
there.

## M-2 — Task 1 Step 7 cites the wrong file for the clap attribute

Step 7 says *"In `cmd/compose.rs` (also a FRAGMENT — a clap attribute on a
field)"*. The `Compose` variant is declared at
`crates/md-cli/src/main.rs:287-312`; `cmd/compose.rs` has only `pub fn run`
(`:590`). The `File Structure` section gets it right (*"Modify
`crates/md-cli/src/main.rs` (or wherever `Compose` is declared)"*), so the plan
contradicts itself in the one region the build gate does not cover.

Executed the fragment itself: `#[arg(...)]` placed **before** the doc comment
compiles and clap still picks the doc up as help text (verified in a scratch
clap 4 crate — `--unspendable <KIND>  Which unspendable taproot internal key…
[default: nums]`). It is legal but inverts the house style at `main.rs:288-311`,
where every sibling field puts the doc comment first.

## M-3 — Task 2 Step 1 asserts something the shape under test already guarantees

```rust
assert!(!out.contains("UNSPENDABLE(liana)"), …);
```

On `--path 1of1 --path 2of3,older=26280`, `internal_key_path` returns
`Some(0)`, so `compose/tr.rs:45` yields `InternalKey::Slot(0)` and
`render.rs:200-207` can only emit `@0`. The marker is unrenderable for this
input, so the assertion passes with the entire Task 2 warning deleted. The
test's real content is the two `err.contains(...)` lines (which are sound — I
confirmed compose's default stderr is only `note: stdout is a keyless
descriptor template (no keys)`, containing neither `@0` nor `first`). Drop the
vacuous line or move it to the `--unspendable nums` control.

## M-4 — F-641's prescribed remedy is unimplementable at the only site the check can live

F-641 (`mnemonic-engrave/design/FOLLOWUPS.md:19541`) and Task 5 Step 3 both say
*"Check the parse position, not the preceding text."* `validate_marker_position`
(`parse/template.rs:1119-1145`) deliberately runs **before**
`substitute_synthetic` and `Descriptor::from_str` — documented at `:1101-1118`
and `:2792-2799` as the I-1 fix from the whole-branch review — because the
parser rejects the literal marker text outright. There is no parse position at
that point. The implementable version is positional-from-string-start: the
marker must begin at byte 3 **and** the template must start with `tr(`, which
refuses both reproductions (confirmed today: `xtr(UNSPENDABLE(liana),…)` and
`wsh(tr(UNSPENDABLE(liana),…))` each fail with *"unrecognized name
'fa1446b1…'"*). Say that in the step; `prescribed-fixes-are-not-authoritative`.

---

# NIT

## N-1 — Task 3 Step 1 cites an 8/8 measurement to justify a 4-case test

*"The whole-branch review measured 8/8 for this"*, then the step scopes the
test to the four ACCEPT cases. Descriptor equality is a rendering property and
holds for all eight — Liana's verdict is downstream of it. I measured 4/4 for
the ACCEPT set; extending to 8 costs one loop condition and removes the
mismatch between the cited evidence and the assertion. (§8.2 says "the four
ACCEPT shapes", so widening is a bonus, not a correction — but then cite 4/4,
not 8/8.)

## N-2 — the Self-Review does not cover Tasks 5 and 6

The `## Self-Review` section sits between Task 5 and Task 6, and its "Spec
coverage" paragraph enumerates Tasks 1–4 only. Task 5 (three message
follow-ups) and Task 6 (the reconciliation sweep, which the plan itself calls a
"standing rule this establishes") are outside the plan's own self-review — the
two tasks most likely to be added late are the two nobody checked.

---

# What I verified and found SOUND (do not re-derive)

- **The stage's headline claim works end to end.** Composer → marker → `md
  encode` → `md descriptor` → Liana v15.0 **ACCEPT with addresses**, on the
  composer's own origin layout:
  `md compose --wrapper tr --preset kofn-recovery,2of3,older=26280`, NUMS hex
  swapped for `UNSPENDABLE(liana)`, keyed with vendored material →
  `ok=True, recv0=bc1pj6davmeetfe2uutrjlgxcjytq50ggvyd82xtxx2eazm24hsdhlnqxvh4v4`.
  The same template also encodes to an md1 string. Brief Q3 is answered yes
  **for `kofn-recovery` and `tiered-recovery`**; C-1 and I-6 are the other four.
- **Task 3 is achievable exactly as written**, and its numbers are right:
  4 ACCEPT cases, descriptor equality 4/4 byte-exact **including checksum**,
  address equality **24/24**, oracle = the recorded `liana_receive`/`liana_change`
  (not recomputed from the descriptor under test).
- **Task 2's split is correct** (brief Q4, second half): the codec has no
  stderr, `Composed` already carries `experimental: Vec<Experimental>` rendered
  by `cmd/compose.rs:568-587`'s `describe`, and there is exactly one `Composed
  { … }` literal in the crate (`compose/lowering.rs:285`), so adding a field is
  cheap. The trigger `ik.is_some() && unspendable == Liana` is the right
  predicate **for `tr`** — `internal_key_path` (`compose/tr.rs:10-12`) is
  precisely §6 row 3's "a path list containing a bare single". I-3 is the
  wrapper gap, not a fault in the predicate itself.
- **Task 5's three premises reproduce.** F-641 confirmed above; F-638 and
  F-636 are message-precision items whose sites exist
  (`validate.rs:596-607` returns `UnspendableUseSiteNotCanonical` with no `@N`
  from both the shared-default and the override branch).
- **Brief Q6, direct answer:** no test written in Tasks 1–4 pins wording Task 5
  will break. Task 2's assertions target a *new* warning Task 5 does not touch;
  Tasks 1, 3, 4 pin no error text. The real ordering hazard is I-5, which runs
  the other way.
- **`scripts/plan-api-check.sh` is clean** on this plan (16 candidate symbols
  from 6 rust blocks; the three "undefined constants" — `UNSPENDABLE`, `WHICH`,
  `KIND` — are prose and a `value_name`, not code).

---

## Gate

**0C / 0I is not met: 3 Critical, 7 Important.** Recommended order of fold:
C-3 first (it changes what Task 4 does and therefore what Task 3 pins, I-5),
then C-1 and I-6 (they add a refusal and a warning that Task 1's test list and
C-2's anchored floor must cover), then C-2, then the rest. Re-run
`./scripts/plan-build-gate-md.sh` **and** a `clippy -D warnings` pass on the
fold before re-dispatch — M-1 is the demonstration that the former alone does
not cover the latter.

---

# APPENDIX A — C-3 reproduction artifact, and why three constructions failed

Added after the coordinator asked for the artifact rather than the conclusion.
**C-3 stands, and reproduces.** Below is the exact descriptor, the exact
harness output, and — more useful than the positive itself — the measured
explanation for the coordinator's three failures.

## A.1 The accepted nested descriptor, verbatim

Harness: `/scratch/code/shibboleth/.tmp/liana-harness-target/debug/liana-harness parse`,
Liana **v15.0** at `/scratch/code/shibboleth/.tmp/fable-liana-src-v15/liana`.

```
tr(xpub661MyMwAqRbcFn1aHFGgZ359mzBqgj6rjSy73VLCdi92kFK4uHH71MRMGuWX5LqsouhUowavHR6PpPyxjctLm96D6JSZyPAr9RufepmuyEz/<0;1>/*,{multi_a(2,[73c5da0a/48'/0'/0'/3']xpub6DXuQW1Q2JpZyweiMewTZuMPvjG8hKhV2qoF6wL9VFxsMBExtbfqAAoR4oMG4GyxFzVdfas1v2eAdfLxyjc4Ceo5B6w6zTpf7F2BuXCJ52i/<0;1>/*,[3f635a63/48'/0'/1'/3']xpub6DXuQW1Q2Jpa1hNtFUcghdx7Q8kTDsqo7b54YAqZBNCH8EuSvmNSAKbAvkZ4HspgftJ1aqSMeFiZ4sr2QNEGm9geaEre3zDwiJD7C5gx5VH/<0;1>/*,[66d455ea/48'/0'/2'/3']xpub6DXuQW1Q2JpZyteDRGW1pD34uhumfnZJfTsmjDkgd4xcq3L5XX2KUE1n4rmvcDT3RmdchfhbD9DkvSyVhUMBjUYi691iFszgKtf4Bfqe2nL/<0;1>/*),{and_v(v:pk([73c5da0a/48'/0'/3'/3']xpub6DXuQW1Q2JpZzLV9igdwdnmCSoaPVd4ZNZnvfgUsGvQ8AbNAhEmfBEMCMHctwZBuxWK8HkjqUW5F72MCSJCFfisVwRY62Kb1FuDZ66nNQe1/<0;1>/*),older(26280)),and_v(v:pk([73c5da0a/48'/0'/4'/3']xpub6DXuQW1Q2JpZzZHXLadWbvXrMTD8ysfE7L4YZHsEvpWQ3KQ7CVporF7mSQKcphivSAdwGdLuLLHvrgaQXUeNMwpz5c1HAJHXgxvesUgj4Mb/<0;1>/*),older(52560))}})#ev25ssa9
```

The tree is `{multi_a, {leaf, leaf}}` — depth 2 on the right branch. It is
`md compose --wrapper tr --path 2of3 --path 1of1,older=26280 --path
1of1,older=52560` seated with vendored keys and rendered through
`md descriptor --template "tr(UNSPENDABLE(liana),…)"`, so md's own composer
produces this shape.

## A.2 The full run: five probes, one control, one line each

```
CONTROL-kofn-recovery-flat-ACCEPTED-before ok=True  recv0=bc1pj6davmeetfe2uutrjlgxcjytq50ggvyd82xtxx2eazm24hsdhlnqxvh4v4
A-mine-nested-2of3-distinct                ok=True  recv0=bc1p32ny335apyeq787eqe6r0wyt5hfjkkkmhtw68ce5kav4my7xtvestf3grq
B-coord-nested-2of2-distinct               ok=True  recv0=bc1pxsqjjzze0aguwh4ldu5qgsh6x0yhdv2qlyhyg4uv8jd5r6t058lqg359pq
C-coord-nested-same-timelock               ok=False Descriptor is not compatible with a Liana spending policy.
D-flat-2of2-distinct-depth1                ok=True  recv0=bc1pcc26jep58suatdhafv7zw6k6vjhmrevlh8cwte60vqp756xvxqasajv0w0
E-B-with-STALE-internal-key                ok=False Descriptor is not compatible with a Liana spending policy.
F-B-tree-with-RAW-NUMS-internal-key        ok=False Descriptor is not compatible with a Liana spending policy.
```

(F was run in a second invocation with its own copy of the same control, which
also returned `ok=True`. Inputs: `/scratch/code/shibboleth/.tmp/r0probe{3,4}.jsonl`.)

`A` returns `is_taproot: true`, primary = the 2-of-3 `multi_a` (the internal
key correctly excluded from the policy), and **two recovery paths at `older`
26280 and 52560 inferred from a nested tree**.

## A.3 **Your shape 1 is accepted. `B` IS your shape 1.**

`B` is literally `{multi_a(2,A,B),{and_v(v:pk(C),older(26280)),and_v(v:pk(D),older(52560))}}`
— the construction you report as refused twice — and it imports:

```
tr(xpub661MyMwAqRbcFswVugWFBxmD7r3bQLsmHovc3p3wTFfgk3EWb36m3QfsezgaR6h5cXXgPG3R2XmctBn55sAt35wzLnrYy82sLKYF8CRsak7/<0;1>/*,{multi_a(2,[73c5da0a/48'/0'/0'/3']xpub6DXuQW1Q2JpZyweiMewTZuMPvjG8hKhV2qoF6wL9VFxsMBExtbfqAAoR4oMG4GyxFzVdfas1v2eAdfLxyjc4Ceo5B6w6zTpf7F2BuXCJ52i/<0;1>/*,[3f635a63/48'/0'/1'/3']xpub6DXuQW1Q2Jpa1hNtFUcghdx7Q8kTDsqo7b54YAqZBNCH8EuSvmNSAKbAvkZ4HspgftJ1aqSMeFiZ4sr2QNEGm9geaEre3zDwiJD7C5gx5VH/<0;1>/*),{and_v(v:pk([66d455ea/48'/0'/2'/3']xpub6DXuQW1Q2JpZyteDRGW1pD34uhumfnZJfTsmjDkgd4xcq3L5XX2KUE1n4rmvcDT3RmdchfhbD9DkvSyVhUMBjUYi691iFszgKtf4Bfqe2nL/<0;1>/*),older(26280)),and_v(v:pk([73c5da0a/48'/0'/3'/3']xpub6DXuQW1Q2JpZzLV9igdwdnmCSoaPVd4ZNZnvfgUsGvQ8AbNAhEmfBEMCMHctwZBuxWK8HkjqUW5F72MCSJCFfisVwRY62Kb1FuDZ66nNQe1/<0;1>/*),older(52560))}})#t7n8w9qv
```

So the difference is not the shape. It is the **internal key**.

## A.4 THE FINDING — three distinct causes share one error string, and a control does not separate them

`analysis.rs:595-604`:

```rust
let tree_policy = tree.lift()...;                                    // :595
let unspend_int_xpub = unspendable_internal_xpub(desc)
    .ok_or(LianaPolicyError::IncompatibleDesc)?;                     // :596-597
let desc_int_xpub = get_multi_xkey(desc.internal_key())
    .ok_or(LianaPolicyError::IncompatibleDesc)?;                     // :598-599
if *desc_int_xpub == unspend_int_xpub { tree_policy }                // :600
else { SemanticPolicy::Thresh(Threshold::or(Key(internal), tree_policy)) }
```

Liana **recomputes** the unspendable xpub from the descriptor's own leaves and
compares. Three different faults land on the identical message
*"Descriptor is not compatible with a Liana spending policy."*:

| # | cause | mechanism | probe |
| --- | --- | --- | --- |
| 1 | genuine policy-shape refusal | two recovery paths at one timelock → `recovery_paths.contains_key(&timelock)`, `:659` | **C** |
| 2 | **stale / mismatched unspendable xpub** | `:600` falls to the `else`, policy becomes `or(Key(IK), tree)` → two non-timelocked paths → `from_recovery_path` fails | **E** |
| 3 | **raw NUMS x-only internal key** | `get_multi_xkey` returns `None` on a non-xkey → `:598-599` | **F** |

`E` is `B`'s exact tree with `A`'s internal key spliced in — nothing else
changed — and it flips `ok=True` to the refusal. The recipe is
structure-independent but **leaf-set-dependent**: `B`, `C` and `D` share one
internal key because they share four leaf keys in the same order; `A` has five
leaves and therefore a different one.

**This is the sibling of your own recon caution, and it is sharper.** You wrote
that `"Error while parsing xkey"` is a parse refusal and proves nothing about
policy, so every probe carries a control. Causes 2 and 3 defeat that
discipline: the control carries **its own correct internal key**, so it passes
in the same run while the probe fails for a key reason that reads exactly like
a policy reason. A passing control proves the harness and the keys are good; it
does **not** prove the probe's internal key matches the probe's leaves.

Your attempt 2 — *"same shape, real keys from an accepted case"* — is the
textbook trigger: real keys carried over, but the accepted case's internal key
carried over with them, and the leaf set changed.

**Task 4 must state this as a probe rule**, alongside the control rule:
*every probe's internal key is RECOMPUTED for that probe's exact leaf set and
order (`md descriptor --template "tr(UNSPENDABLE(liana),…)" --key …`), never
copied from another descriptor; a probe whose internal key was not recomputed
is void, exactly as a run whose control failed is void.* Cheap mechanisation:
have the gate script assert, for each probe, that the internal key equals
`liana_unspendable_xpub(leaves)` before sending it — the script already holds
the leaves.

## A.5 What this settles

- **F-640 CLOSES with a vector.** §8.1's "a nested taptree that Liana ACCEPTS"
  is satisfiable; `A` and `B` both satisfy it. Task 4 vectors one (prefer `B`:
  four keys, reuses the corpus's existing key set and internal key, so it slots
  into `cases.json` beside `preset-kofn-recovery-tr` with the smallest
  diff — and note `B` is a **3-path** list, so md reaches it via `--path`,
  not via any of the six presets).
- **Nesting is not the variable.** `D` is `B`'s depth-1 twin — same keys, same
  two timelocks, flat tree — and is also accepted. Accepted at both depths,
  refused at both depths for content reasons. The corpus's nested refusal
  (`preset-decaying-multisig-tr`) is fully explained by content: every tier
  timelocked, plus an `after()`.
- **C-3's severity is unchanged.** The plan's "amend SPEC §8.1" branch would
  retire a satisfiable requirement, and would have done so on a premise three
  probes appeared to support while measuring something else.

---

# APPENDIX B — RULING: refuse or warn? (coordinator's scope question)

**Recommendation: (a) REFUSE — but the dilemma is false, and the two cases in
it want different answers.**

## B.1 The duplication objection does not apply

The premise of (b) is that (a) forces compose to "run §6's predicate, which
today lives only at `encode.rs:230` — duplicating a rule". It does not.
`validate_unspendable_shape` is **`pub`** (`crates/md-codec/src/validate.rs:585`),
and `cmd/compose.rs` already holds a fully built `composed.descriptor` before
it prints anything. (a) is one call to the existing function — **one home, two
callers** — which is the opposite of the defect this project names. The defect
is a *second copy* of a rule; the remedy it prescribes is to ask the question
where the knowledge lives, and the knowledge lives in `validate.rs`.

This is not even a new precedent. It is the **established** one, in this exact
file, written by the F-600 fold twelve lines from where the new call would go
(`cmd/compose.rs:688-690`):

> *"So compose now reads back what it is about to emit, with the SAME parser
> `md encode` uses. This is not a second implementation of the rules: it is the
> same function, which is what keeps the two verbs from drifting."*

F-600 is the same defect class as C-1 — compose exiting 0 on a template encode
refuses — and the project already ruled on it, in favour of (a), by reusing the
function. (b) would be the departure, and it would say that the F-600 guard was
right to catch malleability and repeated keys but should stop short of the one
new rule this cycle adds.

## B.2 (b)'s premise — "encode is the real gate" — is false as measured

(b) assumes the operator who ignores the warning hits a wall at `md encode`.
They do not necessarily reach `md encode`. Measured at `25acb33c`:

```
$ md descriptor --template "tr(UNSPENDABLE(liana),sortedmulti_a(2,@0/…,@1/…,@2/…))" --key … --key … --key …
tr(xpub661MyMwAqRbcFJqq8Bk…/<0;1>/*,sortedmulti_a(2,…))#ctjy7seq      # exit 0
```

`md descriptor` shares compose's parse path and never calls
`validate_unspendable_shape`, so under (b) the operator gets a **pasteable,
fundable descriptor for a wallet that has no md1 encoding at all**. Money can
go in; no card can ever come out. Encode is not a gate on the path to funds —
it is a gate on the path to a *backup*, and that is the asymmetry that loses
coins.

This project has already ruled on that direction, again in this file
(`cmd/compose.rs:627-634`, the every-path-hashlock warning): *"THE ASYMMETRY IS
THE POINT … so the direction that loses money was the unwarned one."* Under
(b) the direction that loses money is the unrefused one.

## B.3 A warning is the wrong instrument when nothing downstream can accept

A warning is right when the shape is legal and the operator might mean it — the
hashlock case, where `--experimental` and a deliberate choice exist. Here there
is no `--force`, no flag, and no downstream verb that accepts a kind-1
`sortedmulti_a`. Every continuation ends at the encode wall or at the
`md descriptor` trap. A warning on a path with no good ending is decoration.

## B.4 The dilemma is false: C-1 and I-6 are different rules with different owners

This is the part I would actually change in the plan. They are not one question:

| case | who refuses it | ruling |
| --- | --- | --- |
| **C-1** — kind 1 + `sortedmulti_a` (`plain-multisig`) | **md** (SPEC §6 row 1) | **REFUSE at compose**, by calling `validate_unspendable_shape` |
| **I-6** — `hashlock-gated`, `decaying-multisig` | **Liana** (SPEC §0a, classes 4 / 5+7) | **WARN and proceed** |

md may refuse what md's own spec forbids. md may **not** refuse a wallet that
is legal under its own rules merely because one third-party coordinator will
not import it — that would hard-code Liana's policy model into md's lowering,
break the composer's contract (*"no search, no cost model, the same text from
every implementation"*), and it is precisely what SPEC §0a puts out of scope.
So I-6 gets Task 2's stderr vocabulary, not a refusal.

Answering them separately dissolves the trade-off: the rule with one home keeps
one home and gains a caller; the rule md does not own never becomes a refusal.

## B.5 Implementation notes that keep (a) from creating new defects

1. **Gate the refusal on the flag, never on the shape alone.** It must fire
   only when `--unspendable liana` was requested. A refusal that fires on the
   default would break `plain-multisig` for every existing operator and turn 65
   vendored vectors red — the C-2 floor must carry a `plain-multisig` row
   asserting the **default** still composes to the NUMS hex at exit 0.
2. **Call the whole function, not row 1.** Rows 2 and 4 are unreachable from
   compose today (it always emits `<0;1>`; it never nests `tr` under a
   wrapper), so scoping the call to row 1 would look equivalent and would
   silently stop being equivalent the first time compose widens. Calling
   `validate_unspendable_shape` costs nothing and cannot drift.
3. **Do not reword the refusal.** Surface `Error::UnspendableWithSortedMultiA`'s
   existing text and add the compose-level prescription
   (`--unspendable nums`, or a different preset) around it. A second phrasing
   of one refusal is the duplication (b) was right to fear — just in the string
   table rather than the logic.
4. **Run the prescribed recipe in the test.** Whatever command the refusal
   tells the operator to run next, the test executes it and asserts exit 0.
5. **Where this leaves `md descriptor`.** The B.2 trap is pre-existing and not
   stage 2's to fix — but it is now *named*, and it is the reason (a) is
   required rather than merely preferable. File it as a follow-up against
   `md descriptor`/`md address` rendering shapes `md encode` refuses, owning
   phase stage 2 if cheap, otherwise stage 4a.

**Net effect on the gate:** adopting this ruling closes C-1 and I-6. It does
not touch C-2 (the floor is still self-referential) or C-3 (now closed by
Appendix A, which converts the "amend the spec" branch into "vector it and
close F-640").
