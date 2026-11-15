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
protection, alarm 4000, reach 2000 and a 1-second attack delay. Each
head has 600 health, 200 hand damage, 25 protection, alarm 6000 and
reach 6000; the left/right attack delays are 1.5/1.7 seconds. The
body's file sets `isActive3D=1`, `noTurn=1`, `monUseGravity=0`,
`colliderType=1`, and `colliderMask=0`. Each head's file sets
`isActive3D=1`, `noTurn=1`, and `colliderMask=0`. The head constructor
also sets a special collision type containing the projectile-immunity
bit described in `ranged-weapons.md`. These are the authored/initial
values; the precise interaction of mask zero with every ray and
physical collision test has not been separately traced.

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
`attack` movement has no `Do` cue. Its 25 hand-damage statistic must
therefore not be treated as proof that this animation hits Connor.
Whether a different common contact or special route uses that value
in this encounter remains open. The body has 100 protection in the
installed type data; the class trace did not establish a separate
body-specific damage formula.

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
encounter-ending branch runs when that count exceeds one. It asks the
named `dragbody` to enter its death state, handles the authored
`dragdbox.kq` collision object, and removes `draglhed`, `dragrhed`,
and `dragwall` by name, in that order after the body-state request.
The body has its own replacement for entering
the death state and a later death finalizer. The finalizer arranges a
`dragwall.cs` callback. In the installed script, Connor's action at
the wall speaks one of two lines according to whether `dragwall`
still exists. Exact body animation/collision ordering and the
callback's scheduling arguments need a narrower trace before they
can be specified as a frame-exact rule.

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
- Every body damage/contact route and whether it can enter death
  independently of the two permanently defeated heads.
- Frame order of head collision removal, rebirth restoration, body
  death animation and `dragwall.cs` callback.
- Binary disk-load reattachment of head state/timers, temporary fire
  emitters, and the named body/wall references.
- The effect of authored `colliderMask=0` on all physical and weapon
  collision queries.

An unattended original-data encounter check can assert separate
body/head ownership and assets, head `Do` timing, bite versus fire
radius and damage kind, temporary `lopt`/rebirth when Connor is not
in melee mode with the equipped Flame Sword, and the two-head
permanent-defeat transition when he is. The
open points above should remain scoped questions rather than silently
filled by common monster behavior.
