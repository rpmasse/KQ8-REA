# Connor's ordinary combat death (Q192)

Study session A, 2026-11-06. Sources: the installed GOG 1.03
`Mask.exe`, Connor's original movement lists/models and the existing
main-menu and vortex behavior notes. Static executable/data evidence in
our own words; original runtime, rendered death and native save/reload
continuation are **not tested**. This note extends `fighting.md` rule 29
and qualifies the timing in `main-menu.md` rule 25.

## Purpose

Describe an ordinary lethal hit, the native Die controller, its final
animation and the later game-over menu. These are separate stages. The
Temple 4 vortex uses the same completion action but skips the ordinary
death animation with `Die Now`.

## Rules

1. **Lethal means health below zero, not equal to zero.** Ordinary hit
   damage asks for Die when the resulting health is negative. A state
   request still passes the receiving controller's normal exit gate:
   - In standing **Wait**, the retained state movement name must be
     `Stop` to accept an ordinary Die request. A running attack retained
     as `Attack` therefore defers it, even though the damage has already
     happened. With no intervening replacement request, the attack's
     end handling requests `Stop`, then retries and can enter the
     pending Die in that same end handling.
   - **SwapWeapons** refuses Die until its final swap movement has
     finished. That completion enables leaving the swap; its normal
     request for Wait first admits the already pending Die. The hit does
     not forcibly truncate the swap. The completed swap determines the
     weapon mode/list in which death is then requested.
   - **Move** can accept Die without Wait's retained-`Stop` requirement.
     State entry and playback are still distinct: a death animation
     requested behind a currently uninterruptible movement can wait for
     that movement's normal end. Do not turn every animation's cut-short
     flag into a universal death-state-entry gate.
   These are the ordinary controllers relevant here, not a claim about
   every specialised state. The normal single pending-state mechanism
   still applies; an unrelated later request can affect it.

2. **Accepted Die entry sets health and its display to zero.** While a
   lethal request is still pending, stored health can remain negative.
   Its meter uses the ordinary damage-display calculation, not an early
   Die clamp: negative percentages are clipped to zero by the bar; if
   division cuts a small negative value to zero while the controller is
   not yet Die, the existing non-Die minimum can show 1 percent instead.
   For example, health -1 at level 30 gives that small-negative case.
   Actual Die entry explicitly sets both health and the meter to zero.
   This is an entry action, not a permanent override of later scripted
   stats changes (`traps.md`, Q194).

3. **Die remains the owning controller.** Ordinary death does not enter
   a Special controller with a `KillConnor` tail, or change to a second
   native Dead controller after its animation. Die refuses ordinary
   state changes and Connor's normal hit guard ignores later attacks.
   The eventual menu's dead mark is a separate game/menu condition.

4. **The last hit kind selects the movement at Die entry.**

   | Hit kind | Requested death movement |
   | --- | --- |
   | Normal | `Die` |
   | Projectile | `projdie` if the active list has it, otherwise `Die` |
   | Fire | `dropweapon`, then `firedie` |
   | Ice | `dropweapon`, then `icedie` |

   The chosen movement uses the ordinary animation request and its
   animation input lock, including normal transition/waiting behavior.
   In an ordinary first-person death the view is switched to third
   person before that movement is selected. When `dropweapon` ends, the
   Fire/Ice movement is requested; this first end does not start the
   game-over hold. A preceding old movement's end likewise does not
   complete death merely because Die now owns the controller. The
   recognized final ends are `Die`, `projdie`, `firedie` and `icedie`.

5. **Ordinary death entry does not itself turn scripted `handsOff` on
   or disable gravity, ground keeping or effective collision.** Its
   animation request locks movement input, while the death movement's
   normal root motion/physics can continue. Existing scripted restrictions
   still apply. The physical freeze and scripted hands-off change below
   occur at the final death movement's completion, not merely because
   health became negative or Die was accepted.

6. **The reincarnation setting is checked at that final completion.**
   If `KQConner::reincarnate` is nonzero then, health becomes **10 times
   the current level**, the health meter becomes 100 percent, the Die
   controller is reset and Connor returns through ordinary Wait setup.
   Wait's `Stop` request has animation lock off. This branch skips the
   three-second game-over timer and the freeze/hands-off actions below.
   It does not by itself clear a pre-existing scripted `handsOff` or
   undo arbitrary earlier script changes to physics. Do not assume the
   setting was sampled at the original lethal hit.

7. **With reincarnation off, final completion starts a three-second
   hold.** A game-over event is queued for **the completion handler's
   current simulation time plus 3.0 seconds**. At this point gravity and
   `keepOnGround` are disabled, effective collision type and mask become
   zero, and ordinary scripted `handsOff` is enabled (the normal panel/
   spinning-mask form). The world and simulation clock continue during
   the hold. Do not open the menu at the final movement's end, or start
   these three seconds at the lethal hit. The event runs when the queue
   reaches its deadline; no particular rendering-frame phase is claimed.

8. **The deadline checks that Connor is still in Die.** If so, it clears
   scripted `handsOff`, dismisses the gameplay popup, sets the menu's
   Connor-dead mark, opens the main menu and applies the normal game
   pause. New is selected; Save and Back to Game are disabled. Load is
   available under the ordinary saved-game checks. There is no separate
   automatic resurrection choice added to this menu. If Connor is no
   longer in Die, the delayed action does nothing. See `main-menu.md`
   for menu sounds, input delay and the other recovery choices.

9. **Escape/Save before and after the hold are different.** In ordinary
   full-game gameplay, Escape checks scripted `handsOff`, not the
   animation input lock, health or Die identity. With scripted hands-off
   off, Escape can open the paused live-game menu during a pending
   attack/swap death or an ordinary death movement. That route clears
   the menu's dead mark, so Save and Back to Game retain their usual
   availability. The Save action enters the normal save pipeline and
   includes Connor; it has no additional Die/health rejection in this
   path. The animation/world clock is paused while that menu is open.
   This establishes eligibility, not successful native save/reload
   continuation. Once the final completion has enabled scripted
   `handsOff`, Escape is blocked during the three-second hold. After
   automatic game over, the dead-marked menu disables Save/Back. Do not
   impose that later restriction at the first lethal hit. A pre-existing
   scripted `handsOff` can already block Escape earlier.

10. **`Die Now` skips the movement stage, not the hold.** Once Die is
    entered with that argument, health/meter are set to zero and the
    same reincarnation/completion action runs immediately. With
    reincarnation off, its three seconds and freeze/hands-off therefore
    start at Die entry. It does not take the ordinary first-person
    switch/death-movement selection path. The vortex script already
    selects third person and its own camera shot, and explicitly turns
    reincarnation off. `Now` is not evidence that every preceding
    controller's state-entry gate can be bypassed. See
    `connor-vortex-death.md` for that scripted shot.

## Numbers, units and original data

- Lethal threshold: integer health **less than 0**; entry clamp: 0.
- Maximum/reincarnated health: **10 times level**; meter: 100 percent.
- Post-animation hold: **3.0 simulation seconds**, independent of the
  authored death movement's length.
- For the earned SmallAx in the Henchman encounter, `conAxe.anm`
  requests `ca_die.kqs` from its `die` movement. The model has **45
  sequences**, with a stored loop length of approximately **2.9333
  seconds**. The movement starts at phase 0, speed 1, repeat off,
  cut-short off and no transitions. Use the normal frame player's real
  end notification; the stored length alone is not a measured
  lethal-hit-to-menu duration. Pending attack/swap time comes before
  death playback, and the separate three seconds come after it.
- Fire/Ice sequencing and Projectile fallback come from the native Die
  handling; movement models, loop settings and sounds come from the
  active original `.anm`/`.kqs` files. No Rust behavior was used as
  original-engine evidence.

## Edge cases and do not reproduce

Preserve the strict lethal threshold, deferred state entry, distinction
between animation lock and scripted hands-off, and separate menu dead
mark for the 1:1 target. Do not replace the final animation's completion
with immediate game over. Do not fabricate an extra death-animation end
for a movement that was merely waiting or overwritten. Further ordinary
hits are guarded only after Die is actually current; during a deferred
request they can still arrive and change the last recorded hit kind.
The common animation-end handling can start a newer waiting movement
before the Die completion action; the recognized **ended name** still
determines completion. The disabled gravity/ground/collision properties
are not an instruction to pause the world or all animation clocks.
Native menu eligibility does not promise a safe or faithful continuation
from every saved death state.

## How to check

Log lethal-hit time and health, owning controller, pending Die, current/
waiting/retained movement names, both input locks, physics/collision
flags, final movement-end delivery, queued deadline and menu dead mark.
Use original data and controlled event inputs for these derived checks:

- Compare health exactly 0 with below 0. For a lethal Normal hit in
  ordinary standing Stop, expect Die entry, health/meter 0 and the
  animation lock before the later physical freeze/scripted hands-off.
- Deliver the same hit during a Wait attack and during a weapon swap.
  With no newer competing request, expect the existing movement/swap
  to reach its appropriate completion before the pending Die enters.
  Compare Move's different acceptance gate separately.
- For SmallAx, check `Die`; for Projectile without `projdie`, the same
  fallback; for Fire/Ice, check that `dropweapon` ending starts the
  second movement and does not start the three-second deadline yet.
- Escape during the ordinary movement with scripted hands-off off:
  expect the live-game menu, Save/Back enabled and no simulation
  progress until returning. Repeat with pre-existing scripted hands-off
  on, and during the post-animation hold: Escape must be blocked.
- At the real final end, with reincarnation off, expect freeze plus
  scripted hands-off and a deadline three simulation seconds later.
  At that deadline, expect the paused dead-marked menu with New selected.
  With reincarnation on at completion, expect full health and Wait,
  with no game-over timer from that completion.
- Compare `Die Now`: the same final-completion actions begin at entry,
  without an ordinary death movement. Do not substitute a vortex
  victory branch or movie for the losing/game-over branch.

These are static/data-derived checks, **not tested** against an original
runtime capture. The model metadata can be read without launching the
game with `target/debug/kq8.exe shape-parts GAME/Common/ca_die.kqs --game
"..\King's Quest 8 GOG"`.

## Open questions and evidence limits

- Original runtime timing, rendered transitions and a native save/reload
  during a deferred death, death movement or hold are **not tested**.
  Exact saved pending-event/Init ordering remains Q191, not answered by
  Save being available here.
- General death-camera presentation beyond the ordinary first-person
  switch and the existing scripted vortex shot was not studied.
- This bounded follow-up does not enumerate every specialised
  controller, malformed hit kind, fabricated end notification or
  script-forced screen/save operation. No new universal event-versus-
  movement phase rule is inferred.
