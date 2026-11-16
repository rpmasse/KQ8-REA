# M5 combat completion audit

2026-11-16. This is an inventory of **automated Rust evidence**, not a
claim of original-game parity. M5's gate in `DESIGN.md` requires melee and
ranged hits and misses, ammunition, health changes, potion expiry, rewards,
drops and game over, with every distinct weapon and enemy rule linked to a
passing scenario. Manual gameplay comparison belongs after the Rust
recreation is feature-complete.

| Area | Evidence already recorded | Remaining M5 work |
| --- | --- | --- |
| Connor's 13 hand banks | `Connor.kq` lists 13; `hand_weapon_matrix` checks 12 weapon hits in one encounter, and `first_person_weapon_matrix` covers Fists and all hand banks. Core `connor_s_do_hits_his_target_misses_or_hits_dead_air` checks a hit, a missed swing and dead air; `daventry_hero_ring` records an ordinary Henchman miss. | Resolve object damage against the exact original rule; Q139(e) still calls that calculation a stand-in. |
| Connor's 9 ranged banks | `Connor.kq` lists 9; `ordinary_ranged_items` covers five in one encounter, `live_crossbow_combat` covers Small Crossbow, `elemental_ranged_death` covers Fire and Ice, and `hammer_hit_return` covers Azriel's Hammer. `ranged-weapons.md` rule 20 says ammunition is unlimited, and `live_crossbow_combat` fires three shots without an ammo count. | No separate ammunition depletion rule exists; finish source-to-scenario links for other ranged edge cases before claiming all rules covered. |
| Held Rock | `rock_throw` now checks one Normal hit on an original skeleton, lower health and recovery at contact in a single launch; `rock_flight_checkpoint` keeps the Daventry wall/floor and fresh-process arc check. The grouped core matrix checks authored damage and hit kind at launch. | No distinct Rock-rule gap identified; original-runtime parity belongs to M7. |
| Ordinary enemy AI | `goblin_live_combat`, `live_crossbow_combat`, `swamp_slime_retreat`, `elemental_enemy_attacks`, `built_in_alarm_cry`, `surprise_line`, `monster_friendly_fire` and `spawned_monster_combat` cover distinct attack, retreat, speech and contact paths. `available_monster_add` covers a placed type, an unplaced but world-available type, and a type file absent from the current world. | Confirm dynamic-monster combat scaling: Q3 in `fighting.md` still leaves a possible second factor on `KQMonster::add` unresolved. |
| Damage and survival | `live_combat_difficulty`, `shield_live_combat`, `strength_live_melee`, `invisible_live_ai` and `terrain_hazards` cover difficulty, protection, potion expiry and liquid damage. | Resolve Q139(e)'s original object damage rule before treating breakable objects as faithful. |
| Death, drops and game over | `death_checkpoint`, `sprgnbow_death_drops`, `corpse_skin_checkpoint`, `live_enemy_game_over` and boss checks cover ordinary death, rewards, named drops, corpse presentation and Connor's game-over transition. | Complete the unrelocated Tower Skeleton fall; `tower_live_fall` currently places the original actor at the cliff edge. A Connor-only lure/hit reached `reactback` on supported ground. The measured east edge is x=188000–188200; identify a side approach or authored displacement before another route. |
| Special enemies and saves | Controlled Mandragor, Gnome Dragon, Ice Dragon, Skeleton King, Lucreto, Bossprig, Snowmane, Barren ant/dwarf and Weeper checks exercise their distinct controllers, several across fresh-process saves. The live Mandragor root fight now saves a linked down root and verifies timed rebirth in a fresh process. | Q209, Q210 and Q212 retain precise original-order or visual/save rules. Full karma scripts and earned campaign approaches are M6 work. |

These test names identify the existing scenarios. Only `rock_throw` was
rerun for this update; earlier pass results and limits are in `STATUS.md`.

## Rock enemy-contact setup and result

Two automated Daventry probes used the already passing Rock throw and
recovery input. The first requested `KQMonster::add rockGoblin goblin2`;
the Rust world reported that type undefined, so there was no target. The
second moved the installed `goblin22` to (277790, 198700, 13594) at update
10. It then walked south to (277790, 197610, 13531) by update 48. Connor
launched the Rock at update 59 from (277813, 198922, 15585). It met a
wall at (277783, 198618, 15574) on update 61, entered falling, and
recovered on the floor at update 65. No enemy contact or damage occurred.
Logs are in the ignored `kq8-artifacts/rock-enemy-probe` and
`kq8-artifacts/rock-enemy-probe-2` directories.

The first probe exposed a Rust loader gap, rather than an absent
original type: installed `Monstats.mtl` defines `goblin2`, and the
Daventry list places it. The 2026-11-17 loader change now prepares its
model and movements for later `KQMonster::add` use. An unattended
Daventry check created and drew `testGoblin` with live AI and combat
state (1/1 passed); it did not repeat the Rock throw.

The follow-up used the existing Dead City corridor where prior
projectiles reached `skeleton1` at (183300, 45000, 864), and its logged
1,750-unit collision height. The Rock's installed `throw` frame launches
it roughly 1,950 units above Connor's feet. An aim through the skeleton's
chest extended beyond it to the terrain and missed. Aiming through its
ground point clipped the pointer line at (183300, 45000, 864); the Rock
then descended into the skeleton's box on update 62. It delivered one
Normal hit for 19 raw damage, lowering health 60 → 47, and `rockHit.cs`
created the recoverable Rock at the contact point. The revised
`rock_throw` release test passed 1/1 in 12.39 seconds. The separate
Daventry flight checkpoint preserves the original wall/floor case, so
this new check replaces its earlier single-launch Rock recovery test
without adding another world load to the focused test suite.
