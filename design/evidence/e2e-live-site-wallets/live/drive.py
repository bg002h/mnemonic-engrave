#!/usr/bin/env python3
"""Drive the LIVE SH2 emulator page. Usage: drive.py <steps.js> [url]
steps.js is the body of an async function using W.*; it returns a JSON-able value.
"""
import asyncio, json, sys, os
from playwright.async_api import async_playwright
URL = sys.argv[2] if len(sys.argv) > 2 else "https://quantoshi.xyz/SH2/emu/index.html"
H = open(os.path.join(os.path.dirname(os.path.abspath(__file__)), "helpers.js")).read()
body = open(sys.argv[1]).read()
async def main():
    async with async_playwright() as pw:
        b = await pw.chromium.launch()
        pg = await b.new_page()
        errs = []
        pg.on("pageerror", lambda e: errs.append(str(e)))
        await pg.goto(URL)
        await pg.wait_for_function("window.shScreen !== undefined && window.shTargets !== undefined", timeout=120000)
        await pg.wait_for_timeout(2500)
        await pg.add_script_tag(content=H)
        try:
            res = await pg.evaluate("async () => { const trace=[]; const snap=(l)=>trace.push({l, s: W.screen(), n: window.shTargets().length}); try { const r = await (async()=>{" + body + "})(); return {ok:true, r, trace}; } catch(e) { return {ok:false, err: String(e), screen: W.screen(), trace}; } }")
        except Exception as e:
            res = {"ok": False, "err": str(e)}
        res["pageerrors"] = errs
        print(json.dumps(res, indent=1))
        await b.close()
asyncio.run(main())
