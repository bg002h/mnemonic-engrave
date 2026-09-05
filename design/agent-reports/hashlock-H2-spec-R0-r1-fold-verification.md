# R0 round 1 — fold verification on `design/SPEC_hashlock_H2_device.md`

Reviewer: independent (sonnet, targeted). Fold under test: `60a86f6`, over draft
`bfd042e`, responding to `hashlock-H2-spec-R0-r0-fidelity.md` (`3f88280`,
3C/5I/6M/2N) and `hashlock-H2-spec-R0-r0-journey.md` (`a70f950`, 2C/6I/5M/1N).
Read-only throughout on all three repos (`mnemonic-engrave`, `seedhammer` at
`c4a64fc`, `mnemonic-secret` at `cd0a60f`); nothing committed or modified in any
of them; no scratch copies or worktrees; no sub-agents; no `.jsonl` read. One
throwaway Go program was compiled and run in `/tmp` (outside all three repos) to
execute `seal.NormalisePassphrase`'s exact algorithm against specific strings —
that is machine verification of a claim, not a repo edit.

**ONE QUESTION: did the fold address every Critical and Important from both
reports, without introducing a contradiction or a false claim of its own?**
**Answer: mostly yes, with one round-0 Important only partially closed and two
new Important-severity defects introduced by the fold itself.** Both round-0
journey Criticals and two of three fidelity Criticals are cleanly fixed; the
third (C-1) is fixed for its two originally-required test rows but the fold's
own restatement of a mutation test against a third, self-added row is false.
Gate: **NOT GREEN.**

---

## Finding table — Criticals and Importants (16)

| id | source | verdict | why |
| --- | --- | --- | --- |
| C-1 | fidelity | **FIXED** | §2 names the forbidden mechanism (`seal.NormalisePassphrase`, plus `TrimSpace`/`Fields`/`ToLower`/`ToUpper`/Unicode normalisation), citing `sysw/open.go:55` and `seal/open.go:231`. §7.1/§7.2/§7.5 now drive `Correct Horse Battery Staple` and `  a  b `, both machine-confirmed non-fixed-points of `seal.NormalisePassphrase` (see Executed checks). The fold's own §7.1 sentence also claims a third row, `correct-horse,battery staple`, fails under this exact mutation — that specific sub-claim is **false** (see NF-A); it does not undo C-1's core fix. |
| C-2 | fidelity | **FIXED** | §2 rule 3 now restates `looks_like_ms1`/`is_ms1_shaped` clause for clause: trim, lowercase, strip separators, `MIN_MS1_LEN`, `ms1` prefix, bech32 charset, no checksum — confirmed against `argv_guard.rs:148-164` and `format.rs:12-14` (`is_display_separator` = whitespace, `-`, `,`) at `cd0a60f`. The five ms1-shaped `refusals` rows the spec says exist (lowercase/UPPERCASE/grouped-by-5/padded/grouped-by-2) are exactly the corpus's five `rule: "ms1-shaped"` rows — confirmed by parsing the corpus JSON. Minor wording nit only: §2 says "strip the display separators (space, `-`, `,`)" where the host strips *any* whitespace, not just the space character — moot in practice since rule 2 already restricts input to printable ASCII, so no other whitespace variant can reach rule 3 on this device. |
| C-3 | fidelity | **FIXED** | Recomputed directly from the corpus's anchor row: `sha256_h[:8]` = `b867db87`, `sha256_h[56:]` = `edbc96cb` → `b867db87..edbc96cb`, matching §7.5 and §8 exactly. |
| C-1 | journey | **FIXED** | §4.5 carries the backup line ("Write down this phrase and the method now...") unconditionally; §4.7 gives §8h a phrase-route form ("Back up the phrase and its method, or the preimage plate, separately") and keeps it at Done only, where its `composerEveryPathHashed` guard is actually true (this also resolves fidelity I-2's "false heading" complaint, since the heading can no longer fire while untrue). |
| C-2 | journey | **FIXED** | The relation line (`matches hash <i> in the payload` / `no hash: record...`, omitted with no records) and the reconciliation line ("run ms hashlock with this phrase and method... and check the digest matches") are both present in §4.5 exactly as suggested. |
| I-1 | fidelity | **FIXED** | §4.6 states the loop normatively: `false` only at `Which hash?` itself; §7.2 requires tests through `composerAddPath` (the creation entry point) asserting the path still **exists**, not only that `Hash` is unchanged — the exact class of false-PASS the round-0 report demonstrated. |
| I-2 | fidelity | **FIXED** | §4.7 moves §8h out of the per-path modal entirely, back to its original Done-only guard, so its "HASH ON EVERY PATH" heading can never fire while false. |
| I-3 | fidelity | **FIXED** | The ms1 refusal now names a route that exists: "On the host, run ms hashlock with it and load the hash: record it prints." |
| I-4 | fidelity | **FIXED** | §7.1 requires every `refusals` row (15, confirmed by parsing the corpus), the `kind` row, and states the corpus's `lockstep` array drives coverage in both directions. |
| I-5 | fidelity | **PARTIAL, claimed FIXED** | The code-comment half is addressed: §1 and the opening paragraph commit to rewriting `gui/composer_hash.go:27-28` when H2 is implemented. But the round-0 suggestion's second, explicit ask — "one sentence in §9 to say the spec record fold is still H3, so the two are not confused" — is **absent**; instead §1 item 5 lists "the two records above" as an H2 in-scope item, and the opening paragraph's present-tense "Both records are folded to say exactly that" reads as already done. I checked `design/SPEC_wallet_policy_composer.md:386` directly: it is **unchanged**, still reading "The composer never derives, stores or engraves a preimage this cycle (§14)" — false once H2 ships. This reopens exactly the ambiguity I-5 was raised to prevent, and contradicts the H2/H3 stage split fidelity's own round-0 "Verified clean #13" confirmed was correct. See NF-C. |
| I-1 | journey | **FIXED** | Same clause as fidelity I-3 (shared refusal string). |
| I-2 | journey | **FIXED** | Phrase screen gains a lead ("Use a phrase you have never used anywhere else"); both method modals gain "If you have used this phrase anywhere else, press Back and choose another." Confirmed against `gui/gui.go:656-679`: `ConfirmWarningScreen`'s decline button (`cancelBtn`) is in fact rendered with `assets.IconBack` — the copy's instruction to "press Back" is mechanically accurate, not a misnomer. |
| I-3 | journey | **FIXED** | "Even a 20-character phrase falls in about 72 days on one GPU, and shorter ones fall sooner..." replaces the misleading threshold-only statement. |
| I-4 | journey | **FIXED** | Same as fidelity I-1: §4.6's loop, §7.2 through `composerAddPath`. |
| I-5 | journey | **FIXED** | §4.3 states explicitly: "Declining either modal returns to the method pick with the phrase intact." |
| I-6 | journey | **FIXED** | §4.7 removes §8h from the per-path confirm modal, keeping it only at Done where the predicate is true. |

**Criticals: 5/5 substantively fixed** (one, C-1, carries a new adjacent defect
in its own test description — NF-A). **Importants: 10/11 fixed, 1 partial**
(fidelity I-5). **Two new Important-severity defects** (NF-A, NF-B) were
introduced by the fold and are not responses to any round-0 finding.

---

## Executed checks

**Fidelity C-1 (non-fixed-point rows), machine-verified in Go** (compiled and
run in `/tmp`, replicating `seal.NormalisePassphrase` from `seal/open.go:76-78`
exactly):

```
input="correct horse battery staple" normalised="correct horse battery staple" fixed_point=true   (anchor — expected)
input="Correct Horse Battery Staple" normalised="correct horse battery staple" fixed_point=false  (as claimed)
input="  a  b "                      normalised="a b"                          fixed_point=false  (as claimed)
input="correct-horse,battery staple" normalised="correct-horse,battery staple" fixed_point=true   (NOT as claimed)
```

The corpus (`crates/ms-codec/tests/vectors/hashlock-v0.8.json` at `cd0a60f`)
does contain `correct-horse,battery staple` as `derivation` row 9 (confirmed by
parsing the JSON), so citing it as a real corpus row is fine, and it is a
legitimate row for the `lockstep` array's "the hyphen+comma row" requirement.
The defect is narrower and sits in §7.1's own prose:

> **NF-A (Important, new).** §7.1 states the mutation test as: "fold the phrase
> through `seal.NormalisePassphrase` before deriving → the `Correct Horse
> Battery Staple`, `  a  b ` and `correct-horse,battery staple` rows fail." As
> shown above, `NormalisePassphrase` leaves `correct-horse,battery staple`
> **unchanged** — that specific mutation would **not** make this row fail. This
> is exactly the "test that cannot fail on what it names" class item 5 of the
> brief asks about: an implementer reading §7.1 believes this row adds a third
> independent trip-wire against a passphrase-normalising fold; it does not (it
> would catch a *different* mutation — one that strips separators — which is
> how §2's own framing of the same row, "(the separators a plate would
> strip)," correctly hints at it). §2's sentence and §7.1's sentence disagree
> about what this row is for, and §7.1's is the one that's wrong. Does not
> undo C-1's core fix, since the two originally-required rows (`Correct Horse
> Battery Staple`, `  a  b `) are genuinely non-fixed-points and correctly
> catch the defect C-1 describes.

**Fidelity C-2** (host predicate), verified against `mnemonic-secret` source at
`cd0a60f`: `argv_guard.rs:148-164` (`looks_like_ms1`/`is_ms1_shaped`,
`MIN_MS1_LEN = 48` at `:103`) and `format.rs:12-14` (`is_display_separator` =
`c.is_whitespace() || c == '-' || c == ','`). Clause-for-clause match confirmed.
The corpus's five `rule: "ms1-shaped"` refusal rows (lowercase, UPPERCASE,
grouped-by-5-with-spaces, padded, grouped-by-2/112-chars) were parsed directly
from the JSON and match the spec's description exactly.

**Fidelity C-3**: `sha256_h` of `derivation[0]` (the anchor row, `correct horse
battery staple`) parsed directly from the corpus JSON is
`b867db875479bcc0287352cdaa4a1755689b8338777d0915e9acd9f6edbc96cb`;
`[:8]`/`[56:]` = `b867db87`/`edbc96cb`, matching the spec's literal exactly.

**Journey C-1/C-2**: read §4.5 and §4.7 directly (quoted in the finding table
above); both the backup line and reconciliation line are unconditional, the
relation line is conditional on payload records exactly as the journey report
asked, and §8h's phrase-route form and Done-only placement were confirmed
against `gui/composer_shape.go:443`/`composerEveryPathHashed` (fork citation
already re-grepped by the controller per the brief).

**The Back contract (§4.6), item 4**: grepped every "Back" mention across the
whole file (§1, §3, §4.2, §4.3, §4.4, §4.5, §4.6, §4.7, §7.2 — 15 occurrences).
Every one is consistent with §4.6's single normative statement: §1 only points
to §4.6 rather than restating it; §4.2 ("Back returns to `Which hash?` and
drops the phrase"), §4.3 ("declining... returns to the method pick with the
phrase intact"), §4.4 ("returning to the method pick with the phrase intact"),
and §4.5 ("Back returns to the method pick with the phrase intact") all agree
with §4.6 exactly. No leftover "Back discards" or "returns to Which hash?" from
a screen §4.6 sends elsewhere. **No contradiction found.**

**New contradiction, item 5 (I-5's scope claim)**: checked
`design/SPEC_wallet_policy_composer.md:386` directly — still reads "The
composer never derives, stores or engraves a preimage this cycle (§14)."
The H2 device spec's opening paragraph claims "Both records are folded to say
exactly that," present tense; this one demonstrably is not. See NF-C below and
the I-5 row above.

> **NF-C (fidelity I-5, reclassified PARTIAL — see Severity).** §1 item 5 lists
> "the two records above" as an H2 in-scope item with no H3 pointer, and the
> opening paragraph's present-tense claim is false for the composer-spec
> record as of this commit. The round-0 suggestion's specific ask (a
> clarifying "still H3" sentence) is absent from both §1 and §9.

---

## Modal-fit measurement (item 3)

**The cited constants are wrong, and one is not a constant at all.**

- `grep -rn "588" --include=*.go .` inside `seedhammer` at `c4a64fc` returns
  exactly one hit in the whole repo: a comment in `gui/modal_fits_test.go:32`
  describing ONE historical measurement ("Measured 2026-08-16 on
  `sh2DisplaySize` with 5-to-7-letter filler, both modal shapes drew 588
  normalized characters in full") of a *different* rendering scenario (generic
  filler prose, not this spec's copy). **`composer_copy_test.go` contains
  neither `588` nor `modalBodyMargin` anywhere** — grepped directly, zero hits.
- The only real, portable constant is `const modalBodyMargin = 80`
  (`gui/modal_fits_test.go:51`), used by `assertModalBodyFits`.
  `assertModalBodyFits` does **not** check a body against a fixed capacity
  minus margin; it renders the SPECIFIC body via `firstModalFrame`, then
  computes headroom for **that exact text** via `modalHeadroom`'s binary
  search over appended filler — a check that is deliberately wrap-sensitive
  (the file's own comment: "capacity depends on how the words WRAP, not on how
  many there are... which is why this measures each body instead of budgeting
  all of them"). This is precisely the "18-character unbreakable [hash] token"
  risk the journey lens flagged, and it means no fixed "~508 effective budget"
  exists to check §4.5 against.
- **NF-B (Important, new).** §4's preamble and §10's citation table both state
  "the modal-fits gate | `gui/composer_copy_test.go` (normalised capacity 588,
  margin 80)". This is new in this fold — grepped `bfd042e` for `588` and
  `composer_copy_test`, zero hits in either case; the citation did not exist
  pre-fold. It is factually wrong on the file (588 is in `modal_fits_test.go`,
  not `composer_copy_test.go`) and wrong in kind (588 is not a capacity
  constant the gate enforces; it is a one-off measurement of unrelated text).

Applying the SAME "measured length vs. ~508 effective budget" heuristic both
round-0 reports used (not a repo-verified computation, but the only number
available pending an actual render), I computed §4.5's longest variant
directly from the current spec text (normalizing case and all whitespace, per
`normalizeDrawn` in `modal_fits_test.go:60-71`), using the longer of the two
relation-line variants and `chars: 100`:

```
hash  b867db87..edbc96cb                                          22
method: hardened   chars: 100                                     24
no hash: record in the payload has this digest                    38
Write down this phrase and the method now. They are
not on this device and not on your plates. Without
both, this path can never be spent.                              112
One phrase per policy. ... test guesses at the phrase itself.    194
Before you fund this wallet, ... matches.                         94
                                                          total = 484 normalized characters
```

Under the (unverifiable, likely-wrong) 588/80 heuristic, that leaves only
**~24 characters of headroom** — far tighter than round 0's own measurements
of the smaller pre-fold body (290/421/442/494, all comfortably clear). §7.2's
"the confirm modal fits with the relation line present and `chars: 100`" is a
well-formed, runnable test requirement (it can fail), but the spec's own
narrative claim that it *does* fit rests on a citation that is wrong about
which file holds the numbers and wrong about there being a fixed capacity at
all. Nobody — round 0, this fold, or this review — has actually rendered this
exact text against the real harness. **Not settled.**

---

## Minors and Nits (14)

| id | source | verdict |
| --- | --- | --- |
| M-1 | fidelity | FIXED — §3 now says `*[32]byte`, cites `md/compose.go:167`. |
| M-2 | fidelity | PARTIAL — §3 restores the driver's signature in full; §4.4 does not repeat the one-clause forbid reminder the suggestion also asked for. Non-blocking (Minor). |
| M-3 | fidelity | FIXED — §7.4 now says "the acceptance record's plate," not "the corpus plate." |
| M-4 | fidelity | PARTIAL — the "§8i shown twice" half is fixed (§4.7: fires once, at pick). The "name `composerConfirmScreen`/`composerConfirmBody`, HOLD not CONTINUE" half is not: §4.5 still says "**CONTINUE** sets..." with no mention of the shipped hold-to-confirm gesture. The commit message's "M-1..M-6 FIXED" overclaims here; the spec's own closing paragraph is more careful, crediting M-4 only with "§8i once." Non-blocking (Minor). |
| M-5 | fidelity | FIXED — §4.5 restores the full reuse clause including "a spend publishes the preimage, and anyone can then test guesses at the phrase itself." |
| M-6 | fidelity | FIXED — §3/§4.4 consistently quote "Deriving. This takes about 10 seconds." as the zero-state lead. |
| N-1 | fidelity | FIXED — §6 states a share returns `errMSBadPrefix`; §7.4 adds the corresponding test row. |
| N-2 | fidelity | RECORDED — correctly left as non-gating per the operator's secret-handling ruling; §4.2 notes it explicitly. |
| M-1 | journey | FIXED — §4.1's lead now offers both routes ("Type a phrase below, or make one with ms hashlock on the host"). |
| M-2 | journey | FIXED as a requirement (§7.2 asserts the longest variant) — see NF-B for why the underlying fit claim is unverified. |
| M-3 | journey | FIXED — same fix as fidelity M-6. |
| M-4 | journey | FIXED — §4.4: "A power loss ends the composition, as it does at any other point in this flow." |
| M-5 | journey | FIXED — §4.5's method line now carries `chars: <n>`. |
| N-1 | journey | FIXED within H2's scope — §8i no longer appears in the confirm modal (fires once, at pick). The third occurrence the journey report noted, at the composer's existing consent screen, is pre-existing general composer behavior outside H2's scope (fires whenever any branch has a digest, unrelated to the phrase route), not something H2 can or should touch. |

---

## Closing counts

| category | count |
| --- | --- |
| Round-0 Criticals fixed | 5 / 5 (fidelity C-1's fix is sound for its two required rows; a new adjacent defect in its own test description is NF-A, tracked separately) |
| Round-0 Importants fixed | 10 / 11 (fidelity I-5 partial) |
| Round-0 Minors fixed | 9 / 11 (M-2, M-4 fidelity partial; non-blocking) |
| Round-0 Nits fixed/recorded | 3 / 3 |
| New Important findings (this round) | 2 (NF-A, NF-B) |
| New Critical/Important findings escalated by the "claimed FIXED but not" rule | 1 (fidelity I-5) |
| New contradictions in normative text | 0 beyond the I-5 scope claim (Back contract, §5, §8i/§8h placement all checked consistent) |

**GATE: NOT GREEN.** Three items block: fidelity I-5 (claimed FIXED, actually
partial — the composer-spec §14 record is unedited and no H3-deferral sentence
exists to say so, contradicting the established H2/H3 split), NF-A (a false
mutation-test claim in §7.1 about `correct-horse,battery staple`), and NF-B (a
factually wrong citation for the modal-fits gate's constants, which leaves the
confirm modal's fit unverified against the real, per-body, wrap-sensitive
check). All three are narrow, single-paragraph fixes: (a) correct §7.1's
mutation sentence to attribute the third row to separator-stripping, not
`NormalisePassphrase`, matching §2's own framing; (b) fix the citation to
`gui/modal_fits_test.go` and drop the false "capacity 588" framing in favor of
stating only the real constant (`modalBodyMargin = 80`) and that the actual
check is per-body; (c) either add the "still H3" sentence to §9 (matching the
original suggestion) or, if H2 now genuinely intends to edit
`SPEC_wallet_policy_composer.md`'s §14 row itself, say so explicitly and make
that edit part of this stage's actual deliverable rather than an unfulfilled
present-tense claim.
