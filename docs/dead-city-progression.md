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

Work through the paddle pedestals, the hall of death's letter tiles
and the encounter with Death before trying the gate again. The good
skull, the statue that produces the feather, and the scales still
need verification with earned items. The existing focused scales
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
