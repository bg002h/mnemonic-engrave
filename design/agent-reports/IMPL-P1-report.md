# IMPL — P1 of `IMPLEMENTATION_PLAN_descriptor_input_S1_S3.md`

**Executed by:** the single P1 implementer, 2026-08-29.
**Plan:** GREEN at round 5, executed as written; P1.1 + P1.2 exactly.
**Branch:** `impl/descriptor-s1s3`, head `b8004f4` (nothing pushed, no tags, no
publishes). Worktree `/scratch/code/shibboleth/_work/impl-s1s3`; the main
checkout was never touched and stays on `master`.

**Verdict: P1 is COMPLETE and every clause of the P1 gate passes.** The two
P1-tagged host-column tests are un-ignored and green; the four that remain all
name P2. **F-1 is CONFIRMED, in code and by assertion on all 71 rows** — the
vector file is unchanged and both sha256 literals still read
`0393592f234b0a5264eb7f49553ab3b3911085cd2d1cd8052690018c7fe80584`.

Four findings below. Two are structural facts about the vector file that the
mutation table forced into the open (conjuncts 4 and 6 have no row that can red
them); one is a false justification in a row's `source` annotation; one is a
named intra-branch window that P2.1 closes.

---

## 1. What was built

`crates/me-cli/src/lib.rs` gains `#[doc(hidden)] pub mod descriptor` — lib-public
so `tests/descriptor_seam.rs` can call the predicate and the gate directly (the
codex32 precedent's shape), doc-hidden to keep the published surface deliberate.
PLAN-r1's C2, as written.

| file | lines | what it is |
| --- | ---: | --- |
| `descriptor/cascade.rs` | 1242 | §4.1–§4.6: the four branches in the device's order, the parsed types, the canonical re-encoder, §6's five-step cause selection |
| `descriptor/refusal.rs` | 833 | §6's 36-row cause taxonomy and the text each row prints |
| `descriptor/gate.rs` | 552 | §5.1's shape gate (T1–T4), the whole-input discriminator, §5.4's carriage rule, §5.1's choice block |
| `descriptor/cascade_tests.rs` | 383 | the cascade's own unit tests |
| `descriptor/admit.rs` | 349 | §4.7 as eight explicit conjuncts, §5.2's classification predicate, §5.3's representability |
| `descriptor/secp.rs` | 294 | the on-curve check `hdkeychain.NewKeyFromString` makes |
| `descriptor/base58.rs` | 149 | base58check |
| `descriptor/checksum.rs` | 127 | BIP-380's descriptor checksum |
| `descriptor/mod.rs` | 57 | the module doc, including the F-1 reading |

Plus 92 changed lines in `main.rs` and 204 in `tests/descriptor_seam.rs`.
**12 files, +4,279 / −12.** `Cargo.toml` and `Cargo.lock` are byte-untouched:
`git diff e0d3d65..HEAD -- Cargo.toml Cargo.lock crates/me-cli/Cargo.toml` is
empty. **No new dependencies.**

### The clauses of P1.1 and P1.2, each with where it lives

* **the four branches in device order** — `cascade::cascade`, returning on the
  first success exactly as `OutputDescriptor` does.
* **the four BlueWallet refusals** — `parse_bluewallet`: no `Format:`, zero
  cosigner lines (F-419), any cosigner key that would carry an empty origin
  path, and a fingerprint that is not exactly 8 hex characters. All four make
  branch 1 **fail**, which is what F-1's reading requires and what the file's
  `format: "none"` on all five `narrowed-4.2` rows pins.
* **exactly-8-hex fingerprints** — `cascade::is_8_hex`, checked at line-parse
  time because the device PANICS below 4 bytes and this file must never reach
  it. The same function is §5.1's gate test T2, shared so they cannot drift.
* **the five version bytes** — `KeyVersion::admitted`, refused inside
  `parse_extended_key`. All ten SLIP-132 spellings are named, because §6's
  remedy has to be per-version.
* **case-insensitive JSON** — `parse_json` matches exact-then-case-insensitive,
  as Go's `encoding/json` does, and reproduces `json.Unmarshal`'s own
  acceptance shape (objects and `null`; a non-string in either field means the
  branch is not claimed at all).
* **promotion with the key-as-supplied echo** — `gate::promotion_announcement`
  prints `key as supplied:` verbatim and `inferred wallet:` in full. R0's I5,
  and the unit test proves it on a DEPTH-4 key, where the canonical
  re-serialisation is a base58 string the operator has never seen. On the
  depth-3 fixture the two coincide and the test would have proved nothing.
* **whitespace normalisation** — `cascade::normalise`, CRLF→LF then trim, before
  the cascade.
* **the seven shapes + the `multi` twins + conjuncts 2–8** — `admit::admit`,
  with `Path::Descriptor` / `Path::Md1` as conjunct 1's only difference.
* **§6's TWO key-identity rows** — `key_identity` (origin contradiction) and
  `key_identity_duplicate` (duplicated slot), separate texts, separate slugs.
  PLAN-r4's NEW-M4 is folded into the second: it now names the risk the refusal
  exists for (one holder producing two of the required signatures).
* **the discriminator + the shape gate, built FROM the 37 gate rows** —
  `gate::consult`, wired into `main.rs` at the exact moment record
  classification fails.
* **errors carrying §6's cause taxonomy** — `refusal::Row`, 36 variants, and
  `tests/descriptor_seam.rs` asserts the enum's slug set equals the file's
  `refusal_rows` keys.

### One structural change to `main.rs`

`read_records` now returns `Records { records, document }`. `document` is the
whole input; `records` is the shipped newline-separated stream. The reason is
not stylistic: a multi-line BlueWallet file and a pretty-printed JSON export are
ONE descriptor and stop being one the moment they are split into records, and
§4.6's "the whole input" and §5.1's whole-input parse both need the bytes. stdin
is read once and both shapes come from that read (`read_stdin_raw`), which is
the defect `no_records_guard`'s own doc comment records from the other
direction.

The gate is consulted between `admit_check`'s failure and the shipped refusal.
`Outcome::RecordRefusal` falls straight through, unchanged — that fall-through
IS §5.1's invariant 1.

---

## 2. The P1 gate — actual output, pasted

### (a) The two P1 ignores removed, RED first, then green

`ffabdff` removed only the two attributes and captured the red:

```
        FAIL [   0.004s] (1/8) mnemonic-engrave::descriptor_seam the_gate_rows_pin_the_real_invocation
        FAIL [   0.005s] (6/8) mnemonic-engrave::descriptor_seam the_host_column_matches_the_admission_predicate
     Summary [   0.006s] 8 tests run: 6 passed, 2 failed, 4 skipped
```

Green at `b8004f4`:

```
$ cargo nextest run --locked -p mnemonic-engrave --test descriptor_seam
        PASS  mnemonic-engrave::descriptor_seam the_row_schema_holds_on_every_row
        PASS  mnemonic-engrave::descriptor_seam the_file_is_the_one_the_fork_pins
        PASS  mnemonic-engrave::descriptor_seam every_row_pins_the_digest_of_its_own_input
        PASS  mnemonic-engrave::descriptor_seam the_refusal_row_vocabulary_is_the_same_set_on_both_sides
        PASS  mnemonic-engrave::descriptor_seam the_coverage_manifest_is_met_by_count_not_by_reading
        PASS  mnemonic-engrave::descriptor_seam every_column_has_the_expected_population
        PASS  mnemonic-engrave::descriptor_seam the_row_set_is_not_vacuous
        PASS  mnemonic-engrave::descriptor_seam the_encoder_produces_every_canonical_the_file_carries
        PASS  mnemonic-engrave::descriptor_seam the_host_column_matches_the_admission_predicate
        PASS  mnemonic-engrave::descriptor_seam the_gate_rows_pin_the_real_invocation
     Summary [   0.047s] 10 tests run: 10 passed, 4 skipped
```

### (b) The remaining four ignores all name P2

```
$ grep -c '^#.ignore' crates/me-cli/tests/descriptor_seam.rs
4
754:#[ignore = "P2: `--as md1` is not built"]
765:#[ignore = "P2: `--as md1` is not built"]
774:#[ignore = "P2: the in-process md_codec build is not written"]
784:#[ignore = "P2: `--as md1` is not built"]
```

### (c) Full workspace suite

```
$ cargo nextest run --locked
     Summary [  32.165s] 485 tests run: 485 passed, 5 skipped
```

P0 closed at `446 tests run: 446 passed, 7 skipped`. **+39 tests, −2 skips**,
and no pre-existing test was changed.

### (d) Lints

```
$ cargo clippy --all-targets --locked -- -D warnings
    Finished `dev` profile [optimized + debuginfo] target(s) in 0.06s
(exit 0)

$ cargo fmt --check
(exit 0, no output)
```

### (e) The fork's Go seam test, re-run

The vector file was not touched, so its sha256 is unchanged in both repos:

```
$ sha256sum crates/me-cli/testdata/descriptor_seam_vectors.json \
            /scratch/code/shibboleth/_work/seam-fork/nonstandard/testdata/descriptor_seam_vectors.json
0393592f234b0a5264eb7f49553ab3b3911085cd2d1cd8052690018c7fe80584  crates/me-cli/testdata/descriptor_seam_vectors.json
0393592f234b0a5264eb7f49553ab3b3911085cd2d1cd8052690018c7fe80584  .../seam-fork/nonstandard/testdata/descriptor_seam_vectors.json

$ git -C /scratch/code/shibboleth/_work/seam-fork status --porcelain
(empty)

$ go test ./nonstandard/ -v
--- PASS: TestDescriptors (0.00s)
--- PASS: TestDecoder (0.00s)
--- PASS: TestElectrumSeed (0.00s)
--- PASS: TestDescriptorSeamDeviceColumn (0.00s)
--- PASS: TestDescriptorSeamInvariant (0.00s)
--- PASS: TestDescriptorSeamAddresses (0.01s)
--- PASS: TestDescriptorSeamWalletID (0.00s)
--- SKIP: TestDescriptorSeamSyswClass (0.00s)
PASS
ok  	seedhammer.com/nonstandard

$ go vet ./nonstandard/    -> clean
$ gofmt -l nonstandard/    -> clean
```

### (f) Staleness re-check, all three repos

```
$ ./scripts/plan-staleness-check.sh design/IMPLEMENTATION_PLAN_descriptor_input_S1_S3.md . e0d3d65
═══ design/IMPLEMENTATION_PLAN_descriptor_input_S1_S3.md
─── against . at e0d3d65 .. b8004f4
─── unchanged: 4 ; DRIFTED: 0 ; not in this repo: 4

$ ./scripts/plan-staleness-check.sh <plan> /scratch/code/shibboleth/_work/seam-fork d402f18
─── against /scratch/code/shibboleth/_work/seam-fork at d402f18 .. 1f09537
─── unchanged: 3 ; DRIFTED: 0 ; not in this repo: 5

$ ./scripts/plan-staleness-check.sh <plan> /scratch/code/shibboleth/descriptor-mnemonic 6864f377
─── against /scratch/code/shibboleth/descriptor-mnemonic at 6864f377 .. 6864f377
─── unchanged: 1 ; DRIFTED: 0 ; not in this repo: 7
```

**0 DRIFTED in all three.** P1's `main.rs` edits are all below line 1300, so the
`main.rs:335` anchor did not move; `sysw/mod.rs:205`, `codex32_seam.rs:60` and
`sysw_cli.rs:1928` were not touched at all.

---

## 3. Mutation testing — the predicate's assertions can actually fail

Every conjunct removed or inverted in turn; the two P1 tests re-run; the row
that reds it named. **19 mutations, 17 caught.** The two that are not caught are
finding F-1 below, and they were re-run against the WHOLE 484-test workspace
suite to be sure the claim is "no test anywhere", not "no test in this file".

| mutation | verdict | first row that reds |
| --- | :-: | --- |
| conjunct 1 (shape) → always Ok | CAUGHT | `narrowed/tr-sortedmulti` |
| conjunct 2 (threshold) → always Ok | CAUGHT | `narrowed/threshold-zero` |
| conjunct 3 (key count) → always Ok | CAUGHT | `narrowed/sh-sortedmulti-16-keys` |
| **conjunct 4 (versions) → always Ok** | **NOT CAUGHT** | — (see F-1) |
| conjunct 5 (network) → always Ok | CAUGHT | `narrowed/mixed-network` |
| **conjunct 6 (origins) → always Ok** | **NOT CAUGHT** | — (see F-1) |
| conjunct 7 (use-site) → always Ok | CAUGHT | `narrowed/use-site-hardened` |
| conjunct 8 (key identity) → always Ok | CAUGHT | `gate/colliding-origin-sortedmulti` |
| conjunct 8(a) alone disabled | CAUGHT | `gate/colliding-origin-sortedmulti` |
| conjunct 8(b) alone disabled | CAUGHT | `gate/duplicate-key-same-use-site` |
| conjunct 7 admits a hardened use-site | CAUGHT | `narrowed/use-site-hardened` |
| conjunct 7 admits a non-consecutive pair | CAUGHT | `narrowed/use-site-non-consecutive` |
| conjunct 3 uses 20 under a DIRECT `sh` | CAUGHT | `narrowed/sh-sortedmulti-16-keys` |
| conjunct 3 uses 15 under `sh(wsh)` | CAUGHT | `accepted/sh-wsh-sortedmulti-16-keys` |
| UPSTREAM: the cascade admits any key version | CAUGHT | `neither/full-origin-ypub` |
| UPSTREAM: branch 1 admits origin-less keys | CAUGHT | `bluewallet/derivation-after-keys` |
| UPSTREAM: `me` promotes a bare `tpub` | CAUGHT | `promotion/15-bare-tpub-host-refused` |
| GATE: the shape gate always opens | CAUGHT | `gate/record-text-parentheses` |
| GATE: the shape gate never opens | CAUGHT | `promotion/01-bare-xpub` |

The last four rows are the ones worth keeping: the two `sh` bound mutations
catch the predicate in BOTH directions (too strict reds the accepted 16-key
`sh(wsh(…))`, too loose reds the refused 16-key `sh(…)`), and the two gate
mutations show that §5.1's two invariants are each pinned by a row — an
always-open gate reds invariant 1's record row, an always-closed one reds
invariant 2's promotion row.

The script is at
`/tmp/…/scratchpad/mutate.py` and reverts with `git checkout` after each
mutation; the tree was verified clean afterwards.

---

## 4. Findings

### F-1 (Important, on the VECTOR FILE not the code) — conjuncts 4 and 6 have no row that can red them, and the reason is that the cascade enforces both first

Delete `conjunct_4_versions` and `conjunct_6_origins` entirely and the **whole
workspace suite stays green** — measured separately for each, `484 tests run:
484 passed, 5 skipped` both times, at the tree state before the canonical test
landed. Neither is
dead code in the ordinary sense — both are the NORMATIVE predicate §4.7 states
— but neither is *gated*.

The cause is structural and follows from F-1's own confirmed reading:

* **Conjunct 4** is enforced first inside `parse_extended_key`, because a
  non-admitted version has to make the CASCADE fail: that is what makes
  `neither/full-origin-ypub` carry `format: "none"` rather than
  `format: "bip380"`. Mutating the cascade's gate instead **does** red that row.
* **Conjunct 6** is enforced first inside branch 1, because §4.2 states it as a
  thing `me` refuses about a BlueWallet FILE: that is what makes all five
  `narrowed-4.2` rows carry `format: "none"`. Mutating branch 1's check instead
  **does** red `bluewallet/derivation-after-keys`.

So the vector file gates the behaviour at its real enforcement point, and the
restatement in `admit.rs` is ungated. **No row was invented to fix this**, per
the brief. The honest options for P2 or P3, in order of cost:

1. Leave it, with `admit.rs`'s module doc stating the fact (done — the table at
   the top of `admit.rs` names both, and says which vector row reds the upstream
   site). This is defensible only while every caller of `admit()` receives
   descriptors from `cascade()`.
2. **The reason it will not stay defensible: P2.2 builds `md_codec` descriptors
   in process.** If any future caller constructs a `Parsed` by another route,
   these two conjuncts become the only enforcement and are untested. A unit test
   in `admit.rs` constructing a `Parsed` directly — not a new vector row — is
   the cheap close, and it belongs with P2.2.

### F-2 (Minor, on the VECTOR FILE) — `gate/deadbeef-fronts-an-xpub`'s `source` states a device precedence that does not hold; the row's pinned outcome is right anyway

The row's `source` reads: *"refusal_row is the device's own precedence:
parseBlueWalletDescriptor succeeds and OutputDescriptor's Title != "" gate is
what refuses"*. **Measured against `parseBlueWalletDescriptor` at fork
`1f09537`, it does not succeed:**

```
deadbeef-only    ERR  bluewallet: expected 0 keys, but got 1
name+deadbeef    ERR  bluewallet: expected 0 keys, but got 1
policy+deadbeef  OK   title="" keys=1 thr=1 script=Unknown
```

The file carries no `Policy:` header, so `nkeys` is 0 while one key was
appended, and the count check at `nonstandard/parse.go:151` fires before the
`Title` gate at `:37` is ever consulted. (The row's `device_admits: false` is
correct regardless — `OutputDescriptor` refuses it either way.)

**The pinned `refusal_row: bluewallet-no-name` is nonetheless the right answer,
and P1 implements it**, ordering `me`'s `Name:` gate ahead of its key-count
check. The justification is not the device's precedence but §6's own standard:
the count row's text is *"`Policy: 2 of 3` declares 3 cosigners; the file has
2"*, and for a file with no `Policy:` line at all that sentence is FALSE about
the operator's file. §7's rule ("where any reading of the guidance disagrees
with a gate row, the row is the answer") settles it, and the truth of the two
candidate texts agrees with the row.

**Action for P3:** correct the row's `source` string. That changes the file, so
it re-pins both sha256 literals — cheap, but it is a file edit and belongs in a
commit of its own, not smuggled into a fold.

**Second-order note for P2.4:** §6's no-`Name:` text also enumerates *"it has
`Policy`, `Derivation` and `Format` headers and `N` cosigner lines"*, which is
false for this one-line file. P1 substitutes the enumeration from what the file
actually contains — it prints *"this is a BlueWallet setup file -- it has 1
cosigner line -- but no `Name:` header…"*. **P2.4's "verbatim" test for this row
must assert the SUBSTITUTED form**, exactly as §5.3's two window-substituted
rows already do.

### F-3 (Minor, self-inflicted and named) — `MD1_PATH_SHIPPED` is true from P1 while the `--as` flag lands in P2.1

`gate::MD1_PATH_SHIPPED = true` on this commit, and `me` does not yet accept
`--as md1` — so the choice block offers a flag the binary would reject. This is
a real intra-branch window, and it is deliberate: §5.4's carriage rule decides
whether the choice block fires at all, and computing carriage from the CURRENT
tree makes all seven `as-decides` gate rows unsatisfiable in P1, which the
plan's own P1 gate forbids. The constant is documented at its definition with
this reasoning and a pointer here.

**P2.1 closes it by implementing the flag.** Nothing else needs to change.

### F-4 (Minor, RESOLVED in-code) — three defects found by READING the emitted refusals, none of which any assertion caught

The suite went green before these existed. They were found by running the binary
over the vector inputs and reading the output as an operator would:

1. `key @0 ([dc567276/48h/0h/0h/2h][dc567276/48h/…an/0/*)` — the origin printed
   **twice**, because the elision helper prepended a rebuilt origin to
   `as_supplied`, which already carries one.
2. The "supply the descriptor" remedies printed an **elided** key —
   `tr([4bbaa801/86h/0h/0h]xpub6C9j4wAxxk…acoGnx/<0;1>/*)`. That is a
   placeholder wearing the operator's fingerprint, and
   `SPEC_constellation_cli_uniformity` §6h forbids exactly it.
3. The `Vpub` remedy had an unbalanced paren.

All three are fixed in `24c64de`, and (2) is now gated:
`every_promotion_remedy_me_prints_is_an_input_me_admits` runs the generator's
output back through the cascade — 3 executable remedies, all admitted, with a
counted floor so the loop cannot pass by skipping. The `<the other cosigners>`
forms are exempt and the exemption is narrow: a multisig cosigner key has no
single-key wallet, and the other cosigners are information the operator holds
and `me` does not.

Verified by hand as well, with the emitted bytes fed straight back:

```
$ me sysw pack --in p86.txt   →  remedy: tr([4bbaa801/86h/0h/0h]xpub…/<0;1>/*)
$ me sysw pack --in remedy.txt → "this input is a wallet descriptor, and `--as` decides…"  (exit 2)
```

### F-5 (informational) — the on-curve check, and why it is in P1 at all

`bip380.ParseKey` → `hdkeychain.NewKeyFromString` does not stop at the
base58check trailer: for a public key it runs `btcec.ParsePubKey`, which
decompresses the point and fails if `x³ + 7` has no square root
(`btcutil/v2@v2.0.0/hdkeychain/extendedkey.go:724–731`, read at implementation
time). A host that checked only the trailer and the `0x02`/`0x03` prefix would
therefore **ADMIT an extended key the device REFUSES** — §7's one forbidden
direction, with no vector row covering it.

`descriptor/secp.rs` implements that one predicate and nothing else (no point
arithmetic, no signatures), in 294 lines with no new dependency. **Cross-checked
against `bip380.ParseExtendedKey` at fork `1f09537` on five hand-built 78-byte
envelopes, agreeing 5 of 5:**

```
off-curve x=5          REFUSE hdkey: invalid extended key
off-curve x=7          REFUSE hdkey: invalid extended key
on-curve x=1           ACCEPT
on-curve G             ACCEPT
x=p (out of field)     REFUSE hdkey: invalid extended key
```

The five keys are pinned in `cascade_tests.rs` with those verdicts.

Worth recording because the first draft of the unit test asserted `x = 1` was
OFF the curve, reasoning that `1³ + 7 = 8` is a non-residue. It is not:
`p ≡ 7 (mod 8)`, so 2 is a residue and 8 with it. **The test failed and the code
was right** — the negative cases are now taken from an independent computation
rather than from an argument.

`me` is additionally NARROWER than the device on one corner: `NewKeyFromString`
accepts a `0x00`-prefixed PRIVATE key body under `xpub` version bytes and
derives its public key. `me` refuses any prefix but `0x02`/`0x03`. The host may
be narrower; a private key arriving on a public channel is not a thing to
quietly accept.

### F-6 (informational) — F-413's default is implemented spec-as-written

`ypub`/`upub`/`vpub`/`Upub`/`Vpub` are refused, with the per-version executable
remedy R0 r2's NEW-I3 requires (one template cannot serve five; four of the five
are testnet). Measured emission:

```
me: the device admits exactly `xpub`, `tpub`, `zpub`, `Ypub`, `Zpub`. This key is
    `ypub`, whose equivalent is `xpub`: sh(wpkh([4bbaa801/49h/0h/0h]xpub6C9j4…/<0;1>/*))
```

and that remedy runs back through `me` clean (exit 2, the choice block).

**If the P1.0 consult rules for host-side normalisation instead**, the change is
small and localised: `parse_extended_key`'s `if !v.admitted()` arm rewrites the
version bytes rather than erroring, and `refusal::unsupported_key_version` loses
its callers. Nothing else moves — and note the vector file would move with it
(`neither/full-origin-ypub` is `host_admits=false`, `format: "none"`), so the
ruling costs a re-pin of both sha256 literals whichever way it goes.

---

## 5. Two clauses of P1.2 that P2 owns, stated so they are not assumed done

* **`--as`'s single-document input contract.** P1 implements the whole-input
  READ (`Records::document`) and the whole-input discriminator, which is the
  half the `--as`-omitted gate rows exercise. The `--as`-PRESENT half —
  *"Supplying `--as` with more than one argv operand, or with both argv and
  `--in`, is `EXIT_USAGE` (2)"* — needs the flag, and the flag is **P2.1** by
  the plan's own split. It is not implemented here.
* **§4.2's zero-fingerprint WARNING.** *"`me` warns on the `--as descriptor`
  path, once per affected key, whenever an origin path the INPUT SUPPLIED is
  dropped"* is scoped by R0 r4's NEW-M3 to `--as descriptor` alone, which is S2.
  Not implemented, correctly.

---

## 6. What P2's implementer must know

1. **`descriptor::admit::admit(&Parsed, Path::Md1)` is the predicate P2.2 must
   run BEFORE encoding.** Conjunct 8 in particular: the published `md-codec`
   0.42 crate `me` links lacks F-217/F-218's validators, so the impossible-wallet
   gap must never reach the codec. `carriage()` in `gate.rs` already calls it in
   the right order and is the model.
2. **`md1_representable(d, remedy_a, remedy_a2)` takes its two remedy sentences
   as parameters**, which is where §5.3's window substitution lives. P2.1's
   window refusal should build them the same way rather than inventing a second
   substitution site; `gate::remedy_fixed_index` / `remedy_no_wildcard` are the
   two builders, including §6's `multi`-form replacement (a neither-path refusal
   routes nowhere, so "wait for the update" would be false forever).
3. **`Parsed::encode()` is `Descriptor.Encode()`**, and it is now gated:
   `the_encoder_produces_every_canonical_the_file_carries` asserts it against
   **all 19** `canonical` values in the file — measured by P0 through the DEVICE
   route, reproduced by the Rust route first try, checksums included. It is also
   a fixed point on the fork's own `#hfwurrvt` fixture. P2.2 can build the md1
   descriptor from the same `Parsed` rather than re-parsing.
4. **`refusal::Row` is the full 36-slug vocabulary, and P1 built texts for 32 of
   them** — 31 named constructors in `refusal.rs` plus `WindowNotInBuild`,
   constructed inline in `gate::carriage`. The **four with no text** are all
   P2's or already covered elsewhere: `EmptyFile` and `WhitespaceOnly` are the
   shipped `no records in <file>` path at exit 2 and were never `me`'s to write;
   `AsOmitted` is the choice block, which `gate::choice_block` emits as a
   `String` rather than through a `Refusal`; `BitcoinAddress` needs an address
   decoder this crate does not have. `MultiUnderDescriptor` has text but is
   unprintable until `--as descriptor` exists. **P2.4's 36 named tests
   key to these slugs**, and `the_refusal_row_vocabulary_is_the_same_set_on_both_sides`
   already fails if either side drifts.
5. **§6's "both rows fire" case is unimplemented and deliberately so.** A
   descriptor mixing an (a)-shaped and an (a″)-shaped key matches BOTH §5.3
   rows; `md1_representable` returns the FIRST offender, because a gate row
   names exactly one `refusal_row`. Emitting both belongs with P2.4's per-row
   texts. `admit.rs`'s doc comment says so at the function.
6. **The `is_miniscript_fragment` list is a judgement call with no vector row
   behind it.** `neither/miniscript` is not `gate`-tagged, so its `refusal_row`
   is unpinned; P1 routes an unknown inner script name that is a known
   miniscript fragment to §6's miniscript row and everything else to the
   four-forms row. If P2.4 wants that pinned, it needs a row, not a longer list.
7. **The gate test's four fields come from the real run, and the link is the
   refusal TEXT.** `the_gate_rows_pin_the_real_invocation` reads exit code and
   outcome class off the process, then asserts stderr CONTAINS the text of the
   `Refusal` the library selects for that input. A binary that took a different
   branch prints different text and reds. Keep that shape when P2 extends it —
   a marker string would be cheaper and would stop proving anything.
8. **`is_record_refusal` matches `(records count from 0)`, not one message.**
   The first version matched one arm's tail and reported `unclassified` for five
   of the six hostile-payload rows: §5.1's invariant 1 is about the record
   SURFACE, not about a single sentence.

---

## 7. Commits

| sha | subject |
| --- | --- |
| `ffabdff` | P1: un-ignore the two P1 host-column tests — RED before green |
| `5b5bc46` | P1.1/P1.2: the cascade, the admission predicate and the §5.1 gate |
| `24c64de` | P1: read the emitted refusals, and fix what reading them found |
| `b8004f4` | P1: assert the encoder against the file's own 19 canonical strings |

This report lands in its own commit on top of `b8004f4`, so the four work
commits above are exactly the diff a reviewer reads.

Nothing pushed. No tags, no releases, no publishes, no on-device actions — the
overnight boundaries hold. The fork worktree
(`/scratch/code/shibboleth/_work/seam-fork`, `seam/descriptor-vectors` at
`1f09537`) is unchanged and its `git status --porcelain` is empty.
