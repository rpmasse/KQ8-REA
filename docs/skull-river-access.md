# Skull chamber: river access and western archer

The earned western-archer encounter and a fresh-process reload pass
once in **48.14 seconds**. This advances the genuine Dead City campaign
from WesternPortalCrossedReload. The eastern archer, river crossing and
good skull remain incomplete.

## Original data and route

The installed Dead City NEWWORLD.CS places river-wall death triggers on
both sides of the northwest chamber. TERRAIN.KQ, the floor/material
images and envInfo.kq identify the central strip as fast poison, flag
0x80. The bone bridge is in the eastern chamber. A terrain-only walking
route cannot establish a safe crossing here.

MONSTERS.KQ places skeleton16 and skeleton17 on the far bank. Their
extra stats add to the skelbow base in Monstats.mtl: each has 160 health,
35 damage, 10 protection and a 400-experience award at Normal difficulty.
The tall block.31 on the south bank can provide cover.

The passing route selects the earned Small Crossbow through the HUD,
switches to first person while standing, and moves to approximately
(33300,192800). It uses normal camera rays and projectile collisions to
shoot skeleton17. The original block intercepts skeleton16's shot.
Connor's death, enemy AI, damage, terrain and trigger rules stay active.
The test neither grants inventory nor invokes a damage/script callback.

## Failed attempts retained

| Attempt | Result | Local evidence |
| --- | --- | --- |
| Walk around the river's west edge | Died at update 132 after entering rivrtrap.01; no skull or new save. | good-skull-native.log; earned-good-skull/1794044067750593900 |
| Fight from the exposed south bank | Both archers damaged Connor; he died at update 190. | skull-river-guards-native.log; earned-skull-river-guards/1794044546535599600 |
| Fight beside the tall block | The camera selected skeleton17, but Connor's bolts hit block.31 after 712 units. He died at update 238. | skull-river-guards-cover.log; earned-skull-river-guards/1794044813029029800 |

After the two combat failures, native variants stopped for a numerical
projectile audit. A local probe used the recorded original oriented
boxes, live muzzle offsets and collision radii: 235 for Connor's bolt,
65 for the archer's arrow. It reproduced the failed block contact near
(33590,194580,1807) and identified a clear outgoing lane farther west
with a blocked return path from skeleton16. This is planning evidence;
the subsequent native run established the actual hit and survival.
The probe's output is skull-river-projectile-sweep.log. It omits moving
actors and future poses and is not an original-engine comparison.

The abandoned floor-only skull test is preserved locally as
kq8-artifacts/good-skull-failed-route-prototype.rs. It is not shipped as
a working campaign route.

## Automated command and recorded result

Run from the repository root in PowerShell, with no other viewer running:

```powershell
$env:KQ8_GAME_DIR=(Resolve-Path -LiteralPath "../King's Quest 8 GOG").Path
$env:KQ8_WESTERN_PORTAL_SLOT=(Resolve-Path -LiteralPath 'kq8-artifacts/earned-western-portal/1794043427585987500/slots/slot-4.json').Path
cargo test --locked --release -p kq8-viewer --test daventry_cottage_shield dead_city::earned_crossbow_defeats_the_western_skull_archer_and_reloads -- --ignored --exact --nocapture
```

Expected: one passing test. It checks visible projectile attacks, one
completed death and award, unchanged inventory and quest flags, the
Rock-held western plate, and identical stats after a fresh-process
reload without another award. Original Options.cs must be unchanged.

The recorded run is skull-western-archer-native.log. Compilation took
11.66 seconds; the two-process test took 48.14 seconds. The death award
ran at update 195. Final stats are [52,85,27,11,2074], level 12, and
position is (33253,192808,864). Skeleton16 remains, and no GoodSkull is
carried. The genuine continuation is:

`kq8-artifacts/earned-skull-river-guards/1794045168852782700/slots/slot-3.json`

Its label is SkullWesternArcherReload. Slot-2 is the encounter save.
Use this continuation for the next stage; do not repeat the passed
encounter merely to recover the same state.

## Return portal and earned healing

The installed PORTRAP6.CS override returns Connor to (63691,110436,1584),
sets TwoOn, ThreeOn, FourOn and FiveOn true and SixOn false, restores the
three involved portal masks to 4, and returns control without awarding
XP again. Its paired setThirdPerson calls restore the preceding view;
first person after this sequence agrees with camera.md rule 7.

The first portal viewer passed and wrote a genuine SkullReturnPortal
save. Its test then failed after 36.86 seconds because the Cue 5
assertion looked in doors.log; the event is in traps.log at update 161.
The corrected assertion finds exactly one occurrence in that existing
log. The successful traversal was not repeated.

The first resumed well attempt reached (67035,105055,1584), but the
automatic aim could not see well2 from first person. It failed after
21.61 seconds without healing. The next attempt used the ordinary F2
key while standing to select the third-person view used by earlier
well routes. It passed once in **40.30 seconds** across the well leg and
a fresh-process reload. Fourteen visible clicks each invoked the
original five-health event, restoring 52 to the level-12 maximum 120.

Inventory, XP, quest flags, the held Rock and defeated skeleton17 remain
unchanged. Skeleton16 remains present. Portal flags and masks persist,
and original Options.cs remains unchanged. Final stats are
[120,85,27,11,2074], level 12. Genuine continuation:

`kq8-artifacts/earned-skull-return-well/1794045759136946000/slots/slot-3.json`

The label is SkullReturnWellReload, at (66544,104638,1584), in third
person. Evidence logs: skull-return-well-native.log (wrong-log assertion),
skull-return-well-resumed.log (first-person aim failure), and
skull-return-well-third-person.log (pass).

The command, from the repository root, is:

```powershell
$env:KQ8_GAME_DIR=(Resolve-Path -LiteralPath "../King's Quest 8 GOG").Path
$env:KQ8_SKULL_WESTERN_ARCHER_SLOT=(Resolve-Path -LiteralPath 'kq8-artifacts/earned-skull-river-guards/1794045168852782700/slots/slot-3.json').Path
# Use the genuine return checkpoint to avoid repeating its completed leg.
$env:KQ8_SKULL_RETURN_PORTAL_SLOT=(Resolve-Path -LiteralPath 'kq8-artifacts/earned-skull-return-well/1794045576576558000/slots/slot-2.json').Path
cargo test --locked --release -p kq8-viewer --test daventry_cottage_shield dead_city::skull_return_portal_and_sylph_healing_preserve_the_earned_victory -- --ignored --exact --nocapture
```

Expected: one passing test, fourteen original healing events, full health
and matching saved/restored state. Omitting KQ8_SKULL_RETURN_PORTAL_SLOT
also runs the portal leg. That corrected combined entry point compiled;
the recorded native evidence uses the successful portal checkpoint and
the passing continuation, rather than repeating the completed leg.

## Remaining work

Return through portal5 with full health, then complete the second archer,
the original block/jump crossing, good-skull pickup and
replacement clicker, statue/feather/scales and the remaining campaign.
The jumping route and skull pickup are **not tested successfully**.
No runtime source changed in this step; unrelated core tests were not
rerun. No successful native test was repeated. No manual testing.
