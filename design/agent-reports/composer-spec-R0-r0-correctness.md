# R0 round 0 — `SPEC_wallet_policy_composer.md`, CORRECTNESS AND INTERNAL CONSISTENCY lens

**VERDICT: 1 Critical / 11 Important / 10 Minor / 2 Nit — NOT GREEN.**

Artifact: `design/SPEC_wallet_policy_composer.md` at mnemonic-engrave `b452a79`.
Trees measured against: fork `169073c` (clean), descriptor-mnemonic working tree
`3b0944fb` (spec pins `790fc224`, which exists), rust-miniscript-fork
`src/` as checked out, `md 0.14.0` / `me 0.7.0` / `ms 0.16.0` / `mk 0.13.0`
resolved BY PATH (`~/.cargo/bin/md`; bare `md` on this box is `mkdir`).

Scope honoured: the 29 rulings are treated as final; §5's witness costs were not
re-derived; lock ranges were re-checked against BIP text only because the brief
permitted it, and they hold. Nothing in the repo was modified except this file.

---

## CRITICAL

### C1 — §6b's absolute-date entry has NO lower bound, so a mistyped year emits a block-HEIGHT lock (already satisfied) while the screen echoes a date

**Section and sentence.** §6b, lock-entry table:

> `| absolute | date | `YYYY-MM-DD` | `after(unix at 00:00:00 UTC)`; ceiling 2038-01-19 | "DATE 00:00 UTC" + "at least N days after this payload was packed on <pack date>" when `now:` is present |`

and the only date refusal in the section:

> "A date or height BELOW the `now:` value → "That is before this payload was
> packed. Choose a later date." Without `now:` the echo shows the typed value
> alone; the copy never says "now"."

**The defect.** The row states a ceiling and no floor. §4c's own table splits
`after(n)` at `LOCKTIME_THRESHOLD`: **height** is `1..=499,999,999`, **time** is
`500,000,000..=2,147,483,647` (1985-11-05 .. 2038-01-19). Any date the operator
types before **1985-11-05 00:53:20 UTC** converts to a value in the HEIGHT band,
which consensus reads as a block height — and every such height is already
mined, so the lock is **no lock at all**. §4c does not save §6b: the emitted
operand is inside §4c's admitted set, merely in the wrong row of it.

The floor is also unreachable by the `now:` refusal, because `now:` is optional
by construction — C26 explicitly allows Build with **no payload**, and a payload
with no `now:` record is permitted by §6a. On that path there is no lower bound
of any kind.

**Evidence.**

```
$ md encode "wsh(or_i(pkh(@0/<0;1>/*),and_v(v:pkh(@1/<0;1>/*),after(1))))"          # OK
$ md encode "wsh(or_i(pkh(@0/<0;1>/*),and_v(v:pkh(@1/<0;1>/*),after(500000000))))"  # OK
$ md encode "wsh(or_i(pkh(@0/<0;1>/*),and_v(v:pkh(@1/<0;1>/*),after(2147483648))))"
md: template parse error: miniscript parse failed: absolute locktimes in
Miniscript have a maximum value of 0x7fffffff; got 0x80000000
```

`after(345600)` — 1970-01-05 00:00 UTC — is inside the height row md accepts, and
BIP-65 l.27 is explicit that the comparison is like-with-like: *"the lock-time
type (height vs. timestamp) of the top stack item and the nLockTime field are not
the same; or"* → the script fails only if the types differ, so a height operand
is enforced as a height. Core `LOCKTIME_THRESHOLD{500'000'000}` (BIP-65 l.241-250,
quoted in the BIP's own reference implementation block).

**Consequence.** §4c's stated guarantee — *"Every other operand miniscript would
accept is either masked by consensus to a different lock or to no lock
(`older(0x400000)`), and the composer never emits one"* — is unmet by the entry
path that §4c delegates to (*"The device enforces these tables itself (§6b)"*).
The operator's consent screen says a future date; the engraved recovery path is
spendable now, on steel, unrecoverably. This is the same defect class as the
already-filed `older(0x400000)` acceptance, on the absolute side and in the
composer's own UI rather than in md.

**Minimal fix.** One clause in §6b's date row, independent of `now:`: reject any
date whose Unix seconds are `< 500,000,000` — *"Dates before 1985-11-05 cannot be
written as a time lock."* — and state that the date entry's admitted band is
exactly §4c's time row. (The height row already carries its floor and ceiling,
`1..499,999,999`; only the date row is one-sided.)

---

## IMPORTANT

### I1 — §5's taproot spine is undefined for ZERO and for ONE leaf; the literal one-leaf form `{P1}` is refused, and the zero-leaf case has two lowerings with DIFFERENT addresses

**Section and sentence.** §5, `paths combine`, tr column:

> "one leaf per path on a right spine in listed order `{P1,{P2,{P3,P4}}}`; path k
> at depth min(k, n−1)"

together with the `internal key` row: *"the FIRST-LISTED unlocked, unhashed
one-key path (then not a leaf); otherwise NUMS"*.

**The defect, case A (one leaf).** Two paths under `tr` where path 1 is a single
unlocked key: path 1 is extracted as the internal key, leaving exactly one leaf.
The rule's brace notation applied literally gives `tr(IK,{P2})`, which is not
BIP-386 TREE grammar (`TREE := SCRIPT | '{' TREE ',' TREE '}'`) and is refused:

```
$ md encode "tr(@0/<0;1>/*,{multi_a(2,@1/<0;1>/*,@2/<0;1>/*)})"
md: template parse error: miniscript parse failed: taptree branch must have 2
children, but found 1
$ md encode "tr(@0/<0;1>/*,multi_a(2,@1/<0;1>/*,@2/<0;1>/*)")      # OK  (bare)
$ md encode "tr(50929b74…3ac0,{sortedmulti_a(2,@0/…,@1/…)})"       # REFUSED, same
$ md encode "tr(50929b74…3ac0,sortedmulti_a(2,@0/…,@1/…))"         # OK  (bare)
```

The one-leaf case is also reached with NO extraction: a single path under `tr`
that is not a lone unlocked key (NUMS + one leaf) — the commonest taproot
multisig there is.

**The defect, case B (zero leaves).** One path under `tr`, one key, unlocked,
unhashed. §4b admits `n in 1..=9` and §4e refuses nothing here, so the list is
composable; the internal-key rule then removes the only path from the leaves.
Two implementers diverge on a wallet, not on text: `tr(@0/<0;1>/*)` (accepted by
md — measured) versus keeping it as a leaf under NUMS, `tr(NUMS,pk(@0/<0;1>/*))`.
Those are **different output keys and different addresses** for the same operator
input.

**Minimal fix.** Add to the tr `paths combine` row: *"a single leaf is written
bare (`tr(IK,P)`); braces spell a branch only. With the internal key extracted
and no path left, the output is `tr(@0/<0;1>/*)` with no tree."* Add both as
vector families (see I10).

### I2 — §5 defines no lowering for the `sh`/`sh(wsh)` wrappers that §4a admits, and §4a conflicts with §5's key-set row at n = 1

**Section and sentences.** §4a: *"`sh(wsh)`, `sh` | ONLY a single path that is an
unlocked, unhashed `sortedmulti` (the Multisig migration, C7); n ≤ 15 for `sh`"*.
§5's lowering table has exactly two columns, **wsh** and **tr**.

**The defect.** The lowering is NORMATIVE and must define an output for every
admitted input; `sh` and `sh(wsh)` are admitted and have no rule. Worse, §4b
permits `n in 1..=9`, so a single-key path under `sh` is composable, and the two
sections then disagree: §4a says the only admitted shape is a `sortedmulti`,
while §5's key-set row says *"one key: `pkh`"*. One implementer emits
`sh(pkh(@0/<0;1>/*))`, another refuses at the picker, a third emits
`sortedmulti(1,@0/…)`. Both wrapper forms encode fine, so nothing downstream
catches the divergence:

```
$ md encode "sh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*))"        # OK
$ md encode "sh(wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*)))"   # OK
```

**Minimal fix.** Either add an `sh`/`sh(wsh)` column to §5 (*"the single path
lowers exactly as the wsh single-path case, wrapped; n = 1 is refused at the
picker"*), or state in §4a that these wrappers require `n ≥ 2` and delegate to
the wsh column. Then give them a vector family.

### I3 — §5b's `md compile` cross-check leg CANNOT RUN for the keyless-wsh family, which §12 item 1 requires it to hold for

**Section and sentence.** §5b: *"For every composable list the emitted template:
parses in its context; passes `sanity_check` (wsh keyless paths admitted via
`ExtParams::top_unsafe()`, review point 8); `lift()`s to the same semantic policy
as `md compile` of the equivalent Concrete policy; and survives `md encode` →
`md decode` byte-identically (C1)."* §12 item 1 then requires *"the §5b
cross-check holds"* for **every** composable shape family, one of which is
`keyless-wsh`.

**Evidence (measured today).**

```
$ md compile "or(pk(@0),and(pk(@1),older(100)))" --context segwitv0
wsh(or_d(pk(@0),and_v(v:pk(@1),older(100))))                       # control: OK
$ md compile "or(pk(@0),and(sha256(a84d…08ad),older(100)))" --context segwitv0
md: compile error: compile: Top Level script is not safe on some spendpath
$ md compile "or(pk(@0),and(sha256(a84d…08ad),older(100)))" --context tap
md: compile error: compile: Top Level script is not safe on some spendpath
$ md compile --help        # options are only --context, --unspendable-key, --json
```

There is no `--experimental` on `md compile` (unlike `md encode`), so the
compiler leg is unreachable for exactly the family the EXPERIMENTAL allowance
exists for.

**Consequence.** A harness written to §5b either skips the keyless family
silently — a gate reporting green over a case it never ran, the repo's blocking
class — or stalls at implementation. This is the "a gate that has never executed
is a hypothesis" rule applied to a spec-level contract.

**Minimal fix.** One sentence in §5b: *"the `md compile` lift-equality leg is
carved out for keyless paths — the compiler refuses any sigless spend path
(measured: `Top Level script is not safe on some spendpath`, both contexts) — and
those vectors keep the parse, `top_unsafe` sanity, and round-trip legs, plus a
lift comparison against a hand-written `Semantic` policy."*

### I4 — §7d has no refusal for two slots resolving to the SAME xpub, which BIP-388 forbids and md refuses only at the very end

**Section and sentence.** §7d: *"Each key is used at most once."*, over *"a pick
list of the REMAINING sources"*.

**The defect.** "Each key is used at most once" dedupes SOURCES, not key
material. Two distinct sources can carry one xpub — a `key:` record and an mk1
card for the same cosigner is the ordinary case when an operator packs both — and
nothing in §7d, §7g or §4e refuses it. BIP-388 l.193 is explicit:

> "The public keys obtained by deserializing elements of the key information
> vector must be pairwise distinct"

and in wsh it is a rust-miniscript sanity failure (`AnalysisError::RepeatedPubkeys`,
reproduced in a scratch crate against the fork). The host refuses it, but only at
encode time, after composition and seating:

```
$ md encode "wsh(sortedmulti(2,@0/48'/0'/0'/2'/<0;1>/*,@1/48'/0'/1'/2'/<0;1>/*))" \
    --key @0=<X> --key @1=<X> --fingerprint @0=73c5da0a --fingerprint @1=73c5da0a
md: codec error: @0 and @1 carry the same key at the same use-site: this policy
names 2 cosigners but one of them holds two of the seats
```

The device builder is NEW code (§9 item 1) and the spec imposes no such check on
it. C29's warning is a different condition (same fingerprint, *distinct*
accounts — which is legal and is C5's normal case).

**Consequence.** Either a late, confusing failure after the operator has done all
the work, or — if the Go builder does not reproduce md's check — an engraved
wallet that is witness-malleable in wsh and invalid per BIP-388.

**Minimal fix.** Add to §7d and to §7g's divergence table: *"two slots resolving
to the same xpub → REFUSE at the mapping review, naming both slots (BIP-388
l.193, pairwise distinct)."*

### I5 — §6a and §9 item 8 name `seal.Classify`; the systemwide payload's classifier is `sysw.Classify`, and `seal/` is FROZEN

**Sections and sentences.** §6a heading: *"Three new payload record classes (host
`me sysw pack` + device `seal.Classify`, lockstep)"*. §9 item 8: *"Three payload
classes in `seal.Classify` (§6a), lockstep with the host."*

**Evidence.**

```
$ grep -rn "func Classify" --include='*.go' /scratch/code/shibboleth/seedhammer
seal/record.go:194:func Classify(b []byte) Classification     # sealed container, []byte
sysw/record.go:100:func Classify(record string) Class         # systemwide payload, string
$ sed -n '14,34p' sysw/record.go
const ( TextPrefix = "text:" ; PassPrefix = "pass:" ; TxPrefix = "tx:" )
type Class int
const ( ClassUnknown Class = iota ; ClassMnemonic ; … )
```

`sysw.Class` is its OWN type (`sysw/record.go:24`), not an alias of `seal.Class`,
and `gui/sysw_admit.go` is written against `sysw.Class`. `seal/` is frozen by the
payload spec's decision 1 — `SPEC_systemwide_payloads.md` §13 D8: *"decision 1
freezes `seal/` outright"*, with a visibility-only carve-out that explicitly bars
touching a body, a signature or an output.

**Consequence.** A work item that, followed literally, edits a frozen module and
adds classes the systemwide classifier would never see. It also mis-states where
the host/device lockstep actually binds (`sysw/record.go` ⟷
`crates/me-cli/src/sysw/record.rs`).

**Minimal fix.** Replace `seal.Classify` with `sysw.Classify`
(`sysw/record.go:100`) in §6a and §9 item 8, and note the three reserved prefixes
that exist today (`text:`, `pass:`, `tx:` — `sysw/record.go:15-21`) so the new
ones join a list rather than invent one.

### I6 — §6a edits a row that does not exist: `SPEC_systemwide_payloads.md` §3.3.2 has NO Wallet Policy row (known gap, F-415)

**Section and sentence.** §6a: *"The admission table
(`SPEC_systemwide_payloads.md` section 3.3.2) gains three columns and Wallet
Policy's row becomes: Mnem •, Cdx32 •, Passph •, Descr •, MDMK •, Key •, Hash •,
Now •."*

**Evidence.**

```
$ grep -c "Wallet Policy" design/SPEC_systemwide_payloads.md
0
```

§3.3.2's table runs Backup Wallet, BIP-39 Password, Engrave Text, Account Xpub,
Engrave Bundle, Engrave Single-Sig, Engrave Multisig, BIP-85, *Sealed Payload* —
no Wallet Policy and no Engrave Transaction, both of which exist in
`gui/sysw_admit.go`. This is already recorded by a sibling spec,
`SPEC_descriptor_input.md` §2.3: *"**It has no Wallet Policy row at all**, so
`progWalletPolicy`'s `ClassDescriptor` cell is code-only drift with no normative
source. Reconciling that table is `SPEC_systemwide_payloads`' own change
(F-415)."*

**Consequence.** The one instruction that makes the host/device admission change
reviewable cannot be transcribed, and the payload spec's own invariant — *"the
table is the normative RECORD"*, reconciled by a structural test against every
call site — silently does not cover this program.

**Minimal fix.** Say so: *"§3.3.2 has no Wallet Policy row today (F-415, drift
recorded in `SPEC_descriptor_input.md` §2.3). This cycle CREATES it with the
cells below, and adds the three class rows to §3.3.1 with `secret? = no`."*

### I7 — §4f's citation is the LOCKED shared origin, not the per-slot account the row claims; reusing it gives one seed the SAME xpub at two slots

**Section and sentence.** §4f: *"| `wsh`, `sh(wsh)`, `sh` | `m/48'/coin'/account'/2'`
(unchanged, `gui/multisig_build.go:1359`) | by ordinal among the slots that master
fills (C5/C12) |"*.

**Evidence.** `gui/multisig_build.go:1359` is `func multisigSharedOrigin()`,
commented as the **LOCKED** shared origin for `OriginShared` mode, returning a
fixed `m/48'/0'/0'/2'` — coin and account hard-coded to `0'`. The
ordinal-by-master machinery is a *different* code path at `:594-601` (*"A held
slot's account is its ordinal among the held slots that share a … 1, @2 = B
account 0."*), which the row also cites in its third column.

**Consequence.** An implementer reading "unchanged, `:1359`" reuses
`multisigSharedOrigin()` and every seed-derived slot gets the same
`m/48'/0'/0'/2'` — hence the same xpub at two slots, which is precisely the
BIP-388 violation of I4, and it dissolves C5's entire "reuse the MASTER via
distinct hardened accounts" mechanism. The `coin'` degree of freedom in the row
has no source in the fork either (the shipped path hard-codes `0'`).

**Minimal fix.** Cite `:594-601` for the account rule and `:1359` only for the
`2'` script-type constant, and say plainly that the composer uses the per-slot
account path, not `OriginShared`. State what supplies `coin'` (or drop it to
`0'`).

### I8 — §10 item 2's automatic `now:` contradicts C24 and is unscoped: unconditional? at which record index? still reproducible?

**Sections and sentences.** §10 item 2: *"`me sysw pack`: `key:`, `hash:`, `now:`
classes (§6a); `now:` written automatically at pack time."* §6a: *"written by `me
sysw pack` at pack time"*. C24, same paragraph as the shape: *"The record is
operator-authored and affects ONLY echoes and refusals, never the encoded
operand."*

**The defect.** Operator-authored and written-automatically are different
behaviours, and three things hang on which one is meant:

1. **Scope.** Every `me sysw pack` invocation, or only when composer records are
   present, or behind a flag? A transaction payload gaining a timestamp is a
   change to a shipped, unrelated flow.
2. **Position.** `me sysw pack --in`'s own help makes record index load-bearing:
   *"a record's index is its position among the NON-blank lines"*, and the
   device's plate identity is index-derived (`cardKey` returns `uniq: i + 1`,
   payload spec §5.3.2). Injecting a record shifts every index after it.
   Unspecified whether `now:` goes first or last.
3. **Reproducibility.** `me`'s pack API is explicitly split into `pack` and
   `pack_deterministic{,_with}` (`crates/me-cli/src/sysw/mod.rs:279-328`), where
   the ONLY nondeterminism is the injected salt/IV and the records are a
   parameter. An automatic timestamp puts nondeterminism *into the record list*,
   where the deterministic entry point cannot reach it — so two packs of one
   input file stop producing one blob, and the digest an operator compares under
   §5.4 differs run to run.

**Minimal fix.** State the ruling: e.g. *"`me sysw pack` appends `now:` as the
LAST record when `--now` is given (default on for interactive packs, off for
`pack_deterministic`); a fixture supplies it explicitly, so pack output stays a
pure function of its inputs."* Any concrete answer will do; the spec must pick
one.

### I9 — §6b's below-`now:` refusal is undefined when `now:` carries no block height

**Section and sentence.** §6a: *"`now:<hex>` | hex of the UTF-8 text
`<unix-seconds>[,<block-height>]`"* — the height is optional. §6b: *"A date or
height BELOW the `now:` value → "That is before this payload was packed. Choose a
later date.""*

**The defect.** The ECHO row is guarded (*"+ lower-bound line if `now:` carries a
height"*); the REFUSAL is not. With a seconds-only `now:`, one implementer
applies no height refusal, another estimates a height from the timestamp and
refuses legitimate heights (or, converting the other way, refuses legitimate
dates). Both readings are available from the text, and the two devices refuse
different operator inputs.

**Minimal fix.** *"A height is compared only against `now:`'s height field and a
date only against its seconds field; when the field is absent, no refusal
fires."*

### I10 — §12 item 1's vector families do not cover four branches §4/§5 make normative

**Section and sentence.** §12 item 1: *"Every composable shape family
(single/multi keys × none/lock/hash × wsh/tr × sorted/unsorted × keyless-wsh) has
a Rust vector…"*

**Uncovered branches, each normative elsewhere:**

| branch | where it is normative | why the axis list misses it |
| --- | --- | --- |
| `sh` / `sh(wsh)` wrappers | §4a | the wrapper axis is "wsh/tr" only (see I2) |
| internal key extracted vs NUMS | §5 `internal key` row | no axis; §10 item 1 asks for one such vector, but §10 is a work list, not the acceptance |
| `or_d` head vs `or_i` head | §5 `paths combine` (C21/C23, the most-argued ruling of the cycle) | no axis; "multi keys" alone does not force a bare unlocked multi head |
| path count / spine depth | §5 tr spine, §4b's 1..8 | no axis; a 1-path and an 8-path policy are one "family" (and see I1's degenerate counts) |
| the four lock encodings of §4c | §4c, §6b | "none/lock/hash" collapses blocks, 512-s units, height and time into one value — the exact axis C20 exists to pin |

**Consequence.** The acceptance can pass with the lowering's most contested rules
untested, and §12 item 1 is the only gate that pins Rust↔Go byte identity.

**Minimal fix.** Extend the product with `wrapper ∈ {wsh, tr, sh, sh(wsh)}`,
`head ∈ {bare-multi, single-key, locked}`, `internal-key ∈ {extracted, NUMS,
none}`, `paths ∈ {1, 2, ≥3}`, `lock ∈ {none, blocks, 512s, height, time}` — or
name the branches explicitly as a list of required vectors.

### I11 — §8a/§8b give the EXPERIMENTAL copy but never state when it fires or that it cannot be skipped, which C16 requires

**Sections.** §8a (keyless path) and §8b (unsorted keys) are copy blocks under a
heading that says only *"Copy — operator-facing strings"*. C16's ruling is
narrower than that: *"(1) KEYLESS hashlock-only paths ADMITTED behind an
**unskippable** EXPERIMENTAL screen naming bearer access; … (3) … unsorted
`multi`/`multi_a` PERMITTED under the same EXPERIMENTAL warning"*.

**The defect, two readings each.**
*Unskippable* is dropped everywhere in the spec — §4b marks the path
"EXPERIMENTAL", §7g's divergence table does not list either screen, and §11's
refusal roll-up covers refusals only. One implementer gates the path behind a
confirm; another shows a dismissible banner.
*Trigger* is unstated for §8b: §5's row is *"unsorted where sorted was legal"*,
but the lowering FORCES `multi`/`multi_a` for every locked or hashed multi-key
path (brainstorm §3.8, settled). So does the screen fire on every emitted
`multi`, or only when the operator declined a legal `sortedmulti`? The first
reading fires it on most wallets and trains operators to dismiss it.

**Minimal fix.** One line in §7b or §8: *"§8a fires once per keyless path and §8b
once per key set where `sortedmulti`/`sortedmulti_a` was legal and declined;
both are unskippable confirms (C16), and neither fires on a `multi` the lowering
forced."*

---

## MINOR

**M1 — §3's keyless-tap-leaf row is incomplete and cites the wrong section.**
> `| md admission: keyless tap leaf | refused ("All spend paths must require a signature") | brainstorm record section 3.7, review I2 |`

md refuses it *by default* and **admits it with `--experimental`** — there is a
dedicated md-cli test for exactly this
(`crates/md-cli/tests/cmd_encode.rs:875` `experimental_admits_a_keyless_spend_path`),
and I reproduced both halves. And brainstorm §3.7 is the *`md compile` is a
validity oracle* section; it contains no keyless measurement (the fact comes from
the fable review's I2). Fix: *"refused by default; admitted by `md encode
--experimental`"*, and cite the review only.

**M2 — §4e's refusal copy asserts a false capability.** *"Taproot cannot hold a
key-less path. Use wsh, or add a key."* Taproot can: the leaf is well-formed
script, md encodes it under `--experimental`, and the refusal is a composer
ruling (I2/C19) sitting on top of rust-miniscript's per-leaf sanity policy —
`Descriptor::from_str` gates `tr` and not `wsh`, which is why the two contexts
differ at all. Suggest: *"This build will not put a key-less path in taproot. Use
wsh, or add a key."*

**M3 — §4a's `n ≤ 15 for sh` is unreachable and mis-attributed.** §4b caps `n` at
9, so the bound never binds. Review point 7 also measured that the binding limit
arrives earlier and elsewhere: `sh(sortedmulti)` with 16 keys fails on the
520-byte redeemScript (*"cannot be larger than 520 bytes, but got 547 bytes"*),
with `MAX_P2SH_SIGOPS{15}` at `policy.h:42` (not `script.h`). Keep it as a
footnote or drop it.

**M4 — §12 item 4's premise does not exist in this spec.** *"the `sortedmulti`
preset with seed-derived slots reproduces `gui/testdata/t6b_multisig_full.md1.txt`"*
— §4d's five presets are the toolkit archetypes and none is a plain
`sortedmulti`. C7 made this check conditional (*"if the composer ever ships a
`sortedmulti` preset"*). Either add the preset to §4d or restate the condition.
(The testdata file does exist.)

**M5 — two §3 citations do not carry their claim.**
`gui/multisig_build_census.go:475` is `return "Full (seed + keys, NOT passphrase)"`
— it supports the Full half only; the `"Watch-only (keys)"` literal is
`gui/multisig_build.go:455`. And `gui/wallet_policy.go:35-142` stops before the
consent content it describes: `walletPolicyConsentLines` (id line, `md1Summary`,
`walletPolicyAddressLines`) runs ~148-232.

**M6 — §5's tr column contradicts itself in two adjacent rows.** *"one leaf per
path"* vs the internal-key row's *"(then not a leaf)"*. Fix: *"one leaf per path,
except the path extracted as the internal key."*

**M7 — §6b's refusal names the wrong unit on the height row.** *"That is before
this payload was packed. Choose a later date."* is shown for a below-`now:`
**height** too. §11 promises every refusal names what to do instead.

**M8 — §7f's "Read-back integrity" cannot mean read-back.** The SH2 has no
camera and can never read a plate back; the property is recovery-time error
detection (md1/mk1 BCH vs a BIP-380 checksum). Rename to avoid implying a
device capability that does not and will not exist.

**M9 — §6a's `hash:` body is hex of BYTES while `key:`/`now:` are hex of TEXT.**
The row is internally clear (*"the 32-byte digest itself, 64 lowercase hex"*),
but the section's opening rule is *"a lowercase-hex body"* modelled on
`text:`/`pass:`, which are hex of UTF-8. One clause prevents a double-encoded
128-char body.

**M10 — §6a names §3.3.2 but not §3.3.1.** The payload spec's class table
(§3.3.1: class / secret? / source) needs three new rows with `secret? = no`;
§6a's *"None is secret"* implies it without naming the edit target.

---

## NIT

**N1 — `n` is overloaded in §5's depth formula.** *"path k at depth min(k, n−1)"*
uses `n` for the leaf count, while §4b defines `n` as the key count (*"n in
1..=9"*). The formula is correct with `k` and `n` leaf-indexed (verified against
the right spine for 2, 3 and 4 leaves); rename to `L`/`i`.

**N2 — head-pin drift.** The header pins descriptor-mnemonic `790fc224`; the
local tree is at `3b0944fb` (the pinned commit exists). All md measurements in
this report were made with the installed `md 0.14.0`, which is not provably built
from the pinned rev — worth stating in the spec's header which binary the
measurements used.

---

## CHECKED AND CLEAN — do not re-derive next round

1. **All 30 `file:line` citations resolve and, apart from M5 and I7, say what the
   spec claims.** Fork is at `169073c` with a clean tree, matching the header.
   Verified individually: `wallet_policy.go:97` (no-payload really does fall
   through to the NFC gather with no screen), `:194` vs `template_engrave.go:70`
   (both print the literal label `Template-ID`, 16 bytes vs 4 — §7c's ambiguity
   claim is exact), `key_card_seating.go:53` (declaration match, all-or-nothing,
   `errSeatSlotUnfilled`), `encode.go:159/374/461` (`writeNode`,
   `encodePayload`, `encodeMD1String`), `md1_expand.go:149` (only bare `*` and
   `<a;a+1>/*`), `template_id.go:122/163` (`[4]byte` / `[16]byte` + kind),
   `multisig_build.go:594-601` and `:738`, `transaction.go:1145` and `:1369`
   (ceiling by binary search, *"MEASURED BY SEARCH, not written down"*),
   `passphrase_keyboard.go:21`, `derive_xpub.go:104`, `gui.go:1262/2856`,
   `mk/encode.go:39` (deterministic; `Stubs [][4]byte // len >= 1`),
   `tapleaves.go:188` (calls `emitFragment`), `sysw/wire.go:28`
   (`RegionLen = 65536` = 64 KiB), `gui/sysw_admit.go` (Descriptor + MDMK only),
   `gui.go:191` (the enum comment does carry *"came from OUTSIDE this device"*
   and *"needs neither a seed requirement"*). `gui/raster_test.go` and
   `gui/testdata/t6b_multisig_full.md1.txt` both exist. `txqr.MaxSymbols = 16`
   (`txqr/txqr.go:38`).
2. **Absence claims in §3 hold, with a positive control.** `grep -c tagAndOr
   md/script_emit.go` = 0, `grep -c tagPkH` = 0, control `grep -c tagOrI` = 1.
   (Note for future greps: `tagPkh` 0x04 is a different, existing tag from
   `tagPkH` 0x0B — the spec cites the fragment tag correctly.)
3. **Every BIP line number in §4b/§4c is right on the line** (fetched from
   bitcoin/bips master today): BIP-68 l.30 (bit 31 disable), l.36 (bit 22 units),
   l.40 (mask `0x0000ffff`), l.46 (*"A relative time-based lock-time of zero
   indicates an input which can be included in any block"*), l.74-83 (the C++
   constants); BIP-112 l.28-33; BIP-65 l.27 and the `LOCKTIME_THRESHOLD` block at
   l.241-250; BIP-379 l.135 (*"`older(n)`, `after(n)` | 1 ≤ n < 2^31"*); BIP-341
   l.157 (NUMS `H`, and the "everyone agrees" alternative C20 cites); BIP-388
   l.191 (*"A wallet policy must have at least one key placeholder"*), l.193
   (pairwise distinct), l.199 (first-appearance ordering), l.305-309 (the invalid
   examples). **§5's NUMS hex matches BIP-341 l.157 byte for byte.**
4. **rust-miniscript-fork `src/miniscript/limits.rs:35` = `MAX_PUBKEYS_PER_MULTISIG:
   usize = 20` and `:38` = `MAX_PUBKEYS_IN_CHECKSIGADD: usize = 999`** — §4b's
   cite is exact. `src/primitives/absolute_locktime.rs:10` =
   `MAX_ABSOLUTE_LOCKTIME: u32 = 0x7FFF_FFFF` — §4c's cite is exact.
5. **§5b's `ExtParams::top_unsafe()` claim is exactly right**, measured in a
   scratch crate against the fork: a wsh keyless path fails `sanity_check()` with
   `Err(SiglessBranch)` and passes `ext_check(&ExtParams::new().top_unsafe())`;
   `Descriptor::from_str` sanity-gates `tr` only (`Err("All spend paths must
   require a signature")`) and admits the same shape inside `wsh` — which is why
   md needs no flag for keyless wsh and refuses keyless tr. `md encode
   --experimental` relaxes **only** `top_unsafe` (`parse/template.rs:2642-2700`).
6. **The placeholder-numbering rule IS gated by §5b's round trip** — this was my
   main suspicion and it is unfounded. md1 carries placeholders positionally:
   `tr(@3,{multi_a(2,@0,@1),and_v(v:pk(@2),older(100))})` and its first-appearance
   spelling encode to the SAME md1, and `md decode` returns the renumbered form.
   So "survives `md encode` → `md decode` byte-identically" fails on a
   misnumbering. (md itself enforces only density: `@0,@2` → *"@1 not present;
   placeholders must be dense 0..n"*; order is not enforced.)
7. **C9/§7c's "the template stub is final before any seating" is TRUE, measured.**
   `wallet-descriptor-template-id` = `aad0e0e0718cbe91da67cc2bd72c68c9` for all of:
   no origins, origins `0'`/`1'`, origins `5'`/`7'`, `--path m/48h/0h/0h/2h`,
   `--path m/48h/0h/0h/3h` — while `wallet-policy-id` changes with every one
   (`465a5fc1…`, `cc13725a…`, `4e7e4af3…`, `f8d855b0…`). Origin-invariant and
   key-independent, exactly as claimed.
8. **§5's combine and key-set rows match both persisted reviews verbatim** —
   `or_d(P,R)` iff `P` is a bare unlocked unhashed `multi` with n ≥ 2, otherwise
   `or_i`, recursive at every level, single key `or_i(pkh(K),R)` (I1 review §4
   minimal rule text; fable review §3 corrected table). §5a's WU figures
   (+34/−26/−2/+k, break-even 27.8 %) match the I1 review's measured table and
   were not re-derived. C18's raw-`H` decision and C20's M1 fallback are folded
   as the reviews and rulings state.
9. **Timelock mixing cannot fire in this grammar** — one lock per path (§4b) and
   paths combine by disjunction, so `HeightTimelockCombination` is unreachable;
   measured `Ok(())` for `or_i` of an `older` path with an `after` path, and for
   blocks-vs-512-s relative locks in the two arms.
10. **§4c's ranges are what md enforces** (probed): `older(65535)` OK,
    `older(65536)` refused naming the mask, `older(4194304)` (= `0x400000`)
    **ACCEPTED** — the filed `md-older-zero-time-units-not-refused` defect,
    reproduced, exactly as §3 and §4c say; `older(4194305)` and `older(4259839)`
    OK; `after(1)`, `after(500000000)`, `after(2147483647)` OK; `after(2^31)`
    refused by miniscript's ceiling.
11. **§6b's arithmetic recomputes.** 388 d → `ceil(388·86400/512)` = 65475 ≤
    65535; 389 d → 65644, correctly out of range; 65535 blocks = 455.1 d; 65535
    units = 388.4 d. §8c's *"90 days = 15188 units of 512 s (90.0 days)"* is
    exact (7,776,000/512 = 15187.5 → 15188), and 2026-09-01 → 2027-03-01 is
    exactly 181 days.
12. **§6a's hex-body requirement is load-bearing and correctly applied.** The
    payload spec §5.3.1 hex-encodes because EPD §6.6 hashes *lowercased* records
    and §6.4 bans interior spaces; an xpub is case-sensitive base58, so a raw
    `key:[fp/path]xpub` record would not survive canonicalisation. The
    classification-order and reserved-prefix rules in §6a are transcribed
    correctly from §5.3.1.
13. **§10 item 5 fits the existing surface.** `ms derive --template` today offers
    `bip44|bip49|bip84|bip86|bip48-p2wsh|bip48-p2sh-p2wsh|bip48` (ms 0.16.0), so
    `bip48-p2tr` matches the convention; the toolkit's `bip48-tr-multi-a` at
    script type `3'` exists (`crates/mnemonic-toolkit/src/cmd/xpub_search/candidate_paths.rs:85`),
    as §4f claims.
14. **Every other composer-emitted shape probed encodes on md unchanged**:
    `wsh(pkh(@0))`, `wsh(sortedmulti(2,…))`, `wsh(or_d(multi(2,…),and_v(v:pkh,older)))`,
    the 3-path `or_i` chain, `and_v(v:multi,and_v(v:sha256,older))`,
    `wsh(or_i(pkh,and_v(v:sha256,older)))`, `wsh(or_i(pkh,sha256))`,
    `tr(@0)`, `tr(@0,and_v(v:pk(@1),older(100)))`, 2- and 3-leaf spines,
    `sh(sortedmulti)`, `sh(wsh(sortedmulti))`. `wsh(and_v(v:sha256,older))` alone
    is refused (*"template contains no @i placeholders"*), matching §4b's I4.

---

## WHAT I RAN

- `CITE_FORK_ROOT=/scratch/code/shibboleth/seedhammer ./scripts/plan-cite-check.sh
  design/SPEC_wallet_policy_composer.md` → 30/30, then read every printed line
  against its claim (the gate proves existence only).
- Read end to end: the spec; `BRAINSTORM_wallet_policy_composer.md` §2 (C1-C29),
  §3.3-§3.11; both persisted lowering reviews (corrected tables, points 1-10);
  `SPEC_systemwide_payloads.md` §3.3, §5.3, §13 D8; `SPEC_descriptor_input.md`
  §2.3.
- Fetched from `bitcoin/bips` master: bip-0065, 0068, 0112, 0341, 0379, 0386,
  0387, 0388; checked every cited line number by `sed -n`.
- `md 0.14.0` (by path) — 39 `md encode` probes across wrapper/head/lock/keyless/
  spine/numbering shapes; `md compile` in both contexts with a keyed control;
  `md inspect` for the template-id invariance matrix; `md decode` for the
  renumbering round trip. Every result quoted above is from these runs.
- A scratch crate (`/tmp/.../scratchpad/mscheck`) against
  `rust-miniscript-fork`: `sanity_check()` vs `ext_check(ExtParams::new().top_unsafe())`
  on seven composer shapes, plus `Descriptor::from_str` in both contexts.
- Fork greps for `func Classify`, prefixes, `MaxSymbols`, `buildMultisigSeedHook`,
  and the tag-emitter counts with a positive control; `git log`/`git status` on
  the fork and descriptor-mnemonic to confirm the header's heads.
- One sonnet subagent, read-only, returned an inline table verifying 16 fork
  citations on the line (its two PARTIAL verdicts are I7 and M5; a third became
  M5's second half). No repo file was written by it or by me except this report.
