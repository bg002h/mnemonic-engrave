# R10 — Decision gaps in IMPLEMENTATION_PLAN_mt_v0_1.md

**Question asked:** not "is the plan correct," but "where does this plan leave
a decision the single implementer agent would have to invent, silently,
because the plan does not tell them what to do?"

**Method.** Walked P0–P6 line by line against `design/SPEC_mt_v0_1.md`
(read in full, all ~3,755 lines, via targeted section reads) and against the
R8 agent reports the plan cites, to confirm whether a given gap is actually
settled elsewhere before reporting it. `./scripts/plan-cite-check.sh`'s clean
result is taken as given — no citation/formatting findings below. The four
explicitly-carried-open items (flag spellings, refusal-message format, exit
codes beyond 0, repo creation) are excluded per the brief.

**Result: 3 Important, 2 Minor.** No Critical-scale findings — nothing here
would corrupt the wire format or misencode a transaction; the gaps are in
process (who builds the load-bearing test artifact, and when) and in a few
under-specified behaviors/structures the plan asserts are fully ruled but
are not, quite.

---

## Important

### I-1 — The pinned byte-exact vector has no owning phase, and does not exist yet

**Lines 97–102** (§1, item 1):

> "**A spec-authored, independently derived PINNED BYTE-EXACT VECTOR** — a real
> signed segwit transaction to its exact `mt1` strings, plus a 13-symbol
> checksum micro-vector, with the generator script committed. **It lands in the
> spec before `mt-codec`'s first commit**, so the implementation is checked
> against bytes it did not produce. This is the load-bearing item; everything
> else is a tripwire."

**Lines 158–166** (P0 deliverable):

> "The **spec and its pinned vector are copied into `mnemonic-transaction`**
> — `design/SPEC_mt_v0_1.md` and the vector file — because P5's exhaustiveness
> gate and P6's journeys read them and **no phase put them there** (R8
> coverage I-12). They are copied with the commit SHA they came from recorded
> alongside..."

**Lines 237–244** (P1, tests first, item 1): "The pinned byte-exact vector
(§1) — asserted against **before** any other test is written, because it is
the only artifact here `mt-codec` did not produce."

**The decision the implementer would have to make.** P0 is written as a
*copy* step — it assumes the vector file already exists somewhere for
`mnemonic-transaction` to copy it from. It does not. I checked:

- `grep -n "13-symbol\|micro-vector\|generator script\|pinned" design/SPEC_mt_v0_1.md`
  — the only "13-symbol" hit is unrelated (§12.12's BCH-fill discussion); no
  vector data, no reference-script name, no frozen `mt1` strings appear
  anywhere in the spec text.
- `find design -iname "*vector*"` in this repo — no such file, under
  `design/` or `design/measurements/`.
- §10.13(a) and §12.22 (the NUMS-constant sections the vector is meant to
  defend) contain only the derivation of the *constant* (`0x1a2fc877f9528d7c1`
  from `SHA-256("shibbolethnumstransaction")`), never a transaction-to-strings
  vector.

So when the implementer reaches P0, there is nothing to copy. Worse, this is
a regression from the plan's own source material:
`design/agent-reports/R8-fable-ruling-nums-defence.md` (the ruling §1's D1
defence is built from) is far more concrete than what the plan carried
forward — it names the transaction shape ("one fixed, minimal signed
transaction (1-in-1-out, dummy key)"), the reference-script pattern
(`scripts/mt-reference-vector.*`, which "MUST take the constant, HRP, and
header layout **from the spec text, never from any codec crate**"), the exact
test names (`spec_vector_byte_exact`, `checksum_micro_vector`), and an
explicit ordering ("Order of work," step 2: *"Write the reference script;
generate and freeze the D1 vector + micro-vector into the spec; commit script
and vector together"* — scheduled **before** P0's "first commit" milestone).
None of that — not the transaction shape, not the script name, not who
writes it or when relative to P0 — survived into the plan. `grep`-ing the
plan for `"1-in-1-out"`, `"dummy key"`, `"reference script"`,
`"scripts/mt-"` returns nothing.

**Why this matters more than an ordinary missing fixture.** The whole
argument for this vector (§1, and D1 in the fable ruling) is that it must be
**independently derived** — computed without using `mt-codec`, so it can
falsify a self-consistent-but-wrong implementation. If the single implementer
reaches P0/P1, finds no vector, and (reasonably, since the plan gives no other
instruction) writes one by running their own not-yet-reviewed `mt-codec`
against a transaction of their choosing, the vector stops being independent —
exactly the "corpus can be uniformly wrong" failure the plan itself invokes
(line 271) to justify the vector's existence. The one artifact the plan calls
load-bearing is the one thing no phase is tasked with actually building.

**What the plan should say instead.** A phase (P0, or an explicit P-1/"before
P0" step) that: (a) names the fixed transaction (or points at the fable
ruling's "1-in-1-out, dummy key" spec and commits to it), (b) tasks someone
with writing `scripts/mt-reference-vector.*` against the spec text only, (c)
states this must complete and be frozen into `SPEC_mt_v0_1.md` **before**
`mt-codec`'s first commit, matching the ruling's own ordering, and (d) only
*then* has P0 copy the result into the new repo.

### I-2 — The TO free-text label's overflow behavior is not ruled, and P2 is told to build the path without it

**Lines 280–285** (P2, input paths):

> "an input path for every operator-supplied value §10.10 requires — R8
> coverage I-5. Per-input values (§8.2c), the `FROM`/`TO` identities, the
> free-text `TO` label behind its own flag (§10.4), and the node location.
> ... Flag *spellings* remain open; the *paths* are P2's to build"

I checked §12.4 (what §10.10/§10.4 point to for the TO label, since the
plan's own §4 item 3 says §10.13 "is marked RULED" but is silent on §10.4's
status beyond calling it a flag-spelling matter). §12.4 says, verbatim:

> "**Still to specify (§10.10's CLI work, not a design question):** the flag's
> name, and **what `mt` does with a label too long for the field** — §5's
> budget gives `TO` 34 characters including the amount, so a label has
> roughly 16. Refusing with the limit named fits §8's rule that every refusal
> names its number; **silent truncation does not**."

This is not a flag-spelling question (the carve-out this review is told to
skip) — it is a behavioral fork: refuse when the label is too long, silently
truncate, or accept unbounded length and let the stderr legend suggestion
overrun its stated 34-character/6-line budget (§5, lines 1660–1662, "**152
characters, 6 lines** — measured"). The spec leans toward refusing (the
quoted sentence explicitly disfavors silent truncation) but never states a
ruling, and the plan's P2 bullet only tasks building "the path" (wiring the
flag), not what happens when the supplied value is too long. P5's refusal
list is likewise silent — I grepped the plan for "too long"/"16 char" and
found nothing, so it's unclear whether this is meant to be one of P5's
numbered `tests/refusals.toml` entries at all.

**The decision the implementer would have to make.** Whether an over-length
`--to-label` (or whatever spelling P2 lands on) is refused with a message
naming the limit, silently truncated, or accepted unbounded — three different
behaviors, one of which (silent truncation) is the one the spec's own
reasoning argues against.

**What the plan should say instead.** Add to P2's bullet list: "a
too-long `TO` label (> the budget stated in §5, 34 chars minus the rendered
amount) is refused, naming the limit — silent truncation is out, per §12.4,"
and add the corresponding entry to P5's refusal set so the exhaustiveness
gate covers it.

### I-3 — `tests/refusals.toml`'s schema and how `mutate-refusals.sh` locates "the named check" are both unspecified

**Lines 451–465** (P5, tests-first / mutation discipline):

> "P5 commits `scripts/mutate-refusals.sh`, which for each entry **comments
> out the named check**, runs **only that refusal's test**, and asserts it
> goes **red**..."

**Lines 490–496** (P5, exhaustiveness fix):

> "P5 commits `tests/refusals.toml` in `mnemonic-transaction`, **one entry per
> v0.1 refusal with its spec §-reference and its test name**, and the script
> asserts a **bijection between that file and the tests that exist**."

The plan states the toml file's schema has exactly two fields per entry:
spec §-reference and test name. Neither of those identifies *where in the
implementation's source* the corresponding check lives, yet
`mutate-refusals.sh` is required to mechanically "comment out the named
check" per entry. A spec §-reference and a test name tell a human where to
look; they do not give a script anything to `sed` against. The implementer
therefore has to invent a third piece of information — a source
file:line, a function/marker-comment convention (e.g. `// REFUSAL: 2b`
sprinkled through the refusal-checking code), or a regex — and decide whether
it lives in the toml (a third column) or in the source as a paired marker.

**Why this is Important and not a shrug.** The plan spends two paragraphs
(lines 462–465) on exactly the failure mode a wrong choice here produces:
*"The script must fail loudly if a mutation does not apply. A `sed` that
matches nothing leaves the code intact, the test passes, and the run reports
success — a vacuous control, which has already happened twice in this cycle
alone."* The "assert it changed the file" check catches *a* wrong mechanism
(a sed pattern matching nothing), but does not tell the implementer what
mechanism to build in the first place — two different implementers given
this plan could build a fragile per-entry regex-in-toml scheme and a
robust marker-comment scheme, and only one of those survives a refactor of
the refusal-checking code without silently going vacuous again.

**What the plan should say instead.** Pin the mechanism: e.g., "Each
`tests/refusals.toml` entry carries a `marker` field naming a `// REFUSAL:
<marker>`-tagged line the check lives behind in source; `mutate-refusals.sh`
greps for that exact marker comment and comments out the line(s) immediately
following it, asserting the marker was found before running the test." Any
concrete, greppable convention works — the point is that the plan currently
names none.

---

## Minor

### M-1 — §10.20's required caveat sentence still has no owning phase

Plan §4 item 3, **line 578**: "**Where the remaining spec open questions
land.** §10 holds §10.10, §10.13, §10.14 and §10.20." The body that follows
(lines 581–600) discusses §10.10 at length and notes §10.13 is "RULED rather
than open," but never returns to §10.20 — it is named in the opening
sentence and then dropped. §10.20 (spec lines 3384–3391) is *not* marked
SETTLED (unlike §10.21–§10.23 right below it) and still reads: "Worth a
sentence somewhere a recoverer will read" — i.e., the spec itself has not
picked the phase either. I grepped the plan for "malleab", "superseded",
"third party", "re-encoded" — zero hits anywhere in P0–P6. This is the same
gap `design/agent-reports/R8-plan-spec-coverage.md` filed as M-1 ("The
plan's §4 item 2 promises to say 'where the four remaining spec open
questions land' and lands only §10.10"); it does not appear to have been
folded. The natural home is P4's `inspect` report (near the `STATUS`/
liveness discussion) or P3's re-derivation-failure text, but the plan picks
neither, so the implementer has no cue to write it at all — it will not
silently diverge, it will simply never appear.

### M-2 — The live-node smoke test still has no scheduled moment

**Lines 443–445** (P4): "A live-node smoke test is a separate, non-gating
check. A synced `bitcoind` is available on this machine, and one manual run
against it is worth doing — but it must not gate CI, which has no node."
This is the same gap R8 filed as M-2 ("as written it is a suggestion with no
owner and no moment... Bind it to P4's close as a non-CI checklist item, or
it will not happen") and the current text is unchanged from that
description — it still reads as an aside rather than a step bound to a
phase's close. Low stakes (it is explicitly non-gating), but per R8's own
diagnosis, an unbound "worth doing" item does not happen.

---

## What I checked and ruled out (false-positive guards)

- **P0 "workspace lints matching the constellation"** — verified
  `descriptor-mnemonic/Cargo.toml` and `mnemonic-key/Cargo.toml`: both carry
  identical `[workspace.lints]` (`rust.missing_docs = "warn"`,
  `clippy.all = "warn"`). "The constellation" has one pattern, not several —
  not a gap.
- **P2's ordered sniffing procedure** — checked against §8.2e (spec lines
  2274–2360): the 4-step ordered procedure, the binary-before-whitespace
  rule, and the hex-encoded-PSBT ambiguity are all stated exactly as the
  plan summarizes. Fully specified, not a gap.
- **§10.10's "five inputs"** — checked against the §10.10 table (spec lines
  3022–3038) and the closing paragraph (lines 3124–3132): the plan's list
  (per-input values, FROM, TO, TO-label, node location) correctly excludes
  the table's `plate budget` row (its underlying refusal, §8.7, is
  MOVED/deferred with `mt qr`) and `module size` (deleted). Not a gap.
- **Header layout, NUMS constant, content-id derivation (P1)** — fully
  specified in §10.13(a)/(a2)/(b)/(c); the plan's restated bit widths and
  field order match verbatim. Not a gap.
- **Duplicate-resolution rule (P3)** — the `n`-candidate, BCH-then-bytes
  partition rule with its three-row table (genuine/forgeA cases) is fully
  specified at spec §1 (lines 255–293). Not a gap.
- **P4's report layout** — the exact ASCII block and row-presence table at
  spec §1.1 (lines 637–747) are normative and were checked byte-for-byte
  against the plan's bullets; no invented content found.
- **Fixture corpus content (P2)** — the plan's 9 named fixture shapes
  (binary/base64/CRLF/uppercase-hex/etc.) don't need byte-exact
  independence the way the NUMS vector does (I-1) since they test *shape*
  detection, not codec correctness against an external reference — arbitrary
  transaction content under each shape suffices. Not a gap.

## Counts

**Important: 3. Minor: 2.**
