# Temple 4: Lucreto changes fighting form

## Purpose

During the last altar fight, Lucreto uses a close-combat character and a ranged character. The original GOG executable switches between the two while the fight is running. The switch uses two characters that the region has already loaded.

## Rules and numbers

1. On each regular character update in Lucreto's **Wait** or **Move** state, use the straight-line distance from his feet to Connor's feet. All three world axes count, including height. The check precedes ordinary decisions about sight, chasing and attacking. His current attack target and sight result are not prerequisites.
2. The form with a projectile switches to `LucEnd` when Connor is **strictly closer** than its `changeDist`. The form without a projectile switches to `LucEndFar` when Connor is **strictly farther** than its `changeDist`. At exactly that distance the current form continues. The default and Temple 4 value are both **2,000 world units**.
3. The reaction pause has priority over the distance check. From the end of `reactback`, the wait lasts a randomly selected duration between `reactPause` minus one second and `reactPause` seconds. Temple 4 sets `reactPause=2`, so the range is **1 to 2 seconds**. During the pause Move yields to Wait. Once the pause has expired, including at the deadline, the next eligible update tests the distance again.
4. Both directions take the outgoing form's full current world placement, including orientation. The outgoing form's gravity goes off and it is hidden. The incoming named character takes that placement, becomes visible and has gravity switched on. A switch ends that form's combat decision for the current update.
5. When `LucEndFar` takes over, its home becomes the handoff position in all three axes. When `LucEnd` takes over, it keeps its earlier home. The handoff contains no transfer of health, attack target, current movement, animation time, reaction timer or inventory from the outgoing character to the incoming one. Each remains its own character. Ordinary consequences of visibility and activation still apply (see [Hidden objects](objects.md#hidden-objects-q18) and [Character movement](character-movement.md), rule 21).

## Data used

Temple 4's `monsters.kq` lists both characters, both of type `lucend`. The far entry equips `INVITEM_LucretoFire` and guards. The close entry chases. The shared `lucend.kq` supplies the distance, reaction time and initial hidden state. The altar script moves the close form to (101180, 94750, 5938) with heading 3.14, sets both homes there, reveals the close form, hides the far form and assigns the far animation file `lucendfr.anm`.

## Edge cases and scope

The original relies on both named characters being available. If the data is incomplete, log the missing character and keep the current form. This is a recovery choice for the recreation. Immediate resumption of a particular animation or target after showing a character was not established. The regular character rules can affect that outcome.

## How to check

For either Wait or Move, start the actors on level ground. With the close form visible, compare separations of 2,000 and 2,001. Only the latter should show the far form. With the far form visible, compare 2,000 and 1,999. Only the latter should show the close form. A horizontal separation of 1,600 and height difference of 1,200 gives exactly 2,000. Increasing only the height difference by one should switch close to far. Log both forms' positions, orientations, homes, visibility and gravity at each change. Repeat a crossing during the reaction pause, then wait through its deadline.

These are expected results from a static trace of the owned GOG `Mask.exe` and a read of the shipped altar and character data. Runtime test of the original: **not tested**.
