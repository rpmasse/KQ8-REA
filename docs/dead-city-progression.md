# Dead City campaign route

## The entrance archers and first hall

The route starts from the `DeadCityReload` earned by using the Ring
of Light at Daventry’s tomb. Crossing the first trigger runs the
original `introtp.cs`. It takes control, walks Connor north, and has
the west skeleton archer speak two lines. When its script returns
control, Connor is already beyond the entrance pillars.

An early replay turned back to an obsolete waypoint after the scene,
and the two skeleton archers killed Connor. The passing replay waits
for the scene to finish. Connor uses the Small Axe he earned in
Daventry against both archers. Their death animations each award 100
experience and leave their corpses. The first award takes Connor to
level seven. A normal click on the first hall's paddle rack then runs
`dpaddle.cs` and gives him one Paddle. A fresh process restored the
paddle and the completed encounter without awarding experience again.
The two-process regression passed in 158.21 seconds. Its final save
had 58 health and 137 experience toward the next level.

## The way to the skull

The direct road beyond the hall meets the closed spike gate. The
`spmove.cs` script consumes the Key to Death's Maze to open it.
Walking into that gate without the key stopped the replay. The gate
is a gameplay prerequisite. The original `Death.cs` creates the key
sequence, and `keycycle.cs` gives the key when its animation finishes.

## The paddle pedestals and letter tiles

The earned route uses Scary3, Scary2, Scary4 and Scary5. Connor puts
the Paddle on each pedestal using the quest strip, then pushes it from
the side. Both push directions occur. After the first push he defeats
the skeleton that approaches the pedestal. One earlier run waited
there and lost too much health to survive the next attack. The first
three pedestals each return the Paddle. The fourth starts the laser
sequence, opens both hall doors and returns control.

Entering the hall starts the original `dhalltrp.cs` sequence and
places Connor at the south edge of an eight-by-eight floor. A disk
save used to lose the tile controller: loading at the entrance made
the Forward key walk normally. A current save restores the letter
Connor needs next, his movement and the remaining puzzle state. It
rebuilds the 64 stone references from the loaded world. A forward
jump onto the wrong stone sends Connor back to the strip. A save
made halfway through that jump resumes and produces one landing.
The two-process in-flight check saved `tilejump1` at phase 0.5 and
passed in 64.97 seconds. Loading a real pre-fix save from the original
hall entrance also recovered the controller and passed in 36.27 seconds.
Another three-launch check saved during both halves of D's teleport.
The restored camera finished its flight, the letter remark played once
and E remained the next letter. It passed in 98.67 seconds.

Connor jumps across the start strip and onto D. A fresh process
loads the resulting save near E. Regular turn and Jump input takes
him through E, A, T and H. Death then delivers his original dialogue.
The `givekey` movement loads `keycycle.kq`; its final callback grants
one Key to Death's Maze. The `DeathMazeKey` save has Connor back at
the first hall with the key. The eight-process regression from
`FirstHallReload` completed in 740.44 seconds. The final fresh
load recorded one key, 80 health and 1,197 experience at level eight.

The good skull, the statue that produces the feather and the scales
still need verification in this campaign. The existing focused scales
test starts with a supplied feather.

## Crossing the spike gate

From `MazeKeyReload`, Connor uses the earned key on `spmove` through
the gameplay camera. Its original script consumes the key, adds the
gate to the map and lowers the spikes. A second process loads the open
gate and walks across its collision plane. The 90.29-second regression
ended on the far side at y 105245 with 40 health.

The first route to the north corridor ended in combat. One attempt
defeated the archer but died with two skeletons pursuing Connor. A
second attempt tried to fight them immediately and died near the gate.
Both runs used ordinary combat from an earned save. The entrance
battle also left a health crystal and a broadsword in the first hall.
The equipped route revisits those drops from the maze-key save.

## The three maze guards

Connor returns to the crystal and broadsword left by the first hall's
fights. The replay finds them by item type and position in that hall,
then clicks each through the gameplay camera. The same Small Axe he
brought from Daventry appears where the broadsword lay. His hand
strength rises from 60 to 62 and his experience stays the same. A
fresh process finds the same two items in his inventory and the same
axe on the ground. This two-process check passed in 28.98 seconds.

Starting with that equipment, Connor unlocks the gate and draws
the broadsword before crossing. In a second process he keeps it
drawn while fighting the two swordsmen, uses the crystal once with
the second inventory key, and defeats the archer farther north. The
guards award 300, 300 and 200 experience. A third process restores
the cleared area and the same level-nine stats, 80 health and 717
experience toward the next level. This release check passed in
84.55 seconds. The two earlier defeats used the axe and lacked the
crystal. The northern maze and the campaign route to the scales remain open.

Connor then collects both Mushrooms dropped by the east swordsman,
runs west through the passage and saves at its northwest turn. A
fresh process restores that position, both Mushrooms, 80 health and
the same experience. This two-process continuation passed in 47.97
seconds. The next northern junction contains a separate mace
skeleton and two archers.

## The northern junction

The next encounter contains skeleton29 with a Mace and archers
skeleton28 and skeleton27. Connor draws the broadsword and uses
the two Mushrooms collected from the gate guards. The first
canonical replay circled an overly tight waypoint while an archer
kept firing and died after defeating the group. The movement log
showed repeated distances of 323–329 units against a 235-unit
arrival radius.

With combat arrival areas widened to 400 units, the two-process
test passed in 66.44 seconds. Each death awards experience once,
250 for the mace guard and 200 per archer. The fresh reload retains
all three victories, 12 health and 1,367 experience at level nine.
Their Mace and three Mushroom drops can support the next western
stretch. The route continues from the earned corner save.

## Junction equipment and the closed cells

The next two-process check collects the Mace and three Mushrooms
dropped by the junction guards. The Mace replaces the broadsword
and raises melee strength from 67 to 70. Three presses of the first
inventory key consume the Mushrooms and raise health from 12 to 27.
The fresh reload restores that health, the equipped Mace, and the
broadsword where the Mace originally lay. The test passed in 36.12
seconds and ends on the road east of the closed cells.

An earlier westward probe stopped at Cell04. The offline planner
included terrain and scenery but omitted animated doors. The
original ASwitch4 script controls Cell04 from a switch across the
maze. Adding the save’s 59 closed doors to the local diagnostic
found a route around the cells to the north. The direct route to
the western Weeper remains unverified.

Two attempts along the outer route reached its stronger mace guard,
skeleton56, and ended in Connor’s death. The first counterattack run
killed the guard after Connor’s health had already fallen below zero.
The second began the approach on foot earlier. The verified handoff
remains `NorthSuppliesReload`, before this encounter. The guard has
225 health and a half-second attack delay. Its pursuing movement needs
an input and combat audit before another route attempt.

## The outer mace guard

The audit found that the retreat replay inherited the road’s enabled
Run toggle. Its Forward input therefore continued running into the
guard. The controller now presses the configured Run key to switch
to walking before its retreat and counterattack inputs.

The distant approach then timed out following the patrol. The original
MONSTERS.KQ sends skeleton56 between (157871, 152500) and
(128395, 152500), and its walking speed exceeds Connor’s. The closer
approach intercepts the patrol before switching to walking.

The replay now also waits for the enemy's actual `Do` hit cue before
counterattacking, replacing its assumed halfway point.

The two-process regression passed in 66.89 seconds. It observes
ordinary enemy attacks miss, Mace strikes damage skeleton56, and
one death award of 450 experience. The fresh process retains the
defeated guard, the Mace, 27 health and 1,817 experience at level nine.
Cell04 stays closed throughout this outer route. The check starts
from the earned `NorthSuppliesReload` save.

The earlier Run-only check passed at 16 health in 66.37 seconds.
Two western patrol attempts from that save ended in Connor's death.
The cue correction left skeleton55 at 28 health, but did not complete
that fight. Those logs are retained in local `dead-road-westernpatrol`
artifacts. Further encounter variations are stopped under the project
handoff rule. Original-runtime comparison is **not tested**.

## Recovering the remaining first-hall supplies

The earned outer-guard save still contains four Mushroom drops from
the cleared first hall. The route returns through the open spike
gate, clicks each actual drop, saves, and starts a fresh process for
the return. Four ordinary health-slot inputs raise health from 27
to 47 while experience stays at 1,817 and level stays at nine. The
exploratory pickup and return both passed. The canonical three-process
test passed in 166.50 seconds, including the final fresh-process
reload with 47 health, the Mace and Small Crossbow, no remaining
Mushrooms, and Cell04 still closed.

The initial southbound attempt stopped at flametow.10, a solid prop
beside closed Cell04. Its running waypoint accepted a line too far
east. The corrected route walks at x 161885 through the gap, using
a 90-unit arrival radius, then resumes running. It preserves both
colliders and all enemy and quest state.

## The western patrol

Starting with the earned 47-health `CorridorSuppliesReload`, the
exploratory ordinary-retreat fight defeated skeleton55. Its 500-point
death award leaves 2317 experience at level nine, with 11 health.
The separate fresh-process reload check passed in 10.14 seconds:
it preserves those stats, both weapons and the removed guard, without
repeating the award or changing the GOG Options.cs.

This is not a reliable encounter replay yet. The canonical plain-swing
repeat died with the enemy at 15 health. The original Shift-click big
swing was then checked through the existing input dispatcher; two hits
left 10 enemy health, but Connor died. Further melee variations are
stopped. The encounter regression remains ignored with an explicit
known-failing reason. The successful exploratory save and the two
failed replays are all retained locally; no health, enemy rules or
quest flags were changed. Weeper15 and the boatman route remain open.

To check only the successful victory's restoration, set
`KQ8_WESTERN_PATROL_SLOT` to a genuine `WesternPatrol` save, then run:

```powershell
cargo test --locked --release -p kq8-viewer --test daventry_cottage_shield dead_city::saved_western_patrol_victory_reloads_without_another_award -- --ignored --exact --test-threads=1
```

The retained failing encounter check is
`dead_city::supplied_western_patrol_victory_and_award_reload`, with
optional earned input `KQ8_CORRIDOR_SUPPLIES_SLOT`. It currently
reproduces death before saving; do not count it as a passing route.

## The river Weeper

The earned `WesternPatrolReload` save continues along the cleared
road to a firing position near (74250,158750). Connor switches to
first person while standing, selects his Small Crossbow through the
ordinary HUD, faces weeper15 and fires through visible gameplay rays.
Ten projectile hits defeat it. Its one 575-point award raises Connor
from level nine to ten and restores health from 11 to 100, leaving
332 experience. The two-process check passed in 76.51 seconds and
restores the victory, both weapons and those stats as
`RiverWeeperReload`. Normal AI and damage remain enabled; GOG options
are unchanged.

The route's new `ranged: true` fight modifier selects and uses the
normal ranged weapon. First-person facing uses ordinary turn keys.
The visibility gate uses the original ranged aim mask, rather than
the interaction mask that excludes enemies. Geometry can still block
shots. Earlier preflight attempts and the unrelated F2/stoprun stall
are recorded in STATUS and Q200. The stall remains unresolved; this
passing route changes view before it starts moving. The boatman and
the rest of the regional quest remain open.

Set `KQ8_WESTERN_PATROL_RELOAD_SLOT` to a genuine
`WesternPatrolReload` save and use the configured GOG installation:

```powershell
cargo test --locked --release -p kq8-viewer --test daventry_cottage_shield dead_city::earned_crossbow_defeats_the_river_weeper_and_restores_its_award -- --ignored --exact --test-threads=1
```

## The boathouse guard

The earned RiverWeeperReload continues along the cleared road to
skeleton38. Connor switches to third person while standing, chooses
the melee HUD slot and attacks with his earned Mace. One normal
Shift-click hit defeats the guard on update 282; its death completes
on 317 and awards 300 experience once. The two-process regression
passed in 72.82 seconds. BoathouseGuardReload retains level ten,
stats [94,75,23,9,632], both weapons, third-person view and the
defeated skeleton55, weeper15 and skeleton38. AI and damage remain
enabled, and the GOG options are unchanged. The introductory Ferryman
topics continue from this checkpoint below.

Set `KQ8_RIVER_WEEPER_SLOT` to a genuine `RiverWeeperReload` save
and use the configured GOG installation:

```powershell
cargo test --locked --release -p kq8-viewer --test daventry_cottage_shield dead_city::earned_mace_clears_the_boathouse_guard_and_reloads -- --ignored --exact --test-threads=1
```

## The Ferryman's introductory topics

From BoathouseGuardReload, Connor walks through the real boathouse
doorway and clicks Ferryman through a visible gameplay ray. He stays
at the talking spot while each topic finishes, saves, and starts the
next topic after a fresh process loads that save. These are the
original reaper.cs outcomes:

| Topic | Conversation case / final sequence | Earned result |
| --- | --- | --- |
| 1 | 41 / 7 | metConnor 1; one 2000-point award, experience 632 to 2632 |
| 2 | 42 / 7 | metConnor 2; no further award |
| 3 | 43 / 5 | metConnor 3; KnowAboutFeather; Feather description metadata |
| 4 | 44 / 5 | metConnor 4; one 3000-point award, level eleven, 110 health and 512 experience |

The final five-process check passed in 319.03 seconds. The
FerrymanKnowledgeReload retains [110,80,25,10,512], both weapons and
the knowledge flag without another reward. No Feather item is
granted. AI and damage remain enabled; GOG Options.cs is unchanged.
The later item offers and bridge-dependent topics remain untested.

The third topic exposed a rejected short item name in `invClick`.
That command now uses the existing item-name prefix rule. Its script
regression failed before the correction and passes afterward; all
878 standard workspace tests pass. The final complete native rerun
also verifies that the original `invClick Feather` is accepted on
update 366. It retains the installed description numbers '76 0 0 1'.
The first native attempt's log-wording assertion failure and the
earlier baseline without this correction are recorded in STATUS.

Set `KQ8_BOATHOUSE_GUARD_SLOT` to a genuine `BoathouseGuardReload`
save and use the configured GOG installation:

```powershell
cargo test --locked -p kq8-core inventory_description_script_accepts_short_and_full_type_names
cargo test --locked --release -p kq8-viewer --test daventry_cottage_shield dead_city::four_ferryman_topics_earn_knowledge_and_reload -- --ignored --exact --test-threads=1
```

## The western Rock

The original WORLDINV.CS places one Rock on the western road at
(56446,125813). From a genuine FerrymanKnowledgeReload, the unattended
route leaves the boathouse, defeats BowGuys16, 15 and 17 with the
earned Mace, and clicks that Rock through a visible gameplay ray.
Each archer awards 200 experience once. The route and a fresh-process
reload passed in 155.92 seconds: WesternRockReload retains one Rock,
both weapons, the three defeated archers and Ferryman knowledge,
with stats [61,80,25,10,1112] at level eleven. AI and damage remain
enabled; GOG Options.cs is unchanged.

The first attempt stopped against an archer's body because its
arrival mark lay inside the collision boundary. The corrected
mark begins ordinary attack input outside that boundary; no combat
or movement rules changed. Both results are recorded in STATUS.
The later portal, skull and feather routes remain unverified.

Set `KQ8_GAME_DIR` to the original GOG installation and
`KQ8_FERRYMAN_KNOWLEDGE_SLOT` to a genuine FerrymanKnowledgeReload:

```powershell
cargo test --locked --release -p kq8-viewer --test daventry_cottage_shield dead_city::western_archers_and_rock_survive_reload -- --ignored --exact --test-threads=1
```

The test should pass after two unattended Windows launches, including
the disk reload. Local evidence is
`kq8-artifacts/earned-western-rock/1794026411163329300`.

## The Sylph pressure blocks

The first plate is verified from WesternRockReload. Ordinary forward
movement with empty hands pushes block15 off trptrp15; its original
Off callback raises trip15 and lights the nearby tower. The two-process
check passed in 52.94 seconds. FirstSylphBlockReload retains the moved
block, empty trigger, B15 up and flame.port501, with unchanged stats,
inventory and GOG options. The other plates remain down, so no
Sylph appearance or reward is expected yet.

Set `KQ8_WESTERN_ROCK_SLOT` to a genuine WesternRockReload and use
the configured GOG installation:

```powershell
cargo test --locked --release -p kq8-viewer --test daventry_cottage_shield dead_city::first_sylph_block_lights_its_tower_and_reloads -- --ignored --exact --test-threads=1
```

This should pass after the push and fresh-process reload. Evidence:
`earned-sylph-first-block/1794027010744231100` under kq8-artifacts.
The first draft's log-text assertion error is retained in STATUS.

**The continuation now passes.** The first three-block route
kept pushing block14 down the road until higher ground stopped it.
The revised route cleared blocks14 and13 but collided with torch.dt4
on the block12 approach. Both stopped without a continuation save.
Native retries stopped after two failures for a steering/clearance
audit. The driver accepts a 0.13-radian heading error; the long leg
did not leave enough room beside the torch for the actual animation
steps. The audited route crosses at Y82500 with closer marks. All
791 offline corridor samples pass the terrain/static-bound check.

The subsequent live continuation and fresh-process reload passed in
125.28 seconds. Blocks14,13 and12 clear on updates240,670 and1054;
the last callback summons Sylph and awards 2000 experience once.
SylphSummonedReload retains all four flags up, empty plate triggers,
moved blocks and Sylph's home at (70662,105677,1584), with level eleven
stats [61,80,25,10,3112]. Inventory and GOG options stay unchanged;
reload does not repeat the award. Evidence is
earned-sylph-summoned/1794027970800884800 and
sylph-summoned-audited-native.log under kq8-artifacts.

Set `KQ8_FIRST_SYLPH_BLOCK_SLOT` to a genuine FirstSylphBlockReload.
With `KQ8_GAME_DIR` configured, this command should pass after the
remaining pushes, appearance award and fresh-process reload:

```powershell
cargo test --locked --release -p kq8-viewer --test daventry_cottage_shield dead_city::all_four_sylph_plates_summon_her_once_and_reload -- --ignored --exact --test-threads=1
```

This verifies the pressure-plate puzzle. The subsequent conversation
check is below; the item quest remains **not tested** on this route.

## The Sylph conversations

The earned SylphSummonedReload continues around the western portal
structures into the original SylphInfo conversation area. Ordinary
visible clicks start all three introductory topics. The original
sylph.cs final callbacks advance metConnor to 1, 2 and 3; the second
topic supplies the one-time 3000 experience award. Each topic uses a
fresh viewer process and the final result is loaded once more.

The check passed in 252.41 seconds. SylphKnowledgeReload retains
topic 3, the summoned Sylph, all four raised plates, unchanged block
placements and inventory, and level eleven stats [61,80,25,10,6112].
Neither the appearance award nor the conversation award repeats on
reload. GOG options, damage and AI settings remain unchanged. Evidence:
earned-sylph-topics/1794028407978349500 and sylph-topics-native.log.

Set `KQ8_GAME_DIR` to the GOG installation and `KQ8_SYLPH_SUMMONED_SLOT`
to a genuine SylphSummonedReload. This should pass after the three
conversations and final reload:

```powershell
cargo test --locked --release -p kq8-viewer --test daventry_cottage_shield dead_city::three_sylph_topics_award_experience_once_and_reload -- --ignored --exact --test-threads=1
```

The initial offline corridor had excluded samples beside the portal
structures and was not run. A wider obstacle-margin search supplied
the successful detour. Her item quest remains **not tested**: the
original scale.cs creates Mold during the later scales completion.

## Southern passage and second Rock

The original building geometry resolves the approach that the earlier
terrain planner marked unreachable. A terrain-only diagnostic cannot
reach the western high plateau but does find the winding ground-level
route. A second diagnostic replaces whole-building exclusions with
collision sweeps for 25 original placements whose bounds match the
loaded scene; other bounds, objects and closed doors remain blocked.
These helpers guide input routes, not original engine behavior, and
their paths still require live verification. The navigation lead is
recorded in [RESEARCH](RESEARCH.md).

Three consecutive sections now pass with fresh-process reloads:

| Earned section | Result | Native duration |
| --- | --- | --- |
| SylphKnowledgeReload → SouthernPassageReload | Ordinary movement to the southern passage; inventory and stats unchanged | 136.28 s |
| SouthernPassageReload → PassageWeeperReload | Earned Small Crossbow defeats weeper16; original 500 experience once, no health loss | 69.48 s |
| PassageWeeperReload → SecondRockReload | Visible pickup of WORLDINV.CS's NoName44 at (112595,70784,864); two Rocks retained | 62.28 s |

SecondRockReload ends at (112862.055,70938.58,864), level eleven with
stats [61,80,25,10,6612]. Other items and Sylph knowledge remain intact;
the defeated Weeper and its award persist correctly. GOG options are
unchanged, and no damage, AI, item or location cheats are used.
The first Rock assertion's wrong field name and successful corrected
rerun are both retained in STATUS. No movement or combat rules changed.

With `KQ8_GAME_DIR` configured, supply the genuine starting save via
`KQ8_SYLPH_KNOWLEDGE_SLOT`, `KQ8_SOUTHERN_PASSAGE_SLOT` or
`KQ8_PASSAGE_WEEPER_SLOT` for the corresponding command. Each should
pass its route/encounter/pickup and final reload:

```powershell
cargo test --locked --release -p kq8-viewer --test daventry_cottage_shield dead_city::sylph_to_southern_passage_preserves_campaign_and_reloads -- --ignored --exact --test-threads=1
cargo test --locked --release -p kq8-viewer --test daventry_cottage_shield dead_city::passage_weeper_with_earned_crossbow_survives_reload -- --ignored --exact --test-threads=1
cargo test --locked --release -p kq8-viewer --test daventry_cottage_shield dead_city::second_placed_rock_is_picked_up_and_reloaded -- --ignored --exact --test-threads=1
```

Evidence under kq8-artifacts: earned-southern-passage/1794029366147771600,
earned-passage-weeper/1794029563826960700 and
earned-second-rock/1794029819717808900. This has **not** yet reached
the graveyard or enabled its four portals. Those sections remain open.

## Central passage combat and the Sylph well

The first two native combat attempts beyond SecondRockReload failed.
That route was stopped, then resumed only after independently
verifying the original well healing described below:

| Attempt | Observed result | Native duration |
| --- | --- | --- |
| Walk, then use the Small Crossbow | skeleton2 defeated; skeleton3 kills Connor on update 790; skeleton1 also joins and interrupts shots | 56.40 s |
| Run and use the earned Mace | skeleton1 defeated on update 796, its 200 experience awarded on 811; Connor dies approaching skeleton2 on 877 | 62.19 s |

The source has 61 of 110 maximum health, no carried healing items,
and the earned Mace and Small Crossbow. The movement trace shows
running interrupted by incoming arrows; these failures do not prove
an engine defect. No damage, enemy AI or inventory rules were changed.
Evidence is under kq8-artifacts/earned-central-archers/1794030315905121300
and earned-central-guards/1794030484043099000, with corresponding
central-archers-native.log and central-guards-native.log.

The failed Mace revision is retained in commit 8fff486. The current
test instead starts from the genuine healed `KQ8_SYLPH_WELL_SLOT`
(`SylphWellReload`) and passes with the original `KQ8_GAME_DIR`:

```powershell
cargo test --locked --release -p kq8-viewer --test daventry_cottage_shield dead_city::central_passage_guards_with_earned_mace_survive_reload -- --ignored --exact --test-threads=1
```

A read-only proximity audit also lists BowGuys1/2 near the eastern
bend, MaceBoy1/2 near the later northern turn, skeleton52 on the
western approach, and zombie5 near the graveyard destination.
It uses original saved placements and alarm distances, with a
300-unit allowance; it does not simulate visibility, team alarms,
births or movement. Being listed does not establish an encounter.
Evidence: central-guards-threat-audit.log and graveyard-route-threat-audit.log.
The first diagnostic draft mishandled characters without a mover;
the corrected audit uses their saved animation placement and fails
if neither position is available. This was a diagnostic error, not
another native combat attempt.

Original sylphwel.cs heals five points for each ordinary ConnorAction.
Its canDrink flag controls the sound delay rather than the health
event. The independent well check returns from SecondRockReload
through the cleared route and uses visible clicks on well2, retaining
both Rocks and the existing quest state. It does not retry the stopped
central combat route.

The well check and fresh-process reload **passed in 169.75 seconds**.
Ten visible clicks on updates 1176-1184 heal Connor from 61 to 110;
the final event clamps at the original level-eleven maximum.
SylphWellReload retains stats [110,80,25,10,6612], both Rocks, all
other items, quest flags and moved blocks. The defeated Weeper stays
gone, no healing repeats on reload, and GOG Options.cs is unchanged.
Final position is (66725.86,104720.02,1584). Evidence:
kq8-artifacts/earned-sylph-well/1794030725771876400/slots/slot-3.json
and sylph-well-native.log. The discarded offline approach target
was unreachable; only its western approach was replayed.

With the same original `KQ8_GAME_DIR` and genuine `KQ8_SECOND_ROCK_SLOT`,
this command should pass the return, healing and reload assertions:

```powershell
cargo test --locked --release -p kq8-viewer --test daventry_cottage_shield dead_city::earned_return_to_sylph_well_heals_through_visible_clicks_and_reloads -- --ignored --exact --test-threads=1
```

## Prepared central and western passage

The healed continuation passed in **179.31 seconds**, including a
safe approach save and two fresh-process loads. It defeats all three
original central team members with the earned Mace, awards their
combined 500 experience once and retains both Rocks and quest flags.
CentralGuardsReload ends at (129515.93,104362.805,864), with level
eleven stats [23,80,25,10,7112]. Ordinary incoming damage is retained.
Evidence: earned-prepared-central-guards/1794031235820492500 and
prepared-central-guards-native.log under kq8-artifacts.

The next native check passed in **96.07 seconds**. Visible clicks
collect the seven actual drops: four Mushrooms, coin stacks of 7 and
18, and one Strength potion. Four uses of inventory key 1 heal 20
points. CentralSuppliesReload retains [43,80,25,10,7112], 25 coins,
one unused Strength potion, both Rocks and all earlier quest state.
Original Invinfo.itl gives each Mushroom [5,0,0,0,0]; no other stat
changes. Evidence: earned-central-supplies/1794031529033744000 and
central-supplies-native.log. With `KQ8_CENTRAL_GUARDS_SLOT` pointing
to the genuine CentralGuardsReload, this command should pass:

```powershell
cargo test --locked --release -p kq8-viewer --test daventry_cottage_shield dead_city::central_guard_drops_heal_and_preserve_supplies_after_reload -- --ignored --exact --test-threads=1
```

The original terrain blocks 14 sampled lines from MaceBoy1/2 to the
northern bend. The native continuation reaches the western guard
without their damage. The diagnostic uses actual enemy eye heights,
terrain plus 1250 at Connor's planned position, matched building
collision lines and terrain samples every 100 units. It omits other
objects, unmatched buildings, saved terrain changes and moving actors;
an unobstructed result does not prove live visibility. Evidence:
northern-guard-sight-band.log, northern-passage-ground.log and
western-guard-sight.log.

The first western guard attempt failed in **45.22 seconds**.
The earned Strength potion raises hand damage from 80 to 120, but
stationary charged attacks allow five incoming hits. Connor reaches
negative health on update 598, kills skeleton52 with the still-pending
charged swing on 614, and enters Die on 622. That is a failed campaign
continuation, not a verified victory. Evidence:
earned-western-passage-guard/1794031852408072000 and
western-passage-guard-native.log. The follow-up uses the existing
retreat/counterattack input sequence from the outer Mace encounter.

That follow-up and fresh-process reload **pass in 89.47 seconds**.
Skeleton52 misses on updates 505 and 529 at distances 1430 and 1370,
beyond his original 1300 reach. Three normal Mace hits defeat him;
Connor takes four damage. The original 400 experience is awarded
once on update 590. Strength starts on 430 and expires on 730:
exactly 300 updates, or 20 seconds. Hand damage returns from 120
to 80 and the consumed potion is removed.

WesternPassageReload ends at (107494.71,117003.98,192), with level
eleven stats [39,80,25,10,7512], both Rocks and 25 coins. The guard
stays gone and the reward does not repeat after reload. Original
options, other inventory and quest flags remain unchanged. Evidence:
earned-western-passage-guard/1794031981737135600/slots/slot-3.json
and western-passage-retreat-native.log under kq8-artifacts.
With `KQ8_CENTRAL_SUPPLIES_SLOT` pointing to the genuine
CentralSuppliesReload and `KQ8_GAME_DIR` set, this command should pass:

```powershell
cargo test --locked --release -p kq8-viewer --test daventry_cottage_shield dead_city::western_passage_guard_uses_earned_strength_and_reloads_after_expiry -- --ignored --exact --test-threads=1
```

The remaining winding approach, graveyard zombies and portal plates
are **not yet verified** by this earned route.

The standard workspace suite was not rerun for these ignored native
route additions; no game-engine implementation changed.

## Repeat the automated route

Set `KQ8_GAME_DIR` to the GOG installation. To resume directly at
the region arrival, set `KQ8_DEAD_CITY_ARRIVAL_SLOT` to a genuine
`DeadCityReload` save. Without it, the test builds its prior stages
from New Game or their configured earned saves.

```powershell
cargo test --locked -p kq8-viewer --test daventry_cottage_shield dead_city::entrance_archers_and_paddle -- --ignored --exact --test-threads=1
```

The verified continuation used the earned arrival save. A single
invocation from New Game through this hall has not been run.

| Test in `daventry_cottage_shield` | Optional earned input | Output |
| --- | --- | --- |
| `dead_city::four_paddle_pedestals_open_the_death_hall` | `KQ8_DEAD_CITY_PADDLE_SLOT` (`FirstHallReload`) | `HallOpened` |
| `dead_city::opened_doors_to_tile_floor` | `KQ8_DEATH_HALL_SLOT` (`HallOpened`) | `TileEntry` |
| `dead_city::saved_tile_jump_finishes_once_and_returns_to_the_strip` | `KQ8_TILE_ENTRY_SLOT` (`TileEntry`) | `TileLanding` |
| `dead_city::tile_teleport_resumes_its_flight_and_speaks_once` | `KQ8_TILE_ENTRY_SLOT` (`TileEntry`) | `TileTeleportReload` |
| `dead_city::old_tile_entrance_recovers_the_original_first_jump` | `KQ8_LEGACY_TILE_ENTRY_SLOT` (older `TileEntry`) | `RecoveredTile` |
| `dead_city::restored_letter_sequence_earns_deaths_key` | `KQ8_TILE_ENTRY_SLOT` (`TileEntry`) | `MazeKeyReload` |
| `dead_city::death_key_opens_the_spike_gate` | `KQ8_DEATH_KEY_SLOT` (`MazeKeyReload`) | `AcrossMazeGate` |
| `dead_city::hall_combat_drops_replace_the_axe_and_reload` | `KQ8_DEATH_KEY_SLOT` (`MazeKeyReload`) | `HallEquipment`, `HallEquipmentReload` |
| `dead_city::equipped_connor_defeats_the_three_maze_guards` | `KQ8_HALL_EQUIPMENT_SLOT` (`HallEquipment`) | `MazeGuardsReload` |
| `dead_city::gate_drops_and_northwest_passage_reload` | `KQ8_MAZE_GUARDS_SLOT` (`MazeGuardsReload`) | `NorthCornerReload` |
| `dead_city::northern_mace_guard_and_archers_reload` | `KQ8_NORTH_MAZE_CORNER_SLOT` (`NorthCornerReload`) | `NorthJunctionReload` |
| `dead_city::northern_drops_equip_mace_and_heal_before_the_cell_gate` | `KQ8_NORTH_JUNCTION_SLOT` (`NorthJunctionReload`) | `NorthSuppliesReload` |
| `dead_city::outer_mace_guard_retreats_after_running_and_reloads` | `KQ8_NORTH_SUPPLIES_SLOT` (`NorthSuppliesReload`) | `OuterGuardReload` |
| `dead_city::first_hall_supplies_heal_the_return_to_the_outer_corridor` | `KQ8_OUTER_GUARD_SLOT` (`OuterGuardReload`) | `CorridorSuppliesReload` |
| `dead_city::saved_western_patrol_victory_reloads_without_another_award` | Required genuine `KQ8_WESTERN_PATROL_SLOT` (`WesternPatrol`) | `WesternPatrolReload`; reload only |
| `dead_city::earned_crossbow_defeats_the_river_weeper_and_restores_its_award` | Required genuine `KQ8_WESTERN_PATROL_RELOAD_SLOT` (`WesternPatrolReload`) | `RiverWeeperReload` |
| `dead_city::earned_mace_clears_the_boathouse_guard_and_reloads` | Required genuine `KQ8_RIVER_WEEPER_SLOT` (`RiverWeeperReload`) | `BoathouseGuardReload` |
| `dead_city::four_ferryman_topics_earn_knowledge_and_reload` | Required genuine `KQ8_BOATHOUSE_GUARD_SLOT` (`BoathouseGuardReload`) | `FerrymanKnowledgeReload` |
| `dead_city::western_archers_and_rock_survive_reload` | Required genuine `KQ8_FERRYMAN_KNOWLEDGE_SLOT` (`FerrymanKnowledgeReload`) | `WesternRockReload` |
| `dead_city::first_sylph_block_lights_its_tower_and_reloads` | Required genuine `KQ8_WESTERN_ROCK_SLOT` (`WesternRockReload`) | `FirstSylphBlockReload` |
| `dead_city::all_four_sylph_plates_summon_her_once_and_reload` | Required genuine `KQ8_FIRST_SYLPH_BLOCK_SLOT` (`FirstSylphBlockReload`) | `SylphSummonedReload` |
| `dead_city::three_sylph_topics_award_experience_once_and_reload` | Required genuine `KQ8_SYLPH_SUMMONED_SLOT` (`SylphSummonedReload`) | `SylphKnowledgeReload` |
| `dead_city::sylph_to_southern_passage_preserves_campaign_and_reloads` | Required genuine `KQ8_SYLPH_KNOWLEDGE_SLOT` (`SylphKnowledgeReload`) | `SouthernPassageReload` |
| `dead_city::passage_weeper_with_earned_crossbow_survives_reload` | Required genuine `KQ8_SOUTHERN_PASSAGE_SLOT` (`SouthernPassageReload`) | `PassageWeeperReload` |
| `dead_city::second_placed_rock_is_picked_up_and_reloaded` | Required genuine `KQ8_PASSAGE_WEEPER_SLOT` (`PassageWeeperReload`) | `SecondRockReload` |
| `dead_city::earned_return_to_sylph_well_heals_through_visible_clicks_and_reloads` | Required genuine `KQ8_SECOND_ROCK_SLOT` (`SecondRockReload`) | `SylphWellReload` |
| `dead_city::central_passage_guards_with_earned_mace_survive_reload` | Required genuine `KQ8_SYLPH_WELL_SLOT` (`SylphWellReload`) | `CentralPrepared`, `CentralGuardsReload` |
| `dead_city::central_guard_drops_heal_and_preserve_supplies_after_reload` | Required genuine `KQ8_CENTRAL_GUARDS_SLOT` (`CentralGuardsReload`) | `CentralSuppliesReload` |
| `dead_city::western_passage_guard_uses_earned_strength_and_reloads_after_expiry` | Required genuine `KQ8_CENTRAL_SUPPLIES_SLOT` (`CentralSuppliesReload`) | `WesternPassageReload` |

```powershell
cargo test --locked -p kq8-viewer --test daventry_cottage_shield dead_city::restored_letter_sequence_earns_deaths_key -- --ignored --exact --test-threads=1
cargo test --locked --release -p kq8-viewer --test daventry_cottage_shield dead_city::hall_combat_drops_replace_the_axe_and_reload -- --ignored --exact --test-threads=1
cargo test --locked --release -p kq8-viewer --test daventry_cottage_shield dead_city::equipped_connor_defeats_the_three_maze_guards -- --ignored --exact --test-threads=1
cargo test --locked --release -p kq8-viewer --test daventry_cottage_shield dead_city::gate_drops_and_northwest_passage_reload -- --ignored --exact --test-threads=1
cargo test --locked --release -p kq8-viewer --test daventry_cottage_shield dead_city::northern_mace_guard_and_archers_reload -- --ignored --exact --test-threads=1
cargo test --locked --release -p kq8-viewer --test daventry_cottage_shield dead_city::northern_drops_equip_mace_and_heal_before_the_cell_gate -- --ignored --exact --test-threads=1
cargo test --locked --release -p kq8-viewer --test daventry_cottage_shield dead_city::outer_mace_guard_retreats_after_running_and_reloads -- --ignored --exact --test-threads=1
cargo test --locked --release -p kq8-viewer --test daventry_cottage_shield dead_city::first_hall_supplies_heal_the_return_to_the_outer_corridor -- --ignored --exact --test-threads=1
```
