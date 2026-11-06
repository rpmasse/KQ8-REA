# Camera shake (Q187)

Study session A, 2026-11-06. Static study of the installed GOG 1.03
`Mask.exe`, its camera command and update/save handling, and the original
Daventry/Barren scripts. Original runtime capture, manual play and native
save/reload are **not tested**. Numerical sample examples below are
derived from the traced constants, not captured original frames.

## Purpose

The beer and gong scenes wobble the camera's orientation on its own
15-Hz clock. The effect also has mode-specific interactions with ordinary
follow, fixed cut cameras and flights; it is not a world-position shake.

## Rules

1. **Command and replacement.** `KQCamera::shake seconds xMax xPeriod
   zMax zPeriod` requires exactly five arguments. All are read as C
   `atof` numbers (`-.6` works; an unparseable word becomes zero).
   The command enables one shake, sets its duration/maxima/rates, and
   resets its sample counter and both current amplitudes to zero. A
   second command replaces the first rather than summing two effects.
   It does not immediately redraw or rewind the camera's current view.
2. **Clock.** Every camera sync update advances the shake, independently
   of Connor's moved messages. These are the game's **15-per-second**
   updates, not rendered frames, wall time or distance travelled.
   `handsOff` and `follow none` do not stop it. The game's paused
   simulation clock also pauses these updates; there is no wall-time
   catch-up when play resumes.
3. **Two angular waves.** X is the camera's **pitch/tilt**, Z its
   **heading/yaw**; both maxima are **radians**. There is no shake
   displacement and no shake roll/Y component. They use the same Euler
   convention as `move KQCamera`: positive pitch looks up; positive
   heading turns anticlockwise seen from above. Pitch is around the
   camera's horizontal right axis for its current heading; heading
   turns around the world's upright Z. First person combines these
   angles with the followed actor's heading as described below.
4. **Envelope.** The amplitudes begin at zero, rise linearly toward
   the supplied maxima during the first half of the requested duration,
   and fall during the second half. Each update's rise/fall amount is
   **maximum divided by (7.5 times duration)**. The decision to rise
   is made using the sample count before incrementing it: counts less
   than **7.5 times duration** rise, later counts fall. These are
   accumulated single-precision amplitudes, not recomputed exact
   fractions; small rounding residue at the end is expected.
5. **Phase and the first sample.** After changing the amplitudes, the
   sample counter increases by one. The X sample is its new amplitude
   times **sin(sample number × xPeriod × 0.293)**; Z uses its own rate
   and amplitude the same way. There is no random phase, and the first
   sample is number **1**, not zero. **0.293 is the actual phase
   constant in radians**; do not substitute 2π/15. Despite their names,
   the period arguments multiply the phase: larger values oscillate
   faster. The approximate frequency is **0.699485975 times the supplied
   value, in cycles per second**.
6. **End sample and stop.** Before each update, a count at least
   **15 times duration** ends the shake without producing another sine
   sample. A four-second shake therefore produces samples **1–60**,
   and its following sync turns it off. Its sample-60 amplitudes are
   approximately zero (with floating-point residue). A nonintegral
   sample count is not rounded beforehand: it produces the whole
   number of samples needed to reach or pass that limit, so the
   envelope need not end exactly at zero.
7. **Stopping resets parameters, not every visible camera pose.** Stop
   clears the active flag, sample count and both amplitudes. For third
   person it restores the ordinary default pitch and heading-offset
   values, not a saved copy of the player's pre-shake tilt. In first
   person the restored pitch/roll/heading-offset are **−0.175, 0,
   3.14 radians**. The stop branch does not itself rebuild the visible
   view. Normal movement, a flight or a later explicit camera action
   can rebuild it; do not invent an unconditional cut back to the
   pose at which the shake began.

### Which view receives the waves

8. **Following in third person.** The samples replace the camera's
   current pitch and heading-offset parameters; for the ordinary
   placing/following path the pitch also receives the standard **−0.2**
   downward bias. The normal camera update then applies its existing
   aim, heading-drag, range and height rules (`camera.md`). Thus the
   heading sample participates in the normal desired-heading/error
   calculation; it is not simply an extra final rotation after that
   calculation. Repeated full camera updates during the shake can also
   advance normal height/range corrections, but there is no separate
   oscillating position term.
9. **A fixed third-person cut (`follow none`).** Each X/Z sample is
   **added to the camera's current drawn pitch/heading**. The fixed
   position and roll stay the same. The samples consequently accumulate
   as angular increments, rather than being offsets from an immutable
   pre-shake view. This is the path used by both original Barren gong
   calls. Replacing or stopping a shake does not undo accumulated view
   changes. An aim-only third-person camera uses this non-placing
   shake path too; a separate normal aiming update can subsequently
   affect the result.
10. **First person.** The samples become pitch/heading offsets with
    **−0.175 pitch and 3.14 heading** added. The view is rebuilt using
    the followed actor's heading and the existing first-person camera
    offset, not by adding the samples to world position. There is no
    random bob or added roll in this effect.
11. **Cuts, placing and flights.** `move KQCamera`, a cut, starting a
    flight or changing follow does not cancel the shake flag or reset
    its counter/amplitudes. Those actions can replace the current view
    immediately; the next sync continues the retained shake in whatever
    camera mode is then active. A flight's own step runs **after** the
    shake work in the same sync. It advances its ordinary path/roll,
    resets the heading-offset to zero and rebuilds the view. Accordingly
    the flight can overwrite the shake's earlier view update; its final
    rebuild retains the current pitch parameter but does not preserve
    that sync's Z heading-offset sample. Do not suspend the flight,
    restart the shake or sum an independent final yaw over it.
12. **Save continuation.** The camera's persisted fields include active
    shake flag, integer sample count, duration, X/Z maxima, X/Z phase
    rates and **both accumulated current amplitudes**, alongside its
    normal camera rotation/direction/mode and flight state. Restoring
    them continues with the next numbered sample and remaining ramp,
    rather than issuing the command again or deriving elapsed wall
    time. For a fixed cut, the accumulated current view must also
    survive. No exact native file ordering is asserted here; native
    original-game save/reload was not exercised in this study.

## Numbers and units

| Original calls | Duration | X maximum / rate | Z maximum / rate |
| --- | --- | --- | --- |
| Daventry `beer42.cs`, `hibeer.cs`, `hibeer2.cs` | 4 s | .01 rad / 2 | .01 rad / 2 |
| Barren `gongtp1.cs`, `gsetup.cs` | 4 s | .02 rad / 4 | .03 rad / 5 |

All **292 script entries** in the two KQ archives were read with the
existing data CLI; these were the five shake calls found. Beer is an
ordinary following scene with hands off. Gong sets third person,
`follow none`, cuts to the razetta view and tilts it before shaking.
The gong's delayed two-second event does not by itself end the
four-second shake.

| Derived sample | Beer X/Z (.01, rate 2) | Gong X (.02, rate 4) | Gong Z (.03, rate 5) |
| --- | --- | --- | --- |
| 1 | +0.0001843443 rad | +0.0006143527 rad | +0.0009944087 rad |
| 30 | −0.0095496923 rad | −0.0113337655 rad | −0.0009687461 rad |
| 60 | approximately 0 | approximately 0 | approximately 0 |

The peak **envelope** is at sample 30; the sine itself need not peak
there. For a fixed cut, summing the gong samples leaves about
**−0.0011265 pitch and −0.000000017 heading radians** after sample 60
with single-precision accumulation. This small residual is a derived
consequence, not an original runtime measurement. Exact floating-point
last bits are not claimed; a **1e−7-radian** tolerance is suitable for
the tabulated input samples.

## Data used

The five scripts above, the camera's persisted state and the ordinary
camera rules in `camera.md`. Shake parameters do not come from a
separate model or movement list.

## Edge cases

- Zero or negative duration stops at the next sync before division;
  it does not generate a sample. An unchanged command still replaces
  and restarts the effect.
- A zero phase-rate produces a zero sine on that axis; a zero maximum
  likewise produces no angular contribution. Negative maxima/rates
  invert the corresponding oscillation; no positive clamp was found.
- No cleanup callback or script Cue is sent at the end. Scripts time
  their own later actions independently.
- Malformed/nonfinite numeric inputs are outside the authored data;
  safe refusal and a log are appropriate rather than reproducing
  invalid arithmetic.

## Do not reproduce

No correction to the authored waveform is recommended. In particular,
keep 0.293, the amplitude ramp, fixed-camera accumulation and replacement
rules. Native runtime evidence could refine numerical tolerances but
the period words must not be interpreted as seconds per cycle.

## How to check

- Log every sync's sample count, two amplitudes, two sine samples,
  camera mode and resulting pose; verify 60 samples then one stop for
  each original call. With stationary Connor the count must still rise.
- Check the table at samples 1 and 30 and near-zero amplitudes at 60.
  For fixed gong cuts also check the cumulative angular sum, not just
  an overlay offset that disappears when the flag clears.
- Replace one shake halfway through: the next sample is 1 for the new
  parameters while the existing fixed-camera view remains the base
  for its next increment. Check no second effect remains active.
- Save at sample 29 and restore in a fresh process. The next sample
  is 30 and reaches the retained amplitude peak without replaying
  samples 1–29. Pause during a shake and check no samples advance.
- While shaking, separately place/cut, change follow and start a
  flight. Log the shake state remaining live, and the final pose after
  the flight's later update, rather than assuming simple addition.

## Open questions

Original runtime comparisons, native saved-slot resumption and the
complete beer/gong campaign routes remain untested. No absolute
wall-clock start delay is asserted: the first sample is the next
scheduled camera sync after the command.
