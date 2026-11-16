# IceWorld dragon body and heads

## Scope and sources

This note covers the installed GOG 1.03 IceWorld dragon data and the
original executable's `KQDragonBody` and `KQDragonHead` class methods.
It records rules supported by the authored files and class/state-method
trace. No original gameplay was run. Unresolved behavior is listed at
the end, rather than inferred from the current Rust viewer.

The relevant installed files are IceWorld's `monsters.kq`,
`dragbody.kq`, `draglhed.kq`, `dragrhed.kq`, their three `.anm` files,
`draglfir.kq`, `dragrfir.kq`, `dragwall.cs`, and the shared
`Monstats.mtl`. `emitters.md` rule 11 establishes how the two fire
emitters follow the heads' `fire` mesh part.

## Creation and animation ownership

The monster list creates three distinct actors: `dragbody` at
approximately (182334, 490176, 30902), `draglhed` at
(182734, 488910, 32300), and `dragrhed` at
(182000, 488910, 32200). The two heads belong to the authored
`dragteam`; the body has no authored team. The executable's class
hierarchy makes `KQDragonBody` and `KQDragonHead` sibling subclasses
of `KQMonster`, with the common animation/object ancestry. Both
constructors run the common monster constructor before installing
their specialized methods. Each monster therefore owns its own
position, health, animation list and state. The class constructors
and setup do not establish a parent-transform link between the heads
and body. A head state later looks the body and both heads up by their
authored names for the encounter-ending transition.

The body reads `dragbody.anm`; the heads read `draglhed.anm` and
`dragrhed.anm` respectively. These files provide separate models and
movement timing. Body `attack`, `react`, `stop`, and `walk` all use its
30-frame stop model/sequence and carry no `Do` cue. Its `die` and
elemental death entries use the 108-frame death model. Each head has
`attack` (bite), `attack2` (fire), `die`, `fire`, `lopt`, `react`,
`rebirth`, `stop`, `synge`, and `walk` movements. Each head's attack
and attack2 carries one `Do` cue. The left cues are at normalized
phases 0.615385 and 0.565217; the right cues are at 0.526316 and
0.608696. The heads' `fire` movement has no hit cue; fire damage is
sent by the attack2 cue, while the configured fire emitter supplies
the effect. The heads' death, fire-death and ice-death entries share
their respective cut model. The original method table retains the
common drawing path; no distinct dragon draw override was found.

## Initial state and collision

`Monstats.mtl` gives the body 300 health, 25 hand damage, 100
protection, **50,000 experience**, alarm 4000, reach 2000 and a
1-second attack delay. Each
head has 600 health, 200 hand damage, 25 protection, alarm 6000 and
reach 6000; the left/right attack delays are 1.5/1.7 seconds. The
body's file sets `isActive3D=1`, `noTurn=1`, `monUseGravity=0`,
`colliderType=1`, and `colliderMask=0`. Each head's file sets
`isActive3D=1`, `noTurn=1`, and `colliderMask=0`. The head constructor
also sets a special collision type containing the projectile-immunity
bit described in `ranged-weapons.md`. These are the authored/initial
values. For the ordinary collision pair rule, mask zero means the
dragon actor is not told about a touch and its own mover cannot use
other objects or terrain as obstacles (`character-movement.md` rule 35;
`characters.md` rule 8). It does **not** erase the actor's collision
type: Connor's and a projectile's own masks can still select that type
for a hit, if the dragon has a collision shape (`connor-collision.md`
rules 2–4; `ranged-weapons.md` rule 29). The heads' type contains the
projectile-immunity bit, so a projectile contact sends zero damage
(`ranged-weapons.md` rule 30); the body's type 1 does not contain that
bit, although its 100 protection blocks ordinary health loss. The
body has no gravity, so
the lack of a terrain bit does not make it fall. The installed bounds
list has `dragonbody` and `dragonheadleft`/`dragonheadright` entries,
but no literal `dragbody`, `draglhed` or `dragrhed` entry; how these
names are resolved to active collision shapes remains untraced. Thus
the exact physical obstruction and projectile interception in this
encounter are not claimed from the mask values alone.

The head's property reader accepts `firstLoad`, `deadPause`,
`fullHealth`, `reallyDead`, `attackLoc`, `biteRad`, `fireRad`, and
`fireFile`. Initial defaults include `firstLoad=1`, `deadPause=3`,
`biteRad=250`, and `fireRad=500`; the shipped head files replace
`biteRad` with 3000, set `fireFile` to their side's emitter file, and
repeat `deadPause=3`. On the first setup pass, the head clears
`firstLoad` and records its current monster health as `fullHealth`.
If it was already marked `reallyDead`, that setup also contributes to
the encounter's count of permanently defeated heads. The precise
disk-load ordering for this pass remains open.

## Targeting, attacks and hit delivery

The heads enter a specialized version of the ordinary monster fight
state. It keeps the common monster's target and timing decisions and
replaces the movement chosen once an attack is due. The common alarm
rule in `character-movement.md` rules 20, 23–26 applies: an active head
that sees Connor within its 6000-unit alarm radius targets him, and
the two heads' `dragteam` can spread that alarm. Their type's `none`
chase mode gives them no chase radius, so they stay at their authored
positions and fight when Connor is in range. The fight-state trace
confirms that its shared update invokes the head's specialized attack
selector only after it has a target and Connor is within its ordinary
6000-unit attack distance.

When the head is able to start that movement, the selector's random
branch chooses `attack` (bite) for one quarter of its possible random
outputs and `attack2` (fire) for the other three quarters. That is a
nominal 25%/75% choice, not a forced alternation. `attack2` also creates the emitter
named by that head's `fireFile`; `attack` does not. The visual emitter
follows the named head at its animated `fire` part, as established in
`emitters.md` rule 11. There is no separately traced emitter contact
damage in this dragon path.

At a head's `Do` cue, the specialized handler asks its current target
for a position. It compares straight three-dimensional distance from
the head to that position. With `attack`, a target within `biteRad`
(3000 in the shipped files) receives a `Normal` hit carrying the
head's current hand damage. With `attack2`, the comparison uses the
head's ordinary reach (6000 in `Monstats.mtl`) and sends a `Fire` hit
of the same current hand damage. A target beyond the relevant radius
gets no hit. The dedicated cue handler does not repeat the common
front-facing test described for ordinary monsters in `fighting.md`;
the earlier attack-state selection and target acquisition have their
own gates. When the cue is not the head's attack `Do`, it delegates to
the common animation event handler. The head accepts an `attackLoc`
property, but its specialized movement selector and `Do` hit handler
do not read that point. The shipped head files do not set it; the
default is zero. Thus it does not displace the target check in the
traced shipped attack path. Any other use of this property is open.

The body keeps the common monster attack-state machinery; its authored
`attack` movement has no `Do` cue. The common monster melee hit is
delivered at that cue (`fighting.md` rule 13), so this authored attack
does **not** deal its 25 hand damage. No independent body contact-damage
handler or specialized incoming-hit/damage rule appears in the body
class methods: a physical touch alone sends no damage to Connor. A
script could still request a hit by another route; none was established
for this encounter.

The body's collision type is 1, so the ordinary Connor click filter
for types with bit 0x20 or 0x40 (`ranged-weapons.md` rule 32) does not
select it for a hand swing. If a projectile or direct scripted hit
does reach it, the common monster hit rules apply. Its shipped
protection is **exactly 100**; under `fighting.md` rule 16, ordinary
hits of any damage kind then cause **zero health loss**, although
other common hit bookkeeping can still run. The body therefore does
not independently die from ordinary shipped combat damage. A script
that changes its protection or death state would be a separate path;
no such IceWorld command was established here.

## Head defeat, rebirth and encounter completion

The heads replace the common death state. Entry sets that head's
health to zero, removes its ordinary collision, and chooses one of
two routes by checking Connor's current fighting mode and equipped
hand-weapon slot at death-state entry. The permanent route requires
melee mode (`doMode=0`) **and** the Flame Sword (inventory item 1110)
in that slot. The check does not inspect the killing hit's damage kind.
Without both conditions, the head plays `lopt`, a
looping disabled movement, and a deadline is set from `deadPause`.
After that deadline, it requests `rebirth`, restores health from the
recorded `fullHealth`, and on completion restores ordinary collision
groups and enters its attack state. This is a temporary defeat. The
head's death-state property reader includes its completion deadline
and `rebirthed` flag, as well as the disabled-pose flag.

With both permanent-route conditions, the head marks `reallyDead`, requests
`synge`, and contributes to a shared permanent-defeat count. The
encounter-ending branch runs when that count exceeds one. It **first**
asks the named `dragbody` to enter its death state, **then** loads the
authored `dragdbox.kq` object, **then** removes `draglhed`,
`dragrhed`, and `dragwall` by name, in that order. The box is loaded
at the second head's permanent-defeat transition, not at the body's
animation completion. Its file names `dragwall.cs` as its script,
sets collision type 33, and locates it at (182974, 490884, 30902).

The head requests the body's **Die state directly**, without an
incoming attack. Its generic Die-state entry sets the body's health
to **zero**, starts its authored `die` movement, removes it from the
active monster list, disables its movement and gravity, and sets both
collision type and mask to **zero**. This is a dying actor, not a new
corpse object. The state entry runs the ordinary item-drop helper and
may schedule the normal once-only first-kill tip for Connor if no
earlier non-chicken kill has done so. It does not add a hit count,
play an ouch, or invoke an unseen-hit surprise line. The shipped body
has no held or dropped item, so its item-drop call produces none.

When the body's `die` movement completes, its specialized finalizer
calls the **same inherited item-drop helper** again (the helper also
handles a pending weapon kept until death completion), then changes
the body's script to `dragwall.cs` and **executes that script's `Init`
event synchronously**. The body finalizer bypasses ordinary monster
death completion: the authored **50,000 experience is not awarded**,
no separate `db_dead.KQS` corpse is created despite its `deadDts` key,
no `Die` script event is sent, and `dragbody` is not deleted. The body
actor stays in its final death pose with zero collision. `dragwall.cs`
has no `Init` branch, so this
call does not speak a line or move the box. Its only action is on
`ConnorAction` with action `do`: it asks Connor to speak
6000/35/14/**53** while `dragwall` exists, or
6000/35/14/**106** after it has been removed. The newly loaded
`dragdbox.kq` uses this same script for a later Connor action. Exact
subframe timing of the visual collision change at death entry remains
open; the
ordering of box load, named removals, script assignment and `Init`
execution is established statically.

A late ordinary physical projectile cannot select the dying body
through its zero collision type and mask. If a `KQAttackEvent` is
delivered directly while `die` plays, the generic dying-state hit
handling still runs, but the shipped 100 protection prevents health
loss from any ordinary positive-damage hit. Health therefore stays
at zero, no new Die-state request is made, and the death movement is
not interrupted. Behavior after a script changes the body's
protection or directly forces another state was not traced.

An additional head handler changes the `chains.bmp` material flag
when it enters a special state, and the body has a corresponding
material adjustment after its death transition. These are appearance
changes in the original executable, not new textures.

## Save continuation and open points

The original head reader has explicit fields for `firstLoad`,
`fullHealth`, `reallyDead`, the attack radii/file, and its death-state
deadline/flags. The first-setup method also reconstructs the shared
permanent-defeat count from heads already marked `reallyDead`. This
supports saving and restoring the two distinct head outcomes.
However, the binary save codec and the order in which actor fields,
state objects and named-world lookups are restored have not been
traced. A recreated save must preserve each head's health,
`reallyDead` state, current movement/state and rebirth deadline, plus
the body's death/removal outcome. Whether the original stores every
one of these directly or derives some on load is open.

Remaining questions for Q210:

- Whether `attackLoc` has an effect outside the traced head attack
  selector and cue handler; the shipped files do not set it.
- Whether any script can alter the body's protection or force its death
  independently of the two permanently defeated heads; ordinary
  shipped combat damage cannot.
- Subframe order of head collision removal, rebirth restoration and
  the body's visual collision change at death entry; the box-load,
  body health/Die entry and later `dragwall.cs` `Init` ordering is
  established above.
- Binary disk-load reattachment of head state/timers, temporary fire
  emitters, and the named body/wall references.
- Whether the authored dragon bounds names resolve to active shapes;
  the generic meaning of `colliderMask=0` is established above.

An unattended original-data encounter check can assert separate
body/head ownership and assets, head `Do` timing, bite versus fire
radius and damage kind, temporary `lopt`/rebirth when Connor is not
in melee mode with the equipped Flame Sword, and the two-head
permanent-defeat transition when he is. The
open points above should remain scoped questions rather than silently
filled by common monster behavior.
