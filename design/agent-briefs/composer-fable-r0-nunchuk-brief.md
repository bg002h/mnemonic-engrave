# Lens 2 — USING the SH2-composed wallets in NUNCHUK

Read `composer-fable-r0-common.md` first. Your worktree slug: `nunchuk`.
Your regtest ports (if you need Core as a proxy): `-rpcport=18643 -port=18644`.
Your report: `design/agent-reports/composer-fable-r0-nunchuk.md`.

## THE ONE QUESTION

Take each wallet the SH2 composer can produce — form A (keyed md1 decoded on
the host by `md` into descriptor text) and form B (keyless template + mk1
cards, restored with `mnemonic`/`md`) — and answer, per wallet shape: can
the operator's **Nunchuk** import it, by which route, and does Nunchuk then
**derive the same addresses and expose the same spending paths** as Bitcoin
Core / `md`? Where it cannot, exactly what must the operator do, and are the
spec's own Nunchuk claims TRUE?

The spec's Nunchuk claims to verify (each becomes a row, TRUE / FALSE /
UNVERIFIED with evidence class):
- §5 around l.251: the raw `H` hardened spelling "is valid in Bitcoin Core
  and imported by Nunchuk".
- §8f (l.711-714): NUMS key path — "Bitcoin Core and Nunchuk import this
  form."
- §13 item 3: Nunchuk UI treatment of `or_i` vs `or_d` and of custom
  miniscript templates — UNVERIFIED at spec time; verify it now.
- §13 item 4: import tests of composed outputs into Nunchuk were deferred to
  the journey / F-449 — this is that test.
- `composer-recon-same-fingerprint-two-accounts-import.md` §3: Nunchuk
  rows were SOURCED (C++ read, not built) with "overall admission
  UNVERIFIED" for the miniscript forms — move as many as you can to RUN.

## Nunchuk on this machine

- The operator's install: `~/Applications/nunchuk-linux-v2.1.1_be55150d7db009c75d95c21c636ba07d.AppImage`
  (static-pie ELF; `--appimage-extract` works). An older 1.9.42 zip sits in
  `~/Downloads`. The desktop app SOURCE for 1.9.54 is at
  `/scratch/code/nunchuk-desktop-1.9.54` (`contrib/libnunchuk` is an
  UNPOPULATED submodule; `.gitmodules` names
  `https://github.com/nunchuk-io/libnunchuk.git`).
- **NEVER write to the operator's live Nunchuk state**: `~/.config/nunchuk`,
  `~/.cache/nunchuk`, `~/.local/share/nunchuk` (or anything else under
  `$HOME` the app touches). Reading a config to learn its schema is fine.
  If you run the AppImage at all, run it with `HOME`, `XDG_CONFIG_HOME`,
  `XDG_DATA_HOME`, `XDG_CACHE_HOME` pointed at your scratch directory and
  say so in the report. It is a Qt GUI; you cannot click it. `DISPLAY=:0`
  exists but is the operator's desktop — do not open windows on it.
- Evidence ladder, best first: (a) **RUN** — build libnunchuk (clone
  `nunchuk-io/libnunchuk` into `/scratch/code/shibboleth/.tmp/fable-nunchuk-lib`
  at the tag/commit matching desktop v2.1.1 — find it from the desktop repo's
  tags or the AppImage's embedded version strings) or at least its
  descriptor units + the vendored Bitcoin Core it embeds, and run its parser
  (`ParseDescriptors`, `GetDescriptorsImportString`, `ParseTrDescriptor`,
  `ParseWshDescriptor`, `ParseSortedMultiDescriptor` — names from the prior
  recon; confirm them) on the real SH2 outputs. Budget: if a full build is
  not plausibly under ~30 minutes on 24 cores, build only what parses
  descriptors, or write a small harness that links the pieces. Do not put
  the build tree on `/tmp`. (b) **SOURCED** — libnunchuk source at that tag,
  with file:line. (c) **PROXY** — Bitcoin Core at the version libnunchuk
  vendors (nix), labelled as a proxy. Label every cell with its class.
- Find which Bitcoin Core version libnunchuk vendors at that tag: that
  decides `tr()` miniscript, BIP-389 multipath `<0;1>`, and `H` spelling
  support. Nunchuk also has its own descriptor idiom (`/**`, its BSMS and
  Coldcard-JSON importers, its "custom miniscript template" flow in 2.x) —
  map each SH2 output shape to the route that accepts it.

## SH2 outputs to feed it

Generate them yourself from the fork tip (`md.Compose` in the worktree, or
the demo payload's wallet: `mnemonic-engrave/demo/sh2/build-payload.sh` shows
three seeds and the `key:` records the device seats). Cover at minimum:
single-key `tr` and `wsh`; plain k-of-n under `sh(wsh)`, `wsh`, `tr`
(`multi_a`); each of the five presets under `wsh` and `tr`; a hashlock
policy; a keyless `wsh` hash path; a NUMS-fallback `tr`; a same-seed
two-accounts wallet (the recon's case). Decode with `md 0.16.2` to
descriptor text — note that md-codec 0.44.2 now renders xpub headers with
depth/child from the origin and a ZERO parent fingerprint: does Nunchuk's
importer key anything on the xpub header bytes (depth, parent fp, child)?
That is a first-time question — answer it from source, then by run if you
can.

Address oracle: `md`/`mnemonic` derivation and Bitcoin Core (`deriveaddresses`).
State the Core version for every comparison.

## Deliverable specifics

1. The **matrix**: shape → SH2 form → descriptor spelling → Nunchuk route →
   accept / refuse (verbatim message) → addresses match → spend paths
   visible in Nunchuk → evidence class.
2. The spec-claims table above, each TRUE / FALSE / UNVERIFIED with evidence.
3. An **operator runbook** for the live check the operator can do by
   clicking: the exact text to paste (or file to import) for the demo
   payload's wallet, the route, and the first three receive addresses on
   each chain that Nunchuk should show. The operator is present and can
   walk it; make the runbook good enough that a mismatch is unmistakable.
4. Anything the SH2 side should CHANGE so Nunchuk import works (spelling,
   header, route) goes under findings with severity — a wallet the
   operator cannot import is Important; a wallet Nunchuk imports with
   DIFFERENT addresses is Critical.
