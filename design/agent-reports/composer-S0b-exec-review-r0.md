# composer S0b — independent adversarial EXECUTION review, round 0

**Artifact:** branch `composer-s0b`, worktree `/scratch/code/shibboleth/wt-composer-s0b`, tip
`87bc10fff17714d8564bc9ec2f22f547934ea244` (3 commits over descriptor-mnemonic `main` = `66bdf2f4`).
**Against:** `IMPLEMENTATION_PLAN_composer_S0b_presets.md` (master, STATUS R0 GREEN) and
`SPEC_wallet_policy_composer.md` §4d / §4e / §12 item 1.
**Reviewer:** independent execution reviewer; did not write the diff. Every claim below is a
command that was run in this session, not a reading of the implementer's report.
**Method:** counterexample construction, five code mutations with proof-of-execution, byte-identity
comparison, and a stale-claim sweep. Read-only: every mutation reverted, `git status --porcelain`
empty at exit.

## Counts

**0 Critical / 0 Important / 4 Minor / 4 Nit.**

Nothing blocks. The diff does what the plan and §4d require: six archetypes reachable from
`md compose --preset`, byte-identical to the equivalent `--path` list, six MANIFEST vectors built by
calling the constructors, correct refusals, and no panic on any input constructed against it.

## Lens disposition

| lens | outcome |
| --- | --- |
| 1. counterexamples against `parse_preset` | ~46 inputs constructed. **No Critical, no Important.** Yielded M-2, M-3, N-1, N-2. |
| 2. byte identity | **Found nothing.** All six archetypes identical across all four comparisons, plus a round trip the brief did not ask for. |
| 3. mutation-testing the tests | **Found nothing.** 5 of 5 behaviour-changing mutations caught; the 6th (M2a) changes no behaviour and is correctly unobservable. No test that cannot fail. |
| 4. what the diff made false elsewhere | Yielded M-1, M-4, N-3. Every count claim in the CHANGELOG and the plan verified true. |
| 5. whole-repo gates as CI runs them | Settled items not re-derived. Closed one gate the settled list omitted (`cargo doc`, clean). Yielded N-4, which nothing gates. |

## Environment

```
CARGO_TARGET_DIR=/scratch/code/shibboleth/.s0b-review-target
TMPDIR=/scratch/code/shibboleth/.tmp
binary under test: /scratch/code/shibboleth/.s0b-review-target/debug/md  (cargo build -p md-cli --locked)
```

---

## Lens 1 — counterexamples against `parse_preset`

30 constructed inputs in batch 1, 16 more in batch 4. **No panic, no accepted malformed input, no
input that was accepted and produced a policy other than the one named.** Every refusal exits 1 with
a wording the plan's refusal table lists, or exits 2 as clap's own. Trimmed results:

```
$ md compose --wrapper wsh --preset kofn-recovery,older=26280,2of3     [exit 0]  (documented: named params by name, in any order)
$ md compose --wrapper wsh --preset " plain-multisig,2of3"             [exit 1] md: --preset  plain-multisig: expected one of plain-multisig, ...
$ md compose --wrapper wsh --preset "plain-multisig, 2of3"             [exit 1] md: preset plain-multisig: k ` 2` is not a small number
$ md compose --wrapper wsh --preset PLAIN-MULTISIG,2of3                [exit 1] md: --preset PLAIN-MULTISIG: expected one of ...
$ md compose --wrapper wsh --preset plain-multisig,2OF3                [exit 1] md: preset plain-multisig: `2OF3` is not <k>of<n>
$ md compose --wrapper wsh --preset plain-multisig,5of3                [exit 1] md: path 1: 5-of-3 is not admitted (1 <= k <= n <= 9)
$ md compose --wrapper wsh --preset plain-multisig,0of0                [exit 1] md: path 1: 0-of-0 is not admitted (1 <= k <= n <= 9)
$ md compose --wrapper wsh --preset hashlock-gated,sha256=<UPPERCASE>,older=26280
                                                                       [exit 1] md: preset hashlock-gated: sha256 needs 64 hex characters, lowercase
$ md compose --wrapper wsh --preset simple-timelocked-inheritance,older=0
                                                                       [exit 1] md: path 2: older in blocks needs 1..=65535
$ md compose --wrapper wsh --preset simple-timelocked-inheritance,older=65536
                                                                       [exit 1] md: path 2: older in blocks needs 1..=65535
$ md compose --wrapper wsh --preset simple-timelocked-inheritance,older=65535   [exit 0]  (band edge, accepted)
$ md compose --wrapper wsh --preset decaying-multisig,...,after=499999999       [exit 0]  (band edge, accepted)
$ md compose --wrapper wsh --preset decaying-multisig,...,after=500000000
      [exit 1] md: preset decaying-multisig: after=500000000 reads as a block height and is above the
               height band (1..=499999999); presets cannot express a Unix time -- use --path with
               `after=500000000t` instead
$ md compose --wrapper wsh --preset decaying-multisig,...,after=0      [exit 1] md: path 3: after height needs 1..=499999999
$ md compose --wrapper wsh --preset plain-multisig,2of3,2of3           [exit 1] md: preset plain-multisig needs exactly 1 <k>of<n> parameter, got 2
$ md compose --wrapper wsh --preset plain-multisig,2of3,               [exit 1] md: preset plain-multisig: `` is not <k>of<n>
$ md compose --wrapper wsh --preset "plain-multisig,,2of3"             [exit 1] md: preset plain-multisig: `` is not <k>of<n>
$ md compose --wrapper wsh --preset ""                                 [exit 1] md: --preset : expected one of ...
$ md compose --wrapper wsh --preset plain-multisig                     [exit 1] md: preset plain-multisig needs exactly 1 <k>of<n> parameter, got 0
$ md compose --wrapper wsh --preset plain-multisig,2of3,unsorted       [exit 1] md: preset plain-multisig: `unsorted` is not <k>of<n>
$ md compose --wrapper wsh --preset kofn-recovery,2of3,older=26280,older=26280
                                                                       [exit 1] md: preset kofn-recovery: `older=` given twice
$ md compose --wrapper wsh --preset plain-multisig,2of3,bogus=1        [exit 1] md: preset plain-multisig admits no bogus= parameter
$ md compose --wrapper wsh --preset "hashlock-gated,sha256=<63 hex>,older=26280"
                                                                       [exit 1] md: preset hashlock-gated: sha256 needs 64 hex characters, lowercase
$ md compose --wrapper wsh --preset "hashlock-gated,sha256=<65 hex>,older=26280"   (same)
$ md compose --wrapper wsh --preset hashlock-gated,older=26280         [exit 1] md: preset hashlock-gated needs sha256=<64 hex>
$ md compose --wrapper wsh --preset "hashlock-gated,sha256=<H>"        [exit 1] md: preset hashlock-gated needs older=<n>
$ md compose --wrapper wsh --preset decaying-multisig,2of2,1of1,older1=13140,older2=13140,after=100
      [exit 1] md: preset: decaying tiers must unlock progressively later (the second older must exceed the first)
$ md compose --wrapper wsh --preset decaying-multisig,1of2,2of2,older1=1,older2=2,after=1
      [exit 1] md: preset: a decaying multisig decays: the recovery threshold cannot exceed the primary threshold
$ md compose --wrapper wsh --preset plain-multisig,2of10               [exit 1] md: path 1: 2-of-10 is not admitted (1 <= k <= n <= 9)
$ md compose --wrapper wsh --preset plain-multisig,300of400            [exit 1] md: preset plain-multisig: k `300` is not a small number
$ md compose --wrapper wsh --path 2of3 --preset plain-multisig,2of3    [exit 2] error: the argument '--path <PATH>' cannot be used with '--preset <PRESET>'
$ md compose --wrapper wsh                                             [exit 2] error: the following required arguments were not provided:
                                                                                  <--path <PATH>|--preset <PRESET>>
$ md compose --preset plain-multisig,2of3                              [exit 2] (missing --wrapper)
$ md compose --wrapper wsh --preset plain-multisig,2of3 --preset kofn-recovery,2of3,older=26280
                                                                       [exit 2] error: the argument '--preset <PRESET>' cannot be used multiple times
```

Panic-hunting inputs (both refuse, neither panics):

```
$ md compose --wrapper wsh --preset "plain-multisig,2of3,x=1,x=1,...(100 tokens)"   [exit 1] md: preset plain-multisig: `x=` given twice
$ md compose --wrapper wsh --preset "plain-multisig,999...(5000 digits)of3"         [exit 1] md: preset plain-multisig: k `999...` is not a small number
```

**No Critical.** The two structural risks I probed for are both closed: `parse_kofn`'s
`split_once("of")` cannot mis-split a name-like token (`older` contains no `of`; `2ofof3` and
`1of2of3` both fail the `n` parse), and every path into the six constructors runs through
`validate`, so `k > n`, `n > 9`, `older` outside `1..=65535` and `after` outside `1..=499999999` all
surface the codec's own wording.

Findings from this lens: **M-2**, **M-3**, **N-1**, **N-2** below.

---

## Lens 2 — byte identity

For each of the six archetypes, four comparisons: `--preset` `--json` `.template` vs the `family()`
row's hand-typed literal; `--preset` vs the equivalent explicit `--path` list on both `.template`
and `.template_with_origins`; and `md encode --force-chunked` of both templates.

```
$ bash /scratch/code/shibboleth/.tmp/probe2.sh
plain_multisig                   tmpl(preset==path)=OK  origins(preset==path)=OK  origins==corpus=OK  chunks(preset==path)=OK
simple_timelocked_inheritance    tmpl(preset==path)=OK  origins(preset==path)=OK  origins==corpus=OK  chunks(preset==path)=OK
kofn_recovery                    tmpl(preset==path)=OK  origins(preset==path)=OK  origins==corpus=OK  chunks(preset==path)=OK
tiered_recovery                  tmpl(preset==path)=OK  origins(preset==path)=OK  origins==corpus=OK  chunks(preset==path)=OK
hashlock_gated                   tmpl(preset==path)=OK  origins(preset==path)=OK  origins==corpus=OK  chunks(preset==path)=OK
decaying_multisig                tmpl(preset==path)=OK  origins(preset==path)=OK  origins==corpus=OK  chunks(preset==path)=OK
fail=0
```

`origins==corpus` compares `--json`'s `template_with_origins` against the committed
`crates/md-codec/tests/vectors/keyed_compose_preset_*.template` file — the MANIFEST form. The
equivalent `--path` lists used were, in order: `2of3`; `1of1` + `1of1,older=26280`; `2of3` +
`1of1,older=26280` (tr); `2of2` + `1of2,older=26280`; `1of1,sha256=<H>` + `1of1,older=26280`;
`2of2,older=13140` + `1of1,older=26280` + `1of1,after=1000000`.

Separately, `--json`'s `.template` was resolved against the `family()` literal by extracting the
Rust literal from `compose_support.rs` and substituting `{NUMS}` / `{HH}`:

```
keyed_compose_preset_plain_multisig                  template==family() literal: OK
keyed_compose_preset_simple_timelocked_inheritance   template==family() literal: OK
keyed_compose_preset_kofn_recovery                   template==family() literal: OK
keyed_compose_preset_tiered_recovery                 template==family() literal: OK
keyed_compose_preset_hashlock_gated                  template==family() literal: OK
keyed_compose_preset_decaying_multisig               template==family() literal: OK
ALL OK
```

**End-to-end round trip** (beyond the brief; "a round trip is not a restore test", so streams were
separated and the read-back compared field by field):

```
$ bash /scratch/code/shibboleth/.tmp/rt2.sh
plain-multisig,2of3                          decode.stdout==json.template: OK | origins: m/48'/0'/0'/2' m/48'/0'/1'/2' m/48'/0'/2'/2'
simple-timelocked-inheritance,older=26280    decode.stdout==json.template: OK | origins: m/48'/0'/0'/2' m/48'/0'/1'/2'
kofn-recovery,2of3,older=26280               decode.stdout==json.template: OK | origins: m/48'/0'/0'/2' ... m/48'/0'/3'/2'
tiered-recovery,2of2,1of2,older=26280        decode.stdout==json.template: OK | origins: m/48'/0'/0'/2' ... m/48'/0'/3'/2'
hashlock-gated,sha256=<H>,older=26280        decode.stdout==json.template: OK | origins: m/48'/0'/0'/2' m/48'/0'/1'/2'
decaying-multisig,2of2,1of1,...              decode.stdout==json.template: OK | origins: m/48'/0'/0'/2' ... m/48'/0'/3'/2'
```

`md encode --force-chunked` of the origin-ful template, then `md decode` of the resulting chunks,
returns the origin-less template on stdout with the origins reported on stderr — distinct §4f
accounts per slot, no origin lost. **No difference anywhere. No Critical.**

### §4d wrapper rule, all 24 (archetype, wrapper) pairs

§4d: *"presets are offered under `wsh` and `tr`; under `sh`/`sh(wsh)` only the plain k-of-n preset
is offered."* The diff adds no CLI-side special case and relies on `ComposeError::LegacyWrapperShape`.
Exercised live:

```
tr     × 6 archetypes  -> 6/6 exit 0
wsh    × 6 archetypes  -> 6/6 exit 0
sh-wsh -> plain-multisig exit 0; the other 5 exit 1:
   md: legacy wrappers hold one plain sorted multisig only (n >= 2, no lock, no hash); use wsh or tr
sh     -> plain-multisig exit 0; the other 5 exit 1 (same wording)
```

The reliance is structurally sound, not lucky: every non-plain archetype hardcodes ≥ 2 paths and at
least one lock, and `validate`'s legacy arm requires exactly one bare, unlocked, unhashed, sorted
path with n ≥ 2 (`crates/md-codec/src/compose/mod.rs:368-376`). No parameterisation of a non-plain
preset can reach that shape.

---

## Lens 3 — mutation-testing the tests

Five mutations. For each: the mutation, **proof the mutated line RAN**, and which test failed. All
reverted; `git status --porcelain` verified empty after each.

### M1a — reverse the two tiers in `family()`'s `tiered_recovery` call

`crates/md-codec/tests/compose_support.rs:317`, `presets::tiered_recovery(Wrapper::Wsh, 2, 2, 1, 2, 26280)`
→ `(Wrapper::Wsh, 1, 2, 2, 2, 26280)`.

```
$ cargo nextest run -p md-codec --test compose_vectors --locked
FAIL every_family_entry_renders_as_listed
  assertion `left == right` failed: keyed_compose_preset_tiered_recovery
    left: "wsh(or_d(multi(1,@0/<0;1>/*,@1/<0;1>/*),and_v(v:multi(2,@2/<0;1>/*,@3/<0;1>/*),older(26280))))"
   right: "wsh(or_d(multi(2,@0/<0;1>/*,@1/<0;1>/*),and_v(v:multi(1,@2/<0;1>/*,@3/<0;1>/*),older(26280))))"
FAIL every_compose_vector_in_the_manifest_is_exactly_what_compose_renders
  assertion `left == right` failed: keyed_compose_preset_tiered_recovery
    left: "wsh(or_d(multi(1,@0/48'/0'/0'/2'/<0;1>/*,...
   right: "wsh(or_d(multi(2,@0/48'/0'/0'/2'/<0;1>/*,...
 Summary  6 tests run: 4 passed, 2 failed
```

**CAUGHT by 2 tests.** The mutated line demonstrably ran: the failure message quotes `multi(1,` on
the left, which only the mutated call can produce.

### M1b — reverse the two tiers in the CLI's `parse_preset` call

`crates/md-cli/src/cmd/compose.rs:339`, `presets::tiered_recovery(wrapper, k1, n1, k2, n2, older_blocks)`
→ `(wrapper, k2, n2, k1, n1, older_blocks)`.

```
PROOF THE MUTATED LINE RAN:
$ md compose --wrapper wsh --preset tiered-recovery,2of2,1of2,older=26280
wsh(or_d(multi(1,@0/48'/0'/0'/2'/<0;1>/*,@1/48'/0'/1'/2'/<0;1>/*),and_v(v:multi(2,@2/...),older(26280))))
        ^^^^^^^^ tiers swapped in real output

$ cargo nextest run -p md-cli --test cli_compose_preset --locked
FAIL preset_tiered_recovery_and_decaying_multisig_and_hashlock_gated_compose
 Summary  21 tests run: 20 passed, 1 failed
```

**CAUGHT.**

### M2a — drop `checked()` from one preset constructor

`crates/md-codec/src/compose/presets.rs`, `simple_timelocked_inheritance`'s `checked(PathList{..})`
→ `Ok(PathList{..})`.

```
$ md compose --wrapper sh --preset simple-timelocked-inheritance,older=26280
md: legacy wrappers hold one plain sorted multisig only (n >= 2, no lock, no hash); use wsh or tr   [exit 1]

$ cargo nextest run -p md-codec -p md-cli --locked -E 'test(/preset|legacy|compose/)'
 Summary  97 tests run: 97 passed
```

**NOT caught — and correctly so.** `compose()` calls `validate()` unconditionally
(`crates/md-codec/src/compose/mod.rs:325`, reached from `compose`/`compose_with`), so `presets::checked`
is redundant defence for the CLI path and dropping it changes no user-visible behaviour. There is
nothing for a test to observe. Recorded because the plan's `parse_preset` doc comment
(`compose.rs:238-241`) reads as though `checked` is what produces the legacy refusal; it is not — see
M2b. This is defence in depth, not a false PASS, and `presets.rs` is S0 code this diff does not touch.

### M2b — drop the legacy-wrapper refusal at the layer that produces it

`crates/md-codec/src/compose/mod.rs:372`, `if !(sole && sorted) {` → `if !(sole && sorted) && list.paths.len() != 2 {`.

```
PROOF THE MUTATED LINE RAN — a §4d-forbidden shape is now ACCEPTED:
$ md compose --wrapper sh --preset simple-timelocked-inheritance,older=26280
sh(or_i(pkh(@0/48'/0'/0'/2'/<0;1>/*),and_v(v:pkh(@1/48'/0'/1'/2'/<0;1>/*),older(26280))))   [exit 0]

$ cargo nextest run -p md-cli --test cli_compose_preset --locked
FAIL preset_every_non_plain_archetype_refuses_under_both_legacy_wrappers_spec_4d_shape
 Summary  21 tests run: 20 passed, 1 failed
```

**CAUGHT.** The §4d test can fail, and fails for the right reason.

### M3 — make an unknown name fall through to token parsing

`crates/md-cli/src/cmd/compose.rs:246-251`: the `if !PRESET_NAMES.contains(&name) { ... }` guard removed.

```
PROOF THE MUTATED LINE RAN — and NO PANIC:
$ md compose --wrapper wsh --preset phantom-preset,2of3
md: preset phantom-preset: internal error -- PRESET_NAMES advertises this name but no lowering rule
exists for it (this is a bug in md, not a mistake in your command)                          [exit 1]
$ md compose --wrapper wsh --preset phantom-preset,NOTAKOFN
md: preset phantom-preset: `NOTAKOFN` is not <k>of<n>

$ cargo nextest run -p md-cli --locked -E 'test(/preset/)'
FAIL preset_refuses_an_unknown_name
FAIL preset_unknown_name_wins_over_a_malformed_token
 Summary  22 tests run: 20 passed, 2 failed
```

**CAUGHT by 2 tests**, and the R0 round-1 fold's second guard is confirmed: with the name guard gone
the CLI still exits 1 via the `other =>` arm rather than panicking.

### M4 — change one MANIFEST template by one character

`crates/md-codec/src/test_vectors.rs:447`, `sortedmulti(2,` → `sortedmulti(3,`.

```
$ cargo nextest run --workspace --all-features --locked
FAIL md-cli::vector_corpus vectors_output_matches_committed_corpus
  thread panicked at crates/md-cli/tests/vector_corpus.rs:41:5: vectors corpus drift detected
 Summary  1171/1340 tests run: 1170 passed, 1 failed, 3 skipped   (nextest fail-fast)

$ cargo nextest run -p md-codec --test compose_vectors --locked --no-fail-fast
FAIL every_compose_vector_in_the_manifest_is_exactly_what_compose_renders
 Summary  6 tests run: 5 passed, 1 failed
```

**CAUGHT by 2 tests**, one on each side (corpus drift in md-cli, MANIFEST-vs-lowering in md-codec).

### M5 — `PRESET_NAMES` gains a 7th name with no `match` arm (the R0 round-1 drift class)

`crates/md-cli/src/cmd/compose.rs:253`, `[&str; 6]` → `[&str; 7]` with `"phantom-preset"` prepended.
This is the mutation the plan claims to have closed; verified independently rather than taken on the
plan's word.

```
$ md compose --wrapper wsh --preset phantom-preset,2of3
md: preset phantom-preset: internal error -- PRESET_NAMES advertises this name but no lowering rule
exists for it ...                                                                          [exit 1, NO PANIC]

$ cargo nextest run -p md-cli --locked --no-fail-fast -E 'test(every_preset_name_parses)'
FAIL md-cli::bin/md cmd::compose::tests::every_preset_name_parses_with_some_valid_parameters
  panicked at crates/md-cli/src/cmd/compose.rs:582:26:
  PRESET_NAMES gained `phantom-preset` with no valid-parameter fixture in this test
```

**CAUGHT**, with the exact message the plan predicted. The embedded unit test does run under the
workspace suite — confirmed separately, unmutated:

```
$ cargo nextest run -p md-cli --locked -E 'test(every_preset_name_parses)'
PASS md-cli::bin/md cmd::compose::tests::every_preset_name_parses_with_some_valid_parameters
```

**5 of 5 behaviour-changing mutations caught; the one absorbed mutation (M2a) changes no behaviour.**
No test in the diff was found that cannot fail.

---

## Lens 4 — what did the diff make false elsewhere

| surface | state |
| --- | --- |
| `md compose --help` | correct. `--path` says "Repeatable. Mutually exclusive with --preset"; `--preset` documents the grammar and states that `older1` locks the FIRST tier; `--json` names the new `preset` field. Usage line reads `<--path <PATH>\|--preset <PRESET>>`. |
| `CHANGELOG.md` `## md-cli [Unreleased]` (:9) and `## md-codec [Unreleased]` (:80) | both entries land in the right crate's `[Unreleased]` `### Added`. No version bump needed (md-cli 0.14.0, md-codec 0.42.0 unchanged, both still unreleased). |
| CHANGELOG count claim "34 tagged / 32 in MANIFEST" | **verified true**: `family()` rows = 34; MANIFEST `(keyed_)?compose_` vectors = 32 (28 keyed + 4 unkeyed). |
| plan's fork claim "126 → 156" | **verified true**: `ls crates/md-codec/tests/vectors/ \| grep -cE '^(keyed_)?compose_'` = **156** = 28×5 + 4×4. Corpus total 287 files, of which 30 are `keyed_compose_preset_*` (6 × 5). |
| `compose_support.rs` doc comments (two-vector rule, `SINGULAR_TAGS`) | updated, but the stated contract no longer matches its contents — **M-1**. |
| `crates/md-cli/README.md`, root `README.md` | neither documents `md compose` at all, at HEAD **or at base `66bdf2f4`** — a pre-existing S0 gap the diff inherits, not one it created. **N-3**. |
| man pages | generated at runtime by `clap_mangen` from the same `Command` tree (`cmd/gen_man.rs`); no committed golden to go stale. |
| insta snapshots | none reference `compose`. |
| `md compose --json` schema | `SCHEMA = "md-cli/1"` is one shared constant (`format/json.rs:6`) across every `--json` command; the new `preset` field is additive and `null` for a `--path`-built policy (verified live). No per-command schema-version rule exists in the repo to violate. |
| Go / fork consumers of `md compose --json` | none (`grep` over `/scratch/code/shibboleth/seedhammer` `*.go`: no hits). |
| any other doc citing a corpus or vector count | swept `--include='*.md' --include='*.rs'`; every hit is a historical `design/agent-reports/*` record about a different corpus. Nothing live falsified. |
| fork `vendor-compose-vectors.sh` follow-on | recorded under F-453 with owning phase S3 — but the two hand edits live only in the S0b plan's prose, and the S3 plan's A10 still says the opposite. **M-4**. |

---

## Lens 5 — whole-repo gates as CI runs them

The brief settles fmt, CI-form clippy, `cargo nextest run --workspace --all-features --locked`
(1340 passed / 3 skipped), threaded `cargo test --workspace --all-features --locked` (1340 passed),
the `design/display-grouping-vectors.tsv` checksum, the corpus counts and the two `--preset` probes.
Not re-derived.

Two things worth recording:

1. **The full suite was re-run unmutated in this session** as a side effect of a mis-targeted
   mutation attempt (the `python3` replace asserted 0 matches and made no edit, so the run that
   followed was against the pristine tree):
   `Summary [25.332s] 1340 tests run: 1340 passed, 3 skipped` — independently reconfirming the
   settled figure at tip `87bc10ff`.

2. **`cargo doc` was NOT in the settled list and is a required CI job** (`ci.yml` job `doc`,
   `RUSTDOCFLAGS: "-D warnings"`). The diff adds ten new doc comments containing `[`-bracketed text
   (`[,<k>of<n>]*`), which is the shape that trips rustdoc's intra-doc-link resolver. Run in CI's
   exact form:

   ```
   $ RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --document-private-items --all-features
   DOC EXIT: 0
   ```

   **Clean.** Gate closed.

`cargo clippy -p md-cli --no-default-features --all-targets -- -D warnings` fails, but on
pre-existing dead code in files this diff never touches, and CI never builds without `json` — see
**N-4**.

---

## Findings

### M-1 (Minor) — `SINGULAR_TAGS`' stated contract is not what seven of its eight entries satisfy

**Where:** `crates/md-codec/tests/compose_support.rs:334-352` (the doc comment and the list),
`crates/md-codec/tests/compose_vectors.rs:68-88` (the enforcing test).

**What.** §12 item 1 is normative and grants exactly one exemption from the two-vector rule: *"a tag
with exactly one legal shape, which is named as such in the test (m = 0 is one unlocked single key
and nothing else)"*. `spine:0` meets that — there is only one legal m = 0 taptree. The seven tags
this diff adds to `SINGULAR_TAGS` do not:

- `head:hashed` (`:346`) is a *shape* tag on the same axis as `head:single` / `head:bare-multi` /
  `head:locked`. A head path that is keys-plus-hash-unlocked has many legal shapes — `2of3` plus a
  hash, the same under `tr`, a three-path list. The diff's own comment is honest about this: it
  justifies the entry as *"the ONLY family vector whose head path is..."*, a statement about the
  corpus, not about legal shapes.
- The six `preset:<name>` tags (`:347-352`) are justified as *"each has exactly one legal vector by
  construction, not by coverage gap"*. That is false as written: `plain-multisig,2of4` under `tr` is
  an equally legal `preset:plain-multisig` vector. They are singular by *deliverable scope* (F-453
  specifies one vector per archetype), which is a scheduling fact, not a structural one.

**Reproduction.** `crates/md-codec/tests/compose_vectors.rs:83-88` asserts `count.get(t) == Some(&1)`
for every `SINGULAR_TAGS` member, and the module doc at `compose_support.rs:334` reads "Tags with
exactly ONE legal shape".

**Why it is only Minor.** Coverage strictly improves against base: `head:hashed`'s shape had **zero**
vectors at `66bdf2f4` and has one now. And the test pins `== 1` rather than `>= 1`, so adding a
second vector later forces an explicit decision rather than silently widening the exemption. Nothing
in §12 item 1's *required* tag list is weakened — that list is untouched and every member still has
≥ 2 vectors (`every_tag_appears_in_at_least_two_vectors` passes).

**Hypothesis (not prescriptive).** Either split the constant into `SINGULAR_TAGS` (one legal shape,
per §12) and a separately-named `SCOPED_SINGLE_VECTOR_TAGS` (one vector by deliverable scope), or
reword the header to state both grounds and drop the "by construction" claim. A second
`head:hashed` vector — the same shape under `tr`, or with `2of3` — would close the coverage half
outright and cost one `family()` row.

### M-2 (Minor) — a wrong `<k>of<n>` count masks a bad named parameter, so one refusal-table row is unreachable in combination

**Where:** `crates/md-cli/src/cmd/compose.rs:257-274` (`need_ofs` defined before `named_only`) and the
per-archetype arms at `:300`, `:319`, … which all call `need_ofs(..)?` before `named_only(..)?`.

**Reproduction.**

```
$ md compose --wrapper wsh --preset plain-multisig,2of3,bogus=1
md: preset plain-multisig admits no bogus= parameter                        <- correct row

$ md compose --wrapper wsh --preset plain-multisig,bogus=1
md: preset plain-multisig needs exactly 1 <k>of<n> parameter, got 0         <- `bogus=` never mentioned

$ md compose --wrapper wsh --preset "plain-multisig,=5"
md: preset plain-multisig needs exactly 1 <k>of<n> parameter, got 0         <- empty key never mentioned

$ md compose --wrapper wsh --preset "plain-multisig,2of3=x"
md: preset plain-multisig needs exactly 1 <k>of<n> parameter, got 0         <- the operator typed a k-of-n
```

The last case is the one an operator can actually hit: a stray `=x` after a well-formed `2of3` turns
the token into a named parameter, and the message then reports *"got 0"* about a command that
visibly contains `2of3`.

**Severity.** Minor, not Important: every case still refuses with exit 1 and nothing malformed is
accepted. The plan's table row *"an admitted-nowhere parameter → `preset <name> admits no <param>=
parameter`"* is simply unreachable when the `<k>of<n>` count is also wrong.

**Hypothesis.** Run `named_only(..)?` before `need_ofs(..)?` in each arm — an unknown key name is the
more specific diagnosis and does not depend on the k-of-n count being right. Alternatively have
`need_ofs`'s message mention any named tokens present, e.g. `… got 0 (\`2of3=x\` was read as a named
parameter)`.

### M-3 (Minor) — `--path`'s `Nu` / `Ht` lock spellings refuse with no remedy, while the sibling case was deliberately given one

**Where:** `crates/md-cli/src/cmd/compose.rs:275-288` (`need_u32`) vs `:289-299` (`need_after_height`).

**Reproduction.**

```
$ md compose --wrapper wsh --preset simple-timelocked-inheritance,older=100u
md: preset simple-timelocked-inheritance older: `100u` is not a number in 0..=4294967295

$ md compose --wrapper wsh --preset decaying-multisig,2of2,1of1,older1=13140,older2=26280,after=500000001t
md: preset decaying-multisig after: `500000001t` is not a number in 0..=4294967295

$ md compose --wrapper wsh --preset decaying-multisig,2of2,1of1,older1=13140,older2=26280,after=500000000
md: preset decaying-multisig: after=500000000 reads as a block height and is above the height band
(1..=499999999); presets cannot express a Unix time -- use --path with `after=500000000t` instead
```

**What.** All three are the same class of operator error — *"the preset grammar cannot express this
lock"* — and the plan reasoned about exactly this class, deliberately adding the `--path` remedy to
the third (R0 fidelity M-3, `need_after_height`). The first two are the *more* likely slips, because
`older=100u` and `after=…t` are legal `--path` spellings the operator may be carrying over from the
sibling flag, yet they get a bare "is not a number" that names neither the reason nor the remedy.

**Severity.** Minor. The outcome is a refusal, not a wrong result, and the plan's refusal table lists
this exact wording (so the implementation matches the plan). Under the journey rule the wrong
outcome here is not worse than telling the operator nothing — it is merely less than the design
already decided the operator deserves in the adjacent case.

**Hypothesis.** In `need_u32`, detect a trailing `u` (for `older*`) or `t` (for `after`) and emit the
`--path` remedy in the same shape `need_after_height` already uses, rather than falling through to
the generic number message.

### M-4 (Minor, cross-repo, owning phase S3) — the fork's pin test now has a live break with the correction recorded only in this stage's plan prose

**Where:** `mnemonic-engrave/design/IMPLEMENTATION_PLAN_composer_S3_fork_gui.md` Task A10
(`:4686-4689` Files list, `:4709` Step 1, `:4736` `git add`) vs
`IMPLEMENTATION_PLAN_composer_S0b_presets.md:1814`, and `mnemonic-engrave/design/FOLLOWUPS.md:15411`
(F-453).

**What.** With the six vectors now committed, re-running the fork's `scripts/vendor-compose-vectors.sh`
moves the vendored corpus from 126 files / 26 names to **156 / 32** — the 156 half of which I
verified directly in this session. The fork's `md/compose_vectors_pin_test.go` hardcodes both the
26-name `composeVectorNames` list and the literal `126`, so it goes red at re-vendor. The S0b plan
states this correctly at `:1814` and states that **A10's own text asserts the opposite** ("it
asserts the file count, so the constant... moves with it"). A10 was not corrected, its Files list
does not name `compose_vectors_pin_test.go`, and its `git add` does not stage it. F-453's own entry
says only "then vendor into the fork", which reads as "run the script".

**Severity.** Minor here, deliberately: the S0b diff is descriptor-mnemonic-only, the plan explicitly
scoped the fork out ("this plan does not touch the fork either way"), and the ownership *is* recorded
in a git-tracked document, so it outlives the agent. But it is now a **live** break rather than a
hypothetical one, and the document the S3 implementer will actually execute still says the wrong
thing.

**Hypothesis.** A one-commit correction to A10 (add `md/compose_vectors_pin_test.go` to its Files
list and its `git add`, replace the Step 1 sentence, fix the `preset_*` → `keyed_compose_preset_*`
prefix and the missing fifth `.conformance.json` file per vector), scheduled before S3 dispatch.

### N-1 (Nit) — lenient numeric parsing accepts `+2of3`, `02of03`, `older=+26280`

```
$ md compose --wrapper wsh --preset plain-multisig,+2of3     [exit 0] wsh(sortedmulti(2,...))
$ md compose --wrapper wsh --preset plain-multisig,02of03    [exit 0] wsh(sortedmulti(2,...))
$ md compose --wrapper wsh --preset simple-timelocked-inheritance,older=+26280  [exit 0] older(26280)
```

Rust's `FromStr` for `u8`/`u32` accepts a leading `+` and leading zeros. Inherited verbatim from
`--path`'s existing `.parse::<u8>()` / `parse_u32` (`compose.rs:27-29`, `:47-53`), so it is
pre-existing behaviour, not something this diff introduced. No wrong result — the accepted spellings
denote the value the operator wrote.

### N-2 (Nit) — `ComposeError::PresetShape` does not name the archetype

```
$ md compose --wrapper wsh --preset decaying-multisig,2of2,1of1,older1=13140,older2=13140,after=100
md: preset: decaying tiers must unlock progressively later (the second older must exceed the first)
```

Every CLI-side message says `preset <name>: …`; this one says bare `preset: …`. The `Display` impl is
S0 code (`crates/md-codec/src/compose/mod.rs:315`) and the plan's table row deliberately specifies
"the `ComposeError`'s own `Display`, verbatim", so this is intentional and unambiguous in practice
(only one preset can be under invocation). Recorded for consistency only.

### N-3 (Nit, pre-existing) — `md compose` is undocumented in both READMEs

`crates/md-cli/README.md`'s command table has no `compose` row (only `decompose`, `:48`), and root
`README.md:147`'s subcommand list omits it. Confirmed identical at base `66bdf2f4`, so `--preset`
inherits an S0 gap rather than creating one. Nothing the diff wrote is falsified; there is simply
nowhere the new flag would have been added.

### N-4 (Nit, not gated) — `--no-default-features` clippy fails, on pre-existing dead code

```
$ cargo clippy -p md-cli --no-default-features --all-targets -- -D warnings
error: function `wrapper_name` is never used          <- compose.rs:544, untouched by this diff
error: fields `network_str` and `json` are never read <- encode.rs:28,36, untouched by this diff
error: ... `PresetParams` variants never constructed  <- new, same cause
error: could not compile `md-cli` due to 11 previous errors
```

`hex32` and `PresetParams` are reachable only from `preset_params_json`, which is
`#[cfg(feature = "json")]`. This adds to an *already broken* configuration: `wrapper_name` and the
`encode.rs` fields fail the same way at base. **No CI job builds md-cli without `json`** — `ci.yml`
uses `--all-features` throughout, and the `musl` and `freebsd` jobs use default features (which
include `json`). Not gated, not a regression; recorded so it is a known state rather than a surprise
if someone ever adds a no-default-features job.

---

## What I did NOT find

Stated explicitly so a later round does not re-spend budget here:

- **No Critical and no Important.** No accepted malformed input, no panic on any of the ~46
  constructed inputs, no vector pinned to a shape the constructors do not produce, no test that
  cannot fail.
- **Lens 2 found nothing.** All six archetypes are byte-identical to their explicit `--path`
  equivalent on both template forms, identical to the committed corpus, identical in `md encode
  --force-chunked` chunks, and round-trip through `md decode` with origins intact.
- **The §4d wrapper rule is exactly enforced**, 10/10 legacy refusals and 12/12 acceptances, with no
  CLI special case and no shape that could slip past `LegacyWrapperShape`.
- **The R0 round-1 fold's two fixes are real**, verified against the same mutation the plan names
  (M5): the unit test fails with the predicted message and the CLI exits 1 instead of panicking.
- **`preset_never_needs_experimental` is a true claim, structurally**: `Experimental` has exactly two
  variants (`KeylessPath`, `UnsortedKeys`, `mod.rs:173-178`), `presets::ks` hardcodes `sorted: true`,
  and every preset path carries keys — so neither variant is reachable from `--preset`. Confirmed
  live: no stderr warning under `--experimental`.
- **No slot- or path-cap overflow is reachable** from any preset parameterisation: presets emit at
  most 3 paths (cap 8) and at most 19 slots (`tiered-recovery,9of9,9of9` = 18; `decaying-multisig`
  9+9+1 = 19; cap 32).

## Closing counts

**0 Critical / 0 Important / 4 Minor (M-1 … M-4) / 4 Nit (N-1 … N-4).**

Nothing blocks the gate. M-4 is the only item with a scheduling consequence, and it belongs to S3.

Worktree left clean: `git status --porcelain` empty at tip `87bc10ff`; all five mutations reverted
from backups in `/scratch/code/shibboleth/.tmp/bak/` and each revert verified before the next
mutation.
