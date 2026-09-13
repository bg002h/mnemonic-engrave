#!/usr/bin/env python3
"""policy-differential.py -- a three-way differential harness for wallet POLICIES.

The md1 decoder is fuzzed four ways and the codec has property tests on its
primitives.  The POLICY space -- compose a wallet, encode it to md1, decode it
back, seat the keys, derive an address -- has nothing generating cases at all,
and that seam is where the findings have been coming from.  This generates
random policies inside the composer's own limits and pushes each one through
three INDEPENDENT implementations, then compares what they say.

The three legs:

  1. Rust, the primary            -- `md` from descriptor-mnemonic.
  2. The device, Go               -- `policyprobe` from the seedhammer fork.
  3. Bitcoin Core                 -- `getdescriptorinfo` + `deriveaddresses`.

Core is the leg that matters most.  It shares no code with either constellation
half, so it is the only one of the three that can catch the other two agreeing
and both being wrong.

------------------------------------------------------------------------------
THE NETWORK SEAM
------------------------------------------------------------------------------
The device is mainnet-only by design.  Core here runs on a throwaway REGTEST
datadir, and a regtest Core will not parse a mainnet-version `xpub`.  So the
legs are not asked the same question in the same units, and the harness must
not pretend they are:

  * Rust      is asked for MAINNET addresses (bc1...)  -- compared with the device
  * Rust      is also asked for REGTEST addresses (bcrt1...) -- compared with Core
  * Core      is given the same descriptor with its keys re-serialised to tpub
              version bytes, and answers in REGTEST (bcrt1...)

Those are then made comparable by `normalize_address`, which throws the network
away and keeps the scriptPubKey: `bc1q<prog>` and `bcrt1q<prog>` both normalise
to `wit:0:<prog>`.  Comparing normalised forms compares the SCRIPT, which is the
thing that actually has to agree.

That gives a free extra check worth stating: Rust's own mainnet and regtest
answers must normalise identically.  If a network flag ever changes the script,
that is a finding on its own (`RUST_NETWORK_VARIANCE`), and it is checked on
every case rather than assumed.

------------------------------------------------------------------------------
BITCOIN CORE SAFETY
------------------------------------------------------------------------------
`~/.bitcoin` is the operator's live mainnet node.  Every Core invocation here
carries an explicit `-datadir` under /scratch/code/shibboleth/.tmp/, and
`_assert_safe_datadir()` refuses to run at all if that datadir resolves inside
$HOME or outside the scratch tree.  The node is started with `listen=0`,
`maxconnections=0` and `dnsseed=0`: it binds an RPC port nothing else holds,
never connects to a peer and never downloads anything.  `getdescriptorinfo` and
`deriveaddresses` are pure functions of their argument -- no chain state is
needed, so an empty regtest datadir is enough.

Usage
-----
    scripts/policy-differential.py --count 100 --seed 1
    scripts/policy-differential.py --count 100 --seed 1 --no-core   # fast loop
    scripts/policy-differential.py --seed 1 --replay 37             # one case

Run `--help` for the rest.
"""

from __future__ import annotations

import argparse
import json
import os
import random
import re
import shutil
import signal
import subprocess
import sys
import time
from dataclasses import dataclass, replace
from pathlib import Path as FsPath
from typing import Dict, List, Optional, Sequence, Tuple

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from policy_keys import (  # noqa: E402
    SCRIPT_TYPE_BY_WRAPPER,
    VERSION_TPUB,
    VERSION_XPUB,
    b58decode_check,
    b58encode_check,
    derive_account,
    normalize_address,
)

# --------------------------------------------------------------------------
# The composer's own limits.
#
# Read from the fork's md/compose.go at startup rather than hardcoded, so this
# harness cannot silently drift past a limit the composer later tightens -- the
# whole point is to generate at the boundary, and a stale boundary tests nothing.
# --------------------------------------------------------------------------

DEFAULT_COMPOSE_GO = "/scratch/code/shibboleth/seedhammer/md/compose.go"

DEFAULT_MD = "/scratch/code/shibboleth/descriptor-mnemonic/target/debug/md"
DEFAULT_BITCOIN_BIN = "/scratch/code/shibboleth/.tmp/bitcoin-31.1/bin"
DEFAULT_DATADIR = "/scratch/code/shibboleth/.tmp/policy-diff-regtest"
DEFAULT_RPC_PORT = 18988
SCRATCH_ROOT = "/scratch/code/shibboleth/.tmp"

# Candidate locations for the device leg, built by the other implementer.
#
# The FIXED build (fork branch policyprobe-fix, tip b12c32a) comes first on
# purpose. The earlier build exported only complexAddressSource -- the second of
# the two routes gui.policyAddressAt tries -- so it answered "the device declined
# this policy shape" for every single-key and plain-multisig shape, and reported
# a codec failure for single-string keyless cards. Both are F-512. A run against
# that build produces three device divergences that are entirely artefacts of it,
# so the ordering here is what stops a stale binary on the path from silently
# re-manufacturing them.
POLICYPROBE_CANDIDATES = [
    "/scratch/code/shibboleth/.tmp/policyprobe-fixed",
    "/scratch/code/shibboleth/seedhammer/policyprobe",
    "/scratch/code/shibboleth/seedhammer/cmd/policyprobe/policyprobe",
    "/scratch/code/shibboleth/.tmp/policyprobe",
]


@dataclass(frozen=True)
class Limits:
    max_paths: int
    max_keys_per_path: int
    max_slots: int
    source: str


def read_limits(compose_go: str) -> Limits:
    """Parse ComposeMaxPaths / ComposeMaxKeysPerPath / ComposeMaxSlots out of the
    fork's md/compose.go.  Refuses rather than guesses: a wrong bound here makes
    every 'refused' count meaningless."""
    try:
        text = FsPath(compose_go).read_text()
    except OSError as exc:
        raise SystemExit(f"cannot read composer limits from {compose_go}: {exc}")
    found = {}
    for name in ("ComposeMaxPaths", "ComposeMaxKeysPerPath", "ComposeMaxSlots"):
        m = re.search(rf"^\s*{name}\s*=\s*(\d+)\s*$", text, re.M)
        if not m:
            raise SystemExit(f"{compose_go}: could not find {name}")
        found[name] = int(m.group(1))
    return Limits(
        max_paths=found["ComposeMaxPaths"],
        max_keys_per_path=found["ComposeMaxKeysPerPath"],
        max_slots=found["ComposeMaxSlots"],
        source=compose_go,
    )


# --------------------------------------------------------------------------
# The policy model
# --------------------------------------------------------------------------

LOCK_KINDS = ("older_blocks", "older_units", "after_height", "after_time")

# §4c operand ranges, from md/compose.go's Lock.operand().
LOCK_RANGES = {
    "older_blocks": (1, 0xFFFF),
    "older_units": (1, 0xFFFF),
    "after_height": (1, 499_999_999),
    "after_time": (500_000_000, 2_147_483_647),
}


@dataclass(frozen=True)
class SpendPath:
    k: int              # threshold; 0 for a keyless path
    n: int              # key count; 0 for a keyless path
    lock: Optional[Tuple[str, int]] = None
    sha256: Optional[str] = None
    unsorted: bool = False

    @property
    def keyless(self) -> bool:
        return self.n == 0

    def spec(self) -> str:
        """The `--path` argument md compose wants."""
        parts: List[str]
        if self.keyless:
            # grammar: keyless,sha256=HEX[,older=..|after=..]
            assert self.sha256, "a keyless path must carry a hashlock"
            parts = ["keyless", f"sha256={self.sha256}"]
            if self.lock:
                parts.append(_lock_token(self.lock))
        else:
            # grammar: <k>of<n>[,lock][,sha256=HEX][,unsorted]
            parts = [f"{self.k}of{self.n}"]
            if self.lock:
                parts.append(_lock_token(self.lock))
            if self.sha256:
                parts.append(f"sha256={self.sha256}")
            if self.unsorted:
                parts.append("unsorted")
        return ",".join(parts)


def _lock_token(lock: Tuple[str, int]) -> str:
    kind, value = lock
    if kind == "older_blocks":
        return f"older={value}"
    if kind == "older_units":
        return f"older={value}u"
    if kind == "after_height":
        return f"after={value}"
    if kind == "after_time":
        return f"after={value}t"
    raise ValueError(f"unknown lock kind {kind}")


@dataclass(frozen=True)
class Policy:
    wrapper: str
    paths: Tuple[SpendPath, ...]

    @property
    def slots(self) -> int:
        return sum(p.n for p in self.paths)

    @property
    def needs_experimental(self) -> bool:
        return any(p.keyless for p in self.paths) or any(p.unsorted for p in self.paths)

    def compose_args(self, experimental: bool) -> List[str]:
        args = ["compose", "--wrapper", self.wrapper]
        for p in self.paths:
            args += ["--path", p.spec()]
        if experimental:
            args.append("--experimental")
        return args

    def describe(self) -> str:
        return f"--wrapper {self.wrapper} " + " ".join(f"--path {p.spec()}" for p in self.paths)

    def to_json(self) -> dict:
        return {
            "wrapper": self.wrapper,
            "paths": [
                {
                    "k": p.k, "n": p.n,
                    "lock": list(p.lock) if p.lock else None,
                    "sha256": p.sha256,
                    "unsorted": p.unsorted,
                }
                for p in self.paths
            ],
        }


# --------------------------------------------------------------------------
# Generation
# --------------------------------------------------------------------------

LEGACY_WRAPPERS = ("sh", "sh-wsh")


def generate_policy(rng: random.Random, limits: Limits) -> Policy:
    """One random policy, inside the composer's stated bounds.

    Deliberately generated INSIDE validate()'s rules -- including the legacy
    single-sorted-multi shape and the no-keyless-under-tr rule -- so that a
    refusal from `md compose` is a real finding rather than the generator
    walking off a documented edge.  The interesting refusals live one stage
    later, at `md encode`, where rust-miniscript's resource limits and timelock
    rules apply and the composer's own bounds say nothing.
    """
    wrapper = rng.choices(["wsh", "tr", "sh-wsh", "sh"], weights=[42, 34, 12, 12])[0]

    if wrapper in LEGACY_WRAPPERS:
        # ErrComposeLegacyWrapperShape: exactly one bare sorted multi, n >= 2.
        n = rng.randint(2, limits.max_keys_per_path)
        k = rng.randint(1, n)
        return Policy(wrapper, (SpendPath(k=k, n=n),))

    n_paths = rng.randint(1, limits.max_paths)
    budget = limits.max_slots
    paths: List[SpendPath] = []
    keyed_count = 0

    for i in range(n_paths):
        remaining_paths = n_paths - i
        # A keyless path is only expressible outside tr, must carry a hashlock,
        # and cannot be the only path (validate() wants at least one keyed path).
        can_be_keyless = (
            wrapper != "tr"
            and budget >= 1
            and (keyed_count > 0 or remaining_paths > 1)
        )
        keyless = can_be_keyless and rng.random() < 0.12

        if not keyless:
            if budget < 1:
                break
            n = rng.randint(1, min(limits.max_keys_per_path, budget))
            k = rng.randint(1, n)
            budget -= n
            keyed_count += 1
        else:
            n = k = 0

        lock = None
        if rng.random() < 0.55:
            kind = rng.choice(LOCK_KINDS)
            lo, hi = LOCK_RANGES[kind]
            # Half the time, SNAP to a boundary instead of drawing uniformly.
            #
            # A uniform draw over after_height's 1..499,999,999 reaches either
            # end with probability 2e-9, so a run of a thousand policies tests
            # the middle of every range and never an edge -- and the edges are
            # where the bugs are. after=499,999,999 is the largest value that
            # reads as a HEIGHT and after=500,000,000 the smallest that reads as
            # a TIME, one integer apart and meaning entirely different things;
            # older=Nu sets BIP-68's type flag, so older=1u lowers to
            # older(4194305) via a 22-bit shift that an independent
            # reimplementation has every chance of getting wrong.
            #
            # Uniform draws are kept for the other half: a boundary-only
            # generator stops being a fuzzer.
            lock = (kind, rng.choice((lo, hi)) if rng.random() < 0.5
                    else rng.randint(lo, hi))

        sha = None
        if keyless or rng.random() < 0.30:
            sha = bytes(rng.randrange(256) for _ in range(32)).hex()

        unsorted = (not keyless) and n >= 2 and rng.random() < 0.25
        paths.append(SpendPath(k=k, n=n, lock=lock, sha256=sha, unsorted=unsorted))

    if keyed_count == 0:
        # Budget ran out before any keyed path landed; force one.
        n = rng.randint(1, min(limits.max_keys_per_path, max(1, budget)))
        paths.append(SpendPath(k=rng.randint(1, n), n=n))

    return Policy(wrapper, tuple(paths))


# --------------------------------------------------------------------------
# Leg 1: Rust (`md`)
# --------------------------------------------------------------------------

@dataclass
class Refusal:
    stage: str
    text: str

    def to_json(self) -> dict:
        return {"stage": self.stage, "error": self.text}


class Runner:
    def __init__(self, args):
        self.md = os.path.abspath(args.md)
        if not os.path.isfile(self.md) or not os.access(self.md, os.X_OK):
            raise SystemExit(f"md binary not executable: {self.md}")
        self.indices = list(range(args.indices))
        self.timeout = args.timeout
        self.md_calls = 0

    # -- raw invocation ----------------------------------------------------
    def _md(self, argv: Sequence[str]) -> Tuple[int, str, str]:
        self.md_calls += 1
        proc = subprocess.run(
            [self.md, *argv],
            capture_output=True, text=True, timeout=self.timeout,
        )
        return proc.returncode, proc.stdout, proc.stderr

    # -- compose -----------------------------------------------------------
    def compose(self, policy: Policy):
        """-> (composed_json, used_experimental) or Refusal."""
        for experimental in ([False, True] if not policy.needs_experimental else [True]):
            rc, out, err = self._md([*policy.compose_args(experimental), "--json"])
            if rc == 0:
                try:
                    return json.loads(out), experimental
                except json.JSONDecodeError as exc:
                    return Refusal("compose", f"unparseable --json output: {exc}: {out[:200]}")
            # Only retry when md itself says the flag is what is missing.
            if "--experimental" not in err:
                return Refusal("compose", _clean(err))
        return Refusal("compose", _clean(err))

    # -- encode ------------------------------------------------------------
    def encode(self, composed: dict, experimental: bool):
        """-> (list of md1 chunk strings, used_experimental) or Refusal.

        Returns which flag actually worked rather than which one was guessed:
        whether `--experimental` was needed is what separates a Core "not sane"
        refusal that md itself documents from one worth chasing.
        """
        template = composed["template_with_origins"]
        slots = composed["slots"]
        argv = ["encode", template, "--network", "mainnet", "--group-size", "0"]
        for slot in slots:
            i = slot["index"]
            acct = derive_account(i, SCRIPT_TYPE_BY_WRAPPER[composed["wrapper"]])
            argv += ["--key", f"@{i}={acct['xpub']}", "--fingerprint", f"@{i}={acct['fingerprint']}"]
        for exp in ([experimental] if experimental else [False, True]):
            rc, out, err = self._md([*argv, "--experimental"] if exp else argv)
            if rc == 0:
                chunks = [ln.strip() for ln in out.splitlines() if ln.strip().startswith("md1")]
                if not chunks:
                    return Refusal("encode", f"no md1 chunks on stdout: {out[:200]}")
                return chunks, exp
            if "--experimental" not in err:
                return Refusal("encode", _clean(err))
        return Refusal("encode", _clean(err))

    # -- address / descriptor ---------------------------------------------
    def addresses(self, chunks: List[str], network: str):
        """-> {"receive":[...], "change":[...]} or Refusal."""
        out_lists = {}
        for chain, key in ((0, "receive"), (1, "change")):
            rc, out, err = self._md([
                "address", *chunks,
                "--network", network,
                "--chain", str(chain),
                "--index", str(self.indices[0]),
                "--count", str(len(self.indices)),
            ])
            if rc != 0:
                return Refusal(f"address/{network}/chain{chain}", _clean(err))
            addrs = [ln.strip() for ln in out.splitlines() if ln.strip()]
            if len(addrs) != len(self.indices):
                return Refusal(
                    f"address/{network}/chain{chain}",
                    f"expected {len(self.indices)} addresses, got {len(addrs)}",
                )
            out_lists[key] = addrs
        return out_lists

    def descriptor(self, chunks: List[str], network: str, chain: int):
        rc, out, err = self._md([
            "descriptor", *chunks, "--network", network, "--chain", str(chain),
        ])
        if rc != 0:
            return Refusal(f"descriptor/{network}/chain{chain}", _clean(err))
        line = out.strip().splitlines()
        if not line:
            return Refusal(f"descriptor/{network}/chain{chain}", "empty stdout")
        return line[0].strip()


def _clean(err: str) -> str:
    """First meaningful stderr line, minus warnings/notes."""
    for ln in err.splitlines():
        s = ln.strip()
        if s and not s.startswith(("warning:", "note:")):
            return s
    return err.strip().splitlines()[0] if err.strip() else "(no stderr)"


def _clean_rpc(err: str) -> str:
    """bitcoin-cli spreads one error over three lines:

        error code: -5
        error message:
        <the part that actually says what went wrong>

    Taking the first line yields `error code: -5`, which is useless in a report.
    Join the lot and keep the message.
    """
    lines = [ln.strip() for ln in err.splitlines() if ln.strip()]
    if not lines:
        return "(no stderr)"
    if len(lines) >= 3 and lines[1].rstrip(":").lower() == "error message":
        return f"{lines[0]}: {' '.join(lines[2:])}"
    return " ".join(lines)


# --------------------------------------------------------------------------
# Leg 3: Bitcoin Core
# --------------------------------------------------------------------------

XPUB_RE = re.compile(r"\bxpub[1-9A-HJ-NP-Za-km-z]+")


def to_tpub_descriptor(desc: str) -> str:
    """Re-serialise every xpub in a descriptor under testnet version bytes.

    A pure version-byte swap: depth, parent fingerprint, child number, chain code
    and public key are carried through untouched, so the derived scripts are
    identical and only the address encoding differs.  Asserted below rather than
    assumed, because a silent corruption here would read as a Core disagreement.
    """
    body = desc.split("#", 1)[0]

    def conv(m: re.Match) -> str:
        raw = b58decode_check(m.group(0))
        if len(raw) != 78:
            raise ValueError(f"not a 78-byte BIP-32 record: {len(raw)} bytes")
        if int.from_bytes(raw[:4], "big") != VERSION_XPUB:
            raise ValueError(f"expected an xpub version, got {raw[:4].hex()}")
        swapped = VERSION_TPUB.to_bytes(4, "big") + raw[4:]
        return b58encode_check(swapped)

    return XPUB_RE.sub(conv, body)


class CoreLeg:
    def __init__(self, args):
        self.enabled = not args.no_core
        self.proc_pid: Optional[int] = None
        self.datadir = os.path.abspath(args.datadir)
        self.bin_dir = os.path.abspath(args.bitcoin_bin)
        self.rpc_port = args.rpc_port
        self.timeout = args.timeout
        self.rpc_calls = 0
        self.started_here = False
        if not self.enabled:
            return
        _assert_safe_datadir(self.datadir)
        self.cli = os.path.join(self.bin_dir, "bitcoin-cli")
        self.bitcoind = os.path.join(self.bin_dir, "bitcoind")
        for p in (self.cli, self.bitcoind):
            if not os.path.isfile(p):
                raise SystemExit(
                    f"bitcoin binary missing: {p}\n"
                    "Download and verify Core first (see the harness docstring), "
                    "or pass --no-core."
                )

    # -- lifecycle ---------------------------------------------------------
    def _cli_argv(self) -> List[str]:
        return [
            self.cli,
            f"-datadir={self.datadir}",
            f"-conf={os.path.join(self.datadir, 'bitcoin.conf')}",
            f"-rpcport={self.rpc_port}",
            "-rpcuser=policydiff",
            "-rpcpassword=policydiff",
        ]

    def alive(self) -> bool:
        try:
            proc = subprocess.run(
                [*self._cli_argv(), "getblockchaininfo"],
                capture_output=True, text=True, timeout=self.timeout,
            )
            return proc.returncode == 0
        except (subprocess.TimeoutExpired, OSError):
            return False

    def start(self) -> None:
        if not self.enabled or self.alive():
            return
        _assert_safe_datadir(self.datadir)
        os.makedirs(self.datadir, exist_ok=True)
        conf = os.path.join(self.datadir, "bitcoin.conf")
        FsPath(conf).write_text(
            "regtest=1\nserver=1\nlisten=0\ndnsseed=0\nupnp=0\nnatpmp=0\n"
            "maxconnections=0\nprinttoconsole=0\n"
            "rpcuser=policydiff\nrpcpassword=policydiff\n"
            "[regtest]\nrpcbind=127.0.0.1\n"
            f"rpcport={self.rpc_port}\nrpcallowip=127.0.0.1\n"
        )
        # start_new_session detaches it from this process group, so it is not
        # reaped when the tool call that spawned the harness ends.
        proc = subprocess.Popen(
            [self.bitcoind, f"-datadir={self.datadir}", f"-conf={conf}"],
            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
            stdin=subprocess.DEVNULL, start_new_session=True,
        )
        self.proc_pid = proc.pid
        self.started_here = True
        deadline = time.time() + 60
        while time.time() < deadline:
            if self.alive():
                return
            time.sleep(0.4)
        raise SystemExit(f"bitcoind did not come up on port {self.rpc_port} within 60s")

    def stop(self) -> None:
        """Stop only a node THIS run started; never one that was already up."""
        if not (self.enabled and self.started_here):
            return
        try:
            subprocess.run([*self._cli_argv(), "stop"],
                           capture_output=True, text=True, timeout=self.timeout)
        except (subprocess.TimeoutExpired, OSError):
            pass
        if self.proc_pid:
            deadline = time.time() + 20
            while time.time() < deadline:
                try:
                    os.kill(self.proc_pid, 0)
                except OSError:
                    return
                time.sleep(0.3)
            try:
                os.kill(self.proc_pid, signal.SIGTERM)
            except OSError:
                pass

    # -- rpc ---------------------------------------------------------------
    def _rpc(self, *argv: str) -> Tuple[int, str, str]:
        self.rpc_calls += 1
        proc = subprocess.run([*self._cli_argv(), *argv],
                              capture_output=True, text=True, timeout=self.timeout)
        return proc.returncode, proc.stdout, proc.stderr

    def derive(self, mainnet_descriptor: str, count: int, start: int):
        """-> list of regtest addresses, or Refusal."""
        if not self.enabled:
            return None
        try:
            tdesc = to_tpub_descriptor(mainnet_descriptor)
        except (ValueError, AssertionError) as exc:
            return Refusal("core/convert", f"tpub conversion failed: {exc}")
        rc, out, err = self._rpc("getdescriptorinfo", tdesc)
        if rc != 0:
            return Refusal("core/getdescriptorinfo", _clean_rpc(err))
        try:
            checksummed = json.loads(out)["descriptor"]
        except (json.JSONDecodeError, KeyError) as exc:
            return Refusal("core/getdescriptorinfo", f"unparseable: {exc}")
        rng_arg = json.dumps([start, start + count - 1])
        rc, out, err = self._rpc("deriveaddresses", checksummed, rng_arg)
        if rc != 0:
            return Refusal("core/deriveaddresses", _clean_rpc(err))
        try:
            addrs = json.loads(out)
        except json.JSONDecodeError as exc:
            return Refusal("core/deriveaddresses", f"unparseable: {exc}")
        if not isinstance(addrs, list) or len(addrs) != count:
            return Refusal("core/deriveaddresses",
                           f"expected {count} addresses, got {addrs!r:.120}")
        return addrs


def _assert_safe_datadir(datadir: str) -> None:
    """The operator's live mainnet node lives in ~/.bitcoin on a CoW filesystem.
    Nothing here may ever address it.  A `-datadir` flag is what keeps us out,
    so this refuses any datadir that is not squarely inside the scratch tree."""
    real = os.path.realpath(datadir)
    home = os.path.realpath(os.path.expanduser("~"))
    root = os.path.realpath(SCRATCH_ROOT)
    if not (real == root or real.startswith(root + os.sep)):
        raise SystemExit(
            f"REFUSING: Core datadir {real} is not under {root}. "
            "This harness only ever runs against a throwaway scratch datadir."
        )
    if real == home or real.startswith(home + os.sep):
        raise SystemExit(f"REFUSING: Core datadir {real} is inside $HOME ({home}).")
    if os.path.basename(real) == ".bitcoin":
        raise SystemExit(f"REFUSING: Core datadir {real} is named .bitcoin.")


# --------------------------------------------------------------------------
# Leg 2: the device (`policyprobe`)
# --------------------------------------------------------------------------

class DeviceLeg:
    """JSONL in, JSONL out, one line per case.

    Request:  {"id", "chunks":[...], "indices":[...]}
    Response: {"id","ok":true,"keys",N,"receive":[...],"change":[...]}
          or  {"id","ok":false,"stage":"expand|source|derive","error":"..."}

    Mainnet-only by design, so its answers are compared against Rust's MAINNET
    answers.  Absent binary is not an error: the other two legs still run and
    the report says the device leg was unavailable.
    """

    STAGES = ("expand", "source", "derive")

    def __init__(self, args):
        self.path = self._locate(args.policyprobe)
        self.available = self.path is not None
        self.timeout = args.timeout
        self.batches = 0
        self.reason = None if self.available else (
            f"policyprobe not found (looked at: {args.policyprobe or ', '.join(POLICYPROBE_CANDIDATES)})"
        )

    @staticmethod
    def _locate(explicit: Optional[str]) -> Optional[str]:
        if explicit:
            p = os.path.abspath(explicit)
            return p if os.path.isfile(p) and os.access(p, os.X_OK) else None
        which = shutil.which("policyprobe")
        if which:
            return which
        for cand in POLICYPROBE_CANDIDATES:
            if os.path.isfile(cand) and os.access(cand, os.X_OK):
                return cand
        return None

    def run(self, requests: List[dict]) -> Dict[str, dict]:
        """-> {id: response}.  Missing ids simply do not appear."""
        if not self.available or not requests:
            return {}
        self.batches += 1
        payload = "\n".join(json.dumps(r) for r in requests) + "\n"
        try:
            proc = subprocess.run(
                [self.path], input=payload, capture_output=True, text=True,
                timeout=max(self.timeout, 10 * len(requests)),
            )
        except (subprocess.TimeoutExpired, OSError) as exc:
            return {r["id"]: {"id": r["id"], "ok": False, "stage": "harness",
                              "error": f"policyprobe invocation failed: {exc}"}
                    for r in requests}
        out: Dict[str, dict] = {}
        for ln in proc.stdout.splitlines():
            ln = ln.strip()
            if not ln:
                continue
            try:
                obj = json.loads(ln)
            except json.JSONDecodeError:
                continue
            if isinstance(obj, dict) and "id" in obj:
                out[str(obj["id"])] = obj
        return out


# --------------------------------------------------------------------------
# Comparison
# --------------------------------------------------------------------------

@dataclass
class Finding:
    kind: str
    detail: str
    data: dict
    signature: str = ""      # what makes two occurrences the SAME divergence
    expected: bool = False   # a documented, deliberate difference between legs

    def __post_init__(self):
        if not self.signature:
            self.signature = f"{self.kind}|{self.detail}"


# Divergences that are DOCUMENTED rather than defects.  Kept narrow on purpose,
# and never silently dropped -- they are counted and named in their own section
# of the report, so widening this table is visible in a diff.
#
# `md encode --experimental`'s own help states the first one outright:
#   "rust-miniscript refuses these by default with 'All spend paths must require
#    a signature' -- a safety policy, not a language rule."
# Core enforces that same safety policy and has no flag to relax it, so a policy
# md would only encode under --experimental is one Core is expected to call
# insane.  Suppressing this is what keeps a real Core disagreement visible; the
# 30-case pilot run had 15 of these and nothing else.
EXPECTED_CORE_SANITY = "witnesses without signature exist"


def _is_expected_core_refusal(refusal: "Refusal", used_experimental: bool) -> bool:
    return (
        used_experimental
        and refusal.stage.startswith("core/")
        and EXPECTED_CORE_SANITY in refusal.text
    )


def _normalize_signature_text(text: str) -> str:
    """Strip the operands out of a refusal so two instances of the same refusal
    share a signature.  Hex blobs, xpubs, numbers and quoted operands all vary
    per case and none of them changes which divergence this is."""
    t = re.sub(r"\b(?:xpub|tpub)[1-9A-HJ-NP-Za-km-z]+", "<key>", text)
    t = re.sub(r"\b[0-9a-f]{16,}\b", "<hex>", t)
    t = re.sub(r"\b\d+\b", "<n>", t)
    t = re.sub(r"\s+", " ", t).strip()
    # Core quotes the whole descriptor back before saying what is wrong with it
    # ("<descriptor> is not sane: <reason>").  The descriptor differs on every
    # case and the reason is the divergence, so eliding it is what collapses one
    # class into one entry instead of one per generated policy.
    t = re.sub(r"(?:[A-Za-z_]+\(.*?\))(?= is not sane)", "<descriptor>", t)
    t = re.sub(r"[A-Za-z_]{2,}\([^\s]{24,}", "<descriptor>", t)
    return t[:200]


def _normalize_list(addrs: Sequence[str]) -> Tuple[List[str], Optional[str]]:
    try:
        return [normalize_address(a) for a in addrs], None
    except ValueError as exc:
        return [], str(exc)


def compare(case_id: str,
            policy: Policy,
            composed: dict,
            chunks: List[str],
            rust_main,
            rust_regtest,
            core_receive,
            core_change,
            device,
            device_available: bool,
            experimental: bool = False) -> List[Finding]:
    """Every disagreement between the legs, as a list of findings."""
    findings: List[Finding] = []

    # -- Rust internal: the network flag must not change the script -------
    if isinstance(rust_main, dict) and isinstance(rust_regtest, dict):
        for chain in ("receive", "change"):
            nm, e1 = _normalize_list(rust_main[chain])
            nr, e2 = _normalize_list(rust_regtest[chain])
            if e1 or e2:
                findings.append(Finding(
                    "UNPARSEABLE_ADDRESS",
                    f"{chain}: could not normalise a Rust address: {e1 or e2}",
                    {"mainnet": rust_main[chain], "regtest": rust_regtest[chain]},
                ))
            elif nm != nr:
                findings.append(Finding(
                    "RUST_NETWORK_VARIANCE",
                    f"{chain}: md's mainnet and regtest answers are not the same script",
                    {"mainnet": rust_main[chain], "regtest": rust_regtest[chain]},
                ))
    elif isinstance(rust_main, Refusal) != isinstance(rust_regtest, Refusal):
        findings.append(Finding(
            "RUST_NETWORK_ACCEPTANCE",
            "md accepted on one network and refused on the other",
            {"mainnet": _leg_json(rust_main), "regtest": _leg_json(rust_regtest)},
        ))

    rust_ok = isinstance(rust_main, dict)
    rust_norm: Dict[str, List[str]] = {}
    if rust_ok:
        for chain in ("receive", "change"):
            n, err = _normalize_list(rust_main[chain])
            if err is None:
                rust_norm[chain] = n

    # -- Rust vs Core (mainnet script identity vs regtest script identity) -
    if core_receive is not None:
        core_by_chain = {"receive": core_receive, "change": core_change}
        for chain in ("receive", "change"):
            core_ans = core_by_chain[chain]
            if isinstance(core_ans, Refusal):
                if rust_ok:
                    findings.append(Finding(
                        "ACCEPTANCE_MISMATCH",
                        f"{chain}: md derived addresses, Core refused the same descriptor",
                        {"accepted_by": ["rust"], "refused_by": ["core"],
                         "used_experimental": experimental,
                         "core_refusal": core_ans.to_json()},
                        signature="ACCEPTANCE_MISMATCH|rust>core|"
                                  + _normalize_signature_text(core_ans.text),
                        expected=_is_expected_core_refusal(core_ans, experimental),
                    ))
                continue
            if not rust_ok:
                findings.append(Finding(
                    "ACCEPTANCE_MISMATCH",
                    f"{chain}: Core derived addresses, md refused",
                    {"accepted_by": ["core"], "refused_by": ["rust"],
                     "rust_refusal": _leg_json(rust_main)},
                    signature="ACCEPTANCE_MISMATCH|core>rust|"
                              + _normalize_signature_text(str(_leg_json(rust_main))),
                ))
                continue
            cn, err = _normalize_list(core_ans)
            if err:
                findings.append(Finding("UNPARSEABLE_ADDRESS",
                                        f"{chain}: could not normalise a Core address: {err}",
                                        {"core": core_ans}))
            elif chain in rust_norm and cn != rust_norm[chain]:
                findings.append(Finding(
                    "ADDRESS_MISMATCH",
                    f"{chain}: md and Bitcoin Core derive different scripts",
                    {"rust_mainnet": rust_main[chain], "core_regtest": core_ans,
                     "rust_normalized": rust_norm[chain], "core_normalized": cn},
                    signature="ADDRESS_MISMATCH|rust-vs-core",
                ))

    # -- Rust vs the device (both mainnet) --------------------------------
    if device_available:
        if device is None:
            findings.append(Finding(
                "DEVICE_SILENT",
                "policyprobe returned no line for this case id",
                {"id": case_id},
            ))
        elif not device.get("ok"):
            stage = device.get("stage")
            if stage not in DeviceLeg.STAGES and stage != "harness":
                findings.append(Finding(
                    "DEVICE_CONTRACT",
                    f"refusal stage {stage!r} is outside the contract "
                    f"({'|'.join(DeviceLeg.STAGES)})",
                    {"device": device},
                ))
            if rust_ok:
                findings.append(Finding(
                    "ACCEPTANCE_MISMATCH",
                    "md derived addresses, the device refused",
                    {"accepted_by": ["rust"], "refused_by": ["device"],
                     "device_refusal": {"stage": device.get("stage"),
                                        "error": device.get("error")}},
                    # The wrapper is part of the signature because the device
                    # answers three structurally different refusals -- bare sh,
                    # sh(wsh(..)) and key-path-only tr -- with one identical
                    # message. Without it they collapse into a single group and
                    # the report shows a reproduction for only one of them.
                    signature="ACCEPTANCE_MISMATCH|rust>device|"
                              + policy.wrapper + "|"
                              + str(device.get("stage")) + "|"
                              + _normalize_signature_text(str(device.get("error"))),
                ))
        else:
            if not rust_ok:
                findings.append(Finding(
                    "ACCEPTANCE_MISMATCH",
                    "the device derived addresses, md refused",
                    {"accepted_by": ["device"], "refused_by": ["rust"],
                     "rust_refusal": _leg_json(rust_main)},
                    signature="ACCEPTANCE_MISMATCH|device>rust|"
                              + _normalize_signature_text(str(_leg_json(rust_main))),
                ))
            else:
                declared = device.get("keys")
                expected_slots = len(composed.get("slots", []))
                if isinstance(declared, int) and declared != expected_slots:
                    findings.append(Finding(
                        "SLOT_COUNT_MISMATCH",
                        f"device reports {declared} keys, compose declared {expected_slots} slots",
                        {"device_keys": declared, "compose_slots": expected_slots},
                    ))
                for chain in ("receive", "change"):
                    dev_addrs = device.get(chain)
                    if not isinstance(dev_addrs, list):
                        findings.append(Finding(
                            "DEVICE_CONTRACT",
                            f"response has no {chain!r} address list",
                            {"device": device},
                        ))
                        continue
                    if len(dev_addrs) != len(rust_main[chain]):
                        findings.append(Finding(
                            "DEVICE_CONTRACT",
                            f"{chain}: device returned {len(dev_addrs)} addresses, "
                            f"asked for {len(rust_main[chain])}",
                            {"device": dev_addrs, "rust": rust_main[chain]},
                            signature="DEVICE_CONTRACT|address-count",
                        ))
                        continue
                    dn, err = _normalize_list(dev_addrs)
                    if err:
                        findings.append(Finding("UNPARSEABLE_ADDRESS",
                                                f"{chain}: could not normalise a device address: {err}",
                                                {"device": dev_addrs}))
                    elif chain in rust_norm and dn != rust_norm[chain]:
                        findings.append(Finding(
                            "ADDRESS_MISMATCH",
                            f"{chain}: md and the device derive different scripts",
                            {"rust_mainnet": rust_main[chain], "device_mainnet": dev_addrs},
                            signature="ADDRESS_MISMATCH|rust-vs-device",
                        ))
    return findings


def _leg_json(leg):
    if isinstance(leg, Refusal):
        return leg.to_json()
    return leg


# --------------------------------------------------------------------------
# One case, end to end
# --------------------------------------------------------------------------

@dataclass
class CaseResult:
    case_id: str
    policy: Policy
    stage: str                      # where it stopped: ok | compose | encode | address
    refusal: Optional[Refusal]
    findings: List[Finding]
    composed: Optional[dict] = None
    chunks: Optional[List[str]] = None
    experimental: bool = False
    rust_main = None
    rust_regtest = None
    core_receive = None
    core_change = None
    device = None


class Harness:
    def __init__(self, args, limits: Limits):
        self.args = args
        self.limits = limits
        self.runner = Runner(args)
        self.core = CoreLeg(args)
        self.device = DeviceLeg(args)
        self.indices = list(range(args.indices))

    def prepare(self, case_id: str, policy: Policy) -> CaseResult:
        """Everything up to (not including) the device leg, which is batched."""
        composed = self.runner.compose(policy)
        if isinstance(composed, Refusal):
            return CaseResult(case_id, policy, "compose", composed, [])
        composed, experimental = composed

        encoded = self.runner.encode(composed, experimental)
        if isinstance(encoded, Refusal):
            return CaseResult(case_id, policy, "encode", encoded, [],
                              composed=composed, experimental=experimental)
        chunks, experimental = encoded

        res = CaseResult(case_id, policy, "ok", None, [],
                         composed=composed, chunks=chunks, experimental=experimental)
        res.rust_main = self.runner.addresses(chunks, "mainnet")
        res.rust_regtest = self.runner.addresses(chunks, "regtest")

        res.core_receive = res.core_change = None
        if self.core.enabled:
            for chain, attr in ((0, "core_receive"), (1, "core_change")):
                desc = self.runner.descriptor(chunks, "mainnet", chain)
                if isinstance(desc, Refusal):
                    setattr(res, attr, desc)
                else:
                    setattr(res, attr, self.core.derive(desc, len(self.indices), self.indices[0]))
        return res

    def finish(self, res: CaseResult, device_response: Optional[dict]) -> CaseResult:
        if res.stage != "ok":
            return res
        res.device = device_response
        res.findings = compare(
            res.case_id, res.policy, res.composed, res.chunks,
            res.rust_main, res.rust_regtest,
            res.core_receive, res.core_change,
            device_response, self.device.available,
            res.experimental,
        )
        return res

    def run_one(self, case_id: str, policy: Policy) -> CaseResult:
        """A single case including its device call.  Used by the shrinker, where
        batching buys nothing and a fresh answer per candidate is what is wanted."""
        res = self.prepare(case_id, policy)
        device_response = None
        if res.stage == "ok" and self.device.available:
            got = self.device.run([{"id": case_id, "chunks": res.chunks,
                                    "indices": self.indices}])
            device_response = got.get(case_id)
        return self.finish(res, device_response)


# --------------------------------------------------------------------------
# Shrinking
# --------------------------------------------------------------------------

def shrink(harness: Harness, case: CaseResult, signature: Optional[str] = None,
           max_steps: int = 400) -> CaseResult:
    """Reduce a failing policy until it stops failing, and return the smallest
    still-failing one.

    Order is the brief's: path count, then fragments, then key counts.  Paths
    first because dropping one removes the most policy at once; fragments next
    because a lock or a hashlock is usually the actual subject; key counts last
    because they shrink the reproduction's length without changing its shape.

    A candidate counts as still-failing only if it reproduces the SAME finding
    SIGNATURE -- not merely 'some finding'.  Shrinking on any-finding-at-all lets
    the search wander onto a different divergence and report a reproduction for
    something nobody was chasing; a signature pins it to the one under study.
    """
    targets = {signature} if signature else {f.signature for f in case.findings}
    if not targets:
        return case

    best = case
    steps = 0

    def still_fails(policy: Policy) -> Optional[CaseResult]:
        nonlocal steps
        if steps >= max_steps:
            return None
        steps += 1
        cand = harness.run_one(f"{case.case_id}-shrink{steps}", policy)
        if cand.stage != "ok":
            return None
        if targets & {f.signature for f in cand.findings}:
            return cand
        return None

    progress = True
    while progress and steps < max_steps:
        progress = False
        p = best.policy

        # 1. drop a whole spend path
        if len(p.paths) > 1:
            for i in range(len(p.paths)):
                trial = list(p.paths)
                del trial[i]
                if not any(not q.keyless for q in trial):
                    continue  # validate() wants at least one keyed path
                cand = still_fails(replace(p, paths=tuple(trial)))
                if cand:
                    best, progress = cand, True
                    break
            if progress:
                continue

        # 2. drop fragments: the hashlock, then the timelock, then unsorted
        for i, path in enumerate(p.paths):
            for mutate in (
                lambda q: replace(q, sha256=None) if (q.sha256 and not q.keyless) else None,
                lambda q: replace(q, lock=None) if q.lock else None,
                lambda q: replace(q, unsorted=False) if q.unsorted else None,
            ):
                newp = mutate(path)
                if newp is None:
                    continue
                trial = list(p.paths)
                trial[i] = newp
                cand = still_fails(replace(p, paths=tuple(trial)))
                if cand:
                    best, progress = cand, True
                    break
            if progress:
                break
        if progress:
            continue

        # 3. reduce key counts: n first, then k
        for i, path in enumerate(p.paths):
            if path.keyless:
                continue
            for newn in range(1, path.n):
                trial = list(p.paths)
                trial[i] = replace(path, n=newn, k=min(path.k, newn))
                cand = still_fails(replace(p, paths=tuple(trial)))
                if cand:
                    best, progress = cand, True
                    break
            if progress:
                break
            for newk in range(1, path.k):
                trial = list(p.paths)
                trial[i] = replace(path, k=newk)
                cand = still_fails(replace(p, paths=tuple(trial)))
                if cand:
                    best, progress = cand, True
                    break
            if progress:
                break

    return best


# --------------------------------------------------------------------------
# Reporting
# --------------------------------------------------------------------------

def group_findings(cases: List[CaseResult]) -> Dict[str, dict]:
    """Collapse every finding across every case into one entry per SIGNATURE.

    A differential harness at 100+ cases produces the same divergence over and
    over -- one per policy that happens to hit it.  The count is worth knowing;
    a hundred copies of the text is not.  Ordered so unexpected divergences come
    before documented ones, and commoner ones before rarer ones.
    """
    groups: Dict[str, dict] = {}
    for c in cases:
        for f in c.findings:
            g = groups.setdefault(f.signature, {
                "kind": f.kind,
                "detail": f.detail,
                "expected": f.expected,
                "count": 0,
                "cases": [],
                "example": f,
            })
            g["count"] += 1
            if c not in g["cases"]:
                g["cases"].append(c)
            # One unexpected occurrence makes the whole class unexpected.
            g["expected"] = g["expected"] and f.expected
    return dict(sorted(groups.items(),
                       key=lambda kv: (kv[1]["expected"], -kv[1]["count"])))


def replay_commands(harness: Harness, case: CaseResult) -> List[str]:
    """The exact commands that reproduce this case by hand."""
    md = harness.runner.md
    cmds = [f"{md} {' '.join(case.policy.compose_args(case.experimental))} --json"]
    if case.composed:
        tmpl = case.composed["template_with_origins"]
        keyargs = []
        for slot in case.composed["slots"]:
            i = slot["index"]
            acct = derive_account(i, SCRIPT_TYPE_BY_WRAPPER[case.composed["wrapper"]])
            keyargs.append(f"--key '@{i}={acct['xpub']}' --fingerprint '@{i}={acct['fingerprint']}'")
        cmds.append(
            f"{md} encode \"{tmpl}\" {' '.join(keyargs)} "
            f"--network mainnet --group-size 0"
            + (" --experimental" if case.experimental else "")
        )
    if case.chunks:
        joined = " ".join(case.chunks)
        cmds.append(f"{md} address {joined} --network mainnet --count {len(harness.indices)}")
        cmds.append(f"{md} address {joined} --network mainnet --count {len(harness.indices)} --change")
        cmds.append(f"{md} descriptor {joined} --network mainnet --chain 0")
        if harness.device.available:
            req = json.dumps({"id": case.case_id, "chunks": case.chunks,
                              "indices": harness.indices})
            cmds.append(f"echo '{req}' | {harness.device.path}")
    return cmds


def write_report(path: str, args, limits: Limits, harness: Harness,
                 cases: List[CaseResult], shrunk: Dict[str, CaseResult],
                 elapsed: float) -> None:
    generated = len(cases)
    groups = group_findings(cases)
    unexpected = {k: v for k, v in groups.items() if not v["expected"]}
    documented = {k: v for k, v in groups.items() if v["expected"]}

    def _has_unexpected(c: CaseResult) -> bool:
        return any(f.signature in unexpected for f in c.findings)

    minted = [c for c in cases if c.stage == "ok"]
    agreed = [c for c in minted if not c.findings]
    disagreed = [c for c in minted if _has_unexpected(c)]
    only_documented = [c for c in minted if c.findings and not _has_unexpected(c)]
    refused_compose = [c for c in cases if c.stage == "compose"]
    refused_encode = [c for c in cases if c.stage == "encode"]

    refusal_tally: Dict[str, int] = {}
    for c in refused_compose + refused_encode:
        key = f"{c.stage}: {c.refusal.text}"
        refusal_tally[key] = refusal_tally.get(key, 0) + 1

    wrapper_tally: Dict[str, int] = {}
    for c in cases:
        wrapper_tally[c.policy.wrapper] = wrapper_tally.get(c.policy.wrapper, 0) + 1

    out: List[str] = []
    w = out.append
    w("# Policy differential run\n")
    w(f"- **Seed**: `{args.seed}`  (`--seed {args.seed} --count {args.count} "
      f"--indices {args.indices}` reproduces this run exactly)")
    w(f"- Generated: **{generated}**   (minted a card: {len(minted)})")
    w(f"- Agreed — every available leg identical on every index: **{len(agreed)}**")
    w(f"- Disagreed: **{len(disagreed)}** cases, in **{len(unexpected)}** "
      f"distinct divergence{'' if len(unexpected) == 1 else 's'}")
    w(f"- Documented divergences only: **{len(only_documented)}** cases, in "
      f"**{len(documented)}** class{'' if len(documented) == 1 else 'es'}")
    w(f"- Refused before a card was minted: **{len(refused_compose) + len(refused_encode)}** "
      f"(compose {len(refused_compose)}, encode {len(refused_encode)})")
    w(f"- Wall clock: {elapsed:.1f}s; `md` invocations: {harness.runner.md_calls}; "
      f"Core RPCs: {harness.core.rpc_calls}")
    w("")
    w("## Legs")
    w(f"- **Rust** `{harness.runner.md}` — mainnet and regtest")
    if harness.core.enabled:
        w(f"- **Bitcoin Core** `{harness.core.bin_dir}` — regtest, datadir "
          f"`{harness.core.datadir}`, RPC port {harness.core.rpc_port}")
    else:
        w("- **Bitcoin Core** — disabled (`--no-core`)")
    if harness.device.available:
        w(f"- **Device** `{harness.device.path}` — mainnet only")
    else:
        w(f"- **Device** — UNAVAILABLE: {harness.device.reason}. "
          "Every other leg still ran; the device column of this run is empty.")
    w("")
    w("### Which comparisons are on which network")
    w("The device is mainnet-only; Core here is on a throwaway regtest datadir and "
      "will not parse a mainnet `xpub`. So:")
    w("")
    w("| comparison | Rust asked for | other leg answers in | compared as |")
    w("| --- | --- | --- | --- |")
    w("| Rust vs device | mainnet (`bc1…`) | mainnet (`bc1…`) | normalised scriptPubKey |")
    w("| Rust vs Core | regtest (`bcrt1…`) | regtest (`bcrt1…`) | normalised scriptPubKey |")
    w("| Rust vs itself | mainnet and regtest | — | normalised scriptPubKey |")
    w("")
    w("`normalize_address` (scripts/policy_keys.py) discards the network and keeps the "
      "witness program or script hash, so `bc1q<prog>` and `bcrt1q<prog>` compare equal. "
      "The third row is a check in its own right: a network flag must never change the "
      "script, and `RUST_NETWORK_VARIANCE` fires if it does.")
    w("")
    w("## Generator bounds")
    w(f"Read at runtime from `{limits.source}`:")
    w("")
    w(f"- `ComposeMaxPaths` = {limits.max_paths}")
    w(f"- `ComposeMaxKeysPerPath` = {limits.max_keys_per_path}")
    w(f"- `ComposeMaxSlots` = {limits.max_slots}")
    w("")
    w("Varied per case: wrapper (`wsh`, `tr`, `sh-wsh`, `sh`), path count, k-of-n per "
      "path, lock kind (none / `older=N` blocks / `older=Nu` units / `after=H` height / "
      "`after=Tt` time), `sha256` hashlock presence, `unsorted`, and keyless paths.")
    w("")
    w("Wrapper distribution this run: "
      + ", ".join(f"`{k}` {v}" for k, v in sorted(wrapper_tally.items())))
    w("")

    w("## Findings")
    w("")
    w("Occurrences are collapsed by **signature** — the same divergence hit by "
      "several generated policies is one entry with a count, not N copies. Each "
      "entry carries the smallest policy that still reproduces it.")
    w("")
    if not unexpected:
        w("**No unexpected divergence.** Every case that minted a card produced "
          "identical scripts on every available leg, for all requested receive and "
          "change indices, except for the documented classes listed below.")
    else:
        for i, (sig, grp) in enumerate(unexpected.items(), 1):
            small = shrunk.get(sig)
            example = small or grp["cases"][0]
            w(f"### F{i}. {grp['kind']} — {grp['detail']}")
            w("")
            w(f"- Occurrences: **{grp['count']}** across {len(grp['cases'])} "
              f"generated polic{'y' if len(grp['cases']) == 1 else 'ies'} "
              f"({', '.join(c.case_id for c in grp['cases'][:8])}"
              f"{', …' if len(grp['cases']) > 8 else ''})")
            w(f"- Signature: `{sig}`")
            w(f"- **Smallest still-failing policy**: `{example.policy.describe()}`"
              + ("" if small else "  *(not shrunk — `--no-shrink`)*"))
            if example.composed:
                w(f"- Template: `{example.composed['template_with_origins']}`")
            w("")
            shown = [f for f in example.findings if f.signature == sig] or example.findings[:1]
            for f in shown[:1]:
                w("```json")
                w(json.dumps(f.data, indent=2)[:3000])
                w("```")
            w("")
            w("Replay:")
            w("")
            w("```sh")
            for cmd in replay_commands(harness, example):
                w(cmd)
            w("```")
            w("")
    w("")

    w("## Documented divergences (not defects)")
    w("")
    if not documented:
        w("None encountered this run.")
    else:
        w("These are differences the implementations are *meant* to have. They are "
          "counted and named rather than silently dropped, so that widening the "
          "suppression list shows up in a diff.")
        w("")
        for sig, grp in documented.items():
            small = shrunk.get(sig)
            example = small or grp["cases"][0]
            w(f"### {grp['kind']} — {grp['detail']}")
            w("")
            w(f"- Occurrences: **{grp['count']}** across {len(grp['cases'])} policies")
            w(f"- Smallest reproduction: `{example.policy.describe()}`")
            if example.composed:
                w(f"- Template: `{example.composed['template_with_origins']}`")
            w("")
            w("Why this is expected: `md encode --experimental` exists precisely to "
              "relax rust-miniscript's *\"all spend paths must require a signature\"* "
              "rule, which its own `--help` calls \"a safety policy, not a language "
              "rule\". Bitcoin Core enforces that same policy in `IsSane` and offers no "
              "flag to relax it, so a policy md will only encode under `--experimental` "
              "is one Core is expected to refuse. Only refusals carrying "
              f"`{EXPECTED_CORE_SANITY}` **and** on a case that needed `--experimental` "
              "are classed here; every other Core refusal stays a finding.")
            w("")
    w("")

    w("## Refusals before a card was minted")
    if not refusal_tally:
        w("None — every generated policy composed and encoded.")
    else:
        w("These are policies the generator produced inside the composer's stated "
          "limits that some stage of the Rust leg declined. A compose refusal means the "
          "generator stepped outside a rule `validate()` enforces; an encode refusal "
          "usually means rust-miniscript's own resource or timelock rules, which the "
          "composer's bounds say nothing about.")
        w("")
        w("| count | stage and message |")
        w("| --- | --- |")
        for msg, n in sorted(refusal_tally.items(), key=lambda kv: -kv[1]):
            w(f"| {n} | `{msg[:300]}` |")
    w("")

    w("## Reproducing")
    w("```sh")
    w(f"{os.path.abspath(__file__)} \\")
    w(f"  --seed {args.seed} --count {args.count} --indices {args.indices} \\")
    w(f"  --report <path>")
    w("```")
    w("")
    w("Bitcoin Core is started on demand against the scratch datadir and stopped when "
      "the run ends. `--no-core` drops that leg for a fast iteration loop; "
      "`--keep-core` leaves the node up between runs.")
    w("")

    FsPath(path).parent.mkdir(parents=True, exist_ok=True)
    FsPath(path).write_text("\n".join(out) + "\n")


# --------------------------------------------------------------------------
# main
# --------------------------------------------------------------------------

def main(argv=None) -> int:
    ap = argparse.ArgumentParser(
        description="Three-way differential harness for wallet policies "
                    "(Rust `md`, the SeedHammer device, Bitcoin Core).",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    ap.add_argument("--seed", type=int, default=1,
                    help="PRNG seed; the same seed reproduces the same run exactly")
    ap.add_argument("--count", type=int, default=100, help="policies to generate")
    ap.add_argument("--indices", type=int, default=3,
                    help="how many consecutive indices to derive on each chain")
    ap.add_argument("--md", default=DEFAULT_MD, help="path to the Rust `md` binary")
    ap.add_argument("--compose-go", default=DEFAULT_COMPOSE_GO,
                    help="fork's md/compose.go, read for the composer's limits")
    ap.add_argument("--policyprobe", default=None,
                    help="path to the device leg's policyprobe binary")
    ap.add_argument("--bitcoin-bin", default=DEFAULT_BITCOIN_BIN,
                    help="directory holding bitcoind and bitcoin-cli")
    ap.add_argument("--datadir", default=DEFAULT_DATADIR,
                    help="THROWAWAY Core datadir; must be under " + SCRATCH_ROOT)
    ap.add_argument("--rpc-port", type=int, default=DEFAULT_RPC_PORT)
    ap.add_argument("--no-core", action="store_true", help="skip the Bitcoin Core leg")
    ap.add_argument("--keep-core", action="store_true",
                    help="leave bitcoind running after the run")
    ap.add_argument("--no-shrink", action="store_true", help="do not shrink failing cases")
    ap.add_argument("--timeout", type=float, default=120.0, help="per-subprocess timeout")
    ap.add_argument("--report", default=None, help="write the run report here")
    ap.add_argument("--replay", default=None,
                    help="run only this case id from the seeded stream (e.g. 37)")
    ap.add_argument("--quiet", action="store_true")
    args = ap.parse_args(argv)

    limits = read_limits(args.compose_go)
    harness = Harness(args, limits)

    rng = random.Random(args.seed)
    policies = [(f"{args.seed}-{i:04d}", generate_policy(rng, limits))
                for i in range(args.count)]
    if args.replay is not None:
        want = str(args.replay)
        policies = [(cid, p) for cid, p in policies
                    if cid == want or cid.endswith(f"-{int(want):04d}")] \
            if want.lstrip("-").isdigit() else [(cid, p) for cid, p in policies if cid == want]
        if not policies:
            raise SystemExit(f"--replay {args.replay}: no such case in this seed's stream")

    started = time.time()
    if harness.core.enabled:
        if not args.quiet:
            print(f"[core] starting bitcoind on {harness.core.datadir} "
                  f"(port {harness.core.rpc_port})", file=sys.stderr)
        harness.core.start()

    cases: List[CaseResult] = []
    try:
        # Pass 1: Rust and Core, per case.
        for n, (cid, policy) in enumerate(policies, 1):
            cases.append(harness.prepare(cid, policy))
            if not args.quiet and n % 25 == 0:
                print(f"[run] {n}/{len(policies)}", file=sys.stderr)

        # Pass 2: the device leg, batched -- the contract is JSONL in, JSONL out,
        # so one invocation answers every case that minted a card.
        device_out: Dict[str, dict] = {}
        if harness.device.available:
            reqs = [{"id": c.case_id, "chunks": c.chunks, "indices": harness.indices}
                    for c in cases if c.stage == "ok"]
            device_out = harness.device.run(reqs)

        for c in cases:
            harness.finish(c, device_out.get(c.case_id))

        # Pass 3: shrink ONE case per distinct finding signature.
        #
        # Shrinking every failing case separately is the wrong unit of work: a
        # divergence that fires on every `sh-wsh` policy would be shrunk a dozen
        # times to a dozen near-identical reproductions, and a report with a
        # dozen copies of one finding is as unfixable as one with none.  Group
        # by signature, start from the group's already-smallest member, and
        # shrink that.
        groups = group_findings(cases)
        shrunk: Dict[str, CaseResult] = {}
        if groups and not args.no_shrink:
            for sig, grp in groups.items():
                seed_case = min(grp["cases"],
                                key=lambda c: (len(c.policy.paths), c.policy.slots))
                if not args.quiet:
                    print(f"[shrink] {sig[:70]} (from {seed_case.case_id}, "
                          f"{len(grp['cases'])} case(s))", file=sys.stderr)
                shrunk[sig] = shrink(harness, seed_case, signature=sig)
    finally:
        if harness.core.enabled and not args.keep_core:
            harness.core.stop()

    elapsed = time.time() - started
    report = args.report or os.path.join(
        os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
        "design", f"policy-differential-seed{args.seed}.md")
    write_report(report, args, limits, harness, cases, shrunk, elapsed)

    agreed = sum(1 for c in cases if c.stage == "ok" and not c.findings)
    disagreed = sum(1 for c in cases if c.stage == "ok" and c.findings)
    refused = sum(1 for c in cases if c.stage in ("compose", "encode"))
    print(f"generated={len(cases)} agreed={agreed} disagreed={disagreed} "
          f"refused={refused} seed={args.seed}")
    print(f"report: {report}")
    if not harness.device.available:
        print(f"NOTE: device leg unavailable — {harness.device.reason}", file=sys.stderr)
    return 1 if disagreed else 0


if __name__ == "__main__":
    sys.exit(main())
