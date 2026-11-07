# Saved-world clocks, queued events and animation listeners

Study session A, 2026-11-07, answering the shared restoration rules in
Q208. Evidence is static tracing of the read-only GOG 1.03 executable,
its persistent event types and text codecs, plus the installed `mask.cs`.
The established generated-world sequence in `world-change.md` and the
object-loading sequence in `scripted-controller-commands.md` are reused.
No Rust was read or written. Original execution and native save
roundtrips are **not tested**. Numeric evidence and study tools remain
in ignored local study files.

## Purpose

Preserve a region's simulation clock and pending game messages when a
saved game or kept region is restored. Restore an animated object's
single end-of-movement listener without confusing that binding with
pending messages or with new work performed by initialization.

## Rules

### What is saved

1. **Disk saves and regional snapshots use the same event selection,
   traced.** Both call the same world snapshot writer, which writes
   the restored objects and `WorldState.kq`. The event selection does
   not receive a different disk-save versus region-departure policy.
   `WorldState.kq` contains the pending queue entries belonging to the
   persistent game-event family, rather than every engine event.
   Entries already removed for delivery, including the currently
   executing event, are no longer pending entries to serialize.
2. **Selection is by event family, traced.** The persistent game-event
   base and its following concrete event types pass the save filter
   when they are present in the queue. These are the persistent event
   type names; this list specifies selection, not a claim that every
   type is queued by a shipped scene or that every subtype's payload
   has been independently audited.

   | Family | Event types selected |
   | --- | --- |
   | Generic messages and lookup | `KQSimEvent`, `KQCueEvent`, `KQLookupEvent` |
   | Animation and movement | `KQAnimCueEvent`, `KQAnimPlayEvent`, `KQAnimTransitionEvent`, `KQMoveEvent`, `KQMoveNotifyEvent` |
   | Character state and action | `KQStateChangeEvent`, `KQStateChangeNotifyEvent`, `KQConnerDoEvent` |
   | Stats and contact | `KQStatsChangeRequestEvent`, `KQAttackEvent`, `KQCollisionEvent`, `KQSetGravEvent` |
   | Inventory | `KQInventoryNotifyEvent`, `KQInventoryChangeEvent`, `KQInventoryItemReceiveEvent` |
   | Conversation | `KQStartConversationEvent`, `KQTalkPlaySequenceEvent`, `KQTalkMessageSetupEvent`, `KQTalkMessageCompleteEvent` |
   | Music | `KQMusicPlayEvent`, `KQMusicPushEvent`, `KQMusicPopEvent` |
   | Other object commands | `KQDoorEvent`, `KQPuzzleEvent`, `KQRevealEvent`, `KQTogglePersistAcceptEvent` |

   Ordinary engine time ticks, timer events, action/input events,
   object/group add/remove/delete notifications, generic engine
   messages, and low-level collision/wakeup events are outside that
   family and are not copied into `WorldState.kq`. In particular,
   `KQTalkMouthPositionEvent` is excluded, despite having a textual
   conversion available elsewhere. The game's `KQCollisionEvent`
   above must not be confused with the excluded low-level collision
   events. Do not use “can be converted to text” as the queue-save
   eligibility test.
3. **Targets are not filtered by the object's save eligibility,
   traced.** The event-selection loop does not require its target to
   be in the persist manager, to be a creature, or to be other than
   Connor. It also does not exclude an eligible event merely because
   its deadline is due now. An event's ordinary saved representation
   records its type, target's object name, absolute simulation-time
   deadline and subtype payload. A valid unnamed game target receives
   the usual generated object name before that name is written.
   Events directed at permanent objects can therefore be retained even
   though those objects are rebuilt from original data rather than
   copied into the region snapshot.
4. **The queue belongs to the saved region, traced.** The regional
   snapshot contains its `WorldState.kq`. Connor, the map and game
   hand-over state are additionally written through their separate
   existing paths; that is not a transfer of this whole event queue
   to the destination. A pending eligible event addressed to Connor
   is not specially removed from the source snapshot. When that
   region is restored, its saved target name resolves against the
   objects then loaded, including the arriving Connor. This must not
   be implemented as a reference to an obsolete Connor identity.
   State stored inside Connor, including his own persistent controller
   properties, remains a separate matter from this world queue.

### Order, targets and merging

5. **Entries are written in current queue order, traced.** The queue
   is sorted by deadline, earliest first. Equal-deadline events retain
   insertion order. The writer walks that order, omits the ineligible
   events, and numbers the survivors `event0`, `event1`, and so on,
   with `nEvents` giving their count. It does not regroup them by
   target, type, script, or original send order across different
   deadlines.
6. **Restoration merges; it does not replace or deduplicate, traced.**
   The reader reconstructs entries in their numbered order and inserts
   each with its saved absolute deadline into the queue already in
   existence. It does not clear Init work or compare payloads for
   duplicates. An insertion follows entries already present at the
   same deadline. Thus equal-deadline Init work already queued precedes
   equal-deadline saved work; saved entries preserve their order among
   themselves. Earlier deadlines still take precedence over later
   insertion. This generalizes `secondary-tile-puzzles.md` rule 52;
   it is not a tile-specific exception.
7. **A saved target name is resolved during event reconstruction,
   traced.** The name is looked up when the saved event is read, after
   the generated restore script's ordinary object loads. The event
   thereafter retains that resolved object identity, not a name to
   keep retrying. If the target is absent, the original still inserts
   the reconstructed event without a valid receiver. When due, ordinary
   queue service discards it without delivering a callback. Creating
   an object with the missing name later does not attach that already
   reconstructed event to it. This differs from a target that exists
   at reconstruction and is removed later: normal object removal
   removes queued messages addressed to that identity.

### Clock and loading sequence

8. **World time and event deadlines are single-precision seconds,
   traced.** Their saved text uses six digits after the decimal point;
   loading converts the decimal value back to single precision. Six
   decimal places do not make the underlying clock microsecond-precise.
   Very close deadlines can become equal in saved text; the serialized
   order is then the order retained when they are reinserted. Neither
   clock nor saved deadline is a wall-clock timestamp or a new delay
   starting when loading finishes.
9. **Restoring the clock also shifts already-live queued work,
   traced.** `setCurrentTime` calculates the difference between the new
   and preceding simulation time and adds that difference to every
   deadline already queued. An entry due exactly at the preceding time
   is placed exactly at the new time. This preserves the remaining
   delays of work that exists before the generated script sets the
   clock, such as initialization or internal scheduler work. Saved
   `WorldState.kq` events are loaded later with absolute deadlines and
   must not receive this shift again. This command does not empty the
   queue.
10. **Restore operations retain their existing inline order,
    traced/data.** The relevant shared sequence is:

    1. Installed `mask.cs RestoreWorld` prepares the resource search,
       loads `gameState.kq`, calls world `Init`, then calls world `Load`.
    2. World `Load` invokes `newWorld.cs` inline. For a saved/kept world,
       its generated version restores current time, reads `WorldVar.cs`,
       loads terrain, Connor, camera and map, rebuilds permanent objects,
       then loads the remaining saved objects in their recorded order.
    3. Each object has its saved fields before its normal registration
       and inline `Init`; eligible synchronous contacts retain the
       rules in `scripted-controller-commands.md`. Posted work remains
       queued.
    4. The generated script loads `WorldState.kq` after those objects,
       merges its events, and finishes its progress update. The remaining
       authored world `Load` statements then execute before `Load`
       returns. They can inspect or change the loaded world and post
       additional work.
    5. `RestoreWorld` completes its archive and presentation cleanup.
       The engine then services the simulation queue through **0.0667
       additional seconds**, represented as a single-precision value,
       before unpausing. The new-game path has the same explicit service
       interval after its startup script.
    6. Unpausing resets the wall-time sampling baseline. World `Begin`
       remains the first-drawn-frame callback documented in
       `world-change.md`; it is not a replacement for any earlier
       `Load` or object `Init` operation.

    Consequently, due saved messages, Init messages and deferred
    lookups can be delivered before ordinary gameplay resumes. The
    shared service drains due work in time order, including zero-delay
    work posted while delivering another due event. It is not a
    one-event or one-callback-only flush. Load progress drawing does
    not itself advance the paused simulation through elapsed wall time.
11. **Time away does not expire a region's pending work, traced.** A
    region saved at simulation time 103 with a message due at 107 still
    has a four-second simulation interval before that deadline when
    its saved clock is restored. The explicit 0.0667-second startup
    service consumes that much of the interval before normal gameplay,
    subject to floating-point precision; time spent loading or playing
    another region does not consume it. This refines the point at which
    “resumed time” begins in the existing frozen-world rule.

### End-of-movement listeners

12. **The listener is a single saved binding, traced.** An animated
    object's `notifyObject` property stores the object name of its
    current `getEndLoop` listener. It is not a listener list, and it
    is separate from queued `EndLoop` or Cue 300 messages. The fresh
    loaded object initially has no resolved listener; a nonempty saved
    name remains pending while its ordinary `Init` runs.
13. **The saved name is bound by deferred lookup after Init, traced.**
    After the animated object's ordinary initialization returns, it
    posts a due-now lookup if a saved listener name is still pending.
    When serviced, the lookup resolves that name to an ordinary game
    object or trap and replaces the live listener. It consumes the
    saved name whether resolution succeeds or fails. A missing or
    unsuitable saved listener therefore becomes no listener, with no
    continuing retry. The normal shared post-load service occurs after
    all inline object loads and the rest of world `Load`, so a listener
    loaded later in that sequence can be found.
14. **Init registrations do not always win, traced.** A successful
    `getEndLoop <animated> <listener> 1` sets the live listener but
    leaves the pending saved name intact. Deferred saved-name lookup
    can therefore overwrite that registration, including replacing it
    with no listener if the saved name cannot be resolved. A successful
    `getEndLoop ... 0` clears both the live listener and the pending
    saved name, preventing that saved binding from reappearing. The
    command resolves both named objects before either operation, so
    naming a missing listener does not perform a successful clear.
    With no saved name, an Init registration survives normally.
15. **Lookup messages obey the ordinary queue rules, traced.** Saved
    and newly generated lookup events are not deduplicated. Once the
    saved listener name has been consumed or explicitly cleared, a
    later lookup has no saved name to reapply. This once-only binding
    behavior comes from consuming the pending name, not from removing
    apparently duplicate queue entries. Later script registrations
    still change the listener normally.

## Data used and evidence limits

- Installed `mask.cs`: `NewWorld`, `RestoreWorld` and `UnloadWorld`
  script order.
- Generated saved-world `newWorld.cs`, `WorldVar.cs` and
  `WorldState.kq`: established file roles in `world-change.md`, with
  their shared writer and reader rechecked statically here.
- Saved event properties `nEvents` and `eventN`, and animated-object
  property `notifyObject`; original assets and native save data remain
  outside the repository.
- Static checks covered queue selection, common event envelope,
  insertion/delivery, target removal, clock assignment and post-load
  service, plus animation-listener save/load and `getEndLoop` effects.
  They do not establish every rare event payload's independent codec
  correctness or every character-specific controller continuation.

## Edge cases and do not reproduce

- Do not replace a whole queue, deduplicate equal script messages, or
  discard all non-tile events as a shortcut for restoration.
- Do not infer that a zero `nEvents` in an observed save means the game
  intentionally discards future game messages.
- The original writer assumes a selected game event has a valid game
  target whose name it can serialize. It does not gracefully filter an
  invalid/null/non-game target at that point. Malformed event records,
  unknown event types, excessively long text and resaving an unresolved
  target before delivery are not safe-format guarantees. Validate and
  report or discard unsupported records safely in the recreation.
- Do not retain an unresolved target name as an indefinite subscription
  for future objects; that changes ordinary saved-event behavior.
- A listener-binding collection or an unconditional “Init overrides
  saved listeners” policy does not match these rules.
- The event type filter does not prove that every pointed-to resource
  inside every subtype payload survives; audit such payloads separately
  rather than copying process-local identities into a save.

## How to check automatically

These are proposed representative checks, **not tested in this study**.
They require no manual play or original-game execution.

1. Save a mixed queue with an ordinary cue, animation play, music event,
   lookup, transient time tick and mouth-position update. Check the
   family filter, original relative order and subtype payload retention.
   Repeat through disk-save and regional-snapshot paths.
2. Use different deadlines with reversed send order and several equal
   deadlines. Restore into a queue already holding Init work. Check
   deadline ordering, equal-time FIFO and delivery of every duplicate.
   Include two close deadlines that serialize to the same six-decimal
   value.
3. Resolve one saved target that exists, one permanent target rebuilt
   fresh, and one absent target. Create the absent name afterward and
   verify it does not receive the old event. Remove an already-resolved
   target and verify its queued delivery is canceled.
4. Restore a clock with already-live due-now and delayed initialization
   work, then merge a saved future event. Check the clock-delta shift
   only for the already-live work, absolute saved deadlines, the
   0.0667-second pre-unpause service and exclusion of loading/away wall
   time. Include zero-delay work posted by a due handler.
5. Use a saved listener A with Init registering B; expect A after lookup.
   Repeat with Init successfully clearing, with A absent, with A loaded
   later in the same load sequence, and with no saved listener. Include
   two queued lookups to verify consuming the saved name once without
   deduplicating the queue.
6. Log world `Init`, generated startup, each object's `Init`, WorldState
   merge, post-startup world `Load` statements, deferred deliveries,
   unpause and world `Begin`. Check their relative order and the world
   time at each point. A tile-only roundtrip cannot establish this
   shared family.

## Open questions

- Original native save/reload and regional-return execution remain
  **not tested**. This note establishes static rules, not runtime parity.
- Exact payload codecs for every selected rare event, including nested
  object/resource references, need their own bounded studies when an
  implementation depends on those payloads. Selection alone is not a
  proof of complete codec coverage.
- Character-specific controllers, animation phases and transient
  subsystem reconstruction remain governed by their own behavior notes.
  Excluding an engine timer from WorldState does not by itself say when
  each subsystem recreates its timer or how every first update behaves.
- The shared sequence assumes the installed ordinary loading scripts;
  arbitrary altered scripts that explicitly advance simulation during
  loading are outside this note's sequence guarantee.
