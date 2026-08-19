# Architect R0 (v2) — PLAN_key_index_legibility.md @ 53b2e82

Reviewer: independent read-only architect agent, 2026-08-19. Same two questions
as round 0 — (a) can a competent implementer execute this without inventing
anything, (b) is any stated assumption factually wrong. No redesign, no
re-litigation of recorded decisions. Prior review:
`design/agent-reports/plan-key-index-legibility-architect-r0.md`.

**Everything below marked "executed" was run on this machine** against the real
binaries (`mnemonic-toolkit/target/release/mnemonic`,
`descriptor-mnemonic/target/release/md`, `mnemonic-key/target/release/mk`,
`mnemonic-engrave/target/release/me`) and a fixture rebuilt from
`design/journeys/inputs-pathological/` exactly as `transcript_pathological.sh`
builds it (3 md1 chunks, 11 cards → 30 mk1 chunks, stub `5b48af35`). The tool
shell here does **not** word-split unquoted `$VAR`; every command was run
through `bash -c`, as the journey scripts do.

---

## Verdict

**(a) No — still not executable, but for a much smaller and sharper set of
reasons than v1.** v2 genuinely closed most of round 0: the representation
decision is made and its rationale is correct, the fallback string is specified,
the collision case has a defined behaviour, the mechanism is `--expect-wallet-id`
and the timing question is answered with a real number, and §4 now has a file, a
heading and an acceptance criterion. The residue is concentrated in **§1**, which
is new work, and in **§2's manifest blast radius**, which v2 asserts is empty and
is not.

The blocking item is not the open choice. §1 escalates *which home* — that is a
legitimate, deliberately-named escalation and I do not count it as a defect. What
is a defect is that **neither candidate home can compute the value with the
inputs its tool has**, and the plan does not say what would have to be built:

- `md inspect` takes only md1 strings (`md inspect --help`: `<STRINGS>...`,
  `--json`). The engraved artifact is a **keyless** template — executed,
  `wallet-policy-mode: false` — so there are no keys to re-serialise. And no
  `md` command can build a keyed md1 carrying this wallet's four *distinct*
  per-key origins: `md encode`'s only path flag is `--path`, documented
  "*Override the inferred origin path with a single shared path (flattens
  Divergent mode to Shared)*" (`crates/md-cli/src/main.rs:94`), there is no
  per-`@N` path flag, `--key` rejects an origin-prefixed value, and a concrete
  descriptor is refused.
- `me bundle` takes an unordered newline-separated string list (`me bundle
  --help`: `--in`, `--manifest`, `--preview`, `--png`). The `WalletPolicyId` is
  **order-sensitive** (`mnemonic-toolkit/src/synthesize.rs:228`: *"`WalletPolicyId`
  is order-sensitive (`identity.rs` never sorts)"*). `me` cannot determine which
  card is `@0` — which is exactly what **§2 of this same plan asserts**. §1(b)
  and §2 contradict each other.

So §1's acceptance ("a command in this repo prints
`ced2270948ecb5af0779249ac7181f4a`") is, under both offered homes, a gate that
cannot be run — the failure mode CLAUDE.md names as "a gate that has never
executed is a hypothesis, not a gate".

**(b) Yes — three wrong assumptions, one of which is load-bearing for §2 and one
for §1.** §2's "existing manifest consumers are unaffected" is false: the
byte-pinned golden gains keys and two `assert_eq!` sites fail. §1's stated root
cause (F-130 xpub re-serialisation) is not the whole cause — executed, the
completed id also moves with the **card metadata**. And §1's own illustrative
target string is a prefix length the tool **refuses**.

The good news, all executed and all reproduced: v2's two Criticals from round 0
are genuinely fixed, and its headline measurements are real.

---

## Prior findings — resolved, restated, or ignored

| # | round-0 finding | status in v2 | evidence |
| --- | --- | --- | --- |
| **F1 (C)** | "a seed is NOT required" is false | **RESOLVED** | §0 now says "A seed IS required", cites `restore.rs:1396`. Verified: line 1396 is the `message:` of the `no_from` `ModeViolation`. |
| **F2 (C)** | `--search-address` refused at n=11 | **RESOLVED** | Reproduced 4×: exit 1 in 0.082–0.208 s. Mechanism switched to `--expect-wallet-id`. |
| **F3 (I)** | the recorded target is not the wallet's id | **PROMOTED to §1** — problem stated correctly, **fix not implementable as offered** (see D1–D3) | table rows 1,2 reproduced exactly; row 3 is ambiguous (two values, below); row 4 reproduced. |
| **F4 (I)** | "No new dependency" conditional | **RESOLVED** | `String` chosen; `grep -rn bitcoin crates/me-cli/Cargo.toml crates/me-cli/src/` → 1 hit, `keywords` line 10. |
| **F5 (I)** | "no normative change" understates the manifest contract | **PARTIAL** — §0 concedes "de facto contract", then §2 asserts consumers unaffected (false) and never names the golden or SPEC §6. |
| **F6 (I)** | "`me` cannot print `@N`" reason is over-broad | **RESTATED** — §2 repeats "a keyless template carries no key order". Still over-broad: `md inspect --json` shows the TLV carries `fingerprints` and `origin_path_overrides` slots. Now actively harmful: it is the same fact that kills §1(b), and the plan does not connect them. |
| F7 | fixture correction correct | **HELD** — re-measured: `key-04` = `b8688df1` / `48'/0'/0'/2'`. §4 limit 2 is right. |
| F8 | the shape is admitted | **HELD** — explicit mode still exit 0. |
| **D1** | representation of the new fields | **RESOLVED** — `String`, decided, rationale verified. |
| **D2** | do they serialize / golden / SPEC | **RESTATED + now FALSE** — see FI-1. |
| **D3** | exact fallback string | **RESOLVED** — both forms given; `'` vs `h` fork explicitly closed. |
| **D4** | empty `origin_path` | **IGNORED** — not in the edge table. (Low reachability: `mk encode --origin-path m` → `error: xpub origin-path mismatch: xpub depth 4 / child 2' vs origin_path depth 0`.) |
| **D5** | collision output | **MOSTLY RESOLVED** — "plus `set <chunk_set_id>`". Position in the string and per-plate-vs-per-card scoping still open (D5 below). |
| **D6** | multi-wallet / multi-stub | **RESOLVED** — "multiple `policy_id_stubs`: irrelevant here; ignore". |
| **D7** | the whole `restore` invocation | **PARTIAL** — flag, seed, `--account 0`, "the other ten cards", 28 chunk args now given; chunk-contiguity still absent (D9 below). |
| **D8** | which target | **RESOLVED** — `--expect-wallet-id`. |
| **D9** | estimate vs realized | **RESOLVED** — 16 s realized, decision made (in the journey). Reproduced: 15 s and 16 s. |
| **D10** | where §2's doc goes | **RESOLVED** — §4 names `design/journeys/README.md`, the section title, and an acceptance. |
| **D11** | template index before/after canonicalisation | **IGNORED** — `canonicalize_placeholder_indices` still exported (`md-codec/src/lib.rs:46`). |
| **D12** | version bump / CHANGELOG | **IGNORED** — `SPEC_me_bundle_phaseA.md:121-123` still records the lockstep; `crates/me-cli/CHANGELOG.md` tracked; `Cargo.toml:3` = `0.6.0`. |
| MA-1 | collision has no acceptance | **PARTIAL** — behaviour defined, still not exercised by an acceptance bullet. |
| MA-2 | "updated deliberately" is intent, not criterion | **RESTATED verbatim** — v2 still says "updated **deliberately**". |
| MA-3 | golden + schema undecided | **IGNORED** — neither named anywhere in v2. |
| MA-4 | the other journey's transcript | **IGNORED** — see H1. |
| MA-6 | which exit code | **ACCEPTABLE** — §3 measured exit 0; "prints its real exit code" now reads as "do not fake it". |
| MA-7 | the `≥ 8`-byte prefix floor | **IGNORED — and v2's own example violates it** (FI-3). |
| H1 | §3's missing prerequisite | **RESOLVED** — that is §1, with an explicit dependency clause. |
| H2 | two transcripts invalidated | **IGNORED** — see H1 below. |
| H3 | pathological transcript regenerated twice | not addressed; cosmetic, not blocking. |
| H4 | §4's decision is a latent invalidator of §3 | **RESOLVED** — §4 is documentation-only, advisory, explicitly not re-deriving. |

---

## Decisions the implementer would still be forced to make

The open item v2 *names* — §7.1, which home for §1 — is a deliberate,
single-sentence escalation to a decision-maker and I do not count it. The
following are decisions the plan does not know it is delegating.

### D1 (Critical) — §1: **how** is the restored-form id computed, and from what?

§1 specifies an output value and two locations. It specifies no algorithm and no
inputs. Executed, the value depends on all three of:

1. the template (11 slots, in canonical order),
2. each slot's **origin** — fingerprint *and* path, taken from the **cards**, and
3. each slot's xpub re-serialised at depth 0.

Evidence for (2), which the plan does not mention at all — same keys, same paths,
same template, cards re-encoded `--privacy-preserving` (no fingerprint), explicit
assignment so no search is involved:

```
$ mnemonic restore --md1 ×3 --from phrase=<master-A> --account 0 \
    --cosigner @1=… … --cosigner @10=…      # fingerprinted cards
  first recv: bc1qkuknuy6dsm0fq44cyyhzqy9wl3ex2n6ed39zxhx867l9wlh4yhlsejms64
✓ wallet-id (completed): ced2270948ecb5af0779249ac7181f4a

$ …same, but every card built with `mk encode --privacy-preserving`
  first recv: bc1qkuknuy6dsm0fq44cyyhzqy9wl3ex2n6ed39zxhx867l9wlh4yhlsejms64
✓ wallet-id (completed): d09840c3aa78035368f2dfb4bc271a27
```

Same wallet, same addresses, **different recovery target**. So "emit the recovery
target" is only well-defined relative to the exact card encoding used at engrave
time — a constraint the plan states nowhere and an implementer must discover.

Neither offered home holds those inputs:

- **(a) `md inspect --restored-form`.** `md inspect` accepts only md1 strings.
  On the engraved keyless md1 there are no keys — executed:
  ```
  wallet-policy-mode: false
  wallet-policy-id: f89e23f13c697ae62ef10328d71d7e24
  ```
  On a keyed md1 the origins are wrong, because `md encode` cannot express
  Divergent per-key origins:
  ```
  $ md inspect --json <keyed md1 built with --path bip48>
    "path_decl": { "data": "m/48'/0'/0'/2'", "tag": "Shared" },
    "fingerprints": null, "origin_path_overrides": null
  $ md encode --key "@0=[73c5da0a/48'/0'/0'/2']xpub…"
  md: --key @0: base58check decode: decode
  $ md encode "<concrete descriptor with origins>"
  md: template parse error: template contains no @i placeholders
  ```
  `md encode --help` offers exactly `--path`, `--key @i=XPUB`,
  `--fingerprint @i=HEX`. There is no per-`@N` path flag.
- **(b) `me bundle` header.** Input is an unordered string list; the id is
  order-sensitive. The plan's stated obstacle — *"`me` would have to compute an id
  it currently has no reason to know"* — understates it: `me` cannot **determine
  the order**, which is what §2 of the same document says.

The implementer must therefore invent the missing input surface (a way to feed
ordered cards, or a Divergent-origin encode path in `md`), and the plan does not
mention that work exists. **This is unbuilt work, not a flag.**

Worth stating because the plan's table says the value is printed by
"**nothing**": a producer *does* exist, one repo over, and it is not a search —
`mnemonic restore` in **explicit-assignment** mode returns it in **0 s** with the
inputs an operator has at backup time (own seed + the cards in known order). The
plan's prose rationale ("an operator cannot be asked to obtain it by performing
the recovery they are preparing for") does not apply to that mode, and §7.1's
choice is presented without it.

### D2 (Important) — §1: what does the flag/header do in the *normal* case?

For home (a): what does `md inspect --restored-form` print when the md1 is
keyless — which is always, for the engraved artifact? Error, blank, or a
different id? For home (b): does the header go on the stderr checklist, into the
stdout manifest JSON, or both? "checklist header" points at stderr, but the
manifest is the machine-readable artifact and §2 is simultaneously adding fields
to it.

### D3 (Important) — §1: **how much** of the id must the operator record?

Unspecified — and v2's own example for home (b) is *"record this to recover:
`ced22709…`"*, eight hex characters. Executed, that is refused:

```
prefix=ced22709        (8 hex)  exit=4  error: … prefix too weak for this search:
                                        need ≥8 bytes …, got 4
prefix=ced2270948ecb5 (14 hex)  exit=4  … need ≥8 bytes …, got 7
prefix=ced2270948ecb5af (16 hex) exit=0  ✓ wallet-id (completed): ced2270948ecb5af0779249ac7181f4a
```

The floor is `required_prefix_bytes(11!) = 8`
(`mnemonic-toolkit/src/permutation_search.rs:322-337`, doc ladder *"`S = 11!`→8"*).
An implementer copying §1's illustration ships an operator instruction that fails
at exit 4. See FI-3.

### D4 (Important) — §2: what are the new fields **called** in the manifest JSON?

v2 specifies the rendered checklist string byte-for-byte, then says only *"New
`PlateEntry` fields are `Option<String>` with `#[serde(skip_serializing_if …)]`"*.
It never names them. Because `skip_serializing_if` only matters for
serialisation, v2 has decided they **do** appear in `manifest.json` — a de facto
contract the plan itself calls out in §0. `origin_fingerprint`/`origin_path` vs
`fingerprint`/`path` vs `origin` is a coin-flip, and two implementers produce two
different public schemas.

### D5 (Important) — §2: where does `set <chunk_set_id>` go, and what counts as a collision?

The edge table says colliding cards *"render identically, **plus `set
<chunk_set_id>`**"*. Two gaps:

- **Position unspecified.** `mk1 [path …, no fingerprint] chunk 1/3 set 0x0d3f2`
  and `mk1 [path …, no fingerprint, set 0x0d3f2] chunk 1/3` both satisfy the
  sentence. The rest of §2 is specified to the byte precisely so this cannot
  happen.
- **Scope unspecified.** `PlateEntry` is per-**chunk**; a card is a chunk set. All
  3 plates of one card share fingerprint+path, so a naive per-plate scan reports
  every multi-chunk card as colliding with itself. Detection must group by
  `chunk_set_id` first. Not stated. Measured: the fixture is 11 cards → **30**
  plates, so 30 of 30 plates trip a per-plate rule.

### D6 (Important) — §2: must `crates/me-cli/tests/vectors/bundle-md1-mk1.json` be regenerated, and to what?

Not named anywhere in v2, and v2 asserts the opposite of the truth. See FI-1.
The implementer must decide the regenerated content (which is also D4).

### D7 (Minor) — §2: does `design/SPEC_me_bundle_phaseA.md` §6 get the new fields?

§6 (`design/SPEC_me_bundle_phaseA.md:61-88`) writes the manifest schema out
field-by-field and annotates omissions (*"`string` is omitted for the `ms1`
plate; `chunk_set_id`/`chunk_index` omitted for unchunked md1 and ms1"*). An
additive field is a spec delta. v2 never mentions the file.

### D8 (Minor) — §2: where does the privacy-preserving test fixture come from?

Acceptance requires *"a test using a `--privacy-preserving` card"*. `me-cli`'s
test corpus has no such string — `crates/me-cli/tests/cli.rs:5-7` has `MK1_A`/
`MK1_B` only, both fingerprinted (`mk decode` → `origin_fingerprint: aabbccdd`),
and `crates/me-cli/tests/vectors/` holds four `.ndef` blobs, none of them a
privacy-preserving mk1. The implementer must generate one with `mk encode
--privacy-preserving` and paste it as a literal — a step with no owner in the
plan.

### D9 (Important) — §3: the exact invocation, specifically what may be shuffled

§3 says *"cards deliberately shuffled"*. Cards are safe; **chunks are not**, and
the journey stores chunks in a flat file (`out/pathological/mk-encode-raw.txt`,
30 lines, no card boundaries). Executed, chunk-level shuffle of the same ten
cards:

```
$ mnemonic restore --md1 ×3 --from phrase=… --account 0 \
    --cosigner ×28 (globally shuffled) --expect-wallet-id ced2270948ecb5af
exit=1
error: --cosigner mk1 decode: chunked-header malformed: received 28 chunks,
       header declares total_chunks = 2
```

versus card-level shuffle (order 07 03 10 01 09 05 02 08 04 06), the same 28
arguments regrouped: **exit 0, 15–16 s**. The plan does not distinguish them, and
the natural implementation against the flat file is the one that fails.

### D10 (Minor) — §3: `--from phrase=<seed>` on argv

Every run emits `warning: secret material on argv (--from phrase=) — pipe via
--from phrase=- to avoid /proc/$PID/cmdline exposure`. The plan's §0 block shows
the argv form. Whether the journey demonstrates the warned-about form or the safe
one is a decision, and it is visible in a published transcript.

### D11 (Minor) — §4: which "template index"?

§4's convention is *"account index = template index"*. `md-codec` renumbers
placeholders to BIP-388 first-occurrence order at encode time
(`canonicalize_placeholder_indices`, exported at `md-codec/src/lib.rs:46`;
`canonicalize.rs:1-24` — *"reshapes a `Descriptor` in place"*, permuting the tree
indices, the `Divergent` paths vector and the per-`@N` TLV maps). The `@N` a
reader sees after decoding is the canonical index, not necessarily the one the
author wrote. The README section being specified is written for other people's
wallets, so it must say which. Round-0 D11, unchanged.

### D12 (Minor) — release mechanics

`SPEC_me_bundle_phaseA.md:121-123` records the lockstep (*"Bump `me` → 0.2.0;
CHANGELOG entry"*); `crates/me-cli/CHANGELOG.md` is tracked; `Cargo.toml:3` is
`0.6.0`. §2 changes user-visible output **and** the manifest schema. v2 says
nothing about a bump or an entry. Round-0 D12, unchanged.

### D13 (Minor) — §2: `origin_path` may be empty

`mk-codec/src/key_card.rs:50-56` documents *"or `Normal{0}` when `origin_path` is
empty (depth-0 / no-path key)"*, and `DerivationPath`'s `Display` on an empty
path emits the empty string — so v2's second form renders `mk1 [path , no
fingerprint] chunk 1/3`. Not in the edge table. Reachability is low via the CLI
(`mk encode --origin-path m` → `error: xpub origin-path mismatch: xpub depth 4 /
child 2' vs origin_path depth 0 / last None`), so this is a Minor, not a blocker.

---

## Factually incorrect assumptions

### FI-1 (Critical) — §2: "existing manifest consumers are unaffected" is false

v2 §2: *"New `PlateEntry` fields are `Option<String>` with
`#[serde(skip_serializing_if = "Option::is_none")]`, so plates with no card gain
no keys and **existing manifest consumers are unaffected**."*

The first clause is true (md1 and ms1 plates have no card). The second is false,
because the golden's mk1 plates **do** have a card. Executed:

```
$ mk decode "mk1qpzg69pqqsq3zg3ngj4thnxaq5zg3vs7zqsrqq…"  "mk1qpzg69ppsnz4v7cjv3qfj…"
origin_fingerprint:  aabbccdd
origin_path:         48'/0'/0'/2'
```

Those are exactly `MK1_A`/`MK1_B` (`crates/me-cli/tests/cli.rs:5-7`), and
`crates/me-cli/tests/vectors/bundle-md1-mk1.json` pins both plates today with the
key set `["chunk_index","chunk_set_id","integrity","kind","of","plate","string"]`
(measured against the live `me bundle` output). Under §2 both gain keys, so both
byte-for-byte assertions fail:

- `crates/me-cli/tests/cli.rs:308` — `assert_eq!(v, expected);`
- `crates/me-cli/tests/cli.rs:745` — `assert_eq!(v, expected, "no --preview must be byte-for-byte Phase A");`

v2's §2 acceptance does say `crates/me-cli/tests/cli.rs` must be *"updated
deliberately"*, which contradicts the sentence above it — but the file that
actually holds the bytes (`tests/vectors/bundle-md1-mk1.json`) is never named, and
neither is what the new content must be. This is round-0 D2/F5/MA-3, restated
rather than resolved, and now carrying a false claim.

### FI-2 (Important) — §1: the stated root cause is incomplete

v2 §1 attributes the id divergence entirely to F-130: *"`restore` re-serialises
the completed descriptor's xpubs with BIP-32 metadata **zeroed** … Different
serialisation → different descriptor string → different `WalletPolicyId`."* The
F-130 citation itself is accurate (`design/journeys/README.md:63`, verbatim).

But the completed id also moves with **card metadata**, holding the xpubs and the
serialisation constant — the `--privacy-preserving` measurement in D1 above:
`ced2270948ecb5af…` vs `d09840c3aa780353…`, identical `first recv`. And the
descriptor `restore` prints carries the cards' four *distinct* origins
(`[73c5da0a/48'/0'/0'/2']`, `[73c5da0a/48'/0'/1'/2']`, … `[28645006/48'/0'/2'/2']`),
whereas the only keyed md1 `md` can build carries `Shared m/48'/0'/0'/2'` with
`fingerprints: null`.

This matters operationally: it is *why* a "restored-form" flag on an id-only tool
cannot reach the value, and it is the fact an implementer needs before choosing
between §7.1(a) and (b).

Related, and under-specified rather than false: the §1 table's row 3
(`WalletPolicyId`, keyed policy = `232214e4d60c0fa83a6715ba2f7e8ec7`, *"printed by
`md inspect` on a keyed encode"*) is reproducible only for **one particular**
keyed encode. Measured:

```
--key ×11 + --path bip48                    → 232214e4d60c0fa83a6715ba2f7e8ec7   (the plan's value)
--key ×11 + --fingerprint ×11 + --path bip48 → 9f5da760a03ccd0c0d8e3ea819a31358
--key ×11, no --path                         → no wallet-policy-id at all (partial decode; md warns)
```

"A keyed encode" is not one thing, and the table presents it as one.

### FI-3 (Important) — §1's own recording example is a prefix the tool refuses

§1 home (b): *"record this to recover: `ced22709…`"* — 8 hex characters. Executed
(full output in D3): `exit=4`, *"prefix too weak for this search: need ≥8 bytes …,
got 4"*. Round-0 MA-7 named this floor; v2 does not state it in §1's acceptance,
in §3's step 3 (which lists only the `n ≤ 34` ceiling, the 3600 s cap and
`--accept-search-time`), or anywhere else — and then illustrates the exact
failing length.

### FI-4 (Minor) — the `--search-address` estimate is quoted as if it were stable

§0: *"error: estimated exhaustive search time **890788.897152s** exceeds the 3600s
ceiling"*, presented as the measurement. The figure is a per-candidate
micro-calibration and is not reproducible. Eight observations of the identical
command on this machine (four here, four in round 0): 829,804 / 905,472 /
1,735,762 / 1,945,064 / 2,195,079 / 2,307,913 / 2,364,356 / 2,876,401 s — a 3.5×
spread, none equal to the quoted value. Refusal time 0.082–0.208 s, so "84 ms" is
the fast end of a real range. **The conclusion is robust** (every value is 230×–800×
the 3600 s ceiling) and nothing in the plan is built on the number, so this is a
precision defect in a document that opens by claiming every fact was measured —
not a defect in the decision.

### Checked and **correct** — recorded so a later round does not re-derive them

- `restore.rs:1396` is the missing-`--from` message. ✓
- `KeyCard` fields and `Option<Fingerprint>` — `mk-codec/src/key_card.rs:34-57`. ✓
- `me-cli` has `serde`+`serde_json` and **no** `bitcoin` — `Cargo.toml:22-27` is
  `md-codec, mk-codec, clap, zeroize, serde, serde_json`; the only `bitcoin` in
  the file is `keywords` on line 10. The `String` rationale is sound: rendering
  needs `Display`, which does not require naming the type. ✓
- `bundle.rs:279` is `mk_codec::decode(&refs)` with the `KeyCard` discarded. ✓
- `manifest.rs:82-108` is the checklist loop; `manifest.rs:228` is
  `assert!(c.contains("mk1 chunk 1/2"), "{c}")`. ✓
- Plates are emitted in `chunk_set_id` order — `BTreeMap` at `bundle.rs:205-208`. ✓
- `'` not `h`: `mk decode` prints `origin_path: 48'/0'/0'/2'`; `ChildNumber`'s
  `Display` writes `'` unless `f.alternate()` (`bitcoin-0.32/src/bip32.rs:210-221`);
  `DerivationPath`'s `Display` emits no leading `m/` (`:459-471`). The `{}` vs
  `{:#}` fork is real and v2 closes it. ✓
- "All 30 card plates" — measured: 11 cards → 30 mk1 chunks → `me bundle` prints
  30 `mk1 chunk` lines and the manifest holds 30 `mk1-chunk` plates. ✓
- The 16 s recovery, the id, the slot, the address — reproduced exactly
  (`exit=0`, `elapsed=16s`, `ced2270948ecb5af0779249ac7181f4a`, `slot @0`,
  `bc1qkuknuy…ejms64`). ✓
- `n ≤ 34` (`permutation_search.rs:91`, `:479`), `SEARCH_CEILING = 3600s`. ✓
- *"`Unique` is proven-unique rather than first-match"* — `permutation_search.rs:20-27`,
  `:915-924`. ✓
- §4 limit 2 — `@0-@3` are `73c5da0a` accounts `0'-3'`; `@4` is `b8688df1/48'/0'/0'/2'`. ✓
- `me` has no id-producing command (`bundle`, `sysw`, `seal`, `hash`), so "no
  command in this repo prints that value" is true **for this repo**. ✓

---

## Missing acceptance criteria

1. **§1 — the acceptance is unsatisfiable under both offered homes** (D1). It
   names an output value but no producible input path. Per the project's own
   rule, a plan may not close while one of its gates has never been run; this one
   cannot be run at all as scoped.
2. **§1 — nothing defines how much of the id the operator records** (D3/FI-3),
   which is the single number that decides whether the documented recovery works.
3. **§2 — the golden.** `tests/vectors/bundle-md1-mk1.json` must change and "done"
   for it is undefined (D4/D6/FI-1).
4. **§2 — SPEC §6.** No criterion; not mentioned (D7).
5. **§2 — "updated deliberately" is still an intent, not a criterion.** Round-0
   MA-2, verbatim. What must the new `manifest.rs:228` assertion say?
6. **§2 — the collision behaviour has a definition but no test.** The edge table
   defines it; no acceptance bullet exercises it. The `--privacy-preserving`
   bullet tests the *no-fingerprint* form on one card, which is a different case —
   measured, the fixture's 11 cards collapse to **4** distinct path-only labels
   under privacy-preserving encoding, so collision is that mode's normal state.
7. **§2 — the other transcript.** See H1.
8. **§4's acceptance is real and I would accept it**: a named file, a named
   section title, and "states all three limits" is checkable by reading three
   sentences. Round-0's harshest finding (§2 had no deliverable at all) is closed.

---

## Hidden dependencies / ordering problems

**H1 — §2 invalidates `design/journeys/transcript.txt`, which currently cannot be
regenerated, and v2 does not mention it.** Measured line counts of the string
being changed: `transcript_pathological.txt` **31**, `transcript.txt` **24**. v2's
§2 acceptance names only the pathological checklist. `design/FOLLOWUPS.md:7561`
(F-210, **open**, owning phase *"the arbitrary-`tr()`/`wsh()` cycle"*) records a
fresh run of `transcript.sh` producing **9** non-zero exits against the committed
**1**, with `mk`/`ms`/`me` all moved. So §2 makes 24 committed lines false in a
document whose regeneration path is a known-open defect. Either §2 depends on
F-210, or the plan must say those lines stay stale on purpose. Round-0 H2,
unchanged.

**H2 — §1's dependency clause is correct, and is the plan's best feature.** *"Step
2 cannot ship before §1, or the journey must hardcode an id nothing produces"* is
exactly right, and §3's acceptance (*"the target id it uses is one an **earlier
step printed**, not a literal"*) enforces it mechanically. The problem is upstream:
§1 as scoped cannot produce that earlier step (D1), so §3 inherits an
unsatisfiable precondition rather than a missing one.

**H3 — §6's order text is internally inconsistent, harmlessly.** The heading says
**§1 → §2 → §3 → §4**, the body says *"§2 is independent and may run in parallel"*.
Both readings work; a reader has to pick. Not blocking.

**H4 — §1 and §2 both rewrite the pathological checklist, §3 rewrites the same
transcript again.** Three regeneration passes over one artifact. Cosmetic
(round-0 H3), not blocking.

**H5 — §4's re-derivation question is correctly defused.** By making §4
documentation-only, advisory, and explicitly *not* re-deriving the fixture, v2
removes the latent invalidator round 0 flagged as H4. Resolved.

---

## Open / could not determine

- **Whether a Divergent-origin keyed md1 is producible by any route.** The wire
  format supports it (`md inspect --json` exposes `path_decl.tag` and the TLV
  `origin_path_overrides` / `fingerprints` slots), and `md-cli` clearly cannot
  emit one. I did not read `md-codec`'s encoder API far enough to say whether a
  library caller could, which bears directly on whether §1(a) is *buildable at
  cost* or *not buildable*.
- **Whether `transcript_pathological.sh` regenerates cleanly today.** I rebuilt
  its fixture stages by hand (md1, cards, `me bundle`) and all succeeded; I did
  not run the whole script, and F-210 is open against all three journeys.
- **Whether the `tr()` pathological variant behaves the same.**
  `inputs-pathological/backup-strings-tr.txt` and `wallet-policy-tr.txt` are
  tracked; I exercised only the `wsh` path. Project memory warns explicitly that
  measuring one descriptor path gives a wrong answer about the other.
- **Whether an `Ambiguous` outcome is reachable for this wallet.** Both my
  searches returned `Unique` or a floor/ceiling refusal. §3 does not promise to
  document all three outcomes any more (v1 did), so this is smaller than in round
  0.
- **Whether the estimate/realized ratio holds on a smaller machine.** Measured on
  24 cores. The estimate is a single-thread projection and varied 3.5× across
  eight runs; the realized 15–16 s would be ~6× longer on 4 cores. §3 commits the
  demonstration to a journey that must complete every run, on the strength of the
  realized number only.

---

## One line on what would make v2 executable

Answer §1's real question — *which tool will be given the ordered cards, and how
does it compute the id* — before answering §7.1's *where does it print*; state the
16-hex recording floor and fix the `ced22709…` example; replace §2's
"consumers are unaffected" with the golden vector's filename, the new JSON key
names, and SPEC §6; say whether chunks may be shuffled; and say what happens to
`transcript.txt`'s 24 lines.
