# Saved trap contacts and elevator recovery commands

Implementation B, 2026-11-07. The rules come from Study A's
[scripted-controller-commands.md](behaviour/scripted-controller-commands.md),
rules 1 and 5–10, and [traps.md](behaviour/traps.md), Q203.
No original executable or disassembly was read by the implementation session.

## Implemented behavior

`SavedTraps::restore_before_registration` applies one trap's saved fields
before its collider registration and Init. Occupant names remain pending;
they are not copied into live membership. An eligible shape contact silently
attaches a matching saved name, ignoring capitals. An unmatched contact
cannot enter while saved names remain. A repeated contact cannot consume
another saved entry. Once all names attach, ordinary new-entry callbacks
resume. Disabled masks and absent shape contacts cannot establish occupancy.

`KQConner::inTrap` resolves the exact trap name and queries retained live
Connor membership. `KQConner::removeFromTrap` cancels only Connor's unresolved
saved name. It does not remove live membership, emit a leave callback, move
anyone or trigger a contact probe. Neither command services queued events.
Both reject missing/wrongly typed objects and wrong argument counts with
the documented zero result.

The pending list removes each name when it attaches, keeping its count
consistent without reproducing the original malformed-record bookkeeping
defect. Study A's follow-up establishes that the original itself appends
live names to a retained saved-name list at each save, accumulating duplicates
while some names remain unresolved. This recreation preserves the unique
union of unresolved and live occupants instead. Saved records reject duplicate names, including overlap between live
and pending entries, before mutation. Rust snapshots preserve unresolved
names separately as the documented safe handling of that original defect.
Older snapshots without this field preserve the names they actually contain.

## Automated verification

```powershell
cargo test --locked --release -p kq8-core saved_trap_contact_and_command_matrix -- --nocapture
```

The grouped matrix passes first attempt, **1/1 in 0.00 seconds**, using the
shared release test build. It exercises sphere and box contacts, rejected
mask/outside/new-name cases, ordered silent attachment, duplicate contact,
ordinary entry/leave after restoration, actual interpreter command ordering,
exact trap-name lookup, live/pending cancellation, queue retention, snapshot
continuation, legacy records and atomic rejection of invalid or late fields.
Retained log: `kq8-artifacts/logs/saved-trap-contact-matrix.log`.
Previously successful trap and campaign checks were not replayed.

The CLI coverage generator was rebuilt and run successfully against the
read-only original installation; its regenerated report recognizes both
commands. Command recognition alone does not establish loading parity.

```powershell
cargo run --locked --release -p kq8-cli --bin kq8 -- script-coverage --out docs/SCRIPT-COVERAGE.md --game "C:\Users\Robert\Documents\Codex\KQ8\King's Quest 8 GOG"
```

## Remaining integration

The production bulk loader still uses the explicitly labeled late-restore
compatibility paths. The new per-object API must be adopted by the ordered
logical loader alongside real collider registration, saved event preparation
and each object's Init. Merely knowing a name is not evidence of contact.
The original Barren Init and IceWorld Load-suffix elevator consumers, disk
reload and regional return remain required integration checks; they are
**not tested by this matrix**. Study A additionally establishes that every
successfully added ordinary trap queues a due-now Lookup after Init, even
without a script or lookupCall flag. Delivery only invokes the optional
flagged script and never changes pending names or live membership. The
corresponding queue-policy correction has its own affected checks in
[saved-world-core.md](saved-world-core.md). Full Q201 and M6 save
fidelity are not complete.
