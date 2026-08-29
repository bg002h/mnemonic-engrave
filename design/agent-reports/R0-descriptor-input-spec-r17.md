# R0 round 17 — VERIFICATION of the r16 fold

**Target:** `design/SPEC_descriptor_input.md` at `6a12beb` ("spec: fold R0 r16 --
the shape gate's two disjunct bugs"). `6a12beb` is HEAD; tree clean (`git status
--short` empty).
**Source of truth for what was required:**
`design/agent-reports/R0-descriptor-input-spec-r16.md` (0C/2I/1M, 7/7 of r15's
findings fixed, 60/60 decision-table cells single-valued).
**Scope, as briefed:** (1) disposition r16's three findings by re-deriving the
gate column only; (2) verify by diff that nothing outside the two edits moved;
(3) re-run the two standing sweeps. Everything from r1–r16, **including the
60-cell table and §4.5's fifteen-row near-miss table**, taken as settled. **No
fresh audit was performed** — every check below is downstream of one of the two
edited hunks.
**Reviewer:** independent context, opus tier. Read-only; nothing modified,
committed or pushed.
**Diff read in full:** `git diff 52c177a..6a12beb` — the fold commit `6a12beb`
touches **one file, 2 hunks, +12/−4**; the range's other commit `eae3b7c` is the
r16 report's own persist (+464, report only).
**Binaries used:** `/home/bcg/.cargo/bin/me` (`me 0.7.0`), `cargo-nextest`
0.9.140, `python3` for the sweeps.

---

## Counts — NEW findings

| severity | count |
| --- | :-: |
| **Critical** | **0** |
| **Important** | **1** |
| Minor | 2 |
| Nit | 1 |

**The spec does NOT re-close GREEN this round.** All three of r16's findings are
fixed at the sites they name, the diff is clean, and both standing sweeps are
unchanged. But the corrected format-4 test covers **14 of §4.5's 15 rows** — it
misses row 13, an **ACCEPT**, and the miss generalises to every origin-less key
carrying a use-site path, i.e. four of §4.7 conjunct 7's five admitted use-site
shapes. That is r16's new-I1 defect one disjunct over: the fix cites *"§4.5's
own ACCEPT rows"* as its warrant and does not cover them all.

---

# Disposition of r16's three

| finding | verdict | evidence |
| --- | --- | --- |
| **new-I1** (the gate dropped §6 rule 4's `[` disjunct, so every origin-annotated bare key was routed to the exit-4 record refusal) | **FIXED for the filed rows; the same test still under-covers one ACCEPT row — new-I1 below** | The mis-citation *"§6's rule-4 shape tests"* is gone, replaced by *"aligned with §6's cause-selection steps, corrected per R0 r16"*, and the `[` disjunct is restored: *"a single token that is a 78-byte base58check payload **OR begins with `[`**"*. I re-derived all fifteen §4.5 rows against the four tests (table below): rows 5–12, the eight `[`-leading ones r16 filed, now pass. Row **13** (`xpub…/<0;1>/*`, children, no origin — **ACCEPT**) still fails all four. |
| **new-I2** (the gate inherited §6 rule 2's bare-`": "` looseness, so `seed:`-prefixed and `text:`-prefixed records would hear descriptor vocabulary at exit 3) | **FIXED** | The `": "` test is now keyed: *"a line whose `": "` key is a BlueWallet header (`Name`/`Policy`/`Derivation`/`Format`) or an 8-hex fingerprint — a bare `": "` is NOT enough"*. The four header names are exactly `parseBlueWalletDescriptor`'s recognised set (`nonstandard/parse.go:103–124`, `switch key` arms `Name`/`Policy`/`Derivation`/`Format`), and the cosigner key is the hex fingerprint (`default` arm, `hex.DecodeString(key)`) — §4.2's own list, spelled correctly. All four of r16's exemplars keep the shipped exit-4 record refusal, re-measured at `6a12beb` (`me 0.7.0`): `seed: abandon abandon abandoz` → **rc=4**, `text: hello` → **rc=4**, `pass: x` → **rc=4**, `this is not a record of any class` → **rc=4**. `seed`/`text`/`pass` are none of the four headers and none is 8 hex characters, so the gate excludes each one. A bare multi-word mnemonic bears no `(`, no `": "`, is not JSON and is not a single token — excluded by all four tests. The pinned test `crates/me-cli/tests/sysw_cli.rs:1928` re-run green: `PASS [0.003s] … 1 passed, 440 skipped`. |
| **new-M1** (the exemption's stated reason — *"routing nowhere"* — was false about the only site it governs) | **FIXED in its first clause; the second clause is not addressed — new-M2 below** | §5.3 now reads *"NEITHER-PATH refusals are exempt: substituting "wait for the update" into a refusal whose truth is "never, in any build" would be false"*. Checked against the one exempted text (§6 L1267's `multi`-form replacement, referenced again by L1285): its verdict is *"No `me` path engraves this file as written, in any build"*, so the stock replacement's *"it packs when the device update ships"* would contradict it — **the restated reason is TRUE of the site**, and a future reader applying the stated test now gets the right answer, which was r16's complaint. The exemption's **trigger** (*"NEITHER-PATH refusals are exempt"*) is byte-identical across the diff, so the reach re-derivation below is a confirmation, not a re-computation. r16's second clause — that §5.3's closing sentence now carries an unnamed semantic exception — is untouched. |

**Three of three fixed at the sites they name.** No fix was cosmetic and none
closed a finding by deleting the claim.

---

# The gate column, re-derived

The gate at §5.1, verbatim, as the four tests I evaluated:

> **T1** a `(`-bearing expression; **T2** a line whose `": "` key is a BlueWallet
> header (`Name`/`Policy`/`Derivation`/`Format`) or an 8-hex fingerprint;
> **T3** a single token that is a 78-byte base58check payload OR begins with `[`;
> **T4** JSON with a descriptor field.

## (a) §4.5's ACCEPT rows with `--as` omitted — 14/15 rows pass, the miss is an ACCEPT

Row set enumerated mechanically from the table (`| input | verdict | why |`
header, 15 rows — count is the tool's, not mine):

| # | §4.5 row | verdict | test that fires |
| --: | --- | :-: | --- |
| 1 | bare `xpub…` | ACCEPT | T3 (78-byte payload) |
| 2 | bare `zpub…` | ACCEPT | T3 |
| 3 | bare `Zpub…` | REFUSE | T3 (any version, per §6 step 4) |
| 4 | bare `Ypub…` | REFUSE | T3 |
| 5 | `[4bbaa801/44'/0'/0']xpub…` | ACCEPT | T3 (`[`) — **restored by the fold** |
| 6 | `[4bbaa801/49'/0'/0']xpub…` | ACCEPT | T3 (`[`) |
| 7 | `[4bbaa801/84'/0'/0']zpub…` | ACCEPT | T3 (`[`) |
| 8 | `[4bbaa801/86'/0'/0']xpub…` | REFUSE | T3 (`[`) |
| 9 | `[4bbaa801/48'/0'/0'/2']xpub…` | REFUSE | T3 (`[`) |
| 10 | `[4bbaa801/84'/0'/1']zpub…` | REFUSE | T3 (`[`) |
| 11 | `[4bbaa801/84/0/0]zpub…` | REFUSE | T3 (`[`) |
| 12 | `[4bbaa801]xpub…` | REFUSE | T3 (`[`) |
| **13** | **`xpub…/<0;1>/*` (children, no origin)** | **ACCEPT** | **NONE — see new-I1** |
| 14 | `xpub…\n` | REFUSE | T3 (single token after tokenising; §4.6 trims first) |
| 15 | bare `tpub…` | ACCEPT (cascade) | T3 |

## (b) The shipped record refusals are preserved

Measured at `6a12beb`, `me 0.7.0`, one invocation each:

| input | rc today | shipped message class | gate verdict | outcome |
| --- | :-: | --- | :-: | --- |
| `seed: abandon abandon abandoz` | 4 | "not a form this container can place" | not descriptor-shaped | exit 4 preserved |
| `text: hello` | 4 | "begins `text:`… body is not lowercase hex… RESERVED" | not descriptor-shaped | exit 4 preserved |
| `pass: x` | 4 | "begins `pass:`… RESERVED" | not descriptor-shaped | exit 4 preserved |
| a mistyped bare mnemonic (`this is not a record of any class`) | 4 | "not a form this container can place" | not descriptor-shaped | exit 4 preserved, test green |
| `Name: foo` | 4 | "not a form this container can place" | **descriptor-shaped** (T2) | exit 3, branch-1 error — intended by the fold |
| `deadbeef: xpub6ERAp…` | 4 | "not a form this container can place" | **descriptor-shaped** (T2) | exit 3, branch-1 error — intended (a headerless BlueWallet export) |

The two intended flips are the fold's purpose and both improve the message; the
four r16 exemplars are all preserved.

## (c) Coverage of §6's shapes — under and over

**Happy paths, four formats:** BlueWallet → T2 (any `Name`/`Policy`/
`Derivation`/`Format` line; the gate scans *a line*, not only the first
non-comment line, so a leading `# BlueWallet…` comment does not defeat it);
plain BIP-380 → T1; `{label, descriptor}` JSON → T4; promoted bare key → T3
**for the childless spelling only** (new-I1).

**§6's branch-4 rows:** *"a bare key whose path matches no script"*, *"a bare
key at account ≠ 0"*, *"a bare key with a fingerprint and no path"* — all
`[`-leading, T3 ✓. *"a bare `Zpub`/`Ypub`"*, *"a bare `tpub`"*, and the bare-key
arm of the `ypub`/`upub`/`vpub`/`Upub`/`Vpub` row — all 78-byte base58check
envelopes of some version, T3 ✓.

**Not a loss, checked and recorded:** §6's short-fingerprint BlueWallet row
(`ab: xpub…`) fails T2 in isolation, but any real file carrying that defect also
carries `Name:`/`Policy:`/`Format:` lines, so the row stays reachable; and §11
item 4's tests reach every §6 row with `--as` present regardless.

**Over-coverage:** T4 is narrower than §6 step 1 (requires a `descriptor` field
rather than any JSON) and every §4.4 shape it drops carries a `(` and is caught
by T1 — no loss. T3's `[` disjunct is narrower than §6 step 4's *"first
non-whitespace character"* (it requires a single token) — no §4.5 shape is lost,
since every origin-annotated key is one token. T1 is **not** narrowed, and that
is new-M2.

## (d) The exemption's restated reason, against its one site

The exempted text is §6 L1267's `multi`-form replacement (L1285 incorporates it
by reference — *"the `multi`-form remedy replacement of the previous row applies
here identically"*, so one text, two rows):

> *"this is a `multi` policy, which only `--as md1` carries — and md1 cannot
> represent `/0/*`. No `me` path engraves this file as written, in any build.
> Re-export with `<0;1>/*` — carried in every build. (Re-exporting as a
> `sortedmulti` policy keeps `/0/*` but is a DIFFERENT policy — `me` will not
> rewrite it — and needs the scannable-plate path.)"*

The refusal's truth about the operator's file **is** *"never, in any build"* —
stated in its own second sentence — so substituting *"keep the export file; it
packs when the device update ships"* would produce a text that contradicts the
verdict two sentences above it. **The restated reason is true of the site, and
it selects the site correctly.** r16's residual second clause is new-M2 below.

**Substitution reach, re-derived (the trigger did not change, so this confirms
rather than recomputes):** the sites whose refusal text routes to the descriptor
path are §5.3(a)'s and (a″)'s directive sentences, §6 L1267's and L1285's main
remedies (*"Use `--as descriptor`, which carries … exactly"*), and the one
`multi`-form replacement. The first four take the substitution; the last is the
sole exemption. §6's `wsh(multi)`-under-`--as descriptor` row and the
single-key-wrapper `multi` remedy route to `--as md1`, which ships in the window;
the `ypub` family row routes to a re-spelled *input*, not to a flag. Reach
unchanged from r16's enumeration.

---

# NEW findings

## new-I1 (Important) — the format-4 test covers 14 of §4.5's 15 rows; the missed row is an ACCEPT, and the miss is every origin-less key that carries a use-site path

**Where.** §5.1's T3 (*"a single token that is a 78-byte base58check payload OR
begins with `[`"*), against §4.5's row 13 and §4.7 conjunct 7.

**The row.** `| `xpub…/<0;1>/*` (children, no origin) | ACCEPT → `pkh(xpub…/<0;1>/*)` | children do not affect the origin comparison |`

**Why it fails all four tests.** No `(` (T1). No `": "` (T2). Not JSON (T4). It
**is** a single token, but it does not begin with `[`, and it is **not** a
base58check token — `/`, `<`, `;`, `>`, `*` and the digit `0` are all outside
the base58 alphabet, so nothing in it decodes to a 78-byte payload (T3). §6's
step 4 says the same thing in its own words — *"it is a single base58check token
whose payload is 78 bytes"* — so citing §6 verbatim would not have closed this
either.

**Measured**, `6a12beb`, `me 0.7.0`, the row's shape with a real key:

```
$ me sysw pack 'xpub6ERApfZwUNrhLCkDtcHTcxd75RbzS1ed54G1LkBUHQVHQKqhMkhgbmJbZRkrgZw4koxb5JaHWkY4ALHY2grBGRjaDMzQLcgJvLJuZZvRcEJ/<0;1>/*'
rc=4
me: record 0 (records count from 0) is not a form this container can place: not a
BIP-39 mnemonic, not an md1/mk1/ms1/mt1 string, and not a `text:`/`pass:`/`tx:`
record. Descriptors and addresses are not yet classifiable here — see sysw::classify
```

**The mechanism, read from the source rather than inferred.**
`bip380.ParseKey` (`third_party/seedhammer/bip380/bip380.go:390–398`) cuts the
children expression off at the first `/` **before** `ParseExtendedKey` sees the
token, and (`:403–408`) falls back to the SLIP-132 version byte whenever no
origin was supplied. `parsePath` (`:464–504`) accepts `*`, `<i;j>` and fixed
indices. So children genuinely do not affect promotion — §4.5's row is right,
and it is right for **every** children expression, not just `<0;1>/*`.

**The generalisation, which is what makes it more than one row.** §4.7 conjunct
7 admits the closed set `{absent, /*, /i/*, <i;i+1>, <i;i+1>/*}`. Only `absent`
leaves an origin-less promoted key as a bare base58check token. The other four
all fail T3. **For format 4 without an origin prefix, the gate admits exactly
the childless spelling** — and §5.3(a′) exists precisely because the childless
spelling is the one the host has to *materialise* a use-site path into, so the
operator who writes the path out explicitly is doing the more careful thing and
is the one refused.

**Consequence, identical in kind to r16's new-I1.** An input that is
cascade-ACCEPTed, §4.7-admitted, and carried by **both** paths (`<0;1>/*` is
md1-representable) is unreachable through the documented `--as`-omitted
discovery path, and hears a message that names BIP-39, md1/mk1/ms1/mt1 and
`text:`/`pass:`/`tx:` records, never a descriptor route and never `--as`. That
is the status quo §1 and §2.1 exist to remove.

**It is a mandated vector row, not a shape I invented.** §7's required row set
includes *"the promotion near-misses of §4.5 — all **fifteen** rows of that
table"*, with the `promotion-near-miss` tag carrying a minimum of 15 in the
coverage manifest. Row 13 is one of the fifteen.

**And the same claim eleven lines below the gate is still false.** §5.1 asserts
the choice block fires *"matching §11 item 5's tested cases **across all four
formats**"*. A conformant implementation satisfies §11 item 5 with a bare
`xpub…` witness — which passes T3 — so, exactly as in r15's new-M2 and r16's
new-I1, **the acceptance gate cannot execute against this defect**.

**One further routing loss from the same shape, recorded rather than filed
separately because one edit closes both:** §6's cause-selection step 4 has the
identical token test, so a *refusing* origin-less key with children (e.g.
`Zpub…/<0;1>/*`) falls through to step 5's generic five-form message instead of
branch 4's specific bare-key text. That text is true, so it is not itself a
finding — but any repair to T3 should be mirrored in §6 step 4, or the two
diverge again.

**Not prescribing the fix.** The token test needs to see through a `/…` tail —
i.e. test the leading base58 run rather than the whole token — but the same
edit has to reach §6 step 4, and it must not widen far enough to swallow
`text:`-style records (new-M2's class).

---

# Minor

**new-M2 — §5.3's closing absolute still carries the unnamed semantic
exception r16's new-M1 named in its second clause.** The sentence reads *"No
refusal names a flag that refuses in the current build."* The exempted text's
tail names *"the scannable-plate path"*, which under §5.3's own reading rule
(*"naming the flag or otherwise — semantic, not lexical, per R0 r14's
new-M3"*) **is** the `--as descriptor` flag, in a build where that flag refuses.
The ruling stays right — the clause is a caveat against a re-export, not an
offer, and substituting the stock text there would be false — but the absolute
that closes the paragraph is contradicted by the exemption stated two lines
above it. One word repairs it (*"names"* → *"offers"* / *"points the operator
at"*), or the sentence carries the exception explicitly.

**new-M3 — the `(` disjunct keeps the looseness the `": "` disjunct just
lost.** T1 is *"a `(`-bearing expression"*, aligned to §6 step 3's *"input
contains `(`"*, and like every other test it is evaluated over the whole input.
So any unclassifiable record bearing a parenthesis is declared
descriptor-shaped. Measured today at `6a12beb`:

```
$ me sysw pack 'text: my wallet (2 of 3)'
rc=4
me: record 0 … begins `text:`, but its body is not lowercase hex. That prefix is
RESERVED, so a body it cannot decode is refused rather than quietly engraved …
$ me sysw pack 'pass: hunter (2)'
rc=4   (same RESERVED-prefix message)
```

Under the gate both become **exit 3** with *"this is not a wallet descriptor in
any of the four forms `me` reads"*, and the precise reserved-prefix guidance is
lost — the same regression shape as r16's new-I2, at both the exit code and the
text, one disjunct over. It also reaches a multi-record input: a good mnemonic
plus a malformed paren-bearing `text:` record is pulled whole into the
descriptor path.

**Rated Minor, not Important, and the reasons are load-bearing:** the text
printed is *true* (the input genuinely is not a descriptor), no admitted wallet
becomes unreachable, and the absolute the spec states — *"a mistyped mnemonic
word must never hear descriptor vocabulary"* — is not falsified, because a
mnemonic bears no parenthesis. A test that excludes any line whose key is a
RESERVED record prefix (`text:`/`pass:`/`tx:`) closes it, and would have closed
r16's new-I2 as well.

---

# Nit

**new-N1 — the pinned test's operand is not a mnemonic.** §5.1 cites
*"pinned by a green test (`sysw_cli.rs:1928`): a mistyped mnemonic word must
never hear descriptor vocabulary"*. The test at that line is
`an_unpackable_record_is_refused_before_a_passphrase_is_minted` and its operand
is the literal `"this is not a record of any class"`; it asserts the stderr
contains `not a form this container can place`. The exemplar and the operand are
in the same class — colon-free, multi-word, gate-excluded — so the guarantee
does hold and the test is a genuine witness for it. The sentence just reads as
if the test used a mnemonic. r16 verified the citation's line numbers and
function name and re-ran it green; the operand was the one thing left unchecked.

---

# Standing sweeps

| sweep | method | result |
| --- | --- | --- |
| **quoted spans carry no internal identifiers** | multi-line extraction of all `*"…"*` spans, matched against `§\d｜F-\d{3}｜R0｜NEW-[A-Z]\d｜new-[A-Z]\d｜walk W\d｜conjunct \d｜EXIT_｜r1[0-6]\b｜carriage rule｜window substitution` | **45 spans, 0 violations** — identical to r16 at `52c177a`; the fold added no quoted text |
| **substitution reach** | enumerate every refusal text routing to the descriptor path; check the exemption removes exactly the NEITHER-PATH one | 5 substitution sites + 1 exemption (referenced by 2 rows). Unchanged — the trigger sentence *"NEITHER-PATH refusals are exempt"* is byte-identical across the diff; only the parenthetical reason moved |
| **diff containment** | `git show --stat 6a12beb` | 1 file, 2 hunks, **+12/−4**, both hunks are the two briefed edits; `eae3b7c` in the same range is the r16 report's persist (+464, report only). **Nothing outside the two edits changed.** |
| **pinned test** | `cargo nextest run --locked -E 'test(an_unpackable_record_is_refused_before_a_passphrase_is_minted)'` | `PASS [0.003s]` — 1 passed, 440 skipped |

---

# Verdict

**0 Critical / 1 Important / 2 Minor / 1 Nit. The spec does not re-close GREEN.**
One more fold is needed, and it is small: T3 must see past a `/…` tail (with §6
step 4 mirrored), plus the two Minors, which are one clause each.

The walk lens is complete and the decision table stays at 60/60 — this round
moved neither. The open Important lives on §4.5's row 13, a class the 60-cell
table never enumerated and the same blind spot r15 named and r16 landed in: the
gate is the one construct in the spec that must be *exhaustive over an input
space*, and three consecutive folds have each covered it one shape short.

**A note on how this defect keeps recurring, offered for the fold rather than as
a finding.** r15 wrote the gate from §6's ordering; r16 corrected it row by row;
this round finds the row that neither enumerated. Writing the gate as a
*derivation* — "descriptor-shaped ⇔ some branch of §4's cascade could match" —
with the four tests as the stated consequence, would make the next missing shape
a contradiction rather than an omission. The exhaustive check is cheap and
mechanical: fifteen rows in §4.5, four happy paths in §4.2–4.5, and §4.7
conjunct 7's five-member use-site set.

## What the spec's own text leaves open regardless of this round

Recorded so the plan phase inherits it rather than rediscovering it — none of
these gate the GREEN, and none was re-verified this round:

- **§9 residuals 1–7**, unchanged: nothing has run on hardware; the three
  admission-table cells have never been exercised (§9's own
  gate-that-never-executed note); change-chain and testnet address equality
  unmeasured; `md-cli` at repo HEAD vs published `md-codec` 0.42.0 not proven
  byte-identical; TinyGo build of a new `sysw.Classify` arm unchecked; the
  negative claims' search scope named and bounded.
- **§9 item 7 reads stale.** It says §6's refusal texts *"have not been walked
  with the operator"*, but it was written at `ff9a0f2`, before the walk fold at
  `d0647f4` (15 findings, two live journeys), and §6's rows now cite walk
  W5/W11/W13 and F-419 as *"written from the walk"*. What remains true is the
  narrower claim — no systematic row-by-row walk of §6 has been done. Outside
  this round's scope; flagged for the fold to either narrow or discharge.
- **Parked with S2** (F-418, needs the device on the bench): §11 item 1's
  `sysw.Classify` arm and item 6's on-device `ClassDescriptor` display; §6's
  `--as descriptor`-only refusal rows (§11 item 4).
- **Plan-phase items:** §7's vector file does not exist yet (49-row floor,
  8-tag manifest, one sha256 pinned in both repos); F-414 (descriptor + other
  records in one container), F-416 (`SPEC_systemwide_payloads` §5.6's `--in`
  amendment), F-413 (host-side version-byte normalisation), F-422 (the Specter
  question, awaiting an operator ruling).
