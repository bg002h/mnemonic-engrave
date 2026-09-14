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

- The H6 preimage plates and the phrase→**preimage** derivation, by F1. The
  preimage stays `sha256(phrase)`, 32 bytes, for every kind.

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
whitespace seam rows (§4.6).

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
| 4 | payload `phrase:` record | the record carries its kind |
| 5 | payload preimage-plate record | the record carries its kind |
| 6 | **the preset archetype** (`--preset` / a composer preset) | the preset supplies the hash; the kind comes from the preset's own grammar (§9 phase 1), not from a screen |

Only the typed-hex and typed-phrase arms ask. Route 6 is the one that must not be
forgotten: it reaches the same lowering by a path with no screen on it at all.

### 7.2 §8i splits in two

The rule modal fires on **row selection**, before either arm is entered, so at
that moment no kind exists. The predicate was made row-independent deliberately,
so that adding a band cannot leave the 32-byte rule unstated.

- **Entry body:** unchanged, kind-generic, still fired by `taking`.
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
| the JS bridge, incl. its documented API contract | `cmd/emu/composer_js.go:15,35-37,53` |
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
(`me-cli/Cargo.toml:53`) resolves **from crates.io** with a lockfile checksum —
no `[patch]`, no path dependency, no vendor directory. So the record work cannot
compile against the new per-kind API until ms-codec is **released** and the
dependency bumped. That is a publish gate, and it is named here rather than
discovered mid-cycle.

**Order:** phases 1 and 2 are genuinely independent of each other. Phase 3
follows phase 2 across a crates.io release, carrying the `Cargo.lock` /
`cargo vendor` freshness ritual with it. Phase 4 follows all three.

| # | repo | what |
| --- | --- | --- |
| 1 | descriptor-mnemonic | `md-codec`: `HashKind`, `HashLock`, lowering arms, **and `presets::hashlock_gated`'s public `[u8; 32]` parameter** (§9.1). `md-cli`: sibling `ripemd160=` / `hash160=` / `hash256=` options on **both** `--path` and `--preset`, plus the `PresetParams` field, the `named_only` allow-list and the `--json` key. Vectors. |
| 2 | mnemonic-secret | `ms-codec`: one digest function per kind, and the §10 per-kind KAT. `ms hashlock` learns the kind. |
| 3 | mnemonic-engrave | `me-cli`: the §6 record grammar, both directions. **Requires a released ms-codec** (above). Vectors. |
| 4 | seedhammer fork | Go ports of 1-3 **and** the device UI, as ONE phase. |

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
- **Fail-closed**: an unknown kind token is `ClassUnknown` and inert.
- **Cross-repo token agreement**, pinned by `record_class_vectors.provenance.json`
  and `compose_vectors.provenance.json`.

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
| the JS bridge and its documented API contract | `cmd/emu/composer_js.go:15,35-37,53` |
| the walk's 64-hex helper | `cmd/emu/walk_hashlock_phrase.js:71,333` |
| the compose→decode self-check | `gui/composer_selfcheck.go:134,136,138` |
| the cross-language digest KAT | `hashlock/testdata/hashlock-v0.8.json` + its ms-codec source |
| both provenance pin files | — |

One of `composerHexEntry`'s constants is **documented as unreachable**; that stops
being true. The compose-vector pin generator also prints a stale count in its own
`_comment` (156 against 161). It is fixed in this cycle, in phase 1, as part of
re-pinning that file — not filed, because the re-pin touches it anyway and a
generator that prints a wrong count is how the next wrong count goes unnoticed.

`composerPickScreenMaxRows = 24` is **not** on this list: the separate-screen
choice means `Which hash?` gains no band.

## 12. Acceptance

1. Each kind composes end to end and its address matches Core's measured value.
2. An **emulator walk** composes a non-sha256 hashlock on the device **and
   asserts, through the kind-aware hook, that the composition stores that kind
   and that digest**. The assertion is the acceptance, not the composing: a walk
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

## 13. What would falsify this

- **If a later Core gains tapscript miniscript**, F2's uniform wrapper rules stop
  holding and the `tr` row needs its own answer.
- **If `OP_SIZE <32>` ever stops holding for any fragment**, F1 dies and with it
  the scope boundary that keeps H6 out of this cycle.
- `golang.org/x/crypto/ripemd160` is **deprecated** upstream, though present and
  free today. If it is removed, the device needs another source for the bare
  primitive (`hash160` would survive via `address.Hash160`).
