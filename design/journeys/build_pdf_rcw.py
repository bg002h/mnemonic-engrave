#!/usr/bin/env python3
"""Assembles the REASONABLY COMPLEX WALLET journey, both wrappings.

Same rule as the sibling builders: every CLI block below is transcript_rcw.sh's
real stdout+stderr with its actual exit code, and every device frame is one the
emulator rendered while a comparison held. Nothing here is a value somebody
typed into a document.

Device half: capture_rcw.py, four arms — {tr,wsh} × {keyed,seating}.
"""

import base64, hashlib, html, json, os, re, shutil, subprocess, sys

W = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.join(W, "out", "rcw")
IN = os.path.join(W, "inputs-rcw")
SHOTS = os.path.join(W, "shots")

MISSING = []
ALLOW_MISSING = "--allow-missing" in sys.argv


def missing_gate(artifact):
    if not MISSING:
        return
    uniq = sorted(set(MISSING))
    print(f"\n{len(MISSING)} missing asset(s), {len(uniq)} distinct -- "
          f"{artifact} is INCOMPLETE:", file=sys.stderr)
    for name in uniq:
        print(f"  missing: {name}", file=sys.stderr)
    if ALLOW_MISSING:
        print("--allow-missing given: exiting 0 with an incomplete document.", file=sys.stderr)
        return
    print("Refusing to report success.", file=sys.stderr)
    sys.exit(1)


def readfile(p):
    with open(p) as f:
        return f.read()


def b64(p):
    with open(p, "rb") as f:
        return base64.b64encode(f.read()).decode()


def img(name, cls="shot", cap=None):
    """Embed a captured frame, or record it missing.

    SIZE, NOT EXISTENCE: a canvas that failed to rasterise is written as a
    zero-byte PNG with a 200 OK, so a document could otherwise embed nothing and
    still pass its own gate.
    """
    path = os.path.join(SHOTS, name)
    if not os.path.exists(path) or os.path.getsize(path) < 512:
        MISSING.append(name)
        return f'<p class="missing">missing: {html.escape(name)}</p>'
    tag = f'<img class="{cls}" src="data:image/png;base64,{b64(path)}">'
    return f"<figure>{tag}<figcaption>{cap}</figcaption></figure>" if cap else tag


def code(text, limit=None, must_show=None, must_show_all=None, tail=None):
    """Render a transcript block, optionally clipped.

    `must_show` refuses a clip that hides a named punchline. `must_show_all`
    refuses a clip that hides ANY member of an enumerated set — the guard that
    caught this document's sibling showing four of six seeds, because the clip
    limit was a constant chosen before the section grew.

    `tail` keeps the LAST n lines as well as the first `limit`. Some sections
    are a long uniform list with the assertion underneath it — 46 identical
    "public record N: confirmed" lines and then the count that matters. Raising
    the limit to reach the punchline would put the whole list on the page, and
    dropping the guard would hide it; keeping both ends is the honest shape.
    """
    lines = text.rstrip("\n").split("\n")
    clip = limit and len(lines) > limit
    if clip and tail:
        shown = lines[:limit] + [f"… {len(lines) - limit - tail} lines elided …"] + lines[-tail:]
    else:
        shown = lines[:limit] if clip else lines
    if must_show and clip and not any(must_show in l for l in shown):
        sys.exit(f"code(): clip of {limit} lines hides the punchline {must_show!r}")
    if must_show_all and clip:
        total = sum(1 for l in lines if must_show_all in l)
        vis = sum(1 for l in shown if must_show_all in l)
        if vis < total:
            sys.exit(f"code(): clip of {limit} lines shows only {vis} of {total} "
                     f"{must_show_all!r} blocks")
    body = html.escape("\n".join(shown))
    if clip:
        body += "\n<span class='clip'>… clipped; full text in transcript_rcw.txt</span>"
    return f"<pre>{body}</pre>"


def section(transcript, header):
    marks = [i for i, l in enumerate(transcript) if l.startswith("=== ")]
    for n, i in enumerate(marks):
        if header in transcript[i]:
            end = marks[n + 1] if n + 1 < len(marks) else len(transcript)
            return "\n".join(transcript[i + 1:end]).strip("\n")
    MISSING.append(f"transcript section {header!r}")
    return ""


def walk_result(name, *, need):
    """Read a walk's result JSON and REQUIRE the fields the prose leans on.

    Typed numbers are how a document ends up describing a run that never
    happened; a missing field records a MISSING asset so the gate refuses the
    document rather than shipping a confident sentence about nothing.
    """
    path = os.path.join(SHOTS, name)
    if not os.path.exists(path):
        MISSING.append(name)
        return {}
    with open(path) as f:
        d = json.load(f)
    for k in need:
        if k not in d:
            MISSING.append(f"{name}:{k}")
    return d


CSS = open(os.path.join(W, "build_pdf.py")).read().split('CSS = """')[1].split('"""')[0]
CSS += """
.shot { width: 100%; image-rendering: pixelated; border: 1px solid var(--dim); }
.shotrow { display: grid; grid-template-columns: 1fr 1fr; gap: 10px; margin: 8px 0; }
"""

tx = readfile(os.path.join(W, "transcript_rcw.txt")).split("\n")

# Refuse a snapshot produced by a different version of the generator. section()
# catches a renamed header; this catches everything else that goes stale while
# every header still resolves.
_gen = os.path.join(W, "transcript_rcw.sh")
_want = hashlib.sha256(open(_gen, "rb").read()).hexdigest()
_m = re.search(r"^# transcript-generator-sha256: ([0-9a-f]{64})$", "\n".join(tx), re.M)
if not _m:
    sys.exit("transcript_rcw.txt carries no generator stamp.\n"
             "  Regenerate it:  bash transcript_rcw.sh > transcript_rcw.txt")
if _m.group(1) != _want:
    sys.exit(f"transcript_rcw.txt is STALE -- produced by a different generator.\n"
             f"  snapshot: {_m.group(1)}\n  current:  {_want}\n"
             f"  Regenerate it:  bash transcript_rcw.sh > transcript_rcw.txt")

# ── Device-walk numbers, read from the walks' own result JSON ────────────────
ARMS = {}
for wrap in ("tr", "wsh"):
    for route in ("keyed", "seating"):
        ARMS[(wrap, route)] = walk_result(
            f"rcw-{wrap}-{route}-result.json",
            need=["chunksPresented", "matched"])

PFX = {("tr", "keyed"): "tk", ("tr", "seating"): "ts",
       ("wsh", "keyed"): "wk", ("wsh", "seating"): "ws"}


def arm_addrs(wrap, route):
    return ARMS[(wrap, route)].get("matched", {}).get("addresses", [])


def agree(wrap):
    """The equality that makes the seating arm mean something."""
    k, s = arm_addrs(wrap, "keyed"), arm_addrs(wrap, "seating")
    return bool(k) and k == s


def addr_table(wrap):
    rows = "".join(f"<tr><td><code>{html.escape(a)}</code></td></tr>" for a in arm_addrs(wrap, "keyed"))
    return ("<table><tr><th>on the device AND on the host</th></tr>" + rows + "</table>")


# Host-side address counts, straight off disk rather than described.
def host_count(wrap, chain):
    p = os.path.join(OUT, wrap, f"{chain}.txt")
    return sum(1 for l in open(p) if l.startswith("bc1")) if os.path.exists(p) else 0


P = []
P.append(f"""
<section class="page title">
<h1>SeedHammer II — the reasonably complex wallet</h1>
<p class="sub">A four-tier degrading vault, taken end to end in <b>both</b> its
wrappings: taproot and wsh. Six seeds in; two engraved backups out; the device
agreeing with the host on every address, by two different routes.</p>

<div class="note">Every command in this document ran, and its real stdout,
stderr and exit code are reproduced. Every device screen is a frame the
emulator rendered while an assertion held. Regenerate with
<code>bash transcript_rcw.sh &gt; transcript_rcw.txt</code> then
<code>python3 build_pdf_rcw.py</code>; the builder refuses a transcript produced
by a different version of its generator.</div>

<div class="warn"><b>Test material.</b> The six seeds come from entropy
0x000…001 through 0x000…006 and the three passphrases are printed in the
generator. Never put funds behind any of it.</div>
</section>

<section class="page">
<h1>What the starting representation actually is</h1>
{code(section(tx, "1. THE STARTING REPRESENTATION").split("The two policies")[0], None, must_show="tier 4")}
</section>

<section class="page">
<h1>The two policies</h1>
<p>Same four tiers, two wrappings, and they are <b>different wallets</b>. The
wsh form is not the tr form with a different prefix: <code>multi_a</code>
becomes <code>multi</code> because CHECKSIGADD is tapscript-only, and the
taptree — four separate leaves — flattens into one script, so the tiers become
an <code>or_i</code> chain.</p>

<div class="note">The consequence is worth stating: in <code>tr</code> a spend
reveals only the leaf used. In <code>wsh</code> <b>every</b> spend reveals all
four tiers, including the other two hashlock digests and both deadlines.</div>
{code("The two policies" + section(tx, "1. THE STARTING REPRESENTATION").split("The two policies")[1], None, must_show="policy-wsh")}
</section>

<section class="page">
<h1>Six seeds, six fingerprints</h1>
{code(section(tx, "2. Six seeds"), 9, must_show_all="key-")}
{code(section(tx, "3. The seed fingerprint"), 10, must_show="distinct fingerprints")}

<div class="note">The master fingerprint is a property of the <b>seed</b>, not
of a derivation path, so it is identical in both wrappings. The generator
asserts that rather than assuming it — if the two runs ever produced different
fingerprints for one seed, something would be very wrong and silence would be
the worst outcome.</div>
</section>

<section class="page">
<h1>The keys, and why 8 and 9</h1>
{code(section(tx, "4. The keys, at the bg002h path"), 23)}
</section>

<section class="page">
<h1>tr — the cards</h1>
{code(section(tx, "5tr. The md1 KEYLESS template"), 22, must_show="md1 chunk")}
{code(section(tx, "6tr. The md1 KEYED"), 31, must_show="template-id")}
</section>

<section class="page">
<h1>tr — descriptor and addresses</h1>
{code(section(tx, "7tr. The concrete descriptor"), 10)}
<p>Five receive and five change addresses, on the host
({host_count('tr', 'receive')} and {host_count('tr', 'change')} derived):</p>
{code(section(tx, "8tr. First 5 addresses"), 18, must_show="bc1p")}
</section>

<section class="page">
<h1>wsh — the cards</h1>
{code(section(tx, "5wsh. The md1 KEYLESS template"), 22, must_show="md1 chunk")}
{code(section(tx, "6wsh. The md1 KEYED"), 31, must_show="template-id")}
</section>

<section class="page">
<h1>wsh — descriptor and addresses</h1>
{code(section(tx, "7wsh. The concrete descriptor"), 10)}
<p>Five receive and five change addresses, on the host
({host_count('wsh', 'receive')} and {host_count('wsh', 'change')} derived):</p>
{code(section(tx, "8wsh. First 5 addresses"), 18, must_show="bc1q")}
</section>

<section class="page">
<h1>The key cards bind to the TEMPLATE id</h1>
<p>An mk1 card carries a <code>policy_id_stub</code>, and the stub is
<b>form-aware</b>: a card minted against the keyed policy roots on the
wallet-policy id, while one minted against the keyless template roots on the
template id. Same wallet, two ids — so a card made for the wrong form is
refused at membership before seating is even attempted.</p>
{code(section(tx, "9tr. The mk1 key cards"), 16, must_show="stub the cards embed")}
<div class="note">The transcript gates on that equality rather than printing it:
if the cards ever bound to the policy id instead, the run goes red.</div>
</section>

<section class="page">
<h1>Two wallets, and the proof they are two</h1>
{code(section(tx, "11. tr and wsh are DIFFERENT WALLETS"), 6, must_show="Different ids")}

<div class="warn">This is gated, not observed. If a future edit ever made the
two wrappings produce the same template id or the same addresses, the
transcript fails — because at that point one of them is not the wallet the
document says it is.</div>
</section>

<section class="page">
<h1>The device half — four arms</h1>

<p>The same cards carried into the SeedHammer II emulator over NFC, by
<code>capture_rcw.py</code>. Two routes per wrapper, and the second is the one
that matters:</p>

<ul>
<li><b>keyed</b> — the 15-chunk card alone. It carries its own keys, so nothing
is seated. Proves the device can derive this wallet at all.</li>
<li><b>seating</b> — the 5-chunk keyless template plus its six mk1 cards, which
is <em>what actually gets engraved</em>. Proves the device can join them.</li>
</ul>

<div class="shotrow">
{img(PFX[('tr','seating')] + "03-gather-full.png", cap="tr / seating: 5 template chunks assembled, 6 key cards gathered.")}
{img(PFX[('tr','seating')] + "04-consent-p1.png", cap="tr / seating: the addresses, seated from the key cards.")}
</div>

<h2>tr — both routes agree</h2>
{addr_table('tr')}
<p>Keyed and seating produced <b>{'the same' if agree('tr') else 'DIFFERENT'}</b>
addresses.</p>

<h2>wsh — both routes agree</h2>
{addr_table('wsh')}
<p>Keyed and seating produced <b>{'the same' if agree('wsh') else 'DIFFERENT'}</b>
addresses.</p>

<div class="warn"><b>That equality is the whole point.</b> One wallet reached
two ways — from a card that carries its keys, and from a template plus six
separate key cards — across the air gap, through two implementations in two
languages. Either route alone would only prove the device is self-consistent.</div>
</section>

<section class="page">
<h1>Five on the host, two on the device</h1>

<p>The host derives five addresses per chain. The device shows <b>two</b>, and
that is deliberate: <code>addrProofPerChain = 2</code>
(<code>gui/wallet_policy.go:184</code>), chosen by plan D6 as
<em>"receive AND change, FEWER of each — change is where a policy mismatch
silently loses funds, so proving both chains derive beats proving one chain
five times."</em></p>

<p>Asked to reconsider for this journey, the ruling was to leave it
(<code>design/agent-reports/RULING_sh2_address_count.md</code>). The argument:
<b>every realistic mismatch diverges at index 0</b>, and an index-conditioned
backdoor defeats a five-address screen exactly as easily as a two-address one.
The extra three addresses buy reading fatigue on a <em>consent</em> surface and
no additional evidence.</p>

<div class="note">So the device proves a <b>sample</b> and the host proves the
<b>range</b>, and the walk asserts the device's four are byte-equal to the
host's indices 0–1 per chain — rather than quietly comparing fewer and calling
it agreement.</div>

<h2>The comparison can fail</h2>
<p>An assertion that addresses <em>appear</em> would pass for a walk that
silently stopped driving the device. <code>--prove-it-can-fail</code> corrupts
one character of one expected address and requires the walk to refuse; it exits
0 only when the walk fails.</p>
</section>

<section class="page">
<h1>Getting it to the device: the payload route</h1>
{code(section(tx, "12. The payload route"), 22, tail=6, must_show="public records")}

<div class="note">Gated both ways: the container's record count must equal the
number of strings fed in, and the ms1 count must be <b>zero</b> — this wallet's
three passphrases are engraved as text, not as ms1 shares.</div>
</section>

<section class="page">
<h1>The sealed route, and the limit it has</h1>
{code(section(tx, "13. The sealed-payload route"), 45, must_show="sealed UF2")}

<div class="warn"><b>The two routes do not have the same capacity.</b>
<code>me seal</code> accepts 1..=24 records; both bundles together are 46, so
the combined set is <b>refused</b>. Each wrapper's 23 fit, with one to spare.
The unsealed <code>sysw pack</code> took all 46 without complaint.</div>

<div class="note">The transcript demonstrates that refusal and then gates on it
still happening. If the cap is ever raised, that gate goes red and this page
gets rewritten — rather than quietly becoming false.</div>
</section>

<section class="page">
<h1>What this journey does NOT show</h1>

<ul>
<li><b>No real hardware.</b> The device half is the emulator — the same Go
firmware compiled to wasm, driven over the same NFC entry point, but not a
machine with a screen and a hammer.</li>
<li><b>Nothing was engraved.</b> The plates are rendered previews. No steel was
cut, and the payloads were built but not flashed to a device.</li>
<li><b>No wallet-file export.</b> Nunchuk, Sparrow and Bitcoin Core outputs are
a separate piece of work; at the time of writing no export surface exists
anywhere in the constellation. The Sparrow answer is already known to be
<b>no</b>, and worse than no — see
<code>design/agent-reports/PLAN_export_sparrow.md</code>.</li>
<li><b>No spend.</b> Every address here is derived, none is funded, and no tier
has been exercised against a real chain. The timelocks in particular are
arithmetic in this document and consensus rules everywhere else.</li>
</ul>
</section>
""")

out_html = os.path.join(OUT, "rcw-journey.html")
html_doc = (f"<!doctype html><meta charset=utf-8>"
            f"<title>SeedHammer II — the reasonably complex wallet</title>"
            f"<style>{CSS}</style>{''.join(P)}")
os.makedirs(OUT, exist_ok=True)
with open(out_html, "w") as f:
    f.write(html_doc)
print(f"wrote {out_html} ({len(html_doc)//1024} KB)")

# THE GATE BEFORE THE PRINT. A sibling builder once printed its PDF BEFORE
# running this, so the first bad run overwrote the published deliverable with an
# incomplete one. Gate first, print second.
missing_gate(out_html)

PDF = os.path.join(W, "SeedHammer-II-reasonably-complex-wallet-journey.pdf")
chrome = os.environ.get("CHROME") or shutil.which("chromium") or shutil.which("google-chrome")
if not chrome:
    guess = os.path.expanduser("~/.cache/ms-playwright/chromium-1208/chrome-linux64/chrome")
    chrome = guess if os.path.exists(guess) else None
if not chrome:
    print("no chrome/chromium found; set $CHROME to one. HTML is written.", file=sys.stderr)
    sys.exit(1)
subprocess.run(
    [chrome, "--headless", "--disable-gpu", "--no-sandbox",
     "--no-pdf-header-footer", f"--print-to-pdf={PDF}", "file://" + out_html],
    check=True, capture_output=True,
)
print(f"wrote {PDF} ({os.path.getsize(PDF)//1024} KB)")
missing_gate(PDF)
