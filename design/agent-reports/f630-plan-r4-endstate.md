# R4 — did the r3 fold land, does the END STATE close the original defect, and is the plan proportionate?

**Reviewer:** independent of the fold's author.
**Artifact:** `design/IMPLEMENTATION_PLAN_F630_xpub_header_sync.md` at mnemonic-engrave
`41671c9b` (folds `5d22c58a` + `41671c9b`, 178 insertions / 102 deletions over the
r3 artifact `692d86fa`).
**Trees:** fork `/scratch/code/shibboleth/seedhammer` main `95716e97`; primary
`/scratch/code/shibboleth/descriptor-mnemonic` main `b2c5d693`. Go 1.26.7.
**Method:** Question B was answered by EXECUTION, not by reading. The plan's end
state — T2's record/phrase pins, D6's anti-drift rework, D5c's card fix, all
seven record reads routed through pin-preferring loaders in BOTH packages, T3's
three edits and the real widened re-vendor against `b2c5d693`, and T1's full
D1 + D2a + D2b + D2c + D3 + D5g gate — was built in a throwaway
`git worktree` of the fork at `95716e97` and run. Ten corpus mutations were then
applied, each followed by a provenance re-pin (the vendor script copies *then*
hashes, so a real re-vendor always re-pins), and the whole validation surface was
run for each: `go test ./md/ ./sysw/ ./mk/ ./address/ ./bip380/` plus the FULL
`./gui/` package via `scripts/gui-shard-test.sh ./gui/ 24` (1374 tests, partition
asserted exhaustive, ~25s). The worktree was removed with
`git worktree remove --force` and pruned; `git status --porcelain` is empty in
the fork, in descriptor-mnemonic and in mnemonic-engrave, and `git worktree list`
no longer carries it.

**Counts: 2 Critical, 1 Important, 3 Minor, 1 Nit.**
**A-verdict tally: 14 FIXED / 0 PARTIAL / 0 NOT-FIXED (C-1, C-2, I-1…I-5, M-1…M-5, N-1, N-2).**
**B: NO — a silent import of the F-630 shape is still possible.**

---

# A. Did the r3 fold land?

| r3 finding | verdict | the plan text that settles it |
| --- | --- | --- |
| **C-1** the gate cannot reach 46 of 46; T2 freezes three records that fail D1 | **FIXED** | D5g's two tiers, and T3's acceptance now reads "**43 of 46 under correct-header D1, plus 3 under the pinned-legacy shape**, not 46 of 46, which is unreachable by construction" |
| **C-2** routing `md/conformance_keyed_test.go`'s record reds the cross-language gate | **FIXED** | D5c: "**a file that reads a record must obtain its card from the pin-preferring loader too**", and T2.3 names the line: "`md/conformance_keyed_test.go`'s `loadPhraseChunks` → `vectorChunksFor`" |
| **I-1** T2 routes only `gui/` but gates `md/` | **FIXED** | T2.1 creates `gui/vector_fixtures_test.go` **and** `md/vector_fixtures_test.go`; T2.3 "Route **both packages** — `gui/` and `md/` (r3 I-1…)" |
| **I-2** nine of sixteen occurrences are not record reads and get no disposition | **FIXED** | D5d classifies all 16 ("only **6** are record reads. The rest are 4 corpus-enumeration globs, 4 `strings.TrimSuffix` name derivations, and 1 `t.Fatal` message"), moves the enumeration into the fixtures file, rewords the diagnostic |
| **I-3** the loader's file-granular exclusion, placed beside `loadVectorChunks`, blinds the gate | **FIXED** | D5d: "`loadVectorChunks` **moves into that file too**, and this is load-bearing … Leaving it there and excluding its file drops the gate from 16 flagged tokens to 14" |
| **I-4** the self-test proves the matcher, never the walk | **FIXED** | D5e: "a **per-directory** file count for each scanned directory; that a synthetic known-bad placed **as a file inside each scanned directory** is flagged, so the walk itself is exercised" |
| **I-5** `isComposeVectorFile` keeps covering only the old tier | **FIXED** | T3's third edit, with the reproduction: "measured by smuggling in an unpinned `keyed_tr_smuggled.…json`" |
| **M-1** the scan cannot see its own comment marker; a naive `Contains` matches the gate's own source | **FIXED** | D5f: "the marker is a **comment**, which the token scan skips by construction, so honouring it needs a second read (`scanner.ScanComments`, or raw text)… the gate skips its own file" |
| **M-2** "reports" is not "asserts" | **FIXED** | D5f: "the count must be **asserted** (`if markers != 1 { t.Errorf }`), not logged" |
| **M-3** `loadVectorRecord`'s return type unspecified | **FIXED** | T2.1: "The loaders return **`[]byte`**, not a decoded type: the seven call sites unmarshal into four different anonymous struct shapes" |
| **M-4** T2's `-run` filter depends on a name the plan never gives | **FIXED** | T2.5 names it `TestVectorRecordBoundaryHoldsInBothPackages`; T2's gate regex carries both `VectorRecord` and `Boundary` |
| **M-5** T3's two new numbers are not stated | **FIXED** | T3 states 36 → 50 and 176 → 246, lists the 14 names, gives the arithmetic check, rewrites the `:106` comment |
| **N-1** `go/scanner` will not import unaliased into package `gui` | **FIXED** | D5f's last sentence |
| **N-2** a naive `grep -o` returns 37 for a 36-name list | **FIXED** | T3: "**Count that list with `ast`, not `grep -o '\"[^\"]*\"'`**" |

Three of these were re-verified by execution rather than by reading:

- **C-2's remedy works.** With the card switched to `vectorChunksFor` and the
  record to `vectorRecordFor`, `go test ./md/` is `ok` at real T2+T3 state.
- **I-5's remedy works.** Widening `isComposeVectorFile` to `keyed_` and
  smuggling an unpinned file in:
  `keyed_tr_smuggled.conformance.json: in testdata/vectors but not in the provenance pin`
  — flagged, where r3 measured it passing. `keyed_compose_smuggled` still flagged.
- **M-5's numbers are right.** The real widened script printed
  `vendored 246 files, 50 vectors, primary b2c5d6938432`, 0 files added or
  removed, and both pin tests pass with the 246/50 literals.

## D5g — is the two-tier design sound, and can a defect hide in "pinned-legacy"?

**Its premise and its arithmetic are exactly right, and the split is real.**
Measured at the end state, with the pinned arm disabled so every vector is
judged under D1 alone:

```
F630 gate: 46 vectors, PASS 43, FAIL 3
    keyed_tr_multi_a, keyed_tr_sortedmulti_a, keyed_wsh_timelock_hashlock
```

and with the arm restored, `46 vectors, PASS 46, FAIL 0`. The three are exactly
the pinned three, they genuinely cannot satisfy D1, and the legacy arm is tight
in the direction the plan claims: it demands depth 0 **and** child 0 **and**
parent fp 0 **and** a non-empty origin, so a third staleness fails and a
corrected record fails.

**But the tier is selected by the mere existence of a file, and nothing asserts
the tier's membership.** That is where a defect hides — see I-1 below, which
reproduces it end to end.

---

# B. Does the end state close the ORIGINAL defect? — **NO.**

## What the gate does close

The end state is genuinely non-vacuous, and against the actual 0.44.0 drift it
has teeth. Run against the **stale** corpus it reds 41 of 46 (5 pass: the two
bare-fingerprint origins plus the three that reach the pinned arm; at T1, before
the pins exist, this is the plan's stated 44/2). Two constructed regressions are
caught:

| mutation | outcome |
| --- | --- |
| **M1** re-point an account in the rendered bracket only (`/48'/0'/1'/2'` → `/48'/0'/9'/2'`, xpub untouched) | **CAUGHT** by D3 — `md` red, `TestKeyedConformanceDescriptorHeadersAgreeWithTheirOrigins/keyed_wsh_multi_2of3` |
| **M9** drop one key from a rendered `multi(2,…)` | **CAUGHT** by D2b's "every Go slot appears at least once" |

So D1 + D2 + D3 do close the *header* subclass and the *re-pointed origin*
subclass. That is real and it is what the plan set out to build.

## What still slips through — six shapes, all measured GREEN on the whole suite

Each row below was applied to the end-state corpus, re-pinned the way the vendor
script re-pins, and then run against `./md/ ./sysw/ ./mk/ ./address/ ./bip380/`
**and all 1374 `./gui/` tests**. Every one reported `RESULT: ok`.

| # | mutation (descriptor-only; addresses and ids untouched) | verdict |
| --- | --- | --- |
| **M2** | derivation suffix `/0/*` → `/7/*` on one chain of one vector | **SILENT** |
| **M8** | the same suffix regression across the **whole corpus** — 92 of 92 chain descriptors in 46 files | **SILENT** |
| **M3** | swap two `multi()` key positions | **SILENT** |
| **M4** | a wrong BIP-380 checksum (`#uruj67xy` → `#qqqqqqqq`) | **SILENT** |
| **M5** | swap `chains["0"].descriptor` with `chains["1"].descriptor` | **SILENT** |
| **M6** | **`multi(2,…)` → `multi(3,…)`** — the quorum | **SILENT** |
| **M7** | **`wsh(…)` → `sh(wsh(…)))`** — the script type | **SILENT** |

Mutations were asserted to have APPLIED, not assumed. M8, verified after the
run: `M8 APPLIED: 92 of 92 chain descriptors now carry a shifted suffix`, with
`…52T3sWa2bPW/7/*))#uruj67xy` on chain 0 and `/8/*` on chain 1, and
`addresses UNCHANGED: bc1q8z8kvwnpeqy79hfkggrtfm26hkgq2tu708a86tcwtm5gy5wrc85s99fe24`.

**M8 is the F-630 shape, exactly.** The primary's descriptor renderer changes,
every chain descriptor in the corpus moves, no address line and no id line
moves, the re-vendor imports it, the provenance pin is regenerated in the same
action, and the whole suite — including the new gate — says `ok`. That is the
sentence F-630 was filed to make untrue.

**Why.** The gate's walk is `\[fingerprint/path\]xkey` and it stops at the key.
Everything else in the rendered string — the suffix after the key, the ordering
of operands inside `multi()`, the threshold, the script wrapper, the checksum —
is parsed by nobody. At the end state the only two places in the fork that parse
`.chains[].descriptor` are the new gate (`md/f630_header_gate_test.go`) and
`md/conformance_keyed_test.go:22`, which still parses it into a field it never
asserts.

---

## C-1 (Critical) — the plan's residue section claims a coverage that does not exist

Plan, *What this plan does NOT cover*:

> "…the derivation suffix `/0/*` → `/7/*`, and a swap of two `multi()` key
> positions. **The last two are caught as *drift* by
> `TestEveryKeyedVectorReachesAnAddress`**, which compares Go-derived addresses
> per chain against the record's; the checksum and the chain-index suffix are
> covered by nothing."

That test **cannot see a descriptor at all**. Its record type, `policyAddrVector`
(`gui/policy_address_test.go:18-24`), is:

```go
type policyAddrVector struct {
	Name     string `json:"name"`
	Template string `json:"template"`
	Chains   map[string]struct {
		Addresses []string `json:"addresses"`
	} `json:"chains"`
}
```

There is no `Descriptor` field. The test derives addresses from the *card* and
compares them to the record's `addresses` — so it fires on an **address** move,
and the class F-630 exists for is precisely the class where addresses do not
move (41 of the 44 drifted records changed descriptors with **zero** address
lines). Executed at the full end state, M2 and M3 both leave the entire suite
green, including that test.

**State the wrong outcome:** a future reader — or the implementer deciding
whether the residue is acceptable — reads that sentence, concludes two of the
four gaps are already covered, and ships. Nothing catches either. A blind spot
recorded as covered is worse than one recorded as open, which is the plan's own
standard ("a gate that hides its own blind spot is worse than no gate", D5b).

**Remedy, cheap:** delete the clause and move both items into the uncovered
list. If instead the gate should catch them, both are a handful of lines inside
the loop already written: assert the text between the matched key and the next
`,`/`)` is `/<chain key>/*`, and assert the matched slots' order equals the Go
expansion's slot order for non-`sorted` `multi()`.

## C-2 (Critical) — the residue list omits two funds-relevant shapes, and both import silently

The plan enumerates **four** things outside the gate (checksum, chain swap,
derivation suffix, `multi()` key order). Measured, two more are outside it and
neither is named anywhere in the plan:

- **M6 — the quorum.** `wsh(multi(2,…))` rendered as `wsh(multi(3,…))`: whole
  suite green.
- **M7 — the script type.** `wsh(…)` rendered as `sh(wsh(…))`: whole suite green.

These are not spelling. A rendered descriptor is what an operator exports to a
coordinator and what a watch-only wallet is built from; a wrong threshold or a
wrong script wrapper imported from the primary and vendored into the fork is a
funds-safety divergence, and the plan's own framing ("the corpus stops diverging
silently") would read as covering them.

**State the wrong outcome:** the primary regresses its renderer in either of
these ways, the re-vendor imports it, `TestKeyedConformanceDescriptorHeadersAgreeWithTheirOrigins`
goes green on all 46, and the fork's corpus now pins a quorum or a script type
the primary never meant to ship, with nothing red anywhere.

**Remedy:** at minimum, name them in *What this plan does NOT cover* alongside
the other four, so the residue the plan records is the residue that exists. The
full fix (compare the descriptor's script structure against the Go template) is
larger and is legitimately a separate cycle — but it must be *recorded*, not
absent.

## I-1 (Important) — D5g's tier is selected by file existence, and nothing pins its membership

The plan's own rule for the *other* exemption mechanism is explicit (D5f):

> "Never a list in another file: a list drifts silently, a marker is a visible
> line in the diff that adds it."

…and D5f then **asserts the count** (`if markers != 1 { t.Errorf }`), because
"reports" is not "asserts" (r3 M-2, folded). D5g's exemption has neither. Its
tier selector is "does `md/testdata/forkbuilt/<name>.conformance.json` exist",
and no test asserts that the pinned tier holds exactly the three F-529 names.

Reproduced end to end at the full end state:

```
# 1. import the ORIGINAL 0.44.0 defect for one vector (the stale record), re-pin
F630 gate: 46 vectors, PASS 45, FAIL 1
--- FAIL: TestKeyedConformanceDescriptorHeadersAgreeWithTheirOrigins/keyed_wsh_multi_2of3

# 2. copy that same defective record into md/testdata/forkbuilt/
ok  	seedhammer.com/md	0.130s
ok  	seedhammer.com/sysw    ok  	seedhammer.com/mk
RESULT: ok -- all 1374 tests ran across 24 shards
```

One added file turns the gate's only red into a fully green suite — and it works
*because* the record carries the 0.44.0 defect shape, which is exactly what the
legacy arm accepts. T1's warning ("an implementer who tunes the gate until
exactly 41 fail would exempt precisely the three vectors this cycle is most
exposed on") names this hazard and D5g then hands over a general-purpose,
unasserted version of it.

**State the wrong outcome:** a future re-vendor imports a genuine header
regression, the gate reds, and the shortest path to green is to pin the
regressed record — which the plan's design permits, no test objects to, and the
next reader cannot distinguish from the three legitimate pins.

**Remedy, one assertion:** the gate enumerates
`md/testdata/forkbuilt/*.conformance.json`, and fails unless the set equals the
three named F-529 vectors — the same shape as D5f's `markers != 1`.

## M-1 (Minor) — the checksum gap is cheaper to close than the plan implies

The plan lists the BIP-380 checksum as "covered by nothing", which is true
(M4: a `#qqqqqqqq` checksum is silent). But the fork already ships a validator:
`bip380/checksum.go:57`, `func validChecksum(s, c string) bool`. It is
unexported, so the md-side gate cannot call it as-is, but a gui-side clause or a
one-line export makes the fourth residue item disappear for roughly the cost of
recording it.

## M-2 (Minor) — D5e's synthetic known-bad must be a `*_test.go` file written into `gui/` and `md/`

D5b scans "the **STRING TOKENS** of every `*_test.go`", and D5e requires the
probe be "placed **as a file inside each scanned directory**" so the walk is
exercised. Those two together mean the gate writes a `*_test.go` file into a
package directory that the Go toolchain compiles. A panic, a `t.Fatal` before
cleanup, or a killed run leaks it, and the next `go test ./gui/` fails to build
for a reason that has nothing to do with the corpus. Worth a sentence in D5e
naming the cleanup obligation (`t.Cleanup`, not `defer`) and a name that sorts
out of the way.

## M-3 (Minor) — T1's acceptance count changes once T2 lands, and the plan does not say so

T1's acceptance is "44 of 46 fail, 2 pass", correct at T1 because no pinned
record exists yet — the plan says this. But an implementer who re-runs T1's gate
against the stale corpus *after* T2 (a natural thing to do when T3 misbehaves)
measures something else. Executed at the end state against the stale corpus:

```
F630 gate: 46 vectors, PASS 5, FAIL 41
```

5, not 2, because the three pinned records now reach the legacy arm and pass
there. Nothing is wrong; one sentence stops it reading as a regression.

## N-1 (Nit) — D6's recorded template ids are 8-hex prefixes, not the ids

D6 records the "primary moved here" values as `8c1c0566` / `09903620` /
`71ff3b74`. Measured at `b2c5d693` the full ids are:

```
keyed_tr_multi_a             8c1c05666abdf6b29df9c2056c0cb49e
keyed_tr_sortedmulti_a       09903620dbcf4e059300f23391079052
keyed_wsh_timelock_hashlock  71ff3b7424b35d410c16b6d992aa437e
```

An implementer who codes the prefixes gets a materially weaker "exact shape"
pin than D6 promises ("A pinned gap with an exact shape; any *third* policy
under these names fails"). Quote the 32 hex characters.

---

# C. Is the plan proportionate? — **Cut the structural scanner.**

**Cut D5b + D5e + D5f and D5d's forced consolidation — the token scan, the
per-directory counts, the synthetic known-bads, the `//go:vectortier` marker
protocol, and the relocation of `loadVectorChunks` and every enumeration helper
into two new fixtures files. Keep the fixtures files themselves and the routed
pin-preferring loaders.**

Three measurements, not an opinion.

**1. It buys no defect-detection on this cycle's subject.** I built the entire
end state without it — T2's pins, D6's rework, D5c's card fix, all seven record
reads routed in both packages, T3's three edits, the full T1 gate — and ran ten
mutation probes against the whole 1374-test surface. The scanner would have
changed **zero** verdicts. Every silent import in section B is silent for a
reason the scanner does not touch, and every catch is a catch the scanner does
not contribute to.

**2. The failure mode it guards against announces itself.** The scanner exists
so a *fourth* split site cannot appear. I wrote one, in the exact evasive
spelling D5b was built for — a plain string literal for the record, paired with
the non-pin-preferring card loader:

```
--- FAIL: TestProbeFourthSplitSite/keyed_tr_multi_a
      go(card):     fe4d264c6e40999b8695329adfe599d9
      rust(record): 1f26f9b7cdb8e745c898bb93f1134ba7
--- FAIL: TestProbeFourthSplitSite/keyed_tr_sortedmulti_a
--- FAIL: TestProbeFourthSplitSite/keyed_wsh_timelock_hashlock
```

It fails immediately, loudly, on all three pinned vectors, with a cross-language
mismatch that names the problem. The plan already knows this: its "Sites
depending on `keyed_wsh_timelock_hashlock`" section retracted the earlier
"they'd go green" framing precisely because "**Measured, they fail loudly**". A
structural gate that polices a loud failure into a *louder* one is insurance
against a risk that has already been measured as self-reporting.

**3. It is the plan's largest cost centre and its least settled part.** Four of
the twelve decisions (D5b, D5d, D5e, D5f) exist only to make it work. It forces
two new files, the relocation of `loadVectorChunks` out of a file that would
otherwise be untouched, a reworded `t.Fatal` message, an exemption-marker
protocol that had to be told to skip its own source, an aliased import, a
per-directory file count, and synthetic `*_test.go` files written into two live
package directories (M-2). It is also the only component whose own correctness
has consumed three review rounds — r2's Q1, r3's Q2/I-2/I-3/I-4, then M-1/M-2/M-4
in this fold — while the defect it guards has occurred zero times and would be
loud if it did.

**Do not cut** the rest. D5/D5a's routing fixes a *measured*, three-site,
reproduced defect; D5c's one line is what keeps the existing cross-language gate
green; D5g is load-bearing (T3 is unreachable without it); D6's rework is
mandatory (the anti-drift test reds on the re-vendor without it — measured); and
T3's `isComposeVectorFile` edit is one line that closes a verified hole. The
proportionality problem is one component, and it is the scanner.

**If the scanner is kept anyway**, the cheapest substitute for its *actual*
value is one assertion, not a walk: the fixtures file exports the loaders, and a
single test asserts that the count of `keyed_*.conformance.json` reads outside
it is zero **by construction** — i.e. delete the raw-read helpers, so the only
compiled path to a record is the loader. That is a build-time property, not a
scanned one, and it cannot be evaded by a string literal because there is
nothing for the literal to be passed to.

---

## Out of scope, per the brief

Earlier rounds' findings were not re-verified; style and wording were not
reviewed; no implementation exists to review; no new features were proposed. No
tracked file in either repo was modified — fork, primary and mnemonic-engrave
all verify clean, and the throwaway worktree was removed and pruned.
