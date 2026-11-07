# Tossed items and trigger boxes

Tossed loot now notifies matching traps as it moves. Previously the
item thrower handled physical collisions but its caller never checked
trap entry or exit. This implements `traps.md` rules 4-9 and
`thrown-items.md` rules 7/11; it does not change the throw's gravity,
speed, collision response, textures or original game files.

The mover reports the actual up, across, down and slide segments,
ending each segment at its physical contact. Trap checks use those
segments, rather than the diagonal from the update's first position
to its last. The item's ball stack and type 0x220 meet the trap's
own mask. A trap does not block movement. The existing script
dispatcher runs each item's callbacks before the next item moves.

Items with no collision shape and items merely placed by a script
do not enter. Settled items retain their existing membership; saved
trap state restores that membership without a fresh On callback.
The tests use synthetic item bounds and trigger boxes, not embedded
game data or edited campaign saves.

## Automated checks

```powershell
cargo test --locked -p kq8-core item_traps -- --nocapture
```

Seven focused checks pass: actual path versus diagonal; wall clipping and sliding;
shape/mask/placement rules; script callback order; multi-occupant
On/Cue/Off; and settled membership through serialization and restore.
The first compilation exposed a private import in the test fixture;
it was corrected to the public data-module import before these ran.
Logs: kq8-artifacts/tossed-item-traps-tests.log (compile failure),
and tossed-item-traps-tests-fixed.log (six passed).

The additional wall-slide test initially failed its final-position
assertion: it expected 295 in x but the existing documented 25-unit
pull-back on a diagonal approach gives 277.32233. The trigger checks
already passed. Correcting that test expectation, without changing
physics, passes the single test. Logs: tossed-item-slide-test.log
and tossed-item-slide-test-fixed.log. The six earlier successes
were not rerun.

The broader core check excludes these already-passing cases:

```powershell
cargo test --locked -p kq8-core -- --skip item_traps
cargo check --locked --release -p kq8-viewer
```

The broader check passes once: 671 passed, 17 ignored, six previously
passing cases excluded. The viewer release check passes. Logs:
tossed-item-core-regression.log and tossed-item-viewer-check.log.

## Remaining limits

The settled-item check restores the core's serialized trap state;
a native campaign replay of tossed loot on a pressure plate is
**not tested**. The original pickup/deletion release and the Rock
projectile-to-inventory handoff still need study (Q203). The original
Rock aiming and arc are Q202. Rock projectiles and dropped inventory
items are separate from tossed-loot flights; the failed earned
western-plate test has not been rerun or claimed fixed.
