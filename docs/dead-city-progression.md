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

```powershell
cargo test --locked -p kq8-viewer --test daventry_cottage_shield dead_city::restored_letter_sequence_earns_deaths_key -- --ignored --exact --test-threads=1
```
