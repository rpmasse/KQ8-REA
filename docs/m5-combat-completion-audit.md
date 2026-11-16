# M5 combat completion audit

2026-11-16. This is an inventory of **automated Rust evidence**, not a
claim of original-game parity. M5's gate in `DESIGN.md` requires melee and
ranged hits and misses, ammunition, health changes, potion expiry, rewards,
drops and game over, with every distinct weapon and enemy rule linked to a
passing scenario. Manual gameplay comparison belongs after the Rust
recreation is feature-complete.

| Area | Evidence already recorded | Remaining M5 work |
| --- | --- | --- |
| Connor's 13 hand banks | `Connor.kq` lists 13; `hand_weapon_matrix` checks 12 weapon hits in one encounter, and `first_person_weapon_matrix` covers Fists and all hand banks. | Reconcile attack misses and object damage against the exact original rule; Q139(e) still calls the object damage calculation a stand-in. |
| Connor's 9 ranged banks | `Connor.kq` lists 9; `ordinary_ranged_items` covers five in one encounter, `live_crossbow_combat` covers Small Crossbow, `elemental_ranged_death` covers Fire and Ice, and `hammer_hit_return` covers Azriel's Hammer. | Link ammunition depletion and missed shots to their passing scenarios in the final rule matrix. |
| Held Rock | `rock_throw` and `rock_flight_checkpoint` check the click, throw cue, arc, recovery and fresh-process continuation; the grouped core matrix checks authored damage and hit kind at launch. | Check an ordinary live Rock hit on an enemy, its damage kind and subsequent recovery. |
| Ordinary enemy AI | `goblin_live_combat`, `live_crossbow_combat`, `swamp_slime_retreat`, `elemental_enemy_attacks`, `built_in_alarm_cry`, `surprise_line`, `monster_friendly_fire` and `spawned_monster_combat` cover distinct attack, retreat, speech and contact paths. | Confirm dynamic-monster combat scaling: Q3 in `fighting.md` still leaves a possible second factor on `KQMonster::add` unresolved. |
| Damage and survival | `live_combat_difficulty`, `shield_live_combat`, `strength_live_melee`, `invisible_live_ai` and `terrain_hazards` cover difficulty, protection, potion expiry and liquid damage. | Resolve Q139(e)'s original object damage rule before treating breakable objects as faithful. |
| Death, drops and game over | `death_checkpoint`, `sprgnbow_death_drops`, `corpse_skin_checkpoint`, `live_enemy_game_over` and boss checks cover ordinary death, rewards, named drops, corpse presentation and Connor's game-over transition. | Complete the unrelocated Tower Skeleton fall; `tower_live_fall` currently places the original actor at the cliff edge. |
| Special enemies and saves | Controlled Mandragor, Gnome Dragon, Ice Dragon, Skeleton King, Lucreto, Bossprig, Snowmane, Barren ant/dwarf and Weeper checks exercise their distinct controllers, several across fresh-process saves. | Q209, Q210 and Q212 retain precise original-order or visual/save rules. Full karma scripts and earned campaign approaches are M6 work. |

These test names identify the existing scenarios; this audit did not rerun
them. Historical pass results and limitations are recorded in `STATUS.md`.

## Rock enemy-contact handoff

Two automated Daventry probes used the already passing Rock throw and
recovery input. The first requested `KQMonster::add rockGoblin goblin2`;
the world reported that type undefined, so there was no target. The
second moved the installed `goblin22` to (277790, 198700, 13594) at update
10. It then walked south to (277790, 197610, 13531) by update 48. Connor
launched the Rock at update 59 from (277813, 198922, 15585). It met a
wall at (277783, 198618, 15574) on update 61, entered falling, and
recovered on the floor at update 65. No enemy contact or damage occurred.
Logs are in the ignored `kq8-artifacts/rock-enemy-probe` and
`kq8-artifacts/rock-enemy-probe-2` directories.

The next attempt needs a **collision-backed, unobstructed setup** that
keeps a valid original actor in the Rock's path at its actual flight
height. Derive it from the loaded world geometry and actor bounds before
running another viewer check. Do not adjust coordinates or timing by
trial and error, and do not rerun the full weapon matrices for this gap.
