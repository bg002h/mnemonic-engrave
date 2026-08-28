# R0 review — `SPEC_descriptor_input.md`, round 1

**Artifact:** `design/SPEC_descriptor_input.md` (982 lines), status DRAFT round 0.
**Lens:** adversarial correctness + design. One question: *is this spec sound to build from?*
**Reviewer:** independent architect agent, opus tier. Read-only; nothing in any repo was modified.

## Counts

**6 Critical / 8 Important / 7 Minor / 2 Nit**

Every Critical and Important below carries a **constructed** failure — a concrete
input that was actually run, not a plausibility argument. The Go measurements
come from a scratch module with `replace seedhammer.com => /scratch/code/shibboleth/seedhammer`
built with `/nix/store/33fw5m31lfcnk4ff2f0df7j2bxnh8lgk-go-1.26.3/bin/go`
(the fork tree was never written to); the md1 measurements from
`descriptor-mnemonic/target/release/md`; the CLI measurements from
`/home/bcg/.cargo/bin/me` (`me 0.7.0`).

Working tree note: the fork is checked out at `0b656d7` (`ship/tx-engraving`).
`git diff --stat 0b656d7 d402f18 -- nonstandard/ bip380/` is empty, re-run, so
every Go measurement holds against `main` too.

---

## Critical

### C1 — §4.2: a BlueWallet file with **no `Derivation:` header at all** is admitted, and the descriptor `me` packs cannot be re-parsed by the device

**The claim.** §4.2 enumerates the BlueWallet defects and issues three normative
refusals: no `Format:` header, zero cosigner lines, and *"a BlueWallet file whose
first cosigner line precedes its `Derivation:` header."* Defect 3's analysis is
correct as far as it goes — it identifies that `Key.ExtendedKey()` rebuilds depth
from `len(DerivationPath)`, so an empty path re-encodes to `[fp]xpub…`, which
`ParseKey` cannot read back.

**The constructed failure.** The refusal is stated as an *ordering* condition, so
it does not fire when there is no ordering — i.e. when the file has **no
`Derivation:` header at all**. That file is not caught by any of the three
refusals: it has a `Format:`, it has cosigner lines, and no cosigner line
precedes a `Derivation:` header because there is none.

```
Name: x
Policy: 2 of 2
Format: P2WSH
5a0804e3: xpub6DiYrfRwNnjeX4vHsWMajJVFKrbEEnu8gAW9vDuQzgTWEsEHE16sGWeXXUV1LBWQE1yCTmeprSNcqZ3W74hqVdgDbtYHUv3eM4W2TEUhpan
dd4fadee: xpub6DnT4E1fT8VxuAZW29avMjr5i99aYTHBp9d7fiLnpL5t4JEprQqPMbTw7k7rh5tZZ2F5g8PJpssqrZoebzBChaiJrmEvWwUTEMAbHsY39Ge
```

Measured (`nonstandard.OutputDescriptor`):

```
ACCEPT script=Segwit (P2WSH) nkeys=2
canonical = wsh(sortedmulti(2,[5a0804e3]xpub66C1RXMiBuWgpcThmArTkWBb92N7m1NKeqBtWEv5Ly…,[dd4fadee]xpub66Fud5wSGFH1Ci6uup5oNwYRXJvU4fkNnpJrFjMTAd66rbLu…))#h22nywcp
*** CANONICAL DOES NOT RE-PARSE: nonstandard: unrecognized output descriptor format
```

Control, the same file with `Derivation: m/48'/0'/0'/2'` present and first:
`canonical re-parses ok`.

**Why this is Critical.** §5.2 packs the **canonical** string. So `me`, built
exactly to §4.2, admits this file, prints the canonical descriptor as
confirmation (§5.4), packs it, and the operator engraves a plate. The device's
own parser then refuses to read it. That is verbatim the harm §7 opens with:
*"an engraved plate for a wallet that will not load."*

**Why §7's vector file cannot catch it.** §7's invariant is `host_admits ⇒
device_admits`, and for this row **both are true** — the device admits the
*input* fine. What fails is `device_admits(canonical)`, a predicate §7 never
states. See I1(c).

**The fix is not the ordering clause.** The real admission condition is *every
key must carry a non-empty origin path* (equivalently: `MasterFingerprint != 0 ⇒
len(DerivationPath) > 0`, since `Descriptor.encode` emits `[…]` iff `mfp != 0`
and `Path.Encode()` of an empty path is `""`). Stating it that way subsumes
defect 3 and this case, and it is checkable on the canonical string rather than
on the file's line order.

**Evidence:** `nonstandard/parse.go:77–161` (`path` is a single variable applied
per key line; nil when no `Derivation:` is ever seen);
`bip380/bip380.go:94–105` (`ExtendedKey()` uses `uint8(len(k.DerivationPath))`
as depth); `bip380/bip380.go:225–232` (`encode` emits `[…]` iff `mfp != 0`);
`bip380/bip380.go:368–372` (`ParseKey` requires `len(originAndPath) >= 9 &&
originAndPath[8] == '/'`).

---

### C2 — §4.3/§4.7 never say which extended-key version bytes are admitted, and `ypub` is refused by the device **everywhere** — so a host built to spec is WIDER than the device

**The claim.** §4.3 describes keys only as *"Keys via `ParseKey`, which takes
`[fingerprint/path]key/children` with a strict `[` … `]` origin … and children
parsed by `parsePath`."* No version-byte restriction is stated. §4.5 does note
that *"`ypub` is listed in the version constants but has no case in the switch,
so it hits `default` and is refused"* — but that note sits under the sentence
*"if absent, `ParseKey` falls back to the SLIP-132 version bytes"*, which reads
as though the problem is confined to the origin-less promotion branch.

**It is not confined.** `ParseKey` calls `ParseExtendedKey` **unconditionally**,
for every key in every branch, and `ParseExtendedKey` errors on any version byte
outside `{xpub, tpub, zpub, Ypub, Zpub}`. `ypub` (`049d7cb2`, BIP-49 /
SLIP-132 nested segwit — the single most common non-`xpub` serialisation an
operator will hold) is refused even when a full explicit origin is supplied.

**The constructed failure.** Two descriptors differing only in the key's version
bytes, both with a complete `[fingerprint/49h/0h/0h]` origin:

```
sh(wpkh([4bbaa801/49h/0h/0h]ypub…/<0;1>/*))   -> REFUSE  nonstandard: unrecognized output descriptor format
sh(wpkh([4bbaa801/49h/0h/0h]xpub…/<0;1>/*))   -> ACCEPT  enc=sh(wpkh([4bbaa801/49h/0h/0h]xpub…/<0;1>/*))#stw3h2ut
wsh(sortedmulti(2,[…]ypub…,[…]xpub…))         -> REFUSE  nonstandard: unrecognized output descriptor format
```

A Rust implementer reading §4.3 will reach for a standard BIP-32 parser, which
accepts `ypub`, and will find `sh(wpkh(KEY))` sitting in §4.7's accept set and a
✅ in §5.5's `--as descriptor` column. Host admits, device refuses. A packed,
engraved plate the device cannot load — the exact direction §7 calls *"the
direction that matters."*

**What the spec must add (normative).** The admitted set is exactly
`{xpub(0488b21e), tpub(043587cf), zpub(04b24746), Ypub(0295b43f), Zpub(02aa7ed3)}`.
`ypub`, `upub`, `vpub`, `Upub`, `Vpub` are all refused, and §6 needs a row
(see I2).

**Evidence:** `bip380/bip380.go:400` (`ParseKey` calls `ParseExtendedKey(k)`
after origin/children handling, on every path), `bip380/bip380.go:428–466`
(`ParseExtendedKey`: `ypubVer` is declared at line ~437 but appears **only** in
the second, normalisation switch — the classification switch has no `ypubVer`
case and falls to `default: return … "hdkey: unsupported version: %s"`).

---

### C3 — §4.7's `1 ≤ k ≤ n` has no upper bound on `n`; the spec admits multisigs whose scripts can never be spent

**The claim.** §4.3 identifies `sortedmulti(5, …)` with 2 keys as *"unsatisfiable
— funds locked forever"* and §4.7 responds with `1 ≤ k ≤ n`. That is the right
instinct applied to only half the problem.

**BIP-383, quoted from the BIP text (not from the spec):**

> "k must be less than or equal to n."
> "When used inside of a `sh()` expression, the output script produced is the
> redeem script, which is pushed as a single element in the spending scriptSig
> and must therefore not exceed the 520 byte limit on the size of a script
> element. **This allows at most 15 compressed public keys**, or at most 7
> uncompressed ones."
> "**Otherwise the maximum number of keys is 20.**"

**The constructed failure.** Both of these are inside §4.7's accept set
(`sh(sortedmulti(k, KEY…))` and `wsh(sortedmulti(k, KEY…))`, `1 ≤ k ≤ n`), and
both were measured ACCEPT by the device's parser, both yielding a real,
payable-looking address:

```
sh(sortedmulti(2, 16 keys))   -> ACCEPT  addr0 = 37X4tZxehz6hRKQ8bnT7np2ApJzgKQZEEX
wsh(sortedmulti(2, 21 keys))  -> ACCEPT  addr0 = bc1q5ffxheq0j3avnvat28xrk2lc7j9ywt8sm0ls4av6urwwc3j3ecds379hmk
```

The 16-key P2SH redeemScript is `1 + 16×34 + 1 + 1 = 547` bytes, over the
520-byte script-element limit — the scriptSig can never be relayed or accepted,
so anything paid to `37X4tZ…` is unspendable. The 21-key `wsh` exceeds
`OP_CHECKMULTISIG`'s 20-pubkey consensus limit; same outcome.

`me` built to §4.7 packs both, and §5.4's confirmation prints them as valid
wallets. This is the *same harm class* the spec already gates on for `k > n`,
omitted for `n` itself.

**Required:** §4.7 becomes `1 ≤ k ≤ n`, **and** `n ≤ 15` when the outermost
script is `sh(…)` (including `sh(sortedmulti(…))` and — see M1 — any `sh`
wrapper), `n ≤ 20` otherwise. §6 needs a row and §7 needs `n=16 under sh` and
`n=21` rows (`host_admits=false`, `device_admits=true`).

**Evidence:** BIP-383 (`bitcoin/bips` master, `bip-0383.mediawiki`), quoted
above; measured Go probe output as shown.

---

### C4 — §5.3(a) refuses only ONE of the two use-site paths md1 cannot represent; the missing one is a wrong-wallet, and every promoted bare key hits it

**The claim.** §5.3(a) is excellent work and its conclusion is correct: md1's
`UseSitePath` is `Option<Vec<Alternative>>` + a wildcard-hardened bit with
`MIN_ALT_COUNT = 2`, so *"there is no representation for one fixed index"*, and
`--as md1` must refuse `/0/*`.

**What the same struct also means, and the spec does not say.** There is no
representation for an **absent** use-site path either. `UseSitePath { multipath:
None, wildcard_hardened: false }` *is* `/*`. So md1 collapses "no children" into
`/*` exactly as it collapses `/0/*` into `/*` — and the spec's own §5.3
measurement table shows it (`wsh(sortedmulti(2,@0,@1))` and
`wsh(sortedmulti(2,@0/*,@1/*))` both → chunk-set-id `0x9bf18`). The spec printed
that row and did not classify it.

**The constructed failure — measured on both implementations.** The device's
`address.derivePubKey` **defaults an empty children list to `<0;1>/*`**
(`address/address.go:188–202`). md1 does not.

| route | receive address 0 |
| --- | --- |
| device — `address.Receive` on `wsh(sortedmulti(2,[dc567276/48h/0h/0h/2h]xpub…,[f245ae38/48h/0h/0h/2h]xpub…))` (no children) | `bc1qadgf37zk0wtu69j7yclswl99e5jmcv5eh69jp4calgs4ehsecm5sqcjw5a` |
| md1 — `md address` on the card set from `md encode 'wsh(sortedmulti(2,@0,@1))' --path "m/48'/0'/0'/2'" …` | `bc1qu2cc6t70nm0tw0v3tsmgur33gjnw2a32czk6xatccky9jpjxj4eqcedjh9` |

Those are **the identical two addresses §5.3(a) already prints**. The spec
attributes the divergence to `/0/*` alone and refuses only that shape; the
no-children shape produces it too and is admitted.

**And it is not an edge case.** §4.5's promotion branch *always* produces a key
with no children — `bip380.Descriptor{Type: Singlesig, Threshold: 1, Script: s,
Keys: []bip380.Key{k}}` at `nonstandard/parse.go:64–71` sets no `Children`. So:

```
$ printf 'zpub6r…'            > wallet.txt          # a BIP-84 account key, one line
$ me sysw pack --as descriptor --in wallet.txt      # device derives m/<0;1>/*  -> bc1qmj7qns4exnh8p6a9xndvz34msj72arnxl3sapx
$ me sysw pack --as md1        --in wallet.txt      # md1 card decodes to wpkh(@0/*), derives m/*  -> a different address set
```

**Every** promoted bare key diverges between the two `--as` values, and §5.5's
table marks `pkh` / `wpkh` / `sh(wpkh)` single-sig ✅ in both columns with no
caveat.

**Required:** either extend §5.3(a)'s refusal to *any* descriptor whose use-site
path is not a multipath group of ≥2 alternatives followed by a wildcard, or
state normatively that `me` materialises the device's `<0;1>/*` default into the
md1 encoding before encoding — and say which, because they engrave different
plates. §5.5's single-sig row must be corrected either way.

**Evidence:** `md-codec-0.42.0/src/use_site_path.rs:43` (`MIN_ALT_COUNT = 2`),
`:47–54` (`UseSitePath` has no "no wildcard" state);
`seedhammer/address/address.go:188–202` (`if len(children) == 0 { … default to
<0;1>/* … }`); `md encode`/`md decode`/`md address` runs above; the Go probe's
`addr0` values.

---

### C5 — §5.1's help text ("VERBATIM … Keeps the exact key serialisation") directly contradicts §5.2, and is false in measured cases

**The two normative statements.**

§5.1, the exact block `me` must print when `--as` is omitted:

> `--as descriptor   pack the descriptor VERBATIM. … Keeps the exact key serialisation.`

§5.2, two paragraphs later:

> Packs the **canonical re-encoded descriptor string** — `Descriptor::encode()`,
> with its BIP-380 checksum … **The record is the canonical form, not the
> operator's bytes.**

They cannot both hold, and §4.6's whitespace argument *depends* on §5.2's
reading (*"This does not violate §7's invariant, and the reason is mechanical …
the record `me` packs is the canonical re-encoded descriptor string, never the
operator's file"*). So §5.1's sentence is the wrong one — and it is the sentence
the operator reads at the moment they choose which of two artefacts to engrave.

**Constructed, all measured:** the re-encoding is not the identity in at least
four ways.

| input | canonical `Descriptor.Encode()` |
| --- | --- |
| `[4bbaa801/84h/0h/0h]zpub…` | `wpkh([4bbaa801/84h/0h/0h]xpub6BqQzsuvAV4ea…)#3g9lljwv` — **zpub → xpub** |
| origin spelled `[dc567276/48'/0'/0'/2']` | `[dc567276/48h/0h/0h/2h]` — **`'` → `h`** (`bip32.Path.Encode` writes `h`, `bip32/bip32.go:103–117`) |
| bare `xpub6DiYrfRwNnjeX…` (a depth-4 key) | `pkh(xpub6BqQzsuvAV4ea…)#d58fnqc4` — **a different base58 string**; depth is rebuilt as `uint8(len(DerivationPath))` |
| any input with a checksum | checksum recomputed over the re-encoded string; e.g. `#j6g8j0fe` replaces the operator's |

The one row where the canonical *is* byte-identical to the input is the fork's
own JSON fixture (`…/0/*…))#hfwurrvt` in, `#hfwurrvt` out), which is presumably
why the claim survived.

**Fix:** one sentence. `--as descriptor` should say it packs the *canonical
re-encoding* — same wallet, normalised serialisation, checksum recomputed — and
say explicitly that a `zpub`/`Ypub`/`Zpub` becomes an `xpub`. §5.3 already sets
this precedent for `--as md1` (*"must not claim a byte-identical round trip"*);
§5.1's `--as descriptor` line makes exactly the claim §5.3 forbids its sibling.

---

### C6 — two of the four normative input formats have no input channel, and §11's acceptance criteria are unsatisfiable as written

**The claim.** §11.1 and §11.2: *"`me sysw pack --as descriptor --in <each of the
four formats>`"*. §4.6: *"`me` trims leading and trailing ASCII whitespace from
**the whole input**"*. §4.4: pretty-printed JSON with a trailing newline is
accepted.

**The shipped contract of `--in`.** `SPEC_systemwide_payloads` §5.6 (line 838)
and `me sysw pack --help`:

> `--in <FILE>` — Read **newline-separated records** from this file instead of
> argv. Blank lines are skipped, so a record's index is its position among the
> NON-blank lines, not its line number.

**The constructed failure**, run against `me 0.7.0`:

```
$ cat bw.txt                       # the fork's own BlueWallet shape, 9 lines
# BlueWallet Multisig setup file
Name: sh
Policy: 2 of 2
Derivation: m/48h/0h/0h/2h
Format: P2WSH

dc567276: xpub6DiYrfRwNnjeX…

f245ae38: xpub6DnT4E1fT8Vxu…

$ me sysw pack --no-passphrase --in bw.txt ; echo rc=$?
me: record 0 (records count from 0) is not a form this container can place: …
rc=4
```

`record 0` is the string `# BlueWallet Multisig setup file`. The file is never
seen as one document. The same holds for pretty-printed `{label, descriptor}`
JSON (§4.4 explicitly blesses it), and §4.6's "the whole input" has no referent.

**Second consequence: `--as` is a whole-invocation flag on a multi-record
command.** §5.1 says *"`--as` … is **required whenever the input is a
descriptor**"* and its error text says *"this input is a wallet descriptor"*
(singular). The spec never says what happens when `--in` carries a mnemonic and
a descriptor, or two descriptors with different capabilities (one `/0/*`, one
`<0;1>/*`), where one is packable under `--as md1` and the other is not.

**Required:** a normative statement of the channel — a new flag
(`--descriptor-file`), or an explicit rule that `--as` switches `--in` from
records-mode to whole-file mode (and what that does to argv and stdin), plus the
per-record vs per-invocation semantics of `--as`. Until then §11.1/§11.2 cannot
be executed, which under the project's own *closure-is-lens-closure* second
clause ("a plan may not close while any of its own gates has never been run") is
a gate that cannot run.

---

## Important

### I1 — §7's vector file is unsatisfiable as specified, in three independent ways

The section is the load-bearing safety artefact of the whole cycle. As written,
a file satisfying its row requirements makes its own required assertions fail.

**(a) The `device_admits` column has two incompatible definitions.**
§7's row list requires `tr(sortedmulti)`, `wpkh(sortedmulti)`,
`pkh(sortedmulti)`, `k=0` and `k>n` to be `host_admits=false,
device_admits=true` — *"These are the rows the invariant is for."* But §5.2
states the device-side predicate as:

> A record is `ClassDescriptor` **iff** it parses under §4's cascade **and**
> matches §4.7's grammar.

Under that predicate `sysw.Classify` returns `ClassUnknown` for all five, i.e.
`device_admits=false`. §11.1 pins the Go side to `sysw.Classify` (*"the device's
`sysw.Classify` — exercised by §7's Go test — agrees"*), which makes the
contradiction concrete: the Go test asserts `device_admits==true` against a
function specified to return false. Verified: `sysw.Classify` today returns
`ClassUnknown` (0) for every canonical descriptor string I fed it.

The spec needs two columns, not one — `nonstandard_admits` (what the scan door
takes) and `sysw_class` (what the new arm returns) — or an explicit statement
that `device_admits` means `nonstandard.OutputDescriptor`, with §11.1 corrected.

**(b) Requirement 4 contradicts the whitespace rows.** Req 4: *"Both tests
assert the invariant `host_admits ⇒ device_admits` **per row**."* The §4.6
whitespace rows are `host_admits=true, device_admits=false` by construction
(trailing `\n`, CRLF, leading space are all measured REFUSE on the device). The
required assertion fails on the required rows. §7 gestures at the resolution
(*"permitted only because `canonical` is what gets packed"*) but never restates
the invariant to match, so the assertion as written is wrong.

**(c) The invariant that would catch C1 is absent.** The correct safety property
is `host_admits(input) ⇒ device_admits(canonical(input))`. §7 asserts
`device_admits(input)`, which is *true* for C1's file and therefore blind to it.
Every row where the host is admitting should carry `canonical` and assert the
device against **that**.

### I2 — §6's cause-selection algorithm cannot reach three of its own refusal rows, and has no row for `ypub`

§6 specifies a five-step rule and then a table of messages. Rule 4 is *"input
parses as an extended key → report branch 4."*

Measured:

```
bip380.ParseKey(nil, "[4bbaa801]xpub…")          -> err: hdkey: missing or invalid fingerprint
bip380.ParseKey(nil, "[4bbaa801/86h/0h/0h]xpub…") -> err: <nil>
```

So `[4bbaa801]xpub…` does **not** parse as an extended key, rule 4 does not fire,
rule 5 does, and §6's dedicated row — *"`[4bbaa801]xpub…` gives a fingerprint
with no derivation path…"* — is unreachable. §11.4 requires *"a test that reaches
it"*; no test can. The same holds for any bare `ypub` (`ParseExtendedKey`
refuses the version, so `ParseKey` fails), and for a bare `upub`/`vpub`.

And §6 has **no `ypub` row at all**, despite §4.5 recording the behaviour and
despite `ypub` being the commonest non-`xpub` an operator holds. Combined with
C2, an operator pasting a BIP-49 export gets the five-word generic message §6
exists to eliminate.

**Fix:** rule 4 must be *"input's first non-whitespace character is `[`, or the
input decodes as base58check with an extended-key version"* — a shape test, not
a success test — so the branch-4 diagnostics are reachable from the failures they
describe.

### I3 — a second panic in the Go parser, unrecorded; §4.2's stated rule admits its trigger

§4.2 says *"the key is a hex master fingerprint (≤ 4 bytes)"* and claims the
enumeration is complete (*"Three of those rows are defects in the Go parser"*).

`parseBlueWalletDescriptor` checks `len(fp) > 4` and then calls
`binary.BigEndian.Uint32(fp[:])`, which panics for `len(fp) < 4`. Measured:

```
BW fingerprint 'ab'     (1 byte)  -> PANIC in OutputDescriptor: runtime error: index out of range [3] with length 1
BW fingerprint 'abcdef' (3 bytes) -> PANIC in OutputDescriptor: runtime error: index out of range [3] with length 3
```

Three consequences: (i) `me` built to §4.2's "≤ 4 bytes" **admits** a short
fingerprint the device cannot handle; (ii) the §7 Go test, given such a row,
**crashes the suite** rather than reporting a failure — a false-signal shape;
(iii) the same panic is reachable from `gui/scan.go:87`, the device's scan door,
on a scanned QR.

`me`'s rule should be **exactly 4 bytes / 8 hex characters**, matching what
`bip380.ParseKey` already requires of an inline origin, and §4.2's defect list
should carry this as defect 4. (Fixing the Go side stays out of scope per §10;
the *host* rule and the vector row do not.)

**Evidence:** `nonstandard/parse.go:136–149`.

### I4 — §4.7 has no network-consistency rule, and the device cannot derive an address from a mixed-network multisig

Measured — accepted by the parser, canonical re-parses cleanly, address
derivation refuses:

```
wsh(sortedmulti(2,[dc567276/48h/0h/0h/2h]xpub…/<0;1>/*,[f245ae38/48h/0h/0h/2h]tpub…/<0;1>/*))
  -> ACCEPT   canonical re-parses ok
  -> addr0 ERROR: address: multisig descriptor mixes networks: unsupported descriptor
```

Also reachable through BlueWallet (a `Format: P2WSH` file with one mainnet and
one testnet cosigner: ACCEPT, same address error).

§4.7's accept set is stated purely over script shape and `1 ≤ k ≤ n`, so `me`
admits this. Under §5.2's predicate the device classifies it `ClassDescriptor`
and `gui/sysw_admit.go` admits it into `progBundle` / `progMultisig` /
`progWalletPolicy` — programs whose whole job is deriving and displaying
addresses, which for this record they cannot do. §7's `address_0` requirement
has no defined value for such a row.

**Required:** §4.7 gains *"all keys share one network"*, §6 gains a row, §7 gains
a `host_admits=false, device_admits=true` row.

**Evidence:** `address/address.go:105–107`.

### I5 — §4.5's promotion announcement prints a key string the operator cannot recognise, which is the one check the announcement exists for

§4.5 NORMATIVE: *"promotion is **announced, not silent**. `me` prints to stderr
the descriptor it inferred, in full."* §5.4: this is *"what makes §5.1's
no-fallback rule usable: the operator can see that the thing they are about to
engrave is the wallet they meant."*

Measured — the operator pastes one key, and the announced descriptor contains a
**different** key string:

```
input:     xpub6DiYrfRwNnjeX4vHsWMajJVFKrbEEnu8gAW9vDuQzgTWEsEHE16sGWeXX…   (a depth-4 cosigner key)
announced: pkh(xpub6BqQzsuvAV4eaCdsmbNeQ6aye8ZE4H7EjBryZJRkm39rdoqfWGdCj…)#d58fnqc4
```

`Key.ExtendedKey()` rebuilds depth from `uint8(len(k.DerivationPath))` and
childNum from its last element, so any key whose true depth differs from the
length of the origin path re-serialises to a different base58 string. For the
promotion branch, the path is *invented* (44'/0'/0' etc.), so this fires for
every input whose real depth is not 3.

The key material is unchanged, so the wallet is the wallet — but the operator's
only available verification ("is that my key?") returns *no*, on a correct
result. That is worse than silence in the direction the spec cares about: it
trains the operator to ignore the announcement.

**Required:** the announcement must echo the operator's key as supplied
alongside the inferred descriptor, and say that the serialisation was
normalised, or §4.5 must drop the claim that the announcement lets the operator
confirm the wallet.

**Evidence:** `bip380/bip380.go:94–105`; measured probe output above.

### I6 — §6's `k = 0` refusal text states the opposite of the truth, in a funds-safety message

§6's row: *"threshold `k` with only `n` keys. **A `k > n` policy can never be
satisfied and the coins would be unspendable.** Nothing was packed."* — one
message for both `k > n` and `k = 0`. §4.3's table calls `sortedmulti(0, …)`
*"unspendable-by-anyone / spendable-by-nobody-checked"*.

`sortedmulti(0, A, B)` compiles to `OP_0 <A> <B> OP_2 OP_CHECKMULTISIG`: **zero
required signatures**. It is not unspendable — it is spendable by *anyone*.
That is the opposite failure mode and the opposite remedy urgency. Measured, the
device derives a real, payable address for it:

```
wsh(sortedmulti(0,…)) -> ACCEPT thr=0  addr0 = bc1qy83lrzfnsdwezsxlmt3m547dfpd96rf9yf25yt39jgy4a2vcgy0su2ywkk
```

An operator who reaches this refusal is holding a coordinator export for a
wallet that may already have funds in it. Telling them the coins are
*unspendable* when in fact anyone can sweep them is a wrong result in normative
operator-facing text. §4.3's phrasing is at best ambiguous and should also be
corrected.

Split the row: `k > n` → unsatisfiable; `k = 0` → **no signature required, any
party can spend; treat any funds at this address as at risk now.**

### I7 — §6's empty-input row silently regresses a shipped, tested exit code

§6: *"Every refusal in this section is `EXIT_REFUSED` (3) unless marked
otherwise"*, and the empty-file row is not marked otherwise. Measured against
`me 0.7.0`:

```
$ me sysw pack --no-passphrase --in empty.txt ; echo rc=$?
me: no records in …/empty.txt: pass them on argv, with --in, or on stdin.
      An EMPTY input is what a FAILED upstream command leaves behind -- `mt encode --qr > rec.txt`
      writes nothing when it refuses -- so it is refused here rather than packed into a container
      that holds nothing and still flashes.
rc=2
```

The behaviour already exists, already exits **2**, and already carries a message
citing the same C-1 composition hazard §6's row cites. §6 both changes the exit
code and replaces the text, and §11.4 mandates a test asserting the *new* text —
so this would land as a real regression to a tested surface, introduced by a
section that never mentions the existing behaviour. The whitespace-only row has
the same problem (blank lines are skipped, so it also reaches the "no records"
path at rc=2).

Either mark the row `EXIT_USAGE (2)` and keep the shipped text, or state
explicitly that this cycle changes it and why.

### I8 — §2.3's claim that `SPEC_systemwide_payloads` §3.3.2 "lists the same three rows" is false; it lists two

§2.3: *"`gui/sysw_admit.go` admits `sysw.ClassDescriptor` in three programs —
lines 37, 39 and 45, i.e. `progBundle`, `progMultisig`, `progWalletPolicy`.
`SPEC_systemwide_payloads.md` §3.3.2 **is the normative source for that table and
lists the same three rows**."*

The Go side is right (lines 37, 39, 45 confirmed). §3.3.2's table
(`SPEC_systemwide_payloads.md:341–352`) is:

```
| program            | Mnem | Cdx32 | Passph | FreeText | Descr | MDMK | Addr |
| Engrave Bundle     |      |       |        |          |   •   |  •   |      |
| Engrave Multisig   |  •   |   •   |   •    |          |   •   |  •   |      |
```

There is **no Wallet Policy row** — and no Engrave Transaction row either. So
§3.3.2 lists **two** `Descr` cells, not three, and `progWalletPolicy`'s
`ClassDescriptor` cell has no normative source at all. This is the foundational
measurement of the spec's §1 sentence, it is a `§`-reference and therefore
outside `plan-cite-gate.sh`'s reach, and it is exactly the "5 of 22 ungated facts
were false" class the project note warns about.

The fold must either correct §2.3 to two rows (and record that the third cell is
code-only drift), or file the `SPEC_systemwide_payloads` §3.3.2 update — but not
keep asserting agreement that is not there.

---

## Minor

**M1 — `sh(wpkh(sortedmulti(k, …)))` is admitted by the device's parser and is
in none of the spec's enumerations.** Measured ACCEPT (`script=P2SH_P2WPKH,
type=SortedMulti, thr=2`), canonical
`sh(wpkh(sortedmulti(2,…)))#tn35y0pc` — and the device **cannot derive an
address** from it: `address: multisig script: Nested Segwit (P2SH-P2WPKH):
unsupported descriptor`. §4.7's closed accept set correctly refuses it, so no
host defect follows; but §4.3's "four shapes the Go parser accepts and should
not" is really five, and §7's required-row list should name it. It also
interacts with C3: the `n ≤ 15` bound must key on *"outermost script is sh"*,
not on the literal `sh(sortedmulti(…))` form.

**M2 — wrong BIP number, in operator-facing text.** §4.3 and §6 both say
*"taproot multisig is `multi_a`/`sortedmulti_a` (**BIP-386**)"*. Per the BIP
index, **BIP-386 is "tr() Output Script Descriptors"**; `multi_a` and
`sortedmulti_a` are **BIP-387, "Tapscript Multisig Output Script Descriptors"**.
§6's row is printed to the operator verbatim.

**M3 — a negative threshold is admitted by the device and unnamed by §6.**
`wsh(sortedmulti(-1,…))` → ACCEPT, `thr=-1`, and it still derives an address
(`bc1qmzk5m0j6vcnhvng8vj9fww5ha8lnlg7h9l69wfftqrvvjfvpprhsew77wh`) because
`strconv.Atoi` happily returns `-1` (`bip380/bip380.go:341`). §4.7's `1 ≤ k`
refuses it correctly, but §6's message names only `k > n` and `k = 0`, so the
printed text will not match the input.

**M4 — a §-reference to a section that does not exist.** §4.6 cites *"`sysw`
records are LF-separated (`SPEC_systemwide_payloads` §6.4)"*. That document's §6
is "Passphrases" and has subsections 6.1–6.3 only. The LF-separator argument
lives in its **§5.3.1**, which is itself citing **EPD §6.4** — a different
document. Load-bearing, since §4.6's whole no-invariant-violation argument rests
on it.

**M5 — `md-cli/src/parse/template.rs` is 2747 lines, not 2619** (§2.6(b)).
Conclusion unaffected (no `[lib]`, no `src/lib.rs` — both re-verified).

**M6 — §2's `origin/main` note is stale.** `origin/main` is now `d402f18`; the
push landed. The load-bearing part is unaffected: `git diff --stat 0b656d7
d402f18 -- nonstandard/ bip380/` is still empty, re-run.

**M7 — §4.4 omits that the JSON branch does not promote a bare key.** Measured:
`{"label":"x","descriptor":"xpub6DiYrfRwNnjeX…"}` → REFUSE `bip380: script:
missing '('`, while the same key on its own is promoted to `pkh(…)`. A
plausible export shape, and §4.4's list of "measured properties, all of which
`me` must reproduce or consciously decline" is where it belongs.

---

## Nit

**N1 — three citations land one to three lines off the thing they name.**
§4.2 cites `nonstandard/parse.go:74` for `parseBlueWalletDescriptor`, which is
line 77 (74 is the generic-error `return`). §4.4 cites `parse.go:41` for the
JSON branch, which is lines 44–55 (41 is branch 2's `if err == nil`). §4.5 cites
`parse.go:56`, a comment line; the branch is 58–73. All resolve to the right
function, so `plan-cite-gate.sh` passed them; a reader jumping to the line lands
in the previous branch.

**N2 — §2.3 omits that `classifyConstellation` trims first.**
`sysw/classify.go:38` is `record = strings.TrimSpace(record)`, with a comment
explaining that not trimming made the device reject md1 strings the host
accepts. Relevant to §5.2 (where the new descriptor arm goes relative to the
trim) and to §4.6's claim that the device never sees absorbed whitespace.

---

## Claims verified TRUE — do not re-verify in round 2

| § | claim | how checked |
| --- | --- | --- |
| 2 | `me 0.7.0`; `md-codec = "0.42"`; both repos have `serde_json` | `me --version`, `Cargo.toml` read |
| 2 | `nonstandard/` + `bip380/` byte-identical `0b656d7`↔`d402f18` | `git diff --stat` empty, re-run |
| 2.1 | rc=4 on a descriptor; `EXIT_OK/USAGE/REFUSED/INVALID = 0/2/3/4` at `main.rs:335–338`; message at `main.rs:2425`; `sysw/mod.rs:205` is `pub fn classify` | run + read |
| 2.2 | `md encode --help` example is `wsh(multi(2,@0/<0;1>/*,@1/<0;1>/*))` | run |
| 2.3 | `sysw_admit.go` lines 37/39/45 carry `ClassDescriptor` | grep -n |
| 2.3 | `classifyConstellation` arms are mnemonic / ms1 / MD‖MK / MT, then `ClassUnknown`; no descriptor arm | `sysw/classify.go:34–58` |
| 2.3 | `sysw.Classify` returns `ClassUnknown` for canonical descriptor strings and bare xpubs | probe |
| 2.4 | `seal.Classify` returns `ClassDescriptor`; `sysw` does not | `seal/record.go`, probe |
| 2.5 | md-codec 0.42.0 re-exports `Descriptor`, `Tag`, `TlvSection`, `PathDecl`, `OriginPath`, `PathComponent`, `encode_md1_string`, `split`, `descriptor_to_template`, `to_miniscript_descriptor` | `lib.rs:40–72` |
| 2.6a | `descriptor_to_template(&encode::Descriptor)`; `encode::Descriptor` = `{n, path_decl, use_site_path, tree, tlv}` | `render.rs:19,52`, `encode.rs:17–35` |
| 2.6b | `md-cli` has `[[bin]] name = "md"`, no `[lib]`, no `src/lib.rs` | read + `ls` |
| 3 | `nonstandard` is upstream, no Rust counterpart in `me-cli` deps; `me-cli` has no `miniscript`/`bitcoin` dep | `Cargo.toml` `[dependencies]` read whole |
| 4.1 | branch order 1→2→3→4; branch 3 returns early on failure; branches 1/2/4 fall through and discard | `parse.go:36–75` |
| 4.1 | the four branches are pairwise disjoint on the corpus, for the stated structural reasons | code trace + probe |
| 4.2 | always `SortedMulti`; `Title != ""` gate; repeat-header rules; `nkeys != len(Keys)` check | `parse.go:77–161` |
| 4.2 | defects 1–3 all reproduce: no `Format:` → `ENCODE PANIC: unknown script`; `Name: only\n` → 0 keys, same panic; `Derivation:` after keys → `[5a0804e3]xpub66C1RXMi…`, key material intact, **same address** — "not a wrong-wallet break" is correct | probe |
| 4.3 | checksum cut at first `#`, must validate; doubled checksum refused; script set `{wsh,pkh,sh,wpkh,tr}`; one wrapper level only under `sh`; `sortedmulti` only; `'`≡`h`; uppercase fingerprint normalised; `tpub` accepted | `bip380.go:271–360`, probe |
| 4.3 | `wsh(multi(…))` and miniscript both REFUSE; `tr(sortedmulti)`, `wpkh(sortedmulti)`, `pkh(sortedmulti)`, `k=0`, `k>n` all ACCEPT | probe |
| 4.5 | the three promotable paths are exactly `44'/0'/0'`, `84'/0'/0'`, `49'/0'/0'`; `Script.DerivationPath()` defines 86'/48'…2'/48'…1'/45' and all are excluded | `bip380.go:122–160`, `parse.go:58–73` |
| 4.5 | all 15 near-miss rows reproduce, including bare `zpub` → `wpkh(xpub…)` (re-serialised), bare `tpub` → `pkh` at mainnet `44'/0'/0'`, `[4bbaa801]xpub…` REFUSE | probe |
| 4.5 | the table has exactly 15 rows (§7's "all fifteen" is right) | counted |
| 5.3a | `UseSitePath = Option<Vec<Alternative>> + wildcard_hardened`; `MIN_ALT_COUNT = 2`; no single-fixed-index representation; `/0/*` and `<no path>` and `/*` share chunk-set-id `0x9bf18` vs `<0;1>/*` `0x16d62`; `md decode` returns the `/*` form | `use_site_path.rs:43,47–54`; `md encode`/`decode`/`address` runs |
| 5.3a | the two addresses in §5.3's table are correct and do differ | Go + `md` |
| 5.3b | `TlvSection` has exactly `use_site_path_overrides`, `fingerprints`, `pubkeys`, `origin_path_overrides`, `unknown` — no label/title | `tlv.rs:22–40` |
| 5.3b | `gui/gui.go:3161` is `if desc.Title != ""` → body text only | read |
| 5.3 | `ClassMDMK` produced by `classifyConstellation`, admitted by Bundle/SingleSig/Multisig/WalletPolicy | `classify.go:46`, `sysw_admit.go:37–45` |
| 6 | `SPEC_constellation_cli_uniformity` §2 does carry the "0 stdout bytes on failure" row and C-1; §6h is "Remedy text must be executable" | read |
| 7 | `codex32_seam_vectors.json`: 6840 bytes, sha256 `3d53ef88a474f02c15aa60a839f4a31071598a26c853463122a847515926eb6a`, 8 rows, top keys `_comment`/`invariant`/`vectors`, row keys `name`/`string`/`chars`/`host_admits`/`device_admits`/`source`; Go copy at `d402f18` byte-identical; both tests pin the literal (`SEAM_VECTORS_SHA256`, `seamVectorsSHA256`) | `wc -c`, `sha256sum`, `git show`, grep |
| 10 | `ClassAddress` is admitted by zero programs — no `sysw.ClassAddress` cell anywhere in `gui/sysw_admit.go` | read whole file |
| 10 | `md encode` accepts a miniscript **template** | spec's own run; not re-run |

---

## Notes for the fold

- **C1, C2, C3, I4 are all the same shape** — §4.7's accept set is stated over
  *script form* only, and every safety property that is not a script form (key
  version bytes, key count bounds, network consistency, origin-path presence)
  fell out of it. A single "admission predicate" subsection listing all of them
  as conjuncts would close four findings and make the §7 row list derivable
  rather than enumerated by hand.
- **C6 and I1 both make an acceptance gate unrunnable.** Per the project's
  closure rule, neither can be deferred past this spec's GREEN.
- Nothing here questions the operator rulings (broad input / expressive output,
  two explicit `--as` values, no silent fallback, host-side-first). C5 is a
  violation *of* the two-form ruling — an operator cannot choose between two
  artefacts on a description that is false about one of them.
- §9's self-assessment is honest and mostly correct; item 3 ("the `--as md1`
  address equality was measured for ONE descriptor shape") is precisely the gap
  C4 falls into, so the spec came close to catching it by reasoning about its own
  coverage.
