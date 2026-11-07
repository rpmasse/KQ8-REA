# Swamp and Barren tile puzzles

Study session A, 2026-11-07. Static study of the original GOG 1.03
executable, checked against the installed original data. This covers
Connor's `Tile2` and `Tile3` states and the `KQSwampTile` and
`KQLavaTile` object classes. Original runtime, original save continuation
and manual play are **not tested**. No Rust implementation was used to
infer these rules.

## Purpose

These are separate puzzles using the same authored tile-jump motions.
Swamp combines a fixed floor, powered hazards and recoverable Rock
throws. Barren combines cycling platforms with a prime-number sequence
that creates a permanent bridge. Neither uses Death Hall's wrong-stone
teleport/reset rule.

## Rules shared by the two Connor controllers

1. **Entry and state changes.** Entry copies the named tiles' positions
   and initially disallows leaving the controller. Both controllers
   nevertheless permit `Die`; this differs from Death Hall's `Tile`.
   Their numeric StateChange identities are **Tile2 = 16,384** and
   **Tile3 = 65,536**.
   A weapon-swap request is refused, clears the pending requested weapon
   and uses the ordinary refusal line, file 0, noun 0, verb 37,
   condition 23. Other state changes are refused until exit is enabled.
   `tileExit` retains the shared behavior in [tile-puzzle.md](tile-puzzle.md):
   its exit switch also restores gravity and keep-on-ground; argument 1
   additionally requests Wait.
2. **Drawn weapons on entry.** With sword or bow drawn, play `away`,
   request empty hands through the normal weapon path, then force empty
   hands directly. That request meets the refusal above, so the refusal
   line can occur during entry. With empty hands, Tile2 immediately
   requests `stop`, remembers it and refreshes its highlight. Tile3
   does not make that immediate stop request; its normal update handles
   the subsequent input. Neither entry imposes Death Hall's camera
   hold. The authored scripts supply the camera placement or follow mode.
3. **Movement input.** Left and right use the ordinary turning motions;
   Walk or Run requests one forward jump; Back requests one backward
   jump. WaterWalk/WaterRun are ignored. A changed wanted movement starts
   the request. Landing clears the remembered input, so holding forward
   can start another jump after landing. No wanted movement requests
   `stop`. During a jump or Tile2 Rock throw, ordinary wanted movement
   is ignored. These controllers suppress the ordinary movement lean.
4. **Choosing a destination.** Consider the neighboring grid spots,
   including diagonals, subject to each puzzle's restrictions below.
   Select the direction closest to Connor's facing for forward travel,
   or closest to directly behind him for backward travel. Height does
   not decide which neighboring cell wins. Directions use Connor's
   actual current x/y and the copied destination positions, not the
   offsets between abstract cell centers. Equal angular candidates
   retain the first encountered one: increasing x column first, then
   increasing y row within that column. Turn toward the destination
   for a forward jump; turn toward its reflection through Connor's
   position for a backward jump.
5. **Jump motion and physics.** Once the turn finishes, use
   `tilejump1` for straight forward travel, `tilejump2` for diagonal
   forward travel, `tilebackjump` for straight backward travel and
   `tilebackjump2` for diagonal backward travel. Their authored frames,
   durations and movement steps are the shared data documented in
   [tile-puzzle.md](tile-puzzle.md), rule 11. The animation's steps move
   Connor through the normal mover; do not interpolate him directly
   between tile centers or replace the motion with an artificial arc.
   Gravity and keep-on-ground are off for the jump and restored before
   its landing check. Step-up and step-down are not changed: the normal
   Connor values are 325 units. Tile2 marks a requested jump active as
   soon as it requests the motion. Tile3 waits until the actual current
   motion matches that requested jump before marking it active and
   disabling gravity/keep-on-ground.
6. **Jump key.** The ordinary Jump action is also routed to the current
   tile controller's destination chooser. It uses the controller's
   remembered forward/backward direction. Tile3 rejects a new choice
   while a previous choice is still waiting to start; Tile2 lacks that
   check. Neither route independently rejects every in-progress jump.
   This is the same class of reentrant-input defect documented for
   Death Hall: do not reproduce it. Ignore a new Jump action during
   an active jump or throw, and record that deliberate exception.
7. **Highlight.** A selected tile's per-object unlit switch makes it
   full brightness, and the previous highlighted tile is returned to
   normal lighting. This is independent of the magical Reveal event
   and does not change its texture quality. Details differ below.
8. **Landing.** Completion of a tile-jump motion restores physics,
   clears remembered wanted movement, requests `stop` and updates the
   highlight. Then the controller applies its own landing rule. It
   does not require an authored tile script to implement that rule.

## Swamp: Tile2 and KQSwampTile

### Floor, movement and exits

9. **Grid.** The original `newWorld.cs` loads 42 objects named
   `tile<column><row>`, columns 0–5 and rows 0–6. Their x positions are
   50,000 + 2,000 times the column; y is 92,600 + 2,000 times the row.
   Their loaded z comes from the normal authored placement rules, not
   a common artificial flat plane. The state also knows four approach
   spots on the center row, y = 98,600: x = 46,000, 48,000, 62,000 and
   64,000, with heights copied from the adjacent edge tiles.
10. **Cell boundaries.** The cell lookup covers the open rectangle
    x = 45,000–65,000 and y = 91,600–105,600. The approach cells are
    included. Each cell is 2,000 square. An exact outer boundary is
    outside; an interior boundary belongs to the cell on its positive
    side. If both coordinates are outside, north/south classification
    takes priority. Cell lookup uses x/y, not standing height.
11. **Approach restrictions.** Leaving either actual side of the floor
    is allowed only toward its center-row approach spot. Thus the
    center row can leave straight, and either adjacent row can leave
    diagonally toward that same spot. From x = 48,000, a diagonal is
    allowed only into the floor; from x = 62,000 it is likewise allowed
    only into the floor. The farther approach spots remain available
    for straight travel. Other off-floor positions are not invented
    into extra traversable tile rows.
12. **Outside the lookup rectangle.** West/east positions request the
    ordinary `jump` motion, without the tile turn-to-target setup.
    Tile2 still treats its end as a landing and restores physics.
    North/south positions do not obtain an outside tile jump.
13. **Highlight differences.** Tile2's highlight chooser uses the same
    side-corridor restrictions as its destination chooser. Only the
    42 actual tiles can be highlighted. A backward choice uses the
    backward destination; turning or returning to stop updates the
    forward choice. Choosing an approach rather than a real tile
    clears the previous highlight. The outside-area early return does
    not itself perform a general cleanup of the previous highlight.
14. **Landing handoffs.** Landing on either west center-row approach
    spot sends Cue 1 to `fiendoor`. Landing on either east approach
    spot sends an ordinary ConnorAction to `Rtswdoor`, with Connor as
    the source. Other landings do not trigger a controller-level
    correct/wrong result. Hazard contact is the object rule below,
    independent of this x/y landing test.
15. **Script ownership.** `tiletrap.cs` enters Tile2 once per
    `Swamp::Tile::inTileState` visit. When power is off and the switch
    is on it schedules the authored totem cues and turns power on.
    On Off, it calls `tileExit 1` and clears both inTileState and
    PowerOn. `swdoor.cs` and `fiendoor.cs` also hand Connor into or out
    of this controller as part of their door scenes. Fiend-room entry
    clears inTileState, PowerOn and SwitchOn before its later scripted
    movements. Keep these ordinary script effects; do not replace them
    with a controller-only solved switch.

### Powered floor objects

16. **Archetypes and parameters.** `gastrap1.kq`, `gastrap2.kq`,
    `gastrap3.kq`, `stump1.kq` and `stump2.kq` use `KQSwampTile`.
    Its additional parameters are `heightNum` and `damFactor`.
    The default damFactor is the single-precision value written as
    0.48; the installed archetypes do not override it. They disable
    keep-on-ground, accept `INVITEM_Rock`, and use collision type
    16,417 and mask 5. Preserve the authored object geometry and bounds.
17. **Safe tiles are identified by having a script.** The powered
    gas/contact behavior requires that the tile have no script at all.
    It does not compare a particular script filename. The installed
    safe tiles have the intentionally empty `fiensafe.cs`:
    `tile02`, `tile11`, `tile16`, `tile20`, `tile23`, `tile26`,
    `tile31`, `tile32`, `tile34`, `tile35`, `tile45`, `tile54`.
18. **Connor contact gate.** A tile's ConnorCollision handler produces
    the hazard only if both `Swamp::Tile::PowerOn` and
    `Swamp::Tile::SwitchOn` are true, the tile has no script, and the
    time since its previous ConnorCollision is **strictly greater than
    two simulation seconds**. Every ConnorCollision updates that last
    contact time, including contacts that fail the gate. Continuous
    contacts at gaps of two seconds or less therefore keep postponing
    a further hit. This is not a repeating two-second damage timer.
    The original initial contact timestamp is zero.
19. **Contact effect and health.** A qualifying contact creates one
    `swmpgas.kqs` one-shot effect and queues two separate requests, in order:
    first a direct health-only stats change, then a normal attack from
    the tile, kind 2, base damage 1, with the forced-reaction flag zero.
    Both are due now. The direct health delta is the truncation toward
    zero of **−Connor level × damFactor × 10**. Thus it is based on
    maximum health, not current health. There is no minimum-health
    floor in this calculation. At default damFactor, level 1 loses 4
    directly and level 43 loses 206 directly; the later ordinary attack
    is additional and follows the shared attack rules. Preserve the
    stored single-precision damFactor: level 5 gives a direct loss of
    23, because its stored value is slightly below exact 0.48.
    The effect inherits the tile's full current transform, including
    orientation, with the world-z offset in rule 22. It plays at speed
    1 and deletes itself at the end; it is not a permanent static model.
20. **Distinct damage paths.** The contact handler does not apply
    protection, a random damage roll or the normal hit-state guards to
    that first direct stats request. The common stats receiver and the
    later ordinary attack retain their own rules, including their
    different behavior in Rope/Die and god mode; see
    [traps.md](traps.md), Q194, and [fighting.md](fighting.md), rule 29.
    Do not turn both requests into a single generic attack. Tile2
    permits the resulting death state instead of trapping Connor alive
    inside its movement controller.
21. **Rock/gas response.** The tile's attack receiver reacts only to an
    incoming attack whose damage amount is zero, with the same two
    power/switch flags and no-script gate. It creates the gas effect,
    without the contact cooldown and without directly queuing the
    contact health change or the extra one-point attack. A visible gas
    burst is therefore not sufficient evidence that Connor was damaged.
22. **Gas height.** Add the following world-z offset to the tile's
    placement when creating the effect. Other heightNum values use 0.

    | heightNum | Effect height offset |
    | --- | ---: |
    | 1 | 750 |
    | 2 or 12 | 1,000 |
    | 3 or 13 | 1,250 |

23. **Reveal.** Only scripted tiles respond to Reveal. For heightNum
    1–3, enabling it substitutes `I_gastrp.bmp` for `gastrap.bmp` and
    `I_wtowr7.bmp` for `wtower5.bmp`. For heightNum 12–13 it substitutes
    `I_trprng.bmp` for `trapring.bmp` and `I_trunk.bmp` for `trunk.bmp`.
    Disabling Reveal restores the object's normal bitmap overrides.
    Other heightNum values do not acquire these substitutions.
24. **Click and throw while remaining in Tile2.** A ConnorAction on a
    tile turns Connor toward that tile even without a Rock. If its
    supplied inventory object is an `INVITEM_Rock` (type-name comparison
    is case-insensitive), it starts a Rock throw. Outside Tile2 this
    requests the ordinary ThrowRock state. Inside Tile2 it remains in
    Tile2: handsOff is enabled, `away` plays, the previous projectile
    filename is remembered and replaced with the Rock item's projectile,
    and ordinary tile input pauses. At the end of `away`, play `throw`.
    At the end of `throw`, restore the previous projectile, clear the
    busy condition and release handsOff. Do not apply the ordinary
    ThrowRock state's final transition to Wait here. An explicit
    `setState Connor ThrowRock` while in Tile2 uses this same internal
    throw path. The authored `throw` cue and shared Rock projectile/drop
    rules remain those in [ranged-weapons.md](ranged-weapons.md), section
    8, and [traps.md](traps.md), Q203.
25. **Totems are a separate authored hazard.** The generic Cue 500
    projectile behavior and `totem.cs` provide the firing. That script
    schedules another Cue 500 eight seconds later only while both
    PowerOn and SwitchOn are true. This does not replace the floor's
    contact or Rock tests. Follow the script's actual event order,
    including already scheduled cues.

## Barren: Tile3 and KQLavaTile

### Grid, input and sequence

26. **Grid and approach spots.** `newWorld.cs` loads 35 `lavatile.kq`
    objects, `tile<row><column>`, rows 0–4 and columns 0–6. Unlike
    Swamp, the first digit is y: x = 86,640 + 2,000 times column and
    y = 151,943 + 2,000 times row. Their authored initial z is −700.
    Tile3 copies the positions and adds a center-column approach before
    and after the floor, at (92,640, 149,943) and (92,640, 161,943),
    using the nearest center tile's copied height.
27. **Cell boundaries.** Actual floor lookup uses the open rectangle
    x = 85,640–99,640, y = 150,943–160,943, with 2,000-square cells.
    Exact outer boundaries are outside; interior boundaries select
    their positive-side cell. North/south takes priority over west/east.
    The two approach points are outside this rectangle, not extra
    normal tile rows.
28. **Unsolved and solved destinations.** Before Connor's persistent
    Tile3-completion flag is set, ordinary neighbor selection excludes
    `tile42`, `tile43` and `tile44`. After completion, it allows only
    center-column destinations, column 3. It does not exclude a tile
    merely because its platform is down, hidden or currently moving:
    collision and the authored animation decide whether Connor is
    physically supported. A selected neighbor is not a promise of a
    safe landing.
29. **Leaving the floor.** From row 0, columns 2–4, the south approach
    competes with ordinary neighbors by facing angle. From solved row
    4 the north approach also competes. A selected exit point marks
    that jump for exit handling. From outside south, the chooser
    compares row-0 columns 2–4 with the farther south exit direction
    toward (92,640, 144,000, 1,000). From outside north it compares
    `tile43` with the direction toward (92,640, 162,000, 1,000).
    Outside west/east does not gain a tile jump. South choices use
    diagonal motion for columns 2/4 and straight for column 3 or exit;
    the north approach uses straight motion. Backward travel uses the
    corresponding backward motion and mirrored facing target.
    The outside-south branch still compares columns 2–4 after
    completion; rule 28's solved-column restriction belongs to the
    ordinary inside-grid neighbor branch.
30. **Highlight is not the full movement validator.** Tile3's normal
    highlight considers ordinary adjacent actual cells by forward
    angle, without the unsolved/solved destination exclusions in rule
    28 and without the approach exit candidates. A backward jump sets
    the chosen landing tile's highlight as it begins. Normal turn/stop
    highlighting can therefore differ from the permitted jump target.
    Do not infer permission or puzzle correctness from brightness.
    An outside-area early return leaves the old highlight alone.
31. **Prime sequence.** The seven significant cells and their rewards
    are fixed by position, not read from arbitrary tile labels:

    | Tile | Number | Revealed top bitmap | Successful-step sound |
    | --- | ---: | --- | --- |
    | `tile12` | 3 | `lvtile03.bmp` | `T3.aud` |
    | `tile41` | 5 | `lvtile05.bmp` | `T5.aud` |
    | `tile36` | 7 | `lvtile07.bmp` | `T7.aud` |
    | `tile05` | 11 | `lvtile11.bmp` | `T11.aud` |
    | `tile20` | 13 | `lvtile13.bmp` | `T13.aud` |
    | `tile45` | 17 | `lvtile17.bmp` | `T17.aud` |
    | `tile24` | 19 | `lvtile19.bmp` | `T19.aud` |

    Landing on 3 starts or restarts a valid sequence and plays its
    sound. Every other prime is successful only if the sequence is
    still valid and the previously visited significant prime is its
    immediate predecessor. An incorrect prime invalidates the sequence.
    Either way it becomes the remembered previous prime. Landing on a
    nonsignificant tile does **not** reset the sequence or previous
    prime. There is no teleport or direct health penalty for a wrong
    prime. The platform and terrain hazards still apply normally.
32. **Revealing a prime is separate from succeeding.** The first
    landing on each significant cell during this controller's history
    sends Cue 2400 to it with the corresponding bitmap, even if the
    sequence is wrong. Later landings still evaluate sequence progress
    and play a successful-step sound when appropriate, but do not
    repeat that first-reveal cue. Previously revealed prime platforms
    therefore remain held up while the player repairs the sequence.
33. **Completion.** A valid 17→19 step plays `T19.aud`, sets Connor's
    persistent Tile3-completion flag and `Barren::Tile::Complete` true,
    and queues **50,000 experience**. It adds `ba_Order.bmp` at map
    position (155, 177) when the magic-map object exists. The center
    column `tile03`, `tile13`, `tile23`, `tile33`, `tile43` receives
    Cue 2400 with `lvtile01.bmp`; `tile43` receives Cue 2403 first.
    Every other tile except Connor's final `tile24` receives Cue 2401.
    The final platform is preserved until his next landing away from
    it, when it too receives Cue 2401. Completion creates a usable
    center bridge; it does not teleport Connor or exit the controller.
34. **Exit handoff.** When an exit-marked jump ends, physics is restored
    and leaving Tile3 is enabled. If Connor's health is positive, run
    `tiletps2.cs` with `StartExit` as its single argument when the cell
    lookup reports south; otherwise run `tiletpn2.cs` the same way.
    This is a script invocation, not an invented On/Cue callback.
    A nonpositive-health Connor does not receive that handoff. The
    installed loose exit scripts run him to y = 145,100 or **166,800**
    at x = 92,640, then use their ordinary callback chain, including
    a 0.8-second cue, to release handsOff. Their trap callbacks rearm
    the corresponding entrance and collision mask.
35. **Music and leaving the state.** Entry queues `mlava.aud` playback
    after 0.2 simulation seconds. Leaving queues stopping that named
    music after 0.2 seconds. If Connor has not completed Tile3, leaving
    also sends Cue 2402 to all 35 tile objects. Once completed it does
    not send that shutdown sweep, preserving the bridge.

### The moving platform class

36. **Initial state and authored motion.** `lavatile.kq` begins hidden
    in `stop`, with keep-on-ground off. Each platform initially has no
    activation references, no hold request and no marked-metal variant;
    automatic cycling is off. The ordinary pair is `up`/`down` with
    stationary endpoints `stopup`/`stop`; the marked pair is
    `up1`/`down1` with `stopup1`/`stop1`. Read these from the eight
    authored movements. `up` and `up1` have 31 frames over 2 seconds;
    `down` and `down1` have 51 frames over 3.3333 seconds. Endpoint
    motions hold their authored final pose. Use the actual moving
    geometry and ordinary bounds/collision behavior, not one permanent
    collision plane. This class has no separate Swamp-style contact
    damage handler; it inherits ordinary ConnorCollision behavior.
37. **Cue protocol.** All four cues act on one platform's own state:

    | Cue | Meaning and immediate behavior |
    | --- | --- |
    | 2400 | Hold up. With a non-null bitmap argument, convert to the marked variant, remember/apply the replacement for `lvtile01.bmp`, and raise or reverse as described below. A missing bitmap still sets hold-up but does not perform the conversion. |
    | 2401 | Hold down. A stationary raised platform begins its matching down motion; an ascending platform reverses. |
    | 2402 | Remove one activation reference. Only when the resulting count equals zero, turn automatic cycling off and lower/reverse it like the previous row. |
    | 2403 | Add one activation reference. Only when the resulting count equals one, unhide, start the matching up motion and enable cycling. `tile43` is an explicit case-insensitive name exception: it performs those actions regardless of the resulting count. |

    Activation counts are signed and are **not clamped at zero**.
    This matters when Tile3's exit sweep precedes a delayed initial
    activation: a pending activation can bring −1 back to 0 without
    restarting the platform. It also explains why the specially
    activated final bridge tile must not lose its `tile43` exception.
38. **Ordinary repeating cycle.** An ordinary upward motion ends at
    `stopup` and schedules `down` **seven seconds later**. A normal
    downward motion ends at `stop` and, if not held down or turned off,
    schedules `up` **0.1 seconds later**. Otherwise it becomes hidden.
    These are named animation-play events. At delivery the class accepts
    them only while there is no hold request and cycling is enabled.
    Thus a previously queued down/up event does not override a later
    hold or shutdown. A normal uninterrupted ordinary cycle takes about
    12.4333 seconds, before update-boundary/event-delivery rounding.
39. **Marked platforms.** A marked upward motion ends at `stopup1`
    without scheduling the ordinary seven-second descent. A marked
    downward motion ends at `stop1`; it hides when held down or off,
    and does not schedule an automatic ascent. Marked prime tiles are
    stable stepping stones unless later explicitly lowered.
40. **Hold-up conversion in motion.** Cue 2400 with a bitmap requests
    `up1` from `stop`, or `stopup1` from `stopup`. During `up`, it
    makes the current motion interruptible, changes to `up1` while
    preserving the phase, and applies a relative **+50 world-z-unit**
    correction. During `down`, it similarly changes to `down1` at the
    same phase, negates the playback speed and records that it reversed.
    Other current motion names do not receive these conversions, but
    the hold, bitmap and marked-variant state still change.
41. **Lowering and reversed endpoints.** Holding down or turning off
    starts `down`/`down1` from the matching raised stationary pose.
    During `up` or `up1`, it negates playback speed and records reversal.
    When that reversed ascent ends, restore the speed's sign, select
    the matching bottom stationary pose, apply **−50 world-z units**,
    and hide the platform. When a reversed `down1` ends, restore the
    speed's sign, apply **+50 world-z units** and select `stopup1`.
    These are translation corrections, not rotations or texture scaling.
    Preserve the phase and signed playback behavior; restarting a full
    up/down clip would change the time the platform supports Connor.
42. **Other interactions.** The platform's ConnorAction is consumed
    without initiating a puzzle move or forwarding an ordinary tile
    click script. Jump choice belongs to Connor's Tile3 controller.
    Its motion-name comparisons are case-insensitive, as is the
    special `tile43` comparison. This is separate from ordinary
    object-name resolution.

## Authored setup and persistence boundaries

43. **Barren setup uses the installed patch scripts.** Initially
    Complete is false, NorthBridge is Off and SouthBridge is On.
    `barren/patch/tiletps.cs` schedules 32 staggered Cue 2403 requests,
    from 0.7 through 11.4 seconds; `tile42`, `tile43`, `tile44` are
    omitted. Read the individual delays from that script instead of
    making all platforms start together. Initial entry walks Connor
    to the approach and, after its callback and 0.7-second cue, places
    him at (92,640, 149,800, 600), facing 3.14, before entering Tile3.
    Completed south reentry uses the installed **one-second** cue and
    placement (92,640, 149,940, 600). The installed north entry likewise
    uses a one-second cue and (92,640, 162,000, 600), facing 0. Both
    scripts position the camera explicitly. The archives and loose
    overrides differ; loading an archived version directly is not an
    installed-game equivalence check.
44. **Controller state retained in original saves.** Tile2 serializes
    its pending-jump choice, backward direction, exit permission, busy
    condition, remembered wanted motion, highlighted cell and the Rock
    throw's old projectile/phase flag, in addition to base state data.
    Tile3 serializes its pending choice, previous significant prime,
    sequence-valid condition, already-revealed-prime set, highlighted
    cell, backward direction, exit permission, busy condition,
    remembered input, exit-marked jump and deferred lowering of the
    final platform. Connor also retains completion separately.
    The copied tile positions are not serialized by either controller;
    their first update after restoration can rebuild them from the
    named world objects and refresh the remembered current motion.
45. **Platform state retained in original saves.** A KQLavaTile adds
    its hold choice, activation count, selected top bitmap, reversal,
    marked variant and cycling-off condition to ordinary animated-object
    persistence. A KQSwampTile adds heightNum and damFactor to ordinary
    object persistence. These facts establish what must remain distinct
    in a recreation save; they do not establish complete original save
    binary compatibility or event-queue reconstruction.

### Persistence follow-up: restored controllers, objects and timed events

The following rules were traced separately on 2026-11-07 after the
initial note. They narrow the earlier save uncertainties without claiming
an original runtime or native-save test.

46. **Restore binds an existing saved controller; it does not enter it
    again.** Connor's saved current state is reconstructed as that state
    type with its saved properties. Character registration then binds
    that state back to the newly loaded Connor. This binding is distinct
    from the ordinary entry action. A saved Tile2/Tile3 must not replay
    its weapon-away/refusal entry, reset its exit permission, or run
    Tile3's entry music request as though it were a fresh state change.
    In particular, binding is not a Tile3 exit and must not send its
    all-platform Cue 2402 sweep. Connor's ordinary saved properties also
    retain handsOff, the tile-jump turn condition and solvedTile3.
47. **Grid rebuild comes before the restored busy check.** The fresh
    controller initially has no copied grid. Its first normal update
    rebuilds positions from the currently loaded named tiles, then
    refreshes Connor's remembered controller-motion label from his
    actual current motion. Only then does it check whether a saved jump
    or throw is still busy. It does not discard the saved pending jump,
    sequence, reveal history, exit-marked jump or Rock-throw state.
    Building the grid is not a substitute for loading the tile objects
    first. The positions used are the objects' restored positions,
    including their saved motion-related placement, not the pristine
    coordinates from the regional setup script.
48. **Animation phase is ordinary animated-object persistence.** The
    current, next and transition motion names, their start/transition
    information, the current normalized phase and ordinary animation
    flags are saved by the shared animated-object path. Initial playback
    selects the saved current motion and advances its phase to the saved
    position when that position is beyond its starting phase. Loading a
    platform stationary at its top does not itself invent a fresh
    seven-second wait: that wait is a queued animation event, as below.
    The wider question of exact replay/suppression of every authored
    cue around a restored phase is not settled here.
49. **Important reversed-platform limitation.** The platform's
    reversal marker is saved, but its live signed playback speed is not
    a serialized animated-object property. The traced initial-playback
    path takes speed from the authored motion, which is positive for
    these lava motions; KQLavaTile has no separate restore hook that
    reapplies a negative speed. Thus the traced reload path does not
    preserve an in-progress reverse simply because its reversal marker
    survived. It starts the saved motion/phase through ordinary positive
    playback, while the saved reversal marker still selects the
    reversed-end handling in rule 41. This is a static original-save
    limitation, **not runtime-tested**. Do not describe a recreation
    save that deliberately preserves negative speed as proven identical
    to this original reload behavior. Keep that case distinct in the
    continuation checks.
50. **Highlight bookkeeping is not the highlight render flag.** The
    controller's previous highlighted cell is saved. The tile's
    per-object unlit switch is not part of the traced inherited object
    save properties and starts off on reconstruction. The grid-rebuild
    step does not force a highlight repaint. A later selection equal to
    the remembered highlighted cell takes the ordinary no-change path;
    moving the selection to another cell sets the new highlight. A
    recreation that always reapplies the old highlight immediately on
    reload would be adding behavior beyond this traced path. Reveal
    texture substitutions and the lava top bitmap are separate state.
51. **Swamp contact history resets, rather than pausing.** The
    KQSwampTile save path and its inherited object save path do not
    serialize the previous ConnorCollision timestamp. The fresh tile
    starts with timestamp zero, as in rule 18. The world clock itself
    is restored, so the first eligible contact after load tests the
    restored current world time against zero, not against the previous
    session's last contact. When that time is greater than two seconds,
    an unscripted powered tile can qualify immediately. The usual
    registration/contact eligibility still applies, and every received
    contact then updates the timestamp normally. Do not serialize a
    remaining two-second cooldown and call that original continuation.
52. **Saved simulation events are merged into the live queue.** The
    generated restored-world startup loads `WorldState.kq` after the
    restored objects. It reconstructs each saved persistent game event,
    resolves its target by the saved object name, restores its event
    payload and **absolute world-time deadline**, then inserts it into
    the existing queue. This includes the tile cues and named
    animation-play events. The loader does not clear the queue at this
    point, deduplicate equal target/type/payload events, compare script
    source lines, or replace newly posted Init work with an equivalent
    saved event. If an Init event is still queued, it remains alongside
    the saved event, with its own deadline. For equal deadlines, a new
    insertion goes after events already queued at that deadline; the
    saved entries retain their serialized order relative to one another.
53. **Frozen world time preserves remaining delays.** Restored-world
    startup sets the saved world clock before its object-loading work,
    as documented in [world-change.md](world-change.md), rule 14.
    Time spent away from that region does not advance its platform
    cycle or delayed start cues. For example, an existing downward
    animation request due at world time 107, saved at time 103, remains
    due four simulation seconds after that clock is resumed. It does
    not receive a fresh seven seconds and is not made immediately
    overdue by wall time spent loading or in another region. The same
    rule covers the 0.1-second bottom wait, staggered Cue 2403 starts,
    and already queued totem cues. At delivery the platform still uses
    its restored hold/off/activation state to accept or suppress work.
54. **World and actor continuation differ on region return.** A kept
    world's generated startup restores its tile objects and regional
    variables instead of rerunning the original first-visit tile setup.
    The signed activation counts, held/marked platform state and saved
    event queue therefore survive region return without restaggering the
    initial platform starts. Connor arrives from the source world in
    his ordinary Portal/arrival sequence, carrying his own persistent
    solvedTile3 property. He is not restored into an old copy of Tile3
    merely because the destination region once contained that state.
    Reentering its puzzle later uses the scripts' completed/unfinished
    gates and a normal controller entry. A direct save reload while in
    Tile3 instead uses rule 46's state rebinding.
55. **Relationship to Q201.** The controller and event save properties
    above do not require a special tile-specific trap restoration
    algorithm. They do require the shared loading order already traced
    in [scripted-controller-commands.md](scripted-controller-commands.md),
    rules 7–8: saved fields exist before active collider registration;
    eligible initial contacts can happen synchronously during
    registration; object Init follows, and the enclosing Load script
    continues in authored order. A matching pending saved trap occupant
    attaches without a new On callback. This prevents treating every
    restored overlap as a fresh entry scene. Swamp floor contact is a
    separate object callback, not trap occupant restoration: if
    registration produces an eligible ConnorCollision, rule 51's reset
    timestamp and rule 18's power/script gates apply then. Preserve
    posted event ordering rather than postpone all such effects to the
    first requested movement. The unresolved repeated-save case with
    partially reattached trap lists remains Q201's separate limitation.

Rules 52–53 answer the saved-versus-Init queue policy portion of Q191
for these persistent game events. They do not establish every temporary
event class, missing-target failure case, or restoration of external
`getEndLoop` listeners. An empty event count in a particular save does
not imply that future tile cues are intentionally discarded.

## Data used

- Swamp `kq.vol`: `newWorld.cs`, `worldvar.cs`, `tiletrap.cs`,
  `swdoor.cs`, `fiendoor.cs`, `fiensafe.cs`, the five floor archetypes,
  `totem.kq`, `totem.cs`, `prrock.kq`; the shared installed Rock item,
  `rockHit.cs`, Connor movement data and gas effect.
- Barren `kq.vol`: `newWorld.cs`, `worldvar.cs`, `lavatile.kq`; installed
  loose `barren/patch/tiletps.cs`, `tiletpn.cs`, `tiletps2.cs`,
  `tiletpn2.cs`; `resource.vol` platform shapes `lvtileup.kqs`,
  `lvtiledn.kqs`, `lvtileu2.kqs`, `lvtiled2.kqs`; authored number/top
  bitmaps, sounds, music and map overlay.
- Shared rules: [tile-puzzle.md](tile-puzzle.md),
  [connor-physics.md](connor-physics.md), [objects.md](objects.md),
  [object-lighting.md](object-lighting.md), [traps.md](traps.md),
  [terrain-hazards.md](terrain-hazards.md), and
  [ranged-weapons.md](ranged-weapons.md).

## Edge cases and do not reproduce

- Both original grid builders assume every named tile exists. Do not
  reproduce a null-object failure. Log missing tiles and withhold an
  invalid destination or puzzle start.
- Do not substitute Death Hall's immortality/state lock, camera hold,
  expected-letter resets or teleports for these two controllers.
- Keep the explicit Jump-key guard exception in rule 6, rather than
  treating accidental reentrant inputs as a designed feature.
- The Swamp script mentions `totem.07`, but the inspected original
  setup loads only six totems. Do not invent a seventh object to repair
  a reference; preserve the normal missing-target event behavior.
- Do not clamp platform activation counters: their negative values
  participate in canceling delayed entry work. Do not erase prime
  progress merely because a nonsignificant tile was crossed.
- The completion handler itself has no extra global already-awarded
  guard in its successful 17→19 branch. Ordinary solved movement
  restrictions keep subsequent travel on the center bridge. Any
  additional defensive handling of externally forced repeated
  completions should be identified as such, not claimed as traced.

## How to check without manual play

Use representative families, with original data supplied at runtime;
none requires a human playthrough now:

1. **Movement/state family:** one straight and diagonal forward/back
   choice, Swamp side corridor, Barren south/north approach, allowed
   death versus refused weapon swap, and one held-key landing/restart.
   Log selected cell, motion, turn completion, physics switches and
   input ownership. Include Barren's permitted-target/highlight mismatch.
2. **Swamp hazard family:** one scripted safe tile and one unscripted
   tile; both power flags; contact intervals below, at and above two
   seconds; zero versus nonzero attack damage; levels 1, 5 and 43.
   Log the direct stats and attack as separate ordered deliveries.
   One Rock throw checks that Tile2 survives through projectile restore.
3. **Barren sequence family:** correct primes with intervening ordinary
   cells, wrong prime followed by restart at 3, and repeated visits to
   an already revealed prime. Confirm sound versus first-reveal cue,
   one completion's XP/map/bridge effects and deferred final-platform
   lowering. The fixed seven-cell table is data for one rule, not seven
   separate gameplay systems.
4. **Platform lifecycle family:** one full ordinary cycle, one prime
   converted during ascent, one converted during descent, one shutdown
   during ascent, and delayed activation after an early exit. Check
   phase/signed speed, ±50 vertical corrections, visibility, reference
   count, and suppression of stale queued animation-play events.
5. **Script handoff family:** installed south entry, completed reentry
   and one exit callback chain. Check the installed override's delays
   and north run endpoint, rather than silently using the archive.
6. **Persistence family:** serialize the distinct pending/active jump,
   sequence, throw and platform properties above in recreation-owned
   saves. Treat successful recreation roundtrips separately from the
   unanswered original native-save continuation questions below.
   Add one same-deadline Init/saved-event pair, one paused-region
   platform wait, a saved negative activation count, Swamp's reset
   contact timestamp, and the original reversed-playback limitation.
   Rebinding a saved controller must not emit new entry/exit effects.

## Open questions

- Original save continuation was not run. The follow-up establishes
  state rebinding, grid rebuild, ordinary event merging/deadlines,
  contact-timestamp reset and the highlight/signed-speed limitations.
  Exact authored-cue replay near a restored phase, external EndLoop
  listeners and malformed/missing-target event recovery remain outside
  this bounded study. Do not claim native-save parity from recreation
  roundtrips alone.
- This study identifies the ordinary terrain/mover hazard path around
  the Barren platforms, but does not settle the remaining `ontop`
  liquid-surface branches in [terrain-hazards.md](terrain-hazards.md).
  Do not replace those unresolved branches with an invented automatic
  death merely because a platform animation is down.
- The shared Rock path's full aim/damage behavior remains bounded by
  the open items in [ranged-weapons.md](ranged-weapons.md). This study
  establishes the Tile2 ownership exception and the receiving tile's
  zero-damage gas rule, not a new general projectile model.
