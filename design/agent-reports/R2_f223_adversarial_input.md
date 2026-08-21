# R2 — adversarial input review of `mk encode --keys` + `--from-md1` chunk grouping (F-223)

**Reviewer:** independent adversarial agent (did not author the code).
**Scope:** `/scratch/code/shibboleth/mnemonic-key`, `0feaaaa..main`, excluding `vendor/`.
**Focus:** `crates/mk-cli/src/keyfile.rs` (`parse_key_record`, `read_key_records`),
`crates/mk-cli/src/cmd/mod.rs` (`md1_chunk_set_id`, `group_md1_cards`),
`crates/mk-cli/src/cmd/encode.rs` (the batch mint/emit path).
**The one question:** what input breaks `mk encode --keys` or `--from-md1` chunk
grouping — producing a panic, a silently wrong card, a short/partial bundle, or an
accepted record that should have been refused?

**Method:** ~90 attacks actually executed against the built binary. Every finding
below is reproduced by a command in its Evidence block. Scratch harnesses live in
`/tmp/claude-1000/r2attack/` (`a1..a10.py`, `c1.py`, `c2.py`, `b1.py`,
`xpubinfo.py`).

## Binary provenance (read this before reproducing)

The working tree at review time carried an **uncommitted mutation-testing edit** in
`crates/mk-cli/src/cmd/encode.rs` (a deliberate MUTANT that stamps every card with
the first record's path). The binary tested,
`/scratch/code/shibboleth/mnemonic-key/target/release/mk`, is **not** that mutant:
it is dated `15:21:20`, the mutant edit is dated `16:14:08`, and the binary was
confirmed clean by minting a 3-record batch and decoding each card — the three
cards carry `48'/0'/0'/2'`, `48'/0'/1'/2'`, `48'/0'/2'/2'` respectively, which the
mutant could not produce. All findings below are against committed `main`
(`f887d57`) behaviour.

---

## CRITICAL — `--keys` + `--from-md1` never compares the key records to the cosigner set of the policy it is binding them to: a SHORT bundle mints silently with exit 0

`crates/mk-cli/src/cmd/encode.rs:131-179` (stubs come from `--from-md1`, cards come
from `--keys`, the two lists are never joined) · `crates/mk-cli/src/keyfile.rs:99-114`
(`read_key_records` has no set-level validation of any kind)

**Failure scenario:** wallet A is a **2-of-2** whose md1 chunk set carries both
cosigners' fingerprints *and* both cosigners' public key material. The operator
mints the key cards for it with a key file holding **one** record. `mk` derives the
policy stub from the very chunk set that says there are two cosigners, stamps it on
one card, exits 0, and says nothing. Engraved, that is a one-plate bundle for a
two-key wallet — the known past failure mode of this project, produced by the
command that had both halves of the check in hand.

The same silence covers the neighbouring errors: a record whose key is in **no**
slot of the policy still gets a card stamped with that policy's stub (a plate that
falsely claims membership), and 3 cards for a 2-of-2 is equally accepted.

**Evidence** (`/tmp/claude-1000/r2attack/b1.py`):

```
$ md decode --json <wallet A's 4 chunks>
  "n": 2,
  "fingerprints": [[0,"73c5da0a"],[1,"aabbccdd"]],
  "pubkeys": [[0,"bba0c7ca…5763021a3bf5…9f29"],[1,"9960c4a3…6370249094a2…ee3"]]

one_record_for_2of2          ACCEPT card_count=1  (wallet A is a 2-of-2)
both_records                 ACCEPT card_count=2
stranger_only                ACCEPT card_count=1   <-- key belongs to no slot
both_plus_stranger           ACCEPT card_count=3
one_right_one_stranger       ACCEPT card_count=2
```

stderr in every case is only `note: stdout is watch-only …`.

The comparison data is **already parsed in this process**: `derive_stub_from_md1_card`
(`cmd/mod.rs:126-139`) calls `md_codec::reassemble` and holds the `Descriptor`, and
slot 0's `pubkeys` blob above is byte-for-byte `chaincode || pubkey` of the key in
record 1 (verified independently:
`chaincode=bba0c7ca160a870efeb940ab90d0f4284fea1b5e0d2117677e823fc37e2d5763`,
`pub=021a3bf5fbf737d0f36993fd46dc4913093beb532d654fe0dfd98bd27585dc9f29`; slot 1
likewise matches the second key). `card_count` vs `descriptor.n`, and set-equality
of the record keys against the slot keys, are both a byte comparison away.

**Verdict: CONFIRMED.**

---

## CRITICAL — an 11..255-component origin path encodes cleanly and can never be decoded: `mk encode --keys` mints a write-only card and reports success

`crates/mk-codec/src/bytecode/path.rs:85-98` — `encode_path` writes
`out.push(components.len() as u8)` with **no** `MAX_PATH_COMPONENTS` check;
`decode_explicit_path` at `path.rs:113-116` **does** enforce it
(`count > MAX_PATH_COMPONENTS → PathTooDeep`). Reached from
`crates/mk-cli/src/cmd/encode.rs:173`.

**Failure scenario:** a record whose origin path has 11+ components and whose xpub
depth matches it (so the `XpubOriginPathMismatch` guard at
`crates/mk-codec/src/bytecode/encode.rs:36-47` is satisfied). `mk encode` emits a
well-formed multi-chunk mk1 card and exits 0. `mk decode` and `mk verify` both
refuse that same card with `path too deep: 11 components (max 10)`. Engraved in
metal, it is unrecoverable — and in a batch the failure is invisible, because the
other records mint correctly and the run still exits 0 with a full-looking bundle.

**Evidence** (`/tmp/claude-1000/r2attack/a7.py`, `a8.py`):

```
depth4   ENCODE-OK chunks=3  DECODE-OK  verify=0
depth10  ENCODE-OK chunks=3  DECODE-OK  verify=0
depth11  ENCODE-OK chunks=3  DECODE-FAIL(2)  verify=2   path too deep: 11 components (max 10)
depth20  ENCODE-OK chunks=4  DECODE-FAIL(2)  verify=2   path too deep: 20 components (max 10)
depth255 ENCODE-OK chunks=26 DECODE-FAIL(2)  verify=2   path too deep: 255 components (max 10)

# 3-record batch, poisoned record in the middle:
BATCH exit=0  stderr='note: stdout is watch-only — public keys only, cannot spend'
card_count = 3
  card 0: chunks=2 decode=OK   verify_rc=0  48'/0'/0'/2'
  card 1: chunks=3 decode=DEAD verify_rc=2  path too deep: 11 components (max 10)
  card 2: chunks=3 decode=OK   verify_rc=0  48'/0'/2'/2'
TEXT exit=0 blocks=3 stderr='note: stdout is watch-only …'
```

**Not introduced by F-223** — the same input via the single-card path
(`mk encode --xpub … --origin-path m/0'/…/2'`) mints the same dead card, so the
defect is mk-codec's missing encoder-side cap. F-223 is what makes it a *bundle*
problem: one bad line among N still exits 0. The 256-component case is unreachable
(the `as u8` truncation to 0 would need `xpub.depth == 256`, impossible for a `u8`),
so the exposed window is exactly 11..=255.

**Verdict: CONFIRMED.**

---

## IMPORTANT — `read_key_records` performs no cross-record validation: three provably contradictory key sets mint without a murmur, one of them being N cards for N−1 cosigners

`crates/mk-cli/src/keyfile.rs:99-114`

**Failure scenario:** the loop parses each line in isolation and pushes; nothing
ever looks at the set. Three contradictions survive:

1. **the identical record twice** — a hand-edited 3-cosigner file with one key
   pasted twice and one forgotten mints **3 cards for 2 cosigners**. The bundle
   looks complete (3 plates for a 3-cosigner wallet) and is not.
2. **one origin, two different xpubs** — `md-codec` refuses exactly this shape
   intra-card (F-217, `descriptor-mnemonic/crates/md-codec/src/validate.rs:311-345`):
   *"BIP-32 is deterministic: a (master fingerprint, derivation path) pair
   identifies exactly ONE extended key … a card binding one such pair to two
   different xpubs therefore describes a wallet that cannot exist."* `mk --keys`
   mints both plates, each stamped `[73c5da0a/48'/0'/0'/2']`, carrying different
   keys. At recovery a signer following that origin finds one key, so one plate is
   provably wrong metal.
3. **one xpub, two different origins** — the mirror of (2), equally provable
   without a seed, equally accepted.

**Evidence** (`/tmp/claude-1000/r2attack/a9.py`):

```
same_origin_diff_xpubs       ACCEPT cards=2
      card 0: [73c5da0a/48'/0'/0'/2'] xpub6DkFAXWQ2dHxq2vatrt9...
      card 1: [73c5da0a/48'/0'/0'/2'] xpub6DzhyrnFFYQ1HimDiM38...
same_xpub_diff_origins       ACCEPT cards=2
      card 0: [73c5da0a/48'/0'/0'/2'] xpub6DkFAXWQ2dHxq2vatrt9...
      card 1: [73c5da0a/48'/0'/1'/2'] xpub6DkFAXWQ2dHxq2vatrt9...
identical_record_twice       ACCEPT cards=3   (cards 1 and 2 byte-identical)
dup_line_1000x               ACCEPT cards=1000
```

`md` refuses both classes for its own inputs, and refused one of them during this
review while I was building fixtures:
`md: codec error: @0 and @1 declare the same key origin ([73c5da0a/48]) but
different xpubs; one origin identifies exactly one key, so this card describes a
wallet that cannot exist` and
`@0 and @1 carry the same key at the same use-site: this policy names 10 cosigners
but one of them holds two of the seats` (F-218). The sibling repo treats these as
blocking; `--keys` — whose whole premise is "one record carries its fingerprint,
path and key together and cannot come apart" — does not look.

**Verdict: CONFIRMED.**

---

## IMPORTANT — batch output identifies no card, so caption-by-order is the only option, which is the exact desync the module docstring exists to prevent

`crates/mk-cli/src/cmd/encode.rs:187-202` (text: blank-line-separated blocks, no
labels) and `:233-257` (`emit_json_batch` / `card_json`: each entry carries only
`mk1_strings`, `chunk_count`, `code_variant`).

**Failure scenario:** `keyfile.rs:13-19` justifies the record format by citing an
incident where "an ordering assumption captioned 30 plates with the wrong
cosigner", and fixes it on the *input* side. The *output* side then hands back N
unlabelled blocks whose only link to the records is position. An engraving session
that reorders, retries, or re-mints one replacement plate has no way to check a
card against its cosigner except by re-running `mk decode` on each one. The
unsound assumption — card order stays aligned with file order across a 30-plate
session — is the one this project already has an incident for.

**Evidence:**

```
$ mk encode --keys keys3.txt --policy-id-stub 5b48af35 --group-size 0 --json
{"card_count":3,"cards":[{"chunk_count":2,"code_variant":"long","mk1_strings":[…]},…]}
        # no fingerprint, no origin path, no record index anywhere in the envelope

$ mk encode --keys keys3.txt --policy-id-stub 5b48af35 --group-size 0
        # 3 blank-line-separated blocks, 2 / 3 / 3 lines, nothing naming a cosigner
```

Card order *is* file order (verified with comments and blank lines interleaved —
`order_with_comments` in `a9.py` returns the three cards in file order), so this is
a missing affordance, not a wrong output. Echoing `origin_fingerprint` + `origin_path`
per card in the JSON envelope closes it, and would fall out of the CRITICAL-1 fix.

**Verdict: CONFIRMED.**

---

## IMPORTANT — piping batch output into anything that closes early panics (exit 101) after emitting a partial bundle

`crates/mk-cli/src/cmd/encode.rs:188-201` (`println!` inside the emit loop)

**Failure scenario:** `mk encode --keys keys.txt … | head`, `| less` with an early
`q`, or any downstream consumer that stops reading. Rust ignores SIGPIPE, so
`println!` returns `Err` and panics. The operator sees a Rust panic and a truncated
card list. A 1000-record batch emits ~3000 lines, so paging it is the natural thing
to do.

**Evidence:**

```
$ mk encode --keys big1000.txt --policy-id-stub 5b48af35 2>err.txt | head -3 >/dev/null
pipestatus=101 0
$ cat err.txt
thread 'main' panicked at library/std/src/io/stdio.rs:1123:9:
failed printing to stdout: Broken pipe (os error 32)
```

Pre-existing class (the single-card path panics on `| head -1` too), multiplied by
N-card output. Fails loudly, so it cannot silently produce a short bundle — but per
the rubric a panic is Important.

**Verdict: CONFIRMED.**

---

## MINOR — a trailing slash bypasses the explicit "a key card must declare a path" guard

`crates/mk-cli/src/keyfile.rs:48-56`

`[73c5da0a]<xpub>` is refused ("origin … has no derivation path; a key card must
declare one"), because `origin.split_once('/')` fails. `[73c5da0a/]<xpub>` splits
into `("73c5da0a", "")`, becomes `m/`, parses as the **empty** path, and — for a
depth-0 master xpub — mints a card with `origin_path: ""`. That is the
"defaulted to `m`" outcome the comment says is refused, reached by one keystroke.

```
master_empty_path  ACCEPT cards=1
$ mk decode --json <card>
{"origin_fingerprint":"73c5da0a","origin_path":"","policy_id_stubs":["5b48af35"],
 "xpub":"xpub661MyMwAqRbcFtXgS5sYJ…"}
master_bare_fp     REFUSE(64)  "origin \"73c5da0a\" has no derivation path…"
```

Minor rather than Important because the encoder's depth guard backstops every other
xpub: `[fp/]` with a depth-4 key is refused with `xpub origin-path mismatch`, and
a depth-0 card is a shape mk-codec deliberately supports (WIF / no-path, since
mk-codec 0.4.0). **Verdict: CONFIRMED.**

---

## MINOR — the declared origin fingerprint is never checked against the xpub, even at depth 0 and 1 where it is provable

`crates/mk-cli/src/keyfile.rs:73-75` (fingerprint and xpub are parsed independently
and never compared)

At depth 0 the origin fingerprint *is* the key's own fingerprint; at depth 1 it *is*
`xpub.parent_fingerprint`. Both are one hash away and neither is checked:

```
d0_fp_contradiction  ACCEPT   [73c5da0a/]<master whose own fp is 3442193e>
d1_fp_contradiction  ACCEPT   [73c5da0a/0']<xpub whose parent_fp is 3442193e>
d1_parent_lie        ACCEPT   [3442193e/0']<xpub whose parent_fp is deadbeef>
```

The engraved plate then names a master that provably holds no such key. Minor
because depth-0/1 cards are the rare shape here — a multisig cosigner key is depth 4,
where the master fingerprint is genuinely unverifiable from the xpub alone.
**Verdict: CONFIRMED.**

---

## MINOR — `XpubOriginPathMismatch` truncates `path_depth` to `u8`, so a deep path is refused with a self-contradictory message

`crates/mk-codec/src/error.rs:185-194` (`path_depth: u8`), populated at
`crates/mk-codec/src/bytecode/encode.rs:41` with `path_depth as u8`. The comparison
itself is `usize` and sound (`encode.rs:40`); only the message is wrong.

```
# 260-component path, depth-4 xpub:
error: xpub origin-path mismatch: xpub depth 4 / child 2' vs origin_path depth 4 / last Some(Hardened { index: 2 })
# 200001-component path, depth-4 xpub:
error: … vs origin_path depth 65 …
```

The first message asserts a mismatch between two identical values. **CONFIRMED.**

---

## MINOR — duplicate `--from-md1` is silently tolerated for a keyless card and refused with a misleading message for a keyed one

`crates/mk-cli/src/cmd/mod.rs:87-100`

A non-chunk md1 always starts its own group (`key == None`), so pasting the same
single-string card twice appends the **same stub twice**; 255 copies are accepted
and bloat the card (each stub costs 4 bytes of a chunk-limited payload). The same
mistake on a *chunked* card merges into one group and is refused as
`chunk set incomplete: got 8 chunks, expected 4` — which describes a *short* set,
not a duplicated one.

```
single_S1_twice            ACCEPT n=2 stubs=['45775d4d','45775d4d']
single_S1_thrice           ACCEPT n=3 stubs=[… ×3]
S1_and_S1chunked_sameplcy  ACCEPT n=2 stubs=['45775d4d','45775d4d']   # same policy, two forms
many_singles_255           ACCEPT n=255
A_twice                    REFUSE(2) md1 input rejected: chunk set incomplete: got 8 chunks, expected 4
S1C_twice                  REFUSE(2) chunk set incomplete: got 2 chunks, expected 1
```

**CONFIRMED.**

---

## MINOR — more than 255 stubs is refused with the wrong bound in the message

`crates/mk-codec/src/error.rs:115-116` uses one variant for both bounds, so the
`> u8::MAX` rejection at `crates/mk-codec/src/bytecode/encode.rs:26-28` prints:

```
many_singles_256  REFUSE(2) policy_id_stub_count must be >= 1
```

**CONFIRMED.**

---

## MINOR — a Windows-authored key file (BOM) and a CR-only file are refused with messages that never name the real cause

`crates/mk-cli/src/keyfile.rs:38, 100-106`

`str::trim` does not strip U+FEFF and `str::lines` does not split on lone `\r`:

```
bom       REFUSE(64) …:1: expected BIP-380 origin notation `[fingerprint/path]xpub`,
                          got "\u{feff}[73c5da0a/48'/0'/0'/2']xpub6DkFA…"
cr_only   REFUSE(64) …:1: key "xpub6DkFA…\r[73c5da0a/…]xpub6Dzhy…\r[…]" carries a
                          derivation suffix; `--keys` records hold an ORIGIN and a bare xpub
```

Both are safe refusals, so this is only a message-quality finding — but the CR-only
one blames the operator for a "derivation suffix" they did not write, and quotes the
whole file back at them on line 1. CRLF itself is handled correctly.
**CONFIRMED.**

---

## NIT — the SLIP-0132 normalization note names `--xpub` and no line number under `--keys`

`crates/mk-cli/src/cmd/mod.rs:205-220`

```
$ mk encode --keys <file with a ypub record at 49'/0'/0'/2'> …
note: --xpub was a SLIP-0132 ypub (BIP-49 P2SH-P2WPKH); normalized to canonical xpub — …
```

There is no `--xpub` in this invocation, and in a 50-record file nothing says which
record was silently rewritten before engraving. The *refusal* path does carry the
line number (`…:1: SLIP-0132/origin-path mismatch`); the accept-and-rewrite path
does not. **CONFIRMED.**

---

## NIT — stub order is argument order, so a re-mint with the chunk sets swapped is a different card that `mk verify` will reject

`crates/mk-cli/src/cmd/mod.rs:87-100` preserves first-appearance order, and
`mk verify --policy-id-stub/--from-md1` is documented "order-sensitive".

```
A_then_B  ACCEPT stubs=['abdea2f2','41671d95']
B_then_A  ACCEPT stubs=['41671d95','abdea2f2']     # different card, both "correct"
```

Re-cutting a lost plate therefore requires reproducing the original argument order.
Documented behaviour, recorded so it is a choice rather than a surprise.
**CONFIRMED.**

---

## VERDICT: 2 Critical, 3 Important, 6 Minor, 2 Nit

---

## Attacks that FAILED to break it — do not repeat these

Structure: multiple `]`, multiple/nested `[`, `[` in the key position, a trailing `]`
after the xpub, two records on one line, `[]`, a bare `[`, whitespace before the
xpub / inside the origin / inside the path — **all refused**.

Comment stripping (`split('#').next()`) could not be turned into data loss: `#` in a
path, `#` between origin and key, `#` inside an xpub, `#`-only lines, trailing
`# comment` — all either refused or correctly ignored (base58 and BIP-380 both
exclude `#`, and truncating an xpub breaks its checksum).

Encoding and line endings: CRLF throughout, CRLF on one line only, no trailing
newline, tabs and spaces around records, leading blank lines, NUL bytes, invalid
UTF-8, mid-file BOM, VT/FF/U+2028/NEL as separators — accepted correctly where they
should be, refused everywhere else.

Unicode: Cyrillic homoglyph in an xpub, zero-width space in an xpub, RTL mark at
line start, uppercase xpub, truncated xpub, xpub with one extra char — all refused.
Uppercase fingerprint hex is accepted and correct; mixed `'`/`h` markers are accepted
and correct; uppercase `H` markers are refused.

Path and key abuse: `m/` inside the brackets, a leading `m` as the fingerprint,
double slash, trailing slash after a path, index 2^31 / 2^32 / 10^20, negative
index, `0x`-prefixed fingerprint, short/long/odd-length fingerprint, an **xprv** in
the key position (refused: `unknown version magic bytes`), a tpub at a mismatched
depth — all refused. Depth/child contradictions between the declared path and the
xpub are refused by the encoder's `XpubOriginPathMismatch` guard.

Resources: 5000 records → 5000 cards in 0.32 s; a 200001-component path refused in
0.01 s; a 1000-line duplicate file → 1000 cards. No overflow, truncation or
timeout observed. `--keys -` reads stdin once and does not collide with anything
else in `encode`; `--from-md1 -` is treated as a literal string and refused;
empty stdin, `/dev/null`, a directory and a missing path all refuse cleanly. Error
line numbers are accurate with comments and blank lines interleaved.

Chunk grouping — the `reassemble` gate held against everything thrown at it:
a missing chunk, a missing middle chunk, a duplicated chunk, a whole set passed
twice, a set plus one chunk of another set, a single chunk of a 4-chunk set, a
one-character typo in the header or the payload, an empty-string value, `-` as a
value — **all refused**. Chunks in reversed order, one chunk uppercased, all chunks
uppercased, chunks with display separators (`md`'s default grouped output), chunks
with trailing spaces, and two sets interleaved — all **accepted with the correct
stubs**, grouped by chunk-set id as documented. A 1-chunk set works. `MAX_CHUNKS`
(32) is guarded by a real capacity check (`string_layer/chunk.rs:147-152`), not the
`debug_assert`, so >32 chunks cannot be minted.

**Not reached, left for a future reviewer:** two *different* wallets sharing a
20-bit chunk-set id. `md encode` has no `--chunk-set-id` override, so forcing one
needs either a birthday hunt over ~2000 encodes or hand-forged BCH. Analytically the
merged group refuses — the measured `A_twice` (8 chunks, `expected 4`) and
`A_plus_one_B_chunk` (5 chunks, `expected 4`) cases cover the two shapes an
operator can actually produce, and a same-index substitution would still have to
defeat the 4-byte cross-chunk content hash. Also not reached: an md1 chunk set with
more than 32 chunks (would need ~20 distinct depth-3/4 cosigner xpubs; the repo
fixtures are mostly depth-0 masters and `md` refuses those for key slots).
