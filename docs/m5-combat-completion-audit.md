# M5 combat completion audit

2026-11-17. This is an inventory of **automated Rust evidence**, not a
claim of original-game parity. M5's gate in `DESIGN.md` requires melee and
ranged hits and misses, ammunition, health changes, potion expiry, rewards,
drops and game over, with every distinct weapon and enemy rule linked to a
passing scenario. Manual gameplay comparison belongs after the Rust
recreation is feature-complete.

The [ranged rule coverage table](m5-ranged-rule-coverage.md) links
rules 13–31 to current assertions and names the remaining direct gaps.
One combined Daventry crossbow run now checks a Cottage building pick
ahead of a goblin, harmless building contact, an open-sky far-end pick
and the 32.7-degree shot limit without another world load.
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
separate sources. The same process now checks a flying Griffin's Ice
projectile contact, zero shoot-offset fallback, and no-drop/zero-XP
Normal death with its original corpse.
The extended Ice World process now runs the installed Grysword's
`gry_ltgo` movement. Its `grifnymp.cs` EndLoop callback raises three
combat radii to 8,000; normal AI then lands a named 100-point Normal
attack. Controlled death verifies no drop, `gry_dead.kqs` corpse and
zero experience. The preceding rescue trigger remains outside this
combat fixture. The placed Icelord's script describes a cutscene, not
an ordinary attack; it should not be force-killed to claim coverage.
Static original-engine study of the Ice Henchman's chest sequence finds
that terminal repeating `stop` leaves him in Special. Ordinary attack
AI runs in Wait, and the installed chest script does not request Wait.
The lack of combat in the unattended chest-cue diagnostic is
consistent with that rule; original-runtime play has not been checked.
The Barren `pyrod_live_fire` route now verifies a placed Pyrod's Lava
weapon, Fire contact and health loss while Connor is alive, followed by
its no-drop/no-corpse Normal death and 8,000 experience. Earlier same-lane
lava deaths and the passing run's lack of world hits remain separate
terrain evidence, not a resolved terrain-fidelity claim.
The one-process `barren_ant2_vulture_live` route adds a placed `ant2`
actor's normal AI melee health loss, Crystal drop, no corpse and 5,000
experience, plus a nearby vulture's controlled no-drop/no-corpse death
and 500 experience. The vulture did not make a live attack; its zero
alarm and attack distances need a separate rule audit.
The `barren_henchman_live` route starts at the installed chest reveal
cue, observes the hidden Henchman's Special list finish and an ordinary
125-point Normal hit, then checks his three-item drop, corpse and
50,005 experience. It does not play the preceding or final speech,
so the script's death-listener callback remains unverified.
One Dead City process now links the installed `skelrdbw` red bow and
`weeperup` raised Weeper families. The bow's `Prmonarw.kq` Projectile
contacts lowered Connor's health; the Weeper's `Prweeper.kq` linked and
completed 12 health-drain pulses. Both controlled Normal deaths checked
no drops, one 500-experience award and their distinct corpses. These
actors were moved from off-map script placements, leaving the original
summon sequence open.
The grouped Dead City ordinary-variant check now asserts a placed
`skeleton2` zero-damage hit with no health loss, a `skeleton3` Round Key
fighter and a hidden `zombie3` actor's Birth and Normal health loss.
Its `rat2` actor has zero alarm/attack distances and made no attack
in the observed interval. The same process now checks `skelhamr`'s
delayed 50-damage attack, `zombie2`'s 30-damage attack despite its
missing movement box, and hidden `skelredo` rising from Birth and
landing 20-damage hits. Seven controlled deaths check separate drop
lists, corpses and one experience award each. All seven actors were
relocated to the measured lane; earned regional triggers remain open.
The same Dead City process now checks an installed hammer skeleton's
second Attack while Connor holds Back: the Do cue hits at 1,086 units,
outside its normal 1,000-unit reach and inside the logged 1,250-unit
extended reach. The seven death and reward checks still pass.
One Gnome process now links the placed `sprigsurprise` and `zombiepit1`
families. Both land authored Normal hits on living Connor before their
controlled deaths. The test checks the Spriggan's 65 Silver Coins,
`spdeath.kqs` corpse and 5,007 experience, and the Zombie's no-drop,
`zdead.kqs` corpse and 2,502 experience. Both were relocated, and the
Zombie's home moved with it to retain its ordinary chase.
The installed Temple 4 altar route now links both `lucend` forms to
actual damage in one process: a `PrLuc.kq` bolt from `LucEndFar`
contacts Connor and removes 115 health, then `LucEnd`'s Normal swing
removes 64. It still switches far, near and far.
The separate `temple4_lucfight_live` route now starts the first
`lucfight` actor through its altar cue. It checks three ordinary Normal
health losses, a no-drop/no-corpse death with 75,000 experience, and
`lucfight.cs`'s wall removal and delayed altar callback. The preceding
puzzle and later two placements are outside this M5 combat route.
The existing Daventry `monster_friendly_fire` process now continues
after three stationary bolt intercepts. Connor walks as the bow
Spriggan releases its fourth bolt; the test compares his logged position
and speed with the projectile's aim, lead time and 975-unit height raise.
This directly links moving-target lead to an installed ranged enemy.
The Ice World `icewater_plain_click` process opens the installed
`ildorlok` door, whose script loads the dormant `icewater` object.
After the authored cutscene releases controls and Connor readies the
Small Crossbow, the aim pick invokes `icewater.cs` with no bolt. This
links the object-name branch of ranged rule 15.
The combined Daventry `daventry_enemy_variants_live` process links
the second placed bow Spriggan type to live `Prscbolt.kq` Projectile
health loss, no-drop death, corpse and 12 experience. The placed
`sprigan1` lands Normal hits, drops its six Silver Coins and completes
its corpse and 7-experience award. The placed `zograv2` rises from Birth,
lands Normal hits, drops two Mushrooms and completes its corpse and
2-experience award. All three were relocated into the measured lane.
The first order let the bow's death script move Connor away before the
other deaths completed; putting the Zombie and melee Spriggan first
completed all three in one process. Earned routes remain untested.
The grouped `daventry_ambient_families_live` check adds the placed bat,
both chicken types and pig as zero-alarm, zero-attack-distance actors.
None attacked during the observed 45 updates near Connor. Controlled
Normal deaths checked their no-drop, zero-experience outcomes and the
bat's absent corpse versus the chicken and pig corpse shapes. The same
order checks the first-kill tutorial exclusion for bat and `chicken1`:
only `chicken2` caused its first qualifying death event. This is a
controlled lane, not an earned world route.
The grouped `swamp_plant_small_families_live` check links a placed
`crvplant` to its three-second Birth delay, ordinary Normal health loss,
and completed no-drop, no-corpse death with 2,000 experience. A nearby
frog with alarm radius 1.5, a zero-alarm weed and a zero-alarm bat made
no ordinary attack in the observed interval. The frog and bat finished
no-drop, no-corpse deaths with 500 experience each. The weed entered Die
and dropped nothing but did not finalize by update 400; original
`weeds.anm` marks `die` repeating. Original-runtime parity and earned
Swamp approaches remain untested.

| Area | Evidence already recorded | Remaining M5 work |
| --- | --- | --- |
| Connor's 13 hand banks | `Connor.kq` lists 13; `hand_weapon_matrix` checks 12 weapon hits and both plain `Attack`/`AttackB` variants in one encounter, and `first_person_weapon_matrix` covers Fists and all hand banks. Core `connor_s_do_hits_his_target_misses_or_hits_dead_air` checks a hit, a missed swing and dead air; `daventry_hero_ring` records an ordinary Henchman miss. Q139(e) now confirms the ordinary object roll; a grouped core check covers its callbacks. | Finish source-to-scenario links for weapon-specific edge cases and attack reachability. Q219 bounds the checked non-character receiver classes. |
| Connor's 9 ranged banks | `Connor.kq` lists 9; `ordinary_ranged_items` covers five in one encounter, `live_crossbow_combat` covers Small Crossbow, `elemental_ranged_death` covers Fire and Ice, and `hammer_hit_return` covers Azriel's Hammer. `ranged-weapons.md` rule 20 says ammunition is unlimited, and `live_crossbow_combat` fires three shots without an ammo count. The combined `goblin_live_combat` route verifies four ordinary crossbow hits followed by the Wizard's rule-15 plain conversation click with no fifth shot. The crossbow checkpoint covers a new turn cancelling a waiting shot and an early standing click being refused. The core bolt route now checks two standard level-1 projectiles crossing without contact. The bow Spriggan's moving-target lead is checked in `monster_friendly_fire`. | No separate ammunition depletion rule exists; finish direct source-to-scenario links for aim priority and other ranged edge cases before claiming all rules covered. |
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
