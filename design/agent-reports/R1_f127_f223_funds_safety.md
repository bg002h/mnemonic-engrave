# R1 — F-127 / F-223 funds-safety review (independent, adversarial)

- **Repo / range:** `/scratch/code/shibboleth/mnemonic-key`, branch `main`, `0feaaaa..main` (5 commits), `vendor/` excluded.
- **Tree state:** HEAD `f887d57f7c42bbc5d7d81c000092c41142d40441`. Working tree clean except one untracked doc (`design/SPEC_chunk_set_id_verification.md`).
- **Binary under test:** `target/release/mk`, **rebuilt from HEAD at the start of this review** (the pre-existing binary predated commit `d7d192e` by ~1 min; rebuilt to remove the doubt). `md` = `descriptor-mnemonic/target/release/md` 0.13.0.
- **Lens:** ONE question only — can this diff cause a wrong `policy_id_stub`, a card bound to the wrong wallet, or a key paired with an origin it was not derived at, under any input a real operator could supply?
- **Method:** source read of `cmd/mod.rs`, `cmd/encode.rs`, `cmd/verify.rs`, `keyfile.rs`, `cmd/gui_schema.rs` and the vendored `md-codec` 0.42.0 (`chunk.rs`, `codex32.rs`, `decode.rs`, `bch.rs`), then **48 executed CLI cases** against live fixtures, including **wire-header forgery** (a Python port of md-codec's BCH so chunk headers could be rewritten and re-checksummed).

---

## ANSWER TO THE ONE QUESTION: **No.**

Every attack path I could construct — including ones no operator could reach without deliberately forging codex32 checksums — was refused with a loud, specific error and a non-zero exit. I found **no** input that produces a wrong stub, a wrong number of stubs, a silently dropped stub, or a card bound to a wallet other than the one supplied.

Two properties carry the weight, and both were verified by execution rather than by reading:

1. **Grouping cannot mix wallets, because it keys on a value derived from wallet content.** `chunk_set_id` is not random framing — `md_codec::chunk::derive_chunk_set_id` is the top 20 bits of the `Md1EncodingId`, so two chunk sets share an id **only** if their wallets collide on a 20-bit content hash. Verified against the fixtures: wallet A's `md1-encoding-id` is `cd71fa4b616b16619fd043a9526f5ee7` and its wire `chunk_set_id` is `0xcd71f`; wallet B's are `a0368aa3…` and `0xa0368`.

2. **The cross-chunk content-id oracle in `reassemble` is load-bearing, not decorative — and it is the last line, not the only one.** I forged wallet A's four chunks to carry wallet B's `chunk_set_id` (rewriting header bits and recomputing the BCH checksum), which is strictly stronger than any 20-bit collision an operator could stumble into. A **mixed set drawn from two different wallets, complete, index-consistent and gap-free, still decoded to a structurally valid descriptor** — and was caught anyway, by the content-id check alone:

```
F2 MIXED SET: A0',A1' (B csid) + B2,B3 | rc=2 |
  error: md1 input rejected: chunk-set-id mismatch: expected 0xa0368, derived 0x1457f
F3 MIXED SET: B0,B1 + A2',A3' (B csid) | rc=2 |
  error: md1 input rejected: chunk-set-id mismatch: expected 0xa0368, derived 0x9baf
F1 all 4 A-chunks relabelled to B csid | rc=2 |
  error: md1 input rejected: chunk-set-id mismatch: expected 0xa0368, derived 0xcd71f
```

That the mixed sets *decoded* is the important detail: `decode_payload` succeeded on the spliced bytes (a derived csid was computed, which it could not be otherwise), so the structural checks above it — count, index-completeness, header consistency — did **not** save it. Only the content-id oracle did. It fires exactly.

---

## Findings

### ZERO Critical
### ZERO Important

Below are Minor/Nit items. **None of them can produce a wrong stub or a wrong binding**; they are diagnostics, doc-accuracy and consistency observations. They do not gate.

---

**Minor — a batch encode failure names no record; the operator must bisect the key file**
`crates/mk-cli/src/cmd/encode.rs:172-180` (the mint loop) vs `crates/mk-cli/src/keyfile.rs:105-113`

`read_key_records` deliberately prefixes parse errors with `source:line:` ("A key list is edited by hand and a rejected record is the common case, so 'which line' is the whole value of the message" — `keyfile.rs:99-102`). But the mk-codec **encode-time** invariants fire later, inside the mint loop, where that context has been dropped.

*Failure scenario:* an 11-cosigner key file, record 3 has a path one component short. The operator gets an error naming neither the file nor the line, on 11 records that share a fingerprint and differ only in the account level.

*Evidence:*
```
$ printf "[73c5da0a/48'/0'/0'/2']X0\n[73c5da0a/48'/0'/1'/2']X1\n[73c5da0a/48'/0'/2']X2\n[73c5da0a/48'/0'/2'/2']X2\n" > kbad.txt
$ mk encode --keys kbad.txt --policy-id-stub 5b48af35 --group-size 0
error: xpub origin-path mismatch: xpub depth 4 / child 2' vs origin_path depth 3 / last Some(Hardened { index: 2 })
rc=2  stdout mk1 lines: 0
```
Compare a *parse* error on the same file, which is fully attributed:
```
error: kf.txt:1: --origin-path: invalid derivation path "m//48'/0'/0'/2'": invalid child number format
```
*Mitigation already present:* the batch is **atomic** — `stdout mk1 lines: 0`. Nothing is emitted, so no partial batch can be engraved. Verified for both a bad record (above) and a bad `--from-md1` alongside a good key file.

*Verdict:* **CONFIRMED** (diagnostic quality only). Introduced by this diff, since the mint loop is what put N records behind a single error.

---

**Minor — non-JSON `--keys` output carries no per-card identity; cards are positional only**
`crates/mk-cli/src/cmd/encode.rs:186-201`

Cards are separated by a single blank line and nothing else. `keyfile.rs:14-20` justifies the record format by citing a real incident — "an ordering assumption captioned 30 plates with the wrong cosigner" — and the batch output reintroduces exactly that positional mapping on the *output* side, at exactly the scale (11 cards × 2–3 chunks = ~30 lines) where it went wrong before.

*Evidence* (`cat -A`, 3 cards, chunk counts 2/3/3):
```
mk1qpd8cwpqqsq…$
mk1qpd8cwpp806…$
$
mk1qp4dj9zqqsq…$
mk1qp4dj9zp68w…$
mk1qp4dj9zzv30…$
$
mk1qp8lruzqqsq…$
…
```
*Mitigation:* every mk1 string self-describes; `mk decode` recovers fingerprint + path, so a mis-captioned plate is detectable and the card itself is never wrong. Blank-line framing is unambiguous (no card contains a blank line), and `--json` is fully structured with a `cards` array.
*Verdict:* **CONFIRMED** (operational hazard downstream of `mk`, not a defect in the card).

---

**Minor — a crossed record is accepted: the encoder invariant checks depth + last child only**
`crates/mk-cli/src/keyfile.rs:56-62`, enforced downstream by `mk-codec` `XpubOriginPathMismatch`

*Failure scenario:* a record pairs the xpub derived at `48'/0'/0'/2'` with the origin `48'/0'/1'/2'`. Same depth, same terminal child — the invariant passes and the card is minted declaring an origin the key was not derived at.

*Evidence:*
```
$ printf "[73c5da0a/48'/0'/1'/2']xpub6DkFAXWQ2dHxq…r6KFrf\n" > kf2.txt
$ mk encode --keys kf2.txt --policy-id-stub 5b48af35 --group-size 0 | mk decode -
xpub:                xpub6DkFAXWQ2dHxq2vatrt9qyA3bXYU4ToWQwCHbf5XB2mSTexcHZCeKS1VZYcPoBd5X8yVcbXFHJR9R8UCVpt82VX1VhR28mCyxUFL4r6KFrf
origin_fingerprint:  73c5da0a
origin_path:         48'/0'/1'/2'      <-- key was derived at 48'/0'/0'/2'
```
Two records with their xpubs exchanged are likewise both accepted (rc=0, two cards each naming the wrong key). The check does catch the adjacent classes:
```
wrong depth      -> error: xpub origin-path mismatch: xpub depth 4 / child 2' vs origin_path depth 3 / last Some(Hardened { index: 0 })
wrong last child -> error: xpub origin-path mismatch: xpub depth 4 / child 2' vs origin_path depth 4 / last Some(Hardened { index: 1 })
empty path `[fp/]` -> error: xpub origin-path mismatch: … vs origin_path depth 0 / last None
```
*Why this is not Important:* **not a regression.** The single-card route accepts the identical crossing (`--xpub X --origin-path <wrong>`), and this diff neither weakens nor strengthens the invariant. What is inaccurate is the module doc at `keyfile.rs:18-20` — "One record carries its fingerprint, path and key together **and cannot come apart**." That is true against *tool-side* desync (the thing three parallel repeatable flags would have caused, and the stated reason for the format) and false against *authoring-side* crossing. Worth one sentence of narrowing in the doc.
*Verdict:* **CONFIRMED** (pre-existing acceptance; doc over-claim).

---

**Minor — `--from-md1` rejects comma-grouped md1 while `mk`'s own mk1 inputs strip commas**
`crates/mk-cli/src/cmd/mod.rs:66-72` (via `md_codec::codex32::unwrap_string`) vs `crates/mk-cli/src/cmd/mod.rs:154-182` (`read_mk1_strings` → `strip_display_separators`)

`unwrap_string` tolerates whitespace and `-` but not `,`. `md encode --separator comma` is an offered rendering whose output cannot be pasted back into `--from-md1`, while `mk verify`'s *positional* mk1 arguments accept commas. Asymmetric, but the refusal is loud and specific.

*Evidence:*
```
S1 space-grouped (md's default render) | rc=0 | policy_id_stubs: 2038eff1
S2 hyphen-grouped                      | rc=0 | policy_id_stubs: 2038eff1
S3 comma-grouped                       | rc=2 | error: md1 input rejected: codex32 decode error: character ',' not in codex32 alphabet
S4 MIXED: 2 grouped + 2 raw            | rc=0 | policy_id_stubs: 2038eff1
S5 uppercase                           | rc=0 | policy_id_stubs: 2038eff1
```
*Verdict:* **CONFIRMED** (UX inconsistency; fail-closed).

---

**Minor — duplicate-policy handling is form-dependent**
`crates/mk-cli/src/cmd/mod.rs:87-100`

Pasting the *same* keyless single-string md1 twice silently mints a card with two identical stubs; pasting the same *chunked* set twice is a hard error. Same operator mistake, two behaviours, decided by whether the policy happened to exceed 80 symbols.

*Evidence:*
```
T10 T twice              | rc=0 | policy_id_stubs: aad0e0e0, aad0e0e0
T7  A with A0 duplicated | rc=2 | error: md1 input rejected: chunk set incomplete: got 5 chunks, expected 4
```
*Why this is not Important:* the duplicated-stub card is self-consistent and visible in `mk decode`; the direction that would matter — two distinct wallets silently collapsing to one stub — is **impossible**, see the "stub count" section below. Every keyed wallet policy is chunked (246 symbols vs an 80-symbol cap), so the silent branch is reachable only for keyless templates. The `None`-never-merges rule at `mod.rs:93-96` is deliberate and correct.
*Verdict:* **CONFIRMED** (cosmetic asymmetry).

---

**Minor — `verify --from-md1` is order-sensitive across chunk sets, so a correct card can be reported as failing**
`crates/mk-cli/src/cmd/verify.rs:106-119`

Group order is first-appearance order. If the operator hands `verify` the same chunk sets in a different interleaving than `encode` received, a genuine card mismatches.

*Evidence* (card bound to A then B):
```
V1 same order as encode          -> OK
V2 interleaved, A first          -> OK
V4 A chunks shuffled WITHIN sets  -> OK        (reassemble sorts by index)
V3 interleaved, B first          -> error: verify mismatch on policy_id_stubs:
                                     expected d2f4b072,2038eff1, got 2038eff1,d2f4b072
```
*Mitigation:* documented (`--policy-id-stub` / `--from-md1` help both say "order-sensitive"), pre-existing for unchunked input, and the error prints **both vectors**, so a reordering is visually obvious rather than mysterious. Within-set shuffling is immune.
*Verdict:* **CONFIRMED** (documented behaviour; noted because a false alarm at recovery is an operational cost).

---

**Nit — batch `--json` card entries omit `schema_version`, contradicting two docs**
`crates/mk-cli/src/cmd/encode.rs:230-255`

`emit_json_batch`'s doc says entries are "exactly the object single-card `--json` emits. Additive, so a consumer of the single form can read a batch entry without changes," and the CHANGELOG repeats it. `card_json` omits `schema_version`; `emit_json` includes it.

*Evidence:*
```
single: {"chunk_count":2,"code_variant":"long","mk1_strings":[…],"schema_version":1}
batch : {"card_count":1,"cards":[{"chunk_count":2,"code_variant":"long","mk1_strings":[…]}],"schema_version":1}
```
The `mk1_strings` are byte-identical between the two forms; only the version key differs, and it is present at the envelope level. Fix the sentence or add the key.
*Verdict:* **CONFIRMED** (doc/code mismatch, no functional impact).

---

**Nit — `--keys` errors and notes still say `--xpub` / `--origin-path`**
`crates/mk-cli/src/cmd/mod.rs:205-220`, `crates/mk-cli/src/slip132.rs`

```
--- kz2 rc=64
note: --xpub was a SLIP-0132 Zpub (BIP-48 P2WSH multisig); normalized to canonical xpub …
error: kz2.txt:1: SLIP-0132/origin-path mismatch — --xpub is a Zpub … but --origin-path is 48'/0'/0'/1'.
```
The `kz2.txt:1:` prefix makes it traceable, so this is cosmetic.
*Verdict:* **CONFIRMED** (wording).

---

## Attack-by-attack results against the brief's list

### Grouping — `group_md1_cards` (`cmd/mod.rs:87-100`)

Read of the match arm confirms three structural properties, each then executed:
- a `None` key short-circuits `and_then` and **always** starts a new group, so a non-chunk can never be appended to a chunk group and two non-chunks never merge;
- a `Some(k)` key only ever joins a group whose stored key is `Some(k)`, so a chunk can never join a group started by a non-chunk;
- groups are a `Vec` pushed in encounter order, so first-appearance ordering is exact.

| # | Input | Result |
|---|---|---|
| T1 | A's 4 chunks in order | `2038eff1` ✓ |
| T2 | A's 4 chunks reversed | `2038eff1` ✓ (index-sorted) |
| T3 | A then B (8 values) | `2038eff1, d2f4b072` ✓ |
| T4 | A/B interleaved, A first | `2038eff1, d2f4b072` ✓ |
| T5 | A/B interleaved, B first | `d2f4b072, 2038eff1` ✓ (operator's order) |
| T6 | A missing chunk 3 | `chunk set incomplete: got 3, expected 4` |
| T7 | A with chunk 0 duplicated | `chunk set incomplete: got 5, expected 4` |
| T8/T9 | A + keyless template T, both orders | `2038eff1, aad0e0e0` / `aad0e0e0, 2038eff1` ✓ |
| T11 | A0,A1,A2 + B3 | `chunk set incomplete: got 3, expected 4` |
| T12 | lone chunk A0 | `chunk set incomplete: got 1, expected 4` |
| T13 | A complete + B short one | `chunk set incomplete: got 3, expected 4` |
| S6 | one chunk with a 1-char typo | `chunk set incomplete: got 3, expected 4` (BCH rejects it into its own group) |

Ground truth from `md inspect`: A `wallet-policy-id 2038eff1905350d5…` (wallet-policy-mode **true**), B `d2f4b072325fac21…`, T `wallet-descriptor-template-id aad0e0e0718cbe91…` (wallet-policy-mode **false**). All observed stubs are the top 4 bytes of the correct form-appropriate identity — form dispatch is right in every case.

**Cross-wallet 20-bit collision:** requires a content-hash collision to even reach `reassemble`, and then a merged group necessarily has `2×count` chunks → `ChunkSetIncomplete`, or mismatched counts → `ChunkSetInconsistent`. A silent survivor needs the collision **plus** a spliced subset totalling exactly `count` with indices `0..count-1` **plus** a second independent 20-bit condition (the content-id oracle). Forged directly above (F1–F4) and refused every time. An adversary able to forge codex32 checksums could simply hand over a valid md1 for the wrong wallet, which is strictly easier and predates this diff. **No new attack surface.**

### `md1_chunk_set_id` returns `Option` (`cmd/mod.rs:66-72`)

`Some` ⟺ `unwrap_string` succeeds **and** version == `WF_REDESIGN_VERSION` (4) **and** the chunked flag is set — which is the definition of a chunk. `decode_md1_string` (`vendor/md-codec/src/decode.rs:188-196`) dispatches on the chunked flag *alone*, so the two disagree only for a chunked-flag-set string with a wrong version. I forged that case and the neighbouring ones:

| Forged header mutation | Result |
|---|---|
| chunked flag cleared (routes to the single-payload path) | `wire-format version mismatch: got 8, expected 4` |
| version → 5 / 8 / 0, flag still set | `wire-format version mismatch: got 5 / 8 / 0, expected 4` |
| count-1 → 0 (claims a 1-chunk set) | `TLV length 1042 exceeds remaining bits 128` |

A real chunk returns `None` only when it fails BCH, is mixed-case, or has a non-`md1` HRP — and in every such case the string is passed through to `decode_md1_string`, which fails on the same condition and surfaces the codec's own message. The doc comment's claim that a malformed string is "passed through to the codec so the real error text reaches the operator" is accurate:

```
X1 mk1 string given to --from-md1 | error: … string does not start with HRP md1
X2 garbage / X3 empty string      | error: … string does not start with HRP md1
X4 mixed case                     | error: … string mixes upper and lower case (BIP-173 forbids mixed case)
X5 chunk truncated by 1 char      | error: … BCH checksum verification failed
```
No BCH **error correction** is invoked on this path (`decode_with_correction` is never called), so there is no route by which a typo is silently "repaired" into a different wallet.

### Silent stub loss or duplication

No mechanism exists for loss. Merging happens only on an equal `chunk_set_id`; equal ids from distinct wallets require a content collision and then fail closed. Duplication is possible only by supplying the same policy twice, is form-dependent (Minor above), always yields the *correct* stub value, and is visible in `mk decode`. Stub order is first-appearance and identical in `encode` and `verify` — same function, same call site shape.

### `--keys` record integrity

Explicit `--policy-id-stub`/`--from-md1` values are resolved into `stubs` **once, before** any record is read, and every card gets `stubs.clone()` — so no record can influence another's binding. Verified end to end: a 2-record file plus wallet A's 4 chunks produced two cards, each with its own origin, both carrying `2038eff1`.

| Case | Result |
|---|---|
| 3 correct records | rc=0, 8 mk1 strings, order matches file order |
| CRLF line endings | rc=0 (Rust `str::lines()` strips `\r`) |
| `#` comments, blank lines, inline `# alice`, leading/trailing whitespace | all handled |
| empty file / comments only | `--keys …: no key records found` |
| `[fp]xpub` (no path) | `origin … has no derivation path; a key card must declare one` |
| `[fp/]xpub` | caught downstream: `origin_path depth 0 / last None` |
| `[fp//48'…]`, `[fp/48'…/]` | `kf.txt:1: --origin-path: invalid derivation path` |
| `…]xpub…/0/*` (use-site suffix) | `key … carries a derivation suffix` |
| `[fp/path]` with no key | `record … declares an origin but no xpub` |
| stray/missing bracket | attributed usage errors |
| tpub on a mainnet path | `kf.txt:1: invalid xpub …: base58 encoding error` |
| uppercase fingerprint | accepted (hex is case-insensitive) — correct |
| `--keys -` (stdin) | works |
| **atomicity**: bad record 3 of 4, or bad `--from-md1` | rc=2, **0 mk1 lines on stdout** |

**Mutual exclusion** — all five guards fire, with the right message:
```
--keys + --xpub                -> error: --keys and --xpub are mutually exclusive; each --keys record carries its own origin
--keys + --origin-path         -> (same)
--keys + --origin-fingerprint  -> (same)
--keys + --chunk-set-id        -> (same)
--keys + --privacy-preserving  -> error: … a --keys record always declares a fingerprint, and dropping it silently
                                  is how a card gets engraved wrong -- mint privacy-preserving cards one at a time
```

### `parse_xpub_normalized(x, Some(&path))` — is it path-dependent in a way that could alter the key?

**No. It is a gate, never a transform.** `slip132.rs:1-6` states it is a decode-swap-reencode of the 4 version bytes at the base58check layer, and the `origin_path` argument reaches only `Slip132Variant::path_matches`, whose sole effect is `Err(mismatch_help)`. Executed:

```
kz1  Zpub + matching path 48'/0'/0'/2'  -> rc=0, mk1 output md5 98fcb4e2b245b2ec12e14b63afeff262
kz3  plain xpub + same path             -> rc=0, mk1 output md5 98fcb4e2b245b2ec12e14b63afeff262   IDENTICAL
kz2  Zpub + path 48'/0'/0'/1'           -> rc=64, refused (Zpub implies …/2')
```
Same bytes out for the SLIP-0132 and canonical forms; a contradicting path refuses rather than rewrites.

### `verify` vs `encode` asymmetry

They call the identical pair of functions with the identical shape (`encode.rs:126-131`, `verify.rs:106-111`), and both push explicit `--policy-id-stub` values before md1-derived ones. Round-trip confirmed: V1/V2/V4 OK, V3 and V5 fail loudly and correctly. The only asymmetry in the file is the pre-existing one that `verify`'s **positional mk1** arguments go through `strip_display_separators` while `--from-md1` does not — noted as Minor above, fail-closed.

### `gui_schema.rs` (changed, outside the focus list — checked for funds relevance)

`REQUIRED_IN_GUI_FORM` compensates for `required_unless_present("keys")` making `Arg::is_required_set()` report false for `--xpub` and `--origin-path`. Without it the emitted schema would have flipped both to **optional** and silently desynced `mnemonic-gui`'s hand-written mirror, which has no automated gate. That is a correct, load-bearing compensation and I flag it only as a thing worth *keeping* — a future flag with `required_unless_present` will hit the same trap and the constant is easy to forget.

---

## Coverage and limits of this review

- Everything above was **executed**; no finding rests on reading alone. The forged-header cases (F1–F4, CLEARFLAG/VER/COUNT) required a Python port of `md-codec`'s BCH `polymod` (`GEN_REGULAR`, `MD_REGULAR_CONST`, init `0x23181b3`), validated first by round-tripping the genuine headers: A parsed as `(v4, chunked, csid 841503 = 0xcd71f, count 4, index 0..3)`, matching `md`'s own reported `chunk-set-id: 0xcd71f`.
- Scratch work is under `/tmp/claude-1000/r1/` (`t1.sh`–`t9.sh`, `forge.py`). **No tracked file in either repo was modified.** The only write outside scratch is this report.
- Not covered, per brief: the journey/transcript work; the full test suite (settled); the cross-language goldens (settled); `vendor/` as a tree; `md-codec`'s internal correctness beyond the chunk/codex32/decode paths this diff depends on.
- One thing I chose **not** to spend a round on: whether `mk gui-schema` output is byte-identical to the previous tree (a CHANGELOG claim). It is not funds-relevant and the `CLI_ONLY_FLAGS`/`REQUIRED_IN_GUI_FORM` mechanism is verifiable by reading.

## VERDICT: 0 Critical, 0 Important, 6 Minor, 2 Nit
