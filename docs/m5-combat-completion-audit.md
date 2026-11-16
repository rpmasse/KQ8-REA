# M5 combat completion audit

2026-11-17. This is an inventory of **automated Rust evidence**, not a
claim of original-game parity. M5's gate in `DESIGN.md` requires melee and
ranged hits and misses, ammunition, health changes, potion expiry, rewards,
drops and game over, with every distinct weapon and enemy rule linked to a
passing scenario. Manual gameplay comparison belongs after the Rust
recreation is feature-complete.

The [ranged rule coverage table](m5-ranged-rule-coverage.md) links
rules 13–31 to current assertions and names the remaining direct gaps.
The [fighting rule coverage table](m5-fighting-rule-coverage.md) now
maps rules 1–41 to existing scenarios and direct gaps. Its first audit
found the Henchman save/retreat miss and the back-key/90-degree hit-test
limits already covered. Two existing core character scenarios now
assert later health loss during death and the ordinary ouch interval;
the combined core run passed 786 tests (26 ignored).
The [placed-enemy matrix](m5-enemy-rule-matrix.md) inventories the 72
distinct type names in 595 installed monster placements and separates
direct combat assertions from unlinked families. Combined Temple 1 and
Temple 3 scenarios now link Water Demon, victim beast, Shadow Monster,
Temple 3 hammer skeleton, blue bow skeleton and both stationary zombie
placement to observed health loss and distinct Normal death outcomes.
The combined Ice World lane links Frosty, hammer Orc and axe Griffin
Normal attacks to source-specific health loss and their distinct
death/drop/corpse outcomes, while retaining ambient bow-guard hits as
separate sources.

| Area | Evidence already recorded | Remaining M5 work |
| --- | --- | --- |
| Connor's 13 hand banks | `Connor.kq` lists 13; `hand_weapon_matrix` checks 12 weapon hits and both plain `Attack`/`AttackB` variants in one encounter, and `first_person_weapon_matrix` covers Fists and all hand banks. Core `connor_s_do_hits_his_target_misses_or_hits_dead_air` checks a hit, a missed swing and dead air; `daventry_hero_ring` records an ordinary Henchman miss. Q139(e) now confirms the ordinary object roll; a grouped core check covers its callbacks. | Finish source-to-scenario links for weapon-specific edge cases and attack reachability. Q219 bounds the checked non-character receiver classes. |
| Connor's 9 ranged banks | `Connor.kq` lists 9; `ordinary_ranged_items` covers five in one encounter, `live_crossbow_combat` covers Small Crossbow, `elemental_ranged_death` covers Fire and Ice, and `hammer_hit_return` covers Azriel's Hammer. `ranged-weapons.md` rule 20 says ammunition is unlimited, and `live_crossbow_combat` fires three shots without an ammo count. The combined `goblin_live_combat` route verifies four ordinary crossbow hits followed by the Wizard's rule-15 plain conversation click with no fifth shot. The crossbow checkpoint covers a new turn cancelling a waiting shot and an early standing click being refused. The core bolt route now checks two standard level-1 projectiles crossing without contact. | No separate ammunition depletion rule exists; finish direct source-to-scenario links for aim priority and other ranged edge cases before claiming all rules covered. |
| Held Rock | `rock_throw` now checks one Normal hit on an original skeleton, lower health and recovery at contact in a single launch; `rock_flight_checkpoint` keeps the Daventry wall/floor and fresh-process arc check. The grouped core matrix checks authored damage and hit kind at launch. | No distinct Rock-rule gap identified; original-runtime parity belongs to M7. |
| Ordinary enemy AI | `goblin_live_combat`, `live_crossbow_combat`, `swamp_slime_retreat`, `elemental_enemy_attacks`, `temple_spear_live`, `built_in_alarm_cry`, `surprise_line`, `monster_friendly_fire` and `spawned_monster_combat` cover distinct attack, retreat, speech and contact paths. The one-process Temple 2 route asserts two `PRspear.kq` contacts, one `PRorcycle.kq` Ice contact and one guard's Normal melee hit, all lowering Connor's health. `available_monster_add` covers a placed type, an unplaced type whose live Hard attack damages Connor, and a type file absent from the current world. The grouped Dead City rat core check covers both ordered difficulty cuts and the apply switch off. | Q218 resolves the runtime-add scaling rule; the placed-enemy matrix identifies many other distinct enemy rules still lacking source-to-scenario links. |
| Damage and survival | `live_combat_difficulty`, `shield_live_combat`, `strength_live_melee`, `invisible_live_ai` and `terrain_hazards` cover difficulty, protection, potion expiry and liquid damage. Q206 now supplies on-fire dry-ground and ontop branches; the Barren native fixture confirms the collision-free extra 15-unit move and Fire world hit. Q139(e)'s ordinary-object roll, zero-damage and repeated-death boundaries have a focused Rust check; `object_stats` covers a tomb door across a disk save. Q219 finds 18 checked classes sharing that receiver and one distinct receiver, `KQSwampTile`; the grouped secondary-tile core checks exercise its zero-damage gas and scripted suppression. | Ontop physics recovery and original-runtime behavior remain open. Contact masks, regional scripts and other specialized character receivers were outside Q219's bounded scan. |
| Death, drops and game over | `death_checkpoint` checks the installed skeleton's saved Normal `Die` state, gravity off and non-solid collision, followed by fresh-process completion. `temple_spear_live` now asserts three active Temple 2 fighters' Normal death entries, their distinct crystal/invisibility drops, no unlisted held weapon and one experience award each. `sprgnbow_death_drops`, `corpse_skin_checkpoint`, `live_enemy_game_over` and boss checks cover other rewards, named drops, corpse presentation and Connor's game-over transition. `live_crossbow_combat` asserts that an active original skeleton's Projectile death removes active status, turns gravity off and clears collision. The grouped `elemental_ranged_death` fixture checks gravity and collision clearing for Fire and Ice deaths. `tower_live_fall` keeps the original Tower Skeleton at its installed placement: its AI follows Connor along the supported side corridor; one ordinary Small Axe hit starts the complete fall, chain-mail drop, one experience award and control restoration. | The tower route uses test-controlled Connor positions; uninterrupted player traversal and later chain-mail pickup remain open campaign checks. Other enemy placement drop lists and specialized deaths still need source-to-scenario review. |
| Special enemies and saves | Controlled Mandragor, Gnome Dragon, Ice Dragon, Skeleton King, Lucreto, Bossprig, Snowmane, Barren ant/dwarf and Weeper checks exercise their distinct controllers, several across fresh-process saves. The live Mandragor root fight saves a linked down root and verifies timed rebirth in a fresh process. | Q209 confirms one immediate tree lookup and no saved live link. Its follow-up shows that configured collider/drawing fields and the named death state can be restored without proving live death effects are reapplied. Disk-load setup, tree-first order and death-phase visibility/collision remain open; Q210 and Q212 retain other visual/save gaps. Full karma scripts and earned campaign approaches are M6 work. |

These test names identify the existing scenarios. The 2026-11-17
object-damage check passed in core, and `object_stats` passed on the
installed GOG data after its harness was set to windowed. The later
runtime-add difficulty check passed in one grouped core fixture and
the existing two-world Windows fixture at Hard. Earlier pass results
and limits are in `STATUS.md`.

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
