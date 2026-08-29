# REVIEW-S2-P3-r1 — proportional adversarial review of the Go port

**Phase:** P3 of `IMPLEMENTATION_PLAN_descriptor_input_S2.md` (plan GREEN @ `ff670b4`, main repo).
**Reviewer:** independent agent, opus tier. Author ≠ reviewer.
**Targets:**

| target | range | worktree |
| --- | --- | --- |
| fork branch `s2/descriptor-arm` (incl. the two controller pre-fixes at `0abbf81`) | `a5e29b4..fe9475c` | `/scratch/code/shibboleth/sh-worktrees/s2-descriptor-arm` |
| engrave P3.5 fold | `0096462..781d10d` | `/scratch/code/shibboleth/me-worktrees/impl-descriptor-s2` |

**Implementer's report read:** `design/agent-reports/IMPL-S2-P3.md` (engrave branch).
**Nothing modified. Nothing pushed. Both worktrees byte-identical at exit — verified, §7.**

## Verdict

**RED — 2 Critical / 2 Important / 1 Minor / 0 Nit.**

| # | sev | one line |
| --- | --- | --- |
| C1 | Critical | §4.3's base58-run scan reads the WHOLE record, so a JSON-form descriptor whose `label` holds a non-admitted extended key classifies `ClassUnknown` while `host_admits` is true — and the implementer's Deviation 2 argues this is impossible. |
| C2 | Critical | `classifyConstellation`'s `strings.TrimSpace` (Unicode) is wider than the host's `normalise` (`is_ascii_whitespace`), so `\v`/NBSP/NEL/EM-SPACE/IDEOGRAPHIC-SPACE padding makes the device classify `ClassDescriptor` on records `me` refuses — the device-wider direction. |
| I1 | Important | `gui/wallet_policy.go`'s "Re-parsing cannot fail here … over these exact bytes" is false: classification parses the TRIMMED string, the consumer parses the RAW record. Demonstrated on shipped corpus row `whitespace/leading-space-bip380`. |
| I2 | Important | P3.5 omitted a plan-mandated amendment: §7 requirement 3's device-column phrasing, which P3.3 falsified (the Go seam test now asserts the `host_admits` column too). |
| M1 | Minor | `bip380/ypub_test.go:19` quotes the pre-P3.5 refusal text (`"the device admits exactly …"`); `me` now prints `` "`me` admits exactly …" ``. P3.5 falsified a fork comment its own diff never touched. |

**What HELD, with evidence:** the §4.7 conjunct port (81/81 constructed cases agree, §2), the §4.5
promotion narrowing including the tpub-inside-a-descriptor case the narrowing must NOT fire on, arm
order, crash containment in both directions, the first-execution walk (fails under 3 mutations), the
walk fixture's provenance (reproduced byte-identically), the §4.2 defect-4 panic fix, and five of the
seven P3.5 amendments.

---

## 0 — Method and harness

Two probes, both built in the scratchpad, neither touching a worktree:

- **device probe** — Go module with `replace seedhammer.com => <fork worktree>`, calling the real
  `sysw.Classify` and the real `nonstandard.OutputDescriptor`. Toolchain
  `/nix/store/33fw5m31lfcnk4ff2f0df7j2bxnh8lgk-go-1.26.3/bin`, `go1.26.3 linux/amd64`.
- **host probe** — Rust crate with a path dependency on
  `<engrave worktree>/crates/me-cli`, calling `mnemonic_engrave::descriptor::host_admits` and
  `::format_of` — the same function `crates/me-cli/tests/descriptor_seam.rs:595` asserts.

Both read **hex-encoded** lines, so no escape layer can corrupt a control character. (An earlier
run with JSON-quoted input produced four *false* divergences — `\f`, and three `\uXXXX` forms — because
the Rust probe's hand-rolled unquote mishandled them. Recorded because the corrected run is what the
findings below rest on.)

Mutations were applied with `go test -overlay`, which redirects a file at build time and **leaves the
worktree byte-identical**. `git status --porcelain` was empty after every mutation run.

**Corpus fact established first, because it bounds what the shipped gate can see:**
of the 72 rows, **59 are single-line** and only those are asserted by `TestDescriptorSeamSyswClass`.
The one JSON-format row (`formats-happy/json-label-descriptor`) is **multi-line**, so **no gate in
either repo runs the classifier over a JSON-form record.** That is the blind spot C1 lives in.

---

## 1 — C1 (Critical): the §4.3 string-level check refuses admitted records

**Mechanism.** `keyVersionsAdmitted` (`sysw/descriptor.go:121`) scans the **entire record** for
maximal base58 runs of ≥ 100 characters (`eachExtendedKey`, `:151`) and requires every run that
decodes under `hdkeychain.NewKeyFromString` to carry one of §4.3's five versions. The scan has no
notion of which runs the cascade actually treated as keys.

The Go cascade's **branch 3** (`nonstandard/parse.go:43-54`) parses
`{"label": …, "descriptor": …}` and copies `label` into `desc.Title`. **A label is arbitrary text
inside the record**, and `"`, `{`, `}`, `:` and `,` are all outside the base58 alphabet, so a label
is its own maximal run. Put a `ypub` there and the scan refuses a record whose every *key* is an
`xpub`.

**Constructed counterexample and measurement** (`label` = the `version-gap/full-origin-ypub` row's
`ypub`; `descriptor` = the `formats-happy/bip380-sortedmulti-multipath` canonical, three `xpub`s):

```
case                                    sysw.Classify   OutputDescriptor  host_admits
C1 json label=ypub, keys all xpub       ClassUnknown    scanOK            ADMIT      <<< DIVERGE
C1 control: json label=plain            ClassDescriptor scanOK            ADMIT
```

The record parses fine (`scanOK`) — the refusal comes from the version scan alone. The same
divergence reproduces on **19 of the constructed JSON-wrapped rows** in the broad sweep
(§2), i.e. on every `host_admits` canonical wrapped in a JSON label.

**Why this is Critical rather than a curiosity.** The plan states the arm is "a faithful port of
§5.2's predicate", P3.3 states the derived rule "stays EXACT … never relaxed to fit the arm", and
the shipped test's own failure message says the classifier "must answer §5.2's predicate EXACTLY".
This is a constructible single-line input on which it does not. It is also the exact class the
brief named.

**It directly refutes the implementer's Deviation 2.** IMPL-S2-P3 argues:

> "the direction that could be wrong is a false REFUSAL — which would require the parser to accept
> a ≥ 100-character base58 run that is not a key, **which the grammar makes impossible**."

The JSON branch's `label` is precisely a ≥ 100-character base58 run that is not a key, and the
grammar not only permits it, it is the branch's own titling field. The load-bearing half of the
deviation's justification is false.

**Measured impact bound, stated so the controller can weigh the fix.** Neither direction is
reachable from an `me`-written payload:

```
$ me sysw pack --no-passphrase --in <the JSON record> --out /dev/null      → rc=2   (§5.1's window refuses it)
$ me sysw pack --no-passphrase --as descriptor --in <same> --out o2.bin    → rc=0, packs the CANONICAL
      descriptor: wsh(sortedmulti(2,[dc567276/48h/…]xpub…/<0;1>/*,…))#ud8uyjz3
```

So `me` never writes a JSON-form record, and the failure is **fail-closed** (the record goes inert;
no wrong wallet reaches a screen). §7's normative invariant `host_admits(input) ⇒
device_admits(canonical)` is **not** violated. The defect binds a third-party or hand-built payload,
and it binds the plan's exactness guarantee.

**Suggested shape of a fix (not prescriptive — reproduce the defect, not the remedy):** run the
version check over the **key expressions the cascade actually produced** rather than over the record,
or scope the scan to the `descriptor` substring on the JSON branch. Either way the corpus needs a
**single-line** JSON row, or the gate stays blind.

---

## 2 — The conjunct port and the §4.5 narrowing: 81/81, no divergence

Before the findings, what held. An 81-case adversarial battery, every case constructed to sit on a
conjunct boundary, keys minted by re-versioning corpus `xpub`s and re-checksumming (the same
technique `bip380/ypub_test.go` uses for its `upub` twin). **Every one of the 81 agreed.**

| conjunct | cases | notable |
| --- | --- | --- |
| 1 shape | 7 | `wsh(KEY)`, `sh(KEY)`, `tr(sortedmulti)` refused both sides; `sh(sortedmulti)`, `sh(wsh(sortedmulti))`, `tr(KEY)`, `pkh(KEY)` admitted both |
| 2 threshold | 3 | `k=0`, `k=3>n`, `k=-1` refused both |
| 3 key count | 12 | the exact boundary: `sh` 15 ADMIT / 16 REFUSE / 20 REFUSE / 21 REFUSE; `wsh` and `sh(wsh)` 15,16,20 ADMIT / 21 REFUSE |
| 4 versions | 22 | all 11 versions (`xpub tpub zpub Ypub Zpub ypub upub vpub Upub Vpub xprv`) × {inside a descriptor, bare} |
| 5 network | 4 | all-`tpub` ADMIT both; mixed `xpub`/`tpub` REFUSE both; `Zpub`+`xpub` and `Ypub`+`zpub` (both mainnet) ADMIT both |
| 6 origins | 2 | `[fp]` with no path REFUSE both; all-zero fingerprint ADMIT both |
| 7 use-site | 14 | absent, `/*`, `/0/*`, `/<0;1>`, `/<0;1>/*` ADMIT both; `/0h/*`, `/*h`, `/<0;2>/*`, `/<0;2>`, `/0/1/*`, `/*/*`, `/<0;1>/<2;3>`, `/0`, `/<0;1>/0/*` REFUSE both |
| 8 key identity | 9 | see below |
| §4.5 promotion | 7 | see below |

**Conjunct 8, the subtlest, checked by construction against `admit.rs:281-310`.** Rust keys (a) on
`same_origin && identity() != identity()` and (b) on `identity() == identity() && children ==
children`, where `identity()` is `(key_data, chain_code)` (`cascade.rs:254`). Go's
`keyIdentityOK` (`sysw/descriptor.go:355`) merges both loops using `sameKeyMaterial` = `KeyData` +
`ChainCode`. Equivalent, and measured equivalent on the cases that separate them:

- same origin, different key → REFUSE both; same key, same use-site → REFUSE both;
- same key, **different** use-sites (`<0;1>/*` vs `<2;3>/*`) → ADMIT both (the legal two-chain wallet);
- **no fingerprint**, different keys → ADMIT both (an absent fingerprint is not an identity claim);
- **same material spelled `xpub` and `zpub`** → REFUSE both — the case that proves identity is the
  material and not the base58 string, on both sides.

**§4.5's promotion narrowing is correctly SCOPED.** The failure mode a naive port has is firing the
`tpub` ruling on a descriptor. It does not:

```
p4-bare-tpub                    dev=Unknown     host=REFUSE   ok
p4-origin-tpub-49h              dev=Unknown     host=REFUSE   ok
p4-origin-tpub-44h              dev=Unknown     host=REFUSE   ok
p4-desc-with-tpub               dev=Descriptor  host=ADMIT    ok   <-- the narrowing must NOT fire
p4-bare-tpub-children           dev=Unknown     host=REFUSE   ok
p4-json-bare-tpub               dev=Unknown     host=REFUSE   ok
c5-all-tpub  (2-of-2 all tpub)  dev=Descriptor  host=ADMIT    ok
```

**Branch-4 detection (`promotesABareKey`, `:142`) — the brief's two questions, answered.**

1. *Can a FULL descriptor satisfy it?* No, and the implementer's argument checks out against the
   real cascade: `bip380.ParseKey(nil, whole record)` runs `ParseExtendedKey` on the head, which is
   base58check — a branch-2 string carries `(`, a branch-1 string carries `": "`, a branch-3 string
   is JSON, and none of the three is base58. Measured over all 72 corpus rows plus the 81-case
   battery plus the 213-case sweep: no record was ever both promoted-detected and non-branch-4.
   Over-broad detection would also be harmless — `OutputDescriptor` fails first when no promotable
   path matches, so the arm has already returned false.
2. *Can a bare key evade it with whitespace?* No. `classifyConstellation` trims **before** calling
   `isDescriptorRecord` (`sysw/classify.go:37`), so `promotesABareKey` sees the trimmed string. (The
   trim is C2's subject for a different reason.)

---

## 3 — C2 (Critical): `TrimSpace` is Unicode, `normalise` is ASCII

**Mechanism.** `classifyConstellation` (`sysw/classify.go:37`) does
`record = strings.TrimSpace(record)`, and `strings.TrimSpace` uses `unicode.IsSpace`. The host's
`cascade::normalise` (`crates/me-cli/src/descriptor/cascade.rs:36`) trims
`|c: char| c.is_ascii_whitespace()`. The two sets differ by `\v` (U+000B), U+0085, U+00A0, and the
whole Unicode `Zs` category.

**Measurement** (clean-escape run; `-desc` = the multipath canonical, `-bare` = the bare `xpub` row):

```
ws-lead-sp-desc            Descriptor  ADMIT
ws-lead-tab-desc           Descriptor  ADMIT
ws-lead-ff-desc            Descriptor  ADMIT
ws-lead-cr-desc            Descriptor  ADMIT
ws-lead-nl-desc            Descriptor  ADMIT
ws-lead-vt-desc            Descriptor  REFUSE   <<< DIVERGE   (U+000B)
ws-lead-nbsp-desc          Descriptor  REFUSE   <<< DIVERGE   (U+00A0)
ws-lead-nel-desc           Descriptor  REFUSE   <<< DIVERGE   (U+0085)
ws-lead-emsp-desc          Descriptor  REFUSE   <<< DIVERGE   (U+2003)
ws-lead-ideosp-desc        Descriptor  REFUSE   <<< DIVERGE   (U+3000)
```

Identical results for the trailing position and for the bare-key form — **20 diverging cases**, and
the five ASCII forms are clean controls that agree.

**Direction: device WIDER than host.** This is the r1-C3 direction the whole arm exists to close —
a record `me` refuses at the desk classifies `ClassDescriptor`, is offered by `syswOffer` at
`walletPolicyFlow`'s door, and reaches a screen.

**Measured impact bound.** The content behind the padding is a legitimately §4.7-admitted wallet, so
no anyone-can-spend / `k>n` / mixed-network shape gets through this way, and `Descriptor.Encode`
cannot reach its panicking arm. `me` writes the canonical, so no `me` payload carries such a record.
The concrete consequence is I1's error path. Graded Critical under the brief's rubric (a parity break
on a constructible input, against a predicate the plan holds EXACT); the controller may re-grade with
the bound above in hand.

**Note for the fix:** the trim predates P3 and is shared by every class arm — the md1/mk1 arms rely
on it (`sysw/classify.go:34-37` says so). So the fix is likely to be in the descriptor arm alone
(normalise the way the host does before the cascade), not in `classifyConstellation`'s trim.

---

## 4 — I1 (Important): "Re-parsing cannot fail here" is false

`gui/wallet_policy.go`'s new offer carries this argument verbatim:

> "Re-parsing cannot fail here -- classification is what proved it parses, **over these exact bytes**
> (sysw/descriptor.go)"

It is not the same bytes. Classification parses `TrimSpace(record)`; the consumer parses the raw
record: `desc, err := nonstandard.OutputDescriptor([]byte(body))`, where `body` comes from
`syswSession.take` (`gui/sysw_session.go:123`), which returns `r.body` **unmodified**.

**Demonstrated on a row already in the shipped corpus** — no construction needed:

```
I3 corpus row whitespace/leading-space-bip380   sysw.Classify=ClassDescriptor   OutputDescriptor(raw)=ERR   host_admits=ADMIT
```

`whitespace/leading-space-bip380` is `host_admits: true`, `device_admits: false`, single-line, and
the derived rule *requires* it to classify `ClassDescriptor`. Its raw bytes do not re-parse. So the
state the comment calls impossible is a row the suite already asserts into existence.

**What actually happens is benign, and that is the code's credit, not the comment's:** the
implementer handled the error anyway (`showError(...); return`) with the reason "a silent nil
dereference is a worse answer to an impossible state than leaving the program". So the operator gets
"Couldn't read the wallet policy from the payload." and drops out of Wallet Policy. **No crash, no
state corruption** — `take` is read-only, so the record stays in the session.

I also checked the sharper version of this: can the raw parse *succeed* and yield a **different**
descriptor than the one classified? No. The only difference is edge whitespace; branches 1, 2 and 4
all reject it outright, and branch 3 (`encoding/json`) tolerates it and yields the identical
document. So the divergence is confined to succeed-vs-fail.

The finding is the **false claim in a load-bearing safety argument**. Its second half — "the record
is §4.7-ADMITTED, which is the stronger half" — remains true and is what keeps `Encode`'s panicking
arm unreachable (verified: `Name: my wallet` → `scanOK` at the door, `ClassUnknown` from the arm).

---

## 5 — Containment, the walk, and the fixture: all held

### 5.1 `recover()` is correctly scoped — verified in BOTH directions

The `defer`/`recover` sits in `isDescriptorRecord`'s own frame (`sysw/descriptor.go:56-60`), not in
`classifyConstellation`. Verified by injected panic under `-overlay`:

```
C1  panic injected inside admitDescriptor:
      Classify(descriptor) = 0  (ClassUnknown)   <-- caught, fails closed
      Classify(mnemonic)   = 1  (ClassMnemonic)  <-- other arms unaffected
      PASS

C2  panic injected in an EARLIER arm (before isStrictMnemonic):
      Classify(descriptor) = 5  (ClassDescriptor)
      panic: C2 injected: earlier arm            <-- NOT swallowed
      FAIL
```

**Shared state:** `isDescriptorRecord` writes no package-level state; `admittedVersions` is read-only
via `slices.Contains`; `syswSession.take`/`takeAll`/`has` are all read-only. A recovered panic yields
`ClassUnknown` and the record stays in `s.records` with that class. **No corruption path found.**

**Stack overflow (which `recover` cannot catch), checked because TinyGo runs a fixed 16 kB stack:**
`bip380.Parse` is **not recursive** — it calls `parseFunc()` a bounded number of times for at most
three nesting levels (`bip380/bip380.go:300-340`). No unbounded-depth path exists.

**Cost of the new base58 exposure, measured rather than assumed** (the arm now runs a base58check
decode over operator bytes on every record; `sysw/wire.go:59` bounds the plaintext region at 32734
bytes):

| worst-case record | `sysw.Classify` |
| --- | --- |
| one 32 700-char base58 run | 3.92 ms |
| one 16 000-char run | 1.18 ms |
| 260 × 120-char runs inside a `sortedmulti` (31 479 bytes) | 179 µs |

Base58 decode is O(n²), so a single maximal run is the worst case, and 3.9 ms on this box is not a
load-time denial of service even after the device's slowdown factor. **Not a finding.**

### 5.2 The sim walk asserts a RENDERED screen, and it can fail

`pumpUntil` waits on **frame text**, not on absence-of-crash: step (1) waits for
`"Wallet policy from where?"` and additionally requires `"FROM PAYLOAD"`; step (3) waits for
`"Engrave Descriptor"` and additionally requires `"2-of-3 multisig"` and `"Segwit (P2WSH)"`, and
asserts `"(testnet)"` is absent. Mutation-tested under `-overlay`:

| mutation | result |
| --- | --- |
| baseline | `ok seedhammer.com/gui 0.013s` |
| M1 — descriptor offer replaced by a second `ClassMDMK` offer | **FAIL** `wallet_policy_descriptor_walk_test.go:147: the Descriptor offer never drew.` |
| M2 — `isDescriptorRecord` returns false | **FAIL** both tests; `Classify(…) = 0, want ClassDescriptor` |
| M3 — offer taken but `descriptorFlow` not called | **FAIL** `:161: the walk never reached DescriptorScreen.` |

M3 is the one that matters: it proves step (3) is a real render assertion and not satisfied by step
(1). **This is not a gate that cannot fail.**

### 5.3 The fixture's provenance comment is reproducible

Ran the exact invocation in the test header against the engrave binary rebuilt at `781d10d`:

```
$ me sysw pack --no-passphrase --as descriptor --in <formats-happy/bip380-sortedmulti-multipath input> --out repro.bin
      wallet-id: 9e95257e60aacbb260129dac7b36d9f4
      digest:    9c16 bfa9 bb3b ecd4 6c3c f20f e48c 12a9
rc=0
509 bytes
672d8d2c49b6c2004c38849c7b68b6dffa8629eb6bf9ac61f6ebc1e1657c58bb  repro.bin
672d8d2c49b6c2004c38849c7b68b6dffa8629eb6bf9ac61f6ebc1e1657c58bb  gui/testdata/s2_descriptor_payload.bin
$ cmp → FIXTURE-REPRODUCED-BYTE-IDENTICAL
```

Length, sha256, wallet-id and digest all match the header's four pinned values exactly.

---

## 6 — P3.5's amendments, one by one

The plan's P3.5 names **seven** items. Five are correct, one is correct-with-a-caveat, one is missing.

| # | amendment | verdict |
| --- | --- | --- |
| 1 | §9 item 2 — "one cell executed, two declared and inert" | **TRUE.** The walk executes `admits(progWalletPolicy, ClassDescriptor)` to a rendered screen (§5.2). `progBundle`/`progMultisig` have no `syswOffer(…, ClassDescriptor, …)` call — verified by grep; records are unoffered. |
| 2 | §4.3's device clauses | **TRUE, with one caveat — see below.** |
| 3 | §4.5's promotion prose | **TRUE.** `ypub → P2SH_P2WPKH` matches the `bip380/bip380.go:449` arm exactly; "`me` still refuses a bare `ypub` promotion" and "the sysw record classifier refuses `ypub`-keyed records" both measured (`c4-bare-ypub` dev=Unknown host=REFUSE; `c4-desc-ypub` dev=Unknown host=REFUSE). |
| 4 | `cascade.rs:58-62` host comment | **TRUE.** Matches `KeyVersion::admitted()` and the F-426 arm. |
| 5 | `refusal.rs:583` message + pinned test + §6 quote | **TRUE and fully propagated on the engrave side.** `git grep "device admits exactly"` finds no stale copy in any crate or test; the only two spec hits are the amendment's own historical references. Measured live: `me` now prints `` `me` admits exactly `xpub`, `tpub`, `zpub`, `Ypub`, `Zpub`. This key is `ypub`, whose equivalent is `xpub`: sh(wpkh([4bbaa801/49h/0h/0h]xpub6C9j4…/<0;1>/*)) ``. Only one pinned copy exists (`descriptor_refusals.rs:558`); the plan's `:466` cite was stale but the right test was updated. |
| 6 | §7 requirement 3's device-column phrasing | **MISSING — I2.** |
| 7 | §4.2 defect 4's "PANICS the Go parser" | **TRUE.** `nonstandard/parse.go:140` is now `len(fp) != 4`. Measured: the corpus row and a 1-byte and a 3-byte fingerprint, both as a BlueWallet file and as a single-line record, all return cleanly — `Classify=ClassUnknown`, `OutputDescriptor=ERR`, no panic. |

**The caveat on amendment 2, which the brief asked about specifically.** The sentence
*"the device's sysw RECORD classifier holds the same five … so a `ypub`-keyed record classifies
`ClassUnknown` on the device exactly as `me` refuses it at the desk"* is **true as written**, and I
verified the whole five:

```
c4-desc-xpub  Descriptor/ADMIT    c4-desc-ypub  Unknown/REFUSE
c4-desc-tpub  Descriptor/ADMIT    c4-desc-upub  Unknown/REFUSE
c4-desc-zpub  Descriptor/ADMIT    c4-desc-vpub  Unknown/REFUSE
c4-desc-Ypub  Descriptor/ADMIT    c4-desc-Upub  Unknown/REFUSE
c4-desc-Zpub  Descriptor/ADMIT    c4-desc-Vpub  Unknown/REFUSE
```

The `Ypub`- and `Zpub`-keyed admitted records the brief asked for do classify `ClassDescriptor`. The
amendment's companion claim that the scan door now accepts six is also measured true —
`version-gap/full-origin-ypub` is `scanOK` at the door and `ClassUnknown` from the arm, which is
exactly the split the amendment describes.

What the sentence does not survive is C1: "holds the same five" is a statement about *versions*, and
the mechanism holding it also refuses records whose keys are all `xpub`. The amendment is not false;
it is true of the case it describes and silent about C1's. Recorded here rather than as a separate
finding.

### I2 (Important) — the missing amendment

Plan P3.5 lists, per r6's M1: *"§7 requirement 3's device-column phrasing (P3.3 falsifies it)"*.
The spec still reads (`design/SPEC_descriptor_input.md:1595`):

> 3. **The Rust test asserts the host column; the Go test asserts the device column.** Neither
>    implementation is ever compared to the other — both are compared to the file.

P3.3 falsified the first sentence: the Go test now reads `v.HostAdmits` at
`nonstandard/descriptor_seam_test.go:434` and `:472` and derives its whole `sysw` class rule from it.
The Go test asserts **both** columns. (Requirement 1 *was* amended at P2.7 for the neighbouring
`sysw_class` retirement, so the omission is item-specific rather than a whole-section miss, and the
second sentence — the anti-cross-comparison argument — survives intact.)

Graded Important because it is a **phase-owned plan task not delivered**, and the standing rule is
that phase-owned items do not cross their owning phase's gate. The fix is one clause.

### M1 (Minor) — a fork comment P3.5 falsified

`bip380/ypub_test.go:19` quotes the refusal verbatim as
*"the device admits exactly `xpub`, `tpub`, `zpub`, `Ypub`, `Zpub`. …"* and presents it as
"measured against the S2 `me`". P3.4 (`fe9475c`, fork) landed before P3.5 (`781d10d`, engrave)
re-subjected the message, so the quote was true when written and is now wrong — the classic
"a diff falsifies text it never touches" shape, across two repos. It is a comment, not an assertion,
so nothing is red. One-word fix on the fork side when P3.5's re-subject is propagated.

---

## 7 — Deviations, and the exit state

**Deviation 1 — `wantAddress0`/`wantAddress1` made assertable. Sound.** They were declared and never
referenced. The new read in `TestDescriptorSeamAddresses` counts the **file-level** populations
(every row carrying a non-empty `address_0`/`address_1`), which is genuinely a different question
from `wantDeviceAddr0/1`'s device-route counts, so it is not a duplicate guard. Populations unchanged
at 20/5, recounted. No unsoundness.

**Deviation 2 — the base58-run scan. Unsound as justified: see C1.** The mechanism is defensible; the
argument offered for it ("a false REFUSAL … the grammar makes impossible") is not.

**Deviation 3 — `recover()` included. Sound**, and correctly scoped (§5.1).

**Deviation 4 — the walk starts one link in. Sound**, disclosed in the test's own header, and the
link it skips (`syswLoadFlow`) is covered by `gui/chain_walk_test.go`. The fixture is real `me`
output either way (§5.3).

**Deviation 5 — `goprobe/go.mod` untouched. Confirmed harmless** for the engrave suite: no Rust test
invokes it (one doc-comment mention at `crates/me-cli/tests/descriptor_as.rs:781`).

**Deviation 6 — P3.5 not in the implementer's brief.** Correct; the controller folded it, and I2/M1
are findings against that fold, not the implementer's.

### Gate spot-check (one, as permitted)

```
$ go test ./nonstandard/ ./sysw/ ./bip380/ -count=1
ok  	seedhammer.com/nonstandard	0.049s
ok  	seedhammer.com/sysw	0.063s
ok  	seedhammer.com/bip380	0.002s
```

Plus the two walk tests at baseline: `ok seedhammer.com/gui 0.013s`.

### Exit state — both worktrees byte-identical

```
fork    /scratch/code/shibboleth/sh-worktrees/s2-descriptor-arm      HEAD=fe9475c   git status --porcelain: (empty)
engrave /scratch/code/shibboleth/me-worktrees/impl-descriptor-s2     HEAD=0898cc3   git status --porcelain: (empty)
main    /scratch/code/shibboleth/mnemonic-engrave                    HEAD=4b2e557   only new file: this report, untracked, as briefed
```

**The engrave branch moved under the review, and it does not invalidate it.** At dispatch its tip
was `781d10d`; the controller then landed `88cf301` (P4.1's `scripts/f423-fit-measure` probe) and
`0898cc3` (its report). `git diff --stat 781d10d..0898cc3 -- crates/ design/SPEC_descriptor_input.md`
is **empty** — every file in this review's scope is byte-unchanged, and the `me` binary the host
probe and the fixture reproduction were built from carries those same sources. The main repo's two
new commits (`efbf217`, `4b2e557`) are continuity records. Neither worktree was touched by me.

Every probe, mutation and rebuild ran in the scratchpad or through `go test -overlay`. Nothing was
pushed. The engrave `target/debug/me` was rebuilt from unmodified sources at `781d10d`, which is a
build artefact and not tracked.
