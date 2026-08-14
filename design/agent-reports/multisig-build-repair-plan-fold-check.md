# Fold check — IMPLEMENTATION_PLAN_multisig_build_repair.md, `e051dae..3748cc6`

**Verdict: NOT clean.** The fold's 7 checkable factual claims are all TRUE — verified
independently against `bitcoin/bips@master` and both CLIs, not read from the audit
report. But the fold is internally inconsistent: it corrected the description/gate
text in two places while leaving an **adjacent, unedited instruction** stating the old,
now-contradicted fact — once at Critical severity (an unbuildable gate), once at
Important (a false BIP citation re-asserted four lines after the fold explicitly
name-checks the paragraph containing it).

Reviewer: independent context, 2026-08-13. Fork checked out at `a10d00795be4`
(matches the brief). Both repos left clean (`git status --short` empty in
`seedhammer` and `mnemonic-engrave`); all scratch artifacts (BIP mediawiki fetches,
Go harness, mk-verify chunks) removed from `/tmp/.../scratchpad`, none committed.

---

## 1. The 7 checkable claims — command run, result

| # | claim | command | result |
| --- | --- | --- | --- |
| 1 | BIP-383 (not 382) has `wsh(multi(…))`/`wsh(sortedmulti(…))` vectors, output is scriptPubKey not address | `curl -s raw.githubusercontent.com/bitcoin/bips/master/bip-0383.mediawiki` → `grep -c 'multi('` / read Test Vectors section | **TRUE.** 23 `multi(` hits, 11 `sortedmulti(`. Section reads "Valid descriptors followed by **the scripts they produce**"; every `**` line is raw script hex (`0020…`, `5221…`, `a914…`), zero address strings (`grep -ci address` on the file → 0) |
| 2 | BIP-382 contains no `multi(` | `curl -s .../bip-0382.mediawiki \| grep -c 'multi('` | **TRUE.** 0. Title is *Segwit Output Script Descriptors*; only `wpkh()`/`wsh(pk())`/`wsh(pkh())` vectors, also scriptPubKey hex, also 0 address strings (checked as a bonus — relevant to a finding below) |
| 3 | BIP-141 has a P2SH-P2WSH Example section giving scriptPubKey + redeemScript (address derivable, not quoted) | `curl -s .../bip-0141.mediawiki`, located `=== P2WSH nested in BIP16 P2SH ===` (line 218) | **TRUE.** Gives `scriptSig`/`scriptPubKey` as byte **templates** (`<32-byte-hash>`, `<20-byte-hash>` placeholders, no concrete key material) — consistent with "derived locally," since there is no literal address string to quote |
| 4 | BIP-32 vectors never touch `48'`; BIP-48 publishes no test vectors at all | `curl -s .../bip-0032.mediawiki \| grep -n "48'"` ; `curl -s .../bip-0048.mediawiki`, read `==Examples==` | **TRUE.** BIP-32 grep: no output (5 vector sets are `m/0H/1/2H/2/1000000000` etc., none touch 48). BIP-48's Examples table has an `address` **column** whose cells read "first"/"second" (English words), no key material anywhere in the section |
| 5 | BIP-67 publishes usable key-ordering vectors | `curl -s .../bip-0067.mediawiki`, read `==Test vectors==` | **TRUE.** 4 vectors, each with List → Sorted → Script → **Address** (concrete base58 addresses), directly usable |
| 6 | `mk verify --xpub --origin-fingerprint --origin-path --policy-id-stub` exists and runs on fork-encoded chunks; `canonical_payload_bytes` has no `mk` CLI surface | `mk verify --help`; built a Go harness (`replace seedhammer.com => .../seedhammer`, `mk.Encode` on a real depth-4 Card from `mk vectors`' own V1 xpub) → `mk decode --json` → `mk verify --xpub … --origin-fingerprint 73c5da0a --origin-path "m/48'/0'/0'/2'" --policy-id-stub 11223344 <fork chunks>`; `mk bytecode --help` | **TRUE, end-to-end.** All 4 flags exist on `mk verify`. Fork-encoded chunks (`mk1qpmcvpp…`) decoded cleanly and `mk verify` with all 4 flags returned `OK … exit=0`. `mk bytecode` → `error: unrecognized subcommand 'bytecode'` (tip suggests `encode`/`decode`); `mk --help`'s full subcommand list is encode/decode/inspect/verify/vectors/gui-schema/repair/address/derive/gen-man — no `bytecode` |
| 7 | `grep -rn TYPED-ONLY --include='*.go'` in the fork returns 9, distributed exactly as stated, none in verify flows | `grep -rn 'TYPED-ONLY' --include='*.go' .` in `seedhammer` @ `a10d007`; per-file counts; checked the 4 files that actually define `verify`-named flows (`verify_address.go`, `multisig_verify.go`, `singlesig_verify.go`, `derive_xpub.go`) for the literal string | **TRUE, exactly.** 9 hits: `gui/multisig.go` ×4 (17, 24, 60, 102), `gui/bip85.go` ×2 (264, 270), `gui/singlesig.go` ×2 (18, 32), `gui/multisig_build.go` ×1 (67). Zero hits in any of the 4 verify-flow files; those files call `seedEntryFlowTypedOnly` (the identifier, not the phrase) |

All 7 hold up under independent execution — the fold's new positive claims are not
just correctly transcribed from the audit, they are actually true.

## 2. Per-edit specification table

| edit | location | unambiguous? | if not, what must be ruled |
| --- | --- | --- | --- |
| Drift-paragraph rewrite ("coverage catch-up, not a correctness repair") | §1a, lines 59–68 | Yes | Prose rationale; the actionable deliverable (S0-4: `go test ./md/` + README provenance) is unchanged and was already unambiguous |
| mk1 relation (b) → `mk verify --xpub --origin-fingerprint --origin-path --policy-id-stub` | §1a mk1 table row, line 75 | Yes | Names an exact command; verified it runs and exits 0 on fork chunks (§1.6 above) |
| BIP table replacement (383/67/141/39; drops the 32/48 row) | §1a, lines 93–98 | Yes, on its own | Every cell checked true. But see Finding 1 below — an **untouched** deliverable 4 lines later still asserts the old, now-disproved BIP-382 claim |
| Correction paragraph (382→383, no addresses, BIP-32/48 have no `48'` vectors) | §1a, lines 100–107 | Yes | Matches source text exactly; "S0's provenance README must say so" is a documentation instruction, not a test assertion — no result-changing judgement call |
| `TestBip383WshMultiScriptPubKeyMatchesPublishedVectors` | S0 Tests-first, line 207 | Yes | Names BIP-383, asserts scriptPubKey level explicitly, matches source content |
| `TestBip141NestedSegwitScriptDiffersFromLegacy`, incl. "address is **derived locally from** that vector, not quoted" | S0 Tests-first, line 212 | **Mostly** — one soft ambiguity, does not change the boolean result | The test's own name commits to a **script**-level comparison; the "address is derived locally… S0's README must record it" clause reads as a separate instruction to the *provenance README*, not necessarily to the test body. A careful implementer gets the same PASS/FAIL either way (nested P2SH-P2WSH structurally differs from bare/legacy treatment of the same script), so this doesn't change the *result* — but the sentence would be clearer split into two bullets: one for the test, one for the README |
| S3 Gate: `grep -rn TYPED-ONLY --include='*.go'` returns 0, measured baseline 9 across 4 named files, none in verify flows | S3, lines 318–325 | Yes, **taken alone** | Fully machine-checkable, exact counts confirmed. **But** see Finding 1 — the "Implementation" bullet immediately above it (lines 312–316, untouched) and the stage-summary table (line 26, untouched) still say "the four `TYPED-ONLY` comments" and name only 4 of the 9 real locations |

## 3. Findings

### Finding 1 (Critical) — the fold fixed the S3 Gate but left the adjacent Implementation instruction and the stage table saying "four," making the Gate as specified unbuildable

`design/IMPLEMENTATION_PLAN_multisig_build_repair.md:312-316` (S3 "Implementation,"
**untouched by this fold**):

> Delete or correct **the four** `TYPED-ONLY` comments (§2.2 D-5) at
> `gui/bip85.go:264`, `gui/singlesig.go:18`, `gui/multisig.go:24`,
> `gui/multisig_build.go:67`. … a future reader greps `TYPED-ONLY`, **finds four
> hits**, and concludes the payload cannot reach a seed entry.

Four lines later, the fold's own new Gate (line 318-325) reads:

> `grep -rn TYPED-ONLY --include='*.go'` **returns 0**. Measured: there are **9**
> occurrences across 4 files … **not the 4 an earlier draft assumed**.

The Implementation bullet's list of 4 locations is a strict subset of the 9 measured
(§1.7 above). An implementer who executes the Implementation bullet exactly as
written — delete/correct only those 4 named sites — leaves 5 residual occurrences
(`singlesig.go:32`, `bip85.go:270`, `multisig.go:17`, `:60`, `:102`), and the fold's
own Gate (`returns 0`) then **fails**. The stage cannot close green by following its
own instructions.

This is not a distant cross-reference: the diff hunk that added the "9" / "not the 4"
text has, as its own leading **context** line (unchanged, displayed on screen while
writing the fix), the tail end of this exact Implementation bullet ("…concludes the
payload cannot reach a seed entry."). The stale count was one line above the fix when
the fix was written.

A third, independent restatement of "four" survives at the stage-summary table,
`design/IMPLEMENTATION_PLAN_multisig_build_repair.md:26`: `| **S3** | nested segwit is
nameable; **the four stale comments** die | P2 |` — also untouched.

**Fix:** update line 26 ("four" → "nine") and lines 312-316 (list all 9 locations, or
say "all nine — see the Gate below for the current count" and drop the hardcoded list
to avoid a third copy going stale next time).

### Finding 2 (Important) — S0 deliverable 3 still tells an implementer to do the thing this fold just proved impossible

`design/IMPLEMENTATION_PLAN_multisig_build_repair.md:179-180` (S0 deliverable 3,
**untouched by this fold**):

> **A provenance header for `address/address_test.go`'s existing fixtures**: either
> cite where they came from, or **replace them with BIP-382 vectors**. Unattributed
> expected-addresses are self-agreement wearing the costume of a test.

The fold's own correction, two sections earlier in the same document
(lines 100-107), establishes: *"It also promised **addresses** from 382 and 141/143,
which publish scriptPubKeys and no addresses."* Independently confirmed: BIP-382 has
zero `multi(` (claim 2) **and** zero address strings (`grep -ci address
bip-0382.mediawiki` → 0, checked as part of §1.2 above) — it is exclusively
`wpkh()`/`wsh(pk())`/`wsh(pkh())` scriptPubKey hex. "Replace them with BIP-382
vectors" for **address** fixtures cannot be executed as literally written for the
same reason `TestBip382WshMultiAddressesMatchPublishedVectors` couldn't: there are no
addresses in BIP-382 to replace them *with*.

The fold's own new text even name-checks this exact deliverable four sentences later
(line 114-116): *"S0 may not quietly relax to 'the tests we could write passed.' That
is exactly the failure deliverable 3 names…"* — the author's attention was on
deliverable 3 while writing the correction and still didn't re-open its text.

Not gate-blocking (the bullet has an "either / or" — an implementer can satisfy it by
citing provenance instead), so Important rather than Critical. But it is a false,
citable claim left standing inside S0, the stage whose entire purpose this fold
exists to protect.

**Fix:** drop the dead branch, or repoint it — e.g. "cite where they came from, or
retarget at BIP-383 (multisig) / BIP-382 (singlesig) **scriptPubKeys**, deriving the
address locally as S0's other tests do."

### Minor, not gating (recorded only)

- The `TestBip141NestedSegwitScriptDiffersFromLegacy` bullet (line 212) is legible but
  denser than it needs to be — the "derived locally, not quoted" clause reads as if it
  describes the test's own assertion, when closer reading suggests it's an instruction
  to the S0 README. Doesn't change the test's pass/fail semantics; would read cleaner
  split into a test-scope sentence and a README-scope sentence.
- Pre-existing M-2 from the audit (mk-codec pin line says 0.4.2, `Cargo.lock` says
  0.4.1) is untouched by this fold and remains Minor/out of scope per the audit's own
  severity — not re-raised here as a fold defect.

---

## 4. Bottom line

The fold's *new* factual claims — the ones the brief specifically worried might be
"inherited from the auditor's word rather than the source" — all survive independent
re-execution against the primary BIP text and both CLIs; nothing here should be
un-done. What breaks is **consistency of the whole document**: two edits (S3's Gate,
S0's BIP table) were corrected in place while an instruction sitting a few lines away,
stating the identical fact the fold just disproved, was left standing — once making
the Gate the fold just wrote unsatisfiable by the Implementation bullet immediately
above it (Critical), once leaving a false BIP citation inside the stage whose stated
job is to be the trustworthy oracle (Important). Both are narrowly scoped, mechanical
sweeps (propagate "9" to two more lines; fix or drop one dead clause) — not a redesign.
