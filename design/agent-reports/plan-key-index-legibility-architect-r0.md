# Architect R0 — PLAN_key_index_legibility.md @ 2d9fe3e

Reviewer: independent read-only architect agent, 2026-08-19. Scope: the two
questions only — (a) can a competent implementer execute this without inventing
anything, and (b) is any stated assumption factually wrong. No redesign, no
re-litigation of recorded decisions, no repeat of the 46efbae critique round.

**Everything below that says "executed" was actually run** on this machine
(24 cores, `nproc` = 24) against the real binaries
(`/scratch/code/shibboleth/mnemonic-toolkit/target/release/mnemonic`,
`descriptor-mnemonic/target/release/md`, `mnemonic-key/target/release/mk`) and
the committed fixture in `design/journeys/inputs-pathological/`. Note the login
shell here is zsh and does **not** word-split unquoted `$VAR`; every command
below was run through `bash -c`, which is what the journey scripts use.

---

## Verdict

**(a) No.** An implementer cannot execute this plan without making decisions of
their own. §1 says "thread `origin_fingerprint` and `origin_path` into
`PlateEntry`" without saying *in what representation* — and the two obvious
representations differ on whether a new crate dependency is required, whether
the manifest JSON changes, and whether the byte-pinned golden and the specced
schema move. §1 raises three edge cases and specifies the output string for
none of them. §3 says "demonstrate it in the pathological journey, using
`--search-address`" and gives no invocation; the tool's actual input shape for
this fixture is materially different from the one §0 describes, and would have
to be reverse-engineered from `cmd/restore.rs`. §2, after the lint was
withdrawn, has no deliverable location and no acceptance criterion at all.

**(b) Yes — and the two most load-bearing errors are both in the half of §0/§3
that the previous fold *rewrote*.** §0 now asserts "a seed is NOT required in
multisig mode (an earlier critique said it was)". For a keyless multisig
template — the only kind this journey engraves — that is false at a hard floor
in the code, and I executed the refusal. And §3's chosen mechanism
(`--search-address`, "recommended … collision-free") is **refused outright** by
the engine at n=11: estimated 829,804 s against a 3,600 s ceiling. The
measurement §3 defers ("measure first, then decide where it lives") takes
0.076 s to obtain and *invalidates the mechanism the plan already chose*.

The good news, also executed: the recovery §3 wants **does work**, and fast —
15 s wall clock over all 39,916,800 permutations, from shuffled cards, exit 0.
It just needs the id-search form, the operator's own seed, an exact card count,
and a wallet id that **no command in this repo currently prints**. §1's core
claim ("`me` cannot print `@N`") holds for this fixture but is justified by a
reason that is over-broad.

This plan is not executable as written. It is, however, close: the defects are
under-specification and three wrong facts, not a wrong idea.

---

## Decisions the implementer would be forced to make

### D1 — §1: what type do `origin_fingerprint` / `origin_path` become in `PlateEntry`? (blocking)

Plan §1: *"Keep the decoded card. Thread its `origin_fingerprint` and
`origin_path` into `PlateEntry`"*, and *"No new dependency (`mk-codec` is
already a dep, `crates/me-cli/Cargo.toml:23`)"*.

The fields are **`bitcoin` types**, not primitives:

```
mk-codec-0.4.1/src/key_card.rs:40   pub origin_fingerprint: Option<Fingerprint>,
mk-codec-0.4.1/src/key_card.rs:47   pub origin_path: DerivationPath,
mk-codec-0.4.1/src/key_card.rs:9    use bitcoin::bip32::{DerivationPath, Fingerprint, Xpub};
```

(That is the **released** crate the build actually uses —
`Cargo.lock`: `name = "mk-codec" / version = "0.4.1" / source = registry`, not a
path dep on the `mnemonic-key` checkout the plan cites. Same fields; worth
citing the artifact that ships.)

Three facts the plan does not reconcile:

- **`me-cli` has no `bitcoin` dependency.** `crates/me-cli/Cargo.toml` lines
  21–38 list `md-codec, mk-codec, clap, zeroize, serde, serde_json, aes-gcm,
  pbkdf2, sha2, bip39, rand, ms-codec, rpassword`. `grep -rn "bitcoin"
  crates/me-cli/Cargo.toml crates/me-cli/src/` returns exactly one hit — the
  `keywords` array on line 10.
- **`mk-codec` does not re-export `bitcoin`.** `mk-codec-0.4.1/src/lib.rs:51`:
  `pub use key_card::{KeyCard, decode, encode, encode_with_chunk_set_id};` —
  no `pub use bitcoin`.
- **`bitcoin`'s `serde` feature is OFF in this tree.** `Cargo.lock`'s `bitcoin
  0.32.101` dependency list is `base58ck, bech32, bitcoin-io, bitcoin-units,
  bitcoin_hashes, hex-conservative 0.2.2, hex_lit, secp256k1` — no `serde`. So
  `Fingerprint` and `DerivationPath` do **not** implement `Serialize` here, and
  `PlateEntry` is `#[derive(Debug, Serialize, PartialEq, Eq)]`
  (`crates/me-cli/src/manifest.rs:41`).

So the implementer must choose between two designs the plan does not
distinguish:

- **(i) stringify at the `bundle.rs` boundary** — store `Option<String>` /
  `String` in `PlateEntry`. No new dep; the plan's "no new dependency" line
  survives.
- **(ii) store the parsed types** — requires adding `bitcoin = { version =
  "0.32", features = ["serde"] }` to `crates/me-cli/Cargo.toml` (a new direct
  dependency **and** a feature that pulls `serde` into the bitcoin tree), or
  `#[serde(skip)]` on the fields. Either way the plan's "No new dependency"
  sentence becomes false.

This is not a style preference: it changes the dependency graph, the lockfile,
and the answer to D2.

### D2 — §1: do the new fields appear in `manifest.json`, or only in the checklist? (blocking)

The plan says the fields go *into `PlateEntry`* and are *used in the label*. It
never says whether they serialize. `PlateEntry` is the manifest's plate model
(`manifest.rs:41-57`), so by default they would.

If they serialize, three artifacts move that the plan's touch-list does not
name:

- `crates/me-cli/tests/vectors/bundle-md1-mk1.json` — the byte-pinned golden,
  compared at `crates/me-cli/tests/cli.rs:306-308`
  (`assert_eq!(v, expected)`) and again at `:743-745`
  (`"no --preview must be byte-for-byte Phase A"`). Its two `mk1-chunk` entries
  would gain keys.
- `design/SPEC_me_bundle_phaseA.md` §6 — the manifest schema is written out
  field-by-field and annotated ("`string` is omitted for the `ms1` plate;
  `chunk_set_id`/`chunk_index` omitted for unchunked md1 and ms1"). An additive
  field is a spec delta.
- Any consumer. Checked, and the blast radius is genuinely small:
  `design/journeys/build_pdf_pathological.py:271-276` reads only
  `plates[].plate`, `.kind`, `.string`; `build_pdf.py` does not open the
  manifest at all; there is **no Go sidecar source in this repo** (`ls crates/`
  → `me-cli` only), so nothing else parses it.

If instead the label is composed in `bundle.rs` and only a rendered
`Option<String>` label rides along — or the fields are `#[serde(skip)]` — the
golden and the spec are untouched. **The plan does not choose, and the two
choices have different acceptance criteria.** §1's "not a normative change" is
defensible in the codec sense either way, but "the manifest JSON is
contract-free" is not: SPEC §6 plus two byte-for-byte assertions is a contract.

### D3 — §1: the exact fallback string when `origin_fingerprint` is `None`

Plan: *"The label must degrade to something truthful (**e.g.**
`mk1 [path only: 48'/0'/1'/2'] chunk 1/3`)"*. "e.g." is not a spec. Two
implementers produce two different strings, and one of them will be the one the
regenerated transcript pins.

For the main (fingerprint-present) case the plan **is** determinate, and I
verified the primitives render exactly its example:

- `bitcoin-0.32.101/src/internal_macros.rs:197-199` —
  `impl Display for $t { … LowerHex::fmt(self, f) }` via
  `impl_bytes_newtype!(Fingerprint, 4)` (`src/bip32.rs:64`) → 8 lowercase hex.
- `bitcoin-0.32.101/src/bip32.rs:459-471` — `Display for DerivationPath` joins
  components with `/` and emits **no leading `m/`**.
- `src/bip32.rs:210-221` — `Display for ChildNumber` writes `'` for hardened
  unless `f.alternate()`, in which case `h`.

So `format!("mk1 [{fp}/{path}] chunk {i}/{t}")` reproduces
`mk1 [73c5da0a/48'/0'/1'/2'] chunk 1/3` byte-for-byte — provided the
implementer uses `{}` and not `{:#}`. Worth one sentence in the plan; the
`'` vs `h` fork is a real one.

### D4 — §1: empty `origin_path`

`origin_path` is non-optional but may be **empty** — the codec documents the
depth-0 case explicitly (`mk-codec-0.4.1/src/key_card.rs:50-56`:
*"`child_number := last_component(origin_path), or Normal{0} when origin_path is
empty (depth-0 / no-path key)"*). `DerivationPath`'s `Display` on an empty path
emits the empty string, so the plan's own fallback renders
`mk1 [path only: ] chunk 1/3`. Not in the edge list; no instruction.

### D5 — §1: "It should say so rather than silently repeat itself" — say *what*?

The plan raises colliding origins as an edge case and specifies no output. It
also does not say where the collision is computed: `Manifest::checklist()`
(`manifest.rs:76-110`) iterates `self.plates` and *can* see all of them, so
detection is possible — but whether the marker is per-plate (`… (AMBIGUOUS)`),
a trailing note, or a different label shape is left open.

**And this is not a corner case for the shipping fixture.** Measured from
`design/journeys/inputs-pathological/keys/key-*.xpub`:

| origin path | cards |
| --- | --- |
| `48'/0'/0'/2'` | key-00 (A), key-04 (B), key-08 (C) — **3** |
| `48'/0'/1'/2'` | key-01, key-05, key-09 — **3** |
| `48'/0'/2'/2'` | key-02, key-06, key-10 — **3** |
| `48'/0'/3'/2'` | key-03, key-07 — **2** |

With fingerprints present all 11 labels are unique. Encoded
`--privacy-preserving`, the 11 cards collapse to **4 distinct labels**. The
plan's own §1 acceptance ("A privacy-preserving card renders without a
fabricated fingerprint") does not test this at all.

### D6 — §1: multi-wallet bundles and multi-stub cards

`run_bundle`'s doc comment (`crates/me-cli/src/bundle.rs:181`) is *"Validate the
public strings of **one or more wallet backups**"*, and `policy_id_stubs` is a
`Vec<[u8;4]>` (`key_card.rs:34`) — one card may declare several policies. The
plan's edge list covers neither. With an origin-only label, two same-origin
cards belonging to *different* bundled wallets are indistinguishable. The
implementer must decide whether that is in scope (the plan's answer is
presumably "yes, out of scope" — but it must say so, because the label is being
introduced precisely as a disambiguator).

### D7 — §3: the entire `restore` invocation

§3 change 2 is one sentence: *"take one printed receive address, shuffle the
card order, and recover."* The tool's real requirements for this fixture,
established by execution (see F1/F2 below), are:

- `--from <own seed>` is **mandatory** for a keyless multisig template;
- `--account <list>` must name the own seed's account(s), because own keys are
  derived one per `--account` and the pool must be **exactly** n;
- consequently the cards supplied must be **n − k_own**, not "all N";
- cards must be **unassigned** (bare `--cosigner <mk1>`, no `@N=`) — mixing
  forms is refused (`cmd/restore.rs:1609-1613`);
- the chunks of one card must arrive **contiguously and in order**, because
  unassigned chunks are grouped **greedily** (`restore.rs:1583-1606`:
  accumulate into `buf` until `try_decode_cosigner_card` accepts). "Shuffle the
  card order" is safe; shuffling *chunks* is not, and the plan does not
  distinguish them — a live hazard given `me bundle` emits plates in
  `chunk_set_id` order.

None of this is in the plan. An implementer would have to derive it from
`cmd/restore.rs`, which is the definition of a decision pushed onto them.

### D8 — §3: which target, given that the recommended one is refused?

§3 picks `--search-address` and quotes the tool's own "recommended over
`--expect-wallet-id`". Executed, that mechanism does not run at n=11 (F2). The
mechanism that *does* run needs a wallet id the journey does not print (F3).
The plan gives no fallback and no rule for choosing.

### D9 — §3: which number decides where the demo lives?

§3: *"the wall-clock must be measured against the engine's time-cap — if it
refuses or takes minutes, the demonstration belongs in documentation."*

There are **two** numbers and they are ~80× apart:

- The engine's **estimate**, which is what gates: `cap_decision(total,
  per_candidate, accept)` at `permutation_search.rs:388-415`, with
  `per_candidate` from a single-threaded 64-sample micro-calibration
  (`calibrate_per_candidate`, `:446-462`). Thresholds:
  `SILENT_THRESHOLD = 30s` (`:54`), `SEARCH_CEILING = 3600s` (`:59`).
- The **realized** wall clock, which is what a journey experiences — the search
  is parallel (`search_threads()`, `:470-476`).

Measured on the working id-search shape, four runs: printed estimates
187.4 s / 196.7 s / 927.7 s / 1284.9 s (calibration noise across a 7× range),
against a realized **15–16 s**. "Minutes" is undefined; and if the plan means
the estimate, the answer is "document it", while if it means realized time the
answer is "put it in the journey". The plan cannot be executed without picking.

### D10 — §2: where does the documentation go?

*"a stated convention in the journeys README and wherever wallet creation is
described."* `design/journeys/README.md` is 168 lines with these headings:
`The documents` / `Which wallet is "pathological"` / `Corrections already folded
in` / `Corrections to the published documents` / `Findings these runs produced` /
`These scripts did not run for a while…` / `⚠ The shipped PDFs are STALE` /
`Reproducing` / `Test material`. There is **no wallet-creation section**. And
"wherever wallet creation is described" is unbounded — the implementer must both
invent a section and decide the set of other files.

### D11 — §2: "template index" before or after canonicalisation?

`md-codec` renumbers placeholders at encode time to BIP-388 first-occurrence
order — `canonicalize_placeholder_indices`, exported at
`md-codec-0.42.0/src/lib.rs:46`, doc at `src/canonicalize.rs:1-24`: it
*"reshapes a `Descriptor` in place"*, atomically permuting the tree's key
indices **and** the `Divergent` paths vector **and** the per-`@N` TLV maps. So
the `@N` a reader sees after decoding the engraved md1 is the *canonical* index,
which need not be the index the wallet author wrote. A convention that says
"account index = template index" must say which one it binds. (For the
pathological policy the two coincide — its `@0…@10` are already in
first-occurrence order — but the convention is being written for other people's
wallets.)

### D12 — release mechanics

`crates/me-cli/CHANGELOG.md` is tracked, and `SPEC_me_bundle_phaseA.md` §11
records the lockstep convention (version bump + CHANGELOG entry). §1 changes
user-visible output; the plan says nothing about a version bump or a changelog
entry. Minor, but it is a decision.

---

## Factually incorrect assumptions

### F1 (Critical) — "a seed is NOT required in multisig mode" is false for a keyless template

Plan §0, the table row and the sentence beneath it:

> `--from <seed>` | **"REQUIRED for single-sig restore; OPTIONAL in multisig
> mode"**
>
> So: **a seed is NOT required** in multisig mode (an earlier critique said it
> was), but **a target IS**.

The quoted string is the **clap help text** (`cmd/restore.rs:75-79`), which
describes the argument-parser-level rule `required_unless_present = "md1"`. The
runtime floor is stricter. `cmd/restore.rs:1389-1398`, inside
`run_multisig_template_completion`, under the comment
`--- Floor 1(i): --from is REQUIRED + resolve the seed entropy ---`:

```
message: "restore of a keyless MULTISIG TEMPLATE md1 requires --from <seed> \
          (the template carries no keys; the seed derives your cosigner key, and \
          --cosigner <mk1> supplies the others). Supply \
          --from ms1=…/phrase=…/entropy=…/seedqr=…",
```

Executed — the plan's *exact* stated input shape (template + all 11 cards + a
known receive address, no seed):

```
$ mnemonic restore --md1 <3 chunks> --cosigner <all 11 cards> \
    --search-address bc1qkuknuy6dsm0fq44cyyhzqy9wl3ex2n6ed39zxhx867l9wlh4yhlsejms64
exit=2
error: restore of a keyless MULTISIG TEMPLATE md1 requires --from <seed> …
```

The fold overturned the previous round's C1(1) on this point and got it
backwards. `--from` is optional in multisig mode only for a **keyed**
wallet-policy md1, where the descriptor is reconstructed from the md1 alone.
The pathological journey engraves a **keyless** template (`md inspect` on the
engraved chunk set prints `wallet-policy-mode: false`), so §3's whole
demonstration falls on the mandatory-seed side.

Second-order consequence, also executed: once the mandatory own seed
contributes a key, "all N cards" over-supplies the pool.

```
$ mnemonic restore --md1 … --cosigner <all 11 cards> --from phrase=… --search-address …
exit=2
error: too many keys for the cosigner slots: the supplied own keys (one per
--account) + --cosigner keys must EXACTLY equal the wallet's cosigner count. …
```

The gate is `cmd/restore.rs:1903-1919` (`pool.len() < n` → refuse;
`pool.len() > n` → refuse). So the README sentence §3 proposes — *"given the
template, all N cards **in any order**, and a target"* — describes an input the
tool refuses **twice**.

### F2 (Critical) — `--search-address`, the mechanism §3 chose, is refused at n=11

§3 change 2 selects `--search-address` and quotes the tool recommending it. §3
then defers the feasibility question: *"Nobody has run an 11-key search on this
shape … Measure first, then decide where it lives."*

I ran it. The measurement takes 0.076 s and kills the mechanism:

```
$ mnemonic restore --md1 <3 chunks> --from phrase=<master-A> --account 0,1,2,3 \
    --cosigner <7 cards: keys 04..10> \
    --search-address bc1qkuknuy6dsm0fq44cyyhzqy9wl3ex2n6ed39zxhx867l9wlh4yhlsejms64
exit=1
error: estimated exhaustive search time 829804.010112s exceeds the 3600s ceiling;
       re-run with --accept-search-time ≥829804.010112s to acknowledge
real    0m0.076s
```

829,804 s is **9.6 days** of single-thread estimate — 230× the `SEARCH_CEILING`
(`permutation_search.rs:59`). Even divided by this box's 24 threads it is ~9.6 h,
still far over. Per-candidate cost is ~20.8 ms because address-search evaluates
the default `--search-addr-min 0 --search-addr-max 20` window per candidate
(`cmd/restore.rs:178-187`).

§3's own text says *"the engine has a refusal ceiling … Nobody should meet that
at recovery time."* The plan's own demonstration meets it on the first run.

**The recovery itself is fine — it is the flag that is wrong.** Executed, in the
id-search form, with the ten non-own cards supplied in a deliberately scrambled
order (07 03 10 01 09 05 02 08 04 06):

```
$ mnemonic restore --md1 <3 chunks> --from phrase=<master-A> --account 0 \
    --cosigner <10 shuffled cards> --expect-wallet-id ced2270948ecb5af
exit=0 elapsed_s=15
searching 39916800 candidate assignment(s) (est. ≤ 187.3694592s)…
✓ wallet-id (completed): ced2270948ecb5af0779249ac7181f4a
  your seed completes cosigner slot @0
  first recv: bc1qkuknuy6dsm0fq44cyyhzqy9wl3ex2n6ed39zxhx867l9wlh4yhlsejms64
```

**15 seconds**, full 11! space, correct assignment, correct address. So the
demonstration §3 wants belongs in the journey — via `--expect-wallet-id`, not
`--search-address`. The plan's "measure first, then decide" gate was the right
instinct; the measurement simply reverses its already-made choice.

Two corollaries the plan must then state and does not: the id-search prefix
floor is **8 bytes / 16 hex chars** at S = 11!
(`required_prefix_bytes(S) = ceil((log2 S + 32)/8)`,
`permutation_search.rs:322-337`; the doc comment pins the ladder
*"`S = 11!`→8"*), and `--account` must list the own seed's accounts.

### F3 (Important) — the wallet-id target §3 tells operators to record is not the wallet's id

§3 change 1 tells the reader a *"recorded wallet-id prefix"* is a valid target.
The pathological journey does print something under exactly that name — step 9,
`transcript_pathological.txt:176-177`:

```
wallet-descriptor-template-id: 5b48af35d4321a3ac18b43045e2523cc
wallet-policy-id: f89e23f13c697ae62ef10328d71d7e24
```

That is `md inspect` on the **keyless** template. It is not the wallet's id.
Measured, three different values for the same wallet:

| source | value |
| --- | --- |
| `md inspect` on the engraved keyless md1 (what the journey prints) | `f89e23f13c697ae62ef10328d71d7e24` |
| `md inspect` on the journey's keyed `$FULL` md1 (`--path bip48`, §9b) | `232214e4d60c0fa83a6715ba2f7e8ec7` |
| **restore's completed wallet** (the id the search matches) | `ced2270948ecb5af0779249ac7181f4a` |

Feeding the second one to the search returns NO MATCH, exit 4:

```
$ mnemonic restore … --expect-wallet-id 232214e4d60c0fa8
exit=4
searching 39916800 candidate assignment(s) (est. ≤ 196.6700736s)…
✗ NO MATCH
error: restore: multisig-template-search mismatch — derived no key→slot assignment
       of the supplied keys, expected the recorded wallet (--expect-wallet-id / --search-address)
```

The cause: the journey builds its keyed md1 with `--path bip48`
(`transcript_pathological.sh:240`), and `--path` **flattens Divergent to
Shared** (`md-cli/src/main.rs:94`), so all eleven embedded origins become the
same path — while the real wallet's keys sit at four different accounts. The
addresses are unaffected (scriptPubKey does not depend on origins, which is why
§9b's address check passes), but the id is.

So §3's README advice, followed literally against this journey's own output,
produces a target that fails. Worse, `md encode` cannot currently produce the
correct one: it refuses a concrete descriptor
(`md: template parse error: template contains no @i placeholders`) and its
`--key @i=XPUB` / `--fingerprint @i=HEX` flags have no per-key **path**
counterpart. The only producer I found is `restore` itself in explicit mode.

### F4 (Important) — "No new dependency" is conditional, not a fact

Plan §1: *"No new dependency (`mk-codec` is already a dep,
`crates/me-cli/Cargo.toml:23`), no wire change, no normative change."* The
citation is correct (`crates/me-cli/Cargo.toml:23` is `mk-codec = "0.4"`), but
the conclusion depends on D1's unmade choice: threading the *parsed* fields into
`PlateEntry`, which is what §1 literally instructs, requires adding `bitcoin`
(absent from `me-cli`'s manifest and sources) with its `serde` feature (absent
from `Cargo.lock`).

### F5 (Important) — "no normative change" understates the manifest's contract status

True for the codec. But the manifest schema is written out in
`design/SPEC_me_bundle_phaseA.md` §6 and pinned byte-for-byte twice
(`crates/me-cli/tests/cli.rs:306-308`, `:743-745`, against
`tests/vectors/bundle-md1-mk1.json`). Whether that matters is D2's decision; the
plan states the conclusion without stating the condition.

### F6 (Important) — "`me` cannot print `@N`" is true for this fixture, but the stated reason is over-broad

Plan §1: *"It sees cards, and for a **keyless template** the md1 carries no
keys, so there is no key order to match a card against."*

The conclusion holds here — verified directly, `md inspect` on the rebuilt
engraved chunk set:

```
n: 11
wallet-policy-mode: false
wallet-descriptor-template-id: 5b48af35d4321a3ac18b43045e2523cc
note: stdout is a keyless descriptor template (no keys)
```

and the journey encodes with `--path bip48`, i.e. a **Shared** path declaration,
so nothing in the md1 distinguishes `@0` from `@7`.

But "keyless ⇒ no per-`@N` discriminator" is not true in general. A keyless
md1 can carry either of two per-index discriminators without carrying any keys:

- `PathDeclPaths::Divergent(Vec<OriginPath>)` — *"`n` distinct origin paths, one
  per key (header bit 4 = 1)"*, `md-codec-0.42.0/src/origin_path.rs:91-96`; and
- `TlvSection::fingerprints: Option<Vec<(u8, [u8;4])>>` — *"Per-`@N` xpub
  fingerprints (4 bytes each)"*, `md-codec-0.42.0/src/tlv.rs:27-28`, which is a
  field independent of `pubkeys` (wallet-policy mode is
  `pubkeys.is_some() && !pubkeys.unwrap().is_empty()`,
  `src/encode.rs:50-52`).

Either one would let `me` map a card's `(origin_fingerprint, origin_path)` to a
slot with no search at all. The plan's conservative *output* decision ("print
the origin, not a slot number") is still right as a default — but an implementer
handed a Divergent-path md1 has no instruction, and the plan's §6 open question
("Does `me bundle` have the md1 available…? **Unverified.**") is still open in
the committed text although it is a two-line answer: `md1_singles` and
`md1_groups` are locals of `run_bundle` (`crates/me-cli/src/bundle.rs:206-208`)
and the plates vector is built in the same function from line 232 onward, so
yes, the md1 is in hand.

### F7 — the §2 fixture correction is CORRECT (the previous critique was wrong)

Recording this because it is the other half of "flag anything false": the fold's
§2 claim — *"`@0-@3` are master A accounts `0'-3'`, and **`@4` is master B
account `0'`** (`[b8688df1/48'/0'/0'/2']`) — a review asserted `3'` here and is
wrong"* — **holds**. From the tracked key files:

```
key-00 # master A, origin [73c5da0a/48'/0'/0'/2']
key-01 # master A, origin [73c5da0a/48'/0'/1'/2']
key-02 # master A, origin [73c5da0a/48'/0'/2'/2']
key-03 # master A, origin [73c5da0a/48'/0'/3'/2']
key-04 # master B, origin [b8688df1/48'/0'/0'/2']
key-07 # master B, origin [b8688df1/48'/0'/3'/2']
key-08 # master C, origin [28645006/48'/0'/0'/2']
```

`git log -- design/journeys/inputs-pathological/keys/` shows the files last
changed at `e59ce9f`, well before the critique, so this is not drift — the
earlier report simply mis-measured. The fold's correction stands, and its
conclusion ("the current fixture does not follow the convention; `@4` is B's
account 0") is right.

### F8 — §3's "the recoverable case" claim is CORRECT, and the shape is admitted

The previous round left "does the completion engine accept this shape at 11
slots" open. It does. Executed, explicit-assignment mode (no search):

```
$ mnemonic restore --md1 <3 chunks> --from phrase=<master-A> --account 0 \
    --cosigner @1=… … --cosigner @10=…
exit=0
✓ wallet-id (completed): ced2270948ecb5af0779249ac7181f4a
  your seed completes cosigner slot @0
  first recv: bc1qkuknuy6dsm0fq44cyyhzqy9wl3ex2n6ed39zxhx867l9wlh4yhlsejms64
```

`or_i` tree over four `multi()` groups, two `sha256()` hashlocks, all four
timelock kinds, 11 slots — accepted, and the derived first receive address
equals the one in the committed transcript
(`transcript_pathological.txt:192`). One constraint the plan would hit: explicit
mode refuses more than one own account —
`error: explicit --cosigner @N= mode supports a single own account (one --account)`
— so for a master holding 3–4 of these slots, explicit mode is unusable and the
search form is the only route.

---

## Missing acceptance criteria

1. **§1, the collision case.** §1 raises ambiguous labels as an edge case that
   "must be handled, not assumed", then lists three acceptance bullets none of
   which exercises it — and D5 shows it is the fixture's *normal* behaviour
   under privacy-preserving encoding (11 cards → 4 distinct labels).
2. **§1, "updated deliberately, not incidentally"** (the `manifest.rs:228`
   test). That is an intent, not a criterion. What must the new assertion say?
3. **§1, the golden and the schema.** Whether
   `crates/me-cli/tests/vectors/bundle-md1-mk1.json` and
   `SPEC_me_bundle_phaseA.md` §6 change is undecided (D2), so "done" is
   undefined for both.
4. **§1, the other journey.** Acceptance names only the pathological journey.
   The operator journey's committed transcript also carries the string being
   changed — `design/journeys/transcript.txt:137-160`, e.g.
   `plate 2/26  mk1 chunk 1/2  → push via NFC & engrave`. Nothing says it must
   be regenerated, and nothing says whether it can be.
5. **§2 has no acceptance criterion and no deliverable at all.** After the lint
   was withdrawn what remains is: "Documentation, primarily: a stated convention
   in the journeys README and wherever wallet creation is described." No file,
   no section, no test, no way to tell whether it was done. See §2 below.
6. **§3, "with its real exit code"** — which one? Now measurable: `0` for the
   working shape, `1` for the ceiling refusal, `2` for the missing-seed and
   pool-size refusals, `4` for NO MATCH.
7. **§3, the prefix floor.** Acceptance says the README must state "the ceiling,
   the time-cap, and what each of the three outcomes means" — but not the
   `≥ 8-byte` `--expect-wallet-id` floor at n=11, which is the difference
   between the documented recovery working and refusing.

### On §2 specifically (the brief's question 6)

Honest answer: **no, what remains is not actionable.** With the lint withdrawn,
§2 is three paragraphs of reasoning, one "what would change" sentence naming an
unbounded location set, and a "what it does NOT solve" list that ends by
declaring the one concrete sub-question ("re-derive the fixture?") to be *"a
decision, not a step"*. There is no artifact, no location, no criterion, and no
owner. As written it cannot be executed or verified — it can only be agreed
with. If it stays in the plan it needs at minimum: the target file and heading,
the exact convention sentence, and the answer to D11.

---

## Hidden dependencies / ordering problems

**H1 — §3's real prerequisite is missing from the plan, and it is not
documentation.** §3 assumes a target exists. For this fixture the address target
exists but its mechanism is refused (F2); the id target that works is a value
**no command in this repo prints** (F3), and the journey prints a *different*
value under the same name. So §3 has an unlisted, code-or-fixture-shaped
prerequisite — record (or make producible) the completed wallet's id — that must
land **before** §3's README text, or the README ships advice that fails against
its own journey. This is the same gap the previous round flagged as "what the
plan misses #1"; the fold added the "a target IS required" sentence but not the
step that produces one.

**H2 — §1 invalidates two transcripts, and the plan checked one.** The label
change rewrites `design/journeys/transcript_pathological.txt` (31 `mk1 chunk`
lines) **and** `design/journeys/transcript.txt` (lines 137-160). §3's closing
paragraph verifies regeneration for the pathological journey only. F-210
(`design/FOLLOWUPS.md:7561`) is still open and its title names *all three*
transcripts; the repairs (`b822e4a`, `6a42d89`) touched `transcript.sh` and
`transcript_pathological.sh`, so the operator journey plausibly regenerates too
— but the plan neither claims nor verifies it, and §1's acceptance depends on it.

**H3 — 1 → 3 regenerates the pathological transcript twice.** §1 rewrites every
`mk1 chunk` line; §3 then appends a recovery step to the same script. Two
transcript-churning commits where one would do. Cosmetic, but the plan's §5
justifies the order on cost, and this is a cost it did not count.

**H4 — §2 last is fine; §2's *decision* is not orderable.** The order 1 → 3 → 2
is otherwise sound: §1 is independent, §3 depends only on H1, and §2's
convention does not gate either. The one coupling is that §2's "the current
fixture does not follow it" would, if resolved toward re-derivation, move every
key, every card, every wallet id, and therefore §3's recorded target — so §2's
deferred decision is a latent invalidator of §3's artifact. The plan says
"Not obviously worth it", which is a position, but it should be recorded as a
*blocking* answer for §3 rather than an open one.

---

## Open / could not determine

- **Whether the operator journey (`transcript.sh`) regenerates today.** I did
  not run it. F-210 is open and names it; the two repair commits touched it.
  Unknown, and §1's acceptance depends on it (H2).
- **Whether the `tr()` pathological variant behaves the same.**
  `design/journeys/inputs-pathological/backup-strings-tr.txt` and
  `wallet-policy-tr.txt` are tracked; I exercised only the `wsh` path. Project
  memory explicitly warns that measuring one descriptor path gives a wrong
  answer about the other.
- **Whether the ~80× estimate-vs-realized gap is stable.** Measured on a
  24-thread machine; the engine's estimate is a single-thread projection from a
  64-sample calibration and varied 187 s → 1285 s across four runs of the
  identical command. On a 4-core machine the realized time would be ~6× my 15 s
  and the estimate could plausibly cross 3600 s. I did not test that, and it
  bears directly on D9.
- **Whether an `Ambiguous` outcome is reachable for this wallet.** Both my
  searches returned a definite result (`Unique` / NO MATCH). §3's acceptance
  requires the README to explain all three outcomes; I have observed two.
- **Whether any consumer outside this repo reads `manifest.json`.** Searched
  this repo only (`crates/`, `design/journeys/`, tests) — nothing beyond
  `build_pdf_pathological.py`. No cross-repo or GitHub-wide search performed, so
  "no external consumers" rests on local evidence.
- **Whether a keyed md1 with true per-key origins is producible at all** with
  the current `md` CLI. `--key`/`--fingerprint` exist; a per-`@N` path flag does
  not, and a concrete descriptor is refused. I did not read `md`'s encoder far
  enough to rule out another route.

---

## One-line summary of what would make this plan executable

Fix the `--from` fact and the `--search-address` choice in §3, add the
"record/produce the completed wallet id" step §3 silently depends on, make D1/D2
(representation + serialization) explicit decisions in §1 with the golden and
SPEC §6 named as touch points, write the four edge-case output strings, and
either give §2 a file, a heading and a sentence or drop it from the plan.
