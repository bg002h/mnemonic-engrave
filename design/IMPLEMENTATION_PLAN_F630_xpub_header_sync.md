# Implementation plan — F-630, the xpub-header corpus sync and the gate that can see it

**Baseline revisions.** seedhammer (fork) main `95716e97`; descriptor-mnemonic
main `b2c5d693` (md-codec 0.45.0 / md-cli 0.17.0); mnemonic-engrave master
`ed4b98fc`. Every count below was measured against those trees on 2026-09-20,
not read from a report.

## What recon falsified

F-630 was filed as "port md-codec 0.44.0's xpub-header rewrite into the fork".
Three of its four prescribed steps do not survive measurement.

| F-630 / continuity step | measured status |
| --- | --- |
| 1. Assert `.chains[].descriptor` in `TestKeyedConformanceAgreesWithRust` | **NOT DONE — this is the whole remaining job** |
| 2. Re-vendor `keyed_compose_*` from dm `b2c5d693` | **PARTIAL** — the script's pattern reaches 32 of the 41 drifted records |
| 3. Port 0.44.0's header rule into `md/` | **NOT NEEDED — the fork was never wrong** (measured, below) |
| 4. Bring 0.45.0's four `precedence_*` cases across | **ALREADY DONE** — 8 cases run, 4 precedence kinds asserted |

### Step 3 is unnecessary, measured rather than argued

`md-codec` 0.44.0 fixed `assemble_origin_and_xkey`, which built a rendered
xpub's header from nothing while building the origin from `e.origin_path`, so
every emitted key serialised at depth 0 / child 0 under a four-component
origin. The fork has no analogue of that function. Its render path is
`gui/md1_expand.go:expandedKeysToBip380` → `bip380.Key.ExtendedKey()`
(`bip380/bip380.go:97-109`), which has always taken

    depth    = uint8(len(k.DerivationPath))
    childNum = k.DerivationPath[len(k.DerivationPath)-1]

from the one origin it was given. A throwaway probe in package `gui` rendered
`keyed_compose_preset_plain_multisig` from its vendored card:

    wsh(sortedmulti(2,[73c5da0a/48h/0h/0h/2h]xpub6DXuQW1Q2JpZxsEnFKrPvDuiRMmQgU4fz…

which is **byte-identical, key for key, to dm `b2c5d693`'s corrected
`chains["0"].descriptor`**. The fork already satisfies the invariant. What is
stale is the vendored corpus, and what is missing is a gate that can see it.

`ParentFingerprint: 0` is set explicitly at `gui/md1_expand.go:143`, matching
0.44.0's "parent fingerprint stays zero, necessarily" — the parent point is not
on the md1 wire.

### Step 4 is already complete

`go test ./md/ -run TestComposeKeylessCap -v` reports
*7 refused, 1 admitted, kinds map[KeylessUnderTr:1 LegacyWrapperShape:1
NoKeyedPath:1 TooManyKeylessPaths:3 TooManySlots:1]*. The vector is pinned at
`b2c5d693` and the test drives every case out of the file, so all four
`precedence_*` cases already execute. Nothing to port.

## The measured drift

46 keyed conformance records, compared file by file:

| class | count | what moved |
| --- | --- | --- |
| identical | 2 | `keyed_tr_keyonly`, `keyed_wpkh` — bare-fingerprint origins, so depth 0 was already right |
| descriptor-only, script-covered | 32 | `keyed_compose_*` |
| descriptor-only, **not** script-covered | 9 | `keyed_tr_depth2`, `…_rightspine`, `keyed_tr_pathological`, `keyed_tr_with_leaf`, `keyed_wsh_multi_2of3`, `keyed_wsh_or_b`, `keyed_wsh_or_d_degrading`, `keyed_wsh_sortedmulti_2of3`, `keyed_wsh_thresh` |
| semantic — **F-529** | 3 | `keyed_tr_multi_a`, `keyed_tr_sortedmulti_a`, `keyed_wsh_timelock_hashlock` |

82 chain-descriptor entries move across the 41, and 88 across all 44 drifted
records (the F-529 three carry stale descriptors too). Zero address lines and
zero id lines move in those 41 — the three records whose `keys`, `fingerprints`,
`wallet_policy_id`, `md1_encoding_id`, `wallet_descriptor_template_id` and
addresses all move are exactly F-529's three, and none of them is a
`keyed_compose_*`.

**The corpus has two tiers and only one is pinned.** `scripts/vendor-compose-vectors.sh`
selects `^(keyed_)?compose_` (177 files, `compose_vectors.provenance.json`).
The other 14 keyed vectors have no provenance pin at all, and that unpinned
tier is where F-529's divergence lives. A re-vendor run as the script stands
today would fix 32 records and leave 9 stale — shipping the new gate red.

## Decisions

**D1 — the assertion is over the header, not the descriptor string.** The fork
renders multipath (`<0;1>/*`) while the Rust record splits per chain (`/0/*`),
so the two spellings are equivalent and not comparable as strings. For every
`[origin]xpub` in every `chains[].descriptor` the gate asserts: the xpub
base58-decodes with a valid checksum; `depth == len(origin components)`;
`child number == terminal origin component` (hardened-encoded, 0 for an empty
path); `parent fingerprint == 0`.

**D2 — bind the record's two spellings, and bind them to the Go port.** The
0.44.0 defect survived because "a uniformly wrong corpus agrees with itself":
`keys[]` held the correct depth-4 key and `descriptor` held the depth-0
rendering. So the gate also asserts that each descriptor xpub's
(chain code ‖ compressed pubkey) equals some `keys[]` entry's (D2a), **and**
that it equals some slot of the Go port's own `ExpandWalletPolicyChunks`
expansion of the same card, with every Go slot carrying an xpub appearing at
least once (D2b). That last clause is what makes it cross-language rather than
a JSON self-consistency check.

`keys[]` xpubs carry the **real** parent fingerprint (measured: `1cf29716`,
`3edf3f57`, `64f9d328` for `keyed_compose_preset_plain_multisig`) while the
descriptor's carry zero — so the comparison is over the 65 bytes, never the
base58 string.

**D2c — bind the origin bracket's FINGERPRINT (R0 I-7).** D1 relates the
bracket's path to the header and D2 binds key material; nothing bound
`[XXXXXXXX/…]`, and a mutation setting every bracket fingerprint to `deadbeef`
passed the whole gate. In the Rust function 0.44.0 rewrote, the fingerprint is
assembled on the same line as the path (`to_miniscript.rs`,
`e.fingerprint.map(|fp| (Fingerprint::from(fp), path))`), so a regression there
emits a uniformly wrong fingerprint that agrees with itself — the exact failure
mode D2 exists for. Addresses do not depend on it and `wallet_policy_id` is
computed from the card, so no existing test can see it. The gate asserts the
bracket fingerprint equals `ExpandedKey.Fingerprint` (`md/expand.go:56-64`);
measured, they agree **284/284** on the re-vendored corpus today.

**D3 — the elided-origin divergence is pinned on BOTH sides (R0 I-4, N-1).**
For `keyed_wpkh` and `keyed_tr_keyonly` the Rust record emits a bare-fingerprint
origin (`[73c5da0a]`, depth 0) while the Go port canonical-fills from
`canonicalOrigin(tree)` and renders `[73c5da0a/84h/0h/0h]` at depth 3 — the
deliberate R0-I1 divergence `md/expand.go:76-81` documents. Both satisfy D1
independently.

So: the gate **does** assert bracket-path == Go origin path for every vector,
and carries a two-sided allowlist for those two — pinning the Go path
(`m/84h/0h/0h`, `m/86h/0h/0h`) **and the record's expected bracket** (bare
`[73c5da0a]`, zero components). It fails if an allowlisted vector stops
diverging, if a non-allowlisted one starts, **or if an allowlisted record's
bracket changes at all**. The first version pinned only the Go half, and a
record re-pointed to account `9h` — consistently, header and bracket together —
passed every clause: a wrong-account descriptor on the two plainest single-key
vectors in the corpus. Pinning the record half closes that.

Hardening is spelled differently on the two sides — the fork renders `48h`,
the record carries `48'` — so every path comparison normalises before
comparing. Same class as F-627.

**D4 — widen the vendor script to the whole keyed tier, minus the refusal
vectors.** Leaving 9 records stale ships a red gate, which is worse than no
gate. `vendor-compose-vectors.sh:16` moves from `^(keyed_)?compose_` to
`^(keyed_|compose_)` **excluding `compose_refusal_`**. That exclusion is not
cosmetic: the script as it stands already selects 177 files against a pin of
176, because `compose_refusal_keyless_cap.json` is deliberately carried by
`compose_refusal_vectors.provenance.json` instead
(`isComposeVectorFile`, `md/compose_vectors_pin_test.go:79-84`). Re-running the
script today would pull it into the compose pin and trip
`compose_vectors_pin_test.go:134`. Pre-existing rot, detonated by T3.

**D5 — a fork-side pin carries the RECORD as well as the CARD (R0 C-1).** This
is the correction that matters most. `TestEveryKeyedVectorReachesAnAddress`
takes its **record** from a glob of the vendored corpus
(`gui/policy_address_test.go:125`) and its **card** from the pin-preferring
`loadVectorChunks` (`:174`), then compares addresses derived from the card
against the record's expected addresses (`assertMatchesRust`,
`gui/policy_address_test.go:77`). Those are the same policy today. Pin the card
and re-vendor the record and they are **different policies**, and the test
compares one against the other — measured red on all three F-529 vectors, e.g.

    keyed_wsh_timelock_hashlock chain 0 index 0 via complex:
      got  bc1q6h9y4ngdfacaplw0qk67rugxs3vanv0jayk3n3xnhed7uke76zksfqt7py
      want bc1qa9wapjm45uthw7r806zuz9mev2a5cwqmrlwnyuj4pjs0c78d75rqp6j29e (rust)

Of the three ways out, two destroy evidence: moving the vectors into
`stillUnsupported` retires the cross-language address check for every shape the
refusal covers (the test's own comment says so at
`gui/policy_address_test.go:143-147`), and dropping the pin preference deletes
the F-533 and F-514 witnesses outright. So the pin gains a record: a
`loadVectorRecord(t, name)` helper mirroring `loadVectorChunks`, preferring
`md/testdata/forkbuilt/<name>.conformance.json`.

**D5a — the fix is a BOUNDARY, not three more call sites (fold-verify NEW-1).**
The first draft of D5 wired `loadVectorRecord` into one test and then claimed,
globally, that "record and card are always one policy". That sentence was
false. Two more sites pair the pin-preferring card loader with a plain
vendored record read, and both fail identically once T2+T3 land — measured, not
projected:

| site | record | card | F-529 vector reached |
| --- | --- | --- | --- |
| `gui/policy_address_test.go:125` | glob `keyed_*` | `:174` `loadVectorChunks` | all three |
| `gui/taproot_script_path_test.go:30` | glob `keyed_tr_*` | `:55` `loadVectorChunks` | `keyed_tr_multi_a`, `keyed_tr_sortedmulti_a` |
| `gui/wsh_script_emit_test.go:31` | glob `keyed_wsh_*` | `:72` `loadVectorChunks` | `keyed_wsh_timelock_hashlock` |

    keyed_tr_multi_a chain 0 index 0:
      go:   bc1pf4aujydl48hah9qxvk4j0dcce737pl9svne7rmzcprrh7y92znsstul4rt
      rust: bc1pgrupj0fjv79xtj05mzthds4dqzvdhcptx2gzpgzt90uzc86qzfes2a0yhh

The two tr vectors are pinned **today**, pre-implementation, so that hazard is
live already and only needs the record side to move — which T3 does.

Three occurrences of one shape is a wrong shape, so this plan does not add a
third call-site edit. **Every read of a `keyed_*.conformance.json` record goes
through `loadVectorRecord`, and a structural test enforces it.**

**D5b — the scan matches the STRING, not a call shape (r2 Q1).** The first
draft said the gate scans for a
`filepath.Join(.., "vectors", …".conformance.json")` read. That is defeated by
a plain string literal — the *existing* idiom for testdata paths in this very
package (`gui/template_engrave_test.go:207`,
`gui/composer_selfcheck_test.go:269`). Reproduced against a fully-routed,
gate-passing tree: a fourth test reading
`os.ReadFile("../md/testdata/vectors/" + vector + ".conformance.json")` and
pairing it with `loadVectorChunks` compiled clean while the gate PASSED.

So the gate scans the **STRING TOKENS** of every `*_test.go` (`go/scanner`,
whose default mode yields string tokens and skips comments) for the substring
`conformance.json`. Measured on the tree: **18 occurrences across 9 files, 2 of
them in comments** — which is why it is token-based and not a grep.
(`".conformance" + ".json"` evades; contrived, and the gate says so rather than
pretending otherwise.)

**D5c — the boundary is about PAIRS, and that is the rule (r3 C-2).** Stating
it as "every record read goes through the loader" was wrong in a way that reds
an existing gate. `md/conformance_keyed_test.go` globs `keyed_*` — all three
F-529 names included — and takes its card from the non-pin-preferring
`loadPhraseChunks`. Route only its *record* and the pinned record meets a
re-vendored card: C-1's split, inverted, in the sibling package. Reproduced at
real T2+T3 state:

    keyed_tr_multi_a: wallet_policy_id
      go:   fe4d264c6e40999b8695329adfe599d9      (re-vendored card)
      rust: 1f26f9b7cdb8e745c898bb93f1134ba7      (pinned record)

**The rule is therefore: a file that reads a record must obtain its card from
the pin-preferring loader too.** Concretely that is one line —
`md/conformance_keyed_test.go`'s `loadPhraseChunks` → `vectorChunksFor`,
verified to restore green — and the same obligation is latent in
`md/compose_pkh_emit_test.go:37` and `md/compose_stubs_test.go:15`, inert today
only because no `keyed_compose_*` name has a pin.

**D5d — one file owns finding and loading a vector, which is what makes the
scan enforceable (r3 I-2, I-3).** Of the 16 in-code occurrences, only **6** are
record reads. The rest are 4 corpus-enumeration globs, 4 `strings.TrimSuffix`
name derivations, and 1 `t.Fatal` message — none a read, all flagged, and the
plan previously gave them no disposition. Banning the substring while nine
legitimate uses remain would force an implementer to invent exemptions the plan
forbids.

So the enumeration moves too. A new file per package —
`gui/vector_fixtures_test.go` and `md/vector_fixtures_test.go` — owns
`loadVectorRecord`/`vectorRecordFor`, the card loaders, **and** the
glob-plus-name-derivation helpers (`eachKeyedVector`). Every other test asks
that file, so no other file has any reason to name the string, and "fails on
any occurrence outside the fixtures file" becomes true rather than aspirational.
The `t.Fatal` message is reworded to stop naming the file it is about.

`loadVectorChunks` **moves into that file too**, and this is load-bearing: the
exclusion is file-granular, and `loadVectorChunks` lives today at
`gui/taproot_script_path_test.go:136` — the same file as one of D5a's three
split sites. Leaving it there and excluding its file drops the gate from 16
flagged tokens to 14, blinding it to the very site it was built for. Measured.

**D5e — the gate must prove the WALK can fail, not just the matcher (r3 I-4).**
A source scan that matches nothing passes vacuously, and this one did: the
natural first implementation matched bare literals (`*ast.BasicLit`) while
every real call site writes `name+".conformance.json"` (a `BinaryExpr`). It
examined 20+ files, was non-vacuous by file count, and still PASSED on a tree
carrying a real violation.

A file count is also not enough: `gui/` alone supplies 244 of the 281 files, so
any aggregate threshold an implementer would pick is met with an entire package
missing. The gate therefore asserts, in order: a **per-directory** file count
for each scanned directory; that a synthetic known-bad placed **as a file
inside each scanned directory** is flagged, so the walk itself is exercised and
not merely the matcher; and only then that the real tree is clean.

**D5f — exemptions are declared in the exempt file (r2, r3 I-2).** One
survives the consolidation: `md/compose_vectors_pin_test.go:158` stats
`vectorPath(name, "conformance.json")` **to audit the vendored tier itself**,
and a pin-preferring loader there would mask a missing vendored file whenever a
pin exists — the gate hiding the drift the provenance test exists to find. It
carries a `//go:vectortier vendored` marker the scan recognises. Never a list
in another file: a list drifts silently, a marker is a visible line in the diff
that adds it, and the gate reports how many markers it honoured so a second
cannot appear unnoticed.

**D5g — a PINNED record is a fork-maintained fixture, and D1 cannot hold for
it (r3 C-1).** This is the correction the implementability walk forced, and it
is a real contradiction, not a wording slip. T2 freezes the three F-529
records; those three are among the 44 that fail D1, because their
`chains[].descriptor` carries the master xpub under a four-component origin —
the exact 0.44.0 defect D1 detects:

    keyed_tr_multi_a: tr([73c5da0a/48'/0'/0'/2']xpub661MyMwAqRbc…
      decoded: depth 0, child 0   →  D1 wants depth 4, child 2'

And **no corrected rendering exists or can be obtained**: the pin exists
precisely because `b2c5d693` replaced the reuse-BEARING policy with a
reuse-FREE one, so the primary ships no post-0.44.0 record of the policy being
frozen. A gate reading records pin-preferringly reaches 43 of 46, not 46.

The resolution is that the two tiers are not the same kind of artifact. A
vendored record is the primary's current answer; a **pinned record is a
fork-maintained fixture preserving a policy the primary dropped**, and its
value is its addresses and ids — which 0.44.0 did not move, by its own
measurement of zero changed address lines. So:

- **vendored tier** — full D1. All 46 must pass after T3.
- **pinned tier** — the legacy header shape is asserted *exactly* (depth 0,
  child 0, parent fp 0 under a non-empty origin), plus full D2a, D2b and D2c.
  A pinned record whose header is **neither** the legacy shape **nor** the
  correct shape fails, so a third staleness cannot slip in, and one that
  becomes correct fails too — telling us the primary has re-shipped it.

T3's acceptance is therefore **43 of 46 under correct-header D1, plus 3 under
the pinned-legacy shape**, not "46 of 46". Exempting the three outright is the
one thing T1 already warns against; this pins their exact shape instead.

**D6 — the pins' anti-drift check pins a DIVERGENCE, not a deletion (R0 I-1).**
`TestPinnedKeyReuseVectorsStillMatchTheVendoredCorpus` skips only when the
vendored file is **gone** (`md/f533_pinned_vectors_test.go:82-86`). F-529's
premise was that a re-vendor deletes these vectors; measured, `b2c5d693` still
ships all three, **changed** — so the skip never fires and the test fails
instead, on the two existing pins, with no T2 change at all. Worse, its failure
message prescribes the destructive remedy ("re-copy the vendored phrase"),
which would replace the reuse-bearing witnesses with reuse-free cards.

The check becomes: the pin matches the vendored file, **or** the vendored
file's `wallet_descriptor_template_id` equals a recorded "the primary moved
here" value — `8c1c0566` / `09903620` / `71ff3b74`, measured at `b2c5d693`. A
pinned gap with an exact shape; any *third* policy under these names fails.

## Tasks

**No task pushes a red tree; T1 deliberately commits one.** T1's whole purpose
is the RED demonstration, so `go test ./md/` is red between T1 and T4 and the
only push is T5. (The earlier draft asserted "each task ends green before the
next begins", which T1 contradicts by construction.)

**T1 — the gate, RED first.** Extend `keyedConformanceRecord` in
`md/conformance_keyed_test.go` with `Keys`, and add
`TestKeyedConformanceDescriptorHeadersAgreeWithTheirOrigins` implementing D1,
D2a, D2b, D2c, D3 and D5g's two tiers. Cards and records both come from the
pin-preferring loaders (D5c) — but those arrive with T2, and at T1 no pinned
record exists yet, so **at T1 the pinned tier is empty and D5g's second arm is
unreachable**. That is not a hole: T2 is what populates it, and T2's own gate
re-runs this test. Run against the **stale** corpus and record the output in
the commit message.

*Acceptance: 44 of 46 fail, 2 pass* (`keyed_tr_keyonly`, `keyed_wpkh` — bare
fingerprint origins, depth 0 already right). Not 41: the F-529 three carry
stale descriptors too, and an implementer who tunes the gate until exactly 41
fail would exempt precisely the three vectors this cycle is most exposed on.
Gate: `go test ./md/ -run TestKeyedConformance -v`.

**T2 — the fixtures boundary, in both packages (D5, D5a, D5c–D5g, D6).**

1. Create `gui/vector_fixtures_test.go` and `md/vector_fixtures_test.go`. Each
   owns its record loader (`loadVectorRecord` / `vectorRecordFor`), its card
   loader, and `eachKeyedVector` (glob + name derivation). **Move**
   `loadVectorChunks` out of `gui/taproot_script_path_test.go` into the new
   file — D5d measured that leaving it there blinds the gate to one of the
   three split sites.
2. Pin the fixtures: the pre-re-vendor `keyed_wsh_timelock_hashlock` phrase to
   `md/testdata/forkbuilt/…md1.txt`, and the current `.conformance.json` of
   **all three** F-529 vectors to `md/testdata/forkbuilt/`.
3. Route **both packages** — `gui/` and `md/` (r3 I-1: the earlier draft routed
   only `gui/` while gating both). Six record reads move to the loaders; the
   nine globs, `TrimSuffix` calls and one diagnostic move into the fixtures
   files or are reworded. **Every file that reads a record takes its card from
   the pin-preferring loader too** — including the one line that keeps the
   existing cross-language gate green,
   `md/conformance_keyed_test.go`'s `loadPhraseChunks` → `vectorChunksFor`.
4. Mark the single exemption in its own file (D5f).
5. Add the structural gate per D5b/D5e: token scan, per-directory counts,
   per-directory synthetic known-bad, then the clean-tree assertion.
6. Add `pinnedDuplicateVectors` and a shape test asserting the wsh pin carries
   a slot at two use sites under one miniscript — the existing
   `TestPinnedKeyReuseVectorsAreTheShapeTheyClaim` asserts a *taproot*
   internal-key reuse and cannot cover it. Rework the anti-drift check per D6.

Gate: `go test ./md/ ./gui/ -run 'Duplicate|Pinned|ReachesAnAddress|TaprootScriptPathMatchesRust|WshWitnessScriptHashesToRustsAddress|KeyedConformance|VectorRecord|Boundary' -v`.

**T3 — widen the script and re-vendor (one action, D4).** Three edits, not two
(r3 I-5):

- `scripts/vendor-compose-vectors.sh:16` → `^(keyed_|compose_)` minus
  `compose_refusal_`;
- `md/compose_vectors_pin_test.go:103` — `composeVectorNames` goes **36 → 50**
  names — and `:110-111` — the file literal **176 → 246**. Measured twice
  independently (controller and reviewer): 46 keyed vectors carrying five files
  each plus 4 unkeyed `compose_*` carrying four = 246. The `:106` comment
  becomes "46 keyed vectors carry five files, 4 unkeyed carry four: 246";
- `isComposeVectorFile` (`:79-84`), which still returns false for anything not
  prefixed `compose_`/`keyed_compose_` and drives the **directory scan**. Left
  alone, the 14 newly-pinned vectors gain sha256 coverage and no directory
  coverage — measured by smuggling in an unpinned `keyed_tr_smuggled.…json`,
  which passes unflagged while a `keyed_compose_smuggled` one is caught.

**T3 and the re-vendor are the same action** — the script copies and *then*
hashes the destination (`scripts/vendor-compose-vectors.sh:18,20-22`), so there
is no tree state in which the script is widened, the pin regenerated, and the
corpus still stale. Capture the pre-state first (a copy of the vectors
directory) so the diff expectation below can be checked afterwards.

*Expected, and verified with a count before committing:* exactly 41 records
change descriptor lines only (82 chain entries), 3 change wholesale, 0 address
lines and 0 id lines outside those 3, 0 files added or removed. T1's gate must
now read **43 of 46 correct-header plus 3 pinned-legacy** (D5g) — not 46 of 46,
which is unreachable by construction.
Gate: the full `./md/` and `./gui/` suites.

**T4 — whole-surface gate and push.** `go vet` (ArtifactDir baseline only),
`gofmt -l .` against the **five-file** baseline, `./md/ ./sysw/ ./mk/`, the
whole `gui` via `scripts/gui-shard-test.sh ./gui/ 24`, firmware size. Push via
`scripts/push-via-staging.sh` with main frozen for the window.

## What this plan does NOT cover

- No normative Go behavior changes. T1–T3 touch tests, fixtures, vendored data
  and one shell script. If T3 turns any `./gui/` or `./md/` test red for a
  reason other than a stale expected-value, that is a finding, not a fixup —
  it means the fork and the primary disagree somewhere this plan assumed they
  agreed, and it stops the plan.
- **Four parts of the descriptor string stay outside the gate**, as a
  consequence of D1's header scoping: the BIP-380 checksum, a swap of
  `chains["0"]` with `chains["1"]`, the derivation suffix `/0/*` → `/7/*`, and
  a swap of two `multi()` key positions. The last two are caught as *drift* by
  `TestEveryKeyedVectorReachesAnAddress`, which compares Go-derived addresses
  per chain against the record's; the checksum and the chain-index suffix are
  covered by nothing. Recorded, not scheduled.
- F-529 is **narrowed, not closed**, by T2 + T3: its three vectors get
  fork-side card+record witnesses and the corpus stops diverging silently, but
  whether the device should carry reuse-free or reuse-bearing fixtures for
  F-514's warning is a separate ruling.
- The 17 dm-only `.template` files and the 36 fork-only fixtures are out of
  scope; neither tier feeds the conformance gate.

## Sites depending on `keyed_wsh_timelock_hashlock`

Complete, measured (the earlier draft named three and missed the fourth):

| site | after a re-vendor with no pin |
| --- | --- |
| `md/duplicate_keys_test.go:84` | FAILS loudly (`kind=0, want 1`) |
| `gui/composer_flow_test.go:599` | FAILS loudly (`the predicate says 0 (@0), want 1`) |
| `gui/composer_flow_test.go:684` | FAILS loudly (`no longer carries a miniscript duplicate`) |
| `md/compose_shape_test.go:111` | stays green — same branch shape either way |

D5's rationale in the earlier draft was that these would *go green against a
shape no longer carrying the defect*. Measured, they fail loudly. T2 is still
required — the witness must survive — but it is justified by "the evidence
would be deleted", not by "the assertions would go silent". The distinction
matters: the false version frames T2 as sufficient, and T2 alone is exactly
what produces C-1.
