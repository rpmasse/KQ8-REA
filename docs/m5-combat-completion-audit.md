# M5 combat completion audit

## M5 completion gate

As of 2026-11-17, **M5 is complete (100%) against the grouped combat
completion check in `DESIGN.md`**. This is a phase status, not a claim
that the whole game matches the running original. Automated core and
original-data scenarios link the known distinct weapon, enemy, damage,
death, terrain and save rules. Campaign integration remains M6; final
original-runtime and presentation comparisons remain M7.

The scripted-walk controller check now uses the authored request order:
weapon swap first, then `walkTo`. The refused walk remains in the pending
request slot across serialization and starts after the restored swap,
with its target and cue intact. This is a controller serialization check,
not a fresh-process disk load or the complete `ax.cs` cutscene. The
Lucreto finale already asserts real `reactback` staggers, so that
special reaction has a source-to-scenario link without another run.
This closed the earlier checkpoint evidence gap. The later special
actor audit closed the remaining source question below.

### Finite M5 closure queue

| Step | Completion condition | Current state |
| --- | --- | --- |
| Correct the scripted-walk evidence | Exercise swap then walk, serialize the pending request and cue, and resume after the swap. | Done in the existing controller matrix. Fresh-process save/load has not been claimed. |
| Reconcile rule maps | For each partial fighting/ranged row, identify an actual missing Rust branch or an unlinked distinct shipped rule. Classify data variants and original-runtime comparisons separately. | Done; no confirmed missing shared branch remains in the known rules. |
| Reconcile special enemies | For each remaining Dragon, Mandragor and other special row, name a shipped M5 behavior that is missing, or move its campaign trigger to M6 and original-runtime comparison to M7. | Done; Q209 tree outcome was resolved by a bounded study and linked to the existing encounter. |
| Close the gate | Fix only resulting M5 defects, add a direct assertion to an existing grouped scenario, then run the affected check. | Done; the Mandragor encounter passed 1/1 after correcting its variable-damage kill schedule. No production change was needed for that studied rule. |

The gate is closed by the source-to-scenario links below, not by a
percentage of placements or tests. Untraced original behavior remains
explicitly open for M7 comparison; finding a mismatch there can reopen
the responsible implementation milestone.

The final rule-map pass found that the remaining ranged rule 24 entries
are additional installed weapon offsets using the same frame-part
mechanism already checked with several original-data weapons. Rule 26's
absent optional-key defaults are parser evidence, not a known missing
shipped shot. The fighting map's positive held-health bonus has no
installed instance; ordinary timing, damage, death and drops have
direct checks, while specialized deaths are tracked in the separate
row. None of these is a confirmed missing shared Rust branch. The
placed-enemy matrix now distinguishes special behavior from regional
scripts and repeated drop lists.

The special-actor pass resolved the Mandragor tree's death outcome:
ordinary surviving-hit reaction, no authored item drop or script,
20,000 experience, the fixed corpse and linked-root shutdown without
an all-roots-down gate. `mandragor_tree_melee_checkpoint` now asserts
these effects together with fresh-process Rust save continuation. Its
first added run exposed a variable-damage fixture failure: three hits
left 36 health, and a fourth click at update 165 was refused while
Connor reacted. Moving that fourth click past the reaction and the save
to 235 produced the required death and reload; the focused release test
passed 1/1 at the final update-185 schedule. Dragon head attack, death,
rebirth and body completion have direct core and
original-data checks. The body's conditional chain-material activation
has no established shipped trigger; its appearance is an M7 comparison,
not a reason to invent an M5 transition. Q212's unresolved native
emitter identity concerns original save parity in M7; the Rust flying
bolt continuation already has an automated M5 check. Ice Henchman's
repeating terminal `stop` leaves him in Special under the traced rule,
so the lack of an ordinary attack in the chest diagnostic is not an
unlinked combat mechanic. Regional chest/rescue approaches belong to
M6. The Pyrod fixture proves its placed Fire attack and death; an
earlier lava hit in a different diagnostic run does not establish a
Pyrod defect.

| Gate area | Current evidence | Remaining limit |
| --- | --- | --- |
| Shared melee, ranged, equipment, damage, potions and game over | Original-data encounters and the fighting/ranged rule maps exercise the known distinct branches. | M6 must carry combat through earned campaign routes; M7 compares the running original. |
| Specialized enemies and combat receivers | Dragon, Mandragor, Lucreto, Skeleton King, Snowmane, Weeper and ordinary-object checks cover their distinct attacks, deaths and Rust save boundaries. | Q209/Q210/Q212 retain native binary-save, timing and appearance questions for M7, without a known missing shipped M5 branch. |
| Death, drops and combat scripts | The mechanism inventory below links shared death, drop, corpse and reward rules to focused or grouped checks. | Data-only placement variants do not each need a combat test; earned pickup and regional scripts are M6. |

| M5 completion requirement | Direct automated evidence |
| --- | --- |
| Melee hits and misses, weapon choice and damage | `hand_weapon_matrix`, `first_person_weapon_matrix`, `strength_live_melee`, core `connor_s_do_hits_his_target_misses_or_hits_dead_air` and the [fighting rule map](m5-fighting-rule-coverage.md). |
| Ranged hits and misses, aiming, projectile contact and ammunition | `live_crossbow_combat`, `ordinary_ranged_items`, `elemental_ranged_death`, `hammer_hit_return` and the [ranged rule map](m5-ranged-rule-coverage.md); original-data crossbow fire repeats with no ammo depletion. |
| Enemy attacks, decisions and distinct special controllers | Grouped original-data regional encounters and the [placed-enemy matrix](m5-enemy-rule-matrix.md), including Dragon, Mandragor, Lucreto, Snowmane, Weeper and Tower Skeleton routes. |
| Health, protection, potion expiry and terrain damage | `live_combat_difficulty`, `shield_live_combat`, `strength_live_melee`, `invisible_live_ai`, grouped terrain-hazard core checks and the twelve-region/native liquid batch. |
| Death, drops, rewards, save continuation and game over | `death_checkpoint`, `temple_spear_live`, `mandragor_tree_melee_checkpoint`, Dragon and Lucreto checkpoints, `live_enemy_game_over` and the mechanism inventory below. |

### Shared death and drop mechanism inventory

This inventory follows [fighting rules 30–33](behaviour/fighting.md),
not the count of monster-list rows. A test name identifies the branch
it asserts; these Rust checks do not establish running-original parity.

| Distinct mechanism | Direct evidence | Limit |
| --- | --- | --- |
| Death entry clears health, collision, gravity and active status; saved dying state finishes once | Core `a_character_dies_then_gives_experience_a_corpse_and_goes`, `disk_checkpoint_resumes_an_original_monsters_death_once`; installed `death_checkpoint`, `elemental_ranged_death` | Specialized death states are separate. |
| No drop, tossed inventory and held-only equipment exclusion | Installed `daventry_ambient_families_live`, `temple_spear_live`; the latter checks exact crystal and potion counts in one process | Other lists with the same toss rule are data variants. |
| Weapon part kept through death, or thrown when the death pose lacks that part | Installed `sprgnbow_death_drops` checks both Normal and Fire routes; core `a_dying_characters_weapon_is_laid_where_the_death_leaves_it` | The shipped data has no case with two matching weapon parts. |
| Laid weapon behind an obstacle is thrown | Core `a_laid_weapon_behind_a_wall_is_thrown_instead` | No installed route with this exact obstruction is linked. |
| Normal corpse, projectile corpse fallback and distinct projectile corpse, Ice corpse, Fire with no corpse | Installed `death_checkpoint`, `live_crossbow_combat`, `daventry_enemy_variants_live` (`sprigan1` uses `projdie` and `sp_dead8.KQS`), `elemental_ranged_death`, `flame_sword_death` | Mandragor Tree and Dragon Body override ordinary finalization. |
| One experience award, first-kill help and ordinary Die callback | Core `a_character_dies_then_gives_experience_a_corpse_and_goes`; installed `combat_tutorial_delivery`, `temple_spear_live`, `barren_henchman_live` | Earned campaign pickup belongs to M6. |
| Scripted `Die now` without a movement | Core `die_now_skips_the_movement_and_fire_leaves_no_corpse` | No distinct installed encounter has been linked to this command. |

The original type table gives type-level drops only to Tower Skeleton
and Shadow Monster; their installed drops are checked by `tower_live_fall`
and `temple_one_families_live`. This read-only table check found no new
drop mechanism requiring a separate placement run. The remaining M5
death work is in specialized controllers and source-backed save or
visibility boundaries, as in the gate table above.

Earned routes belong to M6; original-runtime comparison and
presentation parity belong to M7. The detailed history below records
narrower assertions without turning each into a new gate.

2026-11-17: Q210's Dragon Head animation-start path sets the original
`chains.bmp` colour-key and full-bright material flag. The Rust viewer
now applies it to each head's own chain material. The existing IceWorld
attack process passed 1/1 and logged eight affected material entries on
each head, with no shader error. The texture remains 16x128. The body's
separate activation branch is still open because its shipped trigger
has not been established. This is one distinct presentation rule closed,
not a new full-M5 percentage claim.

2026-11-17: A bounded original-engine study corrected the Mandragor
death-phase premise. A timed root death clears live collision type and
mask, but does not change drawing groups or hidden status; collision
returns when `birth` ends. The existing Rust path already keeps the
root eligible for drawing while non-solid. Exact mesh visibility in the
running original and native binary-save restoration remain unproved and
do not justify a speculative renderer change or another placement test.

2026-11-17: A Dragon head defeated mid-attack now gets a due attack
deadline after rebirth. The existing IceWorld encounter check was
extended to cover interruption, rejoining, and a later delivered fire
cue in one run; the save-boundary core check also passed. Both focused
checks passed 1/1. M5 remains ~80% because the other distinct gaps in
the table remain open.

2026-11-17: Team alarm now uses the same active-enemy slot rule as
ordinary sight. One focused core check covers replacement of a solo
enemy and refusal when a team-only list is full (1/1 passed); all nine
Mandragor core checks and the existing installed Swamp root timer
scenario also passed. This closes the shared team-slot mismatch but
does not resolve Q209's native disk-load and drawing questions. M5
remains ~80%.

2026-11-17: The existing three-enemy Daventry run now kills its placed
`sprigan1` with a Projectile event and asserts the authored `projdie`
movement and `sp_dead8.KQS` corpse alongside its six-coin drop and
single seven-point reward. It passed 1/1. This closes a distinct
ordinary death branch without another world process. M5 remains ~80%.

2026-11-17. This is an inventory of **automated Rust evidence**, not a
claim of original-game parity. M5's gate in `DESIGN.md` requires melee and
ranged hits and misses, ammunition, health changes, potion expiry, rewards,
drops and game over, with every distinct weapon and enemy rule linked to a
passing scenario. Manual gameplay comparison belongs after the Rust
recreation is feature-complete.

The [ranged rule coverage table](m5-ranged-rule-coverage.md) links
rules 13–31 to current assertions and names the remaining direct gaps.
The existing moving-crossbow save scenario now also checks the installed
`arrow` part's bolt start during Connor's run lean, using logged position,
bank and heading. It now holds Control across a later click with that
crossbow still ready and checks that the click stays a plain interaction
without a third bolt. The combined release check passed 1/1 on 2026-11-17.
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
The existing `snowmane_live_clarity` process now keeps enemy AI and
incoming damage enabled. Snowmane's own attacks lower Connor's health
at 350 hand damage before Clarity and 200 afterward, while two real
Flame Sword hits see its protection change from 30 to 0. The high
health ceiling is test-controlled; the earned approach remains open.
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
and 500 experience. The same process now asserts the placed vulture's
zero alarm and attack distances, active wandering at update 34 and no
attack or Connor hit before its controlled death at update 130. This
links its nonattacking interval without forcing an invented attack.
The `daventry_outhouse_live` route clicks the installed outhouse door
and checks the placed hidden `sprdump` actor's `Special dump` scene.
Its EndLoop callback raises alarm, home and chase radii from zero to
8,000; normal AI then deals a named 15-point Normal hand hit that
lowers Connor's health. The same process checks its no-drop Normal
death, `spd_deth.kqs` corpse and one 23-experience award. This is a
scripted encounter from the original data, not a forced ordinary
spawn; the running original executable remains untested.
The `barren_henchman_live` route now clicks the installed Chest1 from a
test-controlled nearby Connor position. Its authored cue sequence
reveals the hidden Henchman; the run observes his Special list finish and an ordinary
125-point Normal hit, then checks his three-item drop, corpse and
50,005 experience. The same run now sends the later authored Cue 124;
the final speech completes, sets the quest flag, and registers the
death listener. The fatal hit delivers Chest1 Cue 30 once, starting
Connor's `attack3` aftermath. Travel to the chest from Barren's normal
entry point remains untested.
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
The same Dead City run now checks the placed `spirit` family after the
other seven fighters. Its installed `spirit.anm` attack has only a
sound cue, no `Do`; the actor started four close-range attacks without
damaging Connor. A controlled 100,000-point Normal hit caused zero
loss at its 100 protection. This source-backed check resolves the
earlier failed expectation that Spirit should land an ordinary hit or
die from that Normal event.
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
The expanded `swamp_witch_alarm` process checks the Witch's authored
two-line alarm and `Prfire.kq` Fire damage before her no-drop,
no-corpse, 50,000-experience death. The same process relocates the
placed Swamp Henchman, checks his 75-point Normal damage, no-drop,
no-corpse, 50,005-experience death and the `swhnchmn.cs` aftermath
move. Continuous poison at the Witch point requires a raised health
ceiling; the assertions name each enemy source. The earned Henchman
reveal is still outside this fixture.

The same `temple_spear_live` process now checks four more installed
placement drop lists after the three live fighters: the second spear
carrier, second bow guard and two script-parked spear carriers. Their
controlled Normal deaths each finalize and award experience once; the
parked actors' authored summons remain outside this fixture.

The existing two-world `elemental_enemy_attacks` process now checks the
first and later bow starts for both Barren flame bow dwarfs and the Ice
World orc bow guard. Each first shot uses the installed `arrow` part in
frame 3 of `Attack`; later shots retain that distinct offset. The
Barren spawn can overlap continuous Fire terrain, so the test gives
Connor a high current-health ceiling before observing the bow contacts.

The existing Swamp fiend-and-bubble process now releases the placed
bubble from `sgohome1.cs`'s unalarmable `goHome` state after relocation.
Its own attack then lowered Connor's health twice before the already
checked no-drop, no-corpse death. The release was test-controlled;
the earned Swamp route remains outside this fixture.

Two existing encounter processes now assert all three held-item stat
categories with installed actors: Daventry's Small Crossbow adds 5
ranged damage and Leather Armor adds 2 protection, while Dead City's
Skeleton King sword adds 30 hand damage. The live Daventry fights and
Skeleton King death checks still pass. No installed held item in the
current monster inventory has a positive health bonus.

| Area | Evidence already recorded | Remaining M5 work |
| --- | --- | --- |
| Connor's 13 hand banks | `Connor.kq` lists 13; `hand_weapon_matrix` checks 12 weapon hits and both plain `Attack`/`AttackB` variants in one encounter, and `first_person_weapon_matrix` covers Fists and all hand banks. Core `connor_s_do_hits_his_target_misses_or_hits_dead_air` checks a hit, a missed swing and dead air; `daventry_hero_ring` records an ordinary Henchman miss. Q139(e) now confirms the ordinary object roll; a grouped core check covers its callbacks. | Finish source-to-scenario links for weapon-specific edge cases and attack reachability. Q219 bounds the checked non-character receiver classes. |
| Connor's 9 ranged banks | `Connor.kq` lists 9; `ordinary_ranged_items` covers five in one encounter, `live_crossbow_combat` covers Small Crossbow, `elemental_ranged_death` covers Fire and Ice, and `hammer_hit_return` covers Azriel's Hammer. `ranged-weapons.md` rule 20 says ammunition is unlimited, and `live_crossbow_combat` fires three shots without an ammo count. The combined `goblin_live_combat` route verifies four ordinary crossbow hits followed by the Wizard's rule-15 plain conversation click with no fifth shot. The crossbow checkpoint covers a new turn cancelling a waiting shot and an early standing click being refused. The core bolt route now checks two standard level-1 projectiles crossing without contact. The bow Spriggan's moving-target lead is checked in `monster_friendly_fire`. | No separate ammunition depletion rule exists; finish direct source-to-scenario links for aim priority and other ranged edge cases before claiming all rules covered. |
| Held Rock | `rock_throw` now checks one Normal hit on an original skeleton, lower health and recovery at contact in a single launch; `rock_flight_checkpoint` keeps the Daventry wall/floor and fresh-process arc check. The grouped core matrix checks authored damage and hit kind at launch. | No distinct Rock-rule gap identified; original-runtime parity belongs to M7. |
| Ordinary enemy AI | `goblin_live_combat`, `live_crossbow_combat`, `swamp_slime_retreat`, `elemental_enemy_attacks`, `temple_spear_live`, `built_in_alarm_cry`, `surprise_line`, `monster_friendly_fire` and `spawned_monster_combat` cover distinct attack, retreat, speech and contact paths. The one-process Temple 2 route asserts two `PRspear.kq` contacts, one `PRorcycle.kq` Ice contact and one guard's Normal melee hit, all lowering Connor's health. `available_monster_add` covers a placed type, an unplaced type whose live Hard attack damages Connor, and a type file absent from the current world. The grouped Dead City rat core check covers both ordered difficulty cuts and the apply switch off. The placed Swamp Slime now keeps its retreating Move and active-list state through a disk save, stops beyond its far-enough threshold and later obeys its chase leash in a fresh process. | Q218 resolves the runtime-add scaling rule; the placed-enemy matrix identifies many other distinct enemy rules still lacking source-to-scenario links. |
| Damage and survival | `live_combat_difficulty`, `shield_live_combat`, `strength_live_melee`, `invisible_live_ai` and `terrain_hazards` cover difficulty, protection, potion expiry and liquid damage. Q206 now supplies on-fire dry-ground and ontop branches; the Barren native fixture confirms the collision-free extra 15-unit move and Fire world hit. Q139(e)'s ordinary-object roll, zero-damage and repeated-death boundaries have a focused Rust check; `object_stats` covers a tomb door across a disk save. Q219 finds 18 checked classes sharing that receiver and one distinct receiver, `KQSwampTile`; the grouped secondary-tile core checks exercise its zero-damage gas and scripted suppression. | Ontop physics recovery and original-runtime behavior remain open. Contact masks, regional scripts and other specialized character receivers were outside Q219's bounded scan. |
| Death, drops and game over | `death_checkpoint` checks the installed skeleton's saved Normal `Die` state, gravity off and non-solid collision, followed by fresh-process completion. `temple_spear_live` now asserts three active Temple 2 fighters' Normal death entries, their distinct crystal/invisibility drops, no unlisted held weapon and one experience award each. `sprgnbow_death_drops`, `corpse_skin_checkpoint`, `live_enemy_game_over` and boss checks cover other rewards, named drops, corpse presentation and Connor's game-over transition. `live_crossbow_combat` asserts that an active original skeleton's Projectile death removes active status, turns gravity off and clears collision. The grouped `elemental_ranged_death` fixture now checks Fire and Ice deaths from occupied active-list slots, including removal, gravity and collision clearing. The placed `daventry_outhouse_live` Spriggan covers the same active-list removal on Normal death. `tower_live_fall` keeps the original Tower Skeleton at its installed placement: its AI follows Connor along the supported side corridor; one ordinary Small Axe hit starts the complete fall, chain-mail drop, one experience award and control restoration. | The tower route uses test-controlled Connor positions; uninterrupted player traversal and later chain-mail pickup remain open campaign checks. Other enemy placement drop lists and specialized deaths still need source-to-scenario review. |
| Special enemies and saves | Controlled Mandragor, Gnome Dragon, Ice Dragon, Skeleton King, Lucreto, Bossprig, Snowmane, Barren ant/dwarf and Weeper checks exercise their distinct controllers, several across fresh-process saves. The live Mandragor root fight saves a linked down root and verifies timed rebirth in a fresh process. A second save inside that restored process now proves the Rust root remains in Die with zero health/collision, the tree still has one down root and no turn, and the remaining timer decreases; the core checkpoint verifies collision returns only after Birth finishes. | Q209 confirms one immediate native tree lookup and no saved live link. Q209 now establishes that root death clears collision but leaves drawing groups unchanged, matching the Rust waiting phase. Native binary-save order remains unverified; Q210's head chains are implemented, while its body activation and Q212's native emitter save details remain open for later original-game comparison. Full karma scripts and earned campaign approaches are M6 work. |

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
