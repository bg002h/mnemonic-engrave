# Composer Stage 1 — independent adversarial EXECUTION review (round 0)

**Reviewer:** independent execution reviewer (opus), did not author the diff.
**Date:** 2026-09-02.
**Under review:**
- mnemonic-engrave `/scratch/code/shibboleth/wt-composer-s1`, branch `composer-s1`, `59e6f12..90560cb` (5 commits, 10 files, +1503/-2).
- mnemonic-secret `/scratch/code/shibboleth/wt-ms-bip48-p2tr`, branch `bip48-p2tr`, `5f37b43..7f979e5` (1 commit, 4 files, +97/-19).

**Against:** `design/IMPLEMENTATION_PLAN_composer_S1_host_inputs.md` (mnemonic-engrave master `b95df91`) and `design/SPEC_wallet_policy_composer.md` §6a, §8n, §10 items 2 and 5.

**Verdict: 0C / 1I / 3M / 5N.**

**Read-only discipline honoured.** Every mutation was reverted; both worktrees ended
with `git status --porcelain` and `git diff --stat` empty (checked after each mutation
and at the end). Nothing committed, pushed or edited outside this report.

**Master drift noted:** master has moved to `b95df91` since the branch base `59e6f12`,
but the three intervening commits touch only `design/` and `scripts/` — verified with
`git diff --name-only 59e6f12..b95df91`. The branch is not stale against code.

---

## What I re-derived rather than inherited

The brief listed the gates as settled. I re-ran the whole tree once anyway on the
final (reverted) state, because I had mutated it four times:

```
$ cd /scratch/code/shibboleth/wt-composer-s1 && RUSTUP_TOOLCHAIN=1.85.0 \
    cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && \
    cargo nextest run --locked
FMT CLEAN
    Finished `dev` profile [optimized + debuginfo] target(s) in 0.88s   (clippy: no output)
     Summary [  32.194s] 621 tests run: 621 passed, 2 skipped
```

Fixture: 40 rows, sha256 matches the pin.

```
$ python3 -c "import json; r=json.load(open('crates/me-cli/testdata/record_class_vectors.json')); \
              print(len(r)); from collections import Counter; print(Counter(x['class'] for x in r))"
40
Counter({'Unknown': 30, 'Key': 4, 'Now': 4, 'Hash': 2})
```

Every one of the 40 `CASES` names is in the test's `required` list and vice versa
(set difference empty, both directions); no duplicate names.

`sysw_vectors.json` byte-identical to the base — the implementer's claim is true:

```
$ sha256sum crates/me-cli/testdata/sysw_vectors.json ; git show 59e6f12:crates/me-cli/testdata/sysw_vectors.json | sha256sum
7e58779d7f0c80ab4713d17ae50c5200197cc422f77a7ef280a22acbc291a0ac  crates/me-cli/testdata/sysw_vectors.json
7e58779d7f0c80ab4713d17ae50c5200197cc422f77a7ef280a22acbc291a0ac  -
```

**A trap I fell into and am recording so the numbers below can be trusted.** After
mutation D I ran a batch of CLI probes without rebuilding, and the stale mutated
binary made it look as though a `hash:` record never triggers the auto-append. It
does. Every CLI measurement in this report was either taken before the first
mutation or re-taken after `cargo build` on the reverted tree; the ones I re-took
are marked. (This is the "measure by path, not by name" failure mode with a
timestamp instead of a path.)

---

## Lens 1 — counterexample construction against the classifier

Method: craft records the 40-row fixture does not cover, predict the class from a
careful reading of §6a, then run the real binary
(`me sysw pack --in <file> --no-passphrase --no-now`). Depth/child-number variants
were made by base58check-decoding the journey's xpub, patching the serialization
fields, and re-encoding (rust-bitcoin does not verify that a key was actually
derived where its header says).

All measurements below are from the CLEAN rebuilt binary.

| # | record | §6a prediction | measured |
| --- | --- | --- | --- |
| C1 | `key:` depth-4 xpub, origin `48'/0'/0'/2` — last component NON-hardened and equal to the patched child number `2` | admit ("components need not be hardened") | **exit 0, admitted** ✔ |
| C2 | `key:` depth-3 xpub, 3-component origin | admit | **exit 0** ✔ |
| C3 | `key:` depth-0 xpub | refuse | **exit 4** — `xpub depth is not 3 or 4` ✔ |
| C4 | `key:` SLIP-132 `ypub` | refuse (not an xpub) | **exit 4** — `not an extended public key` ✔ |
| C5 | `key:` carrying a **real, well-formed `xprv`** | refuse | **exit 4** — `not an extended public key` ✔ |
| C6 | `now:` body `0001756800` (leading zeros, 10 digits) | admit — `^[0-9]{1,10}$` allows them | **exit 0, admitted** ✔ |
| C7 | `now:` body `00001756800` (11 digits) | refuse | **exit 4** ✔ |
| C8 | `key:` path with uppercase `H` hardened marker | refuse (plan pins it out of scope) | **exit 4** — `path does not parse` ✔ |
| C9 | `now:` body of Arabic-Indic digits `١٧٥٦٦٨٤٨٠٠` (valid UTF-8, non-ASCII) | refuse | **exit 4** ✔ |
| C10 | `key:` odd-length hex body `key:5b3` | refuse | **exit 4** — `body is not lowercase hex` ✔ |
| C11 | `hash:` + 64 hex + a REAL trailing newline on argv | refuse | **exit 4** — `not exactly 64 lowercase hex characters` ✔ |
| C12 | `now:2147483647` (band maximum) | admit | **exit 0** ✔ |
| C13 | `key:` = `[fp/48'/0'/0'/2']xpub…/<0;1>/*` (descriptor form) | refuse | **exit 4** — `not an extended public key` ✔ (but see **M-3**) |

Also checked: `hash:` with a trailing space, `hash:` with an embedded space,
`hash:` with a newline mid-body, a leading space before `key:`, and `HASH:`
(uppercase prefix) — all refused; the last two fall through to the shipped
`Unrecognised` message, which is right (prefixes are case-sensitive, like `text:`).
A record can therefore never carry a newline into the LF-separated public section.

**C5 is the one I most wanted to fail and it does not.** A private extended key
cannot enter the non-secret `Key` class, because rust-bitcoin's `Xpub::decode`
matches the 4-byte version against the xpub/tpub magics. That closes the only
route I could see by which the diff might have made secret material land in a
public section.

**§6a's own naming claim, verified rather than read.** §6a says a `key:` body is
"the key form `md decompose` prints". Measured against the installed `md 0.14.0`
by absolute path (`~/.cargo/bin/md`, because `md` is aliased to `mkdir -p` in this
shell):

```
$ ~/.cargo/bin/md decompose "wpkh([73c5da0a/48'/0'/0'/2']xpub6DkFAX…/<0;1>/*)" --emit keys
[73c5da0a/48'/0'/0'/2']xpub6DkFAXWQ2dHxq2vatrt9qyA3bXYU4ToWQwCHbf5XB2mSTexcHZCeKS1VZYcPoBd5X8yVcbXFHJR9R8UCVpt82VX1VhR28mCyxUFL4r6KFrf
```

No derivation suffix; `key_record()` of that line parses. The spec's claim is true.

**No classification a careful §6a reader would not predict was found.** Lens 1
produced one Minor (M-3, a message-quality issue, not a classification error) and
otherwise nothing.

---

## Lens 2 — mutation-testing the tests

Every mutation was applied with `sed` on one line, the named test run, then
reverted with `sed` (or `git checkout` where the tree was otherwise clean), and
`git diff --stat` confirmed empty after each.

| mut | change | test | result |
| --- | --- | --- | --- |
| A | `main.rs:1600` pre-ceremony `nows.len() > 1` → `> 99` | `a_second_now_is_refused_before_the_passphrase_ceremony` | **FAIL** ✔ |
| B1 | `main.rs:1746` `(*now \|\| composer_record_present)` → `(*now \|\| true)` | `a_payload_without_a_composer_record_packs_byte_identically_to_before` | **FAIL** ✔ |
| B2 | same line → `(*now \|\| false)` | `pack_appends_the_pack_time_when_a_composer_record_is_present_and_says_so` | **FAIL** ✔ |
| C | `main.rs:2833` drop `(records count from 0)` from the `SecondNow` line | `two_operator_supplied_now_records_are_refused_naming_the_second` | **FAIL** ✔ |
| D | drop `Class::Hash` from the append predicate | `pack_appends_the_pack_time_when_a_composer_record_is_present_and_says_so` | **FAIL** ✔ |
| E | ms `derive.rs:188` `Bip48P2tr => Some(3)` → `Some(2)` | `bip48_p2tr_derives_the_composer_taproot_origin`, `bip48_p2tr_json_names_the_path_and_no_assumption` | **both FAIL** ✔ |
| F | ms: delete the `Bip48P2tr` `script_type` arm (falls to `_ => None`, derives at depth 3) | same two | **both FAIL** ✔ |

**The mutated line demonstrably RAN in each case.** Mutation A is the strongest
evidence: with the pre-check neutered the refusal still arrives — from `split`'s
library backstop — but only *after* the ceremony, and the captured failure output
shows exactly that:

```
    strength: 12 words — at or above the threshold
    me: record 2 (records count from 0) is a second now: record.
          record 2: a second now: record; only one is allowed. Remove one.
```

i.e. the passphrase block printed first, which is precisely what the test asserts
must not happen. B1/B2/D changed the observable `show` output; C changed the
asserted substring; E/F changed the derived path and xpub.

**Lens 2 found nothing.** Every behaviour I broke was caught by its named test.

---

## Lens 3 — what did the diff make false elsewhere

Method: grep the whole repo for the sentences the three new prefixes falsify, then
run the repo's own `scripts/fold-propagation-check.sh` over the file the diff
edited.

### The repo's own propagation gate fails on the diff's own file

```
$ ./scripts/fold-propagation-check.sh design/SPEC_sh2_sysw_consumption.md '`text:`/`pass:`/`tx:` record'
== propagation check: SPEC_sh2_sysw_consumption.md ==
  LEFT   `text:`/`pass:`/`tx:` record
           374:| an unclassifiable record | **4** | `me: record 0 (records count from 0) is not a form this container can place: not a BIP-39 mnemonic, not an md1/mk1/ms1/mt1 string, and not a `text:`/`pass:`/`tx:` record.` |

   SUPERSEDED PHRASING SURVIVES -- the fold is not finished.
```

That is **I-1** below. The other two superseded phrasings the diff created
(`` `text:`, `pass:` and `tx:` ``, "three reserved prefixes") report `gone`.

### Everything else in Lens 3 is clean

- `sysw_vectors.json` byte-identical (above), and I confirmed the *behavioural*
  claim independently rather than trusting the fixture: I built master's `me`
  (`/scratch/code/shibboleth/mnemonic-engrave/target/debug/me`, `b95df91`) and
  A/B'd it against the branch binary on three payloads —
  `text:`-only, seed-only, and `text:`+`pass:`+seed — with `--no-passphrase --out`:
  **all three BYTE-IDENTICAL**. §6a's "packs byte-identically to today" holds
  end-to-end, not just at the library seam.
- `README.md`: no mention of `sysw pack` or the reserved prefixes. Nothing to update.
- The `Unrecognised` message now prints with the three new prefixes and still opens
  `record N (records count from 0)`, so `tests/descriptor_seam.rs`'s
  `is_record_refusal` (`err.contains("(records count from 0)")`, `:853-855`) still
  recognises the whole refusal surface. Verified by running a composer refusal and
  reading the first line.
- `record_corpus.rs`'s pre-S2 capture gate: no corpus record starts with one of the
  three prefixes, the three `class_name` arms keep the match exhaustive (the file's
  own doc says a new variant must red it), and the corpus test passes unchanged —
  so invariant 2 ("a record that was already placeable keeps its class") is
  genuinely proved, not merely re-baselined.
- `sysw/expect.rs` matches each `--expect` kind by class *equality*, so the new
  variants cannot be silently absorbed by a wildcard. `--expect key` is correctly
  rejected as an unknown kind (§10 item 2 asks for no vocabulary change).
- The CHANGELOG's claim *"a malformed body is refused with its own line, before any
  passphrase is printed"* — machine-checked with a control:
  `pack --in <seed + key:zz>` (sealed) prints the refusal and no ceremony; the same
  pack without the malformed record prints `write this down` exactly once.

---

## Lens 4 — `ms derive --template bip48-p2tr`

I built an **independent** oracle rather than re-using the plan's: BIP-39 seed via
`hashlib.pbkdf2_hmac`, BIP-32 CKDpriv/serialization written from the BIP text over
the `ecdsa` 0.19.2 secp256k1 group, for the "abandon"×11 + "about" mnemonic with an
empty passphrase.

```
master fp 73c5da0a
m/48'/0'/0'/3' xpub6DkFAXWQ2dHxr7LX1ByDVebj6u3C5KSKTVXWkiVKb3tdYfh9t7FhXzvUVSxNSikoVTRb2bGjvYoW8PqYBReMeswi3megtqDwRCeVs3vxMeH
m/48'/0'/1'/3' xpub6DzhyrnFFYQ1KXnhK7D7U1sD9jf9Cq2E5Ut5HhXdXZFVgEpjz4jNsvEnL1FzP2p4RkMW7MTJC7GWK8CqEWdZsM4XR7Yn8BbbUieRkaTntL2
m/48'/0'/2'/3' xpub6EGx8sPr9FxPQtmPagzaNqpcvG1JsN9m9tFyimaK4tUdfx3kxmJ76M25uDyZVD1mvrH8H1UcX24dVWLEqa51Li5x39WGpWc2eG2jTZdMzrR
m/48'/0'/3'/3' xpub6E6Z3Ss5TXJYQKLeD76XTFYJXyVQzT5FBKY3a7evG61SuqJKBVF2EqzMWydzSEbhyj4ESvnBLpdL8Pde5sSUNL9Y9d6mY214mwuvbspUMK5
m/48'/0'/0'/2' xpub6DkFAXWQ2dHxq2vatrt9qyA3bXYU4ToWQwCHbf5XB2mSTexcHZCeKS1VZYcPoBd5X8yVcbXFHJR9R8UCVpt82VX1VhR28mCyxUFL4r6KFrf
```

`ms derive --hex 00000000000000000000000000000000 --allow-argv-secret --template bip48-p2tr --account N --json`,
accounts 0..3:

```
m/48'/0'/0'/3' xpub6DkFAXWQ2dHxr7LX1ByDVebj6u3C5KSKTVXWkiVKb3tdYfh9t7FhXzvUVSxNSikoVTRb2bGjvYoW8PqYBReMeswi3megtqDwRCeVs3vxMeH defaulted= False mfp= 73c5da0a
m/48'/0'/1'/3' xpub6DzhyrnFFYQ1KXnhK7D7U1sD9jf9Cq2E5Ut5HhXdXZFVgEpjz4jNsvEnL1FzP2p4RkMW7MTJC7GWK8CqEWdZsM4XR7Yn8BbbUieRkaTntL2 defaulted= False mfp= 73c5da0a
m/48'/0'/2'/3' xpub6EGx8sPr9FxPQtmPagzaNqpcvG1JsN9m9tFyimaK4tUdfx3kxmJ76M25uDyZVD1mvrH8H1UcX24dVWLEqa51Li5x39WGpWc2eG2jTZdMzrR defaulted= False mfp= 73c5da0a
m/48'/0'/3'/3' xpub6E6Z3Ss5TXJYQKLeD76XTFYJXyVQzT5FBKY3a7evG61SuqJKBVF2EqzMWydzSEbhyj4ESvnBLpdL8Pde5sSUNL9Y9d6mY214mwuvbspUMK5 defaulted= False mfp= 73c5da0a
```

**All four match the independent oracle byte for byte**, `script_type_defaulted`
is `false` (an explicit script type is a choice, not an assumption), and the
plan's Task 5 table is confirmed by a third implementation. As a bonus, the
oracle also reproduces the shipped `P2WSH_ACCT0` at `m/48'/0'/0'/2'` — which is
the very xpub the composer fixture's `key-journey-cosigner-0` declares an origin
of `[73c5da0a/48'/0'/0'/2']` for, so the fixture's key record is internally
consistent with a real derivation, not a plausible-looking string.

The flipped-and-renamed test **can fail**: mutations E and F above both red it, one
by changing the script type and one by removing the arm so the path silently
shortens to depth 3 (the exact silent-failure mode the plan warned about, since
`script_type` ends in `_ => None`).

**Lens 4 found nothing.**

---

## Lens 5 — journey with the real binary

All runs below are from the clean binary; (a)–(f) were taken before any mutation
and (g)–(k) after the rebuild.

**(a) seed only** — no bound appended (the RULED narrowing).

```
$ me sysw pack --in seed.txt --no-passphrase --out a.bin
sealing:  NOT SEALED — … HOLDS SECRET MATERIAL (record 0 (BIP-39 mnemonic)) …
digest:   f36e 9900 a235 0b1e 1c5f c580 2623 32b9
$ me sysw show a.bin | head -5
pub_len:  93
identity: 07413e737b629b711cb2c00d7629b04340bad2707b004d69d05e0038184dbb36
```
No `now:` record, no note. ✔ §6a: "a payload holding no composer-only record …
carries no bound unless the operator asks".

**(b) seed + `key:`** — bound appended, and the operator is told.

```
me: appended now:1788342957 as the last record (the pack time, a lower bound the device echoes
    next to a time lock; it is never a locktime). Pass --no-now to omit it, or supply your own
    now: record to pin a different bound. Payloads without a key:/hash: record get none unless
    --now is passed.
…
public record 0: cosigner key (key:) — [73c5da0a/48'/0'/0'/2']xpub6DkFAXWQ2dHxq2…
public record 1: pack time (now:) — 1788342957 (seconds): a lower bound on the present the
    device echoes beside a time lock; never a locktime
```

**(c) `key:` + an explicit `now:`** — supplied wins, nothing appended, no note:
`public record 1: pack time (now:) — 1756684800 (seconds)`. ✔

**(d) two `now:`** — refused, second named, exit 4:

```
me: record 1 (records count from 0) is a second now: record.
      record 1: a second now: record; only one is allowed. Remove one.
```
The §8n blockquote is on its own indented line and the seam's
`(records count from 0)` vocabulary opens the message — both invariants hold. ✔

**(e) `--now --no-now`** — clap refuses at exit 2:
`error: the argument '--now' cannot be used with '--no-now'`. ✔

**(f) `--no-now` with a `key:`** — no append; one public record only. ✔

**Extra divergences (question 3: "what else might they reasonably do?"):**

| what else | result | classification |
| --- | --- | --- |
| (g) `--now` **and** a supplied `now:` | supplied wins; `--now` silently does nothing | spec-conformant; **N-3** (no line says the flag was a no-op) |
| (h) `--no-now` with a supplied `now:` | the supplied record is kept — `--no-now` suppresses only the append | correct |
| (i) `--now` on a seed-only payload | appends; the payload gains a public section (`pub_len 118`) it would not otherwise have | the ruling's stated opt-in cost |
| (j) `--now` with no records at all | refused: "no records on stdin…" | correct |
| (k) `--as md1 --now` | the bound lands last, after the three md1 records | correct |
| (l) malformed composer record in a SEALED pack | refused before the ceremony; control confirms the ceremony would otherwise fire | correct |
| (m) operator records that FIT, auto-append overflows the section | exit 4 blaming the operator's records | **M-1** |

---

## Findings

### I-1 (Important) — the diff falsified a measured claim in the very spec file it edited, and the repo's own gate says so

**Where:** `design/SPEC_sh2_sysw_consumption.md:374` (§4.4, *"On the host — measured
by running each one"*).

**What:** the table row for "an unclassifiable record" quotes `me`'s stderr verbatim
as *"… and not a `` `text:`/`pass:`/`tx:` `` record."*. Commit `90560cb` changed that
message to include the three new prefixes (`main.rs:2815`) and updated N9 at
`:265` in the same file — but not this row. The row is now a false quotation of a
program output, inside a normative spec, in a section whose heading asserts every
cell was measured by running it.

**Reproduction:**

```
$ ./scripts/fold-propagation-check.sh design/SPEC_sh2_sysw_consumption.md '`text:`/`pass:`/`tx:` record'
  LEFT   `text:`/`pass:`/`tx:` record
           374:| an unclassifiable record | **4** | `me: … and not a `text:`/`pass:`/`tx:` record.` |
   SUPERSEDED PHRASING SURVIVES -- the fold is not finished.

$ me sysw pack --no-passphrase "this is not a record of any class"
me: record 0 (records count from 0) is not a form this container can place: not a BIP-39
mnemonic, not an md1/mk1/ms1/mt1 string, and not a `text:`/`pass:`/`tx:`/`key:`/`hash:`/`now:`
record. Addresses are not classifiable here, and neither is a wallet descriptor `me` refuses
— see sysw::classify
```

**Why Important rather than Minor.** Two reasons, and neither is the staleness by
itself. First, the plan's Task 7 Step 3 asserts an *exhaustive* enumeration —
*"Four sites enumerate the reserved prefixes as three and become false … (measured
by the R0 fidelity lens, 2026-09-02)"* — and that measurement is wrong: there are
five, and the fifth is in the same file as the first, twelve screens down. An
enumeration claimed complete and demonstrably incomplete is an unsound assumption
carried into the diff. Second, this repo ships a committed gate whose sole purpose
is this defect class, it was not run, and it fails in under a second. §4.4 is the
host-behaviour pin that the fork's Stage 2/3 device work reads.

**Fix HYPOTHESIS (responder to verify):** edit the `:374` cell to the current
message (mirroring the `main.rs:2815` wording), then re-run
`scripts/fold-propagation-check.sh design/SPEC_sh2_sysw_consumption.md '` `text:`/`pass:`/`tx:` record` `'`
and confirm `gone`. Consider also running the propagation check over
`design/` as part of Task 7 rather than relying on a hand-enumerated site list.

---

### M-1 (Minor) — the auto-appended `now:` can overflow the section, and the refusal blames the operator's records

**Where:** `crates/me-cli/src/main.rs:1738-1770` (the append) versus `:1789-1798`
(the `SectionTooLong` message).

**What:** the append adds 25 bytes (a 24-byte `now:` record plus its LF) *after*
every gate, with no check against `MAX_SECTION_LEN` (32734). A payload whose own
records fit is then refused at exit 4 with a message naming a cause that is not
true and a remedy that is not minimal.

**Reproduction** (467 `hash:` records + one 23-byte `text:` record = 32713 bytes of
public section, 21 bytes of headroom):

```
$ me sysw pack --in big.txt --no-passphrase --no-now --out big1.bin
digest:   219a 5fda 686b 42a7 25a3 a017 59c5 2531          # exit 0 — the operator's records fit

$ me sysw pack --in big.txt --no-passphrase --out big2.bin
me: appended now:1788343593 as the last record …
me: these records are too long for one payload: a section caps at 32734 bytes. Split them
    across two payloads.
$ echo $?
4
```

Nothing is written, so no data is lost, and the "appended now:" line directly above
does name `--no-now`. Reachability is low: it needs ~32 KB of composer records
(≈474 hashlocks or ≈200 cosigner keys) landing inside a 25-byte window. That is why
it is Minor and not Important. But *"these records are too long"* is false — the
records are not too long, the payload became too long — and the remedy it offers
(split a wallet-policy backup across two payloads) is materially worse than the
one-flag fix.

**Fix HYPOTHESIS:** compute the would-be public-section length before pushing, and
either skip the append with a named warning, or keep the append and make the
`SectionTooLong` branch say `--no-now` when the append fired. A regression test can
be built from the reproduction above with a smaller cap or the same 467-record file.

---

### M-2 (Minor) — three lockstep divergences the 40-row fixture does not pin

**Where:** `crates/me-cli/src/sysw/composer_records.rs:286-328` (`CASES`).

The fixture exists so a Go port cannot disagree with Rust (§12 item 8, Stage 2
vendors it). Three body-rule behaviours that a reasonable Go port would get wrong
have no row:

1. **Non-ASCII Unicode digits in a `now:` body.** Rust refuses via
   `is_ascii_digit` (measured: `now:` + hex of `١٧٥٦٦٨٤٨٠٠` → exit 4). A Go port
   written as `for _, r := range s { if !unicode.IsDigit(r) … }` would **accept**
   it. This is the sharpest of the three: it is a divergence that produces a
   *different class*, not a different message.
2. **Leading zeros.** `now:` + hex of `0001756800` is **admitted** (measured, exit 0)
   — correct per `^[0-9]{1,10}$`, and worth pinning precisely because it looks like
   it should be refused, and because a Go port reaching for `strconv.Atoi` with base
   0 would read it as octal.
3. **Odd-length hex body.** `key:5b3` is refused (`body is not lowercase hex`); no
   row pins odd length for any of the three classes.

The unit tests cover (1) partially and (2) not at all; neither is in the file the
fork will be measured against.

**Fix HYPOTHESIS:** add three `CASES` rows (`now-unicode-digits`,
`now-leading-zeros-valid`, `key-body-odd-length`), re-run the `regenerate` test,
and re-pin `FIXTURE_SHA256` — the fixture is not yet vendored anywhere, so the
re-pin is free right now and stops being free the moment Stage 2 lands.

---

### M-3 (Minor) — the likeliest operator mis-paste gets a refusal that names the wrong problem

**Where:** `crates/me-cli/src/sysw/composer_records.rs:240` (`Xpub::from_str` arm,
detail `"not an extended public key"`).

A cosigner key copied out of a descriptor rather than out of
`md decompose --emit keys` carries the derivation suffix:
`[73c5da0a/48'/0'/0'/2']xpub6DkFAX…/<0;1>/*`. It is refused (correct — §6a wants the
bare account key), but the operator is told:

```
me: record 0 (records count from 0) is a `key:`/`hash:`/`now:` record whose body fails its
    rule (not an extended public key).
      record 0: key: needs [fingerprint/path]xpub with an origin; a bare xpub is not a key record
```

Neither line is true of what they pasted: it *is* an extended public key, it *does*
have an origin, and it is not a bare xpub. The §8n line is spec-fixed and must not
change (§8n has exactly one line per class), but `detail()` is not, and it is the
line that is supposed to say what actually went wrong.

**Fix HYPOTHESIS:** before `Xpub::from_str`, if `xpub_text` contains `/`, return
`K("the key carries a derivation suffix; give the account xpub alone, as `md decompose --emit keys` prints it")`.
Class and exit code are unchanged, so no fixture row moves; a unit test on
`key_err()`'s detail pins it.

---

### N-1 — `design/FOLLOWUPS.md:12843` (F-301, **OPEN**) quotes the superseded message

Inside F-301's reproduction block. It is a dated reproduction, so leaving it is
defensible — but F-301 is open, and whoever re-runs that pipeline now sees
different text and may misread it as fixed or as a new failure. One line, or a
dated "(message text changed at composer S1)" note.

### N-2 — `design/SPEC_descriptor_input.md:120` quotes it too, but was **already** stale

The same §2.1 transcript also says *"Descriptors and addresses are not yet
classifiable here"*, which the descriptor seam falsified before this cycle. Recorded
as pre-existing, not caused by this diff; do not charge it to this fold.

### N-3 — `--now` is a silent no-op when the operator also supplies a `now:`

Measured (journey g): spec-conformant (*"a supplied `now:` always wins"*), but the
flag the operator typed has no effect and nothing says so. One line on stderr
("a now: record was supplied, so --now appended nothing") would close it; it is
below the bar the journey rule sets ("worse than telling the user nothing"), so
documentation-only unless the operator disagrees.

### N-4 — a test name overclaims what the test checks

`a_payload_without_a_composer_record_packs_byte_identically_to_before`
(`tests/sysw_composer_cli.rs:73`) asserts the absence of the append note and of
`now:` in `show` — it never compares bytes. **The claim itself is true** (I A/B'd
the branch binary against master's on three payload shapes: byte-identical), so
this is naming, not a false PASS. Either rename to
`…_gains_no_pack_time_record`, or make it assert against a committed pre-diff blob.

### N-5 — a sealed composer payload now discloses its pack time in cleartext

`Class::Now` is public by design (the device door must read the bound without a
passphrase), so this is the specified behaviour, not a defect — but it is an
observable change for sealed payloads and the CHANGELOG entry does not mention it.
Worth one clause if the entry is touched again. Not a secret-handling defect: no
secret material is involved.

---

## What I did NOT find

Stated explicitly so a later round does not re-run these:

- **No wrong classification.** 13 constructed counterexamples plus the 40 fixture
  rows all classify as a careful §6a reader predicts.
- **No route for secret material into a public class.** A well-formed `xprv` in a
  `key:` record is refused at the version bytes.
- **No newline or separator hazard.** Every record shape carrying an LF is refused
  before it can split the public section.
- **No test that survives its mutation.** 7 of 7 mutations red their named test,
  and mutation A proves the pre-ceremony ordering is the thing under test, not the
  refusal itself.
- **No byte-level regression for non-composer payloads.** Verified against a
  binary built from master, not against the library seam alone.
- **No stale or unreachable gate.** fmt, clippy, nextest 621/621, fixture sha
  matches, `sysw_vectors.json` unchanged, `record_corpus` invariant-2 capture
  passes unchanged.
- **Implementer's three recorded deviations:** re-derived. The only substantive one
  (Task 4 Step 2, a "refusal test" that failed at Step 2) is a plan-wording slip, as
  reported — mutation A independently confirms the code, not the plan, is right.

---

**0C / 1I / 3M / 5N**
