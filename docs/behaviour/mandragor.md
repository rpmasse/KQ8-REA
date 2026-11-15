# Swamp Mandragor family — class relationship and data inventory

## Purpose and scope

This note records what the installed GOG 1.03 data says about Swamp's
Mandragor tree and two roots, plus their original-engine class
relationship. Creation, ordinary root attack and the tree-to-root death
path are partly answered; visual transitions, some damage paths and
save continuation remain Q209.

## Source-backed class relationship

The original executable's runtime type information identifies
`KQMandragorTree` and `KQMandragorRoot` as two distinct sibling classes
derived from `KQMonster`, through the same animation and object base
classes. Both class constructors call the common monster constructor
before installing their respective class methods. Thus both are
specialized monsters with ordinary monster initialization beneath their
class-specific setup. The two class method tables retain many common
monster methods and replace selected ones. Common animation setup and
the absence of a distinct drawing override are described below.
Collision, AI, combat, death and save behavior still require study.
The separate study scratch and
headless-analysis output are kept outside the repository's source list.

## Creation and root linking

The root and tree each run the common monster setup method before their
own setup. A root then changes the last character of its actor name to
`t` and looks up that sibling actor as a Mandragor tree. For example,
the authored `mand3r` and `mand3l` roots both seek `mand3t`. The root
keeps a reference when the tree is found and initializes a pose from
the tree. It also registers with the tree's common event listener, so
later tree messages can reach the root. Thus the three entries in
`monsters.kq` are separately authored actors, and the root's setup
performs the tree link. The root's world position is the tree's world
position plus its authored `offset` rotated by the tree's orientation.
The engine applies the same calculation again when the linked tree
sends a movement notification. It also records the root as a dependent
of the tree's transform. For setup and these notifications, the root
uses world-aligned orientation rather than copying the tree's turn;
only its offset position rotates with the tree. Whether its own AI
turns it between notifications remains open. Behavior when the tree
is absent and setup order across the three entries remain open.
The shared setup reads the object's configured animation file through
the common animation-list loader. As established for characters in
`characters.md`, each
animation movement names a model and sequence. No distinct Mandragor
drawing override was found in the compared method tables. This supports
loading their existing `.anm` and `.kqs` assets through the ordinary
character drawing path; the detailed per-movement pose and root/tree
alignment still need study. The generic monster-list loader described
in `characters.md` supplies type statistics, but its handling of these
specific types has not been separately traced here.

## Startup and collision

The ordinary monster constructor starts with collision type `0x42`
and mask `0x281F`. Both Mandragor classes replace the type with
`0x4042` while retaining that initial mask. The extra `0x4000` is the
projectile-immunity flag established for other characters in
`ranged-weapons.md` and `henchman.md`. After common setup, both
Mandragor classes turn their mover's gravity off. This is their setup
state, not a permanent gravity override. The authored
`keeponground=1` remains a separate setting.

The roots' animation list contains `birth`, so common character
startup enters the inherited Unborn state. As described in
`characters.md`, Unborn is not drawn or solid until it emerges. The
tree's list has no `birth`; common startup instead queues Wait with
the authored zero birth delay. The roots keep the ordinary Unborn and
Wait state classes; only their death state is replaced. Thus the
common rise gate in `character-movement.md` rule 30 applies: an
inactive root must see Connor within its alarm radius and obtain an
active-monster slot. After the authored zero birth delay it enters
Wait, which delays `birth` while Connor stands within the combined
actor balls. The monster type/list gives each root a 3,000-unit alarm
radius, overriding its `.kq` file's 15,000. When this Wait entry can
proceed, the shared method requests `birth`, restores ordinary groups,
and turns gravity back on because the root retains the default
`monUseGravity=1`. The exact first-frame ordering remains to be
established.

The root's class-specific property reader accepts the authored
`offset`, `rebirthDelay`, `repelDist` and `tooClose`, plus stored baseline
health and birth delay, in addition to common monster properties. The
tree stores how many roots are down and the permitted maximum; its
constructor defaults them to zero and two respectively. These values
support the death and rebirth cycle below.

## First combat and event boundaries

Both classes replace a common monster hit/event handler. For a
nonzero value in that event they delegate to the common monster
handler. For zero, the tree selects its `laugh` movement; a root asks
its linked tree to select `laugh`, if the link exists. This is
consistent with a zero-damage or immune hit, but the payload's exact
meaning and any health effect have not yet been established. Another
root event method distinguishes messages from its linked tree and
updates the root pose before running a proximity response. The trigger
and timing of that message remain open. The response checks Connor's
horizontal distance against the root's `tooClose` value. If Connor is
inside it and in a susceptible movement state, the root sends him a
movement event using `repelDist` and selects his `reactback` movement.
The exact displacement vector and collision aftermath remain open.

The roots retain the common monster attack and animation-cue handler.
Their `attack` movement has a `Do` cue at phase 0.1875, so the shared
melee hit test in `fighting.md` runs there with the root's authored
hand damage and reach. Both roots use the ordinary Wait and Move states;
their class replaces only the Die state. Consequently their attacks
start under the shared sight, distance, due-time and movement
interruption gates in `fighting.md` rule 7. At `Do`, the shared handler
tests the current target's straight-line reach and front half as in
rule 13, then sends the root's 15 hand damage on a hit. The root class
has no separate target-selection or attack-cue override. This
establishes the ordinary Connor-target route; interactions with team
targets and the tree's influence on root facing need further study.
The tree's `attack` movement uses its standing
model and has no `Do` cue. Its ordinary melee hit test therefore has
no cue at which to run. No separate tree attack-hit callback was found
in its class methods; any contact damage or other source remains open.
The tree's Wait-entry method also limits one target-position update to
when Connor's distance is within its alarm radius; the roots retain the
ordinary Wait-entry method. This does not by itself establish the full
targeting or attack schedule.

The tree's class property reader does not consume its file's
`tooClose=1600` or `repelDist=1800`; neither does the common monster
reader. Only the root class reader names these keys. No tree proximity
push should be inferred from those file lines.

## Root death and rebirth

A root uses a dedicated death state instead of the ordinary monster
death state. On entry it sets its health to zero, adds one to the linked
tree's down-root count up to the maximum (default two), and stops the
tree from turning. It requests the root's `die` movement and sets the
root's collision type and mask to zero, as ordinary Die also
does. This makes the root non-solid while the death movement plays.
The exact draw-group transitions remain to be established.

When `die` finishes, the root starts a timer for its `rebirthDelay`:
four seconds for the right root and five for the left in the installed
data. When due, it requests `birth`, restores its stored baseline health,
and runs its proximity response. When `birth` finishes, it restores
collision type `0x4042` and mask `0x201F`, reduces the linked tree's
down count by one, lets the tree turn again if the count reaches zero,
and enters Wait. The mask after rebirth differs from the constructor's
initial `0x281F`;
the reason for this difference and its terrain consequences need
separate study. Until `birth` finishes, this dedicated death state
rejects a request to leave it for Wait or Die. It then permits those
transitions. Event queue persistence and save/load continuation remain
open.

## Tree death boundary

The tree uses the common monster Die state, which selects `die` for a
normal hit, `firedie` for fire and `icedie` for ice. Its animation list
has no `projdie`, so that movement falls back to `die` if requested.
After the death movement, the tree uses a class-specific finalizer. It
retains the common experience award and creates a corpse from its
authored `deadDts=mt_dead.kqs` whenever that field is present, even for
Fire or Ice death. The ordinary monster finalizer would choose its
corpse by damage kind. The tree then marks itself for removal. Its
finalizer does not call the ordinary monster's script `Die` callback
helper. Whether another point in the tree's death path sends a script
callback remains open.

Each root listens for its linked tree's state change. When the tree
enters Die, the root clears its tree link and requests Die too. A root
with no tree link creates the ordinary permanent monster Die state,
rather than its timed root-death state. If a root is already in its
timed death/rebirth cycle, that state does not leave early for the
tree's request. Its `birth` completion checks the link: with a tree it
returns to Wait; without one it enters Die. Thus tree death prevents
further root rebirth, but a root already down can finish its current
cycle before entering ordinary Die. The order of removal notifications
and collision changes in that overlap remains open.

## Save boundary

The class property paths for both types can read and write their
class-specific values through the game's binary property stream as
well as its text form. This covers the root's authored offset, rebirth
delay and proximity distances, plus stored baseline health and birth
delay. The tree stores its down-root count and permitted maximum. The
root's live reference to the tree is not among those class-specific
properties. The dedicated root death state also stores its rebirth
deadline and two phase flags, so its timer is an absolute simulation
deadline rather than a new delay from load time. The shared saved-world
sequence restores an object's saved fields before its normal Init, and
restores objects in recorded order (`saved-world-events.md` rule 10).
Root setup resolves the tree by name and registers its listener, which
provides a possible reconstruction path if the tree is already present.
Whether root setup runs on every disk-load path, whether the tree is
present before each root, and how the dedicated death state and collision
are reactivated after loading have not yet been traced.
Save/load continuation remains open.

## Confirmed data

1. `Swamp/kq/monsters.kq` places 11 groups named `mand3`, `mand4`,
   `mand5`, `mand7` and `mand10`–`mand16`. Each group has a tree (`t`),
   right root (`r`) and left root (`l`) with the same team name and
   location: 33 distinct monster-list entries. The list's positions
   have z = −2 and heading 0. These fields establish authored grouping,
   not how the original engine links or moves the actors.
2. `GAME/Kq/Monstats.mtl` names three distinct types and class files.
   The tree (`mandragorTree`, `mandrg.kq`) has health 300, hand damage
   25, protection 15, experience 20,000, alarm radius 4,000, attack
   distance 2,000 and attack delay 1 s. The right and left roots
   (`mandragorRootR`/`L`, `mandrtr.kq`/`mandrtl.kq`) each have health
   250, hand damage 15, protection 30, experience 0, alarm radius
   3,000 and attack distance 2,000. Their attack delays are 1.2 s and
   1 s respectively. The type and placement entries name no held or
   dropped items.
3. `mandrg.kq` declares `KQMandragorTree`; `mandrtr.kq` and
   `mandrtl.kq` declare `KQMandragorRoot`. All three give
   `alarmRadius=15000`, `ouchSound=manouch.aud`,
   `keeponground=1` and their own movement list. The tree gives
   `deadDts=mt_dead.kqs`, `tooClose=1600`, `repelDist=1800`.
   The roots give `tooClose=700`, `repelDist=800`, plus offsets
   (+600, −1000, 0) on the right and (−600, −1000, 0) on the left.
   Their `rebirthDelay` values are 4 and 5 seconds, as confirmed by the
   death state's world-clock timer. Other key interactions remain open.
4. `mandrg.anm` lists eight movements: `attack`, `die`, `firedie`,
   `icedie`, `laugh`, `react`, `stop`, `walk`. Its `attack` movement has
   no `Do` cue. `mandroot.anm` lists nine: `attack`, `birth`, `die`,
   `firedie`, `hide`, `icedie`, `react`, `stop`, `walk`. Its `attack`
   movement has a `Do` cue at phase 0.1875. The shared melee cue
   handler's effect is described above; other attack sources remain open.
5. A read-only scan of all `.cs` entries in Swamp's `kq.vol` found no
   direct reference to the `mand` actor names or Mandragor classes.
   This does not rule out indirect script or engine interactions.

## Unresolved original behavior

- Manager creation and setup order, missing-tree handling, and how
  team names affect targeting.
- Exact first-frame and root death visibility, tree damage path,
  team targeting and the collision/notification order during root rebirth.
- Tree hit reactions, experience and drops, script callbacks,
  the end condition of a group, and save/load continuation.

## Do not reproduce

No engine bug or unused field is established here. Do not assign the
Mandragor files the ordinary `KQMonster` rules merely because their
entries appear in `monsters.kq`.

## How to check

With `KQ8_GAME_DIR` pointing to the GOG installation, `kq8 text
vol:Swamp/kq/monsters.kq`, `kq8 text vol:Swamp/kq/mandrg.kq`,
`kq8 text vol:Swamp/kq/mandrtr.kq`, `kq8 text
vol:Swamp/kq/mandrtl.kq`, `kq8 text vol:Swamp/kq/mandrg.anm`,
`kq8 text vol:Swamp/kq/mandroot.anm` and `kq8 text
GAME/Kq/Monstats.mtl` expose the cited data. The combat and death rules
above were checked against the read-only original executable, with
scratch analysis kept outside the tracked source. Full disk-load
continuation still needs original-engine tracing. No manual gameplay
test was performed for this note.
