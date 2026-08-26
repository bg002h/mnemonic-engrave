# SPEC — constellation CLI uniformity (`md`, `mk`, `ms`, `mt`)

**Status:** DRAFT. Brainstormed with the operator 2026-08-26. **R0 review not yet
run; no code may be written until it closes 0C/0I** (project CLAUDE.md — this is
risk-set work: it changes normative CLI behaviour and it touches secret and
bearer material).

**Goal, in the operator's words:**

> *"The goal is for the user to not care if they are dealing with mk or md or mt
> or ms."*

---

## 1. The finding that motivates all of it

**The safety gradient runs backwards.** Measured 2026-08-26 against the built
binaries, not read from source:

| tool | material it handles | material on argv | its advice for stdout |
| --- | --- | --- | --- |
| `mt` | one transaction — spends once | **refused**, exit 3, with purge advice | refuses mode 0644, names three remedies |
| `ms` | **BIP-39 seed entropy — spends everything, forever** | **accepted silently**, exit 0 | *"redirect … e.g. `> file.txt`"* |

`ms encode --phrase "<12 words>"` exits 0 and says nothing about that phrase now
being in shell history and in `ps`. Its stderr then recommends `> file.txt`,
which under the default umask produces **mode 0644** — the exact disposal `me`
and `mt` refuse at exit 2 (F-244, F-252).

**The tool holding the most dangerous material in the constellation has the
weakest handling of it.**

## 2. What is already uniform, and what is not

**The verbs are already uniform.** `encode`, `decode`, `verify`, `inspect` exist
in all four. Nothing in this spec changes a verb name. What diverges is
everything around them.

Measured:

| | `md` | `mk` | `ms` | `mt` |
| --- | --- | --- | --- | --- |
| `--json` | yes | yes | yes | yes |
| `--in FILE` | — | — | — | **yes** |
| `--out FILE` | — | — | — | — |
| stdin idiom | positional | positional | `--phrase -` | default, plus bare `-` |
| `--group-size` default | **5** | **5** | **5** | **off** |
| non-artifact lines on stdout | **`chunk-set-id:`** | *unverified — see §8* | none | none |
| material on argv | positional | flags | `--phrase` | **refused (`SPEC_mt_v0_1` section 8.2f)** |
| stdout bytes on failure | **0** | **0** | **0** | **0** |
| exit code, invalid input | 1 | **2** | 1 | 1 |

**The pipeline-safety invariant already holds constellation-wide**: every tool
contributes 0 bytes to stdout on a failure path. That matters because `fish`
reports a pipeline's status as the LAST command's, so a failed upstream is
invisible except by its silence. It is the one thing that did not need fixing.

## 3. THE DEFAULT OUTPUT OF THREE TOOLS CANNOT BE PACKED

The decisive measurement, and the reason this is a defect rather than a
preference:

| `ms encode …` | `… \| me sysw pack` |
| --- | --- |
| **default** (grouped by 5, space separator) | **exit 4 — "not a form this container can place"** |
| `--separator hyphen` | **exit 4** |
| `--group-size 0` | **exit 0, 102-byte payload** |

The same holds for `md`: `md encode --group-size 0 … \| grep '^md1' \| me sysw
pack` → exit 0, a 214-byte payload. The `--group-size 0` and the `grep` are
precisely the two defects — the flag works around the grouped default, the grep
works around the `chunk-set-id:` header.

**So a user today must know two workarounds to compose anything.** That is the
"not care" requirement, unmet.

Grouping is a display concern by these tools' own admission — `md`'s help says
*"Display only; `--json` stays unbroken."* It is already understood as
presentation. It simply leaks into the canonical artifact.

## 4. The principle

**Generalise `SPEC_mt_v0_1` section 3b to the other three.** `mt` already states it:

> *"Opt-in and never the default: grouping affects **stdout**, and the canonical
> artifact is ungrouped."*

Every divergence in §2 is a violation of one rule:

> **stdout carries the canonical artifact and nothing else. Presentation goes to
> stderr. Material never arrives on argv.**

`mt` is the only tool that obeys it, and the only one that composes.

## 5. Decisions taken (operator, 2026-08-26)

| # | Decision | Rejected alternatives |
| --- | --- | --- |
| D1 | **Composition is a shell pipeline into `me sysw pack`.** No new umbrella binary. | a new `m` front-end that sniffs and dispatches |
| D2 | **Host-side only this cycle.** "Engravable as plaintext in a QR" for `md`/`mk`/`ms` needs new record classes and device rendering; it becomes its own spec with a firmware gate. | doing both at once; device first |
| D3 | **Refuse secret/bearer material on argv, with `--allow-argv-secret`.** | refusing with no override; warning only |
| D4 | **stdout is canonical (ungrouped); the grouped form moves to the stderr engraving card.** | flipping the default only; teaching `me sysw pack` to strip separators |
| D5 | **One shared crate owns the IO + safety layer**, depended on by all four. | spec + conformance vectors with copied code; a pure-logic crate with per-repo flag wiring |

**On D4's rejected third option, recorded so it is not revisited:** teaching
`me sysw pack` to strip separators would make admission LOOSER on the
funds-sensitive side. `me` refuses an elided `md1` deliberately; *"strip whatever
looks like a separator"* is how a mangled string gets silently accepted.

**On D5:** four copies of one rule is the shape that let `pack` and
`pack_deterministic` drift until a vector round-trip caught it — that module's
own comment records it. Precedent for a shared crate exists: `mt-codec` is
already consumed across repos.

## 6. The surface, after

### 6.1 Every tool, every verb

- `--in FILE` — read material from a file.
- `-` as a positional — read stdin. Accepted and ignored where stdin is already
  the default (F-250's fix in `mt` is the reference implementation).
- `--out FILE` — write the artifact to a file, **created 0600 by `me`'s
  `write_private`**, never `std::fs::write` (F-244).
- stdout, when neither is given: the canonical artifact, ungrouped, nothing else.
- `--group-size` / `--separator` affect **the stderr card only**.
- `--json` unchanged; it is already uniform and already unbroken.

### 6.2 The separator rule

Whitespace only, everywhere. `mt` restricts to whitespace because its decoder
strips whitespace and nothing else, so a hyphen-grouped string is one `mt`'s own
verbs refuse — after the plates are cut. `ms` currently tolerates hyphens on
decode, so its hyphen option is safe *for `ms`* — but the card is what a human
types back, and a rule that is safe per-tool and unsafe across tools is the kind
an operator carries between tools. **The cost of uniform whitespace-only is one
cosmetic option in three tools; the cost of getting it wrong is a plate.**

### 6.3 Argv refusal (D3)

Shape follows `SPEC_mt_v0_1` section 8.2f, which is already tested and already lands
correctly:

- **Never echo the argument.** Printing it back puts the material in a second
  place — the defect the refusal exists to name.
- Report the CLASS and the LENGTH, not the value.
- Name the private channels: `--in FILE`, `-` for stdin.
- Give the purge commands: shell-history removal and `shred -u`.
- `--allow-argv-secret` proceeds. It is greppable in a script, so a reviewer can
  find it.

**`mt` gains the override too**, for uniformity — it has none today.

### 6.4 The write gate

The layer `me` grew during the 2026-08-25 burndown, generalised:

- A **terminal** is refused for any artifact that is bearer or secret (F-253),
  with the file-shaped route printed instead.
- A **world-readable file** is refused, naming the MODE measured and stating that
  only the file's own mode was checked (F-252) — an ancestor directory may
  already make it unreachable.
- `--allow-world-readable` overrides the mode gate and NOT the terminal gate.
- **No line describing the artifact prints until every gate that can abort the
  write has run** (F-246), and **a refusal about the INPUT outranks one about the
  destination** (the ordering regression the journey caught).

### 6.5 Exit codes

One table, all four. `mk`'s 2-for-invalid-input becomes 1. Codes to be fixed in
the plan against what each tool uses today, so no existing script silently
changes meaning without it being recorded.

## 7. Phasing

| phase | content | gate |
| --- | --- | --- |
| **P0** | the shared crate: `--in`/`--out`/`-`, argv guard, write gate, exit codes, remedy text. Ported FROM `mt`/`me`, which already have the tested versions. | its own tests + the R0 spec review |
| **P1** | `mt` adopts the crate. Least risk: it already behaves this way, so a behaviour change here is a bug in the crate. | `mt`'s suite unchanged, 236 tests |
| **P2** | `ms` — the argv refusal and the 0600 `--out`. **Highest safety value; do it before the cosmetic work.** | round-trip vectors; `ms encode \| me sysw pack` runs |
| **P3** | `md`, `mk` — header off stdout, grouping to stderr, `--in`/`--out`. | `md encode \| me sysw pack` runs with **no flags and no grep** |
| **P4** | the operator journey: one command, several inputs of different kinds, one payload. | a captured journey that regenerates |

**P2 before P3 is deliberate**: the seed-phrase-on-argv hole is the finding with
funds behind it; the grouped default is a usability defect.

## 8. What is NOT verified, and must be before the plan closes

- **`mk`'s stdout shape.** Every `mk encode` invocation attempted during this
  brainstorm failed on argument requirements (`--origin-path`, then
  `--policy-id-stub`/`--from-md1`, then a complete `md1` set that
  `--from-md1` would not accept space-joined). **`mk` is marked "unverified" in
  §2 rather than assumed to match `md`.** Its header behaviour and its default
  grouping must be measured, not inferred.
- **Whether `mk encode --from-md1` can accept a multi-chunk set at all.** The
  space-joined form is rejected with *"character '1' not in codex32 alphabet"*.
  If there is no way to pass a 2-chunk set, that is a defect in its own right and
  belongs in this cycle's scope.
- **Which existing invocations break.** D4 and D3 are behaviour changes. The plan
  must enumerate what stops working and what the migration is, per tool.

## 8a. What the structure gate says, and which of it is real

`scripts/spec-structure-check.sh` reports **7** on this file after the real one
was fixed (it reported 9 before). Read, not counted:

- **5 × "DUPLICATE section 6"** — the gate parses `### 6.1`…`### 6.5` as five
  section 6s. A known limitation of its subsection handling, not a defect here:
  `SPEC_engrave_transaction.md` trips the same class **21** times.
- **2 × table row has 4 cells, header has 3** (the P2 and P3 phase rows) — the
  gate does not honour `\|`, which its own header admits ("pipes inside code
  spans" are not covered). Counted by hand: 5 pipes, 1 escaped, 3 cells. Correct
  as written.
- **2 × cross-reference resolves nowhere** — **REAL, and fixed.** They were
  sections of `SPEC_mt_v0_1.md`, not of this document. **Naming the file was not
  enough**: the gate matches the section sigil wherever it appears, so an
  external reference that keeps the sigil after the filename still resolves
  against *this* file and still fails. (Not quoted here — writing the bad form
  out re-creates it, and re-running the gate is what caught that.) External references here therefore drop the sigil — "section 3b"
  — which keeps the gate's signal clean instead of teaching a reader to ignore
  two permanent FAILs. Verified the file exists and carries both sections before
  citing it.

## 9. Out of scope, explicitly

- **QR-for-everything** (D2). `md`/`mk`/`ms` payloads engraved as plaintext QR
  needs new record classes, device rendering, plate layout and an S0 hardware
  gate. Its own spec.
- **A new umbrella binary** (D1).
- **F-247**, the NFC-fit line on `mt encode --qr` — deferred by the operator,
  unrelated to this cycle.

## 10. Acceptance

The cycle is done when this runs, with **no flags beyond the inputs and no
`grep`**, and the operator has not had to know which tool owns which string:

```
{
  md encode --in wallet.desc
  mk encode --in cosigner1.xpub
  mt encode --qr --in tx.hex
} | me sysw pack --out payload.bin
```

and when `ms encode --phrase "<a real seed>"` refuses, names the exposure, and
prints the commands to purge it.
