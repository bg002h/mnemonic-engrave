# Hash-kinds operator journeys — the three host CLIs (A–E)

**VERDICT: all four kinds reach a payload, but two moments are actively wrong — `ms hashlock`'s digest guard directs the operator into a silent wrong result at `--hex`, and a keyless hashlock cannot reach `md descriptor`/`md address` at all through a refusal that names a flag the verb does not have.**

Walked 2026-09-16 against `ms 0.19.0` (`mnemonic-secret` `65e200f`), `me 0.10.0`
(`mnemonic-engrave` `a3c406c9`), `md 0.15.0` / `md-codec 0.43.0`
(`descriptor-mnemonic` `de629c85`). Every command below was RUN; output is real,
trimmed only for length. No repo was modified; scratch in `/tmp/hkj`.

Fixture: phrase `correct horse battery staple`, `hardened` method, preimage
`c3e97525442520da4cffd5f57aae3f6273990017f2e0fa30c056e32172e22016`, giving

    sha256     3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12
    hash256    98a20fc25dbcdf236fb0307e3f82cad47fca2e807f3ef82c31993549641cd488
    ripemd160  09e7bb5051d89788fb4e4b374126721dbcc2946b
    hash160    b5b72c0e6896ff59dfa99e0d1052a9c0214cd0bd

---

## A. The whole host path, once per kind

**Step A1 — derive the digest. `ms hashlock --kind <k>`.**

*What they have:* a phrase in a file. *What the tool does:* all four kinds
produce a card and a `hash:` record; no divergence in mechanism.

    $ ms hashlock --hashlock-phrase-stdin --kind ripemd160 < phrase.txt
    hash:ripemd160:09e7bb5051d89788fb4e4b374126721dbcc2946b
    digest:          09e7bb5051d89788fb4e4b374126721dbcc2946b
    for md compose:  --path ... ripemd160=09e7bb5051d89788fb4e4b374126721dbcc2946b
                     requires `ripemd160=` support in `md compose`. If it answers
                     "unknown option `ripemd160`", that support has not shipped in
                     your `md` yet -- the preimage and digest above are still correct
    The preimage must be exactly 32 bytes ... before OP_RIPEMD160 (composer spec §8i, F-132)

The card is per-kind correct: the opcode line reads `OP_RIPEMD160` / `OP_HASH256`
/ `OP_HASH160` / `OP_SHA256`, the write-down line names the kind, and the two
"your tool may be too old" notes appear for the three new kinds and not for
`sha256`. The extra `WARNING: this record's hash256: tag is the ONLY thing
distinguishing it from a sha256 record` appears for `hash256` **only**, which is
exactly right.

*Question 3 — what else might they do?* They could paste the `for md compose:`
line verbatim, ellipsis and all. See **C17**. They could omit `--kind`; stdout
then carries the **sha256** record and the four-kind listing goes to stderr —
deliberate, but the command's own `EXAMPLES:` block teaches the pipeline
`ms hashlock --hashlock-phrase-stdin < phrase.txt | me sysw pack --out payload.bin`
with no `--kind`, so the documented one-liner always packs sha256 (**F-553**).

**Step A2 — compose. `md compose`.** All four kinds identical, no divergence:

    $ md compose --wrapper wsh --path 2of3 --path keyless,ripemd160=09e7...946b --experimental
    warning: EXPERIMENTAL: path 2 has no key (bearer access to whoever holds the preimage)
    wsh(or_d(multi(2,@0/48'/0'/0'/2'/<0;1>/*,@1/...,@2/...),ripemd160(09e7...946b)))
    note: stdout is a keyless descriptor template (no keys)

The `hashlock-gated` preset also covers all four, with a good staircase of
refusals that each name the next missing parameter:

    $ md compose --wrapper wsh --preset hashlock-gated
    md: preset hashlock-gated needs one of sha256=<64 hex>, hash256=<64 hex>, ripemd160=<40 hex> or hash160=<40 hex>
    $ md compose --wrapper wsh --preset hashlock-gated,ripemd160=09e7...946b
    md: preset hashlock-gated needs older=<n>
    $ md compose --wrapper wsh --preset hashlock-gated,ripemd160=09e7...946b,older=26280 --experimental
    wsh(or_i(and_v(v:pkh(@0/...),ripemd160(09e7...946b)),and_v(v:pkh(@1/...),older(26280))))   # and the other 3 kinds likewise

**Step A3 — get a descriptor. THE PATH ENDS HERE for a keyless hashlock, all four kinds.**

    $ md descriptor --template "$T" --key @0=xpub.. --key @1=xpub.. --key @2=xpub..
    md: template parse error: miniscript parse failed: All spend paths must require a
        signature ... `md compose` refuses the same shape without --experimental;
        pass --experimental here to encode it as EXPERIMENTAL
    EXIT=1

    $ md descriptor --experimental --template "$T" --key ...
    error: unexpected argument '--experimental' found
    EXIT=2

`--experimental` exists on `encode`, `verify` and `compose`; it does **not**
exist on `descriptor` or `address` (measured: `md <verb> --help | grep -c -- --experimental`
gives encode 1, verify 3, compose 1, descriptor 0, address 0, decode/inspect/
compile/decompose/repair 0). The other route loops back to the same place:

    $ md descriptor md1fqt5gqs9q... md1fqt5gqs2w...
    md: descriptor requires wallet-policy mode ... or rebuild the policy from a
        template with --template <T> --key @i=XPUB

This is the known `md-descriptor-address-template-lack-experimental`
(descriptor-mnemonic `design/FOLLOWUPS.md:2925`), reported here as
**CONFIRMATION with two pieces of new evidence** (**F-547**): (i) the refusal
*prescribes a flag the verb rejects with exit 2*, which no prior write-up
records, and (ii) that entry's closing claim — *"the card-input routes are
unaffected, so nothing engraved is unreadable"* — holds only for a **keyed**
card; the **keyless** card route refuses too, and its remedy text points back at
`--template --key`. A keyless hashlock is precisely the shape this cycle's
preimage plates exist to serve.

**Step A3' — the route that does work.** A **keyed** hashlock path needs no
`--experimental` and goes end to end through `--template` for all four kinds:

    $ md compose --wrapper wsh --path 2of3 --path 1of1,<k>=<H>   # then
    $ md address --template "$T" --key @0=.. --key @1=.. --key @2=.. --key @3=..
    sha256     bc1qvhvjsel9ffq0p7wfc7fj3awrrr4qeey7avu6dk9lpvvduwkegekqde7swu
    hash256    bc1qu48yc8q0vwjksgu6tdfdczqarxj68ah5v5tzgp4v8tdtnhsrnfpsvyx842
    ripemd160  bc1qr7vlkcxumzf3gyjeam802rqgvs6eva3lmlkcrt0hj20h6an3fw8sx0rj3m
    hash160    bc1qgn4fp5hmyvdts0vveavtgc0smklq2z58cpza38h4sxmvkpdwdnes4s3xpn

For the **keyless** shape the operator must detour through `md encode
--experimental` to mint a keyed md1 card and then read the descriptor back off
the card. That detour works for all four kinds (addresses
`bc1qpk9epcl…`, `bc1qzmvrd8j…`, `bc1q8j0twst…`, `bc1qk4exr4l…`), but nothing in
`md compose`'s output, `md descriptor`'s refusal, or `md address`'s refusal names
it. `md decompose --emit commands` — advertised in `md encode`'s own error as
*"the mint commands, ready to run"* — emits commands that are **not** ready to
run for a keyless path; it knows, and says so in a trailing note, but does not
add the flag (**F-548**).

**Step A4 — pack. `me sysw pack`.** All four kinds pack; the three new kinds each
draw a kind-tag note. Digests: `11a0 6e4f…`, `44c8 e638…`, `705f 9cfd…`,
`e323 77b7…` — four distinct payloads, no collapse.

One defect in the note itself: for `ripemd160` and `hash160` it still says
*"an untagged record is read as sha256, and a hash256 digest is also 64 hex, so
stripping it succeeds silently"* — which is false for the kind it is attached to.
Measured both sides:

    $ printf 'hash:09e7bb5051d89788fb4e4b374126721dbcc2946b\n' | me sysw pack ...
    me: record 0 ... hash: sha256 needs exactly 64 lowercase hex characters       EXIT=4
    $ printf 'hash:98a20fc2...641cd488\n' | me sysw pack ...
    (accepted, exit 0, NO kind warning at all)

So stripping succeeds silently for `hash256` and is caught for the 40-hex kinds.
`ms`'s card gets this right (the extra WARNING fires for `hash256` only); `me`'s
pack-time note does not (**F-549**).

---

## B. The operator who holds only a digest

*What they have, exactly:* `09e7bb5051d89788fb4e4b374126721dbcc2946b` on paper.
40 hex. No phrase, no preimage, no note of the kind.

**B1 — they reach for `ms`, the hashlock tool.**

    $ echo 09e7bb...946b | ms hashlock --hex -
    error: --hex is 40 characters; a hashlock preimage is exactly 32 bytes
           (64 hex characters) -- see the composer spec's §8i
    EXIT=1

Safe, but it names the preimage width rule and offers no remedy. It does not say
the one thing that resolves the moment: *40 hex is a ripemd160/hash160 **digest**,
and `ms hashlock` has no verb that takes a digest — it goes straight to `md
compose` and `me sysw pack`.* (Part of **F-550**.)

**B2 — they try `md compose` without naming a kind.**

    $ md compose --wrapper wsh --path 2of3 --path keyless,09e7bb...946b --experimental
    md: path `keyless,09e7bb...946b`: option `09e7bb...946b` needs a value

Clap-shaped; does not say "name a hash kind", does not list the four.

**B3/B4 — nothing tells them which kind.** `ripemd160=` and `hash160=` both
accept the same 40 hex and both compose, silently, to different wallets. Same at
the pack boundary: `hash:ripemd160:<H>` and `hash:hash160:<H>` both pack, to
different payload digests (`705f 9cfd…` vs `7f4b ab08…`).

*Classification:* **not our concern** for the ambiguity itself — the width is
genuinely ambiguous and the operator asserts the kind. It is self-correcting:
the wrong guess yields an address that does not match their funds. **B1's
refusal is the part that earns a change**, because it is the one moment a tool
could hand them the right next command and instead recites a width.

---

## C. The operator who mistypes

`ms --kind` — exemplary; every spelling gets the enumeration and the case rule:

    --kind RIPEMD160 / rmd160 / sha-256 / Sha256   (all exit 64)
    error: invalid value 'RIPEMD160' for '--kind <KIND>': unknown hash kind
      "RIPEMD160": expected sha256, hash256, ripemd160 or hash160
      (lowercase; case is rejected, never folded)

`me sysw pack` — equally good; every mistyped record gets the four kinds, the
case rule, and the do-not-strip-the-tag guard:

    hash:RIPEMD160:<40hex>  ->  hash: unknown hash kind; expected `hash:<hex>` (sha256)
                                or `hash:<kind>:<hex>` with kind hash256, ripemd160
                                or hash160, lowercase                            EXIT=4
    hash:ripemd160:<UPPER>  ->  hash: ripemd160 needs exactly 40 lowercase hex    EXIT=4
    hash:sha256:<40hex>     ->  hash: sha256 needs exactly 64 lowercase hex       EXIT=4
    hash:rmd160:<40hex>     ->  (unknown hash kind, as above)                     EXIT=4

`md compose` — the weakest of the three, and it is the **first** tool the ms card
sends them to:

    C7  keyless,ripemd160=09E7BB...946B  (uppercase digest, correct width)
        md: ... ripemd160 needs 40 hex characters, lowercase
    C8  keyless,RIPEMD160=09e7bb...946b
        md: ... unknown option `RIPEMD160`
    C9  keyless,rmd160=...
        md: ... unknown option `rmd160`
    C10 keyless,sha256=correct horse battery staple
        md: ... sha256 needs 64 hex characters, lowercase

- **C7** leads with the width, which the input already satisfies. An operator
  counts 40 and is stuck; `lowercase` is a trailing word, not the named
  violation. Compare `me`'s *"needs exactly 40 lowercase hex characters"* and
  `ms`'s *"case is rejected, never folded"*.
- **C8/C9 are actively misleading**, and this cycle's own card text is what makes
  them so. The `ms hashlock` card tells the operator: *"If it answers `unknown
  option \`ripemd160\``, that support has not shipped in your `md` yet."* A
  correct, current `md` prints exactly that string for a **case** error. The
  operator is taught to read it as "upgrade `md`" and will go hunting for a
  release instead of pressing shift. `md compose` also never lists the four
  valid spellings here. (**F-551**.)
- **C10** does not observe that the value is a phrase, or point at
  `ms hashlock --hashlock-phrase-stdin`.

**C17 — the verbatim paste.** The card prints `for md compose: --path ...
ripemd160=<H>` where `...` is an ellipsis. Pasted as written:

    $ md compose --wrapper wsh --path 2of3 --path ... ripemd160=09e7bb...946b --experimental
    error: unexpected argument 'ripemd160=09e7bb...946b' found

Nothing points at the ellipsis. **Documentation only** — a Nit (**F-554**).

**C20 — THE BAD ONE. F-539's guard prescribes the failure it exists to prevent.**

*What they have:* a 64-hex sha256 digest from a coordinator. They type it where
a hashlock value goes.

    $ printf 3cf5d421...4c12 | ms hashlock --hashlock-phrase-stdin --kind sha256
    error: that phrase is 64 hex characters, the width of a digest. Hashing it
      commits the wallet to the ASCII of those characters, NOT to the digest they
      spell -- so if you meant to use a digest you already hold, pass it with
      --hex instead. If you really meant this as a phrase, re-run with
      --phrase-looks-like-digest-ok.
    EXIT=1

They do exactly what it says:

    $ printf 3cf5d421...4c12 | ms hashlock --hex - --kind sha256
    hash:98a20fc25dbcdf236fb0307e3f82cad47fca2e807f3ef82c31993549641cd488
    digest:          98a20fc25dbcdf236fb0307e3f82cad47fca2e807f3ef82c31993549641cd488
    for md compose:  --path ... sha256=98a20fc2...641cd488
    preimage (ms1):  ms10h ashsq v70t4 ppete 2nj8t nhsag qyxd6 naga0 xh2tc scdmq 9n6xl 9hpfx pytq9 l44d9 kjs2v
    preimage (hex):  3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12
    EXIT=0     (no warning of any kind)

Exit 0, a clean card, and a digest that is **not the one in their hand**. Their
public digest has been re-labelled as a secret preimage and given an ms1 plate
string to engrave. Two failures compound: the composed wallet is not their
wallet, and the "preimage" they will cut into metal is a value that was on a
coordinator's screen. `ms hashlock` has **no** flag that takes a digest, so the
guard's remedy presupposes a surface that does not exist. The 40-hex arm of the
same sentence lands in B1's dead end instead — wrong, but safe.

Source: `crates/ms-cli/src/cmd/hashlock.rs:237-247` (the guard);
`:276`, `:284`, `:308` are `--hex`'s only guards — width, hexness, ms1-kind.
None looks for a digest. (**F-550**.)

---

## D. Recovery, a year later

**This journey is in good shape.** Three readers, three answers, all naming the kind.

    $ ms decode ms10hashsq0p7jaf9gsj...89hqj
    kind:      preimage (hashlock, 32 bytes / 64 hex characters)
    preimage:  c3e97525442520da4cffd5f57aae3f6273990017f2e0fa30c056e32172e22016
    digests:
      sha256     3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12
      hash256    98a20fc25dbcdf236fb0307e3f82cad47fca2e807f3ef82c31993549641cd488
      ripemd160  09e7bb5051d89788fb4e4b374126721dbcc2946b
      hash160    b5b72c0e6896ff59dfa99e0d1052a9c0214cd0bd
               (a preimage carries no hash kind; match the one your wallet uses)

**F-534 is FIXED** — `ms decode` no longer answers only for sha256, in both text
and `--json` (`digests_by_kind` plus `digest_kind`). The FOLLOWUPS entry still
reads as open ("Owning phase: **phase 3**") and should be marked closed
(**F-555**). The same four-digest fallback fires from `ms hashlock` with no
`--kind`, as the card promises.

    $ md decode <the 7 md1 chunks>
    wsh(or_d(multi(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*),ripemd160(09e7bb...946b)))
    $ me sysw show pay_ripemd160.bin
    public record 0: ripemd160 hashlock (hash:) — 09e7bb50..bcc2946b

So: plate → all four candidate digests; card → the kind and the digest; payload →
the kind. The operator can close the loop by matching. **The journey completes.**

One gap. `me sysw show` takes **no options at all** (`Usage: me sysw show <FILE>`
— no `--json`, no full form) and elides the digest to 4+4 bytes. That is enough
to *confirm* a candidate but not enough to *retype* the digest into `md compose`
when the policy card is the thing that was lost. The full value is in the
container in the clear (`strings pay_ripemd160.bin` →
`hash:ripemd160:09e7bb5051d89788fb4e4b374126721dbcc2946b`), so the answer exists
and only the reader withholds it. (**F-552**.)

---

## E. The handoff — `ms` record → `me sysw pack` → device

**E1 — `ms` warns correctly at the point of emission.**

    $ ms hashlock --hashlock-phrase-stdin --kind ripemd160 --emit-record < phrase.txt
    stdout: hash:ripemd160:09e7bb5051d89788fb4e4b374126721dbcc2946b
    stderr: record (phrase): phrase:68617264656e65642c636f7272656374...
    stderr: NOTE: the phrase: record above carries the METHOD, not the hash kind.
            Packed and read back on the device it derives a SHA256 lock, not
            ripemd160. The kind travels in the hash: record and on the plate's
            own `hash:` line -- keep those with it.

The note is accurate. Confirmed on the device side (read-only):
`gui/composer_hashlock.go:147` is `h := hashlockLockOf(md.KindSha256, &x)` — a
payload `phrase:` record always yields a sha256 lock, with no kind picker; the
preimage-plate route is the same at `gui/composer_hash.go:412`.

**E2 — `me sysw pack`, the consumer, holds both records and says nothing.**

    $ cat e_records.txt
    hash:ripemd160:09e7bb5051d89788fb4e4b374126721dbcc2946b
    phrase:68617264656e65642c636f727265637420686f727365206261747465727920737461706c65
    $ me sysw pack --in e_records.txt --pack-preimage --no-passphrase --no-now --out e_pay.bin
    me: note — this payload carries a kind-tagged hash record (ripemd160). ...
    me: WARNING — this payload carries a hashlock PREIMAGE. ...
    me: WARNING — this payload carries secret material with weak or no passphrase protection.
    digest:   4035 7d1f a833 af5f fe38 860c 65b0 2d8d
    EXIT=0

Three warnings, none about the contradiction. `me sysw show` is silent too:

    public record 0: ripemd160 hashlock (hash:) — 09e7bb50..bcc2946b
    secret record 1: hashlock phrase (phrase:) — not shown

The payload says `ripemd160` in one record and will build `sha256` from the
other. `me` already owns the derivation
(`crates/me-cli/src/sysw/composer_records.rs:85-88`,
`HashlockMethod::preimage`), so it can derive the phrase and compare against
every `hash:` record under all four kinds in one pass; no cross-check exists
(grepped). The device does eventually catch it — `payloadStatesDigest` →
`composerCopyHashlockReconcile` at `gui/composer_hashlock.go:166-169` — so this
costs a flash-and-compose cycle rather than funds, which is why it is Important
rather than Critical. Distinct from **F-535** (which proposes a wire-format
change, and per **F-546** a format change moves three repos at once): this needs
no format change at all. (**F-556**.)

**E3 — `me bundle` is kind-blind, and its plate count is wrong for a hashlock wallet.**

Confirms **F-543** with a measurement it did not have:

    $ me bundle --in card_sha256.txt    --manifest man_sha256.json
    $ me bundle --in card_ripemd160.txt --manifest man_ripemd160.json
    manifests identical after blanking strings/ids: True
    $ grep -ci 'hash|preimage|ripemd' man_ripemd160.json  ->  0
    $ diff err_sha256.txt err_ripemd160.txt   ->  only the manifest FILENAME differs

    me: backup needs 8 plates (7 public + ms1 on device):
      plate 1/8 .. 7/8  md1 policy  → push via NFC & engrave
      plate 8/8         ms1 secret  → TYPE ON DEVICE ...

For a keyless hashlock wallet the preimage/HASHLOCK-PHRASE plate is the only
thing that opens path 2, and *"backup needs 8 plates"* is a **completeness
claim** that omits it. `me bundle` decodes the md1 (it reports
`chunk_set_id 0x79e84`, `integrity set-verified`), so the `ripemd160(...)` node
is visible to it. (**F-557** — an escalation of F-543's severity, not a new gap.)

---

# FOLLOW-UP LIST

Numbers proposed from **F-547**. Severity follows the project rule: wrong
results / unmet guarantees block; secret-handling never does.

### F-547 — `md descriptor`/`md address` refuse a keyless hashlock and prescribe a flag they do not have — CONFIRMS `md-descriptor-address-template-lack-experimental`
*Repro:* `md compose --wrapper wsh --path 2of3 --path keyless,sha256=<64hex> --experimental`,
then `md descriptor --template "$T" --key @0=.. --key @1=.. --key @2=..` →
`pass --experimental here` (exit 1); then `md descriptor --experimental …` →
`error: unexpected argument '--experimental' found` (exit 2). Identical for all
four kinds and for `md address`. New evidence vs the existing entry: (a) the
refusal *prescribes* the missing flag, so the operator's next action is
guaranteed to fail with a clap error; (b) that entry's *"the card-input routes
are unaffected"* covers only **keyed** cards — `md descriptor <keyless md1>`
refuses too and its remedy text points back at `--template --key`, closing the
loop. **Classification: refusal (make it a real one, or add the flag).**
**Worse than saying nothing? YES** — a wrong remedy costs more than no remedy.
Cheapest partial fix without threading `--experimental`: name the working detour
(`md encode --experimental` → read back off the card).

### F-548 — `md decompose --emit commands` says "ready to run" and emits commands that are not, for a keyless path
*Repro:* `md decompose "<concrete descriptor with or_d(...,ripemd160(H))>" --emit commands`
→ both routes print `md encode '<template>' --key … --fingerprint …` with no
`--experimental`; running route 1 verbatim gives *"All spend paths must require a
signature"* (exit 1). The emitter already knows — it appends
*"note: `md encode` may not accept this template as printed …"* — and does not
add the flag. **Classification: default (emit the flag the note describes).**
**Worse than saying nothing? Marginal** — the note is present, so this is a
polish item. Minor.

### F-549 — `me sysw pack`'s kind-tag note tells a `ripemd160`/`hash160` operator a fact that is false for their kind
*Repro:* `printf 'hash:ripemd160:09e7bb5051d89788fb4e4b374126721dbcc2946b\n' | me sysw pack …`
→ note ends *"an untagged record is read as sha256, and a hash256 digest is also
64 hex, so stripping it succeeds silently and commits the payload to a different
digest than the wallet."* Measured: stripping a 40-hex tag is **refused**
(`hash: sha256 needs exactly 64 lowercase hex characters`, exit 4); only
`hash256` strips silently (exit 0, and with **no** warning of its own). `ms`
already gets this right — its extra WARNING paragraph fires for `hash256` only.
**Classification: warning (scope the hash256 clause to hash256).**
**Worse than saying nothing? No** — it over-warns rather than under-warns, but it
teaches a false rule about the 40-hex kinds and undermines a correct refusal.
Minor.

### F-550 — F-539's guard directs the operator into a silent wrong result at `--hex`
*Repro, two commands:*
```
printf 3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12 \
  | ms hashlock --hashlock-phrase-stdin --kind sha256
# -> "... if you meant to use a digest you already hold, pass it with --hex instead."
printf 3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12 \
  | ms hashlock --hex - --kind sha256
# -> EXIT 0, digest: 98a20fc2...641cd488, preimage (ms1): ms10hashsqv70t4ppete...
```
`--hex` takes a **preimage**, and `ms hashlock` has no digest-taking surface at
all, so the remedy is unreachable by construction. The operator gets a wallet
committed to a digest they never chose and an ms1 plate whose "secret" is their
public digest. The 40-hex arm of the same sentence instead dead-ends at
*"--hex is 40 characters; a hashlock preimage is exactly 32 bytes"* — safe, but
still a wrong instruction. Source: `crates/ms-cli/src/cmd/hashlock.rs:237-247`;
`--hex`'s guards are `:276`, `:284`, `:308` and none looks for a digest.
**Classification: refusal — rewrite the remedy clause.** A digest goes to
`md compose --path …,<kind>=<digest>` and `me sysw pack hash:[<kind>:]<digest>`;
`--hex` is the preimage channel and must be named as such in the same sentence.
**Worse than saying nothing? YES, unambiguously** — the tool is the thing
directing them into it, and the outcome (wrong wallet + a bearer plate of public
data) is discovered only at spend time. **Important.** No wire-format change;
one string plus, optionally, a digest-shaped advisory on `--hex` mirroring
`--phrase-looks-like-digest-ok`. RUST-PRIMARY: `ms-codec`/`ms-cli` first.

### F-551 — `md compose`'s hash-kind refusals are the weakest of the three CLIs, and the ms card makes one of them misleading
*Repro:*
```
md compose --wrapper wsh --path 2of3 --path keyless,RIPEMD160=09e7bb…946b --experimental
  -> md: path `…`: unknown option `RIPEMD160`
md compose --wrapper wsh --path 2of3 --path keyless,ripemd160=09E7BB…946B --experimental
  -> md: path `…`: ripemd160 needs 40 hex characters, lowercase   # input IS 40 chars
md compose --wrapper wsh --path 2of3 --path keyless,sha256=correct horse battery staple --experimental
  -> md: path `…`: sha256 needs 64 hex characters, lowercase
```
`ms hashlock`'s card says *"If it answers `unknown option \`ripemd160\``, that
support has not shipped in your `md` yet"* — so a current `md` printing that
string for a **case** error teaches the operator to go looking for a newer
release. And the width-first wording of the uppercase-digest case names a rule
the input already satisfies. `me sysw pack` gets both right
(*"needs exactly 40 lowercase hex characters"*; *"expected … lowercase"* plus the
four kinds); `ms --kind` gets both right (*"case is rejected, never folded"*).
**Classification: refusal (align md's two messages with the sibling tools —
enumerate the four kinds, name the violated clause first).**
**Worse than saying nothing? YES for the case arm** — it sends the operator on a
version hunt for a tool that is already correct. Important-adjacent; filed
Minor because no wrong artifact is produced.

### F-552 — `me sysw show` is the only payload reader and elides the digest, with no `--json` or full form
*Repro:* `me sysw show --help` → `Usage: me sysw show <FILE>`, no options.
`me sysw show pay_ripemd160.bin` → `public record 0: ripemd160 hashlock (hash:) —
09e7bb50..bcc2946b`. The full value is in the container in the clear
(`strings pay_ripemd160.bin` → `hash:ripemd160:09e7bb…946b`). Enough to confirm a
candidate; not enough to retype into `md compose` when the policy card is what
was lost. **Classification: default (add `--json`, or print the full digest for
the public records).** **Worse than saying nothing? No** — it is a recovery
convenience, not a wrong answer. Minor.

### F-553 — the `ms hashlock` EXAMPLES pipeline omits `--kind`, so the documented one-liner always packs sha256
*Repro:* `ms hashlock --help` → `EXAMPLES: ms hashlock --hashlock-phrase-stdin <
phrase.txt | me sysw pack --out payload.bin`. With no `--kind`, stdout carries
`hash:<sha256>` and the four-kind listing goes to stderr (visible, but stdout is
what the pipe consumes). An operator whose wallet is `ripemd160` following the
documented line packs a sha256 payload. **Classification: documentation only.**
**Worse than saying nothing? No** — the stderr line does say *"no --kind given;
stdout carries the sha256 record"*. Nit: add `--kind sha256` to the example so
the kind is always explicit in the taught form.

### F-554 — the card's `for md compose:` line uses a bare `...` ellipsis that fails opaquely when pasted
*Repro:* paste `--path ... ripemd160=09e7bb…946b` into `md compose` →
`error: unexpected argument 'ripemd160=09e7bb…946b' found`. Nothing identifies
the `...` as a placeholder. **Classification: documentation only.**
**Worse than saying nothing? No.** Nit — e.g. `--path <your other paths> --path keyless,ripemd160=<H>`.

### F-555 — F-534 is fixed but still reads as open in FOLLOWUPS
*Repro:* `ms decode <preimage ms1>` now prints all four digests plus
*"(a preimage carries no hash kind; match the one your wallet uses)"*, and
`--json` carries `digests_by_kind` and `digest_kind`. `design/FOLLOWUPS.md`'s
F-534 still says *"Owning phase: **phase 3** … or this cycle's follow-up sweep"*
with no closure note. **Classification: documentation only.** Nit, records only.

### F-556 — `me sysw pack` accepts a payload whose `hash:` kind contradicts what its `phrase:` record will derive, and says nothing
*Repro:*
```
printf 'hash:ripemd160:09e7bb5051d89788fb4e4b374126721dbcc2946b\nphrase:68617264656e65642c636f727265637420686f727365206261747465727920737461706c65\n' > r.txt
me sysw pack --in r.txt --pack-preimage --no-passphrase --no-now --out e_pay.bin
# EXIT 0; three warnings, none about the mismatch
me sysw show e_pay.bin
# public record 0: ripemd160 hashlock (hash:) — 09e7bb50..bcc2946b
# secret record 1: hashlock phrase (phrase:) — not shown
```
A payload `phrase:` record always derives a **sha256** lock on the device
(`gui/composer_hashlock.go:147`), so this payload asserts two different kinds.
`ms` warns at emission time; `me` — the tool that holds both records — does not,
and neither does `me sysw show`. `me` already owns the derivation
(`crates/me-cli/src/sysw/composer_records.rs:85-88`), so the check is a derive
plus four comparisons. **Classification: warning at pack (and a line in `show`).**
**Worse than saying nothing? YES** — but bounded: the device catches it at
`payloadStatesDigest` → `composerCopyHashlockReconcile`
(`gui/composer_hashlock.go:166-169`), so the cost is a wasted flash-and-compose
cycle, not funds. **Important.** Distinct from F-535 (wire-format change, which
per F-546 moves three repos); this needs none. Note the corollary: a payload
carrying **only** a `phrase:` record for a non-sha256 wallet is the same defect
with nothing to compare against — the host cannot detect that one, which is
exactly F-535's case.

### F-557 — `me bundle`'s plate count claims completeness for a hashlock wallet while omitting the preimage plate — ESCALATES F-543
*Repro:*
```
me bundle --in card_sha256.txt --manifest man_sha256.json
me bundle --in card_ripemd160.txt --manifest man_ripemd160.json
# manifests identical after blanking strings/ids; 0 occurrences of hash|preimage|ripemd
# checklists differ only in the manifest filename
# both: "me: backup needs 8 plates (7 public + ms1 on device)"
```
F-543 records that `me bundle` is kind-blind and *"names no preimage plate"*.
The measurement adds the part that matters: the checklist makes a **completeness
claim** (*"backup needs 8 plates"*), and for a keyless hashlock wallet the plate
it omits is the only one that opens path 2. `me bundle` decodes the md1
(`chunk_set_id 0x79e84`, `integrity set-verified`), so the `ripemd160(...)` node
is in hand. **Classification: warning — at minimum, "this policy has a hashlock
path; its preimage/phrase plate is not counted here".**
**Worse than saying nothing? YES** — a count presented as the backup's
requirement is trusted as one. Raise F-543 from a CLI gap to an **Important**
completeness defect; the full kind-aware bundle stays the larger fix.

---

## What did NOT diverge across the four kinds

Recorded so a later reader does not re-measure it: `ms hashlock` (card, opcode
line, write-down line, record, `--emit-record`, `--json`), `ms decode`,
`md compose` (`--path` and the `hashlock-gated` preset), `md encode`,
`md decode`, `md inspect`, `md decompose`, `md descriptor`/`md address` on the
**keyed** hashlock shape, `me sysw pack`, `me sysw show`, and the `me` NFC
converter all behave identically across `sha256` / `hash256` / `ripemd160` /
`hash160`. Every divergence found is between **tools**, not between kinds —
except F-549, which is a note that fails to follow its own kind.
