# Non-character hit receivers

Study session A, 2026-11-17. Source: the original `Mask.exe` hit
receivers, checked against the class and collision behavior already
recorded in the original-data notes. This is a bounded inventory for
M5, not a survey of every script that can react to `Die`. No manual
gameplay test was run.

## Rules

1. **Ordinary objects share one hit rule.** The class receivers for
   `KQObject`, `KQDoor`, `KQChest`, `KQAnimObject`, `KQAnimOnce`,
   `KQInventoryItem`, `KQBoulder`, `KQPushTile`, `KQLavaTile`,
   `KQProjectile` and `KQHammer` all use the same ordinary object hit
   rule in `doors.md` rules 19–21. Thus a door, chest, barrel, other
   animated object or world item does not gain a different damage roll
   merely from its class. Their health and protection change by that
   shared rule; a zero-damage hit offers `ProjImmune` to an attached
   script; health below zero offers `Die` with the remembered hit kind.
   The receiver itself neither opens nor deletes the object. An
   attached script may do that, as the barrel and tomb-door scripts do.
   A script is not needed for health to change.
2. **Other checked classes using the same receiver:** `KQInterior`,
   `KQPortal`, `KQOrbitObj`, `KQAudibleObject`, `KQPaddleObject`,
   `RiverControl` and `KQWeepBolt`. This completes the 18 checked
   classes with the ordinary receiver. `KQHammer` and other projectiles
   have special behavior **when they fly into something**; as victims
   of a delivered hit they still use the ordinary receiver. A shared
   receiver does not mean every source can reach the class: attack
   click selection and range, projectile masks, collision detail and
   projectile immunity are applied before delivery. For example,
   ordinary bolts do not touch world inventory items through their
   default mask, and two level-1 projectiles do not touch each other
   (`ranged-weapons.md` rules 29–30, 32; `azriel-hammer.md` rules 7–8).
3. **Swamp gas tiles are the exception.** `KQSwampTile` does not run the
   ordinary damage rule when hit. It does not subtract health, roll
   protection, remember hit kind, send `ProjImmune` or call `Die`, even
   if its health was already negative. A hit carrying **nonzero** damage
   has no hit effect. A hit carrying **zero** damage creates one
   `swmpgas.kqs` gas effect only when `Swamp::Tile::PowerOn` and
   `Swamp::Tile::SwitchOn` are both true **and** the tile has no script;
   otherwise it has no hit effect. There is no immediate damage from
   this hit handler. The shipped tile's projectile-immunity type bit
   causes a bolt or hammer to deliver zero damage when it touches the
   tile. Safe tiles have a script, which suppresses the gas effect.
   The separate hazard on Connor contact is in
   `secondary-tile-puzzles.md` rules 16–19.
4. **Trigger traps are outside this damage path.** `KQTrap` and
   `KQSpiriTrap` use trigger-box contact and Cue/On/Off behavior, not
   the ordinary object's hit/health receiver. A trap's type 0x10 is
   outside the melee click's 0x40/0x20 selection; a projectile may
   still touch a trap according to its collision mask and the trap's
   accepting mask, but that is a contact, not an ordinary object-health
   hit. See `traps.md` rules 4–8 and its spirit-trap section.

## Scope and checks

The receiver scan covered the named class receivers in the installed
executable, including the classes above, and compared their hit and
death handling. It did **not** exhaustively read every regional script,
every specialized character receiver, or every collision-mask and
placement combination in the GOG worlds. Class membership alone does
not establish that a particular attack can touch a particular object.
For a deterministic check, a delivered positive hit on a scripted
ordinary barrel or door can lower health and then call `Die`; the same
hit on a powered `KQSwampTile` cannot lower its health. A zero-damage
hit on a scripted ordinary object calls `ProjImmune`, while a powered
scriptless `KQSwampTile` creates gas without that script call.
