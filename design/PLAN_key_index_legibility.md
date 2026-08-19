# PLAN — helping the operator know which key is which

**Draft 2026-08-19. Not gated, not reviewed.** Written in response to: *"there
is no way to know what seed phrase / private key material goes with which
index."*

Scope: three changes (1, 2, 3 below). Each says what is already true, what
would change, where, how big, and what it does **not** solve.

---

## 0. The situation, measured

**An `mk1` card does not carry its template index.** Decoded fields are
`policy_id_stubs`, `origin_fingerprint` (**`Option`**), `origin_path`, `xpub`
(`mnemonic-key/crates/mk-codec/src/key_card.rs:34-57`). A card says *"I belong
to wallet `5b48af35`"*, never *"I am `@3`"*.

**But the assignment is already recoverable, not guessed.**
`mnemonic-toolkit/src/permutation_search.rs` searches all `n!` slot bijections
against the wallet id and returns `Unique` / `None` / `Ambiguous`, wired into
`cmd/restore.rs` and `cmd/verify_bundle.rs`. Its stated contract:

> **No silent wrong assembly.** A `Unique` outcome is returned ONLY after the
> engine has proven there is no SECOND match.

Ceiling `n ≤ 34` (`n!` overflows `u128` above). 11 keys = 39,916,800
permutations; 12 = 479,001,600.

**Both policy families are safe, for opposite reasons.** Unsorted `multi()`
makes each permutation a *different* script, so exactly one matches and the
search returns `Unique`. `sortedmulti()` makes every permutation the *same*
script, so the index is irrelevant and `Ambiguous` is harmless. The pathological
wallet uses unsorted `multi`.

**The concrete gap** is at the moment of engraving. The checklist an operator
follows says:

```
plate 20/34  mk1 chunk 1/3  → push via NFC & engrave
plate 21/34  mk1 chunk 2/3  → push via NFC & engrave
```

It never says *whose* key. And `me bundle` emits plates in `chunk_set_id`
order, not key order, so the operator cannot infer it from position either.

---

## 1. Name the card in the engrave checklist — **S, do first**

### What is already true

`me bundle` **already decodes every mk1 set** —
`crates/me-cli/src/bundle.rs:279`, `mk_codec::decode(&refs)` — purely to prove
set integrity, and then **discards the `KeyCard`**, keeping only `total` and an
integrity flag. The identifying data is computed today and thrown away.

### The change

Keep the decoded card. Thread its `origin_fingerprint` and `origin_path` into
`PlateEntry`, and use them in the label built at
`crates/me-cli/src/manifest.rs:82-108`:

```
  plate 20/34  mk1 [73c5da0a/48'/0'/1'/2'] chunk 1/3  → push via NFC & engrave
```

No new dependency (`mk-codec` is already a dep, `crates/me-cli/Cargo.toml:23`), no wire
change, no normative change — this is display text derived from data already in
hand.

### What it does NOT solve, and this is the important part

**`me` cannot print `@N`.** It sees *cards*, and for a **keyless template** the
md1 carries no keys, so there is no key order to match a card against. It can
only state what the card says about itself: its origin.

That is still the operator's practical question answered — *"which of my eleven
cards is this plate?"* — but it is **not** the same as knowing the template
index. The index comes from `restore`'s permutation search (§3), or from the
convention in §2.

An honest label must therefore avoid implying an index. Print the origin, not a
slot number.

### Edge cases that must be handled, not assumed

- **`origin_fingerprint` is `Option`.** Privacy-preserving cards
  (`mk encode --privacy-preserving`) may omit it. The label must degrade to
  something truthful (e.g. `mk1 [path only: 48'/0'/1'/2'] chunk 1/3`) rather
  than printing a placeholder that looks like a fingerprint.
- **Two cards can share an origin.** Nothing forbids two keyholders deriving at
  the same path from different seeds — different fingerprints then disambiguate,
  but if fingerprints are omitted *and* paths collide, the label is ambiguous.
  It should say so rather than silently repeat itself.
- **`ms1` plates** have no card at all; leave their label alone.

### Acceptance

- Checklist for the pathological journey names an origin on all 30 card plates.
- A privacy-preserving card renders without a fabricated fingerprint.
- Existing `manifest.rs` tests (`:228` asserts `"mk1 chunk 1/2"`) updated
  deliberately, not incidentally — that string is the current contract.

---

## 2. Let the origin path carry the index, by convention — **XS docs, S lint**

### The idea

Every card already carries `origin_path`, and `mk decode` already prints it. If
keyholders agree at generation time that **account index = template index**,
then the index is *already on every card* with no format change.

This is strictly stronger than "order the keys lexicographically", because the
reader does not have to sort N xpubs by eye — the answer is printed on the card
in front of them.

### What would change

Documentation, primarily: a stated convention in the journeys README and
wherever wallet creation is described. Optionally a **lint** in `me bundle`
that notes when a bundle's cards do *not* follow it, as an advisory rather than
a refusal.

### What it does NOT solve

- **It is advisory only.** Nothing enforces it, and a wallet assembled from
  pre-existing keys at fixed paths cannot adopt it retroactively.
- **The current fixture does not follow it.** The pathological keys restart
  accounts per master (`@0-@3` = master A accounts 0-3, but `@4` = master B
  account 0). Adopting the convention here would mean re-deriving those keys a
  *second* time — which, per §2 of the item-5 review, changes every wallet id
  again. **Not obviously worth it**; flagged as a decision, not a step.
- It only helps when keyholders coordinate at creation — exactly the case the
  operator already identified as their responsibility.

---

## 3. Surface the permutation search — **S**

### The problem

The search exists, has a strong funds-safety contract, and **nothing tells the
user it exists.** An operator holding a shuffled pile of cards and a template
would reasonably conclude the wallet is lost. That is a documentation failure
with a recovery-shaped consequence.

### The change

1. A section in `design/journeys/README.md` stating plainly: given the template
   and all N cards **in any order**, `restore` recovers the assignment, and
   `Unique` means proven-unique rather than first-match.
2. **Demonstrate it in the pathological journey.** Add a step that shuffles the
   card order and recovers the wallet, printing the outcome. This is the
   strongest form: the claim becomes a thing the transcript *does*, not a thing
   the README *says* — and per this project's own rule, a gate that has never
   run is a hypothesis.
3. State the `n ≤ 34` ceiling and the cost curve, so nobody discovers it at
   recovery time.

### What it does NOT solve

- It does not help someone who has **lost a card**; the search needs all N.
- `Ambiguous` is a real outcome for `sortedmulti` policies and the docs must
  say it is *harmless there* — otherwise it reads as a failure.

### Acceptance

- The pathological journey demonstrates recovery from shuffled input, with its
  real exit code.
- README states the ceiling and what each of the three outcomes means.

---

## 4. Deliberately NOT proposed — an index field in `mk1`

It would be a **normative wire change** (Rust-primary, needs vectors), and it
couples a card to a position in *one* wallet when the same key may sit at
different indices in different wallets. §2 gets the same benefit without that
coupling.

---

## 5. Order, and why

1 first: it is nearly free, needs no coordination, and fixes the moment where a
mistake is expensive (cutting the wrong plate). 3 next: pure documentation plus
a demonstration, and it is what turns "user responsibility" into "the tool does
it". 2 last, and only the documentation half — the fixture question it raises is
a separate decision.

## 6. Open questions for review

- Is printing an origin without an index **more** confusing than printing
  nothing, for an operator who expects `@N`?
- Should the §1 label appear on the **engraved plate itself**, not just the
  checklist? That is plate furniture, not wire format — but it consumes area on
  a physical plate and may collide with the font's minimum-feature rules.
- Does `me bundle` have the md1 available at the point the checklist is built?
  If a **keyed** full-policy md1 is present, `@N` *is* derivable and the label
  could be exact — but the pathological journey engraves a keyless template, so
  the common case stays origin-only. **Unverified.**
