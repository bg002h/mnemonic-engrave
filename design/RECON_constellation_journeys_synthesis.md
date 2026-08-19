# RECON SYNTHESIS — round-trip journeys across the constellation

**2026-08-19. Nine read-only agents, eight repos plus one protocol question.**
Verbatim reports in `design/agent-reports/recon-journeys-*.md` and
`recon-protocol-multisig-xpub-depth.md`, committed at `5a54622` before this
document existed. Scope was fixed by the operator's §8 rulings: inventory what
**exists**; the should-exist catalogue is deferred.

This is a **recommendation document**. Nothing in it has been acted on.

---

## The one-sentence result

Across roughly **50 journey-shaped paths** in seven code repos, **exactly one**
satisfies §4's two-equality requirement — and it is receive-only, so on a strict
reading the count is **zero**.

## What the sweep found that no single repo could show

Five of these are invisible from inside any one repo. That is the argument for
having run it constellation-wide, and also the argument for the deferred
catalogue: a per-repo sweep is structurally blind to gaps *between* repos.

### 1. The two-equality clause is unmet, uniformly

Journeys split cleanly into **structural-only** or **functional-only**, never
both — in `descriptor-mnemonic` (5/5), `mnemonic-key` (6/6), `mnemonic-secret`
(17/17), and `mnemonic-toolkit` (~26 cells *named* "roundtrip" carry no address
assertion at all). The sole exception is `mnemonic-gui`'s
`bundle_restore_independent_oracle.rs`.

This is not a coincidence of style. Structural and functional assertions live in
different test files by convention, and nothing ever joined them.

### 2. Change addresses are absent everywhere — including the one good journey

**Zero** journeys in the constellation assert a chain-1 address. §4 names this
exactly: *"Receive-only is the check that passes while a policy mismatch quietly
loses money on the change chain."* This is the single most consistent finding of
the sweep and the cheapest to fix.

### 3. Four independent skip-passes-instead-of-fails instances

Each prints `ok` and exits 0 while not testing what it claims:

| repo | mechanism |
| --- | --- |
| `mnemonic-secret` | `parity_smoke` prints *"DORMANT, not passing"* then reports `ok` — **reproduced live** |
| `mnemonic-toolkit` | bitcoind-differential passes when its wiring env vars are unset |
| `mnemonic-gui` | tutorial suite; dormant only because CI happens to set the env vars |
| `mnemonic-engrave` | both wallet PDFs build with **100 % of screenshots missing** and exit 0 |

The `mnemonic-gui` one is the shape worth naming: **an env var is what enables
enforcement**, so the check is one config change away from silently vanishing.

### 4. Self-referential oracles — a test blessing its own output

- `mnemonic-toolkit`: 3 of 4 `*-api-roundtrip.rs` examples contain **zero
  in-code assertions**; they are "verified" by diffing a committed golden
  transcript. The 4th never calls encode — it deserializes a hardcoded literal.
- `mnemonic-gui`: the five tutorial journeys assert against a **self-regenerable**
  snapshot.
- `mnemonic-engrave`: `build_pdf_pathological.py` hardcodes an id comparison
  transcribed from a run nobody can repeat.

This is the defect that let the `v:` renderer bug survive a frozen KAT.

### 5. The device's own funds check is never exercised

`seedhammer`'s **`bundle.Verify` is the only mechanism in the constellation that
could supply §4's functional equality on-device** — and no journey reaches it.
Every emulator walk that gets the verify offer **taps Skip**, and the one test
that presses *"Verify now"* **stubs the verify function itself**.

Compounding it: `testEngraver.Write` returns `len(steps), nil` — **verified by
reading it**. Every CI-run GUI walk engraves into a void, so no walk can assert
anything about engraved content even in principle.

---

## The protocol finding — `md`'s depth rule is wrong

Answered against source text, and independently spot-checked by the controller.

**Verdict: the exact-depth check is wrong in both directions.**

- **Too strict.** BIP-87 (Complete) publishes
  `wsh(sortedmulti(2,[xfpForA/87'/0'/0']XpubA/0/*,…))` — a **depth-3** xpub in
  multisig, which `md` rejects. BIP-388's own vector uses a **depth-5** xpub with
  no origin inside `sortedmulti_a`.
- **Too loose.** `44'/0'/0'/100'` passes. So does `m/1/2/3/4`. Depth 4 is not
  evidence of BIP-48, so the rule fails at its own purpose.
- **The rationale is a false citation.** `descriptor-mnemonic/crates/md-cli/src/parse/template.rs:672` says
  *"Depth tracks BIP 388 expectation"*. BIP-388 uses the word "depth" **zero
  times** — as does BIP-87, fetched from source. Verified.
- **The single-sig `== 3` arm fails identically** (BIP-382's `wpkh` vector is
  depth 1). Flagged, not investigated.

**Consequence for our fixture, stated plainly:** re-deriving the eleven
pathological keys to depth 4 unblocked address derivation (real, and it holds),
but it conformed the fixture *to* the defective rule. The suite now encodes the
wrong expectation.

**Proposed replacement**, which is *stronger* rather than merely looser: when an
origin is present require `depth == origin_path.len()` — catching a wrong-level
key at **any** depth, not only non-4 — plus BIP-32's real rule that depth 0 with
non-zero fingerprint/index is invalid; warn rather than reject on unrecognised
schemes. Normative: **Rust first, with vectors, R0-gated.**

---

## What is already good — do not rebuild these

- `mnemonic-gui` `bundle_restore_independent_oracle.rs` — the one journey meeting
  §4. **Use it as the template.**
- `seedhammer` `backup/qrdecode_test.go`'s `decodeQR` — a real independent
  decoder: walks the QR module grid per spec, never calls the writer's encoder,
  fails loudly on the version range it cannot handle. **The T4 decoder pattern
  already exists**, and this refutes §3.1's "engraving is write-only across the
  entire current test surface" (scoped to the passphrase plate).
- `mnemonic-toolkit`'s bitcoind differential — confirmed to have **actually
  executed** 5/5 on schedule. A gate that has run is not a hypothesis.
- `descriptor-mnemonic`'s `bitcoind_address_differential` — green 10/10, a real
  external oracle. Note it **never touches the md1 wire format**, so the repo's
  most rigorous test is not a journey.

---

## Recommendations, in priority order

**P1 — Stop the green lies.** A false PASS is worse than no test, and there are
now four of them plus a void-writing engraver. Make each skip **fail**; never let
an env var be what enables enforcement. Give `testEngraver` a payload sink so a
walk can assert what was engraved. Establish whether the 3 `seedhammer` walk
drivers with no execution evidence can run at all — *a gate that has never run
is a hypothesis*.

**P2 — Close the two funds-relevant holes.** Add a **chain-1 (change) address**
assertion to every journey that has a functional half — cheapest high-value fix
in this document. Chain `mnemonic-secret`'s split → combine → **derive**, so a
recombined secret is proven to still control the funds. Then drive one
`seedhammer` walk through **`bundle.Verify`** instead of tapping Skip.

**P3 — Rule on the depth defect.** Decide whether the replacement check enters
the next cycle. It is normative, so it is a cycle, not a step. Until then the
pathological fixture knowingly encodes the wrong rule.

**P4 — Make journeys re-runnable.** Nothing in `mnemonic-engrave` runs from a
single command. Wire `derive-pathological-keys.sh --check` into the pathological
transcript — **it is referenced nowhere today**, a fresh orphan gate created
while fixing an older one. Add §6 non-coverage statements, which no journey in
any repo currently prints.

**P5 — Records.** `descriptor-mnemonic`'s corpus is **16**, not the 10 its doc
comment and CI workflow claim (counted three ways; the runtime output uses
`corpus().len()` and is correct, so only the prose is stale).
`mnemonic-secret`'s README still calls K-of-N unreleased at v0.1.0 against
shipped 0.16.0. Rename the ~26 `mnemonic-toolkit` cells that say "roundtrip" and
are not.

---

## Two things this recon did not do

1. **It did not enumerate the journeys that *should* exist.** Deferred by ruling.
   The inventory records what each journey *covers*, so that pass is a diff.
2. **It is blind to gaps between repos** — by construction, per-repo agents
   cannot see them. Finding 5 (`bundle.Verify` never reached) was recovered only
   because one agent happened to look across a boundary; there is no reason to
   think it is the only one.
