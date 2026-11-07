# Shared saved-world core contract

Implementation B, 2026-11-07. Source: [saved-world-events.md](behaviour/saved-world-events.md),
including the bounded alias and queue-service follow-up. No original
executable, Ghidra output or `re/` material was read by this implementation.
Original native save/reload and regional-return parity are **not tested**.

## Implemented core mechanisms

`events.rs` captures the documented 29 persistent event families in
deadline/FIFO order. Confirmed internal music, state-change, speech-completion
and trap-lookup aliases retain their actual persistent type. Transient engine
events and mouth-position events are excluded. The envelope retains the
existing Rust token payloads; it is not an implementation of unstudied native
resource-pointer or rare subtype codecs. Selection does not establish delivery
support for every rare subtype.

New snapshots store the world clock and absolute event deadlines after a
single-precision, six-decimal text boundary. The older relative-deadline JSON
shape remains readable with an explicit missing-clock diagnostic. Merging
preserves live initialization work and every duplicate. Equal-time live work
precedes inserted saved work. Restored target names bind once to a registered
recreation object lifetime; an absent receiver cannot bind to a later same-name
object. Actual removal cancels queued/held work and invalidates that lifetime.
Asset-catalog membership alone does not register a receiver.

`saved_world.rs` provides these CPU-only loading phases:

1. `WorldState::prepare_saved_world(&SavedEvents)` restores the clock and
   shifts already-live deadlines, then prepares saved listener names before
   object initialization.
2. `register_event_receiver(name)` marks actual logical registration.
   `finish_saved_object_init(name)` posts a saved-listener lookup after that
   object's ordinary Init. A successful listener enable retains the pending
   saved name; a successful disable clears both. Lookup consumes the saved
   name once, whether or not it resolves. `Active::finish_loaded_trap_init(index)`
   exposes the existing per-trap lookup posting for the ordered loader without
   requeueing all earlier traps; its production loader integration remains pending.
3. `merge_saved_events(SavedEvents)` inserts saved work after restored objects
   exist, before the remaining authored world Load statements.
4. `Active::service_startup_queue()` drains through an additional f32 0.0667
   seconds. Each popped event observes max(previous clock, its deadline),
   including missing receivers. Follow-up work can cascade in the same pass;
   the clock becomes the fixed horizon only after draining. No normal actor,
   animation, potion or diagnostic update is synthesized.

The runtime clock is now independent of `updates`. The existing recreation
still advances its ordinary fixed scheduler by f32(1/15) per update. The study
establishes clock precision and queue service; it does **not** establish that
the original universally runs at exactly 15 Hz. Authored positive-delay
`sendEvent` crosses its six-decimal text boundary; exact-zero posting keeps
the current clock directly.

`restore_events` remains a clearly documented compatibility wrapper for old
late call sites. It cannot make saved properties precede Init retroactively.
The old class-grouped `Active::start`, viewer bulk loading and production disk
and kept-region entry paths still require the separately planned Q201 ordered
logical loader. **These APIs alone do not complete Q201 or full save fidelity.**
Held renderer deliveries are retained only as game-message envelopes associated
with a requested logical load; they do not authorize arbitrary missing-target
retries or saving every internal continuation.

## Automated evidence

```powershell
cargo test --locked --release -p kq8-core saved_world -- --nocapture
```

The three grouped core checks passed on their first attempt: **3/3**, 0.00
seconds after a 60-second build. They cover family selection, aliases, payload
retention, precision collisions and ordering, clock shifts, legacy records,
receiver lifetimes, listener enable/disable/missing/late/double-lookup cases,
and real Active dispatch with overdue and on-time cascades. The shared
snapshot envelope is exercised; production disk/regional entry ordering is
**not tested** by these core tests.

Retained log: `kq8-artifacts/logs/saved-world-core-matrix.log`.

Because clock precision and queue policy affect shared behavior, one broader
run reused that compiled test executable. It excluded the just-passed
registration family, `scripted_drop` name filter and saved-world groups:

```powershell
& target/release/deps/kq8_core-4036e6506ba00dad.exe --skip item_registration_tests:: --skip scripted_drop --skip saved_world --nocapture
```

Result: **705 passed, 13 failed, 21 ignored**, 0.10 seconds. Full failure
output is retained in `kq8-artifacts/logs/saved-world-core-broad.log`.
The `scripted_non_rock_drop` test name did not match that skip filter and
therefore ran and passed again within this broad check; it was not repeated
afterward.
The failed cases exposed numerical fixtures that equated update counters
with elapsed time, old replacement/dedup expectations, mock actors not
registered as live receivers, obsolete Cue500/deletion-log expectations,
and a low-level test command bypassing the item's new continuation. The
paused-animation fixture also injected fresh Open input during reload;
that input must not be discarded by a faithful queue merge.

The corrections keep the original gameplay assertions and explicitly advance
time only in numerical test fixtures. New saved/Init duplicates are retained
as required by Q208. The quest item's identity assertions remain unchanged;
its test invokes the public command continuation. No production change was
made in response to these fixture failures. The 13 failed targets then passed
**13/13**, 0.00 seconds, using the single shared core build with the Rock and
trap batches (83-second build). The 705 successful cases were retained without
a second broad run. Retained retry log:
`kq8-artifacts/logs/saved-world-core-failed-retry.log`.

Actual retry command, using the exact failing names retained from the first log
(with the renamed no-dedup lookup test):

```powershell
$failedTargets = Get-Content kq8-artifacts/scratch/saved-world-failed-targets.txt
& target/release/deps/kq8_core-4036e6506ba00dad.exe --exact @failedTargets --nocapture
```

The shared retry executable SHA-256 was
`5E3B3B60C8FC0EA270F76FFAC0E4621185E0227B256DC2E516F329A4A46723BD`.
No native viewer or original-game process was launched for this core batch.

## Ordinary trap lookup posting follow-up

The subsequent bounded study amended `scripted-controller-commands.md`
rule 7 and `saved-world-events.md` rule 2: every successfully added ordinary
trap posts a due-now lookup after its inline Init opportunity, including
scriptless traps and traps whose `lookupCall` is false. Those settings gate
the delivered callback only. This new evidence required a narrow production
correction to `WorldState::queue_trap_lookup`; gone/deleted traps still do not
post. The delivery gate and occupancy handling were unchanged. Lookup itself
does not attach saved occupants or consume pending occupant names.

The existing trap lookup group now checks the unflagged/scriptless posting
matrix as well as callback gating, removal before delivery, runtime load
ordering, and retained saved/new lookup duplicates. It passed **2/2**, 0.00
seconds, on its first attempt using the same core executable built for the
Rock batch; the prior 705 successes and 13 corrected cases were not rerun.

```powershell
& target/release/deps/kq8_core-4036e6506ba00dad.exe active::trap_lookup_tests --nocapture
```

Retained log: `kq8-artifacts/logs/saved-world-trap-lookup-posting.log`.
The shared executable SHA-256 was
`9CED12B17F3736D6B20D3218F13C9D5E88F668B0B9FB5FF623C3F9AC7D8E1783`.
No additional native run or original-game execution was performed. Ordered
production loading and original-save equivalence remain outside this check.
