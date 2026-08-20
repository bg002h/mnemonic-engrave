# RECON: May `sortedmulti(...)` appear as a nested miniscript fragment?

**Question:** Do the BIP standards permit `sortedmulti(...)` to appear as a nested miniscript
fragment (a leaf inside `and_v`, `or_d`, `thresh`, …), or is it normatively restricted to being
the sole, direct child of `sh()` / `wsh()`? Same question for `sortedmulti_a(...)` under `tr()`.

**Date:** 2026-08-20 · **Sources:** BIP-380/381/382/383/386/387/388 (raw mediawiki/md from
`github.com/bitcoin/bips` @ master), BIP-379 (Miniscript), `bitcoin.sipa.be/miniscript`,
rust-miniscript PR #915 metadata + rev `ff4732e` source.

---

## 1. VERDICT

**`RESTRICTED`** — the BIPs place `sortedmulti` / `sortedmulti_a` outside the Miniscript grammar
entirely, so neither can ever be a leaf inside a miniscript combinator. `md1`'s existing rule is
**standard-conformant. HOLD it.**

The single most decisive fact is an **absence**, and it is a total one:

> **`sortedmulti` occurs ZERO times in BIP-379 (Miniscript), and zero times on
> `bitcoin.sipa.be/miniscript`.** The fragment table of BIP-379 lists `multi(k,key_1,...,key_n)`
> and `multi_a(k,key_1,...,key_n)` — and no sorted variant of either.

And the single most decisive *quote*, from the BIP that defines it:

> "`multi()` and `sortedmulti()` expressions can be used as a top level expression, or inside of
> either a `sh()` or `wsh()` descriptor."
> — **BIP-383, §Specification, line 37**

Because `sh()` and `wsh()` each take **exactly one** script expression, and because the only
descriptor-level nesting machinery that exists is Miniscript — which has no `sortedmulti`
production — "inside of a `sh()`/`wsh()` descriptor" has exactly one realisation: **the direct
argument**. There is no third construction in which a `sortedmulti` could be "inside" a `wsh()`
without being its argument.

---

## 2. Evidence

### 2.1 Where `sortedmulti` may appear (BIP-383)

> "`multi()` and `sortedmulti()` expressions can be used as a top level expression, or inside of
> either a `sh()` or `wsh()` descriptor."
> — BIP-383 §Specification, line 37

This is a **closed enumeration of positions**, phrased identically to the positional clauses in
the sibling BIPs (cf. BIP-381 line 48 for `pkh()`, BIP-382 line 46 for `wsh()`). It is not a
statement about a fragment's type; it is a statement about descriptor position.

### 2.2 `sortedmulti`'s arguments are keys, never scripts (BIP-383)

> "They are written as `multi(k,KEY_1,KEY_2,...,KEY_n)`. `k` is the threshold — the number of keys
> that must sign the input for the script to be valid. `KEY_1,KEY_2,...,KEY_n` are the key
> expressions for the multisig."
> — BIP-383 §Specification, lines 33–35

Every argument is a `KEY`. `sortedmulti` therefore cannot *contain* a sub-script; the question is
only whether something can contain *it*.

### 2.3 `sh()` / `wsh()` take exactly ONE script expression (BIP-381 / BIP-382)

> "The `sh(SCRIPT)` expression can only be used as a top level expression. It takes a single script
> expression as an argument and produces a P2SH output script."
> — BIP-381 §`sh()`, lines 60–61

> "The `wsh(SCRIPT)` expression can be used as a top level expression, or inside of a `sh()`
> descriptor. It takes a single script expression as an argument and produces a P2WSH output
> script."
> — BIP-382 §`wsh()`, lines 46–47

This is the load-bearing pairing. "Inside a `wsh()`" + "`wsh()` takes a single script expression"
⇒ **sole, direct child**, unless some *other* combinator can sit in between. §2.5 shows none can.

### 2.4 Script expressions *can* nest in general — so the restriction is real, not vacuous (BIP-380)

> "The arguments to a script expression are defined by that expression itself. They could be a
> script expression, a key expression, or some other expression entirely."
> — BIP-380 §Script Expressions, lines 60–61

BIP-380 permits nesting **in general**, and delegates the specifics to each expression's own BIP
("defined by that expression itself"). So BIP-383's positional clause is doing real normative
work: it is the per-expression definition BIP-380 defers to. This forecloses the reading "BIP-380
allows nesting generally, so `sortedmulti` may nest" — BIP-380 explicitly hands that decision to
BIP-383, and BIP-383 enumerates three positions, none of which is "as an argument to a
combinator".

### 2.5 The only combinators that exist are Miniscript's — and Miniscript has no `sortedmulti`

The complete BIP-379 fragment table row for the multisig fragments (lines 98–99):

> ```
> | check(key_1) + ... + check(key_n) = k *(P2WSH only)*     | `multi(k,key_1,...,key_n)`    | `<k> <key_1> ... <key_n> <n> CHECKMULTISIG`
> | check(key_1) + ... + check(key_n) = k *(Tapscript only)* | `multi_a(k,key_1,...,key_n)`  | `<key_1> CHECKSIG <key_2> CHECKSIGADD ... <key_n> CHECKSIGADD <k> NUMEQUAL`
> ```
> — BIP-379, §Specification fragment table

The combinators named in the same table are `andor`, `and_v`, `and_b`, `and_n`, `or_b`, `or_c`,
`or_d`, `or_i`, `thresh` (lines 89–97) — the exact fragments named in the question. **No sorted
variant of `multi` appears in that table, in the type table (lines 148–149), in the malleability
table (lines 200–201), or in the satisfaction table (lines 241–242).** Measured: `grep -c
sortedmulti bip-0379.md` → **0**.

Corroborated independently at the Miniscript reference site `bitcoin.sipa.be/miniscript`: the
translation table lists `multi(k,key1,...,keyn)` and `multi_a(k,key1,...,keyn)`; **"sortedmulti"
and "sortedmulti_a" do not appear anywhere on the page.**

BIP-379 also confirms the fragments it *does* share with descriptors, and conspicuously omits the
sorted ones:

> "The `pk()`, `pkh()`, `multi()`, and `multi_a()` fragments overlap with existing descriptors.
> These parse to the same semantic meanings as those descriptors and produce the same scripts."
> — BIP-379 §Backwards Compatibility, lines 406–408

Four overlapping names are enumerated. `sortedmulti` and `sortedmulti_a` are **not** among them —
which is the spec saying, in its own voice, that the sorted descriptors have no Miniscript
counterpart.

### 2.6 Why Miniscript cannot admit `sortedmulti` (structural, not accidental)

Miniscript is defined as a bidirectional mapping between an AST and Script. Sorting is applied at
*encoding* time and destroys the argument order, so the Script cannot be decoded back to a unique
`sortedmulti` AST. rust-miniscript's own PR #915 body states this plainly (see §6):

> "Like sortedmulti_a, sortedmulti is sorted upon encoding and **cannot be decoded into from
> Script**."

That is precisely the property that disqualifies it as a Miniscript fragment, and explains the
omission in §2.5 as principled rather than an oversight.

---

## 3. `sortedmulti` vs `sortedmulti_a` — treated separately

The brief warned against assuming symmetry. They are governed by **different BIPs with different
wording**, and the surrounding tree machinery differs. The verdict coincides, but the reasoning
does not, and there is one genuine asymmetry (3.3) that could be mistaken for permitted nesting.

### 3.1 `sortedmulti` — segwit v0 / legacy — **BIP-383**

> "`multi()` and `sortedmulti()` expressions can be used as a top level expression, or inside of
> either a `sh()` or `wsh()` descriptor." — BIP-383 line 37

Three permitted positions: top level; argument of `sh()`; argument of `wsh()`. Not a Miniscript
fragment (§2.5). **Cannot be a leaf in `and_v`/`or_d`/`thresh`.**

**Nuance worth recording (not a nesting question):** BIP-383 permits **bare top-level**
`sortedmulti(k,...)` with **at most 3 keys** —

> "When used at the top level, there can only be at most 3 keys." — BIP-383 line 40

`md1`'s rule as quoted ("must be the sole child of wsh/sh") is *narrower* than BIP-383 on this
axis, since BIP-383 also allows the unwrapped top-level form. That is a **separate, pre-existing**
scope choice, unaffected by the rust-miniscript rev bump, and it errs conservative (refusing
something the BIP allows) rather than permissive. Flagging it only so it is not conflated with the
nesting question. It is also moot under BIP-388, which does **not** list a bare top-level
`multi`/`sortedmulti` (§4).

### 3.2 `sortedmulti_a` — tapscript — **BIP-387** (note: *not* BIP-383)

`sortedmulti_a` is defined in **BIP-387**, not BIP-383 — BIP-383 does not mention it at all. Its
positional clause is worded differently and is *stricter*:

> "`multi_a()` and `sortedmulti_a()` expressions can only be used inside of a `tr()` descriptor."
> — BIP-387 §Specification, line 38

Note "**can only be used inside of**" — no top-level option, unlike BIP-383's "top level
expression, or inside of…". BIP-386 confirms the pairing from the `tr()` side, and — decisively —
enumerates `sortedmulti_a` as a *sibling* category to Miniscript rather than a member of it:

> "Of the script expressions that existed when this BIP was written, only `pk()` can be used in a
> tree expression. Script expressions specified since then that can be used in a tree expression
> are the Miniscript expressions of [379], which include a `pkh()` fragment, and the `multi_a()`
> and `sortedmulti_a()` expressions of [387]."
> — BIP-386 §Rationale, lines 116–118

This sentence is worth dwelling on: it lists "the Miniscript expressions of BIP 379" **and**
"the `multi_a()` and `sortedmulti_a()` expressions of BIP 387" as two **distinct** things that may
occupy a tree position. If `sortedmulti_a` were a Miniscript fragment, the second clause would be
redundant. The BIP is treating it as a descriptor-level script expression that sits *alongside*
miniscript in a leaf slot — never *inside* one. **Cannot be a leaf in a miniscript combinator.**

### 3.3 The real asymmetry — taptree depth is NOT miniscript nesting

`sortedmulti_a` *can* legitimately appear at arbitrary **depth** inside a `tr()`, because a Tree
Expression is recursive:

> "A Tree Expression is:
> * Any Script Expression that is allowed at the level this Tree Expression is in.
> * A pair of Tree Expressions consisting of: an open brace `{`, a Tree Expression, a comma `,`, a
>   Tree Expression, and a closing brace `}`"
> — BIP-386 §Tree Expression, lines 38–44

So `tr(K,{{sortedmulti_a(2,@0,@1),X},Y})` is well-formed and standard. **This is not the same
thing as being a miniscript leaf.** The `{,}` taptree is a *branching structure over independent
tapleaves*, each of which becomes its own Script; a combinator like `and_v`/`or_d`/`thresh`
composes fragments *within a single Script*. `sortedmulti_a` may be a **whole tapleaf at any
depth**; it may never be a **sub-expression of a tapleaf's script**.

BIP-388's own test vector demonstrates exactly this distinction (line 291):

> `tr(@0/**,{sortedmulti_a(1,@0/<2;3>/*,@1/**),or_b(pk(@2/**),s:pk(@3/**))})`
> — BIP-388 §Test Vectors, "Taproot wallet policy with sortedmulti_a and a miniscript leaf"

The `sortedmulti_a` and the miniscript `or_b(...)` are **siblings in the taptree** — two separate
tapleaves. The standard's one worked example of the two coexisting places them side by side and
**not one inside the other**. If any hypothetical implementation reports "we support nested
sortedmulti_a", check whether it means this (standard) or a miniscript leaf (not standard).

---

## 4. BIP-388 wallet-policy angle (this is the binding one for `md1`)

`md1` templates are BIP-388-shaped (`@0`, `@1`), so BIP-388's grammar is the **directly applicable
normative surface**, and it is the most explicit of all the sources. Its `SCRIPT` list is a
**closed enumeration** with per-item position constraints:

> "A ''wallet descriptor template'' is a `SCRIPT` expression.
>
> `SCRIPT` expressions:
> * `sh(SCRIPT)` (top level only): P2SH embed the argument.
> * `wsh(SCRIPT)` (top level or inside `sh` only): P2WSH embed the argument.
> * `pkh(KEY)` (not inside `tr`): P2PKH output for the given public key.
> * `wpkh(KEY)` (top level or inside `sh` only): P2WPKH output for the given compressed pubkey.
> * `multi(k,KEY_1,KEY_2,...,KEY_n)` (inside `sh` or `wsh` only): ''k''-of-''n'' multisig script.
> * `sortedmulti(k,KEY_1,KEY_2,...,KEY_n)` (inside `sh` or `wsh` only): ''k''-of-''n'' multisig
>   script with keys sorted lexicographically in the resulting script.
> * `tr(KEY)` or `tr(KEY,TREE)` (top level only): P2TR output with the specified key as internal
>   key, and optionally a tree of script paths.
> * any valid miniscript template (inside `wsh` or `tr` only)."
> — BIP-388 §Formal definition → Wallet descriptor template, lines 130–140

Four observations, in ascending order of force:

1. **`sortedmulti` is annotated "(inside `sh` or `wsh` only)"** — its own explicit position
   constraint, in the BIP that governs `md1`'s template shape.
2. **"any valid miniscript template" is a SEPARATE bullet** from `sortedmulti`. The grammar treats
   miniscript and `sortedmulti` as **disjoint alternatives** for a `SCRIPT` slot, exactly as
   BIP-386 §Rationale does (§3.2). A `sortedmulti` is therefore not reachable *via* the miniscript
   bullet.
3. **BIP-388 defers the miniscript production wholesale to BIP-379:**
   > "See [[bip-0379.md|BIP-379]] for a precise specification of all the valid miniscript `SCRIPT`
   > expressions." — BIP-388 line 142
   And BIP-379 contains no `sortedmulti` (§2.5). So the miniscript bullet cannot smuggle one in.
4. **The enumeration contains no combinators at all.** The only `SCRIPT` producers in BIP-388's
   list that take a `SCRIPT` argument are `sh()` and `wsh()`, each taking exactly one. Therefore
   within a BIP-388 template, **the only syntactic position a `sortedmulti` can occupy is the sole
   direct argument of `sh()` or `wsh()`** — which is `md1`'s rule, verbatim.

**Also note:** BIP-388's `SCRIPT` list does **not** include a bare top-level `multi`/`sortedmulti`
(unlike BIP-383, §3.1) — it is annotated "inside `sh` or `wsh` **only**". Under BIP-388, `md1`'s
rule and the standard coincide **exactly**, with no conservatism gap.

**One caveat on `sortedmulti_a` under BIP-388:** it is **absent from the `SCRIPT` enumeration**
(lines 132–140) yet **present in the test vectors** (line 291, quoted in §3.3) and in a worked
example (line 114, `multi_a`). BIP-388 acknowledges its own list is not yet complete:

> "Note: while descriptor templates for miniscript are not formally defined in this version of the
> document (pending standardization), it is straightforward to adapt this approach by adding
> additional `SCRIPT` expressions." — BIP-388 line 177

So BIP-388's *list* is under-specified for tapscript, and the governing text for `sortedmulti_a`
is BIP-387 + BIP-386 (§3.2). This under-specification does **not** create a nesting permission —
BIP-388's `TREE` production is "any `SCRIPT` expression" or a `{,}` pair (lines 144–146),
mirroring BIP-386, i.e. leaf-or-branch, still never a miniscript sub-expression.

---

## 5. Is `sortedmulti` a Miniscript fragment at all?

**No. Neither `sortedmulti` nor `sortedmulti_a` is a Miniscript fragment.**

This is the crux, and it is settled by an exhaustive, machine-checked absence rather than by
interpretation:

| Source | `multi` | `multi_a` | `sortedmulti` | `sortedmulti_a` |
|---|---|---|---|---|
| BIP-379 fragment table (l. 98–99) | ✅ | ✅ | ❌ | ❌ |
| BIP-379 type table (l. 148–149) | ✅ | ✅ | ❌ | ❌ |
| BIP-379 malleability table (l. 200–201) | ✅ | ✅ | ❌ | ❌ |
| BIP-379 satisfaction table (l. 241–242) | ✅ | ✅ | ❌ | ❌ |
| BIP-379 "overlap with descriptors" (l. 406) | ✅ | ✅ | ❌ | ❌ |
| `bitcoin.sipa.be/miniscript` translation table | ✅ | ✅ | ❌ | ❌ |

`grep -c sortedmulti bip-0379.md` → **0** (measured, not inferred).

The guarded confusion in the brief — "miniscript's `multi` and descriptor `multi()` are not
automatically the same thing" — resolves cleanly, and in a way that *sharpens* the verdict:
BIP-379 line 406 states the two `multi`s **do** coincide ("parse to the same semantic meanings as
those descriptors and produce the same scripts"), and that same sentence enumerates the coinciding
set as exactly `pk`, `pkh`, `multi`, `multi_a`. `sortedmulti` is deliberately outside it. So the
descriptor/miniscript distinction is real, and `sortedmulti` falls on the **descriptor-only** side
of it. There is no miniscript `sortedmulti` for a combinator to take as a child.

Where the combinators are concerned: `and_v(X,Y)`, `or_d(X,Z)` and `thresh(k,X_1,...,X_n)` take
**Miniscript sub-expressions** `X`/`Y`/`Z` (BIP-379 fragment table, lines 90/96/97). Since
`sortedmulti` is not a Miniscript expression, it is not a candidate value for `X`. The exclusion is
grammatical, not a side condition.

---

## 6. What rust-miniscript's refactor does and does NOT tell us

**This is the distinction the decision turns on, so it is stated plainly.**

### What it does NOT tell us

**An implementation's internal AST shape is not evidence about what the standard permits.**
rust-miniscript moving `sortedmulti` from `ShInner::SortedMulti`/`WshInner::SortedMulti` into
`Terminal::SortedMulti` changes **where rust-miniscript stores the node**, not **what BIP-379
defines as a fragment**. BIP-379's fragment table is unchanged by a downstream library's
refactor; a library is free to model a descriptor-level construct as an AST terminal for code-reuse
reasons while the grammar it implements says otherwise.

Concretely: the refactor is **not** presented as a spec-conformance fix. PR #915 is titled and
framed as an internal cleanup:

> **Title:** "refactor: remove SortedMultiVec and use Terminal::SortedMulti" *(merged 2026-04-12)*
>
> "- Utilize the Miniscript parsing to handle sortedmulti as a Terminal.
> - Deleted sortedmulti.rs (SortedMultiVec)
> - Refactor Wsh to only wrap a Miniscript now that SortedMultiVec isn't used.
> - Refactor ShInner to remove SortedMulti variant and only use the Ms variant for sortedmulti
>   scripts
>
> Now that sortedmulti_a is supported as a Terminal, I think it makes sense to move sortedmulti
> over in the same way by following what multi does and applying the sorting functions that were
> introduced in eba1aff. Like sortedmulti_a, sortedmulti is sorted upon encoding and **cannot be
> decoded into from Script**."
> — rust-miniscript PR #915, body (via GitHub API)

Every stated motive is code-structural — "utilize the Miniscript parsing", "deleted", "refactor",
"follow what multi does". No BIP is cited; no conformance defect is claimed; no wire-format or
grammar change is asserted. **The rationale is an internal cleanup, by the author's own account.**

The final sentence is in fact evidence for the *opposite* of a nesting permission: "cannot be
decoded into from Script" is exactly the property that keeps `sortedmulti` out of the Miniscript
grammar (§2.6). The PR reuses the Miniscript **parser and AST as machinery** while acknowledging
the construct fails Miniscript's defining round-trip property.

### What it DOES tell us (and why it matters operationally)

At rev `ff4732e`, the refactor makes a nested `sortedmulti` **representable and parseable** in
rust-miniscript, with no guard preventing it. Measured at that rev:

- `"sortedmulti"` is dispatched in the **generic miniscript expression parser**
  (`src/miniscript/mod.rs:1016`), i.e. at **any recursion depth**, not gated to a top-level slot.
- It is assigned a type like any other terminal — `Terminal::SortedMulti(..) => Ok(Self::sortedmulti())`
  (`src/miniscript/types/mod.rs:472`) — so it type-checks as a composable `B`-type sub-expression.
- A grep of `src/miniscript/types/` and `src/miniscript/context.rs` for `SortedMulti` shows
  **context/type handling only, and no depth or parent-position restriction**.

So the practical consequence is: **rust-miniscript at `ff4732e` will likely accept
`wsh(and_v(v:pk(A),sortedmulti(2,B,C)))` even though no BIP defines it.** That is an
implementation being *more permissive than the standard* — a liberal-parser posture, not a
grammar extension. For a funds-critical wire format the correct response to a newly-permissive
dependency is **not** to widen admission to match it.

*(Scope note: the three bullets above are structural facts read from the vendored source at
`~/.cargo/git/checkouts/rust-miniscript-ce5fa57e8900265e/ff4732e/`. I did not compile a test
asserting the parse succeeds — hence "likely accept". If that empirical fact becomes
decision-relevant, it is a ~5-minute check. It does not affect the verdict, which rests on the
BIPs.)*

### The upshot for the decision

The refactor changes **what rust-miniscript can represent**. It changes **nothing about what
BIP-379/383/387/388 define**. Adopting rev `ff4732e` therefore creates **no standards pressure to
relax** `md1`'s rule. Relaxing it would make `md1` encode shapes that **no BIP defines**, that
BIP-388's closed grammar excludes, and that other wallets/signers parsing BIP-388 policies would
be under no obligation to accept — an admission widening with no standard behind it, permanently
baked into a backup wire format.

---

## 7. Confidence, and what would change my mind

**Confidence: HIGH** for both `sortedmulti` and `sortedmulti_a`.

The verdict rests on three mutually independent legs, any one of which alone would be
near-sufficient, and which come from different documents by different authors:

1. **Positional clauses** (BIP-383 l. 37; BIP-387 l. 38) enumerate permitted positions as closed
   lists that exclude combinator arguments.
2. **Grammatical exclusion** — `sortedmulti`/`sortedmulti_a` are absent from BIP-379 and from the
   Miniscript reference site, in *every* table, measured at zero occurrences. A combinator's child
   must be a Miniscript expression; these are not.
3. **BIP-388's closed enumeration** (l. 130–142) lists `sortedmulti` with "(inside `sh` or `wsh`
   only)" and lists miniscript as a *separate* alternative, deferring it to BIP-379 — which has no
   `sortedmulti`. For `md1`'s BIP-388-shaped templates this is directly binding.

Legs 1 and 3 are positional/normative; leg 2 is grammatical. They would have to fail together.

**This is a `RESTRICTED` finding, not a `SILENT` one.** The brief rightly flagged that "the
standard does not address this" is a valid answer — I considered it and rejected it. The standards
are not silent by omission-through-oversight: BIP-383 and BIP-387 *affirmatively state* where these
expressions may be used, BIP-388 *affirmatively annotates* the constraint, and BIP-386 §Rationale
*affirmatively distinguishes* `sortedmulti_a` from "the Miniscript expressions of BIP 379". The
absence from BIP-379 is corroborated by a stated structural reason (no Script→AST decode, §2.6).
That is a specification excluding something, not failing to consider it.

**What would change my mind — concretely:**

1. **A BIP-379 revision adding `sortedmulti`/`sortedmulti_a` to the fragment table.** This is the
   only thing that would make nesting standard-permitted, and it is the check to re-run before any
   future rev bump. It would require resolving the Script→AST decode problem (§2.6), so it is
   unlikely, not merely absent.
2. **A new BIP** (or a BIP-388 revision) explicitly defining `sortedmulti` in a combinator
   position — e.g. BIP-388 completing its tapscript enumeration in a way that permits it inside a
   miniscript template rather than alongside one.
3. **Normative text I did not locate** stating that "inside of a `sh()`/`wsh()` descriptor" in
   BIP-383 means *transitively contained* rather than *direct argument*. I found none; the
   `sh`/`wsh` single-argument clauses (BIP-381 l. 61, BIP-382 l. 47) and the absence of any
   non-miniscript combinator argue the other way, and BIP-381 uses the identical phrasing for
   `pkh()` (l. 48) — where the transitive reading is separately handled by `pkh` *also* being a
   real BIP-379 fragment, which `sortedmulti` is not.
4. *(Would NOT change my mind:)* further rust-miniscript releases accepting nested `sortedmulti`,
   more permissive parsing in Bitcoin Core, or other wallets emitting such descriptors. Per §6
   these are implementation facts. They could motivate a *separate* interoperability decision, but
   they are not evidence about what the BIPs permit.

**Recommendation implied by the finding (stated once, not argued):** HOLD the rule. The assertion
at `crates/md-codec/src/to_miniscript.rs:575-577` is consistent with BIP-383, BIP-387 and BIP-388.
The `ff4732e` adoption is a dependency-internal refactor and does not bear on `md1`'s admission set.

---

## Appendix: sources consulted

| Source | Retrieved | Key lines |
|---|---|---|
| BIP-380 (Output Script Descriptors) | raw.githubusercontent.com/bitcoin/bips/master | 45–66 |
| BIP-381 (`pk`,`pkh`,`sh`) | ″ | 48, 60–61 |
| BIP-382 (`wpkh`,`wsh`) | ″ | 46–47 |
| BIP-383 (`multi`,`sortedmulti`) | ″ | 33–41 |
| BIP-386 (`tr`, Tree Expressions) | ″ | 33–44, 56, 115–118 |
| BIP-387 (`multi_a`,`sortedmulti_a`) | ″ | 33–38 |
| BIP-388 (Wallet Policies) | ″ | 126–146, 177, 290–291 |
| BIP-379 (Miniscript) | ″ (`bip-0379.md`) | 67–72, 88–102, 148–149, 200–201, 241–242, 402–408 |
| Miniscript reference site | bitcoin.sipa.be/miniscript | translation table |
| rust-miniscript PR #915 | api.github.com (title/body/merge date) | — |
| rust-miniscript rev `ff4732e` | local `~/.cargo/git/checkouts/…/ff4732e` | `astelem.rs:155`, `mod.rs:1016`, `types/mod.rs:472` |
