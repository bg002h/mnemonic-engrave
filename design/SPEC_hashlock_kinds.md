# SPEC — hashlock kinds: all four miniscript hash fragments, authorable

**Closes F-507.** That follow-up was filed 2026-09-12 from the operator's
timelock+hashlock experiment and names "the next composer cycle" as its owner.
This is that cycle. It also narrows F-150 item 4.

**Status:** written 2026-09-14, pending the R0 architect review. No code before
0 Critical / 0 Important.

---

## 1. The gap

The md1 wire format and the script lowering already handle all four miniscript
hash fragments in both languages — tags `0x1D` sha256, `0x1E` hash160, `0x1F`
hash256, `0x20` ripemd160, both body widths, and the right opcode emitted for
each. A `ripemd160` policy can be encoded, engraved, restored and derived today.

It cannot be **composed**. One layer above the codec:

| surface | file | today |
| --- | --- | --- |
| composer field | `md-codec` `compose/mod.rs:151`, fork `md/compose.go:167` | a bare 32-byte digest |
| lowering | `compose/lowering.rs:78`, fork `md/compose.go:403` | always `Tag::Sha256` |
| `md compose` CLI | `md-cli` | only a `sha256=` option |
| `hash:` record | `me-cli/sysw/composer_records.rs` | `hash:` + exactly 64 hex, sha256 implied |
| device pad | fork `gui/composer_hash.go` | 64 hex only |
| phrase digest | fork `hashlock/hashlock.go:85`, `ms-codec/hashlock.rs:59` | sha256 only |

**Zero tests and zero vectors compose a non-sha256 hashlock anywhere, in any
language** — confirmed by content search, not filenames. Every test this cycle
writes is net new, and nothing constrains their shape.

## 2. Decisions

Operator rulings, 2026-09-13/14. Settled; a reviewer may test the execution, not
the decision.

1. **All four kinds become authorable from the composer**, on Rust and on the
   device.
2. **The `hash:` record carries the kind**; the device asks when a digest is
   typed by hand; a bare legacy `hash:` keeps meaning sha256.
3. **20-byte kinds warn only when the device did not derive the preimage.**
4. **A kind-tagged `HashLock` with the digest length derived from the kind** —
   chosen over a variable-length slice because the device has 0-alloc paths
   under TinyGo.
5. **The phrase route derives all four**, and `mnemonic-secret` joins the cycle
   so `ms hashlock` can still reproduce a plate's digest.

## 3. Bounding facts

Measured this cycle. Sources: `design/agent-reports/spec-hashkinds-*.md`.

**F1 — the preimage is 32 bytes for every kind.** All four fragments emit
`OP_SIZE <32> OP_EQUALVERIFY <hashop> <h> OP_EQUAL`. This is what keeps the H6
preimage plates and the §8i "32-byte value" rule structurally intact: only the
digest width moves, between 32 and 20.

**F2 — no per-kind wrapper rules.** Bitcoin Core 25.0.0 accepts all four under
`wsh(...)` and `sh(wsh(...))`. Bare `sh(...)` and `tr(...)` refuse with
`Miniscript expressions can only be used in wsh` — **identically for a
miniscript carrying no hash fragment at all**, confirmed by control. That is a
wrapper property. The three new kinds inherit exactly the constraints sha256
lives under.

**F3 — Core's malformed-digest diagnostics are useless.** Every bad case (wrong
length either way, non-hex, empty) returns the same `A function is needed within
P2WSH`. Device-side length validation is therefore the only place a usable
message can come from.

**F4 — the four tags are four functions, not two widths** (F-507). `ripemd160`
emits `OP_RIPEMD160`; `hash160` emits `OP_HASH160` = RIPEMD-160 of SHA-256. Same
20-byte width, different preimage relation: for one digest `h`, a preimage
satisfying one will not satisfy the other. The width is a consequence; the kind
is the semantics.

**F4 applies to the 32-byte pair too, and there it is more dangerous.**
`hash256` is `sha256d` — `sha256(sha256(X))`. An implementer one word short
(`sha256(x)` where `sha256d(x)` was meant) produces a 32-byte value with **no
structural signal whatever**: no truncation, no width change, it type-checks
exactly, it passes `digest_len()*2`, it lowers cleanly, Core agrees with the
address, and it locks the path to a preimage that does not exist. §10's per-kind
digest KAT is the only thing that catches it.

**F5 — the primitive is already on the device at zero marginal flash cost.**
`golang.org/x/crypto/ripemd160` is already linked in (`address.Hash160`,
`bip32.Fingerprint`). `me-cli` already carries `bitcoin = "0.32"`.

## 4. Scope

**In:** the composer layer and above in four repos, **plus one decode-side
change** (§7.4). The wire format and lowering are untouched.

**Out:**

- The phrase→**preimage** derivation, by F1. The preimage stays `sha256(phrase)`,
  32 bytes, for every kind.

  **F1 does NOT put the preimage plate out of scope, and an earlier draft used it
  that way.** F1 is a fact about what the preimage *is*; it says nothing about
  what the plate *says*. The plate's own design rule is that it is read years
  later by someone with neither the tool nor this firmware, so it spells its
  parameters out — and adding a fourth-kind world without touching it leaves that
  spelling one step short. §13 is that surface.

  **This is the boundary the design review found contradicted (C-2), so it is
  stated twice on purpose.** The phrase route IS in scope for the step that turns
  a preimage into a digest — that function is sha256-only today in both languages
  and gains a kind (§9 phases 2 and 4). What is out of scope is everything
  upstream of the preimage. An implementation that picks a kind and then computes
  `sha256(preimage)` anyway produces the first 20 bytes of a SHA-256 hash lowered
  under `Tag::Ripemd160`: a permanently unspendable path that no type check
  catches and every screen reports as held.
- `readChunkHeader`'s misleading "wire version mismatch" on a non-chunked
  string — a separate diagnostic defect, filed independently.
- `me sysw pack`'s packing UX beyond the kind token.

## 5. Data model

**`HashKind`** — four values: `sha256`, `hash256`, `ripemd160`, `hash160`.
Deliberately **not** the 30-value wire `Tag` set: a hashlock field typed as `Tag`
can hold `Wpkh`. The `HashKind ↔ Tag` mapping is a total function.

**`HashKind::digest_len()` → 20 or 32 is the only place a length is written.**
The hex rule becomes `digest_len() * 2`. This retires four separate copies of
"exactly 64 hex" and is what makes the record parser, the pad, the validator and
the formatter agree by construction rather than by review.

**`HashLock { kind, digest: [u8; 32] }`**, accessor returning
`digest[..kind.digest_len()]`. Fixed array for the alloc gate; the zero padding
on a 20-byte kind is unobservable **except** through `ComposerPathHashes` (§7.3),
which this cycle fixes.

**Eleven** map/set/equality sites currently keyed on a bare `[32]byte` — including
`composerState.hashlockHeld`, on which §8's warning depends — re-key on the whole
`HashLock`. Not for collision reasons but semantic ones: the same 32 bytes mean
different things under different kinds.

**One local definition per crate, nothing shared across repo boundaries.**
`me-cli` depends on `md-codec` through **crates.io**, so a shared `HashKind`
would make the phase order a *release* dependency with a `cargo vendor`
freshness ritual attached. Instead `ms-codec` exposes one digest function per
kind and each caller maps its own local kind onto one. What crosses a boundary
is the **wire token and the digest value**, pinned by vectors — the mechanism
this constellation already uses for Rust↔Go. Go genuinely has one definition:
`md` and `sysw` are packages in one module and `md` does not import `sysw`.

## 6. Record grammar

    hash: [<kind>:] <hex>

`<hex>` is `kind.digest_len() * 2` lowercase hex characters. An absent kind means
`sha256`. Tokens are the lowercase miniscript fragment names.

**Case is rejected, never folded.** `HASH160` and `Hash160` are refused, exactly
as the hex body already refuses uppercase (`composer_records.rs:178-192`,
`sysw/composer_records.go:178-190`, pinned by the corpus's `hash-uppercase` row).
Stated as a rule rather than left to "obviously lowercase", which is how two
parsers come to disagree.

**Two axes, one word — and this is a design constraint, not a naming note.**

| axis | what it selects | where it lives | values include |
| --- | --- | --- | --- |
| **method** | how a *preimage* is derived from a phrase | `phrase:` records, the plate's `method:` line, the §8.3 census row | `sha256` |
| **kind** | which hash the *script* commits to | the md1 tag, `HashLock.kind`, the §6 record token | `sha256`, `hash256`, `ripemd160`, `hash160` |

They are orthogonal, and they share the token `sha256`. Three independent reviews
of this cycle each found a defect caused by that collision — a spec sentence
claiming records "carry their kind" when they carry a method (R0 r2 I-1), a
census row showing the method where the operator reads an inventory (restore
I-3), and a confirm screen printing `method: sha256` on a `hash256` wallet
(journey C-1). It is not a wording problem.

**The rule, and every screen and record is bound by it:** where both axes could
be read, **both are named or neither is**. No surface prints one bare token that
an operator could take for the other. A screen that shows `method:` and no kind
on a policy whose kind is not sha256 violates this spec.

**Producer rule.** The host emits the **bare** form for sha256 and the explicit
form for the other three. Every payload that exists today, and every new sha256
payload, is byte-identical to what is packed now.

**Input is liberal, output is conservative.** `hash:sha256:<64hex>` is
**accepted** on input and never emitted.

**Fail-closed, verified by execution rather than argument:** an old host refuses
the whole pack; an old device treats the record as `ClassUnknown` and **inert** —
it reaches no screen and surfaces only in the door's not-understood count; a bare
`hash:` still packs. The rejected alternative is any scheme where an old parser
reads a non-sha256 digest *as* sha256, which composes a wallet nobody can spend,
silently.

**Colons, not spaces.** Records are line-based and this tree carries dedicated
whitespace seam rows — `SPEC_descriptor_input.md` §4.6 (`:388`, `:529`), the
whitespace/CRLF refusals. (Not `SPEC_hashlock_H2_device.md` §4.6, which is the
Back contract. Several documents in this directory carry a §4.6, which is why
the reference is qualified.)

## 7. Device

### 7.1 Where the kind is chosen

`Which hash?` picks a **source** (payload records, phrase, hex, none). The kind
is a different axis and gets its **own** pick screen, reached from the typed-hex
and phrase arms only — a payload record carries its own kind and asks nothing.

Chosen partly to avoid collateral damage: three hand-maintained gates are wired
to `Which hash?`'s row counts, and a new band there disturbs all three for no
design benefit.

**Kind before the pad.** The pad then accepts `digest_len() * 2`, preserving the
shipped property that "an entry N characters long is N *valid* characters by
construction". Rows default to sha256, seeded **once** per screen — this tree has
been bitten by a picker that proposed a setting merely by opening on row zero.

**Four of the six entry routes never reach the kind screen**, not two — an
earlier draft undercounted and left one of them unnamed anywhere in the spec:

| # | route | why it bypasses |
| --- | --- | --- |
| 1 | payload `hash:` record | the record carries its kind (§6) |
| 4 | payload `phrase:` record | **carries no hash kind at all** — its `method` field selects the *preimage* derivation (§4, out of scope), which is a different axis that happens to share the word `sha256`. The hashlock is sha256. |
| 5 | payload preimage-plate record | same: no hash kind in the grammar, so sha256. |
| 6 | **the preset archetype** (`--preset` / a composer preset) | the preset supplies the hash; the kind comes from the preset's own grammar (§9 phase 1), not from a screen |

Only the typed-hex and typed-phrase arms ask. Route 6 is the one that must not be
forgotten: it reaches the same lowering by a path with no screen on it at all.

**Route 6 is `md compose --preset` (CLI).** The device's composer presets are a
different thing and have no kind grammar; conflating them was an earlier draft's
error.

**The Back leg is normative and must be stated — for BOTH arms.** The typed-phrase
arm is a loop whose Back contract is specified in `SPEC_hashlock_H2_device.md`
§4.6, and the kind screen is being inserted in front of it.

**The kind screen precedes the material entry on both arms.** That is what keeps
the KDF out of the Back path: nothing is held when the kind is chosen, so Back
from it costs nothing.

| leg | behaviour |
| --- | --- |
| Back from the **kind screen** | → the source pick (`Which hash?`). Nothing is held yet; nothing is discarded. |
| Back from the **hex pad** | → the kind screen, kind still selected. |
| Back from the **phrase screen** | → the kind screen, kind still selected. H2 §4.6's existing leg is *"Back from the phrase screen → `Which hash?` (phrase dropped)"*; with a screen inserted in front, it stops one earlier, and **the phrase is still dropped there** — but the operator does not pay a second KDF to reach the same point, because the kind screen is upstream of the derivation. |

A navigation leg left unstated is how this tree produced a Critical before — the
Back path that skipped a screen also skipped a guard — and the new leg skips a
screen.

**A stated limitation, not an oversight.** §2 decision 5 puts all four kinds on
the phrase route. Routes 4 and 5 are payload-supplied **preimage material** —
route 4 a `phrase:` record, route 5 a preimage-plate record decoding to
`ms_codec::Payload::Preimage`, which is not phrase material at all — and neither
grammar carries a kind, so a hashlock built from either is sha256.
Every kind remains authorable (§2 decision 1) via the typed arms; extending those
two record grammars is deliberately out of scope for this cycle.

### 7.2 §8i splits in two

The rule modal fires on **row selection**, before either arm is entered, so at
that moment no kind exists. The predicate was made row-independent deliberately,
so that adding a band cannot leave the 32-byte rule unstated.

- **Entry body:** kind-generic, still fired by `taking` — but **not unchanged**.
  The shipped body says the hash must be *SHA-256* of a 32-byte value, which is
  false the moment a fourth-kind world exists. An earlier draft said "unchanged,
  kind-generic", and those cannot both hold. It is reworded to state the part
  that is true for all four (**a 32-byte preimage**) without naming a function.
- **Consent body:** names the kind — `<kind>` of a 32-byte value.

### 7.3 The emulator's window

`ComposerPathHashes() []*[32]byte` is the emulator's read access to a running
composition. Left alone it would hand out the **padded raw array**, so a walk
asserting "path 1 holds digest D" would pass identically for `sha256(D)` and for
a 20-byte kind sharing its first 20 bytes — a gate that cannot fail.

**Three layers, not one.** Repairing only the Go hook relocates the defect one
layer out and leaves it there, because both layers above it hardcode 64 hex:

| layer | file |
| --- | --- |
| the hook | `gui/composer_state_hook.go:81` |
| the JS bridge, incl. its documented API contract | `cmd/emu/composer_js.go:15,35-37,54` |
| the walk's own helper | `cmd/emu/walk_hashlock_phrase.js:71,333` |

`hex.EncodeToString(h[:])` over a padded array still returns 64 characters for a
20-byte kind — 40 real and 24 zeros — so the abbreviation the walk compares stays
byte-identical between the two. All three become kind-aware together.

### 7.4 The decode-side change, and what it flips

`PolicyShape.Sha256Digests` records digests only for `tagSha256` while setting
`Hashlock = true` for all four, so a decoded `ripemd160` card already shows
"hashlock" with no digest. The field becomes kind-carrying.

`policy_shape.go` is the **decompose** direction and runs on payloads this device
never composed, so this is a decode-side behaviour change and is declared as one.

**Four production consumers, not two.** An earlier draft said "both dependent
predicates" and cited one of them a line off. All re-key off **`Hashlock`** —
"does this path have a hash of any kind" — rather than `len(Sha256Digests)`:

- `composer_consent.go:94` — the display loop that draws the digests.
- `composer_consent.go:97` — a `hash160`-locked sole unsorted path today prints
  `UNSORTED (EXPERIMENTAL)` and after this change does not. **A regression if
  left implicit; announced and tested here.**
- `composer_consent.go:199` — a decoded `ripemd160` policy today is consented to
  **without** the 32-byte-preimage rule ever stated, and after this change states
  it. A fix.
- **`composer_selfcheck.go:134,136,138` — the compose→decode round trip, and the
  most important of the four.** It is the only place in the system where a
  SAME-WIDTH kind divergence can be caught structurally: a `hash256` digest that
  should have been `sha256`, or the reverse, changes no length and no type, so
  the round trip comparing what was composed against what decodes back is the one
  gate with a chance of noticing. It must compare the **kind** as well as the
  digest, or it certifies the §3-F4 failure as correct.

### 7.5 Row geometry

The kind-bearing row is **measured, not assumed** — and the first measurement was
wrong in a way worth recording: **character count is not the constraint.** The
face is proportional, so at six characters `rmd160` and `sha256` each measure
409 px and draw on one line, while `mmmmmm` (369 px) and `wwwwww` (351 px) wrap to
two. A token budget stated in characters would pass a review and fail on the
device.

The constraint is the rendered **width** of the whole row at the shipped band.

**The kind screen's own four rows are bound by the same arbiter.** They are new
row forms on a new screen, so they get a row in the geometry gate rather than
inheriting an assumption from the screen they were modelled on.

**The wire tokens (§6) stay full and unambiguous.** The *display* token is a
different string, and choosing it is delegated to the implementation plan under a
stated constraint: the row must draw on one line at the shipped band width, with
`TestWhichHashRowsDrawOnOneLine` as arbiter, and the row *form* re-measured as a
whole rather than having a token appended to it. The plan may buy room by
shortening the elision or the row's fixed text instead of abbreviating the token;
what it may not do is assume a token fits.

### 7.6 Payload matching

Every "is this digest in the payload / does it match the held material?"
comparison computes a sha256 digest. Left alone, a correct `ripemd160` payload
draws a **false host-orphan warning** and loses its `(in payload)` annotation.
These comparisons become kind-aware.

## 8. The 20-byte warning

Fires for `ripemd160`/`hash160` when the digest was **not** device-derived:
payload-supplied and typed digests warn; the phrase route stays silent.
Provenance comes from `composerState.hashlockHeld`, populated only by
device-derived routes — no new plumbing.

**Stated residual imprecision, accepted:** that set is per-digest-ever-seen
rather than per-assignment. A digest once derived on-device and later arriving
from a payload suppresses a warning it should show. Re-keying on the full
`HashLock` (§5) narrows this without closing it. The failure is a missing warning
on a digest the operator personally derived earlier in the same composition;
per-assignment provenance is a larger change than the warning is worth.

## 9. Phases

Four repos, and the order is **not** free. An earlier draft claimed phases 1-3
were mutually independent; that was false and is corrected here.

`me-cli` computes every hashlock digest through `ms_codec::hashlock::digest`
(`me-cli/src/main.rs:2636,2641`), and `ms-codec = "0.9"`
(`me-cli/Cargo.toml:53`) resolves today from crates.io. So phase 3 has a real
**code** dependency on phase 2's new API.

**It is not a publish gate, and an earlier draft wrongly said it was.** The
absence of a `[patch]` or path dep is a fact about the file today, not a
constraint on what phase 3 may write in it — and this very `Cargo.toml` already
carries the counter-example twelve lines below, with its rationale written out
(`me-cli/Cargo.toml:54-74`): `mt-codec` is an unpublished sibling consumed **by
git rev pin**, precisely so that *"publishing is irreversible; pinning a rev is
not"*. Phase 3 does the same for `ms-codec`.

Scheduling `cargo publish ms-codec` mid-cycle would put an irreversible act
before anything has been proven on a device, and `me` cannot be published today
anyway while `mt-codec` is a git rev. The publish belongs to whenever `me` is
next released, which is operator-gated and outside this cycle.

**Order:** phases 1 and 2 are independent of each other. Phase 3 follows phase 2
and pins it by rev. Phase 4 follows all three.

**The vendor ritual belongs to phase 2, not phase 3.** mnemonic-engrave has no
vendor tree and no vendor gate; mnemonic-secret has both (a committed `vendor/`,
`ci/repro/vendor-freshness.sh`, and its workflow), and phase 2 is also the phase
that necessarily moves a lockfile — `ms-codec` has no ripemd160 primitive today,
so `ripemd160` and `hash160` add a dependency, whereas phase 3's change is a
*source* swap on an edge that already exists. So: **phase 2 re-vendors
mnemonic-secret; phase 3 carries only the `Cargo.lock` change**, which
`cargo test --locked` and `cargo clippy --locked` already enforce in CI. Worth
stating because the vendor-freshness check is not a required context, so a stale
tree surfaces at tag time rather than at PR time.

| # | repo | what |
| --- | --- | --- |
| 1 | descriptor-mnemonic | `md-codec`: `HashKind`, `HashLock`, lowering arms, **and `presets::hashlock_gated`'s public `[u8; 32]` parameter** (§9.1). `md-cli`: sibling `ripemd160=` / `hash160=` / `hash256=` options on **both** `--path` and `--preset`, plus the `PresetParams` field, the `named_only` allow-list and the `--json` key. Vectors. |
| 2 | mnemonic-secret | `ms-codec`: one digest function per kind, and the §10 per-kind KAT. `ms hashlock` learns `--kind` (§13.4). **The plate's QR text gains `hash: <kind>` (§13.1)** — `qr_text` lives here. |
| 3 | mnemonic-engrave | `me-cli`: the §6 record grammar, both directions. Depends on phase 2's API, consumed **by git rev pin** (above). Vectors. |
| 4 | seedhammer fork | Go ports of 1-3 **and** the device UI, as ONE phase. Also §13's device surface: the locator `hash` row, the confirm and reconciliation bodies, the masked pick lead, and the census row. |

**Phase 4 is one phase on purpose.** Changing `md.SpendPath.Hash` breaks 39
production and 76 test references atomically; there is no tree where the port has
landed and the UI has not. Splitting it would label fork-native authorship as a
convergence port. It is reviewed to the stricter standard.

`md compose`'s option name is a **grammar** decision, not a rename: `sha256=` is
kind-specific, so the three new kinds are siblings.

## 10. Vectors

All net new (§1).

- **Round trip per kind**: compose → md1 → decode → script → address, under
  `wsh` and `sh(wsh)`.
- **Core's measured verdicts and addresses pinned as data**, so a drift in either
  direction fails a row rather than surfacing on a plate. The measured matrix is
  **keyed-shape only**; the keyless shape for the three new kinds is unmeasured
  and must be measured before this closes.
- **A 40-hex digest through every formatter.** The six hardcoded `[56:]` sites
  panic rather than corrupt.

  **Order matters and the obvious order is impossible.** All six take a fixed
  array (`[32]byte` / `[u8; 32]`) and hex it internally, so the string they slice
  is *always* 64 characters: a 40-hex value cannot reach them until the parameter
  widens. A test written first does not go red, it fails to **compile**, which
  takes the whole `gui` package and its ~1200 other tests with it. The order is
  therefore: widen the parameter to a length-aware type (or add a `…Hex(string)`
  seam), then the red test, then the arithmetic.

- **A per-kind digest KAT — `preimage → digest`, all four kinds.** This is the
  only gate that catches a wrong digest *function*, and without it the funds-loss
  path §4 names is caught by nothing: Core's address vectors derive from the
  descriptor **text** and are structurally blind to it, and checking `ms hashlock`
  against the crate that computes the digest is self-consistency, not a KAT.

  Rows are computed **independently of the implementation** (python3 `hashlib`,
  `bitcoin::hashes`, or Core) and vendored under the existing
  `hashlock-v0.8.json` pin, which `hashlock/hashlock_test.go` already
  cross-checks. That file carries one digest column today; this adds three.

  **THE KAT MUST COVER THE DISPATCH, NOT ONLY THE FOUR FUNCTIONS.** §5 puts one
  function per kind in `ms-codec` and leaves each caller to map its own kind onto
  one — a hand-written four-arm switch, outside the crate the KAT tests. Four
  correct functions plus one mis-wired arm gives `OP_RIPEMD160 <hash160(X)>`, and
  the confusables are named by this spec's own §3: two pairs share a width and the
  four function names differ by one word.

  So each row is exercised **through the entry point the composer itself calls**,
  and **each caller's map gets its own row**. A KAT that only calls the four
  functions directly is green on exactly the failure §4 describes.

  **"Caller" means a package, not a call site.** The map is **one named function
  per package** — `DigestFor(kind, x)` or equivalent — and there are **no inline
  kind switches at call sites**. Measured, `hashlock.Digest(` has five production
  call sites in the fork's `gui/`, of which only the two on the phrase arm need a
  kind (the other three are routes 4/5, §7.1). A scattered implementation with a
  switch at each site would satisfy the letter of "its own row" and defeat its
  intent.
- **Fail-closed**: an unknown kind token is `ClassUnknown` and inert.
- **Right kind, wrong length** — `hash:ripemd160:<64hex>`, `hash:sha256:<40hex>`,
  `hash:hash256:<40hex>`, `hash:hash160:<64hex>`. This is the class Core refuses
  most sharply and least helpfully: all four measured REFUSED behind the single
  non-diagnostic `A function is needed within P2WSH` (F3), so the device's own
  length rule is the only thing that can say anything useful.
- **Both spellings of sha256** — `hash:<64hex>` and the accepted-but-never-emitted
  `hash:sha256:<64hex>` (§6), which must parse to the same `HashLock`.
- **Cross-repo token agreement**, pinned by `record_class_vectors.provenance.json`
  and `compose_vectors.provenance.json`.
- **The preimage plate, per kind**: its QR text, its locator row, and a **layout
  fit** assertion at the worst case (100-character phrase) — §13.1's placement is
  a measurement and decays like one. Goldens for the plate bytes.

## 11. Gates that must move deliberately

| gate | file:line |
| --- | --- |
| the copy table's declared-body count | `gui/composer_copy_test.go:371` (78) |
| §8 spec rows for every new string | `SPEC_wallet_policy_composer.md` |
| the modal fit gate, incl. the new screens | `gui/modal_fits_test.go` |
| `TestWhichHashRowsDrawOnOneLine` | `gui/composer_hash_test.go:239` |
| the literal `"Type 64 hex"`, in code **and** test | `gui/composer_hash.go:348`, `composer_hash_test.go:257` |
| `TestWhichHashPageHoldsFiveRows` | `gui/composer_hashlock_test.go:1433` |
| `composerHexEntry`'s three 64/32 constants | `gui/composer_hash.go:79-105` |
| `hashlockFirst8Last8` — **invisible to the `[56:]` grep** the six-count came from; its arithmetic is already length-relative but its PARAMETER is `[32]byte` | `gui/composer_hashlock.go:247` |
| five sha256-hardcoded operator-facing strings in `me-cli` | `main.rs:2275,2685,3196`, `sysw/composer_records.rs:144` |
| the JS bridge and its documented API contract | `cmd/emu/composer_js.go:15,35-37,54` |
| the walk's 64-hex helper | `cmd/emu/walk_hashlock_phrase.js:71,333` |
| the compose→decode self-check | `gui/composer_selfcheck.go:134,136,138` |
| the cross-language digest KAT | `hashlock/testdata/hashlock-v0.8.json` + its ms-codec source |
| the confirm modal and the reconciliation body | `gui/composer_copy.go:557-571`, `:641-648` |
| the hex pad's LIVE COUNTER — 30 lines past the range an earlier draft cited | `gui/composer_hash.go:135` |
| `ms hashlock`'s operator surface, incl. `for md compose: … sha256=<h>` | mnemonic-secret, `ms hashlock` |
| the preimage plate's QR text and locator row | `MethodLine`, `ms_codec::hashlock::qr_text`, `backup/hashlock` |
| the preimage plate **fit gate and its goldens** — §13.1's placement is a layout claim and must be measured, not asserted | the fork's plate layout tests |
| the masked plate-pick lead, which prints a bare `method:` | `gui/composer_copy.go:717-725` |
| `me-cli`'s hashlock help text | `me-cli/src/main.rs:195,197` |
| the §8.3 census row | the composer's plate inventory |
| `me bundle`'s plate checklist | `me-cli` |
| both provenance pin files | — |

One of `composerHexEntry`'s constants is **documented as unreachable** — the
belt-and-braces `len(raw) != 32`, unreachable only because the pad's own cap and
the `len(frag) == 64` check make it so. Widening the pad to accept 40 characters
removes that guarantee, so the comment must be re-derived rather than carried.

`composer_records.rs:144` is pinned verbatim by `SPEC_wallet_policy_composer.md`
§8n **and** by `host_line` rows in `record_class_vectors.json`, so editing that
string is re-pin work in two places. The compose-vector pin generator also prints a stale count in its own
`_comment` (156 against 161). Both the generator
(`seedhammer/scripts/vendor-compose-vectors.sh:29`) and the file it writes
(`seedhammer/md/testdata/compose_vectors.provenance.json:6`) live in the **fork**,
and the re-pin is run from there — so this is a **phase 4** item, not phase 1 as
an earlier draft said. It is fixed in this cycle rather than filed, because the
re-pin touches it anyway and a generator that prints a wrong count is how the
next wrong count goes unnoticed.

`composerPickScreenMaxRows = 24` is **not** on this list: the separate-screen
choice means `Which hash?` gains no band.

## 12. Acceptance

1. Each kind composes end to end and its address matches Core's measured value.
2. An **emulator walk** composes a non-sha256 hashlock on the device **and
   asserts, through the kind-aware hook, that the composition stores that kind
   and the §10 KAT row's digest for that kind**. Naming the KAT row is the point:
   "the digest it composed" is an expectation the device supplies to itself, and
   `cmd/emu/walk_hashlock_phrase.js:339-345` already argues this exact
   distinction for the sha256 case — *"Comparing short8(stored) against a
   constant this file also compares the stored value against is a tautology."*

   The assertion is the acceptance, not the composing: a walk
   that composes one and never asserts the kind satisfies the sentence and gates
   nothing. This repo's rule is that a plan may not close while one of its own
   gates has never run, and its corollary is that a gate which cannot fail is not
   a gate.
3. A 40-hex digest passes every formatter without panicking.
4. An old parser meeting a new kind token fails closed (§6).
5. `ms hashlock` reproduces a non-sha256 plate's digest. **Not a KAT** — it and
   the digest function are the same crate, so it checks self-consistency. Item 6
   is the gate.
6. **The per-kind digest KAT passes in both languages**, against rows computed
   outside either implementation.
7. **A `hash256` wallet survives its own reconciliation screen** (§13.2): compose
   one, follow the screen's instruction literally, and the check passes. This is
   the cycle's operator-facing Critical and the pair with no width, no truncation
   and no visual distance, so it gets an acceptance item of its own.
8. **A preimage plate cut for a non-sha256 kind names that kind** (§13.1), and
   its QR still measures 53 modules — the reserved envelope, so **the H6
   acceptance's own item 8** (the operator's QR-scan gate, ~43 min) is
   unaffected and need not be re-cut for this cycle.
9. **A written record made by following §13.2's list is sufficient to rebuild the
   descriptor** — the test is `md compose` with nothing but what the operator was
   told to write down.

## 13. The engraved surface and the operator's record

Two lenses that ran after the correctness gate closed found this surface
untouched: seven plate- and restore-related identifiers appear nowhere in the
earlier drafts, and the word "restore" appeared once, in a background clause.
Nothing here is optional.

**13.1 The preimage plate names the kind — in the QR, and on the locator's
`hash` row.** The plate's `method:` line and its `hashlock v1` QR are, by the
plate's own written design rule, the complete definition of the derivation, and a
fourth-kind world makes them one step short — while the device instructs the
operator to store that plate *apart from* the md1 card holding the missing step.

**The QR is free. The engraved text is free only at one placement**, and an
earlier draft said "no layout change" on the strength of the QR measurement
alone. Measured against the fork's own layout, worst case (100-character phrase):

| placement | result |
| --- | --- |
| baseline (no kind) | fits at 3.0 mm, 10 rows, 1.20 mm spare |
| QR text gains `hash: <kind>` | 194 → **210 bytes, still 53 modules**, fits, 1.20 mm spare |
| kind appended to the **`method:` line** (88 chars) | **REFUSES at every rung** — 11 rows at 3.0 mm need 427520 units against a 416000 budget |
| kind as **its own body row** | **REFUSES at every rung**, same arithmetic |
| kind appended to the **locator's `hash` row** (35 chars) | fits at 3.0 mm, 10 rows, 1.20 mm spare |

So the kind goes in the **QR text** and on the **locator's `hash` row**, and
explicitly **not** on the `method:` line — which H6 §6.5 had already pinned at 73
characters and ruled cannot carry more — and **not** as a new row, because either
adds an eleventh row and blows the budget at every font rung.

ACCEPTANCE item 8 of the H6 cycle (the operator's QR-scan gate, ~43 min) is
unaffected: the QR stays at 53 modules, the reserved envelope.

**13.2 The confirm and reconciliation screens name the kind.** This is the
cycle's operator-facing Critical. The reconciliation screen instructs: *"run
`ms hashlock` … if they differ, do not fund this wallet: build it again."* On a
correct `hash256` wallet that check **fails**, because the screen supplies only
`method: sha256` — the other axis (§5) — and `ms hashlock` without a kind returns
the sha256 digest. Both are 64 hex; nothing but a label distinguishes them, and
the label was never printed. The operator, complying exactly, discards a correct
wallet and re-cuts five plates.

**Three screens, not two.** The masked plate-pick lead
(`gui/composer_copy.go:717-725`) prints `method: <method>` and no kind, which §5's
rule condemns as directly as the other two; the fold that wrote that rule missed
the screen it condemned. It is in scope here.

So: all three bodies name the kind, and the reconcile sentence names the flag —
`ms hashlock … --kind <kind>`. Its write-down list gains the kind too; an
operator who writes down exactly what they are told must be able to rebuild the
descriptor, and `md compose` needs one of four option names that list does not
currently contain. A write-down list that omits a field is worse than no list,
because the operator stops writing where the list stops.

**13.3 The census row distinguishes the kinds.** It is the operator's only
inventory of what they must store apart, and today it describes all four
constructions identically — two plates for one preimage under two kinds differ by
sixteen hex characters and nothing else.

**13.4 `ms hashlock` can be given a kind, and says what it does without one.**
The plate-verification route of ACCEPTANCE item 1 needs a kind the plate does not
carry today (13.1 fixes that). For plates already cut, printing all four digests
when no kind is given turns an impossible check into a lookup. This spec
**requires the `--kind` flag** and **permits** the four-digest fallback; it must
not silently assume sha256.

**13.5 Two CLI gaps, named rather than fixed here.** `me bundle` emits
byte-identical six-plate output for a sha256 card and a `ripemd160` card, names
no preimage plate, and cannot cut one at all; and the composer emits **no restore
document** (`restoreDoc` occurs zero times in `gui/composer*.go`), which widens
F-132's open half. Both are recorded as follow-ups owned by this cycle's plan,
not as silent scope.

## 14. What would falsify this

- **If a later Core gains tapscript miniscript**, F2's uniform wrapper rules stop
  holding and the `tr` row needs its own answer.
- **If `OP_SIZE <32>` ever stops holding for any fragment**, F1 dies and with it
  the scope boundary that keeps H6 out of this cycle.
- `golang.org/x/crypto/ripemd160` is **deprecated** upstream, though present and
  free today. If it is removed, the device needs another source for the bare
  primitive (`hash160` would survive via `address.Hash160`).
