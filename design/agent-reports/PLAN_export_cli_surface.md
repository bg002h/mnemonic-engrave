# PLAN: wallet-file export — where it lives and what its command surface is

Date: 2026-08-22. Planning-only recon + surface design. No code was modified.
Every codebase claim below carries a file:line the author read, or a command the
author ran this session (marked MEASURED). Design choices are marked OPINION.

---

## 0. The finding that reframes the question — an export surface already exists

The brief's settled fact "no export surface exists anywhere today" is true of
the four m* CLIs (`ms`/`md`/`mk`/`me`) and **false of the constellation**. The
constellation has a fifth Rust component: `mnemonic-toolkit`, "Top-level
integration crate for the m-format constellation"
(`/scratch/code/shibboleth/mnemonic-toolkit/README.md:5`), which depends on all
three codecs (`ms-codec = "0.7"`, `mk-codec = "0.4.1"`, `md-codec = "0.42.0"` —
`crates/mnemonic-toolkit/Cargo.toml:32-34`) and ships **`mnemonic
export-wallet`**: an 11-format wallet-file exporter with a spec lineage
(`design/SPEC_export_wallet_v0_7.md`, `SPEC_export_wallet_v0_8.md`,
`SPEC_export_wallet_format_descriptor.md`), a refusal contract, and Sparrow +
Bitcoin Core emitters already present
(`crates/mnemonic-toolkit/src/cmd/export_wallet.rs:21-46` — formats:
bitcoin-core, bip388, coldcard, coldcard-multisig, jade, sparrow, specter,
electrum, green, bsms, descriptor; emitters in `src/wallet_export/` — 12 files,
3,998 lines total by `wc -l`, MEASURED).

MEASURED against "our reasonably complex wallet" this session (installed
`mnemonic 0.97.0` == repo HEAD version, `crates/mnemonic-toolkit/Cargo.toml:3`):

| probe | result |
| --- | --- |
| `mnemonic export-wallet --descriptor "<concrete wsh form>" --format bitcoin-core` | **works today**: exit 0, valid `importdescriptors` JSON, 2 entries (receive `/0/*` + change `/1/*`), `range [0,999]`, `timestamp 0` |
| same, `--format sparrow` | exit 1: "`--format sparrow requires --template; descriptor passthrough is not supported by Sparrow's file-import surface`" |
| same with the **tr** form | exit 2: "`All spend paths must require a signature`" — the toolkit has no `--experimental` analog |
| `--slot @0.phrase=…` (secret input) | exit 2: "`mnemonic export-wallet is watch-only by definition; supply only xpub/fingerprint/path slots. To produce an artifact that includes secret material, use 'mnemonic bundle'.`" |
| `nunchuk` anywhere in toolkit `.rs` | zero hits (`grep -rin nunchuk crates --include="*.rs"`, MEASURED) — **no Nunchuk emitter exists** |

So the deliverable is not a new surface; it is **extending a shipped surface**
whose exact gaps for this wallet are now measured: (a) no Nunchuk format, (b)
Sparrow refuses non-archetype descriptors, (c) the keyless tier-4 spend path
refuses on the tr parse, (d) no `--md1` ingest on export-wallet (restore has
one), and (e) the script-type taxonomy misclassifies arbitrary miniscript
(`wallet_export/mod.rs:228`: `Wsh(_) => Ok(WalletScriptType::P2wshMulti)` —
our `or_i` vault is not a multisig, but is classified as one).

---

## 1. Ownership decision

**DECISION (OPINION): watch-only wallet-file export lives in
`mnemonic-toolkit` as extensions to the existing `mnemonic export-wallet`.
`md` keeps exactly one job — rendering the concrete descriptor (`md
descriptor`) — and gains only small parity fixes. No new crate. Hot export, if
built, is a separate toolkit subcommand (§3).**

### Why not a new `md export` subcommand

- **Dependency direction forbids reuse in that direction.** md-cli is upstream
  of the toolkit and cannot depend on it
  (`descriptor-mnemonic/crates/md-cli/src/output_advisory.rs:3-5`: "md-cli is
  upstream of the toolkit and cannot depend on it"). Putting export in `md`
  means re-implementing, not reusing, the toolkit's emitters, slot grammar,
  wallet-name/range/timestamp metadata and refusal contract — ~4k lines with
  bitcoind-differential test backing
  (`export_wallet.rs:118-124` cites `tests/bitcoind_differential.rs` as the
  oracle). Two export surfaces in one constellation will drift.
- **md's own doc says what md is for**: `md descriptor` exists because "paste
  your descriptor" needed a public renderer; the renderer is
  `md_codec::to_miniscript` and "THE RENDERER ALREADY EXISTED"
  (`md-cli/src/cmd/descriptor.rs:8`). Vendor file formats (Sparrow JSON
  envelopes, Core RPC arrays, Nunchuk config) are coordinator knowledge, not
  descriptor-codec knowledge.
- **Consequence for md if we chose it anyway**: md would need a file-output
  story, per-vendor metadata flags, and eventually the secret-hygiene stack
  (see §3) — none of which it has (`md-cli/src` has no mlock/secrets modules;
  directory listing read this session).

### Why not a new crate

Everything a new crate would need — slot parsing, secret advisories, codec
deps, emitters, exit-code ladder, GUI schema surface — already exists in the
toolkit. A sixth component would be pure duplication plus a new release train.

### Consequences for each CLI (the brief's question)

- **`md` (descriptor-mnemonic)**: gains `--experimental` on `descriptor` (and
  `address`) for parity — today `md descriptor --template "$(cat tr.policy)"
  --key …` refuses with "All spend paths must require a signature" (MEASURED,
  exit 1) while the **same wallet renders fine from its 15 keyed md1 chunks**
  (MEASURED, exit 0, full origins + `#he4degaz` checksum). Same argument as
  commit `81938084` ("md verify --experimental: verify must accept what encode
  accepts", `descriptor-mnemonic` git log, read this session). No other change.
- **`ms` (mnemonic-secret)**: no change. Its "no master seed / root xprv /
  private keys on stdout" rule (`ms-cli/src/cmd/derive.rs:1-4`) stays intact
  under both watch-only and the hot ruling in §3.
- **`mk` (mnemonic-key)**: no change; its `--keys` batch record format
  (`[fingerprint/path]xpub`, one per line — `mk-cli/src/keyfile.rs:7-12`) is
  the pattern to reuse *if* a keys-from-file form is ever added to export
  (OPINION: don't add it until a journey needs it).
- **`me` (mnemonic-engrave)**: no change. `me bundle` stays a
  public-strings-in, manifest/plates-out surface
  (`me-cli/src/main.rs:39-58`); wallet files are a PC-side artifact.
- **Go fork / Rust-primary rule**: unaffected. Export is presentation of an
  already-normative descriptor; no normative codec behavior moves. The one
  exception is flagged in Open Questions #2 (xpub re-serialization), which IS
  potentially normative and therefore Rust-first + test-vectored if touched.

---

## 2. Command surface

### 2.1 What already exists (reuse verbatim)

`mnemonic export-wallet` (`export_wallet.rs:206-330`):
`--template <archetype> | --descriptor <desc-or-BIP388-JSON>` (mutually
exclusive, 208-216), `--slot @N.<subkey>=<value>` with stdin via `=-`
(237-265), `--format <11 values>` (267-269), `--output <path|->` default
stdout (271-273), `--network` (226-227), `--wallet-name` (288-294), Core's
`--range/--timestamp/--bitcoin-core-version` (275-286), `--bsms-form`
(315-320), `--from-import-json <path|->` (322-326). Exit-code ladder:
`error.rs:605-650` — 1 bad input, 2 typed refusals (secret input at 632,
missing fields at 631, descriptor parse at 627), 3 future-format, 4 mismatch.

### 2.2 What the export cycle adds (the actual plan)

1. **`--md1 <chunk>…` ingest on `export-wallet`** — the constellation-native
   invocation: the keyed card set already carries the template, all six xpubs
   and all six fingerprints (MEASURED: 15 chunks encode the tr form; `md
   descriptor <chunks>` renders every origin, e.g.
   `[39ec1b6e/270028'/0'/0'/0']`). Mirror `restore --md1`
   (`cmd/restore.rs:76-89`, ingest routes at 300-322); the refusal for a
   pasted md1 where a descriptor was expected already exists
   (`error.rs:272-274`, `Md1CardNotADescriptor { surface: "export-wallet
   --descriptor" }`). Keyless template cards refuse, wording modeled on
   `md-cli/src/cmd/descriptor.rs:55-64` ("a keyless TEMPLATE … has no concrete
   form"). Must tolerate/skip the `chunk-set-id: 0x…` header line `md encode`
   emits (MEASURED).
2. **`--format nunchuk`** — new variant in `CliExportFormat`
   (`export_wallet.rs:21-46`) + `wallet_export/nunchuk.rs`, shape supplied by
   the sibling format agent. The exhaustive matches
   (`format_requires_template`, `export_wallet.rs:54-60`, no `_` arm; the
   `emit_payload` dispatch at 75-105) force every per-format decision at
   compile time — keep that property.
3. **Arbitrary-miniscript class** — add a miniscript-script-path class to
   `WalletScriptType` (`wallet_export/mod.rs:164-173`) so
   `script_type_from_descriptor` (mod.rs:210-247) stops mapping `Wsh(<any>)`
   to `P2wshMulti` (mod.rs:228). Descriptor-faithful formats (bitcoin-core,
   descriptor, sparrow-per-sibling-spec, nunchuk-per-sibling-spec) accept it;
   field-bound formats (electrum, coldcard, jade, green) get a typed exit-2
   refusal naming the class — exactly the discipline the unsorted-multi guard
   already established (`export_wallet.rs:106-137`).
4. **`--experimental`** — same flag name and warning text as `md encode
   --experimental` (`md-cli/src/main.rs:127-138`; "THE PLATE IS BEARER
   ACCESS" warning MEASURED on stderr this session): relaxes only the
   requires-signature sanity rule at the toolkit's descriptor/md1 parse so the
   tr form stops refusing (today exit 2, MEASURED). Warn on every use.
5. **Sparrow miniscript emission** — replace the exit-1 template-only refusal
   for the sibling agent's pinned format. The current Sparrow emitter is
   archetype-shaped (`defaultPolicy.miniscript.script` built from
   `multi/sortedmulti`, rationale at `wallet_export/mod.rs:516-523`).

### 2.3 tr vs wsh at the surface

**They are distinguished by the input artifact, never by a flag.** The tr and
wsh forms are different wallets with different card sets, policy-ids
(`a0b128ce…` vs `9c74e0d2…`) and keys
(`mnemonic-engrave/design/fixtures/reasonably-complex-wallet/README.md:20-49`).
The operator chose the wrapping upstream, at `ms derive --template
bg002h-tr|bg002h-wsh` (`ms-cli/src/cmd/derive.rs:107-126`; level-4 script
`0'`=tr, `1'`=wsh) and at `md encode` time. Export asks "which cards?" —
`--wallet-name vault-tr` / `vault-wsh` is the only human-facing label.

### 2.4 How key material arrives

- **Watch-only, md1 route (preferred)**: no key flags at all — the keyed card
  set embeds the six xpubs + fingerprints (MEASURED, §0/§2.2).
- **Watch-only, descriptor route**: keys ride inside the concrete descriptor
  produced by `md descriptor`.
- **Watch-only, template route**: existing `--slot @i.xpub= @i.fingerprint=`
  grammar (`export_wallet.rs:237-265`), six slots for six cosigners.
- **Hot**: per-slot secrets on the hot subcommand only — §3.

### 2.5 Worked invocations for this wallet

MEASURED this session unless marked (design target). `md` = the repo-HEAD
binary (`descriptor-mnemonic/target/debug/md`, v0.13.0 + descriptor subcommand);
fixture root = `mnemonic-engrave/design/fixtures/reasonably-complex-wallet`.

```sh
# (1) MEASURED — mint the keyed tr card set (6 keys + 6 fingerprints, bearer warning on stderr):
md encode "$(cat tr.policy)" \
  --key @0=xpub6E6MpT6… --fingerprint @0=39ec1b6e  … (×6, from ../../journeys/inputs-hashvault/keys/) \
  --experimental --group-size 0
# → chunk-set-id: 0x5c65a + 15 md1 chunks; exit 0

# (2) MEASURED — render the concrete tr descriptor from the cards (watch-only advisory on stderr):
md descriptor md1ft3j68qpp2… ×15        # exit 0, tr(50929b…,{…multi_a(3,[39ec1b6e/270028'/0'/0'/0']xpub…)…})#he4degaz

# (3) MEASURED — Bitcoin Core watch-only file for the wsh form works TODAY:
mnemonic export-wallet --descriptor "$(md descriptor <wsh cards>)" \
  --format bitcoin-core --output vault-wsh-core.json
# exit 0 → importdescriptors JSON: [{receive /0/*}, {change /1/*}], range [0,999], timestamp 0

# (4) design target — the same wallet, Nunchuk, straight from the cards:
mnemonic export-wallet --md1 md1ft3j68… ×15 --experimental \
  --format nunchuk --wallet-name vault-tr --output vault-tr-nunchuk.bsms
# today: --md1 and --format nunchuk do not exist; tr additionally refuses at parse (exit 2)

# (5) design target — Sparrow, wsh form:
mnemonic export-wallet --md1 <wsh cards> --experimental \
  --format sparrow --wallet-name vault-wsh --output vault-wsh-sparrow.json
# today: exit 1 "requires --template" (MEASURED)

# (6) MEASURED — the refusal that must stay byte-stable:
mnemonic export-wallet --template bip84 --slot @0.phrase="…" --format bitcoin-core
# exit 2: watch-only by definition … use 'mnemonic bundle'
```

stdin/argv/files: md1 strings, templates and xpubs are watch-only-class —
argv is fine (no secret-argv advisory applies; advisories fire only for secret
subkeys, `secret_advisory.rs:41-46`). Output: stdout by default, `--output
<file>` for files, matching the existing surface. Exit codes: reuse the ladder
(`error.rs:605-650`); every new refusal is a typed variant at exit 2.

---

## 3. The hot-wallet ruling

Facts the ruling stands on:

- `md` has **never** constructed private-material output — its
  `OutputClass::PrivateKeyMaterial` is dead code kept only for cross-repo
  advisory byte parity (`md-cli/src/output_advisory.rs:13-15`), and md-cli has
  no mlock/zeroize/secret modules (src listing read; contrast
  `ms-cli/src/mlock.rs`, toolkit `src/mlock.rs`/`secrets.rs`/`secret_string.rs`).
- `ms derive` refuses to be a private-key deriver by design
  (`ms-cli/src/cmd/derive.rs:1-4`).
- `mnemonic export-wallet` is watch-only **by definition**, enforced twice —
  pre-resolve (`wallet_export/mod.rs:111-125`) and post-resolve (134-139) —
  with a typed exit-2 refusal (`error.rs:632`; message MEASURED).
- But the constellation does **not** ban private-key output as such:
  `mnemonic convert --to xprv|wif` emits it (`cmd/convert.rs:42-43, 213-214`)
  with the one-line `PrivateKeyMaterial` stderr advisory (`convert.rs:1215`);
  ms encode/decode/split/combine/repair all emit secret-equivalent output the
  same way (`ms-cli` grep, e.g. `cmd/decode.rs:118`, `cmd/split.rs:107`).
  Secrets on argv warn (`secret_advisory.rs:41-46`); secret-bearing file
  outputs get a world-readable check (`warn_if_world_readable`,
  toolkit `secret_advisory.rs:54-73`).

**RULING (OPINION, so an implementer need not re-decide):**

1. **Hot export does not belong in `md` — not as a flag, not as a
   subcommand.** "md never touches private material" is a structural invariant
   (no secret-hygiene stack to inherit) and it is what keeps md's review
   surface watch-only. Breaking it buys nothing the toolkit doesn't already
   own.
2. **Hot export does not go into `export-wallet` as a flag.** "Watch-only by
   definition" must remain a property of the command name, with
   `validate_watch_only` unconditional. A `--hot` flag would make every
   watch-only invocation one flag-edit (or one shell-history recall) away from
   writing spendable material, and would make a twice-enforced invariant
   conditional on flag parsing.
3. **If hot export is built, it is a distinct toolkit subcommand** — proposed
   `mnemonic export-signer` (the name states what the file becomes). Contract:
   - Wallet source: same `--md1` / `--descriptor` as export-wallet.
   - Key material: the existing secret slot subkeys, allowed here —
     `--slot @i.phrase=-`, `@i.ms1=`, `@i.entropy=-`, `@i.xprv=-`
     (subkey grammar already defined and shared, `export_wallet.rs:237-265`;
     derivation machinery exists, `derive_slot.rs` /
     `derive_bip32_from_entropy` per `restore.rs:31`). Slots not supplied stay
     watch-only in the emitted file — partial-hot is the multisig norm.
   - A supplied secret whose derived key matches no descriptor slot is a hard
     error (a paste error, not a request to embed an unrelated secret).
   - Output: `--output <path>` **required** — no stdout default. Create
     `0600` + refuse-to-overwrite (`create_new`); `--output -` allowed
     explicitly and emits the canonical `PrivateKeyMaterial` advisory
     (byte-stable text, parity-tested across repos —
     `md-cli/src/output_advisory.rs:1-7`). The advisory line also fires when
     writing the file, plus `warn_if_world_readable` on the result.
   - **No interactive confirmation.** The constellation is non-interactive
     throughout; confirmations are theater in scripts. Refusal-by-default is
     expressed structurally: the watch-only command refuses, and the hot
     command must be *named*. That is the confirmation.
   - Process hygiene as elsewhere: `set_non_dumpable`
     (`ms-cli/src/main.rs:168`, `md-cli/src/main.rs:342`), zeroize/mlock from
     the toolkit's existing modules.
4. **Sequencing**: ship watch-only first. Build `export-signer` only when a
   sibling format agent shows a real file-shaped hot artifact for a target
   (Core `importdescriptors` with xprv is real; whether Sparrow/Nunchuk have an
   importable hot *file*, versus in-app seed entry, is theirs to pin).
5. Note for the format agents: for this wallet, hot spendability of tiers
   1/2/4 also requires the **sha256 preimages** (tier 4 needs *only* H3's —
   "the plate is bearer access", fixture README:66-71). No standard wallet
   file carries preimages; a hot export of this wallet is spendable-as-filed
   only for tier 3. Say so in the artifact or refuse hot for
   hashlock-bearing descriptors.

---

## 4. Reuse — what must not be reimplemented

| what | where (read this session) |
| --- | --- |
| Concrete-descriptor rendering | `md_codec::to_miniscript_descriptor{,_multipath}` (`descriptor-mnemonic/crates/md-codec/src/to_miniscript.rs:51,241`), surfaced as `md descriptor` (`md-cli/src/cmd/descriptor.rs:40-97`) |
| Dual ingest shape (phrases xor template+keys) | `md-cli/src/cmd/build.rs:20-43` (`DescriptorInput`/`build_descriptor`), with `strip_md1_inputs` (`md-cli/src/cmd/mod.rs:5`) |
| The whole export pipeline | `emit_payload` dispatch + `collect_missing` refusal channel (`export_wallet.rs:75-105`), `EmitInputs` (`wallet_export/mod.rs:498-538`), `CheckedDescriptor` checksum newtype (mod.rs:442-478), existing emitters incl. `sparrow.rs` (258 lines) and `bitcoin_core.rs` (129 lines) |
| md1→wallet ingest to lift into export-wallet | `restore --md1` (`cmd/restore.rs:76-89`, 300-322) |
| Slot grammar (shared with bundle/verify-bundle) | `slot_input::parse_slot_input` (`export_wallet.rs:259-265`; SPEC_export_wallet_v0_8.md:34) |
| Watch-only enforcement + advisories | `validate_watch_only{,_resolved}` (`wallet_export/mod.rs:111-139`), `secret_advisory.rs:41-73` (argv warning, output-class advisory, `warn_if_world_readable`) |
| Exit-code ladder | `error.rs:605-650` |
| Batch key-record file format, if ever needed | `mk-cli/src/keyfile.rs:7-12` (`[fp/path]xpub` per line, comments allowed) |
| The named fixture + its inputs | `mnemonic-engrave/design/fixtures/reasonably-complex-wallet/` + `design/journeys/inputs-hashvault/{keys,seeds}/key-{0..5}.*` (xpub + origin comment per file; one BIP-39 seed per file — read) |

---

## 5. Open questions

1. **tr/wsh sanity asymmetry (blocks relying on the wsh path).** The keyless
   tier-4 refuses at parse for the **tr** form in both `md` (template path)
   and the toolkit, yet the **wsh** form — same keyless tier — parses clean in
   both without any relaxation (all four outcomes MEASURED). Explain via
   rust-miniscript source (does `Wsh` `from_str` skip `sanity_check`?) before
   building on it: if wsh *should* refuse, a dependency bump could break
   worked invocation (3).
2. **Normalized xpub re-serialization — potentially normative.** `md
   descriptor` renders keys re-serialized at depth 0 (`xpub661…` prefix, all
   six; MEASURED — the supplied `xpub6E6…` account keys do not appear
   byte-for-byte), origins carried separately in `[fp/path]` brackets. Do
   Sparrow / Nunchuk / Core accept an xpub whose depth field disagrees with
   its origin path length? Format agents must pin this; if the original
   serialization must be preserved, that is an md-codec wire question —
   risk-set, Rust-first, test-vectored per
   `mnemonic-engrave/CLAUDE.md` (Rust-primary rule).
3. **Does export-wallet's `--md1` route need fingerprints to be mandatory?**
   Without `--fingerprint` at encode time the rendered descriptor carries no
   origins at all (MEASURED on the wsh template path). An origin-less export
   is near-useless to a coordinator and unseatable on-device (fixture
   README:87-91, F-227). OPINION: warn loudly, don't refuse — mirroring `md
   encode`'s F-227 warning (commit `65cd940a`).
4. **Hot file shapes.** Whether Nunchuk/Sparrow have an importable *hot* file
   at all — gates whether `export-signer` targets anything beyond Bitcoin
   Core. Owned by the sibling format agents.
5. **`md descriptor --experimental` parity fix** — small, owned by
   descriptor-mnemonic (§1, md consequence).
6. **Spec process**: additions in §2.2 amend `SPEC_export_wallet_v0_8.md`
   (or a v0.9) in mnemonic-toolkit; that repo's R0 gate applies since
   export-wallet emits funds-relevant artifacts (risk set b/c of
   funds/addresses, per `mnemonic-engrave/CLAUDE.md` risk-set definition).
