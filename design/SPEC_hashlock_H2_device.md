# SPEC — Hashlock H2: the device leg (SeedHammer II fork)

**STATUS: R0 GREEN 2026-09-05 (0 Critical / 0 Important open).** Round 0:
fidelity (opus, `hashlock-H2-spec-R0-r0-fidelity.md`, 3C/5I/6M/2N) + journey
(opus, `hashlock-H2-spec-R0-r0-journey.md`, 2C/6I/5M/1N), one fold (`60a86f6`).
Round 1: fold verification (sonnet, `hashlock-H2-spec-R0-r1-fold-verification.md`):
5/5 C and 10/11 I fixed, I-5 partial, two new Importants the fold introduced
(a false mutation claim; a fictitious fit-gate constant), folded (`c06a760`).
Round 2: fold verification (sonnet, `hashlock-H2-spec-R0-r2-fold-verification.md`):
NF-A, NF-B, I-5 fixed; one new Important — §3 still named the old confirm
gesture where §4.5 had become HOLD — folded here as one word, closed by the
machine check that the old upper-case gesture word no longer occurs in this
file (the method modals' own "Continue?" question is copy, not the gesture). Lens-closure: fidelity/design,
journey/adversarial, fold-verification ×2. Citations measured at fork main
`c4a64fc`, ms `cd0a60f`.
Previous STATUS: R0 round 0 folded (`60a86f6`); before that DRAFT (`bfd042e`).

This is stage H2 of the hashlock-phrase cycle (`design/BRAINSTORM_hashlock_phrase.md`
§4.1; SPEC_ms_hashlock §9's sequence). H0 (reader guards) is merged in both repos;
H1 (`ms hashlock`, ms-codec 0.8.0) is released and published. H2 gives the device
what the brainstorm's §4.4 agreed (L7, L15, L16) and the r2 security review
sharpened: **type a hashlock phrase on the SH2, pick the method, and set a spend
path's hash to the SAME digest the host derives** — deriving, using and dropping
the preimage, never storing, showing or engraving it. The Go derivation is a
strict port of `ms_codec::hashlock` with the ms-codec 0.8.0 corpus vendored and
pinned (Rust-primary rule; nothing is decided in Go).

Rulings that bind (operator, verbatim in the brainstorm): L5 two methods, the
operator chooses; L7 the device derives H, uses it, scrubs X, never stores/shows/
engraves a preimage this cycle; L12 both warnings warn, never refuse; L15 no scrub
discipline beyond what the composer does by construction; L16 §4.4 agreed; L22
`0x03` inert, no new class (shipped in H0); L24 `TagKindMismatch` refused.

**One sentence this stage makes false, in two places:** the composer spec's §14
(and its §6c line, `SPEC_wallet_policy_composer.md:386`) and the fork's
`gui/composer_hash.go:27-28` SAID the composer "never derives, stores or
engraves a preimage this cycle"; all three are now rewritten. From H2 the
composer DERIVES one, in RAM, for the length of one screen; it still never
stores, shows or engraves it. The fork comment is rewritten by THIS stage's
implementation commit (§1 item 5); the composer spec's two sentences are H3's
(the records stage) — they are named here so H3 cannot miss them (fidelity
I-5; r1 NF-C), and H3 has now folded both (`## H3 fold`).

---

3. **§4.1's no-payload lead** (F-482, from the post-impl review's M-1): the plan
   (build-gate fix 9) has `composerHashRows` REPLACE the "Which hash?" lead with
   the no-payload lead when `len(digests) == 0`, rather than adding a second lead
   line; §4.1's parenthetical now says so.
4. **§4.2's lead** (F-482): the shipped lead prepends "This screen does that
   hashing for you." -- the R0 round-0 answer to journey I-5 (the §8i modal the
   operator has just dismissed); §4.2 now quotes the two-sentence lead. The
   layout consequence (a 44 px lead band) was what starved the readout until
   fork `26fd1dd` removed the 8 px cut (F-481).


## §1. Scope

In:
1. `Which hash?` gains the row **`Type a hashlock phrase`** before `Type 64 hex`
   (§4.1), with the row switch re-keyed by LABEL (§5) — the shipped switch is
   index-keyed and its fallthrough clears the lock (r2 C-4).
2. The phrase screen (§4.2), the method pick (§4.3), the derivation (§3, §4.4),
   the confirm modal (§4.5), and the Back contract that binds all four (§4.6).
3. `hashlock` — a new fork package porting `ms_codec::hashlock` (§3) with the
   corpus `hashlock-v0.8.json` vendored under `hashlock/testdata/` and a
   provenance pin (§7.1).
4. `codex32.DecodeMS1Preimage` (§6): the `0x03` arm as its OWN function with its
   own length rule; `DecodeMS1` unchanged (r2 C-2). No screen calls it this cycle;
   it exists so the kind has one decoder and one test, Rust-first.
5. The fork record above — `gui/composer_hash.go:27-28` becomes *"THE COMPOSER
   DERIVES A PREIMAGE IN RAM FOR ONE SCREEN (H2) AND NEVER STORES, SHOWS OR
   ENGRAVES IT. It puts a digest in a script."* — and a phrase-route form of §8h
   at Done (§4.7). The composer spec's §6c/§14 sentences are folded by H3, not here.

Out (§9): storing, displaying or engraving a preimage; reading a preimage plate
into any flow; a scrub discipline; `ms split` of preimages; salt/iteration
choices (F-469); the `me`/host side (H1b, separate plan); the operator's live
device walk (H4, §8).

---

## §2. The phrase rule — the host's, byte for byte

SPEC_ms_hashlock §4.3 is normative; the device applies the SAME predicate to the
typed bytes, in this order, and refuses with a modal (never silently):

1. **non-empty**;
2. **printable ASCII only**, every byte in `0x20..=0x7E` (the keyboard cannot
   produce anything else — the test still pins it);
3. **ms1-shaped is refused, by the HOST's shape test, not by a parse** (fidelity
   C-2). The host's `looks_like_ms1` (ms-cli `crates/ms-cli/src/argv_guard.rs:148-164`)
   is: trim; lowercase; strip the display separators (space, `-`, `,`); the
   result is at least `MIN_MS1_LEN` characters, starts `ms1`, and every
   remaining character is in the bech32 charset. **No checksum.** A checksum
   parse (`codex32.New`) is strictly narrower — a grouped or mistyped plate the
   host refuses would be derived from on the device, and the two would disagree
   on what a phrase is. The refusal names the route that exists (fidelity I-3,
   journey I-1): *"That is a preimage plate, not a phrase. On the host, run
   ms hashlock with it and load the hash: record it prints."*;
4. **at most 100 characters** — the counter reads `n/100`, is NOT clamped (so
   `101/100` is visible and the lockstep row is constructible), and OK is
   refused above 100: *"A hashlock phrase is at most 100 characters."* The shape
   test (rule 3) precedes the cap, as on the host (the corpus's "grouped by 2,
   112 characters" row pins the order);
5. **exactly 64 hex characters is refused**, either case: *"That is a preimage
   in hex, not a phrase. Use the Type 64 hex row."*

**The bytes are used VERBATIM — and this is a rule with a named forbidden
mechanism, not a description** (fidelity C-1). The device already has a one-line
template that normalises a passphrase before a KDF (`sysw/open.go:55`:
`seal.DeriveKey(seal.NormalisePassphrase(passphrase), h.Salt[:], …)`, and
`seal/open.go:231` the same shape). **`seal.NormalisePassphrase`, and any
`strings.TrimSpace`, `strings.Fields`, `strings.ToLower`/`ToUpper` or Unicode
normalisation on the phrase, is forbidden on this path.** Rule 3 folds a COPY
only to detect a plate; the phrase itself is never changed. `Correct Horse` and
`correct horse` are different phrases. Because the anchor phrase is a fixed
point of every one of those folds, §7.1 and §7.2 MUST drive the corpus rows
that are not. `seal.NormalisePassphrase` is `ToLower(Join(Fields(s), " "))`
(`seal/open.go:76-78`), so it changes exactly two corpus rows: `Correct Horse
Battery Staple` (case) and `  a  b ` (leading, trailing and doubled spaces) —
those two are the witnesses against it. The third row, `correct-horse,battery
staple`, is a fixed point of that normaliser (r1 NF-A) and witnesses a DIFFERENT
fold: a screen that stripped display separators from the phrase the way rule 3
strips them from a COPY to detect a plate. A screen-layer fold of either kind
ships green against the anchor alone.

The limit is a named constant, `hashlock.PhraseMaxChars = 100`, the ONLY source
of the counter's denominator and the rule's bound; a test asserts both read it.

---

## §3. Derivation — the port

`hashlock` package, one file, provenance-pinned to ms `cd0a60f`
(`crates/ms-codec/src/hashlock.rs`, ms-codec 0.8.0):

| name | value | Rust |
| --- | --- | --- |
| `Salt` | the 14 bytes `ms-hashlock-v1` | `HASHLOCK_SALT` (`hashlock.rs:27`) |
| `Iterations` | 100000 | `HASHLOCK_ITERATIONS` (`:30`) |
| `PreimageLen` | 32 | `HASHLOCK_DKLEN` (`:32`) |
| `PhraseMaxChars` | 100 | ms-cli `HASHLOCK_PHRASE_MAX_CHARS` (`crates/ms-cli/src/hashlock_phrase.rs:24`; the codec carries no cap — the rule is the CLI's and the device's, §4.3) |
| `PreimageHardened(phrase []byte) [32]byte` | PBKDF2-HMAC-SHA256(phrase, Salt, Iterations, 32) | `preimage_hardened` |
| `PreimageSHA256(phrase []byte) [32]byte` | SHA-256(phrase) | `preimage_sha256` |
| `Digest(x *[32]byte) [32]byte` | SHA-256(x) | `digest` |

**The driver is new and its signature is part of this spec** (fidelity M-2):
`hashlock.DeriveHardened(phrase []byte, progress func(done, total int) bool) [32]byte`
— it passes `Salt` as its own 14-byte slice and `Iterations` as the count, calls
`progress` every `kdfStepIterations` (500) and stops if it returns false (Back).
**`unlockDerive` and `seal.Header` are NOT used**: `Header.Salt [16]byte` would
zero-pad the 14-byte salt and every device digest would silently diverge (r2
M-5). The only test that can see that class is a comparison against the
vendored corpus CONSTANT, never against a value recomputed by the same Go
function (§7.1).

Hardened runs on the countdown screen the sealed payload already uses
(`gui/unlock_kdf.go:219-236`), retitled `Deriving`, with a zero-state lead of its
own: *"Deriving. This takes about 10 seconds."* (the sealed payload's
`unlockKDFLead` says "about 30 seconds" until the first slice completes — that
string and its fallback are calibrated for the payload's iteration count, not
this one's; fidelity M-6, journey M-3). The measured device rate is 9,715 PBKDF2
iterations/s (brainstorm §3.4), so hardened takes about 10 s — the method row
says so. SHA-256 is instant and shows no countdown.

**The preimage lives on the stack for the derivation and the confirm modal and
is dropped after HOLD (the confirm gesture, §4.5) or Back** (L7; L15: no scrub
beyond that). The digest
is what the composer stores (`st.list.Paths[idx].Hash`, a `*[32]byte` —
`md/compose.go:167`, fidelity M-1), exactly as `Type 64 hex` stores one today.

---

## §4. Screens and copy

All copy ASCII. The fit gate is `gui/modal_fits_test.go`'s `assertModalBodyFits`
(r1 NF-B): it RENDERS each specific body (`firstModalFrame`) and measures the
headroom for that exact text by appending filler (`modalHeadroom`), requiring at
least `modalBodyMargin = 80` normalised characters (`modal_fits_test.go:51`);
there is NO capacity constant — the file's own comment says capacity depends on
how the words wrap, which is why it measures each body instead of budgeting them
(the "588" in its comment is one historical measurement of unrelated filler, not
a budget). Every new or changed body in this stage — the phrase screen's lead
and refusals, both method modals, the confirm modal in its longest variant, the
`Which hash?` no-payload lead, the phrase-route §8h — is ADDED to that test's
table and measured; §4.5 states what to drop if its measurement fails.

### §4.1 `Which hash?`

Rows, in order: the payload's `hash:` records (`hash <i>  <first8>..<last8>`,
unchanged), **`Type a hashlock phrase`**, `Type 64 hex`, `No hash lock`. With no
`hash:` record loaded, the screen's lead reads (journey M-1):

> No hash record in the payload. Type a phrase below, or make one with
> ms hashlock on the host.

(this lead REPLACES the screen's default lead when `len(digests) == 0`, so the literal
"Which hash?" is absent from that frame -- as shipped, build-gate fix 9; H3 fold, F-482).

### §4.2 The phrase screen

Title **`Hashlock phrase`**, lead (journey I-2, and the §8i answer folded at R0 round 0): *"This screen does that
hashing for you. Use a phrase you have never used anywhere else."* The four-page printable-ASCII keyboard built with
`NewPassphraseKeyboard` (`gui/passphrase_keyboard.go:76`) — not `NewTextKeyboard`
(:92, settings gear + newline) nor `NewLineKeyboard` (:112). A NEW flow function
`hashlockPhraseFlow(ctx, th, initial []byte) ([]byte, bool)`, not
`passphraseEntryFlow` (`gui/passphrase_flow.go:74`: it hard-codes the
"Passphrase" title, the pass-proof trigger and an over-length message about
plate legibility — r2 M-4). `initial` is what the operator typed before a Back
from the method pick (§4.6). Counter `n/100` from `hashlock.PhraseMaxChars`. OK
applies §2; Back returns to `Which hash?` and drops the phrase. The keyboard's
reveal (`show`) key is inherited as-is: secret-handling, non-gating (fidelity N-2).

### §4.3 The method pick

A two-row `composerPickScreen` titled `Hashlock method`, lead `Which method?`:

1. **`Hardened (about 10 s)`** — under 20 characters, a confirm modal first
   (journey I-3):
   > Even a 20-character phrase falls in about 72 days
   > on one GPU, and shorter ones fall sooner. Choose it
   > from a generator. If you have used this phrase
   > anywhere else, press Back and choose another.
   > Continue?
2. **`SHA-256`** — always, a confirm modal first:
   > This is the brainwallet construction: anyone
   > holding the digest tests 10^10 phrases per second.
   > A phrase a person chose is not safe here; use six
   > diceware words. If you have used this phrase
   > anywhere else, press Back and choose another.
   > Continue?

Both are confirm-to-proceed (L12: never refusals). **Declining either modal
returns to the method pick with the phrase intact** (journey I-5): the warning
exists to move the operator from SHA-256 to Hardened, which needs the phrase to
survive. The method is a permanent property of the policy; the confirm modal
(§4.5) prints it.

### §4.4 Deriving

Hardened: the countdown screen, title `Deriving`, zero-state lead *"Deriving.
This takes about 10 seconds."*, then `About N seconds left.` from the measured
rate, driven by `hashlock.DeriveHardened` — never `unlockDerive` and never a
`seal.Header` (§3). Back during the derivation abandons it and nothing is assigned (r2 Q3),
returning to the method pick with the phrase intact. A power loss ends the
composition, as it does at any other point in this flow — `composerState` is RAM
(journey M-4).

### §4.5 The confirm modal

The surface is the composer's own confirm screen, `composerConfirmScreen(ctx, th,
title, body)` (`gui/composer_shape.go:77`), whose body ends with
`composerConfirmBody`'s "Hold button to confirm." (`gui/composer_copy.go:32-33`):
the operator HOLDS to confirm and presses Back to decline (fidelity M-4). Title
`Hash lock`. Lines, in order (§8i and §8h are NOT here, see §4.7 and §5):

```
hash  <first8>..<last8>
method: hardened   chars: <n>        (or: method: sha256   chars: <n>)
<relation line, only when the payload holds hash: records>
<other-path line, only when another path of this policy already carries a different
hash: "another path has a different hash: back up every phrase">
Write down this phrase, the method and this digest
now. The phrase and method are not on this device.
Without both, this path can never be spent.
One phrase per policy. Spending any path of a wsh
wallet publishes this digest. Never use this phrase
as a passphrase or a password anywhere else -- a
spend publishes the preimage, and anyone can then
test guesses at the phrase itself.
```

The reconciliation lines are drawn on their own post-HOLD `showError` screen,
shown immediately after HOLD assigns the digest and reachable for every policy
that has a phrase-set hash (the drop order below names this same destination):

```
hash  <first8>..<last8>
method: <m>   chars: <n>
Before you cut plates, run ms hashlock with this
phrase and method on the host and check the digest
matches. If they differ, do not fund this wallet:
build it again.
```

- `chars: <n>` is the phrase's byte count — the one signal that shows a stray
  space when the operator later reconciles against the host card's
  `phrase_chars` (journey M-5).
- The relation line (journey C-2): when the payload holds `hash:` records,
  `matches hash <i> in the payload` if the digest equals record *i*, else
  `no hash: record in the payload has this digest`; omitted when there are none.
- The other-path line (journey I-1): when any OTHER path of the same policy
  already carries a `Hash` that differs from this digest, the modal says so,
  because `md.ValidatePathList` has no clause about two paths' `Hash` values and
  "One phrase per policy" is advice — a second phrase is legal, and a second
  backup burden the operator must choose knowingly. Omitted when no other path
  carries a hash, or when the hashes are equal.
- The backup line (journey C-1) is unconditional: this is the first flow on the
  device that takes a secret, uses it and forgets it, and nothing else on the
  route says so. The reuse lines are the brainstorm's §3.7 copy in full
  (fidelity M-5). The reconciliation line converts a divergence discovered at
  spend time into a five-minute check.
- **HOLD** (the confirm gesture) sets `st.list.Paths[idx].Hash` to the digest
  and returns to the path; **Back** returns to the method pick with the phrase
  intact, nothing assigned (§4.6).
- **Drop order if `assertModalBodyFits` fails on the longest variant** (journey
  M-2; r1 measured the normalised body at about 484 characters, headroom
  uncertain because the 18-character digest token does not wrap): first shorten
  the reuse block to the brainstorm's two sentences ("One phrase per policy.
  Never use this phrase as a passphrase or a password anywhere else."), then
  move the reconciliation line out of the confirm modal and onto its own
  dismissible screen shown immediately after HOLD, where it is reachable for
  every policy that has a phrase-set hash — NOT into the phrase-route §8h at
  Done, whose `composerEveryPathHashed` guard is false for any policy with one
  un-hashed path. The backup line and the relation line are never dropped.

The 64 visible bits (`first8..last8`) are a transcription check, adequate ONLY
because the full-width lockstep vector runs in CI (§7.1); the H4 walk records
both full digests (r2 N-2). §7.2 asserts the fit of this exact body with the
relation line present and `chars: 100`.

### §4.6 The Back contract (fidelity I-1, journey I-4, I-5)

Normative, stated once: `composerHashEdit` runs the phrase route as a LOOP.
Back from the confirm modal → the method pick (phrase intact); Back from a
declined method modal → the method pick (phrase intact); Back from the method
pick → the phrase screen (phrase intact, via `initial`); Back from the phrase
screen → `Which hash?` (phrase dropped); Back during the derivation → the method
pick (phrase intact). **`composerHashEdit` returns `false` ONLY for Back at
`Which hash?` itself.** Today, at path creation, `false` REMOVES the path
(`gui/composer_shape.go:269`) — the `Type 64 hex` route does that, and a phrase
route that copied its sibling would delete a path, and the EXPERIMENTAL key-less
consent already given, because the operator read a digest they did not expect
and pressed Back. §7.2's Back tests run through the CREATION entry point
(`composerAddPath`) and assert the path still exists, not only that `Hash` is
unchanged.

### §4.7 §8h on the phrase route (journey C-1, fidelity I-2, journey I-6)

§8h stays at Done (`gui/composer_shape.go:443`, guarded by
`composerEveryPathHashed(st.list)`), where its predicate is true; it is NOT shown
in the per-path confirm modal, where the shape is partial and the banner would be
false most times it appears. Its copy gains a phrase-route form: when every path
is hashed and at least one hash was set by phrase, the text reads

> HASH ON EVERY PATH
> Every way to spend this wallet needs a hashlock
> preimage. It is not on this device and not on these
> plates. Back up every phrase and its method, and every
> preimage plate, separately.

(the shipped text now ends *"Back up every preimage separately."*, still
naming only "the preimage" -- an artifact this route cannot produce;
`composerCopyHashEveryPath` at `gui/composer_copy.go:169-173`). The
§8i rule modal fires at the pick (§5) as today, once; it is not repeated in the
confirm modal (journey N-1, fidelity M-4).

---

## §5. The row switch is label-keyed (r2 C-4)

`composerHashEdit` (`gui/composer_hash.go:140-172` at `c4a64fc`) builds `rows`
as payload digests + `Type 64 hex` + `No hash lock` and dispatches on
`sel < len(digests)` / `sel == len(digests)` / `default` — and the `default`
arm CLEARS the lock. Inserting a row before `Type 64 hex` under that switch
makes `Type 64 hex` fall into `default` and silently remove the hash.

Normative: the rows are built ONCE into a struct that records each named row's
index (`payloadRows`, `phraseRow`, `hexRow`, `noneRow`); the switch dispatches
on those names; there is NO `default` that assigns — an unknown index is a
programming error (`panic` naming the index), never "clear the lock". The §8i
modal fires when the operator is TAKING a hash (payload row, phrase row or hex
row), stated as that predicate, not as `sel <= len(digests)`. Tests cover every
row by label, with 0, 1 and 2 payload digests loaded (the displaced rows
included). `composerPickScreenMaxRows` is checked against the longest row set
(two payload digests + three fixed rows) in a test.

---

## §6. Preimage strings on the device — inert, one decoder

H0 (fork `c4a64fc`) made a kind-`0x03` single inert on every reader and door;
this stage adds the decoder and nothing that calls it from a screen:

`codex32.DecodeMS1Preimage(s String) (preimage [32]byte, err error)` — accepts
ONLY an unshared string whose data is exactly 33 bytes beginning `0x03` (the
shape `IsPreimage` already tests) and returns the 32 bytes; every other input
returns `errMSBadPrefix` (wrong first byte, or a SHARE — fidelity N-1) or
`errMSBadLength` (unshared, `0x03`, not 33 bytes), the same errors `DecodeMS1`
uses. `DecodeMS1` is UNCHANGED and keeps refusing `0x03` at all five callers
(`gui/ms1_decode.go:22`, `gui/codex32_polish.go:106`, `gui/singlesig_verify.go:185`,
`gui/multisig_verify.go:1237`, `bundle/verify.go:138`) — r2 C-2 — and "Show
secret" stays gated on `err == nil`. A typed or scanned preimage string is
refused everywhere a seed is expected, as H0 measured; the only device-side
signpost to the host route is §2 rule 3's refusal text, and H3's manual chapter
must say the same.

The returned preimage is secret: the doc comment says the caller scrubs, exactly
as `DecodeMS1`'s does.

---

## §7. Tests

### §7.1 Lockstep against the vendored corpus (the gate that matters)

`hashlock/testdata/hashlock-v0.8.json` = ms `crates/ms-codec/tests/vectors/hashlock-v0.8.json`
byte for byte, sha256 `a46c197a3640fe8af4ca4370b46a9637466649227163ce6761bb032354811d30`,
with `hashlock-v0.8.provenance.json` in the shape of `sysw/testdata/record_class_vectors.provenance.json`
(repo, remote, path, commit `cd0a60f`, sha256, row count, recorded_at). The
corpus's own `lockstep` array names what the fork must drive, in BOTH directions
where it says so (fidelity I-4). Tests:

- the file hashes to the pinned literal (drift on either side reds one suite);
- every `derivation` row (11): `PreimageHardened(phrase)` == `hardened_x`,
  `Digest` == `hardened_h`, `PreimageSHA256(phrase)` == `sha256_x`, `Digest` ==
  `sha256_h` — compared against the JSON's CONSTANTS, never against a value the
  Go code recomputed (mutations: zero-pad the salt to 16 bytes → every hardened
  row fails; 99,999 iterations → every hardened row fails; **fold the phrase
  through `seal.NormalisePassphrase` before deriving → the `Correct Horse
  Battery Staple` and `  a  b ` rows fail** (the anchor row alone would not,
  and `correct-horse,battery staple` is a fixed point of that normaliser — r1
  NF-A); **strip display separators from the phrase before deriving → the
  `correct-horse,battery staple` row fails** — fidelity C-1);
- every `refusals` row (15) through the §2 rule: empty, TAB/DEL/0xFF, ` ~`
  accepted, 64-hex both cases refused naming `Type 64 hex`, `beef` accepted,
  the plate lowercase / UPPERCASE / grouped by 5 / with leading and trailing
  spaces / grouped by 2 (112 characters — the shape test precedes the cap), the
  100-character row accepted and the 101-character row refused with the §2 message;
- the `kind` row: `DecodeMS1Preimage` on `kind[0].ms1` returns
  `kind[0].preimage_hex`; the entr-32 pair row → `errMSBadPrefix`;
- `PhraseMaxChars` is read by both the counter and the rule (mutation: change one
  → a test that types 100 characters and expects OK fails).

### §7.2 The screens, on the touch harness

Driven through the real flow (`runUITouch`, as the H0 door tests do), entered
through `composerAddPath` (the CREATION entry point): tap `Type a hashlock
phrase`, type a phrase, pick each method, confirm — the path's `Hash` equals the
corpus's `hardened_h` / `sha256_h` **for the three non-fixed-point rows as well
as the anchor** (typed with their exact case, spaces and separators). Back at
each step per §4.6, asserting the path still EXISTS and `Hash` is unchanged;
decline SHA-256, choose Hardened, and the resulting `Hash` equals `hardened_h`
for the phrase typed ONCE. The §2 refusals driven through the screen with the
counter at `101/100`, with a 64-hex phrase, with an ms1-shaped phrase (grouped
and ungrouped). The two method modals appear when their condition holds and not
otherwise (19 vs 20 characters; SHA-256 always). The confirm modal's relation
line with 0, 1 and 2 payload records (matching and not). Geometry: every body §4 adds
or changes is in `modal_fits_test.go`'s table — the confirm modal in its longest
variant (relation line present, `chars: 100`), the no-payload lead, both method
modals, the phrase-route §8h — and `assertModalBodyFits` passes for each.

### §7.3 The switch

Every `Which hash?` row by label with 0, 1, 2 payload digests: each row does
what its label says; `Type 64 hex` never clears the lock (the C-4 regression
test); the §8i modal fires for the three taking rows and not for `No hash lock`;
the longest row set fits `composerPickScreenMaxRows`.

### §7.4 The decoder

`DecodeMS1Preimage`: the corpus's `kind[0].ms1` → its `preimage_hex`; the
acceptance record's plate (`ms10hashsq0p7jaf…`, `ms-hashlock-H1-acceptance.md`)
→ the corpus anchor row's `hardened_x`; an entr string → `errMSBadPrefix`; a share
→ `errMSBadPrefix`; the `preimage-shape-entr-id` seam row → 32 bytes (the kind is
the prefix byte; the id is not consulted here either — the HOST refuses the
mismatch, ruling L24); an unshared `0x03` string with 17 bytes → `errMSBadLength`.
`DecodeMS1` on the plate → `errMSBadPrefix` still.

### §7.5 The emulator arm

`cmd/emu/walk_hashlock_phrase.js`: from the composer, take a path to `Which
hash?`, tap the phrase row, type the anchor phrase, pick SHA-256 (instant),
confirm, and read the `hash first8..last8` line: it must equal
**`b867db87..edbc96cb`** (`sha256_h` of the anchor row; fidelity C-3); the
negative control types a different phrase and must NOT match; a second run types
`Correct Horse Battery Staple` and must show the corpus's mixed-case digests, not
the anchor's. Hardened is walked once (about 10 s) and compared to
`3cf5d421..b70a4c12`.

### §7.6 Firmware size

Measured at the gate (`nix develop -c tinygo build -size short …`); PBKDF2 and
SHA-256 are already linked (`seal/pbkdf2.go`, `seal/crypto.go`), the keyboard
exists; expect a small delta over `c4a64fc`'s 1,583,132 / 62,800.

---

## §8. Acceptance (H4 — the operator's walk)

H2 is done when, on the flashed device: the operator types the anchor phrase
under each method and the `first8..last8` shown equals `ms hashlock`'s on the
host for the same phrase and method (`3cf5d421..b70a4c12` hardened,
**`b867db87..edbc96cb`** sha256), then types `Correct Horse Battery Staple` and
sees a different pair, records all full 64-hex digests in the continuity; a
`hash:` record packed by `ms hashlock … | me sysw pack` is offered as a payload
row and the confirm modal's relation line says `matches hash 1`; and a preimage
plate presented to the device is still refused (H0's walk). Until the operator
walks it, §7.5's emulator arm is the acceptance.

---

## §9. Out of scope (this cycle)

Storing, displaying or engraving a preimage on the device; reading a preimage
plate into any flow (§6 adds the decoder only); a scrub discipline (L15); the
salt/iteration parameters (F-469); `ms split` of a preimage (F-468); the host's
0.8 bump (H1b, `IMPLEMENTATION_PLAN_hashlock_H1b_me_bump.md`); the flash of H0
(the operator's); the seam-corpus prose correction the H1b fidelity lens filed
with this stage (`bip93-plain-33-byte-payload-0x03`'s `source` says 0.8 refuses
it as `TagKindMismatch`; it is `UnknownTag`) — to be folded when H2 re-vendors
the corpus and re-pins both sha literals.

---

## §10. Citations — measured at fork `c4a64fc`; re-grep at implementation time

| claim | where |
| --- | --- |
| `composerHashEdit` builds rows and dispatches by index; `default` clears | `gui/composer_hash.go:140-172` (`rows = append(rows, "Type 64 hex")` at :147; `composerPickScreen(ctx, th, title, "Which hash?", rows)` at :149; `default: st.list.Paths[idx].Hash = nil` at :172) |
| the header comment this stage makes false | `gui/composer_hash.go:27-28` |
| `false` from `composerHashEdit` removes the path at creation; §8h fires at Done | `gui/composer_shape.go:269`, `:443` (`composerEveryPathHashed`) |
| `composerPickScreen(ctx, th, title, lead string, rows []string) (int, bool)` | `gui/composer_paged.go:259` |
| `composerHexEntry`, `composerHashRow`, `composerPayloadDigests`, `composerCopyHashRule`, `composerCopyHashEveryPath` | `gui/composer_hash.go:69, :38, :47`; `gui/composer_copy.go:175`, `:169-173` |
| `Hash *[32]byte` | `md/compose.go:167` |
| the three keyboards | `gui/passphrase_keyboard.go:76` (`NewPassphraseKeyboard`), `:92` (`NewTextKeyboard`), `:112` (`NewLineKeyboard`) |
| `passphraseEntryFlow` hard-codes its title and messages | `gui/passphrase_flow.go:74` |
| the countdown copy, its zero-state lead and step size; `unlockDerive(ctx, th, h seal.Header, pass []byte)` | `gui/unlock_kdf.go:26` (`kdfStepIterations = 500`), `:219-221` (`unlockKDFLead`, "about 30 seconds"), `:236`, `:242` |
| the normalising template that is forbidden here | `sysw/open.go:55` (`seal.NormalisePassphrase`), `seal/open.go:231`; the composer spec sentence H3 folds: `SPEC_wallet_policy_composer.md:386` |
| PBKDF2 and SHA-256 already linked | `seal/pbkdf2.go`, `seal/crypto.go`, `gui/unlock_kdf.go` |
| `DecodeMS1` and its five callers | `codex32/mspayload.go:35`; `gui/ms1_decode.go:22`, `gui/codex32_polish.go:106`, `gui/singlesig_verify.go:185`, `gui/multisig_verify.go:1237`, `bundle/verify.go:138` |
| `IsPreimage` (H0) | `codex32/mspayload.go:94` (unshared, 33-byte payload, `0x03`) |
| the fit gate: per-body render + headroom, `modalBodyMargin = 80`, no capacity constant | `gui/modal_fits_test.go:51` (`assertModalBodyFits`, `modalHeadroom`, `normalizeDrawn` :60-71; the comment at :32 records one historical 588-character filler measurement) |
| the confirm surface and its hold gesture | `gui/composer_shape.go:77` (`composerConfirmScreen`), `gui/composer_copy.go:32-33` (`composerConfirmBody`: "Hold button to confirm.") |
| what `seal.NormalisePassphrase` does | `seal/open.go:76-78`: `ToLower(Join(Fields(s), " "))` |
| vendored-corpus convention | `sysw/testdata/record_class_vectors.provenance.json`, `sysw/codex32_seam_test.go` (sha pinned as a literal) |
| the corpus, its sha, its `lockstep` and `refusals` arrays | ms `crates/ms-codec/tests/vectors/hashlock-v0.8.json` at `cd0a60f`, `a46c197a…1d30` (CHANGELOG ms-codec 0.8.0) |
| the derivation constants; the phrase cap | ms `crates/ms-codec/src/hashlock.rs:27,30,32`; ms-cli `crates/ms-cli/src/hashlock_phrase.rs:24` |
| the host's shape-only ms1 predicate | ms-cli `crates/ms-cli/src/argv_guard.rs:148-164` (`looks_like_ms1` → `is_ms1_shaped`; `format::strip_display_separators`) |
| the phrase rule and both warnings' copy; the reuse lines | SPEC_ms_hashlock §4.3, §7; brainstorm §3.7 |
| measured KDF rate 9,715 it/s | brainstorm §3.4 |
| the anchor row's digests | `ms-hashlock-H1-acceptance.md` (hardened `3cf5d421…4c12`, sha256 `b867db87…96cb`) |
| ruling L22 and H0's guards | `codex32.IsPreimage`, `sysw.isStrictMs1`, `seal.Classify`, `gui/scan.go`, `engraveCodex32`, `unlockEngraveCodex32` at `c4a64fc` |

---

## R0 round 1 folded here

r1 NF-A → §2 and §7.1 credit the normaliser mutation to the two rows it actually
changes and give the separators row its own mutation; NF-B → §4, §4.5, §7.2 and
§10 cite the real gate (`assertModalBodyFits`, per-body, margin 80, no capacity
constant) and require every new body in its table, with a drop order; NF-C /
fidelity I-5 → the fork comment is this stage's, the composer spec's two
sentences are H3's, said in the opening paragraph, §1 and §10; fidelity M-4 →
`composerConfirmScreen` + HOLD; fidelity M-2 → §4.4 repeats the forbid.

## R0 round 0 folded here

Fidelity: C-1 → §2's forbidden-mechanism rule and the non-fixed-point rows in
§7.1/§7.2/§7.5; C-2 → §2 rule 3 restated as the host's shape test; C-3 →
`b867db87..edbc96cb`; I-1 → §4.6 (the loop, `false` only at `Which hash?`, tests
through `composerAddPath`); I-2 → §4.7 (§8h at Done only, with a phrase-route
form); I-3 → the refusal names the host route; I-4 → §7.1 drives `refusals`,
`kind` and `lockstep`; I-5 → the two records named up front and in §1; M-1
`*[32]byte`; M-2 the driver's signature; M-3 "the acceptance record's plate";
M-4 §8i once; M-5 the full reuse clause; M-6 the zero-state lead and the
100,000-iteration calibration; N-1 shares → `errMSBadPrefix`; N-2 recorded.
Journey: C-1 → the backup line in §4.5 and the phrase-route §8h; C-2 → the
relation line and the reconciliation line; I-1 → the refusal text; I-2 → the
phrase screen's lead and the verb in both modals; I-3 → "Even a 20-character
phrase … shorter ones fall sooner"; I-4/I-5 → §4.6; I-6 → §4.7; M-1 the
no-payload lead; M-2 the fit assertion on the longest variant; M-3 the
zero-state lead; M-4 the power-loss sentence; M-5 `chars: <n>`; N-1 §8i once.

---

## H3 fold

The two spec departures the H2 implementation plan recorded rather than folded
are applied above, at H3 (the records stage). Both are quoted verbatim in
`IMPLEMENTATION_PLAN_hashlock_H2_device.md` under `## R0 round 0 folded here`,
committed at `f60c2df` ("2 spec departures recorded as H3 items"); the plan is
NOT edited by this fold.

1. **§4.5's drop order, last clause.** The plan's `## R0 round 0 folded here`
   item 3 moved the reconciliation line onto its own `showError` screen right
   after HOLD, because §8h's `composerEveryPathHashed` guard
   (`gui/composer_state.go:244` on `hashlock-h2`; `:239` at the fork baseline
   `c4a64fc`) is false for any policy with one un-hashed path, so the drop
   order's original destination was unreachable on the ordinary mixed wallet.
   §4.5's clause now names the new destination. The removal from the modal
   stands; only the destination changed.
2. **§4.5's line list gains the other-path line** (journey I-1). The plan's
   second H3 record item; `hashlockOtherPathLine` compares the new digest
   against every OTHER path's `*p.Hash` and the confirm body draws the line
   between the relation line and the backup line
   (`gui/composer_hashlock.go:64-66`, `gui/composer_copy.go:409-417`).

**One quote differs from the plan's prescription, deliberately.** The plan
prescribed the line's copy as `"another path has a different hash: two phrases
to back up"`, which was the string at `17b3979`. The ultracode-lens fold at
`a1fd139` made it count-free — `composerCopyHashlockOtherPath` now returns
`"another path has a different hash: back up every phrase"`
(`gui/composer_copy.go:454-456`) — so §4.5 quotes the live string. Everything
else in both departures is the plan's wording verbatim.

The composer spec's own two sentences (`SPEC_wallet_policy_composer.md` §6c and
its §14 row), which this spec's opening paragraph and §1 item 5 name as H3's,
are folded in the same commit.
