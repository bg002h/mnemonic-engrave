# IMPL-S1S3 — post-implementation adversarial EXECUTION review

**Scope:** the whole diff `c3fefe4..5b0007a` on `impl/descriptor-s1s3`, worktree
`/scratch/code/shibboleth/_work/impl-s1s3`, binary `target/debug/me` (`me 0.7.0`),
built from that tree at review time.

**Counts: 1 Critical / 1 Important / 3 Minor / 3 Nit.**

**Verdict: RED — not GREEN. One Critical (C1) and one Important (I1) are open.**

This was an execution review, not a re-read. Every finding below is a
constructed failure: a concrete input, run against the built binary, with the
wrong output pasted verbatim. Nothing here was derived from the spec, the plan,
P0/P1/P2's reports, or their reviews.

---

## 0. Oracles used

Three independent address/identity oracles, so that no finding rests on the
artifact under review agreeing with itself:

1. **`md` 0.13.0**, rebuilt from `descriptor-mnemonic` @ `6864f377`
   (`target/release/md`) — the md1 decode/derive route.
2. **The device**, via `scripts/descriptor-seam-vectors/goprobe` built against
   the fork worktree `/scratch/code/shibboleth/_work/seam-fork` @ `1f09537`
   (Go 1.26.3) — `nonstandard.OutputDescriptor`, `bip380.Parse`,
   `address.Receive`, `md.EncodeMultisig` → `md.WalletPolicyIdChunks`.
3. **A from-scratch Python BIP-32/secp256k1/bech32 derivation** written for this
   review (`oracle.py` in the review scratchpad) — no md-codec, no fork code, no
   shared lineage with either of the above.

Plus a **baseline `me`** built from `c3fefe4`'s tree (verified:
`git diff c3fefe4 60d7dc4 -- crates/ Cargo.toml Cargo.lock` is empty) for the
record-surface differential.

---

## 1. Findings

### C1 (CRITICAL) — under `--as descriptor`, conjunct 1's `multi` refusal short-circuits conjuncts 2–8, suppressing funds-safety refusals and asserting a remedy that is false

**Constructed failure.** An anyone-can-spend wallet — `wsh(multi(0, K1, K2))`,
threshold 0, which the spec itself names as the case the ordering rule exists
for:

```
$ K1='[dc567276/48h/0h/0h/2h]xpub6DiYrfRwNnjeX4vHsWMajJVFKrbEEnu8gAW9vDuQzgTWEsEHE16sGWeXXUV1LBWQE1yCTmeprSNcqZ3W74hqVdgDbtYHUv3eM4W2TEUhpan/<0;1>/*'
$ K2='[f245ae38/48h/0h/0h/2h]xpub6DnT4E1fT8VxuAZW29avMjr5i99aYTHBp9d7fiLnpL5t4JEprQqPMbTw7k7rh5tZZ2F5g8PJpssqrZoebzBChaiJrmEvWwUTEMAbHsY39Ge/<0;1>/*'
$ printf 'wsh(multi(0,%s,%s))' "$K1" "$K2" > /tmp/anyonecanspend.txt

$ ./target/debug/me sysw pack --in /tmp/anyonecanspend.txt --as descriptor --no-passphrase
me: the device's descriptor parser accepts `sortedmulti` and not `multi`. This wallet can
    still be engraved: `--as md1` encodes `multi` policies (for use-site paths md1 can
    represent -- otherwise no path carries it, and the refusal says so). (`sortedmulti`
    differs from `multi` only in key ordering at spend time -- it is not a synonym, so
    `me` will not rewrite it for you.)
rc=3

$ ./target/debug/me sysw pack --in /tmp/anyonecanspend.txt --as md1 --no-passphrase
me: threshold 0 means NO signature is required: anyone who can see this script can spend
    from it. This is almost certainly not the wallet you meant -- and if it already holds
    funds, treat them as at risk now. Nothing was packed.
rc=3

$ ./target/debug/me sysw pack --in /tmp/anyonecanspend.txt --no-passphrase
me: threshold 0 means NO signature is required: anyone who can see this script can spend
    from it. ... treat them as at risk now. Nothing was packed.
rc=3
```

**Two things are wrong with the `--as descriptor` output.**

1. **The sentence "This wallet can still be engraved" is FALSE.** `--as md1`
   refuses the same file, permanently, in every build. No `me` path engraves it.
   §6's row hedges only over *use-site paths* ("for use-site paths md1 can
   represent") — threshold 0 is not a use-site path, so the hedge does not
   reach this case and the claim stands unqualified.
2. **The funds-at-risk warning never reaches the operator.** They asked about a
   wallet anyone can spend from and were told the wallet is fine and the flag
   is wrong. Nothing in the message suggests re-running is worth it: it reads
   as "wrong flag", not "your wallet is broken".

**Why this is the spec's own named case.** §5.1's ordering paragraph:

> *"A wallet no path admits gets its admission refusal regardless of flag and
> build — its status is permanent, and possibly funds-urgent: `sortedmulti(0,…)`
> must hear 'treat those funds as at risk now', never 'nothing is lost by
> waiting'."*

And §5.4's carriage rule states the determination explicitly:

> *"the §4.7 admission refusal where no path admits the wallet (**that
> determination quantifies over both paths, so it needs no flag**)"*

For `multi(0, …)` the refusal that quantifies over both paths is conjunct 2.
The `--as`-omitted path implements this correctly. The `--as descriptor` path
contradicts it — the same file, the same build, two different answers, and the
wrong one is on the invocation **both walked journeys reached for first**
(WALK W4 beat, and Journey 2 Beat 1: *"both walked journeys hit the window on
their first real command"*).

**Mechanism** (`crates/me-cli/src/descriptor/admit.rs:55` and `:60–88`).
`admit()` runs `conjunct_1_shape(d, path)?` first; its `(Some(Multi::Unsorted),
_, true)` arm returns `Err(refusal::multi_under_descriptor())` on
`Path::Descriptor`, and the `?` short-circuits conjuncts 2–8. On `Path::Md1`
the same arm returns `Ok(())`, so 2–8 run — which is why only the
`--as descriptor` path is wrong.

**Scope — seven constructed instances, every one reproducible.** Under
`--as descriptor`, a `multi` input suppresses:

| also-failing conjunct | the suppressed sentence |
| --- | --- |
| 2 — `multi(0,…)` | *"anyone who can see this script can spend from it … treat them as at risk now"* |
| 2 — `multi(5,…)` of 2 keys | *"Funds sent to this wallet would be unspendable"* |
| 3 — 21-key `wsh(multi(…))` | *"derive addresses whose coins cannot be spent"* |
| 5 — mixed `xpub`/`tpub` | *"cannot derive any address from it"* |
| 7 — hardened use-site `*h` | *"addresses for a wallet that cannot exist"* |
| 7 — non-consecutive `<0;2>` | *"errors on every address"* |
| 8 — colliding origin (`gate/colliding-origin-multi`'s own input) | *"no wallet matches this description"* |

All seven give the correct refusal under `--as md1` and under `--as` omitted.

**Why the suite does not see it.** `crates/me-cli/tests/descriptor_as.rs` passes
`"descriptor"` in exactly 2 places, and `grep -En '[^d]multi\('` over all three
test files returns only a comment and a remedy-string literal — **no test drives
`--as descriptor` on a `multi` input at all.** The vector file's `multi` rows
are gate rows, and gate rows are `--as`-omitted by construction
(`gate/colliding-origin-multi` pins `exit_code: 3`, `refusal_row: key-identity`
— which the `--as`-omitted path does produce, so the row passes while the
`--as descriptor` path is unexercised).

**Severity.** Critical: an unmet spec guarantee with a funds-safety payload, and
a false claim about what the tool can do — the "defects in what a tool *claims*
to have done" class the project's severity rule keeps blocking. Nothing is
packed, so no wrong plate is cut; that is what keeps it out of "wrong result",
not out of Critical.

**Not prescribing the fix.** Two directions exist (run the flag-independent
conjuncts 2–8 before conjunct 1's `--as`-dependent `multi` arm; or make
conjunct 1's `multi` text conditional on 2–8 passing). Either changes what the
`--as descriptor` path reports for seven shapes, so whichever is chosen needs
its own row-level assertions — the absence of any `--as descriptor` × `multi`
test is itself part of the finding.

---

### I1 (IMPORTANT) — `address 0: not derived … there is no single first address to compare` is FALSE; the address exists and the device derives it

**Constructed failure.**

```
$ K1='[dc567276/48h/0h/0h/2h]xpub6DiYrfRwNnjeX4vHsWMajJVFKrbEEnu8gAW9vDuQzgTWEsEHE16sGWeXXUV1LBWQE1yCTmeprSNcqZ3W74hqVdgDbtYHUv3eM4W2TEUhpan'
$ K2='[f245ae38/48h/0h/0h/2h]xpub6DnT4E1fT8VxuAZW29avMjr5i99aYTHBp9d7fiLnpL5t4JEprQqPMbTw7k7rh5tZZ2F5g8PJpssqrZoebzBChaiJrmEvWwUTEMAbHsY39Ge'
$ printf 'wsh(sortedmulti(2,%s/<2;3>,%s/<0;1>/*))' "$K1" "$K2" > /tmp/mixeddepth.txt
$ ./target/debug/me sysw pack --in /tmp/mixeddepth.txt --as md1 --no-passphrase

      wallet-id: none -- this wallet has no md1 policy form; identify it by the
                 checksum in the descriptor line and by address 0.
      address 0: not derived -- this wallet's keys take their first receive address at
                 different depths, so there is no single first address to compare.
                 Check the descriptor line against your wallet software instead.
```

**The address exists.** Two independent oracles, agreeing to the character:

```
device  — goprobe, address.Receive(desc, 0)  -> bc1qv70wqy0t9vp4ftlku3yz845x53yqkgm5xlus47m3zq8xzzy503hscqluvy
python  — from-scratch BIP-32 + P2WSH        -> bc1qv70wqy0t9vp4ftlku3yz845x53yqkgm5xlus47m3zq8xzzy503hscqluvy
```

A second shape reaches the same branch and is also wrong:
`wsh(sortedmulti(2, K1/<2;3>, K2/<0;1>))` →
device and python both give `bc1qlccgxwlhr0rp7xfedcau022p50ulf9r3e33anqqdrevvdrdeqj9s8leyuw`.

**The stated reason is also false, twice.** In the second shape both keys sit at
the *same* depth (one child level each: `[2]` and `[0]`); they differ in
**index**, not depth. And `/*` (depth 1) mixed with `<0;1>/*` (depth 2) — a
genuinely different-depth pair — does *not* hit this branch and prints a correct
address (`f3g` below, `bc1qghwumhc…`, device-confirmed). So "different depths"
describes neither the trigger nor the obstacle.

**What is actually true** (`crates/me-cli/src/descriptor/md1.rs:204–231`): the
`Derive` twin maps `<i;i+1>` → `/*` at index `i`, so a `<2;3>` key "wants"
address index 2 while a `<0;1>/*` key wants 0; `derive_address` takes one index
for the whole descriptor, so `index0` becomes `None`. That is a limitation of
the twin, not a property of the wallet.

**Why it matters, and why it is Important rather than Minor.** §5.4 is normative
that the block prints on *every* successful whole-input parse, and WALK W13
established the block precisely because *"the verification is worth the most
exactly here"* — on a refusal, where the operator is being asked to make a
wait-or-switch decision about a wallet they cannot otherwise check. This branch
fires only on refusal paths, so it removes the executable check at exactly the
moment the walk ruled it most valuable, and gives a reason that is not true.

The two lines also compose into a dead end the implementer flagged themselves
(IMPL-P2-report F-3): `wallet-id: none` says *"identify it … by address 0"* and
the next line says there is no address 0. I am not counting that separately —
it is the same defect read twice.

**Why the suite does not see it.** `grep -rn "not derived" crates/me-cli/tests/`
returns nothing; the string exists only at `identify.rs:83`. No vector row
carries a `<a;b>` group other than `<0;1>` and `<0;2>`, and `<0;2>` is refused by
conjunct 7 long before this line.

**Ruling on the implementer's F-3 question** (*"arguably should not exist"*).
The branch should not print a false statement. Either derive per-key at each
key's own want-index — which is what the device does and what §5.4 asks for — or
say what is true (this build cannot derive an address for a wallet whose keys
sit at different use-site indices). Refusing the input instead is the wrong
answer: the wallet is admitted by conjunct 7, `--as descriptor` carries it in a
full build, and §5.4's rule is *parse succeeded ⇒ identify the wallet, always*.

---

### M1 (Minor) — `--as descriptor`'s clap help is not build-marked, and the choice block sends the operator there

§5.1 requires the choice block to mark the build-dead value inline, and it does:

```
      --as descriptor (not available in this build)
```

Its last line then says *"They are not interchangeable -- `me sysw pack --help`
has the comparison."* `me sysw pack --help` renders:

```
          Possible values:
          - descriptor: The canonical re-encoded descriptor, as one `Descriptor` record
          - md1:        The BIP-388 decomposition, as md1 text cards (`MdMk` records)
```

— no marking. The operator who follows the block's own pointer loses the one
fact the block was careful to give them. Not blocking: they have already read
the marked line, and choosing `descriptor` yields the window refusal, which is
truthful and names the live alternative.

### M2 (Minor) — the §5.3(b) label warning echoes operator-supplied bytes verbatim, including ANSI escapes, into the verification block

```
$ sed 's/Name: sh/Name: "; rm -rf \/  \x1b[31mRED\x1b[0m/' wallet.txt > evil.txt
$ ./target/debug/me sysw pack --in evil.txt --as md1 --no-passphrase --out /dev/null 2>&1 | grep warning | cat -v
me: warning: the label ""; rm -rf /  ^[[31mRED^[[0m " is not carried by any record format
    and will not appear on the device. Nothing else is lost.
```

Echoing untrusted bytes is a **pre-existing** class in `me` (baseline
`me --in` prints `me: unrecognized HRP 'this is ^[[3' …`), but the baseline
truncates at ~12 bytes and this warning does not, and it lands adjacent to
`address 0:` — the operator's verification surface. A crafted `Name:` carrying
cursor or clear-screen sequences could scroll the address line away at the
moment it is meant to be compared. Recorded, not blocking: it needs a hostile
wallet-export file, and nothing is packed from the label.

### M3 (Minor) — the label warning fires on refusal paths but not on the `--as`-omitted path

Same file, three invocations:

| invocation | label warning |
| --- | --- |
| `--as md1` (packs) | printed |
| `--as descriptor` (window refusal, nothing packed) | printed |
| `--as` omitted (choice block, nothing packed) | **not** printed |

The text — *"will not appear on the device. Nothing else is lost."* — reads as a
statement about what was just packed, and on the two refusal paths nothing was.
§5.4 says the warning "follows the block, where it applies"; which paths it
applies to is not settled, and the implementation settles it three different
ways. Non-blocking; recorded so the inconsistency is a decision rather than an
accident.

### N1 (Nit) — `wallet-id: none`'s wording diverges from §5.4

§5.4: *"identify it by the checksum in the **canonical** line"*. The code says
*"the checksum in the **descriptor** line"*. The block labels that line
`descriptor:`, so the code is the more executable of the two; flagging only so
the divergence is deliberate.

### N2 (Nit) — a two-descriptor stdin produces a refusal whose quoted fragment spans a newline

```
$ { cat multi.txt; echo; cat multi.txt; } | ./target/debug/me sysw pack --as md1 --no-passphrase
me: ... which failed because: the use-site path is not a path: `<0;1>/*))
wsh(multi(2`.
```

The outcome is right (§5.1: with `--as`, stdin is one document; two descriptors
are one malformed document, exit 3). The rendering leaves an apparently
unterminated backtick across two lines.

### N3 (Nit) — WALK W14's parenthetical mislabels the artifact its own number was measured from

W14 records *"The bequest card (**keyed** single-sig `wpkh`, materialised
`<0;1>/*`, **BIP-84 origin**): 2 md1 strings, 85 + 83 = 168 characters"*.
Measured against this build:

```
bare zpub (Journey 2's actual clipboard line)  -> 2 strings, 85 + 83 = 168 chars   [matches]
[4bbaa801/84'/0'/0']zpub…  (keyed, BIP-84)     -> 3 strings, 67 × 3 = 201 chars    [3 plates]
```

The binary reproduces the walk's *number* exactly for the artifact Journey 2
actually holds; the parenthetical describing it as keyed-with-origin is what is
wrong. Worth correcting because W14 has already been corrected once for a
citation-by-presence error (R0 r10's new-I2), and a future re-run would be
measuring the wrong key.

---

## 2. What was executed and came back clean

Recorded so a later reviewer does not re-spend budget here.

### 2.1 The md1 round trip — the strongest single check

Every input the build packs was re-decoded with the **real `md` CLI** and its
address and WalletPolicyId recomputed. **0 mismatches**, over:

* all 14 vector rows with `host_admits ∧ md1_admits`;
* `neither/wsh-multi` (`md1_admits` without `host_admits`);
* 4 constructed `<2;3>` / `/*` mixtures the vector file does not cover
  (`<2;3>/*`+`<0;1>/*`, both-`<2;3>/*`, `/*`+`<0;1>/*`, both-`/*`);
* all three multisig script forms — `wsh`, `sh(wsh(…))` and **bare
  `sh(sortedmulti(…))`, which no vector row round-trips** — against the device
  *and* the from-scratch Python oracle. The R0-C2 P2SH ⁄ P2SH-P2WSH collision
  hazard (`gui/md1_expand.go:128–135`) is **not** present: `3413hYL5…` vs
  `3Duywi53…` vs `bc1qadgf37z…`, four routes agreeing;
* all three BlueWallet `Format:` values (`P2WSH`, `P2SH`, `P2WSH-P2SH`);
* `tr(KEY)`.

### 2.2 `me`'s printed `address 0:` vs the device, on every path

`address.Receive(desc, 0)` compared against the block's line for all 71 vector
inputs × {`--as md1`, `--as descriptor`, omitted} and 10 constructed shapes ×3.
**0 mismatches.** In particular the (a)/(a″) twin prints the **device-route**
address, not the md1-collapsed one — `/0/*` → `bc1qadgf37z…` (not
`bc1qu2cc6t7…`), `<0;1>` → `bc1qu2cc6t7…`, mixed → `bc1qghwumhc…` — each
independently reconfirmed in Python.

### 2.3 Key-order fidelity for `multi`

Constructed the discriminating case the vector file cannot see (its two keys
happen to derive already-sorted, so `multi` and `sortedmulti` coincide):
`wsh(multi(2, K2, K1))` → `me` prints and the md1 card round-trips to
`bc1q93wr5sn5lfkr4l4craezel896jtnl3k8slrqzpx5shzps878vq4qas5ef4`, the *unsorted*
address, against the sorted `bc1qadgf37z…`. Python confirms. No accidental sort.

### 2.4 The `/*` ⁄ childless seam — checked, and safe

md1 cannot distinguish childless from `/*` (shared chunk-set-id `0x9bf18`), and
the device defaults an *empty* children list to `<0;1>/*` — so an explicit `/*`
packed as md1 could in principle read back as a different wallet. It does not:
`gui/md1_expand.go:146–150` maps `!HasMultipath` → `[WildcardDerivation]`
explicitly, with a comment naming this exact hazard. `me` materialises childless
→ `<0;1>/*` (`bc1qadgf37z…`) and leaves `/*` alone (`bc1qu2cc6t7…`), and both
match the device.

### 2.5 F-212 cross-language WalletPolicyId

`me`'s `wallet-id:` vs the fork's Go `md.EncodeMultisig` → `WalletPolicyIdChunks`,
over every multisig row plus 3 constructed ones: **8/8 evaluable agree**. The 2
non-evaluable rows are the CRLF / leading-space whitespace rows, where the
device refuses the raw bytes by design (the invariant is stated over `canonical`).

### 2.6 Regression on the shipped record surface

Baseline `me` (built from `c3fefe4`) vs this build, byte-comparing rc + stderr +
output blob, over 22 record-surface cases and 31 record-shaped / near-record
cases. Every well-formed and every malformed **record** — mnemonic (valid,
11-word, 13-word, mistyped), md1/mk1/ms1/mt1 (valid, truncated, uppercase,
malformed), `text:`/`pass:`/`seed:`/`tx:` (valid, empty, non-hex, with
parentheses, with an inner colon, with an xpub payload), multi-record
combinations, empty file, whitespace-only, bare address — is **byte-identical**.

The only divergences are inputs the gate is specified to open on, and each is
pinned by a vector row or a §6 row: `[`, `xpub…/`, `Name:`/`Policy:`/`Format:`/
`Derivation:`/`FP:`-only lines, `foo(bar)`, `or_d(…)`, `multi_a(…)`,
`[1,2,3]`, and record+descriptor mixes (multi-record row, exit 4 as today).
**§5.1 gate invariant 1 holds.**

### 2.7 The two walk journeys, re-run end to end

* **J1 (BlueWallet `sh` fixture).** W3's converter referral (F-421) fires:
  `me --in wallet.txt` → *"that looks like a wallet DESCRIPTOR … me sysw pack
  --as <descriptor|md1> --in <your export file>"*, exit 4. W4's window refusal
  fires with W5's rewritten text, verdict-first, no internal labels, after the
  identification block. W6's one-step fact is in the help. W8/W9's (a′) note
  fires — and **only** when materialisation occurred (`wallet.txt` yes;
  explicit `<0;1>/*` and explicit `<2;3>/*` no). W10's `wallet-id:` and
  `address 0:` + compare prompt are present and correct
  (`a67e07d16b2500fde6c557a76c7390f6`, `bc1qtahtpjkgtljxl20j…`, both
  reconfirmed through `md`). W15's watch-only line is present in both tiers.
* **W11's refusal loop is closed.** The Specter `/0/*` file: `--as md1` →
  §5.3(a) with the window substitution applied (*"The scannable-plate path is
  not in this build — keep the export file"*), `--as descriptor` → window
  variant 2 (*"--as md1 cannot carry this wallet either"*), `--as` omitted →
  the input's own refusal directly at exit 3 per §5.4's carriage rule. No
  refusal points at a flag that refuses. Verified for all four (a)/(a″) shapes.
* **J2 (bare zpub).** §4.5's announcement prints `key as supplied:` (the
  operator's `zpub6qpFgG…`) **and** `inferred wallet:` (`wpkh(xpub6C9j4…)`)
  with the normalisation sentence, then the FULL block, then the window refusal
  offering `--as md1`. W13 satisfied.
* **W7** re-measured: no passphrase prompt on this journey; `sealing: NOT
  SEALED` + `strength: no passphrase` as the walk recorded.
* **W10's terminal guard** (pty via `script`): unchanged from baseline, still
  refuses stdout, still says BEARER and still names `picotool` — both logged in
  the walk as pre-existing sysw surface, not this cycle's.

### 2.8 The §6 amendment (commit `de35e30`) — verified TRUE at source and in execution

`bip380.Parse`'s script switch (`bip380/bip380.go:333–340`) has
`case "sortedmulti"` and `default: return error` — no `multi` case. Measured
with goprobe: **8 of 8 `multi` twins are device-REFUSED at parse**
(`nonstandard: unrecognized output descriptor format`), and all 8 `sortedmulti`
originals are device-ACCEPTED. So the amended sentence is true for the whole
`multi` class, and the six rows it governs each print it in place of the device
clause — confirmed by running all six twins under both flags. P2.4's texts match
the amended spec, not the old one.

The `sortedmulti` device clauses the amendment left alone are also true, each
re-measured: hardened `*h` → device derives `bc1qadgf37z…`, byte-identical to
the unhardened wallet; `<0;2>` → `address: unsupported range path element`;
mixed network → `mixes networks`; single-key wrapper → `unsupported descriptor`;
21-key `wsh` → derives `bc1qpe8mfg4fcwun…`; 16-key `sh` → derives
`3M6xZ2CpZFS8duWZ…`; `tr(sortedmulti(…))` → accepted.

### 2.9 §6h — every printed remedy was executed

Extracted the descriptor each refusal tells the operator to supply, and ran it.
`ypub`→`sh(wpkh(xpub…))`, keyed `ypub`, `vpub`→`wpkh(tpub…)`, `86'`→`tr(…)`,
account-1→`wpkh(…/84h/0h/1h…)`, fingerprint-no-path→both stated remedies:
**all pack at rc=0** under `--as md1`, and all reach the window refusal (not a
different error) under `--as descriptor`. All five SLIP-132 versions produce
their own per-version target (`ypub`→`xpub`, `upub`/`vpub`/`Upub`/`Vpub`→`tpub`)
with the operator's own key and origin substituted, and the bare-key forms use
the origin-less spelling §6 requires.

### 2.10 §5.4's tiers, and the implementer's item (c)

* FULL tier where conjuncts 2–8 pass and some path admits the shape — including
  a conjunct-8-**passing** `multi` in the window (`wsh(multi(2,…))` prints
  `wallet-id: 0501609a…`, `address 0: bc1qadgf37z…`, compare prompt).
* PARTIAL tier — exactly the first three lines plus watch-only, no `wallet-id:`,
  no `address 0:`, no compare prompt — for conjunct-8 failures
  (`gate/colliding-origin-multi`), no-path shapes (`wsh(KEY)`), `k=0`, and
  out-of-set use-site paths. Verified line-by-line.
* **Item (c) — the §4.5 announcement in both tiers is CORRECT, not a defect.**
  §4.5 is normative that promotion is *"announced, not silent"*, and §5.4's
  PARTIAL exclusion list enumerates exactly three items, none of them the
  announcement. Suppressing it would print a canonical `pkh(…)` the operator
  never wrote with no explanation. The implementer's reading stands; I ran both
  tiers to confirm the text is identical in each.

### 2.11 Item (a) — `bitcoin = "0.32"` as a direct dependency: ACCEPTABLE

`git diff c3fefe4..5b0007a -- Cargo.lock` is **1 insertion**: the string
`"bitcoin",` under `[[package]] name = "mnemonic-engrave"`. No crate is added to
the tree — `bitcoin 0.32.101` was already present as `md-codec`'s own
dependency. It is used only to name `bitcoin::Network` for
`md_codec::Descriptor::derive_address`, not for parsing, so §4.7's rejection of
`rust-miniscript` *for parsing* is untouched and `descriptor::cascade` is still
the small seven-shape parser. The constellation sibling makes the same use.
No finding.

### 2.12 `--as` against the pre-existing flag surface

| combination | result |
| --- | --- |
| single argv operand | packs, rc=0 |
| two argv operands | `--as packs exactly one descriptor per invocation.` rc=2 |
| argv + `--in` | same, rc=2 |
| stdin (whole stream as one document) | packs; CRLF whole-file also packs |
| `--out` on any refusal (7 shapes tried) | **no file created, 0 bytes on stdout** — §6's two binding rules hold |
| `--expect descriptor` | rc=0 · `--expect cosigner` / `mnemonic` → rc=4 |
| `--region` | packs, 65536-byte image |
| `--passphrase-words 4` | packs; correctly reports `--passphrase-words is IGNORED here` (public payload) |
| `--allow-weak` | packs, pre-existing notice |
| `--iterations 1000` | pre-existing out-of-range usage error, identical to baseline |
| `--allow-argv-secret` | not required for a descriptor (public, watch-only) — correct |
| `me --in <file> --as …` (top level) | clap error rc=2, exactly as WALK W3 recorded |

### 2.13 Grammar edges

Valid `#checksum` accepted (feeding `me`'s own canonical line back in packs at
rc=0); invalid checksum refused with §4.1's generic message, as §4.3's table
records. `'`-hardening accepted and normalised to `h`; uppercase `H` refused —
and the **device refuses it too** (goprobe: `unrecognized output descriptor
format`), so this is agreement, not narrowing. Same xpub at *different* use-site
paths admitted (the legal two-chain wallet, conjunct 8 r2's NEW-I1); same xpub at
the same use-site under different declared fingerprints refused. Key-count
boundary exact: 20-key `wsh` packs, 21-key refused.

---

## 3. Items explicitly NOT re-derived

P0's zero findings, P1's GREEN, P2's gates, the spec, the plan, the vector
file's sha256, nextest/clippy/fmt results, the fork Go seam test, the §6 row
count, and the implementers' mutation tables — all taken as given per the brief.
The duplicate-key row's extra clause (*"and it lets one holder produce two of
the required signatures"*) was checked only far enough to confirm it is licensed
by `PLAN-descriptor-S1S3-r4.md:314-326` and already adjudicated in IMPL-P1-review;
not re-opened.

---

## 4. Verdict

**RED. 1 Critical / 1 Important / 3 Minor / 3 Nit.**

The md1 build path is sound — the strongest check available (round-trip every
packed card through the real decoder and recompute wallet-id and address 0
against two further independent oracles) is clean across every shape the build
packs, including three the vector corpus does not cover. The identification
block, the window, the carriage rule, the gate's record-surface invariant and
the §6 amendment all execute as specified.

Both blocking findings are in the same place: **what the tool SAYS on a refusal
path under an explicit `--as` flag**, which is the one surface no test in this
diff drives to a wrong answer. C1 makes a false claim and swallows a
funds-at-risk warning; I1 makes a false claim and withdraws the verification the
walk was written to install. Neither packs a wrong plate — and neither is
reachable from the vector corpus as it stands, which is the finding underneath
both.
