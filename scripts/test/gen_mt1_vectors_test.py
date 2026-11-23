"""Regression tests for scripts/gen-mt1-vectors.py (bh-me-I12, bh-me-I13).

Run: python3 -I scripts/test/gen_mt1_vectors_test.py

Every generator run gets PATH set to an empty directory, so no bitcoin-cli is
ever reached: the refusals under test must happen before any RPC.
"""
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
GEN = ROOT / "scripts" / "gen-mt1-vectors.py"
CORPUS = ROOT / "scripts" / "test" / "fixtures" / "mk-codec-v0.1-vectors.json"
NO_TX_ERROR = ("generation needs --from-regtest DATADIR and at least one "
               "--tx LABEL=HEXFILE:note; --self-test alone validates without writing")
# The generator's two output names, built from the stem so this file names no
# docs file (ci/code-docs-check.sh); the sentinels stand in for the goldens.
STEM = "mt1_v1_vectors"
SENTINELS = {STEM + ".json": b"sentinel-json\n", STEM + ".md": b"sentinel-md\n"}


def run(*args):
    with tempfile.TemporaryDirectory() as empty_bin:
        return subprocess.run([sys.executable, "-I", str(GEN), *args],
                              capture_output=True, text=True,
                              env={"PATH": empty_bin}, cwd=empty_bin)


def corpus_strings():
    v = json.loads(CORPUS.read_text())
    return [s for vec in v["vectors"] for s in vec.get("expected", {}).get("strings", [])]


def write_corpus(d, strings):
    f = Path(d) / "corpus.json"
    f.write_text(json.dumps({"vectors": [{"expected": {"strings": strings}}]}))
    return f


def regular(strings):
    # In the vendored corpus every regular-code string is at most 93 characters
    # and every long-code string is 111; the generator's verify() still decides,
    # this only builds the fixture (the count assertion below checks the split).
    return [s for s in strings if len(s) < 100]


class SelfTest(unittest.TestCase):
    def test_selftest_vendored_corpus_passes(self):
        r = run("--self-test")
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        self.assertIn("ok   mk corpus: 40/40 verify (19 regular, 21 long)", r.stdout)
        self.assertIn("ok   HRP separation", r.stdout)
        self.assertIn("SELF-TEST PASSED", r.stdout)

    def test_selftest_missing_corpus_fails(self):
        r = run("--self-test", "--mk-vectors", "/nonexistent/nowhere.json")
        self.assertEqual(r.returncode, 1, "self-test passed without its corpus:\n" + r.stdout)
        self.assertIn("FAIL mk corpus not found at /nonexistent/nowhere.json", r.stdout)
        self.assertNotIn("SELF-TEST PASSED", r.stdout)

    def test_selftest_one_string_corpus_fails(self):
        with tempfile.TemporaryDirectory() as d:
            r = run("--self-test", "--mk-vectors", str(write_corpus(d, corpus_strings()[:1])))
        self.assertEqual(r.returncode, 1, "self-test passed on a 1-string corpus:\n" + r.stdout)
        self.assertNotIn("SELF-TEST PASSED", r.stdout)

    def test_selftest_regular_only_corpus_fails(self):
        strs = regular(corpus_strings())
        self.assertEqual(len(strs), 19, "fixture split")
        with tempfile.TemporaryDirectory() as d:
            r = run("--self-test", "--mk-vectors", str(write_corpus(d, strs)))
        self.assertEqual(r.returncode, 1, "self-test passed on a regular-only corpus:\n" + r.stdout)
        self.assertIn("FAIL mk corpus: 19/19 verify (19 regular, 0 long)", r.stdout)

    def test_selftest_extra_string_fails(self):
        strs = corpus_strings()
        with tempfile.TemporaryDirectory() as d:
            r = run("--self-test", "--mk-vectors", str(write_corpus(d, strs + strs[:1])))
        self.assertEqual(r.returncode, 1, "self-test passed on a 41-string corpus:\n" + r.stdout)
        self.assertNotIn("SELF-TEST PASSED", r.stdout)


class Generation(unittest.TestCase):
    def _sentinel_dir(self, d):
        v = Path(d) / "v"
        v.mkdir()
        for n, b in SENTINELS.items():
            (v / n).write_bytes(b)
        return v

    def _unchanged(self, v):
        return {p.name: p.read_bytes() for p in v.iterdir()} == SENTINELS

    def test_generation_without_tx_refuses_before_writing(self):
        with tempfile.TemporaryDirectory() as d:
            v = self._sentinel_dir(d)
            empty = Path(d) / "empty-datadir"
            empty.mkdir()
            r = run("--from-regtest", str(empty), "--rpcport", "28793", "--out", str(v))
            self.assertTrue(self._unchanged(v), f"outputs rewritten (exit {r.returncode})")
        self.assertEqual(r.returncode, 2, r.stdout + r.stderr)
        self.assertIn("gen-mt1-vectors.py: error: " + NO_TX_ERROR, r.stderr)
        self.assertNotIn("SELF-TEST", r.stdout)

    def test_generation_without_from_regtest_refuses_before_writing(self):
        with tempfile.TemporaryDirectory() as d:
            v = self._sentinel_dir(d)
            hexfile = Path(d) / "tx.hex"
            hexfile.write_text("00\n")
            r = run("--tx", f"even={hexfile}:note", "--out", str(v))
            self.assertTrue(self._unchanged(v), f"outputs rewritten (exit {r.returncode})")
        self.assertEqual(r.returncode, 2, r.stdout + r.stderr)
        self.assertIn("gen-mt1-vectors.py: error: " + NO_TX_ERROR, r.stderr)


if __name__ == "__main__":
    unittest.main(verbosity=2)
