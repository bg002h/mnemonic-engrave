await W.raceFor(["SeedHammer","systemwide payload is present"]);
await W.choose(0, "Payload Digest");
await W.tap(W.CONFIRM,500); await W.waitFor("Payload Warnings");
await W.tap(W.CONFIRM,500); await W.waitFor("Keep this payload loaded?");
await W.tap(W.CONFIRM,500); await W.waitFor("Loaded."); await W.tap(W.BACK,500); await W.nudge();
await W.goTo("Wallet Policy"); await W.tap(W.CONFIRM,500);
await W.waitFor("Build a new policy"); snap("door");
const PRESET_ROW = 4, KEY_ROW = 0;
const out = {};
await W.choose(1, "Which script?");
await W.choose(0, "Start from?");
await W.choose(PRESET_ROW, "Done");
out.pathList = W.screen();
await W.choose(-1, null); await W.sleep(400); await W.nudge();
if (W.squash(W.screen()).includes("Sortedkeys")) { out.sortQ = W.screen(); await W.choose(0, null); await W.nudge(); }
await W.waitFor("Which key path?");
out.keyPathScreen = W.screen(); out.keyPathTargets = W.targets();
await W.choose(KEY_ROW, "Template-ID");
out.stub1 = await W.readAll();
await W.tap(W.CONFIRM,600); await W.waitFor("Seat keys into this template?");
await W.choose(1, "Where from?");
await W.choose(0, "Source: the systemwide payload");
await W.tap(W.CONFIRM,600); await W.waitFor("Add a BIP-39 passphrase?");
await W.choose(0, "Slot @0");
out.seat = [];
const SEATS = [1,1,1,1], TYPED = {2:"legal winner thank year wave sausage worth useful legal winner thank yellow",3:"letter advice cage absurd amount doctor acoustic avoid letter advice cage above"}; let nSeeds = 1;
for (let i=0;i<10;i++){ await W.nudge(); const s=W.screen(); if(!W.squash(s).includes("Seatkeys")) break; out.seat.push(s);
  const want = SEATS[i];
  while (want > nSeeds) {
    const typeRow = nSeeds; // seeds first, then "Type a seed"
    if (!W.squash(W.screen()).includes("Typeaseed")) throw new Error("no Type a seed row: "+W.screen());
    await W.choose(typeRow, "Where from?");
    await W.choose(1, "Choose number of words");
    await W.choose(0, "word 1 of 12");
    await W.typePhrase(TYPED[nSeeds+1]);
    await W.waitFor("Add a BIP-39 passphrase?");
    await W.choose(0, "Seat keys"); nSeeds++;
    out.seat.push("after typing seed "+nSeeds+": "+W.screen());
  }
  if (!W.squash(W.screen()).includes("seed"+want)) throw new Error("seed row missing: "+W.screen());
  await W.choose(want-1,null); await W.sleep(200);} 
await W.waitFor("Key mapping");
out.mapping = await W.readAll();
await W.tap(W.CONFIRM,600); await W.waitFor("Policy-ID");
out.stub2 = await W.readAll();
await W.tap(W.CONFIRM,600); await W.waitFor("Review");
out.consent = await W.readAll();
await W.tap(W.CONFIRM,600); await W.waitFor("Nothing outside this device");
out.holdScreen = W.screen();
await W.hold();
await W.waitFor("Which form?"); out.formPick = W.screen();
await W.choose(0, null); await W.sleep(400); await W.nudge(); out.modePick = W.screen();
if (W.squash(out.modePick).includes("Watch-only")) { const t = window.shTargets(); 
   // find watch-only row: rows listed in order on screen; Full first? read both
   await W.choose(W.squash(out.modePick).indexOf("Watch-only") < W.squash(out.modePick).indexOf("Full") ? 0 : 1, null); }
await W.waitFor("Plates To Cut");
out.census = await W.readAll();
await W.tap(W.CONFIRM,600);
// engrave tail
const H = [["Holdbuttontostarttheengravingprocess","hold"],["Engravingcompletedsuccessfully","confirm"],["Chooseengraving","choose"],["Bundleengraved","confirm"],["RestoreDoc","confirm"]];
out.acts=[]; out.variants=[]; let stall=0,last=-1;
for (let g=0; g<20000; g++){
  const steps = JSON.parse(window.shToolpath.summary()).steps;
  if (steps===last) stall++; else {stall=0; last=steps;}
  if (stall<3){ await W.sleep(75); continue; }
  await W.tap(W.NOWHERE,150);
  const s = W.squash(W.screen());
  if (s.includes("Buildanewpolicy")) { out.door=true; break; }
  const h = H.find(x=>s.includes(x[0]));
  if (!h){ out.acts.push("STALLED:"+s.slice(0,200)); break; }
  out.acts.push(h[0]);
  if (h[1]==="hold"){ window.shPress(...W.CONFIRM); await W.sleep(1300); window.shRelease(...W.CONFIRM);
     const b=JSON.parse(window.shToolpath.summary()).steps; for(let i=0;i<200;i++){ if(JSON.parse(window.shToolpath.summary()).steps!==b) break; await W.sleep(75);} }
  else if (h[1]==="choose"){ out.variants.push(W.screen()); await W.choose(0,null); for(let i=0;i<200;i++){ await W.tap(W.NOWHERE,75); if(!W.squash(W.screen()).includes(h[0])) break;} }
  else { if (h[0]==="RestoreDoc") out.restoreDoc = W.screen(); window.shTap(...W.CONFIRM); for(let i=0;i<200;i++){ await W.tap(W.NOWHERE,75); if(!W.squash(W.screen()).includes(h[0])) break;} }
  stall=0; last=-1;
}
out.strings = JSON.parse(window.shToolpath.strings());
return out;
