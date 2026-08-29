# IMPL-P1 REVIEW — the independent pass over `git diff e0d3d65..5d236fb`

**Reviewer:** independent P1 reviewer (opus), 2026-08-29.
**Worktree:** `/scratch/code/shibboleth/_work/impl-s1s3`, branch
`impl/descriptor-s1s3`, head `5d236fb`. Tree clean at start and at finish;
nothing modified, committed or pushed.
**Question asked, and the only one answered:** did the P1 diff implement the
plan's P1 section faithfully against `SPEC_descriptor_input.md`, and did it
introduce a defect? Not a fresh audit of P0, the vector file's 71 rows, the
spec or the plan. Not style.

**Counts: 0C / 0I / 6M / 2N.**
**Verdict: GREEN.**

---

# 1. What I verified, and how

Everything below was RUN, not read off the implementer's report. The
implementer's four findings were treated as claims to check, not as givens.

## 1.1 The cascade against the device (brief item 1)

Read `crates/me-cli/src/descriptor/cascade.rs` side by side with the fork at
`/scratch/code/shibboleth/_work/seam-fork` (`1f09537`):
`nonstandard/parse.go:36-155`, `bip380/bip380.go:96-530`,
`bip32/*.go:69-117`. Faithful on every point the spec's §4 rows name, and on
several it does not:

* **Branch order** is 1 BlueWallet → 2 BIP-380 → 3 JSON → 4 promoted key,
  first success returning immediately (`cascade::cascade`), matching
  `OutputDescriptor`. The one divergence is documented and CANNOT change
  admission: Go returns branch 3's failure without trying branch 4, `me`
  keeps going. For that to widen `me`, one input would have to be both a
  document `json.Unmarshal` accepts into `struct{Label,Descriptor string}`
  (an object or `null`) and a `ParseKey`-parseable bare key. No such string
  exists.
* **`parse_func`'s control flow**, including the part that actually matters:
  when the SECOND `parseFunc` fails after a wrapper, Go skips the multi
  switch and leaves the descriptor Singlesig — which is what makes
  `sh(wpkh(KEY))` single-sig. `parse_bip380` reproduces it with `ok = false`,
  and `parse_func` advances `rest` only on success, exactly as the Go
  closure leaves `desc` untouched on error.
* **The BlueWallet admission gate** (`bw.Title != ""`, `parse.go:37`) is
  lifted ahead of the key-count check. That is a deliberate ordering change,
  it is commented at the site, it changes no admission (both orders refuse),
  and it is pinned by `gate/deadbeef-fronts-an-xpub`. F-2's justification
  checks out: for a file with no `Policy:` line the count row's own sentence
  would be false about the operator's file.
* **`seenKeys` dedup applies to cosigner lines too** in Go — `me` reproduces
  it exactly (`seen: BTreeMap`, `continue` on an identical repeat,
  `InconsistentHeader` on a differing one). See M5 for the consequence.
* **`fmt.Sscanf("%d of %d")`**, `strconv.Atoi`'s sign, `bip32.ParsePathElement`'s
  `int64(iu32) != idx` and overflow guards, `parsePath`'s cut on the FIRST
  `;` plus `start > end` and hardened bounds, `bip32.ParsePath`'s required
  leading `m` (so `Derivation: m` is a legal EMPTY path and §4.2's origin
  rule then refuses the file) — all reproduced.
* **`Descriptor.encode`** (`bip380.go:191-240`) and
  **`Key.ExtendedKey().String()`** (`bip380.go:96-111`) are reproduced
  limb-for-limb in `Parsed::encode_no_checksum` and `Key::canonical_string`,
  including the depth-from-`len(DerivationPath)` and
  childNum-from-last-element rebuild that is why a canonical can carry a
  base58 string the operator has never seen.
* **`hdkeychain.NewKeyFromString`'s curve check** is in the right place —
  before the version switch, as in Go — and `descriptor/secp.rs` is the
  narrower direction where it diverges (`0x00` private bodies refused). The
  512→256 reduction, `mul_wide`'s carry index and `reduce`'s loop bound were
  checked for the out-of-range panic they invite; both are unreachable
  because a 256×256 product fits in 512 bits and `hi * C < 2^289`.

**No branch is transposed and no branch has a fallthrough the device does not
have.**

## 1.2 Conjunct 8 against §6's two key-identity rows (brief item 2)

`admit::conjunct_8_key_identity` matches §4.7's wording exactly:

* (a) fires on `same_origin(a,b) && a.identity() != b.identity()`, where
  `same_origin` requires `a.fingerprint != 0` — correct, because an all-zero
  fingerprint means "master unknown", which is not a claim about identity
  and cannot contradict another key's. Two distinct keys both carrying
  `00000000` are a legal wallet, and the canonical omits the origin block
  for both.
* (b) fires on `a.identity() == b.identity() && a.children == b.children` —
  keyed on the **use site**, not the origin, per r2's NEW-I1. `identity()`
  is `(key_data, chain_code)` and deliberately NOT the base58 string, so two
  spellings of one key are one key.
* Slugs `key-identity` and `key-identity-duplicate` are in `Row::ALL`, are
  asserted equal to the file's `refusal_rows` set, and fire on the real
  invocation for `gate/colliding-origin-sortedmulti`,
  `gate/colliding-origin-multi` and `gate/duplicate-key-same-use-site`
  (measured — see §2).

The extra clause on the duplicate row (*"and it lets one holder produce two
of the required signatures"*) is licensed: `PLAN-descriptor-S1S3-r4.md:314-326`
raises exactly that omission as NEW-M4, and the plan's header leaves r4's
minors open as implementer notes. Checked against the report, not inherited.

## 1.3 The discriminator and the carriage rule (brief item 3)

`gate::consult` implements §5.1's four arms in §5.1's order: gate closed →
`RecordRefusal`; whole input parses → `carriage`; whole input fails and a
record parses → `MultiRecord`; neither → `select_cause` over the WHOLE input.
`carriage` puts the §4.7 admission refusal ahead of the §5.3
representability refusal, and fires the choice block iff at least one `--as`
value carries the input in THIS build.

**On the brief's "neither" arm:** §5.1 is explicit that when the gate does
not open, *"the SHIPPED record-classification refusal stands unchanged
(invariant 1)"*. `Outcome::RecordRefusal` falling through to the shipped
refusal IS that arm, and it is what the twelve `record-refusal` gate rows
pin. An input that is neither record-shaped nor descriptor-shaped therefore
gets exit 4 in record vocabulary by design, not by fallthrough — I checked
this is the spec's answer rather than assuming the brief's reading.

Invariant 1's derivation was re-checked against the classifier rather than
against an exemplar: the six shapes `sysw::classify_with` admits are `tx:`,
`pass:`, `text:`, a BIP-39 mnemonic, `mt1`, and bech32 md/mk/ms. None begins
with an identifier followed by `(` (T1); none is a `": "` line whose key is a
BlueWallet header or 8 hex characters (T2); none is a single token that is a
78-byte base58check envelope — the bech32 charset contains `0`, which base58
excludes, and the lengths do not match (T3); none is a JSON object (T4).

## 1.4 Refusal-text discipline (brief item 4)

**Machine-checked, not read.** I ran the binary over 120 inputs (all 71
vector inputs, all 19 `canonical` values, 22 constructed adversarial files,
6 `sortedmulti`→`multi` twins, 3 emitted remedies fed back) and scanned every
stderr byte for `§`, `F-<digit>`, `R0`, `PLAN-r`, `walk W<digit>`,
`P<0-3>.<digit>`, `NEW-[ICMN]<digit>` and bare `S1/S2/S3`.

Six hits, **all six in the PRE-EXISTING shipped record-refusal and
passphrase-warning surface** (`(§5.3.1)`, `(spec §13 D3)`), none in any
P1-introduced text. That is invariant 1 working — the record surface is
untouched — and it is a positive result for the operator-language rule
across every P1 refusal.

**§6h (a remedy never prints an elided key)** holds today on all three
generators. I fed the emitted remedies back through `me`:

```
[4bbaa801/84h/0h/0h]xpub6C9j4…        -> promotion announcement, exit 2
sh(wpkh([4bbaa801/49h/0h/0h]xpub…))   -> choice block, exit 2   (the ypub remedy)
pkh([4bbaa801/84/0/0]zpub…/<0;1>/*)   -> choice block, exit 2
```

**Does `every_promotion_remedy_me_prints_is_an_input_me_admits` gate the
class?** For elision, yes and provably: an elided key
(`xpub6C9j4wAxxk…acoGnx`) is not base58, so `host_admits` is false and the
assertion fires. It cannot pass with a §6h elision violation in
`suggested_descriptor_for`, and the `executable == 3` floor stops it passing
by skipping. But it covers ONE of three remedy generators — see M3 — and it
does not gate remedy CORRECTNESS — see M2.

## 1.5 The `main.rs` seam (brief item 5)

**Is it the right moment?** Yes, and exactly. `sysw::admit_check`
(`crates/me-cli/src/sysw/mod.rs:403-410`) fails on one condition only —
`classify_with(r, adm) == Class::Unknown` — so "the gate is consulted at the
one moment record classification fails" is literally true rather than
approximately.

* **No valid record stream can reach the gate.** `consult` is inside
  `if let Err(e) = sysw::admit_check(...)`. Confirmed live: a mnemonic +
  `text:6869` stream packs at exit 0 with the descriptor module never
  entered.
* **No descriptor-shaped input reaches the old record refusal by a path
  that bypasses the gate.** Everything upstream of `admit_check` was walked:
  the argv bearer/secret guard calls `classify` (a descriptor is `Unknown`,
  so it falls through), `no_records_guard` fires only on an all-blank input
  (which §6 assigns to the shipped "no records" row at exit 2 deliberately),
  and `split_record_stream` does not strip `#` lines, so the fork's own
  BlueWallet fixture never degenerates to zero records.
* **All three channels carry the document.** Measured: `--in` (raw file),
  argv (`argv.join("\n")`), stdin (`read_stdin_raw`, single read). A bare
  xpub on argv and a `wpkh(...)` on stdin both reach the choice block at
  exit 2.
* **Ordering against the ceremony:** a descriptor refusal fires before the
  passphrase ceremony. Measured with the default (no `--no-passphrase`):
  exit 2, choice block, no passphrase generated.
* **`--out` and stdout:** every refusal wrote 0 stdout bytes and created no
  `--out` file (measured on `narrowed/threshold-zero`).

One pre-existing ordering worth naming, not a P1 defect: `--expect` runs
before `admit_check`, so `me sysw pack --in wallet.txt --expect mnemonic`
hears the `--expect` refusal rather than the descriptor block. That order
predates this diff (F-246) and the spec does not rule it.

---

# 2. Independent measurements

Run in this worktree at `5d236fb` with `target/debug/me`.

**(a) All 37 gate rows against the real `--as`-omitted invocation** —
reproduced by hand outside the test harness, feeding each row's `input`
through `--in`. 37/37 land on the row's `outcome`, `refusal_row` and
`exit_code`. The twelve `record-refusal` rows print the shipped
`(records count from 0)` surface unchanged; the seven `as-decides` rows print
the choice block at exit 2; `gate/multi-record-mnemonic-first` prints
`record 1 is a wallet descriptor` at exit 4.

**(b) The canonical fixed point, on the host side.** Every one of the file's
19 `canonical` strings fed back through `me`: 19/19 admitted — 12 to the
choice block at exit 2, 7 to a §5.3 representability refusal at exit 3.
**0 unexpected.** The Go half asserts the device re-parses each canonical to
itself; this is the missing half on `me`'s side, and it is clean.

**(c) F-1's unreachability, verified structurally rather than inherited.**
`Key` is constructed at exactly two sites, `cascade.rs:670` and
`cascade.rs:1107`, both through `parse_extended_key`, which returns `Err`
for every version outside §4.3's five — so `conjunct_4_versions` cannot fail.
`parse_key` sets a non-zero fingerprint only inside the `[…]` branch, which
also sets an origin of length ≥ 1 (`parse_path("")` fails), and branch 1 has
its own `NoOrigin` check — so `conjunct_6_origins` cannot fail. `admit()` has
exactly three call sites (`admit.rs:337`, `gate.rs:223`, `gate.rs:224`), all
fed by `cascade()`.

**(d) The `--as` flag window.** `me sysw pack --as md1` is a clap
`unexpected argument '--as' found` at exit 2. F-3 is real and is exactly as
described.

**(e) Cost of the hand-rolled base58 on a hostile line.** A 20 000-character
single base58 token completes in 0.076 s wall. Quadratic, but not a hazard at
any plausible input size; base64 and hex blobs fail on the first excluded
character.

---

# 3. The implementer's dispositions — judged

## F-1 (conjuncts 4 and 6 have no vector row that can red them) — **deferral is SOUND**

Not "ships an unverifiable conjunct". P1 ships no unverified BEHAVIOUR: both
conjuncts are structurally unreachable from a cascade-produced `Parsed`
(measured, §2c), and their live enforcement points — the version gate inside
`parse_extended_key` and the origin check inside branch 1 — ARE gated, by
`neither/full-origin-ypub` and by all five `narrowed-4.2` rows respectively.
What is ungated is a restatement that cannot execute.

The deferral's owning phase is right, and for a reason the report understates:
P2.2 is the FIRST phase that could introduce a second construction route
(`md_codec::encode::Descriptor` in process), so a direct-construction unit
test in `admit.rs` lands in the same phase as the risk it covers, not after
it. Deferring it to P2.2 rather than P3 is what makes it a schedule decision
instead of a gate move.

One sharpening for P2.2's implementer: the close is a unit test that builds a
`Parsed` with a non-admitted `KeyVersion` and one with
`fingerprint != 0, origin: vec![]`, and asserts `admit()` refuses each. Do not
add a vector row — no cascade-reachable input can produce either state, so a
row would have to be a lie.

## F-3 (`MD1_PATH_SHIPPED == true` from P1, flag lands in P2.1) — **ACCEPTABLE**

The alternative was measured and is worse: computing carriage from the current
tree makes all seven `as-decides` gate rows unsatisfiable, and the plan's own
P1 gate requires every non-md1-execution host assertion green. The plan's build
order closes the window two phases before anything leaves the branch (P2.1
implements the flag; P3.4 is the merge and push), and nothing is pushed —
verified: head `5d236fb`, `git status --porcelain` empty, no tags.

The constant is documented at its definition with the reasoning and a pointer
to the finding, which is what makes it a named window rather than a latent
one. See N2 for the one operator-visible edge it leaves.

## F-2 and F-4 — checked and accepted

F-2's measurement is consistent with `nonstandard/parse.go:151` firing before
the `Title` gate at `:37`; the pinned `bluewallet-no-name` outcome is the true
one for a file with no `Policy:` line, and P1 emits the substituted
enumeration (*"it has 1 cosigner line"*), measured. F-4's three fixes are
present and the elision one is genuinely gated (§1.4).

---

# 4. Findings

## Minor

### M1 — five §6 rows print a device-behaviour claim that is measurably FALSE for a `multi` input

§6 states that the `sortedmulti` rows *"read over BOTH multi forms … and get
the same texts with the form name substituted"*, and carves out exactly ONE
row (the single-key wrapper) where the device-measurement parenthetical does
not transpose, because all three single-key `multi` twins are device REFUSE
**at parse**. That reason is not specific to the wrapper row: the device
refuses EVERY `multi` form at parse (`bip380.Parse`'s switch has only a
`sortedmulti` case), so no `multi` input ever reaches address derivation.

Constructed, by taking each vector row and rewriting `sortedmulti(` to
`multi(`. All six run at exit 3, and five carry a false sentence:

| input | emitted, verbatim | why it is false |
| --- | --- | --- |
| `tr(multi(2,…))` | *"`tr(multi(…))` is not a valid descriptor **even though the device's parser accepts it**"* | it does not; it fails `unknown script type: "multi"` |
| mixed-network `multi` | *"**The device accepts this descriptor** and then cannot derive any address from it"* | refused at parse |
| `…/<0;1>/*h` `multi` | *"**The device would silently derive** the UNhardened child and display addresses"* | refused at parse |
| `<0;2>/*` `multi` | *"**It accepts this descriptor** and then errors on every address"* | refused at parse |
| 16-key `sh(multi(…))`, 21-key `wsh(multi(…))` | *"**The device would accept it** and derive addresses whose coins cannot be spent"* | refused at parse |

**Not an implementation deviation** — P1 follows §6's transposition rule
literally, and the taproot row even substitutes the form name correctly
(`tr(multi(…))`). The residue is §6's, which named one member of the class and
missed five. Recorded here because P2.4 owns §6's per-row texts and because
the risk is specific: **if P2.4 writes its "verbatim" assertions from what
this build prints, it will pin five false statements about the device.**
Owning phase **P2.4**. Non-blocking: the refusal itself is correct in every
case and the operator's next action (fix the export) does not change.

### M2 — `suggested_descriptor_for`'s `_ => P2PKH` fallback names a script the operator's origin does not imply

`refusal.rs:suggested_descriptor_for` maps `path[0]` to a script and falls
back to `Script::P2PKH` for anything outside `{44,45,48,49,84,86}` hardened.
`e.wrapping_sub(HARDENED)` means an UNHARDENED purpose also lands in the
fallback.

Measured, on the file's own `promotion/11-origin-unhardened-refused` row:

```
input:  [4bbaa801/84/0/0]zpub6qpFgGWoG7bKm…kSRzJx
me:     … This one is `m/84/0/0`, which is not inferable.
        Supply the descriptor instead: pkh([4bbaa801/84/0/0]zpub…/<0;1>/*)
```

An unhardened `84/0/0` is a real coordinator export bug, and the remedy hands
the operator a LEGACY P2PKH wallet for a native-segwit key. The remedy is
executable and admitted, so §6h is satisfied and §5.4's compare prompt (P2.3)
would catch it — but the guess is unsound and it is invisible to the one test
that looks at these remedies: **`every_promotion_remedy_me_prints_is_an_input_me_admits`
feeds this exact input and passes, because `pkh(…)` is admitted.**

Minor rather than Important: nothing is packed on this path in P1, `--as` does
not exist yet, the operator's own path is quoted one sentence earlier, and the
key material in the remedy is correct — the failure mode is a harder restore,
not lost funds. Owning phase **P2.3/P2.4**: either name no script for an
unrecognised purpose and say so, or gate the mapping.

### M3 — §6h is gated for one of the three remedy generators

`every_promotion_remedy_me_prints_is_an_input_me_admits` covers
`suggested_descriptor_for` only. Two other generators print operator keys in
a "supply the descriptor" position and no test runs their output back through
the cascade:

* `refusal::promotion_fingerprint_no_path` — `[{fp}/84h/0h/0h]{key}`;
* `refusal::unsupported_key_version` — five per-version remedies.

Both are correct TODAY (measured, §1.4 — all three emitted remedies re-enter
`me` clean), so this is a gate gap, not a live violation: an elision
introduced in either would pass the whole 485-test suite. Owning phase
**P2.4**, which is where the per-row texts get their tests; the cheap close is
to extend the existing loop's input list and raise its floor.

### M4 — P1's §6 texts are paraphrases in several rows, and P2.4's "verbatim" must be written from §6

Deviations from §6's quoted spans, all in the direction of dropping public
citations or compressing a clause:

* key-count row: `(BIP-383)` and `` (`OP_CHECKMULTISIG`) `` both dropped;
* taproot-multisig row: `(BIP-387)` dropped;
* use-site-hardened row: `(BIP-32)` dropped;
* promotion-path-not-inferable: *"— taproot single-sig"* dropped, and quoted
  paths use `h` where §6 writes `'`;
* mixed-network: backticks around `tpub`/`xpub` dropped.

The `'`→`h` change is defensible (it is `bip32.Path.String()`'s own
rendering, and the emitted path must be the one `me` would print), but the
dropped BIP citations are not internal identifiers and the walk-W5 rule does
not license removing them. Non-blocking in P1 — §6's texts are P2.4's
deliverable, not P1's. Recorded so P2.4's author writes the assertions from
`SPEC_descriptor_input.md` §6 and reconciles, rather than from
`me`'s stderr. Owning phase **P2.4**.

### M5 — §6's key-identity row is unreachable from a BlueWallet file, though its own remedy names a duplicated cosigner LINE as the usual cause

§6's origin-contradiction text ends *"Check the export: a duplicated cosigner
line carrying the wrong key is the usual cause."* — i.e. it points at a
BlueWallet file. It cannot fire for one. Constructed:

```
Name: x
Policy: 2 of 2
Derivation: m/48h/0h/0h/2h
Format: P2WSH
DC567276: xpub6DiYrfRwNn…EUhpan
DC567276: xpub6DnT4E1fT8…hsY39Ge
```
```
me: this is not a wallet descriptor in any of the four forms `me` reads: …
    It looks most like a BlueWallet `Key: value` setup file, which failed
    because: the `DC567276` header appears twice with different values.
```

The cause is faithful mirroring of the device: Go's `seenKeys` map covers
cosigner lines, so a same-fingerprint collision becomes
`inconsistent header value`, branch 1 fails, and §6 row 1 (`unparseable`)
carries the reason. And it is exhaustive rather than accidental — in a
BlueWallet file all keys share one `Derivation:`, so two keys have the same
`(fingerprint, origin)` **iff** they have the same header key, which is
exactly the deduped case.

**Conjunct 8(b) IS reachable from a BlueWallet file** — different
fingerprints, same xpub — and fires correctly (measured: *"keys 0 and 1 are
the same key at the same derivation…"*), so only the (a) half is affected.

The emitted message is truthful and names the real line, so the outcome is
not worse than telling the operator nothing; no change is earned by the
journey rule. Recorded because §6's remedy sentence advertises a route that
does not exist. Owning phase **P2.4** (one clause, or an annotation).

### M6 — a truncated JSON export does not open the gate

A valid JSON wrapper opens the gate on T4 and lands on
`json-inner-malformed` (measured:
`{"label":"x","descriptor":"wsh(multi(2,"}` → exit 3, the wrapper row). A
file truncated one character earlier — not valid JSON — matches none of
T1–T4 (its single token starts with `{`, so T1 and T3 fail; no BlueWallet
`": "` key, so T2 fails) and gets the shipped record refusal at exit 4. Same
for `{}` and `{"label":"x"}`.

Faithful to §5.1's T1–T4 guidance, unpinned by any of the 37 gate rows, and
not a violation of either invariant: a truncated export is neither record
material (invariant 1) nor one of §4's happy paths (invariant 2). Recorded
because a truncated or half-copied export is a plausible operator input and
the record refusal's vocabulary (*"see sysw::classify"*) is the wrong
vocabulary for it. Best owner is a journey walk, not a code change; do not
widen T4 without a gate row to pin the widening.

## Nit

### N1 — an empty substitution in the use-site error

`gate/xpub-trailing-slash` prints *"the use-site path is not a path: ``."* —
empty backticks, because the offending tail is the empty string. Correct row,
correct exit code; the sentence just reads as a bug. `refusal.rs`'s
`describe_key` / `KeyError::InvalidChildrenPath`.

### N2 — the `--as` window's operator-visible shape

Consequence of F-3, recorded so it is on the record rather than only in the
implementer's report. In this build the choice block offers `--as md1`, and
running it produces:

```
error: unexpected argument '--as' found
  tip: a similar argument exists: '--no-passphrase'
```

An operator who follows `me`'s own advice is told to try `--no-passphrase`.
Closed by P2.1, which is before anything is merged or pushed; no action.

---

# 5. What I did not find

Stated explicitly, because a negative inherits the scope it was searched in.

* **No transposed branch and no fallthrough the device lacks** — searched by
  reading `cascade.rs` against `nonstandard/parse.go` and `bip380/bip380.go`
  at `1f09537`, statement by statement, plus the 71-row and 19-canonical
  runs above.
* **No path by which `me` is WIDER than the device.** Searched three ways:
  the JSON/branch-4 divergence (proved impossible by input shape), the
  §4.2/§4.5 narrowings (all make their branch FAIL, and no other branch can
  claim a BlueWallet file or a bare key), and the canonical fixed point (19/19).
  Not searched: a generated differential against a live `bip380.Parse` — no Go
  toolchain is on this box's PATH, so I could not run one. That is the one
  gap in this negative, and the Go seam suite covers the 71 rows within it.
* **No internal identifier in any P1-emitted refusal** — searched over 120
  inputs by regex on raw stderr, streams separated (see §1.4). The six hits
  are all pre-existing surface.
* **No path where a valid record stream reaches the gate**, and none where a
  descriptor-shaped input reaches the record refusal other than the
  gate-closed arm §5.1 specifies — searched by reading every branch of
  `read_records` and of `run_sysw`'s `Pack` arm ahead of `admit_check`, and
  by exercising all three input channels.
* **No refusal that writes stdout or creates `--out`**, and none that runs
  after the passphrase ceremony.

---

# 6. Verdict

**0 Critical / 0 Important / 6 Minor / 2 Nit — GREEN.**

P1 implements the plan's P1.0–P1.2 as written and is faithful to §4.1–§4.7,
§5.1, §5.4 and §6's discipline. Both implementer dispositions (F-1's deferral,
F-3's named window) are sound and correctly owned. The six Minors are all
either §6-text residue owned by P2.4 or a gate-coverage gap; none of them
changes an outcome, and none of them blocks the P1 gate or P2's dispatch.

Nothing was modified, committed or pushed. Tree clean at `5d236fb`.
