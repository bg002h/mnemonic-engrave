# RULING — F-412: the origin advisory under `--path` (both tiers)

**Date:** 2026-08-28. **Capacity:** architect consult substituting for the operator, third
ruling in the F-410/F-411 line. **Repo:** descriptor-mnemonic, tip `b6d8b515`.
**Verified before ruling:** captured runs c1–c5 re-read (streams separate, all exit 0),
c6 re-executed against the built binary (silent, mints c2's card 0x8e126), c1/c5 stdout
md5-identical, and the source of `emit_unhardened_origin_note` and its two call sites
(text and `--json` branches, `crates/md-cli/src/cmd/encode.rs:116` and `:234`) read in
full. No premise in the brief was found false.

---

## THE RULE (one rule, both tiers)

**The advisory's PREDICATE reads the template's own declared-origin text and the seated
keys — nothing else. `--path` is invisible to the predicate in BOTH directions: it never
suppresses a note the template's spelling earns (c1, c3, c5), and it never triggers a
note the template did not write (c6). The ONLY `--path`-sensitivity permitted is
evidentiary WORDING: when `--path` is present and at least one tier fired, one shared
trailing line — identical for both tiers, emitted exactly once per invocation, gated on
the override's PRESENCE only, never its content — states that the override replaced the
cited spelling on the minted card and repeats the use-site-tail remedy.**

This is option A with a one-line wording cure, not option B and not option C. The two
tiers remain uniform at every level: same text read, same predicate inputs, same
trailing line, same silence rule. Nothing reads the override's content anywhere in the
function — F-411's exclusion ("its own documentation declares it supplies an origin, so
the misreading this note targets is not reachable through it") is REAFFIRMED, and this
ruling extends the same principle symmetrically: the advisory reads the template, never
the override — it neither stops reading the template when an override appears nor starts
reading the override.

**The trailing line (exact text, one `writeln!`, stderr, after all tier emissions):**

> note: --path replaced the origin declaration(s) cited above; the minted card carries
> the override, not that spelling. This note reads the TEMPLATE's own text, which --path
> supersedes but does not reinterpret: a step meant as DERIVATION is not moved to the
> use-site tail by --path — write it there (`/<0;1>/*`) if that is what you meant.

Both tiers' existing body text is UNTOUCHED. Note, never a refusal: stderr only, stdout
and exit code untouched, on the text and `--json` branches alike (automatic — the line
lives inside the shared function).

---

## Per-case disposition (c1–c6)

| case | invocation | under this ruling |
|---|---|---|
| c1 | `wpkh(@0/84'/0'/0'/0/*)` + K3 + `--path bip84` | **FIRES**: tier-2 note unchanged (citing `/84'/0'/0'/0`, the template's spelling) **+ the trailing line** |
| c2 | same, no `--path` | **FIRES**: tier-2 note, byte-identical to today; no trailing line |
| c3 | `wpkh(@0/0/*)` keyless + `--path bip84` | **FIRES**: tier-1 note unchanged (citing `/0`) **+ the trailing line** |
| c4 | `wpkh(@0/0/*)` keyless, no `--path` | **FIRES**: tier-1 note, byte-identical to today; no trailing line |
| c5 | `wpkh(@0/0/*)` + K3 + `--path bip84` | **FIRES**: tier-1 note (citing `/0`) **+ the trailing line** |
| c6 | `wpkh(@0/*)` + K3 + `--path "m/84'/0'/0'/0"` | **SILENT**, unchanged — F-411's exclusion stands; the trailing line is a suffix to a fired note, never a note of its own |

The c1/c5 asymmetry — byte-identical cards, different notes — STANDS and is coherent
under this rule: the note's subject is the spelling, the spellings differ, and both
invocations now also carry the same trailing line saying what the card actually carries.

---

## Rationale

### Q1 — is a `--path` user "speaking their own words" about the origin?

About the origin **the card carries**: yes — that is F-411, it stands, and it is why c6
stays silent. But the template's declaration is ALSO the user's own words, and its
misreadable shape is evidence of intent that the override does not erase. The decisive
measurement: **the harm the note guards is invariant under `--path`.** c1 and c2 derive
the SAME first address (`bc1qr932kkqd95r3chv9sh36wkjez4jvsmlf46xuc9` — the excess `/0`
is inert whether the card carries it or the override deleted it), and both sit one level
above the descriptor-style intent (`bc1qmxrw6qdh5g3ztfcwm0et5l8mvws4eva24kmp8m`). An
invariant harm demands an invariant note. What `--path` changes is only the note's
card-facing EVIDENCE — so only the wording changes, and only by one honest line.

### Q2 — whose invocation is c1?

**The c1 shape is practically a fingerprint of the misreading.** To an md-literate user,
declaring an origin and simultaneously overriding it wholesale is redundant; their
idiomatic spelling is `wpkh(@0/*) --path bip84`, which is silent. But to a
descriptor-thinker the c1 invocation is not redundant at all — it is their mental model
verbatim: the path after the key is "derivation", `--path` is "the origin". Template
path + `--path` together is exactly how a descriptor user expresses "origin plus
derivation steps". So option B would silence the note precisely on the invocation whose
shape is the strongest evidence FOR the misreading, and the cost lands in the
still-blocking harm class (addresses diverging from intent), while B's entire benefit is
one line of fatigue relief on an invocation a literate user had little reason to type.
The trailing line pays the literate user back directly: the tool visibly knows the card
carries their override, which cures the credibility erosion of being asked to confirm
superseded text — the real fatigue mechanism F-412 identified.

### Q3 — at what level does tier uniformity hold?

At the level of **what text the predicate reads**: template text, both tiers, always —
the status quo, now stated as a rule rather than inherited as an accident. "What the
note is about" is also uniform — the spelling and its use-site consequence — and the
wording layer may acknowledge the card's actual content without re-keying anything.

### Why not B (suppress under `--path`)

c1 and c2 are equal-harm cases with byte-identical first addresses; B notes one and
silences the other, so B is not tracking the harm — it is tracking the card, while
leaving the predicate on the template, an incoherent halfway house (c4 would note and c5
stay silent on the identical spelling `/0`). And it deletes the only line the misreading
user gets, on the invocation shape most characteristic of them.

### Why not C (re-key on the final descriptor)

(a) It fires on c6, re-ruling F-411 against itself with no new evidence — a note
second-guessing an explicit `--path` is squarely the fatigue direction. (b) It silences
c1, the fingerprint invocation, despite the invariant harm. (c) The note's whole
evidentiary basis is the spelling ("In an md template the WHOLE path after `@0` is that
key's origin declaration") and would need a rewrite to accuse a final descriptor whose
origin the operator explicitly supplied. C combines B's loss with a contradiction of a
one-day-old ruling.

### One defect the ruling does cure

Under the override, tier 2's clause "Confirm the xpub seated at @0 is the key
`/84'/0'/0'/0` names" cites text the card does not carry, and for a depth-3 key against
a 4-level path the literal confirmation can never succeed. The clause retains its intent
(did you mean to seat the depth-4 child?) so the body stays; the trailing line supplies
the missing fact — the card carries the override — so the note no longer makes an
implicit false claim about the artifact. A note whose evidence is wrong about the card
is a defect in the tool's own claims; one sentence closes it without moving the
predicate.

---

## Authorized implementation (all of it; nothing more)

1. `emit_unhardened_origin_note` gains a parameter `path_overridden: bool`. Both call
   sites (`encode.rs:116` and `:234`) pass `args.path.is_some()`. **The predicate logic
   of both tiers is byte-for-byte unchanged.** The function never receives or reads the
   override's content.
2. Restructure the tail so the trailing line can be the function's LAST emission: emit
   tier-2 lines, then tier-1's joined note, then — if `path_overridden` AND at least one
   tier emitted (`!deeper.is_empty() || !affected.is_empty()`) — the trailing line above,
   exactly once. (Note the current `if affected.is_empty() { return; }` sits between the
   tiers and would skip the line on a tier-2-only finding; that early return becomes a
   guard around the tier-1 block.)
3. Update the doc comment's final paragraph ("KEYED ON THE TEMPLATE'S OWN TEXT…") to
   record this ruling: predicate reads template text always, `--path` invisible to the
   predicate in both directions, presence-only gates one evidentiary trailing line. Cite
   this file.
4. One CHANGELOG line (matching F-411's precedent).
5. Nothing else. No stdout change, no exit-code change, no help-text change required
   (the trailing line itself is the user-facing explanation; a `--path` help sentence is
   NOT warranted — it would document an advisory interaction most users never see).

## Required tests (existing `note()` harness in `encode.rs` `mod tests`, which gains the
new parameter; add `const OVERRIDE: &str = "--path replaced the origin declaration"`)

1. `override_appends_supersession_line_keyed` — c1 shape, `path_overridden = true`:
   output contains `KEYED` and `OVERRIDE`.
2. `override_appends_supersession_line_keyless` — c3 shape, `true`: contains `NARROW`
   and `OVERRIDE`.
3. `override_does_not_suppress_either_tier` — both shapes, `true`: `KEYED` / `NARROW`
   still present. (Kills an option-B-style early-return mutant.)
4. `no_override_no_supersession_line` — c2 and c4 shapes, `false`: tier note present,
   `OVERRIDE` absent. (Inverting the gate reds this.)
5. `override_alone_is_silent` — `wpkh(@0/*)`, key seated, `true`: output EMPTY. (Pins
   c6 and pins that the trailing line is a suffix, never a standalone note.)
6. `supersession_line_emitted_once` — a two-slot template landing one slot in each tier,
   `true`: both tier phrases present, `OVERRIDE` count exactly 1. (Moving the line into
   a per-slot loop reds this.)

Per the repo's mutation-proving convention, 3/4/5/6 name their mutant explicitly.

## F-412 FOLLOWUPS entry — text to record

> **F-412 — RESOLVED (ruling 2026-08-28, architect consult; third in the F-410/F-411
> line; `design/agent-reports/RULING_f412_path_override_note.md`).** The advisory's
> predicate reads the template's declared-origin text and the seated keys, full stop —
> `--path` is invisible to it in both directions (never suppresses c1/c3/c5, never
> triggers c6), and F-411's exclusion of the override's own content is reaffirmed. The
> only `--path`-sensitivity is wording: when the override is present and a tier fired,
> one shared trailing line (both tiers, once per invocation, gated on presence only)
> states that `--path` replaced the cited spelling on the card and repeats the
> use-site-tail remedy. Decisive measurement: c1 and c2 derive the same first address —
> the excess step is inert with or without the override — so the divergence from the
> descriptor-style intent is invariant under `--path`, and an invariant harm demands an
> invariant note; only the note's card-facing evidence changes, so only wording changes.
> The c1 invocation shape (template path + `--path`) is the descriptor mental model
> verbatim ("derivation" + "origin"), i.e. the note's audience, not its fatigue case.
> Authorized: `path_overridden: bool` parameter, the fixed trailing line as the
> function's last emission, doc-comment + CHANGELOG updates, six pinned tests; predicates
> byte-for-byte unchanged. Note, never refusal: stderr only, stdout and exit codes
> untouched, `--json` parity automatic.
