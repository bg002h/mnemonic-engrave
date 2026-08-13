# Pre-flash conformance review — do the two sysw implementations actually agree?

- **Reviewer lens:** Rust↔Go seam only (agreement, not either side's correctness).
- **Reviewed:** mnemonic-engrave `sysw-container` @ 5596b21 (`crates/me-cli/src/sysw/`, `me sysw` CLI); seedhammer fork `sysw-port` (`sysw/`, plus `gui/sysw_*.go` for how the port is consumed); contract `crates/me-cli/testdata/sysw_vectors.json` (7 vectors).
- **Method:** function-by-function code comparison, then measured disagreements — the same bytes fed to both sides via a throwaway Rust harness (path-dep on me-cli), a throwaway external Go test in the fork (deleted after; fork tree left clean), the real `me` binary, and a `GOARCH=386 CGO_ENABLED=0 go test` run as the 32-bit-int proxy for the tinygo/thumb device build. Every divergence below is measured unless marked otherwise.

## Verdict

The wire format, hashes, and open semantics agree byte-for-byte on every valid
container tested, including the padded region (Q3: **measured agreement**, but
**no vector pins it** — see I5). The seam breaks in four measured places: a
32-bit integer-width hole in the Go header bounds (C1), an eight-input
classification divergence (I1), a host `show` that panics or misreports on
truncated containers the device cleanly refuses (I2), and an empty-passphrase
payload the host can open but the device never can (I3).

---

## Critical

### C1 — Go `ParseHeader`/`TotalLen` break on the device's 32-bit `int`; Rust rejects on every target

`sysw/header.go:44`: `if int(h.PubLen) > MaxSectionLen || int(h.CtLen) > MaxSectionLen`
`sysw/wire.go:72`: `n := HeaderLen + int(h.PubLen) + int(h.CtLen)`

On a 64-bit host `int(uint32)` is always non-negative and the cap holds. On a
32-bit target — tinygo/thumb on the RP2350 is one — `int(h.PubLen)` for any
value ≥ 2³¹ is **negative**, `negative > 8191` is false, and the bound is
bypassed. Measured:

```
GO[64-bit int] ParseHeader(pub_len=0x80000034) => err=sysw: section too long: pub_len=2147483700 …
GO[32-bit int] ParseHeader(pub_len=0x80000034) => h.PubLen=0x80000034 err=<nil>
GO[32-bit int] TotalLen => -2147483544
RS Header::parse(pub_len=0x80000034)           => Err(SectionTooLong)   (u32→usize is zero-extending; correct on 32- and 64-bit)
```

Consequences on the device, traced (not run on hardware):

- `boundBlob`: `total > len(region)` is false for a negative total → returns
  `(negative, nil)` → `XIPReader.Read`'s `make([]byte, n)` **panics**. Same for
  `Open`: `pubEnd` negative → slice panic.
- Worse, the two negatives can wrap **positive**: `pub_len=0x80000010,
  ct_len=0x80000020` gives `TotalLen = 116` on 32-bit — `Probe`/`Read` then
  succeed and hand out a 116-byte "container" (with an identity) that the host
  refuses as malformed. Disagreement without even a panic until `Open` runs.

A single top-bit flip in `pub_len` (`0x00000043 → 0x80000043`) — one corrupted
bit in the exact region being flashed today — turns a valid payload into one
`me sysw show` refuses cleanly and the device panics on (a boot loop, if the
read is wired at boot: the region persists across resets). Host and device
disagree about the same real bytes in a way the operator acts on ("the machine
is dead").

**Reachability caveat, measured by grep:** no production code calls
`SyswReader().Read()` yet — the gui load flow is unwired (`gui/gui.go:3056` is
the only interface mention; `platform_sh2.go:581` the only constructor). The
defect is latent in today's flash but gates any firmware that reads the region.

**Ownership:** genuine Go-only porting error — Rust is already correct — so per
the Rust-primary rule's exemption (a) it is fixed in Go directly. One line:
compare as `uint32` (`h.PubLen > MaxSectionLen` — the untyped constant fits),
and widen explicitly in `TotalLen`. Note `read.go`'s own `clampRegion` comment
already names this exact hazard ("on a 32-bit target it reappears as a very
large unsigned count") — the discipline existed and missed these two sites.

**Recommendation with teeth:** run the vector conformance test under
`GOARCH=386 CGO_ENABLED=0` in CI (measured working on this machine, ~0.005 s);
it is the cheap standing proxy for the device's integer width.

---

## Important

### I1 — `classify` disagrees on eight measured inputs; the device admits what the host refuses

Same strings, both sides:

| input | Rust (normative) | Go | who leads |
|---|---|---|---|
| `md1…` (vector S-C string) | MdMk | MDMK | agree |
| `md1…` + trailing space | **MdMk** | **Unknown** | Rust trims (`validate_record` → `s.trim()`), Go doesn't |
| `md1…` + leading space | **MdMk** | **Unknown** | same |
| 12-word seed, lowercase | Mnemonic | Mnemonic | agree |
| same seed UPPERCASE | **Unknown** | **Mnemonic** | Go `bip39.Parse` case-folds |
| same seed Mixed-case | **Unknown** | **Mnemonic** | same |
| `aban aban … abou` (≥3-char prefixes) | **Unknown** | **Mnemonic** | Go `bip39.Parse` accepts prefix words |
| `abandon abandon about` (3 words, checksum-valid) | **Unknown** | **Mnemonic** | Go `Valid()` has no 12-word floor, only `len%3==0` |
| 127-char BCH-valid `ms1…` | **Unknown** | **Codex32Secret** | Rust maps >90 chars to `MsTooLong` → Unknown; Go has no cap |
| BCH-valid codex32 with HRP `aa` | **Unknown** | **Codex32Secret** | Go `codex32.New` pins no HRP; Rust reaches Ms only via HRP `ms` |
| `text:` (empty body) | FreeText | FreeText | agree |
| `text:C3BF` / `text:abc` | Unknown | Unknown | agree (hex strictness matches) |

Root causes, all in the Go delegates `sysw.Classify` reuses: the fork's
`bip39.Parse` is an entry-oriented, deliberately forgiving parser
(`ToUpper` + `ClosestWord` + ≥3-char prefix match + any `len%3==0` count) where
Rust uses strict `parse_normalized`; `codex32.New` is a general verifier with
no HRP pin and no engraveable-length cap; and Go does not mirror
`validate_record`'s trim.

**Why it matters at the seam:** `pack` refuses `Unknown`, so payloads packed by
`me` never carry the divergent records — but the device classifies **at load,
for any payload** (foreign packer, and NFC records go straight to `Classify`),
and `gui/sysw_admit.go` keys admission on the class. Every Go-more-permissive
row is a record the host tool refuses to touch and the device hands to a
program as a seed/secret.

**Ownership:** classification is normative behaviour; Rust is primary and none
of these rows require a Rust change — Go must converge (strict word matching in
`classifyConstellation`, HRP pin + 90-char cap for the Codex32Secret arm). Two
open questions for the owner, flagged not decided: (a) Rust's *trim* makes the
HOST the permissive side on padded md1/mk1 — if that is judged a defect it must
be fixed in Rust first, with a vector, per the rule; (b) Rust folding
`MsTooLong` (an engraving-capacity rule) into container classification is a
design choice the spec should state, since the Go port had no way to know it.

### I2 — `me sysw show` panics on one truncated container and misreports another; the device refuses both cleanly

`show` (main.rs:893) never checks `blob.len() >= h.total_len()` — the check
`open` has. Measured against real files:

- S-A truncated to 60 bytes (declares `pub_len=31`): prints an identity, then
  **panics** — `range end index 83 out of range for slice of length 60` at
  `main.rs:944` (`print_digest`'s `&blob[HEADER_LEN..end]`). Exit 101.
- S-D truncated to 100 of 161 bytes (`pub_len=0`, so no digest slice): **exit
  0**, `sealed: true`, `identity: 447de116…` — an identity over partial bytes
  that no complete container has (S-D's real identity is `c7158bf9…`).

Go for the same bytes: `boundBlob` → `ErrTooShort`, no identity, no digest. An
operator who verifies a staged region file with `show` before flashing can be
told "healthy, identity 447de116…" about a truncated file the device will
report as unreadable. Rust-side fix (host CLI): bound-check before slicing and
before hashing; the `.min(blob.len())` clamp on the identity line should become
a refusal, not a clamp.

### I3 — a payload sealed with the empty passphrase opens on the host and can never open on the device

Rust models "no passphrase" as `Option::None`, so `Some("")` is a real
passphrase; Go models it as the empty string, so `""` **is** the missing
sentinel (`open.go:53`). Measured:

```
RS pack(seed, Some(""))   => <sealed blob, 234 bytes>
RS open(that, Some(""))   => Ok((0, 1))
GO Open(that, "")         => sysw: this payload is sealed and needs a passphrase
RS open(that, Some(" "))  => Ok((0, 1))      (normalise collapses to "")
GO Open(that, " ")        => <nil>           (agrees — whitespace-only works both sides)
```

The CLI reaches it: `--passphrase-ask` + bare Enter gives `Some("")`, the
strength line warns ("0 words — BELOW the threshold") and proceeds per §13 D3.
The result is host-seals-what-the-device-refuses — the shape this cycle has
already paid for three times — except here no device input works: the operator
cannot even *type* the empty passphrase, because Go's API reads it as absence.
(The single space is the accidental escape hatch; nobody will guess it.)
Cleanest fix is host-side: refuse the empty passphrase at pack (it is
Rust-primary behaviour, so it lands in Rust with a vector); alternatively the
Go API distinguishes missing from empty, which is uglier for the flows.

### I4 — `normalise` disagrees on U+0130 ('İ'): different KDF input, spec §8a violated (inherited from `seal`)

Rust `char::to_lowercase` applies the full Unicode mapping; Go
`strings.ToLower` the simple one. They differ for exactly one relevant code
point. Measured: `normalise("İ")` → Rust `69 cc 87` (`i` + combining dot), Go
`69` (`i`). A passphrase containing 'İ' produces different KDF input on the two
sides — sealed on host, unopenable on device, indistinguishable from a wrong
passphrase. This is `seal::passphrase::normalise` vs `seal.NormalisePassphrase`
— shared with the **frozen** Sealed Payload feature, so it predates sysw and is
not a today-blocker (the device keyboard cannot enter 'İ', so the device side
can never *create* the divergent input). File as a follow-up with an owning
phase; the fix direction needs the frozen-feature owner's ruling, since
changing either side's normalise changes KDF input for existing payloads.

### I5 — the padded region agrees today, and nothing in the contract keeps it that way (Q3)

Measured, S-C blob + `0xFF` to 65536 bytes, the exact artifact `me sysw pack
--region` emits:

```
GO FileReader.Read()      => 228 bytes (bounded by the header's declared total)
GO Identity(read result)  => 8d0854ef9d1576551aabdaf9044628393466a89cdd4c43e2b1a3131438b5eca3
GO digest                 => e2e1636dd3d333f1466f669088324064
me sysw show <regionfile> => identity 8d0854ef… / digest e2e1 636d … (identical, == vector S-C)
RS open(region, PASS12)   => Ok((1, 1))
```

Both sides also agree on what the *naive* whole-64-KiB hash would be
(`fc7d3635…`), so the equality is structural: `boundBlob`/`total_len()` trim
before hashing on both sides. Two things keep this fragile:

- **No vector covers a padded region.** All 7 vectors are bare blobs of exactly
  `total_len`. A future Go change that hashes the raw region — or a gui wiring
  that does — passes the entire conformance suite while making the device's
  identity never match the host's.
- `gui/sysw_session.go:22` documents identity as "over the **region bytes**" —
  the literal reading is the wrong 64-KiB hash. The load flow is unwired, so
  the docstring is the only current statement of what the device will hash.

Add vector **S-R**: same records/passphrase as S-C, `blob` = the full padded
region (or a `padded: true` flag + derivation rule), same `identity`/`digest`
as S-C. That single vector pins Q3 permanently and corrects the docstring by
force.

---

## Minor

- **M1 — `CliffAbove` is case-insensitive, `cliff_above` is not.** Measured:
  `"ABANDON"×5` → Go true, Rust false. Unreachable through wired paths — both
  sides normalise (lowercase) before calling — but the Go port should converge
  to exact-label matching, or the spec should state the normalised-input
  precondition as normative.
- **M2 — record-count byte truncates identically (`as u8` / `byte()`).** Both
  sides digest `count mod 256`; >255 public records is reachable within the
  8191-byte cap (e.g. ~1365 × `text:`). Not a seam break — a shared latent
  quirk the spec should either bless or bound (`records ≤ 255`).
- **M3 — unsealed headers: bytes 9/10 (KDF/AEAD), byte 11 (reserved), and
  `iterations` are unchecked on both sides.** Agreement, by code reading; noted
  so nobody "fixes" one side alone.

---

## Q2 — what the vector set does NOT cover

The derivation (`coverage.rs` → spec §8.3) is internally sound — every named
test is placed, and the build fails otherwise — but its **domain** is too
small, in two structural ways:

1. **§8.3 names behaviour tests, not wire-format edges.** Rejection cases,
   padding, boundary lengths, and encoding edges have no §8.3 id to point at
   them, so `required_vectors()` can never demand them. The gap class is
   exactly "inputs both sides can be wrong about together": C1 lives on an
   input (`pub_len ≥ 2³¹`) no vector can express because the set contains only
   valid containers.
2. **`Where::Unit` discharges obligations for Rust only.** Tests 7 (MNEMBLOB
   magic refused), 11, 12, 18 are placed on Rust unit tests; the Go port
   inherits nothing from a Unit placement. The Go port's refusal of the
   `MNEMBLOB` magic — spec test 7 — is checked by no test in either repo's
   suite.

Concrete missing vectors, in priority order:

| gap | why it matters |
|---|---|
| **S-R: padded region** (I5) | the artifact actually flashed; nothing pins trim-before-hash |
| **Rejection vectors** with expected error kind: bad magic (incl. `MNEMBLOB`), unknown version, `pub_len`/`ct_len` = 8192 and ≥ 2³¹, truncated blob, unknown KDF/AEAD, iterations out of range, non-UTF-8 section, flipped AAD byte, wrong passphrase | the conformance suite currently proves the Go port opens what it should, never that it refuses what it must; C1 and the tinygo width hole are only catchable here (run them under GOARCH=386 too) |
| **A `pass:` record** | ClassPassphrase — one of the two classes this cycle adds — appears in no vector at all |
| **Per-record class expectations** (a `classes` field, or a classify table incl. Unknown rows) | every divergence in I1 is invisible to the contract *in principle*: the schema cannot express "this record must NOT classify" |
| **Multi-record public section** (≥2 public records) | every digest-bearing vector has exactly 1 public record; the count byte and LF join in `PublicDataHash` are cross-checked only by this review's ad-hoc measurement (`["ab","cd"]` → `fc110fdb…`, both sides) |
| **Boundary sizes**: `pub_len = 8191`, blob = `REGION_LEN` exactly | mutation-grade edges the Rust unit suite has and the contract lacks |

## Q4 — the reported `Renderable` bound: refuted for this seam

Measured by grep: zero occurrences of `Renderable` in the Go `sysw/` package
and zero in the whole of `mnemonic-engrave/crates/`. The identifier exists only
in the fork's GUI md-template code (`gui/md1_expand.go`,
`gui/template_engrave.go`, `md.Template.Renderable`) — fork-native
firmware/GUI admission for template engraving, exempt under Rust-primary rule
(b) and unrelated to the sysw container. No Go-side bound exists on this seam
that Rust lacks, other than the measured items above (C1's is the inverse: a
bound Go *loses* on 32-bit).

## Measured agreements (so the other lenses need not re-derive)

Header layout/encode/parse on valid input; `total_len` tag accounting;
`splitRecords` (UTF-8 gate, empty→none, LF split); `text:`/`pass:` hex
strictness (uppercase rejected, odd-length rejected, empty body valid — all
three measured identical); `public_data_hash` construction incl. 2-record join
and count byte; `identity` label and trim-before-hash; unsealed-carrying-secret
(S-B); whitespace-only passphrase; trailing-byte tolerance in `open`/`Open`;
all 7 vectors' identity/digest values.

## Recommended gate disposition

C1 and I2 are one-line-class fixes on opposite sides of the seam; I3 is a
pack-time refusal; all three are cheap before flash. I1 is a Go-side
convergence batch (strict bip39 matching, HRP pin, 90-char cap, trim decision)
that should land with the classify vector table so it can never silently
regress. I5's S-R vector is the single highest-leverage addition for what is
being flashed today. C1's CI proxy (`GOARCH=386` conformance run) makes the
device's integer width a standing check instead of a review finding.

*Throwaway harnesses: Rust scratch crate in the session scratchpad; the Go
external test was deleted after measurement — `git status` in the fork is
clean. No repo files were modified by this review.*
