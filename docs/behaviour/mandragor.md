# Swamp Mandragor family — class relationship and data inventory

## Purpose and scope

This note records what the installed GOG 1.03 data says about Swamp's
Mandragor tree and two roots, plus their original-engine class
relationship. Creation and event handling are partly answered; full
combat, death and save behavior remain Q209.

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
performs the tree link. The exact pose conversion, behavior when the
tree is absent, and setup order across the three entries remain open.
The shared setup reads the
object's configured animation file through the common animation-list
loader. As established for characters in `characters.md`, each
animation movement names a model and sequence. No distinct Mandragor
drawing override was found in the compared method tables. This supports
loading their existing `.anm` and `.kqs` assets through the ordinary
character drawing path; the detailed per-movement pose and root/tree
alignment still need study. The generic monster-list loader described
in `characters.md` supplies type statistics, but its handling of these
specific types has not been separately traced here.

The root's class-specific property reader accepts `offset`,
`origHealth`, `origBirthDelay`, `rebirthDelay`, `repelDist` and
`tooClose` in addition to common monster properties. The tree's reader
accepts `rootDownCount` and `maxRoots`; its constructor defaults them
to zero and two respectively. These keys indicate retained encounter
state, but their update, death and save semantics are still open.

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
   Their `rebirthDelay` values are 4 and 5 (unit unconfirmed). The meaning and
   precedence of these class-specific keys require engine study.
4. `mandrg.anm` lists eight movements: `attack`, `die`, `firedie`,
   `icedie`, `laugh`, `react`, `stop`, `walk`. Its `attack` movement has
   no `Do` cue. `mandroot.anm` lists nine: `attack`, `birth`, `die`,
   `firedie`, `hide`, `icedie`, `react`, `stop`, `walk`. Its `attack`
   movement has a `Do` cue at phase 0.1875. This is animation data;
   whether or how the class uses those cues remains open.
5. A read-only scan of all `.cs` entries in Swamp's `kq.vol` found no
   direct reference to the `mand` actor names or Mandragor classes.
   This does not rule out indirect script or engine interactions.

## Unresolved original behavior

- Manager creation and setup order, missing-tree handling, and how team
  names and root offsets affect attachment or targeting.
- Initial active, visible and solid states, sight and attack decisions,
  damage dispatch, root `birth`/`hide` and `rebirthDelay` timing, and
  the tree's `tooClose`/`repelDist` behavior.
- Root versus tree hit and death effects, experience and drops,
  callbacks, the end condition of a group, and save/load continuation.

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
GAME/Kq/Monstats.mtl` expose the cited data. An engine study is
needed before specifying combat or save assertions. No manual
gameplay test was performed for this note.
