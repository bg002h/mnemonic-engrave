# R0 round 0 — fidelity + design lens on `design/SPEC_hashlock_H2_device.md`

Reviewer: independent (opus). Artifact: engrave `bfd042e`. Sources read: the spec;
`design/BRAINSTORM_hashlock_phrase.md` §2, §3.4, §3.7, §4.1, §4.4, §4.6, §7.1;
`design/SPEC_wallet_policy_composer.md` §6c/§8h/§8i/§14;
`mnemonic-secret/design/SPEC_ms_hashlock.md` §2, §4.3, §4.4, §7, §8;
`mnemonic-secret` at `fb98d73` (`crates/ms-cli/src/hashlock_phrase.rs`,
`crates/ms-cli/src/argv_guard.rs`, `crates/ms-codec/src/hashlock.rs`,
`crates/ms-codec/tests/vectors/hashlock-v0.8.json`); the fork at `c4a64fc`
(`gui/composer_hash.go`, `gui/composer_shape.go`, `gui/composer_paged.go`,
`gui/composer_copy.go`, `gui/composer_copy_test.go`, `gui/modal_fits_test.go`,
`gui/unlock_kdf.go`, `gui/passphrase_keyboard.go`, `gui/composer_discard.go`,
`seal/pbkdf2.go`, `seal/crypto.go`, `seal/open.go`, `sysw/open.go`,
`codex32/codex32.go`, `codex32/mspayload.go`, `md/compose.go`).

Read-only throughout; nothing built, nothing committed, no `.jsonl` opened.

**Counts: 3 Critical / 5 Important / 6 Minor / 2 Nit.**

The one question — *would a fork built exactly to this spec produce the host's
digest?* — answers **no** on two independent paths (C-1, C-2), and the spec's own
acceptance criterion for that equality is mistyped (C-3).

---

## Critical

### C-1 — §7.2/§7.5 cannot detect a screen-layer normalisation, and the device has a one-line template that performs one

**Where:** §3 (the hazard list), §7.2, §7.5.

**The gap.** §3 names exactly one derivation hazard — `unlockDerive`'s
`seal.Header.Salt [16]byte` zero-padding the 14-byte salt (r2 M-5) — and forbids
that one function. It names nothing about the *passphrase* argument. But the
device's actual "derive a key from a typed string" idiom is not `unlockDerive`;
it is `sysw/open.go:55`:

```go
key := seal.DeriveKey(seal.NormalisePassphrase(passphrase), h.Salt[:], int(h.Iterations))
```

and `seal/open.go:231` is the same shape. `seal.NormalisePassphrase`
(`seal/open.go:76-78`) is

```go
func NormalisePassphrase(s string) string {
	return strings.ToLower(strings.Join(strings.Fields(s), " "))
}
```

— it **lowercases and collapses every run of whitespace**. That one line is the
nearest existing model for what §4.4 asks an implementer to build, it sits in the
package the composer already opens payloads through (`ctx.sysw`), and it carries
*both* hazards at once: the fold and the `Salt[:]` pad. The spec forbids only the
second, and forbids it by naming a *different* function.

**The counterexample — why no test in the spec fires.** The anchor phrase is
`correct horse battery staple` (corpus `derivation[0]`; §7.5 and §8 both drive
it). It is already lowercase and single-spaced, so

```
NormalisePassphrase("correct horse battery staple") == "correct horse battery staple"
```

is the **identity**. §7.2 drives the screens with "the anchor phrase". §7.5's
emulator arm types "the anchor phrase". §7.1 is a *package-level* test of
`hashlock.PreimageHardened` / `PreimageSHA256` and never crosses the screen
boundary. So an implementation that folds the phrase between the keyboard and
`hashlock.PreimageHardened` passes **every gate this spec defines**, ships, and
then diverges from the host on the first phrase that is not a fixed point of
`NormalisePassphrase` — including two rows the corpus already carries:

| corpus row | host X | device X after a fold | same? |
| --- | --- | --- | --- |
| `Correct Horse Battery Staple` | of the mixed-case bytes | of `correct horse battery staple` | **no** |
| `  a  b ` | of `0x20 0x20 61 0x20 0x20 62 0x20` | of `a b` | **no** |

§7.1's own mutation list even says "mutation: lowercase the phrase → the
`Correct Horse Battery Staple` row fails" — true of the *package* test, and
exactly why the screen test's blind spot is invisible: the reader is reassured by
a gate that does not cover the layer where the defect lives.

**SUGGESTION.** Two edits, both small.
1. §3: extend the forbid from the salt to the phrase. State that
   `seal.NormalisePassphrase` (and any `strings.Fields`/`ToLower`/`TrimSpace` on
   the phrase) is forbidden on this path, naming `sysw/open.go:55` and
   `seal/open.go:231` as the shape not to copy, and state the new driver's
   signature explicitly (`salt []byte`, `iterations int`, `phrase []byte`).
2. §7.2 and §7.5: **drive the screens with a phrase that is not a fixed point of
   `NormalisePassphrase`.** `Correct Horse Battery Staple` (mixed case) and
   `  a  b ` (leading, interior and trailing spaces) are both corpus rows with
   pinned constants, so the assertion costs nothing new. Make this explicit as
   the reason: "the anchor phrase cannot see a fold; these rows can."

---

### C-2 — §2 rule 3's ms1-shape predicate is strictly narrower than the host's, so the device derives from strings the host refuses

**Where:** §2 rule 3, against `SPEC_ms_hashlock` §4.3 and
`crates/ms-cli/src/argv_guard.rs:148-162`.

**The host's predicate** — one function, shared by the argv guard and both phrase
channels, called at `hashlock_phrase.rs:131`:

```go
pub(crate) fn looks_like_ms1(raw: &str) -> bool {
    is_ms1_shaped(&raw.trim().to_ascii_lowercase())
}
fn is_ms1_shaped(s: &str) -> bool {
    let t: String = crate::format::strip_display_separators(s);
    t.len() >= MIN_MS1_LEN            // 48
        && t.starts_with("ms1")
        && t[3..].chars().all(|c| BECH32_CHARSET.contains(c))
}
```

Three properties: it **strips display separators** (whitespace, `-`, `,`) before
testing, it has a **length floor of 48**, and it is **shape-only — it never
parses and never checks the checksum**. The source comment is explicit: "the
charset half is what makes the near-miss control pass"; the unit test builds its
plate as `format!("ms10hashsq{}", "q".repeat(65))` with the comment *"The checksum
is wrong on purpose -- the shape test must not parse."*

**The spec's predicate** (§2 rule 3): *"the string, trimmed and case-folded,
starts `ms1` and is BCH-valid codex32 (`codex32.New` accepts it)"*. `codex32.New`
(`codex32/codex32.go:98-124`) enforces the length classes, `inputHRP`,
`inputData` over the charset, **`check.isValid()` (the BCH checksum)** and
`sanityCheck()`. It does not strip separators.

**Counterexamples, both from the host's own pinned rows.**

| input | host | device per §2 rule 3 | outcome |
| --- | --- | --- | --- |
| corpus refusal row `<the kind[0].ms1 string, grouped by 5 with spaces>` | `Ms1Shaped`, remedy `--in` | `codex32.New` rejects (spaces are not in the charset) → not ms1-shaped → 89 chars ≤ 100 → not 64-hex → **accepted** | device **derives a digest from the text of a plate** |
| corpus refusal row `<the kind[0].ms1 string, grouped by 2 (112 chars)>` | `Ms1Shaped` (shape precedes the cap — the fix for ms's R0 r0 adversarial N-1) | `codex32.New` rejects → falls to the cap → **"at most 100 characters"** | the exact wrong-message defect the host already fixed, reintroduced on the device |
| `hashlock_phrase.rs:192`'s plate, `ms10hashsq` + `q`×65 (75 chars, deliberately bad checksum) | `Ms1Shaped` | `errInvalidChecksum` → not ms1-shaped → **accepted** | a **hand-typed plate with one typo** is derived from |

The third row is the one that bites in the field. The SH2 has no camera, so a
plate reaches this screen only through 75 hand-typed characters, and a single
mistyped character is the *expected* case — precisely the case the host's
shape-only predicate was designed to still catch, and the case `codex32.New`
cannot.

`SPEC_ms_hashlock` §4.3 states the intent the spec has to hit: *"The shape test is
ONE function that both the argv guard and the phrase channels call … so the two
cannot drift."* Rewriting the predicate in a different form on the device is
exactly the drift that sentence forbids.

**SUGGESTION.** Restate §2 rule 3 as the host's predicate, not as a parse:

> trimmed, lowercased, and with display separators (whitespace, `-`, `,`)
> stripped, the string is at least 48 characters, begins `ms1`, and every
> remaining character is in the bech32 charset `qpzry9x8gf2tvdw0s3jn54khce6mua7l`.
> **No checksum is tested** — a mistyped plate is still a plate.

Give it a named function (`hashlock.LooksLikeMs1`) so §7 can drive it, and add the
five corpus `refusals` ms1 rows (lowercase, UPPERCASE, grouped-by-5,
padded, grouped-by-2) to §7.1 as a table, asserting the *grouped* and
*bad-checksum* spellings refuse. Add the `hashlock_phrase.rs:192` bad-checksum
plate as a sixth row and say why it is there.

---

### C-3 — the sha256 acceptance literal is seven hex characters; the device can never show it

**Where:** §7.5 and §8 (twice), and §8 is the H4 operator's walk.

`sha256_h` of the anchor row is

```
b867db875479bcc0287352cdaa4a1755689b8338777d0915e9acd9f6edbc96cb
```

`composerHashRow` (`gui/composer_hash.go:38-41`) renders `h[:8]` and `h[56:]`, so
the device draws

```
hash  b867db87..edbc96cb
```

The spec says the walk and the emulator arm must read **`b867db87..dbc96cb`** —
seven characters after the dots; the leading `e` of `edbc96cb` is dropped.
(The hardened literal `3cf5d421..b70a4c12` is correct: `h[56:]` of `hardened_h` is
`b70a4c12`.)

This is not cosmetic. §8 is the acceptance for the single guarantee the whole
stage exists to establish — *the device's digest equals the host's* — and it is
performed by a human comparing the shown row against this literal. As written the
comparison **fails on a correct device**, and the two available responses are both
bad: record H2 as a divergence, or "fix" the row renderer. §7.5's emulator arm
written to this literal is a gate that cannot pass.

**SUGGESTION.** `b867db87..edbc96cb`, in both §7.5 and §8. Then add the
constellation's standing guard against this class: state in §7.5 that the
expected row is **computed in the test from the vendored corpus's `sha256_h` /
`hardened_h` by the same `h[:8]`/`h[56:]` slicing**, never typed as a literal, so
a transcription error cannot survive.

---

## Important

### I-1 — "Back returns to `Which hash?`" needs a loop the spec does not ask for, and §7.2's assertion false-PASSes when the path is destroyed instead

**Where:** §1 item 2, §4.2, §4.4, §4.5, §7.2.

`composerHashEdit` (`gui/composer_hash.go:140-176`) is **straight-line**: pick →
§8i → switch → return. It has no loop back to the pick screen. Its `false` return
is consumed at two sites:

- `gui/composer_shape.go:269` (path **creation**):
  `if !composerHashEdit(...) { st.list.Paths = st.list.Paths[:idx]; return }` —
  **the path being created is deleted.**
- `gui/composer_shape.go:346` (path menu): the return is discarded.

The shipped sub-flow behaves that way too: `composerHexEntry` Back →
`return out, false` → `composerHashEdit` returns false → at creation the path is
gone. So an implementer who wires the phrase/method/derive/confirm Backs the same
way satisfies "nothing was assigned" while violating §1 item 2 ("returns to
`Which hash?` with the path unchanged") and §4.4 ("the composer state is
untouched"). Meeting the spec requires restructuring `composerHashEdit` into a
loop — which §5 (the only structural section) does not mention while it does
prescribe the row-struct and the switch.

**The false PASS.** §7.2 asserts *"Back at each step leaves `Hash` unchanged."*
When Back deletes the path, `Hash` is trivially unchanged — the path no longer
exists. The assertion cannot distinguish the required behaviour from the
destructive one, so the defect ships green.

The spec also leaves the phrase row and the hex row with **different Back
semantics** on the same screen without saying so.

**SUGGESTION.** In §5, make the loop normative: `composerHashEdit` runs
`for !ctx.Done { … }` around the pick screen; every sub-flow's Back `continue`s to
the pick screen; only the pick screen's own Back returns `false`. Say whether
`Type 64 hex` joins that contract (it should, or the screen teaches two rules).
In §7.2, assert **`len(st.list.Paths)` and the path's `Keys` are unchanged and
the `Which hash?` screen is on top** after each Back — not only that `Hash` is nil.

---

### I-2 — §4.5 puts §8h's copy in a per-path modal where its own condition is false

**Where:** §4.5, against `gui/composer_copy.go:169-173` and
`gui/composer_shape.go:441-443`.

§4.5 requires the confirm modal to carry, unconditionally, *"the F-132 line the
composer already prints when every path is hashed (§8h)"*. In the fork that string
is

```go
func composerCopyHashEveryPath() string {
	return "HASH ON EVERY PATH\n" +
		"Every way to spend this wallet needs the preimage of a hash. It is not " +
		"on this device and not on these plates. Back the preimage up separately."
}
```

and it fires at exactly one place — the **Done** arm of the spend-paths list,
guarded by `if composerEveryPathHashed(st.list)`. The composer spec's own table
(line 604) records it as `shape | every path hashed | WARNING (§8h)`.

Two things go wrong at §4.5's moment:

1. **The heading is false.** In a 2-of-3-plus-hash-path policy, one path is being
   hashed and the modal announces `HASH ON EVERY PATH`. Copy that is wrong on the
   screen where the operator commits a funds-bearing digest is worse than absent.
2. **The condition is not even evaluable.** §4.5 states that CONTINUE is what
   assigns `st.list.Paths[idx].Hash`. At the moment the modal draws, this path is
   *not* hashed, so `composerEveryPathHashed(st.list)` is false by construction on
   the last path — the modal would announce a state that the operator's next tap
   creates.

The brainstorm §4.4 does say "the §8i line, the F-132 line" for this modal, so the
spec is faithful to its source; the source is what is wrong, and the spec is the
place to fix it since §8h is a shipped, condition-guarded string.

**SUGGESTION.** Do not reuse `composerCopyHashEveryPath()` here. §4.5 needs its own
one-line record of the same duty, true for one path and true for a phrase-derived
preimage — e.g. *"The preimage is not on this device and not on these plates.
Write down the phrase and the method."* Leave §8h where it is, unchanged, firing
at Done on its own condition. Say in §4.5 that the two are different strings and
why.

---

### I-3 — the ms1 refusal names a route the device does not have

**Where:** §2 rule 3, against §6 and §9.

The refusal copy is *"That is an ms1 string, not a phrase. Load it from the
payload instead."* But H0 made a kind-`0x03` single **inert on every reader and
door**; §6 adds `DecodeMS1Preimage` and states "No screen calls it this cycle";
§9 puts "reading a preimage plate into any flow" out of scope. So a preimage
string placed in a payload reaches no screen, by the absent-is-false admission
rule. The remedy the refusal names **cannot be followed on this firmware** — and
§2's own parenthetical concedes it: "there is no device route for a preimage plate
this cycle — §6".

An operator who types 75 characters, is refused, and is told to load it from the
payload will pack it into a payload, load it, watch nothing appear, and have no
signal beyond the door's not-understood count. The host's version of this refusal
names `--in`/`-`, which *do* exist.

**SUGGESTION.** Name a remedy that exists: *"That is an ms1 string, not a phrase.
This device cannot read a preimage plate. Type the phrase, or use `Type 64 hex`
with the preimage from the host."* If the intent is to signal the future route,
say so as future tense, not as an instruction.

---

### I-4 — §7.1 does not drive the corpus rows the corpus assigns to the fork

**Where:** §7.1, against `hashlock-v0.8.json`'s own `lockstep` field.

The vendored corpus declares what the downstream port must drive:

```json
"lockstep": [
  "derivation rows: 100 and 101 characters, the spaces row, the hyphen+comma row",
  "refusals: empty, printable-ascii (TAB, DEL, 0xFF), 64-hex both cases, 101",
  "kind: the entr32 pair; id/prefix mismatch both directions (forged strings in hashlock_kind.rs)",
  "the fork's pin test drives these rows in BOTH directions (encode and decode)"
]
```

§7.1 covers the 11 derivation rows, the 100/101 rows and the 64-hex rows. It does
**not** name the `refusals` array at all, so the `empty` row and the four
printable-ASCII rows (`café`, `0xff`, `a\tb`, `0x61 0x7f`) and the ` ~` boundary
row are driven by nothing. §2 rule 2 says "the keyboard cannot produce anything
else — the test still pins it", and no section names that test.

(The keyboard claim itself is **true** — see the clean list below — but "the test
still pins it" is currently a promise with no test behind it, and the whole point
of a pin is that the keyboard's page strings can be edited.)

**SUGGESTION.** Add a bullet to §7.1: every row of the corpus's `refusals` array
is driven against `hashlock`'s rule function, asserting the row's `rule` field;
the `null`-rule rows (` ~`, `beef`) assert acceptance. Then §2 rule 2's
parenthetical has a test, and the corpus's `lockstep` declaration is satisfied
rather than partly satisfied.

---

### I-5 — the spec edits the file whose header comment it makes false, and does not fold it

**Where:** §1, §9, against `gui/composer_hash.go:27-28`.

The file H2 restructures opens with:

```go
// THE COMPOSER NEVER DERIVES, STORES OR ENGRAVES A PREIMAGE this cycle
// (§14). It takes a digest and puts it in a script.
```

H2 makes the composer derive. The spec's H3 deferral (brainstorm §4.1: "Composer
spec fold (§6c, §8 copy, §12 acceptance, §14 row, C25)") covers the composer
**spec**; it does not reach a comment inside the file the H2 implementer is
editing, and nothing in §1 or §9 mentions it. Left as-is, `gui/composer_hash.go`
ships asserting the opposite of what it does, at the top of the function that does
it — the "comments outlive their conditions" class, at the shortest possible
distance.

**SUGGESTION.** Add to §1: the file header comment at `gui/composer_hash.go:27-28`
is rewritten in this stage to state what is now true (the composer derives a
preimage on the stack, uses its digest, and still never stores, shows or engraves
one — L7), with a pointer to H3 for the composer spec's §14 row. One sentence in
§9 to say the spec *record* fold is still H3, so the two are not confused.

---

## Minor

### M-1 — `Hash` is `*[32]byte`, not `[32]byte`

§3: *"the composer stores (`st.list.Paths[idx].Hash`, a `[32]byte`)"*.
`md/compose.go:167` declares `Hash *[32]byte`; the nil pointer is what "no hash
lock" *means*, and the shipped assignment is `d := digests[sel]; …Hash = &d`
because `&digests[sel]` would alias the slice. **SUGGESTION:** say `*[32]byte`,
and state the copy-then-take-address idiom so an implementer does not alias a
temporary.

### M-2 — §3 drops the driver's signature and §4.4 repeats no forbid

The brainstorm §4.4 is more specific than the spec: *"through a NEW driver taking
`salt []byte` and the iteration count, NOT `unlockDerive`'s `seal.Header`"*. §3
keeps only "through a NEW driver"; §4.4, which is where an implementer looks when
writing the derive step, repeats no forbid at all and says only "the countdown
screen". **SUGGESTION:** restore the signature in §3 and put a one-clause
reminder in §4.4 (this is the same edit C-1 asks for, widened to the phrase).

### M-3 — §7.4 calls the acceptance plate "the corpus plate"

§7.4 says *"the corpus plate (`ms10hashsq0p7jaf…` from the acceptance record …)
→ 32 bytes equal to that row's `hardened_x`"*. `ms10hashsq0p7jaf…` is **not** in
the vendored corpus; it is from `ms-hashlock-H1-acceptance.md`. The corpus's plate
is `kind[0].ms1` = `ms10hashsqw46h2at4…`, and it carries X = `abab…ab`, not any
derivation row's `hardened_x`. (Decoded both: the acceptance plate does yield
`c3e97525…e22016`, so the *assertion* is right — only the label is wrong.) An
implementer reading "the corpus plate" takes the vendored one and writes a test
that fails. **SUGGESTION:** name them separately — the corpus row
`kind[0].ms1` → `abab…ab`, and the acceptance plate → the anchor row's
`hardened_x`.

### M-4 — §4.5's surface is not the fork's confirm surface, and §8i appears twice

The fork's confirm-to-proceed is `composerConfirmScreen` →
`ConfirmWarningScreen`, whose body convention is
`composerConfirmBody(body) = body + "\n\nHold button to confirm."` — a **hold**,
not a "CONTINUE" button. Separately, §5 makes the §8i modal fire at pick time for
the phrase row *and* §4.5 puts the §8i rule inside the confirm modal, so the
operator meets it twice in one flow; the spec does not say that is intended.
**SUGGESTION:** name `composerConfirmScreen`/`composerConfirmBody` in §4.5 and
drop "CONTINUE" for the hold; then choose one home for §8i and say which.

### M-5 — the reuse copy drops the clause that carries the reason

§4.5's reuse lines stop at *"…as a passphrase or a password anywhere else."*
Brainstorm §3.7 item 4 and `SPEC_ms_hashlock` §7 both end *"— a spend publishes
the preimage, and anyone can then test guesses at the phrase itself"*, and §7 says
"the copy is the whole defence". The budget allows it: measured against
`assertModalBodyFits` the §4.5 body is **421** normalized characters (442 with the
hold line) where the gate's measured full-draw capacity is **588** with an
80-character margin — the clause is ~85 characters and still leaves headroom over
the margin, and drops comfortably clear once §8h's 131 characters leave under I-2.
**SUGGESTION:** restore the clause; note the measured budget in §4.5 so a later
edit knows what it is spending.

### M-6 — the countdown copy is quoted two ways and the fallback is calibrated for 300,000 iterations

§3 quotes `gui/unlock_kdf.go:236` as *"Unlocking. About N seconds left."*; §4.4
requires the body *"About N seconds left."* — a different string, so a new lead
function is required and the spec does not say so. `unlockKDFLead`'s zero-sample
fallback (`unlock_kdf.go:220-222`) is the hard-coded *"Unlocking. This takes about
30 seconds."*, calibrated for the sealed payload's 300,000 iterations; at 100,000
it is wrong by 3×. **SUGGESTION:** state that §4.4 needs `hashlockKDFLead`, with
its own fallback ("Deriving. This takes about 10 seconds."), and that
`unlockTitle` becomes a parameter.

---

## Nit

### N-1 — §6's error for a shared 33-byte `0x03` string is unspecified

§6 says `DecodeMS1Preimage` accepts "ONLY an unshared string whose data is exactly
33 bytes beginning `0x03`" and that "every other input returns `errMSBadPrefix` or
`errMSBadLength`". A *share* with a 33-byte data part beginning `0x03` fails
neither test by name (`IsPreimage` rejects it on `!f.Unshared`). §7.4 asserts "a
share → error" without naming which. **SUGGESTION:** say which error the shared
case returns.

### N-2 — the phrase screen inherits the `show` reveal key (secret handling; does not gate)

`NewPassphraseKeyboard` builds a function row containing `{label: "show", action:
ppReveal}`. §4.2 does not say whether the hashlock phrase screen keeps it. L7
forbids showing a *preimage*, and the phrase is not the preimage, so this is not a
ruling violation — and per the operator's 2026-08-27 ruling secret handling never
gates. Logged for the follow-up list, with the note that the H4 walk will exercise
it in whatever room the operator is standing in.

---

## The state-machine walk

Entry: `composerHashEdit(ctx, th, st, idx)`, reached from path creation
(`composer_shape.go:269`, return value **destroys the path**) or the path menu
(`:346`, return value discarded, wrapped in `composerApplyShapeEdit`).

| step | screen / call | what is written | Back does today | Back per the spec | gap |
| --- | --- | --- | --- | --- | --- |
| 1 | `Which hash?` — `composerPickScreen(title, "Which hash?", rows)` (`composer_hash.go:149`) | nothing | `(0,false)` → `composerHashEdit` false → **at creation the path is spliced away** | §1 item 2 does not cover leaving the screen itself | none (shipped, intended) |
| 2 | §8i modal, `sel <= len(digests)` (`:157-159`) | nothing | dismiss only | §5 restates the guard as "taking a hash" (payload/phrase/hex) | none; but see M-4 (§8i also in §4.5) |
| 3 | phrase screen — `hashlockPhraseFlow` (§4.2), `NewPassphraseKeyboard` | nothing (phrase on the stack) | *new code* | "returns to `Which hash?`" | **I-1**: needs a loop; the straight-line `false` deletes the path |
| 4 | §2 refusal modals (empty / non-ASCII / ms1 / >100 / 64-hex) | nothing | back to the keyboard | same | **C-2** (rule 3 predicate), **I-3** (rule 3 remedy) |
| 5 | method pick — 2-row `composerPickScreen` (§4.3) | nothing | *new code* | "returns to `Which hash?`" | **I-1** |
| 6 | method warning modal — hardened <20 chars, sha256 always (§4.3) | nothing | declines → unstated destination | L12: confirm-to-proceed, never refusal | M-4 (surface unnamed) |
| 7 | derive — hardened on the countdown (`Deriving`), sha256 instant (§4.4) | nothing; **X on the stack** | countdown `backBtn` → `(nil,false)`; nothing assigned; `Wipe` deferred | "abandons it; the composer state is untouched" | **C-1** (the fold enters here), M-6 (copy) |
| 8 | confirm modal (§4.5): digest row, method, §8i, reuse, §8h | nothing | discards | discards | **I-2** (§8h false here), M-5, M-4 |
| 9 | **CONTINUE** | `st.list.Paths[idx].Hash = &d` — the **only** assignment in the flow | n/a | n/a | M-1 (`*[32]byte`, copy-then-address) |

**Findings from the walk that hold up:**

- There is **no state in which a preimage is stored, shown or engraved.** X exists
  on the stack between step 7 and step 9 and nothing draws it; the digest is the
  only value that reaches `st.list`.
- There is **no path on which a hash silently changes.** The `default` arm that
  clears the lock is removed by §5; the single assignment is at CONTINUE; a Back
  at any earlier step leaves an existing `Hash` pointer untouched.
- **Seats survive a Back.** Entering from the path menu under `tr`,
  `composerEditCanRenumber(…, composerFieldHash)` may fire the "EDITING THE SHAPE
  CLEARS THE KEYS" confirm first — but `composerApplyShapeEdit`
  (`composer_discard.go:144-156`) diffs `composerShapeSignature` **before and
  after** and discards only when it changed, so an abandoned hash edit discards
  nothing. §4.4's "the composer state is untouched" is true here.
- The one state-machine defect is **I-1**: the loop the spec's Back semantics
  require does not exist, and §7.2's assertion cannot see its absence.

---

## Verified clean (do not re-derive)

Machine-checked during this review; each is a claim the spec makes that holds.

1. **PBKDF2 semantics match Rust exactly.** `seal.NewDeriver`
   (`seal/pbkdf2.go:85-104`) computes `U_1 = HMAC-SHA256(P, S ‖ 00 00 00 01)`,
   sets `done = 1`, then `Step` runs `U_i = HMAC(P, U_{i-1})` XOR-accumulated
   until `done >= total`. With `total = 100_000` that is RFC 8018 `c = 100000`,
   dkLen 32 = one block, block index literally 1 — identical to
   `pbkdf2_hmac::<Sha256>(phrase, HASHLOCK_SALT, HASHLOCK_ITERATIONS, &mut x)` at
   `crates/ms-codec/src/hashlock.rs:35-39`. No block-index subtlety exists.
   Its `salt []byte` parameter takes a 14-byte slice directly; the `[16]byte`
   zero-pad is reachable only through `unlockDerive`'s `seal.Header`, which §3
   already forbids.
2. **The Rust side hashes raw bytes.** `preimage_sha256(phrase: &[u8])` is
   `Sha256::digest(phrase)`; `preimage_hardened` passes `phrase` straight to
   PBKDF2. No normalisation anywhere in `ms-codec`. `HASHLOCK_SALT =
   b"ms-hashlock-v1"` (14 bytes), `HASHLOCK_ITERATIONS = 100_000`,
   `HASHLOCK_DKLEN = 32`, `HASHLOCK_PHRASE_MAX_CHARS = 100` at
   `hashlock_phrase.rs:24` — all as §3's table states.
3. **§2's check ORDER matches `validate_phrase` clause for clause**
   (`hashlock_phrase.rs:118-143`): empty → printable-ASCII `0x20..=0x7E` → ms1
   shape → cap → 64-hex. Shape-before-cap is preserved, which is the ms R0 r0
   adversarial N-1 fix. (Only the *content* of the shape test diverges — C-2.)
4. **The 64-hex rule converges.** Host: `s.len() == 64 && hex::decode(s).is_ok()`;
   Go's `hex.DecodeString` decides per character over the same two cases, so
   lowercase, uppercase and mixed all agree.
5. **The cap agrees.** The host bounds `s.len()` in bytes; every admitted byte is
   printable ASCII, so bytes == characters == the device's `n/100` counter.
6. **The keyboard claim is exactly true.** The four `ppPages`
   (`gui/passphrase_keyboard.go:19-23`) plus the function row's
   `{r: ' ', label: "space"}` cover **95 characters — precisely
   `0x20..=0x7E`, with none missing and none extra** (computed as a set
   difference). §2 rule 2's parenthetical is sound; only the test behind it is
   missing (I-4).
7. **Adding a row cannot overflow a page.** `composerPickScreen` recomputes
   `shown` from the drawn bands each frame, shows the pager button when
   `start > 0 || shown < len(lines)`, and clamps the cursor into the page in both
   directions. `composerPickScreenMaxRows = 24` bounds the hit-area array only,
   and `Which hash?` reaches at most `len(digests)+3` rows. The two-line §4.1 lead
   appears only when `len(digests) == 0`, i.e. when the row count is smallest.
8. **§5's row enumeration is exhaustive** — `payloadRows`, `phraseRow`, `hexRow`,
   `noneRow` covers every row `composerHashEdit` builds. The prescribed `panic` on
   an unknown index is consistent with fork convention for programming errors
   (`gui/event.go:55` panics on an invalid button), not a violation of
   `seal/pbkdf2.go`'s "on a device a panic is a brick", which governs *reachable*
   bad input.
9. **§4's copy fits the gate.** Measured against `assertModalBodyFits`
   (`gui/modal_fits_test.go:201`) and its recorded 588-character full-draw
   capacity with `modalBodyMargin = 80`: §4.5's body is 421 normalized characters
   (442 with the hold line), the sha256 method modal 144, the hardened 78, the
   §4.1 no-payload hint 53. All clear the margin. No string in §4 is too long.
10. **The F-132 line is real** — `composerCopyHashEveryPath`
    (`gui/composer_copy.go:169-173`) — and today fires at exactly one place, the
    Done arm of the spend-paths list under `composerEveryPathHashed`
    (`gui/composer_shape.go:441-443`). That is what makes I-2 a finding rather
    than a misreading.
11. **§7.5's hardened literal is correct**: `h[:8]..h[56:]` of `hardened_h` is
    `3cf5d421..b70a4c12`. Only the sha256 literal is wrong (C-3).
12. **§7.4's expected value is right even though its label is wrong**: the
    acceptance plate `ms10hashsq0p7jaf…` decodes (bech32 → 5→8 regroup, checksum
    dropped) to 33 bytes, `0x03` then `c3e97525442520da4cffd5f57aae3f6273990017f2e0fa30c056e32172e22016`
    = the anchor row's `hardened_x`. The corpus plate decodes to `0x03` then
    `abab…ab` (M-3).
13. **The stage split matches the brainstorm.** §1/§9's In/Out lines agree with
    brainstorm §4.1: H2 = the fork port, H3 = the records (composer spec §6c/§8/§12/§14
    and the ms spec), H4 = the device walk. `DecodeMS1Preimage` with no caller is
    §4.1's own "separate `DecodeMS1Preimage`" ruling under L22/r2 C-2, so it is
    scope the rulings already bought, not creep — the spec's justification ("the
    kind has one decoder and one test, Rust-first") is sound, and since no
    production code references it TinyGo's linker drops it, so §7.6's "small
    delta" is not attributable to it. Worth one sentence in §6 saying so.
14. **The rulings are carried faithfully.** L5 (two methods, §4.3), L7 (derive,
    use, drop; never store/show/engrave — the walk above confirms no such state),
    L12 (both warnings confirm, never refuse), L15 (no scrub beyond construction,
    §3), L16 (§4.4 agreed), L22 (`0x03` inert, no new class — H0), L24
    (`TagKindMismatch` refused by the host, §7.4). None is contradicted.

---

## Closing counts

| severity | count | ids |
| --- | --- | --- |
| Critical | 3 | C-1 screen-layer normalisation invisible to §7.2/§7.5; C-2 ms1-shape predicate narrower than the host's; C-3 the sha256 acceptance literal is 7 hex characters |
| Important | 5 | I-1 Back needs a loop and §7.2 false-PASSes; I-2 §8h's copy in a modal where its condition is false; I-3 the ms1 refusal names a route the device lacks; I-4 §7.1 skips the corpus's `refusals` rows; I-5 the stale header comment in the edited file |
| Minor | 6 | M-1 `*[32]byte`; M-2 driver signature dropped; M-3 "the corpus plate"; M-4 confirm surface + §8i twice; M-5 reuse clause truncated; M-6 countdown copy and fallback |
| Nit | 2 | N-1 the shared-string error; N-2 the inherited `show` key |

**GATE: RED.** 3 Critical / 5 Important open. Two of the three Criticals are
host/device digest divergence paths that every test the spec defines would pass;
the third makes the stage's acceptance criterion unsatisfiable.

The two smallest edits with the largest effect are (a) driving §7.2 and §7.5 with
`Correct Horse Battery Staple` and `  a  b ` instead of the anchor phrase, which
converts C-1 from invisible to caught, and (b) restating §2 rule 3 as the host's
shape-only predicate rather than as `codex32.New`.
