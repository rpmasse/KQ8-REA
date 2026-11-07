# Object projectile cues

Study session A, 2026-11-07. Static study of the original GOG 1.03
executable and installed game data. Original runtime, original save
continuation and manual play are **not tested**. No Rust implementation
was used to infer these rules.

## Purpose

`KQCueEvent` number 500 lets an ordinary `KQObject` fire its configured
projectile. Swamp's tile-puzzle totems use this shared mechanism. Their
fireballs are separate from the gas effects produced by `KQSwampTile`.

## Rules

1. **Delivery and ordering.** Receiving Cue 500 creates and launches one
   projectile from the receiving object's `projectileFile`. It then
   delivers the ordinary `Cue` callback to that object's script, with
   the original cue number and integer data. The script callback does
   not authorize the shot: it happens after the shot has been launched.
   A missing script therefore does not prevent firing.
2. **Projectile identity.** Load a fresh instance of the configured
   `KQProjectile` resource and give it a fresh generated object name.
   The receiving object is its shooter. This path does not use Connor's
   selected weapon, any character's ranged attack controller, or a
   current animated mesh part to replace the configured start offset.
3. **Launch point.** Start at the shooter's current position plus its
   full current rotation applied to `shootOffsetX/Y/Z`. Those offsets
   are local to the shooter; their defaults are zero.
4. **Aim.** Add `relAimPointX/Y/Z` to that launch point to obtain the
   target point. The relative aim vector is in world coordinates: it
   is not rotated with the shooter. The defaults are zero. Neither a
   target-object lookup nor Connor tracking or movement prediction is
   part of this cue. The shared projectile launch computes heading and
   pitch from these two points; it continues straight beyond the aim
   point. Do not reinterpret the aim vector as a fixed range limit.
5. **Damage.** The cue's integer data overrides damage only when it is
   strictly greater than zero and strictly less than 100,000. Otherwise
   use the shooter's `objProjDamage`, whose default is **20**. Thus data
   0, a negative value, and 100,000 all select the configured damage;
   1 and 99,999 are valid overrides. The configured damage itself is
   not subject to that override-range test. The projectile's own
   `weaponType` supplies the attack kind.
6. **Flight and impact.** Use the normal projectile behavior in
   [ranged-weapons.md](ranged-weapons.md), rules 26–30: configured
   velocity and maximum distance, shooter exclusion, ordinary
   projectile collisions, damage and immunity handling, and `Die`
   script delivery when removed. Cue 500 does not implement a separate
   direct hit, area attack, artificial targeting rule or periodic
   damage timer.

## Swamp's authored use

7. **Totem configuration.** `Swamp/kq.vol:totem.kq` is a `KQObject`
   with `projectileFile=PRfire2.kq`, `shootOffsetZ=1075`,
   `relAimPointX=1000` and `script=totem.cs`. The omitted offset and
   relative-aim components are zero; omitted `objProjDamage` is 20.
   Consequently each shot begins 1,075 units above the unrotated
   totem's origin and travels horizontally toward positive world x.
   It is not aimed at Connor, even if Connor is behind the totem.
8. **Fireball configuration.** `Swamp/kq.vol:PRfire2.kq` names
   `classType=KQProjectile`, `shapeName=swmpfrbl.kqs`, velocity
   **9,000 units/second**, `maxDistance=21000`, `collisionDetail=1`,
   `meshName=arrow`, `weaponType=2` (Fire), and `script=swmpfrbl.cs`.
   With the ordinary 15 Hz update, an unobstructed flight advances
   600 units per update. Range removal uses the shared strict
   greater-than limit, not arrival at the 1,000-unit aim point.
9. **Repeated firing.** On any `Cue` callback, `totem.cs` checks both
   `Swamp::Tile::SwitchOn` and `Swamp::Tile::PowerOn`. If both are true,
   it schedules another Cue 500 with data 0 after eight seconds.
   Otherwise it schedules nothing. These conditions govern the next
   cue, not the current launch. A cue already queued still fires once
   after either switch becomes false; the resulting callback stops
   that chain. Multiple queued cues remain multiple firing chains:
   there is no deduplication in this callback.
10. **Impact effect.** The installed shared script is the loose
    `GAME/Kq/swmpfrbl.cs`, not an entry in Swamp's `kq.vol`. On `Die`
    it obtains the projectile's location, loads `endfrbl.kqs` there
    with `loadAnim` at phase 0 and speed 1, then marks the resulting
    animation for deletion when done. It does not add another damage
    event or enable the commented-out sound command.
11. **Starting the sequence.** The puzzle scripts supply staggered
    initial cues; they are not hardcoded by Cue 500. The six loaded
    totems and the extra missing-target reference in `tiletrap.cs`
    are described in [secondary-tile-puzzles.md](secondary-tile-puzzles.md).
    Do not create a seventh totem to satisfy that script reference.

## Persistence

12. **Queued shots.** A pending Cue 500 is a persistent simulation
    event. Its saved payload includes the cue number and integer data;
    its shared event record retains the target object's name and the
    absolute world-clock deadline. On restoration it rejoins the event
    queue, rather than becoming an immediate shot or a new eight-second
    timer. The restored world clock and ordering follow
    [secondary-tile-puzzles.md](secondary-tile-puzzles.md), rules 51–55.
    Time spent outside a kept region does not advance that region's
    pending shot deadlines. Its callback will consult the switches
    when the cue is eventually delivered.
13. **Projectile data saved separately.** The projectile class has
    saved fields for its shooter name, target location, shoot location,
    shoot direction, offset, velocity, maximum distance, weapon kind
    and other ordinary projectile settings. This establishes that
    these values are not intentionally represented only by the
    totem's repeating cue. It does **not**, by itself, establish a
    faithful mid-flight continuation: inclusion of dynamically fired
    projectiles in the saved-object set, reconstruction of their mover,
    exact resumed position/range accounting and shooter reattachment
    remain open in this bounded study. Do not claim that recreating a
    pending Cue also recreates a projectile which has already launched.

## Edge cases and deliberate exceptions

- Missing, empty or invalid `projectileFile`, or a resource that is not
  a projectile, is unsafe in the original cue path. Do not reproduce
  the failure: report the invalid firing object and skip that launch.
  Preserve the ordinary Cue callback where it can be delivered safely.
- A zero relative-aim vector produces a degenerate start/target pair.
  The normal projectile direction calculation for that case has not
  been established here; do not invent an automatic Connor target.
- Persistent cue text uses the generic single-word payload convention.
  Swamp's repeating cues use only the two integers. Malformed optional
  text and missing restored targets are outside this study.

## Automated checks for implementation B

- Rotate a firing object while keeping its relative aim fixed. Its
  start offset must rotate, but its world firing direction must not.
- Check damage data -1, 0, 1, 99,999 and 100,000 against a distinct
  configured `objProjDamage` value.
- Turn a Swamp switch off with a shot still pending: that shot fires,
  its Cue callback runs afterward, and no replacement is scheduled.
- Compare a stationary totem shot with the authored 1,075-unit offset,
  positive-x aim, 9,000 speed, 21,000 range and Fire damage kind.
- Save and restore a pending cue at a nonzero world time: its original
  deadline and damage data survive. Keep this separate from the still
  unresolved already-launched projectile continuation check.

These are proposed automated checks, **not results of original runtime
testing**.
