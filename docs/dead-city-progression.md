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

```powershell
cargo test --locked -p kq8-viewer --test daventry_cottage_shield dead_city::restored_letter_sequence_earns_deaths_key -- --ignored --exact --test-threads=1
cargo test --locked --release -p kq8-viewer --test daventry_cottage_shield dead_city::hall_combat_drops_replace_the_axe_and_reload -- --ignored --exact --test-threads=1
cargo test --locked --release -p kq8-viewer --test daventry_cottage_shield dead_city::equipped_connor_defeats_the_three_maze_guards -- --ignored --exact --test-threads=1
cargo test --locked --release -p kq8-viewer --test daventry_cottage_shield dead_city::gate_drops_and_northwest_passage_reload -- --ignored --exact --test-threads=1
cargo test --locked --release -p kq8-viewer --test daventry_cottage_shield dead_city::northern_mace_guard_and_archers_reload -- --ignored --exact --test-threads=1
cargo test --locked --release -p kq8-viewer --test daventry_cottage_shield dead_city::northern_drops_equip_mace_and_heal_before_the_cell_gate -- --ignored --exact --test-threads=1
```
