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

## Remaining work

Connor has 52 health. Check an earned healing route through the return
portal and Sylph's well before committing to the second archer. Complete
that encounter, the original block/jump crossing, good-skull pickup and
replacement clicker, statue/feather/scales and the remaining campaign.
The jumping route and skull pickup are **not tested successfully**.
No runtime source changed in this step; unrelated core tests were not
rerun. No successful native test was repeated. No manual testing.
