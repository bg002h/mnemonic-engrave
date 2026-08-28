# R0 review — `SPEC_descriptor_input.md`, round 4 (proportional re-review of the r3 fold)

**Artifact:** `design/SPEC_descriptor_input.md` at `8409616` (1314 lines).
**Scope, as briefed:** (1) did the fold close each of r3's eight findings; (2) did the fold
introduce new defects — **only in what changed**. Not a fresh audit. r1's verified-TRUE table,
all r1/r2/r3 measured probe results (including B1/B2/B4 and their addresses), the citation gate,
§8's phase order, the operator rulings and the r1→r2→r3 dispositions were taken as settled.
**Reviewer:** independent agent, opus tier. Read-only; `git status --porcelain` returns 0 lines
in all three repos (`mnemonic-engrave`, `seedhammer`, `descriptor-mnemonic`) after the round.

## Counts — NEW findings only

**0 Critical / 1 Important / 4 Minor / 2 Nit**

**Disposition of r3: 8 FIXED, 0 PARTIAL, 0 NOT FIXED.**

**The fold-scoped lens CLOSES: nothing the fold authored is Critical or Important.** Every one of
r3's eight findings is closed by re-measurement, and the fold's central new object — per-key md1
quantifiers backed by `TLV_USE_SITE_PATH_OVERRIDES` — is not merely consistent but **measured
implementable end to end**, which r3 could only assert for one mixture. The four Minors are
residues inside the fold's own new sentences.

**The one Important is PRE-EXISTING TEXT, surfaced by the brief's own `md1_admits` question**
(*"does any required row still lack a way to express its assertion?"* — yes, `wsh(multi(…))`).
It is not attributable to this fold, but under the project severity rule it blocks independently
of who authored it. The controller decides whether it opens a round; the lens on the fold is
closed either way.

**Measurement environment.** Go probes: scratch module at `…/scratchpad/goprobe4` with
`replace seedhammer.com => /scratch/code/shibboleth/seedhammer`, built with
`/nix/store/33fw5m31lfcnk4ff2f0df7j2bxnh8lgk-go-1.26.3/bin/go`; the fork tree was never written
to. md1 probes: `/scratch/code/shibboleth/descriptor-mnemonic/target/release/md` (the tree-built
binary, per §2's stale-binary note), driven from a Python harness. Keys `K1`/`K2` are the fork's
own `nonstandard/parse_test.go` cosigners (`dc567276` / `f245ae38`).

---

## Disposition table — r3's eight findings

| # | r3 finding | verdict | what re-running it shows |
| --- | --- | :-: | --- |
| NEW-C1 | conjunct 7 per key, §5.3's md1 rules per descriptor | **FIXED** | All three blockquotes now quantify over keys (*"in which ANY key's…"* ×2, *"for EVERY key whose…"*), a mixed-paths paragraph states the partition, and §7 carries the three mixed rows. **Re-measured both halves, not read.** Device: B1/B2/B4 all ACCEPT with canonical **fixed points** (`Encode(Parse(canonical)) == canonical`, asserted, not just re-parsed) and recv0 `bc1qadgf37z…` / `bc1qadgf37z…` / `bc1qghwumhc…` — r3's three values reproduce exactly. md1: per-`@N` divergence is real — `md encode` gives `wsh(sortedmulti(2,@0/*,@1/<0;1>/*))` → `0x9dcdb` and the *transposed* `@0/<0;1>/*,@1/*` → `0x1e10a` (distinct payloads), and `md decode` returns each template verbatim. Four representable mixtures all agree with the device at the address layer (table below). The rule is consistent **and implementable**. |
| NEW-I1 | §7 required rows that are not md1-capable, routed through the md1 round trip | **FIXED** | `md1_admits` added to the row schema (line 1106) and the `address_0` rule re-scoped: Go asserts every row carrying it, Rust asserts through md1 only where `md1_admits`, and asserts a REFUSAL where it is false on a host-admitted row. r3's construction no longer fires: the file can now be authored green — `/0/*` and `<0;1>` rows are `md1_admits=false` and their Rust assertion is the refusal §5.3(a)/(a″) mandates. (The default is backwards for most other required rows — **NEW-M2**, Minor, fails loudly.) |
| NEW-I2 | the multi-record bullet captured multi-LINE single descriptors; its justification was false for them | **FIXED** | §5.1's first boundary bullet is now the whole-input-parse discriminator, and §6's multi-record row is scoped to *"ONLY when the whole input does not parse as one descriptor"*. Both journeys re-traced: **r2's NEW-I5** (mnemonic + descriptor in one file) → whole input does not parse (branch 1 dies on the non-`": "` line, 2/3/4 fail) → §6's row, `EXIT_INVALID (4)`, true message, executable remedy. **r3's NEW-I2** (12-line BlueWallet, `--as` absent) → whole input parses → §5.1's block, `EXIT_USAGE (2)` → §11 item 5 true for all four formats. The brief's ambiguity case (a one-line bip380 file, which is both a whole-document descriptor *and* a record that is a descriptor) is **explicitly resolved in the text**: *"Only when the whole input does NOT parse as one descriptor AND some individual record does"* — the whole-input branch wins, and it is stated, not implied. (The bullet's *"always does"* is a false universal — **NEW-M1**.) |
| NEW-M1 | §6's seven new rows fell outside the table | **FIXED** | Measured, not eyeballed: the table occupies lines 988–1021 contiguously — header, delimiter, **32 data rows**, **0 interior blank lines**. All seven rows, including the four NORMATIVE refusals, are inside it. |
| NEW-M2 | zero-fingerprint warning scoped to BlueWallet | **FIXED** | §4.2's paragraph is now stated over *"an origin path the INPUT SUPPLIED"*, names all three formats, and excludes §4.5 promoted keys with the invented-path reason r3 required. (It is not scoped across `--as`, and is measurably false on the md1 path — **NEW-M3**.) |
| NEW-M3 | §4.3's surviving *"≤ 15 keys under `sh`"* | **FIXED** | Now *"≤ 15 keys when the `sortedmulti` is DIRECTLY under `sh`"*; `grep -n "keys under \`sh\`"` over the whole file returns nothing. |
| NEW-N1 | `<a;b;c>` described as admitted-but-unmeasured | **FIXED** | §4.3's bullet, conjunct 7's parenthetical and §6's closed-set row all corrected. **Citations hand-checked at the line:** `bip380/bip380.go:476` is `starts, ends, ok := strings.Cut(p[1:len(p)-1], ";")`; `:489` is `if start > end || start >= hdkeychain.HardenedKeyStart || …`. Re-measured: `…/<0;1;2>/*` and `…/<1;0>/*` both `nonstandard: unrecognized output descriptor format` — parse refusals, as the new text now says. |
| NEW-N2 | the `Upub` remedy named the wrong multisig form | **FIXED** | Now *"`sh(wsh(sortedmulti(…)))` for `Upub`, `wsh(sortedmulti(…))` for `Vpub`"*. Verified against source rather than lore: `bip380.go:449–451` maps `YpubVer → P2SH_P2WSH` and `ZpubVer → P2WSH`, and `Script.DerivationPath()` (`:152–165`) gives `P2SH_P2WSH = m/48'/0'/0'/1'`, `P2WSH = …/2'`. Both forms are right. (The sibling row one line up still shares one form — **NEW-N1**, Nit.) |

---

## The fold's central claim, re-measured independently

r3 could show md1 carried per-key divergence for **one** mixture. Every representable mixture I
ran round-trips, and every one agrees with the device at the address layer:

```
descriptor (K1 tail + K2 tail)     device address.Receive(_,0)   md1 chunk-set-id / md address --index 0
  /*        +  <0;1>/*             bc1qghwumhc…                  0x9dcdb   bc1qghwumhc…      MATCH
  childless +  /*                  bc1q52sh4mw…                  0x1e10a   bc1q52sh4mw…      MATCH  (materialised per key)
  <0;1>/*   +  <0;1>/*             bc1qadgf37z…                  0x16d62   bc1qadgf37z…      MATCH
  <0;1>/*   +  <2;3>/*             bc1q3k0xapy…                  0x16b3b   bc1q3k0xapy…      MATCH  ("mixed freely", divergent bases)
```

Row 2 is the strongest evidence for the fold and nobody had run it: it is §5.3(a′)'s
materialisation applied **inside a mixed descriptor** — the childless key encoded as `<0;1>/*`
while its neighbour stays `/*` — and it reproduces the device's own address exactly. Row 4 shows
the per-`@N` override carries two *different* multipath bases, which the mixed paragraph's
"mixed freely" asserts and nothing had tested.

Every device row is a canonical **fixed point** (re-encode equals `canonical`), so §7's
invariant holds on all of them, as the fold assumed.

---

## NEW — Important

### NEW-I1 — `wsh(multi(…))`: three sections promise `--as md1` carries it, and §3 + §4 forbid `me` from ever parsing it. The fold's new `md1_admits` column can express narrowing only, so the one required row that needs the widening direction still has no way to state its assertion

**Not authored by this fold.** It is reported because the brief asks whether any required §7 row
still lacks a way to express its assertion after the new column, and this is that row.

**The four sites, verbatim.**

- §5.5 (line 947): `| wsh(multi(k, …)) — unsorted | ❌ device refuses (§4.3) | ✅ md encode takes it |`,
  under a table whose closing sentence is *"The two `❌` columns are not the same shape, **in
  either direction**. That is the whole argument for the operator choosing."*
- §6's `multi` row (line 1000): *"…This wallet can still be engraved: **`--as md1` encodes `multi`
  policies.**"* — an operator-facing remedy naming the flag.
- §10 (line 1258): *"**`wsh(multi(…))` under `--as descriptor`.** … `--as md1` carries it."*
- §7's required rows (line 1139): *"**`wsh(multi(…))`, a miniscript descriptor, and a full-origin
  `ypub`** — `false`/`false`, the `neither` rows the vacuity check needs"* — i.e.
  `host_admits=false`.

**Why `me` can never execute that remedy.** §4.3 is normative that the cascade admits
*"exactly one multi form: **`sortedmulti` and nothing else**"*; §4.7's seven shapes contain no
`multi` form; §5.3 decomposes *"the **parsed** descriptor"*, i.e. the output of that cascade; and
§3 forecloses the escape hatch outright — *"Rust is written to agree with it — narrowing where
§4.7 says to narrow, **never widening**."* So `me sysw pack --as md1` on a `multi` descriptor
refuses at admission, before `--as` is consulted, and the operator who follows §6's remedy gets a
second refusal. That is r2's NEW-I5 shape (a refusal naming a path that then refuses), one input
class over.

**And §7 requirement 4 nails it shut.** The invariant is `host_admits ⇒
device_admits(canonical)`. The device refuses `wsh(multi(…))`, so the row can never be
`host_admits=true` without being *"a defect the tests must reject"*. `md1_admits` is defined as a
qualifier **on top of** `host_admits` (*"where `md1_admits` is false **on a host-admitted row**…"*),
so the combination this row would need — `host_admits=false, md1_admits=true` — is not expressible
and carries no assertion at all. The new column expresses *host admits, md1 refuses*; the case
§5.5 leans on is *host refuses, md1 admits*, and after the fold that direction is still
unrepresentable.

**The contradiction is not vacuous — md1 really can carry it.** Measured:

```
$ …/target/release/md encode 'wsh(multi(2,@0/<0;1>/*,@1/<0;1>/*))' --path "m/48'/0'/0'/2'" \
    --key @0=<K1> --key @1=<K2> --fingerprint @0=dc567276 --fingerprint @1=f245ae38
chunk-set-id: 0xd5e52          (encodes clean)
```

So md1 the *format* carries `multi`; `me` the *tool*, as specified, cannot reach it. §5.5's cell
is honest about what was run (*"`md encode` takes it"*) while sitting under a column headed
`--as md1`, which is a `me` flag — the cell measures one thing and the table's conclusion claims
another.

**Required (one of three, an author's choice, not a prescription).** Either (i) state in §4.7
that the shape conjunct is `--as`-dependent — `multi` is admitted **only** on the md1 path, with
§7 gaining an `--as`-independent way to say so and requirement 4 restated over
`--as descriptor` rows only; or (ii) strike the ✅ from §5.5, rewrite §6's remedy to name `md`
rather than `--as md1`, and correct §10; or (iii) record `multi` as deferred with F-414's shape.
What must not survive is a spec that promises a flag will engrave a wallet its own input contract
refuses at the door.

**Evidence:** `md encode` run above; §3's NORMATIVE narrowing clause; §4.3's `sortedmulti`-only
grammar; §4.7's seven shapes; §7 requirement 4 and the `md1_admits` bullet.

---

## NEW — Minor

**NEW-M1 — §5.1's discriminator asserts a false universal, and its measured citation names a file
that does not produce the number.** The bullet reads *"If it parses as one descriptor — a
BlueWallet file or pretty JSON **always** does (measured: **the fork's own 12-line fixture** read
whole is ACCEPT, **`#0dc3ykny`**, a fixed point)"*. Three defects, one harmless rule:

- *"always"* is false, and the spec falsifies it two sections earlier: §4.2's own measured table
  has *"same, `Name:` removed | **REFUSE**, generic message"*. Re-measured today on the fork's
  `sh` fixture with `Name: sh` deleted → `nonstandard: unrecognized output descriptor format`.
  Also false for a `Format:`-less file (which does not merely refuse — it **panics**
  `bip380.go:214 panic("unknown script")`, reproduced) and for a `Derivation:`-less file (host-
  refused by §4.2's NORMATIVE rule).
- **`#0dc3ykny` is not the fork's fixture.** The fork has exactly two BlueWallet fixtures.
  Measured: the `sh` fixture parses whole to `…#tk50fvpm`; the `V2` fixture to `…#u4qhgqpj`.
  `#0dc3ykny` comes from a BlueWallet file **constructed from the JSON fixture's three keys**
  (`dc567276`/`f245ae38`/`c5d87297`) — a file that exists in no repo. The number is r3's, and it
  was mislabelled there too; the fold transcribed the label.
- *"12-line"* matches neither: the `sh` fixture is **14** lines, `V2` is **11**, and the
  constructed file that yields `#0dc3ykny` is **11**.

The rule itself is sound — the whole-input parse is the discriminator, and it is conditioned on
*"if it parses"*, not on the universal. Fix: drop *"always"* (or qualify it *"a well-formed
BlueWallet file or pretty JSON does"*), and cite the file that was actually run. Measured, the
fork's own `sh` fixture read whole **is** ACCEPT, 3 keys, `…#tk50fvpm`, a fixed point — that is a
true citation available for free.

**NEW-M2 — `md1_admits` defaults to `false`, and `false` on a host-admitted row now means "assert
the md1 path REFUSES" — which is backwards for most of §7's required rows, including the one r3
named.** The rule (line 1148) is unconditional on host-admitted rows, so every required row that
does not explicitly carry `md1_admits: true` asserts a refusal that will not happen. Rows in that
position, from §7's own required list:

- **the BlueWallet happy path** — the fork's fixtures are **childless** (measured canonical:
  `wsh(sortedmulti(2,[5a0804e3/48h/0h/0h/2h]xpub…,…))#tk50fvpm`, no children), so §5.3(a′)
  materialises and md1 carries it. Defaulted false ⇒ asserts a refusal ⇒ red.
- **the promoted-key happy path** — every §4.5 promotion is childless, same shape.
- **`sh(wsh(sortedmulti(2, 16 keys)))`** — required to carry `address_0`, `host_admits=true`,
  `md1_admits` unstated. This is precisely the row r3's NEW-I1 flagged as still ambiguous
  (*"the same ambiguity now sits on the fold's other new row"*); the column was added and the row
  was not marked.
- **the §4.6 whitespace rows** — `host_admits=true` by design; the trimmed descriptor is
  md1-representable, so the refusal assertion is false for them too.

Correctly marked, only `/0/*`, `<0;1>`, the JSON fixture (which is `/0/*`) and the two
`--as descriptor`-only mixed rows are `false`. The failure is loud (a red suite, not a false
PASS), which is why this is Minor rather than Important — but the default as specified is wrong
for the majority of the rows it applies to, and a default that must be overridden on most rows is
not a default. Fix: either invert it (default `true`, mark the four/five exceptions) or require
`md1_admits` explicitly on every `host_admits=true` row and say so in the *"a test that counts"*
requirement (§11 item 3). Second, smaller point: the refusal assertion does not name the reason,
so a refusal for an unrelated cause satisfies it — worth one clause (*"refuses citing
§5.3(a)/(a″)"*), since the bullet claims this is what *"turns §5.3(a)/(a″) from prose into a
gate"*.

**NEW-M3 — the generalised zero-fingerprint warning is not scoped to `--as descriptor`, and it is
measurably false on the md1 path.** §4.2's new rule fires *"whenever an origin path the INPUT
SUPPLIED is dropped this way"* and the message says the path *"is not carried by **the engraved
record**"*. The mechanism cited is `Descriptor.encode` omitting `[…]` when `mfp == 0` — that is
the **canonical BIP-380 re-encoding**, i.e. the `--as descriptor` record. md1 does not use it.
Measured, a zero-fingerprint key round-trips through md1 with its origin **intact**:

```
$ md encode 'wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*))' --path "m/48'/0'/0'/2'" \
    --fingerprint @0=00000000 --fingerprint @1=f245ae38 …      -> chunk-set-id 0xb3602
$ md descriptor <that card set>
  wsh(sortedmulti(2,[00000000/48'/0'/0'/2']xpub…/<0;1>/*,[f245ae38/48'/0'/0'/2']xpub…/<0;1>/*))#t2st4md6
                     ^^^^^^^^^^^^^^^^^^^^^ origin carried
```

The control with `@0=dc567276` gives the same shape (`0x16d62`, `#l9ucx0pn`), so the origin block
is carried irrespective of the fingerprint value. §5.3(b) already says what md1 drops — *"the
label"*, and only the label — so §4.2's rule contradicts §5.3(b) as well as the measurement. As
written, `--as md1` warns the operator that restore metadata was lost when it was not: a false
alarm, which is worse than saying nothing. Fix is one clause: scope the warning to
`--as descriptor`.

**NEW-M4 — §5.3's three blockquotes now require the refusal to "name the offending key"; neither
of §6's two md1-split rows has a slot for it, and §11 item 4 tests §6's text.** The fold added
*"The refusal names **the offending key** and `--as descriptor`"* to (a) and (a″) — which is the
right thing to say once the rules are per-key — but §6's `/0/*` row and its `<0;1>`-under-`--as
md1` row still read *"…Use `--as descriptor`, which carries `/0/*` exactly."* with no key
identifier, while sibling rows in the same table do carry substitution slots (*"cosigner `<fp>`"*,
*"key `N` is `tpub`"*). §11 item 4 requires a test asserting the *text* of every §6 row, so an
implementation that satisfies §6 can violate §5.3 and stay green. In a 3-key descriptor with one
offending key, the operator is told which shape is wrong and not which key carries it. Related and
also unstated: a descriptor mixing an (a)-shaped key **and** an (a″)-shaped key matches **both**
rows, with no precedence given — harmless (both messages are true and both name
`--as descriptor`, which carries both shapes exactly), but one clause would settle it.

---

## NEW — Nit

**NEW-N1 — §6's bare `Zpub`/`Ypub` row still gives both versions one multisig form.** It reads
*"a `Zpub`/`Ypub` declares a **multisig** account (`m/48'/0'/0'/2'` and `…/1'`). … supply the full
`wsh(sortedmulti(…))` descriptor"*. The row itself distinguishes the two paths and then offers one
form; by the mapping the fold just verified for `Upub`/`Vpub` (`bip380.go:449–451` +
`Script.DerivationPath()`), `Ypub` is `P2SH_P2WSH` and its form is `sh(wsh(sortedmulti(…)))`. r3
flagged this as predating NEW-N2, and the fold corrected the `Upub`/`Vpub` row one line below
while the commit message reports the superseded-phrasing sweep as clean — the same four words fix
it.

**NEW-N2 — §4.6's justification for "the whole input" is now stale.** It closes *"'The whole
input' is well-defined because a descriptor invocation is single-document (§5.1)."* The fold's
discriminator introduces a second context in which the whole input is read — `--as` **absent**, an
invocation that is by definition not single-document. The term stays well-defined (it is the whole
file or stream), so nothing behaves differently; the *reason given* no longer covers both readers.

---

## Verified in passing, recorded so a later round does not re-spend it

- **F-417's cross-references are all TRUE.** `design/FOLLOWUPS.md:14511` carries the ruling record
  with the owning phase (*"none — standing decision"*), and its Consequences paragraph matches
  §10's bullet clause for clause. The tripwire doc comment exists at
  `descriptor-mnemonic/crates/md-codec/src/use_site_path.rs:13–25` — *"THE NARROWNESS OF THIS
  BLOCK IS DELIBERATE — NOT A BUG TO FIX (operator ruling 2026-08-28…)"* — and it names the same
  four unrepresentable shapes and the same additive-TLV seam. §10's five factual clauses
  (BIP-388 discipline, `/0/*` carried by `--as descriptor`, the cost list, consumers refuse and
  name, the seam) all resolve.
- **`md-codec/src/tlv.rs:10`** is the doc line *"TLV tag for use-site-path overrides (per-`@N`
  divergent path declarations)"* immediately above `TLV_USE_SITE_PATH_OVERRIDES: u8 = 0x00` —
  the quoted phrase is at the cited line.
- **§4.2's three device defects still reproduce** on the current tree, incidentally re-confirmed
  while testing NEW-M1: `Format:` absent → `panic: unknown script` at `bip380.go:214`;
  `Derivation:` absent → ACCEPT with `[5a0804e3]xpub…` whose canonical does **not** re-parse;
  `Name:` absent → generic refusal.
- **One number in r3's own report does not reproduce, and no spec text depends on it.** r3's
  per-member table gives `/*` recv0 as `bc1qghwumhc…`; measured, uniform `/*` on the 2-key fixture
  gives `bc1qu2cc6t70nm0tw0v3tsmgur33gjnw2a32czk6xatccky9jpjxj4eqcedjh9` (and recv1
  `bc1qrranhxp…`, which is r3's own `<0;1>` chg0 — internally consistent once corrected).
  `bc1qghwumhc…` is the address of the **mixture** `/*` + `<0;1>/*`, which is what the spec cites
  it for, so the spec's use of the number is correct. Recorded only so a future round does not
  chase the discrepancy into the spec.
- **`--as md1` acceptance (§11 item 2) and §5.5's `/0/*` row are consistent** with the fold: the
  fork's JSON fixture (format 3's exemplar) is `/0/*` and is `md1_admits=false`, so item 2's
  *"each of the four formats"* needs a non-`/0/*` JSON file. Pre-existing, stated here rather than
  filed.

---

## Closing

**The fold-scoped lens closes: 8/8 FIXED, 0C/0I attributable to the fold.** The re-quantification
was the right fix and it is now measured rather than argued — md1's per-`@N` override carries
every mixture the new rule calls CARRIED, including the per-key materialisation case nobody had
run, and each one reproduces the device's own address. The four Minors are all one clause each and
none of them changes a wallet: a false universal with a mislabelled citation, a default that is
backwards for most rows it governs, a warning that is true on one `--as` path and false on the
other, and a refusal requirement that reached §5.3 but not §6.

The single Important is older than the fold and larger than it: **§5.5's two-flag argument rests
on `--as md1` accepting `wsh(multi(…))`, and §3 + §4 forbid `me` from parsing it.** Three review
rounds and this one's brief all pointed at `md1_admits` before it surfaced, because it is a
*missing thing at a moment* rather than a wrong thing in a section — the operator who takes §6's
remedy is the only reader who meets it. It should be triaged as its own finding rather than folded
in as this round's, and it wants an author's ruling (widen the md1 path, or retract the claim),
not a transcription fix.
