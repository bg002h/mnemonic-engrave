# PLAN — helping the operator know which key is which

**v3, 2026-08-19.** v1 (`2d9fe3e`) and v2 (`53b2e82`) were both found not
executable. **v2's headline section was false at its root and is deleted.**

Reviews: `plan-key-index-legibility-critique.md`, `…-architect-r0.md`,
`…-critique-v2.md`, `…-architect-r0-v2.md` in `design/agent-reports/`.

**What went wrong twice, stated so it does not happen a third time.** v1 chose a
flag from its doc string that is refused in 84 ms. v2 asserted "no command
prints the recovery target" without checking `verify-bundle` — which prints it
in **4 ms** — and diagnosed a root cause from a correlation without testing it.
**Every claim below is pasted command output.** §8 is a ledger of every prior
finding, so none can be silently dropped again.

Originating question: *"there is no way to know what seed phrase / private key
material goes with which index."*

---

## 0. Measured facts

**A card carries no index.** `KeyCard` = `policy_id_stubs`,
`origin_fingerprint` (**`Option`**), `origin_path`, `xpub`
(`mnemonic-key/crates/mk-codec/src/key_card.rs:34-57`).

**The recovery target IS already printed, at backup time, in 4 ms:**

```
$ mnemonic verify-bundle --network mainnet --from phrase=- \
    --md1 ×3 --cosigner @N=… ×28 --mk1 ×2
✓ wallet_completed: completed WalletPolicyId ced2270948ecb5af0779249ac7181f4a;
  first receive bc1qkuknuy6dsm0fq44cyyhzqy9wl3ex2n6ed39zxhx867l9wlh4yhlsejms64
warning: explicit --cosigner @N= mode builds the wallet from the ASSERTED
  key→slot assignment WITHOUT verifying it against a recorded id/address.
  A wrong assignment produces a wrong wallet silently. Record + check
  --expect-wallet-id or a receive address.
```

The tool already tells the operator to record it. **Nothing in the journey runs
this, and no document mentions it.** That — not a missing capability — is the
whole gap.

**Recovery from shuffled cards works.** Own seed + the other ten cards
scrambled, `--expect-wallet-id`: 39,916,800 candidates, **exit 0 in 16 s**,
correct slot, address matching the journey's.

**A seed IS required** for a keyless template
(`mnemonic-toolkit/crates/mnemonic-toolkit/src/cmd/restore.rs:1396`). Model:
**own seed + the other N−1 cards + a recorded target**.

**`--search-address` is REFUSED at n=11** in 84 ms — *"estimated exhaustive
search time 890788.897152s exceeds the 3600s ceiling"*.

**The `--expect-wallet-id` floor is 8 bytes / 16 hex.** Measured:

```
ced22709          ( 8 hex) → error: multisig-template-floor mismatch
ced2270948ecb5af  (16 hex) → wallet-id (completed)
```

**Two independent causes make the completed id differ from a keyed `md encode`
id** — and neither is BIP-32 metadata. The id preimage stores xpubs as
65-byte `[chain_code‖pubkey]`, so depth/parent/child are **never hashed**.
v2 blamed F-130 from a `depth=4` vs `depth=0` observation without testing it.
The real causes: **`--path bip48` origin-flattening**, and **omitted
fingerprints** (`--privacy-preserving` renders
`origin_fingerprint: (omitted, privacy-preserving mode)` and moves the target
to `d09840c3…` with identical addresses).

**`me-cli` has `serde` + `serde_json`, no `bitcoin`**
(`crates/me-cli/Cargo.toml:22-27`).

**The checklist string appears in BOTH committed transcripts** — measured
`grep -c 'mk1 chunk'`: `design/journeys/transcript.txt` **24**,
`design/journeys/transcript_pathological.txt` **31**.

**The manifest golden is byte-pinned by two full-JSON-equality assertions** —
`crates/me-cli/tests/cli.rs:308` and `:745`, both `assert_eq!(v, expected)`
against `crates/me-cli/tests/vectors/bundle-md1-mk1.json`. Any new field on a
card-bearing plate breaks both; `skip_serializing_if` does not help, because the
golden's mk1 plates **have** cards.

---

## 1. Make the operator record the recovery target — **S**

v2 proposed building an emitter and escalated which tool should own it. Both
candidates were impossible and the capability already existed. **Deleted.**
What remains is small.

### The change

1. **Journey step.** After `me bundle`, the pathological journey runs
   `verify-bundle` in explicit `--cosigner @N=` form and prints the completed
   `WalletPolicyId` and the tool's own warning verbatim.
2. **README.** `design/journeys/README.md` gains a section stating: this id is
   what `--expect-wallet-id` needs at recovery; **record at least 16 hex**;
   it is **not** the id `md inspect` prints for the template or for a keyed
   encode — those are different values for the same wallet, for the two
   reasons in §0.

### Acceptance

- The pathological journey prints `ced2270948ecb5af0779249ac7181f4a` and the
  warning, from a step in the committed script.
- §3's demonstration consumes **that printed value**, not a literal.
- The README states the 16-hex floor and names the three other ids as *not*
  the target.

### Not solved

An operator who recorded **no** target. `--search-address` is refused at n=11,
so for an 11-key wallet there is **no no-target recovery path**. Say so.

---

## 2. Name the card in the engrave checklist — **S**

> **IMPLEMENTED 2026-08-19** — `a6f7829` (feature), `1ac6fee` (the sidecar build
> script the version bump made necessary). All five acceptance bullets below are
> machine-checked: 30/30 pathological card plates name an origin, both
> transcripts regenerated at exit 0, the assertion and both new tests are in
> place, golden +4 lines, SPEC §6/§7 and CHANGELOG updated.
>
> **The open collision-scope question is DECIDED: per CARD, keyed on
> `chunk_set_id` — never per plate.** Every chunk of one card trivially shares
> that card's origin, so a per-plate scan would mark all 34 pathological plates
> ambiguous and suffix every one — noise hiding the single real case the suffix
> exists to flag. Pinned by mutation test: flipping the scan to per-plate fails
> `checklist_names_privacy_preserving_card_by_path`.
>
> Measured on the real journeys: **11 distinct brackets for 11 cards, 12 for 12
> cosigners, zero collision suffixes in either** — so the suffix path is
> exercised only by its dedicated test, not by the journeys.

### What is already true

`me bundle` decodes every mk1 set at `crates/me-cli/src/bundle.rs:279` and
**discards the `KeyCard`**. The checklist at
`crates/me-cli/src/manifest.rs:82-108` renders `mk1 chunk 1/3` with no identity,
and plates are emitted in `chunk_set_id` order, so position tells the operator
nothing.

### Decided: representation

Store **`String`**, converted at the decode site. `me-cli` has no `bitcoin`
dependency and adding one to name a field type is disproportionate for display
text; `serde` is already present, so the constraint is type-naming, not
serialisation.

### Decided: exact rendering

```
fingerprint present:  mk1 [73c5da0a/48'/0'/1'/2'] chunk 1/3
fingerprint absent:   mk1 [path 48'/0'/1'/2', no fingerprint] chunk 1/3
empty path:           mk1 [73c5da0a, no path] chunk 1/3
both absent:          mk1 [unidentified] chunk 1/3
collision:            …] chunk 1/3 set 0xe03a5
```

Hardened markers `'`, matching `mk decode`. The `set <chunk_set_id>` suffix is
appended **only** to plates whose `(fingerprint, path)` pair is not unique
within the bundle, placed **after** the chunk counter.

### Decided: JSON

New `PlateEntry` fields **`card_fingerprint`** and **`card_path`**, both
`Option<String>`, `#[serde(skip_serializing_if = "Option::is_none")]`.

**The golden and the spec BOTH change.** `crates/me-cli/tests/vectors/bundle-md1-mk1.json`
gains those keys on its mk1 plates, and `SPEC_me_bundle_phaseA.md` §6 gains
the two field definitions. Both assertions at `crates/me-cli/tests/cli.rs:308`
and `:745` must be regenerated deliberately, and the diff must show it.

### Decided: release mechanics

`me-cli` version bump + `crates/me-cli/CHANGELOG.md` entry, per the lockstep
convention in `SPEC_me_bundle_phaseA.md` §11. This changes user-visible output.

### It CANNOT print `@N`

`me` sees cards; a keyless template carries no key order. Survived three
rounds. The label states the card's **origin**, never a slot number.

### Acceptance

- All 30 card plates in the pathological checklist name an origin.
- **Both** committed transcripts regenerate — `transcript.txt` (24 lines) and
  `transcript_pathological.txt` (31).
- The `crates/me-cli/src/manifest.rs:228` assertion becomes
  `assert!(c.contains("mk1 [aabbccdd/48'/0'/0'/2'] chunk 1/2"))`.
- A test pins the no-fingerprint form using a `--privacy-preserving` card, and
  a test pins the collision form using two cards sharing `(fingerprint, path)`.
- Golden regenerated; SPEC §6 updated; CHANGELOG entry present.

---

## 3. Demonstrate recovery in the journey — **S**

### The change

1. **README**: the model (own seed + N−1 cards + recorded target), that
   `Unique` is proven-unique not first-match, that a target is **required**,
   that `--search-address` is refused at this scale, the `n ≤ 34` ceiling, the
   3600 s cap, `--accept-search-time`, and the **16-hex** floor.
2. **Journey step**: shuffled-card recovery via `--expect-wallet-id`, consuming
   the id §1 printed.

### The estimate varies — this gates the step

Measured across identical runs the gating **estimate** swung **189.6 s →
1284.9 s** against the 3600 s ceiling. A slower machine can therefore be
refused, turning a document build red for a reason unrelated to the document.

**Decision: pass `--accept-search-time 4000` in the journey step**, so the
gate is the *actual* work (≈16 s), not a variable estimate. The journey states
why the flag is present.

### Shuffle CARDS, not CHUNKS

A card's chunks must stay contiguous and in order; a chunk-level shuffle exits
1 (measured). The step shuffles the order of the ten cards and keeps each
card's chunks together.

### Exit codes, named

`0` working shape · `1` ceiling refusal · `2` missing seed / pool size ·
`4` NO MATCH.

### Dependency

§3 consumes §1's printed id, so **§1 lands first**.

### Acceptance

- The journey performs a shuffled-card recovery, exit `0`, using the id an
  earlier step printed.
- The README states all seven facts listed in step 1.

---

## 4. The account-index convention — documentation only

- **Deliverable:** one section in `design/journeys/README.md`, headed
  *"Choosing key paths so the cards identify themselves"*.
- **The convention sentence:** *"Derive cosigner `@N` at account index `N` of
  its master, so that a card's printed `origin_path` states its own template
  slot."*
- **Which index (D11):** the index **in the encoded template**, after
  `md-codec`'s `canonicalize_placeholder_indices` renumbers placeholders to
  BIP-388 first-occurrence order. **Measured: for this wallet the source and
  canonical orders coincide** (`@0…@10` already appear in first-occurrence
  order), so the distinction is invisible here — which is exactly why it must
  be written down rather than inferred from the fixture.
- **Acceptance:** that section exists and states the convention sentence, the
  canonicalisation answer, and all three limits below.

**Limits:**

1. Advisory; pre-existing keys cannot adopt it retroactively.
2. **The fixture does not follow it** — `@0-@3` are master A accounts `0'-3'`,
   `@4` is master B account `0'` (`[b8688df1/48'/0'/0'/2']`). Adopting it means
   re-deriving those keys again and moving every id: **a decision, not a step.**
3. Helps only where keyholders coordinate at creation.

**No lint** — `me` cannot see the template's key order, and a lint asserting
card↔position coupling contradicts §5.

---

## 5. Deliberately NOT proposed — an index field in `mk1`

A normative wire change that couples a card to a position in *one* wallet when
the same key may sit at different indices elsewhere. §4 gets the benefit
without the coupling. **Acknowledged tension:** §4's convention encodes the
same information in the path — the difference is that a path is the operator's
choice at creation and carries no protocol promise, whereas a wire field would.

---

## 6. Order

**§1 → §2 → §3 → §4.** §1 first (§3 consumes its output). §2 is independent.
§1 and §3 each rewrite the pathological transcript, so regenerate once after
both rather than twice.

---

## 7. Open — one item

**Should the §2 label also appear on the engraved plate**, not only the
checklist? Plate furniture, not wire format — but it consumes plate area and
interacts with the font's minimum-feature rules. **Unmeasured**, and outside
this plan.

---

## 8. Ledger — every prior finding, resolved or explicitly rejected

| finding | disposition |
| --- | --- |
| D1 type | §2: `String`, with rationale |
| D2 manifest fields | §2: yes — `card_fingerprint`, `card_path`; golden + SPEC change |
| D3 `None` fallback | §2: exact string given |
| D4 empty path | §2: exact string given |
| D5 collision | §2: `set <chunk_set_id>` suffix, placement specified |
| D6 multi-wallet / multi-stub | §2: irrelevant to this label; ignored deliberately |
| D7 restore invocation | §3 + §0 transcript give the full command |
| D8 which target | §3: `--expect-wallet-id` (address form refused) |
| D9 which number decides | §3: moot — `--accept-search-time` removes the estimate gate |
| D10 doc location | §4: named file and heading |
| D11 canonicalisation | §4: encoded-template index; measured identity here |
| D12 release mechanics | §2: version bump + CHANGELOG |
| F1 seed required | §0 |
| F2 `--search-address` refused | §0 |
| F3 wrong target id | §1 rewritten around `verify-bundle` |
| F4 dependency conditional | §2: `String` makes it unconditional |
| F5 manifest contract | §2: golden + SPEC named |
| F6 `@N` reason over-broad | §2: claim narrowed to "keyless template, no key order" |
| F7 fixture correction correct | §4 limit 2 |
| F8 recoverable-shape claim correct | §0 |
| MA-1 collision untested | §2 acceptance: a test |
| MA-2 "deliberately" not a criterion | §2 acceptance: exact assertion text |
| MA-3 golden/schema undecided | §2: both change |
| MA-4 other journey | §2 acceptance: both transcripts |
| MA-5 §4 no deliverable | §4: deliverable + sentence + acceptance |
| MA-6 which exit code | §3: all four named |
| MA-7 prefix floor | §1 + §3: 16 hex |
| H1 §3 prerequisite | §3: dependency on §1 |
| H2 two transcripts | §2 acceptance |
| H3 double regeneration | §6: regenerate once after §1 and §3 |
| H4 §2 decision not orderable | resolved — the decision is made in §2 |
| v2-C1 `verify-bundle` prints it | §1 rewritten |
| v2-C2 root cause disproven | §0: origin-flattening + omitted fingerprints |
| v2-I2 estimate variance | §3: `--accept-search-time` |
| v2-I4 empty-path card | §2 rendering table |
| PDF already captions `@N` | noted: the *document* shows it; the *checklist* the operator engraves from does not — §2 targets the latter |
