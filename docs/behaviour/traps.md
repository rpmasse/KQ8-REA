# Traps (trigger boxes)

Study session A, 2026-10-13. Sources: Daventry's trap `.kq` files and
scripts (`MASK/daventry/kq/`), all 12 worlds' trap settings, and
`Mask.exe` (studied, described here in our own words).

## Purpose

A `KQTrap` is an invisible box in the world. When Connor walks into it, or
out of it, the trap runs its script. Daventry uses 50 traps for: switching
between walking and running near houses and the camera range that goes
with it (`walk_la.cs` / `run_la.cs`), music zones (church, wizard), rope
climbing spots, teleporter pads, the cliff-edge cutscene, telling monsters
to go home, and save-game prompts. Nothing in a trap is drawn, and a trap
never blocks anything.

## Rules

### What a trap is made of (data)

1. Keys a trap reads (all optional except the box): `loc` (`locX/Y/Z`),
   `BBMinX/Y/Z`, `BBMaxX/Y/Z` (the box, in the trap's own space, i.e.
   added to `loc`, never turned), `script`, `colliderMask`, `trapType`,
   `innerPercent`. Unknown to traps and ignored: `keepOnGround` (22
   Daventry traps have it; a trap is never moved onto the ground by it).
2. `locZ = -1` means "on the floor grid" (the ground under water,
   `water.md`): the z is replaced by the floor height at the trap's x, y
   when the trap enters the world. Only −1; a −2 is not special for traps.
3. Traps without `locX/Y` (e.g. `conwalk`, `conrun`) are templates; world
   scripts place copies with `loadKQ` and a position (`objects.md`).
4. **Who sets a trap off** (corrected 2026-10-13, Q57, Q58). A trap's own
   collision type is 0x10; its mask is `colliderMask`. **Only the trap's
   own mask counts:** a trap is told about a touch when its mask covers
   the visitor's `colliderType` (`connor-collision.md` rule 2; the
   two-sided rule only decides which pairs are looked at). Connor's own
   mask has bit 0x10, but that only means Connor is told too, and he
   ignores traps (rule 5). Any object can be a visitor, not only Connor.
   The masks in the data:

   | Mask | Set off by | Traps |
   | --- | --- | --- |
   | 4 | Connor (type 4) | 285 traps, all 58 in Daventry |
   | 516 (4 + 0x200) | Connor, inventory items (type 0x220) | 17 in Temple1 (`trap01.kq`: ";Collide with Conner=4, Inventory=512"), IceWorld `pondtrap`, Swamp `platrap` |
   | 517 (4 + 0x200 + 1) | the above and anything with type bit 1 | Gnome `triptrp1–4` |
   | 519 (4 + 0x200 + 2 + 1) | the above and anything with type bit 2 | DeadCity `triptrp2–5`, `trptrp12–15` |
   | 5 (4 + 1) | Connor and type bit 1 | Gnome `agtrap`, `aptrap`, `aptrap2`, `wgtrap`, `wgtrap2` |
   | 1 | type bit 1 only, **not Connor** | Gnome `bouldtrp`, `bouldtp2`; DeadCity `girltrap`; Temple2 `booktrap` |
   | 2 | type bit 2 only | DeadCity `fightrp3` |
   | 6 (4 + 2), 7 (4 + 2 + 1) | Connor and type bit 2 (and 1) | Temple4 `vortrap`; DeadCity `coaltrap` |
   | 0 | **nobody**, until a script adds bits | Barren `gongtp*`, `tiletpn`; DeadCity `portrap2–5`, `portrp2b–5b` |

   Type bit 1 is on most solid objects and many monsters (0x21, 0x101
   `boulder`, 0x121 `pushbook`, 0x141 push blocks, 0xC021 monsters, the
   rolling rocks' 0x1). Type bit 2 is on creatures (monsters 0x42,
   0x4042, 0xC022, 0x8022 gnomes, 0x4002 …). The scripts fit: `bouldtrp`
   checks for the `boulder`, `booktrap` for `pushbook`, `girltrap` for
   the pushed box `girlbx.02`, `coaltrap` sets whatever entered on fire.
   Mask 0 traps get bits from scripts (`KQObject::collision portrap2 +4`,
   `KQObject::collision gongtpX 4`).
5. **A trap is never solid** (traced 2026-10-13). Connor's reaction to a
   touch with type 0x10 is nothing, and a plain object's is nothing too
   (`connor-collision.md` rule 3), so nobody is stopped or pushed by a
   trap.

   **Movement and initial collision registration are separate contact
   opportunities** (corrected 2026-11-07, Q203). Ordinary movement probes
   for touches (`connor-collision.md` rule 1): an item while tossed or
   falling, a box while pushed. Starting collision participation also
   checks eligible overlaps. A newly registered stationary world item
   can therefore enter a trap without a subsequent move; see the Rock
   study below. Merely remaining still does not repeat entry.

   **Placed by a script (traced 2026-10-19, Q102).** A script's `move
   <object> …` (Connor or any other object) goes through the same
   movement routine as an ordinary step, but as a move that takes no
   time. For such a move the routine does **not** sweep: the object's
   collision shape is simply put at the new spot (its "last checked"
   spot becomes the new spot too), and the move is committed at once.
   So:
   - **(a) The path is not tested, and neither is the new spot at that
     moment.** No trap between the old and the new spot is entered, and
     a trap whose box contains the new spot is not entered yet. It is
     entered at the object's **next ordinary move** that sweeps (one that
     changes his position or facing): that sweep starts from the new
     spot, so a box that contains it counts as touched ("starts inside",
     rule 6). For Connor in Daventry's cutscenes that is the first update
     he moves again (an animation with no steps does not count).
   - **(b) The trap he was in is told at the move itself.** Committing
     the move sends the usual "moved" message; the trap runs its stay
     test (rule 7) when that message is handled, which is right after the
     script that did the `move` (and the event that ran it) has finished,
     in the same update. If he is now outside: `Off` (or `Cue 302`) then,
     not at his next move. (Daventry's mill wheel: `wheeltp2` gets `Off`
     right after its cue 2 moves him under the wheel; its script does
     nothing on `Off`, so nothing visible changes.)
   - **(c) New collision registration (corrected Q203).** The earlier
     blanket claim that `loadKQ` cannot cause entry until movement was
     wrong: registering an active collider checks existing eligible
     overlaps at its current placement. This is separate from the
     zero-time move of an already registered collider in (a). The
     complete stationary-contact route is verified for fresh world
     inventory items, including a Rock made by `dropItem`. Entry still
     requires a collision shape, active participation, an eligible
     collision-detail pair, the trap's accepting mask and actual shape
     contact. This is not a rule to send `On` for every `loadKQ`, item
     creation or placement regardless of those conditions.

### Entering and leaving

6. **Entering.** When a visitor's collision shape touches the trap's box,
   it is added to the trap's "inside" list (once; touching again while
   inside does nothing). If it is the **first** one inside, the script
   runs with `$1` = trap name, `$2` = `On`, `$3` = the visitor's object
   name. If someone else was already inside, it runs with `$2` = `Cue`,
   `$3` = `301`, `$4` = the newcomer's name. In Daventry only Connor sets
   traps off, so there it is always `On` with `$3` = `Connor`.
   **"Touches" (traced 2026-10-13):** during movement, found by the same swept test as
   walls (`connor-physics.md` "Collision shape"), using the visitor's own
   shape: for Connor, items and other ball stacks, each ball's path is
   tested against the trap's box widened by the ball's radius; for a box
   object, its box. It is tested **along the whole move**, so a fast
   move that passes right through a thin trap in one update still
   enters it (and the next move's stay test, rule 7, then finds him
   outside: `Off`). A visitor whose shape **starts inside** the box also
   counts as touching, at its next move. Initial registration can also
   find a stationary ball stack touching the trap (Q203 below).
7. **Staying.** Every time an object inside moves, the trap checks
   whether it is still inside: first, is the **centre of the object's
   box** (its position plus the middle of its `BOUNDS.KQ` box) inside the
   trap's box (both corners included)? If yes, still inside. If not, is
   the object's collision shape still touching the trap's box? If yes,
   still inside. Only when both fail has it **left**.
8. **Leaving.** The object is taken off the list. If the list is now
   empty, the script runs with `$2` = `Off`, `$3` = the object's name.
   Otherwise `$2` = `Cue`, `$3` = `302`, `$4` = the name. While the world
   is being shut down, no `Off` is sent.
9. So `On` comes when he first **touches** the box, and `Off` only once
   he is **out of touch and his middle is outside** it.

### Recoverable Rock, registration and removal (Q203, 2026-11-07)

Static study of the original executable, checked against the installed
Dead City archive's `prrock.kq`, `triptrp5.kq` and `triptrp5.cs`, and the
loose `rockHit.cs`, `Invinfo.itl` and `BOUNDS.KQ`. Original runtime,
native save/reload and manual play **not tested**. This corrects rule
5(c)'s earlier Q102 conclusion; the existing-collider script-move rules
5(a–b) remain unchanged.

**Which object holds the plate, and when**

- The authored Rock projectile has collision type **1**, mask **0x1f**
  and `collisionDetail=1`. A trap uses collision detail **1** too.
  **A pair where both have detail 1 is excluded before geometry and
  contact delivery.** Thus the flying projectile does not become this
  trap's occupant, even though `triptrp5`'s mask 519 includes type 1.
  Collision detail is distinct from the choice of box or ball shape.
- On the projectile's `Die`, `rockHit.cs` reads that projectile's final
  position and angles and calls `dropItem Connor INVITEM_Rock` there.
  The command deducts **one carried Rock** and creates a **fresh world
  item of quantity 1**, even when the carried quantity was only 1.
  This is a Rock-category exception, not a general identity rule for
  other dropped items (`items.md` rule 11). The projectile is not
  retained and there is no transfer of trap membership.
- **Last-carried-item edge:** common inventory removal first puts the
  old quantity-1 object into world ownership at its stored carried
  placement, normally the origin. Its quantity stays 1. The Rock branch
  then creates the distinct landing item; no cleanup of the old world
  identity was found in this command chain, including inventory UI
  updates. With more than one carried Rock, only the carried count is
  reduced and the old identity stays carried. Thus dropping the last
  Rock is not a pure replacement of its old identity. This is static
  original behavior, with its runtime consequences **not tested**.
- The new world item has collision type **0x220**, mask **0x809**,
  collision detail **2**, and its ordinary ball-stack shape. When its
  collider becomes active and is registered, stationary overlap is
  tested immediately. Each ball centre is tested against the trap box
  expanded by that ball's radius, including boundaries. The item's
  full current orientation rotates each local ball centre before its
  position is added; the radius stays unchanged. Pitch/roll therefore
  need not leave the stack vertical in world space (heading alone
  leaves a centre on the local z axis unchanged). No nonzero
  displacement is required. `triptrp5`'s mask **519** accepts its 0x200
  bit; the item's mask need not contain the trap's 0x10 bit.
- Successful contact adds the new item's own identity and runs the
  ordinary `On`, or `Cue 301` if another occupant already exists,
  **synchronously during registration**, before `dropItem` returns.
  The callback names the fresh item (`NoName<n>`), not Connor, the
  carried stack or projectile. After the common inventory removal
  described above, `dropItem` sets `NewInvItem` to the selected carried
  object's name; only **after** fresh world
  insertion and its nested trap callback return does it replace
  `NewInvItem` with the fresh item's name. A nested callback must use
  its visitor argument, not assume `NewInvItem` already names it.
  Any contact caused by returning the last old identity to world
  ownership occurs even earlier, before that first assignment.
- Contacts are processed sequentially in existing collision-registration
  order. A callback can change the item's position or collision masks
  before later candidates are tested; later tests use that current
  state. Pending deletion is checked before narrow contact delivery.
  Do not turn a single creation-position snapshot into a batch of
  callbacks that ignores changes made by earlier callbacks.
- After the `Die` script returns, the projectile requests deletion.
  Removal is deferred until the current queued event finishes, before
  the next queued event. The ordinary authored sequence therefore has
  world-item entry and then projectile cleanup, with no projectile
  departure callback. The new item stays motionless on the plate; it
  does not need a fall, synthetic movement or forced `On`.

**Pickup, deletion and saved membership**

- Actual deletion tells every trap holding the deleted identity to
  remove it before the object and its name disappear. This invokes
  ordinary `Off` when last, otherwise `Cue 302`, with the departing
  object's name. World shutdown suppresses these callbacks. Requesting
  deletion and completing deletion are separate moments.
- Picking up this Rock into an existing carried Rock stack merges its
  quantity and requests deletion of the world item; that deletion
  releases the plate. Without a merge, carrying retains the item
  identity, places it at the origin and deactivates it. Committing
  that relocation posts the ordinary stay checks, which release this
  Dead City plate after the current event. This second path is not an
  unconditional direct-release operation: a hypothetical trap at the
  origin can expose different original bookkeeping.
- A save stores occupant object names in the trap's `trappedObject`
  entries. On restoration, contact with a saved name, matched ignoring
  capitals, reattaches that identity **without `On` or `Cue 301`**.
  Initial registration supplies such contact for a stationary restored
  item. While saved names remain to be reattached, other contacts do
  not take the ordinary new-entry callback path. The `Lookup` script
  call is separate and does not itself rebuild membership. After a
  completed Rock landing, the retained occupant is the world item's
  name; pickup/deletion later releases that same identity.

**Derived numeric check, not an original runtime test.** `triptrp5`'s
world box is **(113742,108426,50)–(114242,108926,525)**. The installed
`rok.kqs` bounds yield one ball of radius **105**, at local **(0,0,105)**
(the `Invinfo.itl` radius 250 is not this collision radius). With zero
item rotation, an item placed at **(114047.86,108670.29,0)** has its ball
centre inside the radius-expanded trap box and qualifies immediately
on active registration. This does not establish the original throw's
aim, arc or landing angles; those remain Q202.

For implementation checks, distinguish fresh active registration from
repositioning an existing collider; test both one and several carried
Rocks, first and additional trap occupants, rejected masks/detail pairs,
both pickup branches, ordinary deletion and silent saved-name reattachment.
Record visitor identity, callback order, `NewInvItem` and final membership.

### `trapType` (bit flags)

10. **Bit 2 (`trapType=2`), the "sticky" trap.** When the trap enters the
    world, its box is shrunk once to `innerPercent` of its size (every
    one of the six `BB` numbers multiplied, so it shrinks towards the
    trap's own `loc`, not towards the box's middle). When Connor enters,
    the box grows back to full size (all six divided by `innerPercent`);
    when he leaves, it shrinks again. So he has to get into the **inner**
    box to start the trap, and out of the **full** box to stop it. This
    stops music zones from flickering on and off at the edge. All 14
    such traps in the game are music zones (`chrchtrp` 0.9, `mugltrap`,
    `mwiztrap`, `mwzhtrap` 0.8 in Daventry).
11. **Bit 1 (`trapType=1`): one-shot.** After the `On` call the trap
    deletes itself. Not used by any trap in the data.
12. No `trapType` (0, the usual): the box is used as written.

### Other script calls

13. **`Init`**: when the trap enters the world, the script runs with
    `$2` = `Init` (no other arguments).
14. **`Cue`**: `sendEvent … <trap> KQCueEvent n1 n2` runs the script with
    `$2` = `Cue`, `$3` = n1, `$4` = n2 (and a fifth word when the event
    carries one), as for doors (`doors.md` rule 23). `clifTrap.cs` uses
    this: it sends itself `Cue 10` to switch running back on.
15. Traps also pass on `TalkComplete` (when a conversation they started
    ends) and `StateChange` (a monster's state change they asked to hear
    about) to their script. Those belong to M4/M5. `StateChange`'s
    arguments (`$4` old state, `$5` new state) and the state numbers:
    `characters.md` rules 14–17.
15a. **`Lookup`** (traced 2026-10-28, batch 2): when a trap enters the
    world it runs `Init`, then posts **itself** a "lookup" notice through
    the event queue, so it arrives after the current load or script step
    (the exact moment: not sure). When it arrives, a trap with
    `lookupCall=1` and a script runs `<trap> Lookup`. The same notice is
    posted after a saved game is restored (from earlier study, not
    re-checked). Data: Dead City `trptrp12`–`15` (the Sylph's plates) use
    it to `makeActive` their objects and to sort out the Sylph's position
    once every object of the world exists.
16. Scripts commonly turn a trap off for good with
    `KQObject::collision <trap> -4` (removes bit 4 from its mask, so
    Connor no longer counts; `clifTrap.cs` does it on its first `On`).
    `KQObject::collision <trap> n` adds the bits of n when n is above 0
    (with or without a `+`), removes them when n is below 0, and refuses
    0 with a message; it works on the trap's live mask, the same for any
    bits (traced 2026-10-13, Q57 d).
    **An object already inside stays inside** (traced): removing a bit
    sends nothing at that moment and doesn't empty the list. The stay /
    leave test (rules 7–8) doesn't look at the mask, so when that object
    later leaves, the trap still sends its `Off` (or `Cue 302`). Only new
    entries need the bit. (`clifTrap.cs` has no `Off` branch, so for it
    this changes nothing.)

### Spirit traps (`KQSpiriTrap`, Q188/Q194 follow-up 2026-11-06)

The following replaces the shorter 2026-10-28 description. Static
`Mask.exe` study, checked against the original Dead City data; original
runtime and manual play **not tested**. These traps use the ordinary
trigger-box occupancy machinery with their own entry/exit/Cue handlers.
All four original `spirtp1.kq`–`spirtp4.kq` have mask 4 and `damage=.2`.

**Startup and saved values**

- Default `damage` is **0.2** even if absent. It is a fraction stored as
  a single-precision number, not a whole percentage.
- Default **`spiritState=2`**, **`trapStatus=0`**. This is a material
  correction: an untouched trap does not start as a cleared state 0.
- `spiritState`, `trapStatus` and `damage` are ordinary persisted fields.
  Loading supplied values replaces the defaults; no new drain is issued
  merely by reading them. Entry recognizes spirit states **1 and 2**;
  other values do not select either entry drain. Cue 1 replaces the
  state with 1 regardless of its previous value.

**Occupancy and Cue behavior**

- On entry the visitor first joins the ordinary occupancy list. Only
  when the resulting list contains **one visitor**, and that visitor is
  **Connor** (the actual Connor object identity), does this handler set
  `trapStatus=1`. It then drains according to `spiritState`: state 1
  uses **one** damage fraction; state 2 uses **two** damage fractions.
  State 0 causes no entry drain.
- **Cue 1** sets `spiritState=1`. If `trapStatus` is **nonzero**, it
  applies one damage fraction and the later hit, every time it arrives.
  It does not check whether the state was already 1 and has no cooldown.
  Standing motionless therefore does not prevent repeated authored
  Cue 1 drains.
- **Cue 0** sets `spiritState=0`, with no health change. **Cue 2 does
  nothing**, including no assignment of state 2 and no clear.
- On a normal visitor removal, `trapStatus` clears only when the entire
  occupancy list becomes **empty**. It does not specifically test
  whether the departing visitor is Connor. World shutdown suppresses
  that normal clearing step.
- Consequently the flag is not a continuously recomputed "Connor is
  inside" predicate: if another permitted visitor entered first,
  Connor entering second does not set it; if Connor entered first and
  leaves while another visitor remains, it stays set and Cue 1 can
  still drain Connor elsewhere. The shipped mask 4 admits only Connor,
  so the multi-visitor cases require a changed mask or supplied state.
- `spiritst.kq` and `spirits2.kq` send Cue 1/2, with **no Cue 0** in
  those movement lists. Preserve the resulting retained state 1 and
  repeated drains, rather than interpreting their Cue 2 as a clear.
  `ALTAR_A1.CS` deletes all four traps.

**The two health stages, and their order**

1. **Calculate a direct-stage change, independent of armour.** This
   calculation happens in the entry/Cue handler, using Connor's health
   and level **as they are then**. Take maximum health as **10 times
   Connor's level**. The drain is the damage fraction
   times this maximum, **cut toward zero to a whole point**. For a
   state-2 entry it is twice the fraction before this conversion. Work
   out the remaining health after that whole-point drain. If it is
   below the nominal 1% threshold, replace it with the whole-point
   1% floor, also cut toward zero. **Queue that health difference**
   through the ordinary stats-change request event. This does not
   change health synchronously in the trap handler (Q194).
2. **Then queue an ordinary attack event.** It is a **Normal** (kind 0),
   **strength 1** hit from the trap with **forced-reaction flag 1**.
   It is a separate event posted after the stats-change request. The
   floor is not applied again after this hit. That flag is the same
   fourth integer accepted by the scripted `KQAttackEvent` text form;
   it is not an internal-only extra value (`connor-collision.md`
   rule 16, corrected Q194).

The comparison threshold uses approximately **0.00999999977648** of
maximum health. Its floor is computed as approximately
**0.0999999977648 times level**, cut toward zero. In particular the
floor is **0 at maximum health 10 or 30**. It is not a minimum of one
point and not a preserved fractional health balance. The slightly low
stored constants can also matter at higher integer boundaries; for
example at level 10 the floor expression is just below 1 and cuts to 0.

**Queued delivery and multiple drains (Q194)**

- Both stages are posted for the current game time and return to the
  trap handler without applying either stage. Equal-deadline events
  already in the queue run first; newly posted equal-deadline events
  retain posting order. For one drain, its direct request therefore
  arrives before its hit. It is not a synchronous health subtraction
  followed by one queued hit.
- The direct event carries the **already calculated difference**, not
  a request to rerun the percentage/floor calculation. Its receiver
  adds that stored difference to the health present **at delivery**.
  It uses ordinary stats-change handling, including a request for the
  dying state if the resulting health is below zero. It has no hit's
  rope/dying immunity guard and does not reapply the spirit floor.
- Consequently two entry/Cue handlers that run before either newly
  posted direct request can both calculate against the same health.
  Their deltas still apply separately later. Intervening healing,
  damage, a state change or another trap is not retroactively included
  in the stored difference. The intended floor may therefore be crossed
  by a later direct request itself. If the first pair has already
  entered dying, a later queued direct delta still applies; the paired
  hit is then ignored by the dying hit guard.
- **Derived same-time example:** maximum 10, starting health 1,
  protection below 100, ordinary idle Connor with no pending reaction
  or state lock, `damage=.2`. Put two Cue 1 events in the queue at the
  same deadline before their descendants. Both handlers see health 1
  and each posts a direct delta **−1**, followed by its hit. Their
  deliveries are direct A, hit A, direct B, hit B: direct A gives 0;
  hit A gives −1 and enters dying, whose entry sets health to 0;
  direct B then gives **−1 while already dying**; hit B is ignored.
  The stats receiver has no lower clamp, and the dying state refuses
  re-entry, so these four deliveries end at health **−1**, still
  dying. This is a derived static edge check, **not tested** in the
  original runtime. Processing the second Cue only after the first
  pair instead produces a different stored delta (zero at health 0).
  Do not replace this ordering with two atomic drain-and-hit calls.

For the later hit, ordinary armour/protection, current-state hurt
dispatch and Connor's dying/rope hit guards all apply. With protection
below 100 the strength-1 hit loses one point; protection exactly 100
prevents that point of loss. Forced reaction remains a separate
receiving-state decision; the flag does not turn the event into a
special spirit hit. See the state rules below: it is too strong to say
that every drain immediately makes Connor flinch.

The queued direct stats-change stage does **not** use those attack-immunity
guards. In the rope state the direct drain still applies but its later
hit is ignored; a dying Connor also ignores that later hit. A normal
living Connor can be reduced to **0** by the first stage and then to
**−1 by the hit**, which requests his ordinary dying state; when that
state is entered, health is set back to 0. The receiving controller's
normal state-change eligibility still applies. Thus the nominal floor
does not guarantee survival.

**The force flag and Connor's receiving state (Q194)**

- The nonzero flag is passed with the hit's strength to the state
  Connor has **when the hit arrives**. His ordinary controlled standing
  and moving states (`KQStateConnerWait`, `KQStateConnerMove`),
  **Choice**, and **Connor's Talk** state use the same reaction selector.
  Do not make Choice or Talk generically immune to spirit hits.
- That selector first applies normal hit damage/death handling. A
  lethal result skips reaction selection. Otherwise, with flag zero,
  normal reaction selection requires the rolled hit to be **strictly
  above `normalReactPercent` times the incoming strength**, the current
  movement to be interruptible, and the reaction deadline to be
  **strictly earlier than now** (normally the two-second gap after
  a reaction ends). Connor's default `normalReactPercent` is 0.5.
- With the spirit's **flag 1**, reaction selection is reached even if
  any of those three ordinary selection gates fails. This includes a
  currently uninterruptible movement or an unexpired reaction pause.
  Protection 100 blocking health loss does not block this flag branch.
  A configured `specialReactAnim` still takes priority only when its
  own strict `specialReactPercent` comparison succeeds; otherwise the
  requested movement is `react`.
- **Selection does not force playback.** The chosen movement goes
  through the ordinary animation-request/transition rules. The
  current uninterruptible movement can finish before the request
  starts; a later request can replace a waiting movement. This is
  neither an immediate forced animation cut nor a change of Connor's
  owning state to Special. The ordinary reaction request applies its
  normal animation input lock **at request time**, and a repeated
  request for the already retained reaction name does not create
  another concurrent reaction.
  Choice/Talk remain the owning controller until their own normal
  state rules change that; do not advance a Choice or complete a Talk
  merely on receipt of the hit. Their later animation-end behavior
  remains the existing state-specific behavior. **Q195 follow-up:** a
  later eligible Connor lip gesture requests its movement with this
  animation lock off, even if it must wait behind the still-playing
  reaction. It replaces a merely waiting reaction, but does not force
  an uninterruptible playing reaction to end, erase the state's last
  requested movement name or fabricate a reaction-end deadline. See
  `conversation.md` rule 18a for the exact waiting, lock and Talk
  continuation rules. Natural reaction completion alone does not
  unlock input: standing Wait's subsequent `Stop` request clears the
  lock and, when it starts, retains `Stop`. Completed-approach standing
  Talk makes no such request; without a later request it can hold the
  ended reaction with its lock still on, or play the waiting lip
  gesture with the lock already cleared by that lip request and the
  retained state name still `React`. Wait's extra `Stop` can cut that
  newly started interruptible gesture; Talk does not add it. Move
  instead requests its currently selected movement with the
  lock off when that selection is nonempty and differs from actual
  playback, updating the retained name if it starts. Thus unchanged
  forward/walk intent resumes walking and clears the lock at natural
  reaction end; this must not be folded into Wait's Stop action or
  Talk's absence of an extra request. A standing Special is eligible
  for lip gestures too: the lip handler has no owning-state gate.
- **Special** uses the base damage/death/sound route instead of this
  reaction selector. The same flag reaches that route, but does not
  add a `react` request, erase the scripted movement list or grant
  immunity to health loss. Its ordinary state-change eligibility still
  determines whether a requested death/change is immediate or pending.
  The plain base/no-state route likewise has no Wait forced-flinch
  selector. These findings do not establish identical handling for
  every other specialised Connor state.
- The whole hit is ignored if Connor is already in the dying state or
  the rope state. Those guards occur before the receiving-state
  selector; flag 1 does not bypass them. This leaves the previously
  queued direct stats stage unaffected.

**Numbers and observable checks**

These are derived integer checks from the traced rules, not original
runtime measurements. Log the direct-stage health separately from the
later attack's result, including protection and controller state.

| Maximum / starting health | Trigger with damage .2 | Direct-stage health | After hit, living ordinary state and protection <100 |
| --- | --- | --- | --- |
| 10 / 10 | Cue 1, or state-1 entry | 8 | 7 |
| 30 / 30 | Cue 1, or state-1 entry | 24 | 23 |
| 10 / 10 | untouched state-2 entry | 6 | 5 |
| 30 / 30 | untouched state-2 entry | 18 | 17 |
| 10 / 1 | Cue 1 | 0 | lethal hit; dying health 0 |
| 30 / 1 | Cue 1 | 0 | lethal hit; dying health 0 |

- From full health, repeated Cue 1 without leaving, **letting each
  queued pair deliver before the next Cue**, gives **10 → 7 → 4 → 1 →
  dying**, or **30 → 23 → 16 → 9 → 2 → dying**, with no intervening
  healing/protection changes. Each Cue 1 must be counted, even when
  the saved state was already 1. Simultaneously pending Cue events
  need the separate ordering check above.
- With protection 100 the first table row ends at **8**, not 7:
  the direct drain still applies and the later hit's health loss is
  blocked. The hit's reaction handling is a separate check.
- Compare flag 0/1 using the same scripted four-word attack, in native
  standing/moving, Choice, Connor Talk and Special. Log state identity,
  selected/waiting movement and actual playback separately. With an
  uninterruptible movement and a recent reaction, flag 1 must select a
  request in the Wait-derived states; Special must still take damage
  without gaining that request. Check rope/dying hits separately from
  their queued direct stats changes.
- Save state 0/1/2 with `trapStatus` 0/1, reload without an entry or Cue
  and check no new drain; then send Cue 1 and check the retained flag's
  effect. Log each real entry/exit and the full occupancy count so the
  multi-visitor cases above are distinguishable.

**Quirks to preserve / remaining limits**

The ignored Cue 2, startup state 2, integer zero floor and separate
possibly lethal hit are traced original behavior. The 1:1 target
requires preserving them. No change to authored values or protective
one-point clamp is recommended. The full Dead City spirit/altar route
and native original-game save/reload were not run during this study.

## Numbers and units

| What | Value | Source |
| --- | --- | --- |
| trap's own collision type | 0x10 | traced |
| `colliderMask` in trap data | 4 in 285 traps (all of Daventry); 0, 1, 2, 5, 6, 7, 516, 517, 519 elsewhere (rule 4) | data |
| cue numbers for a second object entering / leaving | 301 / 302 | traced |
| `innerPercent` values in the data | 0.8, 0.85, 0.9 | data |
| `trapType` values in the data | only 2 (14 traps) | data |

## Data used

- Trap `.kq` files: `classType=KQTrap`, `loc`, `BBMin/BBMax`, `script`,
  `colliderMask`, `trapType`, `innerPercent`.
- Their scripts (`*.cs`), and the shared ones in `GAME/Kq/`
  (`walk_la.cs`, `run_la.cs`, ...).
- `GAME/patch/BOUNDS.KQ` for the box of the object inside (Connor's).

## Edge cases

- Template traps placed several times (`walktp1`, `runtp1` ...): each
  copy is its own trap with its own "inside" list.
- Two traps overlapping (e.g. a walk zone inside a music zone) are fully
  independent; both get their `On`.
- A trap whose box is only touched at its very edge: `On` is sent
  (touching counts), and `Off` comes on the next move that takes him out
  of touch, because his centre is already outside.
- Saved games store each trap's "inside" names and whether its box is
  currently shrunk (M6).

## Do not reproduce

- Nothing found. Keep the asymmetric enter/leave test (touch to enter,
  centre-outside-and-not-touching to leave); it is what makes a trap feel
  forgiving at its edge.

## How to check

- **In our logs:** for every trap event: trap name, `On`/`Off`/`Cue`,
  Connor's position, the box used (shrunk or full). Walk Connor into
  `chrchtrp` from outside: `On` once his shape reaches the inner (90%)
  box; walk back out: `Off` only after his centre leaves the full box.
- **In the original, by ear:** near the wizard's house (`mwzhtrap`, a
  sticky trap, about (143974, 206673)), walk slowly in until the wizard
  music starts, then step back a little: the music should keep playing
  until you are clearly further out (the full box).
- **Checked by the user (2026-10-13):** entering the areas around
  Connor's house and near the wizard starts their music, and leaving
  switches back to the world music. (Connor's house area is a plain
  walk zone, `walk_la.cs`, which also pushes `minside.aud`; not
  sticky.) The church music plays inside the church; the user can't
  walk in and out there. `chrchtrp`'s box (x 201,730–217,023, y
  180,518–186,681) covers the church and some ground around it; the
  stickiness itself (step back a little, music stays) is not checked
  yet.

## Open questions

1. ~~Whether removing a trap's mask bit (rule 16) sends `Off` for whoever
   is inside.~~ Answered 2026-10-13: no; they stay inside and get the
   normal `Off` when they leave (rule 16).
2. ~~The exact contact test used for "touching" (rule 6).~~ Answered
   2026-10-13: the swept shape test, along the move (rule 6).
3. ~~Whether an object placed inside a trap by `move` or `loadKQ`, without
   moving there, sets it off (rule 5).~~ Q102's existing-collider `move`
   answer remains: no destination entry then, but the old trap receives
   the deferred stay check. **Corrected 2026-11-07 (Q203):** initial
   active collider registration can enter immediately, including a
   stationary recoverable Rock; see rule 5(c) and the Q203 study above.
