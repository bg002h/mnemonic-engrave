# Adversarial critique — PLAN_key_index_legibility.md (commit 46efbae)

Reviewer: independent read-only agent, 2026-08-19. Every claim below was
machine-checked against the working tree (mnemonic-engrave master, plus the
sibling repos `mnemonic-key` and `mnemonic-toolkit` at their current checkouts).
Nothing was executed against live search runs; where a fact would require a run,
it is listed under **Open** rather than asserted.

## Verdict

The plan's code-level factual base is largely sound — bundle.rs:279 really does
decode-and-discard a full `KeyCard`, the label really does omit the key, and
"`me` cannot print `@N` for a keyless template" holds (all 11 pathological cards
carry the *same* order-independent template-id stub `5b48af35`). But the plan is
not executable as written. **Item 3 is the weakest**: its README sentence
("given the template and all N cards in any order, `restore` recovers the
assignment") is false in that shape — restore hard-requires the operator's own
seed via `--from`, a search *target* (`--expect-wallet-id` of ≥ 8 bytes at
n=11, or a known address), and the pathological journey records no such target
today; its sortedmulti/"Ambiguous is harmless" claim mis-describes the actual
engine behavior; and both items' acceptance criteria run through the
pathological journey, which per F-210 currently cannot regenerate at all — so
every acceptance gate in the plan is, today, a gate that cannot execute. Items
1 and 2 are salvageable with re-scoping; item 3 needs its preconditions written
in and a one-time feasibility run before it can be sized "S".

## Holes

### C1 (Critical) — Item 3 documents a recovery path that does not exist in the stated shape

The plan's §3 change 1 proposes README text: *"given the template and all N
cards **in any order**, `restore` recovers the assignment."* Three preconditions
are missing, each a hard floor in the tool:

1. **The operator's own seed is required.** Keyless multisig-template completion
   refuses without `--from`:
   `mnemonic-toolkit/crates/mnemonic-toolkit/src/cmd/verify_bundle.rs:899-903` —
   *"verifying a keyless MULTISIG TEMPLATE bundle requires the operator's own
   seed via --from \<seed\> (the template carries no keys; the seed derives your
   cosigner key, and --cosigner \<mk1\> supplies the others)"*. Cards alone are
   not an accepted input shape. For the pathological fixture the demo also needs
   the own-account list (master A holds 4 of the 11 keys).
2. **A search target is required and none is recorded anywhere in the journey.**
   The engine matches candidates against *"a recorded `--expect-wallet-id`
   (id-search) or a known `--search-address`"* (`cmd/restore.rs:1125-1127`); with
   neither, it refuses (`restore.rs:2105`). The cards cannot supply the target:
   their `policy_id_stubs` are the key-order-independent
   wallet-descriptor-template-id (transcript_pathological.txt:47,86 — all 11
   cards carry `5b48af35`), and even if a keyed-id stub were present, 4 bytes is
   below the tool's own floor — `required_prefix_bytes` pins **8 bytes for
   S = 11!** (`permutation_search.rs:300-338`, ladder: "n=11, own=4 — S=11!→8").
   The pathological journey never assembles the keyed wallet or records its
   `wallet-policy-id`, so the demo has nothing to search against until a
   *new creation-time step records it* — a step the plan never mentions.
3. **The time cap can refuse.** Estimate > 1h → refusal without
   `--accept-search-time ≥ estimate` (`permutation_search.rs:59`
   `SEARCH_CEILING = 3600s`; `cap_decision` at :387-415). 39.9M candidates ×
   an unmeasured per-candidate cost may land on either side of that ceiling;
   the plan neither measures nor mentions it (see I4).

Why Critical: this is precisely the plan's own diagnosis — *"a documentation
failure with a recovery-shaped consequence"* — about to be re-committed in the
opposite direction: promising a recovery input shape the tool refuses.

### C2 (Critical) — Every acceptance gate in the plan runs through a journey that cannot run

Item 1 acceptance: "Checklist for the pathological journey names an origin on
all 30 card plates." Item 3 acceptance: "The pathological journey demonstrates
recovery from shuffled input." Both require re-running
`transcript_pathological.sh` — and **F-210** (design/FOLLOWUPS.md:7561) records,
measured on 2026-08-18, that all three journey transcripts consume intermediates
nothing writes (fresh run: 9 non-zero exits vs 1 committed) and the tool
versions have moved underneath them. The plan never mentions F-210, never
sequences after its repair, and its §5 ordering ("1 first: nearly free")
presents item 1's acceptance as cheap when its acceptance vehicle is broken.
Per the follow-up-burndown rule, F-210's owning phase is "before it leans on
the pathological journey" — this plan leans on it, so the repair becomes
gating work this plan must own or explicitly sequence behind. House rule
applies verbatim: a gate that has never been executed is a hypothesis — here,
*cannot* be executed.

### I1 (Important) — The sortedmulti paragraph mis-describes the engine, in both §0 and §3

Plan §0: *"`sortedmulti()` makes every permutation the same script, so the index
is irrelevant and `Ambiguous` is harmless."* Plan §3: docs must say `Ambiguous`
*"is harmless there — otherwise it reads as a failure."* The engine does not
behave this way (`cmd/restore.rs:1945-1954`):

- **Address-search collapses n! → 1** for order-independent shapes (identity
  placement only), returning `Unique` — `Ambiguous` is engineered *out*, not
  tolerated.
- **Id-search is deliberately NOT collapsed**: *"`compute_wallet_policy_id`
  never sorts … the recorded id pins a SPECIFIC order the search must still
  resolve. Verified: sortedmulti AB-id ≠ BA-id."* So for a sortedmulti wallet
  under id-search the index is **not** irrelevant — the recorded identity pins
  the original assignment, and the search resolves it to `Unique`.

Meanwhile `complete_multisig_template` treats `Ambiguous` as a refusal ("On
NO-MATCH / AMBIGUOUS / any floor violation it RETURNS the (refuse) error",
restore.rs:1460-1470). Documentation telling operators "Ambiguous is harmless
for sortedmulti" would describe an outcome the engine doesn't produce there,
and would soften a signal that is, everywhere it actually occurs, a refusal.
This is funds-safety prose; it must match the code.

### I2 (Important) — §2's lint is unimplementable in the stated form

The lint is to note "when a bundle's cards do *not* follow" account-index =
template-index. But `me` cannot know a card's template index — that is item 1's
own central claim, and it is correct. The most `me` can check is the weak
necessary condition (the account components form exactly {0..N−1}, each once,
with N read from the decoded md1 template). A bundle can pass that check while
the keys actually sit at *different* positions in the descriptor (a
generation-time mistake §2 exists to catch), so the lint as imagined would
mint false confidence in the one case that matters. Additionally, "the account
component" requires path-family parsing (BIP-48 `m/48'/coin'/acct'/script'` vs
BIP-87 `m/87'/coin'/acct'` vs the mk1 `0xFE` escape-hatch paths, where
"account" is undefined) — the completion engine already had to learn this
lesson (restore.rs:1130-1137: the toolkit's multisig emit defaults to BIP-87,
not BIP-48). "S lint" is undersized and its claim must be weakened to what is
actually checkable.

### I3 (Important) — §2 contradicts §4's rationale

§4 rejects an mk1 index field because it *"couples a card to a position in ONE
wallet when the same key may sit at different indices in different wallets."*
§2's convention carries the identical coupling one layer down: a keyholder whose
key serves wallet A at @1 and wallet B at @2 cannot satisfy account=index in
both with one key — the convention forces a fresh derivation per (wallet,
index) pair. That may even be desirable (no cross-wallet key reuse), but the
plan cannot use non-coupling to reject the wire field and then adopt a coupled
convention without acknowledging it inherits the same cost. Related interop
cost, unstated: cosigner software commonly assumes account 0' for all
participants; a §2-conformant wallet has cosigners at accounts 0..N−1, which
third-party recovery tooling may not scan by default.

### I4 (Important) — Item 3's feasibility was never measured, and the shape has never been run

Two distinct unknowns, both checkable before the plan is sized:

- **Wall-clock.** 39,916,800 candidates × per-candidate (build 11-key
  descriptor + `compute_wallet_policy_id`, restore.rs:2012). At 1µs → 40s
  (silent); at 10µs → ~7min (progress bar); at 100µs → ~66min (**refused**
  without `--accept-search-time`). Nobody has measured the constant; the plan's
  own house rule is "never hand-count what a tool can count."
- **Shape support.** The completion engine's general/thresh path (P3b) is
  proven only at ≤ 3 slots and simple archetypes
  (`tests/prop_template_completion_roundtrip.rs:219-222` — `or_i(pk,pk)`,
  `or_i(pk, and_v(v:pk,pk))`, thresh-3). The pathological template is an `or_i`
  tree over **four separate `multi()` groups** with `sha256()` hashlocks and
  all four timelock kinds at **11 slots** — never executed through restore.
  The early-refusal gates (hardened use-site, taproot subset,
  restore.rs:1509-1537) should pass it (`/<0;1>/*` is unhardened, wsh not tr),
  but "should" is a hypothesis. One explicit-assignment run
  (`--cosigner @N=…`, no search) would settle shape support in minutes.

Consequence for §5's ordering claim: item 3 is not "pure documentation plus a
demonstration" — it is a first execution of an unexercised engine path at an
unprecedented scale, plus new journey plumbing (record the target id at
creation). That is discovery work and should be costed as such.

### M1 (Minor) — The §2 fixture description is wrong against the measured fixture

Plan: *"@0-@3 = master A accounts 0-3, but @4 = master B account 0."* Measured
(`design/journeys/inputs-pathological/keys/key-*.xpub`): @0 = A/account **1'**,
@1 = A/account 0', @4 = B/account **3'** (not 0). Masters hold 4/4/3 keys
(A/B/C) and accounts are deliberately scrambled *within* each master. The
fixture is more pathological than the plan says — which actually strengthens
§2's "re-derive?" question — but a section titled "the situation, measured"
must not contain an unmeasured claim. Bonus measured fact the plan could use:
path `48'/0'/0'/2'` occurs **three times** (A, B, C), so under
privacy-preserving encoding this very fixture would produce three identical
path-only labels — §1's "edge case" is the fixture's actual behavior.

### M2 (Minor) — Item 1's touch-point list is incomplete; "nearly free" is close but understated

Named by the plan: `bundle.rs`, `manifest.rs:82-108`, the `:228` test. Missing:

- **The byte-pinned golden manifest** `crates/me-cli/tests/vectors/bundle-md1-mk1.json`,
  asserted at `crates/me-cli/tests/cli.rs:306` and `:743` ("no --preview must be
  byte-for-byte Phase A") — new `PlateEntry` fields on mk1 plates change it.
- **The manifest schema is a specced contract**: `design/SPEC_me_bundle_phaseA.md`
  §6 documents the JSON shape; an additive field needs a spec delta. "No
  normative change" is fair in the codec sense, but the schema is not
  contract-free.
- Consumers verified safe: `design/journeys/build_pdf_pathological.py:265-272`
  reads only `plate`/`kind`/`string` (additive fields fine);
  `build_pdf.py:133` lifts the checklist with a tolerant regex
  (`me: backup needs .*?(?=\n\[exit)`); the Go sidecar does not parse the
  manifest. So the blast radius is genuinely small — but the golden and the
  spec belong on the list, and the *acceptance* as written is journey-coupled
  (see C2).

### M3 (Minor) — §6's "Unverified" md1-availability question was a two-minute grep, left undone

`run_bundle` partitions md1 strings in the same function that builds the
manifest (`bundle.rs:206-208, 249`), so yes — the md1 is in hand at
checklist-build time, and `md-codec` (already a dep) can decode it. By the
project's own build-gate rule, a machine-checkable claim should have been
checked before reaching a reviewer. The answer also matters: for a **keyed**
md1, matching each card's xpub against the template's key vector yields an
exact `@N` with no search — a branch the plan defers as an open question but
which should be a named, decided option (both current journeys use keyless
templates, so origin-only remains the right default).

### M4 (Minor) — The proposed label omits the one wallet-binding fact the card carries, and `me` never checks it

Plan §0 itself observes a card says *"I belong to wallet `5b48af35`"* — yet the
§1 label prints only origin, not the stub, and `me bundle` performs **no**
stub-vs-template cross-check today (zero matches for "stub" in bundle.rs). In a
multi-wallet bundle (run_bundle accepts "one or more wallet backups",
bundle.rs:180), two same-origin cards serving different wallets would label
identically; the stub — cross-checked against the bundled template's id —
disambiguates and catches a mixed-up bundle. Cheap, uses data already decoded,
and strictly increases the label's honesty.

### M5 (Minor) — Empty-path degradation unhandled

`origin_path` is non-optional but may be *empty* (depth-0 key,
key_card.rs:50-56). The §1 fallback `mk1 [path only: …]` renders as
`[path only: ]`. Add the case to §1's edge list.

### N1 (Nit) — "The pathological wallet uses unsorted multi" under-describes the shape

It is four `multi()` groups inside an `or_i` tree — a *general* (P3b) template,
not canonical multisig (P3a). The distinction decides which completion-engine
path item 3's demo exercises (see I4) and should be stated.

### N2 (Nit) — Adjacent rot the plan will trip over

`build_pdf_pathological.py:315-316` static text still says "The 25 public
plates … eleven key cards at two chunks each" while the committed transcript
shows 33 public plates / 30 mk1 chunks ("BIP-48 origins push most cards to 3").
Pre-existing, but item 1's acceptance walks straight through these files.

## Claims checked

| Claim (plan) | Holds? | Evidence |
| --- | --- | --- |
| `me bundle` decodes each mk1 set and discards the result at bundle.rs:279 | **Yes** | `crates/me-cli/src/bundle.rs:279-280` — `mk_codec::decode(&refs).map_err(…)?;` Ok value dropped; `decode` returns `Result<KeyCard>` (`mnemonic-key/crates/mk-codec/src/key_card.rs:159`) |
| `origin_fingerprint` is `Option` at key_card.rs:40 | **Yes** | `mnemonic-key/crates/mk-codec/src/key_card.rs:40`; privacy mode omits only the fingerprint — `origin_path` always present (mk-cli `encode.rs:76-78`) |
| `permutation_search` returns proven-`Unique` (no-second-match) | **Yes** | `permutation_search.rs:20-27, 915-925`; quoted contract text matches verbatim |
| Wired into `restore` and `verify-bundle` | **Yes, with a caveat** | restore directly (`cmd/restore.rs:1477, 2113`); verify-bundle *indirectly* via shared `complete_multisig_template` (`cmd/verify_bundle.rs:888-891`); verify_bundle.rs itself has 0 direct references. Cited path is wrong: actual file is `mnemonic-toolkit/crates/mnemonic-toolkit/src/permutation_search.rs` |
| Checklist omits the key (manifest.rs:82-108); `:228` asserts `"mk1 chunk 1/2"` | **Yes** | `crates/me-cli/src/manifest.rs:94` label = `mk1 chunk {idx}/{total}`, no origin; test at `:228` exactly |
| `me` cannot print `@N` for a keyless template | **Yes (as scoped)** | All 11 cards carry identical template-id stub `5b48af35` (transcript_pathological.txt:47,86); template carries no keys; me has no search target. Keyed-md1 branch would make `@N` derivable — md1 *is* available in `run_bundle` (bundle.rs:206-208), so §6's "Unverified" was checkable (M3) |
| Plates emitted in chunk_set_id order, not key order | **Yes** | `BTreeMap<u32,…>` keyed by chunk_set_id (bundle.rs:205-208, 276); independently: build_pdf_pathological.py:260-262 documents the same fact from a past silent-wrong-caption bug |
| Ceiling n ≤ 34; 11! = 39,916,800; 12! = 479,001,600 | **Yes** | `permutation_search.rs:478-486` (factorial → None > 34), `:125` error text; arithmetic checks |
| `sortedmulti` → same script, index irrelevant, `Ambiguous` harmless | **No** | restore.rs:1945-1954: address-search collapses n!→1 (→ `Unique`); id-search resolves a SPECIFIC order ("sortedmulti AB-id ≠ BA-id"); `Ambiguous` is a refusal wherever produced (restore.rs:1460-1470) — see I1 |
| Pathological wallet uses unsorted `multi` | **Yes, imprecisely** | `inputs-pathological/wallet-policy.txt` — four `multi()` groups in an `or_i` tree (general shape, not canonical multisig) |
| §0 checklist excerpt `plate 20/34 mk1 chunk 1/3` | **Yes, verbatim** | transcript_pathological.txt:136,156; 34 = 3 md1 + 30 mk1 + 1 ms1 |
| `mk-codec` already a dep at Cargo.toml:23 | **Yes** | `crates/me-cli/Cargo.toml:23` — `mk-codec = "0.4"` |
| "30 card plates" | **Yes** | transcript: "11 key cards -> 30 mk1 chunks"; card-index.txt wc = 30 |
| "@0-@3 = master A accounts 0-3, @4 = master B account 0" | **No** | keys/key-04.xpub = `[b8688df1/48'/0'/3'/2']` (B, account 3'); @0 = A account 1' — see M1 |
| §2 "strictly stronger than lexicographic ordering" | **Contestable** | Both are creation-time conventions that can be silently violated; account=index is more *legible* (read one card) but no more *verifiable* — a mis-derived card confidently lies, and only the permutation search (item 3) can prove the actual assignment either way |
| Item 3 demo input shape accepted by `restore` | **Partially** | Bare (unassigned) `--cosigner` search-form exists (restore.rs:1556+, greedy chunk grouping) — but `--from` + a target are hard floors; "all N cards in any order" alone is refused — see C1 |

## What the plan misses

1. **Record the keyed wallet id at creation — the enabler for everything.** One
   checklist/README line ("record your wallet-policy-id; ≥ 8 hex bytes is the
   floor for an 11-key recovery search") turns item 3 from impossible to
   routine, and is the single cheapest change in this whole space. The plan
   never mentions that the search needs a target.
2. **The journey already solved per-plate `@N` labeling — generalize it.**
   `transcript_pathological.sh:182-199` writes `card-index.txt`
   (string → key-index, fp, path) and `build_pdf_pathological.py:301` captions
   every plate `plate {n} — @{ki} [{fp}/{path}] chunk {nth}/{tot}`. The
   mechanism is proven. A `me bundle --labels <file>` sidecar (operator- or
   generator-supplied string→label map) prints exact `@N` in the checklist with
   zero wire change — strictly stronger than item 1 wherever generation is
   coordinated, degrading to item 1's origin label where it isn't.
3. **The restore document as the carrier.** `build_pdf.py:284` already renders a
   per-key fingerprint table. A printed table with blank `@__` boxes the
   keyholders fill in at the generation ceremony directly implements the
   operator's own suggestion ("generation be done with all key holders") with
   no code at all — the plan engages that suggestion only obliquely via §2.
4. **Stub cross-validation in `me bundle`** (M4): check each card's
   `policy_id_stubs` against the bundled template's id; label the binding.
   Catches a wrong-wallet card at engrave time — the most expensive moment.
5. **The keyed-md1 exact-`@N` branch** decided rather than left open (M3).
6. **The tr() second path.** `wallet-policy-tr.txt` exists and project memory
   warns "two descriptor paths — measuring one gives a wrong answer." Item 3's
   demo covers wsh only; the tr variant meets restore's taproot restorable-subset
   gates (restore.rs:2609-2613). Say which paths the demo proves and which it
   does not.
7. **Hand-marking as the zero-tech fallback**: a checklist line at generation
   time — "write the slot number on the card sleeve / spare plate corner now" —
   costs nothing and survives every failure mode above. §6 considers machine
   plate-furniture only.

## Recommended changes to the plan

1. **Rewrite §3 around the true preconditions** (own seed via `--from` +
   `--account` list, recorded ≥ 8-byte wallet id or known address,
   `--accept-search-time` for large n) and add the "record the wallet id at
   creation" step to the same journey/README change. This dissolves C1.
2. **Fix the sortedmulti paragraphs** (§0 and §3) to match
   restore.rs:1945-1954: address-search collapses to `Unique`; id-search
   resolves a pinned order; `Ambiguous` is a refusal where produced (I1).
3. **Sequence explicitly against F-210**: either gate items 1 and 3 acceptance
   on the journey repair (and say so), or re-scope item 1's acceptance to
   CLI/unit tests + golden regeneration so it does not hang on a journey that
   cannot run (C2).
4. **Before sizing item 3, run two cheap probes**: (a) a per-candidate cost
   measurement at n=11 to place the demo against the 30s/1h thresholds; (b) one
   explicit-assignment (`--cosigner @N=`, no search) restore of the pathological
   template to prove P3b accepts the shape (I4). Both are minutes; either can
   invalidate the item.
5. **Re-scope §2's lint** to the weak checkable condition ({accounts} ≡
   {0..N−1}), state what it cannot prove, name the path-family parsing work,
   and acknowledge the §4-coupling contradiction (I2, I3).
6. **Complete item 1's touch list**: golden `bundle-md1-mk1.json`
   (cli.rs:306/:743), SPEC_me_bundle_phaseA §6 schema delta, empty-path label
   case; decide the keyed-md1 branch; consider the stub cross-check label
   (M2-M5).
7. **Correct the §2 fixture parenthetical** to the measured origins (M1) — and
   use the measured fact that path `48'/0'/0'/2'` occurs three times as the
   concrete privacy-mode collision example.

## Open / could not determine

- **Per-candidate search cost at n=11** — not measured here; nothing in either
  repo pins a throughput number. Until measured, the item 3 demo's wall-clock
  (seconds vs refused-at-1h) is unknown.
- **Whether restore's P3b engine accepts the exact pathological shape**
  (four multi groups + sha256 leaves + 4 timelock kinds at 11 slots). Archetype
  coverage stops at 3 slots; the early-refusal gates appear to pass it; not
  executed.
- **What `wallet-policy-id` printed by `md inspect` on a *keyless* template
  denotes** (transcript_pathological.txt:48 shows one). If key-independent, as
  I believe, it cannot serve as the item-3 search target; I did not trace
  md-codec's identity computation to prove it.
- **Whether the tr pathological variant falls inside restore's restorable
  taproot subset** (its internal key `50929b74…` is the BIP-341 NUMS point,
  which suggests yes; the multi-leaf tap tree may not be — unverified).
- **Whether any *external* consumer reads the manifest JSON** beyond this repo's
  tests and journey scripts — none found (`gh`-level search not performed), so
  "no consumers outside the repo" rests on local evidence only.
