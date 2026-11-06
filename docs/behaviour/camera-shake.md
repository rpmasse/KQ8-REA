# Camera shake (Q187, Q193 integration follow-up)

Study session A, 2026-11-06. Static study of the installed GOG 1.03
`Mask.exe`, its camera command and update/save handling, and the original
Daventry/Barren scripts. Original runtime capture, manual play and native
save/reload are **not tested**. Numerical sample examples below are
derived from the traced constants, not captured original frames.

Q193 follow-up, 2026-11-06: the branch is selected by **aiming**, not
position-following. The shared-angle, input and F11 rules below supersede
the earlier aim-only/non-placing description. `camera.md` now gives the
first-person foundation. The ordinary moved-message deadline relative to
the independent camera timer remains a narrow open question; the camera
sync's internal order and the queue boundary are settled below.

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
   person it restores the retained default pitch and heading-offset
   values, normally **−0.2 and 0 radians**, not a saved copy of the
   player's pre-shake tilt. **Third-person roll is left alone.** In first
   person the restored pitch/roll/heading-offset are **−0.175, 0,
   3.14 radians**. The stop branch does not itself rebuild the visible
   view. Normal movement, a flight or a later explicit camera action
   can rebuild it; do not invent an unconditional cut back to the
   pose at which the shake began.

### Which view receives the waves

8. **Third person with aiming on.** The samples replace the camera's
   current pitch and shared heading-offset parameters, and pitch also
   receives the standard **−0.2** downward bias. A full normal camera
   update then runs, including its tilt reset, aim, heading-drag, range
   and height rules (`camera.md`). **Correction Q193:** this branch uses
   the **aims** switch, including an aim-only follow whose position does
   not follow. The independent **places** switch determines whether the
   calculated position is applied. Z enters the ordinary heading
   controller as detailed below; it is not an independent render-only
   overlay. Repeated full updates can also advance ordinary height/range
   calculations, without a separate oscillating position term.
9. **Third person with aiming off (`follow none`).** Each X/Z sample is
   **added to the camera's current drawn pitch/heading**. The fixed
   position and roll stay the same. The samples consequently accumulate
   as angular increments, rather than being offsets from an immutable
   pre-shake view. This is the path used by both original Barren gong
   calls. Replacing or stopping a shake does not undo accumulated view
   changes. An aim-only follow does **not** use this accumulation branch
   while its aiming switch is on. A later explicit command can of course
   replace the accumulated view.
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

### Shared angles and ordinary updates (Q193)

13. **F11 does not select another shake branch.** Normal/fixed distance
    is separate from first/third person and the aims/places switches.
    F11 keeps the active shake and sample counter. It changes the full
    update's distance behavior and whether movement resets tilt. It does
    not turn following into the fixed-cut accumulation of rule 9.
14. **The shake-triggered full update can erase sampled X immediately.**
    In third person with normal distance, the ordinary aiming update
    compares the followed object's current and retained previous base
    positions. If **|Δx| + |Δy| > 10 units** and the current pitch
    parameter differs from its retained default, it replaces the pitch
    parameter with that default (normally **−0.2**) before producing the
    view pitch. This reset does not require a new moved notification at
    that moment: retained movement is sufficient. Thus it can erase
    `X sample − 0.2` during the shake's own full update. Fixed-distance
    mode suppresses this reset. A later moved-triggered full update can
    perform it as well. Shake sampling itself is not cancelled.
15. **Z is the mouse/aim controller's shared heading offset.** It is
    replaced by the next shake sample, but a full update can use or
    replace it before then. The ordinary heading error includes it;
    after the controller's idle/catch-up/locked/turn-back action, this
    offset is also added to the resulting heading. In the locked state
    the intermediate heading is the direct aim heading, so the result
    is aim plus offset. In idle the intermediate heading is unchanged,
    so another full update can add the retained offset again. Do not
    assume one angular contribution per shake sample when other full
    updates run between samples.
16. **Normal drag and turn-back rules still own that offset.** At drag
    start, the offset may be replaced with the negative current heading
    error; because that error already includes Z, it is not computed
    independently of the sampled Z. During an existing drag the current
    shared offset is used: a new shake sample has overwritten it, and
    that existing drag does not recapture its old pre-shake offset. The
    first update after an ordinary drag release
    resets it to the retained default heading offset (normally zero)
    and changes the heading state to idle. The recentred-drag exception
    in `camera.md` retains it. With no active mouse values, the turn-back
    switch replaces it with the default offset before turning/locking.
    A live drag takes precedence over that no-drag turn-back branch.
17. **Flight and stop are separate later work.** Each flight step adds
    its position/roll step, replaces the shared heading offset with
    **zero**, and runs a full update; on completion it also clears
    turn-back and restores placing. These flight steps occur after the
    shake's sample/update in that camera sync, including its first-person
    direct redraw if first person was active. Stop alone resets only the
    shake flag/count/amplitudes and the angles specified in rule 7. It
    does not directly rebuild the view; later full updates in the same
    sync can nevertheless rebuild it. **Q193 flight clarification:**
    flight setup enables the existing turn-back switch, rather than a
    separate heading interpolator. The same ordinary four-state heading
    controller consequently supplies the one-third/lopsided-snap or
    already-locked behavior on normal non-final steps. Aiming is not
    forced on. Setup does not immediately full-update and does not reset
    that controller's history; subsequent steps do not reassert switches
    that another command changed. On completion, before the last step's
    full update, it clears turn-back, resets the ordinary heading state
    to **idle** and enables placing. That last full update alone performs
    final range/height/aim work with the current distance/input settings;
    no separate final settle or forced exact-yaw pass follows. See
    `camera.md` flight rules 4–5 for start/cut/completion details.
18. **The order inside one camera sync is fixed.** First the stillness
    bookkeeping is advanced. Next comes the shake sample and its full
    update/direct redraw, or its stop/reset when exhausted. Next a flight
    step runs and performs its full update; without a flight, a retained
    turn-back switch instead causes another full update. Finally the
    held Page tilt/zoom input is serviced and can cause another full
    update. These are distinct updates, not a single update with all
    changes combined. The later updates do not advance the shake counter
    again, but can consume the retained sampled angles, reset pitch,
    advance avoidance and change the final view.
19. **Moved notifications are separate queued work.** The moved handler
    copies the current actor transform to its previous record, accepts
    the notification's new actor transform, resets its stillness counter
    and runs a full update. It does not generate a shake sample. The
    timer group's camera sync runs to completion before the event queue
    handles notifications posted during that timer broadcast. The queue
    orders by event time and keeps insertion order at equal times. This
    establishes the boundary, but **does not establish one universal
    order for an ordinary Connor movement notification and an independent
    camera timer due in the same simulation interval**. Their exact
    relative deadlines/creation order in the ordinary route were not
    settled by two static tracing attempts. A script's immediate moved
    notification still runs after its script/event returns, as described
    in `camera.md`. Do not claim exact ordinary route phase parity from
    this follow-up.
20. **An aim-only full update retains calculated position history.**
    With placing off, leash, height and avoidance still calculate a
    candidate position. The update remembers the candidate's final
    aim-to-camera offset, updates its collision probe from that candidate,
    and in normal distance mode remembers that candidate's distance.
    This history/probe update precedes the optional `fixHeight` height
    lock: that lock can then put the candidate Z back at the current
    actual Z without recomputing the retained offset/probe.
    It then leaves the camera's actual location unchanged. The later
    aiming calculation uses that **actual unchanged location**, not the
    candidate, for its pitch/heading toward the aim point. Previous and
    current actor-transform records are updated by moved notifications
    and the stillness bookkeeping, not advanced to a candidate by every
    full update. This distinction matters to later held-mouse updates
    and to resuming position-following.
21. **Inputs are not all deferred to the last phase.** Every full update
    reads current mouse values, calculates their change from its retained
    mouse history and updates that history. A new mouse delta is consumed
    on the first such update; another at unchanged values sees zero delta
    and applies the held-drag previous-offset behavior from `camera.md`.
    The first update of a new drag consumes its initialization instead.
    Normal-distance/first-person held Page tilt is added **once in the
    final sync input phase**, then a full update runs. Fixed-distance
    held Page zoom is applied **inside** a full update, with a simulation
    timestamp guard so it changes the distance at most once at the same
    time; a later full update at that time does not zoom again. The final
    fixed-distance Page update request itself is suppressed while the
    camera mouse button is held; another cause of a full update can still
    reach the timestamp-guarded zoom. Mouse/heading history and avoidance
    do not have a general “only once per sync” guard.
22. **The direct first-person shake redraw differs from a normal full
    first-person update.** Both use the actor-relative eye offset
    described in `camera.md`. The shake redraw composes its offset
    rotation with the followed actor's **pitch and heading**, discarding
    actor roll. It skips the normal support-slope calculation, its
    unsupported-update counter, the ordinary movement tilt reset and the
    full update's tilt clamp. A later normal full update can therefore
    replace that base with the ordinary **support-slope plus heading**
    foundation, clamp/reset the current pitch and redraw without sampling
    again. F11 does not suppress the first-person movement tilt reset.

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
- For Q193, record each full update's cause, current/previous actor
  positions, actual/candidate camera positions, retained candidate
  offset, mouse history, shared pitch/Z offset, heading state and
  simulation timestamp. Compare normal distance and F11 while Connor's
  retained horizontal movement exceeds 10; normal distance can erase X
  within the shake update itself. An aim-only follow must keep actual
  position while aiming from it, and still retain candidate history.
  Unchanged mouse values must not repeat a delta; fixed-distance Page
  zoom must not repeat at one timestamp. Check a separate later full
  update after a first-person shake sample to distinguish slope handling.
  During an unmodified flight with aiming on, also log turn-back and
  heading state: turn-back begins on, completion clears it and starts
  the final full update idle. A small final yaw error can remain. With
  shake and flight together, the shake-triggered full update and the
  flight's later full update can each advance that same heading
  controller; do not add a third flight-specific yaw calculation or
  another final range/height pass.

The first-person body/list/skin presentation follows `camera.md`,
"First-person body and animations" (Q193). In particular, a first-person
shake redraw does not create an arms actor or select another movement
list: F2's Connor mode owns that selection. Actor/model placement is
settled there; exact close-up clipping/depth outcomes remain unresolved.

## Open questions

Original runtime comparisons, native saved-slot resumption and the
complete beer/gong campaign routes remain untested. No absolute
wall-clock start delay is asserted: the first sample is the next
scheduled camera sync after the command. The exact relative deadlines
and ordinary-route order of Connor's moved notifications versus the
independent camera timer remain unverified (rule 19); stop that issue
after the two static attempts above rather than infer an order from the
implementation session's scheduling.
