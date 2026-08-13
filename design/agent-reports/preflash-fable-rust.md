# Pre-flash final gate — RUST lens (fable)

Scope: `crates/me-cli/src/sysw/*` + `Command::Sysw` arm in `crates/me-cli/src/main.rs`,
branch `sysw-container`, diff `master..HEAD -- crates/` (18 commits, +2549).
Every finding below was **reproduced against the built `me` binary** before being
filed; commands and outputs are in the appendix. Nothing here re-derives what the
brief listed as already machine-verified.

Standing constraint respected: no finding below says "warn should be refuse."
Every Critical/Important is in the always-in-scope class: silently wrong result,
data loss, or a misled operator.

## Verdict

The wire format, AAD binding, and the digest/identity constructions are sound —
the questions the brief ranked highest come back clean, with the analysis below.
What is NOT clean is the **emit side**: `pack` does not enforce the invariants its
own parser enforces, so it can produce — with exit 0 and the digest line silently
omitted — containers that every conforming reader, including the device, refuses.
`--region` (today's commit) then happily turns that into a flash-ready image.
That is the one Critical. Do not flash a payload built by this binary until C1 is
folded or the operator's runbook includes a mandatory `me sysw show` read-back of
the exact artifact to be flashed (which currently catches both C1 triggers).

---

## CRITICAL

### C1 — `pack` emits containers its own parser refuses; exit 0; digest silently omitted

`pack_deterministic` never checks `MAX_SECTION_LEN` and never bounds
`iterations`; the only size gate is `bound()` against `REGION_LEN` (65536)
— `crates/me-cli/src/sysw/mod.rs:165-216`. `Header::parse` enforces both
(`crates/me-cli/src/sysw/wire.rs:124-136`), so there is a whole input range where
pack succeeds and every read fails. `print_digest` makes it silent: on parse
failure it just `return`s (`crates/me-cli/src/main.rs:934-936`), so the one line
that would look wrong is simply absent.

Two independent triggers, both reproduced:

- **(a) Section over 8191.** 90 `text:` records of 100 bytes → pack exit **0**,
  no digest line, 18,591-byte file; `me sysw show` on it:
  `me: not a systemwide container: SectionTooLong`, exit 4.
- **(b) `--iterations` outside `[100_000, 2_000_000]`.** `--iterations 5` on a
  sealed pack of a seed → full passphrase ceremony printed ("write this down and
  store it APART from the machine"), exit **0**, no digest line; `show`:
  `me: not a systemwide container: Iterations(5)`. The seed is now encrypted
  inside a container **no conforming reader will ever open**. The `--iterations`
  flag (`main.rs:166-168`) is passed through unvalidated.

Compounding: `pack --region --in bigrecs.txt` produced a byte-perfect **65,536-byte
flash image of the unreadable container**, exit 0 (appendix, test 7). The newest
feature is the one that carries the defect to the machine.

Consequence: the operator flashes a region the device reports as "no payload";
in the sealed case, a seed backup discovered to be unopenable at recovery time.
That is the data-loss shape this gate exists for. The trigger is not exotic:
(b) is a single plausible flag; (a) is any record set over ~8 KiB.

Fix shape (small): enforce `pub_len`/`ct_len ≤ MAX_SECTION_LEN` and the
iteration range inside `pack_deterministic` (new `SyswError` variants), and make
`print_digest`'s parse failure loud instead of silent — a packed blob failing its
own parse should be unrepresentable, and if it ever happens anyway it must not
whisper. Refusing to emit an artifact that is *unreadable* is not the §13 D3
warn-vs-refuse axis; an unreadable artifact is a wrong result.

---

## IMPORTANT

### I1 — `classify` admits records containing `\n`; framing silently corrupts

Two admission paths, both reproduced:

- **Interior newline in a mnemonic.** bip39 2.2.2 `parse_in_normalized` splits on
  *any* whitespace (`~/.cargo/.../bip39-2.2.2/src/lib.rs:438,450:
  s.split_whitespace()`), so `classify` (`mod.rs:94`) accepts
  `"abandon\nabandon … about"` as `Class::Mnemonic`. Packed unsealed on argv:
  the `0x0a` lands raw in the public section (verified in the hex dump), so the
  wire now frames **two** records — `"abandon"` and the 11-word remainder —
  neither of which classifies as anything. Sealed path: same corruption inside
  the plaintext section.
- **Edge newline on an md/mk record.** `seal::record::validate_record` *trims*
  before validating (`crates/me-cli/src/seal/record.rs:118`), so
  `classify("md1…tl3\n")` is `MdMk` and the untrimmed record is stored:
  pub_len 68 vs 67, and on read the section splits into the record plus a
  phantom empty record.

The digest is computed from the wire bytes on both ends, so host and device
*agree about the corrupted form* — the operator's comparison cannot catch it.
No bytes are lost, but the record set recovered is not the record set supplied,
silently. Reachable from argv (shell `$'...'`, or any script assembling records);
the `--in` path is immune because `lines()` strips newlines.

Fix shape: refuse any record containing `\n` (or `\r`) at classify/pack time —
one check, and it is a *framing* invariant, not a security refusal, so it does
not collide with the warn-don't-block ruling.

### I2 — `me sysw show` panics on a truncated container, after printing a plausible identity

`Show` parses the header but never checks `blob.len() >= h.total_len()` (unlike
`open`, which does — `mod.rs:221-223`). Reproduced: a 52-byte file that is a
valid header declaring `pub_len` 67 →

```
sealed:   false
pub_len:  67
ct_len:   0
identity: 38e44dd5a86549be1400d5a8a8332bcf…
thread 'main' panicked at crates/me-cli/src/main.rs:944:42:
range end index 119 out of range for slice of length 52   (exit 101)
```

Two defects in one: the slice panic in `print_digest` (`main.rs:943-944`), and
the `.min(blob.len())` in the identity call (`main.rs:916-918`), which *masks*
the truncation and prints an authoritative-looking identity for a partial
payload instead of reporting "container truncated: file has N of M declared
bytes". `show` is the inspection tool for exactly the suspect artifacts —
corrupt dumps and partial reads — and it is also the read-back that currently
catches C1, so it crashing on hostile input matters doubly. Fix: the same
`total_len` check `open` already performs, with a named message.

### I3 — The vector set is accept-only, and does not pin the two behaviors the flash path depends on

`testdata/sysw_vectors.json` contains seven valid containers and nothing else.
Missing, and inheritable by both implementations exactly as the brief fears:

- **No padded-region vector.** The property commit 5596b21's own message relies
  on — *identity bounds itself by the header's declared total* — exists only as
  a Rust CLI test. Every vector's `blob` is exactly `total_len` bytes, so a Go
  port that hashes **the entire buffer it is handed** passes every vector and
  then disagrees with the host on every real flashed region (the padded image is
  the only form the device ever sees). One vector whose blob carries trailing
  `0xFF` with the same recorded identity closes this permanently.
- **No reject vectors.** Truncation, `SectionTooLong`, `BadMagic`,
  `UnknownVersion`, iteration bounds — all live only in Rust unit tests
  (`coverage.rs` places test 7 as `Unit`, the rest implicitly). A Go port that
  accepts what Rust refuses (or vice versa) passes the shared contract. For a
  container about to hold seeds, refusal behavior IS normative behavior.
- Lesser unpinned edges, same class: unsealed headers with nonzero
  kdf/aead/reserved bytes are accepted by `Header::parse` (nothing checks bytes
  9–11 when `ct_len == 0`, or byte 11 ever); digests over >255 records
  (`records.len() as u8`, `pubhash.rs:26` — analysis in Q3 below shows the wrap
  is harmless on the wire path, but only if the port also uses one byte); the
  vacuous-seal blob shape of I4.

Vector format supports this today (add a `refuse: "<reason>"` variant or a
second array); the derivation gate in `coverage.rs` makes the addition cheap.

### I4 — A sealed pack with no secret records runs the full passphrase ceremony, then emits an unsealed container with 16 undeclared trailing bytes

Reproduced: `me sysw pack <md1>` (generate is the **default** mode) prints a
12-word passphrase with "write this down and store it APART from the machine",
reports `strength: 12 words — at or above the threshold`, exits 0 — and the
emitted container is `sealed: false`. The passphrase gates nothing and the
device will never ask for it. Mechanically: `seal_with` runs with an empty
secret list → `ct_len = 0` → `sealed()` is false → header says unsealed, but
`seal_bytes` still appends a 16-byte GCM tag over the empty plaintext
(`mod.rs:196-207`): file is 135 bytes vs `total_len` 119, with an orphan tag
beyond the declared total that no reader, digest, or identity ever sees.

Two misleads: (1) the operator stores a passphrase believing recovery requires
it; (2) sharper — `text:` free text is deliberately NOT secret-classed (F-123,
`record.rs:45-49`), so an operator who packs a sensitive note as `text:` with a
passphrase gets the ceremony, an above-threshold strength line, **no warning**
(`report_strength` warns only on `secret && !above`, `main.rs:972-977`), and
their note rides cleartext. Fix within the D3 ruling: when a passphrase mode is
active and the secret section is empty, say so loudly ("nothing was sealed —
every record is public-classed; the passphrase protects nothing") and don't emit
the orphan tag (skip `seal_with` when the secret list is empty).

---

## MINOR

- **M1** — `print_digest` prints nothing at all for a non-UTF-8 public section in
  `show` (`main.rs:944-946` silent return), indistinguishable from "forgot to
  look". `pub_len == 0` gets an explicit message; this path should too.
- **M2** — `identity.rs:29-30` doc overclaims: "no two regions collide" — two
  regions differing only beyond the declared total share an identity, which is
  the deliberate design (padding invariance) and should be stated as such rather
  than denied.
- **M3** — `mod.rs:104-117`: the `pack` doc comment paragraph is duplicated
  verbatim (copy-paste doubling).
- **M4** — `read_records`: `--in` with an empty/blank-lines file packs an empty
  (valid, 52-byte) container with exit 0, while empty argv is an error
  (`main.rs:984-995`). Probably fine; worth being deliberate.
- **M5** — Fixture note: S-C and S-D encrypt the same plaintext under the same
  key+IV (shared `FIXTURE_SALT`/`FIXTURE_IV`), so their ciphertext bytes are
  identical in the published JSON (lines 38/52) — harmless for fixtures with
  known plaintext, but worth one comment in `coverage.rs` so the salt/iv reuse
  pattern is never copied anywhere live. Production `pack` randomizes both per
  call (`mod.rs:123-126`) — verified.

---

## The brief's five questions, answered

**Q1 — wire format vs hostile input: SOUND (read side).** `parse` checks length
before any slice, bounds `pub_len`/`ct_len` to 8191 each *before* any arithmetic,
and gates iterations to ≤2M before any KDF. `total_len` maxes at 16,450 — no
overflow on any target width. `open` re-checks `blob.len() >= total_len` before
slicing; `pub_end ≤ total_len` always. Trailing bytes beyond `total_len` are
ignored by `open`, digest, and identity alike — coherent. The hole is the
*write* side (C1): parse's invariants are enforced nowhere in `pack`.

**Q2 — AAD binding: RIGHT.** AAD is `header ‖ public section`, taken from the
assembled/read bytes, never re-encoded (`mod.rs:198-202`, `mod.rs:237`). Every
byte of `[0, pub_end)` is authenticated — the byte-exhaustive crypto-layer test
plus the valid-for-valid entry-point swap test tie it down properly. Bytes 9–11
and iterations/salt/iv are all inside the AAD; tampering iterations costs the
attacker at most one bounded (≤2M-round) KDF before the tag fails. Nothing
security-relevant sits outside the authenticated span within the declared total;
unsealed containers are unauthenticated **by design** (digest is the integrity
signal, spec decision 6). Bytes beyond `total_len` are unauthenticated but
invisible to every reader — see Q3 for the identity bound.

**Q3 — digest and identity: DO WHAT THE OPERATOR IS TOLD, with two documented
sharp edges.** One payload → one digest: both ends compute from the wire bytes
(split, count, hash), so pack-time and device-side values agree — including,
notably, agreeing about I1's corrupted framing. Two payloads sharing a digest:
yes, by design, when they differ only outside the public section — the shipped
vectors themselves show it (S-C and S-E, same digest `e2e1636d…`, different
passphrases/ciphertexts). Safe because identity covers the rest and opening
authenticates via the AEAD; but nobody should ever present the digest as
whole-payload authentication. The `records.len() as u8` wrap (`pubhash.rs:26`)
is not exploitable through the wire: records read from the wire cannot contain
`\n`, so the joined content fully determines the count — identical content
implies identical count, wrap or no wrap. It only matters if a port sizes the
count differently (I3). **Identity's total_len bound: SAFE, not a hole** — every
reader in the system derives meaning exclusively from `[0, total_len)`, so
padding invariance is exactly the right equivalence, and a *different* declared
total is inside the hash so it cannot alias. It becomes a hole only if some
future reader consumes trailing region bytes; and it is currently **unpinned in
the contract** (I3), which is the actionable part.

**Q4 — vectors:** the seven recorded vectors are correct — I verified S-I's
digest independently (`753c5936…` matches a fresh pack of the same record) and
the golden test regenerates byte-identically. No vector *encodes* a bug. The
defects are absences: I3 (accept-only set; identity bound unpinned; no reject
vectors) and the I4 shape uncovered.

**Q5 — commit 5596b21 (`--region`): the mechanism is right.** Exactly 65,536
bytes, container at offset 0, tail 0xFF (erased-state, correct for NOR), no
truncation path — the in-arm oversize check (`main.rs:852-860`) is actually
unreachable because `bound()` refuses >65,536 first, so it is dead-but-harmless
armor. Digest printed from the container pre-padding; identity's total-bound
makes region and container agree — asserted by the new CLI test and re-verified
here via `show` on both forms. The two real problems it inherits are not in its
own arithmetic: it will package a C1 self-refusing container into a flash-ready
image (verified, test 7), and the padding-invariance property it depends on is
not in the cross-implementation contract (I3).

---

## Appendix — reproduction commands (all against `target/debug/me`, this branch)

```
# I2: truncated container -> identity printed, then panic
me sysw pack --no-passphrase <md1> --out full.bin ; head -c 52 full.bin > trunc.bin
me sysw show trunc.bin
  -> identity: 38e44dd5… ; panic at main.rs:944 "range end index 119 out of
     range for slice of length 52"; exit 101

# C1a: >8191-byte public section
90 x "text:" records of 100 bytes -> me sysw pack --no-passphrase --in bigrecs.txt --out big.bin
  -> exit 0, NO digest line, 18591 bytes
me sysw show big.bin -> "me: not a systemwide container: SectionTooLong", exit 4

# C1b: iterations below MIN on a sealed pack
me sysw pack --passphrase-words 12 --iterations 5 "<seed>" --out iter5.bin
  -> exit 0, passphrase ceremony printed, NO digest line
me sysw show iter5.bin -> "me: not a systemwide container: Iterations(5)", exit 4

# C1 + --region: flash image of the unreadable container
me sysw pack --no-passphrase --in bigrecs.txt --region --out bigregion.bin
  -> exit 0, 65536 bytes

# I1a: interior newline in a mnemonic (argv)
me sysw pack --no-passphrase $'abandon\nabandon …about' --out nl.bin
  -> exit 0, digest e892554c… (≠ f36e9900… of the space form); 0x0a raw in the
     public section at offset 59; frames as TWO records on read

# I1b: trailing newline on an md1
me sysw pack --no-passphrase "<md1>"$'\n' -> pub_len 68 (vs 67), digest fd334b65…

# I4: vacuous seal
me sysw pack --passphrase-words 12 "<md1>" --out vac.bin
  -> exit 0, prints passphrase + "strength: … at or above the threshold"
  -> vac.bin: 135 bytes, show says sealed: false, ct_len 0; bytes 119..135 are
     an orphan GCM tag beyond the declared total
```

Gate recommendation: **hold the flash** until C1 (and ideally I2, since `show`
is the read-back that catches C1) is folded; I1/I3/I4 should land before the Go
port freezes against the vectors, because I3 is cheapest to fix while the
contract is still one commit wide.
