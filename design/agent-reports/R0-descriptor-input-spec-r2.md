# R0 review — `SPEC_descriptor_input.md`, round 2 (proportional re-review)

**Artifact:** `design/SPEC_descriptor_input.md` at `e8f2360` (1170 lines), status DRAFT round 1.
**Scope, as briefed:** (1) did the fold close each of r1's 23 findings; (2) did the fold
introduce new defects. Not a fresh audit. The r1 "Claims verified TRUE" table, r1's measured
probe results, the citation gate, the phase-order question and the operator rulings were
taken as settled and were not re-derived.
**Reviewer:** independent agent, opus tier. Read-only; nothing in any repo was modified.

## Counts — NEW findings only

**1 Critical / 5 Important / 2 Minor / 2 Nit**

**Disposition of r1: 23 FIXED, 0 PARTIAL, 0 NOT FIXED.** Every one of the six Criticals and
eight Importants closes the failure r1 constructed — checked by re-running the construction,
not by locating the edit. Two of the fixes are themselves defective in a way r1 could not
have anticipated (NEW-C1 below is adjacent to C4's fix; NEW-I2 is inside C3's fix), which is
why this round is not clean.

**Measurement environment.** Go probes: scratch module at
`…/scratchpad/goprobe` with `replace seedhammer.com => /scratch/code/shibboleth/seedhammer`,
built with `/nix/store/33fw5m31lfcnk4ff2f0df7j2bxnh8lgk-go-1.26.3/bin/go`; **the fork tree was
never written to** (`main` = `origin/main` = `d402f18`, re-checked). md1 probes:
`/scratch/code/shibboleth/descriptor-mnemonic/target/release/md`. CLI probes:
`/home/bcg/.cargo/bin/me` (`me 0.7.0`). Keys are the fork's own `nonstandard/parse_test.go`
fixtures; SLIP-132 variants were produced by re-serialising X1's 74-byte payload under new
version bytes.

---

## Disposition table

| # | r1 finding | verdict | where the fold lands, and what re-running it shows |
| --- | --- | :-: | --- |
| C1 | no-`Derivation:` BlueWallet admitted | **FIXED** | §4.2 NORMATIVE now states rule 3 over the KEYS ("any cosigner key would carry an empty origin path"), §4.7 conjunct 6 restates it on the canonical string, §6 has the row, §7 requires the row with `device_admits=true`/no `canonical`. The constructed file is now refused by conjunct 6. |
| C2 | version bytes unstated; `ypub` device-refused | **FIXED** | §4.3's five-member set `{xpub 0488b21e, tpub 043587cf, zpub 04b24746, Ypub 0295b43f, Zpub 02aa7ed3}` re-verified against `bip380/bip380.go:428–466` — the classification switch has exactly those five arms and `default:` errors. NORMATIVE "`me` admits exactly the same five" closes the wider-than-device gap. (The refusal's *remedy* is defective — NEW-I3.) |
| C3 | `n` unbounded | **FIXED** | §4.7 conjunct 3 adds `n ≤ 15` under `sh`, `n ≤ 20` otherwise; §6 row; §7 rows `n=16` under `sh` and `n=21` under `wsh`. Both unspendable scripts are now refused. (The bound is over-broad — NEW-I2.) |
| C4 | absent use-site path unclassified | **FIXED** | §5.3(a′) materialises `<0;1>/*`. **Verified correct**: device childless ≡ device `<0;1>/*` at receive *and* change 0 for all seven admitted shapes (table in NEW-C1), and md1 `<0;1>/*` reproduces the device address for `pkh`/`wpkh`/`sh(wpkh)`/`tr` and `wsh(sortedmulti)`. §5.5's single-sig row and §11.2 both carry it. |
| C5 | "VERBATIM" help text | **FIXED** | §5.1's block now says "pack the descriptor in CANONICAL form … SLIP-132 versions become xpub, `'` becomes `h`, the checksum is recomputed", and §5.2 names the fourth transformation (depth/child-number rebuild). No surviving claim of byte-identity. |
| C6 | no input channel; §11 unrunnable | **FIXED** | §5.1's single-document mode: `--as` ⇒ one descriptor, `--in` read whole, >1 argv operand or argv+`--in` = `EXIT_USAGE`. §4.6's "the whole input" now has a referent; §11.1/§11.2 both cite it; F-414 exists and says what is deferred. (A dead end remains — NEW-I5.) |
| I1 | §7 unsatisfiable, three ways | **FIXED** | (a) columns split into `device_admits` (nonstandard, input) + optional `sysw_class`, and §11.1 now names `sysw_class`. (b)+(c) requirement 4 is now `host_admits(input) ⇒ device_admits(canonical(input))` with `canonical` REQUIRED on every host-admitted row. **Satisfiability checked, not assumed**: the added fixed-point clause holds on every host-admitted shape probed (17 shapes incl. promoted bare keys, `zpub`→`xpub`, BlueWallet, `sh(wsh(…))`, `sh(sortedmulti)`, whitespace-trimmed) — `Encode(Parse(canonical)) == canonical` in all of them. |
| I2 | branch-4 rows unreachable; no `ypub` row | **FIXED** | §6 rule 4 is now a shape test ("first non-whitespace character is `[`, or a single base58check token whose payload is 78 bytes"). 78 is the BIP-32 serialisation length, so every branch-4 row — `[fp]xpub…`, bare `Zpub`/`Ypub`, bare `tpub`, account ≠ 0 — is reachable from a `ParseKey` failure. `ypub` row added. |
| I3 | second panic (short fingerprint) | **FIXED** | §4.2 defect 4 + NORMATIVE "exactly 8 hex characters"; §6 row; §7 `device_probe: "panic"` with the explicit instruction not to feed the input to the Go parser. |
| I4 | no network-consistency rule | **FIXED** | §4.7 conjunct 5, §6 row, §7 row. |
| I5 | announcement prints an unrecognisable key | **FIXED** | §4.5 now prints `key as supplied` *and* `inferred wallet`, with the normalisation stated. (§5.4's own bullet list did not follow — NEW-N2.) |
| I6 | `k = 0` message states the opposite | **FIXED** | §6 row split: "threshold 0 means NO signature is required: anyone who can see this script can spend from it … treat them as at risk now", and §4.3's table row corrected to "spendable by ANYONE". |
| I7 | empty-input row regresses a shipped exit code | **FIXED** | §6 keeps the shipped text verbatim at `EXIT_USAGE (2)` for both the empty and whitespace-only rows. Re-measured against `me 0.7.0`: empty file → rc=2, whitespace-only file → rc=2, same message. |
| I8 | "§3.3.2 lists the same three rows" is false | **FIXED** | §2.3 now says **two** cells (Engrave Bundle, Engrave Multisig) and states the drift instead of agreement; F-415 filed. Re-read `SPEC_systemwide_payloads.md:341–352`: two `Descr` cells, and no Wallet Policy row exists in that table at all. |
| M1 | `sh(wpkh(sortedmulti))` unlisted | **FIXED** | §4.3's table row added; §7's required-row list names it. |
| M2 | wrong BIP number | **FIXED** | BIP-387 in both §4.3 and §6. |
| M3 | negative threshold unnamed | **FIXED** | §4.3 table row; §6 "or any `k < 1`"; §7 `k=−1` row. |
| M4 | §-reference to a section that does not exist | **FIXED** | §4.6 now cites `SPEC_systemwide_payloads` §5.3.1 — which exists (line 532, "The two new classes collide with EPD§6.4") and does cite EPD §6.4. |
| M5 | `template.rs` line count | **FIXED** | §2.6(b) says 2747; `wc -l` says 2747. |
| M6 | stale `origin/main` note | **FIXED** | §2 says "pushed 2026-08-28; origin/main agrees"; `git rev-parse main origin/main` both `d402f18f6a8c…`. |
| M7 | JSON branch does not promote a bare key | **FIXED** | §4.4 bullet added. |
| N1 | three citations off by 1–3 lines | **FIXED** | `parse.go:77` is `func parseBlueWalletDescriptor`; `:44–55` is the JSON branch (`var jsonDesc struct {` at 44); `:58–73` is the promotion branch (`if k, err := bip380.ParseKey(nil, enc)` at 58). All three now land. |
| N2 | `classifyConstellation` trims first | **FIXED** | §2.3 records `strings.TrimSpace` at `sysw/classify.go:38` and draws the consequence for §4.6/§5.2. |

---

## NEW — Critical

### NEW-C1 — §5.3's md1-representability enumeration is still incomplete: a multipath group with NO wildcard (`/<0;1>`) is admitted by §4.7 and silently engraves a DIFFERENT wallet

**Where the fold left it.** §5.3 now enumerates two use-site-path cases: (a) an explicit single
fixed index → refuse; (a′) an absent path → materialise `<0;1>/*`. §4.7's six-conjunct
predicate constrains shape, threshold, key count, version bytes, network and origins — and says
**nothing about the use-site path**. The device's `parsePath` grammar (§4.3: "child index, `*`,
`*'`/`*h`, or a `<a;b;…>` range") generates more shapes than two, and the ones that are left over
are not safe.

**The constructed failure — measured on both implementations.**

```
device (Go address.Receive/Change on the parsed descriptor):
  wsh(sortedmulti(2,[dc567276/48h/0h/0h/2h]X1/<0;1>,[f245ae38/48h/0h/0h/2h]X2/<0;1>))
    ACCEPT   canonical re-parses, fixed point ok
    recv0 = bc1qu2cc6t70nm0tw0v3tsmgur33gjnw2a32czk6xatccky9jpjxj4eqcedjh9
    chg0  = bc1qrranhxpwcp9s9xvvp3r9avztwswvkksyhhc587spm95xyu56fjtqsk8cv0

md1 (md encode of the same wallet, then md decode / md address):
  md encode 'wsh(sortedmulti(2,@0/<0;1>,@1/<0;1>))' …   -> chunk-set-id: 0x16d62
  md encode 'wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*))' -> chunk-set-id: 0x16d62   <-- SAME CARDS
  md decode  <those cards>  -> wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*))
  md address <those cards>  -> bc1qadgf37zk0wtu69j7yclswl99e5jmcv5eh69jp4calgs4ehsecm5sqcjw5a
```

`bc1qu2cc6t7…` ≠ `bc1qadgf37z…`. This is **the identical divergence, and the identical pair of
addresses, that §5.3(a) already prints** — md1 collapses `<0;1>` into `<0;1>/*` exactly as it
collapses `/0/*` into `/*`, and the chunk-set-id proves the two templates produce the same card
set. §5.3(a) refuses only "a single fixed child index"; `/<0;1>` is not one. §5.3(a′)
materialises only an ABSENT path; `/<0;1>` is not absent. §4.7 admits it. So `me sysw pack --as
md1` on this descriptor packs a card set for a different wallet, and §5.4's confirmation would
print the template the operator supplied.

**Why the invariant does not save it.** §7's row-level invariant is
`host_admits ⇒ device_admits(canonical)`, and `canonical` here re-parses cleanly — the device
reads the descriptor fine. The break is at the md1 layer, which §7 covers only through
`address_0` on "rows carrying `--as md1` capability" — and §7's required-row list has no
use-site-path rows beyond `/0/*`. So no required row would catch it.

**Not a contrivance.** `<0;1>` without a trailing `/*` is what a multipath descriptor looks like
when the wildcard is dropped in transcription, and the device accepts it silently — the two
spellings differ by two characters and by an entire wallet.

**Required.** State the rule over the whole use-site-path grammar rather than over two cases.
The safe closed set is: **absent** (→ materialise `<0;1>/*`, per (a′)), **`/*`** (verified
equal: device `/*` gives recv0 = chg0 = `bc1qu2cc6t7…`, and md1's `/*` gives the same), and
**`<i;i+1>/*`**. Everything else — `/i`, `/i/*`, `<a;b>` with no wildcard, `*h`, `<a;b>` with
`b ≠ a+1` — is refused, with a §6 row and a §7 row. Adding it as **conjunct 7 of §4.7** makes
§7's row list derivable, which is exactly what the fold's own predicate paragraph promises.

**Evidence:** Go probe (`nonstandard.OutputDescriptor` + `address.Receive`/`Change`);
`md encode`/`decode`/`address` runs above; `md_codec::use_site_path::UseSitePath` has no
"no wildcard" state (`src/use_site_path.rs:47–54`, r1's verified table).

---

## NEW — Important

### NEW-I1 — two more use-site-path shapes are admitted by §4.7 and are broken on the device: `<0;1>/*h` (hardened bit silently ignored) and `<0;2>/*` (no address derivable)

Same missing conjunct as NEW-C1, different harm, so it is stated separately — a fix for one
that does not cover the other leaves a live row.

**`*h` — the device ignores it, md1 does not.**

```
device: wsh(sortedmulti(2,…X1/<0;1>/*h,…X2/<0;1>/*h))
   ACCEPT, fixed point ok, recv0 = bc1qadgf37zk0wtu69j7yclswl99e5jmcv5eh69jp4calgs4ehsecm5sqcjw5a
   -- byte-identical to the NON-hardened <0;1>/* address: address/address.go never reads
      Derivation.Hardened at the use site, so it derives the unhardened child and displays it.
md1:    md decode  -> wsh(sortedmulti(2,@0/<0;1>/*',@1/<0;1>/*'))     (the bit IS carried)
        md address -> md: codec error: hardened public-key derivation: use-site path requires
                      hardened component, which BIP 32 forbids on xpub-only restore
```

`md` is right and the device is wrong: hardened derivation from an xpub is impossible. So
`--as descriptor` engraves a plate whose device screen shows addresses **for a wallet that
cannot exist**, and `--as md1` engraves a plate from which no address can ever be derived.
`me` must refuse the shape; fixing the Go side stays out of scope per §10, exactly as with
I3's panic and I4's mixed network.

**`<0;2>/*` — parsed, canonical, and underivable.**

```
device: wsh(sortedmulti(2,…X1/<0;2>/*,…X2/<0;2>/*))
   ACCEPT, canonical re-parses, FIXED POINT ok
   recv0: ERROR address: unsupported range path element     (address/address.go:~205, End != Index+1)
   Supported = true          <-- and address.Supported still reports true
md1:   encodes and decodes it faithfully; md address derives.
```

This is verbatim the I4 class the fold just closed for networks: the host admits, the record is
classified `ClassDescriptor`, and it reaches `progBundle`/`progMultisig`/`progWalletPolicy` —
programs whose whole job is deriving and displaying addresses they cannot derive. `Supported`
returning `true` while `Receive` errors means a device-side gate does not catch it either.

**Required:** covered by NEW-C1's conjunct 7 if that conjunct is written over the grammar rather
than patched case by case; plus a §6 row and §7 rows (`host_admits=false`, `device_admits=true`).

### NEW-I2 — §4.7 conjunct 3 applies BIP-383's 15-key cap to `sh(wsh(sortedmulti(…)))`, where the cap is 20; the §6 message states a reason that is false for that form

**What the fold wrote.** Conjunct 3: *"`n ≤ 15` when the **outermost script is `sh(…)`** (the
redeemScript is one script element, capped at 520 bytes; 16 compressed keys need 547), `n ≤ 20`
otherwise"*. §4.7's accept set includes `sh(wsh(sortedmulti(k, KEY…)))`. (r1's M1 prescribed
"outermost script is sh"; a prescribed fix is not authoritative, and this one is wrong.)

**BIP-383, read from the file in the fork's own testdata**
(`address/testdata/bips/bip-0383.mediawiki:41–47`):

> When used at the top level, there can only be at most 3 keys.
> When used inside of a `sh()` expression, **the output script produced is the redeem script**,
> which is pushed as a single element in the spending scriptSig and must therefore not exceed the
> 520 byte limit on the size of a script element. This allows at most 15 compressed public keys…
> Otherwise the maximum number of keys is 20.

The 15-key cap is conditioned on the multi expression's own output script *being* the redeem
script. In `sh(wsh(sortedmulti(…)))` the multi's output script is the **witnessScript**; the
redeemScript is `OP_0 <32-byte sha256>` — **34 bytes regardless of `n`**. So the binding clause
is "Otherwise the maximum number of keys is 20", and a 16-to-20-key `sh(wsh(sortedmulti(…)))`
is a perfectly spendable wallet (witnessScript for n=16 is 547 bytes, well under the 3600-byte
P2WSH standardness limit).

**Constructed:** `sh(wsh(sortedmulti(2, 16 keys)))` — measured ACCEPT on the device, canonical is
a fixed point, and it derives real addresses `recv0 = 3HBBPgNtmPjjuRonQq7EWpurZt3zd4Xvtc`,
`chg0 = 354rmUyqLH5eTQ97iPvs17BxFQwdSnUtQj`. Under conjunct 3 as written, `me` refuses it, and
§6 prints *"`sh(…)` multisig carries at most 15 keys (the 520-byte script-element limit,
BIP-383)"* — a false statement about this wallet, and the refusal blocks a wallet that is not
defective. Being narrower than the device is permitted by §7; printing a false reason for it is
not, and §6 exists precisely so a refusal says the true thing.

**Required:** conjunct 3 keys on *"the `sortedmulti` is the direct argument of `sh(…)`"*, i.e.
`sh(sortedmulti(…))` only, `n ≤ 20` for `wsh(…)` and `sh(wsh(…))`. §6's row splits the two
reasons. §7's `n=16` row must specify `sh(sortedmulti(2, 16 keys))` — the direct form — and a
`sh(wsh(sortedmulti(2, 16 keys)))` row belongs on the *accepted* side.

### NEW-I3 — §6's SLIP-132 row prints a remedy that is wrong for four of the five versions it covers, and a placeholder for the commonest input

**What the fold wrote.** One row covering *"a descriptor or bare key using
`ypub`/`upub`/`vpub`/`Upub`/`Vpub`"*, whose remedy is *"The same key as the device reads it:
`sh(wpkh([<fp/path>]xpub…/<0;1>/*))`"* — "the equivalent spelling is printed with the operator's
own key converted (the re-encode is mechanical), per the executable-remedy rule."

**Leg 1 — four of the five are TESTNET, and the remedy converts them to mainnet.** `upub`
(`044a5262`) and `vpub` (`045f1cf6`) are testnet BIP-49 / BIP-84; `Upub` (`024289ef`) and `Vpub`
(`02575483`) are testnet multisig. Their only admitted spelling is **`tpub`**, not `xpub`.
Measured — the printed remedy, followed literally for a `vpub`, versus the wallet the operator
actually holds:

```
remedy as printed : sh(wpkh([dc567276/84h/1h/0h]xpub…/<0;1>/*))
                    ACCEPT -> recv0 = 354hXbgwGRqHXywh9ZESRXWW4zxrpeScXQ      (MAINNET P2SH)
the real wallet   : wpkh([dc567276/84h/1h/0h]tpub…/<0;1>/*)
                    ACCEPT -> recv0 = tb1qmj7qns4exnh8p6a9xndvz34msj72arnx4htw64  (testnet P2WPKH)
```

Wrong network **and** wrong script. Both descriptors are admitted by the device, so nothing
downstream catches it — the operator engraves a mainnet plate for a testnet wallet, or (worse,
in the other direction of the same error) a `vpub` holder is handed a `sh(wpkh(…))` spelling for
a BIP-84 account. This is r1's I6 shape: normative operator-facing text that states something
false about funds.

**Leg 2 — for a BARE key the remedy cannot be executed at all.** A bare `ypub` is the commonest
form of this input (an operator exports one line from a BIP-49 wallet). Measured: bare `ypub`
REFUSE. The remedy template contains `[<fp/path>]`, which the input does not supply — a
placeholder, in direct violation of §6's own binding rule ("it prints the descriptor with the
operator's own key and origin substituted in, **not a placeholder**"). And the obvious
substitute — hand back the converted bare key — is a *different wallet*: measured, a bare `xpub`
promotes to `pkh(xpub…)` (recv0 `1M88vKcJFc4KPAe5RHXsuJqWcg3muStyK4`), not to `sh(wpkh(…))`.
The correct bare-key remedy is an origin-less descriptor, `sh(wpkh(<converted xpub>/<0;1>/*))`,
which the device does admit.

**Required:** the row states the conversion target per version — `ypub` → `xpub`; `upub`, `vpub`,
`Upub`, `Vpub` → `tpub` — and the script per version (`ypub`/`upub` → `sh(wpkh(…))`, `vpub` →
`wpkh(…)`, `Upub`/`Vpub` → the multisig form, i.e. no single-key remedy exists), and gives the
bare-key case its own origin-less spelling. Splitting the row per version is the honest shape;
one template cannot serve five.

### NEW-I4 — five shapes §4.7 narrows have no §6 row, and §4.7's claim that §7 gives two of them rows is false

**Cross-reference rot, in the fold's new §4.7 paragraph.** §4.7 says: *"…and, by inspection of
`bip380.Parse`'s grammar rather than by probe, the key-in-a-script-slot forms `wsh(KEY)` and
`sh(KEY)` … That second pair is flagged as **inspection, not measurement**; **§7 gives them
rows**."* §7's "row set MUST include" list names `tr(sortedmulti)`, `wpkh(sortedmulti)`,
`pkh(sortedmulti)`, `sh(wpkh(sortedmulti))`, `k=0`, `k=−1`, `k>n`, `n=16`, `n=21` and a
mixed-network multisig. **`wsh(KEY)` and `sh(KEY)` are not in it.** §11.3 requires a test that
counts §7's bullets, so the count would pass and §4.7's sentence would stay false.

**Inspection is now measurement**, and the pair is exactly the class §7's rows exist for:

```
wsh([dc567276/48h/0h/0h/2h]X1/<0;1>/*)   ACCEPT (Singlesig, P2WSH), fixed point ok
   recv0: ERROR address: singlesig script: Segwit (P2WSH): unsupported descriptor   Supported=false
sh([dc567276/45h]X1/<0;1>/*)             ACCEPT (Singlesig, P2SH), fixed point ok
   recv0: ERROR address: singlesig script: Legacy (P2SH): unsupported descriptor    Supported=false
```

**And §6 has no row for any of them.** Scanning §6's table, the shapes §4.7 narrows that have
**no** message specified are: `wpkh(sortedmulti(…))`, `pkh(sortedmulti(…))`,
`sh(wpkh(sortedmulti(…)))`, `wsh(KEY)` and `sh(KEY)`. Only `tr(sortedmulti)` got a row, and its
text is taproot-specific. Measured, `wpkh(sortedmulti(2,…))` is ACCEPT on the device with
`Supported=false` and `recv0: ERROR address: multisig script: Segwit (P2WPKH): unsupported
descriptor` — a real operator-reachable input with no specified refusal. §6's premise is *"the
device's parser has exactly one message for eleven distinct causes. `me` has one per cause"*, and
§11.4 requires a test per row; five causes currently have no row to test.

**Required:** two §6 rows (one for "a single-key script wrapping a multi", one for "a bare key in
a script slot"), and add `wsh(KEY)`/`sh(KEY)` to §7's bullet — or delete the promise in §4.7.

### NEW-I5 — the single-document rule and the `--as`-required rule form a dead end for a multi-record file containing a descriptor, and the second message is wrong

**Journey, three steps, all grounded.** The operator has a `sysw` batch file — a mnemonic and a
wallet descriptor, one per line — which is the composition the Engrave Bundle / Engrave Multisig
programs exist for (§2.3's admission table: `Descr` + `MDMK` in one container).

1. `me sysw pack --no-passphrase --in mixed.txt` — measured against `me 0.7.0` today:
   `rc=4`, *"record 1 … is not a form this container can place"*. Under this spec §5.1 replaces
   that with `EXIT_USAGE (2)` and the block *"this input is a wallet descriptor, and `--as`
   decides how it is packed"*.
2. The operator does what the message says and adds `--as descriptor`. Now §5.1's single-document
   rule applies: *"`--in <FILE>` is the entire file as one document"*. The whole two-line file
   goes into the cascade.
3. Measured: `nonstandard.OutputDescriptor` on that exact byte string → **REFUSE**, and §6 rule 5
   fires: *"this is not a wallet descriptor in any of the four forms `me` reads…"* — a message
   that is **false about the file**, whose second line is a valid descriptor the same tool
   accepts on its own.

So the first refusal names a flag that cannot help, and the second refusal denies the presence of
the thing the first refusal identified. §5.1 files the *capability* as F-414 but never says what
`me` does when it meets the case, and the executable-remedy rule is violated: the named next
action produces a worse error.

**Required:** one normative sentence — when `--as` is absent and the input is a multi-record
stream in which some record is a descriptor, `me` says so and names the split (`pack the
descriptor into its own container; one container cannot yet carry both — F-414`), rather than
naming `--as`. This is a refusal-text rule, not a widening, and it costs nothing.

---

## NEW — Minor

**NEW-M1 — a BlueWallet cosigner fingerprint of `00000000` slips conjunct 6 and loses the
derivation path from the engraved artefact.** Conjunct 6 is phrased *"every key **with a
fingerprint** carries a non-empty origin path"*, and `Descriptor.encode` emits `[…]` iff
`mfp != 0`. Measured, a file with `00000000: <xpub>` and a valid `Derivation:` header:

```
ACCEPT, canonical = wsh(sortedmulti(2,xpub6DiYrfRwNnjeX…,[f245ae38/48h/0h/0h/2h]xpub6DnT4E1fT8Vxu…))
CANON REPARSE: ACCEPT, FIXED POINT ok        recv0 unchanged (bc1qadgf37z…)
```

The wallet is preserved — same addresses — but the origin path for that cosigner is **silently
dropped from the string that gets engraved**, and a zero master fingerprint is what several
coordinators emit when the seed's master key is unknown. Not funds-safety; it is backup
completeness, and it is invisible because the record still round-trips. One clause fixes it
(refuse an all-zero fingerprint, or state that its origin is dropped).

**NEW-M2 — §5.1 amends another spec's NORMATIVE flag contract without a cross-document note.**
`SPEC_systemwide_payloads` §5.6 defines `--in FILE` as *"read newline-separated records from FILE
instead of argv"*, and the shipped `--in` help adds *"With neither this nor argv records, the
same newline-separated form is read from STDIN"*. §5.1 changes both, for `--as` invocations, to
whole-document reads. That is a sound design decision and it is stated clearly — but §2.3 sets
this spec's own precedent for cross-document drift (state it, file it: F-415), and this change
gets neither a note in §5.6 nor a follow-up. A future reader of §5.6 will read a contract that
is no longer complete.

---

## NEW — Nit

**NEW-N1 — §6's five-step cause rule only ranks *parse* failures, but roughly half the table's
rows are post-parse §4.7 profile refusals** (`k=0`, too many keys, mixed network, `ypub`,
`tr(sortedmulti)`, `/0/*` under `--as md1`). The rule's opening — *"`me` runs all four branches,
keeps each error, and reports the branch the input most resembles"* — does not say that these
rows are selected by a different mechanism entirely. One sentence.

**NEW-N2 — §5.4's confirmation list did not follow I5's fold.** §4.5 now requires the promotion
announcement to print `key as supplied` alongside `inferred wallet`; §5.4's bullet list still
says only *"for a promoted bare key (§4.5), the fact of the promotion"*. The two sections
describe the same stderr block, and an implementer reading §5.4 alone would ship the version I5
rejected.

---

## Verified in passing, recorded so a later round does not re-spend it

- **§5.3(a′) is correct.** Device receive-0 and change-0 for a childless descriptor are
  byte-identical to the explicit `<0;1>/*` form for **all seven** admitted shapes:
  `pkh`, `wpkh`, `sh(wpkh)`, `tr`, `wsh(sortedmulti)`, `sh(wsh(sortedmulti))`, `sh(sortedmulti)`.
  Materialising is the encoding that preserves the wallet, as §5.3(a′) claims.
- **The md1 address equality now has more than one data point** (§9 item 3). Measured
  `md encode` → `md decode` → `md address` against the Go `address` package:
  `pkh` `1M88vKcJFc4KPAe5RHXsuJqWcg3muStyK4`, `wpkh` `bc1qmj7qns4exnh8p6a9xndvz34msj72arnxl3sapx`,
  `sh(wpkh)` `354hXbgwGRqHXywh9ZESRXWW4zxrpeScXQ`, `tr`
  `bc1ppeya86zv0hnpzrvh7czgqxkn5zjxxymxd6nqplhhx7fejxvhk0ysp7zekg` — all four **equal** across
  the two implementations, at `<0;1>/*`. Single-sig and `tr` are no longer unmeasured.
- **§7 requirement 4's fixed-point clause is satisfiable.** `Encode(Parse(canonical)) ==
  canonical` held for every host-admitted row shape probed, including the ones where the first
  encode is *not* the identity (`zpub`→`xpub`, depth rebuild, `'`→`h`, checksum recomputation).
  The clause is a real check that costs nothing to satisfy.
- **§4.3's five-member version set is exactly right** — re-read `bip380.go:428–466`: the
  classification switch has arms for `xpubVer`/`tpubVer`, `zpubVer`, `YpubVer`, `ZpubVer` and a
  `default:` error; `ypubVer` appears only in the normalisation switch below it.
- **I7's exit codes re-measured** on `me 0.7.0`: empty file rc=2, whitespace-only file rc=2, both
  with the shipped text — the fold records the shipped behaviour accurately.

---

## Closing

This lens is **not closed**. All 23 r1 findings are FIXED, and the three fold decisions the brief
singled out are sound in their core claim — (a′) materialises the right thing, single-document
mode is coherent with §11 and §4.6, and the version-byte set is exactly the device's. What the
round found is one Critical and five Importants **in the space the fold's new rules opened**: the
use-site path is the one axis §4.7's predicate does not constrain, and it carries two shapes that
change the wallet and one that cannot be derived; conjunct 3's key bound over-applies BIP-383;
the SLIP-132 remedy is wrong for the four testnet versions; two normative sections promise rows
the others do not carry; and the C6 fix leaves an operator with no next action.

The single highest-value edit is **conjunct 7**: a use-site-path clause written over the grammar
`{absent, /*, <i;i+1>/*}` closes NEW-C1 and NEW-I1 together, and makes §7's row list derivable
the way the fold's own predicate paragraph intends.
