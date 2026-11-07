# Skull chamber: river access and archers

Both archers are now defeated in the earned campaign. The western
encounter/reload passed once in 48.14 seconds; after an earned healing
visit, the portal revisit, eastern encounter and reload passed once in
87.51 seconds. The later original two-block crossing, good-skull pickup
and shelf replacement also survive reload. Returning with that skull
and completing the remaining Dead City quests are still unfinished.

## Return-route diagnostics

The preserved source is `earned-skull-chamber-potion/1794048357391220700/
slots/slot-3.json`, label SkullChamberPotionReload, with level-12 stats
[39,85,27,11,2474]. GoodSkull and the unconsumed Invulnerable potion are
carried. All artifact paths below are relative to `kq8-artifacts/`.

Two integrated native return attempts use the previously native-tested
hazard viewer; they do not validate the new interface or scripted-character
source. Both preserve the source save and original Options hashes.

| Attempt | Evidence | Result |
| --- | --- | --- |
| Diagonal approach to the western launch | earned-skull-return-chapter/1794052215484 | At update 118, position (28315,202143,607), liquid depth 256.8 causes an ordinary protected hit for 357; health 39 becomes -318. No jump requested. |
| Approach along the higher bank | earned-skull-return-chapter/1794052447255 | Reaches (27011,202994,864) at update 134. Then circles the launch target at distances 35–39, outside its 25-unit arrival radius. Stops at update 344 at (27024,201519,695); alive, no jump requested. |

After two failures, native variations stop for an inspection of the
diagnostic arrival rule and ordinary walking step. The real movement,
collision and damage rules must not be changed to make a route pass.
No new campaign checkpoint, return portal or eastern-room arrival is
claimed from these runs.

Numerical planning evidence is separate:

- `skull-return-complete-scene.log`: launch (27000,202000) fails 18/27
  perturbations because of deep landings; the candidate is rejected.
- `skull-return-guarded-launch.log`: launch (27000,201550) passes 27/27
  combinations of position offsets ±40 and heading offsets ±0.13, with
  grounded south-bank landing, no dangerous liquid depth and no swept
  contact with rivrtrap.01. This uses shared core movement/trap code.
- `skull-return-bank-corridor.log`: 837 terrain samples along
  (30059,203052) → (27000,203000) → (27000,201550), including offsets ±80,
  have maximum liquid depth 192.4, leaving 57.6 below the damage threshold.
- `skull-return-nominal-trap-clearance.json`: the nominal recorded arc
  clears the river trap's top by at least 683.2 units where its expanded
  XY bounds overlap. This is a nominal trace, not a native outcome.

The local movement scene matches 25 of 35 recorded building bounds to
original BSP data; none of those 35 bounds overlaps the western launch
rectangle. It is not a complete-world collision reconstruction. The
eastern bone bridge needs its actual raised support geometry; a
terrain-only planner is insufficient and its eastern detour is rejected.

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

## Healed return and eastern archer

The continuation uses SkullReturnWellReload and follows the original
static terrain/building route west out of the well area, through
(66000,104750), (64000,107000), (64000,110000) and (62500,110436).
Normal movement enters portal5. FirstFive remains true, so the script
returns Connor to the skull chamber without another 4,000-XP award.
The genuine HealedSkullChamber checkpoint preserves all 120 health.

From that save, normal F2 and the ranged HUD slot select the earned
Small Crossbow in first person. The south-bank route reaches roughly
(36250,193250), where the complete static bolt sweep found a clear lane
to skeleton16. Actual camera rays and bolts hit that archer; its death
completes once at update 218 and awards 400 XP. An already flying arrow
hits Connor afterwards, leaving 39 health. The western archer is not
recreated or fought again.

The three-process test **passes once in 87.51 seconds**, after a
12.15-second compilation. Both deaths, carried items, quest flags,
portal flags and the held Rock persist; the fresh reload repeats
neither death award. Original Options.cs stays unchanged. Evidence:
skull-eastern-archer-native.log.

Newest genuine continuation:

`kq8-artifacts/earned-skull-eastern-archer/1794046057966712000/slots/slot-4.json`

The label is SkullEasternArcherReload, at (36280,193254,864), in first
person with the weapon stowed. Level-12 stats are [39,85,27,11,2474].
Slot-2 is HealedSkullChamber and slot-3 is SkullEasternArcher.

```powershell
$env:KQ8_GAME_DIR=(Resolve-Path -LiteralPath "../King's Quest 8 GOG").Path
$env:KQ8_SKULL_RETURN_WELL_SLOT=(Resolve-Path -LiteralPath 'kq8-artifacts/earned-skull-return-well/1794045759136946000/slots/slot-3.json').Path
cargo test --locked --release -p kq8-viewer --test daventry_cottage_shield dead_city::healed_return_defeats_the_eastern_skull_archer_and_reloads -- --ignored --exact --nocapture
```

Expected: one passing test with no repeat portal XP, one eastern-archer
award, both deaths retained and matching saved/restored stats. If a
genuine HealedSkullChamber save exists, KQ8_HEALED_SKULL_CHAMBER_SLOT
can skip the completed portal leg. Continue from slot-4 for subsequent
work rather than repeating the successful encounter.

## Original block and jump crossing

The original installed C_jump.kqs and C_rnjmp.kqs movements both take
1.333 seconds at the Conner.anm speed of one. Their horizontal travel
is about 1,423 and 6,356 units respectively. A local numerical probe
uses those complete movements, original terrain/materials and nearby
oriented object bounds with the Rust mover. It plans two standing jumps
onto block.32 (feet at 1,779) and block.31 (feet at 2,689), followed by
the running jump north across the damaging river strip.

The selected launch points are (31550,193400), (32900,194300) and
(34600,196100). Each has 27 safe numerical variants covering position
errors of up to 60 units on each horizontal axis and heading errors of
0.13 radians. The connecting walks remain supported in the local mover.
The building/terrain planner also finds a clear south-bank approach.
These probes omit live callbacks and actors; falling uses zero horizontal
motion. They are planning evidence, not proof of a playable crossing.

The unattended route's running_jump option enables Run while standing
at takeoff, then presses the bound Forward and Jump keys together. It
uses the normal input handler and waits for supported landings. The
movement, collision, damage and animation rules are unchanged. Each
block gets a genuine save so a later failure need not repeat that climb.
KQ8_SKULL_RIVER_CHECKPOINT can resume an earned SkullLowerBlock,
SkullHigherBlock or SkullRiverCrossed save.

```powershell
$env:KQ8_GAME_DIR=(Resolve-Path -LiteralPath "../King's Quest 8 GOG").Path
$env:KQ8_SKULL_EASTERN_ARCHER_SLOT=(Resolve-Path -LiteralPath 'kq8-artifacts/earned-skull-eastern-archer/1794046057966712000/slots/slot-4.json').Path
cargo test --locked --release -p kq8-viewer --test daventry_cottage_shield dead_city::original_blocks_and_jumps_cross_the_skull_river_and_reload -- --ignored --exact --nocapture
```

Expected: one passing test, both original standing jumps and a running
jump accepted, Connor alive on the north bank, and unchanged inventory,
stats, defeated archers, portal flags and held Rock after a fresh reload.
Both standing jumps and the running jump succeed in native play. The
lower block lands at update 149, (32696,194193,1779); the higher block
at update 60 of its leg, (34142,194985,2689). The running jump launches
at update 44, (34603,196076,2689), and lands at update 68,
(34744,202931,864). The ordinary hold/stop transitions also contribute
movement after the authored jump arc.

The initial test fails after 68.49 seconds despite all three native
legs reporting PASS. It demands final y>202900, but after reaching the
flat-bank target, the original walk-stop motion carries Connor onto
the bank slope at (35363,202274,662). That remains less than 250 below
the water surface, outside the deep-liquid damage condition. Health
stays 39. The assertion now checks the actual safe-bank boundary,
y>202250 and z>614, against the existing saved result.

Only the outstanding fresh-process reload is then run; it **passes
once in 10.96 seconds** after a 14.57-second compilation. Original
Options.cs, level-12 stats [39,85,27,11,2474], carried items, quest
flags, both archer deaths and the held Rock remain unchanged. No
successful jump is repeated. Logs: skull-river-crossing-native.log
(initial assertion failure), skull-river-crossing-reload.log (pass).

The passing continuation uses the command above with this additional
environment setting before cargo test:

```powershell
$env:KQ8_SKULL_RIVER_CHECKPOINT=(Resolve-Path -LiteralPath 'kq8-artifacts/earned-skull-river-crossing/1794047217919130800/slots/slot-4.json').Path
```

Newest genuine continuation:
`kq8-artifacts/earned-skull-river-crossing/1794047351774964100/slots/slot-2.json`,
label SkullRiverCrossedReload. Earlier slot-2 and slot-3 in the first
run are SkullLowerBlock and SkullHigherBlock. The corrected full route
compiles, while native evidence uses the completed movement legs and
the passing reload continuation rather than repeating those legs.

## Earned good skull and shelf replacement

From SkullRiverCrossedReload the original static terrain/BSP planner
finds a safe north-bank route via (32750,203250) to (31500,204500).
The native check walks that route and uses a normal camera ray to
skull.22. At update 104 the ray hits that object first and dispatches
its original GSKULL.CS action. At update 110 the single delayed cue
gives one GoodSkull and removes skull.22. No skull is injected.

With Reaper::metConnor already 4, SkullMsg advances from 0 to 1.
StatHasGoodSkull stays false until the statue is solved. The script
creates the replacement clicker NoName100 at (31583,206637,994).
Its identity, file, rotation and position match after a fresh reload.
No BadSkull appears, and level-12 stats [39,85,27,11,2474], other
inventory, both archer deaths, portal flags and the held Rock persist.
Original Options.cs stays unchanged.

The two-process check **passes once in 37.41 seconds**, following a
14.86-second compilation. Evidence: good-skull-earned-native.log.
Newest genuine continuation:
`kq8-artifacts/earned-good-skull/1794047529603851200/slots/slot-3.json`,
GoodSkullReload at (31469,204559,864). The original river jumps and
archer encounters are not repeated.

```powershell
$env:KQ8_GAME_DIR=(Resolve-Path -LiteralPath "../King's Quest 8 GOG").Path
$env:KQ8_SKULL_RIVER_CROSSED_SLOT=(Resolve-Path -LiteralPath 'kq8-artifacts/earned-skull-river-crossing/1794047351774964100/slots/slot-2.json').Path
cargo test --locked --release -p kq8-viewer --test daventry_cottage_shield dead_city::earned_good_skull_and_its_shelf_replacement_survive_reload -- --ignored --exact --nocapture
```

Expected: one passing test, one earned GoodSkull and the same dynamic
shelf replacement after reload, with no repeated pickup cue. Only
tests and documentation change; no unrelated core checks are rerun.

## Remaining work

### North-bank potion and return planning

WORLDINV.CS places NoName49, one Invulnerable potion, at
(33111,203729,2000). Normal walking and a camera ray collect it, then
a fresh process restores it alongside the GoodSkull and unchanged
level-12 stats [39,85,27,11,2474]. Both native legs complete in the
29.74-second run earned-skull-chamber-potion/1794048357391220700.
The integration test then incorrectly compares the removed-object list
without allowing NoName49's normal pickup removal. Corrected checks
pass against both saved results, and the corrected target compiles
with --no-run in 16.13 seconds. No native sequence is repeated.
The final Options.cs byte assertion was not reached and is not tested
for this run. Evidence: skull-chamber-potion-native.log and
skull-chamber-potion-compile.log. Newest earned continuation is its
slots/slot-3.json, SkullChamberPotionReload at (32656,204036,864).

```powershell
$env:KQ8_GAME_DIR=(Resolve-Path -LiteralPath "../King's Quest 8 GOG").Path
$env:KQ8_GOOD_SKULL_SLOT=(Resolve-Path -LiteralPath 'kq8-artifacts/earned-good-skull/1794047529603851200/slots/slot-3.json').Path
cargo test --locked --release -p kq8-viewer --test daventry_cottage_shield dead_city::skull_chamber_potion_is_earned_and_reloaded_with_the_good_skull -- --ignored --exact --nocapture
```

Expected: one carried potion with its original identity, preserved skull
and campaign state, and only that collected item added to the removal
list. The existing successful gameplay evidence is retained; this
command is documented for future use, not a request to repeat it now.

The full-controller numerical probe uses the original Conner.anm
movements, including the falling hold and stopping animations. Central
return jumps briefly enter deep fast-poison. Q206's original-engine
study establishes that the configured zero value means immediate
1,000-point base hits, so the missing Rust damage path cannot be used
as evidence of a safe crossing. The potion does not grant terrain
immunity. A wider search finds western-edge candidates near x=27000
without deep exposure; the sample from (27000,202000,705) lands at
(27000,195394,705) and stops near (26954,194149,864). That model omits
live callbacks and buildings; trap clearance and native traversal are
not yet verified. The next integrated chapter route can use this plan
after the shared hazard implementation, rather than adding another
standalone check for each pickup or encounter.

Complete the return from the skull shelf, statue/feather/scales and
the remaining campaign. Those continuations are **not tested** from
this earned skull state. No successful native test was repeated.
No manual testing.
