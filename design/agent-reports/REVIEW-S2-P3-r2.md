# REVIEW-S2-P3-r2 — did the fold fix r1's findings without introducing a defect?

**Phase:** P3 of `IMPLEMENTATION_PLAN_descriptor_input_S2.md`. Round 2, scoped to the fold.
**Reviewer:** independent agent, opus tier. Author ≠ reviewer.
**Round 1:** `design/agent-reports/REVIEW-S2-P3-r1.md` @ main `cbee2e0` — RED, 2C/2I/1M.

**Targets — the fold, and nothing else:**

| target | range | worktree |
| --- | --- | --- |
| fork `s2/descriptor-arm` (C1+C2, then I1+M1) | `fe9475c..0f92554` | `/scratch/code/shibboleth/sh-worktrees/s2-descriptor-arm` |
| engrave — the controller's I2 clause | `git show 36fd0c3` | `/scratch/code/shibboleth/me-worktrees/impl-descriptor-s2` |
| implementer's fold record | `IMPL-S2-P3.md` ADDENDUM, committed `9be6bfc` | (same) |

**Nothing modified. Nothing pushed. Both worktrees byte-identical at exit — verified, §8.**

## Verdict

**GREEN — 0 Critical / 0 Important / 2 Minor / 1 Nit.**

All five r1 findings are resolved. Across **187 constructed cases** run through a Rust host probe
(`descriptor::host_admits`, the same function `descriptor_seam.rs:595` asserts) and a Go device probe
(the real `sysw.Classify`), there are **0 verdict divergences** — and on the two sharper questions the
brief asked, `asciiNormalise` reproduces the Rust `normalise` **byte-for-byte on all 187**, and the
multiset of extended-key versions the device SCANS equals the multiset the host PARSED on every case
where the host's cascade parsed.

| # | sev | one line |
| --- | --- | --- |
| M1 | Minor | §5.2's "the arm is … composed: parse, then narrowings, then conjuncts" now omits the §4.6 step the arm has since C2's fold. Doc-consistency; the shipped code is correct. |
| M2 | Minor | The C2 guard makes the device NARROWER than the host on an **interior CRLF** — measurable, fail-closed, and unreachable as a record (`splitRecords` splits on LF, `sysw/open.go:74`). |
| N1 | Nit | `blueWalletKeyValues` does not reproduce the host's `seenKeys` dedup, so a duplicated cosigner line is scanned twice. Proved verdict-neutral (§3.4). |

**The fold does more than r1 asked for, and I measured it:** the scoping fix also closes an
**unnamed device-WIDER break** — a `ypub` hidden inside a JSON `descriptor` field by a `y`
escape, which the pre-fold whole-record scan could not see at all (§3.3).

---

## 0 — Method

Two probes, both in the scratchpad, neither touching a worktree.

- **host probe** — Rust crate, path dependency on `<engrave>/crates/me-cli`, calling
  `descriptor::admit::{host_admits, format_of}`, `descriptor::cascade::{normalise, cascade}`.
  Built with `CARGO_TARGET_DIR` pointed at the worktree's gitignored `target/`.
  It reports, per case: the admission verdict, the winning branch, the **hex of `normalise(input)`**,
  and **the version byte of every key the cascade actually parsed** (`Parsed.keys[].version`).
- **device probe** — Go module, `replace seedhammer.com => <fork worktree>`, toolchain
  `/nix/store/33fw…-go-1.26.3/bin` (`go1.26.3 linux/amd64`). It reports `sysw.Classify`, the scan
  door, the hex of `asciiNormalise(raw)`, whether `cascadeKeyText` narrowed, and the versions
  `eachExtendedKey(cascadeKeyText(trimmed))` yields.

The unexported functions are reached through a **probe-only export file injected with
`go build -overlay`** — it forwards to the real `cascadeKeyText` / `asciiNormalise` /
`blueWalletKeyValues` / `eachExtendedKey` / `promotesABareKey` and reimplements nothing. The overlay
never writes to the worktree; `git status --porcelain` was empty after every run.

Both probes read **hex-encoded** input lines, so no escape layer can corrupt a control character —
r1 recorded four false divergences from exactly that, and this run inherits the fix.

**Cases:** 187 = 24 branch-order/scoping (A) + 56 whitespace (B) + 72 shipped corpus rows (C, of
which 59 single-line) + 26 sharper scoping (D) + 9 duplicate-header (E) + 6 CRLF residual (F).

---

## 1 — Fold vs findings

| r1 | remedy | where | verdict |
| --- | --- | --- | --- |
| **C1** — the §4.3 scan read the whole record, so a JSON `label` holding a non-admitted key refused an admitted record | `cascadeKeyText` scopes the scan to what the cascade consumed: branch 3 → the `descriptor` field, branch 1 → the values of unrecognised headers, branches 2/4 → the whole record. JSON tested before the header shape. | fork `a785755` | **RESOLVED**, §3 |
| **C2** — `strings.TrimSpace` (Unicode) is wider than the host's `is_ascii_whitespace` | `classifyConstellation` keeps `raw`; the arm refuses unless `asciiNormalise(raw) == record`. `asciiWhitespace = " \t\n\f\r"`, U+000B deliberately absent. | fork `a785755` | **RESOLVED**, §4 |
| **I1** — "Re-parsing cannot fail here … over these exact bytes" was false | the consumer parses `strings.TrimSpace(body)`; the comment is rewritten to the argument that is true; a new **walk** asserts a rendered screen on the leading-space record. | fork `0f92554` | **RESOLVED**, §5 |
| **M1** — `bip380/ypub_test.go` quoted the pre-P3.5 refusal text | re-quoted and re-measured, with a line recording why it moved. | fork `0f92554` | **RESOLVED**, §6 |
| **I2** — §7 requirement 3's device-column phrasing, falsified by P3.3 | one clause: both tests also assert the derived rule, computed from the host column. | engrave `36fd0c3` | **RESOLVED**, §7 |

---

## 2 — The headline measurement

```
$ hostprobe < cases.txt > host.tsv ; devprobe < cases.txt > dev.tsv     # 187 cases
cases: 187
verdict divergences:                                     0
normalise() divergences (asciiNormalise vs Rust normalise): 0
key-version-multiset divergences (host-parsed vs device-scanned): 0   (5 benign, N1)
consumer-string divergences (TrimSpace(body) != classified string):  0
single-line corpus rows checked:                        59
```

The three questions are independent, and the second and third are the ones that bite: a verdict
match can be luck on a small corpus, but `asciiNormalise(raw) == normalise(input)` **byte-for-byte**
is the whole of C2, and "the device scans exactly the versions the host parsed" is the whole of C1.

---

## 3 — C1: the scan's scope

### 3.1 Branch-order parity, the brief's first pressure point

`cascadeKeyText` tests **JSON first**; the real cascade is **1 BlueWallet → 2 BIP-380 → 3 JSON →
4 promoted key** (`nonstandard/parse.go:36-73`, `cascade.rs:560`). I looked for an input on which the
two orders pick different scopes. **There is none, and the reason is structural:**

- **Branch 2 can never be JSON.** `parseFunc` takes everything before the first `(` as the script
  name and requires it to be exactly `wsh`/`pkh`/`sh`/`wpkh`/`tr` (`bip380/bip380.go:281-302`,
  `cascade.rs:781`). A JSON object begins `{`. Measured: `A12-json-array`, `A13-json-null`,
  `D17-json-nested-json` — no case in 187 was both.
- **Branch 1 can never be JSON.** Every non-empty non-`#` line must split on `": "` with a key of
  `Name`/`Policy`/`Derivation`/`Format`/8-hex. A JSON object's first line starts `{"…`, so the key
  carries `{`; JSON strings cannot span raw newlines, so no later line can start with `Name`.
  Measured on `A08-json-spaced-colon` — a JSON record deliberately spaced so it *is* header-shaped —
  host `json`, device `ClassDescriptor`, scope `narrowed`. The implementer's stated reason for the
  order is exactly this case, and it holds.
- **Branch 4 can never be JSON** (`json.Unmarshal` into a struct succeeds only for an object or
  `null`; a key expression is neither), **and can never be header-shaped** — `A14-headerish-bare-key`,
  `D15-bare-key-with-colon`, `D16-header-shaped-descriptor`, `A15-bip380-with-colon-space`: all four
  fail the parse on **both** sides.
- The **reverse** direction cannot happen either: `cascadeKeyText`'s branch-3 test is *character for
  character* the same two conditions as `OutputDescriptor`'s branch 3 (`json.Unmarshal` into the same
  anonymous struct with the same tags, then `bip380.Parse` of the `descriptor` field), so a record the
  cascade admitted on branch 3 is never scoped anywhere else.

And the order is only reachable at all after a successful parse — `isDescriptorRecord` runs
`OutputDescriptor` **before** `cascadeKeyText`, so records where branch 3 claims-and-fails
(`A11-json-bad-descriptor`) never reach the scoping at all.

### 3.2 The sharper check: the version multisets agree

Rather than infer the branch, I measured what each side actually looked at:

```
host: cascade(normalise(input)).keys[].version         # every key parse_extended_key saw
dev : eachExtendedKey(cascadeKeyText(TrimSpace(raw)))  # every version the scan checks
```

Over all 187 cases, on **every case where the host's cascade parsed**, the two multisets are equal
(five duplicate-only exceptions, N1). Selected rows:

```
case                             host    fmt          dev  scope     host-KV                    dev-KV
A01-json-label-ypub              ADMIT   json         5    narrowed  0488b21e,0488b21e,0488b21e 0488b21e,0488b21e,0488b21e
A02-json-label-ypub-twin         ADMIT   json         5    narrowed  0488b21e                   0488b21e
A03-json-desc-ypub               REFUSE  none         0    narrowed  -                          049d7cb2
A05-json-UPPER-fields-ypub-lbl   ADMIT   json         5    narrowed  0488b21e,0488b21e,0488b21e 0488b21e,0488b21e,0488b21e
A06-json-extra-field-ypub        ADMIT   json         5    narrowed  0488b21e,0488b21e,0488b21e 0488b21e,0488b21e,0488b21e
A21-bw-name-is-ypub              ADMIT   bluewallet   5    narrowed  0488b21e x3                0488b21e x3
A22-bw-cosigner-ypub             REFUSE  none         0    narrowed  -                          049d7cb2,0488b21e,0488b21e
D03-json-nested-ypub             ADMIT   json         5    narrowed  0488b21e x3                0488b21e x3
D08-bw-ypub-in-comment           ADMIT   bluewallet   5    narrowed  0488b21e x3                0488b21e x3
D21-json-ypub-as-key             ADMIT   json         5    narrowed  0488b21e x3                0488b21e x3
```

`A05` is a case neither r1 nor the implementer named: **`{"LABEL":…,"DESCRIPTOR":…}`**. Go's
`encoding/json` matches field names case-insensitively and `cascade.rs:892` reproduces that
deliberately — and because `cascadeKeyText` uses the same `json.Unmarshal`, the scoping follows
automatically. `A22` and `E02`/`E06` are the refusing controls: the branch-1 arm **narrowed** the scan
without **disabling** it — a `ypub` in an actual cosigner slot is still refused on both sides.

### 3.3 The fold closes a device-WIDER break r1 did not name

`D05` is `{"label":"w","descriptor":"sh(wpkh([4bbaa801/49h/0h/0h]ypub6Wyz…/<0;1>/*))"}` — a
`ypub` **inside the descriptor field**, with its leading `y` written as a JSON escape. The raw record
then contains no ≥ 100-character base58 run for that key at all (`0` is not in the alphabet), so the
pre-fold whole-record scan could not see it:

```
$ devprobeB < cases2.txt | grep D05      # devprobeB = the scan reverted to the whole record
D05-json-escaped-ypub-in-desc	5        # ClassDescriptor  <-- device WIDER than the host
$ devprobe  < cases2.txt | grep D05      # shipped
D05-json-escaped-ypub-in-desc	0        # ClassUnknown
$ grep D05 host2.tsv
D05-json-escaped-ypub-in-desc	REFUSE	none
```

Scanning the **decoded** `descriptor` field is what catches it. This is the r1-C3 direction — a wallet
`me` refuses reaching a screen — and it was live before the fold. It strengthens the remedy choice
independently of the argument the implementer gave for it.

### 3.4 The declined remedy — the counterexample is real, and I verified the key material

r1 offered "match each run's key MATERIAL against the parsed keys". The implementer declined and
built `TestJSONLabelHoldingTheYpubTwinOfItsOwnKeyIsStillNotKeyMaterial`
(`sysw/descriptor_parity_test.go:57`). I did not implement the wrong fix; I checked the claim it rests
on — that `skYL` is the `ypub` spelling of `skXP`:

```
xpub version 0488b21e   ypub version 049d7cb2
chain_code equal: True
key_data   equal: True
identity (key_data, chain_code) IDENTICAL: True
```

`identity()` is `(key_data, chain_code)` (`cascade.rs:254`), so the label's run **would** match the
descriptor's own key under the suggested remedy, be version-checked, and be refused — while the host
ADMITS the record (measured, `A02`: `host=ADMIT fmt=json`). **The decline is correct and the test
reds under the suggested remedy's logic.** It passes under the shipped one (§ below).

### 3.5 The branch-1 residual r1 asked about

`Name:` holding an extended key, both shapes:

```
A18-bw-single-name-plain   `Name: my wallet`   host=REFUSE  dev=ClassUnknown  (scanOK at the door)
A17-bw-single-name-ypub    `Name: <ypub>`      host=REFUSE  dev=ClassUnknown
A21-bw-name-is-ypub        the multi-line BlueWallet fixture with Name=<ypub>
                                               host=ADMIT   dev=ClassDescriptor
```

The host's verdict and the arm's now match on every one. (A single-line branch-1 record can only
carry ONE header, so it always has zero keys and `admitDescriptor` refuses it regardless — the
branch-1 scoping is exercised only by multi-line inputs, which is what the shipped test asserts and
what the test's own comment says.)

---

## 4 — C2: `asciiNormalise` against the host's `normalise`, character by character

Rust: `input.replace("\r\n","\n").trim_matches(|c| c.is_ascii_whitespace())` (`cascade.rs:36`).
Go: `strings.Trim(strings.ReplaceAll(s,"\r\n","\n"), " \t\n\f\r")`.
`char::is_ascii_whitespace` is exactly `' ' | '\t' | '\n' | '\x0C' | '\r'`; the cutset is those five
bytes and `strings.Trim` on an all-ASCII cutset is byte-wise, so the two are the same function.

**Measured rather than argued** — the probes print the hex of both normalisations, and they are equal
on **all 187 cases**, including every edge the brief named:

```
B-lead/trail-{sp,tab,ff,cr,lf}-{desc,bare}   20 cases   host=ADMIT   dev=ClassDescriptor   (controls)
B-lead/trail-{vt,nel,nbsp,emsp,ideo}-…       20 cases   host=REFUSE  dev=ClassUnknown      (the finding)
B-lone-cr-trailing        canon + "\r"                  host=ADMIT   dev=ClassDescriptor
B-lone-cr-interior        "wsh(sortedmulti(2,\r…))"     host=REFUSE  dev=ClassUnknown
B-crlf-trailing / -leading                              host=ADMIT   dev=ClassDescriptor
B-crcrlf-leading          "\r\r\n" + canon              host=ADMIT   dev=ClassDescriptor
B-mixed-edges             " \t" + canon + "\n\r "       host=ADMIT   dev=ClassDescriptor
B-vt-then-space / B-space-then-vt / B-cr-lf-vt-trail    host=REFUSE  dev=ClassUnknown
B-vt-interior / B-nbsp-interior                         host=REFUSE  dev=ClassUnknown
B-json-padded-sp                                        host=ADMIT   dev=ClassDescriptor
B-json-padded-nbsp / B-bw-padded-vt                     host=REFUSE  dev=ClassUnknown
```

**The host does refuse U+000B-padded input**, which the brief asked to confirm: `me`'s `normalise`
leaves the vertical tab in place, so `parse_func` reads the script name as `\vwsh` and no branch
claims the input. All four VT cases, both positions, both forms: `host=REFUSE`.

The **invalid-UTF-8** edge where `strings.TrimSpace` (rune-based, and its own fast path includes
`\v`) could part company with `strings.Trim` (byte-based ASCII set) is unreachable: `splitRecords`
rejects a non-UTF-8 payload before any record exists (`sysw/open.go:68`).

The guard's shape is what makes this hold in general rather than case by case: it refuses unless
`asciiNormalise(raw) == TrimSpace(raw)`, so when it passes, the string the arm parses **is**
`normalise(input)`. There is no remaining input on which the device parses a different string from
the host.

---

## 5 — I1: the consumer now parses the string classification proved

`classifyConstellation` computes `record = TrimSpace(raw)` and the arm answers for that string;
`syswSession.take` returns `r.body` unmodified and `Classify(r)` was called on the same `r`
(`gui/sysw_session.go:110-115`). So `strings.TrimSpace(body)` **is** the classified string, by
construction and not by assertion. Measured over all 187 cases as its own column:

```
consumer-string divergences (TrimSpace(body) != classified string): 0
```

**`walletPolicyFlow` is the only live consumer** — the one `syswOffer(…, ClassDescriptor, …)` in the
tree (`gui/wallet_policy.go:43`); `progBundle`/`progMultisig` declare the cell and never offer, and
`gui/transaction.go:286` is a display-name switch. `gui/scan.go:87` is the QR path, not a record.

The addendum's claim that the leading-space corpus row now renders is **true, and the walk can fail:**

```
$ go test ./gui/ -run TestWalkWalletPolicy -count=1 -v
--- PASS: TestWalkWalletPolicyFromAPackedDescriptorRecordToTheDescriptorScreen
--- PASS: TestWalkWalletPolicyRendersARecordWithLeadingWhitespace

$ go test -overlay=…C.json ./gui/ -run TestWalkWalletPolicy -count=1   # consumer parses the RAW body
--- FAIL: TestWalkWalletPolicyRendersARecordWithLeadingWhitespace
    wallet_policy_descriptor_walk_test.go:219: a record the classifier ADMITTED did not reach the screen.
        Last frame: "Couldn'treadthewalletpolicyfromthepayload.WalletPolicy"
```

It fails on the frame the operator would actually have seen, not on an internal value.

---

## 6 — M1: the quote, re-measured live

```
$ me sysw pack --no-passphrase --as descriptor --in <the bare ypub descriptor> --out /dev/null
me: `me` admits exactly `xpub`, `tpub`, `zpub`, `Ypub`, `Zpub`. This key is `ypub`, whose
equivalent is `xpub`: sh(wpkh([4bbaa801/49h/0h/0h]xpub6C9j4wAxx…/<0;1>/*))
rc=3
```

`bip380/ypub_test.go:19-21` now quotes this verbatim. Tree-wide sweep for the retired subject:
`grep -rn "device admits exactly"` finds it only in persisted agent reports (historical, correct to
leave), in the plan's amendment task list, and in `SPEC:55`'s own historical reference to what P3.5
changed. **No live refusal text, test pin or code comment carries it.**

---

## 7 — I2: does the amended sentence describe both sides truthfully?

`36fd0c3` amends `SPEC_descriptor_input.md:1595`. Checked clause by clause against the code:

| clause | evidence | true? |
| --- | --- | --- |
| "the Rust test asserts the host column" | `descriptor_seam.rs:589 the_host_column_matches_the_admission_predicate` | ✓ |
| "the Go test asserts the device column" | `descriptor_seam_test.go:122 TestDescriptorSeamDeviceColumn` reads `v.DeviceAdmits` | ✓ |
| "BOTH tests also assert the derived classification rule, COMPUTED from the host column" | Rust `every_single_line_input_classifies_by_the_admission_column` + `a_canonical_descriptor_classifies_as_a_descriptor_record`; Go `TestDescriptorSeamSyswClass:426` + `TestDescriptorSeamSyswClassCanonical:468` | ✓ |
| "`classify(input) == Descriptor` iff `host_admits` for single-line rows" | both sides, exact equality, both directions | ✓ |
| "`classify(canonical) == Descriptor` on admitted rows" | Go: exhaustive (`if !v.HostAdmits { continue }`, `:472`). Rust: by example, one row | ✓ as a statement of the RULE; see below |
| "so the Go seam test now reads `host_admits`, as data" | `:434`, `:445`, `:472` | ✓ |
| "Neither implementation is ever compared to the other" | survives — both read the file | ✓ |

The one imprecision is that the Rust half asserts the canonical clause on a single named row while
the Go half asserts it exhaustively. The sentence states what the *rule* is and that both tests assert
it, which is true; it makes no coverage claim. Not a finding.

---

## 8 — What the fold falsified elsewhere, and the residuals

### M1 (Minor) — §5.2's composition sentence has not caught up with the arm

`SPEC_descriptor_input.md` §5.2 says:

> The arm is the PREDICATE below, composed: parse via `nonstandard.OutputDescriptor`, then the
> single-line-reachable narrowings of §4's cascade …, then §4.7's conjuncts over the parsed
> descriptor.

Since C2's fold the arm has a **fourth step, and it runs first**: §4.6's ASCII normalisation guard.
Someone implementing from that sentence today would rebuild C2. Mitigating, and why this is Minor
rather than Important: §5.2 also says the predicate "is stated once, and both sides implement it",
and the Rust side is `host_admits`, which is `admit(cascade(normalise(input)))` — so the obligation
is derivable, and the shipped code is correct. The §4.6 section is normative but its subject is `me`,
so it does not by itself bind the device. **One clause, owned by the next spec-touching phase**;
P3.5's amendment batch is closed and this is not one of its seven items.

### M2 (Minor) — the guard makes the device narrower on an interior CRLF

The guard refuses whenever `asciiNormalise(raw) != TrimSpace(raw)`. `ReplaceAll("\r\n","\n")` can
change the string in the **interior**, where `TrimSpace` never reaches:

```
F01-json-fixture-lf     host=ADMIT  fmt=json        dev=ClassDescriptor
F02-json-fixture-crlf   host=ADMIT  fmt=json        dev=ClassUnknown     <<< the device is narrower
F03-bw-fixture-lf       host=ADMIT  fmt=bluewallet  dev=ClassDescriptor
F04-bw-fixture-crlf     host=ADMIT  fmt=bluewallet  dev=ClassUnknown     <<< same
F05-canon-trailing-crlf host=ADMIT  fmt=bip380      dev=ClassDescriptor  (edge CRLF is fine)
```

This is **new with the fold** — before it, `TrimSpace` left the interior CRLF alone and Go's JSON
parser tolerates it, so F02 agreed. Why it does not gate:

1. **It is unreachable as a record.** `\r\n` contains an LF, and `splitRecords` splits the public
   section on `"\n"` (`sysw/open.go:74`), so no record can carry one. Only a whole *file* handed to
   `me` can, and `me` packs the canonical.
2. **The direction is fail-closed.** §7's normative invariant is `host_admits(input) ⇒
   device_admits(canonical(input))`, and `canonical` is single-line — untouched. No wallet the host
   refuses reaches a screen.
3. **It is disclosed in the arm's own comment** (`sysw/descriptor.go:107-113`), which states the
   record-cannot-contain-a-newline argument explicitly, and the shipped
   `TestInteriorCRLFClassifiesUnknown` pins the behaviour deliberately.
4. The gate is stated over single-line rows, and the shipped corpus's own CRLF row
   (`whitespace/crlf-bip380`) is a **trailing** CRLF, which agrees on both sides.

Recorded so the residual is on the record rather than rediscovered.

### N1 (Nit) — the scan multiset can carry a duplicate the host deduplicated

`parseBlueWalletDescriptor` keeps `seenKeys` and `continue`s on a repeated header name whose value is
identical (`nonstandard/parse.go:92-96`); `blueWalletKeyValues` has no such dedup. Measured:

```
E01-bw-dup-cosigner-same    host=ADMIT   dev=ClassDescriptor   host-KV 3 keys   dev-KV 4 runs
```

Verdict-neutral, and provably so: the device's multiset is a superset whose extra elements are values
**identical** to ones already present, hence identical versions, hence the same
`keyVersionsAdmitted` boolean. The only other case — a repeat with a *different* value — makes the
host error (`inconsistent header value`) and the whole cascade fail, so both sides refuse
(`E02-bw-dup-cosigner-diff-ypub`, `E03-…-diff-xpub`, `E04-bw-dup-name-diff`: all `host=REFUSE
dev=ClassUnknown`).

### Nit, recorded without a number — the two table tests have no ran-count assertion

`TestASCIIEdgeWhitespaceStillClassifies` and `TestNonASCIIEdgeWhitespaceClassifiesUnknown` drive 20
assertions each through `eachPadding` with no counter; an emptied `asciiEdge`/`nonASCIIEdge` literal
would pass silently. Their Rust counterpart guards exactly this (`assert_eq!(host_checked, POP.rows)`,
PLAN-r1's I7). Mutation A below proves they run **today**; this is a durability gap, not a live
defect.

### Checked and clean

- `grep -rn "whole record\|entire record"` in the fork: the three hits are all inside the fold's own
  new comments explaining *why* the whole record was wrong. No stale claim survives.
- `grep -rn "exact bytes"`: no stale copy of I1's false safety argument; the only hit in scope is the
  new walk test quoting the retired claim to explain itself.
- `Descriptor.encode` in the new `gui/wallet_policy.go:71` is the **correct** name — the panicking
  default arm is `func (d *Descriptor) encode(compact bool)` at `bip380/bip380.go:193`, not the
  exported `Encode()` wrapper r1 quoted.
- §4.3's P3.5 device clause ("the device's sysw RECORD classifier holds the same five … so a
  `ypub`-keyed record classifies `ClassUnknown`") is still true, and r1's recorded caveat on it —
  that the mechanism also refused all-`xpub` records — is now **retired by C1's fix**.
- IMPL-S2-P3's original Deviation 2 is left standing in the body and **retired explicitly in the
  addendum**, which is the right shape for an append-only record.

---

## 9 — The implementer's claims, spot-checked

### Mutations (3 of the 5 reproduced, including the briefed one)

```
$ go test ./sysw/ -count=1                                    ok  (baseline)

$ go test -overlay=…A.json ./sysw/ -count=1      # asciiWhitespace gains "\v"
--- FAIL: TestNonASCIIEdgeWhitespaceClassifiesUnknown
    descriptor_parity_test.go:165: leading  U+000B vertical-tab, descriptor: Classify = 5, want ClassUnknown
    descriptor_parity_test.go:165: trailing U+000B vertical-tab, descriptor: …
    descriptor_parity_test.go:165: leading  U+000B vertical-tab, bare key:   …
    descriptor_parity_test.go:165: trailing U+000B vertical-tab, bare key:   …
```

**Exactly the 4 cases the implementer reported, and no others** — the one-character mutation is
precisely detected.

```
$ go test -overlay=…B.json ./sysw/ -count=1      # scan reverted to the whole record
--- FAIL: TestJSONLabelIsNotKeyMaterial
--- FAIL: TestJSONLabelHoldingTheYpubTwinOfItsOwnKeyIsStillNotKeyMaterial
--- FAIL: TestBlueWalletNameHoldingAnExtendedKeyIsNotKeyMaterial
```

(The implementer split this into two mutations — the scoping and the branch-1 arm — and reverting the
whole scoping reds all three, which is consistent with both rows.)

Mutation C (the consumer parses the raw body) is in §5. All mutations were applied with
`go test -overlay` / `go build -overlay`; `git status --porcelain` was empty after every run.

### The 54-case probe

The implementer's "22 divergences before, 0 after" is not directly reproducible from the report (the
case list is not in it), but its **conclusion is independently confirmed at 3.5× the case count**:
187 cases, 0 verdict divergences, plus the two structural equalities in §2 that the 54-case probe did
not claim.

### Suites — re-run, not taken on trust

```
$ go test -count=1 $(go list ./... | grep -v "/gui$")
  52 packages ok, 0 FAIL

$ bash scripts/gui-shard-test.sh ./gui/ 24
  1009 top-level tests
  partition verified exhaustive: 1009 == 1009
  === wall: 67s ===
  RESULT: ok -- all 1009 tests ran across 24 shards
```

1008 → 1009 is the new I1 walk, as the addendum says. `gofmt -l` on all six fold-touched files:
empty. `go vet ./sysw/ ./gui/ ./bip380/ ./nonstandard/`: two findings, both pre-existing
`testing.ArtifactDir requires go1.26` on `gui` golden-test files the fold never touched.

**The engrave Rust suite was not re-run, and deliberately.** `git diff --stat 781d10d..HEAD --
crates/` is **empty** — `36fd0c3` and `9be6bfc` touch `design/` only, so no Rust code moved since
r1's target. The shared vector file is byte-identical across the repos at the pinned digest:

```
e7a4160ce064a6cb7ca31dc530e079c861cf2c8a075d75f793ef0d935f583758  <fork>/nonstandard/testdata/descriptor_seam_vectors.json
e7a4160ce064a6cb7ca31dc530e079c861cf2c8a075d75f793ef0d935f583758  <engrave>/crates/me-cli/testdata/descriptor_seam_vectors.json
```

---

## 10 — Exit state

```
fork     /scratch/code/shibboleth/sh-worktrees/s2-descriptor-arm    HEAD=0f92554   git status --porcelain: (empty)
engrave  /scratch/code/shibboleth/me-worktrees/impl-descriptor-s2   HEAD=9be6bfc   git status --porcelain: (empty)
main     /scratch/code/shibboleth/mnemonic-engrave                  HEAD=ffa9cbb   only new file: this report, untracked
```

**The main repo moved under the review and it does not affect the scope:** at dispatch its tip was
`cbee2e0`; `ffa9cbb` is a continuity record touching only
`design/CONTINUITY_2026-08-29-s2.md` (`git diff --stat cbee2e0..ffa9cbb` — 1 file, +11/−8).
Neither reviewed worktree was touched by me. Every probe, mutation and build ran in the scratchpad or
through `-overlay`; the Rust probe used the engrave worktree's gitignored `target/` for its
dependency cache and produced no tracked artefact. Nothing was pushed.

**GREEN — 0 Critical / 0 Important. The P3 review loop closes.** The two Minors and the Nits are
recorded for the follow-up ledger; none of them binds this gate.
