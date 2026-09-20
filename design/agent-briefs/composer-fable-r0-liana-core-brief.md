# Lens 5 — IMPORTABILITY of SH2-composed wallet policies into LIANA and BITCOIN CORE

Read `composer-fable-r0-common.md` first (same directory) — tips, toolchain,
hygiene, settled facts, report contract. This lens was requested by the
operator after lenses 1-4 closed; the fold of those lenses is in flight, so:
- Host CLI `md` is now **0.17.0** (md-codec 0.45.0): `md compose` refuses a
  policy with more than one key-less path at validate time. Use `md compose`
  as your shape generator (it IS the primary; every shape the device can
  cut, it can compose) and `md descriptor` / `md address` for the text and
  the oracle addresses. The fork tip for device cross-checks stays
  `f5b068faf3049ccf97603dfbaa3709f12893df97` (`cmd/policyprobe`).
- Your worktree slug: `liana`. Regtest ports: `-rpcport=18943 -port=18944`.
- Report: `design/agent-reports/composer-fable-r0-liana-core.md`.
- Read lens 2's report (`composer-fable-r0-nunchuk.md`) for the shape list,
  the evidence-class ladder (RUN / SOURCED / PROXY) and the matrix form — and
  lens 1 / lens 3 for what Core already showed (import + funded branch spends
  on v25 and v31.1 for fully seated shapes; refusal of any key-less path).
  Do not re-measure what they measured; go where they did not.

## THE ONE QUESTION

For each wallet shape the composer can produce, can the operator IMPORT it
into (a) **Liana** and (b) **Bitcoin Core** as a working wallet — not merely
parse it — and then RECEIVE and SPEND through that wallet's own machinery,
on every spend path the device's screens promised? Where the answer is no,
what exactly refuses, and what would the operator have to change on the SH2
side (shape, wrapper, key spelling, path form) to make it importable — and
are the spec's claims about Liana and Core TRUE?

## Liana

Liana is Rust (`wizardsardine/liana`); the operator's data dir `~/.liana`
exists (an `installer.log`; older test dirs `~/.liana_orig_test*`) and a
`liana_8.0-1_amd64.deb.asc` sits in `~/Downloads`. NEVER write under
`~/.liana*`; never open a window on `DISPLAY=:0`. Evidence ladder: (a) RUN —
clone `https://github.com/wizardsardine/liana` at the tag matching v8.0 into
`/scratch/code/shibboleth/.tmp/fable-liana-src`, build the `liana` library
crate (descriptor parsing: `liana/src/descriptors/`, `LianaDescriptor`,
`LianaPolicy`) and `lianad`; run `LianaDescriptor::from_str` on every SH2
shape (a small harness binary linking the crate is the gold standard), and
for accepted shapes run `lianad` against your regtest node with the
descriptor to confirm addresses and, if it fits the budget, a spend through
each recovery path (Liana's own timelock semantics: primary path + recovery
paths, `older` only). Own `CARGO_TARGET_DIR` under `.tmp`; never `/tmp`.
(b) SOURCED — the crate's parser rules with file:line. (c) PROXY — none for
Liana; say UNVERIFIED instead. Liana is known (spec §13 item 4, second
lowering review) to refuse any `after` or hashlock path; MEASURE that and the
rest: multisig per path, single-key paths, `tr` vs `wsh`, NUMS forms, the
`<0;1>` multipath spelling, `h` vs `'`, the zero-parent-fingerprint xpub
header md renders, same-seed-two-accounts, and which of the five presets
survive unedited.

## Bitcoin Core

Local `/usr/local/bin/bitcoind` is v25.0 (no multipath, no tapscript
miniscript); a newer Core comes from nix (`nix build nixpkgs#bitcoind`, the
lens 2 agent did it; put `/nix/var/nix/profiles/default/bin` on PATH). Run
BOTH where they differ. Lenses 1 and 3 showed `importdescriptors` and
`deriveaddresses` agree and branch spends succeed for fully seated shapes —
your question is the WALLET journey a real operator runs: create a
descriptor wallet, import the SH2 descriptor (watch-only AND with the
private keys re-versioned for regtest), `getnewaddress`/`listdescriptors`,
fund, then spend each path through `walletcreatefundedpsbt` →
`walletprocesspsbt` → `finalizepsbt` → `sendrawtransaction` (no hand-built
witnesses — that was lens 1's instrument), including a path whose lock has
not yet matured (expect the wallet's own refusal), a hashlock path (can
Core's wallet satisfy a `sha256` preimage at all, and how), and a taproot
NUMS policy (key-path unspendable: does the wallet ever try it). Also the
change chain: does Core derive change on `/1/*` for a multipath import, and
does `md`'s `--chain 1` agree with it. State the Core version for every row.

## Deliverable specifics

1. The **matrix**: shape → descriptor spelling → Liana (accept / refuse
   verbatim, evidence class) → Core v25 / v31.1 (import → receive → each
   path spent through the wallet: Y/N/why) → addresses equal to `md`.
2. A **spec-claims table** for every sentence in `SPEC_wallet_policy_composer.md`
   that names Liana or Core (grep them: §5c, §8a, §8f, §13, §14, F-449),
   TRUE / FALSE / UNVERIFIED with evidence.
3. An **operator runbook** for the demo payload's wallet in Liana (which
   route: descriptor import, the settings, the first three addresses per
   chain) — the operator is present and can click through it.
4. Findings as counterexamples: a wallet Liana or Core imports with DIFFERENT
   addresses is Critical; a composable wallet a coordinator cannot import
   with no notice on the device is Important; a false spec sentence is
   Important.
