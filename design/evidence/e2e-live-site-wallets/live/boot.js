await W.raceFor(["SeedHammer","systemwide payload is present"]);
await W.choose(0, "Payload Digest");
await W.tap(W.CONFIRM,500); await W.waitFor("Payload Warnings");
await W.tap(W.CONFIRM,500); await W.waitFor("Keep this payload loaded?");
await W.tap(W.CONFIRM,500); await W.waitFor("Loaded."); await W.tap(W.BACK,500); await W.nudge();
await W.goTo("Wallet Policy"); await W.tap(W.CONFIRM,500);
await W.waitFor("Build a new policy"); snap("door");
