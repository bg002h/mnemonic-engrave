# R7 — Independent adversarial review of the FOLD (R1..R6 response)

**Scope, strictly.** The edits made in response to `design/agent-reports/R1..R6` (`89470c9`)
and nothing else: `mnemonic-key` `f887d57..main` (`47d7f97`, `6350292`, `9031cda`,
`9b30566`; `vendor/` excluded) and `mnemonic-engrave` `89470c9..ab8e9f4` (`e3da435`,
`b10e5cc`, `ab8e9f4`). Two questions only: did each finding get fixed, and did the fold
introduce a new defect. I did not author any of this and did not re-audit the cycle.

**Binary provenance — read this before reproducing.** The brief's `mk` path
(`/scratch/code/shibboleth/mnemonic-key/target/release/mk`) has an mtime *older* than
several HEAD source files, so I did not trust it. I copied the repo to
`/tmp/claude-1000/r7/mk-src` (tracked files untouched) and built
`cargo build --release --offline --bin mk` there; **every result below is from that
binary**. I later established by behaviour probe (verify multiset + depth-1 fingerprint
refusal, both `9b30566`-only) that the in-repo binary *is* at HEAD after all — the mtime
skew is because it was built from the working tree minutes before the commit. Both
binaries agree on every probe I ran on both.

Mutations were done only in `/tmp/claude-1000/r7/mk-src` and reverted. Final state:
`mnemonic-key` `?? design/SPEC_chunk_set_id_verification.md` and nothing else;
`mnemonic-engrave` completely clean. The pathological transcript was run against a
**copy** of `design/journeys/` under `/tmp`, so the repo's `out/` was never overwritten.

---

## Section A — fold vs. findings

| Finding (report/id) | Fixed? | Evidence |
|---|---|---|
| **R2/C1** — card minted for a non-cosigner of a KEYED policy | **YES** | Genuine stranger (`key-02.xpub`, verified by 65-byte identity **not** in policy A's `{@0,@1}`) → `rc=64`, "is not a cosigner of the wallet policy with stub 38bd7cec". Real member → mints. Keyless template → mints. `--policy-id-stub` only → mints. All four arms run. |
| **R2/C2** — write-only card, `>MAX_PATH_COMPONENTS` encodes but never decodes | **YES** | CLI: n=10 → falls through to the depth guard; n=11/20/255 → `error: path too deep: N components (max 10)`, rc=2. Boundary is `>` not `>=` (`accepts_path_at_exactly_max_components`). Cap runs **before** the depth/child guard, so the operator gets the actionable error. Cannot over-refuse a previously-valid card: `decode_explicit_path` always refused `>10`, and every `STANDARD_PATHS` entry is ≤4 components so the table path is unreachable from the cap. |
| **R5/C** — `json_batch_wraps_the_single_card_object` checked only `cards[0]` | **YES** | Settled by the brief (mutant re-run and killed); test now iterates every index and asserts each card names its record. |
| **R5/I** — `verify_from_keyless_..._stub` never compared to the golden | **YES** | Settled by the brief; anchored to `EXPECTED_TEMPLATE_STUB`. |
| **R2/I** — no cross-record validation; N cards for N+1 cosigners, silent | **YES** (by design) | `note: policy 38bd7cec has 2 cosigner(s); 1 card(s) minted here, 1 not carded`, rc=0. Keyless template prints nothing. Both arms reproduced. (Count wording defect → **B6**.) |
| **R2/I** — batch output identifies no card | **YES** for `--json` | `origin_fingerprint` + `origin_path` on every card, single and batch. Non-JSON form still positional — that is R1's separate Minor, recorded open. |
| **R2/I** — SIGPIPE panic mid-bundle | **YES** | 500-record file `| head`: exit **141**, **0** stderr bytes, **0** panics. Windows gate is inside the fn (`#[cfg(unix)]`), `libc` is an unconditional dep that builds on Windows; CI green (settled). |
| **R3/I** — attribution has no content-based check | **YES** | `build_card_index.py` attacked six ways: permuted `cards[1]/[2]` → FATAL naming both origins; missing card → `FATAL: 10 cards for 11 key records`; wrong origin → FATAL; `null` fingerprint → FATAL; extra card → FATAL; control → 30 rows. Sabotage probe: forced failure inside the script ⇒ `transcript_pathological.sh` exits **1** with `FATAL: card-index.txt was not written`. Clean run of the transcript: rc=0, zero FATAL. |
| **R6/F1** — BIP contradicts itself on the form-aware rule | **PARTIAL** | Glossary (:38) and "Policy ID stubs" (:426-435) rewritten form-aware and agree with "Linkage to MD" (:584-587). But `:48` — inside the very naming note `:37` points readers to — still reads "the Policy ID **and therefore the 4-byte stub** is now the WalletPolicyId", unconditional, with no 2026-08-21 entry appended to that list. See **C5**. |
| **R6/F2** — `docs/MK_CODEC_RUST_API.md` cites a function that never existed | **YES** | `grep -rn compute_policy_id_stub descriptor-mnemonic/crates/` → **0**. Replacement cites `compute_wallet_policy_id` (`identity.rs:186`) and `compute_wallet_descriptor_template_id` (`identity.rs:71`) — both resolve; dispatch matches `cmd/mod.rs:162-166`. |
| **R6/F3** — closure-design Q-2 stale and unmarked | **YES** | Superseded-twice banner added with forward pointer. |
| **R6/F4** — `verify.rs:3` cites `SPEC §3.5.4` | **YES**, and swept | All 9 phantom cites in `crates/**/*.rs` repointed (machine-counted below). Headline count wrong → **C1**; guard has a blind spot → **B5**. |
| **R4** — "33 → 2" invocation count FALSE | **YES** | Re-measured with an instrumented shim: 34 / 35 / 3, method recorded in FOLLOWUPS. Matches R4's independent 35 and 3. |
| R1/Min — batch failure names no record | **YES** | `error: --keys record 3 ([73c5da0a/9']): xpub origin-path mismatch: …`, rc=64. |
| R1/Min — non-JSON `--keys` output has no per-card identity | **NO** | Recorded open in F-224. Deliberate. |
| R1/Min — a crossed record is accepted (depth+child only) | **PARTIAL** | Doc over-claim narrowed (`keyfile.rs`), and depth-0/1 fingerprint check added; depths ≥2 still unprovable, as the fold says. |
| R1/Min — `--from-md1` rejects comma-grouped md1 | **PARTIAL** | Fixed in `encode`; **`mk verify --from-md1` still refuses the identical string**. New asymmetry → **B3**. |
| R1/Min — duplicate-policy handling is form-dependent | **NO** | Declined with reason in F-224. |
| R1/Min — `verify --from-md1` order-sensitive ⇒ correct card reported failing | **YES** | Multiset compare; five arms verified (see B/§3 below). |
| R1/Nit — batch `--json` omits `schema_version` | **NO** | Declined with reason; envelope carries it in both forms. Doc wording → **C8**. |
| R1/Nit + R2/Nit — `--keys` messages still say `--xpub` | **PARTIAL** | The SLIP-0132 *note* was reworded. The SLIP-0132 *mismatch error* (`slip132.rs:112`) still says "`--xpub` is a … but `--origin-path` is …" under `--keys`, and the accept-and-rewrite note still carries no record/line number — R2's actual complaint. |
| R2/Min — trailing slash bypasses the "must declare a path" guard | **NO** | Recorded open in F-224. |
| R2/Min — origin fingerprint never checked at depth 0/1 | **YES** | Depth-0 and depth-1 both refuse a crossed fingerprint (BIP-32 vector-1 keys), both mint when truthful, depth-4 still mints (unprovable). Skip-on-depth-mismatch is correct. |
| R2/Min — `XpubOriginPathMismatch` truncates `path_depth` to `u8` | **PARTIAL / moved** | That truncation is now *unreachable* (the cap fires first for >10), but the identical `as u8` truncation reappears in the new `PathTooDeep` → **B1**. F-224 still lists it against the old variant → **C7**. |
| R2/Min — duplicate `--from-md1` tolerated/misleading | **NO** | Declined with reason. |
| R2/Min — `>255 stubs` refused with the wrong bound | **NO** | Recorded open. |
| R2/Min — BOM / CR-only key files | **YES** | BOM → rc=0. CR-only → rc=64 "the file uses CR-only line endings (classic Mac) …". CRLF → rc=0. |
| R2/Nit — stub order is argument order | **NO** (declined) | Documented rather than changed; `verify` made order-independent instead. |
| R3/Min — awk records every line; nothing compares the two files | **YES** (moot) | awk block deleted; JSON is the source. |
| R3/Min — `:211` prints a command that was never run, with a wrong arg count | **NO** | The fold **edited that exact line** (`--group-size 0` → `--json`) and left the defect: run output line 88 is `$ … mk encode --keys … 8 --from-md1 args --json` while there are **4** `--from-md1` flags (`grep -c '^md1' out/pathological/md1.txt` → 4), still hand-written in the format reserved for real commands. F-224 lists it neither as done, moot, declined, nor open. |
| R3/Min — both committed invocation counts wrong | **YES** | See R4 row. |
| R3/Nit — unanchored `sed 's/key-0*//'` | **NO** | Declined with reason. |
| R3/Nit — `awk -v` escape processing; R3/Nit — double FATAL | **YES** (moot) | awk gone. |

---

## Section B — new defects introduced by the fold

### B1 — Minor: the new path cap truncates its own component count to `u8`, and the comment asserting that is impossible is false

`crates/mk-codec/src/bytecode/encode.rs:48-54`

```rust
// `as u8` is safe: the depth/child guard below requires
// `xpub.depth as usize == component_count`, and `xpub.depth` is a u8,
// so a card with more than 255 components cannot reach encode at all.
return Err(Error::PathTooDeep(component_count as u8));
```

**Failure scenario.** The cap `return`s **before** the depth/child guard it cites, so the
guard never runs and the cast is reached with any count. An operator with a
pathologically deep `--origin-path` is told the path has zero components.

**Evidence** (fresh HEAD binary):

```
n=255 rc=2 : error: path too deep: 255 components (max 10)
n=256 rc=2 : error: path too deep: 0 components (max 10)
n=300 rc=2 : error: path too deep: 44 components (max 10)
n=511 rc=2 : error: path too deep: 255 components (max 10)
```

Refusal is still correct in every case — this is diagnostics only, and no card is minted.
But it is exactly the class R2 filed as a Minor against `XpubOriginPathMismatch`, moved
into the fold's own new error and shipped with a comment that says it cannot happen. The
unit test stops at `n=255`, one short of the boundary that breaks. `Error::PathTooDeep`
takes a `u8`, so a real fix means widening the variant or clamping with an explicit
"≥256" rendering.

**Verdict: CONFIRMED.**

---

### B2 — Important: cosigner membership over-refuses a legitimate card against a *partially keyed* md1 wallet policy

`crates/mk-cli/src/cmd/mod.rs:169-176` (`.filter(|v| !v.is_empty())`) with the membership
loop at `crates/mk-cli/src/cmd/encode.rs` (the `xpub_identity_65` / `cosigners.contains`
block).

The carve-out treats **only a fully keyless** template as "membership undecidable"
(`cosigners == None`). `TlvSection::pubkeys` is
`Option<Vec<(u8, [u8; 65])>>` — **sparse** — so a policy carrying keys for *some* `@N`
is `Some([...])`, `is_wallet_policy()` is true, and the partial list is treated as the
exhaustive cosigner set. Every cosigner whose key is not inlined in that md1 is then
refused.

**Failure scenario.** The primary `md` CLI produces such a card at exit 0. A cosigner
carding their own key against it is refused with a message that asserts something false
("is not a cosigner", "that policy declares 1 key(s)"), and the message names no escape.

**Evidence** (both commands run, `md` 0.42-era release binary, fresh HEAD `mk`):

```
$ md encode "wsh(multi(2,@0/<0;1>/*,@1/<0;1>/*))" --key "@0=<key-02>" \
      --fingerprint "@0=73c5da0a" --force-chunked --group-size 0
chunk-set-id: 0x6d5ca      (3 md1 chunks, rc=0)

$ md inspect <those 3 chunks>
n: 2
wallet-policy-mode: true
wallet-policy-id: 7d267ac4121423bbd53f0bb67dbfe19d

$ mk encode --xpub <key-03> --origin-fingerprint 73c5da0a \
      --origin-path "m/48'/0'/3'/2'" --from-md1 <those 3 chunks> --group-size 0
error: xpub xpub6E6Z3Ss5TXJYN… (origin 48'/0'/3'/2') is not a cosigner of the wallet
policy with stub 7d267ac4; that policy declares 1 key(s). …
rc=64

$ mk encode --xpub <key-02> … --from-md1 <those 3 chunks> --group-size 0
mk1qpl652zqqsq…   rc=0
```

Before the fold this minted. The workaround (`--policy-id-stub 7d267ac4` by hand, which
is *not* membership-checked — verified rc=0) exists but is undocumented and is exactly
the bypass C1 was added to close, so the check pushes the careful operator onto the
unchecked path.

Mitigating: the refusal is loud, no wrong card is produced, and a partially keyed policy
is a degenerate shape (`md inspect` itself prints the self-contradictory
`wallet-policy-mode: true` alongside `note: stdout is a keyless descriptor template (no
keys)`). The behaviour is demonstrated; whether "partial keying + per-key mk1 cards" is a
workflow worth supporting is the arguable half. Minimal shape of a fix: treat a pubkeys
list shorter than the template's `n` the same as `None`, or say in the message that the
policy carries keys for only *k of n* slots.

**Verdict: CONFIRMED** (behaviour), **PLAUSIBLE** (that a real operator reaches it).

---

### B3 — Minor: the display-separator fix landed on `encode` only; `verify` still refuses the same string

`crates/mk-cli/src/cmd/encode.rs` (`strip_display_separators` in the `from_md1` map) vs
`crates/mk-cli/src/cmd/verify.rs:114` (`group_md1_cards(&args.from_md1)`, unstripped).

**Failure scenario.** An operator mints a card from an md1 they pasted from
`md encode --separator comma`, then cannot check the engraved plate against that same
string. `verify` is the recovery-side tool; a refusal there is the more expensive half.

**Evidence:**

```
comma-grouped = md1yq,pqqxz,q2qwf,v8urt,848e

$ mk encode --xpub … --from-md1 "$GC" --group-size 0            rc=0
$ mk verify <that card> --from-md1 "$GC"
error: md1 input rejected: codex32 decode error: character ',' not in codex32 alphabet
rc=2
$ mk verify <that card> --from-md1 md1yqpqqxzq2qwfv8urt848e      rc=0  (control)
```

Before the fold both refused, which was at least consistent. `md_codec::codex32::unwrap_string`
tolerates whitespace and `-` but not `,` (`codex32.rs:160`), which is why the space-grouped
default was never affected either way.

**Verdict: CONFIRMED.**

---

### B4 — Minor: `mk verify --help` still advertises the semantics the fold removed

`crates/mk-cli/src/cmd/verify.rs:40` and `:44`

```rust
/// Expected `policy_id_stub` (repeatable; order-sensitive).
/// Expected `policy_id_stub` derived from md1 strings (repeatable; order-sensitive).
```

The comparison is now a multiset. The flag help — the only place most operators read the
contract — still says order-sensitive, and it is the *stale claim* half of the exact
finding the fold was closing.

**Evidence:** `mk verify --help` prints both lines verbatim; the behaviour probe in §3
below returns `rc=0` for a swapped order. `mk gui-schema` carries no help text
(`mk gui-schema | grep -c order-sensitive` → 0), so the cross-repo mirror is unaffected.

**Verdict: CONFIRMED.**

---

### B5 — Minor: the new cite guard misses the shape one of the nine real phantoms actually had, and is narrower than "every citation"

`crates/mk-cli/tests/spec_cites_resolve.rs` (patterns `["SPEC §", "SPEC_mk_v0_1.md §"]`,
sources = `crates/**/*.rs` only).

The pre-fold `crates/mk-cli/src/error.rs:3` read:

```
//! Realizes SPEC §3.5.6 (JSON error envelope) and §3.5.7 (exit-code table)
```

The second cite has no repeated `SPEC ` prefix. The guard would not have seen it.
I injected each shape into the scratch copy and ran the test:

```
CONTROL                                                     ok
A  //! Realizes SPEC §9.9.9.                                FAILED (named, correct)
B  //! Realizes SPEC §3.3 (envelope) and §9.9.9 (…).         ok   <-- MISSED
C  //! See CLAUDE.md SPEC §9.9.9 …                           ok   <-- MISSED (the
                                                                   `ends_with("md")`
                                                                   cross-doc exclusion)
D  //! Realizes SPEC §9.9.9. SPEC-CITE-EXEMPT                ok   (marker is opt-out
                                                                   by design, but
                                                                   nothing bounds it)
```

Scope: the guard walks `crates/` for `.rs` only. A **live** phantom survives outside it —
`design/FOLLOWUPS.md:151`, "the SHA-pinned v0.1 `mk-codec` test-vector corpus (SPEC
§3.5.5)". §3.5 has no subsections (SPEC headings stop at 3.7).

Everything else the fold left alone checks out: `§3.2.4` (×2, `mk-codec/src/error.rs`) is
a *plan* cite, `§5.4` is an implementation-plan cite, `§6.3` is Lin & Costello, `§1.4` and
`§5.3`/`§8.1` are cross-document. No false positives.

**Verdict: CONFIRMED.**

---

### B6 — Minor: the coverage note reports a distinct-key count as a card count

`crates/mk-cli/src/cmd/encode.rs` (the coverage block: `carded` is a
`HashSet<[u8; 65]>` and `carded.len()` is printed as "N card(s) minted here").

**Evidence** — a `--keys` file with the same record twice against keyed policy A mints
**two** cards:

```
note: policy 38bd7cec has 2 cosigner(s); 1 card(s) minted here, 1 not carded
```

The `missing` count is right; the "minted here" count is not. Operator-facing, introduced
by this fold. **Verdict: CONFIRMED.**

---

### B7 — Nit: the journey's identity join is only as strong as origin-uniqueness, which `mk` does not enforce

`design/journeys/build_card_index.py:47-56` joins on `(origin_fingerprint, origin_path)`,
which is the only identity `mk encode --json` emits. `mk encode --keys` accepts two
records with the **same** origin and **different** xpubs and mints both at exit 0 with
identical `origin_fingerprint`/`origin_path` — verified. Such a pair remains
interchangeable to the check. (`md` refuses the analogous shape outright: "@0 and @1
declare the same key origin … but different xpubs; one origin identifies exactly one
key".) I could not reach this from the journey's own fixtures (11 distinct origins), so
it is a bound on the guarantee, not a live hole. **Verdict: CONFIRMED (as a limitation).**

---

### B8 — Nit: the new cap comment was inserted between the pre-existing invariant comment and the code it documents

`crates/mk-codec/src/bytecode/encode.rs:32-56`. The block ending "A card encodes iff it
survives compact-drop + reconstruction unchanged" now sits above the *path cap*; the
depth/child guard it describes is 20 lines further down. `component_count` and
`path_depth` are also two identical `.into_iter().count()` calls back to back.

---

### B9 — Nit: two doc comments were spliced mid-sentence

`crates/mk-cli/src/cmd/inspect.rs:6-12` and `crates/mk-cli/src/cmd/vectors.rs:5-11` now
read `… See FOLLOWUPS F-224.) v0.2 inspect output is intentionally less rich …` — the
inserted parenthetical lands inside the original sentence rather than after it.

---

## Section C — false claims in the fold's own commit messages / changelog / ledger

### C1 — Minor. "EIGHT" phantom SPEC cites. Machine-counted: **NINE**.

Sites: commit `9b30566` ("FILED AS ONE, TURNED OUT TO BE EIGHT"), `crates/mk-cli/CHANGELOG.md`
("Sweeping the class found **eight**:"), `mnemonic-engrave design/FOLLOWUPS.md` F-224
("was **one of eight**"), and `tests/spec_cites_resolve.rs`'s own module doc
("sweeping the whole crate found EIGHT"). The enumeration printed in the same sentence
lists **nine** items (`§3.5.2, .3, .4, .5, .6 ×2, .7 ×2, and §1.1` = 9).

```
$ git archive f887d57 crates | tar -x -C /tmp/…/pre
$ grep -rn --include='*.rs' -E '§3\.5\.[0-9]|§1\.1' /tmp/…/pre
crates/mk-codec/tests/common/mod.rs:65   §1.1
crates/mk-cli/src/cmd/decode.rs:3        §3.5.2
crates/mk-cli/src/cmd/inspect.rs:3       §3.5.3
crates/mk-cli/src/cmd/verify.rs:3        §3.5.4
crates/mk-cli/src/cmd/vectors.rs:3       §3.5.5
crates/mk-cli/src/error.rs:3             §3.5.6  and  §3.5.7   (two on one line)
crates/mk-cli/src/error.rs:100           §3.5.7
crates/mk-cli/src/main.rs:122            §3.5.6
                                          --> 9 occurrences, all nine repointed
```

The *work* is complete — all nine are gone. Only the count is wrong, in four places, in
the very entry that exists because a count was wrong before.

### C2 — Minor (same defect as B5). "`tests/spec_cites_resolve.rs` now resolves **every** citation against the SPEC's real headings" (CHANGELOG; commit `9b30566` says "a command instead of a discipline"). It resolves every citation in `crates/**/*.rs` that carries a repeated `SPEC §` prefix. One of the nine it was written for would have slipped it (B5/mutant B), and `design/FOLLOWUPS.md:151` still carries a live `§3.5.5`.

### C3 — Nit. CHANGELOG: "`--from-md1` accepts display-grouped md1. **`md` prints grouped strings by default** … so a copy-pasted md1 was refused by the one flag that exists to consume it." `md`'s default separator is a **space**, and `unwrap_string` has always tolerated whitespace and `-` (`md-codec/src/codex32.rs:160`). Verified: the default space-grouped string round-tripped through `--from-md1` before *and* after. The refusal R1 actually reported was for `--separator comma`, which is not the default. The fix is real; the stated cause is not.

### C4 — (same defect as B1). Source comment: "`as u8` is safe … a card with more than 255 components cannot reach encode at all." Falsified above at n=256.

### C5 — Minor. Commit `47d7f97` / CHANGELOG: "**All three sections now agree**" (BIP). Three do. A fourth — `bip/bip-mnemonic-key.mediawiki:48`, inside the "Naming note" that the freshly corrected glossary line `:37` explicitly directs the reader to — still states, unconditionally and with no 2026-08-21 entry: "the Policy ID — and therefore the 4-byte stub — is now the **WalletPolicyId**". This is the same incomplete-propagation class R6/F1 raised.

### C6 — Nit. `e3da435`'s message: "committed unfolded in `45d4e5d`". `git cat-file -t 45d4e5d` → *not a valid object name*; the reports commit is `89470c9`. Disclosed and corrected in FOLLOWUPS by `b10e5cc`, but the commit message itself stands wrong.

### C7 — Nit. F-224 "Still open: … a `u8`-truncated depth in `XpubOriginPathMismatch`". That truncation is now unreachable — the new cap fires first for any path over 10, so `path_depth as u8` is always exact (verified: n=10 reports "origin_path depth 10"). The live truncation is in `PathTooDeep` (B1), which the ledger does not mention.

### C8 — Nit. `emit_json_batch` doc: "a `cards` array whose entries are **exactly** the object single-card `--json` emits." The single-card object additionally carries `schema_version` (`emit_json` sets it on the card object). The deliberate decision is defensible and is recorded; the word "exactly" is not accurate.

---

## What I attacked and could NOT break

- **Multiset stub comparison** — five arms plus one: as minted rc=0; swapped rc=0 with a correct stderr note; wrong stub rc=4; missing stub rc=4; **extra** stub rc=4; duplicate standing in for a distinct one rc=4 (both directions). It is a genuine multiset (`sort_unstable` on copies, originals kept for the message), not a set.
- **Membership check, over-refusal direction** — keyless template, `--policy-id-stub`-only, and a real cosigner all mint; a key in policy A but not policy C is refused against C and vice versa, which is what the BIP's own "a cosigner participating in multiple wallets MAY stamp multiple stubs" semantics requires. The 65-byte identity is exactly md-codec's `TlvSection::pubkeys` layout (32-byte chain code ‖ 33-byte compressed pubkey, `tlv.rs:32`), so it is immune to SLIP-132 re-serialization and to differing depth/parent/child metadata — I confirmed a depth-4 xpub and md1's depth-0 reconstruction of the same key produce identical identities. The only hole I found is the *partial* pubkeys set (B2).
- **Path cap** — boundary is `>` not `>=`; fires before the depth guard; cannot refuse anything the decoder would have accepted; standard-table paths are all ≤4 components so it cannot touch them.
- **Depth-0/1 fingerprint check** — mathematically sound (a depth-1 key's parent *is* the master by BIP-32 construction); correctly skipped when `xpub.depth != path depth`, which is the case where a non-root "master" fingerprint would otherwise be compared against the wrong key; depth ≥2 still mints. I could not make it refuse a truthful record. (`--origin-fingerprint 00000000` at depth 0/1 is now refused, but `--privacy-preserving` is the sanctioned unknown-master route and it omits the field entirely.)
- **SIGPIPE** — exit 141, zero stderr, zero panics; `--json` unaffected; no other platform assumption in the fold (`libc` unconditional and Windows-buildable, `keyfile.rs` uses only portable std, the cite guard uses path joins).
- **`--json` envelope shape** — keys are BTreeMap-sorted and were sorted before, so ordering did not change; the two new fields are purely additive; `origin_path` uses the same `m/`-less rendering as `mk decode --json` and feeds straight back into `--origin-path` (verified round-trip). `mnemonic-gui` does not parse `mk`'s JSON at all (`grep -rn mk1_strings mnemonic-gui/src/` → 0 hits), so the hand-written mirror is not exposed to this change.
- **`build_card_index.py` + `transcript_pathological.sh`** — permutation, missing card, extra card, foreign origin and null fingerprint all die loudly and name the mismatch; a forced failure inside the script propagates to transcript exit 1 (the pre-created empty `card-index.txt` makes the `-s` test fire correctly); the full transcript runs clean end to end in a fresh tree with zero FATAL and zero CAPTURE FAILED.
- **Key-file intake** — BOM stripped, CR-only refused by name, CRLF works, batch failures name the record and the batch stays atomic.

---

## VERDICT: 0 Critical, 1 Important, 7 Minor, 7 Nit

**This gate does not close.** The one Important (B2 — membership over-refuses against a
partially keyed md1 wallet policy) is a behaviour change in a funds-adjacent minting path
that refuses a card the tool previously produced, with a message that states something
false and names no escape. Everything else is diagnostics, doc propagation, or a count.

The two Criticals, all three Importants with operator consequence, and both false-passing
tests are genuinely fixed, and I could not break any of them from the directions the
brief named. The failure mode this fold shows is the one the project already has a lesson
for: **the fix lands, and the sentence describing it does not follow** — a count of eight
where the same sentence lists nine (four sites), a guard advertised as "every citation"
that misses one of the nine shapes it was built for, a `verify --help` still promising the
semantics that were just removed, a BIP section corrected in three places out of four, and
an `as u8` safety comment that a four-line probe falsifies.
