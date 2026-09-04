# R0 round 1 — hashlock-phrase brainstorm, fold verification (sonnet, read-only)

**Date:** 2026-09-03
**Model:** sonnet (single agent, read-only)
**Repo HEAD at review time:** `5811044` (`git rev-parse --short HEAD`)
**Commits compared:** round-0 report persisted at `d13819e`; fold at `d2e8f68`
(`git diff d13819e..d2e8f68 -- design/BRAINSTORM_hashlock_phrase.md design/FOLLOWUPS.md`).
No further commits touched either file between `d2e8f68` and `5811044`
(`git log --oneline d2e8f68..HEAD -- <both files>` is empty).
**Brief:** `design/agent-briefs/hashlock-brainstorm-R0-r1-fold-verification-brief.md`

---

## Counts

FIXED:16 PARTIAL:1 NOT:0

C:0 I:1 M:0 N:0

---

## Per-finding table

All line numbers are in `design/BRAINSTORM_hashlock_phrase.md` at `5811044`
(identical to `d2e8f68` for these two files).

| finding | verdict | where | note |
| --- | --- | --- | --- |
| C-1 | FIXED | L12 (46); §4.2 (245-252); §5 (377) | sha256 always warns/never refuses, hardened keeps 20-char warning; matches L12 exactly. No stray "shared floor" sentence survives (grepped, see Confirmed clean). |
| I-1 | FIXED | §3.4 (120-140); L13 (47); §5 (375); FOLLOWUPS.md:15580 (F-469) | "years per GPU" explicitly withdrawn (131); 72-day figure in its place; fixed salt kept, shared-table consequence stated; F-469 filed and reads correctly. |
| I-2 | FIXED | §4.3 (315-320, 324); Vectors (350-352) | explicit length check before construction, `Error::PreimageLengthMismatch`, `try_from` instead of indexing; vector rows 16/32/34/46 — a verbatim match to the round-0 remedy's own wording ("Pin all of 16, 32, 34 and 46 payload bytes"). |
| I-3 | FIXED (see New Finding 1) | §4.2 (292-296); §4.3 (329-336); MIGRATION bullet (359-360) | Core of the finding — verify/derive refusal placed on `Ok((tag,payload))` arm before the `payload_entropy_and_language` helper, `combine` gets its own arm, MIGRATION tells downstreams to sweep — is present and correct. But §4.3's summary sentence ("each converted to a typed refusal") overclaims for 2 of the 4 sites; see New Finding 1. |
| I-4 | PARTIAL | disposition claim at line 420; actual text §4.3 (306-320), MIGRATION bullet (356-368) | id `hash` (L14), `RESERVED_ID_BLOCKLIST` gain, and "length no longer implies kind" are all stated — but only in MIGRATION.md (§4.3, lines 356-358). The disposition table's own claim is "stated in MIGRATION/spec/manual (4.3)"; grepped the whole record for "manual" (101-102, 364, 420, 425) and "ms spec" (216, 392) — neither ties to I-4's statement. "The ms spec" and "the manual" are future artifacts (§4.1 H3, not yet drafted) that the record cannot itself carry the statement into; the disposition table overclaims scope it did not fold. Minor-leaning (no ruling contradicted, no funds/design defect — a documentation-completeness overclaim in the table's own wording), but genuinely PARTIAL against its own claim. |
| I-5 | FIXED | §3.7 (172-201, four numbered consequences); §5 (386) | H-public-at-first-wsh-spend and the phrase-oracle consequence both added as claimed; card/modal copy present. |
| I-6 | FIXED | §4.2 (258-263); §5 (379) | 64-hex refusal naming `--hex`, host and device, matches. |
| M-1 | FIXED | §4.3 (321-324) | `Payload::Preimage(Zeroizing<[u8; 32]>)`, built via `try_from` after the length check. |
| M-2 | FIXED | §4.2 (279-281); §5 (380) | character count printed next to the method line. |
| M-3 | FIXED | MIGRATION bullet (360-365) | names `--method sha256` as the pre-tool recipe, and explicitly extends to "the manual chapter and F-465's `Which hash?` hint" (this one names both homes, unlike I-4). |
| M-4 | FIXED | §4.2 (276-279); also L5 (39) | reworded to "try each method that shipped with the version named on this card"; old "try both" prescriptive wording does not survive anywhere (both remaining occurrences of the string "try both" are inside quotes *naming* the superseded phrase, at 278 and 426). |
| M-5 | FIXED | §4.2 (236-240); §5 (384) | both halves stated: "nothing can be guessed... and nothing can be remembered." |
| M-6 | FIXED | §4.2 (254-256); §4.3 (352-354); §5 (381) | dedicated `HASHLOCK_PHRASE_MAX_CHARS`, explicitly not `passphrase.MaxLen`; lockstep 100/101-char vector rows. |
| N-1 | FIXED | §4.2 (295-296) | `combine` named with its arm, cited as the fourth `unreachable!` site. |
| N-2 | FIXED | §3.5 (144-152) | Liana/Sparrow no longer named as confirmed; text explicitly states "no coordinator is NAMED here until one is verified," cites `PSBT_IN_SHA256` instead. |
| reviewer Q2 | FIXED | §4.2 (245-252) | copy names a generator (six diceware words, `--random`); floor stays in characters, as the disposition says was the choice taken. |
| reviewer Q5 | FIXED | §4.2 (261) | "the device's phrase screen applies the same check" — both sides covered. |

---

## Numbers — recomputed

All commands run 2026-09-03 against the cited rates (8,865.7 kH/s at 999
iterations; 21,975.5 MH/s SHA-256).

```
$ python3 - <<'EOF'
pbkdf2_999 = 8865.7e3; sha256 = 21975.5e6
g_hardened = pbkdf2_999 * 999 / 100_000
g_sha256   = sha256 / 2
print("hardened guesses/s: %.3g" % g_hardened)
print("sha256   guesses/s: %.3g" % g_sha256)
ratio = g_sha256/g_hardened
print("ratio: %.0f x" % ratio)
for bits in (40,56):
    for name, r in (("hardened", g_hardened), ("sha256", g_sha256)):
        s = 2**bits / r / 2
        print(f"  {bits} bits {name:9s}: {s:11.4g} s = {s/86400:9.4g} days = {s/31557600:9.4g} yr")
t32 = 2**32 / g_hardened
print("2^32 seconds:", t32, "hours:", t32/3600)
import math
print("log2(7776) =", math.log2(7776))
print("6 words bits (7776-word diceware list):", 6*math.log2(7776))
EOF
hardened guesses/s: 8.86e+04
sha256   guesses/s: 1.1e+10
ratio: 124060 x
  40 bits hardened :   6.207e+06 s =     71.84 days =    0.1967 yr
  40 bits sha256   :       50.03 s = 0.0005791 days = 1.585e-06 yr
  56 bits hardened :   4.068e+11 s = 4.708e+06 days = 1.289e+04 yr
  56 bits sha256   :   3.279e+06 s =     37.95 days =    0.1039 yr
2^32 seconds: 48493.25560939986 hours: 13.47034878038885
log2(7776) = 12.92481250360578
6 words bits (7776-word diceware list): 77.54887502163469
```

Matches against the record: 8.9e4 (377 shown as 8.86e4 rounds to that) —
match; 1.1e10 — match; 124,060 ratio (line 46, 416) — match; 71.84 days →
"72 days" (128, 377) — match (nearest day); 50.03 s → "50 seconds" (128) —
match; 12,890 yr → "12,900 years" (129) — match at 3 sig figs; 37.95 days →
"38 days" (129) — match; 13.47 hr → "13.5 hours" (133) — match; 77.55 bits
→ "~77 bits" (six diceware words, line 138) — the record's figure is 0.55
bits low of the exact 12.9248-bit/word value, but it is stated with a tilde
as an approximation and the gap changes no conclusion in the record; not
flagged as a wrong number.

```
$ python3 - <<'EOF'
import math
ENTR=[16,20,24,28,32]
sl=lambda b: 9 + math.ceil(b*8/5) + 13
entr={sl(n+1) for n in ENTR}; mnem={sl(n+2) for n in ENTR}; pre={sl(33)}
print("entr :", sorted(entr))
print("mnem :", sorted(mnem))
print("preim:", sorted(pre))
print("payload bytes reachable: %d .. %d" % (26*5//8, 74*5//8))
EOF
entr : [50, 56, 62, 69, 75]
mnem : [51, 58, 64, 70, 77]
preim: [75]
payload bytes reachable: 16 .. 46
```

Matches the record's length sets (50/56/62/69/75, 51/58/64/70/77, 75,
§4.3 line 307) and "BIP-93's bracket reaches 16..46 payload bytes"
(Vectors bullet, line 351) exactly.

---

## New findings

### 1. §4.3's "each converted to a typed refusal" is false for 2 of the 4 `unreachable!` sites it names

**Severity:** Important.

**Claim.** §4.3's I-3 paragraph states: *"The H1 plan carries the four as an
explicit checklist, each converted to a typed refusal with one test per
site"* (lines 335-336), naming `cmd/combine.rs:166`, `cmd/decode.rs:107` and
`:112`, `cmd/payload_lang.rs:61` as the four sites. But §4.2, unchanged by
this fold and immediately above in the same document, states the opposite
for `decode`: *"`decode` prints kind, preimage hex and digest, never words"*
(line 290-291) — i.e. `decode` must functionally **handle** the new
`Payload::Preimage` kind, not refuse it.

**Evidence.** Read `crates/ms-cli/src/cmd/decode.rs` in mnemonic-secret
(`7fc1e58`): its entire `run()` function calls `ms_codec::decode(&ms1)?`
once, then falls straight into the two-arm match containing the cited
`unreachable!` sites (lines 107, 112) to extract entropy for BIP-39
word-printing. There is no other branch in the file. For `decode` to satisfy
§4.2 ("prints kind, preimage hex and digest, never words"), a
`Payload::Preimage` arm at this match point must produce hex/digest output,
not a refusal — under neither of the two plausible implementations (an
earlier explicit branch that never reaches these arms, or converting the
arms themselves to a value-producing match) does "converted to a typed
refusal" correctly describe `decode.rs:107`/`:112`. It correctly describes
only `combine.rs:166` (which should refuse, per §4.2's "encode --hex stays
entr, so ms hashlock is the only door") and `payload_lang.rs:61` (reached
only from `verify`/`derive`, which do refuse per §4.2).

**Remedy.** In §4.3's I-3 paragraph, split the checklist: `combine.rs:166`
and `payload_lang.rs:61` become typed refusals (as stated); `decode.rs:107`
and `:112` gain a `Payload::Preimage` arm that prints hex+digest instead —
not a refusal. This is a one-sentence text fix, not a design change; §4.2's
correct instruction does not need to move.

---

## Confirmed clean

- No stale "shared floor / warns on stderr under 20 characters for both
  methods" sentence survives — grepped `"Under 20 characters"`, zero hits.
- No stale prescriptive "try both" survives — the only two occurrences of
  that string (278, 426) are inside quotes naming the superseded M-4
  phrasing, not live instruction text.
- No stale unqualified "years per GPU" claim survives — all three
  occurrences (131, 417, 455) explicitly state it was wrong and withdrawn.
- No bare (unwrapped) `[u8; 32]` claim for `Payload::Preimage` survives —
  the three occurrences of the literal string are the correct
  `Zeroizing<[u8; 32]>` type (321), the `try_from` call (324), and
  rust-miniscript's distinct `Preimage32` type citation (71), which is a
  different, correctly-attributed type.
- §3.5 names no coordinator (grepped "coordinator", 4 hits, all either
  citing the PSBT field generically or explicitly stating "no coordinator
  is NAMED here until one is verified") — matches N-2's disposition.
- All four `_ => unreachable!` sites cited in §4.3 (`combine.rs:166`,
  `decode.rs:107`, `decode.rs:112`, `payload_lang.rs:61`) exist verbatim in
  mnemonic-secret `7fc1e58` (`grep -rn '_ => unreachable' crates/ms-cli/src`
  = exactly these 4).
- The two caller sites cited (`verify.rs:99`, `derive.rs:434`) exist and are
  the `Ok((_tag, payload)) => crate::cmd::payload_lang::payload_entropy_and_language(...)`
  call in each file, matching the record's claim about where the refusal
  must be placed.
- `crates/me-cli/src/seal/wire.rs:13` is `SALT_LEN: usize = 16` and `:17` is
  `MIN_ITERATIONS: u32 = 100_000`, exactly as cited.
- `SPEC_wallet_policy_composer.md` §4a (lines 115-116) states `sh(wsh)` is
  admitted "ONLY a single path that is an unlocked, unhashed key set" —
  matches §3.7 consequence 3's citation exactly.
- F-469 exists in `design/FOLLOWUPS.md` (line 15580) with the `2**32 /
  8.86e4 = 48,470 s` arithmetic reproducing and the ruling ("no flag this
  cycle") correctly cross-referenced to L13.
