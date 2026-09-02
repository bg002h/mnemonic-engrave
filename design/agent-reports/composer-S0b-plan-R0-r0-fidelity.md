# composer S0b plan — R0 round 0, FIDELITY + DESIGN lens

**Artifact:** `design/IMPLEMENTATION_PLAN_composer_S0b_presets.md` (1562 lines, mnemonic-engrave `ec2720a`)
**Reviewer:** independent architect (did not author the plan)
**Baseline read:** descriptor-mnemonic `66bdf2f4`; seedhammer `main` at `321acb5`; spec `design/SPEC_wallet_policy_composer.md`; sibling plan `design/IMPLEMENTATION_PLAN_composer_S3_fork_gui.md`
**Date:** 2026-09-02

**Answer to the one question, up front.** The plan is faithful to §4d/§5 in the
things that could go silently wrong: the six constructors are *called*, never
re-derived; the tier order is not reversed; the lowering is untouched; no
normative behaviour is invented in the CLI layer. What it does NOT do is settle
whether the parameters it picks are the DEVICE's preset shape, and it does not
line up with the Stage 3 plan that consumes it — the two documents disagree
about which wrapper each preset vector is for, about which fork file owns the
pin edit, and about what artifact A10 reads to build its Go table. The grammar
itself is sound; its refusal *table* is a strict subset of the refusals the code
emits.

**Not re-derived (already machine-verified by the controller, per the brief):**
compile/build, the 52/52 md-codec compose suites, the 23/23 md-cli compose CLI
suites, clippy/fmt, the 6-vector / 30-file export, citation resolution, glyph and
table gates.

---

## Lens 1 — parameter defaults as a NORMATIVE choice

**Checked.** Plan `:68` ("Default parameters (decided here…)"), the six family
rows `:383-399`, spec §4d (`:146-157`), §9 item 5 (`design/SPEC_wallet_policy_composer.md:786-788`),
and S3 A10 (`design/IMPLEMENTATION_PLAN_composer_S3_fork_gui.md:4681-4745`).

**Found.** §4d says presets "POPULATE a path list the operator then edits", so
the parameters a preset ships with ARE what the device puts on screen first.
S3 A10 `:4723` makes this binding: *"Each entry's `list` is transcribed from the
primary's exported vector's `descriptor.json` path list… **Do not invent a shape
here**"*, and A10's test rule at `:4717` is *"If a preset's chunks differ, the
VECTOR wins and the Go table changes"*. So the vectors' parameters propagate to
the device by construction. The S0b plan never says so, and its stated reason for
two of them is a test-fixture budget, not a UX choice.

- I-1, I-2 below.
- Verified clean: `plain_multisig 2of3`, `simple_timelocked older=26280`,
  `kofn_recovery 2of3 + older=26280` are the journey's own canonical values and
  need no defence.

## Lens 2 — grammar: positional `<k>of<n>`, and which argument is the head

**Checked by reading the source, not the plan.**
`crates/md-codec/src/compose/presets.rs:71-85` — `tiered_recovery(w, k1, n1, k2, n2, older)`
builds `paths: vec![ks(k1, n1), tier2]` where `tier2` is the one carrying
`blocks(older, 1)`. So `(k1, n1)` is the **head** (path 0, unlocked, spends now)
and `(k2, n2)` is the locked recovery tier.
`presets.rs:111-140` — `decaying_multisig` builds `[t1(k1,n1)+older1,
t2(k2,n2)+older2, t3(1,1)+after]`, head first.

Plan `:507`/`:509` map `ofs[0] → (k1, n1)`, `ofs[1] → (k2, n2)` (`:1204-1207`,
`:1229-1232`), i.e. head path first, matching §5's "paths combine listed order".
**No reversal.** The one thing a reversed tier would have shown up in — the
hand-typed expected literals at `:391-398` — renders `multi(2,…)` unlocked first
for tiered-recovery and `older(13140)` on the head for decaying-multisig, both
consistent with the constructor.

- M-1 below (a documentation gap, not an order defect).

## Lens 3 — refusals

**Checked.** Plan's refusal table `:513-522`; the ten tests at `:625-760`;
`ComposeError`'s `Display` (`crates/md-codec/src/compose/mod.rs:264-317`);
`validate`'s legacy arm (`mod.rs:367-376`); the exit-code map
(`crates/md-cli/src/main.rs:806-819` — every `CliError` exits 1 except `BadArg`
at 2; `Compose` is not `BadArg`, so the plan's "exit 1" column is right).

**Wording:** actionable in every table row. `LegacyWrapperShape` names the
remedy ("use wsh or tr"); the unknown-name refusal lists all six; the
missing/extra-parameter refusals name the parameter. Every test asserts the exact
text via `predicate::str::contains` on the verbatim string and asserts the exit
code.

**Does the legacy refusal fire for EVERY non-plain preset under `sh`/`sh(wsh)`?**
Yes — I constructed the counterexample rather than reasoning about it. Using the
shipped `target/debug/md` at `66bdf2f4` with the `--path` equivalent of each
archetype's path list, all **ten** pairs (5 non-plain archetypes × {`sh`,
`sh-wsh`}) plus `plain-multisig` at `n=1` refuse with exactly:

```
md: legacy wrappers hold one plain sorted multisig only (n >= 2, no lock, no hash); use wsh or tr
```

The mechanism is structural: every non-plain constructor builds ≥ 2 paths
unconditionally, so `sole = list.paths.len() == 1 && …` (`mod.rs:368`) can never
hold. There is no escape, and one edge case the plan does not mention is also
covered: `plain-multisig,1of1` under a legacy wrapper refuses too, because
`is_bare_multi` requires `n >= 2` (`mod.rs:131-135`) — §4a's "n = 1 is refused"
is therefore satisfied without a CLI special case.

- M-2 (test coverage of that mechanism), M-3 (one refusal that names no remedy),
  I-3 (the table is not exhaustive) below.

## Lens 4 — §12 item 1, the six `preset:<name>` tags as SINGULAR

**Checked.** `crates/md-codec/tests/compose_vectors.rs:67-131` — the whole rule.
`every_tag_appears_in_at_least_two_vectors` (a) fails any tag with count < 2 that
is not in `SINGULAR_TAGS`, (b) asserts each `SINGULAR_TAGS` entry has count
**exactly 1**, (c) asserts a fixed required-tag list is present.

**Found: no weakening.** The six rows are pure *additions* to `family()`, so
every existing tag's count can only rise — nothing can drop below two. Each of
the six `preset:*` tags appears exactly once by construction, satisfying (b).
The required list (`compose_vectors.rs:91-125`) contains no preset axis and the
plan leaves it untouched (`:404-410`), which is correct: §12 item 1's required
tags are wrappers / path counts / spine shapes / internal-key cases / lock
encodings / hash / sorted / unsorted / keyless-wsh / fingerprint cases / origins,
and "preset" is not among them.

Counts machine-checked, not read off the plan:
`grep -cE '^\s+\("(keyed_)?compose_' compose_support.rs` → **28** today, → 34;
`grep -cE 'name: "(keyed_)?compose_' test_vectors.rs` → **26** today, → 32;
`ls crates/md-codec/tests/vectors | grep -cE '^(keyed_)?compose_'` → **126**
today, → 156. All three match the plan's CHANGELOG prose at `:1533-1537`
("34 tagged / 32 in MANIFEST") and Task 3's fork note at `:1552` (126 → 156).

- N-1 below.

## Lens 5 — the `--json` `preset` field

**Checked.** `crates/md-cli/src/format/json.rs:6` — `SCHEMA = "md-cli/1"`. No
`compose` JSON snapshot exists (`crates/md-cli/tests/snapshots/` holds
decode/inspect/address only), and `cmd_gui_schema.rs` / `cli_output_class.rs`
contain no `compose` (verified: `grep -rln compose` over `crates/md-cli/tests/`
lists neither).

**Found: adequate, and the fork does not need it.** The field is always present
(`null` for a `--path` policy — asserted at `:815`), so the addition is purely
additive for any consumer. A10 reads the *vendored vectors*, never
`md compose --json` (`:4692`, `:4723`), which is the right coupling. The
per-preset `params` key set varies, so a consumer must switch on
`preset.name` — acceptable for a debug/automation surface, and the alternative
(a flat union of all seven parameter names) would be worse.

- N-2 below.

## Lens 6 — blast radius

**Checked and clean:**

- **`--path` losing `required = true`.** No test or doc asserts clap's
  required-args error for `md compose`. The only two files mentioning that
  string are `crates/md-cli/tests/cli_stdin_dash.rs:14` (about stdin) and
  `crates/md-cli/src/cmd/gui_schema.rs:338` (a doc comment about `<STRINGS>`),
  neither about compose. `crates/md-cli/tests/help_examples.rs` checks only
  `decompose` (`:57-60`), so the new `--preset` help text runs through no
  example harness. No README/docs usage of `md compose` exists.
  The plan's own `compose_refuses_when_neither_path_nor_preset_given` (`:648`)
  is the replacement gate, and it asserts only the exit code, which is the right
  granularity for a clap-owned message.
- **CHANGELOG.** `## md-cli [Unreleased]` is at `:7` and `## md-codec
  [Unreleased]` at `:69` exactly as the plan says; the `md compose` bullet
  (`:28`) and the "28 tagged compose vectors" bullet (`:83`) both exist as
  insertion anchors. The prose numbers are correct (see Lens 4).
- **The fork's vendor script.** `scripts/vendor-compose-vectors.sh:16` is
  `grep -E '^(keyed_)?compose_'` — `keyed_compose_preset_*` matches with no
  script change, exactly as the plan claims (`:26`, `:1552`). Confirmed by
  reading the script.

**Found:** I-4 below (the fork pin-test ownership).

## Lens 7 — could the presets be data rather than six code paths?

**Recommendation: no change.** The six constructors have six *different*
arities and parameter names (`(w,k,n)`, `(w,u32)`, `(w,k,n,u32)`,
`(w,k,n,k,n,u32)`, `(w,[u8;32],u32)`, `(w,k,n,k,n,u32,u32,u32)`), so a table
would still need a per-row closure to call the right constructor — the same six
code paths with an indirection on top, and a table row that can silently
disagree with its own closure. The shared machinery (`need_ofs`, `named_only`,
`need_u32`, `parse_kofn`, `parse_sha256_hex`) is already factored out, which is
where the real duplication was. Keeping the arms is also what makes the
"nothing normative in the CLI" property auditable: each arm is a single call
into the shipped constructor and nothing else.

One thing the current shape *does* leave open — M-4 below.

---

# Findings

## Critical

None.

## Important

### I-1 — the plan never says whether these parameters are the DEVICE's preset shape, and via A10 they become it

**Plan `:68`; spec §4d `:148-150`, §9 item 5 `:786-788`; S3 A10 `:4723`.**

The plan frames its defaults as the vectors' parameters only: *"§4d fixes no
defaults, so this plan sets them and says why"*, and the "why" for two of the six
is a **test-fixture budget** — `tiered_recovery` is `2of2,1of2` "instead of S0
Task 7's illustrative `2of2,2of3` — which sums to 5 slots, one over the 4-key
budget every OTHER `keyed_compose_*` vector obeys", and `decaying_multisig` is
`2of2,1of1` instead of `2of3,1of2` "which sums to 6 slots". That budget is
`keyed_compose_vectors_bind_at_most_the_four_journey_keys`
(`compose_vectors.rs:133-151`) — a property of the *fixture*, not of the device,
which admits 32 slots (§4b).

S3 A10 then makes the vector's list the device's list verbatim (`:4723`, "Do not
invent a shape here"), so the operator's first-offered tiered-recovery becomes
"2-of-2 now, 1-of-2 after 26280" for a reason that exists nowhere in §4d and that
nobody chose on UX grounds. §4d's "populate a path list the operator then edits"
makes this survivable — but it is still a normative-facing default arrived at by
accident, and neither plan states the coupling.

**Hypothesis for the fold (not prescriptive).** One added sentence in the
"Default parameters" paragraph deciding, explicitly, which of these two the
numbers are:

- (a) *vector parameters only* — then S3 A10 `:4723` must be amended to say the
  Go table's shapes are chosen for the device and only *checked* against the
  vectors' chunks for the pairs that exist; or
- (b) *the device's offered shape too* — then the two shrunk archetypes want a
  one-line UX justification that is not "the fixture has four xpubs", and
  ideally a note that a 5th/6th slot would need the fixture widened, not the
  archetype narrowed.

Either way the S0b plan should name S3 A10 as the consumer that inherits them.

### I-2 — S3 A10 offers all six presets under BOTH `wsh` and `tr`; S0b exports each archetype under exactly ONE wrapper, and `kofn-recovery` is the `tr` one

**Plan `:383-399` (family rows) and `:440-459` (MANIFEST); S3 A10 `:4695`, `:4717`.**

S0b's six vectors are: `plain_multisig`/**Wsh**, `simple_timelocked_inheritance`/**Wsh**,
`kofn_recovery`/**Tr**, `tiered_recovery`/**Wsh**, `hashlock_gated`/**Wsh**,
`decaying_multisig`/**Wsh**. One per archetype, exactly as F-453 words it
(`design/FOLLOWUPS.md:15411`).

S3 A10 `:4695`: *"**Which presets are offered depends on the wrapper (§4d):** all
six under `wsh` and `tr`"*, and its gate at `:4717` is
`TestComposerPresetsReproduceTheirVendoredVectors`: *"for each of the six presets
`composerPresets(md.ComposeWsh)` returns, `md.Compose(p.list)` then `Chunks()`
equals the vendored `md/testdata/vectors/preset_<name>.phrase.txt`"*.

Under `md.ComposeWsh` that loop needs six **wsh** vectors. Five exist. The sixth,
`kofn-recovery`, is a **tr** vector — its chunks encode a taproot tree with a NUMS
internal key. Two ways that lands, both bad:

1. The file it looks for does not exist under the name it expects (see I-4 on the
   `preset_<name>` prefix), so the test `t.Fatalf`s naming F-453 — A10 reports
   its own precondition unmet after F-453 has in fact shipped; or
2. It resolves the `keyed_compose_preset_kofn_recovery` file, the chunks differ,
   and A10's stated rule — *"the VECTOR wins and the Go table changes"* — pushes a
   **`tr` path list into the `wsh` preset picker**. A10 does carry a second
   clause ("if the vector looks wrong, STOP and record it"), so this should be
   caught by a human; but the first-stated rule points the wrong way, and the
   wrapper is not visible in the preset's *name*.

And symmetrically, `composerPresets(md.ComposeTr)`'s six entries have exactly one
tr vector to check against.

**Hypothesis.** Cheapest is to make the coverage claim explicit rather than
export twelve vectors: S0b states that a preset's *PathList* is
wrapper-parameterised and that one vector per archetype pins the archetype's
**parameter order and lowering**, not the wrapper cross-product; and S3 A10's
test is narrowed to "each preset that HAS a vector at this wrapper", with the
wrapper named in the assertion. If instead the intent is a vector per
(preset, wrapper) pair, that is a scope change to F-453 and should be decided
here, not discovered in A10.

### I-3 — the refusal table is a strict subset of the refusals `parse_preset` emits

**Plan `:513-522` (the table, headed "Refusals (one test each, Task 2 Step 1)")
versus `:1078-1085`, `:1108-1112`, `:1190-1194`, `:1000-1010`.**

Every table row does have a test, and I traced each refusal to a real code path —
none of them fails to refuse. But `parse_preset` emits at least six wordings the
table does not list and no test exercises:

| wording (plan line) | reached by |
| --- | --- |
| `` {ctx}: `{k}=` given twice `` (`:1110`) | `--preset kofn-recovery,2of3,older=1,older=2` |
| `` {ctx}: `{tok}` is not <k>of<n> `` (`:1080`) | `--preset plain-multisig,2/3` |
| `` {ctx}: k `{k}` is not a small number `` (`:1083`) | `--preset plain-multisig,300of3` |
| `` {ctx}: n `{n}` is not a small number `` (`:1086`) | `--preset plain-multisig,2of300` |
| `{ctx} {k}: `{s}` is not a number in 0..=4294967295` (`:874`, via `need_u32`) | `--preset kofn-recovery,2of3,older=soon` |
| `{ctx} needs sha256=<64 hex>` (`:1192`) and `{ctx}: sha256 needs 64 hex characters, lowercase` (`:1006`) | `--preset hashlock-gated,older=1` / a short hex |

Two of these are worth more than table hygiene. First, the `is not <k>of<n>`
arm runs **before** the name is matched, so an unknown preset with a malformed
token reports a token error rather than the six valid names: `--preset
multisig,2/3` says ``preset multisig: `2/3` is not <k>of<n>`` and never tells the
operator that `multisig` is not a preset. Second, `k`/`n` overflow (`300of3`)
produces "is not a small number" rather than §4e's `BadThreshold` wording, so two
different messages describe the same class depending on magnitude.

**Hypothesis.** Add the rows to the table (they are the operator-facing surface a
reviewer reads as complete), add two tests (a duplicate `=` and a malformed
`<k>of<n>`), and consider matching the preset NAME before parsing its tokens so
an unknown name always gets the "expected one of …" line.

### I-4 — nobody owns the fork's pin-test edit, and S3 A10 asserts it needs none

**Plan `:1552`; S3 A10 `:4686-4689`, `:4709`, `:4736`; fork
`md/compose_vectors_pin_test.go:36-56, 82-86`.**

S0b correctly identifies the two hand edits and defers them: *"both are
one-line-each, fork-side, Stage 3 (or a dedicated F-453-follow-on) territory,
not this plan's."* S3 A10 does not take them:

- A10's **Files** list (`:4686-4689`) names `gui/composer_presets.go`,
  `md/testdata/vectors/preset_*`, `md/testdata/compose_vectors.provenance.json`
  and `gui/composer_presets_test.go`. `md/compose_vectors_pin_test.go` is not in
  it, and A10's Step 5 `git add` (`:4736`) does not stage it.
- A10 Step 1 `:4709` states the opposite of the truth: *"the pin test passes at
  the new counts (**it asserts the file count, so the constant in
  `md/compose_vectors_pin_test.go` moves with it**)"*. It does not move. The test
  reads `if len(p.Files) != 126` (`:83-84`, a Go literal) and `if p.Vectors !=
  len(composeVectorNames)` (`:79-81`, a hand-maintained 26-name slice at
  `:36-51`). Re-running the vendor script alone takes the pin JSON to 156
  files / 32 vectors and turns `TestComposeVectorsMatchTheirProvenancePin`
  **red**, exactly as S0b predicts and A10 denies.

Two further items nobody has counted:

- `scripts/vendor-compose-vectors.sh:29` writes a generated `_comment` line
  reading *"if the file count is not 126"* into the pin JSON — the script itself
  carries the stale literal and will keep re-emitting it after the re-vendor.
- A10's Files line `:4688` names the vendored files as
  `md/testdata/vectors/preset_*.{bytes.hex,phrase.txt,descriptor.json,template}`.
  Both halves are wrong against S0b: the exported prefix is
  `keyed_compose_preset_*` (which is why no script change is needed — the same
  line says so, contradicting itself), and there is a **fifth** file,
  `.conformance.json`, which the fork's own
  `TestEveryKeyedComposeVectorHasAConformanceRecord`
  (`compose_vectors_pin_test.go:126-133`) requires for every `keyed_`-prefixed
  name.

Related to I-2: A10 `:4723` says the Go table's `list` is *"transcribed from the
primary's exported vector's `descriptor.json` path list"*. There is no path list
in that file. I read
`crates/md-codec/tests/vectors/keyed_compose_wsh_two_path_or_d.descriptor.json`:
it is the **lowered md tree** (`Wsh` → `OrD` → `Multi`/`AndV`/`Verify`/`PkH`/`Older`)
plus TLVs — the output of `compose`, not its input. An A10 implementer following
that sentence literally must either reverse-engineer a `PathList` from the lowered
tree (which is "inventing a shape", the thing `:4723` forbids) or take A10's own
escape hatch and stop, reporting F-453 incomplete. The *check* still holds either
way, because A10's assertion is a chunk comparison — so this is a wrong
transcription source, not a hole in the gate.

**Hypothesis.** S0b Task 3's fork note names the exact edits (the six names, the
`126 → 156` literal, the `:29` comment string) and says which artifact owns them
— either a new follow-up with owning phase **composer S3, task A10**, or an
explicit instruction that A10's Files list and `git add` must grow
`md/compose_vectors_pin_test.go`. Whichever is chosen, S3 A10 `:4688`/`:4709`/`:4723`
need the matching correction, because a plan that says "no edit needed" is what
turns a one-line fix into a red suite nobody scheduled.

## Minor

### M-1 — `decaying-multisig`'s `older1` locks the PRIMARY tier, and nothing the operator reads says so

Plan `:509` lists the grammar as
`<k1>of<n1>,<k2>of<n2>,older1=<n>,older2=<n>,after=<n>` with no note that
`older1` is a lock on the *first* path. The constructor is explicit
(`presets.rs:103`, "k1-of-n1 after `older1`"; `presets.rs:131-132`,
`t1.lock = Some(blocks(older1, 0))`), and the family literal at `:398` renders
`or_i(and_v(v:multi(2,…),older(13140)),…)` — head locked. A reader of
`decaying-multisig,2of2,1of1,older1=13140,…` will reasonably assume the 2-of-2
spends immediately; it cannot spend for ~3 months. The `--preset` help text at
`:1444-1447` does not say it either. Fix: one clause in the grammar table and in
the help ("`older1` locks the primary tier; nothing spends before it").

### M-2 — the legacy-wrapper refusal is tested for 1 of 10 (archetype, wrapper) pairs, and `sh-wsh` is never tested at all

`preset_kofn_recovery_refuses_under_legacy_wrappers_with_the_spec_4d_shape`
(`:722-747`) covers `kofn-recovery` under `sh` only. §4d names both legacy
wrappers. I verified all ten pairs refuse today (see Lens 3), so this is coverage,
not a defect — but a loop over `PRESET_NAMES[1..]` × {`sh`, `sh-wsh`} is two lines
and would catch a future constructor that grows a one-path form.

### M-3 — a `--preset` `after=` in the time band gets a band refusal with no remedy, where `--path` names one

`presets::decaying_multisig` always builds `Lock::AfterHeight`
(`presets.rs:136`), and `Lock::operand` rejects `h >= LOCKTIME_THRESHOLD`
(`compose/mod.rs`), so `--preset decaying-multisig,…,after=1893456000` yields
`path 3: after height needs 1..=499999999`. The `--path` DSL deliberately does
better — measured live on the shipped binary:

```
md: path `1of1,after=1893456000`: after=1893456000 reads as a block height and is
above the height band (1..=499999999); for a Unix time write after=1893456000t
```

`--preset` has no `t` suffix and cannot express a time lock at all (the plan says
so at `:511`), so the remedy is "use `--path`" — and nothing says it. Fix: name
`--path` in the `--preset` help text, or map the same band case to a message that
does.

### M-4 — `PRESET_NAMES` and the six `match` arms are two lists that can drift

`PRESET_NAMES` (`:1063-1070`) is used only to render the unknown-name refusal
(`:1246-1249`); the actual set of accepted names is the `match` at `:1152-1245`.
A name added to one and not the other compiles and clippy-passes, and produces
an "expected one of …" line advertising a name that does not work. One test —
for each `PRESET_NAMES` entry, some parameter set parses — closes it, and is the
kind of thing worth having before a Go table is generated from the same list.

## Nit

### N-1 — `keyed_compose_preset_hashlock_gated` is the only `wsh` family row with no `head:` tag

Its head path is one key **plus a hash**, unlocked — which is neither
`head:bare-multi` (n = 1), nor `head:single` (`is_bare_single` requires no hash,
`compose/mod.rs:137-139`), nor `head:locked`. The vocabulary documented at
`:341-352` has no term for it, so the row (`:394-396`) carries none. Nothing
fails — `head:*` is not in §12's required list — but §12 item 1 says every vector
"names the §5 rows … it exercises". Either add a term (`head:hashed`) or say in
the doc comment why this row has no head tag.

### N-2 — a new top-level `--json` key with no schema signal

`preset` joins the compose JSON object while `SCHEMA` stays `"md-cli/1"`
(`crates/md-cli/src/format/json.rs:6`). Additive and always present (`null` for
`--path`), so nothing breaks, and no compose snapshot pins the shape. Recording
it only because the schema string is the one place a consumer could have learned
the surface grew.

---

# Counts

| severity | count |
| --- | --- |
| Critical | 0 |
| Important | 4 (I-1, I-2, I-3, I-4) |
| Minor | 4 (M-1, M-2, M-3, M-4) |
| Nit | 2 (N-1, N-2) |

**Not green.** The four Importants are all *statements* rather than code: three
of them are disagreements between this plan and `IMPLEMENTATION_PLAN_composer_S3_fork_gui.md`,
and closing them will touch that plan as well as this one. Nothing in the Rust
this plan ships is wrong as far as this lens could reach: the constructors are
called and not re-derived, the tier order is head-first, every refusal refuses
(ten legacy pairs verified live), and the tag rule is not weakened.
