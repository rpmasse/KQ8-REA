# Scripted character preparation, health and trap recovery

Study session A, 2026-11-07. Static study of the original GOG executable,
checked against the installed game scripts without changing the install.
This answers the ordinary command and loading rules in Q201 and the
missing `preSpecial` / `reduceHealth` command family. Original runtime,
native save/reload and manual play **not tested**. No Rust source was
read or written for this study.

Bounded follow-up, 2026-11-07: rechecked ordinary trap addition, lookup
delivery, contact restoration and the trap's save/read path to separate
lookup notification from membership and settle partially restored saves.

## Purpose

These commands let a script prepare an actor for a cutscene, adjust
Connor's health during a scene, and recover elevator occupancy after a
world load. Their names conceal important differences: preparation does
not start a scene, and removing a saved trap occupant is different from
making a live occupant leave a trap.

The shared animation rules remain in [anim-scripts.md](anim-scripts.md)
and [character-movement.md](character-movement.md); ordinary trap contacts and persistence
are in [traps.md](traps.md), including Q203. Loading builds on
[objects.md](objects.md) and [world-change.md](world-change.md), rules
8–10 and 14–15. This note supplies the missing connections, rather than
separate rules for each authored cutscene or elevator.

## Rules

### 1. Command arguments and results

| Command | Arguments after the command name | Result |
| --- | --- | --- |
| `KQMonster::preSpecial` | One monster name or numeric object identifier | 0 |
| `KQConner::reduceHealth` | Reduction fraction and minimum-health fraction | 0 |
| `KQConner::inTrap` | One trap name | 1 when Connor is a retained live occupant; otherwise 0 |
| `KQConner::removeFromTrap` | One trap name | 0 |

The command names above retain their original spelling, including
`Conner`. Wrong argument counts print usage and return 0 without the
ordinary command effect. `preSpecial` first looks for the supplied
object name, with case-sensitive matching; if that fails, it tries the decimal numeric identifier.
The object must be a `KQMonster`; Connor qualifies. Missing or wrongly
typed objects report a failure and return 0.

Both trap commands resolve the trap **by name only**, require a
`KQTrap`, and report failure with result 0 for an absent or wrongly typed
object. They do not fall back to a numeric identifier. Object-name
lookup here is case-sensitive. That is separate from the
case-insensitive comparison of saved occupant names in rules 5–6.

### 2. `preSpecial` prepares the current motion

The effect is immediate. The actor's currently playing motion becomes
interruptible, preserving its other playback conditions, and the
character controller's remembered motion name becomes `Stop`.

This command does not play a stop motion, reset the current motion's
phase, switch the animation list, move or turn the actor, choose the
Special state, take control away from the player, or send an animation
completion notification. It does not itself install a new controller.
The commands following it perform those operations when required.

The remembered name matters to controller eligibility. In particular,
Connor's Wait controller uses the remembered Stop condition when
deciding whether a state change may proceed. Making a motion
interruptible also allows the ordinary motion-request rules to consider
replacing it. Authored transitions are still considered by those rules;
this is not a universal bypass of transitions or of every controller's
state-change restrictions.

An observation immediately after `preSpecial`, before a following state
or motion command, can therefore show the previous motion still playing
at its previous phase even though the remembered name is Stop and the
motion can be interrupted. The animation player's actual/current and
waiting motion names are separate from that remembered controller name.
If the old motion finishes without replacement, its completion notice
still identifies that old motion, not Stop. The command creates no separate timeout or
deferred scene request. Subsequent animation/controller processing owns
the next changes.

### 3. `reduceHealth` calculates a direct health adjustment

The first fraction describes a share of Connor's **maximum health**,
not a share of his remaining health. Maximum health is ten times his
current level. Both level and current integer health are sampled when
the command executes.

The requested reduction is the first fraction times maximum health,
truncated toward zero to a whole health point. Subtract that reduction
from the sampled current health. If the resulting fraction of maximum
health is less than the second argument, the intended result instead
becomes the minimum fraction times maximum health, again truncated
toward zero.

The minimum is thus an integer conversion of a fraction, not an
upward-rounded guarantee. The shipped `.02` minimum can become zero at
low levels. If current health is already below the calculated minimum,
the command can increase health to that minimum.

The numeric conversion has one relevant precision distinction: the
reduction argument keeps its decimal-to-double conversion, whereas the
minimum argument is rounded to a single-precision float before the
comparison and minimum-health calculation. Neither fraction is clamped
to 0–1 by the command, despite the usage message describing that range.
Malformed ordinary numeric text follows the original decimal conversion
and can become zero; non-finite values, overflow and a nonsensical
zero-level Connor are outside this study's supported checks.

### 4. The health adjustment is queued, not assigned immediately

The command posts an ordinary `KQStatsChangeRequestEvent` due now. Its
health delta is the intended result minus the health sampled by the
command; all other stat deltas are zero. The normal event queue delivers
it after the current event/script work, rather than changing health
inside the command.

The minimum is not recomputed when that queued event arrives. Two calls
made before either event is delivered can calculate their deltas from
the same starting health. Their later combined effect need not preserve
the minimum that each call calculated in isolation. Other intervening
health changes likewise do not alter an already queued delta.

This is a direct stats adjustment. It has no attack source or damage
kind and does not apply armour, protection percentages, random damage,
attack-immunity tests or an attack's hurt/flinch response. The rope and
dying attack guards do not suppress the direct stats-change stage; see
the separately established Q194 direct-stats rules in `traps.md`.
The ordinary stats receiver still owns maximum-health limiting, health
meter updates, any resulting death request and its state eligibility.
Health zero alone does not satisfy its strictly-negative death test.
Normal experience/level processing also still occurs; an existing
excess of experience can produce a level-up even though this event adds
no experience, as specified in [interface.md](interface.md), rule 30.
This command does not separately rewrite the relative-damage/relative-
health script variables used by other commands.

### 5. `inTrap` asks about a retained live occupant

The command looks for Connor's current object identity among the named
trap's live occupants. It does not recompute overlap, sweep a collision
shape, resolve a saved name, run a trap callback or flush the event
queue.

A geometrically overlapping Connor is not enough unless an eligible
contact has already attached him. Conversely, a retained Connor remains
reported until normal leave/removal bookkeeping releases that identity.
A saved `trappedObject` name awaiting a qualifying contact is not yet a
live occupant and by itself returns 0.

### 6. `removeFromTrap` cancels saved-name restoration

The command searches the trap's retained saved occupant names for
Connor's name, ignoring capitals. For the normal single saved Connor
entry it removes that name and reduces the outstanding restoration
count. With no matching saved name, it has no effect.

It does **not** remove Connor from the trap's live occupant identities,
unregister that live occupant's movement/deletion observations, send
`Off` or `Cue 302`, change collision participation, or relocate anyone.
In particular, calling it on a trap already holding a live Connor does
not make `inTrap` become 0. The elevator scripts deliberately call it
only after `inTrap` returns 0.

Cancelling the last outstanding saved occupant allows subsequent
eligible contacts to use the ordinary new-entry path again. The command
does not manufacture such a contact or send `On` immediately.

### 7. Saved contacts attach before later loading commands when eligible

A restored trap has saved `trappedObject` names and an outstanding
restoration count before it starts active collision participation.
While that count is nonzero, an eligible contact matching a saved name,
ignoring capitals, attaches the live identity without `On` or `Cue 301`
and reduces the count. Already retained identities are ignored. A
contact whose name does not match does not take the ordinary new-entry
path while restoration is outstanding. When all saved occupants have
reattached, the saved-name restoration list is cleared. Before that
point, the original retains even names that have already reattached;
the outstanding count, not the list length, says how many remain.

Starting an active collider checks eligible existing overlaps
**synchronously**. Resulting trap contact handling finishes before that
registration returns. For a trap being loaded, saved-name preparation
precedes its active registration, which precedes the trap's `Init`
script. Therefore an eligible stationary saved Connor can already be a
live occupant before the trap's `Init`, before `loadKQ` returns, and
before the next line of the enclosing load script.

This retains the normal active-participation, mask, collision-detail
and shape-contact requirements. A disabled or otherwise ineligible
collider does not acquire occupants just because objects have been
loaded. Loading order matters: a later eligible actor registration can
provide the contact for an earlier trap too. Unmatched names can remain
pending after `newWorld.cs` returns.

Every successfully added ordinary trap posts itself a `KQLookupEvent`
due at the current simulation time after its inline `Init` opportunity.
This posting is unconditional with respect to saved names, `lookupCall`
and script presence: a fresh trap, a restored trap with no saved names,
and a trap with no script all still post it. `lookupCall` and script
presence gate only the later script callback.

On delivery, a trap with nonzero `lookupCall` and a script invokes
`<trap> Lookup`; otherwise the notification has no trap-specific effect.
The receiver itself does not look up saved names, check overlap, attach
or remove live occupants, clear pending names, or adjust the outstanding
restoration count. A script reached through that callback can of course
issue its own commands. Thus lookup is separate from the synchronous
contact path and is not a prerequisite for `inTrap` or `removeFromTrap`.
Unmatched names can remain pending after lookup delivery as well as
after `newWorld.cs`. Pending lookup events use the persistence and merge
rules in [saved-world-events.md](saved-world-events.md); an additional
saved lookup can produce an additional enabled script callback.

### 8. World `Load` and object `Init` preserve their authored order

The existing world lifecycle applies to new games, saved games and
world arrivals: world `Init` sets up the world, world `Load` executes,
and world `Begin` runs on the first drawn frame after loading. A script
file invoked from `Load`, including `newWorld.cs`, executes inline
before the following `Load` command.

Each `loadKQ` restores or creates its object and runs its initialization
in that order before the enclosing script continues. A restored world
uses its generated kept-world object sequence; saved fields are
available to each object's `Init`. Initial active contacts can attach
saved occupants within those operations, as in rule 7. `Load` therefore
cannot be treated only as a source of shadow/palette preferences: its
remaining script statements can query and change the just-built world.

This does not call for running `newWorld.cs` or object `Init` twice, or
moving the post-`newWorld` statements to `Begin`. Ordinary posted events
remain posted events; for example an elevator animation request issued
in the remaining `Load` script is not direct animation assignment.

### 9. The two elevator callers exercise different lifecycle positions

In the installed Barren `Elevator.cs`, the elevator's `Init` queries
`portalt6.01`, stores the result in `Barren::Elevator::On`, and cancels
Connor's pending saved trap entry only when the result is 0. The
authored Barren `newWorld.cs` loads that portal trap before the elevator,
and Connor before both. This is an object-initialization caller.

In installed IceWorld `WORLD.CS`, the recovery is inside **world
`Load`**, after its inline `newWorld.cs` and the screen/palette setup.
If Connor's `UsedElevator` flag is set, the script clears it, queries
`portalt6`, and stores the result in `IceWorld::Elevator::On`. Result 1
queues the elevator's `invisible` motion. Result 0 cancels Connor's
pending saved trap entry and queues the elevator's `stop` motion. With
no UsedElevator flag, the separate saved elevator-Up condition can
still queue `stop`.

IceWorld's authored `teletrap.kq` creates the object named `portalt6`;
the authored `newWorld.cs` loads Connor early and the elevator, portal
and that trap later. The recovery runs after all those inline loads
complete. It observes already established live contacts and can repair
an unmatched saved Connor name; it does not defer that decision until
Connor moves on the first playable frame.

### 10. Persistence scope

These trap commands create no separate persistent flag. Ordinary trap
saving records names through `ntrappedObject` and numbered
`trappedObject` entries. The writer retains the existing saved-name list,
appends the current live occupants' names, and writes the resulting list.
It does not first discard unresolved names or deduplicate the result.
Therefore an unresolved saved occupant survives a later save even when
there is no live occupant, and a partially restored save contains both
the retained names and the live occupants' additional contributions.

The writer leaves that name list in place and does not change the current
outstanding restoration count. With no intervening contact, cancellation
or deletion, repeated saves append the live names again. For example,
after loading saved names `A, B` and silently attaching only A, the first
save writes `A, B, A`; a second writes `A, B, A, A`. If neither has
attached, repeated saves continue to write `A, B`. These are static
consequences of the traced writer, not measured native save roundtrips.
On a later load, the complete written name list supplies the initial
restoration count; the original contact handler counts each matching
name entry while retaining each live identity only once. See the unsafe
bookkeeping caveat below.

A pending-only Connor removed by rule 6 no longer contributes that
retained name, and the live-occupant contribution does not add him unless
he has subsequently become live. Removing a name cannot erase a live
Connor from a later save. Disk and kept-region snapshots use this same
ordinary trap writer, as established by the shared snapshot path in
[saved-world-events.md](saved-world-events.md).

## Data used

- Installed Swamp `Swhnchmn.cs`, in its death response: player control
  is disabled separately, then `preSpecial Connor`, placement and a
  Special-state request prepare the scene. Script coverage reports
  eight `preSpecial` calls across Swamp, Gnome, Barren and IceWorld.
- Temple 4 `altar.cs` has five `reduceHealth` calls, all with `.5 .02`,
  during authored Connor/Lucreto scenes. They use the same health rule.
- Loose installed Barren `patch/Elevator.cs` and IceWorld
  `patch/WORLD.CS`, plus the worlds' archived `newWorld.cs` and IceWorld
  `teletrap.kq`. The loose installed scripts take precedence over
  similarly named archived scripts.

## Representative automated checks

These are expected results derived from static evidence, not original
runtime measurements. Test the shared families rather than repeating
each authored object.

1. On a monster playing an uninterruptible motion, call `preSpecial`
   alone. Check its unchanged actual motion/phase and owning state,
   newly interruptible motion and remembered Stop condition. Then
   exercise a normal state request and an authored transition. Include
   invalid count/type and name-versus-numeric lookup cases.
2. For `.5 .02`, level 43 has maximum health 430: starting health 430
   produces intended health 215; starting 100 or 3 produces intended
   health 8. Level 1, health 1 produces intended health 0, not 1.
   Check health before and after event delivery, protection/rope
   independence, and absence of a new attack reaction. Two queued calls
   from level 43, health 430 calculate two deltas of −215 before either
   arrives; the deltas remain distinct ordinary queued adjustments.
3. Exercise a trap with no occupant, a retained live Connor, a
   pending-only saved Connor, and a matching stationary restored
   contact. Verify the 0/1 query, silent reattachment, cancellation
   without Off, and ordinary On on a later new contact after pending
   restoration has been cancelled. Vary saved-name capitals separately
   from the exact trap-name lookup.
4. Use the original Barren object-Init caller and IceWorld post-load
   caller as the two lifecycle representatives. Record object creation,
   initial contacts, Init, remaining Load statements, queued events and
   Begin in order. Require one build/init per object and both branches
   of IceWorld recovery, including its UsedElevator flag consumption.
5. Cross saved-name presence, script presence and `lookupCall` 0/1.
   Each successfully added ordinary trap queues one due-now lookup after
   Init; only the enabled scripted cases invoke `Lookup`. Deliver it with
   an unmatched saved name and verify unchanged pending/live membership.
   Merge a separately saved lookup and preserve both callbacks when
   enabled. For a partial save, preserve unresolved B alongside live A;
   the original writer's duplicate accumulation is the caveat below,
   not a reason to discard B.

## Edge cases and do not reproduce

- Do not implement `preSpecial` as an animation restart or a forced
  unconditional Special-state transition.
- Do not replace `reduceHealth` with an attack, a fraction of remaining
  health, an immediate assignment, or a minimum rounded upward.
- Do not implement `inTrap` as a fresh bounds test, or
  `removeFromTrap` as an ordinary live exit.
- The original retains already matched names while other names remain
  outstanding, and its save writer can itself introduce duplicate names.
  Repeated saves therefore need not produce a unique occupant-name list.
  Cancellation of an already reattached name, or later leave/re-entry
  while other names remain pending, can disturb its separate restoration
  count. The observed guarded elevator calls do not require these unsafe
  effects. Do not reproduce duplicate accumulation or unsafe counter/
  iteration behavior: keep live and unresolved names distinct, preserve
  their unique union when saving, and keep pending restoration consistent.
- Commands used without an existing Connor, invalid non-finite health
  fractions, allocation failure and integer overflow are not useful
  original game behavior to reproduce as crashes.

## Open questions

- Original runtime save/reload during partially completed multi-occupant
  restoration, including repeated saves, remains **not tested**. The
  name retention and append behavior above is established statically.
- Persistence of `preSpecial`'s transient remembered-motion and
  interruptibility changes across an intervening save was not traced.
- Ordering of every ordinary stay notification against first-frame work
  remains outside this study. Lookup is due now after Init and follows
  the shared startup queue service in `saved-world-events.md`; the
  synchronous initial contacts needed by the elevator callers remain
  separate.
- No original-runtime or end-to-end elevator/cutscene comparison was
  performed. The automated expectations above still need implementation
  checks; manual play remains deferred under the project plan.
