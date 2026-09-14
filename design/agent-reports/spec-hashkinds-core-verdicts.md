# RECON: Bitcoin Core verdicts on the four hash miniscript fragments, per wrapper

Measured verbatim against a real node. No design or code-quality opinions below —
facts only.

## Environment

- Binary: `/usr/local/bin/bitcoind`, `/usr/local/bin/bitcoin-cli`
- `bitcoind --version` banner (verbatim, two lines — see "Note on the binary" below):
  ```
  Bitcoin Satellite version v0.2.4
  Bitcoin Core version v25.0.0
  ```
- `bitcoin-cli --version`: `Bitcoin Core RPC client version v25.0.0`
- RPC-reported version (`getnetworkinfo`): `"version": 250000`, `"subversion": "/Satoshi:25.0.0/"`
- **Note on the binary**: the daemon prints an extra "Bitcoin Satellite version
  v0.2.4" line before the Core version line — this local build is a Bitcoin
  Satellite fork layered on Core 25.0.0 (blocks-via-satellite relay patch). The
  RPC subversion string (`/Satoshi:25.0.0/`) and protocol version (70016) match
  plain Core 25.0.0, and nothing in the satellite-relay feature touches
  descriptor/miniscript parsing. Recorded per instructions to confirm the
  version rather than assume it; flagging it because a "Satellite" line in a
  version banner is not what a reader expects from "Core 25.0.0" and could
  otherwise read as a typo.
- Throwaway regtest datadir: `/scratch/code/shibboleth/.tmp/recon-hashkinds-datadir`
  (created and deleted this session; the node ran on `regtest=1` with
  `rpcport=18832` / `port=18833` inside a `[regtest]` section). A second,
  pre-existing `bitcoind` process (PID 139471, running since before this
  session) was left untouched throughout.
- Keys used (both derived from real regtest wallet addresses, not invented):
  - `PUBKEY` (leaf-script key): `037cbb57c008891dec0c33bb43ac50edeb21e0b62ce0ac98e9211fd08c98c52f7f`
  - `PUBKEY2` (taproot internal key): `02b0fc684fd68289114d58915632d5d0e3a58050b388e8e07be0358ea3a3c6ee24`
  - No xpub was needed — both are already regtest-agnostic raw compressed
    pubkeys, so the mainnet/testnet version-byte issue does not arise for this
    matrix.
- Digests used (real hash outputs of the ASCII string `hashkinds-recon`, not
  placeholder zero-filled hex):
  - 32-byte digest (for `sha256`/`hash256`): `a3b6adf6ac28d93a3048704fddf8523be64c1ecfca7e29a00c3bcdeb84ad6b95` (64 hex chars)
  - 20-byte digest (for `ripemd160`/`hash160`): `bd49799e253f26a68e5514e9f45084bf8db3777e` (40 hex chars)
- Miniscript used in every cell: `and_v(v:pk(PUBKEY),<fragment>)` — valid under
  every wrapper's context rules independent of the hash fragment, per the recon
  brief.

## Headline finding — read this before the table

**Bare `sh(...)` and the taproot leaf `tr(KEY,{...})` refuse with the exact
same error string as each other, for all four hash kinds, and that string
names no hash fragment at all:**

```
error code: -5
error message:
Miniscript expressions can only be used in wsh
```

This confirms the brief's expectation for `tr(...)` (Core 25.0.0 has no
tapscript miniscript) and additionally shows the **bare `sh(...)` refusal is
the identical rule, not the 520-byte redeemScript limit and not anything
hash-fragment-specific**. Practically: Core's descriptor-miniscript parser
gates on the *immediate* enclosing context being P2WSH (segwit v0) — being
nested under `sh(wsh(...))` still counts as "in wsh" and is accepted; being
directly under `sh(...)` (Legacy/P2SH) or `tr(...)` (Tapscript) is refused
before any script-size or fragment-specific check ever runs. **None of the
four hash kinds behaves differently from the others under `sh` or `tr` — the
refusal is identical in cause and in wording across all four.** This is
exactly the class of thing F-514 is understood to have gotten wrong: an
operator-facing warning must not attribute the `sh`/`tr` refusal to the hash
fragment (kind, digest length, etc.) — it is refused regardless of which
fragment, or even with no hash fragment present at all (see verification
below).

**Confirms design decision 4** independently: Core itself enforces the digest
length strictly per fragment kind (32 bytes for `sha256`/`hash256`, 20 bytes
for `ripemd160`/`hash160`), so "digest length derived from kind" in
`HashLock { kind, digest }` matches how Core actually validates these
fragments — a mismatched length is rejected, not coerced or truncated.

No measurement here contradicts design decisions 1-3 (composer authorability,
`hash:` record/device-prompt behavior, or the 20-byte-warning condition) —
those are composer/device UX choices this recon does not touch; Core's
descriptor parser has no opinion on them.

## The full matrix

All commands run via `bitcoin-cli getdescriptorinfo <descriptor>` (no
checksum needed as input) on regtest; ACCEPTED rows also ran
`bitcoin-cli deriveaddresses <checksummed descriptor>` and recorded address 0.

| kind | wrapper | verdict | detail |
|---|---|---|---|
| sha256 | `wsh(...)` | ACCEPTED | addr: `bcrt1qt9y8rsyary0d4eewm59ren7n6rcw8cq90r3k50y2jqs6p2yrwvpq35ke0k` |
| sha256 | `sh(wsh(...))` | ACCEPTED | addr: `2N1VP6TmkCWDq8ZR6XitQG4iU6BuzqCcujx` |
| sha256 | bare `sh(...)` | REFUSED | `Miniscript expressions can only be used in wsh` |
| sha256 | `tr(KEY,{...})` | REFUSED | `Miniscript expressions can only be used in wsh` |
| hash256 | `wsh(...)` | ACCEPTED | addr: `bcrt1qsppr3kufza0479ltd9z86rn869grmhplzum62vwg9pr9gyu42g8qygl96c` |
| hash256 | `sh(wsh(...))` | ACCEPTED | addr: `2Mz4aXKmY6LjkAZxnNmGFJP3GNL5Mmx9J2R` |
| hash256 | bare `sh(...)` | REFUSED | `Miniscript expressions can only be used in wsh` |
| hash256 | `tr(KEY,{...})` | REFUSED | `Miniscript expressions can only be used in wsh` |
| ripemd160 | `wsh(...)` | ACCEPTED | addr: `bcrt1q383u5465vjyq4gzyjzl5u7csguedy2xucwst5tja8uhq6vn8x75swj39lp` |
| ripemd160 | `sh(wsh(...))` | ACCEPTED | addr: `2NAC2h4K3ZKhKRAsiap3d34U1sC5zy6xSwp` |
| ripemd160 | bare `sh(...)` | REFUSED | `Miniscript expressions can only be used in wsh` |
| ripemd160 | `tr(KEY,{...})` | REFUSED | `Miniscript expressions can only be used in wsh` |
| hash160 | `wsh(...)` | ACCEPTED | addr: `bcrt1qcy34w35p8m0f3vekxskp4njc7gp05l4tr739kq0ggtfg832yxj9sjwt2u6` |
| hash160 | `sh(wsh(...))` | ACCEPTED | addr: `2N2SnjibKdDiwu7DyDpNqW6MnSBCoDp2a57` |
| hash160 | bare `sh(...)` | REFUSED | `Miniscript expressions can only be used in wsh` |
| hash160 | `tr(KEY,{...})` | REFUSED | `Miniscript expressions can only be used in wsh` |

**On the 520-byte redeemScript limit and Legacy context rules noted in the
brief**: they never get a chance to matter for this matrix. The bare-`sh(...)`
refusal fires at the "is this immediately inside wsh" gate, which runs before
any script-size or Legacy-context-specific fragment check. So the REFUSED
verdict for every bare-`sh` row above is **not** a hash-fragment finding and
**not** a 520-byte-limit finding — it is the same "Miniscript expressions can
only be used in wsh" rule as `tr`, independent of the fragment entirely. To
make sure this is Core's blanket miniscript-outside-wsh rule and not somehow
coincidentally about hashes, the same `sh(and_v(v:pk(PUBKEY),v:pk(PUBKEY2)))`
(no hash fragment at all) was also tried and produced the identical error —
recorded in the raw transcript.

## Digest-length and malformed-argument behavior

Tested by swapping in a 20-byte digest where a fragment expects 32 bytes (and
vice versa), all under `wsh(and_v(v:pk(PUBKEY),<fragment>))`:

| test | verdict | error |
|---|---|---|
| `ripemd160(<64-hex, 32-byte digest>)` | REFUSED | `A function is needed within P2WSH` |
| `sha256(<40-hex, 20-byte digest>)` | REFUSED | `A function is needed within P2WSH` |
| `hash160(<64-hex, 32-byte digest>)` | REFUSED | `A function is needed within P2WSH` |
| `hash256(<40-hex, 20-byte digest>)` | REFUSED | `A function is needed within P2WSH` |
| `ripemd160(<38-hex, 19-byte, off-by-one>)` | REFUSED | `A function is needed within P2WSH` |
| `ripemd160(<40-char string with a non-hex 'z'>)` | REFUSED | `A function is needed within P2WSH` |
| `ripemd160()` (empty argument) | REFUSED | `A function is needed within P2WSH` |

**Finding: Core gives one identical, generic error string
(`A function is needed within P2WSH`) for every kind of malformed hash
argument tested** — wrong length in either direction, off-by-one length,
non-hex characters, and an empty argument all produce the exact same message.
Core does **not** report the actual expected/received length, and the message
does not name the fragment or the specific problem. A spec or UI must not
promise a length-specific or content-specific error from Core here; the
string carries no diagnostic detail beyond "this failed to parse as a
function inside P2WSH."

Digest length **is** enforced (every mismatched-length or malformed case
above is REFUSED, never silently accepted/truncated/padded) — it just isn't
reported informatively.

## ripemd160 vs hash160, sha256 vs hash256

Across every measurement above (the 4x4 wrapper matrix and the digest-length
probes), `ripemd160` and `hash160` behaved identically to each other, and
`sha256` and `hash256` behaved identically to each other, in every dimension
tested: wrapper acceptance/refusal (same wsh-only gate), digest-length
enforcement (20 bytes for the ripemd-family, 32 bytes for the sha-family),
and error strings (same generic messages). No behavioral difference beyond
the opcode itself was observed in any test run this session — if one exists
elsewhere in Core's descriptor code, it was not surfaced by this matrix.

## Exact commands (rerun recipe)

```sh
# 1. throwaway datadir + conf
mkdir -p /scratch/code/shibboleth/.tmp/recon-hashkinds-datadir
cat > /scratch/code/shibboleth/.tmp/recon-hashkinds-datadir/bitcoin.conf <<'EOF'
regtest=1
server=1
listen=0
txindex=0
fallbackfee=0.0001
[regtest]
rpcport=18832
port=18833
EOF

# 2. start node
/usr/local/bin/bitcoind -datadir=/scratch/code/shibboleth/.tmp/recon-hashkinds-datadir -daemon
CLI="/usr/local/bin/bitcoin-cli -datadir=/scratch/code/shibboleth/.tmp/recon-hashkinds-datadir -regtest"
$CLI getblockchaininfo   # wait for RPC to come up

# 3. keys (from a real wallet, not invented)
$CLI createwallet w1
ADDR=$($CLI -rpcwallet=w1 getnewaddress "" bech32)
PUBKEY=$($CLI -rpcwallet=w1 getaddressinfo "$ADDR" | grep '"pubkey"')
ADDR2=$($CLI -rpcwallet=w1 getnewaddress "" bech32)
PUBKEY2=$($CLI -rpcwallet=w1 getaddressinfo "$ADDR2" | grep '"pubkey"')

# 4. digests
SHA256_HEX=$(echo -n "hashkinds-recon" | openssl dgst -sha256 -binary | xxd -p -c 256 | tr -d '\n')
RIPEMD160_HEX=$(echo -n "hashkinds-recon" | openssl dgst -ripemd160 -binary | xxd -p -c 256 | tr -d '\n')

# 5. matrix cell, e.g. sha256 x wsh
$CLI getdescriptorinfo "wsh(and_v(v:pk($PUBKEY),sha256($SHA256_HEX)))"
$CLI deriveaddresses "<descriptor-with-checksum-from-previous-output>"

# ... repeat with fragment swapped for hash256($SHA256_HEX),
#     ripemd160($RIPEMD160_HEX), hash160($RIPEMD160_HEX), and wrapper swapped
#     for sh(wsh(...)), sh(...), tr($PUBKEY2,...)

# 6. teardown
$CLI stop
rm -rf /scratch/code/shibboleth/.tmp/recon-hashkinds-datadir
```

Full raw transcripts (all 16 matrix cells plus the 7 length/malformed-argument
probes) are saved for this session at
`/scratch/code/shibboleth/.tmp/recon-hashkinds/matrix_results.txt`,
`length_results.txt`, and `length_extra_results.txt` — these are under
`.tmp/` (not tracked, not part of this report) and may not survive cleanup;
the tables above are the durable record.

## Cleanup performed

- Throwaway node stopped (`bitcoin-cli stop`), confirmed RPC connection
  refused afterward.
- Throwaway datadir `/scratch/code/shibboleth/.tmp/recon-hashkinds-datadir`
  deleted.
- Pre-existing `bitcoind` (PID 139471, running since before this session) was
  never touched.
