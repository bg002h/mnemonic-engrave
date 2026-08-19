# Item 5 — key re-derivation execution review

**Commit under review:** `e59ce9f12c48e0cb154067acdede0cb86ba0b623`
(`fix(journeys): item 5 -- re-derive the 11 keys at depth 4, so ADDRESSES DERIVE`)
**Repo:** `/scratch/code/shibboleth/mnemonic-engrave`
**Reviewer:** independent adversarial execution review, read-only on tracked files.
**Date:** 2026-08-19
**The one question:** does this commit derive the correct keys, and are the
addresses it now produces the right addresses for the intended wallet?

Tree was clean at start and clean at finish (`git status --porcelain` empty both
times). Nothing tracked was written. `design/journeys/out/**` was regenerated,
which is untracked build output.

---

## Verdict

**The keys are correct and the addresses are correct.** All eleven committed
xpubs reproduce exactly under a from-scratch BIP-39/BIP-32 derivation written for
this review (pure Python, `hashlib` + `ecdsa` point arithmetic, own base58check),
self-tested first against BIP-32's published test vector 1 and BIP-39's published
`TREZOR`-passphrase seed vectors — a route that shares no code with `ms`, `md`,
`mk` or `me`. The master fingerprints are unchanged (`73c5da0a` / `b8688df1` /
`28645006`), every new key is depth 4, the pre-commit files reproduce exactly at
`m/84'/0'/N'` and the post-commit files exactly at `m/48'/0'/N'/2'`, so the
`@i → (master, account)` mapping provably did not move. All six addresses in the
re-recorded transcript — 3 receive and 3 change — were reproduced **byte-for-byte
by Bitcoin Core v25.0.0 `deriveaddresses`**, driven from `wallet-policy.txt` and
the committed key files directly, with Core doing its own miniscript compilation.
The control claim holds: running the generator with `--template bip84` reproduces
all eleven pre-commit files byte-for-byte (`diff -r` clean, matching md5). The
`--check` mode really fails, under all six mutation classes tried, including a
swap of two *valid* keys between slots. Both the transcript and every artifact in
`out/` are byte-identical across two runs from a removed `out/`.

**Nothing found here touches key or address correctness. What is defective is the
document built on top of them.** The commit's own stated remedy for the plate
captions does not work: `card-index.txt` records the right per-key chunk counts,
but in *key-file order*, while `me bundle` emits plates ordered by
`chunk_set_id`. Measured against `mk decode` of what is actually on each plate,
**30 of 30 card-plate captions in the generated document name the wrong key.**
The previous `(n-4)//2` was equally wrong; the change removed a loud `IndexError`
and left a silent wrong answer in its place. Three sentences of `bip84` prose that
the diff never touched are now false and still render, the shipped tracked PDF was
not rebuilt and still carries the old depth-3 xpubs, and the transcript's new
`9b. THE ADDRESSES` section — the commit's headline result — is never placed by
the builder, so the addresses appear nowhere in the journey document.

---

## Independent key verification

Method: `mnemonic_to_seed` = PBKDF2-HMAC-SHA512(NFKD mnemonic, `"mnemonic"`,
2048, 64); BIP-32 master = HMAC-SHA512(`b"Bitcoin seed"`, seed); hardened CKD;
own base58check serialisation. Validated before use:

```
m xpub   xpub661MyMwAqRbcFtXgS5sYJABqqG9YLmC4Q1Rdap9gSE8NqtwybGhePY2gZ29ESFjqJoCu1Rupje8YtGqsefD265TMg7usUDFdp6W1EGMcet8   (BIP-32 vector 1, exact)
deep     xpub6H1LXWLaKsWFhvm6RVpEL9P4KfRZSW7abD2ttkWP3SSQvnyA8FSVqNTEcYFgJS2UaFcxupHiYkro49S8yGasTvXEYBVPamhGW6cFJodrTHy   (m/0'/1/2'/2/1000000000, exact)
MATCH abandon / MATCH legal / MATCH letter   (BIP-39 published vectors, passphrase "TREZOR")
```

Masters, independently computed: `A 73c5da0a`, `B b8688df1`, `C 28645006` —
identical to the fingerprints the committed headers carry, and to the pre-commit
headers.

| key | master / acct | committed xpub (HEAD) | independent derivation of `m/48'/0'/N'/2'` | match |
| --- | --- | --- | --- | --- |
| key-00 | A / 0 | `xpub6DkFAXWQ2dHxq2vatrt9qyA3bXYU4ToWQwCHbf5XB2mSTexcHZCeKS1VZYcPoBd5X8yVcbXFHJR9R8UCVpt82VX1VhR28mCyxUFL4r6KFrf` | identical | ✅ depth 4 |
| key-01 | A / 1 | `xpub6DzhyrnFFYQ1HimDiM388xHnDiRPNdZJFBmmxge3Y1WWcHLtMJLfRuhRHqnQCPbTj3fGKTuKFLHzzwpJkp5Dtc3UtLKZKaVZe1yqMBXd6Vk` | identical | ✅ depth 4 |
| key-02 | A / 2 | `xpub6EGx8sPr9FxPPE1rbZazhqWwpMXA3Hf5DYKtZbL7c4BSddzmQktp96UaTvecEkoCZysuaj79GMCFZYT1KKk7Ph2M3Kf5g8B82KZ8TZ9SKQR` | identical | ✅ depth 4 |
| key-03 | A / 3 | `xpub6E6Z3Ss5TXJYNJp4U1q3NZ3pCn82i7KXQAKUtNnzLJ3cCdchQeSdFvXemizaHUF7wNwRQAB8mPdoZhGHLiv49cWPtCnoJY3Az3E8JKxH9Mq` | identical | ✅ depth 4 |
| key-04 | B / 0 | `xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX` | identical | ✅ depth 4 |
| key-05 | B / 1 | `xpub6EAMBJLn1jiquajTsNRkZXU1oKnA4WJMNvcz4FRR4QmFKdfHxJVvfRLoysWfcc16AMTR4CoMD8UNjvs9JtbsLeuLwpTczgq8zuuERnp8YZF` | identical | ✅ depth 4 |
| key-06 | B / 2 | `xpub6EGceQV5wHrRemjYkJykRWQefTvsNTw1Lr7JUpQTgaaWz2P6Cg75o3LktGoyMPshKbCqrgy2RXxAmkqBXkbyqPe52CnwA1LUnyBkNUuMhRt` | identical | ✅ depth 4 |
| key-07 | B / 3 | `xpub6ELyn7moeEZ9hJ1HmSm6G5hAfgcCeTa71X9PzYJ39tHNUj93e9MaKb2tjAFrMFb94NMeTG7MW8dwxuhhnPhn5swY6r5dxZH6cyiuPd25AQ5` | identical | ✅ depth 4 |
| key-08 | C / 0 | `xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6` | identical | ✅ depth 4 |
| key-09 | C / 1 | `xpub6F6gx8ZP9R3R3eYsU2PeS5EPJ4jN7Wbt9uwyHNXLoaJxQjNT92FGAfCNDjUDRhCHwzjfgDuqAZ7Gk9SugPRMa6A8PnzLVvnyEKBW9jHRGRp` | identical | ✅ depth 4 |
| key-10 | C / 2 | `xpub6Dh4twkBRkzxDSau8FdhVY16jARGnPtfdRUk6LbePrGUGok6xE8SvGtaygCMLV7beB55YtzkR9dJoic9zYbYy3C7EKc2FGLkKoPT4mQUjPg` | identical | ✅ depth 4 |

The `origin [fp/path]` header of each file also matched the independently
computed fingerprint and path string, character for character.

**Master/account mapping (adversarial angle 5) — verified without trusting the
`ROWS` table.** The same eleven `(master, account)` pairs were derived at
`m/84'/0'/N'` and compared against the **pre-commit** files
(`git show e59ce9f^:…/key-NN.xpub`):

```
=== PRE-COMMIT (e59ce9f^) vs INDEPENDENT m/84h/0h/Nh ===
key-00 depth 3 MATCH  … key-10 depth 3 MATCH
ALL MATCH
```

So the mapping `ROWS` hardcodes at `derive-pathological-keys.sh:47-59` is
provably the mapping the pre-commit fixture already used. No master and no
account index moved; only the path template did.

---

## Address verification

**Route:** Bitcoin Core v25.0.0, a wholly separate implementation, with its own
miniscript parser and its own key derivation. The descriptor was assembled in
Python from `design/journeys/inputs-pathological/wallet-policy.txt` (unmodified by
this commit) with each `@i` replaced by `[fp/path]xpub` read out of the committed
key files, then `<0;1>` split into `/0/*` and `/1/*`. `md` was not involved.

`getdescriptorinfo` returned `"issolvable": true` for both, checksums
`#0cejh7k2` (receive) and `#489khcsk` (change). `deriveaddresses "<desc>" '[0,2]'`:

| chain | idx | Core v25 `deriveaddresses` | `transcript_pathological.txt` (`md address`) | match |
| --- | --- | --- | --- | --- |
| 0 | 0 | `bc1qkuknuy6dsm0fq44cyyhzqy9wl3ex2n6ed39zxhx867l9wlh4yhlsejms64` | line 202, identical | ✅ |
| 0 | 1 | `bc1q6k2emh2epd57p5qfwswnahtphjlzsuc5x92c7zphr2lnnckzrpzqjdaraf` | line 203, identical | ✅ |
| 0 | 2 | `bc1q97cvjd2wj75hu62w4at6w4ay0mz3janslla3y3zrml4lglymm38qfh70pa` | line 204, identical | ✅ |
| 1 | 0 | `bc1qxtlrmq23lczrtx8l9x3nsfp3u5h7vt9zjmy94m6srl354ps9amus2q56lx` | line 209, identical | ✅ |
| 1 | 1 | `bc1q6keuktp5nwv0mtr593gj8p4yvpvt5gl4hwhzyxzuyu66mtj7leaquyk8qt` | line 210, identical | ✅ |
| 1 | 2 | `bc1qcyn83peqe3n6flckqwf3fvr2g0y2z8hplfxfm8nfecazxas4ux2sjjwuld` | line 211, identical | ✅ |

This is a stronger result than a spot check: `md address` in the transcript
derives from the **encoded md1 payload** (`md encode --force-chunked --path bip48
--key @i=…` at `transcript_pathological.sh:224-230`), so the agreement also
demonstrates that `md`'s encode → md1 → address path preserved all eleven keys
through the wire format without corrupting one.

The flattened shared origin that `--path bip48` records (`m/48'/0'/0'/2'` for all
eleven, per `descriptor-mnemonic/crates/md-cli/src/parse/path.rs:31`) does **not**
affect these addresses — origin metadata is descriptive, and the derivation runs
from the xpub plus `/chain/index`. My Core descriptor carried each key's *true*
per-key origin and produced the same addresses, which confirms it.

---

## Findings

### Critical — none

No defect was found in the key derivation, the master/account mapping, the depth,
the fingerprints, the address derivation, the control, or the drift check.

### Important

**I-1. Every card-plate caption in the generated journey document names the wrong
key. The commit's stated fix for this does not work.**
`build_pdf_pathological.py:257-265` (`_plate_owner`) walks `card-index.txt` in
key-file order 0…10 and assumes plate order follows it. It does not: `me bundle`
groups mk1 chunks into a `BTreeMap` keyed by `chunk_set_id`
(`crates/me-cli/src/bundle.rs:205-208`, *"BTreeMap keeps a deterministic order"*)
and emits plates in **chunk-set-id order**, an unrelated permutation of key order.
Measured by decoding what is actually engraved on each plate with `mk decode` and
comparing to the caption `build_pdf_pathological.py:282` produces:

```
plate  CAPTION (builder)                       ACTUAL (mk decode)
    4  @0 73c5da0a/48'/0'/0'/2' 1/2            b8688df1/48'/0'/1'/2' 1/3    WRONG
    5  @0 73c5da0a/48'/0'/0'/2' 2/2            b8688df1/48'/0'/1'/2' 2/3    WRONG
   13  @3 73c5da0a/48'/0'/3'/2' 2/3            73c5da0a/48'/0'/0'/2' 1/2    WRONG
   15  @4 b8688df1/48'/0'/0'/2' 1/2            28645006/48'/0'/0'/2' 1/2    WRONG
   33  @10 28645006/48'/0'/2'/2' 3/3           28645006/48'/0'/1'/2' 3/3    WRONG

30 of 30 card plates carry a WRONG caption
```

Confirmed in the rendered document, not only in simulation — after running
`python3 build_pdf_pathological.py`, `out/pathological/journey_pathological.html`
contains `plate 4 — @0 [73c5da0a/48'/0'/0'/2'] chunk 1/2`, while plate 4 in the
same run's `manifest.json` is `mk1qpqs7n…`, which `mk decode` resolves to
`b8688df1 / 48'/0'/1'/2'` — key `@5`.

The true plate→key mapping for this run is
`4-6→@5, 7-9→@3, 10-12→@2, 13-14→@0, 15-16→@8, 17-19→@10, 20-22→@1, 23-25→@6,
26-28→@7, 29-30→@4, 31-33→@9`.

Failure scenario: the engraved **bytes** are correct, so a restore that reads all
33 plates recovers the right wallet — this is not funds-losing on its own. But an
operator distributing cosigner cards by the document's labels hands every
cosigner the wrong card; an operator auditing "have I engraved all of @0's
chunks?" is told 1/2 and 2/2 when the real card is 1/3, 2/3, 3/3; and if a plate
is lost, the document identifies the wrong cosigner as compromised.

Note precisely what changed: the total is right (`card-index` sums to 30, and the
manifest has exactly 30 mk1 plates), so the `IndexError` the commit message
describes is genuinely gone. It has been replaced by a silent wrong answer, which
is the worse of the two. The pre-commit `(n-4)//2` was wrong in the same way, so
this is not a regression — but it is the specific defect this commit set out to
fix, and it is not fixed. The fix is to key the caption on the plate's own
`chunk_set_id` (already in `manifest.json`) rather than on ordinal position, or to
record `chunk_set_id` in `card-index.txt` and join on it.

**I-2. Three sentences of `bip84` prose the diff never touched are now false and
still render.** `build_pdf_pathological.py:134-135` — *"The 11 keys come from
three masters at divergent account indices (`84'/0'/0..3'`), so no key is
reused."* And `:181-183` — *"It records ONE shared origin — `bip84` and
`m/84'/0'/0'` produce a byte-identical chunk set here. These eleven keys actually
sit at four account indices (`84'/0'/0..3'`) across three masters…"* All three
are false after this commit; the transcript block rendered immediately beneath the
second one now shows `--path bip48`. Confirmed to reach the output: the generated
`journey_pathological.html` contains `84'/0'/0..3'` twice and the
`<code>bip84</code> and <code>m/84'/0'/0'</code>` sentence once. The surrounding
*argument* (shared origin flattens; the key cards carry the truth) is still sound
— only the paths are stale.

**I-3. The shipped, tracked PDF was not rebuilt and still publishes the old
depth-3 keys.** `design/journeys/SeedHammer-II-pathological-wallet-journey.pdf` is
tracked and its last touching commit is `bdf954f`, not `e59ce9f`. `pdftotext`
extraction: **4** occurrences of the retired `xpub6CatWdiZiodmU…`, **2**
occurrences of `84'/0'/0..3'`, and **0** occurrences of `bc1q`. `design/journeys/
README.md:6` states *"Nothing in these documents is illustrative."* Failure
scenario: a reader who opens the published PDF — the artifact the README points
them at first — reads eleven xpubs from which, as this very commit establishes, no
address can be derived, with no indication they have been superseded. (Mitigating:
the README already carries a "Corrections to the published documents" section, so
there is an established place to say so, and F-210 covers the general class.)

### Minor

**M-1. The commit message's plate count is wrong on the "before" side.** It
states `PLATES 25 -> 34`. The pre-commit transcript's own checklist ends
`plate 26/26  ms1 secret` (`git show e59ce9f^:…/transcript_pathological.txt`), and
the post-commit one ends `plate 34/34`. So the transition is **26 → 34**; `25` is
the count of *rendered PNGs* before (33 after), mixing two conventions in one line.
The other two numbers on that list are exact: mk1 chunks 22 → 30 and bundle
25 → 33 lines both reproduce (`card-index` sums to 30; `wc -l backup-strings.txt`
= 33).

**M-2. The commit's headline result never reaches the journey document.**
`transcript_pathological.sh:211` emits section `9b. THE ADDRESSES`, but
`build_pdf_pathological.py` places only ten sections and `9b` is not among them
(`grep -on "sect('[^']*')"` → `versions, 1., 2., 3., 4., 5., 9., 7., 8., 10.`).
The generated `journey_pathological.html` contains **0** occurrences of
`bc1qkuknuy6…`. Sections `6.` and `7b.` are likewise emitted and never placed.
Nothing checks that the document covers the transcript, so a new section is silent
when it is dropped.

**M-3. `--check` cannot detect a change of derivation template.** It compares each
file against a fresh derivation using whatever `--template` the script currently
names (`derive-pathological-keys.sh:70-71`); nothing asserts the result is depth 4
or that the path is BIP-48. Editing the template and re-running the generator
leaves `--check` green on depth-3 keys. **Measured mitigation:** `md encode` still
refuses depth 3 —
`md: --key @0: expected depth 4 for this script context, got 3` — and
`transcript_pathological.sh:231-234` FATALs on an empty `$FULL`, so a regression
cannot pass silently through the journey. Downgraded to Minor on that basis.

### Nit

**N-1. The generator has no callers.** `grep -rn "derive-pathological-keys"` over
the repo returns only three hits, all inside the script's own comments. `--check`
is a gate that nothing runs — it works (proven below), but only when a human
remembers it. `transcript_pathological.sh` would be the natural place.

---

## Claims checked

| claim | holds? |
| --- | --- |
| Master fingerprints unchanged: `73c5da0a` / `b8688df1` / `28645006` | ✅ independently computed from the three seeds; identical pre- and post-commit |
| The three seeds are BIP-39 published test vectors | ✅ `abandon×11 about`, `legal winner…yellow`, `letter advice…above` — entropy `00…`, `7f…`, `80…`; all three reproduce the published `TREZOR` seeds exactly |
| Keys re-derived at BIP-48 P2WSH `m/48'/0'/N'/2'` | ✅ all 11 reproduce under independent BIP-32; all depth 4 |
| Only the account path moved (same master, same account index per `@i`) | ✅ pre-commit files reproduce exactly at `m/84'/0'/N'` for the same `(master, account)` pairs |
| Control: `--template bip84` reproduces all 11 pre-commit files byte-for-byte | ✅ `diff -r` clean over all 11; md5 `d22ceffa4af669d8323becfc9c5b0112` matches on key-00 |
| The two addresses quoted in the commit message | ✅ both reproduced by Bitcoin Core v25 `deriveaddresses` |
| All six addresses in the transcript are right for this wallet | ✅ 6/6 match Core, derived from `wallet-policy.txt` + the committed key files |
| `--check` detects drift and exits non-zero | ✅ 6/6 mutation classes caught: one flipped base58 char; **two valid keys swapped between slots**; a changed fingerprint in the *comment* only; a stripped trailing newline; a deleted file; clean again after restore (exit 0) |
| `mk1` chunks 22 → 30, accounts 1-3 need 3 and account 0 fits 2 | ✅ `card-index.txt` reads `2,3,3,3,2,3,3,3,2,3,3` = 30; manifest has 30 mk1 plates |
| Bundle 25 → 33 lines | ✅ `wc -l backup-strings.txt` = 33 |
| Plates 25 → 34 | ⚠️ post is 34; **pre was 26**, not 25 (M-1) |
| Operator journey unaffected — its twelve keys were already depth 4 | ✅ all 12 `inputs/keys/cosigner-*.xpub` decode to depth 4 |
| Step 5's origin is no longer hardcoded | ✅ read out of `key-00.xpub` at `transcript_pathological.sh:98-101` |
| Determinism: transcript reproduces the committed `.txt` | ✅ `diff` clean against `transcript_pathological.txt` on a run from a removed `out/` |
| Determinism: two runs from a removed `out/` agree | ✅ transcript identical; `diff -r` over all of `out/pathological` identical |
| Generator is idempotent on repeat runs | ✅ `--check` green immediately after a write run |
| The journey now records the plate→key mapping and the builder reads it | ❌ it records per-key chunk *counts* in key order; plate order is chunk-set-id order — **30/30 captions wrong (I-1)** |
| Nothing else is still on the old path | ❌ `build_pdf_pathological.py:134-135,181,183` (I-2); the tracked PDF (I-3) |

---

## Open / could not determine

- **Whether `md` is *right* to demand depth 4 is still unanswered, and the commit
  did not answer it.** `design/DoNextList.md:310-311` names this explicitly:
  *"Unresearched and it decides between 1 and 2: is md right to demand depth 4?
  That is an external-protocol question nobody has checked."* The commit took
  option 1 without checking. I can state only the narrow part: BIP-48 does place
  multisig account keys at `m/48'/coin'/account'/script_type'` with `2'` = P2WSH,
  so `m/48'/0'/N'/2'` is a conventionally correct four-level multisig path and the
  chosen fix is consistent with BIP-48. Whether `md` is right to *require* depth 4
  for every `wsh(<miniscript>)` — BIP-388 wallet policies do not mandate a depth,
  and the check at `crates/md-cli/src/parse/keys.rs:67-77` is an exact equality,
  not a minimum — is a normative admission question outside this review's scope
  and I did not settle it.

- **Whether using four *account indices* of the same master as four distinct
  cosigner keys is the intended wallet shape.** Keys @0-@3 are all master A at
  accounts 0-3, @4-@7 all master B, @8-@10 all master C. That is unusual for a
  real multisig (normally one cosigner device contributes one key) but it is
  identical to the pre-commit structure and appropriate for a synthetic fixture. I
  treat it as intended, not as a finding, but I did not find a document that says
  so.

- **Whether the flattened shared origin is right for restore.** `--path bip48`
  records one shared origin for all eleven keys while their true origins differ by
  account. This is pre-existing, documented in the PDF as its own "finding 3", and
  does not affect the addresses (verified above). Unresolved as a design question,
  and untouched by this commit.

- **The taproot sibling fixtures were not exercised.**
  `inputs-pathological/backup-strings-tr.txt` and `wallet-policy-tr.txt` are
  keyless templates carrying no xpubs, are unmodified by this commit, and have
  zero consumers (already recorded in `design/agent-reports/decision-item8-item9.md:181`).
  I confirmed the md1 decodes to the tr template and left it there.

---

## Method notes (for reproduction)

- Independent BIP-32/BIP-39: `hashlib`/`hmac` + `ecdsa` 0.19.2 for secp256k1
  points, own base58check. Self-tested against BIP-32 vector 1 (master + the
  `m/0'/1/2'/2/1000000000` deep node) and the three BIP-39 `TREZOR` seed vectors
  before any project key was touched.
- Independent addresses: `bitcoind` / `bitcoin-cli` **v25.0.0**, mainnet, isolated
  datadir under the scratchpad, `-maxconnections=0 -dnsseed=0 -listen=0`, RPC port
  18999; stopped afterwards. A pre-existing user `bitcoind` (PID 760577, started
  2026-08-02) was not touched.
- Control and mutation runs were done on **copies** of `design/journeys/` in the
  scratchpad. The only in-repo commands were `--check` (read-only), the transcript,
  and the HTML builder — all of which write solely under untracked
  `design/journeys/out/`.
