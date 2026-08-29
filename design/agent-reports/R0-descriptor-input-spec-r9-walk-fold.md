# R0 round 9 — PROPORTIONAL review of the journey-walk fold

**Target:** `design/SPEC_descriptor_input.md` at `d0647f4` ("spec: fold the journey
walk -- 15 findings from two live journeys").
**Source of truth for what was required:** `design/WALK_descriptor_input_2026-08-28.md`.
**Scope:** the fold only. Not a fresh audit. r1–r8 results, the walk's own
measurements, the citation gate, F-417/F-418/F-422 rulings taken as settled.
**Reviewer:** independent context, opus tier. Read-only against
`mnemonic-engrave`, `descriptor-mnemonic`, `seedhammer`; one scratch crate built
outside the repos.

---

## Counts — NEW findings

| severity | count |
| --- | :-: |
| **Critical** | **1** |
| **Important** | **5** |
| Minor | 6 |
| Nit | 1 |

**The spec does NOT re-close GREEN this round.** One Critical and five
Importants are open, four of them arising from the fold itself (two of those
from normative rules the fold introduced and did not sweep its own artifact
against).

---

## Disposition of the walk's spec-fold subset

| walk finding | required | verdict | note |
| --- | --- | :-: | --- |
| **W1** (modality: §5.1 flag lines + §5.5 two rows) | plate/restore clause on each `--as` line; §5.5 rows *on the plate* and *restored by* | **FIXED** | both lines rewritten (L744–758); both rows present (L1069–1070). |
| **W14** (two modality dimensions + measured facts) | ecosystem-dependence and hand-reproducibility into the modality table; BCH sentence | **FIXED** | ecosystem dependence lands in *restored by*; new *hand-copyable* row L1071. 2 strings / ~168 chars present. See **M5** — the one-plate fact the commit message claims is not in the artifact. |
| **W4** (S3-window refusal) | §5.1 window clause, `EXIT_REFUSED` (3); §6 row; §11 item 5 sibling test | **FIXED** | L807–830, §6 row L1132, §11 item 5 L1550–1554. |
| **W5** (operator language) | (1) use the rewrite; (2) NORMATIVE §6 preamble rule; (3) **sweep** §6's quoted texts for leaks | **PARTIAL** | (1) verbatim ✓. (2) rule added L1109–1114 ✓. (3) **the sweep is incomplete — 3 quoted texts still carry `§` references.** See **I5**. The commit message's "sweep clean" is false. |
| **W11** (window refusal's conditional alternative + the symmetric caveat) | conditional alternative clause ✓ **and** "§5.3(a)/(a″)'s remedies … append *(--as descriptor is not in this build yet — keep the file.)*" | **PARTIAL** | the conditional clause landed (L822–830). **The symmetric half was not folded at any of its 4 sites.** See **I4**. |
| **W6** (the one-step fact) | §5.1's `--as md1` help line **and** "the W4 window refusal's alternative line gains the same three words" | **PARTIAL** | help line ✓ (L751–753). Window refusal's alternative line ✗. See **M2**. |
| **W8** (a′ annotation shown *and explained*) | one annotation line in operator terms; (a′)'s "so the operator sees it" upgraded | **FIXED** | L944–951; §5.4 bullet L1034–1035 cross-references it. But the annotation's content is wrong for most (a′) inputs — see **I3**. |
| **W9** (BIP-cited) | "Add the BIP grounding **as the leading rationale**"; annotation cites it operator-checkably | **PARTIAL** | the annotation cites BIP-48/BIP-388 ✓, but §5.3(a′)'s rationale paragraph still leads with *"the device is the reader of both artefacts"* (L940–943) — the private authority W9 asked to demote. See **M3**. And the cited BIP is wrong for single-sig (**I3**). |
| **W10** (wallet-id + address 0 + compare prompt; §7 `wallet_id`) | three additions | **PARTIAL** | all three text additions present (L1029–1035, L1234–1235, L1373–1378). **The `wallet-id` line is not satisfiable as specified** — see **C1** — and its "same identifier under BOTH `--as` values" clause is false under the spec's own (a′) rule — see **I2**. |
| **W13** (identify on EVERY successful parse, before pack or refusal) | one rule: parse succeeded ⇒ identify, then say what can/cannot be done | **PARTIAL** | rule stated L1019–1024 ✓, but its follower set is enumerated as {pack, window refusal, §5.3 refusal} while §6 places **§4.7 admission refusals** after a successful parse too — where `address 0:` is underivable. See **I1**. |
| **W15** (owner-quotable watch-only line) | one line, both halves | **FIXED** | L1036–1039, verbatim from the walk. |
| **F-419** (zero-cosigner row) | a §6 row | **FIXED** | L1131. Cites "§4.2 rule 2"; verified — §4.2's NORMATIVE four-shape sentence lists *zero cosigner lines* second. Text is W5-compliant. |
| **§11 item 5 window sibling** | sibling test pinning both variants | **FIXED** | L1550–1554, both alternative variants named. |

---

# NEW findings

## C1 — `wallet-id:` is UNCOMPUTABLE for exactly the wallets §5.4 now promises it for, and the obvious implementation prints a DIFFERENT wallet's identifier

**Where.** §5.4 L1019–1024 (the block "prints on EVERY successful host-side
parse, BEFORE … a §5.3 refusal") and L1029–1031:

> - **`wallet-id:` the WalletPolicyId fingerprint** — the same identifier for
>   the same wallet under BOTH `--as` values, computed host-side …

Unconditional. No representability caveat.

**Why it cannot hold.** `WalletPolicyId` is defined over the **md1 policy
encoding**, in both implementations:

- Rust: `compute_wallet_policy_id(d: &md_codec::encode::Descriptor)` —
  `descriptor-mnemonic/crates/md-codec/src/identity.rs:186`. Its preimage is
  the placeholder tree bytes plus per-`@N` origin / **use-site** / fp / xpub
  records.
- Go: `md.WalletPolicyId(d *descriptor)` —
  `seedhammer/md/walletpolicyid.go:30`, a declared byte-exact port; the only
  public entry points are `WalletPolicyIdChunks(strs []string)` /
  `WalletPolicyIDStubChunks` (`:138`, `:148`), i.e. **from md1 strings**.

And `UseSitePath` — the field the preimage hashes — cannot denote the shapes
§5.3(a)/(a″) refuse. `md-codec/src/use_site_path.rs:63–68` is
`multipath: Option<Vec<Alternative>>` + `wildcard_hardened: bool`, with
`MIN_ALT_COUNT = 2`. The spec says this itself at L890–893 ("There is **no
representation for one fixed index**"), and F-417 records it as deliberate,
naming "`WalletPolicyId` identity" among the things a wire change would touch.

**Constructed failure** (scratch crate, path-dep on the local `md-codec`
0.42.0, `default-features = false`; source kept at
`/tmp/claude-1000/-scratch-code-shibboleth-mnemonic-engrave/676332c4-5ca1-4168-9b4e-027157d04a28/scratchpad/wpid-probe/src/main.rs`):

```
--- what UseSitePath CAN denote ---
/*            (multipath None)     OK   wallet-id 3bf32c0e90bf86a22885eea2fd2c2b4e
/*h           (hardened star)      OK   wallet-id 2fad0885b2599b9a531d7df3fe25f0d0
<0;1>/*       (standard)           OK   wallet-id 24bcacf52ad82ff0c6c8f629001b9fe3
<1;2>/*                            OK   wallet-id f12edc70f929960b0ad28be3a25712e8

--- the (a)-shaped wallet: /0/*  (spec 5.3(a), Specter export) ---
/0/*  as 1-alt group               ERR  AltCountOutOfRange { got: 1 }
<0;1> (no trailing wildcard)       UNREPRESENTABLE: UseSitePath has no
                                   'no wildcard' state (wildcard_hardened: bool)
/0/1/* (deeper tail)               UNREPRESENTABLE: no depth field
```

(`wsh(sortedmulti(2,@0,@1))`, origin `m/48'/0'/0'/2'`, fps `dc567276`/`f245ae38`,
two deterministic 65-byte xpubs. The only structural attempt at a single fixed
index is a 1-alternative group, and it **errors**.)

**The two failure modes, both bad.**

1. **Unsatisfiable.** The `/0/*` Specter export is the case F-417 calls "the one
   real-world case … carried EXACTLY by `--as descriptor`" and the case W11/W13
   built the whole identify-before-refuse rule around. On BOTH the supported
   `--as descriptor` path and the `--as md1` §5.3(a) refusal path, §5.4 requires
   a `wallet-id:` line that has no value. Same for every (a″)-shaped input, and
   for `/0/1/*`.
2. **Silently wrong.** An implementer who satisfies §5.4 by building the md1
   `Descriptor` anyway gets `md encode`'s documented collapse (§5.3(a) L897–901:
   `@0/0/*` and `@0/*` share chunk-set-id `0x9bf18`) and prints:

   ```
   id of the /*      wallet: 3bf32c0e90bf86a22885eea2fd2c2b4e
   id of the <0;1>/* wallet: 24bcacf52ad82ff0c6c8f629001b9fe3
   equal? false
   ```

   i.e. the identifier of a **different wallet** — §5.3(a)'s measured
   `bc1qu2cc6t7…`-vs-`bc1qadgf37z…` divergence — printed two lines above
   *"compare against your wallet software's first receive address before
   engraving."* This is the exact silent-collapse hazard §5.3(a) exists to
   prevent, re-entering through the surface the walk added to prevent errors.

**Corroboration inside the fold itself.** §7 (L1373–1375) says rows "**may
also** carry `wallet_id`" — optional, correctly, because md1-unrepresentable
rows cannot. §5.4 makes the same value mandatory. The two halves of one commit
disagree about whether the identifier always exists.

**Not prescribing a fix** (the fold owns that), but the shape of the gap is:
§5.4 must state the condition under which `wallet-id:` is emitted, what is
printed in its place when the wallet has no md1 policy encoding, and — if the id
is to be claimed identical across `--as` values — which descriptor it is computed
over (see **I2**).

---

## I1 — the identification block's follower set is enumerated smaller than the rule that governs it, and `address 0:` is underivable on four §6 rows inside the gap

**Where.** §5.4 L1019–1024 states the rule twice, at two different widths:

> **NORMATIVE — and the block prints on EVERY successful host-side parse,
> BEFORE whatever follows: a pack, §5.1's window refusal, or a §5.3 refusal
> (walk W13: parse succeeded ⇒ identify the wallet, always …)**

The governing clause is "EVERY successful host-side parse". The enumeration is
three items. **§6 states, in its own words, that the enumeration is incomplete**
(L1101–1104):

> Rows below that arise from **§4.7's admission predicate** or §5.3's
> representability limits **fire from their own checks, after a successful
> parse** — the rule never selects them (R0 r2's NEW-N1).

So the boundary *is* derivable — and the derivation makes the rule bind §4.7's
admission refusals, which §5.4 never mentions. On four of those rows the block's
`address 0:` line has no value, by the spec's own measurements:

| §6 row | line | why `address 0:` cannot be produced (spec's own text) |
| --- | :-: | --- |
| multisig mixing mainnet and testnet keys | 1141 | *"…then cannot derive any address from it"* (§4.7 conjunct 5) |
| single-key script wrapping a multi — `wpkh(sortedmulti(…))` | 1152 | *"cannot derive any address from it (measured: `address: multisig script: … unsupported descriptor`)"* (conjunct 1) |
| bare key in a script slot — `wsh(KEY)`, `sh(KEY)` | 1153 | *"`Supported=false`, `address: singlesig script: … unsupported descriptor`"* (conjunct 1) |
| hardened use-site component — `…/<0;1>/*h` | 1154 | *"a hardened use-site step cannot be derived from an xpub (BIP-32)"* (conjunct 7) |

And on two more the line is derivable but the **instruction attached to it is
wrong**: `sortedmulti(k,…)` with `k > n` (L1139, "no combination of signatures
reaches `k`. Funds sent to this wallet would be unspendable") and
`sortedmulti(0,…)` (L1140, "anyone who can see this script can spend from it")
would print *"compare against your wallet software's first receive address
before engraving"* for a wallet the same screen has just called unspendable or
anyone-can-spend.

**A third conflict site, outside §4.7.** `--as` omitted (§6 row L1130,
`EXIT_USAGE` 2) fires *after* a successful whole-input parse — that is exactly
how §5.1's discriminator works (L818–826 of the pre-fold text: "If it parses as
one descriptor … the input IS a descriptor and gets the '--as decides' block at
`EXIT_USAGE` (2)"). The governing clause says identify; the enumeration says
nothing; §11 item 4 requires a test asserting the text. Two readings, materially
different stderr, and no way to choose from the document.

(The §5.3(b) label **warning** is a fourth, smaller instance: it also follows a
successful parse and its position relative to the block is unstated.)

**Why this is Important and not cosmetic.** §11 item 4 requires a text-asserting
test per §6 row. For the four rows above, no implementer can write one that
satisfies §5.4 as worded.

---

## I2 — "the same identifier for the same wallet under BOTH `--as` values" is FALSE for every childless descriptor, under the spec's own (a′) rule

**Where.** §5.4 L1029–1031, the clause quoted in C1.

**The construction.** §5.3(a′) is scoped to one flag: *"for EVERY key whose
use-site path is ABSENT, **`--as md1`** materialises the device's default,
`<0;1>/*`, into that key's encoding"* (L936–938). `--as descriptor` has no such
rule — §5.2 packs `Descriptor::encode()` of the literal, childless input.

So an implementer reading the spec literally computes the id over two different
md1 `Descriptor`s depending on the flag:

| `--as` value | md1 `Descriptor` the id is computed over | measured `wallet-id` |
| --- | --- | --- |
| `md1` | (a′)-materialised → `<0;1>/*` | `24bcacf52ad82ff0c6c8f629001b9fe3` |
| `descriptor` | the literal childless input → `/*` (§5.3(a′) L926: `{multipath: None, wildcard_hardened: false}` **IS** `/*`) | `3bf32c0e90bf86a22885eea2fd2c2b4e` |

Same input, same wallet, two ids — measured by the probe above. The claim is
falsified by the spec's own two sections.

This is not hypothetical scope: §5.3(a′) L929–931 says "**every §4.5 promoted
bare key is childless**", so it covers the walk's entire journey 2 and every
BlueWallet file without a use-site tail.

The fix direction §5.4 must state explicitly (it currently does not): the id is
computed over the **(a′)-materialised** policy on both paths, which is what the
address-layer equality argument at L938–943 already justifies. As written, an
implementer has a 50% chance of shipping the divergence.

---

## I3 — the (a′) annotation asserts "your keys' BIP-48 origin" to operators whose keys have a BIP-44/49/84 origin, including the walk's own journey 2

**Where.** §5.3(a′), L947–951, the operator-facing quoted text the fold added:

> *"note: your file names no derivation below the key origins; `<0;1>/*` is the
> **standard receive/change continuation of your keys' BIP-48 origin**
> (BIP-388's canonical tail). Addresses are unchanged by making it explicit."*

**Why it is wrong.** (a′) fires for *every* key with an absent use-site path.
§5.3(a′) L929–931 states that **every §4.5 promoted bare key is childless**, and
§6's bare-key row (L1145) lists the three inferable origins: `m/44'/0'/0'`
(`pkh`), `m/84'/0'/0'` (`wpkh`), `m/49'/0'/0'` (`sh(wpkh)`). None is BIP-48.

**The walk's own journey 2 is a counterexample.** W14 records the bequest card as
*"keyed single-sig `wpkh`, materialised `<0;1>/*`, **BIP-84 origin**"* — i.e. the
one journey the annotation was measured on would be told a false fact about the
operator's own key, at the moment W8 identified as "the journey's highest-stakes
moment".

**This defeats W9's stated purpose.** W9's argument was *"An authority the
operator can verify beats 'the device does this'."* An operator who takes the
invitation and opens BIP-48 for their `m/84'/0'/0'` `zpub` finds a multisig
specification that never mentions their path — strictly worse than the device
sentence it replaced, because it invites a check that fails.

The text is quoted as literal output with no substitution marker, unlike §6's
rows which mark substitution explicitly (`<fp>`, `@N`, and §6's binding rule
*"the remedy must be executable … not a placeholder"*, L1160–1163). So it is not
readable as a template.

---

## I4 — W11's symmetric half was not folded: four sites still name `--as descriptor` as a remedy in a build where it refuses, violating the rule the same commit made NORMATIVE

**What W11 required** (walk log, W11 classification, second sentence of the
disposition):

> Symmetrically, §5.3(a)/(a″)'s remedies, in a build where S2 has not shipped,
> append: *"(--as descriptor is not in this build yet — keep the file.)"* **No
> refusal may point at a flag that refuses in the CURRENT build** — the r5 rule,
> now stated over build windows, not just admission.

**What the fold did.** Only the first half — the window refusal's conditional
alternative clause (L822–830). The four remedy sites are untouched:

| site | line | text |
| --- | :-: | --- |
| §5.3(a) NORMATIVE | 919–921 | "The refusal names the offending key and `--as descriptor`, which carries that shape exactly" |
| §5.3(a″) NORMATIVE | 964–966 | "The refusal names the offending key and `--as descriptor`, which carries it exactly" |
| §6 `/0/*` row (quoted) | 1138 | *"Use `--as descriptor`, which carries `/0/*` exactly."* |
| §6 `<0;1>` row (quoted) | 1156 | *"Use `--as descriptor`, which carries `<0;1>` exactly."* |

**The fold made this a self-contradiction.** The same commit added, at L1109–1114,
a NORMATIVE rule binding "every quoted text below":

> … and names only next actions **executable in the CURRENT build** (walk W11).

Lines 1138 and 1156 are quoted texts below that rule and they violate it
directly. Machine-checked: a scan of §6's `*"…"*` spans for `--as descriptor`
returns exactly these two rows.

**Residual operator cost.** The composed loop W11 found is *broken* (the window
refusal's variant 2 terminates rather than pointing back), so this is Important,
not Critical. But the archival operator is still sent on a full wasted round
trip by a message that promises a path this build does not have — the outcome
W11 classified as worse than silence.

A related site: §6's "any other use-site path shape" row (L1155) states *"use-site
paths `me` packs: absent, `/*`, `/i/*`, `<i;i+1>`, `<i;i+1>/*`"* — in the
S3-only build `me` packs neither `/i/*` nor `<i;i+1>` by any flag.

---

## I5 — W5's sweep is incomplete: three quoted refusal texts still carry spec `§` references, and the fold's commit message records the sweep as clean

**What W5 required** (item 3 of its classification): *"Sweep §6's quoted texts
for existing leaks: **at least** the multi-record row's quoted message contains
'(F-414)' today. Fix in the walk fold."*

**What the fold did.** Fixed the one named instance (F-414 moved outside the
quotes at L1157) and stopped. Mechanical re-sweep of every `*"…"*` span in §6's
table against `§\s*\d | F-\d{3} | \bS[123]\b | R0 | NEW-[A-Z]\d`:

| line | row | leak inside the quoted text |
| :-: | --- | --- |
| 1136 | `wsh(multi(…))` under `--as descriptor` | *"(for md1-representable use-site paths — otherwise **the §5.3 rows** state that neither path carries it)"* |
| 1137 | a miniscript descriptor, either `--as` | *"`md encode` accepts miniscript **templates** — **see §10**."* |
| 1150 | a bitcoin address | *"No program on the device consumes an address record — **see §10**."* |

Three operator-facing messages instruct the operator to consult sections of a
design document they do not have. That is precisely the class W5 was opened by
("*Now I understand through a convoluted message*" / "no spec § references
inside the quoted text"), and line 1136's leak is additionally *load-bearing* —
it is the only thing telling the operator that a `multi` policy with a `/0/*`
path is refused on both paths.

**The rule's first clause is also unswept.** L1110 binds every quoted text to
"**leads with the verdict**". Row 1138 (the `/0/*` refusal) opens with the key
name and two sentences of codec mechanism before the verdict — the exact
mechanism-first, verdict-last shape W5 rejected in the drafted window refusal.
I am not putting a count on this clause (it is judgment-laden where the `§`
leaks are not), but as written §11 item 4's per-row text tests and L1110 give an
implementer contradictory instructions on several rows.

**The commit message asserts the opposite:** *"the F-414 leak moved out of the
quoted text; … sweep clean."* Not clean — one instance of a class the walk
flagged as "at least".

---

# Minor

**M1 — the "citation corrected" edit made a correct citation wrong.** §5.4 L1041–1043
now reads "(R0 **r3's** NEW-N2, citation corrected from r2 in the walk fold …)".
The original `r2` was right and the correction is wrong:

- `R0-descriptor-input-spec-r2.md:338` — **"NEW-N2 — §5.4's confirmation list did
  not follow I5's fold."** Body: §5.4's bullet "still says only *'for a promoted
  bare key (§4.5), the fact of the promotion'*. The two sections describe the
  same stderr block…". This is verbatim what §5.4's bullet cites.
- `R0-descriptor-input-spec-r3.md:321` — r3's **own** NEW-N2 is *"the `Upub`
  remedy names the wrong multisig form"*, an unrelated Nit about §6's SLIP-132
  row.
- `R0-descriptor-input-spec-r3.md:44` is r3's **disposition table row**
  dispositioning **r2's** NEW-N2 as FIXED — which is, most likely, what was read
  as "r3's NEW-N2".

Restore `R0 r2's NEW-N2` and drop the "corrected from r2" parenthetical. The
commit message's "One citation misattribution corrected in passing (r2 -> r3
NEW-N2)" should be retracted in the fold that fixes this.

**M2 — W6's second half.** W6: *"The W4 window refusal's alternative line gains
the same three words ('me converts and packs in one step')."* §5.1's help line
got the one-step fact (L751–753); the window refusal's variant-1 clause
(L822–826) did not. The operator who reaches the window refusal without having
seen the `--as`-omitted block — i.e. both walked journeys, which typed `--as
descriptor` on their first command — never learns the one-step fact.

**M3 — W9's "leading rationale" not moved.** §5.3(a′)'s rationale still opens
"the device is the reader of both artefacts, and `<0;1>/*` is what it already
derives" (L940–943); the BIP grounding appears only inside the operator quote.
W9's three-way grounding (BIP-48 continuation, BIP-388 `/**` ≡ `/<0;1>/*` with
the F-411 machine-verified byte-identical result, BIP-389 notation) is not in the
spec at all. Fixing **I3** is the natural moment to write it in properly.

**M4 — the window refusal's variant 2 lost the operator's own path.** W11's text
was *"--as md1 cannot carry this wallet's **`/0/*`** path either"*; the fold
generalised it to *"this wallet's **use-site path**"* (L827). "Use-site path" is
codec vocabulary the operator has never met, and §6's binding rule (L1160–1163)
requires the operator's own values substituted in. Print the actual path.

**M5 — the one-plate fact the commit message claims is not in the artifact.**
The message says §5.5 carries "the measured 2-strings/**168-chars/1-plate**
facts"; §5.5 (L1069–1071) has the first two and not the third. Verified the
citation is real — `seedhammer/gui/bundle_engrave_test.go:47`,
`func TestBundlePlanSingleMD1OnePlate`. The operator's stated hope in W14 ("*we
are hopeful it will be a short 1 plate engraving*") is left unanswered by the
capability table, and it is the one measured fact that answers "how much steel
is this?".

**M6 — §5.1's help block offers `--as descriptor` as a live choice inside the
window.** The `--as`-omitted block (L744–758, `EXIT_USAGE` 2) presents both flags
symmetrically; in the S3-only build one of them refuses. W1's corollary assigned
this question to *the plan*, not the spec, so it is not a fold miss — but the
fold's own new rule at L1114 ("names only next actions executable in the CURRENT
build") now bears on it, and §6's `--as`-omitted row (L1130) imports the block by
reference into the table that rule governs. Rule the interaction explicitly in
one of the two documents.

# Nit

**N1 — "survives up to 4 wrong characters" is substitution-only.** §5.1 L754–756
and §5.5 L1071 both sell the BCH budget to a hand-stamper. Verified: t = 4 is
correct and per string (`md-codec/src/bch_decode.rs:433`, "> 4 errors is above
the BCH(93, 80, 8) / t = 4 capacity"; `chunk.rs:323` "per-chunk BCH via
`unwrap_string`"), and `decode_regular_errors` returns `(positions, magnitudes)`
to XOR into existing symbols — i.e. **substitutions**. A *missing* or *extra*
punch changes the codeword length and is not inside the budget at all. "4
mis-struck characters" is the honest phrasing for the hand-stamping sentence.

---

# Verified in passing — recorded so a later round does not re-spend it

- **All ten walk-W citations in the spec resolve** to headings that exist in
  `WALK_descriptor_input_2026-08-28.md` (W1, W4, W5, W8, W9, W10, W11, W13, W14,
  W15), and each is used for the finding it names.
- **F-422 consistency: clean.** `FOLLOWUPS.md:14560–14567` records the interim
  status-quo ruling ("W11's neither-path window text is the shipped behaviour");
  §5.1's variant-2 clause is that text. No transform is implied anywhere in the
  fold.
- **The W5 rewrite is verbatim.** §5.1 L816–826 reproduces the walk's drafted
  message word for word, including the line breaks.
- **W15's watch-only line is verbatim**, both halves.
- **F-419's row cites correctly.** §4.2's NORMATIVE sentence (L363–366) lists the
  four refused shapes in order; "zero cosigner lines" is the second, so "§4.2
  rule 2" lands.
- **§7's `wallet_id` addition is internally sound** — "rows **may also** carry
  `wallet_id`" (L1373) is the right modality, and the Go side can assert it via
  `md.WalletPolicyIdChunks` (`seedhammer/md/walletpolicyid.go:138`) for
  md1-admitting rows. It is §5.4, not §7, that over-promises.
- **The fork test the walk cites exists:** `TestBundlePlanSingleMD1OnePlate`,
  `seedhammer/gui/bundle_engrave_test.go:47`.
- **No §6 row wrongly implies identification precedes a pre-parse refusal.** The
  window row (L1132) is the only row that mentions the block, and it is correct.
  The multi-record row fires only when the whole-input parse *fails*, so it is
  correctly outside the rule. The defect in **I1** is the governing clause
  over-reaching, not a §6 row over-claiming.

---

# What would re-close the round

C1, I1, I2, I3, I4, I5 folded, then a re-review scoped to *"did the fold fix
each of the six, and did it introduce a new defect"*. C1 and I2 are one edit's
worth of decision (what `wallet-id` is computed over, and when it is omitted);
I1 is one sentence naming the follower set precisely; I3 and I4 are text; I5 is a
three-line sweep. The Minors and the Nit can ride along in the same fold.

The walk lens itself is complete — every finding the walk classified as a spec
change is present in the artifact in some form, and nothing in this round comes
from a question the walk did not ask. What is open is fidelity and the two
normative rules the fold wrote without sweeping its own document against them.
