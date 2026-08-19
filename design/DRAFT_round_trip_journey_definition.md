# DRAFT — what a "round-trip journey" is, constellation-wide

**Status: DRAFT for user reaction, 2026-08-18.** Not agreed, not gated. Written
before the utility audit so that every agent dispatched into that audit measures
the same thing. If this is wrong, it is cheap to fix now and expensive to fix
after eight reports have been written against it.

---

## 1. The definition

> A **round-trip journey** is a named, single-command-executable path through the
> constellation that begins at a stated origin artifact, passes through every
> layer a real operator would traverse, and ends in **two independent equality
> assertions at different layers** — one *structural*, one *functional*. It
> states its tier, and it states what it does not cover.

Each clause below exists because something in this project's history went wrong
without it.

## 2. Origin — where a journey starts

Two kinds, and conflating them is the first trap:

- **Generative** — starts at entropy or a seed. Exercises the *encode* side.
  `entropy → phrase → card(s) → … → back`.
- **Custodial** — starts at an artifact already in hand (an `md1`, `mk1`, `ms1`
  card, a payload, a tag). Exercises only the *decode* side.

**Both are journeys; only generative ones prove the constellation can produce a
backup.** An audit that finds only custodial coverage has found a hole, not a
pass. Every journey names its kind in one word.

## 3. Tier — how far the loop physically extends

Four tiers, each named, each with what it proves and what it cannot. **A journey
declares exactly one.** The point of naming them is that a claim of coverage at
one tier must never be read as coverage at a higher one.

| tier | loop | proves | blind to |
| --- | --- | --- | --- |
| **T1 codec** | in-process encode → decode | the codec is faithful | anything about tools, screens or media |
| **T2 tool** | CLI → files → CLI, separate processes, real exit codes | **the tools compose** | anything about the device |
| **T3 operator** | the emulator walks the real device flow, real screens, real input transport | **a user can do the thing** | the physical media |
| **T4-sim** | simulated engraving under an **advanced clock**, plate rasterized from the toolpath, **read back by a decoder** | the engraving is **legible**, not merely emitted | real material behaviour — burrs, glare, oxidation, tool deflection |
| **T4-metal** | real engraving, photograph of real steel, same decoder | the physical loop | nothing — but ~21 min/plate and consumes material |

T1 is CI-cheap and should be exhaustive. T2 is where this project's defects
actually live. T3 is what proves a feature is reachable rather than merely
built. **T4-sim is in scope and should run routinely** (user ruling
2026-08-18); T4-metal stays rare and deliberate.

### 3.1 T4 needs a reader that does not exist yet — DEPRIORITIZED near-term

**User ruling 2026-08-18: the engraving decoder is LOW PRIORITY for the near
term.** So T4-sim is *defined* but not *staffed*: journeys record T4 coverage as
**absent-by-decision**, not as an oversight, and no near-term work is blocked on
it. The rest of this subsection is the design constraint for when it is picked
up — written now because the reasoning is cheap to record and expensive to
reconstruct.

**Engraving is write-only across the entire current test surface.** Nothing
anywhere decodes a plate back into a string. So T4's structural assertion has
never been provable even in principle: the constellation can write to metal, and
nothing has ever confirmed the metal is *readable*. A **plate decoder** —
engraved text and SeedQR — is therefore not a convenience for T4, it is what
makes T4 a round trip rather than a one-way trip. Expect the audit to find it
absent, and treat "absent" as the finding rather than a gap to paper over.

**The trap it must avoid, and it is this document's §5 rule applied to pixels.**
If the decoder reads *the preview image the renderer drew*, sharing the glyph
code with the writer, the loop proves only that a function agrees with itself —
the same defect that let a frozen snapshot bless the `v:` renderer bug. So:

- read from the **toolpath the machine would actually cut**, rasterized
  independently — not from the preview the GUI drew;
- do **not** share glyph-rendering code between writer and decoder. A decoder
  that inherits the writer's idea of a glyph cannot discover that the glyph is
  ambiguous;
- an **advanced clock** is a legitimate substitution (it only compresses time),
  but every *other* substitution the harness makes must be named in the
  journey's non-coverage statement.

**The payoff is larger than round-tripping.** A decoder makes legibility
*measurable*: degrade the raster — a scratch across a stroke, a burr, partial
occlusion, the axis play once misdiagnosed as four different software bugs — and
find the threshold at which decoding fails. That converts the engraving-font
rules (2-stroke-width minimum feature; single-feature glyphs losing identity to
one scratch) from asserted principles into measured margins, on a project whose
entire value proposition is that the metal is still readable in twenty years.

## 4. The terminal assertion — two equalities, different layers

**This is the load-bearing clause.** A journey ends in *both*:

1. a **structural** equality — the bytes, phrase, template string, or chunk set
   round-tripped intact; and
2. a **functional** equality — something that controls funds matches: a
   **receive address and a change address**, a master fingerprint, or the
   applicable **wallet id**.

**Neither alone is sufficient, and the failure modes are different in each
direction.** Bytes can match while a wrong derivation path, network or use-site
sends funds somewhere the operator never sees. And a tool can be made to
*accept* input it previously rejected while silently dropping part of it — the
structural check is what catches that.

Where two wallet ids exist, the assertion **names which one**:
`WalletDescriptorTemplateId` is key-stable, `WalletPolicyId` is key-dependent,
and they differ for the same wallet. An unqualified "the id matched" is not an
assertion.

**Change addresses are not optional.** Receive-only is the check that passes
while a policy mismatch quietly loses money on the change chain.

## 5. Anti-requirements — what disqualifies a journey

Each of these has cost this project a cycle.

- **It must be re-runnable by one command.** *A journey that cannot be re-run is
  not a journey, it is a transcript.* F-210: four intermediates have never had a
  writer in any committed version, and `transcript_pathological.sh:18` reads
  `out/md1.txt` sixteen lines before the only command that could produce it. The
  artifact kept vouching for a path that had rotted.
- **It must not read an intermediate that nothing in the journey writes.** Same
  defect, stated as a rule a tool can check.
- **It must not assert against a value the journey itself produced** with no
  independent source. A snapshot test blesses whatever the code did, bug
  included — which is exactly how the `v:` renderer defect survived a frozen KAT.
- **A skipped step must fail, not pass.** A skip prints `ok` and exits 0. If a
  stage cannot run, the journey is red, never green-by-absence.
- **Every gate in it must have executed at least once.** A gate that has never
  run is a hypothesis. A journey whose assertions are unsatisfiable by
  construction is worse than no journey, because it reports success.
- **Empty output is not proof of absence.** A negative may mean the check never
  ran; streams are separated so a wrapper's stderr cannot be mistaken for a
  result.

## 6. Coverage statement — mandatory

Every journey ends by stating, in its own output, **what it did not cover**: the
tier above it, the shapes it did not exercise, the transports it did not use.
A gate that hides its own blind spot is worse than no gate, and a journey that
implies more coverage than it has is the same defect wearing a friendlier face.

## 7. The unit, for the audit's purposes

For the utility audit, a journey is identified by:

```
name | kind (generative|custodial) | tier (T1..T4) | origin artifact
     | ordered invocations, repo by repo
     | structural assertion | functional assertion
     | one command that runs it | stated non-coverage
```

An existing path that lacks any field is **a finding**, not a journey. In
particular: a path with no functional assertion, a path spanning repos with no
single command, and a path whose "expected" values were transcribed by hand from
a run nobody has repeated.

## 8. Open — for the user to rule before dispatch

1. ~~Is T4 in scope for the audit at all?~~ **RULED 2026-08-18: yes — performed
   via the simulator under an advanced clock (T4-sim), with a
   simulator-adjacent decoder built to read a simulated engraving of a string
   or QR code.** See §3.1. Remaining sub-question: does the decoder read the
   **toolpath** (independent, recommended) or the rendered preview (shares code
   with the writer, and proves much less)?
2. **Does a generative journey have to start at entropy**, or is starting from a
   fixed test seed enough? (Fixed seeds are reproducible; real entropy is what
   operators actually use.)
3. **Is the audit's job to inventory journeys that exist, or to enumerate the
   journeys that *should* exist and mark each present/absent?** The second is
   more work per agent and is the one that finds holes — a per-repo sweep is
   structurally blind to gaps *between* repos, which is where round trips break.
4. **Passphrase, network and account-index variation** — dimensions of a journey,
   or separate journeys?
