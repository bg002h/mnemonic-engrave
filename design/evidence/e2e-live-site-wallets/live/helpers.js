window.W = (() => {
const sleep = ms => new Promise(r => setTimeout(r, ms));
const squash = s => String(s).replace(/\s+/g, "");
const BACK=[453,70], CONFIRM=[453,249], NEXT=[455,160], PAGE=[453,160], NOWHERE=[5,5];
const log = [];
const tap = async ([x,y], settle=300) => { window.shTap(x,y); await sleep(settle); };
const nudge = async () => { window.shTap(...NOWHERE); await sleep(120); };
async function waitFor(needle, timeoutMs=20000) {
  const want = squash(needle); const dl = Date.now()+timeoutMs; let p=0;
  for(;;){ const t=window.shScreen(); if(squash(t).includes(want)) return t;
    if(Date.now()>=dl) throw new Error(`waitFor(${needle}) timed out; screen: ${t}`);
    if(++p%6===0) await nudge(); await sleep(50);} }
async function raceFor(ns, timeoutMs=30000){ const dl=Date.now()+timeoutMs; for(;;){const t=squash(window.shScreen()); for(const n of ns) if(t.includes(squash(n))) return n; if(Date.now()>=dl) throw new Error("raceFor timeout: "+window.shScreen()); await sleep(50);} }
function rows(){ return window.shTargets(); }
async function choose(i, expect, settle=400){ const t=window.shTargets(); if(i<0) i=t.length+i; if(i>=t.length) throw new Error(`row ${i} of ${t.length}; screen: ${window.shScreen()}`); await tap([t[i].cx,t[i].cy],300); await tap(CONFIRM,settle); if(expect) await waitFor(expect); await nudge(); return window.shScreen(); }
async function chooseText(txt, expect, settle=400){ // pick row whose label matches, by reading targets order vs screen? fallback
  throw new Error("unused"); }
async function readAll(max=30){ const pages=[]; for(let i=0;i<max;i++){ await nudge(); const t=window.shScreen(); if(pages.length && squash(t)===squash(pages[0])) break; if(pages.map(squash).includes(squash(t))) break; pages.push(t); await tap(PAGE,400);} return pages; }
async function goTo(prog, max=14){ for(let i=0;i<max;i++){ if(squash(window.shScreen()).startsWith(squash(prog))) return i; await tap(NEXT,250);} throw new Error("goTo "+prog+": "+window.shScreen()); }
async function hold(){ window.shPress(...CONFIRM); await sleep(1300); window.shRelease(...CONFIRM); await sleep(400); }
function screen(){ return window.shScreen(); }
function targets(){ return JSON.stringify(window.shTargets()); }
return {sleep,squash,tap,nudge,waitFor,raceFor,choose,readAll,goTo,hold,screen,targets,rows,BACK,CONFIRM,NEXT,PAGE,NOWHERE};
})();
window.W.KEY_ROWS = [{letters:"qwertyuiop",x0:87,y:198},{letters:"asdfghjkl",x0:104,y:244},{letters:"zxcvbnm",x0:138,y:290}];
window.W.keyPoint = ch => { for (const r of W.KEY_ROWS){ const j=r.letters.indexOf(ch); if(j>=0) return [r.x0+j*34, r.y]; } throw new Error("no key "+ch); };
window.W.typePhrase = async (phrase) => { const ws=phrase.trim().split(/\s+/);
  await W.waitFor(`word 1 of ${ws.length}`);
  for (let n=1;n<=ws.length;n++){ const w=ws[n-1];
    for (let i=0;i<w.length;i++){ await W.tap(W.keyPoint(w[i]),120); const want=`${n}:${w.slice(0,i+1).toUpperCase()}`;
      if(!W.squash(W.screen()).includes(want)) throw new Error(`typing ${w} letter ${i+1}: screen ${W.screen()}`); }
    await W.tap(W.CONFIRM,400); if(n<ws.length) await W.waitFor(`word ${n+1} of ${ws.length}`); }
};
