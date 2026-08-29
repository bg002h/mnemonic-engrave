# REVIEW-S2-P5-whole-r1 — post-implementation adversarial EXECUTION review, whole S2 cycle

**Reviewer:** independent execution reviewer (opus), dispatched as the project's
mandatory non-deferrable post-implementation gate (CLAUDE.md, per-phase pattern
step 4). **Date:** 2026-08-29.

**Targets, both read-only:**

| repo | worktree | range | tip |
| --- | --- | --- | --- |
| mnemonic-engrave | `/scratch/code/shibboleth/me-worktrees/impl-descriptor-s2` (`impl/descriptor-s2`) | `0144f02..9e4ba47` | `9e4ba47` S2 P5.1: FOLLOWUPS reconciliation + the CHANGELOG entry |
| seedhammer fork | `/scratch/code/shibboleth/sh-worktrees/s2-descriptor-arm` (`s2/descriptor-arm`) | `a5e29b4..231b7c2` | `231b7c2` gui: F-423 — bundlePlatePlan packs a card's strings onto as many plates as fit |

Engrave diff: 38 files, 4011 insertions / 514 deletions, 22 commits.
Fork diff: 44 files, 2228 insertions / 269 deletions, 9 commits.

## VERDICT — **GREEN (0 Critical / 0 Important)**

**0 C / 0 I / 3 Minor / 2 Nit.** Nothing found in this review blocks the merge.
The §5.2 canonical round trip holds end to end on every §4 input form and every
adversarial variant I could construct: **no wallet-identity divergence anywhere,
at any layer, in either language.** All six suites are green. The three Minors
are records-accuracy defects — a spec paragraph that contradicts itself, a
plate-count fact that propagated to five files and missed a sixth, and a
mislabelled fixture that re-introduces a correction the walk log had already
made. None of them changes a wallet, an exit code, an operator action, or a
plate.

Per the project's re-review rule, a clean round **closes** this loop. Item 6 of
§11 — a `ClassDescriptor` record displayed on real hardware — remains outside
any desk review's reach and stays the operator's, as both records say.

### Findings at a glance

| # | sev | finding |
| --- | --- | --- |
| M1 | Minor | `SPEC_descriptor_input.md` §11 item 1's S2 amendment contradicts itself about whether the device half closes at the desk; the CHANGELOG resolves it silently in one direction |
| M2 | Minor | F-423's new plate counts propagated to 5 files and missed `gui/bundle_flow.go` (4 sites still say 9/6-9/7-of-9) |
| M3 | Minor | "the keyed single-sig card … two strings" mislabels the BARE card (measured: bare = 2 strings, keyed = 3), re-introducing the walk log's own correction 3 — in `design/FOLLOWUPS.md` F-423 and `gui/bundle_engrave_test.go:112` |
| N1 | Nit | §6's `multi` remedy quote says "keeps `/0/*`"; the shipped text says "keeps the fixed index" (the shipped wording is the better one — it also covers the `<0;1>` shape) |
| N2 | Nit | `print_descriptor_confirmation` is a separate pass after mdmk/mt, so a mixed container would print descriptor lines out of record order — unreachable today (single-document mode; F-414 owns mixed containers) |

---

## 1. Method, and what I did NOT re-audit

Settled by the per-phase gates and deliberately not re-derived: the 7-round R0
plan loop; REVIEW-S2-P1P2-r1 and its fold; REVIEW-S2-P3-r1, its fold and r2's
GREEN close (187-case parity, `asciiNormalise` byte parity); P4.2's implementer
gates and mutations.

What this review adds is the thing per-phase review structurally could not see:
**the whole loop executed against the real binaries.** Every number below was
produced by running something, not by reading a report. The device half is not
simulated — it is the fork branch's own `sysw.Open`, `sysw.Classify`,
`nonstandard.OutputDescriptor`, `address.Receive` and `md.WalletPolicyIdChunks`,
compiled from `s2/descriptor-arm` and driven over containers written by the real
`me` binary built from `impl/descriptor-s2`.

**Harness** (all in the scratchpad; neither worktree was written to):

- `probe/` — a Go module with `replace seedhammer.com => <the fork worktree>`,
  so the device code under review is linked verbatim without touching it. It
  opens a container, classifies every record, and for each `ClassDescriptor`
  record re-parses it on `walletPolicyFlow`'s exact route
  (`nonstandard.OutputDescriptor([]byte(strings.TrimSpace(body)))`,
  `gui/wallet_policy.go`), then derives address 0, address 1 and the
  WalletPolicyId from **its own parse**. `walletIDOf` is
  `nonstandard/descriptor_seam_test.go:334` verbatim.
- `rt/` — materialises all 72 vector inputs to files and drives
  pack → show → probe.

**Two harness errors of my own, recorded so the numbers are readable.** My first
engrave suite run failed one test — `cross_lang
rust_ndef_parses_in_seedhammer_go_reader` — because I set `ME_REQUIRE_GO=1`
without Go on `PATH`. That is the gate working exactly as designed: it refused to
report a pass it could not earn. Re-run correctly, green. Separately, four
`--out /dev/null` invocations returned rc=2 on the *write*, not the logic; re-run
against real files. Both are noted because a reader tracing my commands would
otherwise reproduce them.

---

## 2. The §5.2 canonical ROUND TRIP, hammered

**The question:** pack → classify → decode → *same wallet*, for every §4 input
form. Any wallet-identity divergence anywhere in the loop is Critical.

**The loop, per row:**

```
me sysw pack --as descriptor --no-passphrase --in <input> --out <blob>
me sysw show <blob>                                   # host re-parse + §5.4 block
rtprobe <blob>                                        # sysw.Open → sysw.Classify
                                                      # → nonstandard.OutputDescriptor
                                                      # → address.Receive(0), (1)
                                                      # → md.WalletPolicyIdChunks
```

All **19 host-admitted rows** of `descriptor_seam_vectors.json` were run — every
`format` the file carries, plus the adversarial variants the brief names.

| # | input form (vector row) | §4 format | pack | `show` records | device `Classify` | record bytes == `canonical` | canonical: host == device | address 0: host == device | address 1 (device) | wallet-id: host == device |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | `accepted/sh-wsh-sortedmulti-16-keys` | bip380 | rc=0 | 1 × `Descriptor` | `ClassDescriptor` | **EQ** | **EQ** | **EQ** | `3AszGumRQUxNkAFngP…` | **EQ** |
| 2 | `formats-happy/bip380-sortedmulti-multipath` | bip380 | rc=0 | 1 × `Descriptor` | `ClassDescriptor` | **EQ** | **EQ** | **EQ** | `bc1q3m9z4nhe7y376u…` | **EQ** |
| 3 | `formats-happy/bluewallet-sh-fixture` | bluewallet | rc=0 | 1 × `Descriptor` | `ClassDescriptor` | **EQ** | **EQ** | **EQ** | `bc1qnww8rjenwn24ps…` | **EQ** |
| 4 | `formats-happy/json-label-descriptor` | json | rc=0 | 1 × `Descriptor` | `ClassDescriptor` | **EQ** | **EQ** | **EQ** | `bc1q3m9z4nhe7y376u…` | n/a (no md1 policy form) |
| 5 | `md1-split/childless` | bip380 | rc=0 | 1 × `Descriptor` | `ClassDescriptor` | **EQ** | **EQ** | **EQ** | `bc1q9edtz99nhdf95k…` | **EQ** |
| 6 | `md1-split/fixed-index` | bip380 | rc=0 | 1 × `Descriptor` | `ClassDescriptor` | **EQ** | **EQ** | **EQ** | `bc1q9edtz99nhdf95k…` | n/a (no md1 policy form) |
| 7 | `md1-split/mixed-childless-and-multipath` | bip380 | rc=0 | 1 × `Descriptor` | `ClassDescriptor` | **EQ** | **EQ** | **EQ** | `bc1q9edtz99nhdf95k…` | **EQ** |
| 8 | `md1-split/mixed-fixed-and-multipath` | bip380 | rc=0 | 1 × `Descriptor` | `ClassDescriptor` | **EQ** | **EQ** | **EQ** | `bc1q9edtz99nhdf95k…` | n/a (no md1 policy form) |
| 9 | `md1-split/mixed-nowildcard-and-multipath` | bip380 | rc=0 | 1 × `Descriptor` | `ClassDescriptor` | **EQ** | **EQ** | **EQ** | `bc1qk9l3n9d959l9k3…` | n/a (no md1 policy form) |
| 10 | `md1-split/multipath-no-wildcard` | bip380 | rc=0 | 1 × `Descriptor` | `ClassDescriptor` | **EQ** | **EQ** | **EQ** | `bc1qu2cc6t70nm0tw0…` | n/a (no md1 policy form) |
| 11 | `promotion/01-bare-xpub` | promoted-key | rc=0 | 1 × `Descriptor` | `ClassDescriptor` | **EQ** | **EQ** | **EQ** | `1L433eak3CakwF1oif…` | **DIVERGE** |
| 12 | `promotion/02-bare-zpub` | promoted-key | rc=0 | 1 × `Descriptor` | `ClassDescriptor` | **EQ** | **EQ** | **EQ** | `bc1q6r74smzann29rr…` | **DIVERGE** |
| 13 | `promotion/05-origin-44h` | promoted-key | rc=0 | 1 × `Descriptor` | `ClassDescriptor` | **EQ** | **EQ** | **EQ** | `1L433eak3CakwF1oif…` | **DIVERGE** |
| 14 | `promotion/06-origin-49h` | promoted-key | rc=0 | 1 × `Descriptor` | `ClassDescriptor` | **EQ** | **EQ** | **EQ** | `3EDaYD2EhnyGbX8XKE…` | **DIVERGE** |
| 15 | `promotion/07-origin-84h-zpub` | promoted-key | rc=0 | 1 × `Descriptor` | `ClassDescriptor` | **EQ** | **EQ** | **EQ** | `bc1q6r74smzann29rr…` | **DIVERGE** |
| 16 | `promotion/13-children-no-origin` | promoted-key | rc=0 | 1 × `Descriptor` | `ClassDescriptor` | **EQ** | **EQ** | **EQ** | `1L433eak3CakwF1oif…` | **DIVERGE** |
| 17 | `promotion/14-bare-xpub-trailing-newline` | promoted-key | rc=0 | 1 × `Descriptor` | `ClassDescriptor` | **EQ** | **EQ** | **EQ** | `1L433eak3CakwF1oif…` | **DIVERGE** |
| 18 | `whitespace/crlf-bip380` | bip380 | rc=0 | 1 × `Descriptor` | `ClassDescriptor` | **EQ** | **EQ** | **EQ** | `bc1q3m9z4nhe7y376u…` | **EQ** |
| 19 | `whitespace/leading-space-bip380` | bip380 | rc=0 | 1 × `Descriptor` | `ClassDescriptor` | **EQ** | **EQ** | **EQ** | `bc1q3m9z4nhe7y376u…` | **EQ** |

**Result: 19/19 rows, zero divergences at every layer.**

- **`pack` rc=0 on 19/19.**
- **Exactly one `Descriptor` record on 19/19** — `show` reports
  `public record 0: descriptor — complete in one record` and nothing else
  (§11 item 1's host half, and §5.1's single-document mode).
- **Device classification: `ClassDescriptor` on 19/19.** Including the two §4.6
  whitespace rows and `promotion/14-bare-xpub-trailing-newline`, whose *raw
  bytes* are `device_admits: false` — the host normalises, packs the canonical,
  and the canonical classifies. That asymmetry is exactly what §4.6 specifies
  and the round trip confirms it does not leak into the record.
- **Raw record bytes == the file's `canonical` on 19/19** (byte comparison of
  `p.Public[0]` against the `canonical` column — not of `show`'s re-encode, which
  would have hidden a packer that wrote something else and re-rendered it).
- **Address 0 equal, host vs device, on 19/19.**
- **Wallet-id equal, host vs device, on all 7 rows where both compute one.** The
  other 12: 5 rows where the host prints `wallet-id: none — this wallet has no
  md1 policy form` (correct — these are the `/0/*` and no-wildcard shapes
  §5.3(a) refuses, and naming an id there would collapse three distinct
  descriptors onto one value, which is the `/0/*` collapse §7 exists to catch),
  and 7 promoted single-key rows where my probe's `walletIDOf` has no arm
  (`md.EncodeMultisig` is multisig-only, exactly as that helper's own comment
  states — a probe limit, not a divergence; those rows are covered by the md1
  half in §3 below).
- **`DescriptorScreen` displays no wallet-id**, so nothing on the device names an
  id for the shapes the host declines to name one for. Checked at
  `gui/gui.go` `descriptorFlow` → `DescriptorScreen{Descriptor: desc}`. This is
  the one place a `/0/*` collapse could have reached a screen, and it does not.

### 2.1 Canonical idempotence — a fixed point, all 19

Re-packing the canonical must return the canonical, or a second pass through
`me` would silently change the wallet.

```
$ # for each admitted row: write `canonical` to a file, pack it, read the record back
canonical re-packed on 19 admitted rows
FIXED-POINT FAILURES: 0
```

Host record bytes and the device's independent re-encode both equal the input
canonical on 19/19.

---

## 3. Walk journeys, re-run

### 3.1 W14 — the bequest walk, BOTH halves, BOTH languages

The walk's central claim is that the `--as md1` cards and the `--as descriptor`
plate cut from one wallet name **the same wallet**. Four independent
computations per row: `me`'s §5.4 block on each `--as` path, plus the fork's own
code on each resulting container (`walletIDOf` over the parsed `Descriptor`
record; `md.WalletPolicyIdChunks` over the md1 records).

| wallet (vector row) | host `--as descriptor` | device, `Descriptor` record | host `--as md1` | device, md1 cards | verdict |
| --- | --- | --- | --- | --- | --- |
| `accepted/sh-wsh-sortedmulti-16-keys` | `bbd2dc3af0bd1c6e301ca2e00bb5197f` | `bbd2dc3af0bd1c6e301ca2e00bb5197f` | `bbd2dc3af0bd1c6e301ca2e00bb5197f` | `bbd2dc3af0bd1c6e301ca2e00bb5197f` | **SAME WALLET** |
| `formats-happy/bip380-sortedmulti-multipath` | `9e95257e60aacbb260129dac7b36d9f4` | `9e95257e60aacbb260129dac7b36d9f4` | `9e95257e60aacbb260129dac7b36d9f4` | `9e95257e60aacbb260129dac7b36d9f4` | **SAME WALLET** |
| `formats-happy/bluewallet-sh-fixture` | `a67e07d16b2500fde6c557a76c7390f6` | `a67e07d16b2500fde6c557a76c7390f6` | `a67e07d16b2500fde6c557a76c7390f6` | `a67e07d16b2500fde6c557a76c7390f6` | **SAME WALLET** |
| `md1-split/childless` | `47ecf2de11530f266e9b08640734447a` | `47ecf2de11530f266e9b08640734447a` | `47ecf2de11530f266e9b08640734447a` | `47ecf2de11530f266e9b08640734447a` | **SAME WALLET** |
| `md1-split/mixed-childless-and-multipath` | `47ecf2de11530f266e9b08640734447a` | `47ecf2de11530f266e9b08640734447a` | `47ecf2de11530f266e9b08640734447a` | `47ecf2de11530f266e9b08640734447a` | **SAME WALLET** |
| `promotion/01-bare-xpub` | `81ef604b1e538c302929a6fa4c4dcf60` | `— (probe helper is multisig-only)` | `81ef604b1e538c302929a6fa4c4dcf60` | `81ef604b1e538c302929a6fa4c4dcf60` | **SAME WALLET** |
| `promotion/02-bare-zpub` | `e657ccee1d44ccd746a5ba2b82ceed16` | `— (probe helper is multisig-only)` | `e657ccee1d44ccd746a5ba2b82ceed16` | `e657ccee1d44ccd746a5ba2b82ceed16` | **SAME WALLET** |
| `promotion/05-origin-44h` | `ed04976348033a75a1049e3e6b9d9180` | `— (probe helper is multisig-only)` | `ed04976348033a75a1049e3e6b9d9180` | `ed04976348033a75a1049e3e6b9d9180` | **SAME WALLET** |
| `promotion/06-origin-49h` | `16d03d377bf15f3a7eeb4e24a605c0dd` | `— (probe helper is multisig-only)` | `16d03d377bf15f3a7eeb4e24a605c0dd` | `16d03d377bf15f3a7eeb4e24a605c0dd` | **SAME WALLET** |
| `promotion/07-origin-84h-zpub` | `010e289c332fd65ef26910f8ceb839b7` | `— (probe helper is multisig-only)` | `010e289c332fd65ef26910f8ceb839b7` | `010e289c332fd65ef26910f8ceb839b7` | **SAME WALLET** |
| `promotion/13-children-no-origin` | `81ef604b1e538c302929a6fa4c4dcf60` | `— (probe helper is multisig-only)` | `81ef604b1e538c302929a6fa4c4dcf60` | `81ef604b1e538c302929a6fa4c4dcf60` | **SAME WALLET** |
| `promotion/14-bare-xpub-trailing-newline` | `81ef604b1e538c302929a6fa4c4dcf60` | `— (probe helper is multisig-only)` | `81ef604b1e538c302929a6fa4c4dcf60` | `81ef604b1e538c302929a6fa4c4dcf60` | **SAME WALLET** |
| `whitespace/crlf-bip380` | `9e95257e60aacbb260129dac7b36d9f4` | `9e95257e60aacbb260129dac7b36d9f4` | `9e95257e60aacbb260129dac7b36d9f4` | `9e95257e60aacbb260129dac7b36d9f4` | **SAME WALLET** |
| `whitespace/leading-space-bip380` | `9e95257e60aacbb260129dac7b36d9f4` | `9e95257e60aacbb260129dac7b36d9f4` | `9e95257e60aacbb260129dac7b36d9f4` | `9e95257e60aacbb260129dac7b36d9f4` | **SAME WALLET** |

**14 rows where both `--as` values carry. Zero divergences.** Every wallet is
named identically by the host on both plate types and by the device on both
plate types. The `—` cells are the probe-helper limit described in §2, not a
disagreement: on those rows the host and the device md1 route still agree.

This is the F-212 class made concrete at the container layer rather than the
string layer, and it holds.

### 3.2 The omitted-`--as` journey, on EVERY §4 form

P1's consult-first ordering and P2's shipped flip meet here. §11 item 5 requires
`EXIT_USAGE` (2) plus §5.1's block for an input at least one `--as` value
carries.

```
$ me sysw pack --no-passphrase --in <input> --out <blob>     # --as omitted
```

**19/19 admitted rows exit 2**, each printing §5.4's identification block
followed by §5.1's choice block. The "read as:" line names the right branch on
every one — `a BlueWallet ``Key: value`` setup file` (1), `a plain BIP-380
descriptor` (10), `a ``{"label":…,"descriptor":…}`` JSON export` (1), `a single
extended key` (7).

Two rulings verified as *specified*, not as accidents:

- The choice block **marks nothing** — no `(not available in this build)` — which
  is §5.1's post-S2 state, both values shipped.
- On a row only one value carries (`formats-happy/json-label-descriptor`,
  `md1_admits: false`), the block **still offers both, unmarked**. That is
  §5.1's explicit r14 new-M2 ruling ("the operator who picks the input-dead
  value gets that path's own refusal, which names the working flag"), not an
  omission. I checked the ruling before classifying the behaviour.

### 3.3 The five-case matrix, by hand

| # | case | want | got | message |
| --- | --- | --- | --- | --- |
| 1 | carried, `--as` omitted | 2 | **2** | §5.1's choice block |
| 2 | inadmissible (`ypub` key), `--as` omitted | 3 | **3** | "`me` admits exactly `xpub`, `tpub`, `zpub`, `Ypub`, `Zpub`…" |
| 3 | carried by NEITHER — `wsh(multi(2,…/0/*))` | 3 | **3** | the substituted `multi` remedy (below) |
| 4 | inadmissible + explicit `--as descriptor` | 3 | **3** | the admission refusal, ordering per r14 new-I1 |
| 5 | `multi` + explicit `--as descriptor` | 3 | **3** | "the device's descriptor parser accepts `sortedmulti` and not `multi`…" |
| 6 | old witness `sortedmulti(…/0/*)` — the 3→2 flip | 2 | **2** | §5.1's choice block |

Case 3 is the one worth reading in full, because a generic `/0/*` remedy here
would send the operator to `--as descriptor`, which refuses `multi` — a path
that cannot work. §6 line 1527 specifies a *substituted* remedy for `multi`
inputs, and the shipped text carries it:

```
me: md1 cannot carry this wallet as written: key @0 (…) uses `/0/*`, and key @1
    (…) uses `/0/*`, a single fixed chain index, which has no md1 form --
    encoding it would silently produce a DIFFERENT wallet. This is a `multi`
    policy, which only `--as md1` carries -- and md1 cannot represent `/0/*`.
    No `me` path engraves this file as written, in any build. Re-export with
    `<0;1>/*` -- carried in every build. (Re-exporting as a `sortedmulti` policy
    keeps the fixed index but is a DIFFERENT policy -- `me` will not rewrite it
    -- and needs the scannable-plate path.)
```

The substitution fires, the dead-end is named as permanent, and the remedy is
executable. (The one wording variance from §6's quote is **N1** below.)

Eight further §4.7 narrowing refusals under explicit `--as descriptor`, all
rc=3 with the specific cause named: `tr-sortedmulti`, `wpkh-sortedmulti`,
`pkh-sortedmulti`, `sh-wpkh-sortedmulti`, `wsh-sortedmulti-21-keys`,
`miniscript`, `bare-Ypub-refused`, `colliding-origin-sortedmulti`.

### 3.4 §11 item 1's four formats, and `--expect`

All four §4 formats pack, show exactly one `Descriptor` record, and classify
`ClassDescriptor` on the device (rows 1–3 and the promotion rows in §2's table).

`--expect` (P1.3 — "names both carriers"), measured:

| invocation | rc |
| --- | --- |
| `--as descriptor … --expect descriptor` | **0** |
| `--as md1 … --expect descriptor` | **0** |
| `--as descriptor … --expect mnemonic` | **4** |
| `--as descriptor … --expect cosigner` | **4** |

And the ordering claim in `main.rs`'s new comment — that the gate runs *after*
`--expect`, so a stated expectation wins over a choice prompt:

```
$ me sysw pack --no-passphrase --in <a descriptor> --expect mnemonic --out …
rc=4          # not 2 — the expectation that was actually stated
```

---

## 4. Cross-phase integration seams nobody owned

### 4.1 `show` on containers holding mdmk / mt / text — no cross-contamination

Two independent guarantees, both checked:

**The change is structurally additive.** `git show cde5c8b --stat` is
`202 insertions(+)`, **zero deletions**, across three files;
`crates/me-cli/src/main.rs | 36 ++++++`. `print_descriptor_confirmation` is a
new function appended after `print_mt_confirmation`, guarded by
`if sysw::classify(r) != sysw::record::Class::Descriptor { continue; }`. No line
of the pre-existing `show` path was touched, so the byte-identity pin's
provenance is not load-bearing — the diff itself proves additivity.

**Invariant 2 is pinned to a genuinely pre-arm capture.** I verified the
provenance rather than accepting it:

```
$ git log --oneline 0144f02..9e4ba47 -- crates/me-cli/testdata/record_corpus_pre_s2.json
b8f0538 S2 P1.0: the pack-path gate keys on identification, not classification failure
$ git log --oneline 0144f02..9e4ba47 -- crates/me-cli/src/sysw/mod.rs
6efd7b5 …
282a071 S2 P1.1: sysw::classify gains the Descriptor arm, delegating to host_admits
$ git show b8f0538:crates/me-cli/src/sysw/mod.rs | grep -n Descriptor
200:/// **Descriptor and Address are deliberately absent**, …
```

The capture landed at P1.0, the arm at P1.1 — the capture predates the arm, and
the file has not been touched since. Its 33 records classify
`Unknown ×12, MdMk ×8, Mt ×6, FreeText ×2, Codex32Secret ×2, Mnemonic ×1, Tx ×1,
Passphrase ×1` — **zero `Descriptor`**, and all 33 `consult` outcomes are
`record-refusal`. The 12 `Unknown` records are precisely the class the new arm
takes records from, and none moved.

Executed, not just read: an `--as md1` container of 6 md1 records shows
6 × `md1/mk1 — confirmed` and **0** `descriptor —` lines.

### 4.2 Sealed payloads at the walletPolicy door

`me` **cannot produce** a sealed container carrying a `Descriptor` record, and
says so rather than silently ignoring the flag:

```
$ me sysw pack --as descriptor --passphrase-words 4 --in <bip380> --out …
sealing:  NOT SEALED — no record in this payload is secret material, so there
      is nothing to encrypt. …
      --passphrase-words is IGNORED here, deliberately. `pack` encrypts only
      secret-class records, so a passphrase would have opened nothing and
      protected nothing — and you would have had to keep it forever.
rc=0
```

A descriptor is not secret-class, so it never reaches the secret section; and
§5.1's single-document mode means it never shares a container with one. The
sealed-plus-descriptor configuration is therefore unreachable from the shipped
tools, and the door is not exercised by it. (`sysw.Open`'s AAD is
`header || public section`, so splicing a record into a sealed container's
public section fails the tag — but that is shipped pre-S2 behaviour which S2
does not touch, and `open_wires_the_same_aad_in` passed in the run below.)

The multi-record journey — records *and* a descriptor in one file — refuses
correctly and names the split rather than sending the operator to a whole-file
read:

```
$ printf '%s\n%s\n' "<mnemonic>" "<canonical descriptor>" | … --in mixed.txt
me: record 1 is a wallet descriptor. A descriptor is packed ALONE: run
    `me sysw pack --as <descriptor|md1>` with just the descriptor -- one
    container cannot yet carry a descriptor plus other records. The other
    records pack without `--as`, as usual.
rc=4
```

### 4.3 F-423's packer against mixed md1 / mk1 / ms1 cards

The marking rule and the card boundary are the funds-relevant parts here, and
both are pinned non-vacuously:

- `TestBundlePlanNeverPacksAcrossCards` proves a `cardMS1` string never shares a
  plate with a `cardMD1` string — and it first asserts
  `bundlePlateTextFits(params, []string{a, b})` is **true**, so only the card
  boundary can be what keeps them apart. Without that guard the test would pass
  whenever the two simply did not fit.
- `bundlePlateMark`: every kind but `cardMS1` takes the caller's marking; a
  `cardMS1` plate is **never** marked. Unchanged by F-423, and the
  never-pack-across-cards rule is what keeps it meaningful once plates hold
  several strings.
- **The census defect F-423 would otherwise have created was caught and fixed.**
  `buildPlateCensusLines` and `buildPlateInventoryLines` now take the counts off
  `bundlePlatePlan` via `bundleCardPlateCounts(plan, …)` instead of
  `len(c.strings)`. Left alone, the census would have told the operator
  "md1 policy: 6 plates" on a run totalling 2 — on the one screen whose job is
  how many blanks to have ready. The restore document's headline count is
  computed from the same object (`plateWord(len(plan), …)`,
  `gui/multisig_build_census.go:85`), so the total and the enumeration are two
  readings of one plan and cannot disagree.
- The plate arithmetic is pinned and re-derived rather than asserted as a
  literal: `TestBundlePlanPacksACardOntoFewerPlates` runs the table
  `{1→1, 2→1, 3→1, 5→1, 6→2, 11→3}` and then recomputes
  `bundlePlateMD1Capacity` from the packer, so the constant cannot go stale
  while still reading as measured truth.

### 4.4 The consumer's parse route

`walletPolicyFlow` classifies with the §4.7-narrowed `isDescriptorRecord` but
parses with the wider `nonstandard.OutputDescriptor`. The safety argument is
that classification already proved admission — and the fold made it structural
rather than asserted by applying the same `strings.TrimSpace` the classifier
uses, with `whitespace/leading-space-bip380` named as the standing
counterexample. **Measured over all 19 admitted rows: the device's parse of the
packed record re-encodes to exactly the host's canonical every time**, so the
wider door never widens the *result*.

One design observation, not a finding: `walletPolicyFlow` offers `ClassMDMK`
first and `ClassDescriptor` in the `else if`, so an operator who answers "ENTER
IT" at the md1 prompt is then offered the descriptor. Declining both lands in
manual md1 entry, and taking the descriptor is an explicit second consent, so
no path is entered without the operator choosing it.

---

## 5. Records audit — every measurable claim, machine-checked

Records are the weak half, so each claim below was resolved against the tree or
the tooling rather than read.

| record | claim | check | verdict |
| --- | --- | --- | --- |
| CHANGELOG | vector file sha `e7a4160c…`, pinned byte-identical in the fork | `sha256sum` both copies + `cmp` | **TRUE** — `e7a4160ce064a6cb7ca31dc530e079c861cf2c8a075d75f793ef0d935f583758`, byte-identical |
| CHANGELOG | "72 rows" | `len(d['vectors'])` | **TRUE** — 72 |
| CHANGELOG | "§6's table is 35 rows post-S2" | `Row::ALL` = 35; `refusal_rows` = 35; `descriptor_refusals.rs:138` asserts 35 | **TRUE** — all three agree |
| CHANGELOG | "the `sysw_class` sample column replaced by an exhaustive derived rule asserted in both languages" | `TestDescriptorSeamSyswClass` / `…Canonical` (Go) + `every_single_line_input_classifies_by_the_admission_column` (Rust) | **TRUE** |
| CHANGELOG | "187-case parity probe, 0 divergences" | REVIEW-S2-P3-r2 (settled ground; cited, not re-derived) | consistent |
| CHANGELOG | "keyed single-sig 2→1" | test table `{2, 1}` | arithmetic **TRUE**; the *label* is wrong — see **M3** |
| CHANGELOG | "full 2-of-3 build 9→4" | `TestChainMdMkFromTheEmulatorsOwnPayloadToFourPlates` asserts `plates == 4` and `"This engraves 4 plates"` | **TRUE** (9 *strings* → 4 plates) |
| CHANGELOG | "Items 1-5 are discharged by the suites" | vs `SPEC` §11 item 1 lines 2098–2103 | **contradicts the spec** — see **M1** |
| CHANGELOG | "item 6 … and every flash remain the operator's" | — | **TRUE**, and correctly prominent |
| F-423 | "packer landed (fork `231b7c2`)" | `git log -1 231b7c2` | **TRUE** |
| F-423 | "capacity 5×85-char strings/plate" | `bundlePlateMD1Capacity = 5`, re-derived from the packer inside the test | **TRUE** |
| F-423 | "W14 bequest → 1 plate" | bare card = 2 strings, keyed = 3; table pins both `2→1` and `3→1` | **TRUE** for both spellings |
| F-423 | "a PACKED plate offers TEXT ONLY (no QR variant)" | `gui/gui.go:2576-2588`: `len(strs)==1` → TEXT+QR / TEXT ONLY / QR ONLY; else TEXT ONLY only | **TRUE**, and the unpacked case keeps its QR |
| F-426 | "`ypubVer` case landed (fork `0abbf81`, tests `fe9475c`)" | both resolve; `bip380.go` gains `case ypubVer: script = P2SH_P2WPKH` | **TRUE** |
| F-428 | "engrave `70f566e`, fork copy `29cb930`" | both resolve to the regeneration commits | **TRUE** |
| F-430 | "RESOLVED by `scripts/lint-gate.sh` (S2 P0.3, engrave `5deb88e`)" | `5deb88e` = "scripts: lint-gate.sh — the S2 plan's lint gate as one command (P0.3)"; ancestor of HEAD, before the review baseline | **TRUE** |
| F-431 | "`gui/sysw_admit.go` still admits `ClassDescriptor` to `progBundle` and `progMultisig` with no consumer" | read `sysw_admit.go:40-52`; only `progWalletPolicy`'s cell has a consumer | **TRUE** |
| F-432 | "`goprobe/go.mod`'s `replace` points at the transient S2 fork worktree" | `replace seedhammer.com => /scratch/code/shibboleth/sh-worktrees/s2-descriptor-arm` | **TRUE** |
| F-435 | "`backup.TestAPackedBodyCanCoverTheFooterRow` fails when done" | exists, `backup/engravetext_test.go:461` | **TRUE** |
| F-436 | "the corpus's only JSON row is multi-line" | format census: `json ×1`, its `input` contains `\n` | **TRUE** |
| fork | short-fingerprint panic fixed "as Rust convergence" | host: rc=3, *"the origin block's fingerprint is not 8 hex characters"*; device: `err=nonstandard: unrecognized output descriptor format`, no panic | **TRUE** — the host was already correct, so this is convergence and needs no Rust change (Rust-primary rule satisfied) |

---

## 6. Implementation-introduced regressions — the non-descriptor surface

Every file in the fork diff outside `sysw`/`nonstandard`/descriptor code, with
the reason each one changed:

| file | change | reason |
| --- | --- | --- |
| `bip380/bip380.go` | `case ypubVer: script = P2SH_P2WPKH` | F-426's device half |
| `nonstandard/parse.go` | `len(fp) > 4` → `len(fp) != 4` | short-fingerprint panic (S4.2 defect 4); Rust convergence, verified above |
| `gui/engraved_hook.go`, `…_tinygo.go` | `notifyPlateText(…, text string)` → `(…, strs []string)` | F-423: a plate now carries several strings; the join moved off the device hot path |
| `gui/multisig.go`, `gui/singlesig.go` | thread `EngraverParams` into the census calls | F-423: the census must measure real fit |
| `gui/multisig_verify.go` | comment "already cut nine plates" → "already cut every plate of the set" | F-423 count propagation — **done here** |
| `scripts/chain-mutation-check.sh` | test name `…ToNinePlates` → `…ToFourPlates` | F-423 rename |
| `cmd/emu/walk_build_policy.js` | loop bound `plates = 9` → `4`, with the arithmetic in the doc comment | F-423 |
| `cmd/emu/walk_trace_a.js` | loop bound `plates = 6` → `3`, with the arithmetic | F-423 |
| `backup/backup.go` | `Text.FooterRow` | P4.2's fit check; F-435 tracks the cleanup |

The engrave diff touches no non-descriptor surface: `main.rs`'s two hunks are
(a) moving §5.1's gate ahead of `admit_check` and scoping it to
`r#as.is_none()`, and (b) the additive `show` block. Everything else is
`descriptor/`, `sysw/`, testdata, tests, `design/` and `scripts/`.

**S1+S3 regression spot checks on the shipped `--as md1` surface** — S2 must not
have moved them:

| invocation | rc |
| --- | --- |
| `--as md1` bip380 multipath | **0** |
| `--as md1` BlueWallet fixture | **0** |
| `--as md1` promoted bare xpub | **0** |
| `--as md1` 16-key extreme | **0** |
| `--as md1` JSON `/0/*` | **3** (§5.3(a)) |
| `--as md1` `md1-split/fixed-index` | **3** (§5.3(a)) |

Unchanged.

---

## 7. Full validation surface — both repos, captured once

| suite | command | result |
| --- | --- | --- |
| engrave tests | `ME_REQUIRE_GO=1 cargo nextest run --locked --no-fail-fast` | `Summary [32.258s] 579 tests run: 579 passed, 1 skipped` — **exit 0** |
| engrave lint | `./scripts/lint-gate.sh` | fmt + clippy 1.85.0 + clippy nightly → `lint-gate: PASS` — **exit 0** |
| fork non-gui | `go test $(go list ./... \| grep -v '/gui$')` | every package `ok`, **0** `FAIL` lines — **exit 0** |
| fork gui | `scripts/gui-shard-test.sh ./gui/ 24` | `RESULT: ok -- all 1013 tests ran across 24 shards`, wall 24s — **exit 0** |
| fork TinyGo | `nix develop --command tinygo build … -target pico-plus2 … ./cmd/controller` | builds, total 1 498 636 bytes — **exit 0** |

Go 1.26.3 throughout, per the project note that `go.mod`'s pinned 1.25.10 cannot
build `./gui/`.

---

## 8. Findings

### M1 (Minor) — §11 item 1's amendment contradicts itself; the CHANGELOG resolves it silently

**Where:** `design/SPEC_descriptor_input.md:2094-2103`; `crates/me-cli/CHANGELOG.md:57`.

The S2 amendment to §11 item 1 says both of these, three lines apart:

```
… And the mechanism clause died with
its column: the device side is exercised by the Go test's DERIVED rule
(§7's `sysw_class` amendment), which asserts `sysw.Classify` over every row
in the file rather than four sampled ones. The host half of this item —
one record per format, classifying `Descriptor`, byte-equal to the
`canonical` the device measured — closes at the desk; the DEVICE half
still needs the flashed build.
```

Item 1's device conjunct is *"the device's `sysw.Classify` classifies that record
`Descriptor`"*. Nothing in it requires hardware — display on a real device is
**item 6**, which is separately and correctly reserved. So the first sentence is
right and the trailing clause conflates item 1 with item 6.

The CHANGELOG then writes `Items 1-5 are discharged by the suites`, picking the
first reading without noting that §11 item 1 says otherwise. One of the two
records is wrong about the other.

**Why Minor and not Important.** The safety-relevant fact is stated correctly and
prominently in both records — §9 item 1 ("Nothing has been run on hardware") and
the CHANGELOG's own next sentence ("Item 6 … and every flash remain the
operator's acceptance"). No reader is misled about what has and has not touched a
device. And my own measurements support the CHANGELOG's reading: `sysw.Classify`
from `s2/descriptor-arm` returned `ClassDescriptor` on containers written by the
real `me` binary for **19/19** admitted forms.

**Suggested resolution:** strike or rewrite the trailing clause of §11 item 1 so
it names item 6 as the hardware gate, rather than duplicating it. One sentence.

### M2 (Minor) — F-423's plate counts propagated to five files and missed `gui/bundle_flow.go`

**Where:** `gui/bundle_flow.go:543, 558, 681, 695` (fork).

F-423 changed a full 2-of-3 build from 9 plates to 4 and Trace A from 6 to 3, and
the fold swept the counts through `gui/multisig_build_census.go`,
`gui/multisig_verify.go`, `scripts/chain-mutation-check.sh`,
`cmd/emu/walk_build_policy.js` and `cmd/emu/walk_trace_a.js`. `bundle_flow.go`
was not swept:

```
543: // end of a Build-policy engrave too -- measured: S2's walk cuts 9 plates and
558: // restore document printed, headed "This backup is 9 plates ... If any of them
681: // 6 to 9 plates over hours, nothing records which were cut, and a power loss
695: // sentence at face value at plate 7 of 9 has exactly three options, all bad: cut
```

`git diff a5e29b4..231b7c2 -- gui/bundle_flow.go` matches none of these lines —
the file was edited, but not at these four.

**No functional impact.** These are code comments; the operator-facing count is
computed (`plateWord(len(plan), …)`, `multisig_build_census.go:85`) and was
verified correct. Line 543's load-bearing claim ("the modal is reached at the end
of a Build-policy engrave") is still true; only its incidental number is stale.
This is the "folds fail by incomplete propagation" class — the facts are right
and the duplicates are left — and it is worth recording precisely because five of
six sites *were* swept, which is what makes the sixth easy to trust.

### M3 (Minor) — "the keyed single-sig card … two strings" mislabels the BARE card

**Where:** `design/FOLLOWUPS.md:14658` (F-423);
`gui/bundle_engrave_test.go:112` (fork).

Both records call the 2-string bequest card "the keyed single-sig card". Measured
on this build, through the real binary:

```
$ me sysw pack --as md1 --no-passphrase --in <bare zpub>            # promotion/02
  strings: 2  lengths: [85, 83]  total: 168
$ me sysw pack --as md1 --no-passphrase --in <keyed, 84h origin>    # promotion/07
  strings: 3  lengths: [67, 67, 67]  total: 201
```

Two strings is the **bare** card; the keyed card is three. This is exactly the
error `WALK_descriptor_input_2026-08-28.md` correction 3 identified and fixed
("the walk measured the BARE key and wrote down the keyed one"), re-introduced
into two new records after being corrected — and correction 3 itself notes it was
"the second citation-by-description error in W14".

**No arithmetic impact:** the packer's table pins both `2→1` and `3→1`, so
F-423's "W14 bequest → 1 plate" is true for either spelling, and the capacity
constant is re-derived from the packer rather than trusted. Only the label is
wrong. Fixing it is two words in each place; leaving it means a future re-run
following the description measures the wrong artifact, which is precisely what
correction 3 warned about.

### N1 (Nit) — §6's `multi` remedy quote vs the shipped text

`design/SPEC_descriptor_input.md:1527` quotes *"(Re-exporting as a `sortedmulti`
policy keeps `/0/*` but is a DIFFERENT policy…)"*; the shipped text says *"keeps
the fixed index"*. The shipped wording is the **better** one — it also covers the
`<0;1>` no-wildcard shape the same row serves — so the spec quote is what should
move. No test asserts the spec's spelling, so nothing is red.

### N2 (Nit) — descriptor `show` lines would print out of record order in a mixed container

`print_descriptor_confirmation(&records)` is a third pass after
`print_mdmk_confirmation`'s loop and `print_mt_confirmation`, each looping all
records with its own class guard. In a container holding, say, `[Descriptor,
MdMk]`, the md1 line would print before the descriptor block despite the
descriptor being record 0. **Unreachable today** — `--as` makes the invocation
single-document, and F-414 owns the mixed container — so this is only worth a
line in F-414 whenever that flag is designed, not a change now.

---

## 9. What this review did NOT establish

Stated plainly, because a review that only lists what it proved is claiming the
rest.

1. **Nothing ran on hardware.** Every device-side measurement here is fork code
   compiled and called on this machine. §11 item 6 — a `ClassDescriptor` record
   loaded and DISPLAYED on the real SeedHammer II — is untouched by this review
   and remains the operator's, as does F-423's physical test plate.
2. **Change addresses and testnet remain unmeasured**, exactly as §9 item 3
   already says. My round trip derived receive 0 and receive 1 only.
3. **The sealed-plus-`Descriptor` container was reasoned about, not
   constructed.** I established it is unreachable from the shipped tools rather
   than building one by hand and driving it through the door. If F-414 ever
   makes it reachable, that door needs its own walk.
4. **I did not re-run the P3 parity probe.** The 187-case result is cited from
   REVIEW-S2-P3-r2 as settled ground per the brief.
5. **`walletIDOf` has no single-sig arm**, so 7 promoted-key rows have no
   device-side descriptor wallet-id in §2's table. Their identity is covered by
   the md1 route in §3.1, where host and device agree.

---

## 10. Worktree cleanliness

Both worktrees were read-only for the whole review. The probe module lives in the
scratchpad and reaches the fork through a `replace` directive rather than by
being placed inside it.

```
$ cd /scratch/code/shibboleth/me-worktrees/impl-descriptor-s2 && git status --porcelain | wc -l
0
$ cd /scratch/code/shibboleth/sh-worktrees/s2-descriptor-arm && git status --porcelain | wc -l
0
```

**Both byte-identical to their dispatch state. Nothing was committed and nothing
was pushed.**

---

## 11. Bottom line

**GREEN — 0 Critical / 0 Important.** The S2 cycle's central guarantee holds
under execution: a wallet packed by `me` as a `Descriptor` record is the same
wallet the device reads back, on every §4 input form, every adversarial variant,
both `--as` values and both languages — 19/19 rows on the round trip, 14/14 on
the bequest walk, 19/19 fixed points, zero divergences anywhere. Six suites
green. The three Minors are records defects that change no behaviour; M1 and M3
are each a one-sentence edit and are worth making before the merge because both
are the kind of claim a future cycle will read as measured truth.

This closes the mandatory post-implementation gate for merge. It does not
discharge §11 item 6.
