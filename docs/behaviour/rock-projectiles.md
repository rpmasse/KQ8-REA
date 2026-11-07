# Held Rocks: aim, flight and recovery

Study session A, 2026-11-07. Bounded static study for Q202. Sources are the
read-only GOG 1.03 `Mask.exe`, `prrock.kq`, `GAME/Kq/Conner.anm`,
`GAME/Kq/Invinfo.itl`, `GAME/Kq/rockHit.cs`, and Dead City's `rockme.cs`.
The executable evidence follows the held-item click, target acceptance,
ThrowRock state, character projectile launch, projectile mover and contact
handler. Raw trace material stays in ignored study scratch. Original runtime
and visual behavior are **not tested**.

## Purpose

A held Rock is thrown as a temporary projectile. Its death script later takes
one Rock from Connor's inventory and puts a recoverable inventory item at the
projectile's final placement. This note describes the intervening aim and
flight; [traps.md](traps.md#recoverable-rock-registration-and-removal-q203-2026-11-07)
and `items.md` rule 11 establish item identity and pressure-plate registration.

## Rules

### Choosing the aim and starting the throw

1. A held item in the **Rock category** takes the Rock click path before the
   ordinary weapon-mode click. The ordinary input exclusions still apply
   (`ranged-weapons.md` rule 13). This is separate from the bow's aiming path.

2. The Rock path makes the camera-to-pointer line described in `clicking.md`.
   It first shortens the line at the **drawn terrain** intersection, if any,
   then finds the nearest eligible collision shape on that shortened line.
   The shape filter is **0x29**: a collision type containing any of **1, 8 or
   0x20** qualifies. Thus plain type-0x41 props, clickable objects and
   buildings can intercept it. This query does not use the bow's 0x48 mask.
   A shape whose collision type is zero is excluded even if the object has
   `acceptsInventory=INVITEM_Rock`.

3. Normally the aim is that collision intersection. If no eligible object
   intercepts the line, it is the shortened line's end: the drawn terrain
   point, or the original far endpoint for sky. There is **no bow steepness
   clamp** and no automatic conversion of a character hit to the middle of
   its feet on this path.

4. A picked game object gets a special opportunity to handle the Rock:
   it must accept the held item's type using the common acceptance rule, and
   its ordinary Connor-feet-to-object-box reach distance must be **strictly
   less than the Rock projectile file's `maxDistance`**, 2,500 for `prrock.kq`.
   This is the usual reach helper described in `doors.md`/`clicking.md`, not
   camera distance or distance to the ray's contact point. When both tests
   pass, the aim becomes the object's **placement position**, including its
   z, and a queued `ConnorAction` supplies the held object's identity and
   type. The engine does not additionally start an automatic throw in this
   branch: the receiving object or script decides what to do. `rockme.cs`
   requests `ThrowRock` for `INVITEM_Rock`.

5. A hit object that declines Rock, or fails that strict reach test, still
   supplies the ordinary ray-hit aim from rule 3. Connor starts the normal
   Rock throw without the ordinary unusable-item or too-far remark from an
   item-offering click. Merely placing a Rock-accepting object nearby does
   not attract a ground or sky throw to it.

6. For the direct throw, third-person Connor receives an ordinary turn-to-
   aim request, then enters `ThrowRock` immediately. A click made while in
   first person skips that turn request; entering `ThrowRock` then forces
   third person. The Rock path has **no 45-degree shot gate and no pending
   shot released when a turn is nearly finished**. Turning and the preparation
   animation can proceed together. The ordinary turn-request restrictions
   still apply. For a scripted accepting object, its ordinary ConnorAction
   handling requests the turn after the script returns, unless the object's
   normal turn-suppression setting prevents it; ThrowRock is among the states
   allowed by that check.

7. The state remembers Connor's weapon mode and projectile file, plays
   `away`, temporarily selects the Rock item's `prrock.kq`, then plays
   `throw`. `Conner.anm` uses `c_rktoss.kqs`; its `do` cue is at phase
   **0.777778**, and `tossrock.aud` is requested at **0.111111**. The projectile
   is created on `do`, not on the initial click or state entry. At animation
   completion the old projectile file and weapon mode return, with `out`
   when needed, then Stop/Wait as described in `ranged-weapons.md` section 8.
   A Tile2 throw keeps the tile state and has its separate choreography
   (`secondary-tile-puzzles.md` rule 24); do not replace that with this state.

### Launch placement and orientation

8. The projectile uses `meshName=rock`. At the queued launch cue it reads the
   **currently selected animation frame's** named model part, using the same
   frame timing as `ranged-weapons.md` rule 24. The part's local translation
   supplies the launch offset. Connor's current complete orientation rotates
   that offset, then his current position is added. The clicked aim is kept
   in world coordinates; it is not recomputed from the mouse or led for target
   motion at launch.

9. A parabolic projectile also retains the named part's **authored rotation
   from that frame**. Its position is changed to the world launch point, but
   that rotation is not replaced with the bow's aim pitch or multiplied by
   Connor's heading on this path. Flight changes the projectile's position
   without spinning it or steering its displayed rotation along the curve.
   The movement direction is calculated separately from launch to aim.
   Consequently the recovered item's angles can be pitched or rolled: do
   not flatten it or substitute Connor's heading at landing. The missing-part
   case is a defect boundary below, not an established full-transform
   fallback.

10. The ordinary `ThrowRock` launch sets damage kind **4** and carries
    Connor's current ranged damage value, through the same character launch
    path as a bow shot. It is not a hard-coded zero-damage projectile. Kind 4
    yields the ordinary `Normal` death description on a killed character,
    as already established in `ranged-weapons.md`. This kind override is
    conditional on the actual ThrowRock state; this trace does not extend it
    to every script or Tile2 launch that happens to use `prrock.kq`.

### The authored arc

11. On launch, the effective horizontal range is the smaller of the file's
    `maxDistance` and the **horizontal distance from the actual launch point
    to the aim point**. It is not measured from Connor's feet, and the aim's
    height does not change this range. The arc follows that horizontal
    heading. Target z does not set its endpoint height or act as a landing
    plane. A far sky aim therefore still gets the Rock's limited arc.

12. The engine's `parabola` name does not describe a gravity-driven ballistic
    integration. A horizontal progress distance advances at each ordinary
    **1/15-second** update. It uses `upVelocity` until the update on which
    progress first passes one third of the effective range; thereafter it
    uses `velocity`. Both are **2,500 units/s** in `prrock.kq`, so ordinary
    progress steps are approximately **166.6667 units**. A progress step is
    charged before evaluating the curve or testing the range. The boundary
    update still uses `upVelocity`; the later rate applies on subsequent
    updates. Values are stored as single-precision floats.

13. The height curve consists of two ellipse-like segments. The following
    mathematical description specifies the observable curve, using new
    notation rather than original program identifiers:

    - Let **R** be the effective horizontal range, **s** the accumulated
      progress, **L** the world launch height, **h** the named part's local
      launch-offset z, and **p** the file's `peakHeightOffset` (**500**).
    - From the start through **s = R/3**, world height is
      **L − p + |p| sqrt(1 − ((R/3 − s)/(R/3))²)**.
    - Beyond **R/3** through **R**, world height is
      **L − (h + p) + |h + p| sqrt(1 − ((s − R/3)/(2R/3))²)**.

    This specifies a world height at each progress value; the mover does not
    repeatedly add the entire curve height to the preceding height. The
    horizontal position is the launch position plus progress along the fixed
    horizontal heading. For normal positive authored heights, the peak is
    **L**, not L+500. The curve's starting limit is **L−500**, even though the
    initial projectile placement itself is L. Its endpoint limit is
    **L−h−500**. For an upright Connor that endpoint is normally 500 units
    below his feet, so a flat floor usually terminates the trajectory before
    that endpoint. These unusual height references are present in the static
    trace; replacing them with a symmetric physical parabola changes behavior.

14. Each desired step goes through the ordinary collision mover. The
    projectile ignores its shooter, uses its usual collision pair rules, and
    checks the **floor grid** height after movement, distinct from the drawn
    terrain used for clicking. Once its position is at or below that floor,
    a parabolic projectile is set to that floor height at its current x/y,
    receives `Die` and is removed. This is not a water-surface snap. The final
    position supplied to the death script is the resolved/clamped placement,
    not the originally clicked point.

### Obstruction and range completion

15. For a parabolic projectile, exceeding R does **not** immediately delete
    it. The test is strictly **progress > R**. It instead enters a falling
    phase and stops horizontal travel. The newly exceeded progress is not
    used to move it farther along the curve. The original falling branch has
    the undefined vertical-step issue described below, so a fully deterministic
    original descent rate cannot be asserted from this trace.

16. Collision handling has a separate Rock branch before ordinary damage:
    the shooter is ignored; a collision whose reported contact position is
    the all-zero vector is also ignored by that branch. For a nonzero contact,
    each supplied surface normal is normalized. If **any normal's z is less
    than 0.707**, the projectile is moved to the contact point and put into
    the falling phase. This includes side walls and undersides. It does not
    bounce, retain forward movement or deliver the ordinary projectile damage
    in that branch. A contact with no such steep normal takes the ordinary
    projectile hit/death route; a flat upward normal qualifies for that route.
    The exact zero-length-normal behavior is not established.

17. On the ordinary hit/death route, a placed game object receives the normal
    projectile hit event, with zero damage for the projectile-immunity bit
    and the ordinary `ProjImmune` notification. Buildings receive no character
    damage. The projectile is placed at the collision position before `Die`;
    for the no-specific-solid collision form, x/y come from the contact while
    z remains the mover's proposed/resolved z. The established floor test
    provides its own floor snap. There is no general snap of every object hit
    to the clicked target's centre or to the ground beneath it.

18. Projectiles and pressure traps both have collision detail 1, so the pair
    is excluded before geometry. The flying Rock is not a plate occupant.
    A recoverable Rock has the ordinary inventory collision behavior and can
    become an occupant immediately on insertion; see the Q203 rules in
    `traps.md`. In particular, a type-zero decorative plate does not need to
    be a click target or projectile collision target for the recovered item
    to hold its associated pressure trap.

### Recovery and inventory timing

19. `rockHit.cs` handles `Die` by obtaining the projectile's final **position
    and angles**, then calling `dropItem Connor INVITEM_Rock` with that
    placement. There is **no carried-quantity deduction at click, away,
    throw start or projectile launch** in this chain. The pointer is cleared
    after the accepted Rock click, but clearing the pointer is not consuming
    the inventory unit. Normal death handling then removes the projectile.
    Removal of a kind-4 projectile also plays its `ouchSound` at its final
    location; the authored Rock uses `rock.aud`.

20. Consumption therefore depends on the inventory present when that script
    executes. It removes one carried Rock and creates the fresh quantity-1
    world item according to `items.md` rule 11, including the separately
    documented last-carried-identity edge. It does not reserve a particular
    unit at launch, convert the projectile into that inventory object, or
    transfer the projectile's collision membership. Recovery is not guaranteed
    if Connor no longer carries a Rock when the script calls `dropItem`.
    Exceptional interruption/save/world-change behavior is outside this trace.

## Numbers and data used

| Source/field | Authored value or role |
| --- | --- |
| `Invinfo.itl`, `INVITEM_Rock` | Rock category, `rok.kqs`, `prrock.kq`, Rock pointer and sounds |
| `prrock.kq` | `KQProjectile`, `shapeName=rok.kqs`, `meshName=rock`, `script=rockHit.cs` |
| `velocity`, `upVelocity` | 2,500 units/s each; horizontal progress rates |
| `maxDistance` | 2,500 units before the nearer-aim reduction |
| `peakHeightOffset` | 500 units; curve parameter in rule 13 |
| `collisionDetail` | 1; traps/projectiles with detail 1 do not pair |
| Default projectile type/mask | 1 / 0x1f |
| `ouchSound` | `rock.aud`; the file's dry/wet landing sound lines are comments |
| Surface-normal threshold | z < 0.707 enters the falling branch |
| Falling downward term | −8,000 units/s, subject to the defect below |

## Edge cases and do not reproduce

- **Undefined falling displacement:** the original falling branch clears the
  two horizontal displacement components and adds −8,000 × update-seconds
  to a vertical temporary that has no initialization on that branch. It is
  not a persistent, explicitly initialized vertical velocity, and no gravity
  integration or acceleration state was found there. Static evidence settles
  the horizontal stop and downward term, but not the total repeatable vertical
  displacement. Do not imitate uninitialized memory. A safe recreation policy
  is to initialize the displacement and descend vertically at **8,000 units/s**
  with the same collision/floor termination. That is a deliberate compatibility
  safeguard, **not a verified original numeric trajectory**.
- The ordinary authored launch reads a named model part. If that part or
  its frame is missing, the character path preserves a translation fallback,
  but a valid complete rotation transform is not established. Do not source
  rotation from uninitialized memory; a defined fallback must be labelled a
  safeguard. The specific `c_rktoss` cue frame's numeric part transform has
  not been extracted in this study.
- Zero effective range, degenerate normal vectors and malformed negative
  height parameters are not validated by the original curve. Do not propagate
  nonfinite coordinates into world collision or item placement.
- `parabola=1` is not permission to use a generic gravity solver. Nor does
  `peakHeightOffset=500` mean the peak is 500 above the world launch point.

## How to check

These are numeric/data checks for implementation; original gameplay remains
**not tested**. No manual test was performed or requested by this study.

1. Inspect the named installed data files and compare the values and cues in
   the table. Log separately the ray endpoint, chosen collision point, any
   accepted object's placement, strict reach decision and final launch point.
   A type-zero plate must not bypass the ray filter. At exact reach 2,500 the
   accepting-object special branch must not run.
2. With an isolated numeric launch at L=1,000, local offset h=1,000,
   R=2,500 and p=500, compare curve reference samples: at s=R/3 the height is
   1,000; at s=1,500 it is about 874.773; at s=R the mathematical endpoint
   is −500. These are curve landmarks, not a claim that a 15 Hz float update
   lands exactly on each landmark. Terrain/collision can terminate earlier.
3. Log the unchanged inventory count at launch and the one-unit deduction at
   `rockHit.cs`'s `Die` handling. Compare final projectile position/angles to
   the fresh inventory item's placement. Keep the Q203 identity and immediate
   trap-registration checks separate from these flight checks.
4. Test a shallow upward normal and a side normal independently. The side
   contact must stop horizontal travel without the ordinary damage/death
   branch. Mark any deterministic falling trajectory as the safeguard above,
   not original-runtime evidence.

## Open questions

- The original undefined falling displacement cannot establish a stable
  numeric descent without original-runtime evidence; no such test was run.
- Numeric `rock` part translation/rotation at the delivered `c_rktoss` cue
  frame, and behavior when that part is absent, remain unmeasured here.
- Midflight persistence, world departure, disappearance of Connor or depletion
  of Rocks before `Die`, and nonstandard scripts using this projectile need
  separate studies if their exact exceptional recovery outcome is required.
- Full terrain/solid swept-collision precision and unusual simultaneous-contact
  normal sets are inherited from the shared collision system; this note does
  not re-establish all of that geometry.
