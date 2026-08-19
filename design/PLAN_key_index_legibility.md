# PLAN — helping the operator know which key is which

**Rewritten 2026-08-19 (v2).** v1 was found **not executable**: 12 forced
decisions, 2 Critical, 4 Important
(`design/agent-reports/plan-key-index-legibility-architect-r0.md`), after an
earlier critique (`design/agent-reports/plan-key-index-legibility-critique.md`).
v1 is preserved in git at `2d9fe3e`.

**Every claim below was produced by running a command, not by reading help
text.** That was v1's failure mode and it cost two review rounds: v1 chose a
flag from its doc string that is refused in 84 ms, and "corrected" a true
finding into a false one from a help sentence.

Originating question: *"there is no way to know what seed phrase / private key
material goes with which index."*

---

## 0. Measured facts

**A card does not carry its index.** `KeyCard` is `policy_id_stubs`,
`origin_fingerprint` (**`Option`**), `origin_path`, `xpub`
(`mnemonic-key/crates/mk-codec/src/key_card.rs:34-57`).

**Recovery works, and is fast.** Own seed plus the other ten cards in
deliberately scrambled order:

```
$ mnemonic restore --md1 ×3 --from phrase=<master-A> --account 0 \
    --cosigner ×28 --expect-wallet-id ced2270948ecb5af
searching 39916800 candidate assignment(s) (est. ≤ 189.6048s)…
✓ wallet-id (completed): ced2270948ecb5af0779249ac7181f4a
  your seed completes cosigner slot @0
  first recv: bc1qkuknuy6dsm0fq44cyyhzqy9wl3ex2n6ed39zxhx867l9wlh4yhlsejms64
elapsed=16s
```

**16 seconds** for the whole `11!` space, correct slot, and an address matching
what the journey prints.

**A seed IS required.** v1 claimed otherwise from the `--from` help text
(*"OPTIONAL in multisig mode"*). For a **keyless template** it is required
(`mnemonic-toolkit/crates/mnemonic-toolkit/src/cmd/restore.rs:1396`). The real
model is **own seed + the other N−1 cards + a recorded target**, not "template
+ all N cards".

**`--search-address` is REFUSED at n=11**, measured in 84 ms:

```
error: estimated exhaustive search time 890788.897152s exceeds the 3600s
ceiling; re-run with --accept-search-time ≥890788.897152s to acknowledge
```

v1 rebuilt its demonstration around `--search-address` on the strength of its
"recommended … collision-free" doc string. `--expect-wallet-id` is the flag
that works. The `--accept-search-time` hatch exists; at ~10 days it is not a
recovery path.

**`me-cli` has `serde` + `serde_json` and NO `bitcoin` dependency**
(`crates/me-cli/Cargo.toml:22-27`).

**`manifest.json` has four consumers, one a test** —
`design/journeys/transcript.sh`, `design/journeys/transcript_pathological.sh`,
`design/journeys/build_pdf_pathological.py`, `crates/me-cli/tests/cli.rs`. It
is a de facto contract; v1's "not a normative change" waved at that.

---

## 1. THE RECOVERY TARGET — the blocking problem, which v1 missed entirely

### The defect

`--expect-wallet-id` needs `ced2270948ecb5af…`. **No command in this repo
prints that value.** Measured — one wallet, four ids:

| id | value | printed by |
| --- | --- | --- |
| `WalletDescriptorTemplateId` | `5b48af35d4321a3ac18b43045e2523cc` | `md inspect`; journey prints it |
| `WalletPolicyId`, **keyless** template | `f89e23f13c697ae62ef10328d71d7e24` | `md inspect`; journey prints it |
| `WalletPolicyId`, **keyed** policy | `232214e4d60c0fa83a6715ba2f7e8ec7` | `md inspect` on a keyed encode |
| **what `restore` matches** | **`ced2270948ecb5af0779249ac7181f4a`** | **nothing** |

An operator who records the id their own tooling prints, and reaches for it at
recovery, is refused. Supplying `232214e4…` was executed and rejected.

### Root cause — a known finding nobody had connected to this

`restore` re-serialises the completed descriptor's xpubs with BIP-32 metadata
**zeroed**. Measured, same key:

```
committed xpub  depth=4  parent_fp=1cf29716  child=80000002
restored xpub   depth=0  parent_fp=00000000  child=00000000
```

Different serialisation → different descriptor string → different
`WalletPolicyId`. This is **F-130**, already in
`design/journeys/README.md:63`: *"Keys byte-identical, addresses unaffected —
but restored xpubs lose depth/parent/child, so the descriptor string and its
checksum differ."* Addresses are unaffected — the `first recv` above matches.
Only the **id** moves.

### The change

**Emit the recovery target at BACKUP time.** An operator cannot be asked to
obtain it by performing the recovery they are preparing for.

Two candidate homes. **This plan deliberately does not choose — see §7.1.**

- **(a) `md inspect --restored-form`** — prints the restored-serialisation id
  beside the existing two. Smallest surface; keeps identity in the tool that
  owns it.
- **(b) `me bundle` checklist header** — "record this to recover: `ced22709…`".
  Where the operator is already writing things down, but `me` would have to
  compute an id it currently has no reason to know.

**Acceptance (either home):** a command in this repo prints
`ced2270948ecb5af0779249ac7181f4a` for the pathological wallet, and the journey
prints it under a label saying what it is for.

### What this does NOT solve

An operator who recorded **no** target. That case needs `--search-address`,
refused at this scale — so for an 11-key wallet **there is currently no
no-target recovery path**. State it in the docs rather than let it be
discovered.

---

## 2. Name the card in the engrave checklist

### What is already true

`me bundle` decodes every mk1 set at `crates/me-cli/src/bundle.rs:279` and
**discards the `KeyCard`**, keeping only `total` and an integrity flag. The
identifying data is computed today and thrown away.

The checklist at `crates/me-cli/src/manifest.rs:82-108` renders `mk1 chunk 1/3`
with no key identity, and plates are emitted in `chunk_set_id` order, so
position carries no information either.

### The change, with the representation DECIDED

Store **`String`**, converted at the decode site — not the `bitcoin` types.

This is the decision v1 left open and the architect flagged as blocking.
Rationale: `me-cli` has **no `bitcoin` dependency**, and adding one to name a
field type is disproportionate for display text. `serde` is already present, so
serialisation is not the constraint; the type-naming is.

Rendering, specified so two implementers cannot differ:

```
fingerprint present:  mk1 [73c5da0a/48'/0'/1'/2'] chunk 1/3
fingerprint absent:   mk1 [path 48'/0'/1'/2', no fingerprint] chunk 1/3
```

- Hardened markers rendered `'`, matching `mk decode`'s existing output, not `h`.
- New `PlateEntry` fields are `Option<String>` with
  `#[serde(skip_serializing_if = "Option::is_none")]`, so plates with no card
  gain no keys and existing manifest consumers are unaffected.

### It CANNOT print `@N`, and must not imply one

`me` sees cards; a keyless template carries no key order to match against.
This survived both review rounds. The label states the card's **origin**, never
a slot number. The index comes from §1+§3, or §4's convention.

### Edge cases, each with a defined behaviour

| case | behaviour |
| --- | --- |
| `origin_fingerprint` is `None` | second form above; never fabricate a fingerprint |
| two cards share fingerprint+path | render identically, **plus `set <chunk_set_id>`** so they stay distinguishable |
| `ms1` plate | unchanged — it has no card |
| multiple `policy_id_stubs` | irrelevant here; ignore |

### Acceptance

- All 30 card plates in the pathological checklist name an origin.
- `crates/me-cli/tests/cli.rs` and the `crates/me-cli/src/manifest.rs:228` assertion
  (`"mk1 chunk 1/2"`) updated **deliberately** — that string is today's
  contract and the diff must show it changing on purpose.
- The no-fingerprint form is pinned by a test using a
  `--privacy-preserving` card.

---

## 3. Surface and demonstrate recovery

### The change

1. **`design/journeys/README.md`**: state the model as measured — **own seed +
   the other N−1 cards + a recorded target id** — that `Unique` is
   proven-unique rather than first-match, that a target is **required**, and
   that `--search-address` is refused at this scale.
2. **Demonstrate it in the pathological journey** via `--expect-wallet-id`,
   cards deliberately shuffled. Measured **16 s**, exit 0 — affordable in a
   journey that must complete every run.
3. State the `n ≤ 34` ceiling, the 3600 s cap, and `--accept-search-time`.

### Dependency

**Step 2 cannot ship before §1**, or the journey must hardcode an id nothing
produces — the exact no-producer defect this session has fixed four times.

### Acceptance

- The journey performs a shuffled recovery and prints its real exit code.
- The target id it uses is one an **earlier step printed**, not a literal.

---

## 4. The account-index convention — documentation only

Every card carries `origin_path` and `mk decode` prints it. If keyholders agree
at generation that **account index = template index**, the index is on every
card with no format change.

**Deliverable, location, acceptance — the three things v1 lacked:**

- **Deliverable:** one section in `design/journeys/README.md`, *"Choosing key
  paths so the cards identify themselves"*.
- **Acceptance:** that section exists and states all three limits below.

**Limits, all three required:**

1. Advisory only; pre-existing keys cannot adopt it retroactively.
2. **The current fixture does not follow it.** Measured: `@0-@3` are master A
   accounts `0'-3'`, but `@4` is master B account `0'`
   (`[b8688df1/48'/0'/0'/2']`). Adopting it means re-deriving those keys again
   and moving every id — **a decision, not a step**.
3. It helps only where keyholders coordinate at creation.

**No lint.** `me` cannot see the template's key order, so it cannot check the
convention, and a lint asserting card↔position coupling contradicts §5.

---

## 5. Deliberately NOT proposed — an index field in `mk1`

A normative wire change, and it couples a card to a position in *one* wallet
when the same key may sit at different indices elsewhere. §4 gets the benefit
without the coupling.

---

## 6. Order

**§1 → §2 → §3 → §4.** §1 leads: §3 is unbuildable without it, and it is the
only item addressing a case where an operator loses access. §2 is independent
and may run in parallel. §4 is documentation, last.

---

## 7. Open — escalated, not assumed

1. **§1's home: (a) `md inspect --restored-form`, or (b) `me bundle` header?**
   The one decision this plan deliberately leaves open.
2. **Should `restore` instead accept the keyed-policy id and normalise
   internally?** That would make F-130 invisible to operators rather than
   documented at them, and might be a better fix than §1 entirely.
   **Unresearched.**
3. Should §2's label also appear on the **engraved plate**, not only the
   checklist? Plate furniture, not wire format — but it consumes plate area and
   interacts with the font's minimum-feature rules. **Unmeasured.**
