# IMPL — P2 of `IMPLEMENTATION_PLAN_descriptor_input_S1_S3.md`

**Executed by:** the single P2 implementer, 2026-08-29.
**Branch:** `impl/descriptor-s1s3`, head **`5abffc9`**. Worktree
`/scratch/code/shibboleth/_work/impl-s1s3`. Nothing pushed, no tags, no
publishes, nothing on-device; the main checkout at
`/scratch/code/shibboleth/mnemonic-engrave` was never touched.

**Verdict: P2 is COMPLETE and every clause of the P2 gate passes.** `--as md1`
runs end to end on all four of §4's formats; §6's 36 rows each have a named test
asserting the TEXT; the four P2-tagged `#[ignore]`s are gone and three of the
assertions they were parking are CROSS-LANGUAGE — the four `wallet_id` values
`me` computes match the four the Go route measured, first run.

All nine carry items from the P1 round are disposed of below, each with its
measurement. Six findings of my own follow them.

---

## 1. What was built, per step

| file | lines | what it is |
| --- | ---: | --- |
| `descriptor/md1.rs` | 425 | §5.3's in-process `md_codec::Descriptor` build, and the DERIVATION TWIN |
| `descriptor/identify.rs` | 222 | §5.4's two-tier block, §5.3(b)'s label warning, §5.1's window text |
| `descriptor/as_flag.rs` | 127 | the `--as` driver: single-document contract, block, follower |
| `tests/descriptor_refusals.rs` | 905 | §6's 36 rows + the self-counting gate + §11 item 5 |
| `tests/descriptor_as.rs` | 496 | P2.1's flag surface, §11 item 2's walk, F-421 |

Plus edits to `admit.rs`, `cascade.rs`, `cascade_tests.rs`, `gate.rs`,
`mod.rs`, `refusal.rs`, `main.rs`, `tests/descriptor_seam.rs`.
**17 files, +3,238 / −186** (`git diff --stat 44e121a..HEAD`).

### P2.1 — the flag surface

`--as <descriptor|md1>` on `me sysw pack`, a clap `ValueEnum` so a third value
is a usage error rather than a runtime surprise. §5.1's **single-document
contract** is implemented: more than one argv operand, or argv together with
`--in`, is `EXIT_USAGE` (2) — *"--as packs exactly one descriptor per
invocation."* This is the half P1 could not implement, having no flag to hang it
on, and it closes IMPL-P1's F-3 and N2: the choice block no longer advertises a
flag the binary rejects.

**The driver sits AFTER `read_records`, deliberately.** The argv bearer/secret
guard runs in there, and a flag-shape usage error must not pre-empt a refusal
about material already in `/proc`, `ps` and the shell history. Measured and now
pinned (`the_argv_bearer_guard_precedes_the_single_document_check`).

### P2.2 — the md1 build path

`descriptor::md1` builds `md_codec::encode::Descriptor` in process: per-key
(a)/(a′)/(a″), the `multi` twins, the TLVs, shared-vs-divergent path decl,
`split` for the records — which `sysw::classify` already calls `Class::MdMk`, so
there is no new class and the three programs that admit a descriptor record
admit these today.

**Four construction decisions are read from the Go route's source, not
inferred**, because §7's `wallet_id` column is the gate between the two
languages:

| decision | here | `md/encode_multisig.go` |
| --- | --- | --- |
| pubkeys TLV | every key, idx-ascending | `pubPresent: true`, all `n` (`:150`) |
| fingerprints TLV | only non-zero fingerprints | `if c.FpPresent` (`:157`) |
| key order | input order fixes `@0..@n-1` | the ordering contract (`:16-24`) |
| use-site | §5.3(a′)'s materialised `<0;1>/*` | hard-coded `<0;1>/*` (`:167`) |

The origin declaration is the one deliberate difference — Go always writes
`OriginDivergent`, this writes `Shared` when every key's origin is the same
path. `compute_wallet_policy_id` resolves per-`@N` origins through
`expand_per_at_n` before hashing, so the id is identical either way. That is
**asserted, not assumed**: 4 of 4 `wallet_id` rows match.

**Conjunct 8 refuses BEFORE any of it runs** (`as_flag::md1_follower` calls
`admit(d, Path::Md1)` first), so the published `md-codec` 0.42.0's missing
F-217/F-218 validators are never reached.

### P2.3 — §5.4's identification block

Two tiers. The tier predicate is exactly `admit(d, Path::Md1).is_ok()` — one
call, not a re-derivation — because the md1 path's shape set is the descriptor
path's plus the three `multi` twins, so *"passes conjuncts 2–8 AND at least one
path admits the shape"* reduces to it.

**The block prints on EVERY successful whole-input parse**, including the
`--as`-OMITTED path: §5.1's choice block is one of §5.4's own enumerated
followers, not an exception to it. `Outcome::AsDecides` accordingly lost its
`announcement` payload — §4.5's announcement is the block's last element now.

`address 0:` needed a derivation for wallets md1 cannot ENCODE (an (a)- or
(a″)-shaped wallet is FULL-tier and its `wallet-id: none` line explicitly tells
the operator to identify it *by address 0*). The **derivation twin** solves it,
and each mapping is an EQUALITY rather than an approximation:

| use-site | encoded as | derived via, for address 0 |
| --- | --- | --- |
| absent | `<0;1>/*` (a′) | chain 0, index 0 |
| `/*` | `/*` | index 0 |
| `<i;i+1>/*` | `<i;i+1>/*` | chain 0, index 0 |
| `/i/*` | REFUSED (a) | `<i;i+1>/*`, chain 0 index 0 — `key/i/0` |
| `<i;i+1>` | REFUSED (a″) | `/*` at index `i` — `key/i` |

The twin is never encoded, never packed and never hashed into an identity.
It is validated against the file's own device-measured values: `md1-split/
fixed-index` → `bc1qadgf37z…`, `md1-split/multipath-no-wildcard` →
`bc1qu2cc6t7…`, `md1-split/mixed-nowildcard-and-multipath` → `bc1qghwumhc…`.
All three are the addresses the Go `address` package derives.

### P2.1's window text (written after P2.3, per the build order)

Both variants, chosen by md1-representability, with every offending key named.
**The §4.7 admission refusal precedes it**, so a `multi` form under
`--as descriptor` gets conjunct 1's PERMANENT refusal and never the window.

### P2.4 / P2.5

Below, with the carry items.

---

## 2. The P2 gate — actual output, pasted

```
$ cargo nextest run --locked
     Summary [  32.167s] 544 tests run: 544 passed, 1 skipped
```

P1 closed at `485 passed, 5 skipped`. **+59 tests, −4 skips.** The one remaining
skip is `sysw::vectors::tests::regenerate`, whose own doc comment reads *"Not a
test. `--ignored` so it never runs in CI: a fixture that rewrites itself asserts
only that today's code agrees with today's code"* — pre-existing since
`f2bb8c2` (2026-08-11), a fixture GENERATOR and not a parked assertion.

```
$ grep -c '^#.ignore' crates/me-cli/tests/descriptor_seam.rs     -> 0
$ grep -rn '^#\[ignore' crates/ | wc -l                          -> 0
$ grep -rn '#\[ignore' crates/                                   -> 1 hit, the generator above
```

```
$ cargo clippy --all-targets --locked -- -D warnings
    Finished `dev` profile [optimized + debuginfo] target(s) in 0.32s   (exit 0)

$ cargo fmt --check
    (exit 0, no output)
```

**The row-count assertion**, run and read rather than described:

```
$ grep -c '^fn row_' crates/me-cli/tests/descriptor_refusals.rs  -> 36
    PASS mnemonic-engrave::descriptor_refusals the_file_carries_one_named_test_per_section_6_row
```

**Per-binary test counts** (from the single captured run, greppped twice):
`descriptor_refusals` 41 · `descriptor_seam` 14 · `descriptor_as` 11 ·
`descriptor::*` unit tests 38.

**The fork's Go seam test**, with the vector file byte-untouched:

```
$ sha256sum crates/me-cli/testdata/descriptor_seam_vectors.json \
            /scratch/code/shibboleth/_work/seam-fork/nonstandard/testdata/descriptor_seam_vectors.json
0393592f234b0a5264eb7f49553ab3b3911085cd2d1cd8052690018c7fe80584  (both)

$ git diff 44e121a..HEAD --stat -- crates/me-cli/testdata/descriptor_seam_vectors.json
(empty — the file is byte-identical to the P1 head)

$ git -C .../seam-fork status --porcelain     (empty; seam/descriptor-vectors @ 1f09537)

$ go test ./nonstandard/ -count=1     ->  ok  seedhammer.com/nonstandard  0.019s
$ go vet ./nonstandard/               ->  clean
$ gofmt -l nonstandard/               ->  clean
```

All 7 Go tests pass and `TestDescriptorSeamSyswClass` skips with S2's named
reason. **A Go toolchain IS on this box** — `/nix/store/33fw5m31lfcnk4ff2f0df7j2bxnh8lgk-go-1.26.3/bin`,
go1.26.3 — which closes the one gap the P1 reviewer named in their negative
("no Go toolchain is on this box's PATH, so I could not run [a differential]").
It is not on the default `PATH`; it has to be prepended.

**Staleness, all three repos:**

```
$ ./scripts/plan-staleness-check.sh <plan> . eccbc74
─── unchanged: 4 ; DRIFTED: 0 ; not in this repo: 4

$ ./scripts/plan-staleness-check.sh <plan> .../seam-fork d402f18
─── unchanged: 3 ; DRIFTED: 0 ; not in this repo: 5

$ ./scripts/plan-staleness-check.sh <plan> .../descriptor-mnemonic 6864f377
─── unchanged: 1 ; DRIFTED: 0 ; not in this repo: 7
```

The phase's ONE drift was `main.rs:335` → `:371` (P2.1's value enum pushed
`const EXIT_OK` down 36 lines). Re-pinned in its own commit `eccbc74`, which is
what the plan's "re-pinned at each phase gate" rule calls for; the run above is
against that commit.

---

## 3. Carry-item dispositions

### 1. M1 + the spec amendment (P2.4) — **FOLDED, in a spec-only commit**

`de35e30`, spec only, before the P2.4 code that pins the texts.

§6's transposition carve-out is generalised from ONE row to the whole `multi`
class: `bip380.Parse`'s script switch has a `sortedmulti` case and no `multi`
case, so the device refuses every `multi` form at PARSE and no
device-behaviour claim transposes to one. The replacement clause is stated once
in the spec and once in the code (`MULTI_AT_PARSE`), and the status header marks
the amendment exactly as the conjunct-8 amendment is marked.

Five rows changed for a `multi` input; the sixth (single-key wrapper) was
already carved out and keeps its own remedy substitution — it is the test's
CONTROL, so an over-broad amendment would red.

Measured after the fold, on the same inputs the review constructed:

| input | now prints |
| --- | --- |
| `tr(multi(…))` | *"…is not a valid descriptor -- and the device's parser refuses `multi` outright, so this file never reaches address derivation there. Check the export."* |
| mixed-network `multi` | *"…The device's parser refuses `multi` outright, so this file never reaches address derivation there. All keys must share one network."* |
| `<0;1>/*h` `multi` | *"…(BIP-32). The device's parser refuses `multi` outright… This is refused on both `--as` paths regardless."* |
| `<0;2>` `multi` | *"the device derives only `<i;i+1>` pairs (receive; change). The device's parser refuses `multi` outright…"* |
| 16-key `sh(multi(…))` | *"`sh(multi(…))` carries at most 15 keys … The device's parser refuses `multi` outright…"* |

Note the last row also substitutes the FORM NAME in the bound sentence
(`sh(sortedmulti(…))` → `sh(multi(…))`), which §6's transposition rule requires;
the BOUND itself does not transpose, because it is the redeemScript's and not
the ordering's.

**Two defects found by READING the emitted text, neither caught by any
assertion** — the IMPL-P1 F-4 lesson repeating: a doubled period where the
clause spliced mid-sentence (`…derivation there.. Check the export.`), and a
`\` line-continuation that `rustfmt` had collapsed leaving six literal spaces
inside the sentence (`so this file      never reaches`).

### 2. M4 (P2.4) — **FOLDED; the code moved to the spec, not the reverse**

Restored: `(BIP-383)` and `` (`OP_CHECKMULTISIG`) `` on the key-count row,
`(BIP-387)` on the taproot row, `(BIP-32)` on the hardened use-site row,
*"— taproot single-sig"* on the promotion row, and backticks around
`` `tpub` ``/`` `xpub` `` on the mixed-network row. The promotion gloss is
generalised past §6's one example: 86h → *"taproot single-sig"*, 48h →
*"a multisig cosigner account"*, 45h → *"a legacy multisig account"*.

**`'` vs `h`: DECIDED as `h`, consistently, and recorded.** Reasons, in order:
it is `bip32.Path.String()`'s own rendering; it is what the canonical
`descriptor:` line two lines above already prints; and §5.2 names `'` → `h` as
part of the canonicalisation the operator is explicitly told about. Printing
`'` in one sentence and `h` in the next IS the drift.

I also reconciled §5.3's **window substitution** to the spec's own stock
clause, which P1 had paraphrased at greater length:

```
before: "The path that carries a fixed chain index exactly is the scannable QR
         plate, and it needs device firmware this release does not include.
         Nothing is lost by waiting: keep your export file, and it packs the
         day the update ships."
after:  "The scannable-plate path is not in this build -- keep the export file;
         it packs when the device update ships."      (§5.3, verbatim)
```

and the `multi`-form neither-path replacement to §6's own wording with the
offending path substituted.

### 3. M2 (P2.3/P2.4) — **CHOSEN: name no script, and say so**

`suggested_descriptor_for` returns `Option<String>`. `purpose_script` is
hardened-only and closed over `{44,45,48,49,84,86}`; an unhardened purpose falls
out BEFORE the subtraction rather than through `wrapping_sub`.

**Stated reason:** the constellation's precedent is F-417's md1-narrow ruling —
refuse and name, rather than widen to fit a use site. A `pkh(…)` remedy for a
native-segwit key is executable and admitted, so §6h was satisfied while the
guess was unsound; the failure mode is a harder restore, and the cost of
refusing is one sentence.

Measured on the file's own row, before and after:

```
input:  [4bbaa801/84/0/0]zpub6qpFgGWoG7…kSRzJx
before: "This one is `m/84/0/0`, which is not inferable.
         Supply the descriptor instead: pkh([4bbaa801/84/0/0]zpub…/<0;1>/*)"
after:  "This one is `m/84/0/0`, whose first step is not one `me` recognises --
         it names no script, and `me` will not guess which wallet you meant.
         Supply the descriptor your wallet software shows, with this key and
         this origin in it."
```

**The §6h gate CAN now fail on an unsound guess**, which is the half M2 asked
for and which the review measured missing. `an_unrecognised_purpose_names_no_script`
exercises BOTH `None` arms (the unhardened `84/0/0` the review measured, and a
hardened-but-unmapped `99h`), asserts no script token follows a `: `, and
asserts the refusal SAYS it is not guessing. Mutation table below.

### 4. M3 (P2.4) — **FOLDED; the loop runs all three generators**

`every_remedy_me_prints_is_an_input_me_admits` (renamed from
`every_promotion_remedy_…`) now covers `suggested_descriptor_for`,
`promotion_fingerprint_no_path` and `unsupported_key_version`'s five
per-version remedies. Two remedy builders were EXTRACTED
(`remedy_full_origin`, `remedy_for_version`) so the gate feeds back the string
the row actually built rather than scraping it out of a sentence — and each row
is asserted to CONTAIN the remedy it built, so the two cannot drift into an
elided copy.

**The floor is re-derived, not guessed:** 2 (generator 1 — of 4 inputs, one is
the multisig-cosigner exemption and one is M2's no-script case) + 2 (generator
2) + 3 (generator 3 — of 5 versions, `Upub`/`Vpub` are testnet MULTISIG accounts
with no single-key form) = **7**. P1's floor was 3. Generator 3 additionally
asserts §6's per-version SOUNDNESS from a table in the test rather than from the
code that produced it.

### 5. M5 (P2.4) — **FOLDED, in the same spec commit as M1**

§6's key-identity row now reads *"Check the export: one of the two entries
carries the wrong key, and a copied-and-edited cosigner is the usual cause."*
The annotation outside the quote records the measurement: in a BlueWallet file
all keys share one `Derivation:` header, so a same-`(fingerprint, origin)` pair
IS a same-header-key pair, which `seenKeys` catches first as `inconsistent
header value` — branch 1 fails and §6 row 1 carries the reason. The row is
reachable from a BIP-380 descriptor and from the JSON wrapper. `refusal.rs`'s
doc comment carries the same reasoning at the constructor.

### 6. F-1 (P2.2) — **CLOSED, the direct-construction unit test**

`admit::conjunct_reachability` builds a `Parsed` with no parser in the way —
every field of `Key` is public, which is exactly the risk F-1 names — and
asserts conjunct 4 refuses each of the five non-admitted versions and conjunct 6
refuses a fingerprint-without-origin. **Each has a control**: an admitted
version on the identical descriptor passes, and an all-zero fingerprint with no
origin is admitted (without that half, a conjunct 6 that merely required a
non-empty origin would satisfy the first assertion).

It is a unit test and not a vector row, per the reviewer's own scoping: no
cascade-reachable input can produce either state, so a row would be a lie about
what the two parsers do.

### 7. F-2 (P2.4) — **ASSERTED in its substituted form**

`row_bluewallet_no_name` pins *"this is a BlueWallet setup file -- it has 1
cosigner line -- but no `Name:` header, and the device requires one. Add a line
`Name: <anything>`."* The test's own doc comment states why the enumeration
substitutes. The row's `source` annotation in the vector file is untouched —
that is P3's, and the file is byte-identical.

### 8. N1 (inline) — **FIXED**

```
before: "…which failed because: the use-site path is not a path: ``."
after:  "…which failed because: the key ends in `/` with no use-site path after it."
```

A guard arm ahead of the general one; the general arm is unchanged and still
correct for a non-empty tail.

### 9. M6 — **MEASURED, left as-is, noted for P3**

The §11 item 2 walk did NOT surface it (the walk's exemplars are well-formed).
Measured separately, and there is a NEW half worth recording:

```
$ printf '{"label":"x","descriptor":"wsh(sortedmulti(2,' > trunc.json

$ me sysw pack --in trunc.json                 (--as OMITTED)
  me: record 0 … is not a form this container can place … see sysw::classify
  (the shipped record refusal, exit 4 — unchanged from P1, faithful to §5.1's T1–T4)

$ me sysw pack --as md1 --in trunc.json        (--as PRESENT)
  me: this is not a wallet descriptor in any of the four forms `me` reads: …
      It looks most like a plain BIP-380 descriptor, which failed because:
      script: missing `)`.
```

So P2 gives the operator a route to the real reason that P1 did not have: naming
`--as` bypasses the shape gate by declaration. **T4 was not widened** — the
review's instruction was explicit that it must not be without a gate row to pin
the widening. For P3's records: the residue is only the `--as`-omitted spelling,
and the best owner is still a journey walk.

---

## 4. Mutation table — the new enforcement points

Every mutation run against the WHOLE workspace suite with `--no-fail-fast`, so
"caught" means "no test anywhere passes it", not "no test in this file".

| # | mutation | verdict | what reds |
| --- | --- | :-: | --- |
| 1 | `conjunct_4_versions` removed from `admit()` | CAUGHT | `conjunct_4_refuses_a_version_outside_the_admitted_five` (489: 488 pass, 1 FAIL) |
| 2 | `conjunct_6_origins` removed from `admit()` | CAUGHT | `conjunct_6_refuses_a_fingerprint_with_no_origin` (489: 488 pass, 1 FAIL) |
| 3 | `device_clause` transposes to `multi` again | CAUGHT | `no_device_behaviour_claim_transposes_to_a_multi_input` (541: 540 pass, 1 FAIL) |
| 4 | the pre-fix `suggested_descriptor_for` restored (`wrapping_sub` + `_ => P2PKH`) | CAUGHT | `an_unrecognised_purpose_names_no_script` **and** `every_remedy_…` (500: 498 pass, 2 FAIL) |
| 5 | generator 3 dropped from the §6h loop | CAUGHT | `every_remedy_me_prints_is_an_input_me_admits` (541: 540 pass, 1 FAIL) |
| 6 | one `fn row_*` renamed away | CAUGHT | `the_file_carries_one_named_test_per_section_6_row` (541: 540 pass, 1 FAIL) |
| 7 | F-421's referral removed | CAUGHT | `the_converter_refers_a_descriptor_to_sysw_pack` (542: 541 pass, 1 FAIL) |

**7 of 7 caught.** Mutations 1 and 2 are the ones that matter most: P1 measured
each of them leaving the whole 485-test suite GREEN.

A partial mutation is worth recording as a NEGATIVE result. My first attempt at
#4 restored only `_ => Script::P2PKH` and left the unhardened guard in place —
**the suite stayed green at 500/500**, because the unhardened case
short-circuits before that arm. The test was then widened to exercise both
`None` arms, and the full revert is what mutation 4 above runs. A mutation that
does not red is either a missing test or a mis-aimed mutation, and here it was
the second telling me about the first.

The mutation harness is `cp` to a scratch backup and `cp` back — **not**
`git checkout`. An earlier round used `git checkout <file>` and silently
reverted uncommitted work on that file along with the mutation; the loss was
caught by `git status` immediately afterwards and the work re-applied, but the
lesson is worth the line.

---

## 5. §11 item 2 — the walk transcript

`me sysw pack --as md1 --out container.bin --in <each of the four formats>`,
then `me sysw show`. Run at `5abffc9`; the test
`item_2_every_format_packs_reads_back_and_derives_the_device_address` asserts
the same four through the CONTAINER (`sysw::open` → classify → `reassemble` →
`descriptor_to_template` → `derive_address`), not through the builder.

### (1) §4.2 BlueWallet — the fork's own 14-line `sh` fixture (J1's artifact)

```
me: read as: a BlueWallet `Key: value` setup file
      descriptor: wsh(sortedmulti(2,[5a0804e3/48h/0h/0h/2h]xpub6F148…FiFLAYk8,[dd4fadee/…]xpub6Dnedi…cXMeVjf,[9bacd5c0/…]xpub6EefrC…N6VLEC))#tk50fvpm
      script: wsh, sortedmulti 2 of 3 keys
      wallet-id: a67e07d16b2500fde6c557a76c7390f6
      address 0: bc1qtahtpjkgtljxl20jgevs2tjhgzvd87jepcrsd92kcyvtzkj34mnsq0j928
      compare against your wallet software's first receive address before engraving.
      watch-only: public keys only -- this wallet description can SHOW its addresses and balances; it cannot spend. …
      template: wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*))
      keys: @0=5a0804e3 @1=dd4fadee @2=9bacd5c0
      note: your input names no derivation below the key origins; `<0;1>/*` is the standard receive/change continuation …
me: warning: the label "sh" is not carried by any record format and will not appear on the device. Nothing else is lost.
rc=0
$ me sysw show container.bin
  sealed: false · pub_len: 508 · digest: e160 45b3 8b80 cf81 f0f4 63de e919 5f23
  public record 0..5: md1/mk1 — confirmed          (6 records)
```

`wallet-id` and `address 0` are the vector file's own values, measured by P0
through the Go route. **§5.3(b)'s label warning fires on J1's `Name: sh`
fixture** — PLAN-r1's I5, discharged.

### (2) §4.3 plain BIP-380

```
me: read as: a plain BIP-380 descriptor
      descriptor: wsh(sortedmulti(2,…/<0;1>/*,…/<0;1>/*,…/<0;1>/*))#ud8uyjz3
      script: wsh, sortedmulti 2 of 3 keys
      wallet-id: 9e95257e60aacbb260129dac7b36d9f4
      address 0: bc1q4taqq6q6l8fvguva6ftvrz3qgdjy6p3w2s0ds0nl6qrjw7t0hfhqgrqcwd
      template: wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*))
      keys: @0=dc567276 @1=f245ae38 @2=c5d87297
rc=0        digest: f631 1fd0 ac3a a97c 0ee1 4ea1 5e30 2883 · 6 records
```

No (a′) note, correctly — this input names its own `<0;1>/*`.

### (3) §4.4 `{label, descriptor}` JSON — **a non-`/0/*` exemplar, as item 2 requires**

CONSTRUCTED: the fork's own JSON fixture is `/0/*`, which `--as md1` refuses per
§5.3(a). The descriptor inside is the vector file's own BIP-380 row, so the
`address_0` compared against is still a Go-measured value.

```
me: read as: a `{"label":…,"descriptor":…}` JSON export
      … wallet-id: 9e95257e60aacbb260129dac7b36d9f4
      address 0: bc1q4taqq6q6l8fvguva6ftvrz3qgdjy6p3w2s0ds0nl6qrjw7t0hfhqgrqcwd
me: warning: the label "Test Multisig 2-of-3" is not carried by any record format …
rc=0        digest: f631 1fd0 ac3a a97c 0ee1 4ea1 5e30 2883 · 6 records
```

**The container digest is byte-identical to (2)'s** — the same wallet through a
different wrapper produces the same records, and the label is the only thing
lost, which is exactly what §5.3(b) says.

### (4) §4.5 the promoted bare key

```
me: read as: a single extended key
      descriptor: wpkh(xpub6C9j4wAxxk…acoGnx)#u8s2vf65
      script: wpkh, single-key
      wallet-id: e657ccee1d44ccd746a5ba2b82ceed16
      address 0: bc1q5hrky4qgk2yqdg334z0g0r4348jvuj8khremkg
      template: wpkh(@0/<0;1>/*)
      keys: @0=<no master fingerprint>
      note: your input names no derivation below the key origins; …
      this is a single extended key, and `me` inferred a whole wallet from it:
      key as supplied: zpub6qpFgGWoG7bKmDDMvmwHBvg6inZAb2KF2Vg8h4fKJ2ickSZ71PsMmRg1FyRWAS6PqPCSzd5CB6PHixx64k6q5svZNZd9bEoCWJuMSkSRzJx
      inferred wallet: wpkh(xpub6C9j4wAxxk…acoGnx)#u8s2vf65
      The key's serialisation was normalised …
rc=0        digest: f2eb a1c7 bbf7 3735 a41b 3a5f 9584 7f8e · 2 records
```

`address 0` is the vector row's own value. `@0=<no master fingerprint>` rather
than `@0=00000000`: an all-zero fingerprint is *"master unknown"*, and printing
zeros would invite a comparison against a coordinator that can never match.

**All four formats: exit 0, records `MdMk`, template read back through the
container, address 0 equal to the Go route's.**

---

## 6. Findings

### F-1 (decision, stated rather than slipped in) — a new DIRECT dependency, `bitcoin = "0.32"`

§5.4's `address 0:` line is the executable check the operator makes before
anything is engraved, and it must exist for wallets md1 cannot encode.
`md_codec::Descriptor::derive_address` is the derivation; its signature takes a
`bitcoin::Network`, so the type has to be nameable in this crate.

**It is not a parser dependency.** §4.7 weighed `rust-miniscript` for PARSING
and rejected it; that decision stands and `descriptor::cascade` is still the
small seven-shape parser. This is the same use the constellation sibling makes
of the crate (`descriptor-mnemonic` `crates/md-cli/Cargo.toml:34`,
`cmd/address.rs:44`). It adds NO crate to the tree — measured:

```
$ git diff --stat Cargo.lock       ->  1 file changed, 1 insertion(+)
+  "bitcoin",         (under [[package]] name = "mnemonic-engrave")
```

`bitcoin 0.32.101` and `miniscript 13.1.0` were already in `Cargo.lock` as
md-codec's own dependencies (its `derive` feature is default-on).

### F-2 (structural, and it is the reason P2.1 and P2.2 share a commit)

The plan's build order is `P2.1's flag skeleton → P2.2 → P2.3 → P2.1's window
text → P2.4`. The ORDER is preserved exactly — the window text is written after
md1-representability and the block exist. But P2.1 and P2.2 could not be two
commits: a flag whose success path is unimplemented does something no spec
sentence describes, and the P2 gate forbids an `#[ignore]` to park it under.
They land as `d711a59` with the reason in the message.

### F-3 (Minor, on §5.4) — one address-line branch has no spec text behind it

For a descriptor mixing a `<i;i+1>`-without-wildcard key at `i ≠ 0` with a key
of another shape, no single receive index serves every key, and the twin cannot
derive one address 0. §5.4 does not contemplate the case. Rather than print a
wrong address above a "compare this" prompt — the one failure that line exists
to prevent — the block prints:

> *"address 0: not derived -- this wallet's keys take their first receive
> address at different depths, so there is no single first address to compare.
> Check the descriptor line against your wallet software instead."*

**This text is mine, not the spec's**, and I am flagging it as such. It is
unreachable from any vector row — the only `<a;b>` groups in the file are
`<0;1>` and `<0;2>`, and `<0;2>` is refused by conjunct 7 long before this line
— but the branch IS reachable, and I constructed it rather than asserting it
was not:

```
$ me sysw pack --as md1 'wsh(sortedmulti(2,[dc567276/48h/0h/0h/2h]xpub…/<2;3>,[f245ae38/48h/0h/0h/2h]xpub…/<0;1>/*))'
      wallet-id: none -- this wallet has no md1 policy form; identify it by the
                 checksum in the descriptor line and by address 0.
      address 0: not derived -- this wallet's keys take their first receive
                 address at different depths, so there is no single first
                 address to compare. …
```

Note the interaction the pair of lines has: `wallet-id: none` tells the
operator to identify the wallet *"by address 0"*, and the next line says there
is no single address 0. That is honest but it is a dead end, and it is the
strongest argument for a reviewer ruling the branch out of existence instead.
Owner: P3's records, or a spec sentence.

### F-4 (Minor, on §5.4's PARTIAL tier) — a reading I had to make

§5.4 defines the PARTIAL block as *"the first three lines plus the watch-only
line — no `wallet-id:`, no `address 0:`, no compare prompt"*. The exclusion list
enumerates exactly three things, and §4.5's promotion announcement is a
CONDITIONAL bullet of the full list rather than one of them.

I print the promotion announcement in **both** tiers, because §4.5 is normative
that promotion is *"ANNOUNCED, not silent"* and a PARTIAL-tier promoted key
would otherwise show the operator a canonical `pkh(…)` they never wrote with no
explanation. Reachable, and run rather than reasoned about:

```
$ me sysw pack --as md1 "[4bbaa801/44'/0'/0']xpub…/0/1/*"
me: read as: a single extended key
      descriptor: pkh([4bbaa801/44h/0h/0h]xpub…/0/1/*)#86runggg
      script: pkh, single-key
      watch-only: …                                    <- PARTIAL: no wallet-id,
      this is a single extended key, and `me` inferred     no address 0, no
      a whole wallet from it: …                            compare prompt
me: use-site paths `me` ACCEPTS: absent, `/*`, `/i/*`, `<i;i+1>`, `<i;i+1>/*`. …
```

Recorded so a reviewer can rule the other way cheaply.

### F-5 (Minor, on the vector file — for P3, alongside F-2's `source` fix)

`md1-split/mixed-nowildcard-and-multipath` carries `md1_admits: false` and an
`address_0` (`bc1qghwumhc…`) that only the DEVICE route derives. That is
correct and the Go half owns the assertion. But the Rust half's address loop
therefore skips it, and the row's address is the one value in the file that
`me`'s own derivation reproduces (through the twin, in the identification block)
without any test comparing the two. I verified it by running it — the block prints
`bc1qghwumhcahkfca7qktym7f3htf5wqakz2tyvxraf3fk5k8w0yrzwsg0m3sd` for that
input, character for character the file's value — and did not add an assertion, because doing so
would mean asserting a value on a row whose `md1_admits` column says the md1
route does not carry it. **Best fixed by a vector-file note, in P3's own
commit**, not by widening a loop past what the column means.

### F-6 (informational) — `bitcoin-address` needed an implementation, and it is a SHAPE test

§6's address row was the one row P1 left unimplemented (*"needs an address
decoder this crate does not have"*), and the plan's P2.4 clause is explicit that
the S2-parked set is EMPTY. `cascade::is_bitcoin_address` is narrow and
deliberately checksum-FREE — a mistyped address is still an address and earns
the same sentence:

* a single token whose HRP is `bc`/`tb`/`bcrt` with a bech32-charset body
  (either case, never mixed);
* a single token that base58check-decodes to exactly 21 bytes under
  `0x00`/`0x05`/`0x6f`/`0xc4`.

Nothing in §4's four formats can match either — an extended key's payload is 78
bytes and no constellation record's HRP is one of the three. It is consulted
AHEAD of §6's five-step rule, which would otherwise report step 5's generic
four-forms text and bury the fact. Both spellings measured; the row's test runs
the bech32 one.

Note the reachability: with `--as` OMITTED a bare address does NOT open §5.1's
gate (it fails T1–T4), so it keeps the record refusal — which is invariant 1
working. The row fires when `--as` is present, which declares the input
single-document.

---

## 7. What P3's implementer must know

1. **`descriptor::md1::derivation_twin` is never to be encoded.** It exists so
   §5.4 can print `address 0:` for a wallet md1 cannot carry. Encoding either
   twin mapping IS the silent wallet change §5.3(a)/(a″) refuse. The doc comment
   says so at the function and the table above says why each mapping is an
   equality.
2. **Two vector-file edits are queued for P3, and they re-pin BOTH sha256
   literals**: IMPL-P1's F-2 (`gate/deadbeef-fronts-an-xpub`'s `source` states a
   device precedence that does not hold) and this report's F-5. One commit of
   their own, per P1's own instruction — not smuggled into a fold.
3. **`refusal::Row::ALL` is now gated in three places**, and they must move
   together: the file's `refusal_rows` map, the enum, and
   `tests/descriptor_refusals.rs`'s `fn row_*` set. Adding a §6 row without a
   test reds `the_file_carries_one_named_test_per_section_6_row`.
4. **`DESCRIPTOR_PATH_SHIPPED` is the S2 switch, and three sites read it**:
   `gate::carriage`, `gate::window_remedy` and `as_flag::descriptor_follower`.
   Flipping it turns the last one into the place §5.2's canonical `Descriptor`
   record gets packed; the other two change their remedy text automatically.
5. **The Go toolchain is at
   `/nix/store/33fw5m31lfcnk4ff2f0df7j2bxnh8lgk-go-1.26.3/bin`** and is not on
   the default `PATH`. The P1 reviewer's one stated gap ("no Go toolchain on
   this box") was a PATH fact, not a machine fact.
6. **F-424 still owns the `md-codec` bump.** Conjunct 8 is enforced host-side
   because the published 0.42.0 lacks F-217/F-218's encode validators; when the
   bump lands, the host check becomes belt-and-braces rather than the only
   enforcement, and `md1.rs`'s doc comment should be re-read at that point.

---

## 8. Commits

| sha | subject |
| --- | --- |
| `d711a59` | P2.1/P2.2/P2.3: the `--as` flag, the in-process md1 build, the §5.4 block |
| `de35e30` | spec: §6's multi-class device-clause rule, and the key-identity cause clause (SPEC ONLY) |
| `9889853` | P2.4: §6's texts reconciled to the AMENDED spec, one named test per row |
| `49f49a2` | P2.5: F-421 — the converter refers a descriptor to `sysw pack` |
| `eccbc74` | plan: re-pin the `main.rs` anchor at the P2 gate (PLAN ONLY) |
| `5abffc9` | P2: pin where `--as` sits among the shipped gates |

This report lands in its own commit on top of `5abffc9`, so the six above are
exactly the diff a reviewer reads. The spec amendment and the plan re-pin are
each isolated, so `git show de35e30` and `git show eccbc74` are one command each.

---

## 9. Propagation sweep over my own edits

Every superseded phrase or count I changed, grepped across `crates/` and the
spec, with the expected hit count:

| superseded form | hits | |
| --- | ---: | --- |
| `a duplicated cosigner line carrying the wrong key` | **0** | M5 |
| `_ => Script::P2PKH` | **0** | M2 |
| `e.wrapping_sub(HARDENED)` | **0** | M2 |
| `Use \`--as descriptor\`, which carries it exactly.` | **0** | M4 / §5.3 |
| `The path that carries a fixed chain index exactly` | **0** | §5.3 stock text |
| `The path that carries a multipath with no wildcard exactly` | **0** | §5.3 stock text |
| `every_promotion_remedy_me_prints_is_an_input_me_admits` | **0** | M3 rename |
| `AsDecides { announcement` | **0** | P2.3 |
| `suggested_descriptor_for(&k.origin, k))` (the `&str` call form) | **0** | M2 |
| `#[ignore` in `crates/` | **1** | the pre-existing fixture generator, §2 |

Three phrases survive and are checked to be CORRECT rather than merely present:

* `the use-site path is not a path: \`{p}\`` — **1 hit**, the non-empty arm,
  which N1's new guard now precedes;
* `The device would accept it and derive addresses whose coins cannot be spent.`
  — **2 hits**: the SORTEDMULTI arm of `key_count_exceeded` (true there) and
  §6's own row text (which is the sortedmulti row);
* `is not a valid descriptor even though the device's parser accepts it. Check`
  — **2 hits**: the row test's expectation and §6's row text. The code path is
  the `Multi::Sorted` arm.

And the amended texts appear exactly once each, in the code and in the spec:
`one of the two entries carries the wrong key` → 1 hit in `refusal.rs`, 1 in
`SPEC_descriptor_input.md`.

Nothing pushed. No tags, no releases, no publishes, no on-device actions. The
fork worktree (`/scratch/code/shibboleth/_work/seam-fork`,
`seam/descriptor-vectors` at `1f09537`) is unchanged and its
`git status --porcelain` is empty.
