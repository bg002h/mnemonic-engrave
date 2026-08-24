# R12 — pre-implementation gate RERUN, `mt` v0.1

**Artifact:** `design/IMPLEMENTATION_PLAN_mt_v0_1.md` and `design/SPEC_mt_v0_1.md`
@ `6f711b3` (tip at time of review), against the R11 gate
(`design/agent-reports/R11-pre-implementation-gate.md`, NOT SAFE TO EXECUTE, 3C
/ 6I / 10m) and its two fold commits, `946e376` (C1–C3, I1–I6) and `bd7f191`
(Minors).

**Question asked:** did the fold close each R11 finding, did it introduce a new
defect, and is the plan safe to execute unattended tonight?

**Scope, per the dispatch brief:** mechanical verification of the fold, not a
fresh audit. All arithmetic recomputed independently (not read from the diff);
both documents grepped for named stale phrases; the regtest-vector decision and
the `--bitcoin-cli /nonexistent` offline mechanism walked for consistency; the
whole plan walked S0 → P6 as an implementer.

---

## Section A — per-finding status

| # | Finding | Status | Evidence |
| --- | --- | --- | --- |
| C1 | Plan's own line 3 halts the implementer | **FIXED** | Status block now reads `GREEN — 0 Critical / 0 Important as of 2026-08-23 ... Implementation may begin at S0.` (see caveat under new-defect #1 — the *citation* target is questionable, the *instruction* is not) |
| C2 | Open question 4 (repo creation) stale and blocking | **FIXED** | `~~Repo creation~~ **CLOSED 2026-08-23** — bg002h/mnemonic-transaction exists, is **EMPTY** and **PRIVATE**.` Boundaries (no publish/tag/release/public) recorded in the same block. |
| C3 | S0 vector under-specified three ways (provenance, input values, offline forcing) | **FIXED** | S0 now: (a) regtest-produced, never broadcast, with the mainnet-refusal mechanism explained and the discarded mainnet candidate kept as a labelled aside; (b) records all four forms — raw hex, txid+wtxid, finalized PSBT base64, per-input value+outpoint; (c) `--bitcoin-cli /nonexistent` named as THE offline mechanism, with the `PATH`-editing hazard it avoids stated. Verified consistent: `regtest` appears only in S0 (as intended — it's a one-time provenance decision, not a per-phase flag), no `mainnet` vector references survive outside the historical aside, `--bitcoin-cli /nonexistent` is stated once in P2 and explicitly extended by name to both P4's gate and journey B in the same blockquote. |
| I1 | Vector was Markdown-only; `mk`'s SHA-pin pattern needs a JSON file a Rust test reads | **PARTIAL** | S0 correctly emits both `.md` and `.json` from one generator and states the pin covers the JSON (`design/IMPLEMENTATION_PLAN_mt_v0_1.md:174–185`). But **P0's own Deliverable section never lists the JSON copy** — see new defect #2, Section B. The mechanism is right; the phase that must execute it doesn't carry the instruction. |
| I2 | §10.13(a2) says header is 10 symbols, 60 lines under the table that says 11 | **FIXED** | `design/SPEC_mt_v0_1.md:3375`: "Since the header is exactly **11** symbols, the payload begins at symbol index **11**." Zero remaining hits for "exactly 10 symbols" or "symbol 10 of" anywhere in either document. |
| I3 | Every character-length figure computed under the stale 49-bit header | **PARTIAL** | The table (spec ~969–976) and all seven `1,228`→`1,242` sites are fixed and **independently recomputed by me from first principles** — every value matches exactly (see arithmetic recomputation below). But R11's own minimal fix named three more prose sites, and two are unambiguously still wrong: spec line 1006's worked error example still reads `string 7: 88 characters (expected 89)` — neither 88 nor 89 occurs anywhere in the corrected table (80/87/90/91/91/91), so the example no longer corresponds to any real case. Spec line 937 ("a person cutting 90 characters") is stale relative to the "91 is the cap" framing the fold itself installed two paragraphs later (line 978: "91 occurs only when the arithmetic lands on a 40-byte chunk"). The historical 41-bit/49-bit box at spec 1262–1266 was left completely untouched — not labelled as history, as R11's minimal fix asked. None of these three are load-bearing for any gate (P1/P3 length tests read the generator's computed values, not this prose), so this doesn't block, but the finding as R11 scoped it is not fully closed. |
| I4 | §10.10's tail not swept by two earlier folds (stale "unspecified" claim, deleted node-location flag reappearing, stale THREE FLAGS block) | **FIXED** | All four passages corrected: exit-codes sentence now scoped to "beyond 0"; refusal-message format marked RULED with the reason it was previously wrong stated inline; node-location sentence rewritten "**NOT among them** — (b1) deleted it"; THREE FLAGS/SEVEN INPUTS block struck through and marked `CLOSED 2026-08-23`. Zero live hits for "and the node location" as a requirement — the three survivors are all inside their own retraction. |
| I5 | P2's deliverable list contradicted itself on node-location and flag spellings | **FIXED** | Line 410: `**Not the node location** — bitcoin-cli holds it`; line 413: `**Spellings are ruled above**; the input paths are P2's to build`. |
| I6 | TTY welcome line and §6a's encode-time no-node warning had no owning phase | **FIXED** | Both added as P2 deliverable bullets (plan lines 434–443), each citing R11 I6 and the operator-confusion origin of the TTY finding. |
| M1 | P2's fixture gate was a vacuous disjunction | **FIXED** | Gate reworded to "every SNIFFING fixture is accepted or rejected ... P5 owns the refusal fixtures." |
| M2 | `tests/refusals.toml` had no seeded list | **FIXED** | Table added: 12 REFUSE / 3 WARN / 4 EXCLUDED. I independently re-derived §8's full numbering against this table — every numbered item (8.1, 8.2, 8.2b–8.2g, 8.3, 8.4, 8.5, 8.6a/b, 8.7, 8.7b, 8.7c, 8.8, 8.9) is accounted for exactly once. |
| M3 | §8.2c contradicted §8.2e on the raw-no-node case | **FIXED** | "from a PSBT" added; blockquote explains the coin-flip it closes and states the resolution plainly. Matches §10.10's table scoping exactly. |
| M4 | Refusal fixtures could trip two refusals at once | **FIXED** | "Each refusal fixture must be clean in all other respects" added to P5. |
| M5 | §8.7b's fixture is ~1.3 MB | **FIXED** | "SYNTHESISED and GENERATED AT TEST TIME, not committed" added. |
| M6 | P3's 2^20 grind re-runs every CI run | **FIXED** | "Ground out ONCE and PINNED as a fixture ... S0 grinds it and records the colliding transaction beside the vector." |
| M7 | Generator's location not recorded next to the pin | **FIXED** | Pin now records "repo, path and commit SHA" for `scripts/gen-mt1-vectors.py`. |
| M8 | Licence/toolchain unstated | **FIXED** | P0: `license = "MIT OR Unlicense"`, `rust-version = "1.85"`, `rust-toolchain.toml` pinning `1.85.0`. |
| M9 | `CUT` row / `PREFIX` row summary mismatch | **NOT FOLDED, by design** | R11 itself called this cosmetic and said no implementer is misled since the plan lists both rows; the fold commit message states this explicitly. Acceptable as-is. |
| M10 | "six" fields in a five-field table | **FIXED** | "not the least important of the **five**" — matches the table (`FORMAT: mt1 codex32` row). |

**Criticals: 3/3 FIXED. Importants: 4/6 FIXED, 2/6 PARTIAL (I1, I3 — neither PARTIAL blocks a gate on its own; I1's gap does, via new defect #2 below). Minors: 9/10 FIXED, 1/10 not folded by design.**

### Arithmetic recomputation (independent, not read from the diff)

`count = ceil(len/40)`, `bytes_per_chunk = ceil(len/count)`, `last = len −
(count−1)·bytes_per_chunk`, `strlen(b) = 3 + ceil((55 + 8b)/5) + 13`:

| bytes | chunks | b/chunk | full (computed) | full (spec) | last (computed) | last (spec) |
| --- | --- | --- | --- | --- | --- | --- |
| 162 | 5 | 33 | 80 | 80 | 75 | 75 |
| 405 | 11 | 37 | 87 | 87 | 83 | 83 |
| 535 | 14 | 39 | 90 | 90 | 72 | 72 |
| 742 | 19 | 40 | 91 | 91 | 63 | 63 |
| 560 | 14 | 40 | 91 | 91 | 91 | 91 |
| 2,498 | 63 | 40 | 91 | 91 | 56 | 56 |

**6 of 6 full-string, 6 of 6 last-string values match exactly.** 535-byte total:
`13×90 + 72 = 1,242` — matches. Elision: saves `8×(n−1) = 8×13 = 104` on the
14-string case (1,242 − 104 = 1,138). Net versus the old 50-bit non-elided
layout (13×89 + 71 = 1,228): `1,228 − 1,138 = 90` — matches the spec's "90
characters cheaper" claim exactly. BCH margin: a 40-byte chunk is `55 + 320 =
375` bits `= 75` data symbols against `BCH(93,80,8)`'s 80-symbol capacity (5
symbols headroom in the data field); `75 + 13 = 88` total codeword symbols
against 93 (5 symbols headroom overall). Invariant prefix: `version(5) +
chunk_set_id(20) + count−1(15) = 40` bits `= 8` symbols exactly; `index(15
bits) = 3` symbols; `8 + 3 = 11` symbols `= 55` bits. **Every number checks.**

### Grep sweep (check b)

`1,228`, `89-character`/`89 character` (as a per-string-length claim), `exactly
10 symbols`/`symbol 10 of`, `DRAFT, pre-R0`, `does not exist yet`, `six fields`:
**zero live hits** in either document. `--rpc` and `node location`: all
surviving hits are inside their own retraction (`no --rpc`, `--rpc was
deleted`, `NOT AN INPUT`, `The node location is NOT among them`) — confirmed by
reading each site in context, not by hit-count alone.

### Regtest-vector consistency (check c)

`regtest` appears exactly where it should — inside S0's provenance decision —
and nowhere else; no phase downstream re-derives or contradicts it. `mainnet`
appears only inside the labelled aside recording the discarded candidate. §8.5's
non-firing under a regtest outpoint queried against a mainnet node
(`gettxout` → null, parent not confirmed) is stated correctly and matches the
already-verified empirical result cited in the brief. P2's gate ("reproduces
the vector's strings exactly") does not depend on node state either way, so it
passes online or offline as claimed.

### Offline mechanism (check d)

`--bitcoin-cli /nonexistent` is named once, in P2 (plan line 477), and that
same blockquote explicitly extends it by name to both P4's gate ("BOTH with
node fixtures and offline") and journey B ("no node") — so a sequential reader
has the mechanism before reaching either. P4's own "node fixtures" tests are
Rust-level mocks of the node-query boundary (plan line 584: "Node responses are
fixtures, not a live node"), not CLI subprocess invocations, so the flag
doesn't apply there directly — consistent. Neither P4's gate text nor journey
B's own table row repeats the flag inline; see the observation under Section B
below for why that specific omission is sharper than it first looks, though I
am not treating it as a new defect since it is pre-empted by name.

---

## Section B — new defects

### 1. (Important, blocking) P0's own deliverable list omits the JSON vector copy that I1's own fix assigns to it

S0's blockquote (`design/IMPLEMENTATION_PLAN_mt_v0_1.md:184`) states plainly:
*"P0 copies the `.json` to `crates/mt-codec/src/test_vectors/mt1_v1.json`,
matching `mk`'s location shape, and P1 pins **that** file's hash."* This is the
entire point of I1's fix — the SHA-256 pin has to cover a file a Rust test
actually reads.

But P0's own **Deliverable** section (lines 274–276) was never updated to match:

> *"The spec and the S0 vector are copied into `mnemonic-transaction` —
> `design/SPEC_mt_v0_1.md` and `design/vectors/mt1_v1_vectors.md` —"*

The JSON file is named nowhere in P0's own text. `grep -n "mt1_v1\.json\|test_vectors"
design/IMPLEMENTATION_PLAN_mt_v0_1.md` returns only the two occurrences inside
S0's blockquote (lines 175, 184) — zero inside P0's own section.

An implementer building P0 from P0's own deliverable bullets — the phase's
operative instruction, and the standard this whole document holds itself to
(C1's own reasoning: *"An implementer following the plan **exactly**..."*) —
will not create `crates/mt-codec/src/test_vectors/mt1_v1.json`. P1's SHA-256
pin test, which reads exactly that path, then fails on a missing file. This is
the same propagation-failure class R11 already caught twice (I4, I5): the
fix exists in the place that explains *why*, and never reached the phase whose
bullet list is what actually gets built.

**Fix is one line.** Add to P0's deliverable list: *"the machine-readable
`mt1_v1.json` copied to `crates/mt-codec/src/test_vectors/mt1_v1.json` (§ S0,
R11 I1)."*

### 2. (Minor) Plan's status-line citation is premature, and breaks the pattern the spec's own status line uses correctly two lines below it

`design/IMPLEMENTATION_PLAN_mt_v0_1.md:3–4`: *"Status: GREEN — 0 Critical / 0
Important as of 2026-08-23, closed by
`design/agent-reports/R11-pre-implementation-gate.md`..."* — but R11's own
recorded verdict, in that exact file, is *"NOT SAFE TO EXECUTE ... 3 Critical /
6 Important / 10 Minor."* R11 is the finding report, not a verification report.

Contrast with the spec's own status line six lines later: *"GREEN at 0C/0I as
of 2026-08-23 after R6 (three lenses, 6C + 27I) and **R7 (fold verification,
30 FIXED / 3 PARTIAL / 0 NOT FIXED)**"* — correctly citing the fold-verification
report (R7), not the finding report (R6), for its closure claim. The plan's own
line does not follow the pattern sitting right next to it.

Not blocking — the operative instruction (*"Implementation may begin at S0"*)
is unambiguous regardless of which report is named, and this is inherent to the
gate cycle's own shape (the fold that answers a NOT-SAFE gate is written before
the report that verifies the fold exists). Recommend updating the citation to
this report (R12) once it is committed, matching the spec's already-correct
pattern.

### 3. (Minor) I3's own scope: two of three named prose sites still stale, one unlabelled

Detailed under I3 in Section A. Spec lines 937 and 1006, plus the unlabelled
historical box at 1262–1266. None are read by any gate or test; all three are
prose an implementer or future reader would notice as inconsistent with the
corrected table sitting a few paragraphs away.

### 4. (Minor) S0 dropped previously-deliberate transaction-shape detail

Pre-fold, S0 required *"1 input P2WPKH, 1 output P2TR, `nLockTime` set to a
past height."* Post-fold, S0 says only *"one real signed segwit transaction ...
produced on a local regtest node"* — `grep -n "P2WPKH\|P2TR\|nLockTime\|past
height" design/IMPLEMENTATION_PLAN_mt_v0_1.md` returns zero hits. Likely
harmless in practice: Bitcoin Core's regtest wallet defaults to native segwit
addresses, so "segwit" is satisfied without the explicit flag, and P4's
locktime-check tests run against canned node-response fixtures rather than the
vector's own locktime field. Still a genuine loss of a previously-stated
constraint; a one-line restoration would remove the ambiguity for whoever
writes the generator.

### 5. (Minor, pre-existing — not introduced by this fold cycle) Spec's own top-of-file status line is stale

`design/SPEC_mt_v0_1.md:3–5`: *"Status: DRAFT, in R0. ... No code may be
written against this until a re-review closes it at 0 Critical / 0
Important."* `git log` shows this line was last edited during the original R0
fold, well before R6/R7 closed the spec GREEN — it predates the current
fold cycle entirely and is outside R11's named grep list, so it wasn't this
gate's job to catch it. Not blocking, since the plan's explicit, evidenced
GREEN claim for the spec is the operative instruction an implementer follows —
but anyone opening the spec directly (which they must, as source of truth)
hits the same species of contradiction C1 was about. Cheap to fix whenever
convenient.

### 6. (Observation, not a defect — already adequately covered) Journey B's implicit reliance on a cross-reference

P4's gate text and journey B's own table row do not repeat `--bitcoin-cli
/nonexistent` inline, relying instead on P2's blockquote naming both by role.
Worth recording *why* this matters more than it looks: this exact development
machine has a real, reachable mainnet node with `-txindex` on `PATH`. Per the
liveness table (§1.1), a `null` `gettxout` whose parent is "not found" **with**
`-txindex` present classifies as **PENDING**, not **UNKNOWN** — `UNKNOWN` is
reserved specifically for "no `-txindex`". So if journey B's script were ever
run without explicitly forcing `--bitcoin-cli /nonexistent`, it would not
coincidentally produce the same UNKNOWN report by luck — it would hit the real
node and misclassify the regtest vector as PENDING, failing journey B's own
gate ("every row reads UNKNOWN"). This is exactly the scenario P2's blockquote
already names and prescribes the fix for, so I am not treating it as a new
defect. But it's the reason a one-line explicit restatement in P6's own journey
B row (rather than relying on a reader carrying the fact forward from P2) would
be worth the sentence.

---

## Section C — verdict

**NOT SAFE TO EXECUTE** — one Important, mechanically-verified gap: P0's own
deliverable list never says to copy the machine-readable vector JSON to
`crates/mt-codec/src/test_vectors/mt1_v1.json`, which P1's SHA-256 pin test
requires at that exact path. Everything else — all 3 Criticals, 4 of 6
Importants, and 9 of 10 Minors from R11 — is correctly and verifiably fixed;
the regtest-vector provenance fix and the offline mechanism are both sound and
consistent everywhere they're used; every recomputed number matches
independently. The fix is one line in P0's deliverable list (see Section B.1);
after it lands, this plan is safe to execute unattended.
