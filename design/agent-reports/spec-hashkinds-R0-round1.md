# R0 ARCHITECT REVIEW — SPEC_hashlock_kinds.md, round 1

Artifact: `design/SPEC_hashlock_kinds.md` at mnemonic-engrave `780254a0`.
Trees measured: descriptor-mnemonic `40c400de`, mnemonic-engrave `780254a0`,
mnemonic-secret `7a0e96f`, seedhammer `0562e811` — the same commits the four
recon reports and the design review were written against, so nothing below is
drift.

Every number, line, width and dependency edge below was **run**. One untracked
probe test (`gui/zz_probe_r0_test.go`) was created, measured, and deleted;
`git status --porcelain` is clean in all four repos, verified after deletion.
No tracked file was modified other than this report.

The question answered: **can a competent implementer follow this spec without
making a decision the spec should have made, and without building something
wrong?**

---

## 1. Citation table — every citation checked

The brief's warning is borne out in the opposite direction from last time: the
spec's citations are unusually good. 27 of 32 are exactly right. The defects are
concentrated not in the file:line citations but in **two structural claims**
(§9's phase independence, §10's test ordering) and in **three findings reported
as folded that are folded incompletely**.

| # | § | Citation | Verdict | Measured |
|---|---|---|---|---|
| 1 | §1 | `md-codec` `compose/mod.rs:151` = a bare 32-byte digest | **TRUE** | `pub hash: Option<[u8; 32]>,` |
| 2 | §1 | fork `md/compose.go:167` | **TRUE** | `Hash *[32]byte` |
| 3 | §1 | `compose/lowering.rs:78` always `Tag::Sha256` | **TRUE** | `tag: Tag::Sha256,` |
| 4 | §1 | fork `md/compose.go:403` | **TRUE** | `parts = append(parts, node{tag: tagSha256, body: hash256Body(*h)})` |
| 5 | §1 | `me-cli/sysw/composer_records.rs` = `hash:` + exactly 64 hex | **TRUE** | `composer_records.rs:303` `if body.len() != 64` |
| 6 | §1 | fork `gui/composer_hash.go` = 64 hex only | **TRUE** | `:87` `> 64`, `:91` `== 64`, `:97` `!= 32` |
| 7 | §1 | fork `hashlock/hashlock.go:85` | **TRUE** (block start) | `:85` doc comment, `:86` `func Digest(x *[32]byte) [32]byte` |
| 8 | §1 | `ms-codec/hashlock.rs:59` | **TRUE** | `pub fn digest(preimage: &[u8; 32]) -> [u8; 32] {` |
| 9 | §1 | "Zero tests and zero vectors compose a non-sha256 hashlock, in any language" | **TRUE** | Rust hits for `ripemd160\|hash160` are `tests/common/mod.rs`, `proptest_to_miniscript.rs`, `render_template_snapshot.rs` — all at the `Node`/`Tag` layer; `crates/md-codec/tests/vectors/` has zero hits; Go has one hit, `md/testdata_test.go:162-163`, a tag-name map; `md/compose.go` has zero non-sha256 tags |
| 10 | §3 F2 | Core 25.0.0 accepts all four under `wsh`/`sh(wsh)`; bare `sh`/`tr` refuse identically | **TRUE** | core-verdicts report lines 91-106, `deriveaddresses` addresses recorded for all 8 ACCEPTED cells |
| 11 | §3 F3 | every malformed digest returns `A function is needed within P2WSH` | **TRUE** | core-verdicts lines 127-133, 7 rows |
| 12 | §3 F5 | `golang.org/x/crypto/ripemd160` already linked into the firmware | **TRUE** | `btcd/address/v2@v2.0.0/hash160.go:11` imports it, `:22` calls `ripemd160.New()`; called by `bip32/bip32.go:39` and `address/address.go:139,142`; `go.mod:14` `golang.org/x/crypto v0.52.0` |
| 13 | §3 F5 | `me-cli` already carries `bitcoin = "0.32"` | **TRUE** | `crates/me-cli/Cargo.toml:38` |
| 14 | §5 | **Eleven** map/set/equality sites on a bare `[32]byte` | **TRUE** | 3 maps (`composer_state.go:53`, `:81`, `composer_preimage_plate.go:78`) + 7 Go equality (`composer_hashlock.go:176,218,240`, `composer_hash.go:370,375`, `composer_selfcheck.go:138`, `composer_preimage_plate.go:152`) + 1 Rust (`me-cli/src/main.rs:2598`) = 11 |
| 15 | §5 | "retires four separate copies of exactly-64-hex" | **TRUE** | `md-cli/src/cmd/compose.rs:135`, `me-cli/.../composer_records.rs:303`, `sysw/composer_records.go:194`, `gui/composer_hash.go:87/91` |
| 16 | §5 | `me-cli` depends on `md-codec` through crates.io | **TRUE** | `crates/me-cli/Cargo.toml:26` `md-codec = "0.42"`; `Cargo.lock` `source = "registry+…crates.io-index"`; no `[patch]`, no path override, no `vendor/` dir |
| 17 | §5 | Go: `md` and `sysw` are one module and `md` does not import `sysw` | **TRUE** | re-confirmed, empty grep |
| 18 | §7.1 | "three hand-maintained gates wired to `Which hash?`'s row counts" | **TRUE** | `composer_hash_test.go:95` (`n+3`), `:177` (`3*n+3`), `composer_hashlock_test.go:1441` (`!= 6`) |
| 19 | §7.1 | "two of the six entry routes cannot reach the kind screen" | **FALSE** | four of six cannot (routes 1, 4, 5, 6). Route 6, the preset archetype, is unnamed anywhere in the spec — see **I-5** |
| 20 | §7.3 | `ComposerPathHashes() []*[32]byte` is the emulator's only read access | **TRUE** | `gui/composer_state_hook.go:81`; sole consumer `cmd/emu/composer_js.go:40` |
| 21 | §7.4 | `PolicyShape.Sha256Digests` appends only for `tagSha256`, sets `Hashlock` for all four | **TRUE** | `md/policy_shape.go:284-291` |
| 22 | §7.4 | `composer_consent.go:96` = the UNSORTED predicate | **FALSE** | the predicate is at **`:97`**; `:96` is the closing brace of the display loop. (Inherited from the design review, which also said `:96`.) |
| 23 | §7.4 | `composer_consent.go:198` = the §8i restatement predicate | **TRUE** (block start) | `:198` is `for _, b := range shape.Branches {`; the predicate is `:199` `if len(b.Sha256Digests) > 0` |
| 24 | §7.4 | "**Both** dependent predicates" | **FALSE** | three production consumers of the field, not two: `composer_consent.go:94` (display loop), `:97`, `:199` — plus `composer_selfcheck.go:134,136,138`. See **I-6** |
| 25 | §7.5 | the band is **411 px** | **TRUE** | measured by probe at `sh2DisplaySize` (480×320) through `composerTextBand` → `BAND WIDTH = 411 px` |
| 26 | §7.5 | `rmd160` clears it with **2 px** to spare | **TRUE** | 409 px, 1 line |
| 27 | §7.5 | "only a **≤6-character** token fits" | **FALSE** | character count is not the constraint. Measured, all at 6 chars in the worst row: `rmd160` 409 px / 1 line, `sha256` 409 / 1, `hash25` 409 / 1, but **`wwwwww` 351 px / 2 lines** and **`mmmmmm` 369 px / 2 lines**. Proportional font. See **M-2** |
| 28 | §9 | 39 production references to `SpendPath.Hash` | **TRUE** | 39, re-derived over the recon's 12 named production files with its stated exclusions |
| 29 | §9 | 76 test references | **TRUE** | 76, re-derived over the recon's 13 named test files |
| 30 | §9 | **"Phases 1-3 are mutually independent"** | **FALSE** | `me-cli` computes every hashlock digest through `ms_codec::hashlock::digest` (`main.rs:2636`, `:2641`), and `ms-codec = "0.9"` (`Cargo.toml:53`) resolves from crates.io with a lockfile checksum. See **I-1** |
| 31 | §11 | `gui/composer_copy_test.go:371` = `if declared != 78` | **TRUE** | exact |
| 32 | §11 | `gui/composer_hash.go:348` = the literal `"Type 64 hex"` | **TRUE** | exact |
| 33 | §11 | `gui/composer_hash_test.go:239` = `TestWhichHashRowsDrawOnOneLine` | **TRUE** | exact (func decl) |
| 34 | §11 | `gui/composer_hash_test.go:257` = the literal in the test | **TRUE** | exact |
| 35 | §11 | `gui/composer_hashlock_test.go:1433` = `TestWhichHashPageHoldsFiveRows` | **TRUE** | exact (func decl) |
| 36 | §11 | `gui/composer_hash.go:79-105` holds `composerHexEntry`'s three 64/32 constants | **TRUE** | func at `:79`; constants at `:87`, `:91`, `:97` |
| 37 | §11 | `composerPickScreenMaxRows = 24` (and is correctly NOT on the list) | **TRUE** | `gui/composer_paged.go:243` |
| 38 | §11 | one `composerHexEntry` constant is documented as unreachable | **TRUE** | `gui/composer_hash.go:98-102`, "The pad offers hex alone, so this is unreachable" |
| 39 | §11 | the pin generator prints **156 against 161** | **TRUE**, wrong phase | `scripts/vendor-compose-vectors.sh:29` and `md/testdata/compose_vectors.provenance.json:6` say 156; `md/compose_vectors_pin_test.go:91` asserts 161 and `len(files) == 161`. Both files are in the **fork**, not descriptor-mnemonic. See **M-3** |
| 40 | §10 | the six hardcoded `[56:]`/`[56..]` sites | **TRUE** | `composer_hash.go:50,173,188`, `composer_consent.go:63`, `me-cli/main.rs:2277,2602` |
| 41 | §12.1 | Core's measured addresses exist to compare against | **TRUE** | core-verdicts lines 91-106 |
| 42 | §6 | "(§4.6)" whitespace seam rows | **UNRESOLVABLE** | no §4.6 in this spec; two candidates in the same directory — `SPEC_hashlock_H2_device.md:341` §4.6 = *the Back contract*, `SPEC_descriptor_input.md` §4.6 = whitespace/CRLF refusals. See **M-6** |

---

## 2. Fold check — the design review's 20 findings against the spec text

Checked against the spec's words, not the commit message.

| finding | folded? | where |
|---|---|---|
| C-1 `md-cli` unnamed | **PARTIAL** | §1 row 3, §9 phase 1, §9 last para. `--path` only; `--preset`, `presets::hashlock_gated`'s public signature and `--json`'s field are dropped → **I-4** |
| C-2 phrase arm / kind-aware digest | YES | §2 d5, §4 "Out" (stated twice, as the review asked), §9 phases 3+4. But it buys a release dependency §9 denies → **I-1**, and the verification half is missing → **C-1** |
| I-1 row does not fit | YES | §7.5, with the measurement and a bounded delegation (the ≤6-char rule is overstated → M-2) |
| I-2 eleven not five | YES | §5, verified eleven |
| I-3 comparisons must be kind-aware | YES | §7.6 |
| I-4 §8i fires before the kind | YES | §7.2, split in two exactly as proposed |
| I-5 `HashKind ↔ Tag` not shareable | YES, then re-opened | §5 "One local definition per crate" — and then puts a *function* across the same crates.io boundary → **I-1** |
| I-6 C and D not separable | YES | §9 phase 4 is one phase, with the reviewed-to-the-stricter-standard sentence |
| I-7 no device acceptance; `[]*[32]byte` | **PARTIAL** | §7.3 + §12.2 name the hook; the two layers above it that actually form the walk's assertion are unnamed and §12.2 does not require a kind assertion → **I-3** |
| I-8 is `hash:sha256:<64hex>` accepted? | YES (decided), vectors PARTIAL | §6 "accepted on input and never emitted". The both-spelling vector rows I-8 asked for are absent from §10 → M-5 |
| I-9 `PolicyShape` is decode-side | YES | §7.4, declared as a decode-side change and both consequences named. Third consumer missing → **I-6**; `:96` off by one → M-1 |
| I-10 gates omitted | YES | §11, all six device citations verified TRUE |
| M-1 routes 5 and 6 | **PARTIAL** | §7.1 folds route 5, substitutes route 4, **drops route 6** → **I-5** |
| M-2 keyless shape unmeasured | YES | §10 bullet 2, with "must be measured before this closes" |
| M-3 fifth Go elision site | **NO** | `hashlockFirst8Last8` (`gui/composer_hashlock.go:247`, takes `h [32]byte`) appears nowhere → M-4 |
| M-4 kind-token case | **PARTIAL** | §6 says tokens are lowercase; never says uppercase is refused rather than folded → M-7 |
| M-5 five sha256-hardcoded me-cli strings | **NO** | none of `main.rs:2275`, `:2685`, `:3196`, `composer_records.rs:144`, §8n appears → M-4 |
| M-6 right-kind-wrong-length vectors | **NO** | §10 has no such row → M-5 |
| M-7 ripemd160 deprecated | YES | §13 bullet 3 |
| N-1 §8i out-of-scope contradiction | YES | §7.2; §4's "Out" no longer lists §8i |

---

# CRITICAL

## C-1 — Nothing in §10 or §12 pins `preimage → digest` per kind, so the exact funds-loss path §4 names is caught by no gate

**Spec text at fault.** §4, "Out": *"An implementation that picks a kind and then
computes `sha256(preimage)` anyway produces the first 20 bytes of a SHA-256 hash
lowered under `Tag::Ripemd160`: a permanently unspendable path that no type check
catches and every screen reports as held."* §10's five vector bullets. §12's five
acceptance items.

**The gap.** The spec identifies the failure, scopes the work (§9 phase 3, "one
digest function per kind"), and then lists no vector and no acceptance item that
can detect a wrong digest function. Walking §12 item by item, measured:

- **§12.1** — *"Each kind composes end to end and its address matches Core's
  measured value."* Core derives the address from the descriptor **text**, which
  already contains whatever digest we computed: the recon's own method is
  `bitcoin-cli deriveaddresses <descriptor-with-checksum>`
  (`spec-hashkinds-core-verdicts.md:87,184-194`). An address vector is
  structurally blind to a wrong `preimage → digest` function. It pins the
  *lowering*, not the derivation.
- **§12.5** — *"`ms hashlock` reproduces a non-sha256 plate's digest."* Under §9
  phase 3, `ms hashlock` and the digest function are the **same crate**, so a
  wrong function agrees with itself. This is a self-consistency check, not a KAT.
- **§10 bullet 1** — the round trip starts from a digest (*"compose → md1 →
  decode → script → address"*). The preimage never enters it.
- **§10 bullet 5** — *"Cross-repo token agreement"* pins **tokens**, not digest
  values. Verified: `record_class_vectors.provenance.json` pins record strings
  and classes (68 rows); `compose_vectors.provenance.json` pins compose→bytes
  (161 files / 33 vectors). Neither carries a preimage.
- §5 claims *"What crosses a boundary is the wire token and the digest value,
  pinned by vectors"* — the digest **value** is pinned by no listed vector.

**The concrete failure, and why the 32-byte pair is worse than the one §4 names.**
§4's worked example is `ripemd160`, where the wrong value is a *truncation* — at
least the width changes. The other same-width pair has no structural signal at
all. `hash256` is `sha256d`, i.e. `sha256(sha256(X))`. An implementer who writes

    fn digest_hash256(x: &[u8; 32]) -> [u8; 32] { sha256(x) }   // one word short

produces a 32-byte value that:

- type-checks into `HashLock { kind: Hash256, digest: [u8; 32] }` exactly, with
  no padding, so §5's *"the zero padding on a 20-byte kind is unobservable except
  through `ComposerPathHashes`"* reasoning never fires;
- passes `digest_len() * 2` = 64 hex everywhere (§5);
- lowers cleanly into `Body::Hash256Body([u8;32])` / `hash256Body [32]byte`;
- produces an address Core agrees with (§12.1 green);
- is reproduced identically by `ms hashlock` (§12.5 green);
- round-trips through compose→decode→script→address (§10 bullet 1 green);
- and locks the path to a preimage that does not exist.

Every §12 item passes. `F4`'s own sentence — *"for one digest `h`, a preimage
satisfying one will not satisfy the other"* — is stated about ripemd160/hash160
and is equally true of sha256/hash256, which the spec never says.

**How I verified the gate is absent, and that the mechanism to close it already
exists.** The constellation's one cross-language digest KAT is
`seedhammer/hashlock/testdata/hashlock-v0.8.json`, vendored from
`mnemonic-secret/crates/ms-codec/tests/vectors/hashlock-v0.8.json`. Its
provenance `_comment` states: *"The rows are MEASURED constants (python3 hashlib
+ openssl kdf) … `hashlock/hashlock_test.go` fails if sha256 and the file
disagree or if any derivation row disagrees with the Go port."* Its `kind[0]` row
carries exactly one digest column:

```json
"preimage_hex": "abab…abab",
"digest": "9a2db2e23f1504cd056606553ac049c5e718e8f9ce9233876df1a7a1821af885"
```

That is `sha256(X)`. Three columns are missing, and adding them is the whole fix.

**Close it by** naming, in §10 and in §12, a per-kind digest KAT: one row per
preimage carrying `digest_sha256` / `digest_hash256` / `digest_ripemd160` /
`digest_hash160`, computed **independently of the implementation** (python3
`hashlib`, or `bitcoin::hashes`, or Core), vendored into the fork under the
existing `hashlock-v0.8.json` pin so `hashlock/hashlock_test.go` cross-checks the
Go port against it. Without it, §9 phase 3's "Vectors." is undefined at the one
place where being undefined costs funds.

---

# IMPORTANT

## I-1 — §9's "Phases 1-3 are mutually independent" is false: phase 2 cannot compile without a phase 3 **release**

**Spec text at fault.** §9: *"**Phases 1-3 are mutually independent** — a
consequence of sharing no types across repo boundaries (§5) — and may land in any
order or in parallel."* §5: *"Instead `ms-codec` exposes one digest function per
kind and each caller maps its own local kind onto one."*

**Measured.**

```
crates/me-cli/Cargo.toml:53   ms-codec = "0.9"
Cargo.lock                    name = "ms-codec" / version = "0.9.0"
                              source = "registry+https://github.com/rust-lang/crates.io-index"
                              checksum = "6a7836a85212f27fb1b4a261cd1ac2c2c8a6f0654504d3061e88cff008ebd5a3"
```

No `[patch]` section, no path override in `Cargo.toml` or `.cargo/config.toml`,
and **no `vendor/` directory in mnemonic-engrave at all**. `me-cli` computes
hashlock digests in exactly two places, both `ms_codec::hashlock::digest`:

```
crates/me-cli/src/main.rs:2636    Some(ms_codec::hashlock::digest(&x))
crates/me-cli/src/main.rs:2641    Ok((_, ms_codec::Payload::Preimage(x))) => Some(ms_codec::hashlock::digest(&x)),
```

**The contradiction.** §7.6 requires me-cli's §8.2.3 orphan check to compute the
digest under each record's kind — that is phase 2 work. Under §5's rule that
`ms-codec` owns the per-kind digest functions, phase 2's code cannot compile
until `ms-codec` 0.10 is **published to crates.io** and me-cli's version
requirement bumped, with the lockfile/vendor-freshness ritual attached. §5
rejected a shared `HashKind` for precisely that reason — *"a shared `HashKind`
would make the phase order a release dependency with a `cargo vendor` freshness
ritual attached"* — and then put a shared **function** across the same boundary
one sentence later. A function crossing crates.io is the same release dependency
as a type crossing crates.io.

**Why this is not a quibble.** The design review's C-2 offered both options
explicitly: *"naming a kind-aware digest function in each language … **or**
`ms_codec::hashlock::digest` left alone and the three non-sha256 digests computed
in `me-cli` from `bitcoin::hashes`"*. The spec chose neither in a way an
implementer can act on: §2 decision 5 brings mnemonic-secret in for **`ms
hashlock`'s** sake (*"so `ms hashlock` can still reproduce a plate's digest"*),
which is a phase-3-internal reason; §5's sentence reads as me-cli calling into
ms-codec. Under one reading §9 is true and under the other it is false, and a
plan author scheduling 1/2/3 in parallel from §9 stalls on the first build.

§3 F5 already establishes the resolution: `bitcoin = "0.32"` is in me-cli's tree
today (`Cargo.toml:38`), so `bitcoin::hashes::{ripemd160, hash160, sha256d}` are
free. **Close it by** stating which crate computes me-cli's non-sha256 digests,
and if it is `ms-codec`, replacing §9's independence claim with the release edge.

## I-2 — §10 bullet 3 and §12.3 are unsatisfiable as written: a 40-hex digest cannot reach any formatter before the type changes

**Spec text at fault.** §10: *"**A 40-hex digest through every formatter.** The
six hardcoded `[56:]` sites panic rather than corrupt, so this test exists
**before the type changes**."* §12.3: *"A 40-hex digest passes every formatter
without panicking."*

**Measured — every one of the six sites takes a fixed array, not a string:**

```
gui/composer_hash.go:48     func composerHashRow(i int, digest [32]byte) string
gui/composer_hash.go:165    func composerHashInPayloadRow(i int, digest [32]byte) string
gui/composer_hash.go:171    func composerHashPreimageRow(i int, digest [32]byte) string
gui/composer_hash.go:183    func composerHashPhraseRow(i int, d *[32]byte) string
gui/composer_consent.go:61  func composerDigestShort(d [32]byte) string
```

Each hexes its own argument internally (`hex.EncodeToString(digest[:])`), so the
string it slices `[56:]` is **always** 64 characters. The two Rust sites are the
same shape: `main.rs:2277` and `:2602` slice `hx = hex(&digest)` where `digest:
[u8; 32]`.

**The consequence.** There is no way to hand a 40-hex value to any of them before
the parameter type widens. A Go test that tries does not produce a red test — it
fails to compile, which takes the **whole `gui` package** with it and stops all
~1200 other tests in it from running. The spec's stated TDD ordering ("this test
exists before the type changes") therefore cannot be executed, and an implementer
following it literally breaks the build at step one of a phase-4 task the spec
calls "reviewed to the stricter standard".

**Close it by** stating the actual order — widen the parameter to a length-aware
type (or add a `…Hex(string)` seam) first, then the red test, then the arithmetic
— or by dropping the "before the type changes" clause and making §12.3 a
post-change gate.

## I-3 — §7.3 repairs the hook but not the two layers above it, and §12.2 does not require the walk to assert a kind

**Spec text at fault.** §7.3: *"`ComposerPathHashes() []*[32]byte` is the
emulator's only read access … It becomes kind-aware in this cycle."* §12.2: *"An
**emulator walk** composes a non-sha256 hashlock on the device."*

**The chain the walk's assertion actually travels, measured:**

```
gui/composer_state_hook.go:81   func ComposerPathHashes() []*[32]byte      <- the only layer the spec names
cmd/emu/composer_js.go:53       out = append(out, hex.EncodeToString(h[:]))
cmd/emu/composer_js.go:35-37    // FULL 64 HEX, not the first8..last8 the screens draw …
cmd/emu/composer_js.go:15       //  shComposerPathHashes()   [ "<64 hex>" | null, ... ]
cmd/emu/walk_hashlock_phrase.js:333  /** first8..last8 of a 64-hex digest … */
cmd/emu/walk_hashlock_phrase.js:71   //  * after the hold, it is the corpus's FULL 64-hex hardened digest
```

Both layers above the hook hardcode 64 hex — one in a documented JS API contract,
one in the walk's own helper. Neither is named in §7.3 and neither is in §11's
"gates that must move deliberately" table.

**The concrete failure.** An implementer changes the Go hook per §7.3 and stops.
`hex.EncodeToString(h[:])` over a padded `[u8;32]` still returns 64 characters for
a 20-byte kind — 40 real plus 24 zeros — so the JS walk compares an abbreviation
that is **byte-identical** between `sha256(D)` and a 20-byte kind sharing D's
first 20 bytes. That is precisely the failure §7.3 exists to repair (*"a walk
asserting 'path 1 holds digest D' would pass identically … a gate that cannot
fail"*), relocated one layer outward and left there.

**Separately, §12.2's wording does not close it either.** *"An emulator walk
composes a non-sha256 hashlock on the device"* is satisfied by a walk that
composes one and never asserts the kind. Given this repo's own rule that a plan
may not close while one of its gates has never run — and its corollary that a
gate which cannot fail is not a gate — the acceptance item should read *"…and
asserts, through the kind-aware hook, that the composition stores that kind and
that digest"*.

**Close it by** naming `cmd/emu/composer_js.go` and `cmd/emu/walk_hashlock_phrase.js`
in §7.3 and in §11's table, and strengthening §12.2 to require the kind assertion.

## I-4 — C-1 was folded for `--path` only; the `--preset` arm and `presets::hashlock_gated`'s public signature are unnamed, and one of them will not compile

**Spec text at fault.** §9 phase 1: *"`md-cli`: sibling `ripemd160=` /
`hash160=` / `hash256=` options for `md compose --path`."* §9 closing: *"`md
compose`'s option name is a **grammar** decision, not a rename."*

**Measured — the hash option has a second, unnamed home, and the preset grammar
hard-refuses anything but `sha256`:**

```
md-cli/src/cmd/compose.rs:133  /// Shared by `--path ...,sha256=HEX` and `--preset hashlock-gated,sha256=HEX`.
md-cli/src/cmd/compose.rs:369  named_only(&["sha256", "older"])?;
md-cli/src/cmd/compose.rs:373  .get("sha256").ok_or_else(|| … "{ctx} needs sha256=<64 hex>")?
md-cli/src/cmd/compose.rs:376  presets::hashlock_gated(wrapper, sha256, older_blocks)
md-cli/src/cmd/compose.rs:187  sha256: [u8; 32],              // PresetParams variant field
md-cli/src/cmd/compose.rs:444  serde_json::json!({ "sha256": hex32(&sha256), … })   // --json surface
md-codec/src/compose/presets.rs:88-90   pub fn hashlock_gated(… hash: [u8; 32], …)
```

**Two consequences, one of them a compile error.**

1. `presets::hashlock_gated` is a **public `md-codec` API taking a bare
   `[u8; 32]`**. The moment phase 1 changes `SpendPath.hash` to carry a kind,
   this function must construct a `HashLock` — so either its signature changes
   (a public API break the spec never authorises) or it hardcodes sha256 forever.
   The spec gives no rule, so the implementer invents one, and whichever they
   pick is normative and unreviewed.
2. `--preset hashlock-gated,ripemd160=…` stays refused by name
   (`named_only(&["sha256","older"])`), and the `--json` contract keeps emitting a
   field literally called `sha256`. Decision 1 — *"All four kinds become
   authorable from the composer, on Rust and on the device"* — is then still
   partly unreachable on Rust, which is the exact shape the design review called
   C-1 Critical for.

**Close it by** adding the `--preset` arm, `PresetParams`'s field, the `--json`
key and `presets::hashlock_gated`'s signature to §9 phase 1, with the same
grammar ruling §9's closing paragraph already makes for `--path`.

## I-5 — §7.1's "two of the six entry routes" drops the preset archetype, which needs a kind at construction in both languages

**Spec text at fault.** §7.1: *"**Two of the six entry routes cannot reach the
kind screen** (payload-`phrase:` and payload-preimage-plate records); they are
sha256 by their record's kind."*

**Measured.** The device-surface recon's six routes, against §7.1: route 1
(payload `hash:`) carries its own kind and is handled; routes 2 and 3 (typed hex,
typed phrase) reach the screen; routes 4 and 5 are the two §7.1 names. **Route 6,
the preset archetype, is absent from the entire spec** — the string "preset" does
not appear in it.

```
gui/composer_presets.go:54-60  func composerPresetDigest() *[32]byte   // 32 bytes of 0xa8
   doc comment: "It is READ OFF THE VECTOR, never typed from memory: a hashlock
   whose preimage nobody holds is a path that can never be spent"
md-codec/src/compose/presets.rs:88-90  pub fn hashlock_gated(… hash: [u8; 32], …)
```

Both are digest **constructors** for `SpendPath.Hash`, and both are vector-pinned
(`keyed_compose_preset_hashlock_gated.*`, five files in
`compose_vectors.provenance.json`). The `HashLock` change breaks both at compile
time in both languages, and the spec says nothing about which kind they take.

Two further consequences the spec leaves open: under **decision 3**, a 20-byte
preset digest would carry no held material and therefore warn about a
placeholder; and re-pinning the preset vectors is phase-1 and phase-4 work that
§11's table covers only as the generic "both provenance pin files".

Also: §7.1's *"four of six"* arithmetic. The sentence as written tells an
implementer that exactly two routes bypass the kind screen. Four do. That is a
scope statement, and it is short by two.

**Close it by** naming route 6 in §7.1 and giving `composerPresetDigest` /
`presets::hashlock_gated` an explicit kind (sha256, presumably) as a stated
decision rather than an implementer's default.

## I-6 — §7.4 says "Both dependent predicates", but the third consumer is the composer's compose→decode funds gate, and the spec gives it no rule

**Spec text at fault.** §7.4: *"Both dependent predicates re-key off `Hashlock` —
'does this path have a hash of any kind' — not off `len(Sha256Digests)`"*,
followed by two bullets.

**Measured — every production consumer of the field:**

```
gui/composer_consent.go:94   for _, d := range b.Sha256Digests {          <- display, unnamed
gui/composer_consent.go:97   … && len(b.Sha256Digests) == 0 {             <- §7.4 bullet 1 (cited as :96)
gui/composer_consent.go:199  if len(b.Sha256Digests) > 0 {                <- §7.4 bullet 2 (cited as :198)
gui/composer_selfcheck.go:134  if len(b.Sha256Digests) != wantHash {      <- UNNAMED
gui/composer_selfcheck.go:138  if p.Hash != nil && b.Sha256Digests[0] != *p.Hash {   <- UNNAMED
```

`composerSelfCheck` is the composer's **compose → md1 → decode → compare**
round-trip gate. It is the one place a divergence between the kind the composer
chose and the tag the lowering emitted can be caught — and that divergence is
type-invisible for the same-width pairs, because `ripemd160` and `hash160` share
`hash160Body [20]byte` and `sha256`/`hash256` share `hash256Body [32]byte`. A
mis-wired lowering arm produces no compile error and no length error; only a
kind-comparing self-check catches it.

**Two things go wrong as written.** First, an implementer applying §7.4's stated
rule uniformly ("re-key off `Hashlock`, not off `len(Sha256Digests)`") to
`selfcheck.go:134` turns a digest-count assertion into `b.Hashlock != (p.Hash !=
nil)` and **drops the value comparison at `:138` entirely** — the self-check then
proves only that a hash exists. Second, nothing in the spec states that the
self-check must compare the **kind**. §5's *"eleven map/set/equality sites … re-key
on the whole `HashLock`"* reaches `:138` but not `:134`/`:136`, and only if the
field becomes `[]HashLock` rather than gaining a parallel kind slice — which §7.4
does not say ("becomes kind-carrying" admits both).

**Close it by** naming `composer_selfcheck.go:130-139` in §7.4 with its own rule:
the length check stays a length check over the kind-carrying field, and the value
comparison compares **kind and digest**, with a test that a `hash160` composition
decoded as `ripemd160` fails it.

---

# MINOR

## M-1 — `composer_consent.go:96` is off by one; `:198` points at the loop, not the predicate

`grep -n` is authoritative: the `UNSORTED (EXPERIMENTAL)` predicate is at
**`:97`** (`:96` is the closing brace of the display loop above it), and the §8i
restatement predicate is at **`:199`** (`:198` is `for _, b := range
shape.Branches {`). The `:96` error is inherited verbatim from the design review.
Costs an implementer one `grep`; recorded because §7.4's two bullets are the only
two line numbers in the spec's most behaviour-changing section.

## M-2 — "only a ≤6-character token fits the 411 px band" is not the constraint

Band width **411 px**, confirmed by probe. `rmd160` at **409 px / 1 line**,
2 px to spare — both true. But character count does not decide it. Measured in the
same probe, in the worst row (`hash 10  b867db87..edbc96cb  <tok>  (in payload)`):

| token | chars | px | lines |
|---|---:|---:|---:|
| `rmd160` | 6 | 409 | 1 |
| `sha256` | 6 | 409 | 1 |
| `hash25` | 6 | 409 | 1 |
| `wwwwww` | 6 | 351 | **2** |
| `mmmmmm` | 6 | 369 | **2** |

(For a wrapped row the px column is the post-wrap width, so lines is the verdict.)
The rule is a pixel budget in a proportional font. §7.5's arbiter clause —
*"`TestWhichHashRowsDrawOnOneLine` as arbiter … what it may not do is assume a
token fits"* — contains the damage, which is why this is Minor and not Important.
State the budget as **px**, with `rmd160` as the measured example.

## M-3 — §11 assigns the `156`/`161` fix to phase 1, but the file is in the fork

Verified: the stale count lives at `seedhammer/scripts/vendor-compose-vectors.sh:29`
and is copied into `seedhammer/md/testdata/compose_vectors.provenance.json:6`;
`seedhammer/md/compose_vectors_pin_test.go:91` asserts **161** and the array holds
161 entries. §11 says *"It is fixed in this cycle, **in phase 1**, as part of
re-pinning that file — **not filed**, because the re-pin touches it anyway."*
Phase 1 is descriptor-mnemonic and cannot touch either file; the re-pin is run
from the fork (phase 4) via `scripts/vendor-compose-vectors.sh
/path/to/descriptor-mnemonic`. With no follow-up entry and the wrong owning
phase, this item has nothing left pointing at it. Move it to phase 4.

## M-4 — Two unfolded design-review findings: the fifth Go elision site, and five sha256-hardcoded me-cli strings

M-3 and M-5 of the design review appear nowhere in the spec. Verified still
present:

```
gui/composer_hashlock.go:247  func hashlockFirst8Last8(h [32]byte) string   // arithmetic already length-relative; the PARAMETER is not
me-cli/src/main.rs:2275       "public record {i}: sha256 hashlock (hash:) — {}..{}"
me-cli/src/main.rs:2685       C::Hash => "sha256 hashlock (hash:)"
me-cli/src/main.rs:3196       "a hash record is `hash:` + the 32-byte digest as 64 lowercase hex"
me-cli/.../composer_records.rs:144   "record {index}: hash: must be exactly 64 hex characters"
```

`hashlockFirst8Last8` is invisible to the `[56:]` grep the six-count came from, so
§10 bullet 3's "every formatter" does not reach it. `composer_records.rs:144` is
pinned verbatim by `SPEC_wallet_policy_composer.md` §8n and by `host_line` rows in
`record_class_vectors.json`, so changing it is re-pin work §11's table covers only
generically.

## M-5 — §10 omits the vector rows the design review asked for twice

M-6's "right kind, wrong length" rows — `hash:ripemd160:<64hex>`,
`hash:sha256:<40hex>`, `hash:hash256:<40hex>`, `hash:hash160:<64hex>` — are the
class Core refuses most sharply (core-verdicts lines 127-133, all four measured
REFUSED with one non-diagnostic message), and none is named. Neither are I-8's
both-spelling rows for the accepted-but-never-emitted `hash:sha256:<64hex>` form
that §6 now explicitly admits. The corpus currently has 68 rows, 2 classified
`Hash`.

## M-6 — §6's "(§4.6)" is an unqualified cross-document reference with two candidates

*"Colons, not spaces. Records are line-based and this tree carries dedicated
whitespace seam rows (§4.6)."* This spec has no §4.6. Two design documents in the
same directory do: `SPEC_hashlock_H2_device.md:341` §4.6 = *The Back contract*
(the composer-adjacent one, and the wrong one), and `SPEC_descriptor_input.md`
§4.6 = whitespace/CRLF refusals (`:388`, `:529`). Name the document.

## M-7 — M-4's kind-token case rule is stated as a description, not a rule

§6: *"Tokens are the lowercase miniscript fragment names."* That describes what
the emitter produces; it does not say what the parser does with `HASH160`. The hex
body's rule in this tree is strict rejection, never case folding
(`composer_records.rs:178-192`, `sysw/composer_records.go:178-190`), and the corpus
pins it with a `hash-uppercase` row. Say the same of the token, in one clause,
because "obviously lowercase" is how two parsers end up disagreeing.

---

# NIT

## N-1 — §11's "that stops being true" is asserted, not derived

*"One of `composerHexEntry`'s constants is **documented as unreachable**; that
stops being true."* The comment (`gui/composer_hash.go:98-102`) says `len(raw) !=
32` is unreachable **because the pad offers hex alone** — the pad caps the
fragment and the `== 64` gate makes the decode length a consequence. If the pad
caps at `digest_len()*2` and the decode checks `digest_len()`, the same argument
holds and it stays unreachable. Whether it becomes reachable depends on how the
kind is threaded, which is a plan matter. As written an implementer may read it as
an instruction to add an error path that cannot fire.

## N-2 — Two different things in this tree are called a "hash kind"

`design/IMPLEMENTATION_PLAN_hashlock_F503_hash_kind_inert.md` (shipped) is about
the **ms1 payload kind byte** `0x03` under the id `hash`. This spec's "kind" is the
miniscript hash fragment. They meet in the same `hash:`/preimage neighbourhood and
in the same `sysw` classifier. One disambiguating sentence in §5 would save the
next reader a wrong turn.

---

# VERIFIED SOUND — do not spend the next round re-deriving these

Checked directly, not carried over from the recon summaries.

1. **The fork takes no Rust crate dependency.** `go.mod` names none of the three
   repos; the cross-repo binding is vendored JSON plus a provenance pin, and the
   existing `record_class_vectors.provenance.json` pins a commit on a *branch*
   (`h6-b`), not a release. So §9's *"Phase 4 depends on all three"* is
   satisfiable with no publish gate. The release edge is Rust-internal only (I-1).
2. **The eleven re-key sites are eleven**, enumerated above, and
   `composerState.hashlockHeld` (`composer_state.go:81`) is among them as §5 says.
3. **39 / 76 are exact.** Re-derived independently over the recon's own file
   lists and exclusions.
4. **411 px is exact**, measured by probe at `sh2DisplaySize` through
   `composerTextBand`, the same path `TestWhichHashRowsDrawOnOneLine` uses.
5. **The six `[56:]`/`[56..]` sites are six**, at the exact lines §10 implies.
6. **All six §11 device gate citations are exact** — every file:line and the
   literal `78`. The decision to keep `composerPickScreenMaxRows` **off** the list
   is right and is correctly justified by the separate-screen choice.
7. **F2, F3 and F5 are all true**, including the load-bearing half of F5: the bare
   `ripemd160.New()` primitive — not merely `Hash160` — is already linked, because
   `btcd/address/v2@v2.0.0/hash160.go:22` calls it.
8. **§1's "zero tests and zero vectors compose a non-sha256 hashlock" is true**,
   confirmed by content search in both languages and in the vector directories.
9. **§7.2's split of §8i is correct** and respects the row-independence the
   shipped predicate was deliberately given.
10. **§6's grammar and fail-closed argument** need no re-derivation; the design
    review executed them and nothing in this fold touched the mechanism.

---

# Counts

| severity | n |
|---|---:|
| Critical | 1 |
| Important | 6 |
| Minor | 7 |
| Nit | 2 |

Citations checked: **42**. TRUE **36** (two of them "block start, predicate one
line down"; one TRUE but assigned to the wrong phase), FALSE **5**, unresolvable **1**.

# Verdict

**NOT GREEN.** The spec is a strong fold — it closed 14 of the design review's 20
findings cleanly, its device citations are exact, and its two hardest structural
claims (the 32-byte preimage boundary, the fail-closed grammar) hold. What blocks
it is one missing gate and six joints: **C-1**, the funds-loss path §4 itself
names has no vector and no acceptance item that can detect it; **I-1**, §9's phase
independence is false because `ms-codec` crosses crates.io; **I-2**, §10's
test-before-types ordering cannot be executed; **I-3**, the emulator gate is
repaired one layer below where the walk actually asserts; **I-4** and **I-5**, the
preset arm and archetype are unnamed in both languages and one of them will not
compile; **I-6**, the compose→decode self-check is the third consumer of the field
§7.4 changes and has no rule. All seven are scope and joints, not rework, and all
seven are cheap to close in text.
