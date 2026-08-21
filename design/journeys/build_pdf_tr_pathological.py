#!/usr/bin/env python3
"""Assembles the TAPROOT PATHOLOGICAL operator journey.

Same rule as the other three: every CLI block is transcript_tr_pathological.sh's
real stdout+stderr with its actual exit code, and every screenshot is the
emulator's own framebuffer, captured by capture_tr_pathological.py.

What is new here is that the document's central claim is CHECKED BY THE
CAPTURE, not by this script: the walk is handed the host's wallet id and
addresses and throws if the screen disagrees. So the pages below cannot show a
device agreeing with a host unless it did.
"""

import base64, html, json, os, shutil, subprocess, sys

W = os.path.dirname(os.path.abspath(__file__))
SHOTS = os.path.join(W, "shots")
OUT = os.path.join(W, "out", "tr-pathological")
IN = os.path.join(W, "inputs-pathological")

# ── Missing-asset gate ─────────────────────────────────────────────────────
# Kept identical to the three sibling builders on purpose. A skipped step must
# FAIL, not pass: both wallet journeys once built with 100% of their referenced
# screenshots missing and still exited 0. Enforcement is the DEFAULT;
# --allow-missing is the explicit opt-OUT for a draft.
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
    print("Refusing to report success. Run capture_tr_pathological.py, or pass "
          "--allow-missing for a draft.", file=sys.stderr)
    sys.exit(1)


def b64(p):
    with open(p, "rb") as f:
        return base64.b64encode(f.read()).decode()


def img(name, cls="shot", cap=None):
    path = os.path.join(SHOTS, name)
    if not os.path.exists(path) or os.path.getsize(path) < 512:
        MISSING.append(name)
        return f'<p class="missing">missing: {html.escape(name)}</p>'
    tag = f'<img class="{cls}" src="data:image/png;base64,{b64(path)}">'
    return f"<figure>{tag}<figcaption>{cap}</figcaption></figure>" if cap else tag


def code(text, limit=None):
    lines = text.rstrip("\n").split("\n")
    clip = limit and len(lines) > limit
    if clip:
        lines = lines[:limit]
    body = html.escape("\n".join(lines))
    if clip:
        body += "\n<span class='clip'>… clipped; full text in transcript_tr_pathological.txt</span>"
    return f"<pre>{body}</pre>"


def readfile(p):
    if not os.path.exists(p):
        MISSING.append(os.path.basename(p))
        return ""
    with open(p) as f:
        return f.read()


def section(transcript, header):
    """Return the transcript block under `=== header ... ===`.

    Slicing the transcript by its own headers rather than retyping its output:
    a journey that quotes a command's result by hand is exactly the thing these
    documents claim not to be.
    """
    marks = [i for i, l in enumerate(transcript) if l.startswith("=== ")]
    for n, i in enumerate(marks):
        if header in transcript[i]:
            end = marks[n + 1] if n + 1 < len(marks) else len(transcript)
            return "\n".join(transcript[i + 1:end]).strip("\n")
    MISSING.append(f"transcript section {header!r}")
    return ""


CSS = open(os.path.join(W, "build_pdf.py")).read().split('CSS = """')[1].split('"""')[0]

tx = readfile(os.path.join(W, "transcript_tr_pathological.txt")).split("\n")
result_path = os.path.join(SHOTS, "tr-pathological-result.json")
result = json.load(open(result_path)) if os.path.exists(result_path) else MISSING.append(
    "tr-pathological-result.json") or {}

template = readfile(os.path.join(IN, "wallet-policy-tr.txt")).strip()
ids = readfile(os.path.join(OUT, "tr.id.txt"))
recv = readfile(os.path.join(OUT, "tr.receive.txt")).split()
chg = readfile(os.path.join(OUT, "tr.change.txt")).split()
md1 = [l for l in readfile(os.path.join(OUT, "md1-keyed.txt")).split("\n") if l.strip()]

P = []

tmpl_cards = [l for l in readfile(os.path.join(OUT, "md1-template.txt")).split("\n") if l.strip()]

P.append(f"""
<section class="page">
<h1>SeedHammer II — the taproot pathological wallet</h1>
<p class="sub">The hardest wallet this constellation describes, round-tripped on the machine.</p>

<p>This is the four-tier degrading vault — the wallet <code>CORPUS.md</code>
calls <em>the</em> pathological example — expressed as a <strong>depth-3 taproot
tree</strong>:</p>
<pre>{html.escape(template)}</pre>

<p>Two <code>sha256</code> hashlocks. Both Bitcoin timelock flavours, absolute
and relative. <code>multi_a</code> at thresholds 3, 2, 2 and 1. A <strong>NUMS
internal key</strong>, so there is no key-path spend at all — every route to the
coins goes through a script leaf. Eleven cosigners across <strong>three
masters</strong>.</p>

<p class="note"><strong>Until three commits ago this document could not exist.</strong>
Every one of its four leaves is <code>and_v(v:…)</code> wrapping a timelock or a
hashlock, and the device's tap-leaf reader named only
<code>pk</code>&nbsp;/&nbsp;<code>multi_a</code>&nbsp;/&nbsp;<code>sortedmulti_a</code>.
It returned <em>zero</em> leaves and the consent screen read "Complex policy —
display only", while the host derived the same wallet's addresses without
complaint. That was F-214; the fix replaced the vocabulary with an emitter, and
this journey is the proof it worked.</p>

<p class="note">The keys are BIP-39 test vectors. Never put funds behind them.</p>
</section>
""")

P.append(f"""
<section class="page">
<h2>1 · Every slot's origin matches its own key file</h2>
<p>The template carries eleven per-key origins, written by hand; the eleven key
files declare them independently. If the two disagreed, the card would name a
path the key does not live at — and <strong>nothing downstream would notice</strong>,
because addresses come from the xpub a card carries, never from the origin it
declares. That is F-217's lesson, and this gate is the answer to it.</p>
{code(section(tx, "2. Every slot's origin matches its key file"), 16)}

<h2>2 · The engraved artifact</h2>
<p>The keyless template set — {len(tmpl_cards)} chunks. No <code>--path</code>:
the origins ride in the template, and <code>--path</code> would flatten eleven
distinct origins onto one, which over eleven different keys is a card
<code>md encode</code> now refuses outright.</p>
{code(section(tx, "3. The engraved artifact"), 14)}
</section>
""")

P.append(f"""
<section class="page">
<h2>3 · E1 — the cards re-encode to the policy they came from</h2>
{code(section(tx, "4. E1"), 10)}

<p class="note"><strong>Why <code>md verify</code> and not a <code>md decode</code>
diff.</strong> Decode renders the per-key origins <em>away</em>: the text it
returns is well-formed, missing the one field a signer needs to find its keys,
and re-encodes to a <em>different card</em>. A journey asserting decode-equality
would either fail on a perfectly correct plate or be quietly weakened until it
passed. <code>md verify</code> exists for exactly this question, returns 0 or 1,
and compares payloads rather than rendered text. The gap is filed as F-219.</p>

<h2>4 · The bind — both card sets are the same wallet</h2>
<p>The engraved set is keyless; the set the device gathers carries the eleven
xpubs. Without this assertion the journey could engrave one wallet and prove
another. The <em>template</em> id is key-stable, which is precisely what makes it
the right thing to compare.</p>
{code(section(tx, "7. THE BIND"), 26)}
</section>
""")

P.append(f"""
<section class="page">
<h2>5 · What the device must prove to</h2>
<p>Derived from the <em>cards</em>, not from the argument list that made them.</p>
<div class="kv">
<div><span>wallet id</span><code>{html.escape(ids.strip())}</code></div>
<div><span>receive 0..1</span><code>{html.escape(chr(10).join(recv[:2]))}</code></div>
<div><span>change 0..1</span><code>{html.escape(chr(10).join(chg[:2]))}</code></div>
</div>

<h2>6 · The device</h2>
<div class="row">
{img("t00-boot.png", cap="Home.")}
{img("t01-carousel.png", cap="Wallet Policy.")}
</div>
<div class="row">
{img("t02-gather-empty.png", cap="The gather, empty.")}
{img("t03-gather-full.png", cap=f"After all {len(md1)} chunks cross the reader: ONE card.")}
</div>
</section>
""")

consent_imgs = [n for n in sorted(os.listdir(SHOTS)) if n.startswith("t04-consent-p")] if os.path.isdir(SHOTS) else []
P.append(f"""
<section class="page">
<h2>7 · The proof, on the consent path</h2>
<p>Eleven key slots print before the addresses do, so the screen pages
{len(consent_imgs)} times — and the walk reads every page, because a bound tuned
for a four-key wallet would stop before reaching the half that matters.</p>
<div class="row">
{"".join(img(n, cap=f"consent, page {i + 1}") for i, n in enumerate(consent_imgs))}
</div>
</section>
""")

matched = result.get("matched", {})
P.append(f"""
<section class="page">
<h2>8 · What this run established</h2>

<p>The capture is a <strong>comparison</strong>. It is handed the host's id and
addresses and throws if the screen disagrees, and the absence assertions reject
"display only" and "no addresses" outright — so the device cannot pass by
refusing politely.</p>

<div class="kv">
<div><span>chunks presented</span><code>{result.get("chunksPresented", "—")}</code></div>
<div><span>cards gathered</span><code>{result.get("cardsGathered", "—")}</code></div>
<div><span>consent pages read</span><code>{len(result.get("consentPages", []))}</code></div>
<div><span>wallet id matched</span><code>{html.escape(str(matched.get("walletPolicyId", "—")))}</code></div>
<div><span>addresses matched</span><code>{html.escape(chr(10).join(matched.get("addresses", [])))}</code></div>
</div>

<p><strong>The negative control is a command, not a memory.</strong>
<code>python3 capture_tr_pathological.py --prove-it-can-fail</code> corrupts one
character of one expected address and exits 0 only if the walk <em>caught</em>
it. A comparison nobody has made fail is not evidence that it can.</p>

<h2>Reproducing</h2>
<pre>bash transcript_tr_pathological.sh &gt; transcript_tr_pathological.txt 2&gt;&amp;1
python3 capture_tr_pathological.py                    # rebuilds emu.wasm, drives it
python3 capture_tr_pathological.py --prove-it-can-fail # the negative control
python3 build_pdf_tr_pathological.py                  # this document</pre>

<h2>What this journey does NOT show</h2>
<ul>
<li><strong>No spend, and for tiers 1–2 no spend is possible from this backup
at all.</strong> Both hashlock tiers need the 32-byte preimage to spend, and
<em>zero</em> backup strings carry it (F-132).</li>
<li><strong>The tiers do not degrade in the order they read.</strong> Tier 4
(1-of-3) matures at 365 days; tier 3 (2-of-2) at ≈455 — the weakest key-set
unlocks ~90 days <em>first</em> (F-133).</li>
<li><strong>No plate read-back.</strong> There is no reader; the card-set
equality above is the honest proxy.</li>
<li><strong>No ms1 secret leg.</strong> Taproot-independent, and already shown
in the earlier journeys.</li>
<li><strong>The origins are not visible on read-back</strong> (F-219) — the card
carries them, and neither <code>decode</code> nor <code>inspect</code> will show
them to an operator.</li>
</ul>
</section>
""")

doc = ("<!doctype html><meta charset='utf-8'>"
       "<title>SeedHammer II — the Wallet Policy journey</title>"
       f"<style>{CSS}</style>" + "".join(P))
p = os.path.join(OUT, "tr-pathological-journey.html")
os.makedirs(OUT, exist_ok=True)
with open(p, "w") as f:
    f.write(doc)
print(f"wrote {p} ({os.path.getsize(p)//1024} KB)")

# THE GATE BEFORE THE PRINT. The pathological builder once printed its PDF
# BEFORE running this, so the first bad run overwrote the published deliverable
# with one captioned "no caption captured for this plate".
missing_gate(p)

PDF = os.path.join(W, "SeedHammer-II-tr-pathological-journey.pdf")
chrome = os.environ.get("CHROME") or shutil.which("chromium") or shutil.which("google-chrome")
if not chrome:
    guess = os.path.expanduser("~/.cache/ms-playwright/chromium-1208/chrome-linux64/chrome")
    chrome = guess if os.path.exists(guess) else None
if not chrome:
    print("no chrome/chromium found; set $CHROME to one. HTML is written.", file=sys.stderr)
    sys.exit(1)
subprocess.run(
    [chrome, "--headless", "--disable-gpu", "--no-sandbox",
     "--no-pdf-header-footer", f"--print-to-pdf={PDF}", "file://" + p],
    check=True, capture_output=True,
)
print(f"wrote {PDF} ({os.path.getsize(PDF)//1024} KB)")
missing_gate(PDF)
