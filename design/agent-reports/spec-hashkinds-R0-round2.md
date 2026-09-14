# R0 ARCHITECT REVIEW — SPEC_hashlock_kinds.md, round 2 (the fold)

Artifact: the fold `git diff 60647cf9..1aa31c30` on `design/SPEC_hashlock_kinds.md`
(mnemonic-engrave `1aa31c30`, spec now 411 lines).
Round 1 report: `design/agent-reports/spec-hashkinds-R0-round1.md` at `60647cf9`
(1 Critical / 6 Important / 7 Minor / 2 Nit).

Trees measured — identical to round 1's, so nothing below is drift:
descriptor-mnemonic `40c400de`, mnemonic-engrave `1aa31c30`,
mnemonic-secret `7a0e96f`, seedhammer `0562e811`.
`git status --porcelain` verified clean in all four, before and after.
No tracked file modified but this report. No probe file was created this round.

Two questions only, per the brief: **did the fold close each round-1 finding**, and
**did the fold introduce a new defect**. This is not a fresh audit; round 1's
VERIFIED SOUND list, §2's operator decisions and the recon reports were not
re-derived.

---

## 1. Citations the fold ADDED or CHANGED — every one checked

| # | § | citation | verdict | measured |
|---|---|---|---|---|
| 1 | §9 | `me-cli/src/main.rs:2636` | **TRUE** | `Some(ms_codec::hashlock::digest(&x))` |
| 2 | §9 | `me-cli/src/main.rs:2641` | **TRUE** | `Ok((_, ms_codec::Payload::Preimage(x))) => Some(ms_codec::hashlock::digest(&x)),` |
| 3 | §9 | `me-cli/Cargo.toml:53` = `ms-codec = "0.9"` | **TRUE** | exact |
| 4 | §9 | ms-codec resolves from crates.io, no `[patch]`, no path dep, no vendor dir | **TRUE** | `Cargo.lock`: `source = "registry+https://…crates.io-index"`, `checksum = 6a7836a8…`; no `[patch]` anywhere; no `vendor/`; no `.cargo/config.toml` |
| 5 | §7.4 | `composer_consent.go:94` = the display loop | **TRUE** | `for _, d := range b.Sha256Digests {` |
| 6 | §7.4 | `composer_consent.go:97` = the UNSORTED predicate | **TRUE** | `if sole && !b.Sorted && b.N >= 2 && len(b.Locks) == 0 && len(b.Sha256Digests) == 0 {` — round 1's M-1 fix lands exactly |
| 7 | §7.4 | `composer_consent.go:199` = the §8i restatement predicate | **TRUE** | `if len(b.Sha256Digests) > 0 {` |
| 8 | §7.4, §11 | `composer_selfcheck.go:134,136,138` | **TRUE** | `:134` `if len(b.Sha256Digests) != wantHash {`, `:136` the count in the error args, `:138` `if p.Hash != nil && b.Sha256Digests[0] != *p.Hash {` |
| 9 | §7.3, §11 | `cmd/emu/composer_js.go:15` | **TRUE** | `//	shComposerPathHashes()   [ "<64 hex>" | null, ... ]` |
| 10 | §7.3, §11 | `cmd/emu/composer_js.go:35-37` | **TRUE** | `// FULL 64 HEX, not the first8..last8 the screens draw…` runs `:35-37` exactly |
| 11 | §7.3, §11 | `cmd/emu/composer_js.go:53` | **FALSE** | `:53` is `}` (the close of `if h == nil`). `hex.EncodeToString(h[:])` is at **`:54`**. Inherited verbatim from round 1's own evidence block, and now in the spec **twice**. See **M-2** |
| 12 | §7.3, §11 | `cmd/emu/walk_hashlock_phrase.js:71` | **TRUE** | `//   * after the hold, it is the corpus's FULL 64-hex hardened digest, and its` |
| 13 | §7.3, §11 | `cmd/emu/walk_hashlock_phrase.js:333` | **TRUE** | `/** first8..last8 of a 64-hex digest -- the abbreviation gui.hashlockFirst8Last8 draws. */` |
| 14 | §7.3 | `gui/composer_state_hook.go:81` | **TRUE** | `func ComposerPathHashes() []*[32]byte {` |
| 15 | §10, §11 | `hashlock-v0.8.json` + `hashlock/hashlock_test.go` already cross-checks it | **TRUE** | fork `hashlock/testdata/hashlock-v0.8.json` (15,720 B) and `hashlock/hashlock_test.go:194` `TestKindRowPreimageDigest`, `:200` `if got, want := Digest(&x), mustHex(t, c.Kind[0].Digest)` — with a recorded mutation test at `:189-193` |
| 16 | §10 | "That file carries one digest column today; this adds three" | **TRUE** | the `kind` array holds one row; its only digest field is `"digest"` |
| 17 | §11 | ms-codec source of the KAT | **TRUE** | `mnemonic-secret/crates/ms-codec/tests/vectors/hashlock-v0.8.json`, byte-identical size; provenance pin `hashlock-v0.8.provenance.json` names repo/path/commit `ffdb77d4…`/sha256 |
| 18 | §9 tbl | `presets::hashlock_gated` takes a public `[u8; 32]` | **TRUE** | `md-codec/src/compose/presets.rs:88-92` `pub fn hashlock_gated(wrapper: Wrapper, hash: [u8; 32], older_blocks: u32)` |
| 19 | §9 tbl | `PresetParams` field, `named_only` allow-list, `--json` key | **TRUE** | `md-cli/src/cmd/compose.rs:187` `sha256: [u8; 32],`, `:369` `named_only(&["sha256", "older"])?;`, `:444` `json!({ "sha256": hex32(&sha256), … })` |
| 20 | §10 | "All six take a fixed array … and hex it internally" | **TRUE** | the six are `composer_hash.go:50,173,188`, `composer_consent.go:63`, `me-cli/main.rs:2277,2602`; enclosing signatures `composerHashRow(i int, digest [32]byte)`, `composerHashPreimageRow(i int, digest [32]byte)`, `composerHashPhraseRow(i int, d *[32]byte)`, `composerDigestShort(d [32]byte)`; Rust both slice `hx = hex(&digest)` over `[u8; 32]` |
| 21 | §10 | "~1200 other tests" in `gui` | **understated** | `grep -h "^func Test" gui/*_test.go \| wc -l` = **1319**, `TestMain` 0. See **N-1** |
| 22 | §7.5 | `rmd160`/`sha256` 409 px, `mmmmmm` 369 px, `wwwwww` 351 px | **TRUE as transcription** | byte-exact against round 1's measured M-2 table. Not re-measured (round 1 probed it; the figures are round 1's, and re-deriving them is out of this round's scope). But the fold dropped the **411 px band** and round 1's post-wrap caveat — see **M-3** |
| 23 | §4 | `(§9 phases 2 and 4)` after the renumbering | **TRUE** | the preimage→digest function is `ms-codec/src/hashlock.rs:59` (mnemonic-secret = new phase 2) and `hashlock/hashlock.go:86` (fork = phase 4) |
| 24 | §7.1 | route 6 `(§9 phase 1)` | **HALF TRUE** | `md-cli --preset` is phase 1 ✓; the fork's `gui/composer_presets.go:54` `composerPresetDigest()` has no grammar at all and is phase 4. See **M-5** |
| 25 | §9 tbl | `(§9.1)` | **DANGLING** | the spec has no §9.1 — §9 "Phases" has no subsections (headings verified: 1,2,3,4,5,6,7,7.1–7.6,8,9,10,11,12,13). See **M-1** |
| 26 | §11 | the 156/161 stale count "is fixed … in phase 1" | **FALSE, unchanged** | `seedhammer/scripts/vendor-compose-vectors.sh:29` says 156; `seedhammer/md/compose_vectors_pin_test.go:91` asserts 161. Both in the fork = phase 4. Round 1's M-3, untouched |
| 27 | §7.1 | routes 4 and 5 — "the record carries its kind" | **FALSE** | `phrase:` carries a **method** (`hardened`/`sha256`), not a kind — `composer_records.rs:60,75,97`; the preimage-plate record is an ms1 string whose only "kind" is the payload kind byte. Neither carries a miniscript hash kind, and §6 gives one only to `hash:`. See **I-1** |

Citations checked this round: **27**. TRUE **22**, false **3** (`composer_js.go:53`,
§7.1 routes 4/5, §11's "phase 1" — the last inherited), dangling **1**,
understated **1**.

---

## 2. Round-1 findings — CLOSED / PARTIAL / NOT CLOSED

### C-1 — no gate pins `preimage → digest` per kind → **PARTIAL**

**Closed:** §3 gains the `hash256`/`sha256d` no-signal paragraph (`:77-83`), §10
gains the per-kind KAT bullet (`:345-354`) with rows computed outside either
implementation and vendored under the existing pin, §12 gains item 6 and demotes
item 5 to self-consistency in words (`:397-401`). The mechanism the fold leans on
is real and verified: `hashlock/hashlock_test.go:194-205` already cross-checks
`Digest(&x)` against `kind[0].digest`, and the Rust side does the same at
`ms-codec/tests/hashlock_derivation.rs:134`. I confirmed the corpus's one digest
column is independently reproducible — `python3 hashlib.sha256(b'\xab'*32)` =
`9a2db2e23f1504cd056606553ac049c5e718e8f9ce9233876df1a7a1821af885`, which is the
file's constant byte for byte.

**Not closed:** the KAT pins the **functions**; nothing pins the **selection**.
See **C-2** below — a wrong digest *function* is now caught and a correct function
*invoked for the wrong kind* is not, and that is the same funds-loss path with the
same all-green acceptance sheet.

### I-1 — "phases 1-3 mutually independent" is false → **CLOSED** (with a stale-prose residue)

§9 is rewritten (`:294-314`): the claim is retracted by name, the ms-codec edge is
stated with its citations (all verified above), phases renumbered so mnemonic-secret
is 2 and mnemonic-engrave is 3, order stated as 1∥2 → 3 → 4 with the `cargo vendor`
ritual attached. That is precisely round 1's prescribed remedy.

Two things ride along: the *mandatory publish* inference is unsound (**I-2**), and
§5's rationale for rejecting a shared `HashKind` is now contradicted by §9 and was
not updated (**M-4**).

### I-2 — the 40-hex test ordering is unsatisfiable → **CLOSED**

§10 `:334-343` states the order explicitly: widen the parameter or add a
`…Hex(string)` seam, then the red test, then the arithmetic; and says why (a
compile failure, not a red test, taking the whole `gui` package). The "before the
type changes" clause is gone. The supporting claim "all six take a fixed array" is
verified TRUE (row 20).

### I-3 — hook repaired, two layers above it not → **CLOSED**

§7.3 `:211-222` names all three layers in a table; §11 `:370-371` gains both
`cmd/emu` rows; §12.2 `:388-394` now requires the walk to assert the kind and says
in terms that composing without asserting gates nothing. One line number in the
new table is off by one (row 11 / **M-2**).

### I-4 — C-1 folded for `--path` only → **CLOSED on scope**

§9's phase-1 row `:311` names `--preset`, the `PresetParams` field, the
`named_only` allow-list, the `--json` key **and** `presets::hashlock_gated`'s
public `[u8; 32]` parameter. All five citations verified TRUE (rows 18, 19). The
`(§9.1)` pointer beside the signature item leads nowhere (**M-1**), so the *rule*
for the public API break is named as in-scope work but not stated anywhere; under
§5's `HashLock` the intended shape is inferable, so this is Minor, not a re-open.

### I-5 — four of six routes bypass; route 6 unnamed → **PARTIAL**

§7.1 `:182-193` replaces the sentence with a four-row table, names the preset
archetype as route 6, and the route numbering matches the device-surface recon's
(1 payload `hash:`, 4 payload `phrase:`, 5 preimage plate, 6 preset; 2 and 3 are
the typed arms that ask). But:

- rows 4 and 5 now assert something **false** — see **I-1** below, which is the
  larger half of this and a new defect;
- round 1's second clause — *"giving `composerPresetDigest` / `presets::hashlock_gated`
  an explicit kind … as a stated decision rather than an implementer's default"* —
  is not folded. See **M-5**.

### I-6 — `composer_selfcheck.go` missed → **CLOSED**

§7.4 `:233-250` now enumerates four consumers, all citations verified, and gives
the self-check its own rule in the terms round 1 asked for: it must compare the
**kind** as well as the digest, or it certifies the F4 failure as correct. The
count assertion and the value comparison are both preserved in the prose.

### The five false citations → **4 of 5 fixed, 1 new introduced**

| round-1 defect | fold | verdict |
|---|---|---|
| `composer_consent.go:96` → `:97` | done | **fixed**, verified |
| "both predicates" → four consumers | done | **fixed**, verified |
| "two of six routes" → four | done | **fixed** — but the new table carries a new false claim (**I-1**) |
| "phases 1-3 independent" | retracted | **fixed** |
| "≤6-character token" → a width constraint | done | **fixed** — but the presentation is now self-contradicting (**M-3**) |
| — | — | **new:** `cmd/emu/composer_js.go:53` should be `:54`, in two places (**M-2**) |

### Round-1 Minors and Nits

| # | subject | status |
|---|---|---|
| M-1 | `:96`/`:198` off by one | **CLOSED** (`:97`, `:199`, verified) |
| M-2 | "≤6-character token" is not the constraint | **CLOSED on substance**, new presentation defect **M-3** |
| M-3 | 156/161 fix assigned to phase 1, file is in the fork | **NOT CLOSED**, no reason stated. `:378` still reads "in phase 1"; the two files are `seedhammer/scripts/vendor-compose-vectors.sh:29` and `seedhammer/md/compose_vectors_pin_test.go:91`, both phase 4 |
| M-4 | fifth Go elision site + five sha256-hardcoded me-cli strings | **NOT CLOSED**, no reason stated. `gui/composer_hashlock.go:247 func hashlockFirst8Last8(h [32]byte)` is still absent from the spec, and §10's bullet still says "every formatter" over an enumeration of six that does not include it |
| M-5 | right-kind-wrong-length and both-spelling vector rows | **NOT CLOSED**, no reason stated |
| M-6 | §6's unqualified "(§4.6)" | **NOT CLOSED**, no reason stated (`:163` unchanged) — and the fold added a second dangling reference of the same class (**M-1**) |
| M-7 | kind-token case stated as description, not rule | **NOT CLOSED**, no reason stated (`:146` unchanged) |
| N-1 | §11's "that stops being true" is asserted | **NOT CLOSED**, no reason stated (`:376` unchanged) |
| N-2 | two different things are called a "hash kind" | **NOT CLOSED**, no reason stated — and the fold's new §7.1 rows 4/5 walk straight into it (**I-1**) |

Seven of nine Minors/Nits carried with no stated reason. None gates; they are
recorded here so the next fold has one list.

---

# CRITICAL

## C-2 — the KAT pins the four digest functions and nothing pins the kind→function **selection**, so §4's funds-loss path still passes every acceptance item

**Spec text at fault.** §3 `:82-83`: *"§10's per-kind digest KAT is the only thing
that catches it."* §10 `:345-346`: *"A per-kind digest KAT — `preimage → digest`,
all four kinds. This is the only gate that catches a wrong digest function."*
§12.2 `:388-391`: *"asserts, through the kind-aware hook, that the composition
stores **that kind and that digest**."* §12.6. §5 `:132-136`.

**What the fold actually bought.** A KAT over `hashlock-v0.8.json`'s `kind[0]` row
with four digest columns proves that `DigestSha256`, `DigestHash256`,
`DigestRipemd160` and `DigestHash160` each compute what they claim. Verified that
this mechanism exists and bites: `hashlock/hashlock_test.go:189-193` records the
mutation (double-hash inside `Digest`) and the failure it produces.

**What it does not reach.** §5 `:135-136` fixes the shape of the API: *"`ms-codec`
exposes **one digest function per kind** and each caller **maps its own local kind
onto one**."* So the mapping — a hand-written four-arm switch — lives in every
caller, outside the crate the KAT tests. The confusable pairs are named by the
spec itself (F4: `ripemd160`/`hash160` share a width; the fold's new paragraph:
`sha256`/`hash256` share a width), the four function names differ by one word, and
the same package already exports `PreimageSHA256` beside a future `DigestSHA256`.

**The constructed failure.** In `gui/`, the phrase arm's switch has one arm wrong:

```go
case md.HashKindRipemd160:
    d = hashlock.DigestHash160(&x)      // correct function, wrong kind
```

Now walk §12 item by item, measured:

- **§12.6, the new KAT** — green. All four functions are correct; the KAT never
  calls the switch.
- **§12.1 / §10 bullet 1** — green. Core derives the address from the descriptor
  **text** (`spec-hashkinds-core-verdicts.md:87,184-194`), and the round trip
  starts from a digest — the preimage never enters it. Both structurally blind,
  exactly as the fold's own §10 now says.
- **§7.4's self-check** (`composer_selfcheck.go:134-140`) — green. It compares the
  composed `HashLock` against the decoded one. Both carry `kind = ripemd160` and
  the same 20 bytes; the round trip is self-consistent.
- **§12.5** — green, and already demoted.
- **§12.3, §12.4** — untouched by this.
- **§12.2, the emulator walk** — green **or** red, and the spec does not say
  which. "Stores that kind and that digest" names no source of truth for *that
  digest*. Read as "the digest it composed", the assertion reads the hook and
  compares it to what the device just produced: a tautology, and this repo's
  fourth gate-that-cannot-fail this session. Read as "the KAT row for the chosen
  kind", it fires. `walk_hashlock_phrase.js:339-345` argues this exact distinction
  at length for the sha256 case — *"Comparing short8(stored) against a constant
  this file also compares the stored value against is a tautology"* — and the
  spec's new sentence does not carry that requirement forward.

Result: `OP_RIPEMD160 <hash160(X)>` on a plate, every screen reporting the path as
held, every acceptance item green, and a path whose preimage does not exist. That
is verbatim the failure §4 `:103-106` names and the failure round 1 called
Critical; the fold moved it one level inside the remedy rather than closing it.

The same hole exists on `ms hashlock` (§2 decision 5 gives it a kind, so it gains
the same switch; §12.5 is explicitly *not* a gate and §12.6 tests the crate
functions, not the CLI's selection). `me-cli`'s two sites are lower stakes — a
wrong arm there produces a false orphan warning (§7.6), not an unspendable path.

**Close it by** two clauses, no rework:

1. §10 — the KAT covers the **dispatch**, not only the four functions: one row per
   kind exercised through whatever entry point the composer itself calls, so that
   a mis-wired arm fails the vector. (If §5's "one function per kind, mapped by
   each caller" stands, then each caller's map needs its own row.)
2. §12.2 — name the source of truth: *"…asserts that the composition stores that
   kind and the **§10 KAT row's digest for that kind**."* Without a named external
   expectation the item is satisfiable by a walk that cannot fail, which §12.2's
   own last sentence says is not a gate.

---

# IMPORTANT

## I-1 — §7.1's new table says routes 4 and 5 "carry their kind"; neither record carries one, and the fold deleted the only sentence that said what they do

**Spec text at fault.** §7.1 `:185-190`:

| # | route | why it bypasses |
| --- | --- | --- |
| 4 | payload `phrase:` record | the record carries its kind |
| 5 | payload preimage-plate record | the record carries its kind |

The text this replaced said: *"**Two of the six entry routes cannot reach the kind
screen** (payload-`phrase:` and payload-preimage-plate records); **they are sha256
by their record's kind.**"* The count was wrong — that is what round 1's I-5
fixed — but the **answer** was there, and the fold removed it.

**Measured — neither record carries a hash kind.**

```
me-cli/src/sysw/composer_records.rs:40   pub const PHRASE_PREFIX: &str = "phrase:";
                                    :33  /// `phrase:<hex of "<method>,<phrase>">` — a hashlock PHRASE and its method
                                    :60  /// The method SELECTOR a `phrase:` record carries.
                                    :97  pub method: HashlockMethod,
                                    :85  pub fn preimage(self, phrase: &[u8]) -> Zeroizing<[u8;32]> {
                                    :87      HashlockMethod::Hardened => ms_codec::hashlock::preimage_hardened(phrase),
                                    :88      HashlockMethod::Sha256   => ms_codec::hashlock::preimage_sha256(phrase),
```

The `phrase:` record carries a **method**, and the method selects the
**preimage** derivation — the step §4 `:96-97` puts explicitly *out of scope*. Its
two values are `hardened` and **`sha256`**. The preimage-plate record (route 5) is
an ms1 string; `me-cli/src/main.rs:2640-2641` decodes it to
`ms_codec::Payload::Preimage(x)` and its only "kind" is the ms1 payload kind byte
— the *other* thing this tree calls a hash kind, which is round 1's N-2 and is
still unfolded. §6 `:143` gives a kind token to `hash:` and to nothing else.

**The concrete failure.** An implementer building route 4 reads "the record
carries its kind", goes to §6 for the `phrase:` grammar, finds none, and has three
defensible readings and no ruling:

1. **default sha256** — the deleted answer, and the shipped behaviour;
2. **read the record's `method` as the kind** — it is a selector, spelled
   `hardened`/`sha256`, sitting on the same record, and one of its two values is
   also a `HashKind` value. This reading is wrong at the semantic layer §3 F4
   exists to protect, and it is the confusion round 1's N-2 warned would happen;
3. **extend §6's grammar to `phrase:`** — which §6's producer rule and the
   byte-identical-payload guarantee (`:148-150`) do not cover, and which §4 keeps
   out of scope.

And the hole is not cosmetic, because §2 decision 5 `:47` says *"**The phrase
route derives all four**"* — which route 4 cannot do with no kind anywhere in its
record and no screen on its path. The spec now asserts a capability for a route
whose kind it declines to specify.

**Close it by** restoring the answer inside the new table — *"routes 4 and 5 carry
no kind and are sha256; the `phrase:` record's `method` is the **preimage**
selector (§4, out), not a hash kind"* — which also discharges N-2 for the one
place it actually bites.

## I-2 — §9's release gate is an unsound inference, and the same `Cargo.toml` carries the counter-example twelve lines above the citation

**Spec text at fault.** §9 `:297-307`: *"`ms-codec = "0.9"` (`me-cli/Cargo.toml:53`)
resolves **from crates.io** with a lockfile checksum — no `[patch]`, no path
dependency, no vendor directory. **So the record work cannot compile against the
new per-kind API until ms-codec is released and the dependency bumped.** That is a
publish gate… Phase 3 follows phase 2 **across a crates.io release**."*

The measurement is exactly right (verified, row 4). The **inference** is not: the
absence of a `[patch]`, a path or a vendor dir is a fact about the file today, not
a constraint on what phase 3 may write in it. A dependency line can be changed.

**Measured — this crate already does exactly that, for exactly this situation:**

```
me-cli/Cargo.toml:54-74
  # The mt1 wire format …
  # A GIT dependency pinned to a rev, not a path and not a publish.
  # A git dep works TODAY because the repo is public, needs no deploy key, and --
  # the point -- keeps `cargo publish mt-codec` DEFERRED. Publishing is
  # irreversible; pinning a rev is not.
  mt-codec = { git = "https://github.com/bg002h/mnemonic-transaction", rev = "72b79b87…" }

Cargo.lock:  source = "git+https://github.com/bg002h/mnemonic-transaction?rev=72b79b87…"
```

An unpublished sibling crate, consumed by this binary, by rev pin, with the
rationale written down in the file the spec cites.

**Why it matters rather than being a quibble.** §9's order is the thing a plan
author schedules from, and as written it schedules an **irreversible** act —
`cargo publish ms-codec 0.10` — in the middle of a cycle, before the fork phase
has run and before anything has been proven on a device. This repo's standing
posture is the opposite (the comment above; the constellation rule that publishing
is operator-gated and deferred; `mnemonic-io-lib` is a path dep at `:25` for the
same stated reason). The publish is genuinely needed only when `me` itself is
released, and `me` cannot be published today anyway while `mt-codec` is a git rev.

**Close it by** stating the edge rather than the ritual: *"phase 3 follows phase 2
across a **dependency edge** — satisfied by a git rev pin on ms-codec, as this
crate already does for `mt-codec` (`Cargo.toml:54-74`), or by a crates.io release;
the release is required before `me` ships, not before phase 3 compiles. Either way
the `Cargo.lock` / `cargo vendor` freshness ritual applies."* That keeps everything
true and removes a scheduled irreversible act from the middle of the cycle.

---

# MINOR

## M-1 — `(§9.1)` is a new dangling cross-reference, same class as the unfolded M-6

§9's phase-1 row `:311` ends the `presets::hashlock_gated` item with *"(§9.1)"*.
There is no §9.1: §9 has no subsections (headings enumerated — 1, 2, 3, 4, 5, 6,
7, 7.1–7.6, 8, 9, 10, 11, 12, 13). The pointer promises a rule for a **public API
break on a published crate** (md-codec 0.42) and delivers nothing, which is the
half of round 1's I-4 that asked what the signature becomes. §5's `HashLock`
makes the intent inferable, so this is Minor — but the spec now carries two
dangling section references (this and §6's unfolded `(§4.6)`), and one of them the
fold added while the other was a round-1 finding.

## M-2 — `cmd/emu/composer_js.go:53` is off by one, in two places

`:53` is the closing brace of `if h == nil`; the 64-hex hardcoding the spec is
pointing at is `:54` `out = append(out, hex.EncodeToString(h[:]))`. It appears in
§7.3's layer table `:217` and in §11's gate table `:370`. Inherited verbatim from
round 1's own evidence block — the same way `:96` was inherited from the design
review — which is worth noting because the fold's commit message counts "five
false citations fixed" while introducing a sixth of the identical class from the
identical source. Costs an implementer one `grep`; recorded, not blocking.

## M-3 — §7.5 now presents four pixel figures and no threshold, so its own evidence appears to refute its rule

§7.5 `:254-261` says *"character count is not the constraint"*, gives `rmd160` and
`sha256` at **409 px** on one line and `mmmmmm` (**369 px**) / `wwwwww` (**351 px**)
wrapping to two, and concludes *"The constraint is the rendered **width** of the
whole row at the shipped band."* Two things were dropped in the fold:

- the **411 px band** — the old sentence carried it (*"the 411 px band, with 2 px
  to spare"*), the new text carries no band width at all, so none of the four
  figures has anything to be compared against;
- round 1's disambiguation — *"(For a wrapped row the px column is the post-wrap
  width, so lines is the verdict.)"*

Without either, the section reads as: the constraint is width, and the two tokens
that fail are **narrower** than the two that pass. A reader who trusts the numbers
concludes the rule is wrong. The transcription itself is byte-exact against round
1's table (row 22), and §7.5's arbiter clause still contains the damage — which is
why this is Minor and not a re-open of M-2.

## M-4 — §5's rationale is now contradicted by §9, and the fold changed only one side

§5 `:132-136` still reads: *"**One local definition per crate, nothing shared
across repo boundaries.** `me-cli` depends on `md-codec` through crates.io, so a
shared `HashKind` would make the phase order a *release* dependency with a `cargo
vendor` freshness ritual attached. Instead `ms-codec` exposes one digest function
per kind…"*

§9 now says the phase order **is** a release dependency with a `cargo vendor`
ritual attached, for the ms-codec edge, because of that very "instead". So the
stated reason for rejecting a shared `HashKind` is a cost the spec has since
accepted anyway, and §5's own next sentence shares something across a repo
boundary immediately after declaring that nothing is. Round 1 named this
contradiction inside I-1; the fold fixed §9's false claim and left §5's argument
standing.

The decision is still defensible on other grounds — `md-codec` and `ms-codec` share
no dependency edge, and the Go port shares nothing at all — but as written a plan
author meets a rationale the spec disproves 160 lines later, and the obvious
response ("then share the type after all") is forbidden by a reason that no longer
holds. One sentence in §5 closes it.

## M-5 — round 1's I-5 second clause is unfolded: the device preset still has no kind, and §7.1 points at the wrong repo's grammar for it

§7.1 route 6 `:190` covers *"`--preset` / a composer preset"* and explains both
with *"the kind comes from the preset's own grammar (§9 phase 1)"*. That is true of
`md compose --preset` (phase 1, descriptor-mnemonic). The fork's half has no
grammar:

```
gui/composer_presets.go:54-60   func composerPresetDigest() *[32]byte   // 32 bytes of 0xa8
```

It is a hardcoded constructor in phase 4, and when `SpendPath.Hash` becomes a
`HashLock` it must name a kind that the spec still does not state — which is
exactly round 1's I-5 close-by. Minor rather than Important because the wrong
choice is **loud**: the preset is vector-pinned
(`keyed_compose_preset_hashlock_gated.*`, five files in
`compose_vectors.provenance.json`), so any kind but sha256 changes the lowered
bytes and fails `md/compose_vectors_pin_test.go`. Say "sha256" once and the
implementer does not have to discover that by going red.

## M-6 — §10's list of independent KAT generators includes the one most likely to *be* the implementation

§10 `:351-352`: *"Rows are computed **independently of the implementation**
(python3 `hashlib`, `bitcoin::hashes`, or Core)."* The governing clause is right.
But ms-codec today depends on `sha2` and `pbkdf2` only (`crates/ms-codec/Cargo.toml`)
and has no ripemd160 primitive, so the natural way to implement the three new
kinds in Rust is `bitcoin::hashes` — the second of the three blessed generators.
Rows generated from the same crate the implementation calls are self-consistency
again, the precise defect C-1 exists to prevent. Verified that the safe option
works on this box: `python3 -c "hashlib.new('ripemd160')"` succeeds (OpenSSL
3.6.3) and `ripemd160(0xab*32)` = `5786aabcae0e6cd2dfaeca2767dc8996c98f43f4`. One
clause — *"and not the crate the implementation itself calls"* — removes the trap.

---

# NIT

## N-1 — "~1200 other tests" is understated; measured 1319

§10 `:341`. `grep -h "^func Test" gui/*_test.go | wc -l` = **1319**, `TestMain` 0.
(This repo's CLAUDE.md still records 886 from an older era, so the drift is in the
records, not the fold.) Inside a tilde and in an explanatory clause, so it changes
nothing — recorded only because the number is now measurable in one command.

---

# What the fold got right, and should not be re-derived next round

1. **The renumbering is clean everywhere else.** §4's `(§9 phases 2 and 4)` is
   correct against the new table (verified against both digest functions'
   locations). No other document in this repo cites this spec's phase numbers —
   `grep -rl "SPEC_hashlock_kinds"` returns only round 1's report, which is a
   frozen historical artifact and correctly still cites the old numbers.
2. **Phase 4 needs no publish gate**, confirming round 1's VERIFIED SOUND #1 from
   the other direction: the KAT's own re-sync is `cp ../mnemonic-secret/crates/
   ms-codec/tests/vectors/hashlock-v0.8.json hashlock/testdata/`
   (`hashlock-v0.8.provenance.json`), a local-path vendor with a commit pin — and
   `scripts/vendor-compose-vectors.sh /path/to/descriptor-mnemonic` is the same
   shape. The fork consumes all three Rust repos as vendored JSON plus provenance
   pins and takes no crate dependency.
3. **The KAT mechanism is real in both languages already.** Go:
   `hashlock/hashlock_test.go:194-205` with a recorded mutation. Rust:
   `ms-codec/tests/hashlock_derivation.rs:134`. §10 names only the Go reader; the
   Rust one exists too, which makes §12.6's "both languages" cheaper than the
   spec implies.
4. **§7.4's four-consumer enumeration is complete** — re-grepped every production
   reference to `Sha256Digests`: `composer_consent.go:94,97,199` and
   `composer_selfcheck.go:134,136,138`, nothing else outside tests.
5. **The six `[56:]`/`[56..]` sites and their fixed-array signatures** are as §10
   now describes them.
6. **`hashlockFirst8Last8` is genuinely a seventh formatter**, not one of the six —
   which is why M-4 of round 1 is still open rather than folded implicitly.

---

# Counts

| severity | n |
|---|---:|
| Critical | 1 |
| Important | 2 |
| Minor | 6 |
| Nit | 1 |

Round-1 disposition: **C-1 PARTIAL**, **I-1 CLOSED**, **I-2 CLOSED**,
**I-3 CLOSED**, **I-4 CLOSED**, **I-5 PARTIAL**, **I-6 CLOSED**;
4 of 5 false citations fixed, 1 new introduced; M-1 and M-2 closed, M-3/M-4/M-5/
M-6/M-7/N-1/N-2 carried with no stated reason.

# Verdict

**NOT GREEN.** The fold is a real fold, not a paper one: five of six Importants are
closed against their own prescribed remedies, every structural citation it added is
exact bar one off-by-one, and the KAT mechanism it reaches for turns out to exist
and to bite in both languages already. What holds the gate is one Critical and two
Importants, all three of them joints rather than rework:

- **C-2** — the new KAT pins the four digest *functions* and nothing pins the
  kind→function *selection* that §5 deliberately pushes out into every caller. A
  single wrong switch arm produces `OP_RIPEMD160 <hash160(X)>` with §12.1, §12.3,
  §12.4, §12.5, §12.6 and the self-check all green, and §12.2 — the only item that
  could catch it — names no source of truth for the digest it asserts. Two clauses
  close it.
- **I-1** — §7.1's new table replaced a wrong count with a wrong answer: routes 4
  and 5 carry no hash kind (the `phrase:` record carries a *preimage method* whose
  values include `sha256`), and the sentence that said what they do was deleted
  rather than corrected, while §2 decision 5 promises those routes all four kinds.
- **I-2** — §9's publish gate is inferred from a snapshot of a dependency line,
  and the same `Cargo.toml` carries a git-rev-pinned unpublished sibling twelve
  lines above the cited one, with the rationale written out. As written the spec
  schedules an irreversible `cargo publish` mid-cycle that the cycle does not need.

The six Minors and one Nit are recorded, not blocking; seven round-1 Minors/Nits
remain carried with no stated reason and should get either a fold or a sentence
saying why not, so the next round is not re-deriving them a third time.
