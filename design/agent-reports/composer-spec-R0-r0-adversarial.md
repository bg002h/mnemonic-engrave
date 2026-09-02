# R0 round 0 — adversarial review, LENS: FUNDS SAFETY BY COUNTEREXAMPLE

**Artifact:** `design/SPEC_wallet_policy_composer.md` (DRAFT 2026-09-01).
**Reviewer:** independent adversarial agent, read-only. Rulings C1-C29 treated as FINAL;
each finding below is a constructed wrong result, not an opinion about a ruling.
**Measured against:** `md` 0.14.0, `mk` 0.13.0, `me` 0.7.0 (all resolved by PATH —
`md` is shell-aliased to `mkdir -p` on this box), fork worktree
`/scratch/code/shibboleth/seedhammer`.

## VERDICT: 5 Critical / 5 Important / 2 Minor / 1 Nit — NOT GREEN

---

# CRITICAL

## C-1. The consent screen §7e names cannot describe ANY policy the composer exists to author — and the one surface that tries reports a two-path wsh wallet as **one** spend path

**Spec defeated:** §7e ("The existing Wallet Policy consent surface
(`walletPolicyConsentLines`): the structural summary, ..."), §3 inventory row
"Wallet Policy program: ... consent (summary, named id, receive+change) ... shipped",
§6c ("The consent screen shows the digest"), §8c (lock echoes).

**Constructed input.** The spec's own worked wallet — path list
`[P1: 2-of-3 unlocked] [P2: 1 key + older(26280)]`, wrapper `wsh`. Per §5 this
lowers to:

```
wsh(or_d(multi(2,@0/48'/0'/0'/2'/<0;1>/*,@1/48'/0'/1'/2'/<0;1>/*,@2/48'/0'/2'/2'/<0;1>/*),
         and_v(v:pkh(@3/48'/0'/3'/2'/<0;1>/*),older(26280))))
```

Measured: encodes to 1 keyless chunk / 8 keyed chunks, `wallet-descriptor-template-id
7a426a7ec63f9c1305282efa16267a26`, `wallet-policy-id 5290c2a74d12e25f253df790c52dd213`.

**Demonstration — trace the shipped code the spec reuses.**

1. `walletPolicyConsentLines` (`gui/wallet_policy.go:163`) builds its structural
   half from exactly one call: `lines = append(lines, md1Summary(tpl)...)`
   (`gui/wallet_policy.go:232`). It never calls `md.PolicyShapeChunks`
   (`grep -rn "PolicyShape" gui/*.go` → only `gui/multisig_build.go:800` and
   `gui/template_engrave.go`).
2. `md1Summary` (`gui/md1_inspect.go:84-99`):

   ```go
   if tpl.Renderable { lines = append(lines, "Type: "+scriptName(tpl)+" "+policyLine(tpl)) }
   else { lines = append(lines, "Complex policy - cannot display safely.", fmt.Sprintf("Keys: %d", tpl.N)) }
   ```

   and `policyLine` (`gui/md1_inspect.go:65-80`) returns a real label only for
   `PolicySingle | PolicyMulti | PolicySortedMulti | PolicyMultiA |
   PolicySortedMultiA`; every other policy returns the string `"complex"`.
3. `Renderable` is FALSE for exactly the shapes the composer produces. The
   fork's own pinned test says so: `md/md_test.go:337-344` —
   *"but the wsh body is and_v(...), outside §4.2 → Renderable=false"* — and
   `md/md_test.go:416-423` pins `tr(NUMS, sortedmulti_a)` → `Renderable=false`.

So for the wallet above the operator's consent screen, immediately before an
irreversible engrave, reads:

```
Policy-ID: 5290c2a74d12e25f253df790c52dd213
Complex policy - cannot display safely.
Keys: 4
@0 73c5da0a 48'/0'/0'/2' <0;1>/*
@1 73c5da0a 48'/0'/1'/2' <0;1>/*
@2 73c5da0a 48'/0'/2'/2' <0;1>/*
@3 73c5da0a 48'/0'/3'/2' <0;1>/*
<addresses>
```

**Nothing on it states that a SINGLE key spends the whole wallet alone after
26280 blocks.** The only structural claim is "Keys: 4", which an operator reads
as a 4-key multisig.

**And the better surface is worse than silent — it is FALSE.** The other consent
path, `templateConsentLines` → `policySummaryLines`
(`gui/template_engrave.go:63,142`), does render a shape. But `policyShape`
(`md/policy_shape.go:99-120`) handles `wsh`/`sh` by calling
`branchOf(inner, 0)` on the whole inner script and appending **exactly one**
`Branch`; `collect` (`md/policy_shape.go:230-243`) walks `tagOrI` and `tagOrD`
*into that single branch* rather than splitting on them. For the wallet above it
therefore emits:

```
Spend paths: 1
  1: 4 key(s), custom +timelock
```

The wallet has **two** independently satisfiable spend paths, and the second is
one key. "Spend paths: 1" is a false structural summary presented as consent
proof. (Taproot is unaffected — `walkTapTree` appends one Branch per leaf.)

**Neither surface ever shows a lock OPERAND or a hash DIGEST.** `Branch.Timelock`
/ `Branch.Hashlock` are documented as *"presence flags, not counts: ... the exact
value is a render"* (`md/policy_shape.go:52-57`). So §6c's "The consent screen
shows the digest" and §8c's lock echoes have no carrier at consent: the echo
exists only during entry (§6b), and the operator can Back out of entry, change
their mind, and never see the value again.

**Consequence.** The composer's entire reason to exist is authoring timelocks,
hashlocks and multiple spend paths, and §7e hands the consent gate to a surface
that renders none of them — or renders a wrong path count. Class (c): a threshold
the operator misreads, at the one screen that is supposed to prevent it.

**Minimal fix.**
1. Add a §9 device work item: split `or_i`/`or_d` (and `andor`, for consumed
   cards) into separate `Branch`es in `md/policy_shape.go`, and carry the lock
   operand + hash digest on `Branch` so they can be rendered.
2. Add a §9 item routing `walletPolicyConsentLines` through
   `PolicyShapeChunks`/`policySummaryLines` (today it calls neither).
3. Rewrite §7e to name the NEW surface and to state, as a normative list, what
   the composer's consent screen must show: path count, per-path k-of-n, the
   lock kind AND its value in operator units, the digest, and the key-path
   spendability line.
4. Add an acceptance row to §12: for every vector family, the consent lines are
   captured and asserted to name every path and every operand.

---

## C-2. A date entered at §6b that lands before 1985-11-06 is silently encoded as a BLOCK HEIGHT — the spec's own stated floor is one of them

**Spec defeated:** §6b (absolute/date row: "`after(unix at 00:00:00 UTC)`;
ceiling 2038-01-19" — a ceiling and no floor), §4c (`after(n)` time =
"500,000,000..=2,147,483,647 | Unix time, **1985-11-05** .. 2038-01-19 UTC").

**Constructed input.** Build with **no payload** — explicitly blessed by C26 and
§7g row 1, so there is no `now:` record and §6b's only date refusal ("BELOW the
`now:` value") never fires. Wrapper `wsh`; paths
`[P1: 2-of-3 unlocked] [P2: 1 key + absolute date 1985-11-05]`.

**Demonstration.**

```
$ python3 -c "..."
LOCKTIME_THRESHOLD = 500000000 -> 1985-11-05T00:53:20+00:00
1985-11-05 00:00 UTC -> 499996800 <500000000 => CLTV reads it as HEIGHT
1985-11-06 00:00 UTC -> 500083200 >=500000000 => CLTV reads it as TIME
```

`LOCKTIME_THRESHOLD` falls at **00:53:20** on 1985-11-05, so *midnight on the
date §4c names as the first time-based date is 3,200 seconds BELOW the
threshold*. §6b's "unix at 00:00:00 UTC" therefore converts the spec's own
documented floor into a height.

Nothing downstream catches it:

```
$ md encode --in t_after_date.txt --key @0=.. --key @1=.. --key @2=.. --key @3=.. --fingerprint ...
chunk-set-id: 0xf1878
md1f7xrcrs9q6tvyyy5jmpprjjtvyy49ykcgfw2sqrqnqvzyxfnf0d3mn2csqxygrnchdq5h83gaqlc8ery4aksw
... (8 chunks)
```

template used, verbatim:

```
wsh(or_d(multi(2,@0/48'/0'/0'/2'/<0;1>/*,@1/48'/0'/1'/2'/<0;1>/*,@2/48'/0'/2'/2'/<0;1>/*),
         and_v(v:pkh(@3/48'/0'/3'/2'/<0;1>/*),after(499996800))))
```

and the consent screen still shows a perfectly good address as proof:

```
$ md address --template "<above>" --key ... --fingerprint ...
bc1qmtmdl799equ5wyq602z3tvqu6ymqpz8lasrv0rkyucvvsvvzddvsq0sr37
```

**Consequence.** §6b's echo says `1985-11-05 00:00 UTC`. Consensus reads
`after(499996800)` as *block height 499,996,800*, reachable in roughly 9,500
years. The heir path is permanently dead; the operator is shown a date and an
address as proof. This is class (a) wrong result **and** class (b) false proof in
one artifact, and the whole class is invisible to md, to `sanity_check`, and to
the address derivation. Same hole for any date the operator mistypes into the
pre-1985 range on a digit pad (`YYYYMMDD` with a dropped or transposed leading
digit); the no-`now:` case simply removes the only guard §6b has.

**Minimal fix.** Two lines, both normative:
- §6b: date entry REFUSES any date whose 00:00:00 UTC value is `< 500,000,000`
  ("Dates before 1985-11-06 encode as a block height, not a date"), independently
  of whether a `now:` record is present. Better still, floor it at the genesis
  date (2009-01-03) since no earlier date is ever meaningful.
- §4c: correct the time row's human range to **1985-11-06 .. 2038-01-19 UTC** for
  *date entry*, keeping 500,000,000 as the operand floor, and say the two differ.

---

## C-3. The composed keyless template is never required to declare FINGERPRINTS, so the world's most common multisig setup engraves a template the device's own seating refuses

**Spec defeated:** §7f (form B = "keyless md1 + mk1 cards"), §5 (the lowering
table specifies use-sites and placeholder numbering and says nothing about
fingerprints), §12.1 (vectors pin "path list → template text → md1 chunks" — the
BIP-388 template TEXT carries no fingerprints, so the vectors as specified pin a
fingerprint-less md1), §7g (no divergence row for colliding declared origins).

**Constructed input.** Three cosigners — three different masters — each supplying
a `key:` record at the standard BIP-48 path, which is what every Sparrow /
Nunchuk / Coldcard 2-of-3 in existence uses:

```
key:<hex of "[73c5da0a/48'/0'/0'/2']xpub6DkFAXWQ2dHxq...">
key:<hex of "[1b2c3d4e/48'/0'/0'/2']xpub6DzhyrnFFYQ1H...">
key:<hex of "[5f6a7b8c/48'/0'/0'/2']xpub6EGx8sPr9FxPP...">
```

Wrapper `wsh`, one path, 2-of-3 unlocked → §5 key-set rule "unlocked single-path:
`sortedmulti`" →
`wsh(sortedmulti(2,@0/48'/0'/0'/2'/<0;1>/*,@1/48'/0'/0'/2'/<0;1>/*,@2/48'/0'/0'/2'/<0;1>/*))`.
§4f is explicit that the declared origin is carried as given, so all three slots
declare the same path.

**Demonstration.**

```
$ md encode --in same_origin.txt
md1yzfdsssj5qqcy8pzrqrxahye32v7pju
warning: this keyless template's slots cannot be told apart — @0, @1, @2 all
declare m/48'/0'/0'/2'. Restoring seats one key card per slot by matching the
slot's declared origin (and its fingerprint, when declared), so a card here
matches several slots and a device that will not guess must refuse the whole
set. Pass one --fingerprint @N=HEX per slot to make them distinguishable; it
costs about one extra md1 chunk and changes no path, no key and no policy.

$ md encode --in same_origin.txt --fingerprint @0=73c5da0a --fingerprint @1=1b2c3d4e --fingerprint @2=5f6a7b8c
md1yzfdsssj5qqcy8pzrqhesu79mg9ydjc02wjldfaccy9tldqxc25r9d      # no warning
```

The device end confirms md's warning is not hypothetical.
`slotMatchesCard` (`gui/key_card_seating.go:129-152`) checks the fingerprint
**only when the template declares one** —

```go
if slot.FingerprintPresent { ... }
```

— so with no fingerprints every card matches every slot, and `seatKeyCards`
(`gui/key_card_seating.go:88-92`) hits

```go
return nil, fmt.Errorf("%w: @%d claimed by cards %d and %d", errSeatSlotContested, ...)
```

`walletPolicyConsentLines` turns that into a hard refusal
(`gui/wallet_policy.go:179-183`).

**Consequence.** Class (d): the operator engraves a keyless template plus three
key cards, and neither the SH2's Wallet Policy program nor `md address
--from-mk1` can reassemble it — every restore attempt refuses the whole set. The
plates are recoverable only by a human retyping xpubs into `md encode --key` at a
host, which is the failure mode the whole template+cards form exists to avoid.
The host tool already warns about this; the spec that authorises the device to
mint such templates does not mention fingerprints once.

**Minimal fix.** One NORMATIVE line in §5 or §7f: *every slot of a composed
keyless template declares BOTH the origin path and the master fingerprint of the
key seated into it; a keyless template with two slots at the same declared origin
and no fingerprints is never engraved.* Add the fingerprint columns to §12.1's
vector shape, and add a §7g row for it.

---

## C-4. §7d's card re-mint appends the POLICY stub, which is the one stub that makes the form-B artifact unseatable

**Spec defeated:** §7d ("a seated card is RE-MINTED for engraving with **the new
policy's stub** appended to its existing stubs (`mk.Encode`)") against §7c ("the
screen recommends stamping BOTH stubs") and §7f (form B = keyless template + mk1
cards).

**Constructed input.** The C-1 wallet, engraved as form B. Measured ids for it:

```
$ md inspect <keyless md1>
wallet-descriptor-template-id: 7a426a7ec63f9c1305282efa16267a26   -> TEMPLATE stub 7a426a7e
$ md inspect <keyed md1>
wallet-policy-id:              5290c2a74d12e25f253df790c52dd213   -> POLICY   stub 5290c2a7
```

Re-minting per §7d's literal text — "the new policy's stub":

```
$ mk encode --xpub xpub6DkFAXWQ2dHxq... --origin-path "m/48'/0'/0'/2'" \
            --origin-fingerprint 73c5da0a --policy-id-stub 5290c2a7 --group-size 0
$ mk decode <that card>
xpub:                xpub6DkFAXWQ2dHxq2vatrt9qyA3bXYU4ToWQwCHbf5XB2mSTexcHZCeKS1VZYcPoBd5X8yVcbXFHJR9R8UCVpt82VX1VhR28mCyxUFL4r6KFrf
origin_fingerprint:  73c5da0a
origin_path:         48'/0'/0'/2'
policy_id_stubs:     5290c2a7
```

versus the §7c-compliant card:

```
$ mk encode ... --policy-id-stub 5290c2a7 --policy-id-stub 7a426a7e --group-size 0
$ mk decode <that card>
policy_id_stubs:     5290c2a7, 7a426a7e
```

**Demonstration that the first one is dead.** `seatKeyCards` LAYER 1
(`gui/key_card_seating.go:53-73`):

```go
stub, err := md.FormAwareStubChunks(templateMd1)   // keyless template -> WalletDescriptorTemplateIdStub = 7a426a7e
...
if !hasStub(c.Stubs, stub) {
    return nil, fmt.Errorf("%w: card %d (%s)", errSeatNotThisPolicy, i+1, c.Path)
}
```

`FormAwareStub` selects by form: *"a keyed wallet-policy roots on WalletPolicyId;
a keyless template roots on WalletDescriptorTemplateId"* (`md/template_id.go:106-118`).
Card A carries only `5290c2a7`; `hasStub([5290c2a7], 7a426a7e)` is false; every
card is refused with *"card does not belong to this policy"*.

**Consequence.** Class (d) again, and this one is caused purely by the spec's own
wording: the artifact §7f tells the operator to engrave is refused by the
device's own restore path, and the refusal text ("does not belong to this
policy") points the operator at the wrong problem. Note the two spec sentences
disagree with each other — §7c says both, §7d says one — and §7d is the normative
one an implementer transcribes.

**Minimal fix.** Change §7d to: *the re-mint appends BOTH the composed policy's
stub (`WalletPolicyId[0..4]`) and the composed template's stub
(`WalletDescriptorTemplateId[0..4]`) to the card's existing stubs, so one card
seats into either engraved form.* Add a §12 acceptance row: for every vector,
`seatKeyCards(engraved keyless md1, re-minted cards)` succeeds and reproduces the
keyed policy's addresses.

---

## C-5. A hash-gated SOLE spend path is admitted with no preimage anywhere, no warning, and an address shown as proof

**Spec defeated:** §4b (`HASH`: "at most one `sha256(H)`"), §4e (the structural
refusal table has no row for it), §4d (`hashlock-gated` is a ONE-TAP PRESET),
§6c ("On-device preimage derivation is DEFERRED"), §8a (the EXPERIMENTAL screen
covers only the KEYLESS direction), §11 ("Every refusal names what to do
instead"), §7g (no row).

**Constructed input.** Wrapper `wsh`, ONE path: keys 2-of-3, hash = a `hash:`
record from the payload, no lock. §5 key-set rule "locked/hashed multi-key:
`multi`"; §5 inside-a-path rule `and_v(v:KEYS, and_v(v:sha256(H), LOCK))` with
LOCK dropped:

```
wsh(and_v(v:multi(2,@0/48'/0'/0'/2'/<0;1>/*,@1/48'/0'/1'/2'/<0;1>/*,@2/48'/0'/2'/2'/<0;1>/*),
          sha256(6c60f404f8167a38fc70eaf8aa17ac351f8c4606012d3d4ce2e4e9c0b6b4b6b4)))
```

**Demonstration.**

```
$ md encode --in tt.txt
rc=0
chunk-set-id: 0x8703d
md1fsupaqs9qjtvyyy5jmpprjjtvyy49gqpsfxdrq3pn4kxpaqylqg6l7j4a4kvnse4
md1fsupaqsw0gu0cu82lz4p0tp4r7xyvpsp9575echya8qtdd9kksq6ta6t5kkcteam
```

The taproot sibling (`tr(H,and_v(v:multi_a(2,...),sha256(H)))`, chunk-set-id
`0x6c13e`) and the single-key form (`wsh(and_v(v:pkh(@0/...),sha256(H)))`)
encode too. Walk §4e's refusal table row by row: *no path with keys* — has keys;
*neither keys nor hash* — has both; *keyless under tr* — not keyless; *>8 paths /
n>9* — one path, n=3; *sh with a lock or hash* — wrapper is wsh. **Admitted.**

**Consequence.** The device never has the preimage (§6c/§14), §7f engraves no
plate for it, and no copy anywhere tells the operator to keep it. The consent
screen derives and shows a valid address, so the operator sees proof of a wallet
whose funds nobody can ever move once the host copy of the preimage is gone. The
product's guarantee is "the plates are the backup"; here the plates are
*structurally* an incomplete backup and the device says nothing. The same shape
covers a sole path gated on `after(2147483647)` (measured: encodes,
`md15zfdsssjjtvyyw2fdssj54qqxpye5vzyxdhlllllu83f05kdungjvn`) — but time passes
and a lost preimage does not, which is why this one is Critical rather than
informational.

**Minimal fix.** A NORMATIVE warning screen in §8 and a row in §7g, fired when
**every** spend path carries a hash: *"Every way to spend this wallet needs the
preimage of a hash. It is not on this device and it is not on these plates. If
you lose it, the coins cannot be moved. Back up the preimage separately."* Plus
one §4e/§6c line stating the composer never derives or engraves a preimage this
cycle. (A refusal would be wrong — the shape is legitimate — but silence is worse
than telling the operator nothing, which is exactly the journey rule's test.)

---

# IMPORTANT

## I-1. §5's key-set table has no row for a bare unlocked multi-key path that is NOT the sole path — and one of the two readings is refused by md and by the device's own restore guard

**Spec defeated:** §5 key-set row: "unlocked single-path: `sortedmulti`;
locked/hashed multi-key: `multi`; one key: `pkh`".

**Constructed input.** Two paths, wsh: `[P1: 1 key + older(26280)] [P2: 2-of-3
unlocked]`. P2 is unlocked, unhashed and multi-key, and it is **not** the sole
path, so it matches none of the three rows. §5's `or_d` row calls such a path
"a bare unlocked, unhashed `multi(k,…)`", implying `multi`; the key-set row's
word "unlocked" implies `sortedmulti`. Both readings are available to an
implementer, and they are different wallets with different addresses.

**Demonstration.** The `sortedmulti` reading does not encode at all:

```
$ md encode --in p3a.txt
md: sortedmulti() is valid only as the sole child of sh() or wsh() (BIP-388
§Descriptor Templates, BIP-383); it cannot be nested inside a miniscript fragment
   (rc=2)
```

while the `multi` reading does (`md15rfdssju5jmppp9ykcggu5jmpp9f2qqvzv5e5k8qqqpn2sxpzdsrjuuhvmcmtudl`).
The device has the same trap on the restore side: `walletPolicyConsentLines`
runs `md.TemplateEngraveShapeGuardChunks` on any keyless template and the fork's
own comment names what it refuses — *"`sortedmulti` under a combinator, which our
own encoder rejects outright"* (`gui/wallet_policy.go:208-214`).

**Consequence.** An implementer taking the "unlocked → sortedmulti" reading ships
a composer that dead-ends at engrave time after the operator has walked shape,
seating, stub-teaching and consent. §3 already records the measurement
(brainstorm §3.8), but §5 — the NORMATIVE table an implementer transcribes —
does not encode it. Note the tr column is fine: a bare unlocked leaf IS a whole
leaf, so `sortedmulti_a` is legal there.

**Minimal fix.** Replace the wsh key-set row with three explicit rows:
*sole path, unlocked, unhashed, n≥2 → `sortedmulti`; any other unlocked/locked/
hashed multi-key path → `multi`; one key → `pkh`* — and add a sentence naming
BIP-383/BIP-388's sole-child rule as the reason.

## I-2. §5's taproot depth formula is stated over PATHS, but internal-key extraction removes one; the two readings give different taptrees and therefore different addresses

**Spec defeated:** §5 tr row ("one leaf per path on a right spine in listed order
`{P1,{P2,{P3,P4}}}`; path k at depth min(k, n−1)") against §5 internal-key row
("the FIRST-LISTED unlocked, unhashed one-key path (then not a leaf)").

**Constructed input.** Four paths under `tr`, where the extracted one is **not**
first — exactly the vector C1 requires in §10 item 1:

```
P1: 2-of-3 unlocked
P2: 1 key + older(26280)
P3: 1 key unlocked            <- extracted as the internal key, @0
P4: 1 key + sha256(H)
```

Reading A (n = 4 paths): leaves P1, P2, P4 at depths min(1,3)=1, min(2,3)=2,
min(4,3)=3. **Unsatisfiable** — a right spine of three leaves has depths 1, 2, 2.
Reading B (n = 3 leaves, renumbered): depths 1, 2, 2.

**Demonstration that the ambiguity is address-affecting.** Two four-key taptrees
that differ only in shape:

```
$ md address --template "tr(@0/...,{pk(@1/...),{pk(@2/...),pk(@3/...)}})" ...
bc1psyzqndustsn4x4t8cqhyyxu3maw06g9s3l9acqyswxhx6p9qpt2q26gas5

$ md address --template "tr(@0/...,{{pk(@1/...),pk(@2/...)},pk(@3/...)})" ...
bc1p2cycz66cyzltrvy5xd6v6g0jz3m5f9tjtz0aqkx9a5xet23su5rsa4mewu
```

**Consequence.** §5 is the Rust-first normative definition that the Go port must
reproduce byte-for-byte. A rule whose only well-formed reading has to be inferred
by noticing the other one is unsatisfiable is not a rule; the Rust/Go byte
equality in §5b is then a *test* standing in for a missing *definition*, and a
disagreement lands as a wrong address on a plate.

**Minimal fix.** Restate the tr row over LEAVES: *"let `L` be the path list with
the extracted internal-key path removed, `m = |L|`; leaf j (1-based, in listed
order) sits at depth min(j, m−1) on a right spine"*, and add a vector where the
extracted path is P3 of 4 (§10 item 1 already asks for a not-path-1 case; make it
a not-path-1 case with ≥4 paths so the depth formula is exercised).

## I-3. §4e admits `sh(pkh(K))`, which §4a forbids and which is not a BIP-388 descriptor template

**Spec defeated:** §4a (`sh(wsh)`, `sh` admitted "ONLY [for] a single path that
is an unlocked, unhashed `sortedmulti`") against §4e (the enforcing row is
"`sh`/`sh(wsh)` with more than one path or any lock/hash → REFUSE").

**Constructed input.** Wrapper `sh`, one path, n=1, k=1, no lock, no hash. §4e's
condition is not met (one path, no lock, no hash), so nothing refuses; §5's
key-set rule "one key: `pkh`" then emits `sh(pkh(@0/48'/0'/0'/2'/<0;1>/*))`.

**Demonstration.**

```
$ md encode --in tt.txt        # sh(pkh(@0/48'/0'/0'/2'/<0;1>/*))
rc=0
md1yqfdsssj5qqcxtvk00nuhpwfcz3

$ md encode --in tt.txt        # sh(sortedmulti(1,@0/48'/0'/0'/2'/<0;1>/*))
rc=0
md1yqfdsssj5qqcx8qqm4tpp64zzekhc
warning: sh(multi)/sh(sortedmulti) is legacy P2SH multisig — ... prefer wsh(...) or sh(wsh(...))
```

**Consequence.** The operator picks the wrapper labelled "Multisig migration" and
gets a P2SH-wrapped single-sig that BIP-388 does not list as a descriptor
template, so a strict wallet-policy registration (Ledger, BIP-388-strict
coordinators) refuses the wallet the plates describe. §4a's stated admission and
§4e's enforcing condition are not the same predicate.

**Minimal fix.** Make §4e's row match §4a exactly: *"`sh`/`sh(wsh)` with anything
other than a single unlocked, unhashed path whose key set is a `sortedmulti` with
n ≥ 2 → REFUSE"*.

## I-4. Nothing invalidates seating when the operator Backs into the shape and edits it, and §5 renumbers slots by first appearance

**Spec defeated:** §7b ("Back preserves everything"), §7d ("Back keeps
assignments"), §5 ("`@i` by FIRST APPEARANCE in the emitted text; slot labels
shown to the operator are these indices, computed after lowering"), §7g (no row).

**Constructed step-by-step through the spec's own rules.**

1. Paths `[P1: 1 key] [P2: 2-of-3 + older(26280)]` → §5 emits
   `wsh(or_i(pkh(@0),and_v(v:multi(2,@1,@2,@3),older(26280))))`.
   Slots: @0 = the sole signer; @1..@3 = the 2-of-3 cosigners.
2. Operator seats @0=Alice, @1=Bob, @2=Carol, @3=Dave (§7d).
3. Operator Backs to shape (§7b/§7d both promise nothing is lost) and DELETES
   P1.
4. Re-lowering: `wsh(and_v(v:multi(2,@0,@1,@2),older(26280)))`. Slot labels are
   recomputed by first appearance (§5), so @0..@2 now name the cosigner seats.
5. §7d's retained assignments are keyed by slot index. @0 → Alice, @1 → Bob,
   @2 → Carol; Dave is dropped.

**Consequence.** Alice — the key the operator chose to be able to spend alone — is
now one seat of the timelocked 2-of-3, and Dave is not in the wallet at all.
Nothing in §7b, §7d or §7g states that a shape edit invalidates seating, and the
spec's two "Back loses nothing" promises actively encourage retention. The
mapping-review screen would show the new mapping, so this is Important rather
than Critical — but it is a silent key-in-the-wrong-slot generator that the spec
creates by combining two of its own rules, and the seed-derived case is worse:
§7d assigns hardened accounts "by ordinal among the slots that master fills", so
a renumber also changes the DERIVED xpub.

**Minimal fix.** One normative sentence in §7d: *"any change to the path list
after seating begins discards all assignments; the operator is told so before the
edit is accepted"*, plus a §7g row classifying it as a WARNING-before-edit. And
fix §7d's ordinal rule to name its order explicitly ("by ascending emitted slot
index"), which today is unstated.

## I-5. §6a specifies only HEX-validity for the three new record classes; every body-level malformation is undefined

**Spec defeated:** §6a ("a reserved prefix, a lowercase-hex body, matched BEFORE
the sniffers, and a prefixed record whose body is not valid hex is `ClassUnknown`
and refused"). That is inherited verbatim from `SPEC_systemwide_payloads.md`
§5.3.1, where the two classes it was written for (`text:`, `pass:`) accept
*arbitrary* UTF-8 — so hex-validity is the whole contract. All three new classes
have structured bodies, and none of the structure is specified.

**Constructed inputs, each valid lowercase hex and therefore admitted as its
class:**

| record | body hex-decodes to | what §6a says happens |
| --- | --- | --- |
| `hash:<66 hex>` | 33 bytes | nothing — §6a says "the 32-byte digest itself, 64 lowercase hex" but states no length refusal. A truncation or a pad is a hashlock whose preimage nobody knows (see C-5). |
| `key:<hex>` | `wsh(sortedmulti(2,...))` | nothing — the body is valid hex; §6a's only stated refusal is "a bare xpub is refused naming the fix". |
| `key:<hex>` | `[73c5da0a/48'/0'/0'/2']xpub<depth-3 key>` | nothing — the origin has 4 components, the xpub has depth 3. md1 carries 65 key bytes and no depth (brainstorm §3.4), so the discrepancy is unrecoverable from the plate. |
| `now:<hex>` | `4294967295` / `-1` / `not-a-number` / `1756684800,` | nothing — no numeric range, no sign rule, no trailing-comma rule. |
| two `now:` records | two different pack times | nothing — no precedence rule; the earlier one silently weakens §6b's only date refusal. |

**Demonstration that this is genuinely new surface, not inherited behaviour:**
`me` 0.7.0 refuses all three prefixes today, so nothing existing constrains them —

```
$ me sysw pack --no-passphrase --in pack_in.txt --out payload.bin
me: record 0 ... is not a form this container can place: not a BIP-39 mnemonic,
not an md1/mk1/ms1/mt1 string, and not a `text:`/`pass:`/`tx:` record.
```

**Consequence.** Class (e): records admitted and mis-decoded. Each row above is
an implementer's judgement call today, and the Rust host and the Go device are
required to be lockstep (§6a, §9 item 8) — divergent judgement calls there are
the exact shape F-212 measured.

**Minimal fix.** Extend §6a's normative paragraph: after hex-decoding, `hash:`
MUST be exactly 32 bytes; `key:` MUST parse as BIP-380 key-origin notation with a
non-empty origin whose component count equals the xpub's depth; `now:` MUST match
`^[0-9]{1,10}(,[0-9]{1,9})?$` with seconds in `1..=2147483647`; any failure is
`ClassUnknown` and refused. At most one `now:` record per payload; two or more is
a refusal, not a silent pick. State the refusal copy for each, per §11.

---

# MINOR

## M-1. §6b's "BELOW the `now:` value" does not say WHICH component a height lock is compared against

`now:` is `<unix-seconds>[,<block-height>]`. §6b refuses "A date or height BELOW
the `now:` value" — singular. An implementer comparing an absolute *height* entry
(range 1..499,999,999) against `now:`'s *seconds* (≈1.75e9) refuses every legal
height. The intended pairing is obvious but unstated, and the height component is
optional, so the height case has no bound at all when it is absent — which §6b
only half-says ("+ lower-bound line if `now:` carries a height"). One sentence:
*dates compare against the seconds component; heights compare against the height
component and are unbounded below when it is absent.*

## M-2. §4a's `n ≤ 15` bound for `sh` is dead text

§4b caps `n` at 9 for every path, so the `sh` row's "n ≤ 15 (Core
`MAX_P2SH_SIGOPS`)" can never bind. Harmless, but it reads as an active
constraint and a future widening of §4b to 15 would silently make it load-bearing
without anyone re-checking the citation. Say "n ≤ 9 by §4b; the P2SH ceiling of
15 is not the binding one".

---

# NIT

## N-1. §4c's `after`-time row labels the range "1985-11-05 .. 2038-01-19 UTC"

The operand floor is 500,000,000 = 1985-11-05 **00:53:20** UTC. Every earlier
instant that day is a height. Covered by C-2's fix; recorded separately because
the table is cited as SOURCED and a reader will take the date at face value.

---

# ATTACKS TRIED THAT FAILED — do not re-run these

1. **§7c's stub screen shows a stub that changes after seating.** C9 claims the
   `WalletDescriptorTemplateId` is "key-independent and origin-invariant".
   **VERIFIED TRUE by measurement**, three ways: same shape with origins
   `48'/0'/{0,1,2,3}'/2'`, with origins `48'/0'/{7,8,9,5}'/2'`, and with
   fingerprints added — all three give
   `wallet-descriptor-template-id: 7a426a7ec63f9c1305282efa16267a26`, while
   `md1-encoding-id` and `wallet-policy-id` all differ. The §7c screen can
   legitimately be shown before seating.
2. **Keyless wsh paths (§4b/§8a EXPERIMENTAL) refused by `md encode`.** Not
   refused. Both `wsh(or_i(pkh(@0/...),sha256(H)))` (chunk-set-id `0x5e071`) and
   `wsh(or_i(pkh(@0/...),and_v(v:sha256(H),older(1000))))` (`0x71849`) encode
   cleanly.
3. **Mixing lock kinds across paths trips miniscript's timelock-mixing rule.**
   It does not. `after(800000)` in one path with `after(1900000000)` in another
   encodes (`md15pfdsssjjtvyyw2sqrqn9xd9nvqqcdgqfnfwmwylmxqqkac6l7gxy7mca`), as
   does `older(1000)` with `older(4194304)`. §4b's one-lock-per-path rule makes
   within-path mixing unconstructible, so the §4c exclusion needs no entry-time
   enforcement.
4. **The three new prefixes collide with something `me sysw pack` accepts
   today.** They do not: `me` 0.7.0 refuses `key:`, `hash:` and `now:` lines
   outright (output quoted in I-5), so no existing payload can carry them and
   there is no reclassification hazard for already-packed payloads.
5. **§6b's day→unit arithmetic is wrong.** It is right. 90 d → ceil(90×86400/512)
   = 15188 units → 90.003 d, matching §8c's echo verbatim; the 388-day cap gives
   65475 ≤ 65535, and 389 would give 65643 > 65535, so the cap is exactly tight.
6. **The composer can seat one key into two slots and reach consent.** It cannot:
   §7d forbids it, `md encode` refuses "same key at the same use-site", and
   `walletPolicyConsentLines` refuses again at `md.DuplicateKeySlots`
   (`gui/wallet_policy.go:225-231`, F-218) — a genuine belt-and-braces.
7. **A tr policy whose only path is an unlocked single key has no valid
   lowering.** It does: `tr(@0/48'/0'/0'/3'/<0;1>/*)` encodes
   (`md1yqfdsssjuqqczqd536xms3y3e3e`); an internal key with no tree is fine.
8. **`md` accepts a keyless template with no origins, defeating restore.** It
   accepts, but warns loudly and names the fix ("supply --path (e.g. --path
   bip48) for a fully-decodable backup"), and §6a already makes the origin
   mandatory on `key:` records. Not a composer defect.

---

# WHAT I RAN

Scratch dir `/tmp/claude-1000/.../scratchpad`. All tools by absolute path
(`md` is aliased to `mkdir -p` in this shell — measure by path, not by name).

```
md --version                        # md 0.14.0
mk --version                        # mk 0.13.0
me --version                        # me 0.7.0
md encode --in <template>  [--key @i=XPUB] [--fingerprint @i=HEX]     # 14 templates
md inspect <md1 chunks>                                               # 5 chunk sets
md address --template <T> --key ... --fingerprint ...                 # 4 policies
mk encode --xpub .. --origin-path .. --origin-fingerprint .. --policy-id-stub .. [x2]
mk decode <mk1>                                                       # 2 cards
me sysw pack --no-passphrase --in pack_in.txt --out payload.bin       # 1 (refused)
python3  (LOCKTIME_THRESHOLD date arithmetic)
```

Fork source read (no file modified):
`gui/wallet_policy.go`, `gui/key_card_seating.go`, `gui/md1_inspect.go`,
`gui/template_engrave.go`, `md/template_id.go`, `md/policy_shape.go`,
`md/md_test.go` (Renderable pins).
Spec/brainstorm read in full: `design/SPEC_wallet_policy_composer.md`,
`design/BRAINSTORM_wallet_policy_composer.md` §1-§5,
`design/SPEC_systemwide_payloads.md` §3.3, §5.3.

Fixtures: `design/journeys/inputs-walletpolicy/key{0..3}.xpub`, `master.fingerprint`.
