# R0 ARCHITECT REVIEW — SPEC_hashlock_kinds.md, round 3 (the round-2 fold)

Artifact: the fold `git diff f02d6679..24026605` on `design/SPEC_hashlock_kinds.md`
(mnemonic-engrave `24026605`; spec now **473 lines**, +82/-20).
Round-2 report: `design/agent-reports/spec-hashkinds-R0-round2.md` at `f02d6679`
(1 Critical / 2 Important / 6 Minor / 1 Nit).

Trees measured — **identical to rounds 1 and 2**, so nothing below is drift:
descriptor-mnemonic `40c400de`, mnemonic-engrave `24026605`,
mnemonic-secret `7a0e96f`, seedhammer `0562e811`.
`git status --porcelain` clean in all four, before and after. No tracked file
modified but this report. No probe file created. Two network calls (`gh repo view`
on two public repos, for I-2's feasibility); nothing written anywhere.

Two questions only, per the brief: **did the fold close each round-2 finding**,
and **did the fold introduce a new defect**. Not a fresh audit. §2's operator
decisions, round 1's VERIFIED SOUND list, and findings both earlier rounds closed
were not re-derived.

---

## 1. Citations the fold ADDED or CHANGED — every one checked

| # | § | citation | verdict | measured |
|---|---|---|---|---|
| 1 | §7.3, §11 | `cmd/emu/composer_js.go:54` (both places) | **TRUE** | `:54` `out = append(out, hex.EncodeToString(h[:]))`. Round 2's M-2 off-by-one is fixed at `:235` **and** `:418` |
| 2 | §7.3, §11 | `composer_js.go:15`, `:35-37` (unchanged) | **TRUE** | `:15` `shComposerPathHashes()   [ "<64 hex>" | null, ... ]`; `:35-37` the FULL-64-HEX contract paragraph, exactly |
| 3 | §12.2 | `walk_hashlock_phrase.js:339-345` | **TRUE** | `:339-345` is the "THIS IS WHAT MAKES THE STORED-VERSUS-DISPLAYED ASSERTION FALSIFIABLE" block; the quoted sentence runs `:339-341` |
| 4 | §12.2 | the quote *"Comparing short8(stored) against a constant this file also compares the stored value against is a tautology."* | **TRUE as transcription** | byte-exact against `:339-341` except a terminal `.` where the source has `:`. Noted because this tree has had misquotes introduced by transcription before |
| 5 | §11 | `gui/composer_hashlock.go:247` | **TRUE** | `func hashlockFirst8Last8(h [32]byte) string`; `:248-249` slice a 64-char string length-relatively (`s[:8] + ".." + s[len(s)-8:]`), parameter fixed at `[32]byte` — exactly as the row describes |
| 6 | §11 | `main.rs:2275` | **TRUE** | `"public record {i}: sha256 hashlock (hash:) — {}..{}"` |
| 7 | §11 | `main.rs:2685` | **TRUE** | `C::Hash => "sha256 hashlock (hash:)"` |
| 8 | §11 | `main.rs:3196` | **TRUE** | `is \`hash:\` + the 32-byte digest as 64 lowercase hex` (string spans `:3191-3199`) |
| 9 | §11 | `sysw/composer_records.rs:144` | **TRUE** | `ComposerRecordError::Hash => format!("record {index}: hash: must be exactly 64 hex characters")` |
| 10 | §11 | that string is pinned by `SPEC_wallet_policy_composer.md` §8n | **TRUE** | §8n heading `:809`; the pinned line `:814` `> record N: hash: must be exactly 64 hex characters` |
| 11 | §11 | …**and** by `host_line` rows in `record_class_vectors.json` | **TRUE** | **5** rows carry it verbatim in `crates/me-cli/testdata/record_class_vectors.json`, and the same file is vendored to `seedhammer/sysw/testdata/` |
| 12 | §6 | `composer_records.rs:178-192` = the hex body's uppercase refusal | **TRUE on substance, loose range** | `unhex_lower` is `:176-191`; the uppercase-refusing predicate is `:180` `.all(|b| b.is_ascii_digit() \|\| (b'a'..=b'f').contains(&b))`. The cited range contains it but starts two lines inside the fn and ends on a blank line. See **n3** |
| 13 | §6 | `sysw/composer_records.go:178-190` | **TRUE on substance, loose range** | `unhexLower` is `:174-189`; predicate `:180`. Same shape. It is the real gate: `ParseHashRecord:197` calls it. See **n3** |
| 14 | §6 | "pinned by the corpus's `hash-uppercase` row" | **TRUE** | `record_class_vectors.json:141-145`, record `hash:A8A8…`, class `Unknown`, host_line = row 9's string |
| 15 | §9 | `me-cli/Cargo.toml:54-74` — an unpublished sibling by git rev, rationale written out | **TRUE** | `:74` `mt-codec = { git = "https://github.com/bg002h/mnemonic-transaction", rev = "72b79b87…" }`; `:66-68` *"A git dep works TODAY because the repo is public… keeps `cargo publish mt-codec` DEFERRED. Publishing is irreversible; pinning a rev is not."* |
| 16 | §9 | "twelve lines below" | **FALSE** | `ms-codec = "0.9"` is `:53`; the `mt-codec` line is `:74` = **21** lines below; the comment block begins `:54` = **1** line below. Nothing is twelve. See **n2** |
| 17 | §9 | `main.rs:2636,2641`, `Cargo.toml:53` (unchanged) | **TRUE** | re-verified: `:2636` `Some(ms_codec::hashlock::digest(&x))`, `:2641` the `Payload::Preimage` arm, `:53` `ms-codec = "0.9"` |
| 18 | §11 | `seedhammer/scripts/vendor-compose-vectors.sh:29` | **TRUE** | `"if the file count is not 156, or if the vector names drift from its list.",` |
| 19 | §11 | `seedhammer/md/testdata/compose_vectors.provenance.json:6` | **TRUE** | same sentence, the generated copy. Both in the **fork** — the fold's phase-1→phase-4 correction is right |
| 20 | §11 | the counter-assertion 161 | **TRUE** | `md/compose_vectors_pin_test.go:91` `if len(p.Files) != 161 {` |
| 21 | §6 | `SPEC_descriptor_input.md` §4.6 = the whitespace/CRLF refusals | **TRUE** | `:673` `### 4.6 Whitespace — where the host is deliberately more forgiving` |
| 22 | §6 | …at `:388`, `:529` | **TRUE but not §4.6** | both are refusal-table rows that *point at* §4.6 (`:388` CRLF REFUSE, `:529` leading space/trailing `\n`/CRLF REFUSE). The section itself is `:673`. Inherited from round 1's own evidence. See **n4** |
| 23 | §6 | "Not `SPEC_hashlock_H2_device.md` §4.6, which is the Back contract" | **TRUE** | `:341` `### §4.6 The Back contract (fidelity I-1, journey I-4, I-5)` |
| 24 | §6 | "**two** documents in this directory have a §4.6" | **FALSE** | `grep -rlE "^#{1,6} *§?4\.6" design/*.md` = **8** files: SPEC_descriptor_input, SPEC_hashlock_H2_device, SPEC_engrave_transaction, SPEC_multisig_build_repair, SPEC_seedhammer_slip39_recovery, SPEC_sh2_sysw_consumption, IMPLEMENTATION_PLAN_s6a_singlesig_truth, BRAINSTORM_hashlock_phrase. **New, fold-introduced.** See **m1** |
| 25 | §11 | the `len(raw) != 32` unreachability derivation | **TRUE** | `gui/composer_hash.go:87-89` caps the pad at 64, `:91` `valid := len(frag) == 64`, `:98` `if err != nil \|\| len(raw) != 32`, `:99-102` the comment. Widening the pad to 40 does remove the guarantee, exactly as the fold now says |
| 26 | §10 | all four right-kind-wrong-length cases REFUSED behind one message | **TRUE** | `spec-hashkinds-core-verdicts.md:127-130` — `ripemd160(<64hex>)`, `sha256(<40hex>)`, `hash160(<64hex>)`, `hash256(<40hex>)`, all `REFUSED` / `A function is needed within P2WSH` |
| 27 | §11 | "five sha256-hardcoded operator-facing strings in `me-cli`" | **count off by one** | four **me-cli** locations are cited; the fifth in round 1's list is `SPEC_wallet_policy_composer.md` §8n, which is a design doc, not me-cli, and already has its own §11 row. See **n1** |
| 28 | §10 | "the six hardcoded `[56:]` sites" (unchanged) | **TRUE** | re-grepped production only: `composer_consent.go:63`, `composer_hash.go:50,173,188`, `main.rs:2277,2602`. Exactly six |

**Citations checked this round: 28.** TRUE **23**, loose-but-containing **3**
(rows 12, 13, 22), **false 2** (row 16 "twelve lines", row 24 "two documents"),
plus one count off by one (row 27). Both falsehoods are arithmetic in
parentheticals; neither points an implementer at a wrong file or a wrong line —
which is why the round-2 pattern (a false *file:line* every fold) is broken.

---

## 2. Independent checks the fold's new claims required

Three claims in the fold are not citations but **feasibility assertions**, and
round 2 could not have checked them because the fold had not been written. All
three verified:

1. **`ms-codec` can be consumed by git rev.** `mnemonic-secret` is a **public**
   GitHub repo (`gh repo view bg002h/mnemonic-secret --json isPrivate` →
   `{"isPrivate":false}`), so a git dep needs no deploy key — the same condition
   `Cargo.toml:66` records for `mt-codec` (`mnemonic-transaction` also verified
   public). `ms-codec` is a **workspace member with its own publishable manifest**
   (`mnemonic-secret/Cargo.toml` members `["crates/ms-codec", "crates/ms-cli"]`;
   `crates/ms-codec/Cargo.toml` `name = "ms-codec"`, `version = "0.9.0"`, with
   `description`/`keywords`/`categories`), so cargo resolves it by package name
   inside the repo. **The rev pin works.**
2. **No version conflict.** `ms-codec` has exactly **one** consumer in the
   mnemonic-engrave workspace (`grep -rn ms-codec --include=Cargo.toml` →
   `crates/me-cli/Cargo.toml:53` only; workspace members are `me-cli` and
   `mnemonic-io-lib`). Switching that one edge to a git source cannot strand a
   second registry copy.
3. **`me` genuinely cannot be published today**, as §9 asserts — `mt-codec` is a
   git dep and `mnemonic-io-lib` is a versionless path dep
   (`Cargo.toml:23-25`). Both block `cargo publish`.

So **I-2's remedy is sound**, not merely written down.

---

## 3. Round-2 findings — CLOSED / PARTIAL / NOT CLOSED

### C-2 — KAT pins the four functions, nothing pins the kind→function selection → **CLOSED**

The fold adds §10 `:384-393` and rewrites §12.2 `:446-456`.

**Read as an implementer, it describes something buildable that goes red on a
mis-wired arm.** Walked the constructed failure from round 2 — `case
md.HashKindRipemd160: d = hashlock.DigestHash160(&x)` — against the new text:

- §10 `:391-392`: *"each row is exercised **through the entry point the composer
  itself calls**"*. The KAT therefore calls whatever the composer calls, so the
  mis-wired arm is **inside** the code under test and the ripemd160 row gets
  `hash160(x)` where it expects `ripemd160(x)` → RED. The previous text would
  have called the four functions directly and stayed green.
- §10 `:393` states the **falsifier explicitly**: *"A KAT that only calls the four
  functions directly is green on exactly the failure §4 describes."* That is the
  sentence that makes the requirement checkable at plan-review time rather than
  arguable. If no callable entry point exists, the requirement forces one to be
  extracted — the map cannot remain an anonymous inline switch and still satisfy
  "exercised through".
- §12.2 `:448` now names the source of truth: *"stores that kind **and the §10 KAT
  row's digest for that kind**"*, with `:449-452` explaining why ("the digest it
  composed" is an expectation the device supplies to itself) and citing the
  walk's own tautology argument. The "green **or** red, and the spec does not say
  which" ambiguity round 2 found is gone: the expectation is now external to the
  device.
- The walk is coherent with §7.1: the typed-**phrase** arm asks for the kind, so a
  phrase walk can select ripemd160, derive the preimage (unchanged, out of scope
  per §4) and compare the digest against the KAT row. The route exists.

Residual, recorded as **m4**: "each caller's map gets its own row" does not say
whether a *caller* is a package or a call site. Measured, the residual is small —
`grep -rn "hashlock.Digest(" gui/*.go | grep -v _test` gives **five** production
sites (`composer_hashlock.go:74,143`, `composer_hashlock_plates.go:72,136`,
`composer_hash.go:224`), of which only the two in `composer_hashlock.go` need a
kind at all; §7.1 routes 4 and 5 fix the other three at sha256. Minor, not a
re-open.

### I-1 — routes 4/5 "carry their kind" is false → **CLOSED**

§7.1 `:200-201` now reads *"**carries no hash kind at all** — its `method` field
selects the *preimage* derivation (§4, out of scope), which is a different axis
that happens to share the word `sha256`. The hashlock is sha256."* and *"same: no
hash kind in the grammar, so sha256."* That is round 2's prescribed close,
verbatim in substance: the **answer** the previous fold deleted is restored
inside the new table, and the method-vs-kind distinction is stated.

The fold went further than asked and discharged **N-2** (two things called a "hash
kind") at the source: §6 `:154-156` adds a **Naming** paragraph — *"This
document's 'kind' is always the *hashlock* kind of §5. The `phrase:` record's
`method` field is a different axis…"*.

**Checked for the contradiction the brief asks about.** §2 decision 5 (*"The
phrase route derives all four"*) versus §7.1's *"payload-supplied phrase material
is sha256"* is a **correctly-stated limitation, not a contradiction**:

- §7.1 `:207-211` names it as such, scopes it to *payload-supplied* material, and
  preserves decision 1 (*"Every kind remains authorable via the typed arms"*).
- §4 `:99-106` is consistent: the phrase→**preimage** step is out of scope for
  every kind; the **preimage→digest** step is in scope and gains a kind. Route 4's
  limitation is about which kind is *selected*, not about what the digest function
  does.
- §12.5 (`ms hashlock` reproduces a non-sha256 plate's digest) is consistent:
  §9 phase 2 gives `ms hashlock` the kind as a CLI input, which is a different
  surface from the device's route-5 record. No contradiction.

One imprecision rides along (**m3**): `:208` calls routes 4 **and** 5
"payload-supplied *phrase* material"; route 5 is a **preimage** plate, which this
constellation's own terminology (phrase → preimage → digest) keeps distinct. The
table row directly above it is correct.

### I-2 — the publish gate is an unsound inference → **CLOSED**

§9 `:320-335` retracts it by name (*"It is not a publish gate, and an earlier
draft wrongly said it was"*), gives the right reason (*"a fact about the file
today, not a constraint on what phase 3 may write in it"*), cites the
counter-example with its rationale quoted, and rewrites the order to *"Phase 3
follows phase 2 and pins it by rev."* The phase table row 3 changes from
*"**Requires a released ms-codec**"* to *"Depends on phase 2's API, consumed **by
git rev pin**"*. **No `cargo publish` is scheduled anywhere in the spec** —
verified by grep: the only occurrences are `:328` (the act being declined) and
`:330` (deferred to a future `me` release).

Feasibility independently verified above (§2 of this report). One residue,
recorded as **m2**: the trailing *"`cargo vendor` freshness ritual applies to
phase 3"* is half wrong.

### The round-2 M-2 false citation (`composer_js.go:53` → `:54`) → **CLOSED**

Fixed in **both** places (`:235` and `:418`), verified at row 1.

### Round-1 Minors and Nits that round 2 re-listed as carried → **ALL SEVEN CLOSED**

| round-1 # | subject | status this round |
|---|---|---|
| M-3 | 156/161 assigned to phase 1 | **CLOSED** — §11 `:431-436` reassigns it to **phase 4** and names both files, both verified in the fork (rows 18-20) |
| M-4 | `hashlockFirst8Last8` + five me-cli strings | **CLOSED** — two new §11 rows, all citations verified (rows 5-11); the row even explains *why* it was invisible to the `[56:]` grep |
| M-5 | right-kind/wrong-length and both-spelling vector rows | **CLOSED** — §10 `:395-401`, with the Core measurement attached and verified (row 26) |
| M-6 | §6's unqualified `(§4.6)` | **CLOSED** — `:172-175` names the document and rules out the wrong one (rows 21-23); the parenthetical's count is wrong (**m1**) |
| M-7 | kind-token case stated as description, not rule | **CLOSED** — §6 `:148-152` *"Case is rejected, never folded"*, with the existing hex-body precedent and its corpus row cited (rows 12-14) |
| N-1 | §11's "that stops being true" asserted | **CLOSED** — `:424-427` now derives it (row 25) |
| N-2 | two different things called a "hash kind" | **CLOSED** — §6's new Naming paragraph plus §7.1's rows |

### Round-2's own Minors and Nit — six of seven carried, no reason stated

Not blocking; listed so round 4 (if any) does not re-derive them a third time.

| round-2 # | subject | status |
|---|---|---|
| M-1 | `(§9.1)` dangling | **NOT CLOSED** — `:339` still ends *"(§9.1)"*; headings enumerated, there is no §9.1 (1, 2, 3, 4, 5, 6, 7, 7.1-7.6, 8, 9, 10, 11, 12, 13) |
| M-2 | `composer_js.go:53` | **CLOSED** |
| M-3 | §7.5's four pixel figures with no band | **NOT CLOSED** — §7.5 `:270-287` untouched; still no 411 px band and no post-wrap caveat |
| M-4 | §5's rationale contradicted by §9 | **NOT CLOSED, and its shape changed.** §5 `:132-136` still argues a shared `HashKind` is rejected because it would make the phase order *"a release dependency with a `cargo vendor` freshness ritual attached"* — and §9 now **demonstrates the opposite mechanism** (a rev pin makes a cross-repo code dependency without a release). The stated reason is now refuted by the spec's own §9 rather than merely duplicated by it. Still Minor: the decision holds on other grounds (`md-codec` and `ms-codec` share no edge; the Go port shares nothing), and §2 decision 4 is operator-settled |
| M-5 | route 6's device preset has no stated kind | **NOT CLOSED** — §7.1 `:202` unchanged; `gui/composer_presets.go:54` still has no grammar |
| M-6 | §10 blesses `bitcoin::hashes`, the crate the impl will likely call | **NOT CLOSED** — `:379-380` unchanged. Sharper now than round 2 knew: measured, `ms-codec/Cargo.toml` depends on `getrandom`/`pbkdf2`/`sha2`/`zeroize` and has **no** ripemd160 primitive, so the Rust implementation must add one, and `bitcoin::hashes` is the obvious candidate |
| N-1 | "~1200 other tests" | **NOT CLOSED** — `:369` unchanged; measured 1319 |

---

# CRITICAL

**None.**

# IMPORTANT

**None.** I looked specifically for the three shapes the brief names — an
unclosed C-2, an I-1 contradiction between §2 decision 5 and §7.1, and an
infeasible git-rev pin — and constructed against each. All three hold. The
remaining findings are arithmetic, terminology and carried Minors; none changes
what an implementer builds or what a gate catches.

---

# MINOR

## m1 — "two documents in this directory have a §4.6" is false; measured eight

§6 `:174-175`. The sentence's whole job is to disambiguate a cross-document
reference, and its factual claim is wrong:

```
$ grep -rlE "^#{1,6} *§?4\.6" design/*.md | wc -l
8
```

SPEC_descriptor_input, SPEC_hashlock_H2_device, SPEC_engrave_transaction,
SPEC_multisig_build_repair, SPEC_seedhammer_slip39_recovery,
SPEC_sh2_sysw_consumption, IMPLEMENTATION_PLAN_s6a_singlesig_truth,
BRAINSTORM_hashlock_phrase. The disambiguation itself is correct — the right
document and the right content are both named — so no implementer is misdirected;
what is wrong is a hand-count in a repo whose standing rule is *never hand-count
what a tool can count*. Fix: drop the count, or say "several".

## m2 — §9 attaches the `cargo vendor` ritual to phase 3, which has no vendor tree; the phase that needs it is phase 2

§9 `:334-335`: *"The `Cargo.lock` / `cargo vendor` freshness ritual applies to
phase 3's dependency change."* Measured:

- **mnemonic-engrave (phase 3) has no vendor tree and no vendor gate.** No
  `vendor/`, no `.cargo/config.toml`, and `.github/workflows/` contains only
  `release.yml`, which has zero occurrences of `vendor`. The `Cargo.lock` half
  *does* apply — `release.yml:151` runs `cargo test --locked` and `:116`
  `cargo clippy --all-targets --locked`, both of which fail on an out-of-sync
  lockfile. The `cargo vendor` half has nothing to act on.
- **mnemonic-secret (phase 2) has both**: a committed 103 MB `vendor/` tree
  (`git ls-files vendor` is non-empty), `ci/repro/vendor-freshness.sh`, and
  `.github/workflows/vendor-freshness.yml`, whose own header says it REDs *"iff
  the committed `vendor/` tree cannot satisfy the current `Cargo.lock`"* and
  exists because a forgotten re-vendor broke a tag-triggered `--offline --locked`
  release.
- **Phase 2 is the phase that necessarily moves a lockfile.** `ms-codec` has no
  ripemd160 primitive today, so implementing `ripemd160` and `hash160` adds a
  dependency; phase 3's change is a *source* swap on an existing edge.

Minor rather than Important: the ritual is named, a script and a CI workflow
exist for it in the repo that needs it, and the phase-3 half that does apply
(`Cargo.lock` + `--locked`) is enforced by two CI steps. But the repo's own
records note the vendor-freshness check is **not a required** status context, so
a stale vendor tree surfaces at tag time rather than at PR time — which is why
naming the owning phase correctly is worth one clause: *"phase 2 re-vendors
mnemonic-secret (`ci/repro/vendor-freshness.sh`); phase 3 carries only the
`Cargo.lock` change."*

## m3 — §7.1's new limitation paragraph calls a preimage plate "phrase material"

`:208`: *"routes 4 and 5 are *payload-supplied* phrase material"*. Route 5 is the
**preimage-plate** record — an ms1 string decoding to
`ms_codec::Payload::Preimage` (`main.rs:2640-2641`), not phrase material. This
constellation's terminology is deliberately three-stepped (phrase → 32-byte
preimage → digest) and the paragraph immediately above it is the one that
separates axes for a living. The table row at `:201` is correct, so the damage is
contained to the summary sentence. Fix: *"routes 4 and 5 are payload-supplied
material"*.

## m4 — "each caller's map gets its own row" leaves per-package vs per-call-site open

§10 `:392`. The mechanism is stated and falsifiable (see C-2 above), but "caller"
is undefined. Measured in the fork: `hashlock.Digest(` has **five** production
call sites in `gui/` — `composer_hashlock.go:74,143`,
`composer_hashlock_plates.go:72,136`, `composer_hash.go:224`. Under §7.1 only the
two in `composer_hashlock.go` (the phrase arm) need a kind; the other three are
routes 4/5 and stay sha256. So the natural implementation — one named
`DigestFor(kind, x)` in `hashlock`, called from the kind-bearing sites — satisfies
§10 and is covered by one row set. A scattered implementation with an inline
switch per site would satisfy the letter and not the intent. One clause in the
implementation plan (*"the map is a single named function per package; no inline
kind switches at call sites"*) closes it. Minor because the failure mode is loud
at plan review, and because §10's falsifier sentence already rules out the
degenerate KAT.

---

# NIT

**n1 — §11's "five sha256-hardcoded operator-facing strings in `me-cli`" cites
four me-cli locations.** `main.rs:2275,2685,3196` + `sysw/composer_records.rs:144`
= four. Round 1's fifth was `SPEC_wallet_policy_composer.md` §8n, which is a
design document, not me-cli — and §11 already carries a separate row for *"§8 spec
rows for every new string"*. Say "four", or move §8n into the row.

**n2 — §9's "twelve lines below" is wrong.** `ms-codec = "0.9"` is
`Cargo.toml:53`; the `mt-codec` git line is `:74` (21 below) and its comment block
opens at `:54` (1 below). The range citation `:54-74` is correct and is what an
implementer follows, so the number is decoration — but it is a hand-count in a
sentence that exists to make a measured point.

**n3 — both uppercase-refusal ranges in §6 are off at the edges.** Rust
`unhex_lower` is `composer_records.rs:176-191` (cited `:178-192`, ending on a
blank line); Go `unhexLower` is `composer_records.go:174-189` (cited `:178-190`,
same). Both ranges *contain* the load-bearing predicate at `:180`, so an
implementer lands in the right function. Citing `:180` in each would be exact.

**n4 — §6's `SPEC_descriptor_input.md` §4.6 (`:388`, `:529`) points at two table
rows, not at §4.6.** Both lines are whitespace/CRLF **REFUSE** rows that reference
§4.6; the section heading is `:673` (`### 4.6 Whitespace — where the host is
deliberately more forgiving`). Not false — those lines are the refusals the
sentence describes — but a reader following the citation to find §4.6 lands
somewhere else. Inherited verbatim from round 1's evidence block.

**n5 — §10 still says "six" formatters while §11 now names a seventh.** §10
`:362` *"The six hardcoded `[56:]` sites"* and `:365` *"All six take a fixed
array"* are both still exactly true (re-measured: six sites), and §11's new
`hashlockFirst8Last8` row explains why it is outside that grep. But §10's
governing phrase is *"every formatter"* and §12.3 says *"every formatter"*, so the
enumeration and the rule now differ by one. One parenthetical in §10 (*"seven with
`hashlockFirst8Last8`, §11"*) reconciles them.

**n6 — §12.2's spliced sentence left a 200-column line.** `:452` runs the new
quote and *"The assertion is the acceptance…"* together without a break.
Cosmetic; every other paragraph in the file wraps.

---

# What this round verified and what should not be re-derived

1. **The rev-pin remedy is feasible, not just written.** mnemonic-secret is
   public, `ms-codec` is a publishable workspace member, and `me-cli` is its only
   consumer in the workspace. Do not re-check these.
2. **No `cargo publish` is scheduled by the spec.** Verified by grep over the
   whole file.
3. **C-2's remedy names a mechanism AND a falsifier.** The falsifier sentence
   (§10 `:393`) is what makes it checkable at plan review; that is the part worth
   preserving through any later edit.
4. **All seven round-1 Minors/Nits are closed**, each against its own prescribed
   remedy, with every supporting citation verified here.
5. **The fold broke the two-round pattern of introducing a false `file:line`.**
   Its two false claims this round are both arithmetic in parentheticals; no file,
   line, symbol or range it added is wrong.
6. **The 156/161 phase reassignment is right** — both files measured in the fork.

---

# Counts

| severity | n |
|---|---:|
| Critical | 0 |
| Important | 0 |
| Minor | 4 |
| Nit | 6 |

Round-2 disposition: **C-2 CLOSED**, **I-1 CLOSED**, **I-2 CLOSED**, the new
false citation **CLOSED in both places**, all seven carried round-1 Minors/Nits
**CLOSED**. Six of round-2's own seven Minors/Nits carried without a stated
reason (M-1, M-3, M-4, M-5, M-6, N-1); none gates.

# Verdict

**GREEN (0 Critical / 0 Important).**

I tried to break three things and could not. **C-2**: the constructed mis-wire
(`case HashKindRipemd160: DigestHash160(&x)`) now fails the KAT, because §10
requires the rows to run through the composer's own entry point and states in
terms that a KAT calling the four functions directly is green on exactly that
failure — and §12.2 no longer lets the walk supply its own expectation. **I-1**:
§2 decision 5 and §7.1 are consistent, not contradictory — the phrase *route*
derives all four through the typed arm, and the limitation is confined to
payload-supplied material, stated as such, and consistent with §4's scope boundary
and §12.5's different surface. **I-2**: the rev pin is not merely a better
schedule, it is a buildable one — public repo, publishable workspace member, one
consumer, no version conflict, and `me` unpublishable today regardless.

What remains is four Minors and six Nits, of which the two that are actually
*wrong* (an eight-document set called two, and a twelve that is twenty-one) are
parenthetical arithmetic in sentences whose substance is correct. Neither sends an
implementer anywhere. Recorded for the implementation plan, not for another round.

Per this repo's closure rule, this closes the **correctness / did-the-fold-hold**
lens. It does **not** assert the spec is sound under lenses never run against it —
a journey walk and a does-the-walk-run pass are first-time questions, not harder
looks at this one, and the plan that follows will have its own gates.
