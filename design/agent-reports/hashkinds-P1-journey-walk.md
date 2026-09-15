# hashlock-kinds P1 — operator journey walk across the `ms` / `md` seam

**Lens:** operator journey (not correctness). The question asked at every step was
*"what does a real person have in hand, and what happens when they reasonably do
something else?"* — not *"is this section right?"*

**Artifacts under walk**

| tool | version | revision | state |
| --- | --- | --- | --- |
| `ms` (mnemonic-secret) | 0.19.0 | `2bf1b3f` (master, ships `9dcd2e0`) | SHIPPED |
| `md` (descriptor-mnemonic) | 0.15.0 | `496b0767`, branch `hashkinds-p1` | not pushed |
| `me` (mnemonic-engrave) | 0.9.0 | `51d20709` | phase 3 NOT built |

`cargo --version` = `cargo 1.85.0 (d73d2caf9 2024-12-31)` in both build repos.
Every command below was executed; all output is pasted verbatim. Both worktrees
were left clean (0 modified files in all three repos, verified after the walk).

Shared fixture: phrase `correct horse battery staple` (28 chars), via
`--hashlock-phrase-stdin`. Preimage
`c3e97525442520da4cffd5f57aae3f6273990017f2e0fa30c056e32172e22016`.

| kind | digest | width |
| --- | --- | --- |
| sha256 | `3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12` | 64 |
| hash256 | `98a20fc25dbcdf236fb0307e3f82cad47fca2e807f3ef82c31993549641cd488` | 64 |
| ripemd160 | `09e7bb5051d89788fb4e4b374126721dbcc2946b` | 40 |
| hash160 | `b5b72c0e6896ff59dfa99e0d1052a9c0214cd0bd` | 40 |

---

## Journey A — the happy path across the seam (ripemd160)

### A.1 The operator derives the preimage and digest

```
$ ms hashlock --kind ripemd160 --hashlock-phrase-stdin < phrase.txt
```

stdout (one line, the record):

```
hash:ripemd160:09e7bb5051d89788fb4e4b374126721dbcc2946b
```

stderr (the card), verbatim:

```
THIS CARD CARRIES THE PREIMAGE -- the secret. stdout carries only the public digest.
digest:          09e7bb5051d89788fb4e4b374126721dbcc2946b
for md compose:  --path ... ripemd160=09e7bb5051d89788fb4e4b374126721dbcc2946b
                 requires `ripemd160=` support in `md compose`. If it answers "unknown option `ripemd160`", that support has not shipped in your `md` yet -- the preimage and digest above are still correct, and `md` is what has to catch up.
preimage (ms1):  ms10h ashsq 0p7ja f9gsj jpkjv ll2l2 74w8a 388xg qzlew p73sc ptwxg tjugs pvs8t klufg 89hqj
preimage (hex):  c3e97525442520da4cffd5f57aae3f6273990017f2e0fa30c056e32172e22016
method:          preimage = PBKDF2-HMAC-SHA256(password = phrase, salt = "ms-hashlock-v1", iterations = 100000, dkLen = 32)
phrase:          28 characters -- write the method line AND the hash line (ripemd160) next to your phrase unless the phrase is cut on a HASHLOCK PHRASE plate, which carries both; if the method line is lost, try each method that shipped with the version named on this card (ms-cli 0.19.0), and if the hash line is lost, re-run with no --kind and match the digest against your descriptor
The preimage must be exactly 32 bytes (64 hex characters) for every kind: the script checks OP_SIZE 32 before OP_RIPEMD160 (composer spec §8i, F-132).
One phrase per policy. Spending any path of a wsh wallet publishes this digest. Never use this phrase as a passphrase or a password anywhere else -- a spend publishes the preimage, and anyone can then test guesses at the phrase itself.
source:          phrase (stdin)
the record on stdout needs `ripemd160` support in `me sysw pack`. If it asks for "exactly 64 hex characters", that support has not shipped in your `me` yet -- the record is correct and `me` is what has to catch up. Do not reshape the record to satisfy it.
```

**In hand:** a 40-hex digest, a self-labelled `ripemd160=` operand, the preimage in
two encodings, the KDF parameters, and two forward-looking caveats about `md` and `me`.

### A.2 The seam — does `md compose` now take it?

```
$ md compose --wrapper wsh --path 2of3 --path '1of1,ripemd160=09e7bb5051d89788fb4e4b374126721dbcc2946b'
wsh(or_d(multi(2,@0/48'/0'/0'/2'/<0;1>/*,@1/48'/0'/1'/2'/<0;1>/*,@2/48'/0'/2'/2'/<0;1>/*),and_v(v:pkh(@3/48'/0'/3'/2'/<0;1>/*),ripemd160(09e7bb5051d89788fb4e4b374126721dbcc2946b))))
note: stdout is a keyless descriptor template (no keys)
exit=0
```

**THE SEAM CLOSES.** Phase 2's journey finding (`unknown option ripemd160`) is fixed.
The digest appears verbatim in the template, directly comparable with the card's
`digest:` line.

### A.3 Does it survive the engraving format?

```
$ md encode 'wsh(or_d(multi(2,...),and_v(v:pkh(@3/...),ripemd160(09e7...))))'
md15rfdsssjjtvyyw2fdssj55jmpp9e2qqvzvpsgsexd97qz08hdg9rkyh3ra5ujehgyn8y8duc22xku90fe52qzax09

$ md decode md15rfdsss...qzax09 | grep -o 'ripemd160([0-9a-f]*)'
ripemd160(09e7bb5051d89788fb4e4b374126721dbcc2946b)
```

Full round trip intact. The new kinds are engravable, not just composable.

### A.4 All the way to a fundable address

```
$ md descriptor --template "$T" --key @0=[11111111/48'/0'/0'/2']xpub6Bem... --key @1=... --key @2=...
wsh(and_v(v:multi(2,[11111111/48'/0'/0'/2']xpub661MyMwAqRbcFw9fmV2Q3...,...),ripemd160(09e7bb5051d89788fb4e4b374126721dbcc2946b)))#d57g29p5
note: stdout is watch-only — public keys only, cannot spend

$ md address --template "$T" --key ... --count 2
bc1qzrfppa8g0sp8ls2mserk2y4djw3l0ratkn2d2jf43wjqm7a7n3qqzxaj7d
bc1qftswtcyvjqknvkadtg2fqh4d3effm28pacyhpnh06k3m7sskc9vsw8nsg6
note: stdout is watch-only — public keys only, cannot spend
```

Real mainnet addresses. **The operator can fund this wallet.** See F-A1 — this is
the `--path 2of3,ripemd160=...` single-path shape, where the preimage is mandatory
for *every* spend, and no step said so.

### A.5 Coming back later with only the plate

```
$ ms hashlock --kind ripemd160 ms10hashsq0p7ja...
ms: argument 4 on ARGV (arguments count from 0, and 0 is `ms` itself) is an ms1 string [...]
      Refused BEFORE the command line was parsed; nothing was read and nothing was written.
exit=1
```

The argv guard fires and names the private channel. Following it:

```
$ ms hashlock --in plate.txt --kind ripemd160
hash:ripemd160:09e7bb5051d89788fb4e4b374126721dbcc2946b
[...]
method:          preimage supplied
source:          preimage supplied (ms1 plate)
```

Same digest — re-derivation works. Note the `phrase:` line (which carries the
"write the hash line down" instruction and the lost-kind recovery procedure) is
replaced by `method: preimage supplied`, so this path never states either. See F-B2.

**What the plate itself carries:**

```
$ ms inspect --in plate.txt
hrp: ms / threshold: 0 / tag: hash / share_index: s / prefix_byte: 0x03
payload_bytes: c3e97525442520da4cffd5f57aae3f6273990017f2e0fa30c056e32172e22016
kind: preimage
```

`kind: preimage` is the *ms-format* kind. **The hash kind is not on the plate** —
by design (it is a property of the script, not the preimage) and recoverable from
the descriptor, which the operator keeps anyway.

---

## Journey B — the other three kinds

All four run identically in structure. The differences that exist are deliberate:

### B.1 sha256 is untagged; the other three are tagged

```
--kind sha256     -> hash:3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12
--kind hash256    -> hash:hash256:98a20fc25dbcdf236fb0307e3f82cad47fca2e807f3ef82c31993549641cd488
--kind ripemd160  -> hash:ripemd160:09e7bb5051d89788fb4e4b374126721dbcc2946b
--kind hash160    -> hash:hash160:b5b72c0e6896ff59dfa99e0d1052a9c0214cd0bd
```

### B.2 Only the non-sha256 cards carry the two forward caveats

The `--kind sha256` card has **no** `requires ... support in md compose` line and
**no** `the record on stdout needs ... support in me sysw pack` line. Correct:
sha256 is supported everywhere already. The asymmetry is right.

### B.3 Only hash256 carries the extra WARNING — and that is exactly right

```
WARNING: this record's `hash256:` tag is the ONLY thing distinguishing it from a sha256 record -- both digests are 64 hex. If a tool refuses it asking for "exactly 64 hex characters", DO NOT DELETE THE TAG to satisfy it: the result is accepted as a SHA256 hashlock and the wallet it builds cannot be spent with this preimage. The tool is what needs hash256 support.
```

I checked whether `ripemd160`/`hash160` need the same warning. **They do not**, and
I verified why: deleting the tag from `hash:ripemd160:<40hex>` yields `hash:<40hex>`,
which is still refused (sha256 needs 64). Only for hash256 does tag-deletion produce
a *silently valid different record*. `ms` targeted the warning precisely at the one
kind where it bites. Proven in F-C1 below.

### B.4 `--kind` omitted: the lookup table

```
$ ms hashlock --hashlock-phrase-stdin < phrase.txt
[...card, identical to the --kind sha256 card...]
no --kind given; stdout carries the sha256 record. This preimage's digest under each kind:
  sha256     3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12
  hash256    98a20fc25dbcdf236fb0307e3f82cad47fca2e807f3ef82c31993549641cd488
  ripemd160  09e7bb5051d89788fb4e4b374126721dbcc2946b
  hash160    b5b72c0e6896ff59dfa99e0d1052a9c0214cd0bd
```

Machine-diffed against the explicit-sha256 run:

```
$ diff rec_nokind.txt rec_sha256.txt   -> STDOUT IDENTICAL
$ diff card_sha256.txt card_nokind.txt
10a11,15
> no --kind given; stdout carries the sha256 record. This preimage's digest under each kind:
[...]
```

**The undecided state is signalled only by a block appended after line 10 of a
15-line card.** Lines 1–10 assert `sha256` as though chosen: `digest:` shows the
sha256 hex, `for md compose:` shows `sha256=`, `OP_SHA256` is named, and the phrase
line says "write the hash line (sha256)". See F-B1.

The JSON surface does distinguish it properly (`"kind_specified": false` plus
`"digests_by_kind"`), and also exposes `"hash_operand": "ripemd160=09e7..."` — the
clean pasteable operand the human card lacks. See F-D1.

### B.5 Kind validation

```
$ ms hashlock --kind RIPEMD160 ...
error: invalid value 'RIPEMD160' for '--kind <KIND>': unknown hash kind "RIPEMD160": expected sha256, hash256, ripemd160 or hash160 (lowercase; case is rejected, never folded)
$ ms hashlock --kind sha1 ...
error: invalid value 'sha1' for '--kind <KIND>': unknown hash kind "sha1": expected sha256, hash256, ripemd160 or hash160 (lowercase; case is rejected, never folded)
```

Both list the valid set. Good refusals.

---

## Journey C — the wrong turn

### C.1 Cross-*length* paste: caught

```
$ md compose --wrapper wsh --path "2of3,ripemd160=<sha256 digest>"
md: path `2of3,ripemd160=3cf5d421...4c12`: ripemd160 needs 40 hex characters, lowercase
$ md compose --wrapper wsh --path "2of3,sha256=<ripemd160 digest>"
md: path `2of3,sha256=09e7bb50...946b`: sha256 needs 64 hex characters, lowercase
```

### C.2 Same-*length* cross-kind paste: silently accepted

```
$ md compose --wrapper wsh --path "2of3,sha256=<hash256 digest>"
wsh(and_v(v:multi(2,...),sha256(98a20fc25dbcdf236fb0307e3f82cad47fca2e807f3ef82c31993549641cd488)))
exit=0
$ md compose --wrapper wsh --path "2of3,hash256=<sha256 digest>"
wsh(and_v(v:multi(2,...),hash256(3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12)))
exit=0
$ md compose --wrapper wsh --path "2of3,ripemd160=<hash160 digest>"
wsh(and_v(v:multi(2,...),ripemd160(b5b72c0e6896ff59dfa99e0d1052a9c0214cd0bd)))
exit=0
$ md compose --wrapper wsh --path "2of3,hash160=<ripemd160 digest>"
wsh(and_v(v:multi(2,...),hash160(09e7bb5051d89788fb4e4b374126721dbcc2946b)))
exit=0
```

**`md` structurally cannot catch this** — it never sees a preimage. This is inherent,
not a defect in `md`. The mitigations that do exist are real and worth naming:

- the card's `for md compose:` line is **self-labelled** (`ripemd160=<hex>`), so the
  kind travels attached to the hex on a direct copy;
- `md compose` refuses two hashlocks in one path, so the operator cannot hedge;
- **a hashlock-only wallet is impossible** (see C.4), so a mis-paste never costs the
  whole wallet *unless* the hashlock is ANDed onto the only path — which is F-A1.

### C.3 Mangled-paste sweep — nothing is silently dropped

Every realistic paste corruption was refused:

```
space after comma       -> unknown option ` ripemd160`
trailing space          -> ripemd160 needs 40 hex characters, lowercase
leading space           -> k ` 2` is not a small number
uppercase hex           -> ripemd160 needs 40 hex characters, lowercase
0x prefix               -> ripemd160 needs 40 hex characters, lowercase
spaces around equals    -> unknown option `ripemd160 `
hex short by one char   -> ripemd160 needs 40 hex characters, lowercase
two hashlocks in a path -> at most one hash per path
same kind twice         -> at most one hash per path
unknown kind `sha1`     -> unknown option `sha1`
uppercase keyword       -> unknown option `RIPEMD160`
preset + hashlock       -> preset kofn-recovery admits no ripemd160= parameter
```

**No form was found in which the hashlock is accepted-but-dropped.** This is the
failure mode that would have been Critical, and it does not exist.

### C.4 A wallet cannot be hashlock-only

```
$ md compose --wrapper wsh --path "keyless,ripemd160=09e7..."
md: every wallet needs at least one path with a key
exit=1
$ md compose --wrapper wsh --path "keyless,ripemd160=09e7..." --experimental
md: every wallet needs at least one path with a key
exit=1
$ md compose --wrapper wsh --path 2of3 --path "keyless,ripemd160=09e7..." --experimental
warning: EXPERIMENTAL: path 2 has no key (bearer access to whoever holds the preimage)
wsh(or_d(multi(2,...),ripemd160(09e7bb5051d89788fb4e4b374126721dbcc2946b)))
```

A keyed path is always required. This bounds the blast radius of C.2 substantially —
**except** for the single-path AND shape (F-A1).

### C.5 The `me` refusal, and the trap inside its advice

`me sysw pack` refuses all three new kinds, with the exact wording the `ms` card
predicted — the cross-tool claim is **accurate, not stale**:

```
$ echo 'hash:ripemd160:09e7bb50...946b' | me sysw pack --out /tmp/p.bin
me: record 0 (records count from 0) is a `key:`/`hash:`/`now:`/`phrase:` record whose body fails its rule (not exactly 64 lowercase hex characters).
      record 0: hash: must be exactly 64 hex characters
      Build the record with `me sysw pack`'s helpers: [...] a hash record is `hash:` + the 32-byte digest as 64 lowercase hex; [...]
```

That refusal is expected and is **not** a finding (per brief). But the *advice
attached to it* is, and it executes:

```
### Operator obeys "a hash record is hash: + the 32-byte digest as 64 lowercase hex"
### on the HASH256 record -> deletes the tag:
$ echo 'hash:98a20fc25dbcdf236fb0307e3f82cad47fca2e807f3ef82c31993549641cd488' | me sysw pack --out /tmp/trap.bin
sealing:  NOT SEALED — no record in this payload is secret material [...]
me: appended now:1789489374 as the last record [...]
strength: no passphrase — BELOW the threshold
exit=0                                  <-- ACCEPTED, as a SHA256 hashlock

### Same operation on the RIPEMD160 record (40 hex):
$ echo 'hash:09e7bb5051d89788fb4e4b374126721dbcc2946b' | me sysw pack --out /tmp/trap2.bin
me: record 0 [...] hash: must be exactly 64 hex characters      <-- still refused
```

See F-C1. This also *proves* B.3: the tag-deletion trap exists for hash256 alone.

---

## Journey D — copy/paste reality

The card line is `for md compose:  --path ... ripemd160=<hex>`. What a literal copy does:

```
$ md compose --wrapper wsh --path ... ripemd160=09e7bb50...946b
error: unexpected argument 'ripemd160=09e7bb5051d89788fb4e4b374126721dbcc2946b' found
Usage: md compose [OPTIONS] --wrapper <WRAPPER> <--path <PATH>|--preset <PRESET>>
exit=2

$ md compose --wrapper wsh --path '... ripemd160=09e7bb50...946b'
md: path `... ripemd160=09e7bb5051d89788fb4e4b374126721dbcc2946b`: expected <k>of<n> or keyless
exit=1

$ md compose --path ... ripemd160=09e7bb50...946b          # exactly as the card shows, no wrapper
error: unexpected argument 'ripemd160=09e7bb5051d89788fb4e4b374126721dbcc2946b' found
exit=2
```

**The literal `...` does break the paste, in all three spellings, and every one fails
loudly with a non-zero exit.** No silent misbehaviour. What the operator must supply
and nothing tells them: a `--wrapper`, a `<k>of<n>` in place of `...`, and that the
hashlock joins the path with a **comma, not a space**. The comma rule is the one that
is genuinely non-obvious from the card, since the card renders `... ripemd160=` with a
*space*. See F-D1.

---

## FINDINGS

### F-A1 — *Important* — A wallet whose every spend needs the preimage is composed, described and addressed in total silence

**Classification: warning.**

`--path 2of3,ripemd160=HEX` as the *only* path lowers to
`wsh(and_v(v:multi(2,...),ripemd160(...)))`. Every spend requires the preimage; there
is no keyed escape and, unlike a timelock, it never matures. Walked end to end in A.4:
`md compose`, `md descriptor` and `md address` all succeed and print real mainnet
addresses, and **not one of them mentions the preimage.**

The only notes printed are about *keys* — `note: stdout is a keyless descriptor
template (no keys)` and `note: stdout is watch-only — public keys only, cannot
spend`. A hurried operator reads those as the tool having spoken.

The asymmetry is the tell: `md` **does** warn on the `keyless` shape —
`warning: EXPERIMENTAL: path 2 has no key (bearer access to whoever holds the
preimage)` — which is the hashlock as an *extra* way in (a security risk). It is
silent on the `and_v` shape, which is the hashlock as the *only* way in (a
funds-loss risk). The direction that loses money is the unwarned one.

Compare a timelock in the same position: `--path 2of3,older=26280` is equally silent,
but a timelock matures on its own. A lost or wrong-kind preimage is permanent.

**Earns a change?** Yes. Wrong outcome = permanently unspendable funds at a real
mainnet address. That is decisively worse than telling the operator nothing, and
`md` already owns the warning idiom.

**Provenance, stated honestly:** this shape predates the cycle — `sha256=` behaved
the same before `hashkinds-p1`. The cycle amplifies it by adding three kinds, two of
which collide in digest width with existing ones (C.2), so the chance of arriving
here with the *wrong* digest is now materially higher.

### F-C1 — *Important* — `me`'s remediation advice instructs the operator to do exactly what `ms`'s WARNING forbids, and it works

**Classification: warning (or documentation).**

On a `hash:hash256:<64hex>` record, `me sysw pack` refuses and advises: *"a hash
record is `hash:` + the 32-byte digest as 64 lowercase hex"*. Followed literally,
that means deleting the `hash256:` tag. Proven in C.5: the untagged record is then
**accepted, exit 0**, and packed as a **sha256** hashlock — committing the device
payload to a different digest than the wallet was composed with.

`ms` anticipated this exactly and warns in advance, quoting `me`'s error string
verbatim. But the two tools now give contradictory instructions at the same moment,
and the dangerous one is the one on screen when the operator is stuck; the `ms`
warning was on stderr, possibly many commands earlier.

**Why Important and not Critical:** `ms` does pre-warn, with precise targeting; the
window closes when phase 3 ships; and the brief scopes the refusal itself as expected.
What is in scope — and what this is — is *what one tool says about a step in the
other*.

**Earns a change?** Yes. A one-line carve-out in `me`'s advice ("a record carrying a
`<kind>:` tag needs `<kind>` support — do not remove the tag") costs nothing, and the
wrong outcome is a silently mis-committed payload. Shipping phase 3 also closes it.

*Note:* this trap is **created by** this cycle. Before tagged records existed, `ms`
only ever emitted `hash:<64hex>` and the advice was harmless.

### F-B1 — *Minor* — The no-`--kind` card is byte-identical to a `--kind sha256` card except for a block appended at the end

**Classification: documentation only.**

Machine-diffed in B.4: stdout identical, card differs only by lines 11–15. Lines 1–10
assert `sha256` — `digest:`, `for md compose: ... sha256=`, `OP_SHA256`, "the hash
line (sha256)" — so a card produced with **no decision made** reads exactly like one
where sha256 was chosen. Omitting `--kind` silently *defaults* rather than marking
the card undecided.

**Earns a change?** **No.** The trailing block is present, explicit ("no --kind
given"), and lists all four digests; the JSON surface flags it properly with
`kind_specified: false`. The design intent is documented in `--help` ("a plate cut
before this existed carries no kind, and a lookup beats an impossible check"), and
sha256 is the right default. Logged so the asymmetry is on record, not to drive a fix.

### F-B2 — *Minor* — The lost-kind recovery procedure is printed only on the phrase path

**Classification: documentation only.**

The instruction *"if the hash line is lost, re-run with no --kind and match the digest
against your descriptor"* lives on the `phrase:` line. On `--hex`, `--in` and
`--random` that line is replaced by `method: preimage supplied`, so the operator who
created a random preimage is never told either to record the kind or how to recover it.
Confirmed in A.5 and in the `--random` run, whose `--out` file contains only the ms1
string (`ms10hashsqwx3d4...`) with no kind.

**Earns a change?** **No** — and this is the case I most nearly argued myself into.
The kind is not lost data: it is written in the descriptor (`ripemd160(...)` is
literally in the template the operator engraves as md1), and the recovery is
self-discovering — running with no `--kind` prints the four-digest table unprompted.
Nothing is unrecoverable, so silence here is not worse than the alternative.

### F-D1 — *Minor* — The card's `--path ...` fragment is not pasteable, while the JSON's `hash_operand` is

**Classification: documentation only.**

`for md compose:  --path ... ripemd160=<hex>` fails in all three literal spellings
(Journey D). The card never states the two things an operator must supply — a
`--wrapper`, and that the hashlock attaches with a **comma**, where the card shows a
space. Meanwhile `--json` already emits the clean form:
`"hash_operand": "ripemd160=09e7bb5051d89788fb4e4b374126721dbcc2946b"`.

**Earns a change?** **No.** Every failure is loud and non-zero-exit; the `...` reads
as an ellipsis to any human; and `md compose --help` documents the comma grammar
fully. Wrong outcome = one confused retry, which is not worse than telling the
operator nothing. Worth a line in a walkthrough doc if one is written.

### F-M1 — *Minor* (secret-handling) — the ms1 plate on argv is refused, and the refusal is excellent

Not a defect — recorded because the class is logged on sight. `ms hashlock <ms1>`
refuses before parsing, names the private channels (`--in FILE`, `-`), and supplies
per-shell history-purge recipes including the zsh `history -d` trap. Working as
intended.

---

## DIVERGENCES I AM EXPLICITLY NOT ACTING ON

| divergence | class | why no change |
| --- | --- | --- |
| Literal `--path ...` paste fails (F-D1) | documentation | Fails loudly, exit 1/2, no silent damage. `...` reads as an ellipsis. |
| No-`--kind` card looks like a sha256 card (F-B1) | documentation | Trailing block is explicit and lists all four; JSON carries `kind_specified: false`; sha256 is the right default. |
| Recovery procedure absent on non-phrase paths (F-B2) | documentation | Kind is in the descriptor; recovery self-discovers via the no-`--kind` table. Nothing unrecoverable. |
| `md compose --json` has no dedicated hashlock field | not our concern | The template string is the contract; the digest is in it and greppable. |
| Same-length cross-kind paste accepted by `md` (C.2) | not our concern | `md` never sees a preimage — structurally uncatchable there. The card's self-labelled operand is the right mitigation. The *consequence* is covered by F-A1. |
| `me sysw pack` refuses ripemd160/hash160/hash256 records | not our concern | Explicitly out of scope per brief; phase 3 not built. Only the *advice text* (F-C1) is in scope. |
| The hash kind is not carried on the ms1 preimage plate | not our concern | Deliberate: the kind is a property of the script, not the preimage, and lives in the md1 descriptor. |
| `ms` cards claim `md`/`me` "may not have shipped this yet" | not our concern | Verified **accurate**, not stale: `md` on this branch accepts the operand, and `me` refuses with the exact quoted string. Self-testing and self-correcting. |
| Uppercase keyword / hex, `0x` prefix, spaces (C.3) | refusal (already) | All already refused with a precise message. |
| Preset cannot take a hashlock | refusal (already) | `preset kofn-recovery admits no ripemd160= parameter` — clear and correct. |

---

## VERIFIED-GOOD (things this walk confirms rather than faults)

1. **The seam closes.** Phase 2's `unknown option ripemd160` finding is fixed; the
   card's operand is accepted verbatim by `md compose`.
2. **No silent-drop path exists.** 12 mangled pastes, all refused.
3. **The hash256 WARNING is precisely targeted** — and C.5 proves the tag-deletion
   trap bites hash256 *only*, exactly as the warning scopes it.
4. **All four kinds round-trip the engraving format** (compose → encode → decode).
5. **A hashlock-only wallet is impossible**, bounding cross-kind mis-paste damage.
6. **Cross-tool claims in the `ms` cards are accurate**, including `me`'s exact error
   wording.

---

## COUNTS

| severity | count | ids |
| --- | --- | --- |
| Critical | **0** | — |
| Important | **2** | F-A1, F-C1 |
| Minor | **4** | F-B1, F-B2, F-D1, F-M1 |
| Nit | 0 | — |

Divergences examined: 21. Recommended for change: 2. Explicitly declined: 10.

## VERDICT

# NOT GREEN

Two Importants block: **F-A1** (a mandatory-preimage wallet is composed, described
and addressed in silence, reaching a fundable mainnet address) and **F-C1** (`me`'s
remediation text instructs the tag deletion that `ms` warns is unspendable, and it
is accepted).

Neither is a correctness defect in the phase-1 code — both are *missing things at
moments*, which is what this lens exists to find. The `hashkinds-p1` branch's own
parse, validation and lowering behaviour came through the walk clean: the seam
closes, nothing is silently dropped, and the round trip holds.
