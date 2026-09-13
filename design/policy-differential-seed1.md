# Policy differential run

- **Seed**: `1`  (`--seed 1 --count 200 --indices 3` reproduces this run exactly)
- Generated: **200**   (minted a card: 184)
- Agreed — every available leg identical on every index: **116**
- Disagreed: **50** cases, in **3** distinct divergences
- Documented divergences only: **18** cases, in **1** class
- Refused before a card was minted: **16** (compose 0, encode 16)
- Wall clock: 33.7s; `md` invocations: 1555; Core RPCs: 720

## Legs
- **Rust** `/scratch/code/shibboleth/descriptor-mnemonic/target/debug/md` — mainnet and regtest
- **Bitcoin Core** `/scratch/code/shibboleth/.tmp/bitcoin-31.1/bin` — regtest, datadir `/scratch/code/shibboleth/.tmp/policy-diff-regtest`, RPC port 18988
- **Device** `/scratch/code/shibboleth/.tmp/policyprobe` — mainnet only

### Which comparisons are on which network
The device is mainnet-only; Core here is on a throwaway regtest datadir and will not parse a mainnet `xpub`. So:

| comparison | Rust asked for | other leg answers in | compared as |
| --- | --- | --- | --- |
| Rust vs device | mainnet (`bc1…`) | mainnet (`bc1…`) | normalised scriptPubKey |
| Rust vs Core | regtest (`bcrt1…`) | regtest (`bcrt1…`) | normalised scriptPubKey |
| Rust vs itself | mainnet and regtest | — | normalised scriptPubKey |

`normalize_address` (scripts/policy_keys.py) discards the network and keeps the witness program or script hash, so `bc1q<prog>` and `bcrt1q<prog>` compare equal. The third row is a check in its own right: a network flag must never change the script, and `RUST_NETWORK_VARIANCE` fires if it does.

## Generator bounds
Read at runtime from `/scratch/code/shibboleth/seedhammer/md/compose.go`:

- `ComposeMaxPaths` = 8
- `ComposeMaxKeysPerPath` = 9
- `ComposeMaxSlots` = 32

Varied per case: wrapper (`wsh`, `tr`, `sh-wsh`, `sh`), path count, k-of-n per path, lock kind (none / `older=N` blocks / `older=Nu` units / `after=H` height / `after=Tt` time), `sha256` hashlock presence, `unsorted`, and keyless paths.

Wrapper distribution this run: `sh` 26, `sh-wsh` 21, `tr` 74, `wsh` 79

## Findings

Occurrences are collapsed by **signature** — the same divergence hit by several generated policies is one entry with a count, not N copies. Each entry carries the smallest policy that still reproduces it.

### F1. ACCEPTANCE_MISMATCH — md derived addresses, the device refused

- Occurrences: **26** across 26 generated policies (1-0018, 1-0024, 1-0026, 1-0037, 1-0038, 1-0052, 1-0064, 1-0072, …)
- Signature: `ACCEPTANCE_MISMATCH|rust>device|sh|source|the device declined this policy shape: an unsupported use-site, or its index-<n> derive probe failed`
- **Smallest still-failing policy**: `--wrapper sh --path 1of2`
- Template: `sh(sortedmulti(1,@0/48'/0'/0'/2'/<0;1>/*,@1/48'/0'/1'/2'/<0;1>/*))`

```json
{
  "accepted_by": [
    "rust"
  ],
  "refused_by": [
    "device"
  ],
  "device_refusal": {
    "stage": "source",
    "error": "the device declined this policy shape: an unsupported use-site, or its index-0 derive probe failed"
  }
}
```

Replay:

```sh
/scratch/code/shibboleth/descriptor-mnemonic/target/debug/md compose --wrapper sh --path 1of2 --json
/scratch/code/shibboleth/descriptor-mnemonic/target/debug/md encode "sh(sortedmulti(1,@0/48'/0'/0'/2'/<0;1>/*,@1/48'/0'/1'/2'/<0;1>/*))" --key '@0=xpub6EwnWH978Gvtbq5R4bBupC8nn2A3vPRCCx2t7Bqr63zQh6DaHUFAjGN3hcfFb9wXhWZAgs5NVjSDpiEXh925LE2djvBXmni2jWUeYSQq65P' --fingerprint '@0=4cbd2c68' --key '@1=xpub6FAqvqr18BDqgNbbSGDSNCrwBHGAYUC9CdQmqZWJCTSoU4uz2hm9hLiPi5L5TZSsvWTfBXAY7fhz4TuLq7rQJFY3DBNUfouUb4biFBGr132' --fingerprint '@1=b039be94' --network mainnet --group-size 0
/scratch/code/shibboleth/descriptor-mnemonic/target/debug/md address md1f52f0ps9q2tvyyy5jmpprj5qqcx8qpgtcgn9a935ds8xlfg9wpyjn6qdl7aqyf2nthgt3vlc3hs68rmyj0t8h md1f52f0psw2fsed8c4fskgdn3az72rzy39t8uj0eq9mrfp0ua638xq4s0ksqe2n7w7cuttqwughgd4xdnf2aexw md1f52f0psjdnp8q60h7a7zlay675h6d29ymclzwmrf7v2g9pxe8z0ap027gsralp7ye0mlgnsc2nj705z6xlp2u md1f52f0pscwyh8l9szt4v5r43ugyya3w0te5t5k8zknppah45nvwdmtxugatfqsunae87qqcmy3vmxmfmz7x --network mainnet --count 3
/scratch/code/shibboleth/descriptor-mnemonic/target/debug/md address md1f52f0ps9q2tvyyy5jmpprj5qqcx8qpgtcgn9a935ds8xlfg9wpyjn6qdl7aqyf2nthgt3vlc3hs68rmyj0t8h md1f52f0psw2fsed8c4fskgdn3az72rzy39t8uj0eq9mrfp0ua638xq4s0ksqe2n7w7cuttqwughgd4xdnf2aexw md1f52f0psjdnp8q60h7a7zlay675h6d29ymclzwmrf7v2g9pxe8z0ap027gsralp7ye0mlgnsc2nj705z6xlp2u md1f52f0pscwyh8l9szt4v5r43ugyya3w0te5t5k8zknppah45nvwdmtxugatfqsunae87qqcmy3vmxmfmz7x --network mainnet --count 3 --change
/scratch/code/shibboleth/descriptor-mnemonic/target/debug/md descriptor md1f52f0ps9q2tvyyy5jmpprj5qqcx8qpgtcgn9a935ds8xlfg9wpyjn6qdl7aqyf2nthgt3vlc3hs68rmyj0t8h md1f52f0psw2fsed8c4fskgdn3az72rzy39t8uj0eq9mrfp0ua638xq4s0ksqe2n7w7cuttqwughgd4xdnf2aexw md1f52f0psjdnp8q60h7a7zlay675h6d29ymclzwmrf7v2g9pxe8z0ap027gsralp7ye0mlgnsc2nj705z6xlp2u md1f52f0pscwyh8l9szt4v5r43ugyya3w0te5t5k8zknppah45nvwdmtxugatfqsunae87qqcmy3vmxmfmz7x --network mainnet --chain 0
echo '{"id": "1-0131", "chunks": ["md1f52f0ps9q2tvyyy5jmpprj5qqcx8qpgtcgn9a935ds8xlfg9wpyjn6qdl7aqyf2nthgt3vlc3hs68rmyj0t8h", "md1f52f0psw2fsed8c4fskgdn3az72rzy39t8uj0eq9mrfp0ua638xq4s0ksqe2n7w7cuttqwughgd4xdnf2aexw", "md1f52f0psjdnp8q60h7a7zlay675h6d29ymclzwmrf7v2g9pxe8z0ap027gsralp7ye0mlgnsc2nj705z6xlp2u", "md1f52f0pscwyh8l9szt4v5r43ugyya3w0te5t5k8zknppah45nvwdmtxugatfqsunae87qqcmy3vmxmfmz7x"], "indices": [0, 1, 2]}' | /scratch/code/shibboleth/.tmp/policyprobe
```

### F2. ACCEPTANCE_MISMATCH — md derived addresses, the device refused

- Occurrences: **21** across 21 generated policies (1-0010, 1-0017, 1-0027, 1-0032, 1-0039, 1-0042, 1-0044, 1-0055, …)
- Signature: `ACCEPTANCE_MISMATCH|rust>device|sh-wsh|source|the device declined this policy shape: an unsupported use-site, or its index-<n> derive probe failed`
- **Smallest still-failing policy**: `--wrapper sh-wsh --path 1of2`
- Template: `sh(wsh(sortedmulti(1,@0/48'/0'/0'/1'/<0;1>/*,@1/48'/0'/1'/1'/<0;1>/*)))`

```json
{
  "accepted_by": [
    "rust"
  ],
  "refused_by": [
    "device"
  ],
  "device_refusal": {
    "stage": "source",
    "error": "the device declined this policy shape: an unsupported use-site, or its index-0 derive probe failed"
  }
}
```

Replay:

```sh
/scratch/code/shibboleth/descriptor-mnemonic/target/debug/md compose --wrapper sh-wsh --path 1of2 --json
/scratch/code/shibboleth/descriptor-mnemonic/target/debug/md encode "sh(wsh(sortedmulti(1,@0/48'/0'/0'/1'/<0;1>/*,@1/48'/0'/1'/1'/<0;1>/*)))" --key '@0=xpub6EwnWH978GvtZMzDhP9fumjkHb3JZxUBskzwUFGj8kqogFZibS7HRgPtP1sAEPojZgrwXq7HbAytbKNDJ2uWhwucpptQNWvdr1sfMGTtpjQ' --fingerprint '@0=4cbd2c68' --key '@1=xpub6FAqvqr18BDqds2dSBbmqBxpXsVHdURwJpfohVy8e9tbm86UT6fsnx4fFB9LMzSmsRiBvJonc17VUvCLSbEg8XNHf7tSSiGvRDeADwMGXng' --fingerprint '@1=b039be94' --network mainnet --group-size 0
/scratch/code/shibboleth/descriptor-mnemonic/target/debug/md address md1f2h0lps9q2tvyyyd9kzz8rsqrqcgwqzshs3xt6trgmqwd7js2uzf8fg2dqelcuwq4pkgex6q0t6cmxd3xeny3 md1f2h0lps0q9cjmuq29rncp36dulksgz8qfm8agadqd00cqgv3j5enel5vrwrps5qdk3fqzhjqvmz2usa26e4u3 md1f2h0lpss6kywmgruj24qxdvcyhxf42fqv79qnygnpgrajhpucdzedm5cp2xg9zsswqnakrdqe503xq03nwejz md1f2h0lps7njszcqlq9qsru244a3m2239scr60cuf8un92a7a87hlpdgagexn8rq75km4aqhnwfm9v6mepua --network mainnet --count 3
/scratch/code/shibboleth/descriptor-mnemonic/target/debug/md address md1f2h0lps9q2tvyyyd9kzz8rsqrqcgwqzshs3xt6trgmqwd7js2uzf8fg2dqelcuwq4pkgex6q0t6cmxd3xeny3 md1f2h0lps0q9cjmuq29rncp36dulksgz8qfm8agadqd00cqgv3j5enel5vrwrps5qdk3fqzhjqvmz2usa26e4u3 md1f2h0lpss6kywmgruj24qxdvcyhxf42fqv79qnygnpgrajhpucdzedm5cp2xg9zsswqnakrdqe503xq03nwejz md1f2h0lps7njszcqlq9qsru244a3m2239scr60cuf8un92a7a87hlpdgagexn8rq75km4aqhnwfm9v6mepua --network mainnet --count 3 --change
/scratch/code/shibboleth/descriptor-mnemonic/target/debug/md descriptor md1f2h0lps9q2tvyyyd9kzz8rsqrqcgwqzshs3xt6trgmqwd7js2uzf8fg2dqelcuwq4pkgex6q0t6cmxd3xeny3 md1f2h0lps0q9cjmuq29rncp36dulksgz8qfm8agadqd00cqgv3j5enel5vrwrps5qdk3fqzhjqvmz2usa26e4u3 md1f2h0lpss6kywmgruj24qxdvcyhxf42fqv79qnygnpgrajhpucdzedm5cp2xg9zsswqnakrdqe503xq03nwejz md1f2h0lps7njszcqlq9qsru244a3m2239scr60cuf8un92a7a87hlpdgagexn8rq75km4aqhnwfm9v6mepua --network mainnet --chain 0
echo '{"id": "1-0010-shrink2", "chunks": ["md1f2h0lps9q2tvyyyd9kzz8rsqrqcgwqzshs3xt6trgmqwd7js2uzf8fg2dqelcuwq4pkgex6q0t6cmxd3xeny3", "md1f2h0lps0q9cjmuq29rncp36dulksgz8qfm8agadqd00cqgv3j5enel5vrwrps5qdk3fqzhjqvmz2usa26e4u3", "md1f2h0lpss6kywmgruj24qxdvcyhxf42fqv79qnygnpgrajhpucdzedm5cp2xg9zsswqnakrdqe503xq03nwejz", "md1f2h0lps7njszcqlq9qsru244a3m2239scr60cuf8un92a7a87hlpdgagexn8rq75km4aqhnwfm9v6mepua"], "indices": [0, 1, 2]}' | /scratch/code/shibboleth/.tmp/policyprobe
```

### F3. ACCEPTANCE_MISMATCH — md derived addresses, the device refused

- Occurrences: **3** across 3 generated policies (1-0016, 1-0113, 1-0126)
- Signature: `ACCEPTANCE_MISMATCH|rust>device|tr|source|the device declined this policy shape: an unsupported use-site, or its index-<n> derive probe failed`
- **Smallest still-failing policy**: `--wrapper tr --path 1of1`
- Template: `tr(@0/48'/0'/0'/3'/<0;1>/*)`

```json
{
  "accepted_by": [
    "rust"
  ],
  "refused_by": [
    "device"
  ],
  "device_refusal": {
    "stage": "source",
    "error": "the device declined this policy shape: an unsupported use-site, or its index-0 derive probe failed"
  }
}
```

Replay:

```sh
/scratch/code/shibboleth/descriptor-mnemonic/target/debug/md compose --wrapper tr --path 1of1 --json
/scratch/code/shibboleth/descriptor-mnemonic/target/debug/md encode "tr(@0/48'/0'/0'/3'/<0;1>/*)" --key '@0=xpub6EwnWH978GvtfNmZmVcPrdHDPmHn1r8yRuFf3FqRHmJPsv51Wd6LFHrESXkKh4Abg7eGbstaSTdUvwT4NehakugMqFkxvfBvoD4wBJ4nJHA' --fingerprint '@0=4cbd2c68' --network mainnet --group-size 0
/scratch/code/shibboleth/descriptor-mnemonic/target/debug/md address md1fj3lhpqpqztvyyyhqqxqs95pxt6trgz4q37ffgmp24khg30ttphwqkl9t0s2mag0wd md1fj3lhpq0qwdlhk9v62svv76jc8usu34u9u0cmtzvs87c3mtt9kytq6gufenl3rzxg4 md1fj3lhpq4tgglpd2tv7kmdmw8sdh07ylxswu9guavf7seyufsq4hzrglkk77mpa --network mainnet --count 3
/scratch/code/shibboleth/descriptor-mnemonic/target/debug/md address md1fj3lhpqpqztvyyyhqqxqs95pxt6trgz4q37ffgmp24khg30ttphwqkl9t0s2mag0wd md1fj3lhpq0qwdlhk9v62svv76jc8usu34u9u0cmtzvs87c3mtt9kytq6gufenl3rzxg4 md1fj3lhpq4tgglpd2tv7kmdmw8sdh07ylxswu9guavf7seyufsq4hzrglkk77mpa --network mainnet --count 3 --change
/scratch/code/shibboleth/descriptor-mnemonic/target/debug/md descriptor md1fj3lhpqpqztvyyyhqqxqs95pxt6trgz4q37ffgmp24khg30ttphwqkl9t0s2mag0wd md1fj3lhpq0qwdlhk9v62svv76jc8usu34u9u0cmtzvs87c3mtt9kytq6gufenl3rzxg4 md1fj3lhpq4tgglpd2tv7kmdmw8sdh07ylxswu9guavf7seyufsq4hzrglkk77mpa --network mainnet --chain 0
echo '{"id": "1-0016", "chunks": ["md1fj3lhpqpqztvyyyhqqxqs95pxt6trgz4q37ffgmp24khg30ttphwqkl9t0s2mag0wd", "md1fj3lhpq0qwdlhk9v62svv76jc8usu34u9u0cmtzvs87c3mtt9kytq6gufenl3rzxg4", "md1fj3lhpq4tgglpd2tv7kmdmw8sdh07ylxswu9guavf7seyufsq4hzrglkk77mpa"], "indices": [0, 1, 2]}' | /scratch/code/shibboleth/.tmp/policyprobe
```


## Documented divergences (not defects)

These are differences the implementations are *meant* to have. They are counted and named rather than silently dropped, so that widening the suppression list shows up in a diff.

### ACCEPTANCE_MISMATCH — receive: md derived addresses, Core refused the same descriptor

- Occurrences: **36** across 18 policies
- Smallest reproduction: `--wrapper wsh --path keyless,sha256=9d1a1ab55daae4a604e9121cce37d832b2c4dc344a99c472c2e7c2f3d3543f1a --path 1of1`
- Template: `wsh(or_i(sha256(9d1a1ab55daae4a604e9121cce37d832b2c4dc344a99c472c2e7c2f3d3543f1a),pkh(@0/48'/0'/0'/2'/<0;1>/*)))`

Why this is expected: `md encode --experimental` exists precisely to relax rust-miniscript's *"all spend paths must require a signature"* rule, which its own `--help` calls "a safety policy, not a language rule". Bitcoin Core enforces that same policy in `IsSane` and offers no flag to relax it, so a policy md will only encode under `--experimental` is one Core is expected to refuse. Only refusals carrying `witnesses without signature exist` **and** on a case that needed `--experimental` are classed here; every other Core refusal stays a finding.


## Refusals before a card was minted
These are policies the generator produced inside the composer's stated limits that some stage of the Rust leg declined. A compose refusal means the generator stepped outside a rule `validate()` enforces; an encode refusal usually means rust-miniscript's own resource or timelock rules, which the composer's bounds say nothing about.

| count | stage and message |
| --- | --- |
| 9 | `encode: md: codec error: encoding requires 65 chunks; max is 64 per spec §9.8` |
| 7 | `encode: md: template parse error: miniscript parse failed even with --experimental: Miniscript is malleable (--experimental relaxes ONLY the signature rule; malleability, resource limits, repeated keys and timelock mixing still apply)` |

## Reproducing
```sh
/scratch/code/shibboleth/me-worktrees/policy-harness/scripts/policy-differential.py \
  --seed 1 --count 200 --indices 3 \
  --report <path>
```

Bitcoin Core is started on demand against the scratch datadir and stopped when the run ends. `--no-core` drops that leg for a fast iteration loop; `--keep-core` leaves the node up between runs.

