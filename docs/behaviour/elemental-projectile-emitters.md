# Fire and Ice projectile emitters

Study session A, 2026-11-16. Sources: the installed GOG game's
`Prfarrow.kq`, `PRicycle.kq`, `firebits.kq` and `icebits.kq` in its world
archives; a static, read-only study of the installed `Mask.exe`; and the
earlier [emitter](emitters.md) and [ranged-weapon](ranged-weapons.md) notes.
The executable findings below are traced unless labeled as data or an
inference. Original-game runtime and saved-game continuation were **not
tested**.

## Purpose

The Fire arrow and Ice arrow carry small moving particles. Each bolt has
its own emitter while it flies. The effect can finish naturally after the
bolt is removed.

## Rules

1. **Attachment at launch (traced).** When a projectile with a nonempty
   `emitter=` entry enters the world, the game loads that entry as a
   separate `KQEmitter`, sets its `follow` target to this projectile's
   **own object name**, and adds the emitter to the world. The effect
   does not wait for an impact or the first movement update. No fixed
   name for the new emitter itself was established.
2. **Placement and turn (traced plus existing emitter rule 11).** The
   Fire and Ice emitter files specify neither `followOffset` nor
   `followMesh`, so their offset is zero: the emitter follows the bolt's
   position. Ordinary `follow` turns the emitter's particle velocity
   limits with the target's turn. This is the plain follow path; the
   mesh-part restart behavior described in `emitters.md` rule 11(a) does
   not apply. The exact visible position of the first particle on the
   first rendered frame was not measured.
3. **Particle start and update (data; existing emitter rules 1–8).**
   Once added, the effect uses the normal emitter start, staggered
   appearance, 15-per-second updates, gravity, spin and life rules.
   `delayStart=100` means the particles become visible one by one at
   their first restart; its numeric magnitude does not set a 100-tick
   delay. Individual particles are not projectile collision objects.
4. **Contact, queued hit and removal (traced).** On contact with an
   eligible object or character, the bolt first builds a targeted hit
   from its shooter, damage and Fire/Ice kind. Projectile immunity on
   the target changes that hit's damage to zero. The bolt then places
   the hit in the shared simulation queue **at the current simulation
   time**. This is queue insertion, not immediate delivery to the
   target. Next it moves its own position to the sweep's contact point,
   calls its ordinary `Die` hook, and finally requests deletion. The
   Fire and Ice bolt files have no script for that hook to call. The
   deletion request marks the bolt and enters the world's deferred
   removal list. When the bolt is actually taken out of the world, its
   attached emitter is told to die before the bolt's ordinary removal
   finishes. The emitter is not discarded with the bolt.

   The shared event service drains due work in deadline order and keeps
   equal-deadline insertion order (`saved-world-events.md` rule 5).
   However, this static trace does not place that service, deferred
   object removal and emitter iteration in a proven within-update
   order. It therefore establishes the **request** order, but not
   whether the target reacts before the bolt or emitter disappears, or
   whether already alive particles take a further update before the
   emitter first becomes dying. The ranged-weapon note establishes
   removal after floor contact or range expiry (rules 28–30); it does
   not establish which wins if both conditions occur on one update.
5. **Tail after removal (traced; existing emitter rule 10).** A dying
   emitter stops restarting particles. Each particle already alive
   finishes its own life, then disappears; the emitter removes itself
   when empty. Thus a short Fire or Ice trail can outlive the bolt by
   at most a particle's remaining life, subject to the normal emitter
   update rules. This is a tail, not a new impact burst.
6. **Saved continuation (partly traced).** The projectile's `emitter`
   filename is a normal loaded projectile field, and its world-entry
   path creates the effect from that field. The emitter's own `follow`
   and dying state are part of the general emitter behavior in
   `emitters.md`. This does **not** establish how a native saved game
   restores an already flying bolt together with its existing emitter:
   creation order, identity matching, current particle ages and avoiding
   a duplicate effect remain open. The established world-snapshot rules
   in `world-change.md` describe saved objects as `.kq` entries loaded
   again on restoration, but this study did not establish whether the
   attached emitter is written as its own saved object or rebuilt only
   from its bolt. The installed GOG copy has no native in-flight
   elemental-bolt save to inspect. Two bounded static inspections of
   the projectile entry and general emitter fields did not settle that
   identity or the live particle phases. Do not claim saved-game parity
   from the launch rule alone.

## Numbers and data used

| Source file | Data and effect |
| --- | --- |
| `Prfarrow.kq` | `classType=KQProjectile`, `weaponType=2`, `emitter=firebits.kq` |
| `PRicycle.kq` | `classType=KQProjectile`, `weaponType=3`, `emitter=icebits.kq` |
| `firebits.kq` | `shape=firebit.kqs`, `maxParticles=4`, `maxLife=13` ticks, `gravityZ=3` units per tick squared |
| `icebits.kq` | `shape=icebit.kqs`, `maxParticles=8`, `maxLife=15` ticks, `gravityZ=-3` units per tick squared |
| Both emitter files | `delayStart=100`; x and z velocity each −3…3, y velocity 0…75 world units per tick; y spin 0…3.14 radians per tick |

At the normal 15 updates per second, 13 and 15 ticks correspond to about
0.87 and 1 second respectively. The projectile's `velocity` and
`maxDistance` vary by world and determine the bolt flight, not the
particle life; read the active world's `.kq` entry.

## Edge cases and do not reproduce

- The original entry path assumes a valid, loadable emitter file and a
  valid named projectile. Do not reproduce a crash on missing or invalid
  data; skip the effect and report the bad asset.
- The original emitter behavior note says a saved emitter already marked
  dying is removed as it enters the world. Whether that affects a saved
  elemental tail is unresolved here.

## How to check

An automated test can load a Fire and Ice projectile from each world's
original data, log one attached emitter per live bolt, and compare its
follow target, position and turn with the bolt on each 15 Hz update.
After a hit, floor contact or range expiry, log the bolt's removal and
the emitter's later particle counts until it removes itself. A separate
save/reload test must compare an in-flight bolt and its particle ages to
the native game before claiming saved-continuation fidelity. No manual
gameplay test is required for this study.

## Open questions

1. How does the original saved-game loader match or rebuild the attached
   emitter when a Fire or Ice bolt is already in flight? Are current
   particle ages, velocity choices and the dying tail preserved?
2. After the contact path queues a hit at current simulation time,
   when does the event service deliver it relative to deferred bolt
   removal and the emitter's first dying update? When floor and range
   termination both apply on one update, which takes priority?
3. Does the first rendered particle appear at the bolt's launch point
   or only after its first movement/update under `delayStart`?
