# Third-person and first-person camera

Study session A, 2026-10-11; "Obstacles" added 2026-10-13; height rule
and "when the camera updates" refined 2026-10-18 (Q97, Q99); the view's
heading ("Turning the view toward Connor", replacing the wrong "swing
behind Connor"), the mouse value and Q27/Q53/Q100 added 2026-10-19.
Q193 follow-up, 2026-11-06: followed-object records, repeated full updates,
first-person eye/slope/toggle rules and shake integration refined by
static executable study. Original runtime checks for these additions
are **not tested**; `camera-shake.md` records the remaining queue-phase
unknown.
Sources: game data files, the game's own
manual (`MASK/Manual.pdf`, "Controlling the Camera"), and `Mask.exe`
(studied, described here in our own words). Covers the gameplay camera that
follows Connor. Scripted cutscene cameras (`KQCamera::addCamLoc`,
`cutToCamLoc`, `flyToCamLoc`) and camera shake are only summarised.

## Purpose

Keep Connor in view while he moves, let the player swing, zoom and tilt the
camera, avoid putting the camera under the ground, and offer a first-person
view.

## Units

Game units (very likely millimetres; see `connor-movement.md`), radians.
"Per update" means per game update; the original runs at 15 updates per
second (`KQGame::FPSLimit 15` in `console.cs`).

## Controls

From `MASK/Connor.sae`, `MASK/Camera.sae` and the manual:

| Input | Action | Effect |
| --- | --- | --- |
| Hold right mouse + move mouse left/right | `MoveCamera` | Swing the camera around Connor |
| Hold right mouse + move mouse forward/back | `MoveCamera` | Zoom in/out (normal mode) or tilt (fixed mode) |
| Shift (held) | `ModifyDn`/`ModifyUp` | While held, swap the two modes' meaning of forward/back mouse |
| Page Up / Page Down | `RotateX ±0.05` | Tilt up/down (normal mode) or zoom (fixed mode) |
| Home | `ResetX` | Put the tilt back to default |
| F11 | `CameraChangeMode` | Switch normal ↔ fixed-distance mode |
| F2 | `CameraFirstPerson` | Switch third ↔ first person |

The manual's own words: the camera "will attempt to keep Connor in the
middle of the screen"; "continuing to hold the right mouse button down
while Connor moves will maintain the camera in the same relative position";
tilt "will automatically reset itself as soon as Connor moves".

`Camera.sae` also binds letter keys (`a z s f e c o p k l j m`) to direct
camera moves and turns (`MoveX/Y/Z`, `RotateX/Y/Z`, `...2`). These shift
the camera itself by fixed steps and look like development helpers. Not
needed for normal play; keep them behind a debug switch.

## Rules

### What the camera looks at

1. The camera follows one object, normally Connor
   (`KQCamera::follow Connor`; `KQCamera::follow none` stops following,
   used around cutscenes). With `follow none` the camera stays
   **exactly** where it is, place and view, until the next `move
   KQCamera`, `follow`, fly/cut or shake (traced 2026-10-17, Q89 c).
   `follow <object> 1` only turns the camera toward the object without
   moving it (game's usage text). For `follow Connor` after a cutscene,
   see "Shake and cutscene cameras".
2. It aims at a point **1,400 units above** the followed object's base
   (Connor's collision box is 1,300 tall, so a little above his head).
3. The view direction is from the camera toward that point, with two
   adjustments (traced, Q27 b, 2026-10-19):
   - **Up/down (tilt):** set exactly every update to the player's tilt
     plus the angle up from the camera to the aim point. The player's tilt
     is −0.2 by default (negative = down), so at the default the camera
     looks **0.2 rad (about 11°) below** the aim point: Connor appears
     a little above the middle of the screen, not in it. Page Up/Down
     change the player's tilt; the rest follows the aim point.
   - **Left/right (heading):** lazy, see "Turning the view toward
     Connor": Connor is allowed to drift off the middle before the view
     turns.
4. **Start of the game in Daventry (Q27 d, from the data):** `cam3rd.kq`
   gives only the class (no `followObjectName`, so the camera follows
   nothing when it is loaded), and `newWorld.cs` ends with `KQCamera::follow
   none`. On a new game, `world.cs` (`Begin`, first time only) puts the
   camera at (275026, 197541, 14930) with tilt −0.2 and heading −1.8 (the
   same place as the `loadKQ cam3rd.kq` line) and starts Daventry's
   opening scene `stngrl.cs`, whose last part (`start3`) does `KQCamera::follow
   Connor` and `handsOff 0`. That `follow` is what hands the camera to
   Connor. A `.kq` key `followObjectName=<name>` would make a camera follow
   that object as soon as it is loaded; no camera file in the game uses
   it.
5. **When `walk.cs` / `run.cs` run (Q27 e, from the data):** they are trap
   scripts. Daventry's walk and run zones (`walktrap.kq`, `runtrap.kq`,
   `windwalk.kq`, `wizwalk.kq`, `wtrmrun.kq`, and the `*_la.cs` variants
   on the house zones) name them, so they run when Connor enters the
   zone (`On`) and set the range there (`traps.md`).

### Distance (normal mode, the default)

1. The camera does **not** sit at a fixed spot behind Connor. It stays where
   it is until the distance to the aim point goes outside a range:
   - farther than the **maximum** → pulled in to 1 unit inside the maximum
     (so it trails behind Connor like on a leash);
   - closer than the **minimum** → pushed out to 1 unit beyond the minimum.
2. The range is set by scripts with `KQCamera::setRange <min> <max>`
   (min must be smaller than max, otherwise the command is refused):
   - walking: 1,000 – 2,500 (`GAME/Kq/walk.cs`)
   - running: 1,500 – 5,000 (`GAME/Kq/run.cs`)
   - before any script sets it: 1,500 – 5,000.
3. Distances are measured in 3D, from camera to aim point.

### Distance (fixed mode, F11)

1. The camera keeps the distance it had when the mode was switched on, at
   whatever angle Connor's movement leaves it.
2. That distance can be changed by zooming (Shift + right-drag, or Page
   Up/Down) and is kept between minimum + 1 and maximum − 1. Page Up/Down
   change it by 0.05 × 2,500 = 125 units per update while held.

### Height above the ground

1. The camera wants to be at least **1,400 units above the terrain under
   the camera**, and at least 1,400 units above the followed object's base.
   The higher of the two wins (the **wanted height**).
2. **Refined 2026-10-18 (traced, Q97, Q99 c).** In one camera update,
   after the leash (step 3 of "Each camera update"), the height is worked
   out in three steps:
   1. **Lift off the ground, at once.** Let *gap* = (terrain under the
      camera's new spot + 1,400) − the camera's new height. If *gap* is
      more than 0 and not more than the range's maximum, the camera is
      put **straight at** terrain + 1,400 (no easing). If *gap* is more
      than the maximum (the camera would be very deep under the
      ground), it is put where the straight line from its old spot to
      its new spot meets the ground, and the wanted height is taken
      again at that spot. Either way the game remembers "lifted"
      (used in step 3). **Details (traced 2026-10-19, Q100 b):** "the
      ground" for that line is the **drawn surface** (the same exact
      ground test as the view test under "Obstacles", including its
      "any change of side" rule), not the floor grid; the camera is put
      **on** the hit point itself (not 1,400 above it); the wanted height
      of step 2 is then taken from the floor grid under that spot. If
      the line meets nothing, the camera stays at its deep new spot (and
      "lifted" is still remembered); steps 2–3 then work from there.
   2. **Close enough: snap.** If the camera is now within **100** of the
      wanted height, it goes straight to it.
   3. **Otherwise it only moves if Connor moved.** "Moved" here means
      that the aim point has changed since the camera's previous
      record of it by more than **20**, measured as the **absolute value
      of the signed sum**, **|Δx + Δy + Δz|** (Q193 correction; see
      "Do not reproduce"). The two actor records are maintained by
      moved notifications and stillness sync, **not by every full camera
      update**; see "Retained actor records" below. Several full updates
      can therefore use the same movement difference.
      - **Moved:** the new height = the camera's height **at the start
        of this update** + 10% × (wanted height − the height after
        steps 1–2). Both up and down (smooth rise and fall). "Lifted"
        is forgotten. Note the starting point: any height change the
        leash or the lift made in this update is thrown away and
        replaced by this step (see "Edge cases").
      - **Not moved:** the height stays as steps 1–2 left it. Only if
        the player is swinging the camera with the mouse this update
        and the camera was "lifted" in an earlier update does it snap
        to the wanted height.
   Once stillness has made the retained movement difference zero, it
   only snaps (within 100) or lifts off the ground. While an earlier
   displacement is still retained, an extra full update from shake or
   another cause can repeat easing even if Connor's feet have just
   stopped changing.
3. The terrain height under the camera comes from the world's terrain (M1).
   It is the **floor grid** (the ground under water, `water.md`), not the
   drawn surface, so over Daventry's river the camera may come down to
   1,400 above the river bed (corrected 2026-10-13).

### Swinging and zooming with the mouse

1. While the right mouse button is held, sideways mouse movement swings the
   camera around the aim point (around the vertical axis). The swing angle is
   proportional to the mouse movement, with a factor of 3π/4 per unit of the
   game's mouse value. **The mouse value (traced 2026-10-19, Q27 a):** the
   game reads the cursor's position in its window and turns it into a
   fraction of the window's size, 0 at the left (top) edge to 1 at the
   right (bottom) edge. So one unit is one window width sideways (one
   window height for forward/back): dragging across the whole width of
   the window swings the camera 3π/4 (135°), and dragging the whole
   height zooms 5,000 units. On the 800 × 600 game screen that is about
   0.0029 rad per pixel sideways and about 8.3 units per pixel of
   zoom. Not traced: the step between that fraction and the camera (so
   the signs: use the directions the user checked in the original, rule
   7 of this list), and whether the game centres the value first (see
   rule 6).
2. Forward/back mouse movement in normal mode moves the camera along the
   line to the aim point (zoom) by 5,000 units per unit of mouse value,
   whatever the current distance. The result is still kept inside the
   min/max range. If a zoom would carry the camera past the aim point to
   the other side, it is instead placed at the minimum distance on its
   original side.
3. In fixed mode (or normal mode with Shift held) forward/back mouse
   movement tilts instead, by 0.1 × 7.5 × mouse movement.
4. The first update after pressing the button uses no movement (only the
   change since the previous update counts, and there is none yet).
5. Releasing the button clears the stored mouse movement.
6. **More, traced 2026-10-19:**
   - Each mouse movement runs a camera update **at once** (not only on
     Connor's "moved" messages), with the change since the last one.
   - **Holding the button without moving the mouse** (and in any update
     of a drag where neither value changed): the camera's offset from the
     aim point is set back to the offset it had at the end of the
     previous update. So with the button held, the camera keeps exactly
     the same place relative to Connor while he walks, instead of
     trailing on the leash. This is the manual's "continuing to hold the
     right mouse button down while Connor moves will maintain the camera
     in the same relative position". (The range and the obstacle rules
     still apply afterwards.)
   - If the sideways value goes past 0.9 or the forward/back value past
     0.5 (in size), the game sends itself the "button released" action
     (all values to 0, next change counts as none) and remembers it for
     the view-turning rules. With values from 0 to 1 that would cut every
     drag in the lower half of the window, which the user does not
     report, so the camera probably gets a centred value (−0.5…0.5); this
     is **not traced**. Our build may ignore this rule.
   - A zoom that would turn the camera's direction round Connor by 3.14
     rad or more (it passed through the aim point) is refused, and the
     camera is put at the minimum distance + 1 on its old side (rule 2).
7. Directions (checked by the user in the original, Q27 f): mouse
   **down** zooms in, mouse **left** swings the camera left; with Shift,
   mouse down tilts like Page Down.

### Tilt (pitch)

1. Default tilt is **0.2 rad** (about 11°) down. The original stores it
   as −0.2: in the camera's angles negative tilt looks **down** (traced
   2026-10-17 from `move KQCamera`, see "Shake and cutscene cameras";
   answers open question 2).
2. Page Up/Down tilt at 0.1 × 3 × 0.05 = **0.015 rad per update** while held
   (0.225 rad/s at 15 updates per second).
3. Tilt is limited to ±1.57 rad (straight up/down) in third person.
4. Home returns tilt to the default.
5. Tilt also returns to the default by itself when Connor moves: in normal
   (not fixed) mode, once he has moved more than 10 units horizontally
   (measured as |Δx| + |Δy|) while the tilt is not at default, it snaps back.
   This matches the manual ("it will automatically reset itself as soon as
   Connor moves"). **Q193:** the two positions come from retained actor
   transforms, not the immediately preceding full camera update. The
   reset can consequently happen within a shake-triggered full update
   before another moved notification arrives. First person has its own
   reset rule below, which F11 does not suppress.

### Turning the view toward Connor (traced 2026-10-19; was "Swinging back behind Connor")

**Correction.** Earlier versions of this note said the camera swings
round behind Connor's back by itself. That was wrong. Nothing in the
follow camera ever moves the camera's **position** round toward his
back: the position only changes by the leash, the minimum distance, the
height rule, the mouse and the obstacle rules. What these rules turn is
the camera's **heading** (the direction it looks, left/right), so that
Connor drifts back to the middle of the screen. The camera ends up
behind a walking Connor only because the leash drags it along behind
him.

Words used here: the **error** is the angle, round the vertical, from
the direction the camera looks to the direction from the camera to the
aim point (plus the mouse offset below, normally 0), wrapped to −π…π.
Positive = the aim point is to the left (anticlockwise seen from above).
"Moving" and "still" use the notification-age bookkeeping under
"Retained actor records", rather than the number of full camera updates.

The view heading is in one of four states, kept from update to update.
It starts in state 0. Each time the view is aimed (every camera update
in third person, when the camera "aims"; see "Camera flights and cuts"
for the switches):

1. **Start catching up.** If the error is more than **0.25** and Connor
   is moving, or more than **0.5** whatever he does, the state becomes
   **catching up**.
2. **The mouse wins.** While the player is dragging with the right
   mouse button (either mouse value not 0), the state is **locked**,
   with a mouse offset: on the first update of a drag the offset is set
   so that the view keeps its present angle to Connor (offset = minus
   the error), and the view then turns with the camera as it is swung
   round. On the first update after the drag ends, the offset goes back
   to 0 and the state to **idle**. (Exception, not seen in normal play:
   if the cursor was recentred during the drag, see "Swinging and
   zooming" rule 6, the drag's offset is kept and the state stays
   locked until the next drag.)
3. **The "turn back" switch** ("When the camera updates at all", item
   3) overrides 1: the offset is reset to 0, and the state becomes
   **turning back**, or stays **locked** if it was locked in the
   previous update.
4. Then, by state:
   - **Idle:** the heading is not changed. Connor may sit up to 0.25
     rad off the middle while he moves, or 0.5 rad while he stands, with
     nothing happening.
   - **Catching up:** only if Connor is moving or the error is at least
     0.5, and he has not just switched between going forward and going
     backward (see below); otherwise the state goes back to idle and
     nothing turns. Then:
     - error under **0.02**: the state becomes **locked** (no turn this
       update);
     - error over **0.5** (either side): the heading jumps so that the
       error becomes exactly **0.25** on the same side;
     - otherwise (0.02 to 0.5): only if the error **grew** in this
       update (further from 0 on the same side than at the end of the
       previous update), the heading turns toward Connor by **1.25 ×
       the growth**. So the view slowly gains on him while he keeps
       drifting sideways (each update it takes back a quarter more than
       he drifted), and does nothing while he drifts back toward the
       middle by himself.
   - **Locked:** the heading is set exactly onto the aim point (+ the
     mouse offset), every update. It unlocks (state idle, after this
     update's exact aim) when no mouse drag is going on, the "turn back"
     switch is off, and Connor is still or has just switched between
     going forward and going backward.
   - **Turning back:** use the signed, wrapped error **wanted heading +
     shared heading offset − current view heading** as measured before
     this update's heading change. If its magnitude is at most the
     single-precision 0.02 threshold, become locked without turning this
     update. Otherwise turn toward the target, with this original
     asymmetric snap: on the **positive-error side**, take one third
     only when **magnitude / 3 > 0.21**; otherwise take the whole error.
     On the **negative-error side**, take one third only when
     **magnitude × (10 / 3) > 0.21**; otherwise take the whole error.
     These are approximately 0.63 and 0.063 rad thresholds respectively,
     tested **before** applying the turn. Equality takes the whole error.
     The stored floating-point comparisons, rather than rounded decimal
     thresholds, determine boundary cases. Snapping alone does not set
     the state locked; a later update sees the small residual and does
     so. The shared heading offset is added after this state action as
     described in `camera-shake.md`.
5. "The error at the end of the previous update", used for the growth
   test, is measured after the heading change of that update.

**"Switched between going forward and going backward":** for a nonzero
move the game compares the 3D displacement between the retained actor
positions with the facing of the **previous retained actor transform**
(Q193), not the camera's previous full-update pose. A move that goes
along his back by more than 0.1 of its length counts as "backward";
forward/sideways moves outside that condition count as "forward".
It is "switched" when this differs from the previous time it was
measured. It is only measured in the cases named above (catching up, or
locked while he is moving), so it is a rare event: in practice it means
the view stops chasing him when he starts or stops stepping back.
**Zero displacement edge:** this calculation has no zero-length guard.
Its normalization produces an unordered comparison, which takes the
backward classification in the traced path. The earlier description
that every turn in place counts forward was too broad. This edge is
static evidence, not an original runtime observation.

**What this looks like in play:** Connor walks off to one side: the
view holds still until he is a quarter radian (about 14°) off the
middle, then gains on him a little each update until he is centred,
then stays exactly on him for as long as he keeps walking. When he
stops, the next camera update (the first one of his next move) drops
back to idle. A sudden big jump (more than 0.5 rad, e.g. after a `move
KQCamera`) is cut to 0.25 rad at once.

Numbers: start 0.25 rad (moving) / 0.5 rad (any); jump back to 0.25
above 0.5; gain 1.25 × the growth; lock under 0.02 rad; "backward" when
the move is more than 0.1 along his back.

### Obstacles (studied 2026-10-13)

Words used here: the **aim point** is the point 1,400 above Connor's feet
that the camera looks at; the **offset** is the arrow from the aim point
to the camera; turning **around** Connor means turning about the vertical
line through the aim point (the camera's height is left alone).

**What blocks the view.** The test is a straight line from the aim point
to the camera. The nearest of these blocks it:

1. **The drawn ground** (the visible terrain, not the floor grid under
   water). **How it is tested (traced 2026-10-19, Q53 a):** exactly, not
   by stepping. The line is followed square by square across the
   terrain, stopping at every place where it crosses a square's edge or
   either diagonal (so every edge of the two triangles of each square,
   whichever way the square is split). At each such place the ground
   height on that edge is compared with the line's height. The first
   place where the line has changed **side** (it was above the ground at
   its start and is now below, **or** it started below and is now above)
   gives the triangle it crossed in, and the exact crossing point with
   that triangle's plane is the hit. If no edge shows a change, the end
   point is checked the same way against its own triangle. Squares
   marked as **holes** in the terrain are passed through (the side is
   simply flipped there, so a hole never counts as a hit). So a line that
   **starts under** the surface (Connor wading, the aim point below the
   drawn water/ground) is **blocked where it comes up out of the
   ground**, not at once and not only where it goes back down.
2. **Objects whose collision type has bit 0x8 or bit 0x1000.** In the
   data that means:
   - **buildings**: every building (`.dig` interior) gets type 0x28
     unless its `.kq` says otherwise, so all of them block. The `.kq`
     files that do set a type for a building use 0x88 or 0xA8 (Daventry:
     `CAVE`, `CHURCH`, `WIZHOUSE`, `zedrock1`, `zedrock2`), which block
     too;
   - **doors** and door-like objects with bit 0x1000 (Daventry: the 11
     house doors at 0x5021, the keep doors at 0x1021, `tombdoor` at
     0x5040; in other worlds also big push blocks, shelves, `deadhero`).
     A door keeps blocking after a script clears its click bit;
   - a few others with bit 0x8: Barren's `BARRTOMB` (0x29), Snowexit's
     `stnhedge` and DeadCity's `DOORWAY2`/`DOORWAY3` (0x28).

   Plain objects (default type 0x41: trees, signposts, fences, barrels),
   monsters and Connor himself do **not** block: the camera passes
   through them. Objects are tested by their box from `BOUNDS.KQ`
   (turned and moved with the object, so an open door blocks where its
   leaf now stands), buildings by their BSP. These are the same shapes
   Connor collides with. **Buildings (Q53 b, traced 2026-10-19):** the
   line is split by the BSP's planes in the usual way; a piece that ends
   up in a solid part blocks **at the point where it entered that part
   across a plane**. A piece that is in solid from the line's very start
   (the aim point inside a wall) does not block; the line can still be
   blocked by a solid part it enters later. This is what B built.

**Each camera update**, in this order (steps 1–5 are the rules above):

1. Offset from the aim point to where the camera is now.
2. Mouse swing and zoom.
3. Leash distance (max), or the fixed distance.
4. Height above the ground.
5. Push out to the minimum distance.
6. **An avoidance swing already running** (see step 8) turns the offset
   around Connor by its step. If the player is moving the camera with the
   mouse this update, the swing is cancelled instead.
7. **The view test**, from the aim point to the camera's new spot. If
   it's clear, the update is done.
8. **If it's blocked:**
   1. If the player moved the camera with the mouse this update, the
      offset goes back to the previous update's offset, shortened by 5%.
      The 5% is skipped if it would bring the camera under the minimum
      distance. So dragging the camera into a wall does not work; the
      camera only creeps in a little for every update the player keeps
      trying.
   2. Then, only when **all three** of these hold: avoidance is switched
      on (it is by default; nothing found switches it off), Connor is
      moving (see below), and Connor's feet are not under the ground at
      his spot (floor grid):
      - Take Connor's **previous retained actor aim point** (A, Q193)
        and the point
        where the view line was blocked (H). Take the point halfway
        between them (M). The new offset points from the aim point
        towards M, with the same length as before. **This is a 3D
        direction** (Q53 c, traced 2026-10-19): A, H and M keep their
        heights, so the camera can dip below or rise above its height
        here; the height rule is not applied again in this update.
      - Start a **swing**. Its step per update is 0.66 × the angle around
        Connor between the old (blocked) direction and the new one. The
        swing goes on, update after update, turning the same way, until
        the camera has turned as far as the direction from the aim point
        to A. That is, it ends **looking along the path Connor just came
        by**, which is known to be clear. It stops on the update where
        that direction is reached or passed, so it may overshoot by up to
        one step.
      - Test the view line again, along the new direction. If it's still
        blocked, the camera is **pulled in** along that direction to 100
        units short of the new hit point, and the swing is cancelled.
   3. If any of the three conditions fails, the camera stays where step
      8.1 left it, view blocked or not.
9. The result is the camera's new spot. The view test is not repeated,
   and the height rule is not applied again.

**When the camera updates at all** (rewritten 2026-10-18, traced, Q99).
The follow camera runs its update (all of steps 1–9 and the aiming) only
on these occasions:

1. **A "Connor moved" message.** Connor (any followed object) sends it
   when one of his moves is **committed** and his position changed by
   **more than 0.1** on any axis, or his facing changed (any entry of his
   turn matrix by more than 0.0001). A move that changes nothing sends
   nothing. This covers:
   - his ordinary moves, one per update while he walks, runs, falls,
     turns or is carried by an animation's steps;
   - a script's **`move Connor`** (a jump to the new spot, committed at
     once);
   - **not** an update in which he stands still (an animation with no
     steps, gravity holding him on the floor): no message, no camera
     update.
2. **`follow <object>`** itself: it sends one such message at once (so
   there is one camera update right after the command, even if Connor
   doesn't move; the aim point counts as not moved for the height rule).
3. **Every update while the "turn back" switch is on.** `follow
   <object>` switches it on when the camera faces more than 0.4 rad away
   from the object (see "`follow Connor` after a cutscene"). Starting a
   flight also switches it on (Q193 flight clarification below). It is
   switched off only when a flight ends, or by a later `follow` that
   finds the camera farther than the range's maximum. **One `follow`
   that finds both** (more than 0.4 rad off and farther than the
   maximum; Q100 c, traced 2026-10-19): the 0.4 test comes first and
   switches it on, then the distance test switches it **off**, so it
   ends **off**. A `follow` that finds the camera 0.4 rad or less off
   and within the maximum leaves the switch as it was. Nothing else
   switches it off, so after such a `follow` the camera updates every
   update, standing still or not, and its heading stays locked exactly
   onto Connor (heading state "exact", "Camera flights and cuts" rule
   5). Daventry's scripts avoid it: the camera spots they set just
   before `follow Connor` are 0.04–0.22 rad off (`stngrl.cs`,
   `wtrmwalk.cs` cue 6, `miltrap1.cs` cue 300), so in normal Daventry
   play the switch is off.
4. **Every update of a flight** (until it ends), and the one update of a
   cut ("Camera flights and cuts").
5. **The player's camera controls** (traced 2026-10-19): every mouse
   movement during a drag runs one camera update at once ("Swinging and
   zooming" rule 6); while Page Up/Down is held, the tilt changes and
   the camera updates once per update (in fixed mode they zoom instead,
   and update the same way).
6. **Camera shake** (Q187/Q193): each camera sync samples it and can run
   a full normal update even while the followed object stands still.
   Its direct first-person and no-aim redraws differ from that full
   update. Several causes can run distinct full updates in one sync;
   see `camera-shake.md` rules 13–22 for shared angles, retained candidate
   position history and per-input consumption.

**When the message arrives.** Messages go through the game's event
queue: they are handled **after** whatever is running at the moment has
finished, but still within the same update, before the frame is drawn.
An ordinary move is committed at the start of the next update (so the
camera follows one update behind the move it reacts to; it was always so
in normal play). A script's `move Connor` is committed at once, and its
message is handled right after the script (and the event that ran it)
ends.

So when a trap's `On` does `move Connor …` and then `move KQCamera …`
(Daventry's mill wheel, `wheeltp1.cs` lines 8–9), the order is:

1. Connor jumps to the script's spot; the "moved" message is queued.
2. The camera is put at the script's spot and angles.
3. The script ends; the queued message arrives; the camera runs one
   update **from the script's spot**: leash/minimum, height rule, view
   test, aiming (see "`move KQCamera` while following" in "Shake and
   cutscene cameras").
4. Next update: Connor's own move of the update the trap fired in is
   committed (he lands from the script's spot); if he moved more than
   0.1 there is one more camera update.
5. Then nothing until Connor moves again.

If Connor stands still with the view blocked, nothing changes until he
moves or the player moves the camera.

**Retained actor records (Q193, static study).** The camera retains the
followed actor's **current and previous transforms**, including base
positions and facing. On an accepted moved notification it shifts
current into previous, copies the notification into current, sets its
movement age to zero and runs a full camera update. A full update by
itself leaves **both actor records unchanged**, whether caused by shake,
turn-back, a mouse change, held Page input or a flight step. A cut/flight
setup explicitly gives both records the same authored aim point instead.

At the start of each camera sync, movement age advances. A sync starting
at age 0 marks the actor moving; a sync starting at age **3** marks it
still and replaces previous with current. Thus, from a moved notice's
age-zero reset, the copy/still transition is on the **fourth subsequent
camera sync** without another notice, not after three full updates.
The moved handler itself does **not** change that moving/still flag.
A fresh camera initializes age to **0** but the flag to **still**; its
first sync therefore changes the flag to moving even without a notice
yet. Equal retained positions still produce zero displacement. Follow
setup does not directly reset age or the flag: it copies the actor's
current transform and queues an initial moved notice, including when
the actor is stationary. That notice supplies the usual age-zero reset.
The precise original ordinary movement-notice phase relative to that
timer remains open (`camera-shake.md` rule 19).

These retained records feed distinct tests:

| Use | Test/history |
| --- | --- |
| Height easing eligibility | **|Δx + Δy + Δz| > 20** between their base positions; adding the same head height cancels |
| Ordinary third-person tilt reset | **|Δx| + |Δy| > 10**, normal distance only |
| First-person tilt reset | Same horizontal test, no active mouse tilt; independent of F11 |
| Forward/backward switch | Their 3D displacement and previous actor facing; compared with the last classification made by that helper |
| Avoidance's old aim point | Previous actor base plus head height |

The backward helper may change its remembered direction classification
when a full update calls it; that does **not** advance the actor
transforms. Likewise each full update may retain new candidate camera
offset/length history while these actor records stay fixed. Repeating a
full update can repeat height easing or a tilt reset against the same
actor displacement. Do not turn every update into a new actor sample.

#### Obstacle numbers

| What | Value |
| --- | --- |
| pull-in margin before the hit point | 100 units |
| point used for the new direction | halfway between A and H (0.5) |
| swing step per update | 0.66 × the first turn (radians) |
| shortening when dragging into a wall | ×0.95 per update |
| "still" transition | sync starting at notification age 3; fourth subsequent sync after age-zero reset |
| what blocks | the drawn ground, collision type bit 0x8 or 0x1000 |

### First person (F2)

1. **The eye is a fixed actor-relative point**, not a head-bone sample.
   Entering first person sets its offset to **(0, +50, +1200) game
   units** in the followed actor's model axes. The eye is that offset
   rotated by the actor's full rotation, plus the actor's base position.
   Because Connor's model faces **−Y**, this is 50 units behind its
   facing direction and 1,200 above its base when upright. There is no
   animation-frame head bob or terrain-height clamp of this final eye
   point in the path traced here. Camera tilt does not rotate the eye
   offset; the actor's rotation does.
2. **Entering first person sets angle offsets to (−0.175, 0, 3.14)
   radians**: pitch, roll, heading. The ordinary first-person view uses
   an actor-heading base with the support-slope pitch below, discarding
   the actor's own pitch and roll from that base. With the row-vector
   rotation convention in `models-and-textures.md`, the view rotation is
   **offset rotation × base rotation**. The heading offset is the stored
   **3.14**, not an exact π substituted by the recreation.
3. **Tilt controls and reset.** The pitch parameter is limited to
   **−0.54 through +1.57 radians** in a normal full update. Mouse
   forward/back tilts by its value change times **0.1 × 10**; a new drag
   initializes history and adds no change on that first update. If
   **|Δx| + |Δy| > 10** between the retained actor records, pitch differs
   from the retained ordinary default and the mouse's vertical value is
   zero, the parameter resets to that default, normally **−0.2**, even
   though entry and shake-stop use **−0.175**. F11 does not suppress this
   first-person reset. Held Page tilt is handled as described in
   `camera-shake.md` rule 21.
4. **Support slope (Q193; replaces the old rough summary).** It uses
   Connor's current supporting collision normal, including a building
   support, rather than independently sampling the terrain at the eye.
   Let **N = (nx, ny, nz)** be that normal made unit length, **H** its
   horizontal direction made unit length, and **B** the actor's model
   **+Y** direction rotated by the actor's heading alone. Let **a** be
   `acos(B dot H)`, limited to 0…π. The base pitch correction is
   **asin(−(nx² + ny²) / nz) × (π/2 − a) × 0.6366197466850281**.
   The last constant is the stored single-precision approximation of
   2/π. The squared horizontal components really are used; replacing
   that `asin` by the ordinary geometric slope angle changes the result.
   A flat support produces zero correction (the zero horizontal-vector
   normalization's unordered angle is treated as zero on this path).
   This is the base rotation's pitch, before composition with rule 2's
   offset rotation; do not independently add it as a final screen tilt.
5. **Slope limit and loss of support.** Before composing the view, the
   game limits the slope correction so that **pitch parameter + slope
   correction** stays within **−0.54…+1.57**. It remembers that limited
   correction while supported. The unsupported counter advances on
   **normal full first-person updates**, not every rendered frame or
   shake sample: updates 1–3 without support keep the last correction,
   update 4 and later use zero; support resets the counter and refreshes
   the correction. The direct first-person shake redraw does not advance
   this counter or recalculate slope (`camera-shake.md` rule 22).
6. Connor's first-person presentation comes from the active movement's
   model, the `...FpAnimList` entries and `ConnerFpSkinList` in
   `GAME/Kq/Connor.kq`; see "First-person body and animations" below.
   **Correction Q193:** F2's separate zero/one drawing-related change is
   the **shadow mask**, not an ordinary-body visibility switch. Do not
   implement that old description as hiding Connor's whole actor.
7. **`setThirdPerson`** (traced 2026-10-17, Q89 b) is a toggle with a
   memory, kept in the script variable
   `<World>::SetThirdPerson::WasFirstPerson`. Cutscene scripts call it in
   pairs (before and after):
   - In first person: switches to third person, forbids the first-person
     key, sets the variable to 1.
   - In third person with the variable at 1 (the second call): sets it
     to 0, allows the first-person key again and **goes back to first
     person**.
   - In third person with the variable at 0: only **flips** whether the
     first-person key is allowed. Camera place, distance and tilt are
     not touched. (`setThirdPerson 1` in third person does the same and
     sets the variable to 0.)
   - So a call without its pair leaves the first-person key locked, or
     unlocked when it should be locked.
8. **Camera toggle continuation.** Changing to first person first asks
   to follow Connor, saves the third-person offset and angle parameters,
   then installs rules 1–2. Returning restores those saved parameters.
   Both directions seed the view heading toward the followed actor's
   aim point and immediately run a full update. Neither direction
   restores an immutable old world-space camera pose: the internal
   third-person position/range calculations can continue while the drawn
   view is at the first-person eye. Changing to the mode already active
   does nothing; entering first person requires Connor to be found.
   The toggle does not clear or restart an active shake.
9. **Normal F2 eligibility.** `Connor.sae` routes F2 through Connor. His
   `handsOff` or `animSetHandsOff` guard refuses it with `no.aud`.
   Otherwise **`bCanChangeFPMode=0`** silently refuses it; the default
   permits changing. An accepted request reaches the camera toggle and
   changes Connor's own first-person mode, shadow mask and animation
   selection too. Scripted
   `setThirdPerson` keeps the paired-call memory and permission behavior
   in rule 7; do not replace it with an unconditional “set mode false”.
10. **Derived numerical checks.** With an identity actor rotation, the
    eye is base + (0, 50, 1200), independently of looking up/down.
    For a supporting unit normal approximately **(0, .5, .8660254)**,
    the base slope correction is about **−.2928428 rad** with B along
    +Y, **+.2928428** with B along −Y, and approximately zero with B
    perpendicular to H, before limits/composition. This is not a direct
    final Euler-pitch sample. These numbers are derived static checks;
    original runtime output and native save continuation are **not
    tested**. Exact relative queue phase remains open separately.

### First-person body and animations (Q193, 2026-11-06)

These are static executable and original-data findings. Original runtime,
pixel output and native save/resume checks are **not tested**. The limited
clipping questions in rule 7 remain open.

1. **One actor and one current movement model.** Normal F2 does not
   create a second arms actor or keep an independently animated copy of
   the third-person body. It changes Connor's active movement list and
   asks for his current movement name again. When that movement is
   rebuilt, its own `loop=` supplies the model and animation. That model
   supplies the body/arm and weapon parts that are drawn; F2 does not
   hide selected nodes by names such as `body`, `head` or `legs`.
   A node called `body` in a first-person shape is not evidence that the
   third-person body must also be drawn.
2. **Choose by drawn weapon mode, then item type.** With no weapon
   drawn, use the first entry of `FpAnimList` (`connerFp.anm` in the
   shipped data). With a melee weapon drawn, use `ShortFpAnimList` and
   the selected melee item's type; with a ranged weapon drawn, use
   `LongFpAnimList` and the selected ranged item's type. Third person
   uses the corresponding `AnimList`, `ShortAnimList`, `LongAnimList`.
   This is a replacement of the active list, not an additional list
   whose animation runs alongside it. Use the authored type-to-file
   mapping, not filename guesses: for example, first-person
   `SmCrossBow` maps to `Consbowfp.anm`, `ShortBow` to `Coscbwfp.anm`,
   and `NoRanged` to `connerFp.anm`. The lookup expects a matching type;
   no semantic fallback to a similarly named weapon is established.
3. **Switching modes re-requests the movement, not its old phase.**
   F2 retains Connor's gameplay state and current movement name. It
   submits that name through the ordinary movement request, transition
   and interruption rules (`connor-movement.md`, "Starting and
   stopping"). It does not force every in-progress one-shot to end.
   An uninterruptible current animation can therefore keep its existing
   model until ordinary completion/request handling rebuilds it.
   A rebuilt movement takes the newly selected list's authored start
   phase, speed, loop/interruption flags and cues. There is no copying
   of the previous model's normalized phase into its first-person
   counterpart and no saved third-person phase to resume on F2 back.
   The refresh also asks through the active movement controller, so
   handle repeated requests with those same ordinary rules rather than
   creating a second playback clock. This study does not replace the
   existing rules for a queued transition or a one-shot's ending.
4. **Arm skins replace the active model's base arm material.** First
   person chooses `skinList_firstperson` (`ConnerFpSkinList`); third
   person chooses `skinList` (`ConnerSkinList`). F2 keeps the current
   skin entry index and uses that entry of the newly active list; it
   does not choose a fresh armour combination merely because the view
   changed. The list's entry zero is the base material to replace:
   `armcon0.bmp` in first person, `con0.bmp` in third person. Other
   materials, including the weapon's own material, are not all replaced
   indiscriminately. The shipped lists have 76 entries each. Examples
   in the first-person data: mask 0 uses `armcon0.bmp`, gloves mask 1
   uses `armcon1.bmp`, boots mask 2 still uses `armcon0.bmp`, and gloves
   plus boots mask 3 uses `armcon1.bmp`. Strength uses the special
   `armconfx.bmp` entry (mask 2048), as in `potions.md` rule 25. A
   model/animation change refreshes the active skin mapping. Exact
   missing armour-combination and missing-bitmap behavior remains Q184;
   this follow-up does not settle those cases.
5. **Placement comes from the active model and Connor's drawing
   transform.** The ordinary render query submits the current movement
   model using Connor's drawing transform. Its authored mesh-node
   transforms then place its parts in that actor's space, as described
   in `models-and-textures.md`; they are not a new weapon offset copied
   from the camera's eye offset. The traced F2 and current-shape render
   paths add no second camera-relative arms transform, view-pitch
   compensation or independent third-person-body pass. For example,
   `Fp_bsatk.kqs` contains both a sword part and a `body` part; use its
   authored per-frame placement rather than attaching the ordinary
   inventory-world weapon shape at an invented hand point. This
   establishes model/actor placement, not every downstream pixel
   clipping outcome (rule 7).
6. **Shadow and conversation presentation.** Accepted normal F2 entry
   sets Connor's `shadowMask` to **0**; exit sets it to **1**, rather
   than restoring an arbitrary prior mask. The cast test consequently
   produces no Connor shadow in first person. The same first-person
   checks also suppress his ordinary gesture/lip-code presentation.
   They do not create a separate full-body model for conversation.
   F2 itself leaves the interface panels in place (`interface.md`
   rule 17).
7. **Bounded evidence limit: exact near clipping remains unresolved.**
   Two bounded static paths were followed: F2 through list/skin/movement
   changes, and the active shape through render query and actor/model
   transforms. They settle the rules above. They have **not** settled
   the numerical near clipping plane or every material/depth/occlusion
   outcome for arms and weapons close to walls or to the eye. No
   separate first-person clipping plane, depth-buffer clear, always-on-
   top weapon pass or weapon wall-retraction rule is asserted here.
   Stop that remaining clipping issue at this handoff; do not supply a
   guessed overlay policy as original parity. Native saved-game
   continuation of the switched model/skin/queued movement is also
   **not tested**.
8. **Observable/data checks.** Log the same Connor actor identity,
   drawn weapon mode, selected item type, active `.anm`, current and
   waiting movement, active shape/loop, phase and skin entry before
   and after F2. With interruptible `walk`, the new selected movement
   must rebuild using `fp_walk.kqs`; a mode switch must not preserve a
   separately ticking third-person walk. With an uninterruptible
   one-shot, distinguish list selection from the later model rebuild.
   Check `shadowMask` 1 → 0 → 1, including a pre-existing mask other
   than 1. For model placement, a data-only check is
   `target/debug/kq8.exe shape-parts GAME/Common/Fp_bsatk.kqs --game
   "../King's Quest 8 GOG"`: it reports sword and body parts with their
   own frame placements. `fp_stop.kqs` and `fp_walk.kqs` have a `body`
   part whose node translation is (0,0,798.5) in the reported first and
   last frames; the name alone must not trigger a whole-body hide rule.
   These CLI checks read original data; they do not measure native
   renderer pixels or settle rule 7.

### Shake and cutscene cameras

- `KQCamera::shake <seconds> <xMax> <xPeriod> <zMax> <zPeriod>`:
  **Q187 follow-up, 2026-11-06**, see `camera-shake.md` for the traced
  15-Hz angular waveform, 0.293 phase constant, amplitude ramp,
  fixed-cut accumulation, follow/flight interactions and save state.
- Camera flights and cuts (`addCamLoc`, `flyToCamLoc`, `cutToCamLoc`):
  see "Camera flights and cuts" below.
- **`move KQCamera <flags> x y z rx ry rz`** (traced 2026-10-17, Q89 c):
  - Flags: bit 1 = the position is **added** to the current one
    (otherwise it replaces it); bit 2 = the angles are added (otherwise
    they replace them). 0 = both absolute, 3 = both relative. Missing
    numbers count as 0.
  - Angles in radians: **rx = tilt** (positive looks up, negative looks
    down), **ry = roll** (which way it turns the picture: "Camera
    flights and cuts" rule 4a), **rz = heading**: 0 looks along world **+Y**,
    positive turns anticlockwise seen from above (π/2 looks along −X),
    the right-hand rule round Z as for objects. The view direction is
    (−sin rz × cos rx, cos rz × cos rx, sin rx). (The angle order is
    traced; that the camera looks along its own +Y is inferred, but the
    data fit it: the watermill shot's heading 2.632 points at Connor's
    `walkTo` target, 2.64; the bricks shot's 0.9 has Connor a little
    left of centre at 1.10; `miltrap1.cs`'s −0.86 is a high camera
    looking down.)
  - The camera takes that place and view **at once**, and the next frame
    is drawn straight away.
  - **`move KQCamera` while following** (traced 2026-10-18, Q99 d): the
    move itself does not run a camera update. The script's place and
    view stand until the next camera update ("When the camera updates
    at all"); with Connor standing still and the "turn back" switch off
    that may be never, until he moves. The next update starts from the
    moved spot with the normal rules:
    - position: leash/minimum, height rule (with its "moved" test),
      view test, as in any update;
    - **tilt: set at once** to the player's tilt (default −0.2) + the
      angle up to the aim point, so the script's tilt is lost in that
      update;
    - **heading: not snapped.** It goes through "Turning the view toward
      Connor": with the "turn back" switch off, it only starts
      turning when it is more than 0.25 rad off Connor (and he counts as
      moving) or more than 0.5 rad off; within 0.5 rad it is turned only
      by 1.25 × the growth of the error since the previous update, and
      beyond 0.5 rad it is turned back to 0.25 rad off at once. (With the
      switch on: a third of the rest per update, then locked.)
    - **In the later updates** (Q100 a, 2026-10-19): the same rules, update
      after update, from the state the view is in. Worked out for
      Daventry's mill wheel (first shot, 0.259 rad off): if the view was
      idle, the error is just over 0.25, so it starts catching up only
      while Connor counts as moving; from then on it turns only in updates
      where the error grows (by 1.25 × the growth) and locks once under
      0.02 rad; while he plays `wheelover` (no steps, no updates) nothing
      changes, and the flight at cue 1 takes over. If the view was locked
      before the `move` (Connor walking into the trap), it stays locked
      only if he is still moving at that update; otherwise it drops to
      idle and keeps the 0.259 rad.
  - **Daventry's mill wheel (`wheeltp1.cs`), worked out from these
    rules (not tested):** the first shot (camera put at (185500, 226520,
    9750), heading −0.3, tilt 0) gets the update of step 3 in the order
    above, then possibly the one of step 4, then none while Connor plays
    `wheelover` (no steps). In that update: the distance to the aim point
    is about 1,940, inside the range, so no leash; the wanted height is
    about 8,520 (Connor's base + 1,400; the ground under the camera,
    6,924 in our data, + 1,400 is lower); the camera is 1,230 above it,
    so no snap; if Connor's jump to the script's spot (from where he
    entered the trap) is "moved" (absolute signed sum over 20), the height eases
    once: 9,750 + 10% × (8,520 − 9,750) ≈ 9,627; the step-4 update is
    normally "not moved" (he only lands), so the height stays. The tilt
    becomes about −0.2 + (−0.63) ≈ −0.83 (looking down at him). The
    heading is about 0.26 rad off him, just over 0.25, so at most a
    small turn. The second shot (cue 4, 0.8 s later, camera put at
    (185000, 237850, 10850)) comes while he stands still: **no update at
    all**, so it stays exactly as the script put it (no leash, no height
    rule, script's angles) until the flight at cue 1.
- **`follow Connor` after a cutscene** starts from where the camera is,
  with no jump to a preset spot (traced):
  - If the camera is farther from the aim point than the range's
    maximum, it is pulled in to max − 1 at once; if nearer than the
    minimum, pushed out to min + 1 at once; otherwise it stays.
  - Heading: if it faces more than **0.4 rad** away from Connor, it turns
    toward him by **1/3 of the remaining angle per update** and snaps the
    last part (the snap point is lopsided: below 0.63 rad on one side,
    about 0.063 rad on the other; looks like an original bug, see "Do not
    reproduce").
  - Tilt (inferred): set at once to the default tilt plus the angle that
    looks at the aim point.
  - Then the normal follow rules apply: one camera update right after
    the command, then updates only on the occasions in "When the camera
    updates at all" (the "turn back" switch is the one set here when
    the heading is more than 0.4 rad off; it is cleared by the leash
    case above, i.e. when the camera was farther than the maximum).
  - The scripts use this: just before `follow Connor` they `move
    KQCamera` to a spot close to the follow view (bricks cue 10: about
    0.19 rad off), so there is no visible jump.

### Camera flights and cuts: `addCamLoc`, `flyToCamLoc`, `cutToCamLoc` (traced 2026-10-18)

Words used here: a **spot** is a camera place in the camera's list; the
**aim point** is the x y z given to `flyToCamLoc` / `cutToCamLoc`
(the game's usage text: "xyz is of point to AIM at, uses camlocs from
addCamLoc command for camera positions"). Two switches of the camera
matter, both set by `follow` (rule 1): **"places"** (the follow rules
set the camera's position) and **"aims"** (the follow rules turn the
view toward what is followed). `follow <object>` switches both on
(`follow <object> 1`: only "aims"); `follow none` switches both off.

1. **The list.** `KQCamera::addCamLoc x y z [newlist]`: if `newlist` is
   given and not 0, the list is emptied first; then the spot is added at
   the end. Without it (or 0) the spot is added to the spots already
   there. The list stays until the next `newlist` (fly and cut don't
   empty it). Data: 69 of 70 lines give `1` (5 of them as
   `$KQCamera::newLoc 1`), so nearly every cutscene uses a fresh list
   with one spot; only Dead City's `laser2.cs` adds a spot without it.
2. **Which spot.** Fly and cut both use the spot **nearest the camera's
   present position** (3D distance; on a tie the one added first).
3. **`KQCamera::cutToCamLoc x y z [roll]`** (roll in radians, 0 if
   missing; a 5th word is needed for it):
   - the camera is put on the spot at once;
   - it **stops following** (as `follow none` does for the target: Connor's
     moves no longer update it) and obstacle avoidance is switched off;
     the two switches stay as they were;
   - the aim point becomes what the camera "follows" (for the rules
     below, as if it were an object standing at the aim point: the
     camera looks 1,400 above it, "What the camera looks at" rule 2);
   - the player's tilt and heading offsets go back to the defaults
     (tilt −0.2, heading offset 0); the roll becomes `roll`;
   - the heading is set at once to look from the spot toward (x, y);
   - then one camera update runs at once with the normal follow rules:
     if "places" is on, the camera is pulled onto the range around the
     aim point (farther than the maximum → max − 1, nearer than the
     minimum → min + 1, then the height rule, settled as in rule 4's last
     update: the aim point counts as not moved); if "aims" is on, the
     view is aimed as in rule 5. With both off (after `follow none`) it stays
     on the spot, heading toward (x, y), and keeps its tilt.
4. **`KQCamera::flyToCamLoc x y z [seconds [endRoll]]`:**
   - **Duration:** `seconds` (a negative value counts as 0). If it is 0
     or missing: the distance from the camera to the spot ÷ **5,000**
     units per second. The flight takes **N = whole part of (duration ×
     15) + 1 updates** (2 s → 31 updates, 2.067 s; 3 s → 46).
   - **Path and speed:** a **straight line** from where the camera is to
     the spot, the same step every update ((spot − start) ÷ N): constant
     speed, no easing in or out. The position steps reach the spot at N
     before that update's ordinary range/height settling; that settling
     can move it away again.
   - **Roll:** from its present value to `endRoll` in N equal steps.
     `endRoll` is only read when the line has exactly x y z seconds
     endRoll; otherwise it is **0**, so a flight without it levels a
     rolled camera.
   - **At the start (Q193 clarification):** it stops following the live
     object, switches avoidance off and **turn-back on**, and gives both
     retained actor transforms the authored aim point with movement age
     zero. Pitch and shared heading offset return to their retained
     defaults (normally **−0.2 and 0**). **Placing is switched off;
     aiming stays as it was.** It keeps the current drawn heading, roll
     and ordinary heading-controller history; it does not start a second
     flight-yaw controller or immediately run a full camera update.
   - **Every update of the flight:** the step is added to the position,
     the roll step to the roll, the shared heading offset is set to
     **zero**, then a **full normal camera update** runs. Placing remains
     off unless another command changed it; aiming and turn-back use
     their current ordinary switches (rule 5). The step does not reset
     pitch or reassert turn-back/placing-off each time. A shake's X or a
     later camera command can therefore change the pitch used by that
     full update, while the step clears sampled Z.
   - **The last update (N):** before the final position/roll step and
     full update, **turn-back is cleared, the ordinary heading state is
     set to idle, and placing is switched on**. Aiming is unchanged.
     The camera is consequently pulled onto the
     range around the aim point like after a cut: farther than the
     range's maximum → pulled in to max − 1 **along the line to the
     aim point + 1,400**, nearer than the minimum → pushed to min + 1,
     then the height rule ("Height above the ground" rule 2). **Settled
     2026-10-18 (traced, Q97):** a flight or cut sets both the camera's
     present and previous record of the aim point to the aim point, so
     the aim point **counts as not moved** and the height never eases
     on this update. Only: (a) if the camera is now below the ground
     under it + 1,400 it is lifted there at once; then (b) if it is
     within 100 of the wanted height (the higher of ground + 1,400 and
     aim point + 1,400) it snaps there; otherwise its height stays.
     The same holds for a cut's one update. Avoidance is off,
     so no obstacle pull-in or swing. **So the camera can jump at the
     end of a flight** (watermill: about 1,227 units, rule 7).
     **This settling happens solely inside that full update**, with its
     usual candidate/history/probe work; there is no extra final range,
     height or yaw pass. These min/max examples assume normal distance;
     if F11 fixed distance is active, the same full update applies its
     retained fixed length instead, then its ordinary height/minimum
     rules. Later commands that re-enabled avoidance are likewise
     honored by the full update rather than overridden by flight logic.
   - **After the flight** "places" **stays on** (even after a `follow
     none`; "aims" stays as it was), so a later cut is pulled onto the
     range and given the height rule (rule 3). It stays where that last
     update put it: it follows nothing, so it isn't updated again until a `move
     KQCamera`, `follow`, cut, flight or shake (the mouse look is off
     during `handsOff`).
   - A completion notice for Connor is queued in the last-step branch,
     before that position/roll step and full update. It is handled after
     the camera sync returns, so it does not interleave with that update.
     This is an internal message used by some of his special states;
     scripts don't see a completion cue and time their later actions
     with `sendEvent`. The death-hall stones' state uses it
     (`tile-puzzle.md` rule 13. Original runtime not tested here).
4a. **The roll** (traced 2026-10-30, Q182):
   - **Axis:** the roll turns the camera about **its own line of sight**
     (the direction it looks in), after the heading and tilt; the
     camera's place and the point it looks at don't change.
   - **Size:** the number is used **as it is, in radians**: no scaling.
     −0.6 is 34.4°. (The user's "about 45°, not sure" is an estimate by
     eye; the program does not make it bigger.)
   - **Sign:** a **positive** roll leans the camera's "up" toward its
     right, as if the camera were turned **clockwise** seen from behind
     it, so the **picture turns anticlockwise** on screen (the horizon
     goes down on the right). A **negative** roll turns the **picture
     clockwise** (the horizon goes down on the left). So `azrieltp.cs`'s
     `flyToCamLoc … 2.5 -.6` ends with the picture turned 34° clockwise.
     Traced: the camera's turn is built from its three angles exactly as
     an object's turn (rule "`move KQCamera`" above), and with the roll
     alone its right-hand side tips downward for a positive value.
     Inferred, not traced: that the screen shows the camera's own right
     as right (no mirror); that is what makes the heading and tilt come
     out right in our build already. This agrees with the user's check
     of B's G-2 (B's first sign was opposite to the original's) and so
     with G-5; it disagrees with the user's first description
     ("anticlockwise"), which may have described the camera's turn
     rather than the picture's. **How to check:** in that flight, watch
     which side of the horizon drops.
   - The roll is set by `cutToCamLoc` (rule 3), changed in N equal steps
     by `flyToCamLoc` (rule 4) and set or added by `move KQCamera`; the
     follow rules never change it.
4b. **How the camera commands read their words** (traced 2026-10-30,
   Q180): every number is read with the standard C text-to-number
   reading (`atof`): leading spaces skipped, an optional sign, digits
   with an optional decimal point (a leading `.` is fine: `-.6` is
   −0.6, `5.` is 5) and an optional exponent; reading stops at the first
   character that doesn't fit, and a word with no number at its start
   counts as **0** (no error, no message). This is not `test`'s
   number rule (`doors.md` Q45 f); script variables (`$…`) are already
   replaced before the command sees the words.
   - `addCamLoc x y z [newlist]`: exactly 3 or 4 numbers (else the
     usage text); `newlist` is read as a whole number (C `atoi`: `1.9`
     is 1, `abc` is 0).
   - `cutToCamLoc x y z [roll]`: at least 3 numbers (else the usage
     text); the roll is read only when there are **exactly 4**; with 5
     or more the roll is 0 and the rest is ignored.
   - `flyToCamLoc x y z [seconds [endRoll]]`: at least 3 numbers (else
     the usage text); `seconds` when there are 4 or more (below 0 → 0);
     `endRoll` only when there are **exactly 5**; with 6 or more it is 0.
5. **Aiming during a flight or cut (Q193 correction): one ordinary
   third-person heading controller.** With aiming on, the target is
   **1,400 above the authored aim point**, viewed from the actual camera
   position. The full update sets view pitch to the **current pitch
   parameter plus the angle up** to that point; the parameter starts at
   default **−0.2**, but a shake/input/command can replace it. Roll is
   the current flight/cut roll and is not reset by third-person aiming.
   - **An unmodified flight's non-final steps have turn-back on.** In
     the ordinary four-state controller, with no mouse drag, that
     selects turning-back unless the previous full update's heading
     state was already locked; then it keeps exact lock. Turning-back
     uses the usual **1/3 of remaining error**, lopsided snap and
     **0.02-radian** lock test described above. Those are ordinary
     controller actions, not an extra calculation after the full update.
     A live drag uses the ordinary mouse branch instead. More than one
     full update in a sync can advance this controller more than once.
   - **The final step resets it to idle and clears turn-back first.**
     The final full update therefore uses ordinary idle/catch-up rules,
     rather than forcibly turning back or locking. It can leave a small
     remaining yaw error, and there is **no guaranteed exact-end yaw**.
     Do not run a dedicated flight-yaw calculation, force the previous
     lock back on, or add another final aim after this update.
   - **A cut differs at setup:** it directly seeds the heading toward
     the cut's aim point, retains the existing heading state and
     turn-back switch, then runs its one full update. It does not
     independently enable turn-back. Placing/aiming also retain their
     existing settings. A cut during a flight consequently keeps the
     old flight's step/count and current turn-back until another command
     or the flight's eventual completion changes it.
   - With aiming off, the ordinary third-person aiming calculation is
     skipped; an unmodified flight retains drawn heading/pitch while
     position and roll advance. Shake and explicit commands can still
     change those angles. First-person full updates use the separate
     foundation in "First person", not a second flight-yaw calculation.
6. **Together with other commands** (traced; none of these happen in
   Daventry's data):
   - `move KQCamera` during a flight sets place and angles at once; the
     flight's remaining steps are then added from the new place, so it
     ends away from the spot; with "aims" on, the view is turned back to
     the aim point at the next update.
   - `follow Connor` during a flight: both switches on and Connor is
     followed again; from the next update the follow rules place the
     camera behind him (the flight's steps are still added each update
     until N, then overruled by the leash).
   - `follow none` during a flight: the view stops turning; the position
     keeps flying, and at update N it is pulled onto the range around
     the aim point (rule 4).
   - **`cutToCamLoc` during a flight** (traced 2026-10-27, Q172): the
     cut does **not** end the flight. The cut is done as in rule 3 (put
     on its spot, the cut's aim point becomes the new aim point, heading
     toward it, its roll; with "places" off, which a flight has, its one
     update leaves it exactly on the spot). The flight's count of
     updates and its step stay as they were, so on each of its
     remaining updates the **old step is still added**, now from the
     cut's spot, and the old roll step is added to the cut's roll. On
     the flight's last update (update N, counted from the flight's
     start) "places" is switched on and the camera is pulled onto the
     range around the **cut's** aim point + 1,400, then the height
     rule, exactly as rule 4's last update. "Places" then stays on, so
     every later cut is pulled onto the range too. A second
     `addCamLoc … 1` does not touch the running flight either.
   - **Dead City's Shadow fight (`ShadBat.cs`), what follows from this**
     (rules applied to B's log, not seen in the original): the flight
     started at the end of `C_P2SM8` (N = 20) is still running when
     `C_P2SM9` ends 13 updates later. The cut there to (182221, 45750,
     2000) is followed by the 7 remaining steps of the old flight
     (toward the old spot (184500, 47750, 1500)), and at its last
     update the camera is pulled onto the range around (182221, 49000,
     2,264) with the height rule. At `C_P2SM10`'s end the cut puts it
     back on (182221, 45750, 2000), but "places" is now on: the spot is
     795 from the looked-at point (182221, 45000, 2,264), less than the
     range's minimum, so it is **pushed out to minimum + 1 along that
     line, then the height rule**, the same as the earlier cuts of the
     scene that were made while following (B saw about 961 and 2,264).
     That is the farther last shot the user saw in the original. The
     exact count of remaining steps (7) assumes the flight's first step
     comes on the update after the `flyToCamLoc`; B's own count of
     update numbers decides it.
   - `follow Connor` after a flight or cut works as in "`follow Connor`
     after a cutscene".
7. **Daventry's numbers (data + rules above), to check in `camera.log`:**
   - **Watermill room, `wtrmwalk.cs`** (range 1000–2500 from its own
     `setRange` at `On`):
     - cue 3: `follow Connor` (both switches on, leash clamp at once),
       `addCamLoc 190000 230000 10600 1` (the list is this one spot);
     - cue 4 (0.2 s = 3 updates later): `flyToCamLoc 186960 228000 10000
       2`: **N = 31**; step = ((190000, 230000, 10600) − camera at cue
       4) ÷ 31; roll 0 throughout;
     - looked-at point: **(186960, 228000, 11400)**; at the spot the view
       is heading **2.153**, tilt **+0.016** (−0.2 + 0.216);
     - update 31: distance from the spot to the looked-at point
       **3,725.8** > 2,500, so the camera is pulled in to 2,499 along
       that line: **(188999.0, 229341.5, 10863.4)**. The height rule
       wants 11,400 (aim point + 1,400), 536.6 higher: more than 100,
       and the aim point counts as not moved, so **the height stays at
       10,863.4** (Q97). It would only change if the ground under that
       spot + 1,400 were above 10,863.4 (ground above 9,463.4; then it
       is lifted to ground + 1,400), which B's log can check. Same view
       direction, about 1,227 units closer; it stays there until cue 6;
     - cue 5 comes 30 updates after cue 4 (one update before the flight
       ends), cue 6 at 60 updates: `move KQCamera 0 192880 230965 9260
       -.2 0 2.095`, then `follow Connor`.
   - **Mill wheel, `wheeltp1.cs`** (the windmill's other trap, Connor
     knocked under the wheel; no `follow none` in it, so both switches
     are on from normal play): cue 1 `addCamLoc 185000 213300 8800 1`,
     `flyToCamLoc 185800 211300 5610 3`: **N = 46**; looked-at point
     (185800, 211300, 7010); at the spot heading **−2.761**, tilt
     **−0.893**; distance 2,800.7, so at update 46 the camera is pulled
     in to max − 1 if the range's maximum is under 2,800.7 (walking,
     2,500: to (185086.2, 213084.6, 8607.2)), and stays if running
     (5,000). Height on that update (Q97): the aim point + 1,400 is
       7,010, below the camera, so the height stays unless the ground
       under it + 1,400 is above the camera (then lifted) or within 100
       of it (then snapped). Cue 2 comes after 2.5 s (37–38 updates, before the flight
     ends); cue 3 (4 s after cue 2): `follow Connor`.
   - **Windmill tumble, `miltrap1.cs`:** **no flight or cut**. It uses
     `follow none` and three `move KQCamera` (`anim-scripts.md` rule
     30), then `follow Connor` at cue 300. Between them the camera
     stays exactly at each moved spot (rule 1 of "What the camera looks
     at").

### `KQCamera::setCamRelConner` and `setCamRelObj` (traced 2026-10-28, batch 2)

- `KQCamera::setCamRelConner <dist> <zDir>` **does not move the
  camera.** It works out a point `dist` from Connor's position in the
  direction of his heading + `zDir` (0 = in front of him, 3.14 =
  behind) and stores it in the script variable `$KQCamera::newLoc`,
  and Connor's position in `$KQCamera::pointAt`, each as the text
  "x y z". It also prints two console lines. The scripts then run
  `KQCamera::addCamLoc $KQCamera::newLoc 1` and
  `KQCamera::cutToCamLoc $KQCamera::pointAt`.
- `KQCamera::setCamRelObj <obj> <dist> <zDir>` (Barren) is the same
  around an object.
- Not sure: the point's height (probably Connor's own z, flat) and the
  exact base direction of "heading" (log both and compare with the
  laser scene). Data: Dead City `laserctr.cs` `1500 0`, Daventry
  `ravencnr.cs`.

## Data used

- `MASK/Connor.sae`, `MASK/Camera.sae`: bindings.
- `GAME/Kq/walk.cs`, `run.cs`: `KQCamera::setRange` per movement mode.
- `GAME/Kq/Connor.kq`: first/third-person movement-list and skin-list
  mappings; the selected `...Fp.anm` files and their `GAME/Common/*.kqs`
  shapes provide first-person geometry, movement parameters and cues.
- World scripts (`daventry/kq/*.cs` and others): `KQCamera::follow`,
  cutscene commands.
- Terrain height (from M1 terrain work): the floor grid for the height
  rule, the drawn surface for the view test.
- Collision types: `colliderType` in each object's `.kq`. Buildings
  default to 0x28, other objects to 0x41 (`connor-physics.md`). Boxes come
  from `GAME/patch/BOUNDS.KQ`, building BSPs from the `.dig` files.

## Edge cases

- `setRange` with min ≥ max: refused, range unchanged (the game prints an
  error message).
- Switching walk → run widens the range; the camera does not jump, it just
  has more room before it gets pulled or pushed.
- Following `none`: camera stays where it is.
- View blocked very close to the aim point: the pull-in length is the
  distance to the hit minus 100. If the hit is less than 100 away, that
  is negative, and the original puts the camera on the far side of
  Connor (see "Do not reproduce").
- The pulled-in camera can be closer than the minimum distance. On the
  next update the minimum rule pushes it out again and the view test
  pulls it back in, for as long as the obstacle is there.
- **The height easing starts from the old height** (Height rule 2.3):
  in an update where Connor moved, the leash's vertical change and the
  lift off the ground are replaced by "old height + 10% of the gap". On
  a slope where Connor's base + 1,400 is the wanted height and the
  ground under the camera + 1,400 is above the camera's old height, the
  camera can end that update still below ground + 1,400; the next
  update lifts it again. Traced, not seen in the game.
- `flyToCamLoc` / `cutToCamLoc` with an **empty spot list**: the
  original uses a spot that was never set (see "Do not reproduce").
- `flyToCamLoc` with seconds 0 and the camera already on the spot: N = 1,
  a flight of one update (the last-update rule still applies).
- If the first turn is zero (old and new direction the same), the swing
  has a step of 0 and stays "running" without moving until the mouse or
  a new block resets it. No visible effect.

## Do not reproduce

- **The lopsided snap:** the earlier recommendation to use 0.063 rad
  symmetrically is **superseded for the user's 1:1 original-behavior
  scope** (Q193, 2026-11-06). Preserve the signed thresholds and
  comparison order in "Turning the view toward Connor" rule 4.
  Symmetric snapping would be a deliberate later change.

- **Fly or cut with no spot in the list:** the original reads a spot it
  never set (whatever was in memory). Do nothing instead (keep the
  camera where it is) and log an error.
- **The jump at the end of a flight** (rule 4, last update) is the
  original's behaviour as traced, and the user saw it in the original
  (2026-10-18): build it, so the cutscenes frame the same.

- **Per-update rates.** The height smoothing (10% per update), tilt speed
  (0.015 rad per update) and fixed-mode zoom (125 units per update) depend on
  frame rate in the original. Implement them per second so they match the
  original at 15 updates per second:
  - height: close `1 − 0.9^(15 × elapsed seconds)` of the remaining gap;
  - tilt: 0.225 rad/s; fixed-mode zoom: 1,875 units/s.

- **The "moved" test of the height rule** takes the absolute value of
  the signed sum, **|Δx + Δy + Δz| > 20** (Q193), so +300 in x and
  −300 in y still counts as "not moved". A sum-of-magnitudes test is a
  deliberate change, not an original-parity rule. Also preserve the
  notification/stillness ownership of the two actor records when
  checking original timing; extra full updates do not advance them.
- **The "turn back" switch that never clears** ("When the camera
  updates at all", item 3): after a `follow` that finds the camera more
  than 0.4 rad off, the original locks the heading onto Connor and
  updates every frame until the next flight ends. Build it as traced
  (Daventry's scripts never trigger it), but log it whenever it
  switches on, so we notice if another world's script does.
- **Negative pull-in.** When the hit is under 100 units from the aim
  point, put the camera at a small minimum distance in front of the aim
  point (e.g. 1 unit), never behind it.
- **Per-update obstacle rates.** The swing step (a fixed angle per
  update, 0.66 × the first turn) and the ×0.95 drag shortening are per
  update. Per second at 15 updates per second: the swing turns 9.9 × the
  first turn per second; the shortening is 0.95^(15 × seconds).

## How to check

- **Q193 retained records (derived, original runtime not tested):** with
  previous base (0,0,0) and current base (30,−30,0), the height movement
  test is zero but the tilt-reset horizontal test is 60. With current
  (−30,0,0), both tests pass; negative travel is not excluded by height's
  outer absolute value. Run two full updates without another moved
  notice: both actor transforms must remain unchanged. After a notice
  resets age to zero, retain their difference through the next three
  camera syncs and replace previous with current on the fourth. Check
  the fresh-camera flag separately: age zero starts still, then its
  first sync marks moving. For aim-only following, log unchanged actual
  position alongside candidate offset/length/probe history and aiming
  from actual position. Log per-full-update mouse history and fixed-
  distance Page timestamps when several causes run at the same time.
  First-person numerical examples are in that section above; distinguish
  its normal full update from the direct shake redraw.
- Log camera distance to Connor while walking in a straight line: it
  should settle at just under 2,500 (walk) or 5,000 (run).
- Log camera height above terrain on a hill: never below 1,400 for long;
  rises smoothly.
- Hold Page Up for 1 s: tilt changes by about 0.225 rad.
- Right-drag sideways: camera circles Connor at constant distance.
- Obstacles, in the original (by eye) and in our logs (numbers):
  - Walk Connor past a tree or signpost with the camera trailing behind
    it: the camera goes **through** it (trees don't block).
  - Walk round a corner of Connor's house, close to the wall: the camera
    should never end up inside the house. It swings round to look along
    the way he came, and comes closer when the swing isn't enough. Log:
    blocked yes/no, the hit point, the swing step and what's left of it,
    the pulled-in distance (100 short of the hit).
  - Stand still with the house between the camera and Connor: nothing
    happens until he moves.
  - Hold the right mouse button and drag the camera into a wall: it
    doesn't go in. It creeps closer by 5% per update while you keep
    dragging, never closer than the minimum (1,000 walking).
  - A closed door between the camera and Connor blocks the view; an open
    one only where its leaf now stands.
  - **Checked in the original by the user (2026-10-13):** (1) the camera
    flies through trees; (2) walking round a house corner, it never goes
    through the wall, swings round, and comes closer when it has to; (3)
    dragging the camera into a wall is blocked: it comes closer to Connor
    instead of going inside, and "feels very stuck and resistant to
    movement". That fits the rule: every update the drag is refused, only
    the 5% creep gets through. Keep that feel (it's the original's); a
    smoother drag is an M7 improvement, not a fix. Not checked: standing
    still with a blocked view, doors, the exact numbers.
- **Camera flights (our logs, B):** in `camera.log`, per update of a
  flight: update k of N, position, step, roll, heading and tilt, the
  "places"/"aims" switches. Watermill (`cargo play --world Daventry --at
  192408,227039,0 --frames 3000 --sound off`): N = 31 from cue 4; the
  position changes by the same step every update; at update 31 the
  pull-in to (188999.0, 229341.5, ~10863) (rule 7); heading 2.153 and
  tilt +0.016 once locked on; then no change until cue 6.
- **Camera flights (original, user, by eye):** in the watermill room's
  first-visit cutscene, the camera should glide in a straight line at an
  even speed for about 2 s, turning to look into the room, and then
  **jump about a third of the way closer** along its view at the end.
  **Confirmed by the user in the original (2026-10-18):** the camera
  jumps toward the end of the flight. (The size of the jump was not
  measured.)
- **When the camera updates (our logs, B):** log each camera update
  with its cause (Connor moved / `follow` / turn-back switch / flight /
  player control) and the height step taken (lifted / snapped / eased /
  kept, with the signed sum). Standing still on a slope after walking:
  updates stop 1 update after he stops (the last committed move), and
  the height stops changing at once even if it is far from the wanted
  height. Mill wheel (`wheeltp1.cs`): at most 2 updates after the
  trap's `On`, none from then to cue 1; the second shot logged at
  exactly (185000, 237850, 10850), tilt −0.25, heading −2.8.
- **Mill wheel (original, user, by eye):** the second shot (0.8 s after
  Connor is knocked over) should hold perfectly still, high up (the
  script's spot), until the flight; the first shot looks down at Connor
  from about the script's height.
- In the original game, compare screenshots of the same spot with the same
  inputs (you can do this by eye; we only compare numbers we log).

## Open questions

1. How mouse movement (pixels) becomes the game's mouse value: mostly
   answered 2026-10-19 ("Swinging and zooming" rule 1: one unit = one
   window width or height). Still open: the step between the window
   fraction and the camera (signs, and whether the value is centred; see
   rule 6 there). The user's checked directions stand.
2. (Answered 2026-10-17: yes, negative tilt looks down; see Tilt rule 1.)
3. (Answered: tilt resets after Connor moves more than 10 units, in both
   views; see Tilt rule 5.)
4. (Answered 2026-10-13, see "Obstacles".) What switches avoidance
   off: answered 2026-10-18: `follow none`, `flyToCamLoc` and
   `cutToCamLoc` switch it off, `follow <object>` switches it on again
   ("Camera flights and cuts"). Connor's "moved" signal: answered
   2026-10-18 (Q99 b): only when his position changes by more than 0.1
   or his facing changes ("When the camera updates at all"). Still
   open: the turn direction of the swing is inferred from the signs of
   the two angles, not checked in the game.
5. Answered by static study 2026-11-06 (Q193), "First person": fixed
   actor-local eye (0, 50, 1200), support-normal slope formula, limits,
   three-update unsupported hold, angle composition and toggle/lock
   behavior. The bounded body/animation follow-up also settles active
   model/list/skin selection, restart/request rules, actor-space model
   placement and the shadow mask. Exact close-up clipping/depth outcomes
   remain unresolved after two static paths; original runtime and native
   save/resume are **not tested** ("First-person body and animations").
6. Whether the camera updates at the same rate as Connor: answered
   2026-10-18 (Q99): it updates when he sends a "moved" message (one
   per committed move that changes something), handled one update after
   his ordinary moves; see "When the camera updates at all". The
   player's controls: answered 2026-10-19 (item 5 there).
7. The automatic swing behind Connor: answered 2026-10-19. There is no
   swing of the camera's position; the rules turn the view's heading
   ("Turning the view toward Connor").
