# M5 ranged rule coverage

2026-11-17. This maps [ranged-weapons.md](behaviour/ranged-weapons.md)
rules 13–31 to
assertions in the current Rust tree. A named test means its assertions
exercise the described portion of a rule; it is not a comparison with
a running original executable. The recent Wizard extension passed on Windows. Earlier run
results are in `STATUS.md` and the corresponding test files. A partial
row is still open for the M5 rule-to-scenario gate.

| Rule | Current automated evidence | Direct gap |
| --- | --- | --- |
| 13 click priority | Core `an_item_or_control_lock_takes_precedence_over_an_attack_click` checks a held item, blocked input and weapon mode. | None identified for the common branch. |
| 14 Ctrl | The same core check routes a ranged Ctrl click to plain interaction. | No separate original-data Ctrl route. |
| 15 talk instead of shooting | `goblin_live_combat` checks four normal crossbow releases, then Wizard type `0xC021` taking conversation with no fifth release in the same Daventry process. | `icewater` branch has no direct original-data assertion. |
| 16 aim pick | `live_crossbow_combat` checks three character camera picks and hits. The existing Fire-arrow range/emitter test now asserts that its open-space line aims at the drawn ground. | Building and sky aim priority are not directly asserted in a ranged original-data route. |
| 17 shoot offset | Core `connor_s_bolt_starts_at_the_crossbow_part_and_flies_to_the_aim_point` checks fallback and retained part offset. | No distinct missing branch identified. |
| 18 steepness | Core `the_aim_is_never_steeper_than_one_radian_from_the_vertical` checks the 33-degree limit and unchanged shallow aim. | The viewer click-to-bolt path is not directly asserted at this limit. |
| 19 Hammer branch | `hammer_hit_return` and `hammer_first_person` exercise its distinct throw and return. | No distinct missing branch identified here; Hammer-specific rules have their own notes. |
| 20 shot timing | `live_crossbow_combat` checks repeated standing releases; `connor_shot_checkpoint` checks a waiting turn, both sides of the Do cue, refusal of an early click during the saved standing attack, and cancellation by a new turn key without a bolt; `moving_shot_checkpoint` checks the 0.5-second moving delay and 1-second run return. | No distinct missing timing branch identified in these routes; original-executable parity is separate. |
| 21 projectile selection | `ordinary_ranged_items` checks five ordinary item files in one process; `live_crossbow_combat` checks Small Crossbow; `elemental_ranged_death` checks Fire and Ice. Core `connor_s_bolt_starts_at_the_crossbow_part_and_flies_to_the_aim_point` checks NoRanged. | No distinct item-file gap identified. |
| 22 shot damage | The same core shot check uses Connor's ranged damage; `live_crossbow_combat` checks three health reductions, and its incoming enemy shots lower Connor's health. | No distinct missing branch identified. |
| 23 start and rotation | The core Connor shot check asserts the turned model-part offset and bolt start. | No original-data lean-dependent start assertion. |
| 24 frame part | Core Connor and character bolt-start checks assert part fallback, retention and the current frame. `temple_spear_live` asserts that the first placed spear carrier shot uses its `spear` part in the current `Attack` frame and the second retains that offset; the placed bow guard uses its `arrow` part in its `Attack` frame. | Other original-data character weapons have not been checked at this exact post-Do frame. |
| 25 aim and lead | The core Connor shot check aims at a target; `the_shot_leads_a_moving_target` checks moving, still and too-fast targets. | No original-data encounter checks a moving target's exact lead point. |
| 26 projectile file | `ordinary_ranged_items`, `elemental_ranged_death` and the core bolt checks assert authored file, speed, range, shape or kind across ordinary and elemental shots. | Defaults for an absent optional key are checked in data parsing, not in a live original-data shot. |
| 27 direction | Core `a_bolt_flies_straight_and_hits_the_first_character` checks heading and pitched direction. | No distinct missing branch identified. |
| 28 flight and range | Core `a_bolt_ends_at_the_floor_grid_or_its_range_and_spares_its_shooter` checks straight steps and removal beyond range; `elemental_ranged_death` checks Fire range removal. | No distinct missing branch identified. |
| 29 collision stops | The core bolt checks cover the first character, shooter exclusion, floor and range. The same focused check proves that a different shooter's bolt starting inside a character contacts at zero distance, and that two standard level-1 bolts cross without contact or queued damage. `monster_friendly_fire` checks an in-flight intercept with installed data. | No distinct missing branch for the shipped projectile family's collision level; original-executable parity remains separate. |
| 30 hit effects | Core immune-character and queued-contact checks cover zero damage and event service; `live_crossbow_combat` and `elemental_ranged_death` cover real health, death and hit kind. The original-data `rock_throw` checks the shared projectile `Die` callback running `rockHit.cs` at contact, and `rock_flight_checkpoint` checks that callback once after reload. | A normal ranged bolt's building-only miss still lacks a direct original-data assertion. The installed Rock proves the shared scripted removal hook, not every possible bolt script. |
| 31 enemy bolts | `monster_friendly_fire` checks a spriggan bolt hitting a goblin before Connor; `live_crossbow_combat` checks enemy shots injuring Connor. | Exact original-data moving-target lead remains under rule 25. |

The next compact checks should fill one or more partial rows in an
existing scenario. Do not count a rule complete from a code path alone;
the assertion must observe the corresponding effect or lack of effect.
