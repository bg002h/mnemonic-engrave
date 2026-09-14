# DESIGN REVIEW: hash kinds in the composer (pre-spec), against F-507

Reviewed: the design as stated in the dispatch, against the four recon reports
at mnemonic-engrave `88647df2`, and against the trees at
descriptor-mnemonic `40c400de`, mnemonic-engrave `88647df2`, seedhammer `0562e811`.

Everything below that is a count, a width, a verdict or a byte was **run**, not
read off a doc comment. Commands are given inline so each is re-checkable. No
tracked file was modified; one untracked probe test was created in `gui/`,
measured, and deleted (`git status --porcelain` clean afterwards, verified).

Operator decisions 1-4 are taken as settled and are not argued with.

---

## Counts

| severity | n |
|---|---|
| Critical | 2 |
| Important | 10 |
| Minor | 7 |
| Nit | 1 |

**Verdict: NOT ready to become a spec.** Two Criticals are missing scope, not
wrong prose — one of them is a funds-loss path whose tests would be green. Both
are cheap to close now and expensive to close after a spec freezes around them.

---

# CRITICAL

## C-1 — The ordering omits `md-cli`, so decision 1's "on Rust" is unreachable

**What the design says.** *"Ordering. Two primaries ... (A) `md-codec` —
HashKind/HashLock/lowering; (B) `me-cli` — record grammar; (C) fork Go ports of
both, converging only; (D) fork-native device UI."* Scope: *"Only the composer
layer and above changes."*

**The gap.** `md-codec` is a **library**. The operator-facing Rust authoring
surface is `md compose`, in the **`md-cli`** crate, which the design never names
in its scope, its ordering, its gates or its vectors. F-507 itself names it
verbatim: *"cannot be built on the device **or by `md compose`**"*.

**Verified, by execution.**

```
$ md compose --help
  --path <PATH>  One spend path in listed order:
                 `<k>of<n>[,older=N|older=Nu|after=H|after=Tt][,sha256=HEX][,unsorted]`
                 or `keyless,sha256=HEX[,older=..|after=..]`

$ md compose --wrapper wsh --path "1of1,ripemd160=bd49799e253f26a68e5514e9f45084bf8db3777e"
md: path `1of1,ripemd160=...`: unknown option `ripemd160`
```

`sha256=` is the only hash option, in both `--path` and
`--preset hashlock-gated,sha256=HEX`; its parser is
`crates/md-cli/src/cmd/compose.rs:134-147` (`parse_sha256_hex`, `value.len() != 64`).
The option **name** is kind-specific, so this is a grammar decision — new option
names (`ripemd160=`, `hash160=`, `hash256=`) versus a generalised
`hash=<kind>:<hex>` — not a rename. The blast-radius recon flagged exactly this
(`spec-hashkinds-blast-radius.md` §4b: *"a decision, not a rename"*); the design,
written from that report's summary, dropped it.

Two more Rust authoring sites go with it and are also unnamed:
`crates/md-codec/src/compose/presets.rs:90` `pub fn hashlock_gated(hash: [u8; 32], ...)`
(a public signature) and `presets.rs:11,94`.

**Why Critical rather than Important.** Decision 1 is settled: *"All four kinds
become authorable from the composer, **on Rust** and on the device."* As scoped,
A+B+C+D can all land green and no operator can author a `ripemd160()` policy on
the host. This is the shape the brief warns about — every stage green, the
joining call absent.

**Close it by:** adding `md-cli` as a named work item with its own grammar
decision, or stating explicitly that host authorability is deferred and
amending decision 1's claim.

## C-2 — The kind pick reaches the phrase arm, but no kind-aware digest function exists or is named; as written this composes an unspendable path

**What the design says.** Bounding facts: *"...which keeps the H6 preimage
plates, **the phrase route** and the §8i '32-byte value' rule **out of scope**."*
Device flow: *"...gets a SEPARATE new pick screen, reached only from the
typed-hex **and phrase** arms."*

These two sentences contradict each other, and **both readings are broken.**

**Reading A — the phrase arm really can pick a kind (what the Device flow says).**
The phrase arm's derivation is:

- Go: `hashlock/hashlock.go:85-88`
  ```go
  // Digest is digest: H = SHA-256(X).
  func Digest(x *[32]byte) [32]byte { return sha256.Sum256(x[:]) }
  ```
  Sha256-only, no kind parameter, `[32]byte` out. Called by all three
  material-holding routes (`gui/composer_hashlock.go:85, 156, 200`).
- Rust: `ms_codec::hashlock::digest(preimage: &[u8; 32]) -> [u8; 32]`
  (`mnemonic-secret/crates/ms-codec/src/hashlock.rs:59`) — sha256-only, and it
  lives in **mnemonic-secret, a fourth repo the design's ordering never names.**

The design names neither function. An implementer following it literally picks
`ripemd160` on the kind screen, derives X from the phrase, computes
`sha256(X)` — 32 bytes — stores it in `HashLock{Ripemd160, …}`, and the
accessor hands `digest[..20]` to the lowering. That value is the **first 20
bytes of a SHA-256 hash**, lowered to `Tag::Ripemd160` / `hash160Body`, engraved,
and restored. No byte string on earth is its RIPEMD-160 preimage. The path is
**permanently unspendable**, and every screen downstream still says the
composition holds the preimage, because `hashlockHeld` was populated by the
derive route. Nothing in the type system catches it — `[u8;32] -> [u8;20]` is
exactly the truncation the accessor is *designed* to perform.

**Reading B — the phrase arm stays sha256-only (what the Bounding facts say).**
Then **decision 3 is vacuous.** Measured: the only three routes that call
`composerHoldHashlockMaterial` are typed-phrase, payload-`phrase:` and
payload-preimage-plate (`gui/composer_hashlock.go:85-87, 156-158, 200-202`), and
all three derive through the sha256-only `hashlock.Digest`. If none can yield a
20-byte kind, then *every* 20-byte `HashLock` is unheld, `_, held :=
st.hashlockHeld[...]` is constant false, and *"20-byte kinds warn only when the
device did NOT derive the preimage"* becomes "20-byte kinds always warn" — the
operator's ruling with no effect.

**Verified de-risking, so the spec does not re-litigate it:** the primitive is
already on the device at zero marginal flash cost.
`golang.org/x/crypto/ripemd160` is present at the resolved `v0.52.0`
(`ls $(go env GOMODCACHE)/golang.org/x/crypto@v0.52.0 | grep ripe` → `ripemd160`)
and is **already linked into the firmware** — `address.Hash160` imports it
(`.../btcd/address/v2@v2.0.0/hash160.go`) and `bip32.Fingerprint`
(`bip32/bip32.go:39`) and `address/address.go:139,142` call it today. Note
`Hash160(b) = ripemd160(sha256(b))` gives the `hash160` fragment for free; the
`ripemd160` fragment needs the **bare** primitive, which is the same package.
On the Rust host, `me-cli` already carries `bitcoin = "0.32"`
(`crates/me-cli/Cargo.toml:38`), so `bitcoin::hashes::{ripemd160, hash160, sha256d}`
are in reach with no new dependency.

**Close it by:** deciding which reading holds, and — if A — naming a kind-aware
digest function in each language as a first-class work item, with
mnemonic-secret added to the ordering (or `ms_codec::hashlock::digest` left
alone and the three non-sha256 digests computed in `me-cli` from `bitcoin::hashes`).

---

# IMPORTANT

## I-1 — "The display row gains the kind" does not fit; measured, it wraps

**What the design says.** *"The `hash <i> first8..last8` display row gains the
kind for non-sha256."*

**Measured**, by a probe test in `package gui` at `sh2DisplaySize`, through
`composerTextBand` / `widget.Labelw` — i.e. the same measurement
`TestWhichHashRowsDrawOnOneLine` (`gui/composer_hash_test.go:239-269`) makes.
**Band width: 411 px.** (For a row that wraps, the px column is the *post-wrap*
width, so only the line count is the verdict.)

| row | chars | px | lines |
|---|---:|---:|---:|
| `hash 10  b867db87..edbc96cb` (today) | 27 | 236 | 1 |
| `hash 10  b867db87..edbc96cb  (in payload)` (today, worst) | 41 | 343 | 1 |
| `hash 10  b867db87..edbc96cb  ripemd160` | 38 | 327 | 1 |
| **`hash 10  b867db87..edbc96cb  ripemd160  (in payload)`** | 52 | — | **2** |
| **`hash 10  ripemd160  b867db87..edbc96cb  (in payload)`** | 52 | — | **2** |
| **`hash 10  b867db87..edbc96cb  hash160  (in payload)`** | 50 | — | **2** |
| **`hash 10  b867db87..edbc96cb  hash256  (in payload)`** | 50 | — | **2** |
| `hash 10  b867db87..edbc96cb  rmd160  (in payload)` | 49 | 409 | 1 |
| `hash 10  b867db87..edbc96cb  r160  (in payload)` | 47 | 382 | 1 |

**All three new kind names, spelled in full, wrap band 1's annotated row to two
lines** — in either position — and fail the shipped one-line gate. Only a
**six-character-or-shorter** token survives, and `rmd160` clears the band by
**2 px of 411**. The unannotated rows (`preimage … ripemd160` 367 px,
`phrase … ripemd160` 344 px) do fit; it is specifically band 1's `(in payload)`
variant (`composerHashInPayloadRow`, `gui/composer_hash.go:165-167`) that breaks,
and that file's own comment records the same pressure already —
`(in payload)` was chosen over `(preimage in payload)` precisely because the
longer form wrapped.

This is not a styling quibble: `composerHashRow`'s doc comment says a row over
budget *"would be CUT rather than wrapped ... and a cut digest is worse than an
elided one — the operator cannot tell which end is missing."*

**Close it by:** picking the on-screen token as a spec decision with the
measurement attached, and re-measuring the chosen spelling in the probe before
the spec freezes.

## I-2 — "Five map/set/equality sites" is an undercount; measured, there are eleven

**What the design says.** *"Five map/set/equality sites currently keyed on a
bare `[32]byte` (including `composerState.hashlockHeld`) re-key on the whole
`HashLock`."*

**Measured** (`grep -rn "== h\b|!= h\b|== d\b|!= d\b|== \*p\.Hash|!= \*p\.Hash|seen\[|Held\[|Digests\[" --include=*.go gui/composer*.go md/policy_shape.go md/compose.go | grep -v _test`),
then each hit read to confirm it is a digest comparison:

Digest-keyed **maps/sets** (3):
1. `gui/composer_state.go:53` `phraseDigests map[[32]byte]struct{}` (w `:312`, r `:389`)
2. `gui/composer_state.go:81` `hashlockHeld map[[32]byte]hashlockMaterial` (w `:350,:368`, r `:83,:105,:146,:439,:462`)
3. `gui/composer_preimage_plate.go:78` `seen map[[32]byte]bool` (`:80,:87,:92`)

Digest **equality** comparisons (7):
4. `gui/composer_hash.go:370` `p.digest == d` — `composerHashInPayload`, preimage arm
5. `gui/composer_hash.go:375` `*h == d` — `composerHashInPayload`, phrase arm
6. `gui/composer_hashlock.go:176` `d == h` — `payloadStatesDigest`
7. `gui/composer_hashlock.go:218` `d == h` — `hashlockRelationLine`
8. `gui/composer_hashlock.go:240` `*p.Hash != h` — `hashlockOtherPathLine`
9. `gui/composer_preimage_plate.go:152` `*p.Hash == h`
10. `gui/composer_selfcheck.go:138` `b.Sha256Digests[0] != *p.Hash`

Rust (1):
11. `crates/me-cli/src/main.rs:2570` `hashes.contains(&digest)` over `Vec<[u8;32]>`

**Eleven, not five.** The design inherited the five from
`spec-hashkinds-blast-radius.md` §4d, which was written as the *lead item*
("this is the one found"), not as an enumeration. Six of the eleven — items
4,5,6,7,9,10 — are absent from the design entirely, and four of those six drive
**operator-facing copy on the screens that gate funds** (see I-3).

## I-3 — Every "does this digest match the payload / the material?" comparison computes a sha256 digest, so a correct non-sha256 payload is reported as mismatched

**What the design says.** Nothing. The design treats these as re-keying work
(and under-counts them, I-2); it never states that the *value* on one side of the
comparison has to be recomputed under the other side's kind.

**The concrete failure, host side.** `me sysw pack`'s §8.2.3 orphan check
(`main.rs:2566-2615`) collects `hash:` record digests into `Vec<[u8;32]>`, then
for each preimage/`phrase:` carrier computes `preimage_digest_of` — which is
`ms_codec::hashlock::digest`, i.e. **sha256(X)** (`main.rs:2619-2645`) — and tests
`hashes.contains(&digest)`. Pack a payload holding a correct
`hash:ripemd160:<ripemd160(X)>` record **and** the matching preimage plate, and
the host prints:

> `me: WARNING — record N ... is a preimage whose digest <first8>..<last8> matches no `hash:` record in this payload.`

A **false** warning on a correct payload, on the verb whose whole job is to catch
the hand-built-record errors the comment above it enumerates. Re-keying on the
whole `HashLock` (the design's stated remedy) does not fix this — it makes the
miss *structural* instead of accidental. The host must compute the digest under
each candidate record's kind.

**The concrete failure, device side.** The same shape, three more times:

- `composerHashInPayload` (`gui/composer_hash.go:368-380`) decides whether band 1's
  row gets the `(in payload)` annotation, by comparing the `hash:` record's digest
  against `p.digest` (from `hashlock.Digest`, sha256) and against
  `hashlockDerivedDigest`. For a `ripemd160` record whose preimage **is** in the
  payload, the annotation silently disappears — and that annotation exists
  precisely because, per its own comment, band 1 and band 2 otherwise show
  *"identical digest text, adjacent, differing by one word"* with opposite
  consequences for whether a plate is offered at Done.
- `hashlockRelationLine` (`gui/composer_hashlock.go:212-224`) words the confirm
  modal's relation line. It would print the "no record matches" arm for a payload
  that does match.
- `payloadStatesDigest` (`gui/composer_hashlock.go:174-182`) exists, per its own
  comment, *"so the two screens cannot disagree about it (F-496)"* — it would be
  consistently wrong instead.

**Close it by:** stating, as a normative rule, that a digest is only ever
compared to another digest **of the same kind**, and that matching a preimage to
a `hash:` record requires computing the digest under that record's kind — in both
languages — with a vector for each.

## I-4 — The §8i rule fires before the kind is chosen, so its copy cannot name the kind

**What the design says.** *"Kind is chosen BEFORE the pad"* and *"§8i's rule copy
becomes `<kind>` of a 32-byte value."*

**The conflict, in shipped code.** `composerHashEdit`
(`gui/composer_hash.go:404-411`):

```go
taking := sel != rows.noneRow
if taking {
    showError(ctx, th, title, composerCopyHashRule())
}
switch { ... case sel == rows.hexRow: d, ok := composerHexEntry(ctx, th) ... }
```

The §8i modal fires **on row selection**, before the hex arm or the phrase arm is
entered — therefore before any kind pick screen the design places "before the
pad". At that instant no kind exists for those two arms, so the entry copy cannot
say `<kind>`. §8i's own heading is *"at entry **and** at consent"*; only the
consent restatement (`gui/composer_consent.go:198-203`) can name a kind.

Worse, the obvious fix — moving the modal after the kind pick — attacks a
predicate the code deliberately made row-independent. Its comment: *"STATED AS
THE ONE ROW IT IS NOT ... adding a band to a screen would then silently leave the
32-byte rule unstated for it — the failure being that the operator takes a hash
without ever being told what a preimage must be."*

**Close it by:** splitting §8i into a kind-generic **entry** body (unchanged,
still fired by `taking`) and a kind-naming **consent** body — or stating that the
kind pick precedes the §8i modal and re-justifying the predicate.

## I-5 — "A total `HashKind ↔ Tag` mapping defined once per language" is not achievable in Rust as the repos stand

**Measured.** `crates/me-cli/Cargo.toml:26` → `md-codec = "0.42"` — a **published
crates.io dependency**, not a path dep, and the two crates live in different
repos. `Tag` and (per the design) `HashKind` live in `md-codec`.

So in Rust the design forces one of three things, and names none:

1. **Publish md-codec first.** B cannot compile against A's new `HashKind` until
   A is released and `me-cli`'s version requirement is bumped — with the
   `Cargo.lock` / `cargo vendor` freshness ritual that goes with it. "A before B"
   is then not a sequencing preference but a **release** dependency, and the
   design's *"Two primaries"* framing hides a publish gate between them.
2. **Define the kind token set twice in Rust** — once in `md-codec`/`md-cli` for
   the composer grammar, once in `me-cli` for the record grammar. That is exactly
   the drift the design's "once per language" sentence is trying to prevent, and
   the two definitions would sit on opposite sides of a repo boundary with no
   shared test.
3. Move the record grammar's kind token into a shared crate — a structural change
   nothing in the design contemplates.

Note this asymmetry is Rust-only: in Go, `md` and `sysw` are packages in one
module, and `md` does not import `sysw`
(`grep -rn "sysw" md/*.go | grep -v _test` → empty), so `sysw → md` is
cycle-free and one definition genuinely works.

## I-6 — C and D are not separable landings; the Go type change is atomic across `md/` and `gui/`

**What the design says.** *"(C) fork Go ports of both, converging only; (D)
fork-native device UI."*

Changing `md.SpendPath.Hash` from `*[32]byte` to a `*HashLock`
(`md/compose.go:167`) breaks every reader at compile time in the same commit. The
blast-radius recon counted them: **39 production + 76 test references across 12
production files and 13 test files**, and 12 of those 12 production files are in
`gui/` except four in `md/`. There is no intermediate tree where C has landed and
D has not.

That matters because the design's own framing — *"converging only"* for C — is
what the Rust-primary rule keys on. A commit that must simultaneously rewrite
`gui/composer_hash.go`, `gui/composer_hashlock.go`, `gui/composer_preimage_plate.go`
and `gui/composer_state.go` is **not** a convergence port; it is fork-native
authorship wearing a convergence label, and it will be reviewed as the former.

**Close it by:** either declaring C+D one phase with one gate, or defining C as a
strictly additive step (add `HashKind`, `HashLock` and the kind-aware helpers
alongside the existing `*[32]byte` field; flip the field in D) — which is a real
design choice with a real cost, and should be made deliberately rather than
discovered by the implementer.

## I-7 — No device acceptance is named, and the emulator's only window into the composition is `[]*[32]byte`

**What the design says.** Vectors: per-kind round trip under `wsh`/`sh(wsh)`,
Core's addresses pinned, a 40-hex digest through every formatter, a fail-closed
unknown-token vector. **No device walk, no emulator gate, no on-device acceptance.**

Decision 1 makes the device half of this change normative, and this repo's own
closure rule is that *a plan may not close while any of its own gates has never
been run* — with the emulator walk named as the gate for composer work.

There is also a mechanical blocker the design must name.
`gui/composer_state_hook.go:48,81`:

```go
var composerStateHook func() []*[32]byte
func ComposerPathHashes() []*[32]byte   // "Exported because cmd/emu is a different package"
```

This is the emulator's **only** read access to the running composition's hashes.
As a `[]*[32]byte` it cannot express a kind — and worse, after the change it
would hand out the **padded raw array**, so a walk asserting "path 1 holds digest
D" would pass identically for `sha256(D)` and for a 20-byte kind whose first 20
bytes match. That is the one place the design's "zero padding is never
observable, because nothing reads the raw array" claim is false, and it is the
place where being false makes a *gate* unable to fail.

## I-8 — The grammar does not say whether `hash:sha256:<64hex>` is accepted on input

**What the design says.** *"`hash:` `[<kind>:]` `<hex>` ... absent kind means
sha256. The host emits the BARE form for sha256 and the explicit form only for
the other three."* That fixes the **emitter**. It says nothing about the
**parser**.

Both answers have consequences the design must own:

- **Accepted** → two byte-different records carry the identical policy. The
  payload's operator-visible identity digest differs between them (verified: a
  bare-`hash:` pack printed `digest: f694 3c66 66ac 58cb c269 d97a ca29 5aa4`),
  and `record_class_vectors.json` needs rows for both spellings of all four kinds.
- **Refused** → the grammar is asymmetric: the kind token is mandatory for three
  kinds and *mandatorily absent* for the fourth. An operator who has just been
  told to hand-build `hash:ripemd160:<40hex>` will type `hash:sha256:<64hex>`
  next, and today that is exactly what the tool teaches them to expect.

This is a normative wire rule on a cross-language pinned surface
(`crates/me-cli/testdata/record_class_vectors.json`, 68 rows, vendored and
sha256-pinned into `seedhammer/sysw/testdata/`). It cannot be left to the
implementer.

## I-9 — Changing `PolicyShape` is a DECODE-side change, which contradicts the design's own scope sentence and silently flips two shipped consent predicates

**What the design says.** Scope: *"Only the composer layer and above changes;
wire format and lowering are already generic."* Data model:
*"`PolicyShape.Sha256Digests` ... becomes kind-carrying."*

`md/policy_shape.go` is the **decompose** direction. It runs on any md1 payload —
including ones this device never composed — and feeds the consent screen. Making
the field kind-carrying and populating it for all four tags changes two live
predicates for such payloads:

- `gui/composer_consent.go:96`
  ```go
  if sole && !b.Sorted && b.N >= 2 && len(b.Locks) == 0 && len(b.Sha256Digests) == 0 {
      out = append(out, "  UNSORTED (EXPERIMENTAL)")
  }
  ```
  Today a hand-built `hash160`-locked sole unsorted multi path **prints**
  `UNSORTED (EXPERIMENTAL)` (because the field stays empty for that tag —
  `policy_shape.go:290`). After the change it **stops printing**. That is a
  behaviour change on decode of existing payloads, in the opposite direction from
  a fix.
- `gui/composer_consent.go:198-203` gates the §8i rule restatement on
  `len(b.Sha256Digests) > 0`. Today a decoded `ripemd160`-locked policy is
  consented to **without** the 32-byte-preimage rule ever being stated. After the
  change it is stated. That is a genuine fix — but an *unannounced* one, on the
  consent screen, reached by payloads outside the composer.

The blast-radius recon flagged this predicate as *already* wrong and said the
spec *"must decide explicitly whether the fixed field's replacement keys off
`Hashlock` or off 'does this path have a hash of any kind'"*. The design did not
decide it.

## I-10 — The "gates to move deliberately" list omits the gates this change actually breaks

**What the design says.** *"Gates to move deliberately: the 78-body copy table
plus §8 rows, the modal fit gate, `composerPickScreenMaxRows = 24`, both
provenance pin files, and a stale count in the compose-vector pin generator's own
comment."*

Machine-checked, the three cited device constants are **correct**:
`gui/composer_copy_test.go:371` `if declared != 78`; `gui/composer_paged.go:243`
`const composerPickScreenMaxRows = 24`. But `composerPickScreenMaxRows` is the
*least* relevant of them — the design's own separate-screen choice means
`Which hash?` gains no band, and a four-row kind screen is nowhere near 24.

The gates that *do* break are absent:

| gate | file:line | why it breaks |
|---|---|---|
| `TestWhichHashRowsDrawOnOneLine` | `gui/composer_hash_test.go:239-269` | measures **every row form** and the literal labels; I-1 shows the kind-bearing rows wrap |
| the literal `"Type 64 hex"` | `gui/composer_hash.go:348` **and** `composer_hash_test.go:257` | the label is wrong for a 40-hex kind; it is pinned by the geometry gate as a string literal |
| `TestWhichHashPageHoldsFiveRows` | `gui/composer_hashlock_test.go:1433-1460` | pins the page-1/page-2 boundary; its own comment: *"If this moved, every row count in Task 8b moved with it."* |
| `composer_hash.go:79-105` `composerHexEntry` | pad cap `> 64`, `valid := len(frag) == 64`, and the belt-and-braces `len(raw) != 32` | three hardcoded 64/32 constants, one of them *documented as unreachable* — which stops being true |

The `[56:]` claim, by contrast, **checks out exactly**:
`grep -rn "\[56:\]" --include=*.go . | grep -v _test` → 4 hits
(`composer_hash.go:50,173,188`, `composer_consent.go:63`);
`grep -rn "\[56\.\.\]" --include=*.rs crates/` → 2 hits
(`main.rs:2277,2602`). **Six.** Correct.

---

# MINOR

## M-1 — Two of the six digest-entry routes cannot reach the kind screen, and the design does not say so

The device-surface recon enumerates **six** entry paths. The design's kind screen
is *"reached only from the typed-hex and phrase arms"*, which leaves:

- **Route 5, the payload preimage-PLATE record** (`gui/composer_hashlock.go:192-204`).
  An operator holding an H6 preimage plate can therefore only ever build a
  **sha256** lock from it. Given H6 is this constellation's flagship hashlock
  artefact, that deserves a sentence, not silence.
- **Route 6, the preset archetype** (`gui/composer_presets.go:50-60`,
  `composerPresetDigest` = 32 bytes of `0xa8`, and Rust
  `presets::hashlock_gated`). It assigns a digest with **no** held material, so
  under decision 3 a 20-byte preset would warn about a placeholder. It needs a
  `HashKind` at construction either way.

## M-2 — Core's matrix was measured for the KEYED shape only; the keyless shape is unmeasured for the three new kinds

`spec-hashkinds-core-verdicts.md` used `and_v(v:pk(PUBKEY),<fragment>)` in
**every** cell. The composer's keyless path (`keys: None`, wsh-only,
EXPERIMENTAL) is a *hash-only* miniscript, and no cell measured one. The risk is
low — the existing Rust test
`a_keyless_wsh_path_is_admitted_with_top_unsafe_and_refused_by_the_default_sanity`
shows the keyless refusal is a top-level-sanity property, and `to_miniscript`
already proptests all four tags — but the design states the bounding fact
(*"Core accepts all four under `wsh` and `sh(wsh)`"*) wider than the evidence
behind it. Four more `getdescriptorinfo` calls would close the gap.

## M-3 — "the ONE length-relative elision helper" is at least two, and there is a fifth Go site

Two languages means two helpers. And `hashlockFirst8Last8`
(`gui/composer_hashlock.go:247-250`) is *already* length-relative in its
arithmetic (`s[len(s)-8:]`) but takes `h [32]byte` and does
`hex.EncodeToString(h[:])` — so it is a **fifth** Go site needing a parameter
change, invisible to the `[56:]` grep the six-count came from.

## M-4 — Kind-token case is unspecified

The hex rule in both parsers is strict lowercase — uppercase is *rejected*, never
folded (`composer_records.rs:178-192`, `sysw/composer_records.go:178-190`), and
the vector corpus pins that with a `hash-uppercase` row. The kind token should
carry the same rule explicitly; "obviously lowercase" is how two implementations
end up disagreeing on `HASH160`.

## M-5 — Five more sha256-hardcoded operator-facing strings are unnamed

Beyond the 78-body device copy table, all in `me-cli`, all wrong the moment a
second kind exists — and all outside any gate:

- `main.rs:2275` `"public record {i}: sha256 hashlock (hash:) — {}..{}"`
- `main.rs:2685` `C::Hash => "sha256 hashlock (hash:)"` (the class label)
- `main.rs:3196` the build-a-record help: *"a hash record is `hash:` + the 32-byte digest as 64 lowercase hex"*
- `composer_records.rs:144` `"record N: hash: must be exactly 64 hex characters"`
- `SPEC_wallet_policy_composer.md` §8n, which pins that refusal line verbatim

All five reproduced in one run:

```
$ me sysw pack "hash:ripemd160:bd49799e253f26a68e5514e9f45084bf8db3777e"
me: record 0 ... is a `key:`/`hash:`/`now:`/`phrase:` record whose body fails its rule
    (not exactly 64 lowercase hex characters).
      record 0: hash: must be exactly 64 hex characters
      ... a hash record is `hash:` + the 32-byte digest as 64 lowercase hex; ...
```

## M-6 — The vector plan is thin where Core is strictest

*"a fail-closed vector for an unknown kind token"* is one row. The class Core
refuses most sharply is **right kind, wrong length** — `ripemd160(<64 hex>)` and
`sha256(<40 hex>)` both REFUSED, with one generic, non-diagnostic message
(`spec-hashkinds-core-verdicts.md`). Name those rows explicitly:
`hash:ripemd160:<64hex>`, `hash:sha256:<40hex>`, `hash:hash256:<40hex>`,
`hash:hash160:<64hex>` — plus a valid row per kind. The current corpus has 68
rows and **2** classified `Hash`.

## M-7 — `golang.org/x/crypto/ripemd160` is deprecated (but present, and free)

Record it once so a later reviewer does not re-open it: the package is frozen
upstream, is present at the resolved `v0.52.0`, and is **already linked** into the
firmware via `address.Hash160`. Marginal flash cost for the bare primitive:
effectively zero. Worth one line in the spec's dependency note.

---

# NIT

## N-1 — §8i is declared out of scope and then changed

Bounding facts: *"...keeps ... the §8i '32-byte value' rule out of scope."*
Device flow: *"§8i's rule copy becomes `<kind>` of a 32-byte value."* The
*32-byte-ness* is what stays; the *sha256 word* is what moves. Say that, or the
next reader resolves it the wrong way (and I-4 shows the wrong way is reachable).

---

# VERIFIED SOUND — do not spend spec budget re-deriving these

Each was checked against the code or run, not accepted from the recon summaries.

1. **The grammar is unambiguous.** No kind token is valid hex, and `:` is not a
   hex character, so a body containing a colon can never be read as a bare
   digest, and a bare digest can never be read as `<kind>:<hex>`. A 20-byte kind
   whose name were 23 characters long would produce a 64-character body — still
   rejected, because `unhex_lower`/`unhexLower` reject the colon
   (`composer_records.rs:178-192`).

2. **Fail-closed is real, and was executed, not reasoned.** Against the shipped
   `me` (debug build at `88647df2`):
   - `hash:ripemd160:<40hex>` → REFUSED, whole payload, named record index
   - `hash:hash160:<40hex>` → REFUSED
   - `hash:sha256:<64hex>` → REFUSED
   - bare `hash:<64hex>` → **packs**, byte-compatible control ✓

   The refusal is `admit_check` (`sysw/mod.rs:531-546`): **any** `Class::Unknown`
   record refuses the entire pack. On the device the same record is
   `ClassUnknown` (`sysw/composer_records.go:104-122` — prefix matches, parse
   fails, falls through) and **inert**: `gui/composer_door.go:36,57` keeps such
   records in the session and counts them as "not understood"; it does not refuse
   the payload. Old host = loud refusal, old device = silent inertness. Both
   fail closed, as claimed.

3. **"The preimage is always 32 bytes" is airtight.** `md/script_emit.go:458-487`,
   read in full: **both** arms — `case tagSha256, tagHash256` and
   `case tagRipemd160, tagHash160` — emit `opSIZE`, `pushNumber(out, 32)`,
   `opEQUALVERIFY` before the hash opcode. There is no kind-conditional and no
   tapscript exception (unlike `tagMulti`, which *does* branch on `e.tap`
   immediately below). Rust renders all four through `render.rs:199-202` into
   canonical miniscript, which carries the same `SIZE <32> EQUALVERIFY`. **H6
   preimage plates, `codex32.EncodeMS1Preimage`, and the 32-byte-preimage rule
   are correctly out of scope.** This was the most valuable thing that could have
   been wrong, and it is right.

4. **Zero padding cannot reach a script or an id.** The wire body types are
   `Body::Hash160Body([u8; 20])` (`md-codec/src/tree.rs:68`) and
   `hash160Body [20]byte` (`md/md.go:121`) — a 32-byte array does not compile
   into either, in either language, so the truncation is forced at the type level
   rather than trusted. The wallet id/stub is computed over the **encoded
   chunks** (`md.FormAwareIdChunks`, `md/template_id.go:163-174`), which carry the
   20-byte wire form, so padding never enters it either. The only leak found is
   the emulator hook (I-7).

5. **The separate-kind-screen choice is right**, and for a better reason than the
   design gives. Beyond the three row-count gates, `composerHashEdit` ends in
   `default: panic(...)` (`gui/composer_hash.go:452-454`), and its band dispatch
   is by recorded band-start index specifically because *"the shipped default arm
   cleared the lock when a row moved"* (r2 review C-4). A new band in
   `Which hash?` would re-enter that repaired failure. A separate screen does not.

6. **Cited constants that check out:** 78 copy bodies
   (`composer_copy_test.go:371`); `composerPickScreenMaxRows = 24`
   (`composer_paged.go:243`); six hardcoded `[56:]`/`[56..]` elision sites (4 Go,
   2 Rust); `PolicyShape.Sha256Digests` sets `Hashlock` for all four tags but
   appends only for `tagSha256` (`policy_shape.go:284-291`); no `HashLock` name
   collision anywhere in the three trees; no Rust `PolicyShape` counterpart
   exists, so that change is Go-only (which the design's "Data model" section
   should say, since it reads as language-neutral).

7. **`md descriptor` / rendering / address derivation already handle all four**
   (`render.rs:199-202`, `to_miniscript.rs:664-682`, `tag.rs:129-132,193-196`), and
   `me-cli` reaches address derivation through `md_codec::Descriptor::derive_address`
   with no hash-kind involvement. Nothing is missing on that surface.

---

# The one-line verdict

**Not ready.** Close C-1 (name `md-cli` and decide its option grammar) and C-2
(decide whether the phrase arm picks a kind, and if so name the kind-aware digest
function in each language and add mnemonic-secret to the ordering) before
drafting; fold I-1 through I-10 into the draft rather than after it. The
foundation the design rests on — 32-byte preimages for every kind, a fail-closed
grammar, padding that cannot reach a script — is sound and was verified by
execution, so the work that remains is scope and joints, not rework.
