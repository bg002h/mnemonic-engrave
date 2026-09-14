# Journey walk — SPEC_hashlock_kinds.md, one operator, four kinds

**Lens:** the operator journey, not correctness. Three correctness rounds already
closed GREEN; this walk looks for **missing things at moments** rather than wrong
things in sections.

**Walked against:** `design/SPEC_hashlock_kinds.md` at `2a0da99f` (unimplemented),
plus the CURRENT code for every step the spec does not change. Every step below
says which one it is reading.

**Tips walked:** mnemonic-engrave `7485920a` (`me` 0.9.0), descriptor-mnemonic
`40c400de` (`md` 0.14.0), mnemonic-secret `7a0e96f` (`ms` 0.18.0), seedhammer
`0562e81`. The CLI half of the walk was **executed**, not read; the device half
was read from source because it needs hardware.

**Not covered here, by instruction:** the artifact-loss matrix (a separate agent).

---

## The operator, and what they hold

- BIP-39 seed (test fixture xpubs from `md-cli/tests`).
- One key and its path: `[5b666435/48'/0'/0'/2']xpub6DkFA…r6KFrf`.
- A hashlock phrase **on paper**: `correct horse battery staple`.

They want a **concrete** policy — keys seated — that incorporates the phrase, and
engraved steel carrying both the phrase (in whatever form the device cuts) and
the descriptor. Both ways: at the CLI and on the SH2.

---

## The four kinds, from one phrase — computed independently (python3 `hashlib`)

The whole walk uses these. Preimage
`X = PBKDF2-HMAC-SHA256("correct horse battery staple", "ms-hashlock-v1", 100000, 32)`
` = c3e97525442520da4cffd5f57aae3f6273990017f2e0fa30c056e32172e22016`
(produced by `ms hashlock`, so X is measured, not asserted):

| kind | digest | width |
| --- | --- | --- |
| `sha256` | `3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12` | 64 hex |
| `hash256` | `98a20fc25dbcdf236fb0307e3f82cad47fca2e807f3ef82c31993549641cd488` | 64 hex |
| `ripemd160` | `09e7bb5051d89788fb4e4b374126721dbcc2946b` | 40 hex |
| `hash160` | `b5b72c0e6896ff59dfa99e0d1052a9c0214cd0bd` | 40 hex |

The `sha256` row equals what `ms hashlock` printed, so the chain is anchored.
**Keep the first two rows in view for the rest of this document**: they are the
same width, the same character class, and the same shape on every screen in the
system. Nothing but a label distinguishes them.

---

# Part 1 — the walk at the CLI

## Step 1. Phrase → preimage → digest

**Reading: shipped code** (`mnemonic-secret/crates/ms-cli/src/cmd/hashlock.rs`).
Executed:

    $ printf 'correct horse battery staple' | ms hashlock --hashlock-phrase-stdin --emit-record
    hash:3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12     [stdout]
    THIS CARD CARRIES THE PREIMAGE -- the secret. …                           [stderr]
    digest:          3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12
    for md compose:  --path ... sha256=3cf5d421…b70a4c12
    preimage (ms1):  ms10h ashsq 0p7ja f9gsj jpkjv ll2l2 74w8a 388xg qzlew …
    preimage (hex):  c3e97525442520da4cffd5f57aae3f6273990017f2e0fa30c056e32172e22016
    method:          preimage = PBKDF2-HMAC-SHA256(password = phrase, salt = "ms-hashlock-v1", …)
    record (phrase): phrase:68617264656e65642c636f7272656374…
    The preimage must be exactly 32 bytes (64 hex characters): the script checks
        OP_SIZE 32 before OP_SHA256 (composer spec §8i, F-132).

**Q1 — what they hold, exactly.** Three distinct values now exist and the card
labels all three: `phrase` (on paper), `preimage (hex)`/`preimage (ms1)`, and
`digest`. **This step is the one the brief worried about, and the shipped card
already answers it well.** Each hop is named, the secret/public boundary is stated
in the first line, and the polarity (secret to stderr, public to stdout) is
explicit. No finding.

**Q2 — what the tool does under the spec.** §9 phase 2, in full: *"`ms hashlock`
learns the kind."* Six words. Nothing else in the spec says what the card, the
record line or the JSON become.

**Q3 — what else might they do.** They ask for `--kind ripemd160`. Then:

| card line | today | needed | on any gate list? |
| --- | --- | --- | --- |
| stdout record (`hashlock.rs:349`) | `hash:<64hex>` | `hash:ripemd160:<40hex>` (§6) | no |
| `for md compose:` (`:406`) | `--path ... sha256=<h>` | `ripemd160=<h>` | **no** |
| §8i line (`:422`) | `…before OP_SHA256` | `OP_RIPEMD160` | no |
| `--json` `hash_record` (`:366`) | `hash:<64hex>` | kind-tagged | no |
| `--json` `sha256_operand` (`:368-369`) | `sha256=<h>` | kind-named key | no |

§11's string row is **`me-cli`-only**: *"five sha256-hardcoded operator-facing
strings in `me-cli` | `main.rs:2275,2685,3196`, `sysw/composer_records.rs:144`"*.
`ms-cli` appears nowhere on the gate table. → **I-1**.

## Step 2. Compose the policy

**Reading: shipped code** (`descriptor-mnemonic/crates/md-cli/src/cmd/compose.rs`).
Executed:

    $ md compose --wrapper wsh --preset hashlock-gated,sha256=3cf5d421…,older=26280
    wsh(or_i(and_v(v:pkh(@0/48'/0'/0'/2'/<0;1>/*),sha256(3cf5d421…)),
             and_v(v:pkh(@1/48'/0'/1'/2'/<0;1>/*),older(26280))))
    note: stdout is a keyless descriptor template (no keys)

**Q3 — what else might they do.** Under the spec they type `ripemd160=`,
`hash160=` or `hash256=` (§9 phase 1, both `--path` and `--preset`). Today
`named_only(&["sha256","older"])` (`compose.rs:369`) refuses all three — a closed
allow-list, so the pre-phase-1 refusal is clean.

**The divergence that matters is between `hash160=` and `ripemd160=`.** Both take
40 hex. Both parse. Both lower. Both produce a valid address. The operator who
means one and types the other is told nothing, ever, until a spend fails. This is
§3-F4 exactly, and **no tool can catch it** — refusing would be worse than
accepting. *Classification: not our concern, no spec change earned.*

Except that it makes one thing load-bearing: the `for md compose:` line from
Step 1 is **the only mechanism in the system that keeps the digest function and
the `md compose` option token in agreement**, because the operator copies the
whole operand instead of typing the kind twice. Leaving it hardcoded to `sha256=`
does not merely print a stale word — it hands the operator a correctly-computed
`ripemd160` digest inside a `sha256=` operand. → tightens **I-1**.

## Step 3. Seat keys and encode the card

**Reading: shipped code.** Executed:

    $ md encode "$T" --key @0=xpub6DkFA… --key @1=xpub6Dzhy…
    chunk-set-id: 0xeaae2
    md1fa2hzzq9q2tvyyy5jmpprj5qqcyefnfva8n6aggw2725u36uau82qppnw5l28te46j7yqjqxzw05nk6fpl
    …5 chunks…
    note: stdout is watch-only — public keys only, cannot spend

**"Concrete with keys seated" is reachable at the CLI.** Five md1 chunks.

**Measured, and worth stating because it reframes the cycle:** `md encode` accepts
a `ripemd160(...)` template **today**, with no spec change at all —

    $ md encode "wsh(or_i(and_v(v:pkh(@0/48'/0'/0'/2'/<0;1>/*),
        ripemd160(2d711642b726b04401627ca9fbac32f5c8530fb1)), …))" --key … --key …
    chunk-set-id: 0xe0a98   [5 chunks]
    $ md decode <those five>
    wsh(or_i(and_v(v:pkh(@0/<0;1>/*),ripemd160(2d711642…)),and_v(v:pkh(@1/<0;1>/*),older(26280))))
    $ md address <those five>
    bc1q3dkvs9y7hfngkd04zcj9ju726qg0xfe4pxhpexdujnnp0wdpaycq74zs63

So §1's *"can be encoded, engraved, restored and derived today"* is **verified**,
and the operator's escape hatch before this cycle lands is `md encode` with a
hand-computed digest. Which is the reason the cycle exists: hand-computing
`ripemd160` vs `hash160` is precisely the confusable §3-F4 names. `md decode`
prints the fragment name, so **the engraved card is self-describing about its
kind** — that fact carries weight in C-1 below.

## Step 4. Pack the payload for the device

**Reading: shipped code** (`me-cli/src/sysw/composer_records.rs`, `main.rs`). Executed:

    $ me sysw pack --in recs.txt --no-passphrase --pack-preimage --out payload.bin
    me: WARNING — this payload carries a hashlock PREIMAGE. …bearer material…
    sealing:  NOT SEALED — …record 2 (hashlock phrase (phrase:))…
    me: appended now:1789422590 as the last record…
    $ me sysw show payload.bin
    public record 1: sha256 hashlock (hash:) — 3cf5d421..b70a4c12
    secret record 3: hashlock phrase (phrase:) — not shown

**Q3 — the version-skew divergence, measured.** §6 claims an old host refuses the
whole pack. Executed against the shipped `me` 0.9.0:

    $ echo 'hash:ripemd160:2d711642b726b04401627ca9fbac32f5c8530fb1' | me sysw pack --in - …
    me: record 0 … is a `key:`/`hash:`/`now:`/`phrase:` record whose body fails its rule
        (not exactly 64 lowercase hex characters).

and identically for the accepted-on-input `hash:sha256:<64hex>` spelling.
**§6's fail-closed claim is TRUE, verified by execution.** No finding.

**Q3 — what else might they read.** They type `me sysw pack --help` to learn the
grammar, because §13 of the H6 spec says *"this help is the producer"*. It reads:

    `key:<hex of "[fingerprint/path]xpub">`, `hash:<64 lowercase hex>` and `now:…`
    feed the SeedHammer II's Wallet Policy composer: a cosigner key for seating,
    **a sha256 hashlock digest**, and the pack time…

`crates/me-cli/src/main.rs:195,197`. Two sha256/64-hex claims, in the one text the
tree designates as the producer of the record grammar — and §11 cites
`main.rs:2275,2685,3196` and `composer_records.rs:144`, not `:195`. → **I-1(b)**.

## Step 5. The engrave decision at the CLI

**Reading: shipped code** (`me-cli/src/manifest.rs`). Executed, on the sha256 card
and again on the ripemd160 card — **byte-identical output**:

    $ me bundle --in card.txt
    me: backup needs 6 plates (5 public + ms1 on device):
      plate 1/6  md1 policy  → push via NFC & engrave
      …
      plate 6/6  ms1 secret  → TYPE ON DEVICE (New > Input Seed > CODEX32); never via this tool

**This is the moment the walk was for.** The operator asked for steel carrying the
policy **and** the hashlock phrase. `me bundle` is the tool that tells them what to
cut. It enumerates six plates, none of which is a preimage or phrase plate, and
says nothing about a hashlock existing. `manifest.rs` contains no hash or preimage
logic at all — confirmed by content search, and by the two runs being identical
across kinds.

Meanwhile the **device**, on the same policy, stamps `PREIMAGE REQUIRED` on every
md1 and mk1 plate (`gui/composer_preimage_plate.go:479-495`, gated by
`composerAnyPathHashed` — kind-independent, so it survives this cycle unchanged)
and raises `HASH ON EVERY PATH … Back up every preimage separately`
(`gui/composer_copy.go:184-188`).

And the CLI **cannot** cut the phrase plate at all: `me` refuses `ms1`
(`me --help`: *"Refuses secret `ms1`"*), and the ms1 preimage string's only
engraving route is the device's §5.3 plate picker. So the CLI half of the
operator's stated request is **half-satisfiable, and the tool that enumerates the
work does not say which half**. → **I-2**.

---

# Part 2 — the same journey on the SeedHammer II

## Step 6. `Which hash?` — choosing the source

**Reading: shipped code** (`gui/composer_hash.go:311-356`) **+ spec §7.1.**

Six bands: payload `hash:` records, payload preimage-plate records, payload
`phrase:` records, `Type a hashlock phrase`, `Type 64 hex`, `No hash lock`.
`TestWhichHashPageHoldsFiveRows` (`gui/composer_hashlock_test.go:1433`) records
that a page holds **five**, so with three payload record bands populated,
`No hash lock` is already on page 2 today.

Spec §7.1 keeps the kind **off** this screen — its own pick screen, reached from
the typed-hex and phrase arms only. Four of six routes never ask. That reasoning
is sound and I have nothing to add to it.

**Q3 — the route-6 divergence.** §7.1's route-6 row reads *"the preset archetype
(`--preset` / a composer preset) — the preset supplies the hash; the kind comes
from the preset's own grammar (§9 phase 1), not from a screen."* That is true of
`md compose --preset`. It is **not** true of the device's composer preset, which
has no grammar and no parameters: `composerPresetDigest()`
(`gui/composer_presets.go:50-60`) returns 32 bytes of `0xa8`, wired into
`hashlock-gated` at `:101`. One row asserts one mechanism for two different
things, and on the device there is no way to author a non-sha256 preset —
the operator picks the archetype and then edits the path's hash. Reachable,
so the harm is small. → **M-1**, documentation.

## Step 7. The §8i rule modal — fires *before* any arm

**Reading: shipped code + spec §7.2.**

`composerHashEdit` fires the rule modal on **row selection**, for every row except
`No hash lock` (`gui/composer_hash.go:~400-410`, predicate `taking`). The body is
`composerCopyHashRule()` (`gui/composer_copy.go:190-193`):

> The hash must be **SHA-256** of a 32-byte value. A passphrase must be hashed to
> 32 bytes first, then hashed again. A hash of the passphrase itself can never be
> spent.

Spec §7.2: *"**Entry body:** unchanged, kind-generic, still fired by `taking`.
**Consent body:** names the kind."*

**The entry body is not kind-generic.** It names SHA-256 in its first six words.
"Unchanged" and "kind-generic" cannot both be satisfied by the same string, and
the sentence the operator reads immediately before selecting `ripemd160` would
say their hash must be SHA-256. This is one normative bullet contradicting itself
at a funds-gating screen. (It is also **one function used at both sites** —
`composer_hash.go` via `taking`, and `composer_consent.go:199` — so "splits in
two" is two functions, which §11's copy-table gate at `composer_copy_test.go:371`
does anticipate.) → **I-3**.

## Step 8. The kind screen — and the Back leg nobody wrote down

**Reading: spec §7.1/§7.5 + `SPEC_hashlock_H2_device.md` §4.6 (shipped, normative).**

§7.1 says *"Kind before the pad"* — stated for the **hex** arm, where a pad exists.
For the **phrase** arm there is no pad, and the spec never says where the screen
sits.

That matters, because the phrase arm's navigation is normative elsewhere.
`SPEC_hashlock_H2_device.md` §4.6, *"The Back contract … Normative, stated once"*,
enumerates five legs and closes:

> **`composerHashEdit` returns `false` ONLY for Back at `Which hash?` itself.**
> Today, at path creation, `false` REMOVES the path (`gui/composer_shape.go:269`)
> — the `Type 64 hex` route does that, and a phrase route that copied its sibling
> would delete a path, and the EXPERIMENTAL key-less consent already given,
> because the operator read a digest they did not expect and pressed Back.

SPEC_hashlock_kinds.md inserts a **sixth screen into that loop** and says nothing
about its Back. It is aware of the section — §6 cites
`SPEC_hashlock_H2_device.md` §4.6 by name, specifically to disclaim referencing it
— and still does not address it for its own screen.

**Q3, concretely.** The operator on the phrase arm types 28 characters, picks
`hardened`, waits ten seconds, reads the confirm modal, realises they picked
`sha256` when they meant `hash256`, and presses Back. Under §4.6 they land on the
method pick with the phrase intact. Under this spec, is the kind screen between
them and that? Inside the `pick:` loop, outside it, or before the phrase screen —
and if it is before the phrase screen, can they reach it again at all without
dropping the phrase and re-deriving? **I could not determine what the operator
sees here from the spec, and that is the finding.** This tree's own record says a
navigation complaint is a state audit (W-6 → W-7 was exactly this shape). → **I-4**.

*(Related, smaller: §7.5 measures "the kind-bearing row" with
`TestWhichHashRowsDrawOnOneLine` as arbiter — a test scoped to `Which hash?`'s
rows. The kind **pick screen's own four rows** are new rows on a new screen and
have no named arbiter. `ripemd160` is 10 characters against the 6-character
tokens §7.5 measured at 409 px. Four rows fit a five-row page either way, so
the harm test fails and this earns no change; recorded as **N-1**.)*

## Step 9. The pad — and the counter the operator actually reads

**Reading: shipped code + spec §7.1/§11.**

§7.1: *"The pad then accepts `digest_len() * 2`, preserving the shipped property
that an entry N characters long is N valid characters by construction."*

§11 gates *"`composerHexEntry`'s three 64/32 constants | `gui/composer_hash.go:79-105`"*.

**`composerHexEntry` spans lines 79–147**, measured. Inside it:

| line | site | in §11's cited 79-105? |
| --- | --- | --- |
| 87, 88 | `len(kbd.Fragment) > 64` / `[:64]` cap | yes |
| 91 | `valid := len(frag) == 64` | yes |
| 98 | `len(raw) != 32` (the documented-unreachable one) | yes |
| 103 | `"That is not a 32-byte digest."` — a **sentence**, not a constant | yes, but uncounted |
| **135** | **`fmt.Sprintf("%d of 64 hex", len(frag))`** | **no — 30 lines past the range** |

Line 135 is the live counter under the entry field, redrawn on every keystroke.
It is the single string the operator looks at while typing a digest. Under a
20-byte kind the pad accepts 40 characters and the checkmark appears at 40 — while
the counter reads **"40 of 64 hex"**. A screen that shows a checkmark and a
progress counter disagreeing about whether the entry is finished, on the screen
that gates funds. → **I-5**.

## Step 10. Derivation, the confirm modal, and the write-down instruction

**Reading: shipped code** (`gui/composer_hashlock.go:54-105`,
`gui/composer_copy.go:557-571`).

The confirm modal — the screen that assigns the hash — draws:

    hash  3cf5d421..b70a4c12
    method: hardened   chars: 28
    no hash: record in the payload has this digest
    Write down this phrase, the method and this digest now. This composition
    holds them until it ends. Without both, this path can never be spent.
    One phrase per policy. Never use this phrase as a passphrase or a password
    anywhere else.

Then, on its own screen (`composerCopyHashlockReconcile`, `composer_copy.go:641-648`):

    hash  3cf5d421..b70a4c12
    method: hardened   chars: 28
    Before you cut plates, run ms hashlock with this phrase and method on the
    host and check the digest matches. If they differ, do not fund this wallet:
    build it again.

**Neither screen names the hash kind.** Under the spec they still will not: §7.2
enumerates exactly two bodies that change (§8i entry and consent), §11 lists no
`me`/fork copy string beyond those, and §10's vectors are about widths and
functions.

**Q3 — the operator does exactly what the screen says.** They composed under
`hash256`. They run `ms hashlock --hashlock-phrase-stdin --method hardened` on the
host. It prints `3cf5d421…b70a4c12`. The device showed `98a20fc2..641cd488`. Both
are 64 hex. They differ. The screen's printed remedy is *"do not fund this wallet:
build it again"* — for a **correct** wallet, after five plates at ~21 minutes each.

And the operator cannot fix it by being cleverer, because the screen gave them
nothing to pass as `--kind`. The one token on the screen that reads `sha256` —
`method:` — is the **preimage derivation**, a different axis. §6's own "Naming"
paragraph identifies that token collision and scopes it to the record grammar; the
screen that prints it to the operator is never mentioned.

**The write-down instruction inherits the same hole.** *"Write down this phrase,
the method and this digest now … Without both, this path can never be spent."*
An operator who does exactly that, and later loses the md1 plates, holds phrase +
method + digest and **cannot rebuild the descriptor**, because `md compose` needs
to know which of four option names that digest belongs to. Four candidates, one
right, no way to tell from the written record.

**Harm test, applied honestly.** Is the wrong outcome worse than telling the
operator nothing? Yes, twice over. A reconciliation screen that is *silent* leaves
the operator uncertain; this one **actively instructs a check that cannot pass on
a correct artifact** and names a destructive remedy for passing it. And a
write-down list that omits a field is worse than no list, because the operator
stops writing when the list ends. → **C-1**.

*(Mitigation, stated for fairness: the engraved md1 is self-describing — `md decode`
prints `ripemd160(…)`, measured in Step 3 — so the kind is recoverable **while the
policy plates survive**. That is what keeps C-1 a cannot-verify / cannot-rebuild
defect rather than an immediate funds loss. It does not help at the reconciliation
screen, which by construction runs **before** the plates are cut.)*

## Step 11. Done, the plate picker, the locator

**Reading: shipped code** (`gui/composer_hashlock_plates.go`,
`gui/composer_preimage_plate.go:232-268`).

The device cuts the preimage plate in `string` or `phrase` form. Its engraved
locator is:

    path 1
    hash  3cf5d421..b70a4c12
    mk1 stub (policy): <stub>

**No kind.** Under a 20-byte kind, `hashlockFirst8Last8` is on §11's gate list, so
the abbreviation will be right; the kind will still be absent.

**Harm test.** Fails. F1 makes the preimage kind-independent, so the plate is not
*wrong* — it yields the correct 32 bytes for all four kinds — and nothing spends
from a preimage plate alone. What is lost is only the ability to check a
plate-found-alone against its own digest. Not worse than saying nothing. → **M-2**,
documentation only.

## Step 12. The version-skew moment on the device

**Reading: shipped code + spec §6.**

Old firmware, new payload. `hash:ripemd160:…` classifies `ClassUnknown` → inert.
`Which hash?`'s band 1 is empty, so its lead becomes
(`gui/composer_copy.go:474-477`):

> **No hash record in the payload.** Type a phrase below, or make one with
> `ms hashlock` on the host.

A positive false statement — there *is* one — while the truth
(`"1 payload record was not understood."`, `composer_copy.go:293`) sits on the
door, screens earlier.

**Harm test.** The door count exists, and the outcome is a refusal, not a wrong
wallet. Fail-closed as §6 designed, and §6 explicitly prefers this to any scheme
where an old parser reads a non-sha256 digest *as* sha256. Not worse than nothing.
→ **M-3**.

---

# Part 3 — do the two surfaces agree?

Asked directly, because the brief did.

| question | answer |
| --- | --- |
| Can both surfaces produce a **concrete, keys-seated** hashlock policy? | **Yes.** CLI: `md compose` → `md encode --key` (executed). Device: composer + seating. |
| Do they produce the **same descriptor** for the same intent? | **Yes**, by the pinned `keyed_compose_preset_hashlock_gated` vector; the device's `hashlock-gated` differs only by carrying a placeholder digest the operator replaces. |
| Can both produce **all four kinds**? | Under the spec, yes — CLI via `--path`/`--preset` options, device via the typed-hex and typed-phrase arms. **Asymmetry:** the device's *preset* can only start at sha256 (M-1). |
| Can both engrave **the hashlock phrase**? | **No.** Only the device. `me` refuses ms1 by design; the CLI has no preimage-plate producer. And `me bundle` never says so (I-2). |
| If the operator does it **both ways**, do they get the same wallet? | Yes for the policy. Not for the artifact set: the CLI run yields 6 plates and no hashlock material; the device run yields the same 6 plus a preimage plate, all stamped `PREIMAGE REQUIRED`. |

---

# Findings

Severity in the operator's terms. Each carries its divergence classification and
the harm test, applied explicitly.

### C-1 — Critical. The kind is absent from the two screens whose whole job is "write this down" and "check this against the host", and the token they do print is a different axis wearing the same word.

**Where:** `gui/composer_copy.go:557-571` (confirm modal, prints `method: %s   chars: %d`)
and `:641-648` (reconciliation, *"run ms hashlock with this phrase and method on
the host and check the digest matches. If they differ, do not fund this wallet:
build it again."*). Unchanged by the spec: §7.2 changes only the §8i entry and
consent bodies; §11 lists no other fork copy string.

**The sequence.** Compose under `hash256`. Follow the reconciliation screen's
instruction exactly. `ms hashlock` (no kind given, or given the `method:` token
the screen showed) returns the `sha256` digest. It differs. The printed remedy is
to discard a correct wallet and re-cut five plates. There is no third option,
because the screen supplied no kind to pass.

**Why `hash256` and not `ripemd160` is the dangerous one:** identical width,
identical character class, identical abbreviation shape. `3cf5d421..b70a4c12`
against `98a20fc2..641cd488` — nothing but a label separates them, and the label
was never printed. §3-F4 says this exact pair *"type-checks exactly, passes
`digest_len()*2`, lowers cleanly, and Core agrees with the address"*; this finding
is that the same blindness reaches the operator's own eyes, where no KAT can help.

**Second limb.** *"Write down this phrase, the method and this digest now …
Without both, this path can never be spent."* An operator who complies exactly
cannot later rebuild the descriptor from that record: `md compose` needs one of
four option names and the written record names none.

**Classification: warning → it must name the kind.** (Not a refusal: nothing here
should refuse.)

**Harm test.** A silent screen leaves uncertainty; this screen *instructs a check
that cannot pass on a correct artifact* and names a destructive remedy. A
write-down list that omits a field is worse than no list, because the operator
stops writing where the list stops. **Worse than nothing — change earned.**

**Minimum fix:** add the kind to `composerCopyHashlockConfirm` and
`composerCopyHashlockReconcile`, and make the reconcile sentence name the flag
(`ms hashlock … --kind <kind>`). Both bodies are on `modal_fits_test.go`'s
measured surface, so §11 needs the rows.

---

### I-1 — Important. `ms hashlock`'s entire operator surface is sha256-hardcoded and appears on no gate list — including the one line that keeps the digest function and the `md compose` option in agreement.

**Where:** `mnemonic-secret/crates/ms-cli/src/cmd/hashlock.rs:349` (stdout record),
**`:406`** (`for md compose:  --path ... sha256=<h>`), `:422` (`OP_SHA256`),
`:366` (`hash_record`), `:368-369` (`sha256_operand` JSON key). §9 phase 2 gives
six words; §11's string row is scoped to `me-cli`.

**Why `:406` is the sharp one.** Step 2 established that no tool can catch
`ripemd160=` typed where `hash160=` was meant. The one thing that prevents it in
practice is the operator copying a complete, tool-generated operand. Left
unchanged, that line *generates* the mismatch: a correct `ripemd160` digest inside
a `sha256=` operand, ready to paste. `sha256_operand` does the same for scripts.

**(b)** `me-cli/src/main.rs:195,197` — the `me sysw pack --help` text, which the
H6 spec §13 designates *"the producer"* of the record grammar — says
`hash:<64 lowercase hex>` and *"a sha256 hashlock digest"*. §11 cites
`main.rs:2275,2685,3196` and `composer_records.rs:144`; not `:195`.

**Classification: default** (the tool emits an operand; the wrong default is
pasted straight into a policy).

**Harm test.** A card that printed *nothing* for `md compose` would send the
operator to the docs. A card that prints a wrong-but-plausible operand sends them
to a wallet. **Worse than nothing — change earned.**

---

### I-2 — Important. `me bundle` — the CLI's "what must I cut" checklist — has no hashlock awareness at all, and the CLI cannot cut the hashlock plate the operator asked for.

**Measured:** `me bundle --in card.txt` produced byte-identical output for the
sha256 card and the ripemd160 card: *"backup needs 6 plates (5 public + ms1 on
device)"*, the sixth being the **seed** ms1. `crates/me-cli/src/manifest.rs`
contains no hash or preimage logic.

**The device, on the same policy,** stamps `PREIMAGE REQUIRED` on every md1 and
mk1 plate and raises `HASH ON EVERY PATH … Back up every preimage separately`.
The CLI raises neither, and cannot produce a preimage plate at all
(`me` refuses ms1 by design).

**Pre-existing**, and out of §4's stated scope. Named here anyway because the
journey walks straight through it and this cycle multiplies hashlock policies by
four. On a policy where every path is hashed, an operator who cuts the six plates
`me bundle` enumerated and stops has **no spendable backup** — and nothing told
them.

**Classification: warning** (one line naming the hashlock and pointing at the
device's plate flow).

**Harm test.** "Backup needs 6 plates" is a completeness claim. Making it while the
hashlock material is on none of them is **worse than telling the operator
nothing — change earned**, though it may reasonably be filed as a follow-up
against this cycle rather than folded into the spec.

---

### I-3 — Important. §7.2's *"Entry body: unchanged, kind-generic"* is self-contradictory, and executed literally it leaves a false sentence at the moment the operator takes a non-sha256 lock.

**Where:** `gui/composer_copy.go:190-193`, `composerCopyHashRule()` —
*"The hash must be **SHA-256** of a 32-byte value."* Fired by `taking` at entry
(`gui/composer_hash.go`) and again at consent (`gui/composer_consent.go:199`) —
**one function, two sites**.

The body is not kind-generic; "unchanged" and "kind-generic" cannot both hold.
An implementer reading §7.2 as written leaves the operator told their ripemd160
hash must be SHA-256, on a modal that fires immediately before the row that takes
it.

**Classification: documentation → the spec sentence must resolve**, plus the
resulting string change.

**Harm test.** A wrong statement of the hash function at the §8i gate is worse
than a generic one. **Change earned** (it is a one-word spec fix: say what the
generic body becomes).

---

### I-4 — Important. The kind screen is inserted into a loop whose Back contract is normative and stated once, and the spec never says where it sits or what Back does there.

**Where:** spec §7.1 (*"Kind before the pad"* — stated for the hex arm only; the
phrase arm has no pad) against `SPEC_hashlock_H2_device.md` §4.6, which enumerates
five Back legs and warns that `composerHashEdit` returning `false` **removes the
path** at creation (`gui/composer_shape.go:269`) *"and the EXPERIMENTAL key-less
consent already given"*.

The spec cites §4.6 by name in §6 (to disclaim a different reference) and never
addresses it for its own screen.

**The moment.** The operator picks a kind, types 28 characters, waits ten seconds,
reads the confirm modal, sees a kind they did not mean, presses Back. Where do
they land, and is the phrase still there? **I could not determine this from the
spec.** If the new screen copies `Type 64 hex`'s sibling shape it returns `false`
and deletes the path — the precise defect §4.6 exists to prevent.

**Classification: refusal/navigation contract — the spec must state the leg.**

**Harm test.** A deleted path plus a re-typed phrase plus a second ten-second KDF,
triggered by the operator correcting their own mistake, is worse than any
alternative. **Change earned** — and it is one sentence, not a redesign.

---

### I-5 — Important. The hex pad's live counter is outside §11's cited range, so under a 20-byte kind the checkmark and the counter disagree.

**Measured:** `composerHexEntry` spans `gui/composer_hash.go:79-147`. §11 gates
`:79-105` and counts "three 64/32 constants". Line **135** —
`fmt.Sprintf("%d of 64 hex", len(frag))` — is thirty lines past the cited range
and is the string redrawn on every keystroke. Line **103**
(`"That is not a 32-byte digest."`) is inside the range but is a sentence, not one
of the three counted constants, and is wrong for a 20-byte kind.

Under `ripemd160` the pad accepts 40, the checkmark appears at 40, and the counter
says **"40 of 64 hex"**.

**Classification: warning/default — a gate row, not a design change.**

**Harm test.** A counter contradicting the affordance beside it on the screen that
gates funds is worse than no counter. **Change earned** (extend §11's row to the
function, 79-147, and state the count as measured rather than as three).

---

### M-1 — Minor. §7.1 route 6 conflates `md compose --preset` with the device's composer preset, which has no grammar.

`composerPresetDigest()` (`gui/composer_presets.go:50-60`) is 32 bytes of `0xa8`
with no parameters; there is no device path from a preset to a non-sha256 kind.
Reachable via the path edit. *Classification: documentation.* **Harm test fails —
no change earned** beyond correcting the row's wording.

### M-2 — Minor. The engraved preimage-plate locator carries no kind.

`gui/composer_preimage_plate.go:232-248` prints `hash <first8>..<last8>` and the
mk1 stub. *Classification: documentation only.* **Harm test fails** — F1 makes the
preimage kind-independent, so the plate is not wrong, and nothing spends from it
alone. **No change earned.**

### M-3 — Minor. An old device meeting a new kind token draws a positive false statement.

`"No hash record in the payload."` (`gui/composer_copy.go:474-477`) while the door
says `"1 payload record was not understood."` (`:293`), screens apart.
*Classification: warning, already present.* **Harm test fails** — the outcome is a
refusal, not a wrong wallet, and §6 chose this deliberately. **No change earned.**

### M-4 — Minor. `--method` and `--kind` will share the token `sha256` on one `ms hashlock` command line.

§6's "Naming" paragraph names the collision for the record grammar and stops
there. Separately: `refuse_method` (`ms-cli/src/cmd/hashlock.rs:281-288`) refuses
`--method` for non-phrase sources; an implementer copying that shape for `--kind`
would make **§12 acceptance item 5 unsatisfiable**, since reproducing a plate's
digest is exactly the `--hex`/ms1 path. Worth one line in the plan.

### N-1 — Nit. The kind pick screen's own four rows have no named width arbiter.

§7.5 measures "the kind-bearing row" with `TestWhichHashRowsDrawOnOneLine`, scoped
to `Which hash?`. `ripemd160` is 10 characters against the 6-character tokens
measured at 409 px. Four rows fit a five-row page regardless. **No change earned.**

### N-2 — Nit. `"Type 64 hex"` is at `gui/composer_hash.go:349`; §11 cites `:348`.

---

# What I checked and found sound

Recorded so the next reader does not re-derive it.

- **§6's fail-closed claim on the host is TRUE** — executed: `me` 0.9.0 refuses
  both `hash:ripemd160:<40hex>` and `hash:sha256:<64hex>` at pack time, whole pack.
- **§1's "encoded, engraved, restored and derived today" is TRUE** — executed a
  full `ripemd160` round trip through `md encode` → `md decode` → `md address`,
  five chunks, csid `0xe0a98`, address `bc1q3dkvs9…74zs63`.
- **§10's "six hardcoded `[56:]` sites" is exactly right** — counted: four in the
  fork's production Go (`composer_hash.go:50,173,188`, `composer_consent.go:63`)
  plus two in `me-cli` (`main.rs:2277,2602`). `hashlockFirst8Last8` is correctly
  called out as the seventh that the grep misses.
- **§7.3's three layers all verified** — `gui/composer_state_hook.go:81`,
  `cmd/emu/composer_js.go:15,35-37,54` (its doc contract really does say
  "FULL 64 HEX"), `cmd/emu/walk_hashlock_phrase.js`.
- **`PREIMAGE REQUIRED` survives this cycle unchanged** —
  `composerPreimageMarkTitle` gates on `composerAnyPathHashed`, a nil check on
  `p.Hash`, kind-independent.
- **The `ms hashlock` card's phrase→preimage→digest labelling is already good.**
  The brief expected a gap at that hop; there is not one. All three values are
  named, and the secret/public polarity is the card's first line.
- **A payload carrying a `phrase:` record and a `hash:ripemd160:` record degrades
  acceptably.** Picking the phrase row yields sha256; `hashlockRelationLine` says
  *"no hash: record in the payload has this digest"* and `payloadStatesDigest`
  fires the reconciliation screen. *Classification: warning, already adequate —
  no change earned.*

---

# Counts

**1 Critical, 5 Important, 4 Minor, 2 Nit.**

Of those, **C-1, I-3, I-4** are spec defects (a missing field, a contradictory
sentence, an unstated navigation leg). **I-1, I-5** are gate-coverage defects —
real operator-facing strings that no §11 row reaches. **I-2** is a pre-existing
CLI gap the journey walks straight through, better filed as a follow-up than
folded.

# Verdict

The spec's *machinery* is in good shape — widths, tags, dispatch, the emulator
window, fail-closed behaviour all survived the walk, and several claims verified
under execution — but it never tells the operator which of four hash functions
they chose, at any of the three moments where that is the only thing they need to
know: the write-down modal, the reconciliation screen it instructs, and the
`for md compose:` line it hands them to paste.
