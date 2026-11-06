# Pickup tutorials: first receipt, delay and startup flags

Study session A, 2026-11-06, Q196. Sources: static study of the owned
GOG `Mask.exe`, `GAME/Kq/Connor.kq`, `GAME/Kq/Invinfo.itl`, the installed
Dead City and Temple1 `newWorld.cs` and `conStart.cs`, and the developer
world-start scripts. These are behavioral descriptions in our own words.
Original runtime, rendered presentation and native save continuation:
**not tested**. The help panel and its shared delivery behavior are in
[in-game-help.md](in-game-help.md).

## Purpose

The first receipt of certain inventory categories requests a tutorial.
Receipt consumes its one-shot flag immediately; the later help request
waits for a simulation-time deadline and the ordinary help delivery gates.

## Rules

1. **Receipt is the trigger.** A successful ordinary add to Connor's
   inventory sends him its receipt notification during that add. This
   includes direct pickup, chest transfer and the script inventory command
   used by `give`. The source path does not distinguish a player pickup
   from a script grant. An increase of an existing stack is also a receipt;
   there is no requirement that this type was absent before the add.
   Inventory loading has a different path, described in rule 6.

2. **The three pickup notices.** With the relevant bit clear in Connor's
   `startHelp` value, receipt has the following effects. The deadline is
   measured from the current simulation time of Connor's owning manager
   when he handles the receipt notification.

   | Receipt | Message case | Initial delay | Consumed bit |
   | --- | --- | --- | --- |
   | Any `HealthItem` category, including a potion | 131 | 3 seconds | 1 |
   | Any `QuestItem` category | 132 | 3 seconds | 2 |
   | `INVITEM_RopeAndHook` specifically, item ID 4050 | 128 | 4.5 seconds | 4 |

   Connor sets the bit before queuing the delayed help notice. Another
   qualifying receipt while the first notice is pending does not create a
   second one. Case 132 is **not immediately eligible** at receipt.
   `MaskPiece` is a separate category and does not qualify as `QuestItem`
   for this test. RopeAndHook's other receipt effects do not replace its
   separate 4.5-second tutorial deadline.

3. **Creation, delivery and display are separate.** Receipt does not test
   `notStartingHelp`, scripted `handsOff`, speech, Wait/Move ownership or
   the previous help-delivery time before setting these bits and queuing
   their notices. At their deadlines the notices use the same delivery
   path as `doPopup`, without adding another initial three-second delay:

   - Connor must be in Wait or Move, scripted `handsOff` must be clear,
     his speech queue must be empty, and at least three simulation seconds
     must have elapsed since his previous help delivery attempt reached
     the UI. Equality at three seconds is sufficient.
   - Failure of those conditions holds the case and schedules a retry
     one simulation second later. The shared held-tip path can postpone
     a later held tip another three seconds to let an earlier one through.
   - An occupied popup is preserved and the arriving ordinary request is
     retried three seconds later. When an ordinary case below 170 reaches
     the panel, nonzero `notStartingHelp` suppresses it there.

   Reaching the UI records the delivery time even when the preference
   suppresses this ordinary tip. Setting the receipt bit is not evidence
   that a panel opened or was acknowledged. Pausing the simulation does
   not spend these deadlines merely because wall-clock time passes.

4. **One-shot ownership.** `startHelp` belongs to Connor. Its constructor
   default is zero, and it is read and written with Connor's persistent
   data under the visible field name `startHelp`. Receipt does not clear
   a consumed bit when an item is dropped, used, replaced or later received
   again. Delivery, suppression and dismissing the popup do not re-arm
   these three receipt flags. Changing `notStartingHelp` is separate from
   changing them; re-enabling tutorials does not make an already-consumed
   pickup flag fresh.

5. **What `stopHelp` means.** This command assigns **127** to Connor's
   `startHelp`, consuming all seven low bits. It does not change the
   `notStartingHelp` preference and does not cancel already-queued notices
   or clear the held-help list. The later delivery path does not recheck
   the source's one-shot flag. The command therefore prevents later
   qualifying ordinary pickups from creating these notices, including
   pickups after the seed script has finished. It does not generally
   suppress script `doPopup` requests or the F1 catalog.

   The bit-to-notice relationships below are established. Only the three
   pickup sources in rule 2 have their complete source eligibility settled
   by this study. The other rows identify the notice associated with each
   consumed bit; they do not define a complete new trigger rule.

   | Bit | Notice relationship | Scope of evidence |
   | --- | --- | --- |
   | 1 | case 131 | Complete pickup rule above |
   | 2 | case 132 | Complete pickup rule above |
   | 4 | case 128 | Complete pickup rule above |
   | 8 | case 140 | Hit-path flag and case established; full trigger gates outside this study |
   | 16 | case 130 | Hit-path flag and case established; full trigger gates outside this study |
   | 32 | indirectly cases 137 and 149 | Non-Connor death path posts a later Connor notice; its non-demo handler requests these cases. Complete death/source eligibility outside this study |
   | 64 | case 135 | Movement-state end path flag and case established; exact action and full eligibility outside this study |

   Bit 32 is not a direct pickup tutorial with one case. No additional
   meaning for these four non-pickup source gates is assumed here.

6. **Authored inventory is not a new receipt.** Loading an inventory's
   `numItems` / `ItemName` entries restores the list and each item's owner
   without sending the ordinary successful-add notification in rule 1.
   In particular, the authored starting `INVITEM_Piece1A` in `Connor.kq`
   does not itself consume bit 2 or schedule case 132. A carried item found
   after loading a world is not evidence of a fresh pickup. Do not
   reconstruct these one-shot flags by scanning the inventory at each
   world load.

7. **Developer seed items use ordinary receipt after consuming the
   flags.** The studied Dead City and Temple1 `conStart.cs` call
   `stopHelp` before their `KQMonster::inventory Connor ...` grants and
   finish with `Testing` false. Their grants take the ordinary add path,
   with its normal inventory effects, but bits 1, 2 and 4 are already set.
   They do not create these three first-receipt notices. Their
   `newWorld.cs` invokes `conStart.cs` only when `Testing` is true.
   The installed developer world-start scripts select that testing route;
   it is not an automatic part of an ordinary first visit through a portal
   ([world-change.md, rule 21](world-change.md#rules)).

   `stopHelp` is not a temporary seed-only suppression window. Later
   ordinary receipts use the retained flags. Likewise, an ordinary portal
   journey is not a reason to clear `startHelp` or to execute the
   developer seed script. Connor's persistent `startHelp` value is the
   authority; no per-region one-shot reset was found in the receipt path.

## Numbers and units

- Pickup deadlines: current simulation time plus **3** seconds for cases
  131 and 132; plus **4.5** seconds for case 128.
- These event deadlines are stored as single-precision game-time values.
  They are not animation phases or wall-clock timers.
- Shared delivery spacing: **3** seconds; busy-Connor retry: **1** second;
  occupied-panel retry: **3** seconds.
- Default `startHelp`: **0**. `stopHelp`: **127**, the combined bits
  **1, 2, 4, 8, 16, 32, 64**.

## Data used

- `GAME/Kq/Invinfo.itl`: `category`, `IDName` and item ID;
  `INVITEM_RopeAndHook` is ID 4050 and category `Rope`.
- `GAME/Kq/Connor.kq`: authored initial inventory, including Piece1A.
- Dead City and Temple1 `conStart.cs` and `newWorld.cs`: testing-only
  grants, `stopHelp` before them, and later `Testing` false.
- `GAME/Kq/deadcity.cs`, `temple1.cs`, `daventry.cs`: developer start
  selection; Daventry's testing assignment is commented out.
- `GAME/ENGLISH/500.MSG`: tutorial case catalog, as described in
  `in-game-help.md`; this study does not change its text or presentation.

## Edge cases

- Receiving an item during speech or scripted `handsOff` still consumes
  its source flag and creates the delayed notice; those conditions can
  postpone delivery.
- Receiving a health item with a disabled tutorial preference still
  consumes bit 1. Turning the preference back on later does not generate
  another first-health-item notice on the next receipt.
- Two different first-receipt categories may queue at the same deadline.
  Their later separation follows the shared held-tip/popup behavior;
  neither deadline is an unconditional display time.
- Calling `stopHelp` after a notice was already queued is not a queue
  cancellation operation.
- Pending notice save/load and cross-world timing are distinct from
  preserving the consumed bits. Do not infer either from inventory alone.

## Do not reproduce

No deliberate correction to the three native pickup rules is proposed.
Keep any recreation convenience such as re-arming tutorials after seed
items separate from original behavior. The already-open duplicate-held-tip
and event-restore questions are not settled by this note.

## How to check

These are proposed automated behavioral checks, **not executed against
the original**:

- With fresh flags and a quiet Wait Connor, add Candle at simulation
  time 20: bit 2 is consumed during the add, case 132 has deadline 23,
  and no case 132 attempt occurs before 23. A second QuestItem at 21
  creates no additional case 132. A MaskPiece alone does not trigger it.
- Add Mushroom to an existing stack with bit 1 clear: the count rises and
  case 131 is due three simulation seconds later. Repeat with a potion
  as the first HealthItem. Neither requires using the item.
- With bit 4 clear, add RopeAndHook at 20: case 128 is due at 24.5,
  independently of other receipt effects. Check flags and request times
  separately from popup display times.
- Repeat while Connor is speaking or `handsOff`: the source flag and
  initial deadline are unchanged, while delivery retries after the deadline.
  Repeat with `notStartingHelp` enabled and disabled at delivery.
- Execute the studied developer `conStart.cs`: `startHelp` is 127 before
  its first grant and none of these three receipt notices is created.
  A later ordinary pickup, including after ordinary world travel, must
  still consult these consumed flags. Loading only the authored starting
  inventory must not synthesize a first-receipt tutorial.

## Open questions and evidence limits

1. **Native pending-notice save/load continuation is not settled.** Connor's
   persistence explicitly includes `startHelp`, the previous help-delivery
   time (`boxUpTime`) and held case IDs (`holdHelpSize`, `holdHelpN`). That
   establishes saved source/history data, not delivery of a future queued
   notice after restore. A held case ID does not itself establish a restored
   deadline or a re-armed retry. Settling the event queue's save filtering,
   reconstruction, restored time base and ordering with object Init would
   require Q191. This bounded study stops at that boundary; native save
   continuation is **not tested**.
2. Full eligibility and timing of the four non-pickup `startHelp` bits are
   not claimed. Their case relationships in rule 5 are sufficient to avoid
   assigning invented pickup meanings to the bits consumed by `stopHelp`.
3. Original runtime order, exact panel-opening ticks and the interaction
   with an occupied catalog are **not tested** here. Shared panel rules
   retain the evidence limits in `in-game-help.md`.
