# The constellation's release workflows, from one generator

`emit.py` writes `.github/workflows/release.yml` into each CLI repo from the
single template in `gen.py`. Four hand-maintained copies of an 80-line build
matrix drift; one generator does not.

```sh
python3 emit.py        # rewrites all four, then YAML-parses each
```

## What differs per repo, and why

| repo | binary | targets | why |
| --- | --- | --- | --- |
| `descriptor-mnemonic` | `md` | all five | shipped no binaries, only man pages |
| `mnemonic-secret` | `ms` | macOS ×2, Windows | already ships **static musl** Linux builds, which are a better Linux artifact than a glibc-linked one |
| `mnemonic-key` | `mk` | macOS ×2, Windows | same — `musl-binaries.yml` already covers Linux |
| `mnemonic-transaction` | `mt` | all five | shipped nothing |

`mnemonic-engrave` keeps its own hand-written workflow: it has a Go sidecar and
generates Go-module `THIRD_PARTY_LICENSES`, neither of which this template
models. It is the ORIGIN of the matrix, not an output of this generator.

## Composability

Every workflow uses `gh release view || gh release create` then
`gh release upload --clobber`, the idiom `man-pages.yml` and `musl-binaries.yml`
already use. Whichever workflow reaches a tag first creates the release; the
rest attach their own assets. The checksum file is named **`SHA256SUMS.portable`**
so it cannot race the per-arch `SHA256SUMS.<arch>` the musl workflow uploads to
the same tag.

## Signed, and a tag cannot ship unsigned

Every generated workflow minisign-signs `SHA256SUMS.portable` with the
constellation key (`MINISIGN_PUBKEY`, pinned once in the workflow's `env:`) and
verifies the signature against it in the same job. A publishing run (a tag
push, or a backfill dispatch with `tag`) FAILS if `MINISIGN_SECRET_KEY` is
missing or signing / verifying fails, before anything uploads. A
`sign_dry_run` dispatch signs and verifies and uploads nothing: that is how to
prove a repo's secrets work without cutting a release.

`emit.py` output matches the committed `release.yml` byte-for-byte in
mnemonic-toolkit, mnemonic-secret, descriptor-mnemonic and mnemonic-key (md's
`--features cli-compiler` and `--from-policy` smoke are now generator inputs).
mnemonic-transaction's committed file predates signing and has no signing
secret; regenerating it there makes its next tag require one.

## Backfilling an existing release

`workflow_dispatch` takes a `tag` input: it builds that tag and uploads to its
release. That is how `mk-cli-v0.13.0` gets macOS and Windows binaries without
inventing a version bump that carries no code change.
