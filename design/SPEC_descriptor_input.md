# SPEC — descriptor input (`me sysw pack --as descriptor` / `--as md1`)

**Status: GREEN — re-closed at round 19 (2026-08-28), 0 Critical / 0
Important, with the walk lens complete and the 60-cell decision table
single-valued at 60/60**
(`design/agent-reports/R0-descriptor-input-spec-r1.md` … `-r19.md`: the
correctness lens closed at r8 after 23 → 10 → 8 → 7 → 7 → 11 → 6 → 0
blocking findings; rounds 9–19 opened further lenses — the operator walk,
adversarial gate derivation — and re-closed at r19; every round's report
persisted verbatim and every non-trivial fold re-reviewed). After the r19
persist, a TERMINAL fold (this text) moved the §5.1 gate's precision out of
prose into §7's executable gate rows and folded rounds 18–19's non-gating
residue; **the r20 closure review verified that fold: GREEN STANDS (0C/0I,
`R0-descriptor-input-spec-r20-closure.md`)**. **Post-GREEN amendment
2026-08-28:** §4.7 gained conjunct 8 (key identity — the published
`md-codec` 0.42.0 lacks the F-217/F-218 encode refusals), §6 its
key-identity row, §7's gate its clause 8 — moving the manifest arithmetic
**34→37 gate / 85→88 slots / 68→71 floor**, the numbers every downstream
artifact must re-pin (the propagation check's first target); found by
PLAN-r1's C1, corrected by PLAN-r2, **under verification by the plan R0
rounds in flight** (past-tense only when a round closes on it). **Post-GREEN
amendment 2026-08-29:** §6 gained the `multi`-class device-clause rule — the
transposition carve-out generalised from one row to six, after IMPL-P1's review
(M1) measured five §6 rows printing a device claim that is FALSE for a `multi`
input — and the key-identity row's cause clause was restated over ENTRIES
rather than BlueWallet LINES (M5: the row cannot fire for a BlueWallet file,
measured). Neither changes a refusal, a cause or an operator's next action;
both change what a row SAYS about the device, and P2.4's verbatim assertions
are written from the AMENDED text. **Post-GREEN amendment 2026-08-29 (b):**
§7's clause 8 and §5.4's tier parenthetical stated a CONJUNCT ORDER — conjunct
1's `--as`-dependent `multi` arm ahead of conjuncts 2–8 — that the IMPL-S1S3
adversarial review measured as the cause of a Critical: it short-circuited the
flag-independent conjuncts, so an anyone-can-spend `wsh(multi(0,…))` heard
"This wallet can still be engraved" under `--as descriptor`. Both sites now
state the shipped order (the flag-dependent arm runs LAST), which is what makes
§6's key-identity row's *"both `--as` paths"* true. No refusal TEXT changes;
the ordering rule does. §9 item 7 records the walk's
one narrow residual.
**Post-GREEN amendment 2026-08-29 (c) — S2 SHIPPED `--as descriptor`, and
this text was written while it had not.** The build-conditional half of the
spec is now history: `--as descriptor` packs §5.2's record, so §5.1's window
refusal has no build state to fire in and RETIRED with §6's
`window-not-in-build` row (§6 is **35** rows), §5.3's window substitution
never fires, and neither the choice block nor the clap help marks a value.
The `--as`-omitted gate is consulted on IDENTIFICATION rather than after
record classification fails, because `sysw::classify` now has a descriptor
arm and an admitted descriptor no longer falls through to `Unknown`. `me
sysw show` reports the record per §5.4's vocabulary — a surface §11 item 1
assumed and nothing had built. §7's file regenerated ONCE, which retired the
`sysw_class` sample column, retired the `panic:parse` marker with the parse
panic it named, carried a new `neither` row and moved a `ypub` row into a
new `version-gap` bullet: **89 tag-slots, 89 − 17 = 72 rows.** Every
amendment below is marked `AMENDED 2026-08-29 (S2)`. What P2.7 did
NOT touch was the DEVICE: §4.3's and §4.5's device clauses, §9 item 2
and §6's "the device admits exactly" quote described fork `main` before
S2's arm. Those amended in S2's P3.5 (marked `AMENDED 2026-08-29 (S2
P3.5)`), carrying the fork diff that falsified them (`0abbf81..fe9475c`
on `s2/descriptor-arm`).
**No code may be written before the implementation plan passes its own
gate** (project `CLAUDE.md` — this is risk-set work: it changes normative
admission behaviour and it decides which wallet an operator engraves).

**Goal, in the operator's words:**

> *"Broadly accepting on input and expressive on output."*

**Every measurement in this document was produced by running something.** The
command is given at the point the number is used. Nothing here is read from a
doc comment, and nothing is carried over from the brief that framed the cycle —
two of that brief's premises turned out to be false and are corrected in §2.6,
at the point they occur.

---

## 1. The gap, in one sentence

**Nothing in the constellation turns a real wallet export into a packable
record.** The device has a descriptor parser and an admission table with a
descriptor column; the host has neither; and the two halves on the device do
not meet each other.

That sentence is three separate measured facts. §2 establishes each one.

---

## 2. Measured inventory

Binaries and trees, pinned:

```
$ /home/bcg/.cargo/bin/me --version                        -> me 0.7.0
$ grep '^version' crates/me-cli/Cargo.toml                 -> version = "0.7.0"
$ grep -n 'md-codec' crates/me-cli/Cargo.toml              -> md-codec = "0.42"
$ grep -n 'name = "md-codec"' -A 3 Cargo.lock              -> 0.42.0, registry+crates.io
$ cd /scratch/code/shibboleth/seedhammer && git rev-parse main
  d402f18   (pushed 2026-08-28; origin/main agrees)
$ git diff --stat 0b656d7 d402f18 -- nonstandard/ bip380/  -> (empty)
```

That last line is load-bearing: **`nonstandard/` and `bip380/` are byte-identical
between the pushed tip and the unpushed `main`**, so every Go measurement below
is valid against both.

**A stale binary trap, recorded because it nearly produced a false inventory.**
`/home/bcg/.cargo/bin/md` reports `md 0.13.0` and so does
`descriptor-mnemonic/crates/md-cli/Cargo.toml` — but the installed file is dated
`Jul 11 23:30` and the repo tip is `fad69f1f` dated `2026-08-27 19:41`. The
installed binary **does not have the `descriptor` subcommand the repo has**. The
version string is not a version. Every `md` measurement in this document was
therefore run against
`/scratch/code/shibboleth/descriptor-mnemonic/target/release/md`, the binary
built from the tree, and that path is written out at each use.

### 2.1 The host refuses every descriptor form there is

```
$ printf 'wsh(sortedmulti(2,[f57ec65d/48h/0h/0h/2h]xpub…/<0;1>/*,…))' > desc.txt
$ /home/bcg/.cargo/bin/me sysw pack --no-passphrase --in desc.txt ; echo rc=$?
me: record 0 (records count from 0) is not a form this container can place: not
a BIP-39 mnemonic, not an md1/mk1/ms1/mt1 string, and not a `text:`/`pass:`/`tx:`
record. Descriptors and addresses are not yet classifiable here — see sysw::classify
rc=4
```

`rc=4` is `EXIT_INVALID` (`crates/me-cli/src/main.rs:338`; the constants are
`EXIT_OK=0`, `EXIT_USAGE=2`, `EXIT_REFUSED=3`, `EXIT_INVALID=4` at lines
335–338). The message is emitted from `crates/me-cli/src/main.rs:2425`, and
`mnemonic_engrave::sysw::classify` (`crates/me-cli/src/sysw/mod.rs:205`) has no
descriptor arm — its own doc comment at line 201 says so.

**AMENDED 2026-08-29 (S2): the transcript above is HISTORY, and is kept
because it is the measurement this spec was written from.** It has been
false since S1 shipped: that invocation reaches §5.1's gate and exits **2**
with the choice block, not 4. Since S2 it is false twice over — `classify`
HAS a descriptor arm, and the quoted sentence *"Descriptors and addresses
are not yet classifiable here"* was retired with it (a descriptor the §4.7
predicate refuses still lands on that message, in narrower words). Nothing
about the ARGUMENT this section makes depends on the transcript being
current: the gap it recorded is the gap S1–S3 closed.

### 2.2 `md encode` takes a template, never a descriptor

```
$ …/target/release/md encode 'wsh(sortedmulti(2,[dc567276/48h/0h/0h/2h]xpub…,…))'
md: template parse error: template contains no @i placeholders
rc=1
```

`md encode --help` states the input form: *"BIP 388 template, e.g.
`wsh(multi(2,@0/<0;1>/*,@1/<0;1>/*))`"*. The keys arrive separately, through
repeatable `--key @i=XPUB` and `--fingerprint @i=HEX`, with the origin path
through `--path`.

### 2.3 The device admits a class its classifier cannot produce

`gui/sysw_admit.go` admits `sysw.ClassDescriptor` in three programs — lines 37,
39 and 45, i.e. `progBundle`, `progMultisig`, `progWalletPolicy`. `SPEC_systemwide_payloads.md`
§3.3.2 is the normative source for that table — and it lists only **two**
`Descr` cells, Engrave Bundle and Engrave Multisig
(`SPEC_systemwide_payloads.md:341–352`, re-read at fold time). **It has no
Wallet Policy row at all**, so `progWalletPolicy`'s `ClassDescriptor` cell is
code-only drift with no normative source. Reconciling that table is
`SPEC_systemwide_payloads`' own change (F-415); this spec builds against the
code as measured and does not claim the two documents agree.

**`sysw.Classify` never returns it.** `sysw/record.go:97` dispatches the three
reserved prefixes and then calls `classifyConstellation`
(`sysw/classify.go:34`), whose arms are: strict BIP-39 mnemonic, strict `ms1`,
`codex32.ValidMD || codex32.ValidMK`, `codex32.ValidMT` — and then
`ClassUnknown`. There is no descriptor arm, and the comment at
`sysw/record.go:94` says the omission is deliberate and mirrors the Rust
primary. `classifyConstellation` calls `strings.TrimSpace` before its arms
(`sysw/classify.go:38`), so a future descriptor arm sits after the trim —
relevant to §4.6 and §5.2.

Measured, not read. A scratch Go module with `replace seedhammer.com =>
/scratch/code/shibboleth/seedhammer` (the fork tree is never written to), built
with `/nix/store/33fw5m31lfcnk4ff2f0df7j2bxnh8lgk-go-1.26.3/bin/go`, called
`sysw.Classify` on **39 descriptor-shaped inputs** covering all four accepted
formats and every near-miss in §4. **`sysw.Classify` returned `ClassUnknown` for
every one of the 39** — including the **18** that `nonstandard.OutputDescriptor`
accepts.

> **So `ClassDescriptor` is unreachable from a systemwide payload today.** The
> three admission-table cells are live code with no input that can satisfy them.
> This is the single most consequential correction in this document, and §8
> states what it does to the phasing.

### 2.4 Where the device's descriptor parser actually is

`nonstandard.OutputDescriptor` has exactly two non-test callers in the fork
(`grep -rn 'nonstandard.OutputDescriptor' --include='*.go' .`, whole tree):

| caller | what it is |
| --- | --- |
| `gui/scan.go:87` | the **scan/NFC door** — a descriptor tapped or beamed at the device |
| `seal/record.go:206` | the **`seal` container's** classifier, `seal.Classify` |

So the device does accept descriptors, over the scan path, today. It is the
`sysw` container — the one `me sysw pack` writes — that cannot carry one.

`seal.Classify` *does* return `ClassDescriptor` (verified in the same probe: the
same **18 of 39** inputs classified `Descriptor` — exactly the ones
`nonstandard.OutputDescriptor` accepts — and the lone address input classified
`Address`). It gets no further: `seal/record.go`'s
`permitted` is an allow-list of `ClassMDMK` everywhere plus
`ClassCodex32Secret`/`ClassMnemonic` in the encrypted section only. A descriptor
in a `seal` payload is classified and then refused. **Both containers refuse a
descriptor; they refuse it at different layers.**

### 2.5 What already exists, and therefore bounds the work

- `md_codec` **0.42.0, from crates.io** — the exact crate `me` links — exposes
  the whole md1 AST publicly: `Descriptor`, `Tag`, `TlvSection`, `PathDecl`,
  `OriginPath`, `PathComponent`, and the `tree` / `use_site_path` modules, plus
  `encode_md1_string`, `split`, `descriptor_to_template`, and (feature `derive`)
  `to_miniscript_descriptor`. Verified against the published crate's re-export
  block, `~/.cargo/registry/src/*/md-codec-0.42.0/src/lib.rs:46–70`.
- `me-cli` already links `md-codec` and already decodes md1 (chunk/codex32
  layers — `crates/me-cli/src/bundle.rs`, `crates/me-cli/src/validate.rs`).
- Both repos already have `serde_json`.
- The shared-vector pattern this spec depends on already ships and is green —
  §7.

### 2.6 Two premises that framed this cycle are false

Recorded here rather than silently worked around, because a future reader
looking at the brief will otherwise reach the same wrong conclusion.

**(a) `descriptor_to_template` does not do what its name suggests here.**

```rust
// md-codec 0.42.0, src/render.rs:19 and src/render.rs:52
use crate::encode::Descriptor;
pub fn descriptor_to_template(d: &Descriptor) -> Result<String, RenderError>
```

The `Descriptor` is `md_codec::encode::Descriptor` — the **md1 AST**
(`src/encode.rs:17`: `n`, `path_decl`, `use_site_path`, `tree`, `tlv`) — not a
BIP-380 descriptor. The function renders a **decoded md1 card** back to a
template. It is the output direction. **Nothing anywhere converts a concrete
BIP-380 descriptor into a template**, and this function is not a head start on
it.

**(b) The template parser is not reachable from `me`.** It lives in
`descriptor-mnemonic/crates/md-cli/src/parse/template.rs` (2747 lines).
`md-cli/Cargo.toml` declares `[[bin]] name = "md"` and **no `[lib]`**, and
`crates/md-cli/src/lib.rs` does not exist. It is a binary crate. `me` cannot
call into it, and shelling out to `md` would make a process the dependency —
which §5.3 rules against.

The consequence of (a) and (b) together: **`--as md1` must build a
`md_codec::encode::Descriptor` directly.** That is feasible because §2.5's AST is
public, and it is bounded because §4.7 confines the admitted grammar to seven
script shapes.

---

## 3. The Rust-primary rule inverts for this cycle — state it, do not infer it

The constellation rule (project `CLAUDE.md`) is that the fork's Go ports of
constellation codecs are **strictly downstream** of the Rust primaries, and a
change to normative behaviour lands in Rust first, with vectors, before it is
ported.

**`nonstandard` is not a port.** It is upstream SeedHammer code
(`nonstandard/parse.go`, package comment: *"parsing of non-standard bitcoin
output descriptors"*), it has no Rust counterpart anywhere in the constellation,
and it falls under the rule's fork-native exemption. Nobody ported it from Rust,
and a future reader must not think anybody did.

**NORMATIVE, for this cycle only:**

> **The Go `nonstandard` + `bip380` parser is the behavioural specification for
> what a descriptor input *is*.** It ships, and the device already accepts those
> files at its scan door. Rust is written to agree with it — narrowing where
> §4.7 says to narrow, never widening.

The never-widening clause binds the DEVICE-FACING admission: what may be
packed as a `Descriptor` record and what §5.2's classification predicate
answers. It does not bind the `--as md1` path, whose product is `MDMK`
records the device's descriptor parser never reads — §4.7 conjunct 1 uses
that boundary exactly once, for `multi` (R0 r4's NEW-I1). The boundary is
mechanical, not aspirational: the device's descriptor model has no unsorted
arm at all (`bip380.MultisigType` is `Singlesig`/`SortedMulti` only,
`bip380/bip380.go:90–94`), and the md1 consumer never builds a
`bip380.Descriptor` for `multi` at all: `gui/md1_expand.go:102`
(`scriptForTemplate`) maps only the bip380-expressible template shapes to
bip380 scalars and reports !ok for the rest (its own D2 comment), and the
`multi` route derives from the template directly (R0 r5, measured: the
device decodes an md1 `multi` set as `PolicyMulti` and derives the same
addresses as the Rust side at indices 0 and 1).

**NORMATIVE, from the moment this cycle's vectors land:**

> **Rust leads thereafter.** `me`'s descriptor parser becomes the primary. A
> later change to what a descriptor input is lands in Rust first, with a row in
> the shared vector file (§7), and the Go side converges. The vector file is the
> artefact that carries the handover: after this cycle, a Go-side change with no
> corresponding Rust row is the Go port leading, and is prohibited.

This is a **creation of a primary**, not a port and not a convergence fix. It is
the only case of its shape in the constellation so far, which is why it is
written down rather than left to the general rule.

---

## 4. The input contract — NORMATIVE

`nonstandard.OutputDescriptor` (`nonstandard/parse.go:36`) tries four things, in
a fixed order. `me` implements the same four in the same order.

### 4.1 Precedence — normative, and what it actually decides

**The order is: 1 BlueWallet → 2 plain BIP-380 → 3 `{label, descriptor}` JSON →
4 promoted bare key.** First branch that succeeds wins; the function returns
immediately.

**No input in the measured corpus is accepted by two branches**, and that is a
structural property rather than luck:

- A BlueWallet file needs `Key: value` lines; a BIP-380 descriptor is `f(...)`
  with no `": "`, so branch 1 fails on it at the first line.
- A descriptor string is not valid JSON, and a bare `xpub…` is not valid JSON,
  so branch 3 cannot claim either.
- A bare key has no `(`, so branch 2's `parseFunc` fails with `missing '('`.

Checked adversarially: `{"label":"Name: x","descriptor":"wpkh(…)"}` — a JSON
document whose label is spelled like a BlueWallet header — is claimed by branch
3, not branch 1, because branch 1 splits the whole single line on `": "` and
gets the key `{"label":"Name`, which is not a known header and is not a valid
xpub.

**So precedence does not decide admission. It decides the DIAGNOSTIC**, and
that is where it bites:

- **Branch 3 returns early even when it fails.** Once `json.Unmarshal` succeeds,
  `OutputDescriptor` returns whatever `bip380.Parse` said about the inner string
  — so a JSON wrapper surfaces the *real* reason. Measured: `{}` →
  `bip380: script: missing '('`; `{"label":"x","descriptor":"wsh(multi(2,"}` →
  `bip380: script: missing ')'`.
- **Branches 1, 2 and 4 do not.** A failure in any of them falls through to the
  next, and the final `return nil, errors.New("nonstandard: unrecognized output
  descriptor format")` **discards every real reason on the way**. Measured: a
  descriptor with a bad checksum reports `unrecognized output descriptor
  format`, not `bip380: invalid checksum`, even though `bip380.Parse` produced
  the latter.

**NORMATIVE:** `me` reproduces the **admission** order exactly, and does **not**
reproduce the diagnostic loss. It retains the error from every branch it tried
and reports the one from the branch the input most resembles — §6 specifies how
that is chosen and what it prints.

### 4.2 Format 1 — BlueWallet (`parseBlueWalletDescriptor`)

A line-oriented `Key: value` format. `nonstandard/parse.go:77`.

- Lines that are empty or begin with `#` are skipped.
- Every other line **must** split on the two-character separator `": "` — a
  line without it is a hard error (`bluewallet: invalid header: %q`).
- Recognised headers: `Name` → `Title`; `Policy` → `%d of %d` via `Sscanf`,
  giving threshold and expected key count; `Derivation` → a `bip32` path;
  `Format` → one of exactly `P2WSH`, `P2SH`, `P2WSH-P2SH`.
- **Any other key is treated as a cosigner**: the key is a hex master
  fingerprint — **for `me`, exactly 8 hex characters / 4 bytes** (defect 4
  below; the Go parser checks only `len > 4` and panics below 4), the value an
  extended key.
- A repeated header with an identical value is skipped; a repeated header with a
  **different** value is an error (`inconsistent header value`).
- The descriptor type is **always `SortedMulti`**, unconditionally
  (`parse.go:80`). BlueWallet cannot produce a single-sig descriptor here.
- Final check: `nkeys != len(desc.Keys)` is an error.
- **Admission gate at the call site**: `OutputDescriptor` accepts branch 1
  **only if `bw.Title != ""`** (`parse.go:37`). A BlueWallet file with no
  `Name:` header, or with an empty one, falls through to branches 2–4, all of
  which fail, and the operator gets the generic message.

Measured (probe over the fork's own shipped fixtures plus constructed variants):

| input | `OutputDescriptor` | note |
| --- | --- | --- |
| the fork's `parse_test.go` `sh` fixture | ACCEPT, `P2WSH`, 2-of-3, `Title="sh"` | the shipped happy path |
| same, `Name:` removed | **REFUSE**, generic message | the real reason is destroyed |
| same, `Name: ` (empty value) | **REFUSE**, generic message | same |
| same, `Format:` removed | **ACCEPT**, `Script=Unknown`, 3 keys | see below |
| `Policy: 2 of 3` with 2 key lines | REFUSE, generic message | count check fired, message lost |
| `Derivation:` placed **after** the key lines | **ACCEPT**, 2 keys, **every key origin empty** | see below |
| CRLF line endings throughout | **REFUSE**, generic message | see §4.6 |
| `Name: only\n` and nothing else | **ACCEPT**, 0 keys, `Script=Unknown` | see below |

Three of those rows are defects in the Go parser, and `me` must not reproduce
them:

1. **`Format:` absent ⇒ `Script` stays `UnknownScript`, and `Descriptor.Encode()`
   then PANICS** — `bip380.go:167`, `panic("unknown script")`, reached because
   `Script.DerivationPath()` has no arm for `UnknownScript`. Reproduced in the
   probe with a `recover()`: `encode PANIC: unknown script`.
2. **`Name: only\n` alone is accepted as a zero-key, unknown-script
   "descriptor"** — and panics identically on `Encode()`. A one-line file that
   is not a wallet parses as one.
3. **A `Derivation:` header after the key lines silently yields keys with no
   origin path.** The parser holds `path` in a single variable applied at the
   moment each key line is read. The resulting descriptor re-encodes as
   `[dc567276]xpub66C1RXMi…` — a *different xpub string* from the input, because
   `Key.ExtendedKey()` rebuilds depth from `len(DerivationPath)` and the path is
   now empty. Measured consequence: **that re-encoded string does not re-parse**
   (`ParseKey` requires `originAndPath[8] == '/'`, and `dc567276` is 8
   characters with nothing after it). The key material itself is intact — the
   probe compared `KeyData` and `ChainCode` and they are unchanged — so this is
   a round-trip and display break, **not** a wrong-wallet break. Stated
   precisely because the over-claim is tempting.
4. **A fingerprint shorter than 4 bytes PANICKED the Go parser — fixed in
   S2 as Rust convergence** *(AMENDED 2026-08-29 (S2 P3.5))*.
   `parseBlueWalletDescriptor` checked only `len(fp) > 4` before calling
   `binary.BigEndian.Uint32(fp[:])` (`nonstandard/parse.go:136–149` at the
   pre-S2 tree), which panicked for fewer. Measured then: a 1-byte (`ab`)
   and a 3-byte (`abcdef`) fingerprint both panicked `OutputDescriptor`
   (`index out of range`), reachable from the device's scan door
   (`gui/scan.go:87`). S2 tightened the guard to `len(fp) != 4` — the
   parser now errors cleanly (measured: the seam suite feeds the input
   with no panic), converging on `me`, which requires exactly 8 hex
   characters — matching what `bip380.ParseKey` already
   requires of an inline origin.
   **AMENDED 2026-08-29 (S2), the FILE half only.** The previous clause said
   §7 marks these rows `device_probe: "panic:parse"` so the Go test never
   feeds one to the parser. S2's regeneration retired that marker with the
   panic it named: the row carries a measured `device_admits: false` like any
   other, and no row bars the Go test from `OutputDescriptor`. The DEVICE
   sentence above — that the parser panics — describes fork `main`, which
   does not yet carry the `!= 4` guard; it amends in P3.5, with the fix that
   falsifies it.

**NORMATIVE:** `me` **refuses** four shapes: a BlueWallet file with no
`Format:` header; one with zero cosigner lines; one in which **any cosigner
key would carry an empty origin path** — the `Derivation:` header is missing
entirely, or appears after a cosigner line; and one whose cosigner fingerprint
is not exactly 8 hex characters. Each refusal names its cause (§6). Refusing
is free under §7's invariant — the host may be narrower than the device, never
wider.

**The third rule is stated over the KEYS, not over line order — R0's C1.** The
ordering case (defect 3) is one way to produce an origin-less key; a file with
**no `Derivation:` header at all** is another, and no ordering clause catches
it. Measured: such a file is ACCEPTED by the device (`script=Segwit`, 2 keys),
its canonical re-encoding carries `[fp]xpub…` per key — and **that canonical
string does not re-parse** (`nonstandard: unrecognized output descriptor
format`; the control with `Derivation:` present re-parses clean). §5.2 packs
the canonical string, so admitting this file would engrave a plate the
device's own parser refuses — verbatim the harm §7 exists to prevent. The
equivalent predicate, checkable on the canonical string:
`MasterFingerprint != 0 ⇒ len(DerivationPath) > 0` (`Descriptor.encode` emits
`[…]` iff `mfp != 0`, `bip380/bip380.go:225–232`; `ParseKey` requires
`originAndPath[8] == '/'`, `bip380/bip380.go:368–372`).

An ALL-ZERO master fingerprint is the one case where a key legitimately
carries no `[…]` in the canonical — `mfp = 0` means "master unknown",
`Descriptor.encode` omits the origin block for it, so conjunct 6 does not
bind. Measured (R0 r2's NEW-M1; generalised by r3's NEW-M2 — the loss is
identical in a BlueWallet `00000000:` cosigner line, a plain BIP-380
`[00000000/48h/…]xpub…`, and a JSON-wrapped one): the wallet round-trips with
identical addresses, but the key's derivation path is silently absent from
the engraved string. **The loss is the CANONICAL RE-ENCODING's alone, so the
warning is scoped to `--as descriptor`** (R0 r4's NEW-M3): md1 stores the
origin in its own path declaration irrespective of the fingerprint value —
measured, a zero-fingerprint key round-trips through md1 with
`[00000000/48'/0'/0'/2']` intact (`0xb3602`, `#t2st4md6`) — and §5.3(b)
already states that md1 drops the label and only the label. `me` **warns on
the `--as descriptor` path**, once per affected key, whenever an origin path
the INPUT SUPPLIED is dropped this way — *"key `<key…>`: the origin path
`<path>` you supplied is not carried by the engraved record (zero
fingerprint = unknown master); addresses are unaffected, restore metadata
is."* — rather than refusing a file several coordinators legitimately emit;
under `--as md1` nothing is lost and no warning fires. A
§4.5 promoted bare key does NOT warn: its origin was invented by the
promotion, not supplied (measured — a bare `zpub` promotes with `mfp = 0` and
an invented `/84h/0h/0h`), and warning about the loss of an invented path
would be noise.

### 4.3 Format 2 — plain BIP-380 (`bip380.Parse`)

`bip380/bip380.go:271`. **This grammar is much smaller than BIP-380.**

- Optional `#checksum`, cut at the **first** `#`. If present it must validate.
- Outer script, from a closed set: `wsh`, `pkh`, `sh`, `wpkh`, `tr`. Anything
  else: `bip380: unknown script type: %q`.
- One optional wrapper level, and only under `sh`: `sh(wpkh(…))` → `P2SH_P2WPKH`,
  `sh(wsh(…))` → `P2SH_P2WSH`.
- Then, optionally, exactly one multi form: **`sortedmulti` and nothing else.**
- Keys via `ParseKey`, which takes `[fingerprint/path]key/children` with a
  strict `[` … `]` origin (fingerprint exactly 8 hex characters followed by `/`)
  and children parsed by `parsePath` (child index, `*`, `*'`/`*h`, or a
  `<a;b>` PAIR — `parsePath` cuts on the first `;`, so a three-element group
  or a reversed pair is a parse error; `bip380/bip380.go:476,489`).
- **Extended-key version bytes** *(AMENDED 2026-08-29 (S2 P3.5) — F-426's
  device half landed)*: **`me`'s admitted set is exactly five:** `xpub`
  (`0488b21e`), `tpub` (`043587cf`), `zpub` (`04b24746`), `Ypub` (`0295b43f`),
  `Zpub` (`02aa7ed3`). `ParseKey` calls `ParseExtendedKey` unconditionally, on
  every key in every branch. Before S2 the classification switch had **no
  `ypub` case**; since S2 it has one (`ypub` → `P2SH_P2WPKH`, normalising
  to `xpub`), so the device's SCAN DOOR now accepts six — measured: the
  once-refused `sh(wpkh([4bbaa801/49h/0h/0h]ypub…/<0;1>/*))` now ACCEPTS
  at the scan door (its seam row moved to the `version-gap` bullet), while
  `upub`, `vpub`, `Upub`, `Vpub` stay refused. **NORMATIVE: `me` admits
  exactly the five above — unchanged in S2** (the host widening is
  F-426's own later convergence cycle), **and the device's sysw RECORD
  classifier holds the same five** via a string-level version check, so a
  `ypub`-keyed record classifies `ClassUnknown` on the device exactly as
  `me` refuses it at the desk. A
  standard BIP-32 library accepts `ypub`, so a host built on one without this
  gate is WIDER than the device on the commonest non-`xpub` key there is
  (R0's C2). The `ypub` refusal prints the equivalent `xpub` spelling with the
  operator's own key converted (§6). Whether `me` should instead normalise
  SLIP-132 keys host-side is **F-413**, an operator question this spec does
  not decide.

Measured:

| input | verdict |
| --- | --- |
| `wsh(sortedmulti(2,[…]xpub…/0/*,…))#hfwurrvt` (the fork's own JSON fixture) | ACCEPT |
| `wsh(sortedmulti(2,[…]xpub…/<0;1>/*,…))` | ACCEPT |
| **`wsh(multi(2,…))`** — unsorted multi | **REFUSE** |
| `wsh(or_d(pk(…),and_v(v:pkh(…),older(52560))))` — miniscript | **REFUSE** |
| `wpkh([…]zpub…/<0;1>/*)` | ACCEPT |
| `tr([…]xpub…/<0;1>/*)` | ACCEPT |
| `sh(wsh(sortedmulti(2,…)))` | ACCEPT |
| `wpkh(…)#00000000` — bad checksum | REFUSE, **generic message** (§4.1) |
| `wpkh(…)#a3t9av36#a3t9av36` — doubled checksum | REFUSE |
| leading space, or a trailing `\n`, or CRLF | **REFUSE** (§4.6) |
| `48'/0'/…` vs `48h/0h/…` origin spelling | both ACCEPT, identical result |
| uppercase hex fingerprint `[DC567276/…]` | ACCEPT, normalised to lowercase |
| `tpub…` testnet key | ACCEPT |

**`multi` is refused and `sortedmulti` is not.** `md encode --help`'s own
example is `wsh(multi(2,@0/<0;1>/*,@1/<0;1>/*))`, so the two tools' headline
examples disagree about the only multi form the device takes. This is the
sharpest single constraint on `--as descriptor`. (`me`'s own parser DOES read
`multi` — it must, to name it in §6's refusal and to carry it on the md1 path
per §4.7 conjunct 1 — the refusal here is the device's.)

Shapes the Go parser **accepts and should not**, measured:

| input | Go verdict | why it is wrong |
| --- | --- | --- |
| `wsh(sortedmulti(0,…))` | ACCEPT, threshold 0 | **zero required signatures — spendable by ANYONE** (R0's I6; see §6) |
| `wsh(sortedmulti(-1,…))` | ACCEPT, threshold −1, derives an address | `strconv.Atoi` accepts a sign (`bip380/bip380.go:348`) |
| `wsh(sortedmulti(5,…))` with 2 keys | ACCEPT, threshold 5 | **unsatisfiable — funds locked forever** |
| `sh(sortedmulti(2, 16 keys))` | ACCEPT, derives an address | 547-byte redeemScript exceeds the 520-byte script-element limit — **unspendable** (BIP-383: ≤ 15 keys when the `sortedmulti` is DIRECTLY under `sh`) |
| `wsh(sortedmulti(2, 21 keys))` | ACCEPT, derives an address | exceeds `OP_CHECKMULTISIG`'s 20-key limit — **unspendable** |
| `sh(wpkh(sortedmulti(2,…)))` | ACCEPT (`P2SH_P2WPKH`, `SortedMulti`) | a single-key wrapper around a multi; the device cannot even derive an address from it (measured: `address: multisig script: Nested Segwit (P2SH-P2WPKH): unsupported descriptor`) |
| `tr(sortedmulti(2,…))` | ACCEPT | taproot multisig is `multi_a`/`sortedmulti_a` (BIP-387); `tr(sortedmulti(…))` is not a descriptor |
| `wpkh(sortedmulti(2,…))`, `pkh(sortedmulti(2,…))` | ACCEPT | a single-key script wrapping a multi |

### 4.4 Format 3 — `{label, descriptor}` JSON

`nonstandard/parse.go:44–55`. `json.Unmarshal` into `struct{ Label, Descriptor
string }`, then `bip380.Parse(jsonDesc.Descriptor)`, then `desc.Title =
jsonDesc.Label`.

Measured properties, all of which `me` must reproduce or consciously decline:

- **Unknown fields are ignored.** The fork's own fixture carries `blockheight`
  and a `devices` array; both are dropped.
- **Field matching is case-insensitive**, because Go's `encoding/json` is:
  `{"Descriptor":"wpkh(…)"}` **is accepted**, measured.
- **A missing `label` is fine** — `Title` becomes `""`, and unlike branch 1
  there is no non-empty-title gate.
- **Pretty-printing and a trailing newline are fine** — both measured ACCEPT.
  This is the **only one of the four formats tolerant of surrounding
  whitespace**.
- **The branch returns even on failure** (§4.1), which is why this is the one
  format with a useful error message today.
- **The JSON branch does not promote a bare key.** Measured:
  `{"label":"x","descriptor":"xpub…"}` is REFUSED (`bip380: script: missing
  '('`) while the same key alone is promoted by branch 4. `me` reproduces
  this: promotion is only for a bare key arriving bare. §6's wrapper row
  carries the diagnostic.

`me` accepts the same document shape. **NORMATIVE:** `me` matches field names
case-insensitively, exactly as the device does — a host that required lowercase
`descriptor` would refuse a file the device takes, which is a usability defect
rather than a safety one, but is still a needless divergence.

### 4.5 Format 4 — the promoted bare key, and why it needs its own scrutiny

`nonstandard/parse.go:58–73`. `bip380.ParseKey(nil, enc)` on the **whole file**,
then:

```go
for _, s := range []bip380.Script{bip380.P2PKH, bip380.P2WPKH, bip380.P2SH_P2WPKH} {
    if slices.Equal(s.DerivationPath(), k.DerivationPath) { … return a single-sig descriptor … }
}
```

**This branch infers an entire wallet from one key.** It is the only branch that
manufactures structure the input did not state, so its accept set is written out
exhaustively.

**The three qualifying paths, read from `Script.DerivationPath()`
(`bip380/bip380.go:122`) — not from BIP-44/49/84 lore:**

| script | path, exactly | resulting descriptor |
| --- | --- | --- |
| `P2PKH` | `m/44'/0'/0'` | `pkh(KEY)` |
| `P2WPKH` | `m/84'/0'/0'` | `wpkh(KEY)` |
| `P2SH_P2WPKH` | `m/49'/0'/0'` | `sh(wpkh(KEY))` |

**Three hardened components, account index 0, coin type 0. Nothing else
qualifies.** `P2TR`'s `m/86'/0'/0'`, `P2WSH`'s `m/48'/0'/0'/2'`,
`P2SH_P2WSH`'s `m/48'/0'/0'/1'` and `P2SH`'s `m/45'` are all defined by the same
function and all excluded, because the loop lists only three.

The path compared is `k.DerivationPath`, which comes from one of two places:

- the explicit `[fingerprint/path]` prefix, if present; **or**
- if absent, `ParseKey` falls back to the **SLIP-132 version bytes**
  (`ParseExtendedKey`, `bip380.go:428`): `xpub`/`tpub` → `P2PKH`, `zpub` →
  `P2WPKH`, `ypub` → `P2SH_P2WPKH` *(the case S2 added — AMENDED
  2026-08-29 (S2 P3.5); before S2 `ypub` hit `default` and was refused)*,
  `Ypub` → `P2SH_P2WSH`, `Zpub` → `P2WSH`. `me` still refuses a bare
  `ypub` promotion (§4.3's five, unchanged), and the sysw record
  classifier refuses `ypub`-keyed records on the device for the same
  reason.

**The near-misses, measured. This is the table the spec exists to fix.**

| input | verdict | why |
| --- | --- | --- |
| bare `xpub…` | ACCEPT → `pkh(xpub…)` | version ⇒ `44'/0'/0'` |
| bare `zpub…` | ACCEPT → `wpkh(xpub…)` | version ⇒ `84'/0'/0'`; note the key is **re-serialised to `xpub`** |
| bare `Zpub…` | **REFUSE** | version ⇒ `48'/0'/0'/2'`, not in the loop |
| bare `Ypub…` | **REFUSE** | version ⇒ `48'/0'/0'/1'`, not in the loop |
| `[4bbaa801/44'/0'/0']xpub…` | ACCEPT → `pkh` | |
| `[4bbaa801/49'/0'/0']xpub…` | ACCEPT → `sh(wpkh(…))` | |
| `[4bbaa801/84'/0'/0']zpub…` | ACCEPT → `wpkh` | |
| **`[4bbaa801/86'/0'/0']xpub…`** | **REFUSE** | taproot single-sig is not promotable |
| `[4bbaa801/48'/0'/0'/2']xpub…` | **REFUSE** | a multisig cosigner key is not a wallet |
| **`[4bbaa801/84'/0'/1']zpub…`** | **REFUSE** | **account 1 — only account 0 qualifies** |
| `[4bbaa801/84/0/0]zpub…` (unhardened) | **REFUSE** | the comparison is on hardened values |
| **`[4bbaa801]xpub…`** (fingerprint, no path) | **REFUSE** | `ParseKey` needs `originAndPath[8]=='/'`; 8 chars is too short |
| `xpub…/<0;1>/*` (children, no origin) | ACCEPT → `pkh(xpub…/<0;1>/*)` | children do not affect the origin comparison |
| `xpub…\n` (trailing newline) | **REFUSE** | §4.6 |
| a testnet `tpub…`, bare | ACCEPT → `pkh(tpub…)` | version maps to `P2PKH`, i.e. **`44'/0'/0'`, not the testnet `44'/1'/0'`** |

**The three most likely real-world near-misses are `account ≠ 0`, `86'` and a
bare fingerprint with no path**, and all three currently produce the same
generic five-word refusal. §6 makes each one say what it is.

**NORMATIVE, and this is a ruling rather than a transcription:** `me` promotes a
bare key under exactly the three paths above and **refuses `tpub` promotion
entirely**. A testnet key whose only claim to being a wallet is a version byte
that maps to a **mainnet** derivation path is an inference the host declines to
make. The device's behaviour is unchanged and stays wider; §7's invariant
permits the host to be narrower, and this is the clearest case of it.

**NORMATIVE:** promotion is **announced, not silent**. `me` prints to stderr the
descriptor it inferred, in full, before packing anything — the operator supplied
one line and is getting a wallet, and §5.4's host-side-first rule means they see
it first.

**The announcement echoes the operator's key AS SUPPLIED, alongside the
inferred descriptor — R0's I5.** The canonical re-encoding rebuilds an xpub's
depth and child-number bytes from the (invented) origin path
(`Key.ExtendedKey()`, `bip380/bip380.go:94–105`), so for any key whose true
depth is not 3 the inferred descriptor contains a base58 string the operator
has never seen — measured: a depth-4 cosigner xpub promoted to `pkh(…)`
re-serialises to a different `xpub6Bq…`. The one check the announcement exists
for is "is that my key?", and printing only the normalised form makes that
check fail on a correct result. So the announcement prints both — `key as
supplied: <verbatim>` and `inferred wallet: <canonical>` — with one line
stating the serialisation was normalised and the key material is unchanged.

### 4.6 Whitespace — where the host is deliberately more forgiving

Measured across all four formats:

| input shape | branch 1 | branch 2 | branch 3 | branch 4 |
| --- | :-: | :-: | :-: | :-: |
| trailing `\n` | fine (line-based) | **REFUSE** | fine | **REFUSE** |
| leading space | n/a | **REFUSE** | fine | not tested; same mechanism |
| CRLF throughout | **REFUSE** | **REFUSE** | fine | n/a |

**A wallet export is a file, and files end with a newline.** `wpkh(…)` saved
from any editor, and any file that has been through a Windows tool, is refused
by the device's parser today.

**NORMATIVE:** `me` trims leading and trailing ASCII whitespace from the whole
input, and normalises CRLF to LF, **before** the cascade runs.

This does **not** violate §7's invariant, and the reason is mechanical rather
than a judgement call: the record `me` packs is the **canonical re-encoded
descriptor string**, never the operator's file. `sysw` records are LF-separated
(`SPEC_systemwide_payloads` §5.3.1, which itself cites the encrypted-payload
spec's §6.4), so a record cannot contain a newline by construction. The device
never sees the whitespace the host absorbed. "The whole input" is
well-defined because a descriptor invocation reads its input WHOLE in both
contexts: §5.1's single-document mode when `--as` is present, and §5.1's
whole-input-parse discriminator when it is absent (R0 r4's NEW-N2).

### 4.7 The admitted grammar — the narrowing profile, NORMATIVE

`me` admits a descriptor only if, after the cascade, it is one of:

```
pkh(KEY)                          wpkh(KEY)                  sh(wpkh(KEY))
tr(KEY)                           wsh(sortedmulti(k, KEY…))
sh(wsh(sortedmulti(k, KEY…)))     sh(sortedmulti(k, KEY…))
```

…**and** every conjunct of the admission predicate below holds.

**The admission predicate — script shape is ONE conjunct, not the whole rule.**
R0 found that every safety property that is not a script form had fallen out
of this section (key version bytes, key-count bounds, network consistency,
origin-path presence — C1/C2/C3/I4), so the predicate is stated as an explicit
conjunction, and §7's row list is derivable from it:

1. **Shape:** one of the seven forms above — and, on the **`--as md1` path
   ONLY**, the three `multi` twins of the sortedmulti forms
   (`wsh(multi(k,…))`, `sh(multi(k,…))`, `sh(wsh(multi(k,…)))`), which md1
   carries natively (measured: `wsh(multi(2,@0/<0;1>/*,@1/<0;1>/*))` encodes
   clean, chunk-set-id `0xd5e52`) and the device's descriptor parser refuses
   (§4.3). This resolves R0 r4's NEW-I1: §5.5, §6 and §10 all promise `multi`
   via `--as md1`, and an admission stated purely as a device mirror made the
   promise unexecutable. Under `--as descriptor` — and for §5.2's
   classification predicate, which is `--as`-independent and device-facing —
   the shape conjunct remains the seven forms, so a `Descriptor` record can
   never carry `multi` and §7's invariant is untouched (§3's boundary
   paragraph). All other conjuncts (2–8) apply to `multi` identically (r2's NEW-C1: 8 included — the `multi` twins exist only on the md1 path, the one path that reaches the published crate).
2. **Threshold:** `1 ≤ k ≤ n`, where `n` is the number of keys.
3. **Key count (BIP-383):** `n ≤ 15` when the `sortedmulti` — or its `multi`
   twin (the bound is the redeemScript's, not the ordering's) — is the DIRECT
   argument of `sh(…)` — there the multi's own output script IS the
   redeemScript, one script element capped at 520 bytes, and 16 compressed
   keys need 547 — and `n ≤ 20` for `wsh(…)` and `sh(wsh(…))`
   (`OP_CHECKMULTISIG`'s consensus limit; in `sh(wsh(…))` the redeemScript is
   the 34-byte `OP_0 <sha256>`, so the 520-byte limit never binds the key
   count, and a 16-key `sh(wsh(sortedmulti(…)))` is a SPENDABLE wallet the
   device derives addresses for — measured with a RECORDED construction, 16
   unhardened children of the `dc567276` fixture key:
   `39rQdUtKL2dUiiN3tqXYrwPijTMQudnd3Q` (r6; r2's original `3HBBPgNtm…` came
   from unrecorded keys and is not reproducible — R0 r6's NEW-N1).
   R0 r2's NEW-I2: r1 prescribed "outermost script is sh" and the prescription
   was wrong). Measured on the refused side: the device ACCEPTS
   `sh(sortedmulti(2, 16 keys))` and `wsh(sortedmulti(2, 21 keys))` and
   derives payable-looking addresses for both — scripts that can never be
   spent.
4. **Version bytes:** every key's version is in §4.3's five-member set.
5. **Network:** all keys share one network. Measured: a mixed `xpub`/`tpub`
   `sortedmulti` is ACCEPTED by the device's parser and re-parses clean — and
   `address.Receive` then refuses it (`address: multisig descriptor mixes
   networks`, `address/address.go:105–107`), so the record would reach
   programs whose whole job is deriving addresses they cannot derive.
6. **Origins:** every key with a fingerprint carries a non-empty origin path
   (§4.2's canonical-string predicate).
7. **Use-site path (R0 r2's NEW-C1/NEW-I1):** each key's children expression
   is one of **`{absent, /*, /i/*, <i;i+1>, <i;i+1>/*}`** — a closed set, like
   the shapes. The measured-broken classes it excludes: a HARDENED use-site
   component (`*h`, `i'`) — the device silently derives the UNhardened child
   and displays addresses for a wallet that cannot exist (hardened derivation
   from an xpub is impossible; `md` refuses it, the device does not — measured,
   the `*h` address is byte-identical to the non-hardened one); a
   NON-CONSECUTIVE multipath (`<a;b>` with `b ≠ a+1`) — the device parses it,
   `address.Receive` errors `unsupported range path element` while
   `address.Supported` still returns true, conjunct 5's class again.
   Everything else in `parsePath`'s grammar (a bare fixed index,
   multi-component tails like `/0/1/*`) is refused as UNMEASURED, per the
   closed-set rule. (`<a;b;c>` groups and reversed pairs never get this far:
   `parsePath` cuts on the FIRST `;` and checks `start > end`, so both are
   parse REFUSALS — measured; `bip380/bip380.go:476,489`. R0 r3's NEW-N1.) This conjunct gates BOTH `--as`
   values; which members of the admitted set md1 can carry is §5.3's
   per-`--as` split (`/i/*` and `<i;i+1>`-without-wildcard are
   `--as descriptor`-only — and for a `multi` form, which has no
   `--as descriptor` path, carried by NEITHER path; §5.3's refusals say so
   rather than pointing at a flag that also refuses).
8. **Key identity (PLAN-r1's C1; corrected and renumbered per PLAN-r2):** no
   two keys may declare the same `(fingerprint, origin path)` with DIFFERENT
   xpubs — one origin identifies exactly one key, so such a description
   matches no wallet — and no two slots may carry the same
   `(xpub, use-site path)` pair. The duplicate rule is keyed on the USE
   SITE, not the origin (r2's NEW-I1): the same xpub at `<0;1>/*` and
   `<2;3>/*` is a legal two-chain wallet — measured, the tree's `md encode`
   accepts it (`0xbc4ce`) and the device derives a distinct address. Both
   checks are the F-217/F-218 refusals the Rust-primary `md-codec` enforces
   ON ENCODE; the PUBLISHED 0.42.0 crate `me` links predates them
   (measured: the validator functions and their error variants are absent
   from the registry source), so `me` enforces both HOST-SIDE, on both
   `--as` paths, refusing with §6's key-identity row. Convergence with the
   Rust primary, not leading; F-424 owns the codec bump
   (operator-gated).

**The list above plus the predicate is the closed accept set, and the rule is
stated over them, not over an enumeration of exclusions:** anything the cascade
produces that fails any conjunct is refused. The exclusions *measured*
so far are `tr(sortedmulti(…))`, `wpkh(sortedmulti(…))`, `pkh(sortedmulti(…))`,
`k = 0` and `k > n` — the four rows at the end of §4.3 plus a threshold check
the Go parser does not make at all — and, by inspection of `bip380.Parse`'s
grammar rather than by probe, the key-in-a-script-slot forms `wsh(KEY)` and
`sh(KEY)`, which `Parse` builds as `Singlesig` and which are not descriptors.
That second pair is now MEASURED (R0 r2's NEW-I4): both ACCEPT on the device
(`Singlesig`, canonical a fixed point) with `address.Supported` FALSE and no
derivable address — conjunct 5's reaches-programs-that-cannot-derive class —
and §7's narrowed-shape bullet names them, with §6 rows.

**Why a narrowing profile and not a port.** Three options were considered:

- **Port `bip380` to Rust line-for-line.** Guarantees agreement — including
  agreement on `sortedmulti(5, …)` with two keys, which is an unsatisfiable
  script, and on a `panic` reachable from a one-line file. §3 makes Rust the
  primary from the moment this lands; a primary that is bug-compatible by
  construction is the wrong artefact to hand the next cycle.
- **Depend on `rust-miniscript`.** `me` today has **no `miniscript` and no
  `bitcoin` dependency** (`crates/me-cli/Cargo.toml`, whole `[dependencies]`
  block read). It would parse far more than the device accepts, so the profile
  gate would be needed anyway, and it would be carrying a large dependency in
  order to reject most of what it can parse.
- **A small parser for exactly the seven shapes above** — chosen. The grammar is
  closed, the key expression is the only recursive part, and §7's vectors are
  what prove it stays inside the device's accept set.

**Miniscript descriptors are out of scope for both `--as` values** — see §10.

---

## 5. The two output forms — NORMATIVE

### 5.1 `--as` is required; there is no default and no fallback

`me sysw pack` gains `--as <descriptor|md1>`. It is **required whenever the
input is a descriptor** and there is no default value.

Omitting it is a **usage** error, `EXIT_USAGE` (2), not a refusal — FOR AN
INPUT AT LEAST ONE `--as` VALUE CARRIES IN THIS BUILD (§5.4's carriage
rule; an input nothing carries gets its own refusal directly at (3), never
a menu of dead flags). The message does not merely name the flag; it states
the choice, because an operator holding a wallet export does not know which
they want:

```
me: this input is a wallet descriptor, and `--as` decides how it is packed.
      --as descriptor   the SCANNABLE plate. The device engraves the wallet
                        as a QR that any phone or wallet app can read -- no
                        special tooling to restore, ever. Packed in CANONICAL
                        form (SLIP-132 versions become xpub, ' becomes h,
                        checksum recomputed). The engraver itself cannot
                        read the QR back (it has no camera).
      --as md1          the HAND-COPYABLE plate. me converts the descriptor
                        and packs error-corrected md1 text cards in ONE step
                        (no md invocation needed). Restored by transcription;
                        each string survives up to 4 MIS-STRUCK characters
                        (substitutions -- a missing or extra strike is not
                        correctable), so it can even be hand-stamped. Carries
                        policies --as descriptor cannot. Restoring needs an
                        md1 decoder (an open spec; the tooling today is this
                        project's).
    They are not interchangeable -- `me sysw pack --help` has the comparison.
```

In a build where the descriptor path has not shipped, the block marks that
value inline — `--as descriptor (not available in this build)` —
so the choice text never offers a BUILD-dead flag unmarked (R0 r9's M6,
**and since S2 no build state reaches it: both values ship and the block, and
`me sysw pack --help` with it, marks nothing**; the block above is what S2
prints, verbatim, with each description INLINE on a head padded to the
description column),
ruling the interaction the walk assigned to the plan; the walk-W5
current-build rule governs this block too). When NEITHER value carries the
input, the block does not fire at all — §5.4's carriage rule (r13's
new-I2). And when exactly ONE value carries this particular input, the
block still offers both, unmarked, DELIBERATELY (r14's new-M2): the
operator who picks the input-dead value gets that path's own refusal, which
names the working flag — better guidance than a longer menu rule, and the
journey rule's bar ("worse than silence") is not met. This unmarked ruling
covers INPUT-dead values only; a BUILD-dead value is always marked (M6),
and where both apply the build marking wins (r15's minor).

**A path that cannot work fails naming the other path. It never switches.**
This is the operator's ruling and it is the reason §5.4 exists: the two accept
sets genuinely differ, in both directions, so an automatic fallback would
silently change which of two different artefacts the operator engraves.

**How a descriptor arrives — single-document mode. NORMATIVE (R0's C6).**
`--in`'s shipped contract is newline-separated records
(`SPEC_systemwide_payloads` §5.6), which can never carry a multi-line
BlueWallet file or pretty-printed JSON — measured: the fork's own BlueWallet
fixture via `--in` dies on `record 0` (`# BlueWallet Multisig setup file`)
with rc=4. So `--as` changes the input contract of the invocation it appears
in:

> When `--as` is present, the invocation is **single-document**: exactly one
> descriptor, read WHOLE — `--in <FILE>` is the entire file as one document
> (§4.6's "the whole input"), a single argv operand is the document, stdin is
> the entire stream. Supplying `--as` with more than one argv operand, or with
> both argv and `--in`, is `EXIT_USAGE` (2): *"--as packs exactly one
> descriptor per invocation."*

One descriptor, one container, one invocation. Packing a descriptor TOGETHER
with other records (the Engrave Bundle case — one container carrying `Descr`
plus `MDMK`) is deliberately out of this cycle: it needs its own flag design
and is filed as **F-414** rather than half-specified here.

Two boundary rules that fall out of it (R0 r2):

- **The discriminator between "one descriptor in a multi-LINE file" and "a
  descriptor among OTHER records" is the WHOLE-INPUT parse (R0 r3's NEW-I2),
  guarded by a shape GATE whose normative content is an intent, an
  invariant, and §7's executable gate rows — not a prose shape test
  (terminal fold: rounds 15–19 each repaired one disjunct of the prose test
  and broke or half-fixed another — the precision now lives where a suite
  can hold it).** When `--as` is absent, `me` consults the gate, and re-reads
  the whole input through §4's cascade ONLY when the gate opens — when the
  input is DESCRIPTOR-SHAPED.

  **AMENDED 2026-08-29 (S2): the trigger is IDENTIFICATION, not
  classification failure.** This clause used to read *"when `--as` is absent
  and record classification fails"*, and that precondition was safe only
  while no descriptor could classify. S2's `classify` arm places §5.2's
  record, so an admitted single-line descriptor now classifies — and under
  the old trigger the gate would be skipped, §5.4's identification block,
  §5.1's choice block and every omitted-`--as` §6 refusal would silently die,
  and the input would pack RAW at exit 0 (measured). The gate is consulted on
  EVERY omitted-`--as` pack, immediately before record admission and after
  `--expect` resolution, whose position does not move.

  **The gate's intent and invariant — NORMATIVE.** The gate keeps two
  promises at once, and an implementation is conformant exactly when both
  hold:

  1. **No record-shaped input ever hears descriptor vocabulary or a
     changed exit code.** An input whose lines are record material —
     well-formed records of the six shapes the shipped classifier admits
     (`crates/me-cli/src/sysw/mod.rs:211`), or mistyped attempts at them —
     keeps the SHIPPED record-classification refusal unchanged: exit 4,
     record vocabulary, pinned by a green test of the record-refusal
     surface (`sysw_cli.rs:1928`) (R0 r15's new-I4; r17's nit — the pinned
     operand is a record, not specifically a mnemonic). No well-formed
     record line is descriptor-shaped, whatever its payload carries: none
     of the six admitted shapes begins with an identifier followed by `(`
     (r19's derivation — established against the classifier, not an
     exemplar).
  2. **Every admitted descriptor spelling reaches the descriptor
     surfaces.** §4's four formats on their happy paths, all fifteen §4.5
     rows, all five of conjunct 7's use-site spellings, and descriptor
     content buried behind other records in a multi-record input each open
     the gate and land where §11 item 5's cases and §6 put them — the
     "--as decides" block, a §6 row, or §6's multi-record row — never the
     record refusal.

  **The precision is §7's `gate`-tagged vector rows, which are NORMATIVE:
  each pins gate verdict, outcome class, §6 row and exit code for one
  adversarial input, and where any reading of the guidance below disagrees
  with a gate row, the row is the answer.** The required classes are
  enumerated in §7's gate bullet.

  **Implementation guidance — NON-normative, overridden by the gate
  rows.** The shape tests that satisfied the invariant through round 19,
  the first three applied to EVERY line of the input and the JSON test to
  the WHOLE input (per-line except JSON — r19's minor; the per-line scope
  is what lets a buried descriptor open the gate, r18's new-I1): (T1) a
  line whose first token is an identifier immediately followed by `(` — a
  script expression, not any parenthesis: `text: my wallet (2 of 3)` stays
  a record (r17's minor). An identifier is `[A-Za-z][A-Za-z0-9_]*` (r18's
  minor — decided, not inherited): the underscore is included so a bare
  miniscript fragment (`or_d(…)`, `multi_a(…)`) reaches §6's miniscript
  row rather than the record refusal, while the wrapper-prefixed
  `v:pkh(…)` fails on every reading, its identifier being followed by `:`.
  (T2) a line whose `": "` key is a BlueWallet header
  (`Name`/`Policy`/`Derivation`/`Format`) or an 8-hex fingerprint — a bare
  `": "` is NOT enough, or `seed:`/`text:`/`pass:`-style records would
  hear descriptor vocabulary (r16's new-I2). (T3) a line that is a single
  token beginning with `[`, OR whose leading segment before any `/` is a
  78-byte base58check payload — covering the origin-annotated key AND the
  keyed-no-origin spellings like `xpub…/<0;1>/*`, all fifteen §4.5 rows
  (r16's new-I1; r17's new-I1). (T4) a whole input that is JSON with a
  descriptor field.

  When the gate does not open, the SHIPPED record-classification refusal
  stands unchanged (invariant 1). When it opens and the input parses as
  one descriptor — a
  WELL-FORMED BlueWallet file or pretty JSON does (measured: the fork's own
  `sh` fixture, 14 lines, read whole is ACCEPT, `#tk50fvpm`, a fixed point;
  a malformed one does not: no `Name:` is a device REFUSE (measured), while
  no `Format:` is refused by `me` at §4.2's NORMATIVE rule even though the
  DEVICE parses it and then panics on re-encode (§4.2 defect 1) — either
  way it falls through) — the
  input IS a descriptor and gets the "--as decides" block at `EXIT_USAGE`
  (2) when at least one `--as` value carries it in this build — otherwise
  its own refusal, directly, per §5.4's carriage rule — matching §11 item
  5's tested cases across all four formats. When the
  whole input does NOT parse as one descriptor AND some individual record
  does, the refusal names the split (§6's multi-record row) — naming `--as`
  there would send the operator to a whole-file read that refuses with a
  message false about the file (NEW-I5, measured). And when the gate opens
  and NEITHER parse succeeds — a refused promotion, a malformed
  descriptor, a refused key buried among records — §6's cause selection
  chooses the refusal, reading the WHOLE input; §6 states its scope, and
  the buried-key outcome, explicitly (r19's minor).
- **This section amends `--in`'s contract in `SPEC_systemwide_payloads` §5.6**
  for `--as` invocations only. That is a cross-document change of the same
  shape as F-415, filed as **F-416**, so §5.6 gains its note in its own cycle
  rather than drifting silently (NEW-M2).

**The S3-only window — RETIRED 2026-08-29 (S2).** `--as descriptor` ships, so
no build state reaches any of the text below and §6's `window-not-in-build`
row retired with it. The section is kept, marked, because it is the ruling a
future build-gated path would be read against — the ORDERING rule below
(§4.7's admission refusal precedes anything about a flag or a build) is
independent of the window and still binds: it is what makes conjunct 1's
`multi` refusal permanent rather than a wait.

**The S3-only window — NORMATIVE (walk W4/W11; F-418 ships S3 first).** In a
build where the `--as descriptor` path has not shipped, `--as descriptor` is
a REFUSAL at `EXIT_REFUSED` (3) — emitted AFTER the host-side parse and the
§5.4 identification block, so the refusal can be truthful about the
alternative and the operator can still verify the wallet (walk W13). The
text leads with the verdict and contains no internal phase labels (walk W5):

    me: --as descriptor is not available in this build.
          The QR plate needs device firmware this release does not include.

**Ordering (R0 r14's new-I1): the §4.7 admission refusal PRECEDES the
window refusal.** A wallet no path admits gets its admission refusal
regardless of flag and build — its status is permanent, and possibly
funds-urgent: `sortedmulti(0,…)` must hear "treat those funds as at risk
now", never "nothing is lost by waiting". The window refusal fires only for
wallets the descriptor path WOULD CARRY in a full build — conjunct 1's
shape test included, so a `multi` form under `--as descriptor` gets
conjunct 1's permanent refusal instead, in every build (r15's new-I3). The
verdict line is then followed by ONE of two alternative clauses, decided by
md1-representability (walk W11 — no refusal may point at a path that
refuses in the CURRENT build):

- input md1-representable: *"Available now: --as md1 — me converts and
  packs in one step: error-corrected text cards, restored by transcription
  instead of scanning. Your export file is
  all you need to come back for the QR plate later; nothing is lost by
  waiting."*
- input (a)/(a″)-shaped: *"--as md1 cannot carry this wallet either — key
  `@N` uses `<path>`. No path in this build engraves this file. It loses
  nothing by waiting: keep it, and it packs the day the device update
  ships."* (Each offending key and path is substituted; a mixed input
  repeats the key clause per offender — directive outside the verbatim
  span per R0 r11's new-I1.)

Both walked journeys' FIRST commands reached this refusal (walk record) —
it is the front door of the S3 release, and §11 item 5's sibling test pins
both variants.

### 5.2 `--as descriptor`

Packs the **canonical re-encoded descriptor string** — `Descriptor::encode()`,
with its BIP-380 checksum — as one record of class `Descriptor`.

The record is the canonical form, not the operator's bytes: it is single-line by
construction (§4.6), it is what the device's own parser round-trips, and it is
what §7's vectors are stated over. "Canonical" is a real transformation,
measured four ways (R0's C5): SLIP-132 versions normalise (`zpub` → `xpub`),
`'` → `h` (`bip32.Path.Encode`), the checksum is recomputed, and an xpub's
depth/child-number bytes are rebuilt from the origin path — so the base58
string itself can change. §5.1's help text says so: the operator is choosing
between two artefacts and must not be told one of them is their bytes.

**This path requires a device change, and the spec says so rather than
discovering it in implementation.** From §2.3: `sysw.Classify` has no descriptor
arm, so a `ClassDescriptor` record packed today would be `ClassUnknown` on the
device and refused. `--as descriptor` is therefore not complete until
`sysw.Classify` gains that arm — which, under §3, lands in Rust first (as
`mnemonic_engrave::sysw::classify`) with the §7 vectors, and is then ported.

**AMENDED 2026-08-29 (S2): the arm is not a call to
`nonstandard.OutputDescriptor`.** The previous sentence said it was, and that
is measurably wider than the predicate below: over §7's own file,
`device_admits` is TRUE and `host_admits` FALSE on **22 of 72** rows — **18
of them single-line**, and a record cannot contain a newline, so those 18 are
exactly what a scan-door-keyed classifier would place: anyone-can-spend
`sortedmulti(0,…)`, `k > n`, 21 keys, mixed-network, hardened use-sites,
conjunct-8 key-identity failures. Every one would reach a program and a
screen through the already-live admission cells. (Counted from the file at
sha `e7a4160c`; R0 r1 measured 17 against the pre-S2 file, before the `ypub`
row's `device_admits` flipped.) The arm is the PREDICATE below, composed:
§4.6's edge normalisation first (ASCII-whitespace parity with the host's
`normalise` — the arm refuses a record it could only reach by stripping
non-ASCII whitespace the host would not strip; AMENDED 2026-08-29 (S2,
REVIEW-S2-P3-r1 C2's fold)), then parse via `nonstandard.OutputDescriptor`,
then the single-line-reachable
narrowings of §4's cascade (§4.5's promotion table, and §4.3's admitted
versions as a STRING-level check over the key material each cascade branch
actually consumed — `bip380.Key` has no version field and
`ParseExtendedKey` normalises the version away), then §4.7's conjuncts over
the parsed descriptor. The Rust side is `descriptor::host_admits`, which IS
the predicate rather than a restatement of it.

The classification predicate is stated once, and both sides implement it:

> A record is `ClassDescriptor` iff it parses under §4's cascade **and** matches
> §4.7's grammar — the seven forms; conjunct 1's md1-path widening does not
> apply here.

### 5.3 `--as md1`

Decomposes the parsed descriptor into a BIP-388 template plus per-placeholder
keys, encodes it with `md_codec`, and packs the resulting md1 string(s) as
records of class `MdMk`.

**This path needs no device change.** `ClassMDMK` is produced by
`classifyConstellation` (`sysw/classify.go:46`) today and is admitted by the
same three programs that admit `ClassDescriptor`, plus `progSingleSig`
(`gui/sysw_admit.go:37–45`).

**`me` builds the `md_codec::encode::Descriptor` in-process.** It does not shell
out to `md` and it does not depend on `md-cli`. §2.6(b) established that
`md-cli` is bin-only; and a CLI's stdout is a channel that has already caused a
cross-tool defect in this constellation (`SPEC_constellation_cli_uniformity` §3).
The AST is public (§2.5) and §4.7's grammar needs seven `Tag` values, not a
miniscript compiler.

**Two representable-in-md1 limits, both measured, both NORMATIVE refusals.**

**(a) A single fixed use-site index cannot be represented, and encoding one
silently changes the wallet.**

`md_codec::use_site_path::UseSitePath` (published crate, `src/use_site_path.rs:49`)
is `Option<Vec<Alternative>>` plus a wildcard-hardened bit, and the multipath
group carries `MIN_ALT_COUNT = 2` (`src/use_site_path.rs:43`). There is **no representation for one fixed index**.
`md encode` does not refuse it — it drops it:

```
$ …/target/release/md encode '<TEMPLATE>' --path "m/48'/0'/0'/2'" \
    --key @0=<xpub1> --key @1=<xpub2> --fingerprint @0=dc567276 --fingerprint @1=f245ae38

  wsh(sortedmulti(2,@0,@1))            -> chunk-set-id: 0x9bf18
  wsh(sortedmulti(2,@0/*,@1/*))        -> chunk-set-id: 0x9bf18
  wsh(sortedmulti(2,@0/0/*,@1/0/*))    -> chunk-set-id: 0x9bf18
  wsh(sortedmulti(2,@0/1/*,@1/1/*))    -> chunk-set-id: 0x9bf18
  wsh(sortedmulti(2,@0/<0;1>/*,…))     -> chunk-set-id: 0x16d62
```

Four different templates, one payload, no warning. `md decode` of that payload
returns `wsh(sortedmulti(2,@0/*,@1/*))`.

**It is a different wallet, and here are the two addresses.** Same descriptor,
`wsh(sortedmulti(2,[dc567276/48h/0h/0h/2h]xpub…/0/*,[f245ae38/48h/0h/0h/2h]xpub…/0/*))`:

| route | receive address 0 |
| --- | --- |
| the device — Go `address.Receive(desc, 0)` on the parsed descriptor | `bc1qadgf37zk0wtu69j7yclswl99e5jmcv5eh69jp4calgs4ehsecm5sqcjw5a` |
| through md1 — `md address` on the encoded card set | `bc1qu2cc6t70nm0tw0v3tsmgur33gjnw2a32czk6xatccky9jpjxj4eqcedjh9` |

**`/0/*` is exactly the shape of the fork's own shipped JSON test fixture**
(`nonstandard/parse_test.go:22`), i.e. a real coordinator export, not a
contrivance.

> **NORMATIVE: `--as md1` REFUSES a descriptor in which ANY key's use-site
> path is a single fixed child index.** The refusal names the offending key
> and `--as descriptor`, which carries that shape exactly (window
> substitution applies) — UNLESS the
> descriptor is a `multi` form (§4.7 conjunct 1), which `--as descriptor`
> refuses too: there the refusal states that NO `me` path carries this
> descriptor this release, and names the re-export remedy (R0 r5's NEW-I2 —
> two refusals must never point at each other).

**(a′) An ABSENT use-site path is the same representational hole, resolved the
other way — R0's C4.** `UseSitePath` has no "no children" state either:
`{ multipath: None, wildcard_hardened: false }` IS `/*`, and the measurement
table above shows it (`wsh(sortedmulti(2,@0,@1))` and `…@0/*,@1/*…` share
chunk-set-id `0x9bf18`). The device, by contrast, **defaults an empty children
list to `<0;1>/*`** (`address/address.go:188–202`). Left alone, the two `--as`
values would derive different wallets from every childless descriptor —
measured: device `bc1qadgf37z…` vs the md1 route `bc1qu2cc6t7…`, the identical
divergence as (a) — and every §4.5 promoted bare key is childless, so `--as
md1` on a bare `zpub` would silently diverge from `--as descriptor` on the
same input.

> **NORMATIVE: for EVERY key whose use-site path is ABSENT, `--as md1`
> materialises the device's default, `<0;1>/*`, into that key's encoding.** This
> is not a rewrite of the operator's wallet, and not the device's private
> convention (walk W9, moved here per R0 r9's M3): BIP-44/48/49/84 all
> define the levels below an account origin as change (0 receive; 1 change)
> then index; BIP-388 defines `/**` ≡ `/<0;1>/*` as the canonical tail —
> machine-verified byte-identical in the F-410 cycle (attribution corrected
> per R0 r10's new-M1; re-measured, chunk-set-id `0x880c7` both spellings); BIP-389 supplies the
> notation. The device implements these BIPs, and `<0;1>/*` is what it
> already derives for a childless descriptor, so materialising it is the
> only encoding that preserves the wallet md1 claims to carry (the
> address-layer equality below). The §5.4
> confirmation prints the template WITH the materialised `<0;1>/*` AND one
> annotation line, in operator terms (walk W8 — an unexplained novelty at
> steel-imminent stakes earns "is this the wrong derivation path?"; walk W9
> — cite the BIPs, an authority the operator can check, not the device):
> *"note: your input names no derivation below the key origins; `<0;1>/*`
> is the standard receive/change continuation below such origins — the
> convention your wallet software already uses (in the standards: the
> BIP-44 family's change level, and BIP-388's canonical tail). Addresses
> are unchanged by making it explicit."*
>
> (Origin-family-neutral per R0 r9's I3: (a′) fires for BIP-44/49/84
> promoted keys and BIP-48 cosigners alike — the walk's own journey 2 was
> a BIP-84 `zpub`, and a note naming BIP-48 there invites a check that
> fails, defeating W9's purpose.) An EXPLICIT single fixed index (`/0/*`) remains a refusal
> per (a): the device does not default that shape away — it derives it — and
> md1 cannot represent it.

**(a″) A multipath group with NO trailing wildcard (`<0;1>`) is the third
md1-unrepresentable shape — R0 r2's NEW-C1.** `md encode` collapses it into
`<0;1>/*` silently — measured, the two templates produce the SAME chunk-set-id
`0x16d62` — while the device derives a DIFFERENT wallet from it: recv0
`bc1qu2cc6t7…` for `<0;1>` versus `bc1qadgf37z…` for `<0;1>/*`, the identical
address pair (a) prints. Two characters of transcription — a dropped `/*` —
change the wallet. §4.7 conjunct 7 admits the shape because the device
handles it; the hole is md1's alone.

> **NORMATIVE: `--as md1` REFUSES a descriptor in which ANY key's use-site
> path is a multipath group without a trailing wildcard.** The refusal names
> the offending key and `--as descriptor`, which carries it exactly (window
> substitution applies) — the same split as (a), including (a)'s `multi`
> exception: for a `multi` form
> the refusal states that no `me` path carries it and names the re-export
> remedy.

**Window substitution — NORMATIVE (walk W11's symmetric half, folded per
R0 r9's I4).** In a build where the descriptor path has not shipped, every
remedy in this section and in §6 that ROUTES TO the descriptor path —
naming the flag or otherwise (semantic, not lexical, per R0 r14's new-M3;
NEITHER-PATH refusals are exempt: substituting "wait for the update" into
a refusal whose truth is "never, in any build" would be false — r15's
new-I2, reason corrected per r16's minor) —
replaces that clause with: *"the scannable-plate path is not in this build — keep the
export file; it packs when the device update ships."*
**AMENDED 2026-08-29 (S2): the rule stands, and its CONDITION can no longer
occur.** The descriptor path shipped, so no remedy is substituted and the two
§6 rows below print §6's own text, naming the flag that carries their shape.
The rule is kept — marked, not deleted — because it is what a future
build-gated path would be read against, and because the ordering it depends
on (a refusal may DESCRIBE a path's future availability but may never ROUTE
to a flag that refuses in the CURRENT build) is independent of any one build.
No refusal points the
operator at a flag that refuses in the current build; any refusal may
DESCRIBE a path's future availability — describing routes nothing (r17's
minor; stated unconditionally per r18's minor, which measured the stock
replacement above and §5.1's window variant each DESCRIBING the
scannable-plate path's future while routing nowhere — under the previous
neither-path scoping, neither was covered and the absolute was false).

**Mixed use-site paths across keys — the quantifier is PER KEY, matching
conjunct 7 (R0 r3's NEW-C1).** The three rules above bind key by key, because
a descriptor may mix admitted members and conjunct 7 admits it key by key.
Measured: `wsh(sortedmulti(2,K1/0/*,K2/<0;1>/*))` and the childless+`<0;1>/*`
and `<0;1>`+`<0;1>/*` mixtures are all device-ACCEPTED with canonical fixed
points — and per-DESCRIPTOR rules would fire on none of them while md1's
collapse engraves a different wallet (three address swaps, all measured, r3
§NEW-C1). md1 itself carries per-key divergence natively
(`TLV_USE_SITE_PATH_OVERRIDES = 0x00`, "per-`@N` divergent path
declarations", `md-codec/src/tlv.rs:10`) — measured, the all-representable
mixture `wsh(sortedmulti(2,K1/*,K2/<0;1>/*))` round-trips through md1 to the
device's own address `bc1qghwumhc…`. So: a descriptor whose keys all carry
md1-representable paths (absent → materialised per key, `/*`, `<i;i+1>/*`,
mixed freely) is CARRIED; a descriptor with any (a)- or (a″)-shaped key is
REFUSED whole, naming that key. §7 carries the three mixed rows with their
device-route addresses.

**(b) The label is dropped, by both paths.** `md_codec::TlvSection`
(`src/tlv.rs:24`) has fields for use-site overrides, fingerprints, pubkeys,
origin overrides and unknown TLVs — **and no label or title**. The canonical
BIP-380 re-encoding has no title either. So a BlueWallet `Name:` or a JSON
`label` survives neither path.

The label is display-only on the device (`gui/gui.go:3161`, `if desc.Title != ""`
→ add it to the body text), so this is a **warning, not a refusal**:

```
me: warning: the label "Test Multisig 2-of-3" is not carried by any record
    format and will not appear on the device. Nothing else is lost.
```

**What `--as md1` DOES preserve — verified across two independent
implementations.** md1 stores key material (chain code ‖ compressed pubkey), not
the xpub envelope, so `md descriptor` re-serialises the keys at depth 0 and the
descriptor **string** differs from the input. The **wallet** does not. Measured
for the multipath form, the Go device on one side and `md` on the other:

```
Go   address.Receive(desc, 0)  -> bc1qadgf37zk0wtu69j7yclswl99e5jmcv5eh69jp4calgs4ehsecm5sqcjw5a
md   address <the md1 set>     -> bc1qadgf37zk0wtu69j7yclswl99e5jmcv5eh69jp4calgs4ehsecm5sqcjw5a
```

**NORMATIVE:** `--as md1` must not claim a byte-identical round trip. It states
the equality it actually has — same wallet, different serialisation — and the
§7 vectors assert it at the address layer, not the string layer.

### 5.4 Both paths parse host-side first

**NORMATIVE — the block prints on EVERY successful whole-input parse,
BEFORE whatever follows, in TWO tiers (walk W13; R0 r9 I1, r10 new-I1,
r11 new-M1/M2/N1, r12–r13: the three rules below, each stated once).**

**The TIER** is decided by what does not depend on the flag: a wallet that
passes conjuncts 2–8 AND whose shape at least one `--as` path admits gets
the FULL block — every line below. A wallet NO path admits gets the PARTIAL
block — including a conjunct-8 failure, RE-DECIDED per PLAN-r3's I4
(reversing r2-M1's accidental FULL): its addresses derive but are
byte-identical to a clean control (measured), so a compare prompt would
PASS on an impossible wallet, actively reassuring, while the PARTIAL
block's canonical line is what actually shows the operator the two
colliding lines. The PARTIAL block: the first three lines plus the watch-only line — no `wallet-id:`, no
`address 0:`, no compare prompt — because its class is the wallets refused
by the `--as`-independent conjuncts and the no-path shapes: the underivable
(mixed network, hardened use-site, non-consecutive multipath, the wrapped
and script-slot shapes), the unspendable and anyone-can-spend thresholds
and key counts, the refused version bytes and shapes, and the UNMEASURED
closed-set residue (deeper tails and the like — refused as unmeasured, not
as underivable). The class is defined by the conjuncts, not by an
inventory, and a "compare before engraving" prompt would be a wrong
instruction on every member. (A conjunct-8-PASSING `multi` input in the window is FULL-tier —
derivable, spendable, md1-packable when its use-site paths are
md1-representable — and stripping its identification would blind the
operator at the decision their refusal asks; a conjunct-8-FAILING `multi`,
the colliding-origin twin, is PARTIAL like every conjunct-8 failure — the
compare prompt would pass byte-identically on the impossible wallet
(PLAN-r4's NEW-I1, aligning this parenthetical with the rule above it).
**Which FOLLOWER each of the two meets under explicit `--as descriptor`
differs, and the distinction is the amendment of 2026-08-29:** the
conjunct-8-PASSING `multi` meets conjunct 1's PERMANENT shape refusal naming
`--as md1`, in EVERY build — the window text's "come back for the QR plate"
would be false forever for a shape the descriptor record can never carry
(§5.5, §10; R0 r15's new-I3) — while the conjunct-8-FAILING twin meets
conjunct 8's own key-identity refusal, on both `--as` paths, because
conjunct 1's flag-dependent arm runs AFTER the flag-independent conjuncts
(§7 clause 8's amendment; IMPL-S1S3 adversarial review C1). The previous
text applied the first of these to both, which was false for the twin. With
`--as` omitted, the carriage rule governs as usual.)

**The FOLLOWER** is decided independently by §5's own logic — the tier
picks lines, not outcomes, and any tier may precede any follower. The
followers, enumerated once: the `--as`-omitted choice block, §5.1's window
refusal, a §5.3 refusal, a §4.7 admission refusal, or the pack. (A
full-tier `wsh(multi(…))` under `--as descriptor` meets a conjunct-1
admission refusal; the two prior attempts to couple tier to follower each
manufactured a false claim.)

**The CARRIAGE rule — which follower `--as`-omission gets (r12's precedence
rule, keyed on carriage per r13's new-I2):** the choice block (`EXIT_USAGE`
(2)) fires iff at least one `--as` value CARRIES this input in the CURRENT
build — admission, §5.3 representability, and the window all considered.
Otherwise the input's own refusal fires directly at `EXIT_REFUSED` (3): the
§4.7 admission refusal where no path admits the wallet (that determination
quantifies over both paths, so it needs no flag), or the neither-path
refusal (§5.3's `multi` clause, or the window's variant 2) where an
admitted wallet is carried by nothing — `wsh(multi(…/0/*))` in every
build, and the Specter `/0/*` file in the S3-only window. A two-option
menu whose options both refuse is the dead-flag defect §5.1's choice block
was ruled never to be. §5.1's topic and discriminator, §6's rows, and §11
item 5 all carry this rule.

§5.3(b)'s label warning, where it applies, follows the block.

Neither path ships bytes the host has not
understood. `me` prints to stderr:

- the format it matched (§4.1), by name;
- the canonical descriptor, in full;
- the script type, threshold and key count;
- **`wallet-id:` the WalletPolicyId fingerprint, computed over the wallet's
  md1 policy form with §5.3(a′)'s materialisation applied on BOTH paths**
  (walk W10; that uniform base is what makes the id identical under both
  `--as` values — computing over the literal childless input diverges per
  flag, measured `24bcacf5…` vs `3bf32c0e…`, R0 r9's I2). **Emitted only
  when the wallet HAS an md1 policy form.** For a wallet md1 cannot
  represent — (a)/(a″) shapes; deeper tails never reach this line, being
  partial-tier (r13's M1b) — the line is instead:
  *"wallet-id: none — this wallet has no md1 policy form; identify it by
  the checksum in the canonical line and by address 0."* NEVER computed
  over a collapsed encoding: for an (a)-shaped wallet the honest attempt
  ERRORS (`AltCountOutOfRange` — the true id does not exist, R0 r9's C1),
  and encoding anyway collapses to the `/*` wallet (shared chunk-set-id
  `0x9bf18`, divergent addresses `bc1qadgf37z…` vs `bc1qu2cc6t7…`, §5.3(a))
  — whose id (`3bf32c0e…`) would then sit two lines above the compare
  prompt, identifying a different wallet;
- **`address 0:` receive address 0**, followed by: *"compare against your
  wallet software's first receive address before engraving."* — the
  executable check (walk W10; verified in journey 2 against the wallet
  owner's own phone);
- **the watch-only line, owner-quotable, printed in BOTH tiers (walk W15;
  its referent is the wallet DESCRIPTION, which exists whether or not
  anything is packed — R0 r10's new-M5):** *"watch-only: public keys only —
  this wallet description can SHOW its addresses and balances; it cannot
  spend. Whoever holds it can watch the wallet — share it accordingly."*;
- for `--as md1`, the template and the placeholder-to-fingerprint map —
  with §5.3(a′)'s annotation line whenever materialisation occurred (walk
  W8/W9);
- for a promoted bare key (§4.5), the full §4.5 announcement — `key as
  supplied` AND `inferred wallet`, with the normalisation named (R0 r2's
  NEW-N2: this list and §4.5 describe the same stderr block and must
  agree).

This follows the standing ruling that all verification is host-side, and it
is what makes §5.1's no-fallback rule usable: the operator can see — and
CHECK, by one address comparison — that the thing they are about to engrave
is the wallet they meant, even when this build's answer is a refusal (walk
W13: the wait-or-switch decision deserves a verified wallet).

### 5.5 The capability split, measured

This table is why there are two flags and no default. Every cell was run.

| descriptor shape | `--as descriptor` | `--as md1` |
| --- | :-: | :-: |
| `wsh(sortedmulti(k, …/<0;1>/*))` | ✅ | ✅ |
| `wsh(sortedmulti(k, …/0/*))` — single fixed chain | ✅ | **❌ §5.3(a)** |
| `wsh(sortedmulti(k, …/<0;1>))` — multipath, no wildcard | ✅ | **❌ §5.3(a″)** |
| `wsh(multi(k, …))` — unsorted | **❌ device refuses (§4.3)** | ✅ §4.7 conjunct 1's md1-path admission; md1 carries it natively (`0xd5e52`) |
| miniscript, e.g. `wsh(or_d(pk(…),and_v(v:pkh(…),older(…))))` | **❌ device refuses** | ❌ §10 — out of scope this cycle |
| `pkh` / `wpkh` / `sh(wpkh)` single-sig — childless inputs: §5.3(a′) materialises `<0;1>/*` | ✅ | ✅ |
| `tr(KEY)` single-key | ✅ | ✅ |
| carries a label | text only, dropped | dropped |
| needs a firmware change to be readable | **yes, §5.2** — the HOST half shipped in S2 (`--as descriptor` packs the record and `me sysw show` reports it); the device arm and screen are S2's fork half, and until that build is flashed the record is `ClassUnknown` on the machine | **no** |
| on the plate (walk W1) | a QR — machine-scan only | text cards: **ONE plate** for either single-sig shape — the bare key's 2 strings (85 + 83 = 168 chars) and the keyed BIP-84 card's 3 strings (67 × 3 = 201 chars) each pack onto one plate side. **AMENDED 2026-08-29 (S2 P4.2)**: this cell read *"2 strings … = **TWO plates** (one plate per STRING)"*, which was true of `bundlePlatePlan` until F-423 replaced the one-plate-per-string rule with greedy packing WITHIN a card. Measured, not inferred: five 85-character md1 strings fit one plate side at the shipped font (`bundlePlateMD1Capacity`, pinned and re-derived by the fork's `TestBundlePlanPacksACardOntoFewerPlates`); MEASURE-S2-P4-1 confirmed N = 3 by trial fit against `backup.EngraveText` → `toPlate` before the packer existed. A card's strings stay separate visually distinct paragraphs, never reflowed into one another. **One consequence the operator sees:** a PACKED plate is offered `TEXT ONLY` — a per-paragraph QR on a multi-paragraph plate is drawn across its neighbours, so the code variants are offered on single-string plates only |
| restored by | scanning into any wallet app — no project tooling, ever | transcription + an md1 decoder (open spec; tooling is this project's) |
| hand-copyable — letter punches (walk W14) | ❌ | ✅ — BCH corrects up to 4 mis-struck characters per string (substitutions only: a missing or extra strike is outside the budget, R0 r9's N1) |

The two `❌` columns are not the same shape, in either direction. That is the
whole argument for the operator choosing.

---

## 6. Refusals — what the operator SEES

**A refusal that does not say why is the defect this constellation has been
punished for most.** The device's parser has exactly one message for eleven
distinct causes (§4.1). `me` has one per cause.

Every refusal in this section is `EXIT_REFUSED` (3) unless marked otherwise, and
every one of them **names a next action**.

**How the cause is chosen.** `me` runs all four branches, keeps each error, and
reports the branch the input most resembles, by a fixed rule evaluated in this
order:

1. input parses as JSON → report **branch 3**'s inner error;
2. input's first non-comment line contains `": "` → report **branch 1**;
3. input contains `(` → report **branch 2**;
4. input LOOKS like an extended key — its first non-whitespace character is
   `[`, or it is a single token whose leading segment before any `/` is a
   78-byte base58check payload (an extended-key envelope, ANY version, with
   or without a use-site tail — the same leading-segment test as §5.1's
   gate guidance, r18's new-I2)
   → report **branch 4**. A shape test,
   not a parse-success test (R0's I2): the branch-4 rows below are all
   `ParseKey` FAILURES — measured, `bip380.ParseKey(nil, "[4bbaa801]xpub…")`
   errors — so a success test could never reach them;
5. otherwise → "not a descriptor in any form I know", listing the four.

The five-step rule ranks CASCADE (parse) failures only. Rows below that arise
from §4.7's admission predicate or §5.3's representability limits fire from
their own checks, after a successful parse — the rule never selects them
(R0 r2's NEW-N1). And the `sortedmulti` rows below read over BOTH multi
forms: §4.7 conjunct 1's md1-path `multi` twins hit the same conjuncts and
get the same texts with the form name substituted (R0 r5's NEW-M2) — **with
one class of clause excepted, which the amendment below states.**

**AMENDED 2026-08-29 (IMPL-P1's review, M1): NO DEVICE-BEHAVIOUR CLAUSE
TRANSPOSES TO A `multi` INPUT.** `bip380.Parse`'s script switch has a
`sortedmulti` case and no `multi` case, so the device refuses EVERY `multi`
form at PARSE and none of them ever reaches address derivation. The previous
text carved exactly ONE row out for that reason (the single-key wrapper), and
the reason was never specific to that row: it is a property of the whole
`multi` class. Six rows carry a sentence about what the device does with the
descriptor — the single-key-wrapper row plus `tr(sortedmulti(…))`, the mixed
network, the hardened use-site, the non-consecutive multipath and the key
count — and for a `multi` input five of those six were measurably FALSE, each
asserting that the device accepts or derives from a file its parser rejects
(constructed and run against `me 0.7.0`; the table is IMPL-P1-review §M1).

So: **for a `multi` input, every such clause is REPLACED by** *"the device's
parser refuses `multi` outright, so this file never reaches address derivation
there."* The refusal, its cause and the operator's next action are unchanged in
all six — only the device sentence moves. The single-key-wrapper row keeps its
own additional substitution (its REMEDY transposes too, naming the wrapper
change), which this amendment narrows rather than removes.

**Scope: §5.1's gate is per-LINE; the five steps above are whole-INPUT — a
deliberate divergence, not an alignment (r19's minor).** The gate decides
WHETHER an input that failed record classification hears descriptor
vocabulary at all, so it must see a descriptor buried behind other records;
the steps decide WHICH single cause is reported, and they read the whole
input, because the refusal describes the file the operator gave. The two
scopes part company on exactly one class, and its outcome is stated here
rather than left to be discovered: a multi-record input whose only
descriptor-shaped line is a bare key that promotion REFUSES — a bare
`Zpub…`, a bare `tpub…`, a `[fp]xpub…` with no path — opens the gate, but
the whole input does not parse as one descriptor and no individual record
does either, so the multi-record row below does not fire; steps 1–4 then
fail over the whole input, and **step 5 reports the generic four-forms text
at `EXIT_REFUSED` (3)** — true of the whole input, if less specific than
the dedicated bare-`Zpub`/`Ypub`, bare-`tpub` and fingerprint-no-path rows,
which remain reachable for the key supplied alone. Whether cause selection
should instead follow the LINE that opened the gate is a plan-phase design
call; until it is made, the outcome above is the specified one, and §7's
gate rows (clause 6) pin all three spellings.

**Refusal text speaks OPERATOR language — NORMATIVE (walk W5, from the
operator's own verdict "convoluted").** Every quoted text below: leads with
the verdict; contains NO internal identifiers — no phase labels, no F-numbers,
no spec § references inside the quotes (those live in the row's annotation,
outside the quotes); and names only next actions executable in the CURRENT
build (walk W11). The one
exception is the single-key-wrapper row, where — on top of the device-clause
substitution the whole `multi` class now takes (the amendment above) — the
REMEDY's `sortedmulti` forms do not transpose either: for a `multi` input that
row names the mandatory wrapper change instead (R0 r6's NEW-M4; r7's NEW-I2).

| the operator's input | what `me` says |
| --- | --- |
| an unparseable file | *"this is not a wallet descriptor in any of the four forms `me` reads: a BlueWallet `Key: value` setup file, a plain BIP-380 descriptor, a `{"label":…,"descriptor":…}` JSON export, or a single extended key. It looks most like `<form>`, which failed because: `<that branch's error>`."* |
| an **empty file** | the SHIPPED `me 0.7.0` refusal is kept verbatim (R0's I7) — *"no records in `<file>`: pass them on argv, with --in, or on stdin. An EMPTY input is what a FAILED upstream command leaves behind…"* — which already names the C-1 composition hazard. **`EXIT_USAGE` (2)**, measured at fold time (rc=2, 0 stdout bytes); this spec records the existing behaviour rather than silently regressing a tested surface. |
| a file of only whitespace | reaches the same shipped "no records" path (blank records are skipped) — same message, **`EXIT_USAGE` (2)**. |
| **`--as` omitted** (input at least one `--as` value CARRIES in this build) | §5.1's text. **`EXIT_USAGE` (2)**, not 3 — nothing was refused, a choice was not made. For an input nothing carries, the input's own refusal fires directly at (3) — the §4.7 admission refusal, or the neither-path refusal for admitted-but-uncarried wallets (§5.4's carriage rule, r12–r13). **AMENDED 2026-08-29 (S2): which inputs reach which arm MOVED, though the rule did not.** With `--as descriptor` shipped, every admitted descriptor is carried by at least one value, so the four `md1-split/*` rows §5.3 refuses — `/0/*`, `<0;1>`, and the two mixed shapes — exit **2** here rather than 3; their §5.3 refusals are still reachable, through an explicit `--as md1`. The "nothing carries it" arm now needs a wallet BOTH values refuse: `wsh(multi(…/0/*))` is §7's witness for it. |
| a **wrapper whose inner descriptor is malformed** | the wrapper is named, then the inner error, then the position: *"the `{label, descriptor}` JSON parsed, and its `descriptor` field did not: `bip380: script: missing ')'`. The label was `"…"`. The problem is in the descriptor string, not the JSON."* |
| a **BlueWallet file with no `Name:`** | *"this is a BlueWallet setup file — it has `Policy`, `Derivation` and `Format` headers and `N` cosigner lines — but no `Name:` header, and the device requires one. Add a line `Name: <anything>`."* This is the case §4.2 measured as producing the generic message today, with the real reason destroyed. |
| a BlueWallet file with no `Format:` | *"…no `Format:` header, so the script type is undefined. Add `Format: P2WSH` (or `P2SH`, or `P2WSH-P2SH`)."* §4.2 defect 1. |
| a BlueWallet file with **zero cosigner lines** | *"this BlueWallet file has headers but no cosigner lines (`<8-hex-fingerprint>: <xpub>`). There is no wallet here to pack — was the export truncated? Re-export from the coordinator."* §4.2 rule 2 (F-419, written from the walk). |
| ~~**`--as descriptor` in a build where its path has not shipped**~~ | **RETIRED 2026-08-29 (S2).** The row was §5.1's window refusal, both variants, at `EXIT_REFUSED` (3). `--as descriptor` ships: no build state reaches the trigger, so the row has no test that could reach it and was struck rather than kept alive by a weaker assertion. **§6 is 35 data rows.** S2 subtracts one and adds none — the `--as descriptor`-only set S1 recorded as EMPTY was, measured, actually empty. |
| a BlueWallet `Policy: k of n` with a different key count | *"`Policy: 2 of 3` declares 3 cosigners; the file has 2. Cosigner lines are `<8-hex-fingerprint>: <xpub>`."* |
| a BlueWallet file whose keys have **no origin path** (no `Derivation:` header at all, or one placed after cosigner lines) | *"cosigner `<fp>` has no derivation path — the `Derivation:` header is missing or appears after the cosigner lines. The descriptor this file produces cannot be re-read by the device. Put `Derivation: <path>` above the first cosigner line."* §4.2 rule 3 (R0's C1). |
| a BlueWallet cosigner fingerprint that is not 8 hex characters | *"cosigner line `ab: xpub…` — a master fingerprint is exactly 8 hex characters (4 bytes)."* §4.2 defect 4; the device PANICS on fewer, so this file must never reach it. |
| `wsh(multi(…))` under **`--as descriptor`** | *"the device's descriptor parser accepts `sortedmulti` and not `multi`. This wallet can still be engraved: `--as md1` encodes `multi` policies (for use-site paths md1 can represent — otherwise no path carries it, and the refusal says so). (`sortedmulti` differs from `multi` only in key ordering at spend time — it is not a synonym, so `me` will not rewrite it for you.)"* |
| a **miniscript** descriptor, either `--as` | *"`me` reads the descriptor family the device reads: single-sig and `sortedmulti`, optionally under `sh`. This descriptor uses miniscript fragments (`or_d`, `and_v`, …), which neither path handles in this release. `md encode` accepts miniscript **templates** — a different tool and input form."* (Deferral details: §10 — outside the quote per the walk-W5 rule.) |
| a descriptor with **`/0/*`** under **`--as md1`** | *"md1 cannot carry this wallet as written: key `@N` (`[<fp/path>]xpub…`) uses `/0/*`, a single fixed chain index, which has no md1 form — encoding it would silently produce a DIFFERENT wallet. Use `--as descriptor`, which carries `/0/*` exactly."* **AMENDED 2026-08-29 (S2): this is what the row PRINTS now** — §5.3's window substitution has no build state left to fire in, so the remedy is §6's own and names the flag that carries this shape. The offending path is spelled as the encoder writes it, leading slash included. §5.3(a); the offending key is named per §5.3's per-key rule (R0 r4's NEW-M4). For a `multi`-form descriptor the remedy sentence is replaced: *"this is a `multi` policy, which only `--as md1` carries — and md1 cannot represent `/0/*`. No `me` path engraves this file as written, in any build. Re-export with `<0;1>/*` — carried in every build. (Re-exporting as a `sortedmulti` policy keeps `/0/*` but is a DIFFERENT policy — `me` will not rewrite it — and needs the scannable-plate path.)"* (R0 r5's NEW-I2; quote cleaned and the substitution exempted per r15's new-I1/new-I2 — a neither-path refusal routes nowhere, so the window substitution has nothing to replace in it, and "wait for the update" would be false forever for this file.) A descriptor mixing an (a)-shaped and an (a″)-shaped key matches both this row and the next; both fire, both are true, and both name the same remedy — no precedence is needed. |
| **`sortedmulti(k, …)` with `k > n`** | *"threshold `k` of `n` keys can never be satisfied — no combination of signatures reaches `k`. Funds sent to this wallet would be unspendable. Nothing was packed."* §4.7 conjunct 2. |
| **`sortedmulti(0, …)`** — or any `k < 1` | *"threshold 0 means NO signature is required: anyone who can see this script can spend from it. This is almost certainly not the wallet you meant — and if it already holds funds, treat them as at risk now. Nothing was packed."* §4.7 conjunct 2 (R0's I6 — the device derives a real address for `k = 0` and even `k = −1`, so the refusal is the host's alone). |
| `sortedmulti` with **too many keys** | *"`sh(sortedmulti(…))` carries at most 15 keys — there the multi's output script IS the redeemScript, one 520-byte script element (BIP-383). `wsh(…)` and `sh(wsh(…))` carry at most 20 (`OP_CHECKMULTISIG`); their redeemScript is 34 bytes and the 520-byte limit never binds. This descriptor has `n` keys under `<form>`. The device would accept it and derive addresses whose coins cannot be spent."* §4.7 conjunct 3 (R0's C3, bound corrected by r2's NEW-I2). |
| two keys declaring **the same origin with different xpubs** | *"this wallet description contradicts itself: keys `N` and `M` both claim origin `<fp/path>` but name different keys — one origin identifies exactly one key, so no wallet matches this description. Check the export: one of the two entries carries the wrong key, and a copied-and-edited cosigner is the usual cause."* **`EXIT_REFUSED` (3)**, both `--as` paths. §4.7 conjunct 8 (PLAN-r1's C1; row per PLAN-r2's NEW-C2; ordinals per PLAN-r3's N2). **AMENDED 2026-08-29 (IMPL-P1's review, M5):** the previous clause named *"a duplicated cosigner LINE"*, i.e. a BlueWallet file — and this row cannot fire for one. In a BlueWallet file every key shares the single `Derivation:` header, so two keys carry the same `(fingerprint, origin)` **iff** they carry the same header key, which the device's own `seenKeys` map catches FIRST as `inconsistent header value`; branch 1 then fails and §6 row 1 carries the reason (measured). The row is reachable from a plain BIP-380 descriptor and from the JSON wrapper, where two key expressions may carry one origin block and different keys — so the cause clause is now stated over ENTRIES rather than over BlueWallet lines. Conjunct 8's other half, the duplicate row below, IS BlueWallet-reachable (different fingerprints, same xpub) and is unaffected. |
| the **same key at the same derivation in two slots** | *"keys `N` and `M` are the same key at the same derivation — a threshold that needs the same key twice is not the multisig this file describes. Remove the duplicate line, or supply the missing cosigner's key."* **`EXIT_REFUSED` (3)**, both `--as` paths. §4.7 conjunct 8 (split per PLAN-r3's I3 — the primary separates the two causes, and "no wallet matches" is false for a duplicate). |
| a multisig mixing **mainnet and testnet keys** | *"key `N` is `tpub` (testnet) while key 0 is `xpub` (mainnet). The device accepts this descriptor and then cannot derive any address from it. All keys must share one network."* §4.7 conjunct 5 (R0's I4). |
| a descriptor or bare key using **`ypub`/`upub`/`vpub`/`Upub`/`Vpub`** | *"`me` admits exactly `xpub`, `tpub`, `zpub`, `Ypub`, `Zpub`."* *(quote AMENDED 2026-08-29 (S2 P3.5): the device's scan door takes `ypub` since F-426's device half, so `me` is the honest subject; the record classifier still holds the five.)* The remedy names the **per-version** target (R0 r2's NEW-I3 — one template cannot serve five): `ypub` → `xpub` (mainnet BIP-49, `sh(wpkh(…))`); `upub` → `tpub` (**testnet** BIP-49, `sh(wpkh(…))`); `vpub` → `tpub` (**testnet** BIP-84, `wpkh(…)`); `Upub`/`Vpub` → `tpub` (**testnet multisig** — no single-key remedy exists; supply the full multisig descriptor: `sh(wsh(sortedmulti(…)))` for `Upub`, `wsh(sortedmulti(…))` for `Vpub` — or a BlueWallet file). Four of the five are testnet keys, and an `xpub` remedy would name a mainnet wallet the operator does not hold — measured, mainnet `354hXbgw…` versus the real testnet `tb1qmj7qns4…`. For a key WITH an origin, the operator's own fingerprint/path is substituted in; for a BARE key the remedy is the origin-less descriptor spelling — `sh(wpkh(<converted key>/<0;1>/*))`, which the device admits (measured) — because handing back a bare converted key would PROMOTE to a different wallet (`pkh(…)`, measured). §4.3 (R0's C2); F-413 tracks host-side normalisation. |
| `tr(sortedmulti(…))` | *"taproot multisig is `multi_a`/`sortedmulti_a` (BIP-387); `tr(sortedmulti(…))` is not a valid descriptor even though the device's parser accepts it. Check the export."* §4.3. |
| a **bare key whose path matches no script** | the path is quoted back and the three that qualify are listed: *"`[4bbaa801/86'/0'/0']xpub…` is a single extended key. `me` can infer a whole wallet from one only when its origin is `m/44'/0'/0'` (→ `pkh`), `m/84'/0'/0'` (→ `wpkh`) or `m/49'/0'/0'` (→ `sh(wpkh)`). This one is `m/86'/0'/0'` — taproot single-sig, which is not inferable. Supply the descriptor instead: `tr([4bbaa801/86'/0'/0']xpub…/<0;1>/*)`."* |
| a bare key at **account ≠ 0** | *"…this one is `m/84'/0'/1'`. Only account 0 is inferable. Supply the descriptor: `wpkh([…/84'/0'/1']…/<0;1>/*)`."* — measured in §4.5 as a live near-miss. |
| a bare key with a fingerprint and **no path** | *"`[4bbaa801]xpub…` gives a fingerprint with no derivation path, so there is nothing to match a script against. Either give the full origin — `[4bbaa801/84'/0'/0']xpub…` — or drop the brackets entirely, in which case the key's version byte decides."* |
| a bare `Zpub` / `Ypub` | *"a `Zpub`/`Ypub` declares a **multisig** account (`m/48'/0'/0'/2'` and `…/1'`). A multisig cosigner key is not a wallet — supply the full descriptor (`wsh(sortedmulti(…))` for `Zpub`, `sh(wsh(sortedmulti(…)))` for `Ypub`), or a BlueWallet setup file listing every cosigner."* (forms per `Script.DerivationPath()`, R0 r4's NEW-N1) |
| a bare `tpub` | *"this is a testnet key. Its version byte would map to the **mainnet** path `m/44'/0'/0'`, which `me` will not assume. Supply the descriptor with its real origin."* §4.5. |
| a **bitcoin address** | *"that is a bitcoin address, not a descriptor. No program on the device consumes an address record."* (Reasoning: §10 — outside the quote per the walk-W5 rule.) |
| a **single-key script wrapping a multi** — `wpkh(sortedmulti(…))`, `pkh(sortedmulti(…))`, `sh(wpkh(sortedmulti(…)))` | *"a multisig policy cannot live inside a single-key script. The device's parser accepts this spelling and then cannot derive any address from it (measured: `address: multisig script: … unsupported descriptor`). The forms the device derives are `wsh(sortedmulti(…))`, `sh(wsh(sortedmulti(…)))` and `sh(sortedmulti(…))`."* For a `multi` input the remedy is instead: *"a multisig policy cannot live inside a single-key script on EITHER path. Change the wrapper — `wsh(multi(…))`, `sh(multi(…))` or `sh(wsh(multi(…)))` — and use `--as md1`, which carries those forms."* — and the device-measurement parenthetical above does NOT apply to `multi` inputs: all three single-key `multi` twins are device REFUSE at PARSE (measured, R0 r7) and never reach address derivation. (R0 r6's NEW-M4; the prior `multi` remedy named the invocation that had just refused — corrected per r7's NEW-I2.) §4.7 conjunct 1 (R0 r2's NEW-I4). |
| a **bare key in a script slot** — `wsh(KEY)`, `sh(KEY)` | *"`wsh`/`sh` of a single key is not a wallet form the device can derive addresses for (measured: `Supported=false`, `address: singlesig script: … unsupported descriptor`). A single-key wallet is `pkh(…)`, `wpkh(…)`, `sh(wpkh(…))` or `tr(…)`."* §4.7 conjunct 1 (R0 r2's NEW-I4). |
| a **hardened use-site component** — `…/<0;1>/*h` | *"a hardened use-site step cannot be derived from an xpub (BIP-32). The device would silently derive the UNhardened child and display addresses for a wallet that cannot exist, so this is refused on both `--as` paths."* §4.7 conjunct 7 (R0 r2's NEW-I1). |
| a **non-consecutive multipath** — `<0;2>`, `<1;3>` | *"the device derives only `<i;i+1>` pairs (receive; change). It accepts this descriptor and then errors on every address."* §4.7 conjunct 7 (R0 r2's NEW-I1). |
| any **other use-site path shape** — a bare fixed index, `/0/1/*` | *"use-site paths `me` ACCEPTS: absent, `/*`, `/i/*`, `<i;i+1>`, `<i;i+1>/*`. This one is outside the set the device is measured to handle."* ("accepts" not "packs" — admission is build-independent; which flag packs which member is §5.3's and the window's business, R0 r9's I4.) §4.7 conjunct 7 (closed set). |
| a **multipath with no trailing wildcard** (`<0;1>`) under **`--as md1`** | *"md1 cannot carry this wallet as written: key `@N` (`[<fp/path>]xpub…`) uses `<0;1>` with no trailing wildcard, which has no md1 form — encoding it would silently produce the `<0;1>/*` wallet, which derives DIFFERENT addresses. Use `--as descriptor`, which carries `/<0;1>` exactly."* **AMENDED 2026-08-29 (S2):** as the row above — the substitution is gone and this is the shipped text, with the offender spelled as the encoder writes it (`/<0;1>`, leading slash included, matching the cause clause in the same sentence). §5.3(a″) (R0 r2's NEW-C1; key named per R0 r4's NEW-M4; the `multi`-form remedy replacement of the previous row applies here identically). |
| a **multi-record input whose records include a descriptor**, `--as` absent | *"record `N` is a wallet descriptor. A descriptor is packed ALONE: run `me sysw pack --as <descriptor\|md1>` with just the descriptor — one container cannot yet carry a descriptor plus other records. The other records pack without `--as`, as usual."* (The capability gap is F-414 — named here, not in the quoted text, per the walk-W5 rule.) **`EXIT_INVALID` (4)**, as today. Applies ONLY when the whole input does not parse as one descriptor (§5.1's discriminator — a multi-line BlueWallet or JSON file parses whole and gets the "--as decides" block — or, if nothing carries it, its own direct refusal per §5.4's carriage rule — instead); naming `--as` here would send the operator to a whole-file read that refuses with a message false about the file (R0 r2's NEW-I5, measured). |

**Two rules that bind every row above.** Both come from
`SPEC_constellation_cli_uniformity`:

- **The remedy must be executable** (`SPEC_constellation_cli_uniformity` §6h):
  where a row says "supply the descriptor", it prints the descriptor with the
  operator's own key and origin substituted in, not a placeholder.
- **A refusal writes 0 bytes to stdout** (`SPEC_constellation_cli_uniformity`
  §2 — *not* §2 of this document), and `--out` creates no file.

---

## 7. The shared acceptance vectors — NORMATIVE, non-negotiable

**Two independent descriptor parsers will drift.** The Rust one being written
here and the Go one that ships are the same question asked in two languages, and
the direction that matters is asymmetric: a host that admits what the device
refuses packs a payload the device cannot read — **an engraved plate for a
wallet that will not load.**

The pattern already ships in this repo and is green. It is followed exactly, not
approximately.

**The existing instance, read before this section was written:**

- `crates/me-cli/testdata/codex32_seam_vectors.json` — the primary. 6840 bytes,
  sha256 `3d53ef88a474f02c15aa60a839f4a31071598a26c853463122a847515926eb6a`, 8
  rows, top-level keys `_comment` / `invariant` / `vectors`, row keys `name` /
  `string` / `chars` / `host_admits` / `device_admits` / `source`.
- `crates/me-cli/tests/codex32_seam.rs` — asserts the **host** column against
  `mnemonic_engrave::sysw::classify`, and pins the sha256 as the literal
  `SEAM_VECTORS_SHA256`.
- `seedhammer/sysw/codex32_seam_test.go` (at `d402f18`) — asserts the **device**
  column against `sysw.Classify`, and pins the same sha256 as `seamVectorsSHA256`.
  Verified byte-identical: `git show d402f18:sysw/testdata/codex32_seam_vectors.json
  | sha256sum` returns the same digest.

**Requirements, in the same shape:**

1. **One file, `crates/me-cli/testdata/descriptor_seam_vectors.json`**, authored
   in the Rust primary (§3), vendored byte-identically to
   `seedhammer/nonstandard/testdata/descriptor_seam_vectors.json`. The Go
   seam test is **`package nonstandard_test`** (external): once §5.2's arm
   lands, `sysw` imports `nonstandard`, and an internal test importing `sysw`
   to assert the classifier would be an import cycle (R0 r6's NEW-N3, import
   sets measured; **AMENDED 2026-08-29 (S2)** — the reason survives the
   `sysw_class` column's retirement, because the derived rule that replaced
   it calls `sysw.Classify` from the same test).
2. **Its sha256 is pinned as a literal in BOTH tests.** Neither test reaches
   across repos; each reads its own copy and compares to the same constant, so
   the copies cannot drift without one suite going red.
3. **The Rust test asserts the host column; the Go test asserts the device
   column — and, since S2, BOTH tests also assert the derived
   classification rule, which is COMPUTED from the host column**
   *(AMENDED 2026-08-29 (S2 P3.5, completing r5 I2's ownership move):
   `classify(input) == Descriptor` iff `host_admits` for single-line
   rows, and `classify(canonical) == Descriptor` on admitted rows — so
   the Go seam test now reads `host_admits`, as data, not as the other
   implementation's answer)*. Neither implementation is ever compared
   to the other — both are compared to the file. That is why it has to
   be the same file.
4. **The invariant is `host_admits(input) ⇒ device_admits(canonical(input))`,
   asserted per row (R0's I1).** The input-level form is wrong twice over: the
   §4.6 whitespace rows are host-wide by design (the device refuses the raw
   input and reads only the canonical — an input-level assertion fails the
   required rows), and C1's no-`Derivation:` file is device-ACCEPTED as input
   while its canonical does not re-parse (an input-level assertion is blind to
   the one row class the invariant exists for). So: for every row with
   `host_admits=true`, the Go test runs `nonstandard.OutputDescriptor` on the
   row's `canonical` and requires ACCEPT — and requires the re-encoding of
   that parse to equal `canonical` (a fixed point) — with a message naming the
   dangerous direction.
5. **Both tests assert the row set is non-vacuous** — at least one `both`, one
   `device-only`, and one `neither` — or a mutant that refuses everything, or
   admits everything, passes.
6. **A mistyped vector fails loudly.** The codex32 file uses a `chars` count for
   this; the descriptor file uses a per-row `sha256` of `input`, because a
   descriptor is long enough that a character count would not catch a
   transcription error inside an xpub.

**Row schema.** `name`, `input`, `sha256`, `host_admits`, `device_admits`,
`format` (the branch of §4's cascade that `me` MATCHED — `bluewallet` /
`bip380` / `json` / `promoted-key` — or `none` where no branch matched;
R0 r6's NEW-M3: on host/device-disagreeing rows any other reading gives a
different answer), `source`, and the optional value fields `address_0` /
`address_1` / `md_descriptor_contains` / `wallet_id` (asserted per the
paragraph below; R0 r6's NEW-I1; `wallet_id` from walk W10) — plus the columns later rounds forced apart:

- **`device_admits` means `nonstandard.OutputDescriptor` accepts the INPUT** —
  the scan door, nothing else. The classifier is a different predicate with a
  different answer (§2.3), and keeping the two apart is what stopped §7 and
  §11 contradicting each other in round 0. **AMENDED 2026-08-29 (S2): the
  `sysw_class` column is RETIRED.** It was a four-row SAMPLE of the
  classifier's answer, hand-stated, and three of its four inputs were not even
  single lines — so it could not be a population and its input-vs-canonical
  basis was ambiguous. Both suites assert the classifier EXHAUSTIVELY (the Rust half since S2's P2; the Go half lands with the fork's copy at P3.3) and
  DERIVE the expectation from columns already in the file: for every
  single-line row, `Classify(input) == Descriptor` **iff** `host_admits` and
  `== Unknown` otherwise (exact equality — which is also the per-row empirical
  answer to "can a descriptor-shaped string collide with another class"), and
  for every `host_admits: true` row, `Classify(canonical) == Descriptor`. Both
  bases are asserted, separately. No hand-stated per-row class value survives
  to drift, and a Go/Rust divergence anywhere in the file reds one of the two
  suites.
- **`host_admits` means §5.2's classification predicate**: `me` would pack
  this input as a `Descriptor` record. NOT "`me`'s cascade parses it" (`me`
  parses `multi`, and `multi` is `host_admits=false`) and NOT "some `--as`
  succeeds" (`--as md1` succeeds on `multi`). Stated because §7's own history
  shows what an undefined column grows into (R0 r6's NEW-M2).
- **`canonical` is REQUIRED on every row where `host_admits` is true** — the
  re-encoded descriptor string both parsers must produce. Requirement 4 is
  stated over it.
- **`device_probe`** marks a row whose input PANICS the device, and names the
  site (R0 r6's NEW-M6 — §4.2 recorded TWO panic sites). A panic would crash
  the suite rather than fail it, a false-signal shape. **AMENDED 2026-08-29
  (S2): only `"panic:encode"` survives** (§4.2 defects 1–2 — parse ACCEPTS and
  `Descriptor.Encode()` panics; the Go test may parse the input but must NOT
  call `Encode` on the result). S2 fixed §4.2 defect 4's parse panic as
  Rust-convergence — the fingerprint guard is `!= 4` and the parser errors
  cleanly — so `"panic:parse"` retired with it: that row carries a MEASURED
  `device_admits` like any other, **`device_admits` is now present on every
  row**, and requirement 5's non-vacuity count skips none. S2 does not touch
  `Encode`, so the two `"panic:encode"` rows are unchanged.
- **`covers`** (REQUIRED, non-empty array of coverage-manifest tags): which
  required-row bullets this row discharges; additional tags are permitted
  only for the rows the manifest names (R0 r7's NEW-M1 — see the manifest
  below the bullets, which §11 item 3 counts; R0 r6's NEW-I2).
- **`md1_admits`** (boolean, REQUIRED on every row — no default; R0 r4's
  NEW-M2 measured that a false default is backwards for most host-admitted
  rows, and a default that must be overridden on most rows is not a default):
  whether `me sysw pack --as md1` carries this row's input. It is an
  INDEPENDENT axis, not a qualifier on `host_admits` (R0 r4's NEW-I1): the
  `multi` row is `host_admits=false, md1_admits=true` — the widening
  direction §5.5's two-flag argument rests on — and `/0/*` is
  `host_admits=true, md1_admits=false`. §11 item 3's counting test asserts
  the field is present on every row.
- **the gate fields** (REQUIRED on every `gate`-tagged row, absent
  elsewhere — §5.1's gate states an intent and two invariants, and these
  rows are its executable precision; terminal fold): `gate_open`
  (boolean — on an omitted-`--as` pack, does §5.1's gate open?
  **AMENDED 2026-08-29 (S2):** this read *"after record classification
  fails"*, the same precondition §5.1 carried and S2 abolished — the gate is
  consulted on identification, so the question is asked on EVERY omitted-`--as`
  pack, including inputs that classify),
  `outcome` (one of `record-refusal` — the shipped exit-4 record surface,
  unchanged; `as-decides` — §5.1's choice block at `EXIT_USAGE` (2);
  `descriptor-refusal` — a §6 row at `EXIT_REFUSED` (3); `multi-record` —
  §6's multi-record row at `EXIT_INVALID` (4)), `refusal_row` (on the
  `descriptor-refusal` and `multi-record` outcomes only: a slug naming the
  §6 row whose text the test asserts — the slug-to-text binding lives with
  §11 item 4's per-row text tests), and `exit_code` (2, 3 or 4, asserted
  against the real invocation). All four are asserted by the RUST test
  only — the gate is `me`'s surface, and the device has no counterpart;
  the Go test ignores these fields.

**The row set MUST include**, at minimum, one row for each of:

- **each of the four formats**, on its happy path, from the fork's own
  `nonstandard/parse_test.go` fixtures where one exists (provenance in `source`);
- **the promotion near-misses of §4.5** — all **fifteen** rows of that table;
- **every shape §4.7 narrows** — `tr(sortedmulti)`, `wpkh(sortedmulti)`,
  `pkh(sortedmulti)`, `sh(wpkh(sortedmulti))`, `wsh(KEY)`, `sh(KEY)`, `k=0`,
  `k=−1`, `k>n`, `n=16` with the `sortedmulti` DIRECTLY under `sh`, `n=21`
  under `wsh`, a mixed mainnet/testnet multisig, a hardened use-site
  component (`<0;1>/*h`), and a non-consecutive multipath (`<0;2>/*`):
  `host_admits=false`, `device_admits=true`. These are the rows the invariant
  is *for*. And one row on the ACCEPTED side that r1's prescription would
  have wrongly refused: `sh(wsh(sortedmulti(2, 16 keys)))` —
  `host_admits=true`, spendable, with its `canonical` and `address_0`
  (R0 r2's NEW-I2), authored fresh from r6's recorded construction (NEW-N1);
- **the md1-representability splits of §5.3** — `/0/*` and `<0;1>`-without-
  wildcard (host-admitted, `md1_admits=false`) each with the device-route
  `address_0`, plus a CHILDLESS input (`md1_admits=true`) whose `address_0`
  proves (a′)'s materialisation, plus the three MIXED-path rows of §5.3's
  per-key rules (R0 r3's NEW-C1): `K1/0/*`+`K2/<0;1>/*` and
  `K1/<0;1>`+`K2/<0;1>/*` (both `md1_admits=false`) and
  childless+`K2/<0;1>/*` (`md1_admits=true`, materialised per key), each with
  the device-route `address_0`;
- **every shape §4.2 narrows** — BlueWallet with no `Format:`, with zero keys,
  with `Derivation:` after the keys, with **no `Derivation:` at all** (R0's C1
  — `device_admits=true` on the input, host refused, no `canonical`), and a
  short-fingerprint file (**AMENDED 2026-08-29 (S2):** it was marked
  `device_probe: "panic:parse"`; the marker retired with the parse panic and
  the row carries a measured `device_admits: false`); the
  no-`Format:` row and the zero-key row carry `device_probe: "panic:encode"`
  (parse ACCEPTS, `Encode()` panics — measured, R0 r6's NEW-M6; the zero-key
  row is §4.2 defect 2's exact `Name: only` spelling — a zero-key file WITH
  a `Format:` header encodes cleanly (`wsh(sortedmulti(0,))#w47tv00x`,
  measured) and the marker does not apply to it, R0 r7's NEW-M2);
- **`wsh(multi(…/<0;1>/*))`, a miniscript descriptor, and
  `wsh(multi(…/0/*))`** — `false`/`false` on the host/device axes, the
  `neither` rows the vacuity check needs. **AMENDED 2026-08-29 (S2): the
  named three changed by SUBSTITUTION.** The full-origin `ypub` left — the
  device's parser accepts it after S2's `ypubVer` case, so `false`/`false` is
  no longer true of it and a row contradicting the tag's own definition may
  not keep the tag; it is the sole member of the new `version-gap` bullet
  below. `wsh(multi(2,…/0/*))` took the vacated slot: §11 item 5's case-3
  witness, refused by conjunct 1 under `--as descriptor` PERMANENTLY and by
  §5.3(a) under `--as md1`, so NEITHER carrier admits it — the case the
  shipped witness stopped covering when `--as descriptor` began carrying
  `/0/*`. **`The multi row` in the sentences that follow means the
  `<0;1>/*` one alone** (`neither/wsh-multi`): the new row is
  `md1_admits=false` and carries no address or read-back fields. The
  `<0;1>/*` `multi` row additionally carries `md1_admits=true`, its md1-route
  `address_0` AND `address_1`, and pins the read-back via
  `md_descriptor_contains: "wsh(multi("` (measured: `#656zkmsn` — `multi`
  survives the round trip un-normalised; the pin is `"wsh(multi("` and NOT
  `"multi("`, because `"sortedmulti("` CONTAINS `"multi("` and the shorter
  pin passes on the mutant's own read-back — R0 r7's NEW-I1, both read-backs
  measured). `address_1` is load-bearing (R0 r5's NEW-I3): for
  these keys the sorted and unsorted orderings COINCIDE at index 0 (measured
  identical recv0, divergent recv1), so an index-0 assertion cannot catch
  the forbidden `multi` → `sortedmulti` rewrite — the gate must assert where
  the orderings differ, and the next author should not reach for index 0
  again;

- **a full-origin `ypub`** — `host_admits=false`, `device_admits=true`: the
  `version-gap` bullet, NEW 2026-08-29 (S2), and single-member by
  construction. §4.3 refuses the version host-side while the device's scan
  door accepts it, which is F-426's device half landing ahead of its host
  half by that follow-up's own design (the seam-SAFE direction — the file's
  invariant is `host_admits ⇒ device_admits`, and a device that accepts more
  can never be handed an unreadable payload). It is the LIVE tracking witness
  for F-426's open host half: when `me`'s admission widens in F-426's own
  cycle, `host_admits` flips and this bullet retires with the row. It is NOT
  `promotion-near-miss` — that is §4.5's closed fifteen-row membership and
  this is a wrapped descriptor that never reaches the promotion branch — NOT
  `narrowed-4.7` (the refusal is §4.3's version byte, not a §4.7 narrowing),
  and NOT `gate` (it carries no gate fields);
- **the whitespace rows of §4.6** — trailing `\n`, CRLF, leading space. These
  are the only rows where the host is *wider*, and they are permitted **only
  because `canonical` is what gets packed** (§4.6). The file states that in its
  `_comment`, and a row where the host is wider and `canonical` is absent is a
  defect the tests must reject;
- **the §5.1 gate's adversarial set (`gate`) — rounds 16–19's derivations
  made executable; §5.1's gate is NORMATIVE through these rows.** Every
  row carries the four gate fields, asserted by the Rust test against the
  real `--as`-omitted invocation:
  1. all **fifteen §4.5 rows** under `--as` omitted — the same fifteen
     physical rows, second-tagged `gate`: gate OPEN on every one; outcome
     `as-decides` at exit 2 on the seven rows promotion accepts (§4.5
     rows 1, 2, 5, 6, 7, 13, 14), `descriptor-refusal` at exit 3, each
     naming its §6 row, on the eight it refuses (rows 3, 4, 8, 9, 10,
     11, 12, 15);
  2. **six records with hostile payloads** — `text: my wallet (2 of 3)`,
     `pass: hunter (2)`, `text: note: hello`, `seed: abandon abandon
     abandoz`, `tx: zz` (delivered via `--in` — on argv the shipped
     bearer-transaction guard preempts at exit 3, R0 r20's M2), and a
     `text:` record whose payload is a real
     `xpub…` token: parentheses, colons and base58 material inside a
     record payload must not open the gate — gate CLOSED,
     `record-refusal`, exit 4;
  3. a **mistyped bare mnemonic** — gate CLOSED, `record-refusal`, exit 4
     (the operand class of the `sysw_cli.rs:1928` pin); and
     **`deadbeef: xpub…`** — an 8-hex key that is NOT a BlueWallet header
     yet fronts a real xpub: gate OPEN on the fingerprint disjunct,
     `descriptor-refusal` (r17's intended flip — the one previously
     normative outcome the demotion lost, restored as a row per R0 r20's
     M1; invariant-1 boundary witness);
  4. one **malformed string of each bech32 record class** — `md1…`,
     `mk1…`, `ms1…`, `mt1…`: gate CLOSED (the bech32 charset is not a
     base58check envelope), `record-refusal` naming the class, exit 4;
  5. the **multi-record split, MNEMONIC FIRST** — a valid mnemonic line,
     then a `wsh(sortedmulti(…))` line: gate OPEN on the second line,
     outcome `multi-record`, exit 4. The ordering is load-bearing (r19's
     plan note): a descriptor-first row passes with or without per-line
     scope and witnesses nothing;
  6. the **buried refused keys** — three records files, each a valid
     record line followed by a bare `Zpub…`, a bare `tpub…`, or a
     `[fp]xpub…` with no path: gate OPEN, outcome `descriptor-refusal`
     over the WHOLE input — step 5's generic four-forms text, exit 3 (the
     divergence §6 states explicitly);
  7. the **edge tokens** — a base58check token with a valid checksum and
     a 77-byte payload (gate CLOSED, `record-refusal`, exit 4); `[` alone
     (gate OPEN, `descriptor-refusal`, the unparseable-file row carrying
     branch 4's error, exit 3); and `xpub…/` with a trailing slash (gate
     OPEN, `descriptor-refusal`, the unparseable-file row carrying branch
     4's error, exit 3);
  8. **the impossible-wallet trio (conjunct 8, PLAN-r1's C1; r2)** — a
     colliding-origin `wsh(sortedmulti(…))` (two xpubs, one
     `(fingerprint, origin)`), a duplicate-`(xpub, use-site)` slot pair,
     and the colliding-origin `wsh(multi(…))` twin — whose conjunct-8
     refusal binds **BOTH `--as` paths, the `multi` twin included**
     (AMENDED 2026-08-29; see below). Gate OPEN, `descriptor-refusal`
     (`refusal_row: key-identity`, or `key-identity-duplicate` for the
     slot row), exit 3 — all three rows on both `--as` paths.

     **AMENDED 2026-08-29 (IMPL-S1S3 adversarial review, C1).** This clause
     previously read *"whose conjunct-8 refusal binds the `--as md1` path
     ONLY: under `--as descriptor` a `multi` gets conjunct 1's permanent
     refusal first"*. That ordering is exactly what produced C1: running
     conjunct 1's `--as`-DEPENDENT `multi` arm ahead of the flag-independent
     conjuncts short-circuited them, so `wsh(multi(0,…))` — anyone-can-spend
     — answered `--as descriptor` with conjunct 1's *"This wallet can still
     be engraved"* while `--as md1` refused the same file permanently.
     §5.4's carriage rule already said why that cannot be right: the no-path
     determination *"quantifies over both paths, so it needs no flag"*.
     **The shipped order is: conjunct 1's `--as`-INDEPENDENT shape half
     first, then conjuncts 2–8, then conjunct 1's `multi`-under-
     `--as descriptor` arm LAST.** Conjunct 1's referral is therefore reached
     only when 2–8 hold — which is exactly when `--as md1` admits the wallet,
     making the referral true by construction rather than by a text edit. A
     `multi` that PASSES conjunct 8 still gets conjunct 1's permanent refusal
     under `--as descriptor`; a `multi` that FAILS it gets the key-identity
     refusal on both paths, which is what §6's key-identity row
     (*"**`EXIT_REFUSED` (3)**, both `--as` paths"*) had always required and
     the old order contradicted.
     `host_admits=false`, `md1_admits=false`, and **no address fields** —
     a colliding-origin wallet derives byte-identical addresses to a clean
     control (measured, r2's NEW-M5), so the refusal assertion itself is
     the row's witness;

**The coverage manifest — NORMATIVE; §11 item 3 counts against it (R0 r6's
NEW-I2: without a countable anchor, dropping the childless+`<0;1>/*` mixed
row — the row that gates R0 r3's NEW-C1 — left every countable property of
the file intact).** Every required row carries `covers`; the test asserts
every tag present with at least its minimum and no unknown tags. A row may
carry MORE than one tag ONLY where its input genuinely discharges each
bullet — in the required set the qualifying rows are exactly the fifteen
§4.5 rows, which double as the gate bullet's clause 1 and carry `gate` as
a second tag; two of the fifteen carry a third, the original overlap pair
(R0 r7's NEW-M1): the `xpub…\n` near-miss (`promotion-near-miss` +
`whitespace` + `gate`) and the bare-`xpub` happy path (`formats-happy` +
`promotion-near-miss` + `gate`). `covers` entries are distinct within a
row, and the file carries at least **72 physical rows** (the minima sum to
**89** tag-slots; the fifteen shared §4.5 rows absorb 15 of `gate`'s
slots, and the original pair's two extra tags absorb 2 more: 89 − 17 =
72), asserted as a floor — so a dropped row cannot be counted around by
retagging or by duplicate tags (R0 r7's NEW-M1). **AMENDED 2026-08-29 (S2):**
88 → 89 slots and 71 → 72 rows, the new `version-gap` bullet's single row;
the overlap stays 17, because both rows S2 touched are single-tagged:

| tag | bullet | min rows |
| --- | --- | --- |
| `formats-happy` | the four formats, happy path | 4 |
| `promotion-near-miss` | §4.5's fifteen-row table | 15 |
| `narrowed-4.7` | shapes §4.7 narrows | 14 |
| `accepted-extreme` | `sh(wsh(sortedmulti(2, 16 keys)))` | 1 |
| `narrowed-4.2` | BlueWallet shapes §4.2 narrows | 5 |
| `neither` | `wsh(multi)` with `<0;1>/*`, miniscript, `wsh(multi)` with `/0/*` | 3 |
| `version-gap` | a full-origin `ypub`: §4.3-refused, device-ADMITTED (F-426's witness) | 1 |
| `whitespace` | §4.6's rows | 3 |
| `md1-splits` | §5.3's splits: `/0/*`, `<0;1>`, childless, three mixed | 6 |
| `gate` | §5.1's gate — the eight adversarial clauses of the gate bullet | 37 |

**A second, separate assertion, because §5.3 showed a string comparison is not
enough.** Rows may carry `address_0` and `address_1` — receive addresses at
indices 0 and 1 — and `md_descriptor_contains`, a substring the
`md descriptor` read-back of the encoded set must contain. EVERY such field a
row carries is asserted (R0 r6's NEW-I1: r5's fold added the values to the
`multi` row and left this paragraph naming `address_0` alone, so the
`multi` → `sortedmulti` mutant still passed every stated assertion — a gate
that cannot fail, twice). The Go test derives each carried `address_N`
through `address.Receive(…, N)` on the parsed INPUT (the scan door's own
string — the C1 row has no `canonical`; R0 r6's NEW-N2) wherever
`device_admits` is true. The Rust test derives each through the md1 round
trip wherever `md1_admits` is true — including `host_admits=false` rows like
`multi`, whose address assertions run only through the md1 route — and
asserts `md_descriptor_contains` against the round trip's read-back. Where `md1_admits` is false on a row whose input is otherwise
ADMITTED (cascade AND §4.7 — every §5.3-split row is; the rows refused by
§4.7's own conjuncts are not, and their assertion is the `host_admits=false`
column's, citing their own conjunct — R0 r5's NEW-I1), the Rust test asserts that the md1 path REFUSES **citing
§5.3(a)/(a″)** (the citing clause is R0 r4's NEW-M2 second point — a refusal
for an unrelated cause must not satisfy the assertion; this is what turns
§5.3(a)/(a″) from prose into a gate). This is the check that would have
caught the `/0/*` collapse, and no string-level comparison would have.
Rows may also carry **`wallet_id`** — the WalletPolicyId fingerprint (walk
W10): asserted by BOTH suites, each computing it from its own
implementation — the F-212 class (a cross-language identity divergence no
per-repo test can see) made into a standing gate. Requirement 4's
canonical-level invariant binds rows with `host_admits=true` — a `multi`
row packs no `Descriptor` record, so the invariant is vacuous there by
construction, not by exemption.

---

## 8. Scope and ordering

The phasing is **S1** the input cascade plus the shared vector gate, **S3**
`--as md1` end to end, **S2** `--as descriptor` end to end — **S3 before S2,
ruled by the operator 2026-08-28 (F-418)**, reversing the original S2-first
order this section previously recorded as an open question. Two things about
the asymmetry are recorded below because they are what decided it.

**S1 is genuinely shared.** Both `--as` values consume the same cascade (§4) and
the same narrowing profile (§4.7), and the vector file (§7) is what pins them.
Neither S2 nor S3 can be specified without it, and it is the only part of the
cycle both need.

**S2 and S3 are not the same size, and the measured difference runs opposite to
the intuition that ordered them.** From §2.3 and §5.3:

| | `--as descriptor` (S2) | `--as md1` (S3) |
| --- | --- | --- |
| host work | pack the canonical string | build the md1 AST, chunk, encode |
| device work | **`sysw.Classify` needs a descriptor arm** | **none — `ClassMDMK` already classifies and is already admitted** |
| reaches a device without a reflash | **no** | **yes** |

**S2's device arm is a change to normative admission behaviour**, so under §3 it
lands in Rust first with §7's vectors and is then ported — which is what this
cycle already is, so nothing about the rule is bent. But it does mean **S2, the
first shipping phase, is the one that cannot be demonstrated on the operator's
machine without a firmware build and flash**, while S3 could be demonstrated the
day it compiles.

**RULED 2026-08-28 (F-418): S1 → S3 → S2.** The operator is away from the
SeedHammer II and it is not connected, which decides the question this
section previously left open: S2 cannot even be demonstrated without a
firmware build and flash, let alone satisfy §11 item 6's on-device
acceptance, while S3 demonstrates the day it compiles. §11 items 1, 4 (its
`--as descriptor` rows) and 6 bind S2's ship, so S1 and S3 could plan,
build, demonstrate and ship entirely at the desk.

**AMENDED 2026-08-29 (S2): S2 is UNPARKED and building.** The SeedHammer II
is back on the bench, which is the condition F-418 named. The order S1 → S3
→ S2 stands as the record of what shipped when. §11 item 4's
`--as descriptor` rows turned out to be EMPTY — the S3 build reached every
§6 trigger, and S2 SUBTRACTS the one row (`window-not-in-build`) whose
trigger it removed — so what actually binds S2's ship is item 1, and item 6,
which no desk can discharge: a `ClassDescriptor` record loaded and DISPLAYED
on the real device, the operator's eyes being the instrument.

---

## 9. What is NOT verified

Stated plainly, because a spec that only lists what it proved is claiming the
rest.

1. **Nothing has been run on hardware.** Every device-side measurement is a Go
   function called from a scratch module on this machine. No payload has been
   written to flash, no descriptor has been displayed on the screen, and no
   plate has been cut.
2. **One of the three admission-table cells has now been exercised**
   *(AMENDED 2026-08-29 (S2 P3.5))*: S2's P3.2 executed
   `admits(progWalletPolicy, ClassDescriptor) == true` → rendered
   `DescriptorScreen` for the FIRST time — a named headless walk drives a
   real `me sysw pack --as descriptor` container through `walletPolicyFlow`
   to the screen, no panic. The other two cells (`progBundle`,
   `progMultisig`) remain declared and INERT — no consumer, records
   unoffered — with a follow-up filed at S2's records phase. On-hardware
   confirmation of the one built cell stays the operator's (§11 item 6).
   *(Closure-is-lens-closure, second clause: a gate that has never
   executed is a hypothesis. This one has now run, in the simulator;
   the hardware run remains.)*
3. **The `--as md1` address equality now rests on ten descriptor shapes** —
   all seven §4.7 forms including the 15/16/20-key extremes, `wsh(multi)`,
   and the childless-mixed case — with Go and Rust agreeing at receive
   index 0 on every one and at index 1 on seven (r6, measured; the original
   "one data point" caveat is discharged). **Still unmeasured: change
   addresses and testnet.** §7's `address_0`/`address_1` requirements cover
   receive; change-chain and testnet rows remain the gap this item names.
4. **`md-cli` at repo HEAD was used for the `md` measurements; `me` links the
   published `md-codec` 0.42.0.** They agree on version *string*. The `md`
   binary used was built `Aug 27 17:12` and HEAD is `Aug 27 19:41`; the two
   commits in between (`a9b6da1b`, `bb2151dc`) touch dependency declarations
   only, not codec behaviour — checked with `git log --since` over
   `crates/md-cli crates/md-codec`. **Not checked:** whether the *published*
   0.42.0 tarball is byte-identical to the tree's `md-codec`, only that it
   exports the same names with the same signature.
5. **The Go probe used Go 1.26.3**, per the project note that `go.mod`'s pinned
   1.25.10 cannot build `./gui/`. `nonstandard`, `bip380`, `sysw` and `seal`
   built and ran clean under it. Whether TinyGo compiles a new `sysw.Classify`
   arm for the RP2350 target **has not been checked**, and the device build is a
   real gate the fork has been caught by before.
6. **Negative claims and their scope.** "No Rust counterpart to `nonstandard`"
   (§3) was established by searching `mnemonic-engrave/crates` and
   `descriptor-mnemonic/crates` for a BIP-380 descriptor parser and by reading
   `me-cli/Cargo.toml`'s complete `[dependencies]` block — it does **not** cover
   `mnemonic-toolkit`, `mnemonic-secret`, `mk-codec` or `mnemonic-transaction`,
   which were not searched. "No label field in md1" covers
   `md_codec::TlvSection`'s five declared fields and the `unknown` TLV
   passthrough; a caller could smuggle a label through `unknown`, which nothing
   does today.
7. **§6 has not been re-walked row-by-row since its final text landed.** The
   operator journey walk RAN, and its findings are folded throughout — §6's
   own rows cite walk W4/W5/W11/W13 and F-419 — with the walk lens closed
   at r19. What remains unwalked is only the rows added or reworded after
   the walk, which the closure rounds covered as text rather than as a
   live journey. A systematic row-by-row walk of §6's final table is a
   plan-phase option, not a gate this spec holds open.

---

## 10. Out of scope, with reasons

- **`ClassAddress`.** Admitted by **zero** programs — `gui/sysw_admit.go`'s
  table, read whole, has no `sysw.ClassAddress` cell. `seal.Classify` produces
  it and `sysw.Classify` does not (both measured, §2.4). There is nothing on the
  device that would consume one, so packing one would create a record with no
  reader.
- **Miniscript descriptors, both `--as` values.** `--as descriptor` cannot: the
  device's `bip380.Parse` refuses them (§4.3, measured). `--as md1` could in
  principle — `md encode` accepts a miniscript *template* (`wsh(or_d(pk(@0/<0;1>/*),
  and_v(v:pkh(@1/<0;1>/*),older(52560))))` encoded clean, chunk-set-id `0x1fb17`)
  — but decomposing a *concrete* miniscript descriptor into that template needs a
  miniscript parser `me` does not have and §4.7 declined to add. Deferred whole,
  rather than half-supported.
- **`wsh(multi(…))` under `--as descriptor`.** The device refuses it. Widening
  the host's DESCRIPTOR-record path would break §7's invariant, and rewriting
  `multi` to `sortedmulti` is a **different policy** — key ordering at spend
  time — not a normalisation. `--as md1` carries it, through §4.7 conjunct 1's
  md1-path admission.
- **Signing, PSBTs, transactions.** `me` signs nothing; the transaction
  vocabulary is `mt`'s (`crates/me-cli/src/sysw/record.rs`, `TX_PREFIX` doc).
- **Fixing the Go parser's defects in Go.** §4.2's panic, §4.3's threshold gap
  and §4.1's discarded diagnostics are real and are recorded here. Under §3 the
  Rust primary is written correct and the Go side converges *later*, as its own
  cycle. This cycle changes Go in exactly one place — §5.2's `sysw.Classify`
  arm — and nothing else.
- **`me seal`'s descriptor path.** `seal.Classify` already returns
  `ClassDescriptor` and `permitted` already refuses it (§2.4). Whether the
  `seal` container should carry descriptors is a separate question with a
  separate allow-list, and this cycle does not touch it.
- **The `md` / `me` help-text disagreement.** `md encode --help`'s example uses
  `multi`, which the device refuses. Worth a follow-up against `md`; not this
  spec's to change.
- **Widening md1's wire format to arbitrary use-site paths — considered and
  DECLINED (operator ruling 2026-08-28, F-417).** The shapes md1 cannot
  represent are BIP-388's tail discipline, not a codec gap: the discipline is
  what buys an engraved plate compactness inside the BCH budget and an
  unambiguous restore. The one real-world unrepresentable shape (`/0/*`) is
  carried exactly by `--as descriptor` — the two-output ruling working as
  designed — and the rest are transcription slips or unmeasured rarities. A
  wire change is the constellation's costliest class (published crate, BIP
  draft, Go port, shared vectors, `WalletPolicyId` identity, plates already
  cut). Consumers refuse and name the carrying alternative (§5.3); the
  extension seam, if a legitimate exotic wallet ever needs carrying, is an
  additive TLV tag with a criticality story, not a change to this block. A
  tripwire doc comment lives at `md-codec/src/use_site_path.rs`.

---

## 11. Acceptance

This spec is GREEN when a review round returns 0 Critical / 0 Important. It is
**done** when, in addition:

1. `me sysw pack --as descriptor --in <each of the four formats>` produces a
   container whose `me sysw show` reports one `Descriptor` record
   (single-document mode, §5.1), and the device's `sysw.Classify` classifies
   that record `Descriptor`. **S2's item** (F-418): it needs the device arm,
   so S1 and S3 close without it (R0 r6's NEW-M5).
   **AMENDED 2026-08-29 (S2), both halves.** The `show` surface this item
   assumed did not exist: `show` printed per-record lines for `Class::MdMk`
   and `Class::Mt` and NOTHING for a `Descriptor` record. It is built —
   one confirmation block per public `Descriptor` record, reusing §5.4's
   identification block so `show` carries the same vocabulary `pack` printed
   to a stderr that is gone a week later. And the mechanism clause died with
   its column: the device side is exercised by the Go test's DERIVED rule
   (§7's `sysw_class` amendment), which asserts `sysw.Classify` over every row
   in the file rather than four sampled ones. The host half of this item —
   one record per format, classifying `Descriptor`, byte-equal to the
   `canonical` the device measured — closes at the desk; the DEVICE half
   still needs the flashed build.
2. `me sysw pack --as md1 --in <each of the four formats>` produces a container
   (single-document mode, §5.1) whose records `md decode` reads back to the
   expected template — with §5.3(a′)'s materialised `<0;1>/*` where the input
   was childless — and whose derived receive address 0 equals the one the Go
   `address` package derives from the original descriptor. The JSON exemplar
   must use a non-`/0/*` descriptor: the fork's own JSON fixture is `/0/*`,
   which `--as md1` refuses per §5.3(a) (R0 r4, verified in passing).
3. `descriptor_seam_vectors.json` exists in both repos with one sha256, both
   tests pin it, and both suites are green — **and the file's row set covers
   every bullet of §7**, checked by a test that counts the `covers` tags
   against §7's coverage manifest (per-tag minima met, no unknown tags, the
   row floor met, and both `covers` and `md1_admits` present on every row),
   not by reading (R0 r6's NEW-I2; r7's NEW-N1).
4. Every refusal in §6 has a test that reaches it and asserts the *text*, not
   just the exit code. The `--as descriptor`-only rows among them are S2's
   (F-418); the rest bind S3 (R0 r6's NEW-M5).
   **AMENDED 2026-08-29 (S2): that S2 set is EMPTY, measured, and S2
   SUBTRACTED a row.** S3 shipped with all 36 §6 triggers reachable, so no
   row was waiting on `--as descriptor`; S2 retired `window-not-in-build`,
   whose trigger it removed, leaving **35**. The count is asserted in the
   test file itself, against `descriptor::Row::ALL` and against the vector
   file's own `refusal_rows` vocabulary, so the three cannot drift. The multi-record row's test
   reaches it with the MNEMONIC-FIRST ordering (r19's plan note): a
   descriptor-first input passes with or without the per-line gate scope
   and is not a witness for r18's repair.
5. `--as` omitted with a descriptor input at least one `--as` value
   CARRIES in this build exits **2** and prints §5.1's block; with an input
   nothing carries, the input's own refusal fires directly at **3** — the
   admission refusal, or the neither-path refusal (§5.4's carriage rule,
   r12–r13). Five cases tested: carried; inadmissible with `--as` omitted;
   carried by NEITHER value; inadmissible with explicit `--as descriptor` —
   the admission refusal (r14's new-I1 ordering); and a `multi` form with
   explicit `--as descriptor` — conjunct 1's permanent refusal (r15's
   new-I3).
   **AMENDED 2026-08-29 (S2): the sibling case is GONE, and case 3 changed
   witness.** The sibling was *"`--as descriptor` in a build where its path
   has not shipped exits 3 and prints §5.1's window refusal, both alternative
   variants"*; no build state reaches it, so it retired with the window and
   with §6's `window-not-in-build` row. Case 3 was
   `wsh(sortedmulti(2,…/0/*))` — admitted, and carried by nothing while the
   descriptor path was parked. That input is now CARRIED and exits **2**, so
   the witness is `wsh(multi(2,…/0/*))`: conjunct 1 refuses `multi` under
   `--as descriptor` permanently and §5.3(a) refuses `/0/*` under `--as md1`,
   which is what "carried by neither" means in a full build. The old witness
   is kept as a sixth case pinning the 3 → 2 flip itself, so the exit code S2
   changed is asserted rather than merely no longer asserted.
6. §9's item 2 is discharged: a `ClassDescriptor` record has been loaded on a
   real device and displayed, at least once, before this is called shipped —
   binding **S2's ship only** (F-418): S1 and S3 close without it, and it is
   parked with S2 until the device is back on the bench.
