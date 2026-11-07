# Terrain hazards: damage amounts and liquid eligibility

Study session A, 2026-11-07, for Q206. The rules marked traced below
come from static study of the original GOG executable. Regional
configuration and pickup placements are original-data findings.
No original runtime or original save/load comparison was performed:
those are **not tested**. This note supplies the ordinary deep-liquid
path; the remaining `ontop` and special-status questions are explicitly
listed at the end. It does not close all of Q206.

## Purpose

Poison, fast poison/lava and swamp materials can hurt an eligible
object after its movement. Water depth determines whether ordinary
liquid exposure qualifies. The world's `Time` settings determine the
base damage, sometimes using the shared world clock to distribute small
amounts across updates. They are not exposure countdowns started when
Connor enters a river.

## Rules

### Selecting an amount

1. **Material priority, traced.** Among the material's flags, poison
   (`0x40`) wins over fast poison (`0x80`), which wins over swamp
   (`0x100`). Their settings are respectively `KQWorld::PoisonTime`,
   `KQWorld::FastPoisonTime` and `KQWorld::SwampTime`. Plain water, fire
   or ice without one of those three source flags supplies no amount
   through this query. Fire and ice select a hit kind separately.
2. **Zero means 1,000, traced.** A selected setting of zero returns
   base damage **1,000** on every eligible query. It does **not** turn
   damage off and it provides no entry grace period. This applies to
   either sign of floating-point zero. The original variable lookup's
   fallback is also zero.
3. **Positive settings scale with Connor's level, traced.** For a
   positive setting `T`, the unrounded amount is **0.66 × Connor's
   current level ÷ T**. The result is stored at single precision.
   If the amount is at least one, its fractional part is discarded and
   that whole-number base damage is returned on every query. This
   uses Connor's level even if another object is being checked; it does
   not use the victim's health or level. With no live Connor, the query
   returns zero.
4. **Small amounts use world-clock phase, traced.** If the amount is
   below one, take its reciprocal, discard the fractional part and cap
   the resulting stride at eight. The current slot is the fractional
   part of world time in seconds, multiplied by 15 and rounded to the
   nearest integer by adding one half and discarding the fraction.
   The query returns one when that slot is divisible by the stride;
   otherwise it returns zero. Do not replace this with a per-object
   timer or an accumulated fractional-damage balance. The fractional
   time begins again at each whole second; it is not an absolute tick
   number modulo the stride. Do not clamp the rounded slot to 14.
5. **Examples, derived from the traced arithmetic.** At level 12 and
   `PoisonTime=5`, the unrounded amount is about 1.584, so base damage
   is one per eligible query. At level 1 with the same setting, it is
   about 0.132, giving stride seven: slots 0, 7 and 14 qualify. The
   selected `FastPoisonTime=0` case always returns 1,000 regardless of
   level or clock phase. Negative or nonfinite settings are not used
   by the inspected original data and are not specified here.

### Ordinary deep liquid

6. **Depth, traced.** At the object's new horizontal position, the
   surface query must report a liquid flag (water, poison, fast poison
   or swamp), and the surface must be at least **250 units above its
   feet**. Equality qualifies. This is distinct from the **600-unit**
   threshold that selects wading animations. An object can take liquid
   damage before it appears as submerged to the movement controller.
7. **Eligibility, traced.** Ordinary deep-liquid damage requires a
   nonzero amount and at least one current material flag not covered
   by that object's terrain immunity mask. The original compares the
   full material flags against the inverse mask, rather than testing
   only the particular source flag selected in rule 1.
8. **Initial immunity, traced.** Generic objects and ordinary
   `KQMonster` construction start with all terrain flags immune.
   Connor's construction enables the poison, fast-poison, swamp, fire
   and ice flags for damage; the other flags remain immune. The public
   command `KQConner::addImmune` adds the supplied bits to his immunity
   mask. It does not replace the mask. No use of this command was found
   in the inspected regional or loose scripts. Native persistence of
   changes made by this command is **not tested**.
   The command accepts exactly one decimal integer argument, retains
   its low 16 bits and returns zero. Wrong argument counts print the
   command's usage message and also return zero.
9. **Hit type and source, traced.** An eligible exposure queues the
   ordinary attack event against the object, with the world as source,
   the amount from rules 1–4, and no forced-reaction flag. The kind is
   **Fire** if the material has fire (`0x200`); otherwise **Ice** if it
   has ice (`0x400`); otherwise **Normal**. Fire wins if both are set.
   There is no separate poison hit kind on this path. A bare `0x80`
   material therefore emits a Normal hit, despite being called lava or
   fast poison in the material flags.
10. **Ordinary hit handling still applies, traced.** This is an attack
    event, not a direct health subtraction or a stats-change request.
    Connor's existing damage roll, protection, owning-state handling,
    death threshold and dying/rope guards therefore apply. The world
    source temporarily suppresses his usual combat ouch sound; it has
    the separate poison-sound path. See `fighting.md`, `potions.md` and
    `connor-combat-death.md` for those existing rules. No separate
    invulnerability-potion exemption appears in the terrain emitter.
    The separate sound cycles through `poison1.aud`, `poison2.aud` and
    `poison3.aud`, beginning with the first. A world-source hit plays
    the current sound if its own sound gate is ready, then closes that
    gate for **2.5 world seconds**. When the delayed event rearms the
    gate, it advances the sound number, wrapping after three. Repeated
    hits during the wait do not restart the wait. This sound decision
    follows ordinary damage handling and does not test the resulting
    damage or remaining health; the hit that kills Connor can still
    reach it. The already-dying and rope guards run before both damage
    and this sound branch. No additional god-mode or zero-loss check
    appears in the sound branch. Persistence of this sound gate and
    delayed event in an original save is **not tested**.
11. **The shield is not a zero-damage promise.** The Invulnerable potion
    raises protection as described in `potions.md`; it does not add
    terrain immunity. Protection 100 still makes ordinary damage zero,
    but a weaker shield still uses the normal damage calculation. For
    the campaign's protection 11, the potion gives protection 50. A
    1,000-point base hit remains potentially lethal even at full
    level-12 health, and certainly is not a safe exposure guarantee at
    health 39. The exact GodArmor command/item path has not been
    independently re-traced for this note; do not infer a new terrain
    bypass from the item name.

### Clock, movement and saving

12. **No per-exposure timer, traced.** The ordinary amount query only
    reads the selected world setting, Connor's level and shared world
    time. It neither starts nor resets a hazard timer on entry or exit.
    Repeated queries during the same world-clock phase return the same
    amount. The post-movement path checks the object's resulting
    location; the study did not establish a new swept-volume damage
    test between the beginning and end of a move.
13. **Save scope.** There is no separate countdown or remaining
    exposure fraction to save for rules 1–4. Matching their phase after
    loading depends on matching the original world-clock restoration.
    Original save/load behavior of that clock, terrain immunity and
    the special statuses below was **not tested** in this study. Do not
    label a Rust save test an original-runtime comparison.

## Original data

- All twelve regional `WORLD.CS` files set `FastPoisonTime` to **0**
  and `PoisonTime` to **5** during `Init`. Their own comments call
  these terrain damage amounts. Daventry and Swamp additionally set
  `SwampTime` to **5**. Other inspected regions do not explicitly set
  that third variable. Their retained/global-variable behavior is not
  established by the data alone.
- The installed Barren and IceWorld `patch/WORLD.CS` files retain the
  same zero/five values. The script source, not the word "Time", is
  the evidence for the configured numbers; rules 2–4 explain their
  meaning in the executable.
- Dead City's `WORLDINV.CS` places one Invulnerable potion at
  **(33111,203729,2000)**, on the skull chamber's north side.
  `GAME/Kq/Invinfo.itl` gives it duration 20 and no direct stat delta;
  its category-specific behavior is described in `potions.md`.
- The nearby potion does not establish a safe route across the river.
  No nearby additional fire/ice/terrain-immunity pickup was found in
  the queried placement data. The explicitly placed Crystals are much
  farther away and provide health, not protection.
- The separate Dead City river-wall traps run `rivrtrap.cs` and send
  a 5,000-point attack. They are independent of the terrain-material
  damage path described here. Implementing those traps does not
  implement fast-poison materials.

## Special paths and remaining uncertainty

- An existing object status can force poison-rate fire flags and allow
  the ordinary damage path without the liquid-depth condition. Its
  full lifecycle and public meaning have not been established here.
  Do not manufacture a script flag or infer it from the Rust model.
- **`ontop` (`0x800`), partly traced.** If its amount is nonzero, its
  material is not fully immune, and the surface is no more than 100
  units below the feet, a separate branch is selected. It only performs
  the sinking action for Connor in a particular animation/control
  condition; the public meaning of that condition remains unresolved.
  In the action, keep-on-ground and gravity are disabled. While the
  surface is less than 250 above the feet, an additional movement lowers
  him by **15 units**, then the branch queues the hazard attack. Its
  kind has the same fire-before-ice priority as rule 9. The original
  branch does not simply fall through to ordinary deep-liquid behavior
  when its Connor/control gate rejects it.
- What restores gravity/keep-on-ground after `ontop`, the additional
  status's lifecycle, exact native immunity serialization and the
  world-clock save restoration remain **open**. Treat the ordinary
  non-`ontop`, non-special-status path separately from those cases.

## How to check without manual playtesting

Use automated numeric checks for source-flag priority, zero settings,
positive-setting arithmetic at several levels, phase immediately before
and after a whole second, and the exact 249/250 and 599/600 depth
boundaries. Verify fire-over-ice type selection and immune-mask filtering.
For the ordinary world attack, check both normal protection and
protection 100 through the existing hit handler rather than subtracting
health directly. A native Rust run should log the world attack and its
resulting health/death state; a damage-free run without that dispatch
does not prove fidelity. Any later original-runtime comparison should
also be automated and logged, not a requested manual test.

No Rust code or original game data was copied into this note. Raw study
scratch material remains outside the tracked documentation.
