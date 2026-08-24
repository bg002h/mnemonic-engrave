# R8 — Ruling: defending `mt-codec` against a wrong NUMS constant

**Agent:** fable, standing in for the operator on this one decision, 2026-08-23.
**Scope:** the four questions in the dispatch brief. Constraints honored: fork
stays, constant stays, no QR, no shared crate.

Facts re-verified at source before ruling (not taken from the brief):

- `descriptor-mnemonic/crates/md-codec/src/bch.rs:86-105` —
  `bch_create_checksum_regular(hrp, data)` folds `hrp_expand(hrp)` into the
  polymod, and `bch_verify_regular(hrp, ...)` does the same on verify. The HRP
  is cryptographically load-bearing in both directions.
- `mnemonic-key/crates/mk-codec/src/consts.rs` — the sibling precedent is
  **two** tests, not one: `nums_constants_reproduce_from_domain` (drift) *and*
  `nums_string_differs_from_md1` (domain-string inequality). The brief's
  "Defence 1" understates the precedent; mk already half-closes the
  copied-pair hole.
- `SPEC_mt_v0_1.md` §10.13(a) contains the sentence *"without a distinct one
  an `mt1` chunk would verify as a valid `md1` chunk."*

---

## 1. The stated hazard is NOT real as worded — and the spec must stop saying it

Every verifier in the family parses the HRP off the string and feeds
`hrp_expand(hrp)` into the polymod. An `mt1…` string handed to an `md1`
verifier fails on HRP mismatch before the constant is ever consulted, and even
a deliberately HRP-relabelled string fails because `hrp_expand("mt") !=
hrp_expand("md")` shifts the residue regardless of which constant was
compiled in. **Cross-format acceptance is unreachable while the HRPs differ,
in both the copied-constant world and the correct world.** The §10.13(a)
sentence quoted above is a false fact inside a normative rationale, and false
facts propagate *into* review (this constellation has measured that). It gets
a one-line fold — see the order of work below.

**The real hazard is intra-format, and it is worse than the imaginary one.**
A wrong constant — copied from `md`, *or* unique-but-wrong (typo'd domain
string with a faithfully derived constant, wrong shift, endianness slip in
staging the digest) — produces an implementation that is perfectly
self-consistent: encode, verify, repair all green, every self-test passes,
plates get cut. The defect surfaces only when a **second** implementation
(the reference, a Go device port, a from-spec reimplementation years later)
reads the plate: every chunk fails checksum with a *"damaged beyond
correction"* diagnostic that points the recoverer at their steel instead of
at the software that cut it. That is the same failure shape as the R5 HRP
finding (§10.13(b)) — mutual unverifiability with a misattributed diagnostic
— landing on a **permanent bearer artifact at recovery time**. Who discovers
it: the plate holder, last. What it costs: the transaction, or a recut window
that no longer exists.

The copied constant is not a special catastrophe; it is merely the most
*likely* instance of the class. The defence must therefore cover the whole
class, not just the copy.

## 2. The defence, ordered — one load-bearing test, two tripwires, one deletion

### D1 (load-bearing, blocking): a spec-authored, independently derived, byte-exact pinned vector

This is the only test that fails in **every** wrong-constant world — copied
constant, copied constant+domain pair, typo'd domain with honest derivation,
wrong shift, wrong endianness — and it additionally pins the HRP string
("mt" not "mt1"), the 50-bit header layout, chunking, and padding, i.e. the
entire surface whose defects share the same discovered-at-recovery failure
mode.

- **Content:** one fixed, minimal signed transaction (1-in-1-out, dummy key,
  committed as hex in the spec) → the complete ordered set of `mt1` chunk
  strings, **byte-exact**, plus the txid so the vector also pins
  `chunk_set_id`. Frozen in `SPEC_mt_v0_1.md` in a test-vectors section.
- **Provenance rule (this is the whole point):** the vector is computed by a
  standalone reference script committed to the repo (pattern:
  `scripts/mt-reference-vector.*`), which may reuse the public family polymod
  routine but MUST take the constant, HRP, and header layout **from the spec
  text, never from any codec crate**. A vector captured from `mt-codec`'s own
  first output is worthless — a wrong constant generates a matching wrong
  vector, and this constellation has already measured that a corpus can be
  uniformly wrong (9/9 agreeing vectors pinning an impossible wallet).
  Committing the generator is mandatory: reproduction paths decay silently.
- **Tests:** `crates/mt-codec/tests/vectors.rs::spec_vector_byte_exact`
  (encode the fixed tx, assert exact string equality per chunk; decode the
  spec strings back, assert exact tx bytes) and
  `::checksum_micro_vector` (HRP `"mt"`, a fixed 10-symbol data sequence →
  the exact 13-symbol checksum, value frozen in the spec). The micro-vector
  exists to localise failure: when the big vector breaks, it answers "is it
  the checksum layer or the layout" in one test name.

### D2 (tripwires, near-free): keep the drift test, add full sibling inequality

- Keep `nums_constants_reproduce_from_domain` exactly on the mk pattern
  (`mk-codec/src/consts.rs:78-95`). Its known hole — copied pair satisfies
  it — is closed by the next test, not by discarding it; it still catches
  every lone edit of either value.
- Add `nums_differs_from_siblings` in `crates/mt-codec/src/consts.rs` tests:

  ```rust
  assert_ne!(NUMS_DOMAIN, b"shibbolethnums");        // md1
  assert_ne!(NUMS_DOMAIN, b"shibbolethnumskey");     // mk1
  assert_ne!(MT_REGULAR_CONST, 0x0815c07747a3392e7); // MD_REGULAR_CONST
  assert_ne!(MT_REGULAR_CONST, 0x1062435f91072fa5c); // MK_REGULAR_CONST
  ```

  Sibling values are **hardcoded literals with a provenance comment** (crate
  + path + the frozen spec that fixed them) — no sibling-crate imports; the
  fork discipline stands. These are frozen normative constants; hardcoding is
  correct, not a smell. Drift + string-inequality together already imply
  constant-inequality, but the direct asserts cost two lines and do not
  depend on composing two tests to get the property.
- What D2 still misses — the typo'd-domain-honestly-derived case — is exactly
  what D1 exists for. State that division of labour in the test comments so
  nobody later "simplifies" one away believing the other covers it.

### D3 (deletion): do NOT ship the cross-format negative as a NUMS defence

`assert!(md_verify(mt_chunk).is_err())` returns the identical result whether
the constant was copied or not — the HRP alone produces the failure. A test
that cannot fail in the world it claims to guard is a false-reassurance test,
and this project's own discipline (mutation-test your tests; hunt false
PASSes) says it is worse than absent: it sits in the suite vouching for a
property it does not measure. If anyone wants HRP-separation pinned as a
*property of the family*, that is D1's micro-vector plus the sibling specs —
not a negative that green-lights both worlds.

## 3. The fork decision does not change

The defence above is fully achievable under forked crates — D1 is
deliberately *outside* any crate (spec + independent script), and D2 needs no
sibling dependency. A shared BCH crate would not even help with this hazard:
the constant is per-format **by design** and would remain a per-format
literal parameter under any structure, copyable in exactly the same way. The
fork stays; nothing here is evidence against it.

## 4. The ordering answer — three gates, in time order

1. **Before `mt-codec`'s first commit:** the D1 vector and micro-vector land
   in `SPEC_mt_v0_1.md`, computed by the committed reference script, and the
   derivation is recomputed independently once (same ritual already used for
   the constant itself in §10.13(a)). The first implementer's TDD then
   *starts* from the vector — a copied `md-codec` fails on test one, minutes
   in, instead of at someone's recovery.
2. **Before implementation review closes:** `spec_vector_byte_exact`,
   `checksum_micro_vector`, drift, and `nums_differs_from_siblings` are all
   green on the reviewed tree. A plan may not close while one of its gates
   has never run — these are gates from day one, not follow-ups.
3. **Before the first real plate is cut:** the exact string leaving for the
   engraver passes `mt verify` from the same build that passed gate 2, and
   that run is recorded. When a Go device port later decodes `mt1`, the D1
   vector is re-used verbatim as the cross-language corpus seed — the F-212
   lesson (two languages, different ids, 887/887 tests green either way)
   applied in advance rather than in arrears.

## Order of work (actionable now)

1. Fold §10.13(a): replace the false cross-verification sentence with the
   real rationale — the HRP already separates formats on the wire; the
   distinct constant is (a) the family's derivation rule, and (b) defence in
   depth for any same-HRP or HRP-stripped context — and name the real
   wrong-constant cost: self-consistent nonconformant plates, unreadable to
   every other implementation, misdiagnosed as physical damage at recovery.
   One paragraph, no new scope.
2. Write the reference script; generate and freeze the D1 vector + micro-
   vector into the spec; commit script and vector together.
3. Implement D2's four asserts alongside the drift test when `consts.rs` is
   born; carry the D1 tests as the first entries in `tests/vectors.rs`.
4. Strike the cross-format negative from the plan's test list, with a
   pointer to this ruling so it is not re-proposed by a future round.

**Keep / drop of the currently-proposed pair:** keep Defence 1 (drift) —
cheap, catches lone edits, and its hole is closed by D2. Drop Defence 2 (the
cross-format negative) — it measures nothing about the hazard and its
presence actively misleads.
