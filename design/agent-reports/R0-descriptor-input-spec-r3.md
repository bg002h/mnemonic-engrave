# R0 review — `SPEC_descriptor_input.md`, round 3 (proportional re-review of the r2 fold)

**Artifact:** `design/SPEC_descriptor_input.md` at `586966f` (1256 lines).
**Scope, as briefed:** (1) did the fold close each of r2's ten findings; (2) did the fold
introduce new defects — **only in what changed**. Not a fresh audit. r1's verified-TRUE table,
r1/r2's measured probe results, the citation gate, §8's phase order and the operator rulings
were taken as settled and were not re-derived.
**Reviewer:** independent agent, opus tier. Read-only; nothing in any repo was modified
(`seedhammer` `main` = `origin/main` = `d402f18f6a8c…`, `git status` clean before and after).

## Counts — NEW findings only

**1 Critical / 2 Important / 3 Minor / 2 Nit**

**Disposition of r2: 10 FIXED, 0 PARTIAL, 0 NOT FIXED.** Each verdict is by re-running or
re-tracing r2's own constructed failure against the text as it now stands, not by locating the
edit. **This lens does not close.** The Critical is adjacent to the fold's central new object —
conjunct 7 is written **per key** while §5.3's three md1 rules are written **per descriptor**,
and the gap between those two quantifiers is a silently different engraved wallet.

**Measurement environment.** Go probes: scratch module at
`…/scratchpad/goprobe3` with `replace seedhammer.com => /scratch/code/shibboleth/seedhammer`,
built with `/nix/store/33fw5m31lfcnk4ff2f0df7j2bxnh8lgk-go-1.26.3/bin/go`; the fork tree was
never written to. md1 probes: `/scratch/code/shibboleth/descriptor-mnemonic/target/release/md`
(the tree-built binary, per §2's stale-binary note). CLI probes: `/home/bcg/.cargo/bin/me`
(`me 0.7.0`). Keys are the fork's own `nonstandard/parse_test.go` fixtures; SLIP-132 variants
were produced by re-serialising the fixture's 74-byte payload under new version bytes.

---

## Disposition table — r2's ten findings

| # | r2 finding | verdict | where the fold lands, and what re-running it shows |
| --- | --- | :-: | --- |
| NEW-C1 | `<0;1>` no-wildcard admitted, md1 engraves a different wallet | **FIXED** | §5.3(a″) (line 846) refuses it under `--as md1`; §5.5 row; §6 row; §7 row. Re-measured both halves of r2's construction: `md encode 'wsh(sortedmulti(2,@0/<0;1>,@1/<0;1>))'` and the `…/*` twin both give chunk-set-id `0x16d62`, `md decode` returns the `/*` form, `md address` → `bc1qadgf37z…`; the device derives `bc1qu2cc6t7…` for `<0;1>` (recv0=recv1=recv5 — no wildcard, so `index` is never consumed, `address/address.go:217`). Both numbers in (a″) are TRUE, and the shape is now refused on the md1 path. |
| NEW-I1 | `<0;1>/*h` and `<0;2>/*` admitted and broken | **FIXED** | §4.7 conjunct 7 (line 625) refuses hardened components and non-consecutive pairs on **both** `--as` paths; §6 rows (984, 985); §7 narrowed-shape rows. Re-measured a second member of the non-consecutive class, `<0;0>/*`: device ACCEPT, canonical a fixed point, `Supported=true`, every `Receive`/`Change` errors `address: unsupported range path element` — the same trap, now inside the refused set. |
| NEW-I2 | conjunct 3 mis-applied BIP-383's 15-key cap to `sh(wsh(…))` | **FIXED** | Conjunct 3 (line 604) keys on *"the `sortedmulti` is the DIRECT argument of `sh(…)`"*, `n ≤ 20` for `wsh`/`sh(wsh)`; §6's row splits the two reasons and states the 34-byte redeemScript; §7 moves `sh(wsh(sortedmulti(2,16)))` to the **accepted** side. The refusal r2 constructed no longer fires and the false reason is gone. (A superseded parenthetical survives at line 437 — new NEW-M3, not a PARTIAL: it is a different sentence in a descriptive table, and the NORMATIVE rule is unambiguous.) |
| NEW-I3 | SLIP-132 remedy wrong for 4 of 5 versions; placeholder for a bare key | **FIXED** | §6's row (line 973) names per-version targets and scripts and gives the bare key an origin-less spelling. **Every remedy re-measured, not read:** bare `vpub`/`upub`/`ypub` all REFUSE; the printed remedies `wpkh(tpub…/<0;1>/*)` → ACCEPT `tb1q5hrky4qgk2yqdg334z0g0r4348jvuj8ka9zgdm`, `sh(wpkh(tpub…/<0;1>/*))` → ACCEPT `2NFiZesRXXgQz4iAT9bMbosxvVsbSdphSPL`, `sh(wpkh(xpub…/<0;1>/*))` → ACCEPT `3QAMb8VVvDudrvXuUTjjBvyfHXPGtDyGJ4`. Network and script are right in all three, and the control confirms the row's reason: a bare converted `tpub` promotes to `pkh(tpub…)` → `mvdWe9zQJgPAqUuGjhwvKh5kxfAX9iT59J`, a different wallet. |
| NEW-I4 | five narrowed shapes had no §6 row; §4.7's "§7 gives them rows" was false | **FIXED** | Two §6 rows added (982, 983); `wsh(KEY)`/`sh(KEY)` added to §7's narrowed-shape bullet (line 1080); §4.7's sentence now says MEASURED and cites the measurement. The false cross-reference is gone. (The rows' markdown is broken — NEW-M1 — but they exist and are specified.) |
| NEW-I5 | dead end: `--as` named, then a refusal denying the descriptor's presence | **FIXED** | §5.1's first boundary bullet (726) plus §6's row (988): the refusal names the split (`pack the descriptor alone`, F-414) instead of naming `--as`, at `EXIT_INVALID (4)`. r2's three-step journey no longer reaches its false second message. (The rule over-captures a class it should not — new NEW-I2.) |
| NEW-M1 | zero-fingerprint cosigner loses its origin silently | **FIXED** | §4.2 gains the warning paragraph (line 366). Re-measured on a format-2 twin: `[00000000/48h/0h/0h/2h]` parses, the canonical drops the whole `[…]` block, re-parses as a fixed point, addresses unchanged — the harm is exactly as the paragraph states. (The warning is scoped to BlueWallet only — new NEW-M2.) |
| NEW-M2 | §5.6's `--in` contract amended with no cross-document note | **FIXED** | §5.1's second boundary bullet (732) states the amendment; **F-416** exists in `design/FOLLOWUPS.md` with an owning phase ("descriptor-input cycle, at ship"), same shape as F-415. |
| NEW-N1 | the five-step cause rule ranks only parse failures | **FIXED** | The paragraph at line 949 says so and names the other two sources (§4.7's predicate, §5.3's limits). |
| NEW-N2 | §5.4's bullet had not followed I5's fold | **FIXED** | §5.4's bullet now carries `key as supplied` AND `inferred wallet` with the normalisation named, and says explicitly that the two sections describe one stderr block. |

---

## The deliberate deviation from r2's prescription — verdict

**The split is correct, and r2's global prescription would have been wrong.** Attacked as
briefed, member by member of the globally-admitted set `{absent, /*, /i/*, <i;i+1>, <i;i+1>/*}`:

| member | device (`nonstandard.OutputDescriptor` + `address.Receive`/`Change`) | verdict |
| --- | --- | :-: |
| absent | ACCEPT, fixed point, `<0;1>/*` defaulted at `address.go:190–202`, recv0 `bc1qadgf37z…` | sound |
| `/*` | ACCEPT, fixed point, derives (recv0 `bc1qghwumhc…` in the 2-key fixture) | sound |
| `/0/*` | ACCEPT, fixed point, recv0 `bc1qadgf37z…`, recv1 `bc1q9edtz99…`, chg0 = recv0 | sound |
| `/1/*` | ACCEPT, fixed point, recv0 `bc1qadfu2ea…` | sound |
| `<0;1>` | ACCEPT, fixed point, recv0=recv1=recv5 `bc1qu2cc6t7…`, chg0=chg1 `bc1qrranhxp…` | sound |
| `<1;2>` /`*` | ACCEPT, fixed point, recv0 `bc1qadfu2ea…`, chg0 `bc1qls0jxmm…` | sound |
| `<0;1>/*` | ACCEPT, fixed point, recv0 `bc1qadgf37z…` | sound |

No member of the admitted set is broken on the device. Both no-wildcard members (`<0;1>`, and
`/i` which conjunct 7 refuses anyway) derive a **single** address that `Receive` returns for every
index — which is what a wildcard-free descriptor means, not a defect, and `--as descriptor`
carries the string exactly (canonical is a fixed point in every row above).

**The layering is right too.** The two device-broken classes are refused globally by conjunct 7
(a hardened step displays a wallet that cannot exist; a non-consecutive pair reaches
address-deriving programs that error), and the two md1-lossy classes are refused only under
`--as md1` by §5.3(a)/(a″) — which is the only placement that does not contradict §5.3(a)'s own
preserved path. §5.5, §6 and §7 agree with the split everywhere I checked: §5.5's two `❌ md1`
rows cite (a) and (a″); §6's md1 rows are both scoped to `--as md1`; §6's closed-set row lists
exactly conjunct 7's five members; §7's narrowed-shape bullet carries `<0;1>/*h` and `<0;2>/*`
and its md1-splits bullet carries `/0/*` and `<0;1>`.

**What the split did not close is the quantifier.** That is NEW-C1 below.

---

## NEW — Critical

### NEW-C1 — conjunct 7 is stated PER KEY; §5.3's three md1 rules are stated PER DESCRIPTOR. A descriptor whose keys carry *different* admitted use-site paths is admitted, and `--as md1` packs a DIFFERENT wallet — three measured cases

**Where the fold left it.** Conjunct 7 (line 625) quantifies over keys: *"**each key's children
expression** is one of `{absent, /*, /i/*, <i;i+1>, <i;i+1>/*}`"*. The three NORMATIVE md1 rules
that decide what happens next quantify over the descriptor:

- line 818 — *"`--as md1` REFUSES **a descriptor whose use-site path is** a single fixed child index"*
- line 834 — *"when **the parsed descriptor's use-site path is** ABSENT, `--as md1` materialises `<0;1>/*`"*
- line 854 — *"`--as md1` REFUSES **a descriptor whose use-site path is** a multipath group without a trailing wildcard"*

A descriptor that mixes two admitted members has no single "use-site path", so none of the three
rules fires by its own terms, while conjunct 7 admits it key by key. Nothing else in §4.7
requires the keys to agree. **The fold created this**: before it, §4.7 said nothing about
use-site paths at all; the new conjunct is the first per-key statement on the axis, and it was
placed alongside per-descriptor md1 rules that were never re-quantified.

**Constructed — three cases, both routes measured.** Fixture keys `K1`/`K2` are the fork's own
`nonstandard/parse_test.go` cosigners with origins `[dc567276/48h/0h/0h/2h]` / `[f245ae38/48h/0h/0h/2h]`.

```
B1   wsh(sortedmulti(2, K1/0/*   , K2/<0;1>/* ))     -- conjunct 7: /i/* ok, <i;i+1>/* ok  => ADMITTED
     device : ACCEPT, canonical a fixed point, recv0 bc1qadgf37zk0wtu69j7yclswl99e5jmcv5eh69jp4calgs4ehsecm5sqcjw5a
     md1    : chunk-set-id 0x9dcdb ; md decode -> wsh(sortedmulti(2,@0/*,@1/<0;1>/*))   <-- the /0 is GONE
              md address              -> bc1qghwumhcahkfca7qktym7f3htf5wqakz2tyvxraf3fk5k8w0yrzwsg0m3sd

B2   wsh(sortedmulti(2, K1        , K2/<0;1>/* ))     -- conjunct 7: absent ok, <i;i+1>/* ok => ADMITTED
     device : ACCEPT, fixed point,      recv0 bc1qadgf37zk0wtu69j7yclswl99e5jmcv5eh69jp4calgs4ehsecm5sqcjw5a
     md1    : chunk-set-id 0x9dcdb ; md decode -> wsh(sortedmulti(2,@0/*,@1/<0;1>/*))   <-- NOT materialised
              md address              -> bc1qghwumhcahkfca7qktym7f3htf5wqakz2tyvxraf3fk5k8w0yrzwsg0m3sd

B4   wsh(sortedmulti(2, K1/<0;1> , K2/<0;1>/* ))     -- conjunct 7: <i;i+1> ok, <i;i+1>/* ok => ADMITTED
     device : ACCEPT, fixed point,      recv0 bc1qghwumhcahkfca7qktym7f3htf5wqakz2tyvxraf3fk5k8w0yrzwsg0m3sd
     md1    : chunk-set-id 0x16d62 ; md decode -> wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*))
              md address              -> bc1qadgf37zk0wtu69j7yclswl99e5jmcv5eh69jp4calgs4ehsecm5sqcjw5a
```

Every one of the three engraves a card set for a wallet the operator does not hold. B1 and B4
have their addresses **exactly swapped** between the two routes. B1's card set is
byte-identical (same chunk-set-id) to the card set for `K1/*`, a descriptor the operator never
typed.

**B2 is the sharpest, because it is (a′)'s own case.** §5.3(a′) is NORMATIVE that md1
materialises `<0;1>/*` for an absent path — implemented **per key** it builds A2's template
(chunk-set-id `0x16d62`, address `bc1qadgf37z…`, matching the device exactly, measured);
implemented **per descriptor** as written, it does not fire at all, and md1's own collapse
turns the childless key into `/*`. One quantifier decides which wallet gets engraved.

**md1 is not the limitation — the spec is.** `md_codec` carries per-`@N` divergence natively:
`TLV_USE_SITE_PATH_OVERRIDES = 0x00`, *"per-`@N` divergent path declarations"*
(`crates/md-codec/src/tlv.rs:10`), and §5.3(b) already cites those fields at line 859. Measured
proof it works: the mixture whose members are **both** md1-representable —
`wsh(sortedmulti(2,K1/*,K2/<0;1>/*))` — round-trips faithfully (`md decode` returns the same
template, `md address` `bc1qghwumhc…` = the device's `Receive`). So the correct rule is a
per-key refusal, **not** a uniformity ban, and the fix costs nothing in expressiveness.

**Why nothing downstream catches it.** §7's invariant is `host_admits ⇒
device_admits(canonical)` and all three canonicals re-parse as fixed points, so the invariant
holds while the wallet changes. §7's required rows are all uniform-path, so no required row
constructs the case. §5.4's confirmation prints *"the template and the placeholder-to-fingerprint
map"* — the operator is shown the template `me` derived from their input while the cards carry a
different one, so the confirmation actively reassures.

**One sentence in the fold points the safe way and is not normative.** Conjunct 7's closing
clause — *"which members of the admitted set md1 can carry is §5.3's per-`--as` split"* — reads
per-key. Three NORMATIVE blockquotes read per-descriptor. Blockquotes are what §11.4's tests are
written against.

**Required.** Re-quantify the md1 rules over keys: *"if **any key's** use-site path is a single
fixed child index / a multipath group without a trailing wildcard, `--as md1` refuses"*, and
*"`--as md1` materialises `<0;1>/*` for **every key whose** use-site path is absent"*. Add a §7
row for each of B1/B2/B4 (`host_admits=true`, `--as descriptor`-only for B1/B4, md1-capable for
B2) with the device-route `address_0` above — those three addresses are what makes the fix
testable. Either state that md1 carries per-key divergence through
`TLV_USE_SITE_PATH_OVERRIDES` with a vector, or add a conjunct requiring the keys to agree; the
one thing that must not survive is the mismatch between a per-key admission and a
per-descriptor representability rule.

**Evidence:** `md encode`/`decode`/`address` runs above; Go `nonstandard.OutputDescriptor` +
`address.Receive`/`Change`; `address/address.go:189–226`; `crates/md-codec/src/tlv.rs:10`.

---

## NEW — Important

### NEW-I1 — §7's new md1-splits bullet requires rows that carry `address_0` and are NOT md1-capable, while §7's `address_0` rule says the Rust test asserts that column *through the md1 round trip*. The vector file it specifies cannot be authored green

**What the fold wrote.** New bullet, line 1089: *"**the md1-representability splits of §5.3** —
`/0/*` and `<0;1>`-without-wildcard (host-admitted, `--as descriptor`-only) each with the
device-route `address_0`, plus a CHILDLESS input whose `address_0` proves (a′)'s
materialisation"*. Unchanged rule, line 1106: *"Rows carrying `--as md1` capability also carry
`address_0`… **The Rust test asserts it through the md1 round trip**; the Go test asserts it
through `address.Receive`."*

**The contradiction, with the numbers.** The two new rows are exactly the rows §5.3 forbids the
md1 route for, and the md1 route on them yields the *other* wallet's address — measured:

```
row "/0/*"            device-route address_0 = bc1qadgf37zk0wtu69j7yclswl99e5jmcv5eh69jp4calgs4ehsecm5sqcjw5a
                      md1 round trip         = bc1qu2cc6t70nm0tw0v3tsmgur33gjnw2a32czk6xatccky9jpjxj4eqcedjh9
row "<0;1>"           device-route address_0 = bc1qu2cc6t70nm0tw0v3tsmgur33gjnw2a32czk6xatccky9jpjxj4eqcedjh9
                      md1 round trip         = bc1qadgf37zk0wtu69j7yclswl99e5jmcv5eh69jp4calgs4ehsecm5sqcjw5a
```

Implemented as §7 states, the Rust test either refuses to encode (per §5.3(a)/(a″), which is the
*correct* behaviour and still fails the assertion) or asserts one of the mismatches above. Red
either way — and this is the row set §11.3 requires *"a test that counts, not… reading"* to
confirm is complete, so the counting test passes while the suite cannot go green.

**And the schema has no way to tell the test apart.** §7's row schema is `name`, `input`,
`sha256`, `host_admits`, `device_admits`, `format`, `source`, plus `canonical`, `sysw_class`,
`device_probe`, `address_0`. **There is no md1-capability column.** `host_admits` is defined by
§5.2 as *"parses under §4's cascade **and** matches §4.7's grammar"* — `--as`-independent, and
`true` for both new rows. So the file cannot express the very split the bullet was added to pin.
The only alternative — having the Rust test compute md1-capability from its own implementation —
violates §7 requirement 3 (*"Neither implementation is ever compared to the other — both are
compared to the file"*) and would stop pinning the split at all. The same ambiguity now sits on
the fold's other new row, `sh(wsh(sortedmulti(2,16 keys)))`, which is also required to carry
`address_0`.

**Not cosmetic.** Which `--as` value may carry a shape is a normative decision about which
wallet gets engraved, and §7 is the artefact that stops the two implementations drifting on it.
An unsatisfiable gate is r1's I1 class, re-entering through the fold.

**Required.** Add a per-row boolean (`md1_admits`, defaulting false) and state: `address_0` is
asserted by the Go test through `address.Receive` on every row that carries it, and by the Rust
test through the md1 round trip **only** where `md1_admits` is true — where it is false, the
Rust test asserts that the md1 path REFUSES. That second half is what turns §5.3(a)/(a″) from
prose into a gate, and it is currently untested by any bullet in §7 or §11.

### NEW-I2 — §5.1's new multi-record bullet and §6's new multi-record row capture the two MULTI-LINE descriptor formats by their own terms, and the bullet's stated justification is measurably false for them

**What the fold wrote.** §5.1, line 726: *"**A multi-record input containing a descriptor does
NOT get the '--as decides' block.** Adding `--as` to that invocation would read the file whole
and refuse with a message false about the file (NEW-I5, measured)…"*, routed to §6's row (988),
`EXIT_INVALID (4)`.

**The two formats this catches.** With `--as` absent, `--in`'s contract is newline-separated
records (§5.1's own premise), so a **BlueWallet setup file** (format 1, 12 lines) and a
**pretty-printed JSON export** (format 3) are both multi-record inputs, and each one *is* a
descriptor. Measured against `me 0.7.0` today — both give the generic shipped refusal §2.1
quotes as the gap this spec exists to close:

```
$ me sysw pack --no-passphrase --in bw.txt        rc=4  "record 0 … is not a form this container can place…"
$ me sysw pack --no-passphrase --in json.txt      rc=4  same message
```

**The justification is false for exactly these inputs.** The bullet's reason is that adding
`--as` would read the file whole and produce a message false about the file. Measured on the
fork's own BlueWallet fixture read as ONE WHOLE DOCUMENT:

```
nonstandard.OutputDescriptor(<the whole 12-line fixture>)  -> ACCEPT, P2WSH, 3 keys,
  canonical wsh(sortedmulti(2,[dc567276/48h/0h/0h/2h]xpub6DiYr…,…))#0dc3ykny, FIXED POINT ok,
  recv0 bc1q4taqq6q6l8fvguva6ftvrz3qgdjy6p3w2s0ds0nl6qrjw7t0hfhqgrqcwd
```

Adding `--as` to that invocation **works perfectly**. The rule's premise holds only for r2's
actual case — a stream of several distinct records, one of which is a descriptor — and the text
does not say so.

**Both readings are bad, and one of them breaks §11.5.**

- Read the row narrowly (*"whose **records** include a descriptor"* — a BlueWallet file's records
  do not): no rule in §5.1 or §6 covers a multi-line descriptor with `--as` absent, so today's
  generic rc=4 stands for formats 1 and 3 — and §11's acceptance item 5 (*"`--as` omitted with a
  descriptor input exits **2** and prints §5.1's block"*) is false for two of the four formats,
  while a test written on a one-line bip380 input passes.
- Read it broadly (multi-line ⇒ multi-record): the operator gets *"record `N` is a wallet
  descriptor. A descriptor is packed ALONE: run `me sysw pack --as <descriptor|md1>` with just
  the descriptor"* — for a file that **is** just the descriptor. A remedy that instructs the
  operator to do what they already did is the executable-remedy violation NEW-I5 was filed for,
  reappearing one input class over.

Before the fold there was one rule (`--as` is required whenever the input is a descriptor →
§5.1's block), and it covered all four formats. The fold added a competing branch keyed on a
distinction it never defines.

**Required.** One sentence stating the discriminator: when `--as` is absent and record
classification fails, `me` re-reads the **whole input** through §4's cascade — if it parses,
§5.1's block at `EXIT_USAGE (2)`; §6's multi-record row applies only when the whole input does
**not** parse as one descriptor and some individual record does. That is the reading §11.5
already assumes, and it costs one sentence in §5.1 plus a qualifier in §6's row.

---

## NEW — Minor

**NEW-M1 — §6's seven new rows are not in §6's table.** Line 981 is empty, separating the last
original row (`a bitcoin address`, 980) from the seven rows added by this fold (982–988). A GFM
table ends at a blank line and a new table needs a delimiter row, so those seven — including four
NORMATIVE refusals this fold introduced (hardened component, non-consecutive multipath, the
closed-set catch-all, the `<0;1>`-under-md1 split) — render as a paragraph of literal pipe text,
not as table rows. §11.4 requires a test per §6 row; a reader or a counting script working from
the rendered document sees a table of 22 rows and a paragraph. Fix: delete line 981.

**NEW-M2 — the zero-fingerprint warning is scoped to BlueWallet, and the same silent loss
happens on formats 2 and 3 with no warning.** §4.2's new paragraph (366) speaks of *"a
`00000000:` cosigner line"* and lives in the Format-1 section. Measured, an explicit zero
fingerprint in a plain BIP-380 descriptor loses its origin identically:

```
in : wsh(sortedmulti(2,[00000000/48h/0h/0h/2h]K1/<0;1>/*,[f245ae38/48h/0h/0h/2h]K2/<0;1>/*))
out: wsh(sortedmulti(2,K1/<0;1>/*,[f245ae38/48h/0h/0h/2h]K2/<0;1>/*))#69pv96ta   <- [00000000/48h/0h/0h/2h] gone
     fixed point ok, addresses unchanged (recv0 bc1qadgf37z…)
and: wpkh([00000000/84h/0h/0h]xpub…/<0;1>/*)  ->  wpkh(xpub…/<0;1>/*)#npw6pdpq
```

Same measurement, same harm, no warning. Note the warning must NOT simply be generalised to
every `mfp == 0` key: every §4.5 promoted bare key has `mfp = 0` by construction (measured: a
bare `zpub` promotes with `mfp=00000000 path=/84h/0h/0h`, and the canonical
`wpkh(xpub…)#u8s2vf65` drops that path), where the dropped origin was invented by the parser,
not supplied by the operator — and the message says "cosigner", which is wrong for a single-sig
wallet. State the rule over an origin the **input supplied**.

**NEW-M3 — §4.3 line 437 keeps the phrasing conjunct 3 was corrected away from.** The row reads
*"…**unspendable** (BIP-383: ≤ 15 keys **under `sh`**)"*, and `sh(wsh(sortedmulti(…)))` is under
`sh` while carrying 20 by the corrected conjunct 3 and by §6's corrected row. The row's own
subject is the direct form, so the measurement is right and the normative rule is unambiguous —
but this is the exact sentence r2's NEW-I2 identified, surviving in the section a reader reaches
first, and the fold commit's message claims the superseded-phrasing sweep was clean. Fix: *"≤ 15
keys when the `sortedmulti` is directly under `sh`"*.

---

## NEW — Nit

**NEW-N1 — `<a;b;c>` is not in `parsePath`'s grammar; the device refuses it outright.** Conjunct
7 lists *"`<a;b;c>` groups"* among *"everything else **in `parsePath`'s grammar**… refused as
UNMEASURED"*, and §6's closed-set row (986) tells the operator such a shape is *"outside the set
the device is measured to handle"*. Measured: `wsh(sortedmulti(2,K1/<0;1;2>/*,…))` →
`nonstandard: unrecognized output descriptor format`. `parsePath` cuts on the **first** `;` only
(`strings.Cut`, `bip380/bip380.go:476`) and `ParsePathElement("1;2")` then errors, so a
three-element group is a parse REFUSAL, not an admitted-but-unmeasured shape. Both refusals
agree, so no behaviour changes; the claim about the code is false. (`<1;0>` is likewise refused
by the `start > end` check at 489. The pre-existing §4.3 bullet describing the range as
`<a;b;…>` has the same error, and was not touched by this fold.)

**NEW-N2 — the `Upub` remedy names the wrong multisig form.** §6's SLIP-132 row groups
*"`Upub`/`Vpub` → `tpub` (**testnet multisig** … supply the full `wsh(sortedmulti(…))`
descriptor…)"*. By §4.5's own table `Ypub` is `P2SH_P2WSH` (`m/48'/0'/0'/1'`) and `Zpub` is
`P2WSH` (`…/2'`); `Upub`/`Vpub` are their testnet counterparts, so a `Upub` holder's form is
`sh(wsh(sortedmulti(…)))`. The row is a pointer rather than a substituted remedy and the same
imprecision predates it in the bare `Zpub`/`Ypub` row, so this is a Nit — but naming both forms
costs four words.

---

## Verified in passing, recorded so a later round does not re-spend it

- **Every measured number the fold added is TRUE.** (a″)'s `0x16d62` collision and its address
  pair; conjunct 7's `*h`/non-consecutive characterisations (re-derived from
  `address/address.go:189–226`, where `c.Hardened` is never read at the use site and
  `End != Index+1` errors); §6's per-version remedy addresses; `md decode` of a `/0/*` card set
  returning the `/*` form. Nothing in the new text was found describing code from a doc comment.
- **The five-member version set and the promotion table are unchanged and still correct** —
  re-confirmed the `default:` arm by measuring bare `ypub`, `upub`, `vpub`: all three REFUSE.
- **§5.3(a′) holds per key when implemented per key** — the materialised template reproduces the
  device's childless address exactly (`0x16d62`, `bc1qadgf37z…`), which is what makes NEW-C1's
  fix a one-word change rather than a redesign.
- **F-416 is filed correctly** (repo, owning phase, cross-reference to F-415), and F-414 is still
  the referent §5.1 and §6 both cite.
- **`me 0.7.0`'s current behaviour** for a descriptor input with `--as` absent is `rc=4` with the
  generic message, for all three of a one-line BIP-380 descriptor, a BlueWallet file and a
  pretty-printed JSON export — the baseline §5.1 and §11.5 are written against.

---

## Closing

**This lens does not close.** All ten r2 findings are FIXED, and the fold's central judgement
call — refusing r2's global conjunct in favour of a device-broken/md1-lossy split — is **right**,
verified member by member against both implementations; r2's prescription would have contradicted
§5.3(a)'s preserved path, exactly as the fold argued.

What the round found is one Critical and two Importants in the space the new rules opened, and
all three are the same failure mode: **the fold changed the quantifier or the branch condition on
one side of a pair and not the other.** Conjunct 7 became per-key while §5.3 stayed
per-descriptor (NEW-C1). §7's row list gained rows that are not md1-capable while §7's
`address_0` rule still routes every such row through md1 (NEW-I1). §5.1 gained a multi-record
branch while §11.5's all-formats promise stayed (NEW-I2). None needs new measurement to fix —
NEW-C1 is one word in three blockquotes, NEW-I1 is one column, NEW-I2 is one sentence — and the
addresses to test all three with are in this report.
