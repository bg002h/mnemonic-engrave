# SPEC — descriptor input (`me sysw pack --as descriptor` / `--as md1`)

**Status:** DRAFT, round 0. No R0 review has run. **No code may be written
until a round closes 0C/0I** (project `CLAUDE.md` — this is risk-set work: it
changes normative admission behaviour and it decides which wallet an operator
engraves).

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
  d402f18   (NOT pushed; origin/main is 0b656d7 — a permission block, not a code problem)
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
§3.3.2 is the normative source for that table and lists the same three rows.

**`sysw.Classify` never returns it.** `sysw/record.go:97` dispatches the three
reserved prefixes and then calls `classifyConstellation`
(`sysw/classify.go:34`), whose arms are: strict BIP-39 mnemonic, strict `ms1`,
`codex32.ValidMD || codex32.ValidMK`, `codex32.ValidMT` — and then
`ClassUnknown`. There is no descriptor arm, and the comment at
`sysw/record.go:94` says the omission is deliberate and mirrors the Rust
primary.

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
`descriptor-mnemonic/crates/md-cli/src/parse/template.rs` (2619 lines).
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

A line-oriented `Key: value` format. `nonstandard/parse.go:74`.

- Lines that are empty or begin with `#` are skipped.
- Every other line **must** split on the two-character separator `": "` — a
  line without it is a hard error (`bluewallet: invalid header: %q`).
- Recognised headers: `Name` → `Title`; `Policy` → `%d of %d` via `Sscanf`,
  giving threshold and expected key count; `Derivation` → a `bip32` path;
  `Format` → one of exactly `P2WSH`, `P2SH`, `P2WSH-P2SH`.
- **Any other key is treated as a cosigner**: the key is a hex master
  fingerprint (≤ 4 bytes), the value an extended key.
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

**NORMATIVE:** `me` **refuses** all three shapes: a BlueWallet file with no
`Format:` header, a BlueWallet file with zero cosigner lines, and a BlueWallet
file whose first cosigner line precedes its `Derivation:` header. Each refusal
names its cause (§6). Refusing is free under §7's invariant — the host may be
narrower than the device, never wider.

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
  `<a;b;…>` range).

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
sharpest single constraint on `--as descriptor`.

Four shapes the Go parser **accepts and should not**, measured:

| input | Go verdict | why it is wrong |
| --- | --- | --- |
| `wsh(sortedmulti(0,…))` | ACCEPT, threshold 0 | unspendable-by-anyone / spendable-by-nobody-checked |
| `wsh(sortedmulti(5,…))` with 2 keys | ACCEPT, threshold 5 | **unsatisfiable — funds locked forever** |
| `tr(sortedmulti(2,…))` | ACCEPT | taproot multisig is `multi_a`/`sortedmulti_a` (BIP-386); `tr(sortedmulti(…))` is not a descriptor |
| `wpkh(sortedmulti(2,…))`, `pkh(sortedmulti(2,…))` | ACCEPT | a single-key script wrapping a multi |

### 4.4 Format 3 — `{label, descriptor}` JSON

`nonstandard/parse.go:41`. `json.Unmarshal` into `struct{ Label, Descriptor
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

`me` accepts the same document shape. **NORMATIVE:** `me` matches field names
case-insensitively, exactly as the device does — a host that required lowercase
`descriptor` would refuse a file the device takes, which is a usability defect
rather than a safety one, but is still a needless divergence.

### 4.5 Format 4 — the promoted bare key, and why it needs its own scrutiny

`nonstandard/parse.go:56`. `bip380.ParseKey(nil, enc)` on the **whole file**,
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
  `P2WPKH`, `Ypub` → `P2SH_P2WSH`, `Zpub` → `P2WSH`. Note `ypub` is listed in
  the version constants but **has no case in the switch**, so it hits `default`
  and is refused.

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
(`SPEC_systemwide_payloads` §6.4), so a record cannot contain a newline by
construction. The device never sees the whitespace the host absorbed.

### 4.7 The admitted grammar — the narrowing profile, NORMATIVE

`me` admits a descriptor only if, after the cascade, it is one of:

```
pkh(KEY)                          wpkh(KEY)                  sh(wpkh(KEY))
tr(KEY)                           wsh(sortedmulti(k, KEY…))
sh(wsh(sortedmulti(k, KEY…)))     sh(sortedmulti(k, KEY…))
```

…**and** `1 ≤ k ≤ n`, where `n` is the number of keys.

**The list above is the closed accept set, and the rule is stated over it, not
over an enumeration of exclusions:** anything the cascade produces that is not
one of those seven shapes with `1 ≤ k ≤ n` is refused. The exclusions *measured*
so far are `tr(sortedmulti(…))`, `wpkh(sortedmulti(…))`, `pkh(sortedmulti(…))`,
`k = 0` and `k > n` — the four rows at the end of §4.3 plus a threshold check
the Go parser does not make at all — and, by inspection of `bip380.Parse`'s
grammar rather than by probe, the key-in-a-script-slot forms `wsh(KEY)` and
`sh(KEY)`, which `Parse` builds as `Singlesig` and which are not descriptors.
That second pair is flagged as **inspection, not measurement**; §7 gives them
rows.

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

Omitting it is a **usage** error, `EXIT_USAGE` (2), not a refusal. The message
does not merely name the flag; it states the choice, because an operator holding
a wallet export does not know which they want:

```
me: this input is a wallet descriptor, and `--as` decides how it is packed.
      --as descriptor   pack the descriptor VERBATIM. The device's Engrave
                        Bundle / Multisig / Wallet Policy programs read it
                        directly. Keeps the exact key serialisation.
      --as md1          decompose to a BIP-388 template plus keys and pack an
                        md1 card set. Carries policies `--as descriptor`
                        cannot, and needs no firmware change.
    They are not interchangeable — `me sysw pack --help` has the comparison.
```

**A path that cannot work fails naming the other path. It never switches.**
This is the operator's ruling and it is the reason §5.4 exists: the two accept
sets genuinely differ, in both directions, so an automatic fallback would
silently change which of two different artefacts the operator engraves.

### 5.2 `--as descriptor`

Packs the **canonical re-encoded descriptor string** — `Descriptor::encode()`,
with its BIP-380 checksum — as one record of class `Descriptor`.

The record is the canonical form, not the operator's bytes: it is single-line by
construction (§4.6), it is what the device's own parser round-trips, and it is
what §7's vectors are stated over.

**This path requires a device change, and the spec says so rather than
discovering it in implementation.** From §2.3: `sysw.Classify` has no descriptor
arm, so a `ClassDescriptor` record packed today would be `ClassUnknown` on the
device and refused. `--as descriptor` is therefore not complete until
`sysw.Classify` gains an arm that calls `nonstandard.OutputDescriptor` — which,
under §3, lands in Rust first (as `mnemonic_engrave::sysw::classify`) with the
§7 vectors, and is then ported.

The classification predicate is stated once, and both sides implement it:

> A record is `ClassDescriptor` iff it parses under §4's cascade **and** matches
> §4.7's grammar.

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

> **NORMATIVE: `--as md1` REFUSES a descriptor whose use-site path is a single
> fixed child index.** The refusal names `--as descriptor`, which carries that
> shape exactly.

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

**NORMATIVE.** Neither path ships bytes the host has not understood. Before any
record is written, `me` prints to stderr what it read:

- the format it matched (§4.1), by name;
- the canonical descriptor, in full;
- the script type, threshold and key count;
- for `--as md1`, the template and the placeholder-to-fingerprint map;
- for a promoted bare key (§4.5), the fact of the promotion.

This follows the standing ruling that all verification is host-side, and it is
what makes §5.1's no-fallback rule usable: the operator can see that the thing
they are about to engrave is the wallet they meant.

### 5.5 The capability split, measured

This table is why there are two flags and no default. Every cell was run.

| descriptor shape | `--as descriptor` | `--as md1` |
| --- | :-: | :-: |
| `wsh(sortedmulti(k, …/<0;1>/*))` | ✅ | ✅ |
| `wsh(sortedmulti(k, …/0/*))` — single fixed chain | ✅ | **❌ §5.3(a)** |
| `wsh(multi(k, …))` — unsorted | **❌ device refuses (§4.3)** | ✅ `md encode` takes it |
| miniscript, e.g. `wsh(or_d(pk(…),and_v(v:pkh(…),older(…))))` | **❌ device refuses** | ❌ §10 — out of scope this cycle |
| `pkh` / `wpkh` / `sh(wpkh)` single-sig | ✅ | ✅ |
| `tr(KEY)` single-key | ✅ | ✅ |
| carries a label | text only, dropped | dropped |
| needs a firmware change to be readable | **yes, §5.2** | **no** |

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
4. input parses as an extended key → report **branch 4**;
5. otherwise → "not a descriptor in any form I know", listing the four.

| the operator's input | what `me` says |
| --- | --- |
| an unparseable file | *"this is not a wallet descriptor in any of the four forms `me` reads: a BlueWallet `Key: value` setup file, a plain BIP-380 descriptor, a `{"label":…,"descriptor":…}` JSON export, or a single extended key. It looks most like `<form>`, which failed because: `<that branch's error>`."* |
| an **empty file** | *"the input is empty (0 bytes). If you meant to pipe a descriptor in, check the producing command's exit status — a failed producer contributes 0 bytes and this is what that looks like."* — the composition hazard `SPEC_constellation_cli_uniformity` §2 calls C-1. |
| a file of only whitespace | as above, with *"the input is `N` bytes of whitespace only."* |
| **`--as` omitted** | §5.1's text. **`EXIT_USAGE` (2)**, not 3 — nothing was refused, a choice was not made. |
| a **wrapper whose inner descriptor is malformed** | the wrapper is named, then the inner error, then the position: *"the `{label, descriptor}` JSON parsed, and its `descriptor` field did not: `bip380: script: missing ')'`. The label was `"…"`. The problem is in the descriptor string, not the JSON."* |
| a **BlueWallet file with no `Name:`** | *"this is a BlueWallet setup file — it has `Policy`, `Derivation` and `Format` headers and `N` cosigner lines — but no `Name:` header, and the device requires one. Add a line `Name: <anything>`."* This is the case §4.2 measured as producing the generic message today, with the real reason destroyed. |
| a BlueWallet file with no `Format:` | *"…no `Format:` header, so the script type is undefined. Add `Format: P2WSH` (or `P2SH`, or `P2WSH-P2SH`)."* §4.2 defect 1. |
| a BlueWallet `Policy: k of n` with a different key count | *"`Policy: 2 of 3` declares 3 cosigners; the file has 2. Cosigner lines are `<8-hex-fingerprint>: <xpub>`."* |
| a BlueWallet file whose keys precede `Derivation:` | *"the `Derivation:` header appears after the cosigner lines, so those keys would be read with no origin path — a descriptor `me` cannot re-parse. Move `Derivation:` above the first cosigner line."* §4.2 defect 3. |
| `wsh(multi(…))` under **`--as descriptor`** | *"the device's descriptor parser accepts `sortedmulti` and not `multi`. This wallet can still be engraved: `--as md1` encodes `multi` policies. (`sortedmulti` differs from `multi` only in key ordering at spend time — it is not a synonym, so `me` will not rewrite it for you.)"* |
| a **miniscript** descriptor, either `--as` | *"`me` reads the descriptor family the device reads: single-sig and `sortedmulti`, optionally under `sh`. This descriptor uses miniscript fragments (`or_d`, `and_v`, …), which neither path handles in this release. `md encode` accepts miniscript **templates** — see §10."* |
| a descriptor with **`/0/*`** under **`--as md1`** | *"md1 records a use-site path as either a multipath group (`<0;1>`) or a bare wildcard (`/*`) — a single fixed chain index has no representation, and encoding it would silently produce a DIFFERENT wallet. Use `--as descriptor`, which carries `/0/*` exactly."* §5.3(a). |
| **`sortedmulti(k, …)` with `k > n` or `k = 0`** | *"threshold `k` with only `n` keys. A `k > n` policy can never be satisfied and the coins would be unspendable. Nothing was packed."* §4.7. |
| `tr(sortedmulti(…))` | *"taproot multisig is `multi_a`/`sortedmulti_a` (BIP-386); `tr(sortedmulti(…))` is not a valid descriptor even though the device's parser accepts it. Check the export."* §4.3. |
| a **bare key whose path matches no script** | the path is quoted back and the three that qualify are listed: *"`[4bbaa801/86'/0'/0']xpub…` is a single extended key. `me` can infer a whole wallet from one only when its origin is `m/44'/0'/0'` (→ `pkh`), `m/84'/0'/0'` (→ `wpkh`) or `m/49'/0'/0'` (→ `sh(wpkh)`). This one is `m/86'/0'/0'` — taproot single-sig, which is not inferable. Supply the descriptor instead: `tr([4bbaa801/86'/0'/0']xpub…/<0;1>/*)`."* |
| a bare key at **account ≠ 0** | *"…this one is `m/84'/0'/1'`. Only account 0 is inferable. Supply the descriptor: `wpkh([…/84'/0'/1']…/<0;1>/*)`."* — measured in §4.5 as a live near-miss. |
| a bare key with a fingerprint and **no path** | *"`[4bbaa801]xpub…` gives a fingerprint with no derivation path, so there is nothing to match a script against. Either give the full origin — `[4bbaa801/84'/0'/0']xpub…` — or drop the brackets entirely, in which case the key's version byte decides."* |
| a bare `Zpub` / `Ypub` | *"a `Zpub`/`Ypub` declares a **multisig** account (`m/48'/0'/0'/2'` and `…/1'`). A multisig cosigner key is not a wallet — supply the full `wsh(sortedmulti(…))` descriptor, or a BlueWallet setup file listing every cosigner."* |
| a bare `tpub` | *"this is a testnet key. Its version byte would map to the **mainnet** path `m/44'/0'/0'`, which `me` will not assume. Supply the descriptor with its real origin."* §4.5. |
| a **bitcoin address** | *"that is a bitcoin address, not a descriptor. No program on the device consumes an address record — see §10."* |

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
   `seedhammer/nonstandard/testdata/descriptor_seam_vectors.json`.
2. **Its sha256 is pinned as a literal in BOTH tests.** Neither test reaches
   across repos; each reads its own copy and compares to the same constant, so
   the copies cannot drift without one suite going red.
3. **The Rust test asserts the host column; the Go test asserts the device
   column.** Neither implementation is ever compared to the other — both are
   compared to the file. That is why it has to be the same file.
4. **Both tests assert the invariant `host_admits ⇒ device_admits`** per row,
   with a message naming the dangerous direction.
5. **Both tests assert the row set is non-vacuous** — at least one `both`, one
   `device-only`, and one `neither` — or a mutant that refuses everything, or
   admits everything, passes.
6. **A mistyped vector fails loudly.** The codex32 file uses a `chars` count for
   this; the descriptor file uses a per-row `sha256` of `input`, because a
   descriptor is long enough that a character count would not catch a
   transcription error inside an xpub.

**Row schema.** `name`, `input`, `sha256`, `host_admits`, `device_admits`,
`format` (one of `bluewallet` / `bip380` / `json` / `promoted-key` / `none`),
`source`, and — where both sides admit — `canonical`, the re-encoded descriptor
string both parsers must produce.

**The row set MUST include**, at minimum, one row for each of:

- **each of the four formats**, on its happy path, from the fork's own
  `nonstandard/parse_test.go` fixtures where one exists (provenance in `source`);
- **the promotion near-misses of §4.5** — all **fifteen** rows of that table;
- **every shape §4.7 narrows** — `tr(sortedmulti)`, `wpkh(sortedmulti)`,
  `pkh(sortedmulti)`, `k=0`, `k>n`: `host_admits=false`, `device_admits=true`.
  These are the rows the invariant is *for*;
- **every shape §4.2 narrows** — BlueWallet with no `Format:`, with zero keys,
  with `Derivation:` after the keys: same shape, `false`/`true`;
- **`wsh(multi(…))` and a miniscript descriptor** — `false`/`false`, and the
  `neither` rows the vacuity check needs;
- **the whitespace rows of §4.6** — trailing `\n`, CRLF, leading space. These
  are the only rows where the host is *wider*, and they are permitted **only
  because `canonical` is what gets packed** (§4.6). The file states that in its
  `_comment`, and a row where the host is wider and `canonical` is absent is a
  defect the tests must reject.

**A second, separate assertion, because §5.3 showed a string comparison is not
enough.** Rows carrying `--as md1` capability also carry `address_0`, the
receive-address-0 the wallet derives. The Rust test asserts it through the md1
round trip; the Go test asserts it through `address.Receive`. That is the check
that would have caught the `/0/*` collapse, and no string-level comparison
would have.

---

## 8. Scope and ordering

The operator's phasing is **S1** the input cascade plus the shared vector gate,
**S2** `--as descriptor` end to end, **S3** `--as md1` end to end, S2 before S3.
This spec is written to it. Two things about it need recording rather than
quietly resolving.

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

**This spec does not reorder the operator's phases.** It records the
asymmetry so the decision is made with it in view, and flags the question: *is
S2-before-S3 still what you want, now that S2 is the one needing a flash?* That
is an operator ruling, not a design one, and it is left open.

---

## 9. What is NOT verified

Stated plainly, because a spec that only lists what it proved is claiming the
rest.

1. **Nothing has been run on hardware.** Every device-side measurement is a Go
   function called from a scratch module on this machine. No payload has been
   written to flash, no descriptor has been displayed on the screen, and no
   plate has been cut.
2. **The three admission-table cells have never been exercised.** §2.3 shows no
   input can reach them, so the code path from `admits(progWalletPolicy,
   ClassDescriptor) == true` to a rendered screen is **untested by construction**
   — not "lightly tested". The first `ClassDescriptor` record in a `sysw` payload
   will be the first ever. *(Closure-is-lens-closure, second clause: a gate that
   has never executed is a hypothesis. This one has never executed.)*
3. **The `--as md1` address equality was measured for ONE descriptor shape** —
   `wsh(sortedmulti(2, …/<0;1>/*))`, 2 keys, mainnet, receive index 0. It was
   **not** measured for single-sig, for `sh(wsh(…))`, for `tr(…)`, for change
   addresses, for index > 0, or for testnet. §7's `address_0` requirement exists
   to close that, and until it does, "md1 preserves the wallet" is a claim
   supported by one data point.
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
7. **The refusal texts in §6 have not been walked with the operator.** Per the
   standing directive, a live journey walk finds what correctness review cannot,
   and §6 is exactly the kind of section it finds things in. It should be walked
   before the plan closes.

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
  the host would break §7's invariant, and rewriting `multi` to `sortedmulti` is
  a **different policy** — key ordering at spend time — not a normalisation.
  `--as md1` carries it.
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

---

## 11. Acceptance

This spec is GREEN when a review round returns 0 Critical / 0 Important. It is
**done** when, in addition:

1. `me sysw pack --as descriptor --in <each of the four formats>` produces a
   container whose `me sysw show` reports one `Descriptor` record, and the
   device's `sysw.Classify` — exercised by §7's Go test — agrees.
2. `me sysw pack --as md1 --in <each of the four formats>` produces a container
   whose records `md decode` reads back to the expected template, and whose
   derived receive address 0 equals the one the Go `address` package derives
   from the original descriptor.
3. `descriptor_seam_vectors.json` exists in both repos with one sha256, both
   tests pin it, and both suites are green — **and the file's row set covers
   every bullet of §7**, checked by a test that counts, not by reading.
4. Every refusal in §6 has a test that reaches it and asserts the *text*, not
   just the exit code.
5. `--as` omitted with a descriptor input exits **2** and prints §5.1's block.
6. §9's item 2 is discharged: a `ClassDescriptor` record has been loaded on a
   real device and displayed, at least once, before this is called shipped.
